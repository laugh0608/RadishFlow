use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitInspectorDraftField {
    OutletTemperatureK,
    OutletPressurePa,
}

impl UnitInspectorDraftField {
    pub fn key_segment(&self) -> &'static str {
        match self {
            Self::OutletTemperatureK => "outlet_temperature_k",
            Self::OutletPressurePa => "outlet_pressure_pa",
        }
    }

    pub fn command_parameter(&self) -> String {
        self.key_segment().to_string()
    }

    pub fn from_static_key_segment(value: &str) -> Option<Self> {
        match value {
            "outlet_temperature_k" => Some(Self::OutletTemperatureK),
            "outlet_pressure_pa" => Some(Self::OutletPressurePa),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitInspectorDraftUpdateResult {
    pub key: String,
    pub active_target: InspectorTarget,
    pub is_dirty: bool,
    pub validation: DraftValidationState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitInspectorDraftCommitResult {
    pub key: String,
    pub active_target: InspectorTarget,
    pub command: DocumentCommand,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitInspectorDraftDiscardResult {
    pub key: String,
    pub active_target: InspectorTarget,
}

impl AppState {
    pub fn update_unit_inspector_draft(
        &mut self,
        unit_id: &UnitId,
        field: UnitInspectorDraftField,
        raw_value: impl Into<String>,
    ) -> Option<UnitInspectorDraftUpdateResult> {
        let active_target = InspectorTarget::Unit(unit_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return None;
        }

        let unit = self.workspace.document.flowsheet.units.get(unit_id)?;
        if !unit_inspector_draft_fields(unit).contains(&field) {
            return None;
        }
        let original_value =
            unit_inspector_parameter_value(&self.workspace.document.flowsheet, unit_id, &field)?;
        let raw_value = raw_value.into();
        let key = unit_inspector_draft_key(unit_id, &field);
        let (draft_value, is_dirty, validation) =
            unit_draft_value_from_raw(original_value, raw_value, |value| {
                is_valid_unit_parameter_value_for_unit(
                    &self.workspace.document.flowsheet,
                    unit,
                    &field,
                    value,
                )
            });

        if !is_dirty && validation != DraftValidationState::Invalid {
            self.workspace.drafts.fields.remove(&key);
        } else {
            self.workspace
                .drafts
                .fields
                .insert(key.clone(), draft_value);
        }

        Some(UnitInspectorDraftUpdateResult {
            key,
            active_target,
            is_dirty,
            validation,
        })
    }

    pub fn commit_unit_inspector_draft(
        &mut self,
        unit_id: &UnitId,
        field: UnitInspectorDraftField,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<UnitInspectorDraftCommitResult>> {
        let active_target = InspectorTarget::Unit(unit_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return Ok(None);
        }

        if !self
            .workspace
            .document
            .flowsheet
            .units
            .contains_key(unit_id)
        {
            return Ok(None);
        }

        let key = unit_inspector_draft_key(unit_id, &field);
        let Some(draft_value) = self.workspace.drafts.fields.get(&key) else {
            return Ok(None);
        };
        let Some(command_value) = unit_command_value_from_draft(&field, draft_value)? else {
            return Ok(None);
        };

        let mut next_flowsheet = self.workspace.document.flowsheet.clone();
        apply_unit_parameter_value(&mut next_flowsheet, unit_id, &field, &command_value)?;

        let command = DocumentCommand::SetUnitParameter {
            unit_id: unit_id.clone(),
            parameter: field.command_parameter(),
            value: command_value,
        };
        let revision = self.workspace.commit_inspector_document_change(
            command.clone(),
            next_flowsheet,
            changed_at,
        );
        self.workspace.drafts.fields.remove(&key);
        self.refresh_run_panel_state();

        Ok(Some(UnitInspectorDraftCommitResult {
            key,
            active_target,
            command,
            revision,
        }))
    }

    pub fn discard_unit_inspector_draft(
        &mut self,
        unit_id: &UnitId,
        field: UnitInspectorDraftField,
    ) -> Option<UnitInspectorDraftDiscardResult> {
        let active_target = InspectorTarget::Unit(unit_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return None;
        }

        let unit = self.workspace.document.flowsheet.units.get(unit_id)?;
        if !unit_inspector_draft_fields(unit).contains(&field) {
            return None;
        }

        let key = unit_inspector_draft_key(unit_id, &field);
        self.workspace.drafts.fields.remove(&key)?;

        Some(UnitInspectorDraftDiscardResult { key, active_target })
    }
}

pub fn unit_inspector_draft_key(unit_id: &UnitId, field: &UnitInspectorDraftField) -> String {
    format!("unit:{}:{}", unit_id.as_str(), field.key_segment())
}

pub fn unit_inspector_draft_key_parts(key: &str) -> Option<(UnitId, UnitInspectorDraftField)> {
    let rest = key.strip_prefix("unit:")?;
    let (unit_id, field) = rest.rsplit_once(':')?;
    if unit_id.is_empty() {
        return None;
    }
    Some((
        UnitId::new(unit_id),
        UnitInspectorDraftField::from_static_key_segment(field)?,
    ))
}

pub fn unit_inspector_parameter_value(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    field: &UnitInspectorDraftField,
) -> Option<f64> {
    let unit = flowsheet.units.get(unit_id)?;
    match field {
        UnitInspectorDraftField::OutletTemperatureK => unit
            .parameters
            .outlet_temperature_k
            .or_else(|| outlet_stream(flowsheet, unit).map(|stream| stream.temperature_k))
            .or_else(|| default_unit_parameter_value(unit, field)),
        UnitInspectorDraftField::OutletPressurePa => unit
            .parameters
            .outlet_pressure_pa
            .or_else(|| outlet_stream(flowsheet, unit).map(|stream| stream.pressure_pa))
            .or_else(|| default_unit_parameter_value(unit, field)),
    }
}

fn unit_draft_value_from_raw(
    original_value: f64,
    raw_value: String,
    is_valid_number: impl Fn(f64) -> bool,
) -> (DraftValue, bool, DraftValidationState) {
    unit_number_draft_value(original_value, raw_value, is_valid_number)
}

fn unit_number_draft_value<F>(
    original_number: f64,
    raw_value: String,
    is_valid_number: F,
) -> (DraftValue, bool, DraftValidationState)
where
    F: Fn(f64) -> bool,
{
    let original = format_edit_number(original_number);
    let parsed = raw_value.trim().parse::<f64>();
    let validation = match parsed {
        Ok(value) if is_valid_number(value) => DraftValidationState::Valid,
        _ => DraftValidationState::Invalid,
    };
    let is_dirty = match parsed {
        Ok(value) if validation == DraftValidationState::Valid => value != original_number,
        _ => raw_value != original,
    };
    let draft = FieldDraft {
        original,
        current: raw_value,
        is_dirty,
        validation,
    };
    (DraftValue::Number(draft), is_dirty, validation)
}

fn unit_command_value_from_draft(
    field: &UnitInspectorDraftField,
    draft_value: &DraftValue,
) -> RfResult<Option<CommandValue>> {
    match (field, draft_value) {
        (
            UnitInspectorDraftField::OutletTemperatureK | UnitInspectorDraftField::OutletPressurePa,
            DraftValue::Number(draft),
        ) => {
            if !draft.is_dirty || draft.validation != DraftValidationState::Valid {
                return Ok(None);
            }
            let value = draft.current.trim().parse::<f64>().map_err(|_| {
                RfError::invalid_input(format!(
                    "unit inspector draft `{}` is not a valid number",
                    draft.current
                ))
            })?;
            if !is_valid_unit_parameter_value(field, value) {
                return Ok(None);
            }
            Ok(Some(CommandValue::Number(value)))
        }
        _ => Ok(None),
    }
}

fn unit_inspector_draft_fields(unit: &UnitNode) -> Vec<UnitInspectorDraftField> {
    match unit.kind.as_str() {
        rf_unitops::HEATER_KIND | rf_unitops::COOLER_KIND => {
            vec![
                UnitInspectorDraftField::OutletTemperatureK,
                UnitInspectorDraftField::OutletPressurePa,
            ]
        }
        rf_unitops::MIXER_KIND | rf_unitops::VALVE_KIND | rf_unitops::FLASH_DRUM_KIND => {
            vec![UnitInspectorDraftField::OutletPressurePa]
        }
        _ => Vec::new(),
    }
}

fn apply_unit_parameter_value(
    flowsheet: &mut Flowsheet,
    unit_id: &UnitId,
    field: &UnitInspectorDraftField,
    value: &CommandValue,
) -> RfResult<()> {
    let outlet_stream_ids = {
        let unit = flowsheet
            .units
            .get(unit_id)
            .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
        if !unit_inspector_draft_fields(unit).contains(field) {
            return Err(RfError::invalid_input(format!(
                "unit `{}` of kind `{}` does not support parameter `{}`",
                unit.id,
                unit.kind,
                field.command_parameter()
            )));
        }
        outlet_stream_ids(unit)
    };

    let CommandValue::Number(value) = value else {
        return Err(RfError::invalid_input(format!(
            "unit parameter `{}` cannot be set from value `{value:?}`",
            field.command_parameter()
        )));
    };
    let unit = flowsheet
        .units
        .get(unit_id)
        .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
    if !is_valid_unit_parameter_value_for_unit(flowsheet, unit, field, *value) {
        return Err(RfError::invalid_input(format!(
            "unit parameter `{}` is outside the supported SI constraint",
            field.command_parameter()
        )));
    }

    let unit = flowsheet
        .units
        .get_mut(unit_id)
        .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
    match field {
        UnitInspectorDraftField::OutletTemperatureK => {
            unit.parameters.outlet_temperature_k = Some(*value);
        }
        UnitInspectorDraftField::OutletPressurePa => {
            unit.parameters.outlet_pressure_pa = Some(*value);
        }
    }

    for stream_id in outlet_stream_ids {
        if let Some(stream) = flowsheet.streams.get_mut(&stream_id) {
            match field {
                UnitInspectorDraftField::OutletTemperatureK => {
                    stream.temperature_k = *value;
                }
                UnitInspectorDraftField::OutletPressurePa => {
                    stream.pressure_pa = *value;
                }
            }
        }
    }

    Ok(())
}

fn is_valid_unit_parameter_value(_field: &UnitInspectorDraftField, value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn is_valid_unit_parameter_value_for_unit(
    flowsheet: &Flowsheet,
    unit: &UnitNode,
    field: &UnitInspectorDraftField,
    value: f64,
) -> bool {
    if !is_valid_unit_parameter_value(field, value) {
        return false;
    }

    if unit_outlet_pressure_cannot_exceed_inlet(unit)
        && matches!(field, UnitInspectorDraftField::OutletPressurePa)
    {
        return inlet_pressure_limit(flowsheet, unit)
            .map(|pressure_pa| value <= pressure_pa)
            .unwrap_or(true);
    }

    true
}

fn unit_outlet_pressure_cannot_exceed_inlet(unit: &UnitNode) -> bool {
    matches!(
        unit.kind.as_str(),
        rf_unitops::MIXER_KIND
            | rf_unitops::HEATER_KIND
            | rf_unitops::COOLER_KIND
            | rf_unitops::VALVE_KIND
    )
}

fn inlet_pressure_limit(flowsheet: &Flowsheet, unit: &UnitNode) -> Option<f64> {
    unit.ports
        .iter()
        .filter(|port| port.direction == PortDirection::Inlet && port.kind == PortKind::Material)
        .filter_map(|port| port.stream_id.as_ref())
        .filter_map(|stream_id| flowsheet.streams.get(stream_id))
        .map(|stream| stream.pressure_pa)
        .reduce(f64::min)
}

fn outlet_stream<'a>(flowsheet: &'a Flowsheet, unit: &UnitNode) -> Option<&'a MaterialStreamState> {
    outlet_stream_id(unit).and_then(|stream_id| flowsheet.streams.get(stream_id))
}

fn outlet_stream_ids(unit: &UnitNode) -> Vec<StreamId> {
    unit.ports
        .iter()
        .filter(|port| port.direction == PortDirection::Outlet && port.kind == PortKind::Material)
        .filter_map(|port| port.stream_id.clone())
        .collect()
}

fn outlet_stream_id(unit: &UnitNode) -> Option<&StreamId> {
    unit.ports
        .iter()
        .find(|port| port.direction == PortDirection::Outlet && port.kind == PortKind::Material)
        .and_then(|port| port.stream_id.as_ref())
}

fn default_unit_parameter_value(unit: &UnitNode, field: &UnitInspectorDraftField) -> Option<f64> {
    match (unit.kind.as_str(), field) {
        (rf_unitops::HEATER_KIND, UnitInspectorDraftField::OutletTemperatureK) => Some(345.0),
        (rf_unitops::HEATER_KIND, UnitInspectorDraftField::OutletPressurePa) => Some(101_325.0),
        (rf_unitops::COOLER_KIND, UnitInspectorDraftField::OutletTemperatureK) => Some(285.0),
        (rf_unitops::COOLER_KIND, UnitInspectorDraftField::OutletPressurePa) => Some(101_325.0),
        (rf_unitops::MIXER_KIND, UnitInspectorDraftField::OutletPressurePa) => Some(101_325.0),
        (rf_unitops::VALVE_KIND, UnitInspectorDraftField::OutletPressurePa) => Some(90_000.0),
        (rf_unitops::FLASH_DRUM_KIND, UnitInspectorDraftField::OutletPressurePa) => Some(101_325.0),
        _ => None,
    }
}
