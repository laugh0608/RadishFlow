use super::*;
use crate::variable_browser::{
    ObjectId, VariableBrowser, VariableField, VariableSection, VariableValue,
};
use crate::variable_commands::input_command;

#[derive(Debug, Clone, PartialEq)]
pub struct NumericEditReceipt {
    pub variable: VariableId,
    pub revision: u64,
    pub command: Option<DocumentCommand>,
}

pub(super) fn field_key(id: &VariableId) -> Result<String, NumericEditError> {
    if id.section != VariableSection::Inputs {
        return Err(NumericEditError::UnsupportedField);
    }
    Ok(match (&id.object, &id.field) {
        (ObjectId::Unit(unit), VariableField::OutletTemperature) => {
            unit_inspector_draft_key(unit, &UnitInspectorDraftField::OutletTemperatureK)
        }
        (ObjectId::Unit(unit), VariableField::OutletPressure) => {
            unit_inspector_draft_key(unit, &UnitInspectorDraftField::OutletPressurePa)
        }
        (ObjectId::Stream(stream), field) => stream_inspector_draft_key(
            stream,
            &match field {
                VariableField::Temperature => StreamInspectorDraftField::TemperatureK,
                VariableField::Pressure => StreamInspectorDraftField::PressurePa,
                VariableField::MolarFlow => StreamInspectorDraftField::TotalMolarFlowMolS,
                _ => return Err(NumericEditError::UnsupportedField),
            },
        ),
        _ => return Err(NumericEditError::UnsupportedField),
    })
}

impl WorkspaceState {
    pub(super) fn numeric_field(
        &self,
        id: &VariableId,
    ) -> Result<(QuantityKind, FieldBaseline), NumericEditError> {
        field_key(id)?;
        let descriptor = VariableBrowser::new(&self.document, None, None)
            .read(id)
            .map_err(NumericEditError::Lookup)?;
        if descriptor.write_via.is_none() {
            return Err(NumericEditError::UnsupportedField);
        }
        let quantity = descriptor
            .quantity
            .ok_or(NumericEditError::UnsupportedField)?;
        let (value, explicit) = match (&id.object, &id.field) {
            (ObjectId::Unit(unit), field) => {
                let field = match field {
                    VariableField::OutletTemperature => UnitInspectorDraftField::OutletTemperatureK,
                    VariableField::OutletPressure => UnitInspectorDraftField::OutletPressurePa,
                    _ => return Err(NumericEditError::UnsupportedField),
                };
                (
                    unit_inspector_parameter_value(&self.document.flowsheet, unit, &field),
                    unit_inspector_parameter_is_explicit(
                        &self.document.flowsheet.units[unit],
                        &field,
                    ),
                )
            }
            _ => (
                match descriptor.value {
                    Some(VariableValue::Number(v)) => Some(v),
                    _ => None,
                },
                true,
            ),
        };
        let inherited_stream = if !explicit {
            match &id.object {
                ObjectId::Unit(unit) => self.document.flowsheet.units[unit]
                    .ports
                    .iter()
                    .find(|port| {
                        port.direction == PortDirection::Outlet && port.kind == PortKind::Material
                    })
                    .and_then(|port| port.stream_id.as_ref())
                    .filter(|stream| self.document.flowsheet.streams.contains_key(*stream))
                    .cloned(),
                _ => None,
            }
        } else {
            None
        };
        Ok((
            quantity,
            FieldBaseline {
                value,
                explicit,
                inherited_stream,
            },
        ))
    }

    fn next_numeric_generation(&mut self) -> u64 {
        self.drafts.next_numeric_generation = self
            .drafts
            .next_numeric_generation
            .checked_add(1)
            .expect("numeric edit generation exhausted");
        self.drafts.next_numeric_generation
    }

    pub fn numeric_edit(&self, id: &VariableId) -> Result<&NumericEditSession, NumericEditError> {
        match self.drafts.fields.get(&field_key(id)?) {
            Some(DraftValue::Numeric(session)) if session.variable == *id => Ok(session),
            _ => Err(NumericEditError::MissingSession),
        }
    }

    fn validate_numeric_edit(&self, session: &NumericEditSession) -> Result<(), NumericEditError> {
        let (_, baseline) = self.numeric_field(&session.variable)?;
        if baseline != session.baseline {
            return Err(NumericEditError::FieldConflict);
        }
        if session.composition_original.is_some() {
            return Err(NumericEditError::Incomplete);
        }
        let value = session.parsed_si()?;
        let command = input_command(&session.variable, VariableValue::Number(value))
            .map_err(|_| NumericEditError::UnsupportedField)?;
        self.prepare_input_document_command(&command)
            .map_err(NumericEditError::Rejected)?;
        Ok(())
    }

