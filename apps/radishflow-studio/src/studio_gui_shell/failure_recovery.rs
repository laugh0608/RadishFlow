use super::*;
use radishflow_studio::{
    StudioGuiHostCommandOutcome, StudioGuiHostUiCommandDispatchResult, StudioRuntimeDispatch,
};

impl ReadyAppState {
    pub(super) fn render_failure_recovery_action(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        let Some(failure) = window.runtime.latest_failure.as_ref() else {
            return;
        };
        let Some(action) = failure.recovery_action.as_ref() else {
            return;
        };
        let recovery = window
            .runtime
            .control_state
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref());
        let Some(recovery) = recovery else {
            return;
        };
        ui.horizontal_wrapped(|ui| {
            ui.small(match (self.locale, recovery.mutation.is_some()) {
                (StudioShellLocale::ZhCn, true) => "修复模型（可撤销）",
                (StudioShellLocale::ZhCn, false) => "定位问题（不修改模型）",
                (StudioShellLocale::En, true) => "Repair model (undoable)",
                (StudioShellLocale::En, false) => "Locate problem (no model changes)",
            });
            let _ = self.render_small_command_action(ui, action);
        });
    }

    pub(super) fn follow_run_and_recovery_outcome(
        &mut self,
        dispatch: &StudioGuiPlatformExecutedDispatch,
    ) {
        let executed = match &dispatch.dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(executed),
                ),
            )
            | StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowDispatched(
                executed,
            )) => Some(executed),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::LifecycleDispatched(lifecycle),
            ) => lifecycle.dispatch.as_ref(),
            _ => None,
        };
        let Some(executed) = executed else {
            return;
        };
        let recovery = match &executed.effects.runtime_report.dispatch {
            StudioRuntimeDispatch::AppCommand(outcome)
                if matches!(
                    outcome.dispatch,
                    radishflow_studio::StudioAppResultDispatch::WorkspaceRun(_)
                ) =>
            {
                self.update_workbench_tabs_after_run(&dispatch.dispatch.window);
                return;
            }
            StudioRuntimeDispatch::RunPanelRecovery(recovery) => recovery,
            _ => return,
        };
        self.update_workbench_tabs_after_run(&dispatch.dispatch.window);
        let Some(target) = recovery.applied_target.as_ref() else {
            // A successful recovery can remove its target (for example an orphan stream).
            // Clear the old location instead of reporting the intentional removal as a failure.
            self.canvas_command_result = None;
            self.canvas_viewport_navigation.active_anchor = None;
            return;
        };
        let command_id = radishflow_studio::inspector_target_command_id(target);
        let navigation = self.canvas_object_navigation_request(&command_id);
        let located =
            self.record_canvas_viewport_navigation_for_command(&command_id, navigation.as_ref());
        self.record_canvas_object_navigation_feedback(navigation.as_ref(), located, None);
    }

    fn update_workbench_tabs_after_run(&mut self, window: &StudioGuiWindowModel) {
        if window.runtime.latest_failure.is_some() {
            self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
            self.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
        } else if window.runtime.latest_solve_snapshot.is_some() {
            self.right_sidebar_tab = StudioShellRightSidebarTab::ModuleResults;
            self.bottom_drawer_tab = StudioShellBottomDrawerTab::ResultsTable;
        } else {
            self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
            self.bottom_drawer_tab = StudioShellBottomDrawerTab::RunLog;
        }
    }
}
