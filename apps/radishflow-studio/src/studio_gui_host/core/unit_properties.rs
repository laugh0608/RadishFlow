use super::*;

pub(super) fn unit_property_fields(
    flowsheet: &rf_model::Flowsheet,
    unit: &rf_model::UnitNode,
    drafts: &rf_ui::InspectorDraftState,
) -> Vec<StudioGuiInspectorTargetFieldSnapshot> {
    let mut fields: Vec<_> = match unit.kind.as_str() {
        "feed" => [
            (
                rf_ui::UnitInspectorDraftField::OutletTemperatureK,
                "Source temperature (K)",
            ),
            (
                rf_ui::UnitInspectorDraftField::OutletPressurePa,
                "Source pressure (Pa)",
            ),
        ]
        .into_iter()
        .filter_map(|(field, label)| {
            unit_number_property_field(flowsheet, unit, drafts, field, label)
        })
        .collect(),
        "heater" | "cooler" => [
            (
                rf_ui::UnitInspectorDraftField::OutletTemperatureK,
                "Outlet temperature (K)",
            ),
            (
                rf_ui::UnitInspectorDraftField::OutletPressurePa,
                "Outlet pressure (Pa)",
            ),
        ]
        .into_iter()
        .filter_map(|(field, label)| {
            unit_number_property_field(flowsheet, unit, drafts, field, label)
        })
        .collect(),
        "valve" => unit_number_property_field(
            flowsheet,
            unit,
            drafts,
            rf_ui::UnitInspectorDraftField::OutletPressurePa,
            "Outlet pressure (Pa)",
        )
        .into_iter()
        .collect(),
        "mixer" => unit_number_property_field(
            flowsheet,
            unit,
            drafts,
            rf_ui::UnitInspectorDraftField::OutletPressurePa,
            "Outlet pressure (Pa)",
        )
        .into_iter()
        .collect(),
        "flash_drum" => [
            (
                rf_ui::UnitInspectorDraftField::OutletTemperatureK,
                "Flash temperature (K)",
            ),
            (
                rf_ui::UnitInspectorDraftField::OutletPressurePa,
                "Flash pressure (Pa)",
            ),
        ]
        .into_iter()
        .filter_map(|(field, label)| {
            unit_number_property_field(flowsheet, unit, drafts, field, label)
        })
        .collect(),
        _ => Vec::new(),
    };
    let mut name = inspector_text_field(
        drafts,
        rf_ui::unit_inspector_draft_key(&unit.id, &rf_ui::UnitInspectorDraftField::Name),
        "Name",
        unit.name.clone(),
    );
    name.constraint_text = Some("Name cannot be blank; duplicate display names are allowed. Object ID and connections remain unchanged.".to_string());
    fields.insert(0, name);
    fields
}

fn unit_number_property_field(
    flowsheet: &rf_model::Flowsheet,
    unit: &rf_model::UnitNode,
    drafts: &rf_ui::InspectorDraftState,
    field: rf_ui::UnitInspectorDraftField,
    label: &str,
) -> Option<StudioGuiInspectorTargetFieldSnapshot> {
    let original = rf_ui::unit_inspector_parameter_value(flowsheet, &unit.id, &field)?;
    let key = rf_ui::unit_inspector_draft_key(&unit.id, &field);
    let mut property_field = inspector_number_field(drafts, key.clone(), label, original);
    if unit_parameter_display_value_needs_explicit_commit(flowsheet, unit, drafts, &field, original)
    {
        property_field.is_dirty = true;
        property_field.validation = StudioGuiInspectorTargetFieldValidationSnapshot::Valid;
        property_field.commit_command_id = Some(crate::inspector_draft_commit_command_id(&key));
        property_field.discard_command_id = None;
    }
    property_field.constraint_text = Some(unit_parameter_constraint_text(flowsheet, unit, &field));
    Some(property_field)
}

fn unit_parameter_display_value_needs_explicit_commit(
    flowsheet: &rf_model::Flowsheet,
    unit: &rf_model::UnitNode,
    drafts: &rf_ui::InspectorDraftState,
    field: &rf_ui::UnitInspectorDraftField,
    value: f64,
) -> bool {
    let key = rf_ui::unit_inspector_draft_key(&unit.id, field);
    if drafts.fields.contains_key(&key) || rf_ui::unit_inspector_parameter_is_explicit(unit, field)
    {
        return false;
    }
    if !value.is_finite() || value <= 0.0 {
        return false;
    }
    if matches!(field, rf_ui::UnitInspectorDraftField::OutletPressurePa)
        && unit_outlet_pressure_cannot_exceed_inlet(unit)
    {
        return connected_inlet_pressure_limit(flowsheet, unit)
            .map(|pressure_pa| value <= pressure_pa)
            .unwrap_or(true);
    }
    true
}

fn unit_parameter_constraint_text(
    flowsheet: &rf_model::Flowsheet,
    unit: &rf_model::UnitNode,
    field: &rf_ui::UnitInspectorDraftField,
) -> String {
    match field {
        rf_ui::UnitInspectorDraftField::Name => "Name cannot be blank.".to_string(),
        rf_ui::UnitInspectorDraftField::OutletTemperatureK => {
            if unit.kind == "feed" {
                return "Unit K; positive finite source outlet temperature; commit syncs the Feed outlet template.".to_string();
            }
            if unit.kind == "flash_drum" {
                return "Unit K; positive finite flash temperature; commit syncs liquid/vapor outlet templates.".to_string();
            }
            "Unit K; positive finite outlet temperature; commit syncs the outlet stream template."
                .to_string()
        }
        rf_ui::UnitInspectorDraftField::OutletPressurePa => {
            if unit_outlet_pressure_cannot_exceed_inlet(unit) {
                let inlet_limit = connected_inlet_pressure_limit(flowsheet, unit)
                    .map(|pressure_pa| format!(" Inlet limit: {pressure_pa:.0} Pa."));
                return format!(
                    "Unit Pa; positive finite outlet pressure; cannot exceed connected inlet pressure.{}",
                    inlet_limit.unwrap_or_default()
                );
            }
            if unit.kind == "feed" {
                return "Unit Pa; positive finite source outlet pressure; commit syncs the Feed outlet template.".to_string();
            }
            "Unit Pa; positive finite flash pressure; commit syncs liquid/vapor outlet templates."
                .to_string()
        }
    }
}
