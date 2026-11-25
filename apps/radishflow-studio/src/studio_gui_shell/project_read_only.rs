use super::locale::tr;
use super::*;
use radishflow_studio::{StudioH3AssociationEditor, StudioProjectOpen, StudioReadOnlyProject};
use rf_ui::engineering::EquipmentCaseKey;
mod association_editor;
mod h3;
mod h3_flow;

use association_editor::{AssociationEditorView, EditorAction};

struct ReviewControls {
    escape: bool,
    edit_available: bool,
    focus_edit: bool,
    edit: Option<EquipmentCaseKey>,
}

impl ReviewControls {
    fn association_button(
        &mut self,
        ui: &mut egui::Ui,
        member: Option<&EquipmentCaseKey>,
        locale: StudioShellLocale,
        linked: bool,
    ) {
        if self.edit_available
            && let Some(member) = member
        {
            let label = if linked {
                tr(locale, "编辑流股关联", "Edit stream association")
            } else {
                tr(locale, "建立流股关联", "Set up stream association")
            };
            let button = h3::button(ui, label);
            if self.focus_edit {
                button.request_focus();
                self.focus_edit = false;
            }
            if button.clicked() {
                self.edit = Some(member.clone());
            }
        }
    }
}

/// Unsupported projects are inspected separately, never partially loaded into
/// the editable host. The existing workspace is retained until the view closes.
pub(super) struct ReadOnlyProjectView {
    pub(super) path: PathBuf,
    pub(super) project: Box<radishflow_studio::StudioReadOnlyProject>,
    h3: h3::ReviewState,
    h3_flow: h3_flow::ReviewState,
    association_editable: bool,
    editor: Option<AssociationEditorView>,
    focus_edit: bool,
    association_notice: Option<String>,
}
impl ReadOnlyProjectView {
    pub(super) fn new(
        path: PathBuf,
        project: Box<radishflow_studio::StudioReadOnlyProject>,
    ) -> Self {
        let association_editable =
            matches!(
                project.as_ref(),
                StudioReadOnlyProject::H3(_) | StudioReadOnlyProject::H3Flow(_)
            ) && StudioH3AssociationEditor::supports_original(project.original_json());
        Self {
            path,
            project,
            h3: h3::ReviewState::default(),
            h3_flow: h3_flow::ReviewState::default(),
            association_editable,
            editor: None,
            focus_edit: false,
            association_notice: None,
        }
    }

    pub(super) fn is_h3_flow(&self) -> bool {
        matches!(self.project.as_ref(), StudioReadOnlyProject::H3Flow(_))
    }

    pub(super) fn has_association_editor(&self) -> bool {
        self.editor.is_some()
    }

    fn enter_association_editor(&mut self, member: EquipmentCaseKey) {
        match AssociationEditorView::open(&self.path, self.project.original_json(), member) {
            Ok(editor) => {
                self.association_notice = None;
                self.editor = Some(editor);
            }
            Err(error) => self.association_notice = Some(error.to_string()),
        }
    }
}
impl ReadyAppState {
    pub(super) fn close_read_only_project(&mut self) {
        if let Some(mut view) = self.read_only_project.take() {
            if let Some(editor) = &mut view.editor {
                editor.detach(&mut self.h3_flow_run);
            }
            if let StudioReadOnlyProject::H3Flow(review) = view.project.as_mut() {
                review.detach(&mut self.h3_flow_run);
            }
        }
    }

