use super::*;

impl AppState {
    pub fn update_stream_inspector_draft(
        &mut self,
        stream_id: &StreamId,
        field: StreamInspectorDraftField,
        raw_value: impl Into<String>,
    ) -> Option<StreamInspectorDraftUpdateResult> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return None;
        }

        let stream = self.workspace.document.flowsheet.streams.get(stream_id)?;
        if !stream_inspector_draft_fields(stream).contains(&field) {
            return None;
        }
        let raw_value = raw_value.into();
        let numeric_field = match field {
            StreamInspectorDraftField::TemperatureK => {
                Some(crate::variable_browser::VariableField::Temperature)
            }
            StreamInspectorDraftField::PressurePa => {
                Some(crate::variable_browser::VariableField::Pressure)
            }
            StreamInspectorDraftField::TotalMolarFlowMolS => {
                Some(crate::variable_browser::VariableField::MolarFlow)
            }
            _ => None,
        };
        if let Some(numeric_field) = numeric_field {
            let (is_dirty, validation) = self.update_numeric_inspector(
                crate::variable_browser::ObjectId::Stream(stream_id.clone()),
                numeric_field,
                raw_value,
            )?;
            return Some(StreamInspectorDraftUpdateResult {
                key: stream_inspector_draft_key(stream_id, &field),
                active_target,
                is_dirty,
                validation,
            });
        }
        let key = stream_inspector_draft_key(stream_id, &field);
        let (draft_value, is_dirty, validation) =
            stream_draft_value_from_raw(&field, stream, raw_value);

        if inspector_draft_needs_storage(&draft_value, is_dirty, validation) {
            self.workspace
                .drafts
                .fields
                .insert(key.clone(), draft_value);
        } else {
            self.workspace.drafts.fields.remove(&key);
        }

        Some(StreamInspectorDraftUpdateResult {
            key,
            active_target,
            is_dirty,
            validation,
        })
    }

    pub fn commit_stream_inspector_draft(
        &mut self,
        stream_id: &StreamId,
        field: StreamInspectorDraftField,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamInspectorDraftCommitResult>> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return Ok(None);
        }

        if !self
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(stream_id)
        {
            return Ok(None);
        }

        let key = stream_inspector_draft_key(stream_id, &field);
        let Some(draft_value) = self.workspace.drafts.fields.get(&key) else {
            return Ok(None);
        };
        if let DraftValue::Numeric(session) = draft_value {
            let id = session.variable().clone();
            let generation = session.generation();
            if session.validation().is_err() {
                return Ok(None);
            }
            let result = self
                .commit_numeric_edit(&id, generation, changed_at)
                .map_err(|error| RfError::invalid_input(error.to_string()))?;
            return Ok(result
                .command
                .map(|command| StreamInspectorDraftCommitResult {
                    key,
                    active_target,
                    command,
                    revision: result.revision,
                }));
        }
        let Some(command_value) = stream_command_value_from_draft(&field, draft_value)? else {
            return Ok(None);
        };

        let command = DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: field.command_field(),
            value: command_value,
        };
        let revision = self
            .workspace
            .commit_input_document_command(command.clone(), changed_at)?
            .revision;
        self.workspace.drafts.fields.remove(&key);
        self.refresh_run_panel_state();

        Ok(Some(StreamInspectorDraftCommitResult {
            key,
            active_target,
            command,
            revision,
        }))
    }

    pub fn discard_stream_inspector_draft(
        &mut self,
        stream_id: &StreamId,
        field: StreamInspectorDraftField,
    ) -> Option<StreamInspectorDraftDiscardResult> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return None;
        }

        let stream = self.workspace.document.flowsheet.streams.get(stream_id)?;
        if !stream_inspector_draft_fields(stream).contains(&field) {
            return None;
        }

        let key = stream_inspector_draft_key(stream_id, &field);
        self.workspace.drafts.fields.remove(&key)?;

        Some(StreamInspectorDraftDiscardResult { key, active_target })
    }

    pub fn commit_stream_inspector_drafts(
        &mut self,
        stream_id: &StreamId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamInspectorDraftBatchCommitResult>> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return Ok(None);
        }

        if !self
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(stream_id)
        {
            return Ok(None);
        }

        let stream = self
            .workspace
            .document
            .flowsheet
            .streams
            .get(stream_id)
            .expect("stream existence was checked above");
        let fields = stream_inspector_draft_fields(stream);
        self.workspace.refresh_numeric_edits();
        let mut keys = Vec::new();
        let mut values = Vec::new();

        for field in fields {
            let key = stream_inspector_draft_key(stream_id, &field);
            let Some(draft_value) = self.workspace.drafts.fields.get(&key) else {
                continue;
            };
            let Some(command_value) = stream_command_value_from_draft(&field, draft_value)? else {
                continue;
            };

            keys.push(key);
            values.push(StreamSpecificationValue {
                field: field.command_field(),
                value: command_value,
            });
        }

        if values.is_empty() {
            return Ok(None);
        }

        let command = stream_specification_command(stream_id, values);
        let revision = self
            .workspace
            .commit_input_document_command(command.clone(), changed_at)?
            .revision;
        for key in &keys {
            self.workspace.drafts.fields.remove(key);
        }
        self.refresh_run_panel_state();

        Ok(Some(StreamInspectorDraftBatchCommitResult {
            keys,
            active_target,
            command,
            revision,
        }))
    }

    pub fn discard_stream_inspector_drafts(
        &mut self,
        stream_id: &StreamId,
    ) -> Option<StreamInspectorDraftBatchDiscardResult> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return None;
        }

        let stream = self.workspace.document.flowsheet.streams.get(stream_id)?;
        let keys = stream_inspector_draft_fields(stream)
            .into_iter()
            .map(|field| stream_inspector_draft_key(stream_id, &field))
            .filter(|key| self.workspace.drafts.fields.contains_key(key))
            .collect::<Vec<_>>();
        if keys.is_empty() {
            return None;
        }

        for key in &keys {
            self.workspace.drafts.fields.remove(key);
        }

        Some(StreamInspectorDraftBatchDiscardResult {
            keys,
            active_target,
        })
    }

    pub fn normalize_stream_inspector_composition_drafts(
        &mut self,
        stream_id: &StreamId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamInspectorDraftBatchCommitResult>> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return Ok(None);
        }

        let stream = match self.workspace.document.flowsheet.streams.get(stream_id) {
            Some(stream) => stream,
            None => return Ok(None),
        };
        if stream.overall_mole_fractions.is_empty() {
            return Ok(None);
        }

        let mut entries = Vec::new();
        let mut has_composition_draft = false;
        for (component_id, original_value) in &stream.overall_mole_fractions {
            let field = StreamInspectorDraftField::OverallMoleFraction(component_id.clone());
            let key = stream_inspector_draft_key(stream_id, &field);
            let value = match self.workspace.drafts.fields.get(&key) {
                Some(draft_value) => {
                    has_composition_draft = true;
                    match stream_command_value_from_draft(&field, draft_value)? {
                        Some(CommandValue::Number(value)) => value,
                        _ => return Ok(None),
                    }
                }
                None => *original_value,
            };
            entries.push((component_id.clone(), key, *original_value, value));
        }

        let sum = entries.iter().map(|(_, _, _, value)| value).sum::<f64>();
        if !sum.is_finite() || sum <= 0.0 {
            return Ok(None);
        }

        let mut values = Vec::new();
        let mut keys = Vec::new();
        let mut changes_document = false;
        for (component_id, key, original_value, value) in entries {
            let normalized = value / sum;
            changes_document |= normalized != original_value;
            let field = StreamInspectorDraftField::OverallMoleFraction(component_id);
            let command_value = CommandValue::Number(normalized);
            keys.push(key);
            values.push(StreamSpecificationValue {
                field: field.command_field(),
                value: command_value,
            });
        }

        if !changes_document && !has_composition_draft {
            return Ok(None);
        }

        let command = stream_specification_command(stream_id, values);
        let revision = self
            .workspace
            .commit_input_document_command(command.clone(), changed_at)?
            .revision;
        for key in &keys {
            self.workspace.drafts.fields.remove(key);
        }
        self.refresh_run_panel_state();

        Ok(Some(StreamInspectorDraftBatchCommitResult {
            keys,
            active_target,
            command,
            revision,
        }))
    }

    pub fn add_stream_inspector_composition_component(
        &mut self,
        stream_id: &StreamId,
        component_id: ComponentId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamInspectorCompositionComponentAddResult>> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return Ok(None);
        }

        let Some(stream) = self.workspace.document.flowsheet.streams.get(stream_id) else {
            return Ok(None);
        };
        if !self
            .workspace
            .document
            .flowsheet
            .components
            .contains_key(&component_id)
            || stream.overall_mole_fractions.contains_key(&component_id)
        {
            return Ok(None);
        }

        let initial_fraction = if stream.overall_mole_fractions.is_empty() {
            1.0
        } else {
            0.0
        };
        let field = StreamInspectorDraftField::OverallMoleFraction(component_id.clone());
        let key = stream_inspector_draft_key(stream_id, &field);
        let command_value = CommandValue::Number(initial_fraction);

        let command = DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: field.command_field(),
            value: command_value,
        };
        let revision = self
            .workspace
            .commit_input_document_command(command.clone(), changed_at)?
            .revision;
        self.workspace.drafts.fields.remove(&key);
        self.refresh_run_panel_state();

        Ok(Some(StreamInspectorCompositionComponentAddResult {
            key,
            active_target,
            component_id,
            command,
            revision,
        }))
    }

    pub fn remove_stream_inspector_composition_component(
        &mut self,
        stream_id: &StreamId,
        component_id: ComponentId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamInspectorCompositionComponentRemoveResult>> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return Ok(None);
        }

        let Some(stream) = self.workspace.document.flowsheet.streams.get(stream_id) else {
            return Ok(None);
        };
        if !stream.overall_mole_fractions.contains_key(&component_id)
            || stream.overall_mole_fractions.len() <= 1
        {
            return Ok(None);
        }

        let field = StreamInspectorDraftField::OverallMoleFraction(component_id.clone());
        let key = stream_inspector_draft_key(stream_id, &field);
        let mut next_flowsheet = self.workspace.document.flowsheet.clone();
        let next_stream = next_flowsheet
            .streams
            .get_mut(stream_id)
            .ok_or_else(|| RfError::missing_entity("stream", stream_id))?;
        next_stream.overall_mole_fractions.remove(&component_id);
        validate_stream_overall_mole_fractions(next_stream)?;

        let command = DocumentCommand::RemoveStreamCompositionComponent {
            stream_id: stream_id.clone(),
            component_id: component_id.clone(),
        };
        let revision = self.workspace.commit_inspector_document_change(
            command.clone(),
            next_flowsheet,
            changed_at,
        );
        self.workspace.drafts.fields.remove(&key);
        self.refresh_run_panel_state();

        Ok(Some(StreamInspectorCompositionComponentRemoveResult {
            key,
            active_target,
            component_id,
            command,
            revision,
        }))
    }
}