    pub(in crate::state) fn refresh_numeric_edits(&mut self) {
        let updates: Vec<_> = self
            .drafts
            .fields
            .iter()
            .filter_map(|(key, draft)| {
                let DraftValue::Numeric(session) = draft else {
                    return None;
                };
                Some((key.clone(), self.validate_numeric_edit(session)))
            })
            .collect();
        for (key, validation) in updates {
            if let Some(DraftValue::Numeric(session)) = self.drafts.fields.get_mut(&key) {
                session.validation = validation;
                if session.validation.is_ok() {
                    session.baseline_revision = self.document.revision;
                }
            }
        }
    }
}

impl AppState {
    /// Native controls use `None` for the project's display choice. The legacy SI command
    /// bridge passes its canonical unit explicitly to preserve that calling contract.
    pub fn begin_numeric_edit(
        &mut self,
        id: VariableId,
        input_unit: Option<MeasurementUnit>,
    ) -> Result<u64, NumericEditError> {
        self.begin_numeric_edit_with_view(id, input_unit, None)
    }

    pub fn begin_numeric_edit_in_view(
        &mut self,
        id: VariableId,
        view: DisplayUnitViewId,
    ) -> Result<u64, NumericEditError> {
        self.begin_numeric_edit_with_view(id, None, Some(view))
    }

    fn begin_numeric_edit_with_view(
        &mut self,
        id: VariableId,
        input_unit: Option<MeasurementUnit>,
        view: Option<DisplayUnitViewId>,
    ) -> Result<u64, NumericEditError> {
        let key = field_key(&id)?;
        let (quantity, baseline) = self.workspace.numeric_field(&id)?;
        if let Ok(session) = self.workspace.numeric_edit(&id) {
            return Ok(session.generation);
        }
        let origin = if input_unit.is_some() {
            InputUnitOrigin::Explicit
        } else {
            InputUnitOrigin::Display
        };
        let unit = input_unit.unwrap_or_else(|| {
            self.workspace
                .project_presentation
                .effective_unit(view, quantity)
        });
        let display = from_canonical(baseline.value.unwrap_or(0.0), quantity, unit)
            .map_err(NumericEditError::Conversion)?;
        let current = EditContent {
            raw: baseline
                .value
                .map_or_else(String::new, |_| super::super::format_edit_number(display)),
            unit,
            origin,
            parsed: baseline
                .value
                .map_or(NumericParseOutcome::Incomplete, |si| {
                    NumericParseOutcome::Value { si, unit, origin }
                }),
            precision: NumericPrecisionSource::OriginalSi,
            pending: false,
        };
        let generation = self.workspace.next_numeric_generation();
        let mut session = NumericEditSession {
            variable: id,
            quantity,
            generation,
            baseline_revision: self.workspace.document.revision,
            baseline,
            current,
            undo: Vec::new(),
            redo: Vec::new(),
            validation: Ok(()),
            text_group: None,
            composition_original: None,
        };
        session.validation = self.workspace.validate_numeric_edit(&session);
        self.workspace
            .drafts
            .fields
            .insert(key, DraftValue::Numeric(Box::new(session)));
        Ok(generation)
    }

