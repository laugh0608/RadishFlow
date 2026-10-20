use super::*;
use rf_types::units::QuantityKind;

pub(super) fn text_row(ui: &mut egui::Ui, label: &str, value: &mut String) -> egui::Response {
    let id = ui.make_persistent_id(("equipment-text", label));
    let label = ui.label(label);
    ui.add(
        egui::TextEdit::singleline(value)
            .id(id)
            .desired_width(f32::INFINITY),
    )
    .labelled_by(label.id)
}
pub(super) fn multiline(ui: &mut egui::Ui, label: &str, value: &mut String) {
    let id = ui.make_persistent_id(("equipment-multiline", label));
    let label = ui.label(label);
    ui.add(
        egui::TextEdit::multiline(value)
            .id(id)
            .desired_width(f32::INFINITY)
            .desired_rows(2),
    )
    .labelled_by(label.id);
}
pub(super) fn quantity(
    ui: &mut egui::Ui,
    locale: StudioShellLocale,
    key: impl std::hash::Hash,
    quantity: QuantityKind,
    value: &mut QuantityDraft,
    focus: bool,
    label: &str,
) {
    let id = ui.make_persistent_id(("equipment-number", key));
    let mut raw = value.text().to_owned();
    let response = ui.add(
        egui::TextEdit::singleline(&mut raw)
            .id(id)
            .desired_width(ui.available_width().clamp(80.0, 220.0))
            .hint_text(tr(locale, "未填写", "Not entered")),
    );
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::text_edit(ui.is_enabled(), value.text(), &raw);
        info.label = Some(label.into());
        info
    });
    if response.changed() {
        value.set_text(raw);
    }
    if focus {
        response.request_focus();
        response.scroll_to_me(Some(egui::Align::Center));
    }
    let choices = quantity
        .definition()
        .units
        .iter()
        .map(|u| (*u, u.definition().symbol))
        .collect::<Vec<_>>();
    let selection = super::super::unit_selector::unit_selector(
        ui,
        id.with("unit"),
        &format!("{label} {}", tr(locale, "输入单位", "input unit")),
        value.unit(),
        &choices,
    );
    let unit = selection.inner.unwrap_or(value.unit());
    if unit != value.unit()
        && let Err(error) = value.select_unit(quantity, unit)
    {
        ui.colored_label(
            egui::Color32::from_rgb(160, 65, 30),
            edit_error(locale, &error),
        );
    }
    if !value.text().trim().is_empty()
        && let Err(error) = value.value(quantity)
    {
        ui.colored_label(
            egui::Color32::from_rgb(160, 65, 30),
            edit_error(locale, &error),
        );
    }
}
pub(super) fn source_picker(
    ui: &mut egui::Ui,
    locale: StudioShellLocale,
    key: impl std::hash::Hash,
    value: &mut Option<rf_types::SourceId>,
    inputs: &rf_ui::engineering::EngineeringInputs,
) {
    let selected = value
        .as_ref()
        .map(|id| target_label(locale, &EquipmentEditorTarget::Source(id.clone()), inputs))
        .unwrap_or_else(|| tr(locale, "选择来源…", "Choose source…").into());
    egui::ComboBox::from_id_salt(key)
        .selected_text(selected)
        .width(180.0_f32.min(ui.available_width()))
        .show_ui(ui, |ui| {
            ui.selectable_value(value, None, tr(locale, "未选择", "Not selected"));
            for source in &inputs.basis.sources {
                ui.selectable_value(
                    value,
                    Some(source.id.clone()),
                    nonempty(&source.title, tr(locale, "未命名来源", "Untitled source")),
                );
            }
        });
}
pub(super) fn source_kind(
    locale: StudioShellLocale,
    kind: rf_model::engineering::SourceKind,
) -> &'static str {
    use rf_model::engineering::SourceKind::*;
    match kind {
        Measured => tr(locale, "实测", "Measured"),
        Vendor => tr(locale, "供应商", "Vendor"),
        Literature => tr(locale, "文献 / 数据库", "Literature / database"),
        Estimated => tr(locale, "估算", "Estimated"),
        Assumed => tr(locale, "用户假设", "User assumption"),
        Unknown => tr(locale, "未记录", "Unknown"),
    }
}
pub(super) fn source_review(
    locale: StudioShellLocale,
    review: rf_model::engineering::SourceReview,
) -> &'static str {
    use rf_model::engineering::SourceReview::*;
    match review {
        Unreviewed => tr(locale, "未采用 / 待复核", "Not adopted / pending review"),
        Adopted => tr(locale, "已明确采用", "Explicitly adopted"),
        Reviewed => tr(locale, "已审阅依据", "Evidence reviewed"),
    }
}
pub(super) fn quantity_name(locale: StudioShellLocale, q: QuantityKind) -> &'static str {
    use QuantityKind::*;
    match q {
        AbsoluteTemperature => tr(locale, "绝对温度", "Absolute temperature"),
        TemperatureDifference => tr(locale, "温差", "Temperature difference"),
        AbsolutePressure => tr(locale, "绝对压力", "Absolute pressure"),
        MolarFlow => tr(locale, "摩尔流量", "Molar flow"),
        MoleFraction => tr(locale, "摩尔分率", "Mole fraction"),
        MolarPhaseFraction => tr(locale, "相摩尔分率", "Molar phase fraction"),
        MolarEnthalpy => tr(locale, "摩尔焓", "Molar enthalpy"),
        ActualVolumeFlow => tr(locale, "实际体积流量", "Actual volume flow"),
        MassDensity => tr(locale, "质量密度", "Mass density"),
        Velocity => tr(locale, "速度", "Velocity"),
        Duration => tr(locale, "时间", "Duration"),
        Length => tr(locale, "长度", "Length"),
        Area => tr(locale, "面积", "Area"),
        Volume => tr(locale, "体积", "Volume"),
        DimensionlessRatio => tr(locale, "无量纲比值", "Dimensionless ratio"),
    }
}
