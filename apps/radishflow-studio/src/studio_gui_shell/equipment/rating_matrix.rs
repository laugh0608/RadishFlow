//! Matrix selection and finite coverage, separate from the editable row and geometry.
use super::*;
use rf_model::equipment::{SeparatorGeometryInput, SeparatorRatingTask};
use rf_ui::engineering::{EngineeringSession, EquipmentCaseKey, RatingMatrixCapture};
use separator_rating::{ConstraintStatus, MatrixCoverage};

impl ReadyAppState {
    pub(super) fn selected_rating_member(
        &self,
        task: &SeparatorRatingTask,
        target: Option<&EquipmentEditorTarget>,
    ) -> Option<EquipmentCaseKey> {
        let selected = match target {
            Some(EquipmentEditorTarget::RatingCase(key)) if key.task_id == task.id => {
                Some(&key.case_id)
            }
            _ => self.equipment.rating_selection.get(&task.id),
        };
        let case = selected
            .and_then(|id| task.cases.iter().find(|c| c.id == *id))
            .or_else(|| task.cases.first())?;
        Some(EquipmentCaseKey {
            task_id: task.id.clone(),
            case_id: case.id.clone(),
        })
    }
    pub(super) fn sync_rating_selection(
        &mut self,
        origin: Option<&EquipmentEditorTarget>,
        selecting_case: bool,
    ) {
        let w = &self.platform_host.modeling_inputs().workspace;
        let active = w.equipment_editor.active().cloned();
        if let Some(EquipmentEditorTarget::RatingCase(key)) = &active {
            let previous = self
                .equipment
                .rating_selection
                .insert(key.task_id.clone(), key.case_id.clone());
            if previous.as_ref() != Some(&key.case_id) {
                self.equipment.section = self
                    .equipment
                    .rating_sections
                    .get(key)
                    .copied()
                    .unwrap_or_default();
            }
            if selecting_case || active != origin.cloned() {
                self.equipment
                    .rating_overview
                    .insert(key.task_id.clone(), false);
            }
        }
        if active != origin.cloned() {
            self.equipment.restore_rating_focus =
                active.clone().filter(|t| rating::task_id(t).is_some());
        }
        if active.is_none()
            && let Some(task) = origin
                .and_then(rating::task_id)
                .and_then(|id| w.engineering.inputs().rating_task(id))
            && let Some(key) = self.selected_rating_member(task, None)
        {
            self.equipment_command(EquipmentEditorCommand::Select(
                EquipmentEditorTarget::RatingCase(key),
            ));
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_rating_matrix(
        &mut self,
        ui: &mut egui::Ui,
        task: &SeparatorRatingTask,
        selected: &EquipmentCaseKey,
        editor: &EquipmentEditorState,
        session: &EngineeringSession,
        units: &DisplayUnitSet,
        commands: &mut Vec<EquipmentEditorCommand>,
        export: &mut Option<(EquipmentExportTarget, CalculationTableFormat)>,
        wide: bool,
    ) {
        let l = self.locale;
        ui.heading(tr(l, "工况矩阵", "Case matrix"));
        ui.label(tr(
            l,
            "全部工况共用给定几何；逐行手动校核。",
            "All cases share given geometry; rate each row explicitly.",
        ));
        ui.horizontal_wrapped(|ui| {
            for field in SeparatorGeometryInput::ALL {
                let text = task
                    .geometry
                    .inputs
                    .get(&field)
                    .and_then(|b| session.inputs().basis.resolve(field.quantity(), b).ok())
                    .map(|v| results::format_value(v.value.value_si, field.quantity(), units))
                    .unwrap_or_else(|| tr(l, "未填写", "Not entered").into());
                ui.label(format!("{}: {text}", geometry_field_label(l, field)));
            }
        });
        let selected_index = task
            .cases
            .iter()
            .position(|c| c.id == selected.case_id)
            .expect("selected member");
        ui.horizontal_wrapped(|ui| {
            if ui.button(tr(l, "新增工况", "Add case")).clicked() {
                commands.push(EquipmentEditorCommand::AddRatingCase {
                    task: task.id.clone(),
                    copy_from: None,
                });
            }
            if ui
                .add_enabled(
                    !editor.pending_for_case(selected, session),
                    egui::Button::new(tr(l, "复制所选工况", "Copy selected case")),
                )
                .clicked()
            {
                commands.push(EquipmentEditorCommand::AddRatingCase {
                    task: task.id.clone(),
                    copy_from: Some(selected.case_id.clone()),
                });
            }
            for (label, allowed, destination) in [
                (
                    tr(l, "上移", "Move up"),
                    selected_index > 0,
                    selected_index.saturating_sub(1),
                ),
                (
                    tr(l, "下移", "Move down"),
                    selected_index + 1 < task.cases.len(),
                    selected_index + 1,
                ),
            ] {
                if ui.add_enabled(allowed, egui::Button::new(label)).clicked() {
                    let mut cases = task.cases.iter().map(|c| c.id.clone()).collect::<Vec<_>>();
                    cases.swap(selected_index, destination);
                    commands.push(EquipmentEditorCommand::ReorderRatingCases {
                        task: task.id.clone(),
                        cases,
                    });
                }
            }
            if ui
                .add_enabled(
                    task.cases.len() > 1,
                    egui::Button::new(tr(l, "删除所选工况", "Delete selected case")),
                )
                .on_disabled_hover_text(tr(l, "至少保留一个工况。", "Keep at least one case."))
                .clicked()
            {
                commands.push(EquipmentEditorCommand::Delete(
                    EquipmentEditorTarget::RatingCase(selected.clone()),
                ));
            }
        });
        if editor.pending_for_case(selected, session) {
            ui.small(tr(
                l,
                "所选工况有相关草稿；应用或取消后可复制。",
                "Resolve the selected case's related drafts before copying.",
            ));
        }
        ui.horizontal_wrapped(|ui| {
            for (format, label) in [
                (CalculationTableFormat::Csv, tr(l, "矩阵 CSV", "Matrix CSV")),
                (
                    CalculationTableFormat::Json,
                    tr(l, "矩阵 JSON", "Matrix JSON"),
                ),
            ] {
                if ui
                    .add_enabled(
                        !editor.pending_for_task(&task.id, session),
                        egui::Button::new(label),
                    )
                    .clicked()
                {
                    *export = Some((EquipmentExportTarget::Matrix(task.id.clone()), format));
                }
            }
            if ui.button(tr(l, "给定几何", "Given geometry")).clicked() {
                self.equipment
                    .rating_overview
                    .insert(task.id.clone(), false);
                self.equipment.section = EquipmentSection::Inputs;
                commands.push(EquipmentEditorCommand::Select(
                    EquipmentEditorTarget::RatingGeometry(task.id.clone()),
                ));
            }
            if !wide
                && ui
                    .button(tr(l, "查看所选工况", "Open selected case"))
                    .clicked()
            {
                commands.push(EquipmentEditorCommand::Select(
                    EquipmentEditorTarget::RatingCase(selected.clone()),
                ));
            }
        });
        if editor.pending_for_task(&task.id, session) {
            ui.label(tr(l, "矩阵输出须先处理全体成员及依赖的相关草稿。", "Matrix output requires resolving related drafts across all members and their dependencies."));
        } else {
            ui.small(tr(
                l,
                "可导出覆盖不完整的审阅矩阵，包含全部状态与原因。",
                "An incomplete review matrix can be exported with every status and reason.",
            ));
        }
        let capture = match session.capture_rating_matrix(&task.id) {
            Ok(capture) => capture,
            Err(error) => {
                ui.label(error.to_string());
                return;
            }
        };
        matrix_summary(ui, l, &capture, units);
        let mut clicked = None;
        if wide {
            egui::Grid::new(("rating-matrix", &task.id))
                .striped(true)
                .num_columns(7)
                .max_col_width((ui.available_width() / 7.0 - 12.0).max(80.0))
                .show(ui, |ui| {
                    for label in [
                        tr(l, "工况", "Case"),
                        tr(l, "结果资格", "Result status"),
                        tr(l, "计算阶段", "Execution"),
                        tr(l, "气相负荷", "Gas load"),
                        tr(l, "液体负荷", "Liquid load"),
                        tr(l, "两项结论", "Criteria"),
                        tr(l, "控制项", "Controls"),
                    ] {
                        ui.strong(label);
                    }
                    ui.end_row();
                    for (case, row) in task.cases.iter().zip(capture.rows()) {
                        let mut name = case_label(l, case);
                        if editor
                            .draft(&EquipmentEditorTarget::RatingCase(row.member.clone()))
                            .is_some_and(|d| d.is_pending())
                        {
                            name.push_str(" *");
                        }
                        if ui
                            .add(
                                egui::Button::new(name)
                                    .selected(row.member == *selected)
                                    .wrap(),
                            )
                            .clicked()
                        {
                            clicked = Some(row.member.clone());
                        }
                        ui.label(rating_labels::status_label(l, row.status));
                        ui.label(self.equipment_execution_label(&row.member));
                        let p = row.result.as_ref().map(|r| r.performance());
                        for load in [
                            p.map(|p| p.gas.load_fraction),
                            p.map(|p| p.liquid.load_fraction),
                        ] {
                            ui.label(
                                load.map(|v| {
                                    results::format_value(
                                        v,
                                        QuantityKind::DimensionlessRatio,
                                        units,
                                    )
                                })
                                .unwrap_or_else(|| "—".into()),
                            );
                        }
                        ui.label(
                            p.map(|p| rating_labels::constraint(l, p.conclusion()))
                                .unwrap_or("—"),
                        );
                        ui.label(control_label(l, &capture, &case.id));
                        ui.end_row();
                    }
                });
        } else {
            for (case, row) in task.cases.iter().zip(capture.rows()) {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    let name = format!(
                        "{}{}",
                        case_label(l, case),
                        if editor
                            .draft(&EquipmentEditorTarget::RatingCase(row.member.clone()))
                            .is_some_and(|d| d.is_pending())
                        {
                            " *"
                        } else {
                            ""
                        }
                    );
                    if ui
                        .add(
                            egui::Button::new(name)
                                .selected(row.member == *selected)
                                .wrap(),
                        )
                        .clicked()
                    {
                        clicked = Some(row.member.clone());
                    }
                    ui.label(rating_labels::status_label(l, row.status));
                    ui.label(self.equipment_execution_label(&row.member));
                    if let Some(r) = &row.result {
                        let p = r.performance();
                        ui.label(format!(
                            "{}: {} · {}: {}",
                            tr(l, "气相", "Gas"),
                            results::format_value(
                                p.gas.load_fraction,
                                QuantityKind::DimensionlessRatio,
                                units
                            ),
                            tr(l, "液体", "Liquid"),
                            results::format_value(
                                p.liquid.load_fraction,
                                QuantityKind::DimensionlessRatio,
                                units
                            )
                        ));
                        ui.label(rating_labels::constraint(l, p.conclusion()));
                        ui.small(control_label(l, &capture, &case.id));
                    } else if let Some(failure) = &row.failure
                        && let Some(issue) = failure.issues.first()
                    {
                        ui.label(rating_labels::issue_label(l, issue));
                    }
                });
            }
        }
        if let Some(key) = clicked {
            commands.push(EquipmentEditorCommand::Select(
                EquipmentEditorTarget::RatingCase(key),
            ));
        }
    }
}
fn case_label(l: StudioShellLocale, case: &rf_model::equipment::SeparatorRatingCase) -> String {
    format!(
        "{} · {}",
        case.id,
        nonempty(&case.name, tr(l, "未命名工况", "Untitled case"))
    )
}
fn control_label(
    l: StudioShellLocale,
    capture: &RatingMatrixCapture,
    id: &rf_types::EngineeringCaseId,
) -> String {
    let mut labels = vec![];
    for (name, control) in [
        (tr(l, "气相", "Gas"), &capture.summary().gas),
        (tr(l, "液体", "Liquid"), &capture.summary().liquid),
    ] {
        if let Some(control) = control
            && control.case_ids.contains(id)
        {
            labels.push(format!(
                "{name}{}",
                if control.case_ids.len() > 1 {
                    tr(l, "（并列）", " (tie)")
                } else {
                    ""
                }
            ));
        }
    }
    if labels.is_empty() {
        "—".into()
    } else {
        labels.join(" / ")
    }
}
fn matrix_summary(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    capture: &RatingMatrixCapture,
    units: &DisplayUnitSet,
) {
    let summary = capture.summary();
    let complete = summary.coverage == MatrixCoverage::Complete;
    ui.separator();
    ui.strong(format!(
        "{} {}/{}",
        if complete {
            tr(l, "覆盖完整", "Complete coverage")
        } else {
            tr(l, "覆盖不完整", "Incomplete coverage")
        },
        summary.evaluated_case_ids.len(),
        summary.required_case_ids.len()
    ));
    if summary.all_cases_within_criteria() {
        ui.label(tr(
            l,
            "所列工况的两项准则内。",
            "Both criteria are within limits for all listed cases.",
        ));
    } else if let Some(status) = summary.subset_conclusion {
        let text = format!(
            "{}: {}",
            if complete {
                tr(l, "所列工况", "Listed cases")
            } else {
                tr(l, "已完成子集", "Completed subset")
            },
            rating_labels::constraint(l, status)
        );
        if status == ConstraintStatus::Exceeded {
            ui.colored_label(egui::Color32::from_rgb(160, 65, 30), text);
        } else {
            ui.label(text);
        }
    } else {
        ui.label(tr(
            l,
            "暂无有效结果，尚无控制工况。",
            "No current results or controlling cases yet.",
        ));
    }
    ui.small(tr(l,"仅适用于本列表及各行采用条件；不代表连续操作包络、效率或机械合格。", "Applies only to this list and each row's adopted conditions; no continuous operating envelope, efficiency or mechanical certification."));
    for (name, control) in [
        (tr(l, "气相控制", "Gas controls"), &summary.gas),
        (tr(l, "液体控制", "Liquid controls"), &summary.liquid),
    ] {
        if let Some(control) = control {
            ui.label(format!(
                "{name}: {} · {}",
                control
                    .case_ids
                    .iter()
                    .map(|id| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                results::format_value(
                    control.maximum_load_fraction,
                    QuantityKind::DimensionlessRatio,
                    units
                )
            ))
            .on_hover_text(format!(
                "{} = {}",
                tr(l, "未舍入最大负荷", "Unrounded maximum load"),
                control.maximum_load_fraction
            ));
        }
    }
    ui.small(tr(l,"并列按未舍入负荷距最大值 ≤ 1e-10 判断；不改变各行自身结论。", "Ties use unrounded distance from the maximum ≤ 1e-10; each row retains its own conclusion."));
    if capture
        .rows()
        .iter()
        .filter_map(|r| r.result.as_ref())
        .next()
        .is_none_or(|r| r.performance().tangent_to_tangent_height_m.is_none())
    {
        ui.label(tr(
            l,
            "总高尚不确定；缺项不补零。",
            "Total height remains unknown; missing sections are not zero.",
        ));
    }
}
