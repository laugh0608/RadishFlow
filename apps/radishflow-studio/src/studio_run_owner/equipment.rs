use super::{StudioRunOutcome, StudioRunTerminal};
use crate::WorkspaceRunCancelReason;
use rf_ui::AppState;
use rf_ui::engineering::{CalculationQualification, CalculationRequest, CompletedCalculation};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
};

enum Terminal {
    Completed(Box<CompletedCalculation>),
    Cancelled,
    Panicked,
}

pub(super) struct EquipmentRun {
    pub qualification: CalculationQualification,
    cancelled: Arc<AtomicBool>,
    start: SyncSender<bool>,
    result: Receiver<Terminal>,
    handle: JoinHandle<()>,
    received: Option<Terminal>,
    pub started: bool,
    pub cancel_reason: Option<WorkspaceRunCancelReason>,
}

impl EquipmentRun {
    pub fn spawn(request: CalculationRequest) -> std::io::Result<Self> {
        Self::spawn_with(request, CalculationRequest::execute)
    }
    pub(super) fn spawn_with(
        request: CalculationRequest,
        execute: impl FnOnce(CalculationRequest) -> CompletedCalculation + Send + 'static,
    ) -> std::io::Result<Self> {
        let qualification = request.qualification();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        let (start, start_rx) = mpsc::sync_channel(1);
        let (result_tx, result) = mpsc::sync_channel(1);
        let handle = thread::Builder::new()
            .name("radishflow-equipment".into())
            .spawn(move || {
                let terminal = catch_unwind(AssertUnwindSafe(|| {
                    if start_rx.recv().unwrap_or(false) && !flag.load(Ordering::Acquire) {
                        // A device method is one indivisible numeric operation. Cancellation
                        // revokes publication immediately; the thread finishes cooperatively.
                        Terminal::Completed(Box::new(execute(request)))
                    } else {
                        Terminal::Cancelled
                    }
                }))
                .unwrap_or(Terminal::Panicked);
                let _ = result_tx.send(terminal);
            })?;
        Ok(Self {
            qualification,
            cancelled,
            start,
            result,
            handle,
            received: None,
            started: false,
            cancel_reason: None,
        })
    }
    pub fn cancel(&mut self, app: Option<&mut AppState>, reason: WorkspaceRunCancelReason) {
        if let Some(app) = app {
            app.workspace
                .engineering
                .revoke_calculation(&self.qualification);
        }
        self.cancel_reason.get_or_insert(reason);
        self.cancelled.store(true, Ordering::Release);
        let _ = self.start.try_send(false);
    }
    pub fn poll(&mut self, app: Option<&mut AppState>) -> bool {
        let current = app.as_ref().is_some_and(|app| {
            app.workspace
                .engineering
                .calculation_is_current(&self.qualification)
        });
        if !current {
            let reason = if app.is_some() {
                WorkspaceRunCancelReason::QualificationChanged
            } else {
                WorkspaceRunCancelReason::Detached
            };
            self.cancel(app, reason);
        }
        if self.cancel_reason.is_none() && !self.started && self.start.try_send(true).is_ok() {
            self.started = true;
        }
        if self.received.is_none() {
            self.received = self.result.try_recv().ok();
        }
        self.handle.is_finished()
    }
    pub fn finish(mut self, app: Option<&mut AppState>) -> StudioRunTerminal {
        debug_assert!(self.handle.is_finished());
        let joined = self.handle.join();
        if self.received.is_none() {
            self.received = self.result.try_recv().ok();
        }
        let terminal = if joined.is_err() {
            Some(Terminal::Panicked)
        } else {
            self.received
        };
        let diagnostic_code = match &terminal {
            Some(Terminal::Panicked) => Some("equipment.worker_panicked".into()),
            None => Some("equipment.completion_disconnected".into()),
            _ => None,
        };
        let (outcome, applied) = match terminal {
            Some(Terminal::Completed(completed)) => {
                let outcome = if completed.succeeded() {
                    StudioRunOutcome::Succeeded
                } else {
                    StudioRunOutcome::Failed
                };
                let applied = app.is_some_and(|app| {
                    app.workspace
                        .engineering
                        .complete_calculation(*completed)
                        .is_ok()
                });
                (outcome, applied)
            }
            Some(Terminal::Cancelled) => (StudioRunOutcome::Cancelled, false),
            Some(Terminal::Panicked) | None => {
                if let Some(app) = app
                    && app
                        .workspace
                        .engineering
                        .calculation_is_current(&self.qualification)
                {
                    app.workspace
                        .engineering
                        .revoke_calculation(&self.qualification);
                    app.push_log(
                        rf_ui::AppLogLevel::Error,
                        "Equipment calculation worker failed",
                    );
                }
                (StudioRunOutcome::InfrastructureFailed, false)
            }
        };
        StudioRunTerminal {
            outcome,
            applied,
            cancellation: self.cancel_reason,
            diagnostic_code,
        }
    }
}
