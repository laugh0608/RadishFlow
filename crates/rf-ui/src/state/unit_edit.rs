use super::*;

impl AppState {
    /// Delete one unit as a single document transaction. Streams remain explicit objects.
    pub fn delete_unit(&mut self, unit_id: &UnitId, changed_at: DateTimeUtc) -> RfResult<u64> {
        let mut flowsheet = self.workspace.document.flowsheet.clone();
        flowsheet.remove_unit(unit_id)?;
        let revision = self.commit_document_change(
            DocumentCommand::DeleteUnit {
                unit_id: unit_id.clone(),
            },
            flowsheet,
            changed_at,
        );
        self.workspace.prune_focus_against_document();
        Ok(revision)
    }
}
