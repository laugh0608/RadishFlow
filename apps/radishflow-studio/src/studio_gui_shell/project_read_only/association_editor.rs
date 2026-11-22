use super::{StudioShellLocale, h3, tr};
use eframe::egui;
use radishflow_studio::{
    H3AssociationPreviewState, H3AssociationSaveTarget, H3FlowStream, StudioH3AssociationEditor,
    WorkspaceRunOwner, WorkspaceRunPhase,
};
use rf_model::equipment::ConcentricTubeFlowAssociation;
use rf_types::{RfError, units::QuantityKind};
use rf_ui::engineering::EquipmentCaseKey;
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

#[derive(Clone, Default, PartialEq, Eq)]
struct Draft {
    hot: Option<H3FlowStream>,
    cold: Option<H3FlowStream>,
}

impl Draft {
    fn from_saved(pair: Option<&ConcentricTubeFlowAssociation>) -> Self {
        pair.map(|pair| Self {
            hot: Some(pair.hot.clone()),
            cold: Some(pair.cold.clone()),
        })
        .unwrap_or_default()
    }

    fn pair(&self) -> Option<ConcentricTubeFlowAssociation> {
        Some(ConcentricTubeFlowAssociation {
            hot: self.hot.clone()?,
            cold: self.cold.clone()?,
        })
        .filter(|pair| pair.hot.id != pair.cold.id)
    }
}

enum Leave {
    Review,
    Workspace,
    Case(EquipmentCaseKey),
}

enum Dialog {
    Save,
    Discard(Leave),
    Destination,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Hot,
    Cold,
    Preview,
    CancelRun,
    Save,
    SaveAs,
    Return,
    Dialog,
}

pub(super) enum EditorAction {
    Stay,
    ChooseCopy,
    BackToReview,
    BackToWorkspace,
    OpenSaved(PathBuf, EquipmentCaseKey),
}

pub(super) struct AssociationEditorView {
    id: u64,
    application: StudioH3AssociationEditor,
    member: EquipmentCaseKey,
    draft: Draft,
    dialog: Option<Dialog>,
    notice: Option<RfError>,
    source_changed: bool,
    focus: Option<Focus>,
    save_focus: Focus,
    destination: String,
    saved_path: Option<PathBuf>,
}

