use super::*;
use rf_model::equipment::*;

pub(super) fn render(
    ui: &mut egui::Ui,
    locale: StudioShellLocale,
    draft: &mut SeparatorTaskDraft,
    inputs: &EngineeringInputs,
    units: &DisplayUnitSet,
    focus: Option<EquipmentInputField>,
    open_source: &mut Option<(rf_types::SourceId, Option<EquipmentInputField>)>,
) {
    text_row(ui, tr(locale, "设备位号", "Equipment tag"), &mut draft.tag);
    text_row(
        ui,
        tr(locale, "设备名称", "Equipment name"),
        &mut draft.name,
    );
    text_row(
        ui,
        tr(locale, "工况名称", "Case name"),
        &mut draft.case_name,
    );
    text_row(
        ui,
        tr(locale, "工况用途", "Case purpose"),
        &mut draft.purpose,
    );
    text_row(
        ui,
        tr(locale, "物系", "Fluid system"),
        &mut draft.fluid_system,
    );
    ui.separator();
    ui.heading(tr(locale, "适用性声明", "Applicability declarations"));
    applicability(ui, locale, &mut draft.applicability);
    ui.label(tr(
        locale,
        "适用性依据（需明确采用来源）",
        "Applicability evidence (explicitly adopt the source)",
    ));
    for source in &inputs.basis.sources {
        let mut selected = draft.applicability_sources.contains(&source.id);
        ui.horizontal_wrapped(|ui| {
            if ui
                .checkbox(
                    &mut selected,
                    nonempty(&source.title, tr(locale, "未命名来源", "Untitled source")),
                )
                .changed()
            {
                if selected {
                    draft.applicability_sources.push(source.id.clone());
                } else {
                    draft.applicability_sources.retain(|id| id != &source.id);
                }
            }
            if ui
                .small_button(tr(locale, "查看来源", "View source"))
                .clicked()
            {
                *open_source = Some((source.id.clone(), None));
            }
        });
    }
    ui.separator();
    ui.heading(tr(locale, "工况与尺寸输入", "Case and sizing inputs"));
    ui.label(tr(
        locale,
        "所有数值由用户填写；气液流量均为操作条件下实际体积流量。",
        "Enter each value explicitly. Both volume flows are actual at operating conditions.",
    ));
    for field in SeparatorInput::ALL.into_iter().map(EquipmentField::Input) {
        if let Some(source) = binding::render(
            ui,
            locale,
            field.into(),
            draft.fields.get_mut(&field).expect("complete draft"),
            inputs,
            units,
            focus == Some(field.into()),
        ) {
            *open_source = Some((source, Some(field.into())));
        }
    }
    ui.separator();
    ui.heading(tr(locale, "轴向区段 · 可选", "Axial sections · optional"));
    ui.label(tr(locale,"区段未齐全时总高为未知；不以零补齐。LLL → NLL 由工作容积计算。","Total height remains unknown until all sections are entered. LLL → NLL is calculated from working volume."));
    for field in SeparatorAxialSection::ALL
        .into_iter()
        .map(EquipmentField::Axial)
    {
        if let Some(source) = binding::render(
            ui,
            locale,
            field.into(),
            draft.fields.get_mut(&field).expect("complete draft"),
            inputs,
            units,
            focus == Some(field.into()),
        ) {
            *open_source = Some((source, Some(field.into())));
        }
    }
}
fn applicability(ui: &mut egui::Ui, l: StudioShellLocale, a: &mut SeparatorApplicability) {
    applicability::orientation(ui, l, &mut a.orientation);
    applicability::gas_direction(ui, l, &mut a.gas_direction);
    applicability::service(ui, l, &mut a.service);
    applicability::geometry(ui, l, &mut a.geometry);
    applicability::k_basis(ui, l, &mut a.k_basis);
    applicability::flow_basis(ui, l, &mut a.flow_basis);
    text_row(
        ui,
        tr(l, "内件配置说明", "Internals configuration"),
        &mut a.internals_description,
    );
}
