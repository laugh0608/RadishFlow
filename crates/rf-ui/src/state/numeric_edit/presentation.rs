use super::*;

/// Orthogonal facts for numeric controls; no widget colors or status-label parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericFieldPresentation {
    pub variable: VariableId,
    pub quantity: QuantityKind,
    pub source: NumericFieldSource,
    pub display_unit: MeasurementUnit,
    pub input_unit: MeasurementUnit,
    pub text: String,
    pub committed_text: String,
    pub generation: Option<u64>,
    pub pending: bool,
    pub composing: bool,
    pub issue: Option<NumericFieldIssue>,
    pub can_apply: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericFieldSource {
    Specified,
    Inherited,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumericFieldIssue {
    Incomplete,
    Conflict,
    Rejected(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumericEditCommand {
    Begin(VariableId),
    Edit {
        variable: VariableId,
        generation: u64,
        event: NumericEditEvent,
    },
    Commit {
        variable: VariableId,
        generation: u64,
    },
    Cancel {
        variable: VariableId,
        generation: u64,
    },
}

impl WorkspaceState {
    pub fn numeric_field_presentation(
        &self,
        id: &VariableId,
    ) -> Result<NumericFieldPresentation, NumericEditError> {
        let (quantity, baseline) = self.numeric_field(id)?;
        let display_unit = self.project_presentation.display_units().unit_for(quantity);
        let committed_text = baseline
            .value
            .map(|si| {
                from_canonical(si, quantity, display_unit).map(super::super::format_edit_number)
            })
            .transpose()
            .map_err(NumericEditError::Conversion)?
            .unwrap_or_default();
        let session = match self.numeric_edit(id) {
            Ok(session) => Some(session),
            Err(NumericEditError::MissingSession) => None,
            Err(error) => return Err(error),
        };
        let issue = session
            .and_then(|s| s.validation().as_ref().err())
            .map(|error| match error {
                NumericEditError::Incomplete => NumericFieldIssue::Incomplete,
                NumericEditError::FieldConflict => NumericFieldIssue::Conflict,
                error => NumericFieldIssue::Rejected(error.to_string()),
            });
        Ok(NumericFieldPresentation {
            variable: id.clone(),
            quantity,
            source: if baseline.value.is_none() {
                NumericFieldSource::Missing
            } else if baseline.explicit {
                NumericFieldSource::Specified
            } else {
                NumericFieldSource::Inherited
            },
            display_unit,
            input_unit: session.map_or(display_unit, NumericEditSession::input_unit),
            text: session.map_or_else(|| committed_text.clone(), |s| s.raw_text().to_owned()),
            committed_text,
            generation: session.map(NumericEditSession::generation),
            pending: session.is_some_and(NumericEditSession::is_pending),
            composing: session.is_some_and(|s| s.composition_original.is_some()),
            can_apply: session
                .is_some_and(|s| s.validation.is_ok() && (s.is_pending() || !baseline.explicit)),
            can_undo: session.is_some_and(|s| !s.undo.is_empty()),
            can_redo: session.is_some_and(|s| !s.redo.is_empty()),
            issue,
        })
    }
}

impl AppState {
    pub fn dispatch_numeric_edit(
        &mut self,
        command: NumericEditCommand,
        changed_at: DateTimeUtc,
    ) -> Result<Option<u64>, NumericEditError> {
        match command {
            NumericEditCommand::Begin(id) => self.begin_numeric_edit(id, None).map(Some),
            NumericEditCommand::Edit {
                variable,
                generation,
                event,
            } => self.edit_numeric(&variable, generation, event).map(Some),
            NumericEditCommand::Commit {
                variable,
                generation,
            } => self
                .commit_numeric_edit(&variable, generation, changed_at)
                .map(|_| None),
            NumericEditCommand::Cancel {
                variable,
                generation,
            } => self
                .cancel_numeric_edit(&variable, generation)
                .map(|()| None),
        }
    }
}
