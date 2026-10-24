use std::{
    ops::ControlFlow,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
};

use rf_property_package::LocalThermoContext;
use rf_solver::{SolveProgress, SolverControl};
use rf_types::{RfError, RfResult};
use rf_ui::FlowRunId;

use crate::{
    WorkspaceSolveProgress,
    solver_bridge::execution::{
        WorkspacePackageLoadError, WorkspaceSolveControl, WorkspaceSolveJob, WorkspaceSolveOutput,
    },
};

pub(super) type PackageLoader =
    Box<dyn FnOnce(&str) -> Result<LocalThermoContext, WorkspacePackageLoadError> + Send>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRunProgress {
    pub run_id: FlowRunId,
    pub sequence: u64,
    pub progress: WorkspaceSolveProgress,
}

#[derive(Debug)]
pub(super) enum WorkerResult {
    Completed(Box<WorkspaceSolveOutput>),
    Panicked,
}

pub(super) trait WorkerSpawner {
    fn spawn(&self, job: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>>;
}

pub(super) struct ThreadSpawner;
impl WorkerSpawner for ThreadSpawner {
    fn spawn(&self, job: Box<dyn FnOnce() + Send>) -> std::io::Result<JoinHandle<()>> {
        thread::Builder::new()
            .name("radishflow-solve".into())
            .spawn(job)
    }
}

pub(super) struct WorkerChannels {
    pub cancel: Arc<AtomicBool>,
    pub progress: Arc<Mutex<Option<WorkspaceRunProgress>>>,
    pub ready: Receiver<()>,
    pub start: SyncSender<bool>,
    pub result: Receiver<WorkerResult>,
}

pub(super) struct WorkerControl {
    id: FlowRunId,
    cancel: Arc<AtomicBool>,
    progress: Arc<Mutex<Option<WorkspaceRunProgress>>>,
    sequence: u64,
    ready: SyncSender<()>,
    start: Receiver<bool>,
}

impl WorkerChannels {
    pub fn new(id: FlowRunId) -> (Self, WorkerControl, SyncSender<WorkerResult>) {
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(Mutex::new(None));
        let (ready_tx, ready) = mpsc::sync_channel(1);
        let (start, start_rx) = mpsc::sync_channel(1);
        let (result_tx, result) = mpsc::sync_channel(1);
        (
            Self {
                cancel: cancel.clone(),
                progress: progress.clone(),
                ready,
                start,
                result,
            },
            WorkerControl {
                id,
                cancel,
                progress,
                sequence: 0,
                ready: ready_tx,
                start: start_rx,
            },
            result_tx,
        )
    }

    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::Release);
        // Wakes a worker waiting at Ready, including a grant/cancel race.
        let _ = self.start.try_send(false);
    }
}

impl WorkerControl {
    fn emit(&mut self, progress: WorkspaceSolveProgress) -> ControlFlow<()> {
        if self.cancel.load(Ordering::Acquire) {
            return ControlFlow::Break(());
        }
        self.sequence = self
            .sequence
            .checked_add(1)
            .expect("progress sequence exhausted");
        *self.progress.lock().expect("progress slot poisoned") = Some(WorkspaceRunProgress {
            run_id: self.id.clone(),
            sequence: self.sequence,
            progress,
        });
        ControlFlow::Continue(())
    }
}

impl SolverControl for WorkerControl {
    fn checkpoint(&mut self, progress: SolveProgress) -> ControlFlow<()> {
        self.emit(WorkspaceSolveProgress::Solving(progress))
    }
}

impl WorkspaceSolveControl for WorkerControl {
    fn before_load(&mut self) -> ControlFlow<()> {
        self.emit(WorkspaceSolveProgress::Preparing)
    }

    fn ready(&mut self) -> RfResult<ControlFlow<()>> {
        if self.emit(WorkspaceSolveProgress::Ready).is_break() {
            return Ok(ControlFlow::Break(()));
        }
        self.ready.send(()).map_err(|_| {
            infrastructure_error("ready_disconnected", "run owner disconnected before Ready")
        })?;
        let allowed = self.start.recv().map_err(|_| {
            infrastructure_error("start_disconnected", "run owner disconnected during Ready")
        })?;
        Ok(if allowed && !self.cancel.load(Ordering::Acquire) {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        })
    }
}

pub(super) fn spawn_worker(
    spawner: &impl WorkerSpawner,
    job: WorkspaceSolveJob,
    load: PackageLoader,
    cached: bool,
    mut control: WorkerControl,
    result: SyncSender<WorkerResult>,
) -> std::io::Result<JoinHandle<()>> {
    spawner.spawn(Box::new(move || {
        let outcome =
            match catch_unwind(AssertUnwindSafe(|| job.execute(load, cached, &mut control))) {
                Ok(output) => WorkerResult::Completed(Box::new(output)),
                Err(_) => WorkerResult::Panicked,
            };
        // Exactly one terminal payload; progress uses a separate replaceable slot.
        let _ = result.send(outcome);
    }))
}

pub(super) fn infrastructure_error(code: &str, message: &str) -> RfError {
    RfError::invalid_input(message).with_diagnostic_code(format!("workspace.run.{code}"))
}
