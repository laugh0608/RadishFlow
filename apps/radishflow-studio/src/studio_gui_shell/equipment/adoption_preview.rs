use super::*;
use rf_model::equipment::{SeparatorAxialSection, SeparatorGeometryInput};
use rf_ui::engineering::{GeometryAdoptionTarget, PreparedGeometryAdoption};

pub(super) fn dimensions(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    form: &mut adoption::AdoptionForm,
    units: &DisplayUnitSet,
) {
    dimension(
        ui,
        l,
        SeparatorGeometryInput::InternalDiameter,
        form.original_diameter,
        &mut form.diameter,
        units,
        form.error.as_ref(),
    );
    dimension(
        ui,
        l,
        SeparatorGeometryInput::WorkingLiquidHeight,
        form.original_height,
        &mut form.height,
        units,
        form.error.as_ref(),
    );
    ui.label(tr(l, "内径与工作液层独立调整；不自动联动。工作液层来自计算值，不是最大允许高度。", "Diameter and working height are adjusted independently. Working height is the calculated value, not the maximum allowed height."));
}

fn dimension(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    field: SeparatorGeometryInput,
    original: f64,
    draft: &mut Option<DimensionAdjustmentDraft>,
    units: &DisplayUnitSet,
    error: Option<&EquipmentEditError>,
) {
    ui.push_id(field, |ui| {
        ui.separator();
        let label = geometry_field_label(l, field);
        ui.strong(label);
        ui.label(format!("{}: {original} m", tr(l, "原始未舍入 SI", "Original unrounded SI")));
        ui.label(results::format_value(original, QuantityKind::Length, units));
        let mut adjust = draft.is_some();
        if ui.checkbox(&mut adjust, tr(l, "显式调整尺寸", "Explicitly adjust dimension")).changed() {
            *draft = adjust.then(|| DimensionAdjustmentDraft {
                value: QuantityDraft::from_si(original, QuantityKind::Length, units.unit_for(QuantityKind::Length)),
                reason: String::new(),
            });
        }
        if let Some(draft) = draft {
            quantity(ui, l, ("adopted", field), QuantityKind::Length, &mut draft.value, false, label);
            text_row(ui, tr(l, "调整理由（必填）", "Adjustment reason (required)"), &mut draft.reason);
            if let Err(error) = draft.adjustment() {
                ui.colored_label(egui::Color32::from_rgb(160,65,30), edit_error(l, &error));
            }
            if matches!(error, Some(EquipmentEditError::RatingGeometryField(RatingGeometryField::Input(f), _)) if *f == field)
                && ui.button(tr(l, "定位调整字段", "Focus adjustment field")).clicked()
            {
                ui.memory_mut(|m| m.request_focus(ui.make_persistent_id(("equipment-number", ("adopted", field)))));
            }
        } else {
            ui.label(tr(l, "原样采用，保留完整 SI 精度。", "Use unchanged, preserving full SI precision."));
        }
    });
}

pub(super) fn render(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    preview: &PreparedGeometryAdoption,
    current: &EngineeringInputs,
    units: &DisplayUnitSet,
) {
    let target = preview.target().task_id();
    let task = preview
        .inputs()
        .rating_task(target)
        .expect("prepared rating task");
    ui.separator();
    ui.heading(format!(
        "{}: {}",
        tr(l, "预览目标", "Preview target"),
        task_label(l, &task.clone().into())
    ));
    match preview.target() {
        GeometryAdoptionTarget::New { copy_case, .. } => {
            ui.label(if *copy_case {
                tr(l, "新建任务并复制九项工况输入与来源；结果为缺失，需手动校核。", "Create a task and copy nine case inputs and sources. Results start missing; rate manually.")
            } else {
                tr(l, "新建任务并保留一条不完整工况；可保存，填写后手动校核。", "Create a task with one incomplete case. It can be saved; complete it before rating manually.")
            });
        }
        GeometryAdoptionTarget::Existing(_) => {
            ui.strong(tr(
                l,
                "完整替换共同几何，以下工况结果全部过期",
                "Replace all common geometry; these case results become stale",
            ));
            for (i, case) in task.cases.iter().enumerate() {
                ui.label(format!("{}. {} ({})", i + 1, case.name, case.id));
            }
        }
    }
    ui.label(tr(l, "保存将携带固定采纳证据；不支持该能力的旧版本整份只读打开。", "Saving includes fixed adoption evidence; older readers without this capability open the whole project read-only."));
    if let Some(old) = preview.previous_geometry() {
        for field in SeparatorGeometryInput::ALL {
            let value = old
                .inputs
                .get(&field)
                .and_then(|b| current.basis.resolve(field.quantity(), b).ok())
                .map(|v| v.value.value_si);
            ui.label(format!(
                "{} · {}: {}",
                tr(l, "被替换的原几何", "Previous geometry being replaced"),
                geometry_field_label(l, field),
                adoption_evidence::length(l, value, units)
            ));
        }
    }
    ui.strong(tr(
        l,
        "轴向段：仅采用本次有效捕获",
        "Axial sections: only valid captured segments are adopted",
    ));
    for field in SeparatorAxialSection::ALL {
        let a = task.geometry.adoption.as_ref().expect("adoption record");
        let value = a.axial_lengths_m.get(&field).copied();
        ui.label(format!(
            "{}: {}",
            field_label(l, EquipmentField::Axial(field)),
            adoption_evidence::length(l, value, units)
        ));
        if value.is_none()
            && preview
                .previous_geometry()
                .is_some_and(|g| g.axial_sections.contains_key(&field))
        {
            ui.colored_label(
                egui::Color32::from_rgb(160, 65, 30),
                tr(
                    l,
                    "将移除此旧段，不沿用旧几何补齐。",
                    "This old segment will be removed; it will not be retained to fill the gap.",
                ),
            );
        }
    }
    ui.label(tr(l, "缺失段保持未知，总高不倒推补齐。采纳不提升原来源的审阅状态。", "Missing segments remain unknown; total height does not fill gaps. Adoption does not promote source review states."));
    if let Some(a) = &task.geometry.adoption {
        for source in &a.source_basis.sources {
            ui.label(format!(
                "{} · {} · {}",
                source.title,
                source_kind(l, source.kind),
                source_review(l, source.review)
            ));
        }
    }
    egui::CollapsingHeader::new(tr(l, "查看采纳证据", "Adoption evidence"))
        .id_salt("adoption-preview-evidence")
        .show(ui, |ui| {
            adoption_evidence::render(ui, l, &task.geometry, &preview.inputs().basis, units);
        });
}
