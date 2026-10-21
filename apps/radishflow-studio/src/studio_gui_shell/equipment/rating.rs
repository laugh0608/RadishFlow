//! Rating page consumes the application editor and row-qualified result only.
use super::*;
use rf_ui::engineering::editor::separator_rating::RatingIssueTarget;
use rf_ui::engineering::{EngineeringSession, EquipmentCaseKey};
pub(super) fn task_id(target: &EquipmentEditorTarget) -> Option<&rf_types::EquipmentTaskId> {
    match target {
        EquipmentEditorTarget::RatingTask(id) | EquipmentEditorTarget::RatingGeometry(id) => {
            Some(id)
        }
        EquipmentEditorTarget::RatingCase(key) => Some(&key.task_id),
        _ => None,
    }
}
impl ReadyAppState {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_equipment_rating(
        &mut self,
        ui: &mut egui::Ui,
        target: &EquipmentEditorTarget,
        editor: &EquipmentEditorState,
        session: &EngineeringSession,
        units: &DisplayUnitSet,
        commands: &mut Vec<EquipmentEditorCommand>,
        export: &mut Option<(rf_types::EquipmentTaskId, CalculationTableFormat)>,
    ) {
        let l = self.locale;
        let Some(task) = task_id(target).and_then(|id| session.inputs().rating_task(id)) else {
            return;
        };
        let [case] = task.cases.as_slice() else {
            ui.label(tr(
                l,
                "此方法只支持一个校核工况。",
                "This method supports exactly one rating case.",
            ));
            return;
        };
        let key = EquipmentCaseKey {
            task_id: task.id.clone(),
            case_id: case.id.clone(),
        };
        let Some(record) = editor.draft(target) else {
            return;
        };
        let mut draft = record.value().clone();
        let mut open_source = None;
        let mut requested_issue = None;
        let pending = editor.pending_for_case(&key, session);
        let focus = self.equipment.focus_field.take();
        let scroll_top = std::mem::take(&mut self.equipment.rating_scroll_top);
        let mut scroll =
            egui::ScrollArea::vertical().id_salt(("rating-page", target, self.equipment.section));
        if scroll_top {
            scroll = scroll.vertical_scroll_offset(0.0);
        }
        scroll.show(ui, |ui| {
            ui.heading(format!("{} · {}", task_label(l, &task.clone().into()), tr(l, "已有几何校核", "Given-geometry rating",)));
            ui.label(format!(
                "{} · {}",
                nonempty(&case.name, tr(l, "未命名工况", "Untitled case",)),
                rating_labels::status_label(l, session.case_result_status(&key))
            ));
            if let Some(notice) = &self.project_open.notice {
                render_project_notice(ui, notice);
            }
            if pending {
                ui.label(tr(
                    l,
                    "有相关未提交输入。检查、校核与导出已停用；保存只写正式输入。",
                    "Related inputs are unsubmitted. Checking, rating and export are disabled; saving writes committed inputs only.",
                ));
            }
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(!pending, egui::Button::new(tr(l, "检查输入", "Check inputs"))).clicked() {
                    commands.push(EquipmentEditorCommand::CheckRating(key.clone()));
                }
                if ui.add_enabled(!pending, egui::Button::new(tr(l, "校核当前工况", "Rate current case"))).clicked() {
                    commands.push(EquipmentEditorCommand::CalculateRating(key.clone()));
                    self.equipment.section = EquipmentSection::Results;
                }
                for (format, label) in [
                    (CalculationTableFormat::Csv, tr(l, "导出 CSV", "Export CSV")),
                    (CalculationTableFormat::Json, tr(l, "导出 JSON", "Export JSON")),
                ] {
                    if ui.add_enabled(!pending && session.current_rating(&key).is_some(), egui::Button::new(label)).clicked() {
                        *export = Some((task.id.clone(), format));
                    }
                }
            });
            ui.small(tr(
                l,
                "校核表使用项目单位；超限与边界结果可导出。",
                "Rating tables use project units; exceeded and boundary results can be exported.",
            ));
            if session.current_rating(&key).is_none() {
                ui.label(tr(
                    l,
                    "完成当前校核后可导出；过期、失败与尚未校核均不可导出。",
                    "Export requires a current rating; stale, failed or missing results cannot be exported.",
                ));
            }
            if let Some(error) = editor.error() {
                ui.colored_label(egui::Color32::from_rgb(160, 65, 30), edit_error(l, error));
                source::render_references(ui, l, error, session.inputs());
                let issue = match error {
                    EquipmentEditError::RatingGeometryField(RatingGeometryField::Input(field), _) => Some(RatingIssueTarget::GeometryInput(*field)),
                    EquipmentEditError::RatingGeometryField(RatingGeometryField::Axial(field), _) => Some(RatingIssueTarget::AxialSection(*field)),
                    EquipmentEditError::RatingInput(field, _) => Some(RatingIssueTarget::Input { case_id: key.case_id.clone(), field: *field }),
                    _ => None,
                };
                if let Some(issue) = issue
                    && ui.button(tr(l, "定位输入", "Go to input")).clicked()
                {
                    requested_issue = Some(issue);
                }
            }
            if let Some(issues) = editor.checked_rating_issues(&key, session) {
                if issues.is_empty() {
                    ui.label(tr(l, "输入检查通过（未执行校核）。", "Input check passed (rating was not run)."));
                }
                for issue in issues {
                    if ui.link(rating_labels::issue_label(l, issue)).clicked() {
                        requested_issue = Some(issue.target.clone());
                    }
                }
            }
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                for (label, selection) in [
                    (tr(l, "给定几何", "Given geometry"), EquipmentEditorTarget::RatingGeometry(task.id.clone())),
                    (tr(l, "当前工况", "Current case"), EquipmentEditorTarget::RatingCase(key.clone())),
                    (tr(l, "位号与名称", "Tag and name"), EquipmentEditorTarget::RatingTask(task.id.clone())),
                ] {
                    if ui.selectable_label(target == &selection && self.equipment.section == EquipmentSection::Inputs, label).clicked() {
                        commands.push(EquipmentEditorCommand::Select(selection));
                        self.equipment.section = EquipmentSection::Inputs;
                    }
                }
                ui.selectable_value(&mut self.equipment.section, EquipmentSection::Results, tr(l, "校核结果", "Rating results"));
                ui.selectable_value(&mut self.equipment.section, EquipmentSection::Evidence, tr(l, "依据", "Evidence"));
            });
            if self.equipment.section != EquipmentSection::Inputs {
                requested_issue = rating_results::render(ui, l, &key, session, units, self.equipment.section == EquipmentSection::Evidence).or(requested_issue.take());
                return;
            }
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(record.is_pending(), egui::Button::new(tr(l, "应用输入", "Apply inputs"))).clicked() {
                    commands.push(EquipmentEditorCommand::Apply(target.clone()));
                }
                if ui.add_enabled(record.is_pending(), egui::Button::new(tr(l, "取消编辑", "Discard edits"))).clicked() {
                    commands.push(EquipmentEditorCommand::Discard(target.clone()));
                }
                if matches!(target, EquipmentEditorTarget::RatingTask(_)) && ui.button(tr(l, "删除校核任务", "Delete rating task")).clicked() {
                    commands.push(EquipmentEditorCommand::Delete(target.clone()));
                }
            });
            let split = ui.available_width() >= 900.0 * (ui.style().text_styles[&egui::TextStyle::Body].size / 14.0);
            let mut render_inputs = |ui: &mut egui::Ui| {
                ui.push_id(target, |ui| match &mut draft {
                    EquipmentDraft::RatingTask(d) => rating_inputs::labels(ui, l, d),
                    EquipmentDraft::RatingGeometry(d) => rating_inputs::geometry(ui, l, d, &task.geometry, session.inputs(), units, focus, &mut open_source),
                    EquipmentDraft::RatingCase(d) => rating_inputs::case(ui, l, d, case, session.inputs(), units, focus, &mut open_source),
                    _ => {}
                });
            };
            if split {
                ui.columns(2, |cols| {
                    render_inputs(&mut cols[0]);
                    requested_issue = rating_results::render(&mut cols[1], l, &key, session, units, false).or(requested_issue.take());
                });
            } else {
                render_inputs(ui);
            }
        });
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
        if let Some(issue) = requested_issue {
            self.rating_issue_target(issue, &key, commands, &mut open_source);
        }
        if let Some((id, field)) = open_source {
            self.equipment.source_return = Some((target.clone(), field));
            self.equipment.source_readonly = true;
            commands.push(EquipmentEditorCommand::Select(
                EquipmentEditorTarget::Source(id),
            ));
        }
    }
    pub(super) fn rating_issue_target(
        &mut self,
        issue: RatingIssueTarget,
        key: &EquipmentCaseKey,
        commands: &mut Vec<EquipmentEditorCommand>,
        open_source: &mut Option<(rf_types::SourceId, Option<EquipmentInputField>)>,
    ) {
        use RatingIssueTarget::*;
        self.equipment.section = EquipmentSection::Inputs;
        self.equipment.rating_scroll_top = true;
        let target = match issue {
            GeometryInput(field) => {
                self.equipment.focus_field = Some(EquipmentInputField::Geometry(
                    RatingGeometryField::Input(field),
                ));
                EquipmentEditorTarget::RatingGeometry(key.task_id.clone())
            }
            AxialSection(field) => {
                self.equipment.focus_field = Some(EquipmentInputField::Geometry(
                    RatingGeometryField::Axial(field),
                ));
                EquipmentEditorTarget::RatingGeometry(key.task_id.clone())
            }
            Input { case_id, field } => {
                self.equipment.focus_field = Some(EquipmentInputField::Rating(field));
                EquipmentEditorTarget::RatingCase(EquipmentCaseKey {
                    task_id: key.task_id.clone(),
                    case_id,
                })
            }
            Geometry | GeometryApplicability(_) => {
                EquipmentEditorTarget::RatingGeometry(key.task_id.clone())
            }
            Case(case_id) | CaseApplicability { case_id, .. } => {
                EquipmentEditorTarget::RatingCase(EquipmentCaseKey {
                    task_id: key.task_id.clone(),
                    case_id,
                })
            }
            Source(id) => {
                *open_source = Some((id, None));
                return;
            }
            Task | Method | Basis | Calculation => {
                EquipmentEditorTarget::RatingTask(key.task_id.clone())
            }
        };
        commands.push(EquipmentEditorCommand::Select(target));
    }
}
