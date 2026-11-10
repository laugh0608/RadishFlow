use super::{
    StudioRunId, StudioRunOwner, StudioRunTerminal,
    numeric_worker::{NumericWorker, NumericWorkerControl, NumericWorkerOutcome},
};
use crate::{
    WorkspaceRunCancelReason,
    material_balance_actions::{
        MaterialBalanceActionError, material_diagnostic_code, material_outcome, material_preflight,
    },
    run_actions::{StudioRunReply, StudioRunStartFailure},
};
use rf_solver::component_balance::{MaterialBalanceControl, MaterialBalanceProgress};
use rf_ui::{AppState, material_balance::*};
use std::{ops::ControlFlow, sync::Arc, time::SystemTime};

impl MaterialBalanceControl for NumericWorkerControl<MaterialBalanceProgress> {
    fn checkpoint(&mut self, progress: MaterialBalanceProgress) -> ControlFlow<()> {
        self.checkpoint(progress)
    }
}
pub(super) struct MaterialRun {
    pub receipt: MaterialBalanceReceipt,
    worker: NumericWorker<MaterialBalanceOutcome, MaterialBalanceProgress>,
    pub progress: Option<MaterialBalanceProgress>,
    pub cancel_reason: Option<WorkspaceRunCancelReason>,
}
impl MaterialRun {
    fn spawn(
        input: MaterialBalanceInput,
        receipt: MaterialBalanceReceipt,
    ) -> Result<Self, (Box<MaterialBalanceReceipt>, std::io::Error)> {
        Self::spawn_with(input, receipt, MaterialBalanceInput::execute)
    }
    pub(super) fn spawn_with(
        input: MaterialBalanceInput,
        receipt: MaterialBalanceReceipt,
        execute: impl FnOnce(
            MaterialBalanceInput,
            &mut NumericWorkerControl<MaterialBalanceProgress>,
        ) -> MaterialBalanceOutcome
        + Send
        + 'static,
    ) -> Result<Self, (Box<MaterialBalanceReceipt>, std::io::Error)> {
        match NumericWorker::spawn("radishflow-material-balance", move |control| {
            execute(input, control)
        }) {
            Ok(worker) => Ok(Self {
                receipt,
                worker,
                progress: None,
                cancel_reason: None,
            }),
            Err(error) => Err((Box::new(receipt), error)),
        }
    }
    pub fn started(&self) -> bool {
        self.worker.started
    }
    pub fn cancel(&mut self, app: Option<&mut AppState>, reason: WorkspaceRunCancelReason) {
        if let Some(app) = app {
            app.workspace
                .revoke_material_balance(self.receipt.qualification().run_id());
        }
        self.cancel_reason.get_or_insert(reason);
        self.worker.cancel();
    }
    pub fn poll(&mut self, app: Option<&mut AppState>) -> bool {
        if !app.as_ref().is_some_and(|app| {
            app.workspace
                .material_balance_is_current(self.receipt.qualification())
        }) {
            let reason = if app.is_some() {
                WorkspaceRunCancelReason::QualificationChanged
            } else {
                WorkspaceRunCancelReason::Detached
            };
            self.cancel(app, reason);
        }
        if self.cancel_reason.is_none() {
            self.worker.start();
        }
        if let Some(progress) = self.worker.progress() {
            self.progress = Some(progress);
        }
        self.worker.is_finished()
    }
    pub fn finish(mut self, app: Option<&mut AppState>) -> StudioRunTerminal {
        let result = match self.worker.finish() {
            NumericWorkerOutcome::Completed(result) => result,
            NumericWorkerOutcome::Cancelled => MaterialBalanceOutcome::Cancelled,
            NumericWorkerOutcome::Panicked => infrastructure(
                "material.worker_panicked",
                "Material calculation worker panicked",
            ),
            NumericWorkerOutcome::Disconnected => infrastructure(
                "material.completion_disconnected",
                "Material calculation completion disconnected",
            ),
        };
        let outcome = material_outcome(&result);
        let diagnostic_code = material_diagnostic_code(&result);
        let applied = app.is_some_and(|app| {
            app.workspace
                .complete_material_balance(&mut self.receipt, result)
                == MaterialBalanceDisposition::Applied
        });
        StudioRunTerminal {
            outcome,
            applied,
            cancellation: self.cancel_reason,
            diagnostic_code,
        }
    }
}
fn infrastructure(code: &str, message: &str) -> MaterialBalanceOutcome {
    MaterialBalanceOutcome::Failed(Arc::new(MaterialBalanceFailure::Infrastructure(
        rf_types::RfError::invalid_input(message).with_diagnostic_code(code),
    )))
}
impl StudioRunOwner {
    pub fn submit_material_balance(
        &mut self,
        app: &mut AppState,
        now: SystemTime,
    ) -> Result<StudioRunReply, MaterialBalanceActionError> {
        self.submit_material_with(app, now, MaterialRun::spawn)
    }
    pub(super) fn submit_material_with(
        &mut self,
        app: &mut AppState,
        now: SystemTime,
        spawn: impl FnOnce(
            MaterialBalanceInput,
            MaterialBalanceReceipt,
        )
            -> Result<MaterialRun, (Box<MaterialBalanceReceipt>, std::io::Error)>,
    ) -> Result<StudioRunReply, MaterialBalanceActionError> {
        if !self.accepting {
            return Ok(StudioRunReply::Closing);
        }
        material_preflight(app)?;
        if let Some(activity) = self.activity() {
            let same = self.material.as_ref().is_some_and(|run| {
                run.cancel_reason.is_none()
                    && app
                        .workspace
                        .material_balance_is_current(run.receipt.qualification())
            });
            return Ok(if same {
                StudioRunReply::AlreadyRunning(activity.id)
            } else {
                StudioRunReply::Busy(activity.id)
            });
        }
        let (input, receipt) = app
            .workspace
            .prepare_material_balance()
            .expect("preflight validated plan");
        let id = StudioRunId::Material(receipt.qualification().run_id().clone());
        let target = format!(
            "{} · {}",
            app.workspace.document.metadata.title,
            input.plan().specification.mixer_id
        );
        match spawn(input, receipt) {
            Ok(run) => {
                self.material = Some(run);
                self.set_activity(id.clone(), target, now);
                Ok(StudioRunReply::Accepted(id))
            }
            Err((mut receipt, error)) => {
                let error = rf_types::RfError::invalid_input(format!(
                    "Could not start material calculation worker: {error}"
                ))
                .with_diagnostic_code("material.worker_start_failed");
                app.workspace.complete_material_balance(
                    &mut receipt,
                    MaterialBalanceOutcome::Failed(Arc::new(
                        MaterialBalanceFailure::Infrastructure(error.clone()),
                    )),
                );
                Ok(StudioRunReply::StartFailed {
                    id: Some(id),
                    error: StudioRunStartFailure::Infrastructure(error),
                })
            }
        }
    }
    pub fn material_progress(&self) -> Option<MaterialBalanceProgress> {
        self.material.as_ref()?.progress
    }
}
