//! Application run receipts shared by inline and desktop execution adapters.
use crate::{
    StudioAppAuthCacheContext, StudioAppFacade, StudioRunId, StudioRunOutcome, StudioRunSubmission,
    StudioRunSummary, StudioWorkspaceRunDispatch, StudioWorkspaceRunFailed,
    StudioWorkspaceRunOutcome, WorkspaceRunBlock, WorkspaceRunCancelReason, WorkspaceRunCommand,
    WorkspaceRunSubmission, WorkspaceSolveSkipReason,
    flow_actions::{FlowEditConflict, FlowEditContext},
};
use rf_types::{EquipmentTaskId, RfError};
use rf_ui::{
    AppState,
    engineering::{
        EquipmentCaseKey,
        editor::{
            EquipmentActionContext, EquipmentActionError, EquipmentEditError,
            EquipmentEditorCommand,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentCalculationTarget {
    Sizing(EquipmentTaskId),
    Rating(EquipmentCaseKey),
}
impl EquipmentCalculationTarget {
    pub fn editor_command(&self) -> EquipmentEditorCommand {
        match self {
            Self::Sizing(id) => EquipmentEditorCommand::Calculate(id.clone()),
            Self::Rating(member) => EquipmentEditorCommand::CalculateRating(member.clone()),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRunRequest {
    Flow {
        expected: FlowEditContext,
        command: WorkspaceRunCommand,
    },
    Equipment {
        expected: EquipmentActionContext,
        target: EquipmentCalculationTarget,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioRunAccess {
    Allowed,
    ReadOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRunRejection {
    ReadOnly,
    FlowContext(FlowEditConflict),
    EquipmentContext(EquipmentActionError),
    Flow(WorkspaceRunBlock),
    Equipment(EquipmentEditError),
}
/// Execution outcome and publication are distinct. Current result eligibility is
/// deliberately queried from AF5a at the time it is consumed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRunCompletion {
    pub id: StudioRunId,
    pub outcome: StudioRunOutcome,
    pub applied: bool,
    pub cancellation: Option<WorkspaceRunCancelReason>,
    pub diagnostic_code: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRunStartFailure {
    Infrastructure(RfError),
    Flow(StudioWorkspaceRunFailed),
    MissingExecutionEvidence,
}
impl std::fmt::Display for StudioRunStartFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infrastructure(error) => std::fmt::Display::fmt(error, f),
            Self::Flow(failed) => f.write_str(&failed.message),
            Self::MissingExecutionEvidence => {
                f.write_str("inline solve returned without execution evidence")
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRunReply {
    Accepted(StudioRunId),
    AlreadyRunning(StudioRunId),
    Busy(StudioRunId),
    Closing,
    Skipped(WorkspaceSolveSkipReason),
    Blocked(StudioRunRejection),
    StartFailed {
        id: Option<StudioRunId>,
        error: StudioRunStartFailure,
    },
    Completed(StudioRunCompletion),
}
impl From<StudioRunSubmission> for StudioRunReply {
    fn from(submission: StudioRunSubmission) -> Self {
        match submission {
            StudioRunSubmission::Accepted(id) => Self::Accepted(id),
            StudioRunSubmission::AlreadyRunning(id) => Self::AlreadyRunning(id),
            StudioRunSubmission::Busy(id) => Self::Busy(id),
            StudioRunSubmission::Closing => Self::Closing,
            StudioRunSubmission::Flow(flow) => match flow {
                WorkspaceRunSubmission::Accepted(id) => Self::Accepted(StudioRunId::Flow(id)),
                WorkspaceRunSubmission::AlreadyRunning(id) => {
                    Self::AlreadyRunning(StudioRunId::Flow(id))
                }
                WorkspaceRunSubmission::Busy(id) => Self::Busy(StudioRunId::Flow(id)),
                WorkspaceRunSubmission::Skipped(reason) => Self::Skipped(reason),
                WorkspaceRunSubmission::Blocked(WorkspaceRunBlock::Closing) => Self::Closing,
                WorkspaceRunSubmission::Blocked(reason) => {
                    Self::Blocked(StudioRunRejection::Flow(reason))
                }
            },
        }
    }
}
impl StudioRunSummary {
    pub fn completion(&self) -> StudioRunCompletion {
        StudioRunCompletion {
            id: self.activity.id.clone(),
            outcome: self.outcome,
            applied: self.applied,
            cancellation: self.cancellation,
            diagnostic_code: self.diagnostic_code.clone(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentRunError {
    Rejected(EquipmentEditError),
    StartFailed { id: StudioRunId, error: RfError },
}
impl EquipmentRunError {
    pub(crate) fn legacy_error(self) -> RfError {
        match self {
            Self::Rejected(error) => RfError::invalid_input(error.to_string()),
            Self::StartFailed { error, .. } => error,
        }
    }
    fn reply(self) -> StudioRunReply {
        match self {
            Self::Rejected(error) => StudioRunReply::Blocked(StudioRunRejection::Equipment(error)),
            Self::StartFailed { id, error } => StudioRunReply::StartFailed {
                id: Some(id),
                error: StudioRunStartFailure::Infrastructure(error),
            },
        }
    }
}

impl StudioAppFacade {
    /// Current host access and input context are checked on every submission.
    /// Resource loading and single-worker coordination remain in the existing owners.
    pub fn execute_run_action(
        &self,
        app: &mut AppState,
        cache: &StudioAppAuthCacheContext<'_>,
        access: StudioRunAccess,
        request: StudioRunRequest,
    ) -> StudioRunReply {
        if access == StudioRunAccess::ReadOnly {
            return StudioRunReply::Blocked(StudioRunRejection::ReadOnly);
        }
        match request {
            StudioRunRequest::Equipment { expected, target } => {
                if let Err(error) = expected.validate(&app.workspace.engineering) {
                    return StudioRunReply::Blocked(StudioRunRejection::EquipmentContext(error));
                }
                let command = target.editor_command();
                if let Some(background) = &self.background {
                    return match background.owner.borrow_mut().submit_equipment_typed(
                        app,
                        &command,
                        std::time::SystemTime::now(),
                    ) {
                        Ok(submission) => submission.into(),
                        Err(error) => error.reply(),
                    };
                }
                let workspace = &mut app.workspace;
                let request = match workspace
                    .equipment_editor
                    .prepare_calculation(&mut workspace.engineering, &command)
                {
                    Ok(request) => request,
                    Err(error) => {
                        return StudioRunReply::Blocked(StudioRunRejection::Equipment(error));
                    }
                };
                let id = StudioRunId::Equipment(request.qualification().id().clone());
                let completed = request.execute();
                let outcome = if completed.succeeded() {
                    StudioRunOutcome::Succeeded
                } else {
                    StudioRunOutcome::Failed
                };
                let applied = workspace
                    .engineering
                    .complete_calculation(completed)
                    .is_ok();
                StudioRunReply::Completed(StudioRunCompletion {
                    id,
                    outcome,
                    applied,
                    cancellation: None,
                    diagnostic_code: None,
                })
            }
            StudioRunRequest::Flow { expected, command } => {
                if let Err(error) = expected.validate(&app.workspace) {
                    return StudioRunReply::Blocked(StudioRunRejection::FlowContext(error));
                }
                let previous = flow_execution_id(app);
                match self.run_workspace_from_auth_cache(app, cache, &command) {
                    Ok(dispatch) => normalize_flow_dispatch(app, previous, dispatch),
                    Err(error) => StudioRunReply::StartFailed {
                        id: None,
                        error: StudioRunStartFailure::Infrastructure(error),
                    },
                }
            }
        }
    }
}

fn flow_execution_id(app: &AppState) -> Option<StudioRunId> {
    use rf_ui::result_query::FlowResultView;
    match app.workspace.flow_result() {
        FlowResultView::Current { origin, .. } | FlowResultView::Failed { origin, .. } => origin
            .execution()
            .map(|e| StudioRunId::Flow(e.run_id.clone())),
        _ => None,
    }
}
fn normalize_flow_dispatch(
    app: &AppState,
    previous: Option<StudioRunId>,
    dispatch: StudioWorkspaceRunDispatch,
) -> StudioRunReply {
    match dispatch.outcome {
        StudioWorkspaceRunOutcome::Background(submission) => submission.into(),
        StudioWorkspaceRunOutcome::Blocked(blocked) => StudioRunReply::Blocked(
            StudioRunRejection::Flow(WorkspaceRunBlock::Preflight(blocked)),
        ),
        StudioWorkspaceRunOutcome::Skipped(reason) => StudioRunReply::Skipped(reason),
        outcome => {
            let (outcome, error) = match outcome {
                StudioWorkspaceRunOutcome::Started(_) => (StudioRunOutcome::Succeeded, None),
                StudioWorkspaceRunOutcome::Failed(failed) => (
                    StudioRunOutcome::Failed,
                    Some(StudioRunStartFailure::Flow(failed)),
                ),
                _ => unreachable!(),
            };
            match flow_execution_id(app).filter(|id| Some(id) != previous.as_ref()) {
                Some(id) => StudioRunReply::Completed(StudioRunCompletion {
                    id,
                    outcome,
                    applied: true,
                    cancellation: None,
                    diagnostic_code: app
                        .workspace
                        .solve_session
                        .latest_diagnostic
                        .as_ref()
                        .and_then(|d| d.primary_code.clone()),
                }),
                None => StudioRunReply::StartFailed {
                    id: None,
                    error: error.unwrap_or(StudioRunStartFailure::MissingExecutionEvidence),
                },
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioEquipmentDispatch {
    Interaction,
    Action(rf_ui::engineering::editor::EquipmentActionReceipt),
    Run(StudioRunReply),
    Rejected(EquipmentEditError),
}
