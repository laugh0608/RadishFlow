use super::{
    StudioRunOutcome, StudioRunTerminal,
    numeric_worker::{NumericWorker, NumericWorkerOutcome},
};
use crate::WorkspaceRunCancelReason;
use rf_ui::AppState;
use rf_ui::engineering::{CalculationQualification, CalculationRequest, CompletedCalculation};

pub(super) struct EquipmentRun {
    pub qualification: CalculationQualification,
    worker: NumericWorker<Box<CompletedCalculation>, ()>,
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
        let worker =
            NumericWorker::spawn("radishflow-equipment", move |_| Box::new(execute(request)))?;
        Ok(Self {
            qualification,
            worker,
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
        self.worker.cancel();
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
        if self.cancel_reason.is_none() {
            self.worker.start();
            self.started = self.worker.started;
        }
        self.worker.is_finished()
    }
    pub fn finish(self, app: Option<&mut AppState>) -> StudioRunTerminal {
        let terminal = self.worker.finish();
        let diagnostic_code = match &terminal {
            NumericWorkerOutcome::Panicked => Some("equipment.worker_panicked".into()),
            NumericWorkerOutcome::Disconnected => Some("equipment.completion_disconnected".into()),
            _ => None,
        };
        let (outcome, applied) = match terminal {
            NumericWorkerOutcome::Completed(completed) => {
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
            NumericWorkerOutcome::Cancelled => (StudioRunOutcome::Cancelled, false),
            NumericWorkerOutcome::Panicked | NumericWorkerOutcome::Disconnected => {
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
