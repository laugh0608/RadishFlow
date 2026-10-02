use super::app::{
    close_workspace_canceled_notice_title, close_workspace_discard_notice_detail,
    current_workspace_remains_open_notice_detail, unsaved_changes_notice_title,
};
use super::*;

impl ReadyAppState {
    pub(super) fn sync_viewport_close(&mut self, ctx: &egui::Context) -> bool {
        if !ctx.input(|input| input.viewport().close_requested()) {
            return false;
        }

        let should_stop_rendering = self.close_current_window_for_viewport_request();
        if !should_stop_rendering {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        should_stop_rendering
    }

    pub(super) fn close_current_window_for_viewport_request(&mut self) -> bool {
        if self.unit_settings.is_open() || self.pending_format_upgrade.is_some() {
            return false;
        }
        let Some(window_id) = self.current_window_id() else {
            return true;
        };

        if self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .save_state
            .needs_close_confirmation()
        {
            self.request_close_window_confirmation(window_id);
            return false;
        }

        self.close_window_without_confirmation(window_id)
    }

    fn request_close_window_confirmation(&mut self, window_id: StudioWindowHostId) {
        self.project_open.departure_checkpoint = Some(self.project_departure_checkpoint());
        self.project_open.pending_confirmation = None;
        self.project_open.pending_blank_project_confirmation = false;
        self.project_open.pending_authoring_blank_project = None;
        self.project_open.pending_save_as_overwrite = None;
        self.project_open.pending_close_window_confirmation = Some(window_id);
        self.project_open.notice = None;
        self.platform_host
            .record_activity_line("close blocked by unsaved workspace changes".to_string());
    }

    pub(super) fn render_pending_close_window_dialog(&mut self, ctx: &egui::Context) {
        if self.render_format_upgrade(ctx) || self.render_unit_settings(ctx) {
            return;
        }
        if self
            .project_open
            .pending_close_window_confirmation
            .is_none()
        {
            return;
        }

        let response = egui::Modal::new(egui::Id::new("close-project-confirmation"))
            .show(ctx, |ui| {
                ui.set_width((ctx.screen_rect().width() - 32.0).clamp(240.0, 520.0));
                ui.heading(unsaved_changes_notice_title(self.locale));
                egui::ScrollArea::vertical()
                    .max_height((ctx.screen_rect().height() - 160.0).max(120.0))
                    .show(ui, |ui| {
                        render_wrapped_label(ui, close_workspace_discard_notice_detail(self.locale));
                        self.render_project_departure_inputs(ui);
                        if let Some(notice) = &self.project_open.notice {
                            ui.label(&notice.detail);
                        }
                        let state = self
                            .platform_host
                            .snapshot()
                            .runtime
                            .workspace_document
                            .save_state;
                        render_project_save_state(ui, &state, self.locale);
                        if state.pending_input_count > 0 {
                            ui.label(match self.locale {
                                StudioShellLocale::ZhCn => {
                                    "保存仅写入已提交内容。请取消关闭并处理未提交输入，或明确放弃后关闭。"
                                }
                                StudioShellLocale::En => concat!(
                                    "Saving includes committed content only. Cancel closing to handle ",
                                    "unsubmitted inputs, or explicitly discard them before closing."
                                ),
                            });
                        }
                    });
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    let save = ui.button(self.locale.text(ShellText::SaveAndCloseProject));
                    if save.clicked() && self.save_pending_close_window() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    let discard = ui.button(self.locale.text(ShellText::DiscardAndCloseProject));
                    if discard.clicked() && self.confirm_pending_close_window() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    let cancel = ui.button(self.locale.text(ShellText::CancelCloseProject));
                    if !save.has_focus() && !discard.has_focus() && !cancel.has_focus() {
                        // Start on the action that preserves the document and drafts.
                        cancel.request_focus();
                    }
                    if cancel.clicked() {
                        self.cancel_pending_close_window();
                    }
                });
            });
        if response.should_close() {
            self.cancel_pending_close_window();
        }
    }

    pub(super) fn save_pending_close_window(&mut self) -> bool {
        if self
            .project_open
            .pending_close_window_confirmation
            .is_none()
        {
            return false;
        }

        self.save_project();
        if self.pending_format_upgrade.is_some() {
            return false;
        }
        if self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .save_state
            .needs_close_confirmation()
        {
            return false;
        }

        self.finish_saved_close()
    }

    pub(super) fn finish_saved_close(&mut self) -> bool {
        if self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .save_state
            .needs_close_confirmation()
        {
            return false;
        }
        self.project_open.departure_checkpoint = Some(self.project_departure_checkpoint());
        self.confirm_pending_close_window()
    }

    pub(super) fn confirm_pending_close_window(&mut self) -> bool {
        if self
            .project_open
            .pending_close_window_confirmation
            .is_none()
            || !self.validate_project_departure()
        {
            return false;
        }
        let Some(window_id) = self.project_open.pending_close_window_confirmation.take() else {
            return false;
        };
        self.close_window_without_confirmation(window_id)
    }

    pub(super) fn cancel_pending_close_window(&mut self) {
        self.project_open.pending_close_window_confirmation = None;
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Info,
            title: close_workspace_canceled_notice_title(self.locale).to_string(),
            detail: current_workspace_remains_open_notice_detail(self.locale).to_string(),
        });
    }

    fn close_window_without_confirmation(&mut self, window_id: StudioWindowHostId) -> bool {
        self.cancel_drag_session(Some(window_id));
        self.dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id });
        self.logical_window_count() == 0
    }
}
