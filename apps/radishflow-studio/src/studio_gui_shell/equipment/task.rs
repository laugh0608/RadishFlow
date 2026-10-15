use super::*;
use rf_model::equipment::*;

pub(super) fn render(
    ui: &mut egui::Ui,
    locale: StudioShellLocale,
    draft: &mut SeparatorTaskDraft,
    inputs: &EngineeringInputs,
    units: &DisplayUnitSet,
    focus: Option<EquipmentField>,
    open_source: &mut Option<(rf_types::SourceId, Option<EquipmentField>)>,
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
        if let Some(source) = binding(
            ui,
            locale,
            field,
            draft.fields.get_mut(&field).expect("complete draft"),
            inputs,
            units,
            focus == Some(field),
        ) {
            *open_source = Some((source, Some(field)));
        }
    }
    ui.separator();
    ui.heading(tr(locale, "轴向区段 · 可选", "Axial sections · optional"));
    ui.label(tr(locale,"区段未齐全时总高为未知；不以零补齐。LLL → NLL 由工作容积计算。","Total height remains unknown until all sections are entered. LLL → NLL is calculated from working volume."));
    for field in SeparatorAxialSection::ALL
        .into_iter()
        .map(EquipmentField::Axial)
    {
        if let Some(source) = binding(
            ui,
            locale,
            field,
            draft.fields.get_mut(&field).expect("complete draft"),
            inputs,
            units,
            focus == Some(field),
        ) {
            *open_source = Some((source, Some(field)));
        }
    }
}
fn choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    locale: StudioShellLocale,
    key: &str,
    value: &mut Option<T>,
    options: &[(T, &str)],
) {
    ui.label(key);
    let label = options
        .iter()
        .find(|(v, _)| Some(*v) == *value)
        .map(|(_, label)| *label)
        .unwrap_or(tr(locale, "未声明", "Not declared"));
    egui::ComboBox::from_id_salt(key)
        .selected_text(label)
        .width(ui.available_width().min(360.0))
        .show_ui(ui, |ui| {
            ui.selectable_value(value, None, tr(locale, "未声明", "Not declared"));
            for (v, label) in options {
                ui.selectable_value(value, Some(*v), *label);
            }
        });
}
fn applicability(ui: &mut egui::Ui, l: StudioShellLocale, a: &mut SeparatorApplicability) {
    choice(
        ui,
        l,
        tr(l, "方向", "Orientation"),
        &mut a.orientation,
        &[
            (SeparatorOrientation::Vertical, tr(l, "立式", "Vertical")),
            (
                SeparatorOrientation::Horizontal,
                tr(l, "卧式", "Horizontal"),
            ),
        ],
    );
    choice(
        ui,
        l,
        tr(l, "气流方向", "Gas direction"),
        &mut a.gas_direction,
        &[
            (SeparatorGasDirection::Upward, tr(l, "向上", "Upward")),
            (SeparatorGasDirection::Other, tr(l, "其他", "Other")),
        ],
    );
    choice(
        ui,
        l,
        tr(l, "介质工况", "Service"),
        &mut a.service,
        &[
            (
                SeparatorService::CleanNonFoamingTwoPhase,
                tr(
                    l,
                    "洁净、不起泡气液两相",
                    "Clean nonfoaming gas–liquid two phase",
                ),
            ),
            (SeparatorService::SinglePhase, tr(l, "单相", "Single phase")),
            (SeparatorService::Foaming, tr(l, "起泡", "Foaming")),
            (SeparatorService::Solids, tr(l, "含固体", "Solids")),
            (SeparatorService::Slugging, tr(l, "段塞", "Slugging")),
            (SeparatorService::ThreePhase, tr(l, "三相", "Three phase")),
        ],
    );
    choice(
        ui,
        l,
        tr(l, "截面几何", "Section geometry"),
        &mut a.geometry,
        &[
            (
                SeparatorSectionGeometry::FullEqualDiameterCylinder,
                tr(l, "完整等径圆柱", "Full equal-diameter cylinder"),
            ),
            (
                SeparatorSectionGeometry::ReducedEffectiveArea,
                tr(l, "有效面积缩减", "Reduced effective area"),
            ),
            (
                SeparatorSectionGeometry::NonCylindrical,
                tr(l, "非圆柱", "Noncylindrical"),
            ),
        ],
    );
    choice(
        ui,
        l,
        tr(l, "K 的适用范围", "K applicability"),
        &mut a.k_basis,
        &[
            (
                SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals,
                tr(
                    l,
                    "匹配所述内件的整罐有效 K",
                    "Effective whole-vessel K for declared internals",
                ),
            ),
            (
                SeparatorKBasis::MistEliminatorOnly,
                tr(l, "仅除沫器 K", "Mist eliminator only"),
            ),
            (
                SeparatorKBasis::UnmatchedInternals,
                tr(l, "内件不匹配", "Unmatched internals"),
            ),
        ],
    );
    choice(
        ui,
        l,
        tr(l, "流量基准", "Flow basis"),
        &mut a.flow_basis,
        &[
            (
                SeparatorFlowBasis::ActualAtOperatingConditions,
                tr(l, "操作条件实际流量", "Actual at operating conditions"),
            ),
            (
                SeparatorFlowBasis::StandardConditions,
                tr(l, "标准状态流量", "Standard conditions"),
            ),
        ],
    );
    text_row(
        ui,
        tr(l, "内件配置说明", "Internals configuration"),
        &mut a.internals_description,
    );
}
fn binding(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    field: EquipmentField,
    draft: &mut BindingDraft,
    inputs: &EngineeringInputs,
    units: &DisplayUnitSet,
    focus: bool,
) -> Option<rf_types::SourceId> {
    let mut open_source = None;
    let frame = egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.push_id(field, |ui| {
            ui.strong(field_label(l, field));
            let modes = [
                (BindingMode::Missing, tr(l, "未填写", "Not entered")),
                (BindingMode::Direct, tr(l, "直接输入", "Direct input")),
                (BindingMode::BasisRef, tr(l, "引用基础", "Basis reference")),
                (BindingMode::Override, tr(l, "局部覆盖", "Local override")),
            ];
            egui::ComboBox::from_id_salt("binding-mode")
                .selected_text(modes.iter().find(|(m, _)| *m == draft.mode).unwrap().1)
                .show_ui(ui, |ui| {
                    for (mode, label) in modes {
                        ui.selectable_value(&mut draft.mode, mode, label);
                    }
                });
            if matches!(draft.mode, BindingMode::BasisRef | BindingMode::Override) {
                let label = draft
                    .basis
                    .as_ref()
                    .map(|id| target_label(l, &EquipmentEditorTarget::Basis(id.clone()), inputs))
                    .unwrap_or(tr(l, "选择基础条目…", "Choose basis entry…").into());
                egui::ComboBox::from_id_salt("basis-ref")
                    .selected_text(label)
                    .width(ui.available_width().min(300.0))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut draft.basis,
                            None,
                            tr(l, "未选择", "Not selected"),
                        );
                        for basis in inputs
                            .basis
                            .basis_entries
                            .iter()
                            .filter(|b| b.value.quantity == field.quantity())
                        {
                            ui.selectable_value(
                                &mut draft.basis,
                                Some(basis.id.clone()),
                                &basis.name,
                            );
                        }
                    });
                if let Some(basis) = inputs
                    .basis
                    .basis_entries
                    .iter()
                    .find(|b| Some(&b.id) == draft.basis.as_ref())
                {
                    ui.label(format!(
                        "{}：{}",
                        tr(l, "基础值", "Basis value"),
                        results::format_value(basis.value.value_si, field.quantity(), units)
                    ));
                    ui.label(format!(
                        "{}：{}",
                        tr(l, "基础来源", "Basis source"),
                        target_label(
                            l,
                            &EquipmentEditorTarget::Source(basis.source_id.clone()),
                            inputs
                        )
                    ));
                    if ui
                        .small_button(tr(l, "查看基础来源", "View basis source"))
                        .clicked()
                    {
                        open_source = Some(basis.source_id.clone());
                    }
                }
            }
            if !matches!(draft.mode, BindingMode::BasisRef) {
                let before = draft.value.clone();
                quantity(
                    ui,
                    l,
                    "value",
                    field.quantity(),
                    &mut draft.value,
                    focus,
                    field_label(l, field),
                );
                if draft.mode == BindingMode::Missing
                    && before != draft.value
                    && !draft.value.text().is_empty()
                {
                    draft.mode = BindingMode::Direct;
                }
                if draft.mode != BindingMode::Missing {
                    source_picker(ui, l, "source", &mut draft.source, inputs);
                    if let Some(id) = &draft.source {
                        if let Some(source) = inputs.basis.sources.iter().find(|s| s.id == *id) {
                            ui.label(format!(
                                "{} · {}",
                                source_kind(l, source.kind),
                                source_review(l, source.review)
                            ));
                        }
                        if ui
                            .small_button(tr(l, "编辑 / 查看来源", "Edit / view source"))
                            .clicked()
                        {
                            open_source = Some(id.clone());
                        }
                    }
                }
            }
            if draft.mode == BindingMode::Override {
                text_row(ui, tr(l, "覆盖理由", "Override reason"), &mut draft.reason);
            }
        });
    });
    if focus {
        frame.response.scroll_to_me(Some(egui::Align::Center));
    }
    open_source
}
