//! Confirm one saved pair against the current snapshot before freezing its inlets.
use super::*;
use radishflow_studio::{H3AssociationSaveTarget, H3FlowReviewResult, StudioH3FlowReview};

pub(super) struct DetachmentView {
    member: EquipmentCaseKey,
    destination: String,
    focus_cancel: bool,
    notice: Option<String>,
}

enum Action {
    Stay,
    Run,
    ChooseCopy,
    Source,
    Confirm,
    Cancel,
    Reload,
}

impl DetachmentView {
    pub(super) fn new(member: EquipmentCaseKey) -> Self {
        Self {
            member,
            destination: String::new(),
            focus_cancel: false,
            notice: None,
        }
    }

    fn render(
        &mut self,
        ctx: &egui::Context,
        review: &StudioH3FlowReview,
        busy: bool,
        native: bool,
        l: StudioShellLocale,
    ) -> Action {
        let mut action = Action::Stay;
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let prepared = review.prepared_detachment();
                ui.heading(if prepared.is_some() {
                    tr(l, "确认解除流股关联", "Confirm stream detachment")
                } else {
                    tr(l, "解除流股关联", "Detach stream association")
                });
                ui.label(format!("{} / {}", self.member.task_id, self.member.case_id));
                if let Some((preview, path)) = prepared {
                    for (side, (stream, inlet)) in [
                        (tr(l, "热侧", "Hot side"), preview.hot()),
                        (tr(l, "冷侧", "Cold side"), preview.cold()),
                    ] {
                        ui.label(format!(
                            "{side} · {} · {}",
                            stream.id,
                            stream.creation_id.as_str()
                        ));
                        // Qualification requires all three finite SI values on each side.
                        ui.label(format!(
                            "{} K · {} Pa · {} mol/s",
                            inlet.temperature_k.unwrap(),
                            inlet.pressure_pa.unwrap(),
                            inlet.molar_flow_mol_s.unwrap()
                        ));
                    }
                    ui.label(format!(
                        "{} {} · #{}",
                        tr(l, "快照", "Snapshot"),
                        preview.snapshot_id().as_str(),
                        preview.sequence()
                    ));
                    if let Some(package) = preview
                        .origin()
                        .execution()
                        .and_then(|e| e.package.as_ref())
                    {
                        ui.label(format!("{} · v{}", package.package_id, package.version));
                    }
                    let (qualification, revision) = review.qualification_identity();
                    ui.label(format!("{qualification} · v{revision}"));
                    ui.label(path.display().to_string());
                    ui.label(tr(
                        l,
                        "后续流程变化不会更新这些入口。",
                        "Future flowsheet changes will not update these inlets.",
                    ));
                    let cancel = h3::button(ui, tr(l, "取消", "Cancel"));
                    if std::mem::take(&mut self.focus_cancel) {
                        cancel.request_focus();
                    }
                    if cancel.clicked() {
                        action = Action::Cancel;
                    }
                    if h3::button(ui, tr(l, "确认保存独立工况", "Save independent case")).clicked()
                    {
                        action = Action::Confirm;
                    }
                } else {
                    if review.detachment_needs_reload() {
                        ui.label(tr(
                            l,
                            "源工程已变化。请重新载入并核对流股和入口。",
                            "The source changed. Reload and review the streams and inlets.",
                        ));
                        if h3::button(ui, tr(l, "重新载入来源工程", "Reload source project"))
                            .clicked()
                        {
                            action = Action::Reload;
                        }
                    } else if busy {
                        ui.label(tr(l, "正在运行流程…", "Running flowsheet…"));
                    } else if review.has_current_flow() {
                        if !native {
                            ui.label(tr(l, "新工程路径", "New project path"));
                            ui.text_edit_singleline(&mut self.destination);
                        }
                        if ui
                            .add_enabled(
                                native || !self.destination.trim().is_empty(),
                                egui::Button::new(tr(l, "另存为新工程…", "Save as new project…")),
                            )
                            .clicked()
                        {
                            action = Action::ChooseCopy;
                        }
                        if h3::button(ui, tr(l, "写回来源工程…", "Write back to source…")).clicked()
                        {
                            action = Action::Source;
                        }
                    } else if h3::button(ui, tr(l, "运行流程并预览", "Run flowsheet for preview"))
                        .clicked()
                    {
                        action = Action::Run;
                    }
                    if let H3FlowReviewResult::Blocked(error) | H3FlowReviewResult::Failed(error) =
                        review.result(&self.member)
                    {
                        ui.label(error.to_string());
                    }
                    if let Some(notice) = &self.notice {
                        ui.label(notice);
                    }
                    if h3::button(ui, tr(l, "取消", "Cancel")).clicked() {
                        action = Action::Cancel;
                    }
                }
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            Action::Cancel
        } else {
            action
        }
    }
}

