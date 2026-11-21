use super::locale::tr;
use super::*;
mod h3;

/// Unsupported projects are inspected separately, never partially loaded into
/// the editable host. The existing workspace is retained until the view closes.
pub(super) struct ReadOnlyProjectView {
    pub(super) path: PathBuf,
    pub(super) project: Box<radishflow_studio::StudioReadOnlyProject>,
    h3: h3::ReviewState,
}
impl ReadOnlyProjectView {
    pub(super) fn new(
        path: PathBuf,
        project: Box<radishflow_studio::StudioReadOnlyProject>,
    ) -> Self {
        Self {
            path,
            project,
            h3: h3::ReviewState::default(),
        }
    }
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
        let Some(view) = &mut self.read_only_project else {
            return false;
        };
        let zh = self.locale == StudioShellLocale::ZhCn;
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if let radishflow_studio::StudioReadOnlyProject::H3(project) = view.project.as_ref() {
            let close = view
                .h3
                .render(ctx, project, &view.path, self.locale, escape);
            if close || close_requested {
                self.read_only_project = None;
            }
            return true;
        }
        let mut close = escape || close_requested;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(if zh {
                "工程只读检查"
            } else {
                "Read-only project inspection"
            });
            ui.label(view.path.display().to_string());
            ui.label(format!(
                "{} · {}",
                view.project.metadata().title,
                view.project.metadata().document_id
            ));
            ui.label(format!(
                "v{} · {}",
                view.project.schema_version(),
                view.project.reason_summary()
            ));
            ui.label(if zh {
                "整份工程禁止编辑、计算及保存；原始内容完整保留。"
            } else {
                "Editing, calculation and saving are disabled. Original content is preserved."
            });
            if ui
                .button(if zh {
                    "关闭检查，返回工作区"
                } else {
                    "Close inspection and return to workspace"
                })
                .clicked()
            {
                close = true;
            }
            ui.separator();
            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(
                    egui::Label::new(egui::RichText::new(view.project.original_json()).monospace())
                        .selectable(true),
                );
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
            if self.read_only_project.as_ref().is_some_and(|view| {
                matches!(
                    view.project.as_ref(),
                    radishflow_studio::StudioReadOnlyProject::H3(_)
                )
            }) {
                return Err(rf_types::RfError::invalid_input(tr(
                    self.locale,
                    "工程只读。可在复核页复算当前工况。",
                    "This project is read-only. Rate the selected case in the review.",
                )));
            }
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