fn inspector_draft_needs_storage(
    draft_value: &DraftValue,
    is_dirty: bool,
    validation: DraftValidationState,
) -> bool {
    if is_dirty || validation == DraftValidationState::Invalid {
        return true;
    }

    match draft_value {
        DraftValue::Numeric(session) => session.is_pending(),
        DraftValue::Text(draft) | DraftValue::Number(draft) | DraftValue::Choice(draft) => {
            draft.current != draft.original
        }
    }
}

fn stream_draft_value_from_raw(
    field: &StreamInspectorDraftField,
    stream: &MaterialStreamState,
    raw_value: String,
) -> (DraftValue, bool, DraftValidationState) {
    match field {
        StreamInspectorDraftField::Name => {
            let original = stream.name.clone();
            let validation = if raw_value.trim().is_empty() {
                DraftValidationState::Invalid
            } else {
                DraftValidationState::Valid
            };
            let is_dirty = raw_value != original;
            let draft = FieldDraft {
                original,
                current: raw_value,
                is_dirty,
                validation,
            };
            (DraftValue::Text(draft), is_dirty, validation)
        }
        StreamInspectorDraftField::TemperatureK => {
            stream_number_draft_value(stream.temperature_k, raw_value, |value| {
                value.is_finite() && value > 0.0
            })
        }
        StreamInspectorDraftField::PressurePa => {
            stream_number_draft_value(stream.pressure_pa, raw_value, |value| {
                value.is_finite() && value > 0.0
            })
        }
        StreamInspectorDraftField::TotalMolarFlowMolS => {
            stream_number_draft_value(stream.total_molar_flow_mol_s, raw_value, |value| {
                value.is_finite() && value >= 0.0
            })
        }
        StreamInspectorDraftField::OverallMoleFraction(component_id) => {
            let original = stream
                .overall_mole_fractions
                .get(component_id)
                .copied()
                .unwrap_or(0.0);
            stream_number_draft_value(original, raw_value, |value| {
                is_valid_stream_scalar_value(field, value)
            })
        }
    }
}

