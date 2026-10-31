use super::*;

/// Unsupported projects are inspected separately, never partially loaded into
/// the editable host. The existing workspace is retained until the view closes.
pub(super) struct ReadOnlyProjectView {
    pub(super) path: PathBuf,
    pub(super) project: Box<rf_store::project_v3::ReadOnlyProjectV3>,
}
impl ReadyAppState {
    pub(super) fn render_read_only_project(&mut self, ctx: &egui::Context) -> bool {
        if self.read_only_project.is_some() {
            self.render_background_activity(ctx);
            if self.background_run_details_open {
                let window = self.platform_host.snapshot().window_model();
                self.render_bottom_drawer(ctx, &window);
            }
        }
        let Some(view) = &self.read_only_project else {
            return false;
        };
        let zh = self.locale == StudioShellLocale::ZhCn;
        let mut close =
            ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.viewport().close_requested());
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(if zh { "工程只读检查" } else { "Read-only project inspection" });
            ui.label(view.path.display().to_string());
            ui.label(format!("{} · {}", view.project.document().metadata.title, view.project.document().metadata.document_id));
            ui.label(if zh {
                format!("含 {} 个设备任务；方法或旧任务结构不受支持。整份工程禁止编辑、计算及保存，原始数据保持不变。", view.project.task_count())
            } else {
                format!("{} equipment tasks; unsupported method or legacy task payload. Editing, calculation and saving are disabled for the entire project. Original data is preserved.", view.project.task_count())
            });
            if ui.button(if zh { "关闭检查，返回工作区" } else { "Close inspection and return to workspace" }).clicked() { close = true; }
            ui.separator();
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(egui::Label::new(egui::RichText::new(view.project.original_json()).monospace()).selectable(true));
            });
        });
        if close {
            self.read_only_project = None;
        }
        true
    }
}

impl ReadyAppState {
    pub(super) fn check_project_event_access(&self, event: &StudioGuiEvent) -> RfResult<()> {
        if self.read_only_project.is_some() && event_requires_editable_workspace(event) {
            return Err(rf_types::RfError::invalid_input(match self.locale {
                StudioShellLocale::ZhCn => "当前工程只读，不能编辑或启动计算。",
                StudioShellLocale::En => {
                    "This project is read-only; editing and calculation are unavailable."
                }
            }));
        }
        Ok(())
    }
}

fn event_requires_editable_workspace(event: &StudioGuiEvent) -> bool {
    matches!(
        event,
        StudioGuiEvent::WindowTriggerRequested { .. }
            | StudioGuiEvent::UiCommandRequested { .. }
            | StudioGuiEvent::ShortcutPressed { .. }
            | StudioGuiEvent::InspectorFieldDraftUpdateRequested { .. }
            | StudioGuiEvent::InspectorFieldDraftCommitRequested { .. }
            | StudioGuiEvent::InspectorFieldDraftBatchCommitRequested { .. }
            | StudioGuiEvent::InspectorFieldDraftDiscardRequested { .. }
            | StudioGuiEvent::InspectorFieldDraftBatchDiscardRequested { .. }
            | StudioGuiEvent::InspectorCompositionNormalizeRequested { .. }
            | StudioGuiEvent::InspectorCompositionComponentAddRequested { .. }
            | StudioGuiEvent::InspectorCompositionComponentRemoveRequested { .. }
            | StudioGuiEvent::CanvasSuggestionAcceptRequested
            | StudioGuiEvent::CanvasSuggestionAcceptByIdRequested { .. }
            | StudioGuiEvent::CanvasPendingEditCommitRequested { .. }
            | StudioGuiEvent::CanvasUnitLayoutMoveRequested { .. }
    )
}
