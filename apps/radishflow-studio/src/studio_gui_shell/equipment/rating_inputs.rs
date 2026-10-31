use super::*;
use rf_model::{engineering::InputBinding, equipment::*};
pub(super) fn labels(ui: &mut egui::Ui, l: StudioShellLocale, draft: &mut RatingLabelsDraft) {
    text_row(ui, tr(l, "设备位号", "Equipment tag"), &mut draft.tag);
    text_row(ui, tr(l, "设备名称", "Equipment name"), &mut draft.name);
    ui.label(tr(
        l,
        "任务模式固定为已有几何校核。",
        "This task's mode is fixed to given-geometry rating.",
    ));
}
#[allow(clippy::too_many_arguments)]
pub(super) fn geometry(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    draft: &mut RatingGeometryDraft,
    formal: &SeparatorGivenGeometry,
    inputs: &EngineeringInputs,
    units: &DisplayUnitSet,
    focus: Option<EquipmentInputField>,
    open_source: &mut Option<(rf_types::SourceId, Option<EquipmentInputField>)>,
) {
    ui.heading(tr(l, "给定几何", "Given geometry"));
    ui.label(tr(
        l,
        "填写已有内径与 LLL → NLL 工作液层高度，不用推荐尺寸或最大允许高度代替。",
        "Enter the given internal diameter and LLL → NLL working height, not a recommended minimum or maximum allowance.",
    ));
    for field in SeparatorGeometryInput::ALL {
        let key = RatingGeometryField::Input(field);
        let display = EquipmentInputField::Geometry(key);
        if let Some(source) = binding::render(
            ui,
            l,
            display,
            draft.fields.get_mut(&key).expect("geometry draft field"),
            inputs,
            units,
            focus == Some(display),
        ) {
            *open_source = Some((source, Some(display)));
        }
        committed(
            ui,
            l,
            formal.inputs.get(&field),
            field.quantity(),
            inputs,
            units,
        );
    }
    ui.heading(tr(l, "结构与适用性", "Structure and applicability"));
    applicability::orientation(ui, l, &mut draft.applicability.orientation);
    applicability::gas_direction(ui, l, &mut draft.applicability.gas_direction);
    applicability::geometry(ui, l, &mut draft.applicability.geometry);
    text_row(
        ui,
        tr(l, "内件配置说明", "Internals configuration"),
        &mut draft.applicability.internals_description,
    );
    sources(ui, l, &mut draft.applicability_sources, inputs, open_source);
    ui.label(tr(
        l,
        "尺寸 / 图纸或显式假设均须保留依据；几何来源不要求伪造流体温压。",
        "Retain dimensional/drawing evidence or an explicit assumption; geometry sources need no invented fluid conditions.",
    ));
    egui::CollapsingHeader::new(tr(l, "轴向区段 · 可选", "Axial sections · optional"))
        .id_salt("rating-axial")
        .open(matches!(focus, Some(EquipmentInputField::Geometry(RatingGeometryField::Axial(_)))).then_some(true))
        .show(ui, |ui| {
            ui.label(tr(
                l,
                "未填齐时总高未知，不补零。LLL → NLL 使用上方给定高度。",
                "Incomplete sections leave total height unknown, not zero. LLL → NLL uses the given height above.",
            ));
            for section in SeparatorAxialSection::ALL {
                let key = RatingGeometryField::Axial(section);
                let field = EquipmentInputField::Geometry(key);
                if let Some(source) = binding::render(ui, l, field, draft.fields.get_mut(&key).expect("axial draft"), inputs, units, focus == Some(field)) {
                    *open_source = Some((source, Some(field)));
                }
                committed(ui, l, formal.axial_sections.get(&section), QuantityKind::Length, inputs, units);
            }
        });
}
#[allow(clippy::too_many_arguments)]
pub(super) fn case(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    draft: &mut RatingCaseDraft,
    formal: &SeparatorRatingCase,
    inputs: &EngineeringInputs,
    units: &DisplayUnitSet,
    focus: Option<EquipmentInputField>,
    open_source: &mut Option<(rf_types::SourceId, Option<EquipmentInputField>)>,
) {
    ui.heading(tr(l, "当前工况", "Current case"));
    text_row(ui, tr(l, "工况名称", "Case name"), &mut draft.name);
    text_row(ui, tr(l, "工况用途", "Case purpose"), &mut draft.purpose);
    text_row(ui, tr(l, "物系", "Fluid system"), &mut draft.fluid_system);
    ui.label(tr(
        l,
        "两相流量均为操作条件下实际体积流量，密度与 K 必须有匹配依据。",
        "Both flows are actual at operating conditions; densities and K require matching evidence.",
    ));
    for field in SeparatorRatingInput::ALL {
        let display = EquipmentInputField::Rating(field);
        if let Some(source) = binding::render(
            ui,
            l,
            display,
            draft.inputs.get_mut(&field).expect("rating input draft"),
            inputs,
            units,
            focus == Some(display),
        ) {
            *open_source = Some((source, Some(display)));
        }
        committed(
            ui,
            l,
            formal.inputs.get(&field),
            field.quantity(),
            inputs,
            units,
        );
    }
    ui.heading(tr(l, "工况适用性与依据", "Case applicability and evidence"));
    applicability::service(ui, l, &mut draft.applicability.service);
    applicability::k_basis(ui, l, &mut draft.applicability.k_basis);
    applicability::flow_basis(ui, l, &mut draft.applicability.flow_basis);
    sources(ui, l, &mut draft.applicability_sources, inputs, open_source);
}
fn committed(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    binding: Option<&InputBinding>,
    q: QuantityKind,
    inputs: &EngineeringInputs,
    units: &DisplayUnitSet,
) {
    let text = match binding {
        None => tr(l, "未填写", "Not entered").to_owned(),
        Some(binding) => match inputs.basis.resolve(q, binding) {
            Ok(value) => results::format_value(value.value.value_si, q, units),
            Err(_) => tr(
                l,
                "引用无效，请检查依据",
                "Invalid reference; review evidence",
            )
            .to_owned(),
        },
    };
    ui.label(format!("{}：{text}", tr(l, "正式值", "Committed value",)));
}
fn sources(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    selected: &mut Vec<rf_types::SourceId>,
    inputs: &EngineeringInputs,
    open_source: &mut Option<(rf_types::SourceId, Option<EquipmentInputField>)>,
) {
    ui.label(tr(
        l,
        "适用性依据（需明确采用来源）",
        "Applicability evidence (explicitly adopt the source)",
    ));
    if inputs.basis.sources.is_empty() {
        ui.label(tr(
            l,
            "先在工程基础中新建来源。",
            "Create a source in engineering basis first.",
        ));
    }
    for source in &inputs.basis.sources {
        let mut checked = selected.contains(&source.id);
        ui.push_id(&source.id, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .checkbox(
                        &mut checked,
                        nonempty(&source.title, tr(l, "未命名来源", "Untitled source")),
                    )
                    .changed()
                {
                    if checked {
                        selected.push(source.id.clone());
                    } else {
                        selected.retain(|id| *id != source.id);
                    }
                }
                if ui.small_button(tr(l, "查看来源", "View source")).clicked() {
                    *open_source = Some((source.id.clone(), None));
                }
            });
        });
    }
}
