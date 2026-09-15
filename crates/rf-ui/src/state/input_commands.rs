use super::*;

pub(crate) struct InputCommandCommit {
    pub revision: u64,
    pub changed: bool,
}

impl WorkspaceState {
    /// Prepare all values against a clone before committing one existing history transaction.
    /// Inspector callers own their drafts; non-UI callers must check draft/revision conflicts.
    pub(crate) fn commit_input_document_command(
        &mut self,
        command: DocumentCommand,
        changed_at: DateTimeUtc,
    ) -> RfResult<InputCommandCommit> {
        let mut next = self.document.flowsheet.clone();
        match &command {
            DocumentCommand::RenameUnit { unit_id, new_name } => {
                unit_inspector::apply_unit_parameter_value(
                    &mut next,
                    unit_id,
                    &UnitInspectorDraftField::Name,
                    &CommandValue::Text(new_name.clone()),
                )?;
            }
            DocumentCommand::SetUnitParameter {
                unit_id,
                parameter,
                value,
            } => {
                let field = UnitInspectorDraftField::from_static_key_segment(parameter)
                    .ok_or_else(|| RfError::invalid_input("unsupported unit input field"))?;
                unit_inspector::apply_unit_parameter_value(&mut next, unit_id, &field, value)?;
            }
            DocumentCommand::SetStreamSpecification {
                stream_id,
                field,
                value,
            } => {
                apply_stream_input(&mut next, stream_id, field, value)?;
            }
            DocumentCommand::SetStreamSpecifications { stream_id, values } => {
                if values.is_empty() {
                    return Err(RfError::invalid_input(
                        "stream input transaction cannot be empty",
                    ));
                }
                for value in values {
                    apply_stream_input(&mut next, stream_id, &value.field, &value.value)?;
                }
            }
            _ => return Err(RfError::invalid_input("command is not an input edit")),
        }
        if next == self.document.flowsheet {
            return Ok(InputCommandCommit {
                revision: self.document.revision,
                changed: false,
            });
        }
        Ok(InputCommandCommit {
            revision: self.commit_inspector_document_change(command, next, changed_at),
            changed: true,
        })
    }
}

fn apply_stream_input(
    flowsheet: &mut Flowsheet,
    stream_id: &StreamId,
    field: &str,
    value: &CommandValue,
) -> RfResult<()> {
    let field = StreamInspectorDraftField::from_static_key_segment(field)
        .or_else(|| {
            field
                .strip_prefix("overall_mole_fraction:")
                .filter(|component| !component.is_empty())
                .map(|component| {
                    StreamInspectorDraftField::OverallMoleFraction(ComponentId::new(component))
                })
        })
        .ok_or_else(|| RfError::invalid_input("unsupported stream input field"))?;
    apply_stream_specification_value(flowsheet, stream_id, &field, value)
}
