use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitCreateResult {
    pub unit_id: UnitId,
    pub command: DocumentCommand,
    pub revision: u64,
}

impl AppState {
    /// Low-level application transaction; callers own revision and pending-edit checks.
    /// Uses the same canonical ports and session identity allocator as canvas placement.
    pub fn create_builtin_unit(
        &mut self,
        kind: BuiltinUnitKind,
        changed_at: DateTimeUtc,
    ) -> RfResult<UnitCreateResult> {
        let (command, flowsheet, unit_id) = prepare_builtin_unit_creation(
            &self.workspace.document.flowsheet,
            kind,
            &self.workspace.allocated_unit_ids,
        )?;
        let revision = self.commit_document_change(command.clone(), flowsheet, changed_at);
        Ok(UnitCreateResult {
            unit_id,
            command,
            revision,
        })
    }

    /// Low-level connection transaction shared with suggestion acceptance.
    /// Application callers must obtain a current, permitted connection from their rules.
    pub fn commit_material_connection(
        &mut self,
        connection: &CanvasSuggestedMaterialConnection,
        changed_at: DateTimeUtc,
    ) -> RfResult<(DocumentCommand, u64)> {
        let (command, flowsheet) =
            apply_material_connection_acceptance(&self.workspace.document.flowsheet, connection)?;
        let revision = self.commit_document_change(command.clone(), flowsheet, changed_at);
        Ok((command, revision))
    }

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
