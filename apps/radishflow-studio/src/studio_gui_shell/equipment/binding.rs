use super::*;

pub(super) fn render(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    field: EquipmentInputField,
    draft: &mut BindingDraft,
    inputs: &EngineeringInputs,
    units: &DisplayUnitSet,
    focus: bool,
) -> Option<rf_types::SourceId> {
    let mut open_source = None;
    let frame = egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.push_id(field, |ui| {
            ui.strong(field.label(l));
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
                    field.label(l),
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
