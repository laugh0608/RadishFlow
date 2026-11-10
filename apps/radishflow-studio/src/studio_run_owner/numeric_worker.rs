//! Thread lifecycle shared by independent equipment and material calculations.
use std::{
    ops::ControlFlow,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
};

pub(super) enum NumericWorkerOutcome<T> {
    Completed(T),
    Cancelled,
    Panicked,
    Disconnected,
}
pub(super) struct NumericWorkerControl<P> {
    cancelled: Arc<AtomicBool>,
    progress: SyncSender<P>,
}
impl<P> NumericWorkerControl<P> {
    pub fn checkpoint(&mut self, progress: P) -> ControlFlow<()> {
        if self.cancelled.load(Ordering::Acquire) {
            return ControlFlow::Break(());
        }
        let _ = self.progress.try_send(progress);
        ControlFlow::Continue(())
    }
}
pub(super) struct NumericWorker<T, P> {
    cancelled: Arc<AtomicBool>,
    start: SyncSender<bool>,
    result: Receiver<NumericWorkerOutcome<T>>,
    progress: Receiver<P>,
    handle: JoinHandle<()>,
    pub started: bool,
}
impl<T: Send + 'static, P: Send + 'static> NumericWorker<T, P> {
    pub fn spawn(
        name: &str,
        execute: impl FnOnce(&mut NumericWorkerControl<P>) -> T + Send + 'static,
    ) -> std::io::Result<Self> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        let (start, start_rx) = mpsc::sync_channel(1);
        let (result_tx, result) = mpsc::sync_channel(1);
        let (progress_tx, progress) = mpsc::sync_channel(1);
        let handle = thread::Builder::new().name(name.into()).spawn(move || {
            let terminal = catch_unwind(AssertUnwindSafe(|| {
                if start_rx.recv().unwrap_or(false) && !flag.load(Ordering::Acquire) {
                    NumericWorkerOutcome::Completed(execute(&mut NumericWorkerControl {
                        cancelled: flag,
                        progress: progress_tx,
                    }))
                } else {
                    NumericWorkerOutcome::Cancelled
                }
            }))
            .unwrap_or(NumericWorkerOutcome::Panicked);
            let _ = result_tx.send(terminal);
        })?;
        Ok(Self {
            cancelled,
            start,
            result,
            progress,
            handle,
            started: false,
        })
    }
    pub fn cancel(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        let _ = self.start.try_send(false);
    }
    pub fn start(&mut self) {
        if !self.started && self.start.try_send(true).is_ok() {
            self.started = true;
        }
    }
    pub fn progress(&self) -> Option<P> {
        self.progress.try_iter().last()
    }
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
    pub fn finish(self) -> NumericWorkerOutcome<T> {
        debug_assert!(self.handle.is_finished());
        if self.handle.join().is_err() {
            return NumericWorkerOutcome::Panicked;
        }
        self.result
            .try_recv()
            .unwrap_or(NumericWorkerOutcome::Disconnected)
    }
}