    pub(super) fn render_read_only_project(&mut self, ctx: &egui::Context) -> bool {
        if self.read_only_project.is_some() {
            self.render_background_activity(ctx);
            if self.background_run_details_open {
                let window = self.platform_host.snapshot().window_model();
                self.render_bottom_drawer(ctx, &window);
            }
        }
        if self
            .read_only_project
            .as_ref()
            .is_some_and(ReadOnlyProjectView::has_association_editor)
        {
            self.render_h3_association_editor(ctx);
            return true;
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
        let mut controls = ReviewControls {
            escape,
            edit_available: view.association_editable && self.h3_flow_run.is_idle(),
            focus_edit: std::mem::take(&mut view.focus_edit),
            edit: None,
        };
        if let Some(notice) = &view.association_notice {
            egui::TopBottomPanel::bottom("h3-association-open-notice").show(ctx, |ui| {
                ui.add(egui::Label::new(notice).wrap());
            });
        }
        if let StudioReadOnlyProject::H3Flow(project) = view.project.as_mut() {
            let close = close_requested
                || view.h3_flow.render(
                    ctx,
                    project,
                    &mut self.h3_flow_run,
                    &view.path,
                    self.locale,
                    &mut controls,
                );
            if close {
                self.close_read_only_project();
                ctx.request_repaint();
            } else if let Some(member) = controls.edit {
                view.enter_association_editor(member);
                ctx.request_repaint();
            } else {
                let changed = project.poll(&mut self.h3_flow_run, SystemTime::now());
                if changed {
                    view.h3_flow.run_changed(project, &self.h3_flow_run);
                    ctx.request_repaint();
                }
            }
            return true;
        }
        if let StudioReadOnlyProject::H3(project) = view.project.as_ref() {
            let close = view
                .h3
                .render(ctx, project, &view.path, self.locale, &mut controls);
            if close || close_requested {
                self.close_read_only_project();
            } else if let Some(member) = controls.edit {
                view.enter_association_editor(member);
                ctx.request_repaint();
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
            self.close_read_only_project();
        }
        true
    }

    fn render_h3_association_editor(&mut self, ctx: &egui::Context) {
        let retained = self
            .platform_host
            .modeling_inputs()
            .workspace
            .document_path
            .clone();
        let native_picker = self.project_file_picker.supports_project_dialogs();
        let view = self.read_only_project.as_mut().expect("association view");
        let editor = view.editor.as_mut().expect("association editor");
        match editor.render(
            ctx,
            &mut self.h3_flow_run,
            &view.path,
            self.locale,
            retained.as_deref(),
            native_picker,
        ) {
            EditorAction::Stay => {
                if editor.poll(&mut self.h3_flow_run) {
                    ctx.request_repaint();
                }
                return;
            }
            EditorAction::ChooseCopy => {
                editor.copy_picked(
                    self.project_file_picker.pick_save_project_file(),
                    retained.as_deref(),
                );
            }
            EditorAction::BackToWorkspace => self.close_read_only_project(),
            EditorAction::BackToReview => {
                editor.detach(&mut self.h3_flow_run);
                view.editor = None;
                view.focus_edit = true;
                self.refresh_h3_review_on_return();
            }
            EditorAction::OpenSaved(path, member) => match read_studio_gui_project(&path) {
                Ok(StudioProjectOpen::ReadOnly(project)) => {
                    editor.detach(&mut self.h3_flow_run);
                    let mut next = ReadOnlyProjectView::new(path, project);
                    if let StudioReadOnlyProject::H3Flow(review) = next.project.as_ref()
                        && review.case(&member).is_some()
                    {
                        next.h3_flow.select_saved_case(member);
                    }
                    self.read_only_project = Some(next);
                }
                Ok(StudioProjectOpen::Editable(_)) => {
                    editor.reopen_failed(rf_types::RfError::invalid_input(
                        "the saved target no longer opens as an H3 review project",
                    ))
                }
                Err(error) => editor.reopen_failed(error),
            },
        }
        ctx.request_repaint();
    }

    fn refresh_h3_review_on_return(&mut self) {
        let view = self
            .read_only_project
            .as_ref()
            .expect("returning from association editor");
        let path = view.path.clone();
        if std::fs::read_to_string(&path)
            .is_ok_and(|contents| contents == view.project.original_json())
        {
            return;
        }
        match read_studio_gui_project(&path) {
            Ok(StudioProjectOpen::ReadOnly(project)) => {
                self.close_read_only_project();
                self.read_only_project = Some(ReadOnlyProjectView::new(path, project));
            }
            result => {
                self.close_read_only_project();
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: tr(self.locale, "源工程已变化", "The source project changed").into(),
                    detail: match result {
                        Err(error) => error.to_string(),
                        _ => tr(
                            self.locale,
                            "请重新打开源工程。",
                            "Reopen the source project.",
                        )
                        .into(),
                    },
                });
            }
        }
    }
}

impl ReadyAppState {
    pub(super) fn check_project_event_access(&self, event: &StudioGuiEvent) -> RfResult<()> {
        if self.read_only_project.is_some() && event_requires_editable_workspace(event) {
            if self
                .read_only_project
                .as_ref()
                .is_some_and(ReadOnlyProjectView::is_h3_flow)
            {
                return Err(rf_types::RfError::invalid_input(tr(
                    self.locale,
                    "工程只读。可在复核页运行并复核当前工况。",
                    "This project is read-only. Run and review the selected case in the review pane.",
                )));
            }
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
