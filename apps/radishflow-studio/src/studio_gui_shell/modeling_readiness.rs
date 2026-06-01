use super::*;
use radishflow_studio::{
    StudioModelingFocusTarget as ModelingFocusTarget,
    StudioModelingReadinessTask as ModelingReadinessTask, studio_modeling_run_blocked_detail_en,
    studio_modeling_run_blocked_detail_zh, studio_modeling_run_blocked_title_en,
    studio_modeling_run_blocked_title_zh, studio_modeling_run_blocker,
};

impl ReadyAppState {
    pub(super) fn intercept_modeling_readiness_shortcut_if_needed(
        &mut self,
        shortcut: &StudioGuiShortcut,
    ) -> bool {
        let Some(command_id) = modeling_readiness_run_command_from_shortcut(shortcut) else {
            return false;
        };
        self.intercept_modeling_readiness_run_if_needed(command_id)
    }

    pub(super) fn intercept_modeling_readiness_run_if_needed(&mut self, command_id: &str) -> bool {
        if !matches!(
            command_id,
            "run_panel.run_manual" | "run_panel.resume_workspace"
        ) {
            return false;
        }

        let blocker = {
            let document = self.platform_host.document();
            studio_modeling_run_blocker(document)
        };
        let Some(blocker) = blocker else {
            return false;
        };

        self.focus_modeling_readiness_blocker(&blocker.focus_target);
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Warning,
            title: modeling_run_blocked_title(self.locale).to_string(),
            detail: modeling_run_blocked_detail(self.locale, &blocker.task),
        });
        self.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
        self.platform_host.record_activity_line(format!(
            "blocked modeling run before solve: {:?}",
            blocker.task
        ));
        true
    }

    fn focus_modeling_readiness_blocker(&mut self, focus_target: &ModelingFocusTarget) {
        match focus_target {
            ModelingFocusTarget::Package => {
                self.left_sidebar_tab = StudioShellLeftSidebarTab::Project;
                self.right_sidebar_tab = StudioShellRightSidebarTab::Package;
            }
            ModelingFocusTarget::Palette => {
                self.left_sidebar_tab = StudioShellLeftSidebarTab::Palette;
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
            }
            ModelingFocusTarget::Stream(stream_id) => {
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                self.dispatch_ui_command(format!("inspector.focus_stream:{stream_id}"));
            }
            ModelingFocusTarget::Unit(unit_id) => {
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                self.dispatch_ui_command(format!("inspector.focus_unit:{unit_id}"));
            }
        }
    }
}

fn modeling_readiness_run_command_from_shortcut(
    shortcut: &StudioGuiShortcut,
) -> Option<&'static str> {
    match (shortcut.modifiers.as_slice(), shortcut.key) {
        ([], StudioGuiShortcutKey::F5) => Some("run_panel.run_manual"),
        ([StudioGuiShortcutModifier::Shift], StudioGuiShortcutKey::F5) => {
            Some("run_panel.resume_workspace")
        }
        _ => None,
    }
}

fn modeling_run_blocked_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => studio_modeling_run_blocked_title_en(),
        StudioShellLocale::ZhCn => studio_modeling_run_blocked_title_zh(),
    }
}

fn modeling_run_blocked_detail(locale: StudioShellLocale, task: &ModelingReadinessTask) -> String {
    match locale {
        StudioShellLocale::En => studio_modeling_run_blocked_detail_en(task),
        StudioShellLocale::ZhCn => studio_modeling_run_blocked_detail_zh(task),
    }
}