    pub fn edit_numeric(
        &mut self,
        id: &VariableId,
        generation: u64,
        event: NumericEditEvent,
    ) -> Result<u64, NumericEditError> {
        let mut session = self.checked_numeric_session(id, generation)?.clone();
        let group = match &event {
            NumericEditEvent::ReplaceTextGrouped { group, .. } => Some(*group),
            _ => None,
        };
        let coalesce = group.is_some() && group == session.text_group;
        session.text_group = group;
        let finishing_composition = matches!(event, NumericEditEvent::CommitComposition(_));
        let event = match event {
            NumericEditEvent::CommitComposition(raw) => {
                if let Some(original) = session.composition_original.take() {
                    session.current = original;
                }
                NumericEditEvent::ReplaceText(raw)
            }
            event => event,
        };
        if session.composition_original.is_some()
            && !matches!(
                event,
                NumericEditEvent::PreviewComposition(_) | NumericEditEvent::CancelComposition
            )
        {
            return Err(NumericEditError::Incomplete);
        }
        match event {
            NumericEditEvent::ReplaceText(raw)
            | NumericEditEvent::ReplaceTextGrouped { raw, .. } => {
                if raw == session.current.raw && !finishing_composition {
                    return Ok(generation);
                }
                let parsed = parse_numeric_input(
                    &raw,
                    session.quantity,
                    session.current.unit,
                    session.current.origin,
                );
                let (unit, origin) = match &parsed {
                    NumericParseOutcome::Value { unit, origin, .. } => (*unit, *origin),
                    _ => (session.current.unit, session.current.origin),
                };
                let next = EditContent {
                    raw,
                    unit,
                    origin,
                    parsed,
                    precision: NumericPrecisionSource::ParsedText,
                    pending: true,
                };
                if coalesce {
                    session.current = next;
                    session.redo.clear();
                } else {
                    session.replace(next);
                }
            }
            NumericEditEvent::PreviewComposition(raw) => {
                if session.composition_original.is_none() {
                    session.composition_original = Some(session.current.clone());
                }
                session.current.raw = raw;
                session.current.pending = true;
                session.current.parsed = NumericParseOutcome::Incomplete;
            }
            NumericEditEvent::CancelComposition => {
                if let Some(original) = session.composition_original.take() {
                    session.current = original;
                }
            }
            NumericEditEvent::CommitComposition(_) => {
                unreachable!("composition commit normalized above")
            }
            NumericEditEvent::SelectInputUnit(unit) => {
                // Reject invalid/incomplete drafts without mutating text, units, history or generation.
                let si = match session.current.parsed {
                    NumericParseOutcome::Incomplete
                        if !session.current.pending && session.baseline.value.is_none() =>
                    {
                        None
                    }
                    _ => Some(session.candidate_si()?),
                };
                let display = from_canonical(si.unwrap_or(0.0), session.quantity, unit)
                    .map_err(NumericEditError::Conversion)?;
                let mut next = session.current.clone();
                next.raw =
                    si.map_or_else(String::new, |_| super::super::format_edit_number(display));
                next.unit = unit;
                next.origin = InputUnitOrigin::Explicit;
                next.pending = true;
                next.parsed = si.map_or(NumericParseOutcome::Incomplete, |si| {
                    NumericParseOutcome::Value {
                        si,
                        unit,
                        origin: InputUnitOrigin::Explicit,
                    }
                });
                session.replace(next);
            }
            NumericEditEvent::Undo => {
                let Some(previous) = session.undo.pop() else {
                    return Ok(generation);
                };
                session
                    .redo
                    .push(std::mem::replace(&mut session.current, previous));
            }
            NumericEditEvent::Redo => {
                let Some(next) = session.redo.pop() else {
                    return Ok(generation);
                };
                session
                    .undo
                    .push(std::mem::replace(&mut session.current, next));
            }
            NumericEditEvent::AdoptValue => {
                let mut next = session.current.clone();
                next.pending = true;
                session.replace(next);
            }
            NumericEditEvent::Rebase => {
                let (_, baseline) = self.workspace.numeric_field(id)?;
                session.baseline = baseline;
                session.baseline_revision = self.workspace.document.revision;
            }
        }
        session.validation = self.workspace.validate_numeric_edit(&session);
        session.generation = self.workspace.next_numeric_generation();
        let generation = session.generation;
        self.workspace
            .drafts
            .fields
            .insert(field_key(id)?, DraftValue::Numeric(Box::new(session)));
        Ok(generation)
    }

    fn checked_numeric_session(
        &self,
        id: &VariableId,
        generation: u64,
    ) -> Result<&NumericEditSession, NumericEditError> {
        let session = self.workspace.numeric_edit(id)?;
        if session.generation != generation {
            return Err(NumericEditError::StaleGeneration {
                expected: generation,
                actual: session.generation,
            });
        }
        Ok(session)
    }

    pub fn cancel_numeric_edit(
        &mut self,
        id: &VariableId,
        generation: u64,
    ) -> Result<(), NumericEditError> {
        self.checked_numeric_session(id, generation)?;
        self.workspace.drafts.fields.remove(&field_key(id)?);
        Ok(())
    }

    pub fn commit_numeric_edit(
        &mut self,
        id: &VariableId,
        generation: u64,
        changed_at: DateTimeUtc,
    ) -> Result<NumericEditReceipt, NumericEditError> {
        let session = self.checked_numeric_session(id, generation)?;
        self.workspace.validate_numeric_edit(session)?;
        let command = input_command(id, VariableValue::Number(session.parsed_si()?))
            .map_err(|_| NumericEditError::UnsupportedField)?;
        let result = self
            .workspace
            .commit_input_document_command(command.clone(), changed_at)
            .map_err(NumericEditError::Rejected)?;
        self.workspace.drafts.fields.remove(&field_key(id)?);
        self.refresh_run_panel_state();
        Ok(NumericEditReceipt {
            variable: id.clone(),
            revision: result.revision,
            command: result.changed.then_some(command),
        })
    }
}