impl AssociationEditorView {
    pub(super) fn open(
        path: &Path,
        original: &str,
        member: EquipmentCaseKey,
    ) -> Result<Self, RfError> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let application = StudioH3AssociationEditor::open(path, original)?;
        let task = application
            .case(&member)
            .ok_or_else(|| RfError::invalid_input("saved H3 case is unavailable"))?;
        Ok(Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            draft: Draft::from_saved(task.case.flow_association.as_ref()),
            application,
            member,
            dialog: None,
            notice: None,
            source_changed: false,
            focus: Some(Focus::Hot),
            save_focus: Focus::Save,
            destination: String::new(),
            saved_path: None,
        })
    }

    pub(super) fn detach(&mut self, owner: &mut WorkspaceRunOwner) {
        self.application.detach(owner);
    }

    pub(super) fn poll(&mut self, owner: &mut WorkspaceRunOwner) -> bool {
        let running = self.application.activity(owner).is_some();
        let changed = self.application.poll(owner, SystemTime::now());
        if running && changed && self.application.activity(owner).is_none() {
            if let H3AssociationPreviewState::Rejected(error) = self.application.preview() {
                self.notice = Some(error.clone());
                self.source_changed = is_source_changed(error);
            }
            self.focus = Some(if self.can_save() {
                Focus::Save
            } else {
                Focus::Preview
            });
        }
        changed
    }

    fn saved_draft(&self) -> Draft {
        Draft::from_saved(
            self.application
                .case(&self.member)
                .and_then(|task| task.case.flow_association.as_ref()),
        )
    }

    fn changed(&self) -> bool {
        self.draft != self.saved_draft()
    }

    fn can_save(&self) -> bool {
        !self.source_changed
            && self.changed()
            && matches!(
                self.application.preview(),
                H3AssociationPreviewState::Ready(_)
            )
    }

    fn focus(&mut self, response: &egui::Response, target: Focus) {
        if self.focus == Some(target) {
            response.request_focus();
            response.scroll_to_me(None);
            self.focus = None;
        }
    }

    fn error(&mut self, error: RfError) {
        if is_source_changed(&error) {
            self.source_changed = true;
            self.application.clear_preview();
        }
        self.notice = Some(error);
    }

    pub(super) fn reopen_failed(&mut self, error: RfError) {
        self.notice = Some(error);
        self.focus = Some(Focus::Save);
    }

    pub(super) fn copy_picked(&mut self, target: Option<PathBuf>, retained: Option<&Path>) {
        if let Some(path) = target {
            self.prepare(H3AssociationSaveTarget::NewFile(&path), retained);
        } else {
            self.focus = Some(Focus::SaveAs);
        }
    }

    fn prepare(&mut self, target: H3AssociationSaveTarget<'_>, retained: Option<&Path>) {
        if !self.can_save() {
            return;
        }
        self.application.cancel_save();
        self.notice = None;
        if let Err(error) = self.application.prepare_save(target) {
            self.error(error);
            self.focus = Some(self.save_focus);
        } else if self.target_in_workspace(retained) {
            self.refuse_workspace_target();
        } else {
            self.dialog = Some(Dialog::Save);
            self.focus = Some(Focus::Dialog);
        }
    }

    fn target_in_workspace(&self, retained: Option<&Path>) -> bool {
        retained
            .zip(self.application.prepared())
            .is_some_and(|(retained, prepared)| super::paths_match(retained, prepared.path()))
    }

    fn refuse_workspace_target(&mut self) {
        self.application.cancel_save();
        self.error(
            RfError::invalid_input("choose a new path; the retained workspace owns this file")
                .with_diagnostic_code("equipment.h3.flow.workspace_save_target"),
        );
        self.focus = Some(Focus::SaveAs);
    }

    fn leave(&mut self, intent: Leave) -> EditorAction {
        if self.saved_path.is_none() && self.changed() {
            self.dialog = Some(Dialog::Discard(intent));
            self.focus = Some(Focus::Dialog);
            EditorAction::Stay
        } else {
            self.apply_leave(intent)
        }
    }

    fn apply_leave(&mut self, intent: Leave) -> EditorAction {
        match intent {
            Leave::Review => EditorAction::BackToReview,
            Leave::Workspace => EditorAction::BackToWorkspace,
            Leave::Case(member) => {
                self.member = member;
                self.draft = self.saved_draft();
                self.application.clear_preview();
                if !self.source_changed {
                    self.notice = None;
                }
                self.focus = Some(Focus::Hot);
                EditorAction::Stay
            }
        }
    }

    pub(super) fn render(
        &mut self,
        ctx: &egui::Context,
        owner: &mut WorkspaceRunOwner,
        path: &Path,
        locale: StudioShellLocale,
        retained: Option<&Path>,
        native_picker: bool,
    ) -> EditorAction {
        let popup_open = ctx.memory(|m| m.any_popup_open());
        let (close, escape, save, copy) = ctx.input_mut(|i| {
            let copy = i.consume_key(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::S,
            );
            (
                i.viewport().close_requested()
                    || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Q),
                i.key_pressed(egui::Key::Escape),
                !copy && i.consume_key(egui::Modifiers::COMMAND, egui::Key::S),
                copy,
            )
        });
        if close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.application.cancel_save();
            let action = self.leave(Leave::Workspace);
            if !matches!(action, EditorAction::Stay) {
                return action;
            }
        }
        let mut action = EditorAction::Stay;
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .animated(false)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.push_id(("h3-association", self.id), |ui| {
                        ui.heading(tr(locale, "编辑 H3 流程关联", "Edit H3 flow association"));
                        ui.add(egui::Label::new(path.display().to_string()).wrap());
                        ui.add_enabled_ui(self.dialog.is_none(), |ui| {
                            if let Some(destination) = self.saved_path.clone() {
                                action = self.written(ui, locale, destination);
                            } else {
                                action = self.edit(ui, owner, locale, retained, native_picker);
                            }
                        });
                    });
                });
        });
        if !matches!(action, EditorAction::Stay) {
            return action;
        }
        if self.dialog.is_some() {
            return self.render_dialog(ctx, locale, retained);
        }
        if escape && !popup_open {
            return self.leave(Leave::Review);
        }
        if copy {
            return self.request_copy(native_picker);
        }
        if save {
            self.save_focus = Focus::Save;
            self.prepare(H3AssociationSaveTarget::Source, retained);
        }
        EditorAction::Stay
    }

    fn written(
        &mut self,
        ui: &mut egui::Ui,
        l: StudioShellLocale,
        destination: PathBuf,
    ) -> EditorAction {
        ui.label(tr(l, "关联已保存。", "Association saved."));
        ui.add(egui::Label::new(destination.display().to_string()).wrap());
        self.render_notice(ui, l);
        let open = h3::button(ui, tr(l, "打开已保存工程", "Open saved project"));
        self.focus(&open, Focus::Save);
        if open.clicked() {
            EditorAction::OpenSaved(destination, self.member.clone())
        } else if h3::button(ui, tr(l, "返回工况", "Back to case")).clicked() {
            EditorAction::BackToReview
        } else {
            EditorAction::Stay
        }
    }

    fn edit(
        &mut self,
        ui: &mut egui::Ui,
        owner: &mut WorkspaceRunOwner,
        l: StudioShellLocale,
        retained: Option<&Path>,
        native_picker: bool,
    ) -> EditorAction {
        self.render_notice(ui, l);
        let running = self.application.activity(owner);
        let old_member = self.member.clone();
        let mut member = old_member.clone();
        let selected = self
            .application
            .case(&old_member)
            .map(h3::case_title)
            .unwrap_or_else(|| tr(l, "当前工况已移除", "The previous case was removed").into());
        ui.add_enabled_ui(running.is_none(), |ui| {
            ui.label(tr(l, "工况", "Case"));
            egui::ComboBox::from_id_salt("case")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for task in self.application.cases() {
                        ui.selectable_value(&mut member, h3::member(task), h3::case_title(task));
                    }
                });
        });
        if member != old_member {
            return self.leave(Leave::Case(member));
        }
        ui.add_space(6.0);
        if ui.available_width() >= 720.0 {
            ui.columns_const(|[saved, candidate]| {
                self.saved_sources(saved, l);
                self.candidate_sources(candidate, l, running.is_none());
            });
        } else {
            self.saved_sources(ui, l);
            self.candidate_sources(ui, l, running.is_none());
        }
        h3::panel(ui).show(ui, |ui| {
            ui.heading(tr(l, "入口资格和预览", "Inlet qualification and preview"));
            self.render_preview(ui, owner, l, running);
        });
        let mut action = EditorAction::Stay;
        ui.horizontal_wrapped(|ui| {
            let back = h3::button(ui, tr(l, "返回工况", "Back to case"));
            self.focus(&back, Focus::Return);
            if back.clicked() {
                action = self.leave(Leave::Review);
                return;
            }
            let reset = ui.add_enabled(
                self.changed() && running.is_none(),
                egui::Button::new(tr(l, "重置草稿", "Reset draft")).wrap(),
            );
            if reset.clicked() {
                self.draft = self.saved_draft();
                self.clear_candidate();
                self.focus = Some(Focus::Hot);
            }
            let copy = ui.add_enabled(
                self.can_save(),
                egui::Button::new(tr(l, "另存关联…", "Save association as…")).wrap(),
            );
            self.focus(&copy, Focus::SaveAs);
            if copy.clicked() {
                action = self.request_copy(native_picker);
                return;
            }
            let save = ui.add_enabled(
                self.can_save(),
                egui::Button::new(tr(l, "保存关联…", "Save association…")).wrap(),
            );
            self.focus(&save, Focus::Save);
            if save.clicked() {
                self.save_focus = Focus::Save;
                self.prepare(H3AssociationSaveTarget::Source, retained);
            }
        });
        action
    }

    fn render_preview(
        &mut self,
        ui: &mut egui::Ui,
        owner: &mut WorkspaceRunOwner,
        l: StudioShellLocale,
        running: Option<WorkspaceRunPhase>,
    ) {
        if let Some(phase) = running {
            let stopping = phase == WorkspaceRunPhase::CancelRequested;
            ui.label(if stopping {
                tr(l, "正在停止", "Stopping")
            } else {
                tr(
                    l,
                    "正在运行流程以生成预览",
                    "Running flowsheet for the preview",
                )
            });
            let cancel = ui.add_enabled(
                !stopping,
                egui::Button::new(tr(l, "取消预览运行", "Cancel preview run")).wrap(),
            );
            self.focus(&cancel, Focus::CancelRun);
            if cancel.clicked() {
                self.application.cancel(owner);
            }
            return;
        }
        match self.application.preview() {
            H3AssociationPreviewState::Ready(preview) => {
                ui.strong(tr(l, "入口资格通过", "Inlet qualification passed"));
                for (side, port) in [
                    (tr(l, "热侧", "Hot side"), preview.flow().hot()),
                    (tr(l, "冷侧", "Cold side"), preview.flow().cold()),
                ] {
                    let (stream, inlet) = port;
                    let units = self.application.display_units();
                    let values = [
                        (inlet.temperature_k, QuantityKind::AbsoluteTemperature),
                        (inlet.pressure_pa, QuantityKind::AbsolutePressure),
                        (inlet.molar_flow_mol_s, QuantityKind::MolarFlow),
                    ]
                    .into_iter()
                    .filter_map(|(value, kind)| value.map(|value| h3::quantity(value, kind, units)))
                    .collect::<Vec<_>>();
                    ui.label(format!("{side} · {} · {}", stream.id, values.join(" · ")));
                    ui.small(format!(
                        "{} {}",
                        tr(l, "创建身份", "Creation identity"),
                        stream.creation_id.as_str()
                    ));
                }
                ui.small(format!(
                    "{} · h3-pure-water v1",
                    preview.flow().snapshot_id().as_str()
                ));
                if let Some(package) = preview
                    .flow()
                    .origin()
                    .execution()
                    .and_then(|e| e.package.as_ref())
                {
                    ui.small(format!("{} · v{}", package.package_id, package.version));
                }
            }
            H3AssociationPreviewState::Cancelled => {
                ui.label(tr(l, "已取消预览", "Preview cancelled"));
            }
            H3AssociationPreviewState::Missing | H3AssociationPreviewState::Rejected(_) => {
                ui.label(tr(l, "待预览", "Preview needed"));
            }
        };
        if self.source_changed
            && h3::button(ui, tr(l, "重新载入源工程", "Reload source project")).clicked()
        {
            match self.application.reload() {
                Ok(()) => {
                    self.source_changed = false;
                    self.notice = None;
                    self.focus = Some(Focus::Hot);
                }
                Err(error) => self.error(error),
            }
        }
        let pair = self
            .draft
            .pair()
            .filter(|_| self.changed() && self.application.case(&self.member).is_some());
        let run = ui.add_enabled(
            pair.is_some() && !self.source_changed && owner.is_idle(),
            egui::Button::new(tr(l, "运行流程并预览", "Run flowsheet and preview")).wrap(),
        );
        self.focus(&run, Focus::Preview);
        if run.clicked()
            && let Some(pair) = pair
        {
            self.notice = None;
            match self
                .application
                .start(owner, self.member.clone(), pair, SystemTime::now())
            {
                Ok(()) => self.focus = Some(Focus::CancelRun),
                Err(error) => self.error(error),
            }
        }
    }

    fn clear_candidate(&mut self) {
        self.application.clear_preview();
        if !self.source_changed {
            self.notice = None;
        }
    }

    fn saved_sources(&self, ui: &mut egui::Ui, l: StudioShellLocale) {
        h3::panel(ui).show(ui, |ui| {
            ui.heading(tr(l, "已保存的入口来源", "Saved inlet sources"));
            let saved = self.saved_draft();
            if saved.hot.is_none() {
                ui.label(tr(
                    l,
                    "使用独立工况的两侧入口",
                    "Both inlets come from the independent case",
                ));
            } else {
                ui.label(format!(
                    "{} · {}",
                    tr(l, "热侧", "Hot side"),
                    self.stream_title(saved.hot.as_ref(), l)
                ));
                ui.label(format!(
                    "{} · {}",
                    tr(l, "冷侧", "Cold side"),
                    self.stream_title(saved.cold.as_ref(), l)
                ));
            }
        });
    }

    fn candidate_sources(&mut self, ui: &mut egui::Ui, l: StudioShellLocale, enabled: bool) {
        let mut draft = self.draft.clone();
        h3::panel(ui).show(ui, |ui| {
            ui.heading(tr(l, "候选关联", "Proposed association"));
            ui.add_enabled_ui(enabled, |ui| {
                self.stream_choice(ui, l, Focus::Hot, &mut draft.hot);
                self.stream_choice(ui, l, Focus::Cold, &mut draft.cold);
            });
            if draft
                .hot
                .as_ref()
                .zip(draft.cold.as_ref())
                .is_some_and(|(hot, cold)| hot.id == cold.id)
            {
                ui.label(tr(
                    l,
                    "冷热侧需要不同流股。",
                    "Choose different streams for the two sides.",
                ));
            }
        });
        if draft != self.draft {
            self.draft = draft;
            self.clear_candidate();
        }
    }

    fn stream_title(&self, stream: Option<&H3FlowStream>, l: StudioShellLocale) -> String {
        let Some(stream) = stream else {
            return tr(l, "选择流股", "Select a stream").into();
        };
        self.application
            .streams()
            .find(|(candidate, _)| candidate == stream)
            .map_or_else(
                || {
                    format!(
                        "{} · {}",
                        tr(l, "需重新选择", "Selection unavailable"),
                        stream.id
                    )
                },
                |(_, name)| format!("{name} · {}", stream.id),
            )
    }

    fn stream_choice(
        &mut self,
        ui: &mut egui::Ui,
        l: StudioShellLocale,
        side: Focus,
        choice: &mut Option<H3FlowStream>,
    ) {
        let label = if side == Focus::Hot {
            tr(l, "热侧流股", "Hot-side stream")
        } else {
            tr(l, "冷侧流股", "Cold-side stream")
        };
        let field = ui.label(label);
        let response = egui::ComboBox::from_id_salt(label)
            .width(ui.available_width().min(380.0))
            .selected_text(self.stream_title(choice.as_ref(), l))
            .show_ui(ui, |ui| {
                for (stream, name) in self.application.streams() {
                    let entry = format!("{name} · {}", stream.id);
                    ui.selectable_value(choice, Some(stream), entry);
                }
            })
            .response
            .labelled_by(field.id);
        h3::reveal_focus(&response);
        self.focus(&response, side);
    }

    fn render_notice(&mut self, ui: &mut egui::Ui, l: StudioShellLocale) {
        let Some(notice) = &self.notice else { return };
        let code = notice.context().diagnostic_code().unwrap_or_default();
        let message = match code {
            "equipment.h3.flow.source_changed" => tr(
                l,
                "源工程已变化。保留草稿，请重新载入并核对流股。",
                "The source changed. Reload the project and check the draft streams.",
            ),
            "store.project.save_target_changed"
            | "equipment.h3.flow.workspace_save_target"
            | "equipment.h3.flow.copy_target_exists" => tr(
                l,
                "目标不可写，请另选保存位置。草稿已保留。",
                "Choose another save destination. Your draft is intact.",
            ),
            _ => notice.message(),
        };
        ui.add(egui::Label::new(message).wrap());
        if !code.is_empty() {
            ui.small(code);
        }
        let hot = self
            .draft
            .hot
            .as_ref()
            .is_some_and(|stream| notice.context().related_stream_ids().contains(&stream.id));
        let cold = self
            .draft
            .cold
            .as_ref()
            .is_some_and(|stream| notice.context().related_stream_ids().contains(&stream.id));
        for (show, side, label) in [
            (
                hot,
                Focus::Hot,
                tr(l, "查看热侧流股", "Locate hot-side stream"),
            ),
            (
                cold,
                Focus::Cold,
                tr(l, "查看冷侧流股", "Locate cold-side stream"),
            ),
        ] {
            if show && h3::button(ui, label).clicked() {
                self.focus = Some(side);
            }
        }
    }

    fn request_copy(&mut self, native_picker: bool) -> EditorAction {
        if !self.can_save() {
            return EditorAction::Stay;
        }
        self.save_focus = Focus::SaveAs;
        if native_picker {
            EditorAction::ChooseCopy
        } else {
            self.dialog = Some(Dialog::Destination);
            self.focus = Some(Focus::Dialog);
            EditorAction::Stay
        }
    }

    fn render_dialog(
        &mut self,
        ctx: &egui::Context,
        l: StudioShellLocale,
        retained: Option<&Path>,
    ) -> EditorAction {
        let Some(dialog) = self.dialog.take() else {
            return EditorAction::Stay;
        };
        let (mut cancel, mut accept) = (false, false);
        let response = egui::Modal::new(egui::Id::new(("h3-association-confirm", self.id))).show(ctx, |ui| {
            ui.set_width((ctx.screen_rect().width() - 36.0).clamp(220.0, 460.0));
            let (cancel_label, accept_label) = match &dialog {
                Dialog::Save => {
                    ui.heading(tr(l, "确认保存关联", "Confirm association save"));
                    let prepared = self.application.prepared().expect("prepared association");
                    ui.add(egui::Label::new(prepared.path().display().to_string()).wrap());
                    if prepared.requires_upgrade() {
                        ui.label(tr(l, "首次保存将增加 H3 流程关联能力；旧读者将整份工程以只读打开。",
                            "This adds the H3 flow association capability. Older readers open the whole project as read-only."));
                    }
                    ui.label(format!("{} · {}\n{} · {}",
                        tr(l, "热侧", "Hot side"), self.stream_title(self.draft.hot.as_ref(), l),
                        tr(l, "冷侧", "Cold side"), self.stream_title(self.draft.cold.as_ref(), l)));
                    (tr(l, "取消", "Cancel"), tr(l, "确认保存", "Confirm save"))
                }
                Dialog::Discard(_) => {
                    ui.heading(tr(l, "舍弃关联草稿？", "Discard the association draft?"));
                    ui.label(tr(l, "已保存的入口来源仍有效。", "The saved inlet sources are retained."));
                    (tr(l, "继续编辑", "Keep editing"), tr(l, "舍弃草稿", "Discard draft"))
                }
                Dialog::Destination => {
                    ui.heading(tr(l, "选择新的工程文件", "Choose a new project file"));
                    ui.text_edit_singleline(&mut self.destination);
                    (tr(l, "取消", "Cancel"), tr(l, "预览保存目标", "Review save destination"))
                }
            };
            ui.horizontal_wrapped(|ui| {
                let keep = h3::button(ui, cancel_label);
                self.focus(&keep, Focus::Dialog);
                cancel = keep.clicked();
                accept = h3::button(ui, accept_label).clicked();
            });
        });
        if cancel || response.should_close() {
            match dialog {
                Dialog::Save => {
                    self.application.cancel_save();
                    self.focus = Some(self.save_focus);
                }
                Dialog::Discard(_) => self.focus = Some(Focus::Return),
                Dialog::Destination => self.focus = Some(Focus::SaveAs),
            }
        } else if accept {
            match dialog {
                Dialog::Discard(intent) => return self.apply_leave(intent),
                Dialog::Destination => {
                    let path = PathBuf::from(self.destination.trim());
                    self.prepare(H3AssociationSaveTarget::NewFile(&path), retained);
                }
                Dialog::Save if self.target_in_workspace(retained) => {
                    self.refuse_workspace_target()
                }
                Dialog::Save => match self.application.save() {
                    Ok(path) => {
                        self.saved_path = Some(path.clone());
                        return EditorAction::OpenSaved(path, self.member.clone());
                    }
                    Err(error) => {
                        self.error(error);
                        self.focus = Some(self.save_focus);
                    }
                },
            }
        } else {
            self.dialog = Some(dialog);
        }
        EditorAction::Stay
    }
}

fn is_source_changed(error: &RfError) -> bool {
    error.context().diagnostic_code() == Some("equipment.h3.flow.source_changed")
}
