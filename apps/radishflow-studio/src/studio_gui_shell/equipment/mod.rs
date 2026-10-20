//! Independent equipment page; all mutations go through the application editor.
use super::*;
use rf_types::units::{DisplayUnitSet, QuantityKind};
use rf_ui::engineering::{EngineeringInputs, editor::*};
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
    pub(super) pending_export: Option<export::PendingEquipmentExport>,
    pub(super) section: EquipmentSection,
    source_return: Option<(EquipmentEditorTarget, Option<EquipmentField>)>,
    focus_field: Option<EquipmentField>,
    focus_source: bool,
    pub(super) new_project: bool,
}
impl ReadyAppState {
    pub(super) fn equipment_command(&mut self, command: EquipmentEditorCommand) {
        let reset_section = matches!(command, EquipmentEditorCommand::CreateTask);
        let select_task = matches!(
            command,
            EquipmentEditorCommand::Select(EquipmentEditorTarget::Task(_))
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
                if select_task {
                    self.equipment.source_return = None;
                }
                if reset_section {
                    self.equipment.section = EquipmentSection::Inputs;
                }
                if focus_source {
                    self.equipment.focus_source = true;
                }
                if create_related && let Some(target @ EquipmentEditorTarget::Task(_)) = origin {
                    self.equipment.source_return = Some((target, None));
                }
            }
            Err(error) => self
                .platform_host
                .record_activity_line(error.message().to_owned()),
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
        self.equipment.focus_field = field;
        self.equipment.section = EquipmentSection::Inputs;
        self.equipment_command(EquipmentEditorCommand::Select(target));
        true
    }
    pub(super) fn render_equipment_page(&mut self, ctx: &egui::Context) {
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
                "设备设计 · 独立单工况",
                "Equipment design · independent single case",
            ));
            ui.horizontal_wrapped(|ui| {
                for (label, command, enabled) in [
                    (
                        tr(locale, "新建设备", "New equipment"),
                        EquipmentEditorCommand::CreateTask,
                        true,
                    ),
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
            let Some(record) = editor.draft(&target) else {
                return;
            };
            let mut draft = record.value().clone();
            ui.separator();
            ui.heading(target_label(locale, &target, inputs));
            if matches!(target, EquipmentEditorTarget::RatingTask(_) | EquipmentEditorTarget::RatingGeometry(_) | EquipmentEditorTarget::RatingCase(_)) {
                ui.label(tr(locale,
                    "校核任务已载入，将随工程完整保存。此版本暂不提供校核编辑与结果视图。",
                    "The rating task is loaded and will be preserved when saving. Rating editing and result views are not available in this version."));
                return;
            }
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
                            EquipmentEditorTarget::RatingTask(_) | EquipmentEditorTarget::RatingGeometry(_) | EquipmentEditorTarget::RatingCase(_) => false,
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
                        .button(tr(locale, "计算尺寸", "Calculate size"))
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
                let enabled = session.current_candidate(id).is_some()
                    && !editor.pending_for_task(id, &session);
                ui.horizontal_wrapped(|ui| {
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
                            export_request = Some((id.clone(), format));
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
        if let Some((id, format)) = export_request {
            self.export_equipment_from_picker(&id, format);
        }
        self.render_equipment_confirmation(ctx);
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
        open_source: &mut Option<(rf_types::SourceId, Option<EquipmentField>)>,
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
                        self.equipment.focus_field = Some(*field);
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
                        ui.label(tr(
                            locale,
                            "校核编辑暂不可用。",
                            "Rating editing is not available.",
                        ));
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
        open_source: &mut Option<(rf_types::SourceId, Option<EquipmentField>)>,
    ) {
        use separator_sizing::IssueTarget;
        self.equipment.section = EquipmentSection::Inputs;
        match target {
            IssueTarget::Input(field) => {
                self.equipment.focus_field = Some(EquipmentField::Input(field))
            }
            IssueTarget::AxialSection(field) => {
                self.equipment.focus_field = Some(EquipmentField::Axial(field))
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
        let field = match issue.target {
            IssueTarget::Input(field) => Some(EquipmentField::Input(field)),
            IssueTarget::AxialSection(field) => Some(EquipmentField::Axial(field)),
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
