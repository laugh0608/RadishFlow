//! Independent equipment page; all mutations go through the application editor.
use super::*;
use rf_types::units::{DisplayUnitSet, QuantityKind};
use rf_ui::engineering::{EngineeringInputs, editor::*};
mod adoption;
mod adoption_evidence;
mod adoption_preview;
mod applicability;
mod binding;
mod creation;
mod fields;
mod rating;
mod rating_inputs;
mod rating_labels;
mod rating_matrix;
mod rating_results;
pub(super) use creation::EquipmentTaskMode;
mod calculation_table;
mod confirmation;
mod export;
mod labels;
mod navigation;
mod results;
mod source;
mod task;
mod widgets;
pub(super) use calculation_table::CalculationTableFormat;
use export::EquipmentExportTarget;
use fields::EquipmentInputField;
pub(super) use labels::target_label;
use labels::*;
use widgets::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(super) enum EquipmentSection {
    #[default]
    Inputs,
    Results,
    Evidence,
}
#[derive(Default)]
pub(super) struct EquipmentPageState {
    pub(super) adoption: Option<adoption::AdoptionForm>,
    pub(super) adoption_evidence: Option<(rf_types::EquipmentTaskId, Option<egui::Id>)>,
    adoption_evidence_first: bool,
    adoption_return_focus: Option<egui::Id>,
    pub(super) pending_export: Option<export::PendingEquipmentExport>,
    pub(super) section: EquipmentSection,
    source_return: Option<(EquipmentEditorTarget, Option<EquipmentInputField>)>,
    focus_field: Option<EquipmentInputField>,
    focus_source: bool,
    pub(super) new_project: bool,
    pub(super) creating_task: bool,
    task_mode: EquipmentTaskMode,
    rating_scroll_top: bool,
    source_readonly: bool,
    pub(super) rating_selection:
        std::collections::BTreeMap<rf_types::EquipmentTaskId, rf_types::EngineeringCaseId>,
    pub(super) rating_overview: std::collections::BTreeMap<rf_types::EquipmentTaskId, bool>,
    rating_sections:
        std::collections::BTreeMap<rf_ui::engineering::EquipmentCaseKey, EquipmentSection>,
    rating_focus: std::collections::BTreeMap<EquipmentEditorTarget, egui::Id>,
    last_rating_target: Option<EquipmentEditorTarget>,
    restore_rating_focus: Option<EquipmentEditorTarget>,
}
impl ReadyAppState {
    pub(super) fn equipment_command(&mut self, command: EquipmentEditorCommand) {
        let reset_section = matches!(
            command,
            EquipmentEditorCommand::CreateTask | EquipmentEditorCommand::CreateRatingTask
        );
        let select_task = matches!(
            command,
            EquipmentEditorCommand::Select(
                EquipmentEditorTarget::Task(_)
                    | EquipmentEditorTarget::RatingTask(_)
                    | EquipmentEditorTarget::RatingGeometry(_)
                    | EquipmentEditorTarget::RatingCase(_)
            )
        );
        let selecting_case = matches!(
            command,
            EquipmentEditorCommand::Select(EquipmentEditorTarget::RatingCase(_))
        );
        let focus_source = matches!(
            command,
            EquipmentEditorCommand::CreateSource
                | EquipmentEditorCommand::Select(EquipmentEditorTarget::Source(_))
        );
        let origin = self
            .platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .active()
            .cloned();
        if let Some(EquipmentEditorTarget::RatingCase(key)) = &origin {
            self.equipment
                .rating_sections
                .insert(key.clone(), self.equipment.section);
        }
        let create_related = matches!(
            command,
            EquipmentEditorCommand::CreateSource | EquipmentEditorCommand::NewBasis
        );
        let Some(window_id) = self.current_window_id() else {
            return;
        };
        match self.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::Equipment(command),
        }) {
            Ok(_) => {
                self.sync_rating_selection(origin.as_ref(), selecting_case);
                if select_task {
                    self.equipment.source_return = None;
                    self.equipment.source_readonly = false;
                }
                if reset_section {
                    self.equipment.section = EquipmentSection::Inputs;
                    self.equipment.source_return = None;
                    self.equipment.source_readonly = false;
                    self.equipment.focus_field = None;
                }
                if focus_source {
                    self.equipment.focus_source = true;
                }
                if create_related
                    && let Some(target) = origin.filter(|t| {
                        matches!(t, EquipmentEditorTarget::Task(_)) || rating::task_id(t).is_some()
                    })
                {
                    self.equipment.source_readonly = false;
                    self.equipment.source_return = Some((target, None));
                }
            }
            Err(error) => self
                .platform_host
                .record_activity_line(error.message().to_owned()),
        }
    }
    pub(in crate::studio_gui_shell) fn calculate_active_equipment(&mut self) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let command = match workspace.equipment_editor.active() {
            Some(EquipmentEditorTarget::Task(id)) => {
                Some(EquipmentEditorCommand::Calculate(id.clone()))
            }
            Some(target) => rating::task_id(target)
                .and_then(|id| workspace.engineering.inputs().rating_task(id))
                .and_then(|task| self.selected_rating_member(task, Some(target)))
                .map(EquipmentEditorCommand::CalculateRating),
            None => None,
        };
        if let Some(command) = command {
            if let EquipmentEditorCommand::CalculateRating(key) = &command {
                self.equipment
                    .rating_overview
                    .insert(key.task_id.clone(), false);
            }
            self.equipment_command(command);
            self.equipment.section = EquipmentSection::Results;
        }
    }
    pub(super) fn enter_equipment(&mut self) {
        self.screen = StudioShellScreen::Equipment;
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if workspace.equipment_editor.active().is_none()
            && let Some(task) = workspace.engineering.inputs().tasks.first()
        {
            let target = if task.as_sizing().is_some() {
                EquipmentEditorTarget::Task(task.id().clone())
            } else {
                EquipmentEditorTarget::RatingTask(task.id().clone())
            };
            self.equipment_command(EquipmentEditorCommand::Select(target));
        }
    }
    pub(super) fn return_from_equipment_source(&mut self) -> bool {
        let Some((target, field)) = self.equipment.source_return.take() else {
            return false;
        };
        self.equipment.source_readonly = false;
        self.equipment.focus_field = field;
        self.equipment.section = EquipmentSection::Inputs;
        self.equipment_command(EquipmentEditorCommand::Select(target));
        true
    }
    pub(super) fn render_equipment_page(&mut self, ctx: &egui::Context) {
        // Restore after the closing modal's last layer has stopped owning input.
        let return_focus = self.equipment.adoption_return_focus.take();
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let session = workspace.engineering.clone();
        let editor = workspace.equipment_editor.clone();
        let units = workspace.project_presentation.display_units().clone();
        let inputs = session.inputs();
        let locale = self.locale;
        let font_scale = ctx.style().text_styles[&egui::TextStyle::Body].size / 14.0;
        let sidebar = ctx.available_rect().width() >= 1000.0 * font_scale;
        let mut commands = Vec::new();
        let mut export_request = None;
        let mut open_source = None;
        let mut adoption_request = None;
        if sidebar {
            egui::SidePanel::left("equipment-project-navigation")
                .default_width(210.0 * font_scale)
                .resizable(true)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        navigation::render(ui, locale, inputs, &editor, &mut commands)
                    });
                });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(tr(
                locale,
                "设备设计 · 独立工程任务",
                "Equipment design · independent tasks",
            ));
            let compact = !sidebar
                && editor
                    .active()
                    .is_some_and(|t| rating::task_id(t).is_some());
            let mut toolbar = |ui: &mut egui::Ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(tr(locale, "新建设备", "New equipment")).clicked() {
                        self.equipment.creating_task = true;
                    }
                    for (label, command, enabled) in [
                        (
                            tr(locale, "新建来源", "New source"),
                            EquipmentEditorCommand::CreateSource,
                            true,
                        ),
                        (
                            tr(locale, "新建基础条目", "New basis entry"),
                            EquipmentEditorCommand::NewBasis,
                            true,
                        ),
                        (
                            tr(locale, "撤销设备修改", "Undo equipment change"),
                            EquipmentEditorCommand::History { redo: false },
                            session.can_undo(),
                        ),
                        (
                            tr(locale, "重做设备修改", "Redo equipment change"),
                            EquipmentEditorCommand::History { redo: true },
                            session.can_redo(),
                        ),
                    ] {
                        if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                            commands.push(command);
                        }
                    }
                    if ui.button(tr(locale, "保存工程", "Save project")).clicked() {
                        self.save_project();
                    }
                });
            };
            if compact {
                egui::CollapsingHeader::new(tr(
                    locale,
                    "任务与工程操作",
                    "Task and project actions",
                ))
                .show(ui, toolbar);
            } else {
                toolbar(ui);
            }
            if !sidebar {
                egui::CollapsingHeader::new(tr(locale, "设备任务与工程基础", "Tasks and basis"))
                    .default_open(editor.active().is_none())
                    .show(ui, |ui| {
                        navigation::render(ui, locale, inputs, &editor, &mut commands);
                    });
            }
            let Some(target) = editor.active().cloned() else {
                ui.label(tr(
                    locale,
                    "新建设备任务，逐项填写输入与来源。",
                    "Create an equipment task, then enter inputs and sources.",
                ));
                return;
            };
            if rating::task_id(&target).is_some() {
                self.render_equipment_rating(
                    ui,
                    &target,
                    &editor,
                    &session,
                    &units,
                    &mut commands,
                    &mut export_request,
                );
                return;
            }
            if self.equipment.source_readonly && matches!(target, EquipmentEditorTarget::Source(_))
            {
                self.render_equipment_source_snapshot(ui, &target, inputs, &units, &mut commands);
                return;
            }
            let Some(record) = editor.draft(&target) else {
                return;
            };
            let mut draft = record.value().clone();
            ui.separator();
            ui.heading(target_label(locale, &target, inputs));
            if let EquipmentEditorTarget::Task(id) = &target
                && let Some(task) = inputs.task(id)
            {
                ui.label(format!(
                    "{} · {} v{} · {}",
                    nonempty(&task.case.name, tr(locale, "未命名工况", "Untitled case")),
                    tr(
                        locale,
                        "Souders–Brown 与工作容积",
                        "Souders–Brown and working volume"
                    ),
                    task.method.version,
                    status_label(locale, session.result_status(id))
                ));
            }
            ui.horizontal_wrapped(|ui| {
                for (label, command, enabled) in [
                    (
                        tr(locale, "应用输入", "Apply inputs"),
                        EquipmentEditorCommand::Apply(target.clone()),
                        record.is_pending(),
                    ),
                    (
                        tr(locale, "取消编辑", "Discard edits"),
                        EquipmentEditorCommand::Discard(target.clone()),
                        record.is_pending(),
                    ),
                    (
                        tr(locale, "删除记录", "Delete record"),
                        EquipmentEditorCommand::Delete(target.clone()),
                        match &target {
                            EquipmentEditorTarget::Task(id) => inputs.task(id).is_some(),
                            EquipmentEditorTarget::RatingTask(_)
                            | EquipmentEditorTarget::RatingGeometry(_)
                            | EquipmentEditorTarget::RatingCase(_) => false,
                            EquipmentEditorTarget::Source(id) => {
                                inputs.basis.sources.iter().any(|s| s.id == *id)
                            }
                            EquipmentEditorTarget::Basis(id) => {
                                inputs.basis.basis_entries.iter().any(|b| b.id == *id)
                            }
                        },
                    ),
                ] {
                    if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                        commands.push(command);
                    }
                }
                if let EquipmentEditorTarget::Task(id) = &target {
                    if ui.button(tr(locale, "检查输入", "Check inputs")).clicked() {
                        commands.push(EquipmentEditorCommand::Check(id.clone()));
                    }
                    if ui
                        .add_enabled(
                            !self.background_busy(),
                            egui::Button::new(tr(locale, "计算尺寸", "Calculate size")),
                        )
                        .clicked()
                    {
                        commands.push(EquipmentEditorCommand::Calculate(id.clone()));
                        self.equipment.section = EquipmentSection::Results;
                    }
                }
                if self.equipment.source_return.is_some()
                    && ui
                        .button(tr(locale, "返回输入字段", "Return to input field"))
                        .clicked()
                {
                    // Dispatch after the current frame's draft change, preserving any source edits.
                    if let Some((return_to, field)) = self.equipment.source_return.take() {
                        commands.push(EquipmentEditorCommand::Select(return_to));
                        self.equipment.focus_field = field;
                        self.equipment.section = EquipmentSection::Inputs;
                    }
                }
            });
            if let EquipmentEditorTarget::Task(id) = &target {
                let enabled = !self.equipment_target_busy(id, None)
                    && session.current_candidate(id).is_some()
                    && !editor.pending_for_task(id, &session);
                ui.horizontal_wrapped(|ui| {
                    let adopt = ui.add_enabled(
                        enabled,
                        egui::Button::new(tr(locale, "采纳为设计几何", "Adopt as design geometry")),
                    );
                    if adopt.clicked() {
                        adoption_request = Some((id.clone(), adopt.id));
                    }
                    for (format, label) in [
                        (
                            CalculationTableFormat::Csv,
                            tr(locale, "导出 CSV", "Export CSV"),
                        ),
                        (
                            CalculationTableFormat::Json,
                            tr(locale, "导出 JSON", "Export JSON"),
                        ),
                    ] {
                        if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                            export_request =
                                Some((EquipmentExportTarget::Task(id.clone()), format));
                        }
                    }
                    ui.label(tr(locale, "计算表使用项目单位", "Table uses project units"));
                });
                if !enabled {
                    ui.label(tr(
                        locale,
                        "处理相关草稿并完成当前计算后可导出。",
                        "Resolve related drafts and calculate before exporting.",
                    ));
                }
            }
            if let Some(notice) = &self.project_open.notice {
                render_project_notice(ui, notice);
            }
            if record.is_pending() {
                ui.label(tr(
                    locale,
                    "有未提交输入；保存不会自动应用，计算前请先应用。",
                    "Unsubmitted inputs: saving does not apply them. Apply before calculating.",
                ));
            }
            let split = matches!(target, EquipmentEditorTarget::Task(_))
                && ui.available_width() >= 900.0 * font_scale;
            if matches!(target, EquipmentEditorTarget::Task(_)) {
                ui.horizontal_wrapped(|ui| {
                    for (section, label) in [
                        (EquipmentSection::Inputs, tr(locale, "输入", "Inputs")),
                        (EquipmentSection::Results, tr(locale, "结果", "Results")),
                        (EquipmentSection::Evidence, tr(locale, "依据", "Evidence")),
                    ] {
                        ui.selectable_value(&mut self.equipment.section, section, label);
                    }
                });
            }
            if split {
                ui.columns(2, |columns| {
                    self.equipment_editor_body(
                        &mut columns[0],
                        locale,
                        &target,
                        &mut draft,
                        &editor,
                        &session,
                        &units,
                        false,
                        &mut open_source,
                    );
                    let evidence = self.equipment.section == EquipmentSection::Evidence;
                    if let EquipmentEditorTarget::Task(id) = &target {
                        egui::ScrollArea::vertical()
                            .id_salt(("equipment-result", id, evidence))
                            .show(&mut columns[1], |ui| {
                                if let Some(target) =
                                    results::render(ui, locale, id, &session, &units, evidence)
                                {
                                    self.equipment_issue_target(target, &mut open_source);
                                }
                            });
                    }
                });
            } else {
                self.equipment_editor_body(
                    ui,
                    locale,
                    &target,
                    &mut draft,
                    &editor,
                    &session,
                    &units,
                    true,
                    &mut open_source,
                );
            }
            if &draft != record.value() {
                commands.insert(
                    0,
                    EquipmentEditorCommand::Change {
                        target: target.clone(),
                        generation: record.generation(),
                        value: Box::new(draft),
                    },
                );
            }
            if let Some((id, field)) = open_source.take() {
                self.equipment.source_return = Some((target, field));
                commands.push(EquipmentEditorCommand::Select(
                    EquipmentEditorTarget::Source(id),
                ));
            }
        });
        for command in commands {
            self.equipment_command(command);
        }
        if let Some((id, focus)) = adoption_request {
            self.begin_geometry_adoption(&id);
            if let Some(form) = &mut self.equipment.adoption {
                form.return_focus = Some(focus);
            }
        }
        if let Some((target, format)) = export_request {
            if let EquipmentExportTarget::Task(id) = target {
                self.export_equipment_from_picker(&id, format);
            } else {
                self.export_target_from_picker(&target, format);
            }
        }
        self.render_equipment_confirmation(ctx);
        self.render_equipment_creation(ctx);
        self.render_geometry_adoption(ctx);
        self.render_adoption_evidence(ctx);
        if let Some(id) = return_focus {
            ctx.memory_mut(|m| m.request_focus(id));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn equipment_editor_body(
        &mut self,
        ui: &mut egui::Ui,
        locale: StudioShellLocale,
        target: &EquipmentEditorTarget,
        draft: &mut EquipmentDraft,
        editor: &EquipmentEditorState,
        session: &rf_ui::engineering::EngineeringSession,
        units: &DisplayUnitSet,
        allow_results: bool,
        open_source: &mut Option<(rf_types::SourceId, Option<EquipmentInputField>)>,
    ) {
        let section = if allow_results {
            self.equipment.section
        } else {
            EquipmentSection::Inputs
        };
        egui::ScrollArea::vertical()
            .id_salt(("equipment-editor", target, section))
            .show(ui, |ui| {
                if let Some(error) = editor.error() {
                    ui.colored_label(
                        egui::Color32::from_rgb(160, 65, 30),
                        edit_error(locale, error),
                    );
                    source::render_references(ui, locale, error, session.inputs());
                    if let EquipmentEditError::Field(field, _) = error
                        && ui.button(tr(locale, "定位输入", "Go to input")).clicked()
                    {
                        self.equipment.focus_field = Some((*field).into());
                        self.equipment.section = EquipmentSection::Inputs;
                    }
                }
                if let EquipmentEditorTarget::Task(id) = target {
                    if let Some(issues) = editor.checked_issues(id, session) {
                        if issues.is_empty() {
                            ui.label(tr(
                                locale,
                                "输入检查通过（本操作未触发计算）。",
                                "Input check passed (this action did not calculate).",
                            ));
                        }
                        for issue in issues {
                            self.equipment_issue(ui, locale, issue);
                        }
                    }
                    if section != EquipmentSection::Inputs {
                        if let Some(target) = results::render(
                            ui,
                            locale,
                            id,
                            session,
                            units,
                            section == EquipmentSection::Evidence,
                        ) {
                            self.equipment_issue_target(target, open_source);
                        }
                        return;
                    }
                }
                ui.push_id(target, |ui| match draft {
                    EquipmentDraft::Task(draft) => task::render(
                        ui,
                        locale,
                        draft,
                        session.inputs(),
                        units,
                        self.equipment.focus_field.take(),
                        open_source,
                    ),
                    EquipmentDraft::RatingTask(_)
                    | EquipmentDraft::RatingGeometry(_)
                    | EquipmentDraft::RatingCase(_) => {
                        unreachable!("rating drafts are rendered by the dedicated rating page");
                    }
                    EquipmentDraft::Source(draft) => {
                        let focus = std::mem::take(&mut self.equipment.focus_source);
                        source::render(ui, locale, draft, session.inputs(), units, focus);
                    }
                    EquipmentDraft::Basis(draft) => {
                        source::render_basis(ui, locale, draft, session.inputs(), units)
                    }
                });
            });
    }
    fn equipment_issue_target(
        &mut self,
        target: separator_sizing::IssueTarget,
        open_source: &mut Option<(rf_types::SourceId, Option<EquipmentInputField>)>,
    ) {
        use separator_sizing::IssueTarget;
        self.equipment.section = EquipmentSection::Inputs;
        match target {
            IssueTarget::Input(field) => {
                self.equipment.focus_field = Some(EquipmentField::Input(field).into())
            }
            IssueTarget::AxialSection(field) => {
                self.equipment.focus_field = Some(EquipmentField::Axial(field).into())
            }
            IssueTarget::Source(id) => *open_source = Some((id, None)),
            _ => {}
        }
    }
    fn equipment_issue(
        &mut self,
        ui: &mut egui::Ui,
        locale: StudioShellLocale,
        issue: &separator_sizing::SizingIssue,
    ) {
        use separator_sizing::IssueTarget;
        let field: Option<EquipmentInputField> = match issue.target {
            IssueTarget::Input(field) => Some(EquipmentField::Input(field).into()),
            IssueTarget::AxialSection(field) => Some(EquipmentField::Axial(field).into()),
            _ => None,
        };
        if let Some(field) = field {
            if ui.link(issue_label(locale, issue)).clicked() {
                self.equipment.focus_field = Some(field);
                self.equipment.section = EquipmentSection::Inputs;
            }
        } else {
            ui.label(issue_label(locale, issue));
        }
    }
}