fn stream_number_draft_value<F>(
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

fn stream_command_value_from_draft(
    field: &StreamInspectorDraftField,
    draft_value: &DraftValue,
) -> RfResult<Option<CommandValue>> {
    if let DraftValue::Numeric(session) = draft_value {
        return Ok(session.candidate_si().ok().map(CommandValue::Number));
    }
    match (field, draft_value) {
        (StreamInspectorDraftField::Name, DraftValue::Text(draft)) => {
            if !draft.is_dirty || draft.validation != DraftValidationState::Valid {
                return Ok(None);
            }
            Ok(Some(CommandValue::Text(draft.current.clone())))
        }
        (
            StreamInspectorDraftField::TemperatureK
            | StreamInspectorDraftField::PressurePa
            | StreamInspectorDraftField::TotalMolarFlowMolS
            | StreamInspectorDraftField::OverallMoleFraction(_),
            DraftValue::Number(draft),
        ) => {
            if !draft.is_dirty || draft.validation != DraftValidationState::Valid {
                return Ok(None);
            }
            let value = draft.current.trim().parse::<f64>().map_err(|_| {
                RfError::invalid_input(format!(
                    "stream inspector draft `{}` is not a valid number",
                    draft.current
                ))
            })?;
            if !is_valid_stream_scalar_value(field, value) {
                return Ok(None);
            }
            Ok(Some(CommandValue::Number(value)))
        }
        _ => Ok(None),
    }
}

fn stream_inspector_draft_fields(stream: &MaterialStreamState) -> Vec<StreamInspectorDraftField> {
    let mut fields = vec![
        StreamInspectorDraftField::Name,
        StreamInspectorDraftField::TemperatureK,
        StreamInspectorDraftField::PressurePa,
        StreamInspectorDraftField::TotalMolarFlowMolS,
    ];
    fields.extend(
        stream
            .overall_mole_fractions
            .keys()
            .cloned()
            .map(StreamInspectorDraftField::OverallMoleFraction),
    );
    fields
}

fn stream_specification_command(
    stream_id: &StreamId,
    values: Vec<StreamSpecificationValue>,
) -> DocumentCommand {
    let mut values = values;
    if values.len() == 1 {
        let value = values
            .pop()
            .expect("single stream specification value should exist");
        DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: value.field,
            value: value.value,
        }
    } else {
        DocumentCommand::SetStreamSpecifications {
            stream_id: stream_id.clone(),
            values,
        }
    }
}

