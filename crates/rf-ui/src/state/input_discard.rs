use super::*;

/// A confirmation is tied to the document and every input, including numeric edit
/// generations. Reopening a field with identical text must invalidate old consent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEditCheckpoint {
    document_id: String,
    revision: u64,
    fields: BTreeMap<String, InputEditVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InputEditVersion {
    Numeric(u64),
    Text(FieldDraft<String>),
    Number(FieldDraft<String>),
    Choice(FieldDraft<String>),
}

impl InputEditCheckpoint {
    pub fn capture(
        document_id: impl Into<String>,
        revision: u64,
        fields: &BTreeMap<String, DraftValue>,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            revision,
            fields: fields
                .iter()
                .map(|(key, value)| {
                    let version = match value {
                        DraftValue::Numeric(session) => {
                            InputEditVersion::Numeric(session.generation())
                        }
                        DraftValue::Text(draft) => InputEditVersion::Text(draft.clone()),
                        DraftValue::Number(draft) => InputEditVersion::Number(draft.clone()),
                        DraftValue::Choice(draft) => InputEditVersion::Choice(draft.clone()),
                    };
                    (key.clone(), version)
                })
                .collect(),
        }
    }

    pub fn matches(&self, workspace: &WorkspaceState) -> bool {
        *self
            == Self::capture(
                workspace.document.metadata.document_id.as_str(),
                workspace.document.revision,
                &workspace.drafts.fields,
            )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputDiscardScope {
    All,
    Object(InspectorTarget),
}

impl InputDiscardScope {
    pub fn includes(&self, key: &str, value: &DraftValue) -> bool {
        let Self::Object(target) = self else {
            return true;
        };
        match value {
            DraftValue::Numeric(session) => match (target, &session.variable().object) {
                (InspectorTarget::Unit(a), crate::variable_browser::ObjectId::Unit(b)) => a == b,
                (InspectorTarget::Stream(a), crate::variable_browser::ObjectId::Stream(b)) => {
                    a == b
                }
                _ => false,
            },
            _ => match target {
                InspectorTarget::Unit(id) => {
                    unit_inspector_draft_key_parts(key).is_some_and(|(owner, _)| owner == *id)
                }
                InspectorTarget::Stream(id) => {
                    stream_inspector_draft_key_parts(key).is_some_and(|(owner, _)| owner == *id)
                }
            },
        }
    }

    pub fn pending_keys(&self, fields: &BTreeMap<String, DraftValue>) -> Vec<String> {
        fields
            .iter()
            .filter(|(key, value)| {
                self.includes(key, value)
                    && !matches!(value, DraftValue::Numeric(session) if !session.is_pending())
            })
            .map(|(key, _)| key.clone())
            .collect()
    }
}

impl AppState {
    /// Stage a document operation on an isolated application value. Only a successful
    /// document transaction publishes both the operation and the approved discard.
    /// This is for confirmed GUI operations, never a bypass on public variable writes.
    pub fn apply_with_confirmed_input_discard<T>(
        &mut self,
        checkpoint: &InputEditCheckpoint,
        scope: &InputDiscardScope,
        operation: impl FnOnce(&mut AppState) -> RfResult<T>,
    ) -> RfResult<T> {
        if !checkpoint.matches(&self.workspace) {
            return Err(RfError::invalid_input(
                "input discard confirmation is stale; review affected inputs again",
            ));
        }
        let mut candidate = self.clone();
        candidate
            .workspace
            .drafts
            .fields
            .retain(|key, value| !scope.includes(key, value));
        let outcome = operation(&mut candidate)?;
        if candidate.workspace.document.revision == self.workspace.document.revision {
            return Err(RfError::invalid_input(
                "the requested operation made no document change; inputs were preserved",
            ));
        }
        // Existing engineering commands invalidate legacy name/composition drafts.
        // A scoped confirmation may not discard those belonging to another object.
        for (key, value) in &self.workspace.drafts.fields {
            if !scope.includes(key, value) {
                candidate
                    .workspace
                    .drafts
                    .fields
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }
        candidate.workspace.refresh_numeric_edits();
        *self = candidate;
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variable_browser::{ObjectId, VariableField, VariableId, VariableSection};
    use rf_types::units::MeasurementUnit;
    use std::time::UNIX_EPOCH;

    fn edited_app() -> (AppState, VariableId) {
        let mut flowsheet = Flowsheet::new("input protection");
        flowsheet
            .insert_stream(MaterialStreamState::new("s", "Feed"))
            .unwrap();
        let mut app = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Test", UNIX_EPOCH),
        ));
        let variable = VariableId {
            document: "doc".into(),
            object: ObjectId::Stream("s".into()),
            section: VariableSection::Inputs,
            field: VariableField::Pressure,
        };
        app.begin_numeric_edit(variable.clone(), Some(MeasurementUnit::Bar))
            .unwrap();
        let generation = app.workspace.numeric_edit(&variable).unwrap().generation();
        app.edit_numeric(
            &variable,
            generation,
            NumericEditEvent::ReplaceText("1e-".into()),
        )
        .unwrap();
        (app, variable)
    }

    fn checkpoint(app: &AppState) -> InputEditCheckpoint {
        InputEditCheckpoint::capture(
            app.workspace.document.metadata.document_id.as_str(),
            app.workspace.document.revision,
            &app.workspace.drafts.fields,
        )
    }

    #[test]
    fn failed_or_empty_operation_preserves_input_units_history_and_document() {
        let (mut app, _) = edited_app();
        let before = app.clone();
        let guard = checkpoint(&app);
        assert!(
            app.apply_with_confirmed_input_discard(&guard, &InputDiscardScope::All, |candidate| {
                candidate.delete_stream_and_connections(&"s".into(), UNIX_EPOCH)?;
                Err::<(), _>(RfError::invalid_input("operation rejected"))
            })
            .is_err()
        );
        assert_eq!(app, before);
        assert!(
            app.apply_with_confirmed_input_discard(&guard, &InputDiscardScope::All, |candidate| {
                candidate.undo_document_command(UNIX_EPOCH)
            })
            .is_err()
        );
        assert_eq!(app, before);
    }

    #[test]
    fn edit_then_undo_to_identical_text_invalidates_confirmation() {
        let (mut app, variable) = edited_app();
        let guard = checkpoint(&app);
        for event in [
            NumericEditEvent::ReplaceText("2".into()),
            NumericEditEvent::Undo,
        ] {
            let generation = app.workspace.numeric_edit(&variable).unwrap().generation();
            app.edit_numeric(&variable, generation, event).unwrap();
        }
        let before = app.clone();
        assert_eq!(
            app.workspace.numeric_edit(&variable).unwrap().raw_text(),
            "1e-"
        );
        assert!(
            app.apply_with_confirmed_input_discard(&guard, &InputDiscardScope::All, |candidate| {
                candidate.delete_stream_and_connections(&"s".into(), UNIX_EPOCH)
            })
            .is_err()
        );
        assert_eq!(app, before);
    }

    #[test]
    fn object_deletion_preserves_other_objects_numeric_and_text_drafts() {
        let (mut app, variable) = edited_app();
        app.workspace
            .document
            .flowsheet
            .insert_stream(MaterialStreamState::new("other", "Other"))
            .unwrap();
        app.focus_inspector_target(InspectorTarget::Stream("other".into()));
        app.update_stream_inspector_draft(
            &"other".into(),
            StreamInspectorDraftField::Name,
            "unfinished name",
        )
        .unwrap();
        let drafts = app.workspace.drafts.fields.clone();
        let guard = checkpoint(&app);
        app.apply_with_confirmed_input_discard(
            &guard,
            &InputDiscardScope::Object(InspectorTarget::Stream("other".into())),
            |candidate| candidate.delete_stream_and_connections(&"other".into(), UNIX_EPOCH),
        )
        .unwrap();
        assert_eq!(
            app.workspace.numeric_edit(&variable).unwrap().raw_text(),
            "1e-"
        );
        assert_eq!(
            app.workspace.numeric_edit(&variable).unwrap().input_unit(),
            MeasurementUnit::Bar
        );
        assert_eq!(app.workspace.drafts.fields.len(), drafts.len() - 1);
        let guard = checkpoint(&app);
        app.apply_with_confirmed_input_discard(&guard, &InputDiscardScope::All, |candidate| {
            candidate.undo_document_command(UNIX_EPOCH)
        })
        .unwrap();
        assert!(app.workspace.drafts.fields.is_empty());
        assert!(
            app.workspace
                .document
                .flowsheet
                .streams
                .contains_key(&"other".into())
        );
        assert!(
            app.apply_with_confirmed_input_discard(&guard, &InputDiscardScope::All, |candidate| {
                candidate.redo_document_command(UNIX_EPOCH)
            })
            .is_err()
        );
    }
}
