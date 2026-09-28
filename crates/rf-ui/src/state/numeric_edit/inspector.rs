//! Transitional Inspector bridge: existing controls are explicitly SI until I4.
use super::*;
use crate::variable_browser::{ObjectId, VariableField, VariableSection};

impl AppState {
    pub(in crate::state) fn update_numeric_inspector(
        &mut self,
        object: ObjectId,
        field: VariableField,
        raw: String,
    ) -> Option<(bool, DraftValidationState)> {
        let id = VariableId {
            document: self.workspace.document.metadata.document_id.clone(),
            object,
            section: VariableSection::Inputs,
            field,
        };
        let (quantity, baseline) = self.workspace.numeric_field(&id).ok()?;
        let generation = self
            .begin_numeric_edit(id.clone(), Some(quantity.definition().canonical_unit))
            .ok()?;
        let generation = self
            .edit_numeric(&id, generation, NumericEditEvent::ReplaceText(raw))
            .ok()?;
        if !baseline.explicit && !self.workspace.numeric_edit(&id).ok()?.is_pending() {
            self.edit_numeric(&id, generation, NumericEditEvent::AdoptValue)
                .ok()?;
        }
        let session = self.workspace.numeric_edit(&id).ok()?;
        let result = (session.is_dirty(), session.draft_validation());
        if !session.is_pending() {
            self.workspace
                .drafts
                .fields
                .remove(&transaction::field_key(&id).ok()?);
        }
        Some(result)
    }
}