pub(super) fn apply_stream_specification_value(
    flowsheet: &mut Flowsheet,
    stream_id: &StreamId,
    field: &StreamInspectorDraftField,
    value: &CommandValue,
) -> RfResult<()> {
    let stream = flowsheet
        .streams
        .get_mut(stream_id)
        .ok_or_else(|| RfError::missing_entity("stream", stream_id))?;

    match (field, value) {
        (StreamInspectorDraftField::Name, CommandValue::Text(value)) => {
            if value.trim().is_empty() {
                return Err(RfError::invalid_input("stream name cannot be empty"));
            }
            stream.name = value.clone();
        }
        (StreamInspectorDraftField::TemperatureK, CommandValue::Number(value))
            if is_valid_stream_scalar_value(field, *value) =>
        {
            stream.temperature_k = *value;
        }
        (StreamInspectorDraftField::PressurePa, CommandValue::Number(value))
            if is_valid_stream_scalar_value(field, *value) =>
        {
            stream.pressure_pa = *value;
        }
        (StreamInspectorDraftField::TotalMolarFlowMolS, CommandValue::Number(value))
            if is_valid_stream_scalar_value(field, *value) =>
        {
            stream.total_molar_flow_mol_s = *value;
        }
        (
            StreamInspectorDraftField::OverallMoleFraction(component_id),
            CommandValue::Number(value),
        ) if is_valid_stream_scalar_value(field, *value) => {
            stream
                .overall_mole_fractions
                .insert(component_id.clone(), *value);
            validate_stream_overall_mole_fractions(stream)?;
        }
        _ => {
            return Err(RfError::invalid_input(format!(
                "stream field `{}` cannot be set from value `{value:?}`",
                field.command_field()
            )));
        }
    }

    Ok(())
}

fn is_valid_stream_scalar_value(field: &StreamInspectorDraftField, value: f64) -> bool {
    match field {
        StreamInspectorDraftField::Name => false,
        StreamInspectorDraftField::TemperatureK | StreamInspectorDraftField::PressurePa => {
            value.is_finite() && value > 0.0
        }
        StreamInspectorDraftField::TotalMolarFlowMolS => value.is_finite() && value >= 0.0,
        StreamInspectorDraftField::OverallMoleFraction(_) => {
            value.is_finite() && (0.0..=1.0).contains(&value)
        }
    }
}

fn validate_stream_overall_mole_fractions(stream: &MaterialStreamState) -> RfResult<()> {
    if stream.overall_mole_fractions.is_empty() {
        return Err(RfError::invalid_input(format!(
            "stream `{}` must define at least one overall mole fraction entry",
            stream.id
        )));
    }

    let sum = stream
        .overall_mole_fractions
        .values()
        .try_fold(0.0, |sum, value| {
            (value.is_finite() && (0.0..=1.0).contains(value)).then_some(sum + value)
        })
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "stream `{}` overall mole fractions must be finite values between zero and one",
                stream.id
            ))
        })?;
    if sum <= 0.0 {
        return Err(RfError::invalid_input(format!(
            "stream `{}` overall mole fractions must sum to a positive finite value",
            stream.id
        )));
    }

    Ok(())
}