impl ReadyAppState {
    pub(super) fn render_h3_detachment(&mut self, ctx: &egui::Context) {
        let view = self.read_only_project.as_mut().expect("detachment view");
        let dialog = view.detachment.as_mut().expect("detachment");
        let StudioReadOnlyProject::H3Flow(review) = view.project.as_mut() else {
            unreachable!()
        };
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_read_only_project();
            return;
        }
        let native = self.project_file_picker.supports_project_dialogs();
        let action = dialog.render(
            ctx,
            review,
            review.activity(&self.h3_flow_run).is_some() || !self.h3_flow_run.is_idle(),
            native,
            self.locale,
        );
        let retained = self
            .platform_host
            .modeling_inputs()
            .workspace
            .document_path
            .as_deref();
        let protected = |target: &std::path::Path| retained.is_some_and(|p| paths_match(p, target));
        let message = tr(
            self.locale,
            "当前工作区使用此文件，请另选路径。",
            "The current workspace owns this file. Choose a different path.",
        );
        match action {
            Action::Stay => {
                if review.poll(&mut self.h3_flow_run, SystemTime::now()) {
                    ctx.request_repaint();
                }
                return;
            }
            Action::Run => {
                review.start_detachment_preview(
                    &mut self.h3_flow_run,
                    &dialog.member,
                    SystemTime::now(),
                );
            }
            Action::Cancel | Action::Reload => {
                review.cancel(&mut self.h3_flow_run);
                review.cancel_detachment();
                view.detachment = None;
                if matches!(action, Action::Reload) {
                    self.refresh_h3_review_on_return(true);
                }
            }
            Action::ChooseCopy | Action::Source => {
                let path = if matches!(action, Action::ChooseCopy) {
                    if native {
                        let Some(path) = self.project_file_picker.pick_save_project_file() else {
                            return;
                        };
                        Some(path)
                    } else {
                        Some(PathBuf::from(dialog.destination.trim()))
                    }
                } else {
                    None
                };
                if protected(path.as_deref().unwrap_or(&view.path)) {
                    dialog.notice = Some(message.into());
                } else {
                    let target = path.as_deref().map_or(
                        H3AssociationSaveTarget::Source,
                        H3AssociationSaveTarget::NewFile,
                    );
                    dialog.notice = review
                        .prepare_detachment(&dialog.member, target)
                        .err()
                        .map(|e| e.to_string());
                    dialog.focus_cancel = true;
                }
            }
            Action::Confirm => {
                if review
                    .prepared_detachment()
                    .is_some_and(|(_, path)| protected(path))
                {
                    review.cancel_detachment();
                    dialog.notice = Some(message.into());
                } else {
                    match review.confirm_detachment(&dialog.member) {
                        Err(error) => dialog.notice = Some(error.to_string()),
                        Ok(path) => match read_studio_gui_project(&path) {
                            Ok(StudioProjectOpen::ReadOnly(project))
                                if match project.as_ref() {
                                    StudioReadOnlyProject::H3(inputs) => {
                                        inputs.case(&dialog.member).is_some()
                                    }
                                    StudioReadOnlyProject::H3Flow(linked) => linked
                                        .saved_inputs()
                                        .case(&dialog.member)
                                        .is_some_and(|task| task.case.flow_association.is_none()),
                                    _ => false,
                                } =>
                            {
                                let mut next = ReadOnlyProjectView::new(path, project);
                                next.h3.select_saved_case(dialog.member.clone());
                                next.independent_tab = true;
                                self.read_only_project = Some(next);
                            }
                            _ => {
                                view.association_notice = Some(format!(
                                    "{} · {}",
                                    tr(
                                        self.locale,
                                        "已保存，无法重新打开",
                                        "Saved, could not reopen"
                                    ),
                                    path.display()
                                ));
                                view.detachment = None;
                            }
                        },
                    }
                }
            }
        }
        ctx.request_repaint();
    }
}
