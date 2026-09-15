//! Typed input writes for application callers, sharing Inspector validation and transactions.
//! This is an internal Rust API, not a serialized public protocol or action executor.

use crate::variable_browser::{
    BrowseError, ObjectId, VariableBrowser, VariableField, VariableId, VariableSection,
    VariableType, VariableValue,
};
use crate::{
    AppState, CommandValue, DateTimeUtc, DocumentCommand, StreamInspectorDraftField,
    UnitInspectorDraftField,
};
use rf_types::RfError;

#[derive(Debug, Clone, PartialEq)]
pub struct VariableWriteRequest {
    pub variable: VariableId,
    /// Required optimistic concurrency check, even for an otherwise identical value.
    pub expected_revision: u64,
    /// Numeric values use the descriptor's SI unit and molar composition basis.
    pub value: VariableValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableWriteError {
    Lookup(BrowseError),
    ReadOnly,
    RevisionConflict { expected: u64, actual: u64 },
    PendingDrafts,
    TypeMismatch { expected: VariableType },
    Rejected(RfError),
}

impl std::fmt::Display for VariableWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lookup(error) => write!(f, "variable lookup failed: {error:?}"),
            Self::ReadOnly => f.write_str("variable is read-only"),
            Self::RevisionConflict { expected, actual } => {
                write!(f, "expected document revision {expected}, found {actual}")
            }
            Self::PendingDrafts => f.write_str("commit or discard Inspector drafts before writing"),
            Self::TypeMismatch { expected } => write!(f, "variable requires {expected:?}"),
            Self::Rejected(error) => write!(f, "input rejected: {error}"),
        }
    }
}

impl std::error::Error for VariableWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rejected(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableWriteReceipt {
    pub variable: VariableId,
    pub revision: u64,
    /// None means an identical committed value: no new history or result invalidation.
    pub command: Option<DocumentCommand>,
}

impl AppState {
    /// Does not select objects, discard drafts, solve, save, or write result snapshots.
    /// All rejection paths leave document, history, drafts and solve state unchanged.
    pub fn write_variable(
        &mut self,
        request: VariableWriteRequest,
        changed_at: DateTimeUtc,
    ) -> Result<VariableWriteReceipt, VariableWriteError> {
        use VariableWriteError::*;
        let browser = VariableBrowser::new(&self.workspace.document, None, None);
        if &request.variable.document != browser.document_id() {
            return Err(Lookup(BrowseError::DifferentDocument));
        }
        let revision = self.workspace.document.revision;
        if request.expected_revision != revision {
            return Err(RevisionConflict {
                expected: request.expected_revision,
                actual: revision,
            });
        }
        browser.object(&request.variable.object).map_err(Lookup)?;
        if request.variable.section == VariableSection::Results {
            return Err(ReadOnly);
        }
        let descriptor = browser.read(&request.variable).map_err(Lookup)?;
        if descriptor.write_via.is_none() {
            return Err(ReadOnly);
        }
        if !self.workspace.drafts.fields.is_empty() {
            return Err(PendingDrafts);
        }
        if !matches!(
            (&request.value, descriptor.value_type),
            (VariableValue::Number(_), VariableType::Number)
                | (VariableValue::Text(_), VariableType::Text)
        ) {
            return Err(TypeMismatch {
                expected: descriptor.value_type,
            });
        }
        if let VariableField::MoleFraction(component) = &request.variable.field
            && !self
                .workspace
                .document
                .flowsheet
                .components
                .contains_key(component)
        {
            return Err(Rejected(RfError::missing_entity(
                "project component",
                component,
            )));
        }
        let command = input_command(&request.variable, request.value.clone())?;
        let committed = self
            .workspace
            .commit_input_document_command(command.clone(), changed_at)
            .map_err(Rejected)?;
        self.refresh_run_panel_state();
        Ok(VariableWriteReceipt {
            variable: request.variable,
            revision: committed.revision,
            command: committed.changed.then_some(command),
        })
    }
}

fn input_command(
    id: &VariableId,
    value: VariableValue,
) -> Result<DocumentCommand, VariableWriteError> {
    let value = match value {
        VariableValue::Number(v) => CommandValue::Number(v),
        VariableValue::Text(v) => CommandValue::Text(v),
    };
    match &id.object {
        ObjectId::Unit(unit_id) => {
            if let (VariableField::Name, CommandValue::Text(new_name)) = (&id.field, &value) {
                return Ok(DocumentCommand::RenameUnit {
                    unit_id: unit_id.clone(),
                    new_name: new_name.clone(),
                });
            }
            let field = match id.field {
                VariableField::OutletTemperature => UnitInspectorDraftField::OutletTemperatureK,
                VariableField::OutletPressure => UnitInspectorDraftField::OutletPressurePa,
                _ => return Err(VariableWriteError::ReadOnly),
            };
            Ok(DocumentCommand::SetUnitParameter {
                unit_id: unit_id.clone(),
                parameter: field.command_parameter(),
                value,
            })
        }
        ObjectId::Stream(stream_id) => {
            let field = match &id.field {
                VariableField::Name => StreamInspectorDraftField::Name,
                VariableField::Temperature => StreamInspectorDraftField::TemperatureK,
                VariableField::Pressure => StreamInspectorDraftField::PressurePa,
                VariableField::MolarFlow => StreamInspectorDraftField::TotalMolarFlowMolS,
                VariableField::MoleFraction(c) => {
                    StreamInspectorDraftField::OverallMoleFraction(c.clone())
                }
                _ => return Err(VariableWriteError::ReadOnly),
            };
            Ok(DocumentCommand::SetStreamSpecification {
                stream_id: stream_id.clone(),
                field: field.command_field(),
                value,
            })
        }
    }
}
