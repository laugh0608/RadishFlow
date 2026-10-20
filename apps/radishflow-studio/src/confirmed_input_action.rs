use rf_types::{RfError, RfResult};
use rf_ui::{
    AppState, InputDiscardScope, InputEditCheckpoint, InspectorTarget, RunPanelRecoveryAction,
};

use crate::{StudioDocumentHistoryCommand, StudioRuntimeDispatch};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioInputAction {
    History(StudioDocumentHistoryCommand),
    Delete(InspectorTarget),
    Repair(RunPanelRecoveryAction),
}

impl StudioInputAction {
    pub fn discard_scope(&self) -> InputDiscardScope {
        match self {
            Self::Delete(target) => InputDiscardScope::Object(target.clone()),
            Self::History(_) | Self::Repair(_) => InputDiscardScope::All,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioConfirmedInputAction {
    pub checkpoint: InputEditCheckpoint,
    pub action: StudioInputAction,
}

pub(crate) fn apply_confirmed_input_action(
    app: &mut AppState,
    request: &StudioConfirmedInputAction,
) -> RfResult<StudioRuntimeDispatch> {
    if let StudioInputAction::Delete(target) = &request.action
        && app.workspace.drafts.active_target.as_ref() != Some(target)
    {
        return Err(RfError::invalid_input(
            "deletion selection changed; review the operation again",
        ));
    }
    if let StudioInputAction::Repair(expected) = &request.action {
        let current = rf_ui::RunPanelWidgetModel::from_state(&app.workspace.run_panel);
        if current.recovery_action() != Some(expected) || expected.mutation.is_none() {
            return Err(RfError::invalid_input(
                "repair action changed; review the operation again",
            ));
        }
    }
    app.apply_with_confirmed_input_discard(
        &request.checkpoint,
        &request.action.discard_scope(),
        |candidate| match &request.action {
            StudioInputAction::History(command) => {
                crate::dispatch_document_history(candidate, *command)
                    .map(StudioRuntimeDispatch::DocumentHistory)
            }
            StudioInputAction::Delete(target) => {
                let now = std::time::SystemTime::now();
                let revision = match target {
                    InspectorTarget::Unit(id) => candidate.delete_unit(id, now)?,
                    InspectorTarget::Stream(id) => {
                        candidate
                            .delete_stream_and_connections(id, now)?
                            .ok_or_else(|| RfError::invalid_input("stream no longer exists"))?
                            .revision
                    }
                };
                Ok(StudioRuntimeDispatch::ObjectDeletion {
                    target: target.clone(),
                    revision,
                })
            }
            StudioInputAction::Repair(_) => crate::apply_run_panel_recovery_action(candidate)
                .map(StudioRuntimeDispatch::RunPanelRecovery)
                .ok_or_else(|| RfError::invalid_input("repair action is unavailable")),
        },
    )
}
