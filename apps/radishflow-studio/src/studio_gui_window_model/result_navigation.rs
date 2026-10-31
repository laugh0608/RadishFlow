use super::*;
use crate::FlowResultGuard;

impl StudioGuiWindowCommandActionModel {
    pub(super) fn with_result_guard(mut self, guard: Option<&FlowResultGuard>) -> Self {
        self.result_guard = Some(guard.cloned().unwrap_or_else(FlowResultGuard::unavailable));
        self
    }

    /// Called again at activation, because a projected window can outlive its run.
    pub fn result_is_current(&self, workspace: &rf_ui::WorkspaceState) -> bool {
        let Some(guard) = &self.result_guard else {
            return true;
        };
        guard.is_current(workspace)
            && match crate::inspector_target_from_command_id(&self.command_id) {
                Some(rf_ui::InspectorTarget::Unit(id)) => {
                    workspace.document.flowsheet.units.contains_key(&id)
                }
                Some(rf_ui::InspectorTarget::Stream(id)) => {
                    workspace.document.flowsheet.streams.contains_key(&id)
                }
                None => true,
            }
    }
}

pub(super) fn qualify_diagnostics(
    actions: &mut [StudioGuiWindowDiagnosticTargetActionModel],
    guard: Option<&FlowResultGuard>,
) {
    for action in actions {
        action.action = action.action.clone().with_result_guard(guard);
    }
}

pub(super) fn qualify_failure(
    failure: &mut StudioGuiWindowFailureResultModel,
    guard: Option<&FlowResultGuard>,
) {
    qualify_diagnostics(&mut failure.diagnostic_actions, guard);
    if let Some(action) = &mut failure.recovery_action {
        *action = action.clone().with_result_guard(guard);
    }
    if let Some(target) = &mut failure.recovery_target {
        target.action = target.action.clone().with_result_guard(guard);
    }
    let Some(detail) = &mut failure.diagnostic_detail else {
        return;
    };
    for target in detail
        .related_units
        .iter_mut()
        .chain(&mut detail.related_streams)
    {
        target.action = target.action.clone().with_result_guard(guard);
    }
    for stream in &mut detail.related_stream_results {
        stream.focus_action = stream.focus_action.clone().with_result_guard(guard);
    }
    for port in &mut detail.related_ports {
        port.unit_action = port.unit_action.clone().with_result_guard(guard);
        if let Some(stream) = &mut port.stream_result {
            stream.focus_action = stream.focus_action.clone().with_result_guard(guard);
        }
    }
}
