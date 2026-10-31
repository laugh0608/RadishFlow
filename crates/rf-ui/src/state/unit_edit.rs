use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct UnitCreateResult {
    pub unit_id: UnitId,
    pub command: DocumentCommand,
    pub revision: u64,
}

impl AppState {
    /// Read-only preparation for a local recommendation. Incomplete unrelated ports
    /// remain allowed, but the proposed edge must preserve the current acyclic model.
    pub fn validate_acyclic_material_connection(
        &self,
        connection: &CanvasSuggestedMaterialConnection,
    ) -> RfResult<()> {
        let flowsheet = &self.workspace.document.flowsheet;
        for unit_id in
            std::iter::once(&connection.source_unit_id).chain(connection.sink_unit_id.as_ref())
        {
            let unit = flowsheet
                .units
                .get(unit_id)
                .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
            rf_unitops::validate_unit_node(unit)?;
        }
        apply_material_connection_acceptance(flowsheet, connection)?;
        if let Some(sink) = &connection.sink_unit_id {
            // The shared reachability check assumes one source per stream. Never
            // choose an arbitrary source from malformed topology to prove safety.
            let mut sources = BTreeSet::new();
            for port in flowsheet.units.values().flat_map(|unit| &unit.ports) {
                if port.kind == PortKind::Material
                    && port.direction == PortDirection::Outlet
                    && let Some(stream) = &port.stream_id
                    && !sources.insert(stream)
                {
                    return Err(RfError::invalid_connection(
                        "repair ambiguous stream sources before recommending a connection",
                    ));
                }
            }
            if would_create_unit_dependency_cycle(flowsheet, &connection.source_unit_id, sink) {
                return Err(RfError::invalid_connection(
                    "connection would introduce a unit dependency cycle",
                ));
            }
        }
        Ok(())
    }

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

    /// Exact material-port transaction. Application callers enforce input/draft guards.
    /// Does not select an Inspector or delete the stream or its other bindings.
    pub fn disconnect_material_port(
        &mut self,
        unit_id: &UnitId,
        port: &str,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<DocumentCommand>> {
        let (command, next) =
            apply_disconnect_port_mutation(&self.workspace.document.flowsheet, unit_id, port)?;
        if next == self.workspace.document.flowsheet {
            return Ok(None);
        }
        self.commit_document_change(command.clone(), next, changed_at);
        Ok(Some(command))
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
        self.workspace.drafts.fields.retain(|_, draft| !matches!(draft,
            DraftValue::Numeric(session) if session.variable().object == crate::variable_browser::ObjectId::Unit(unit_id.clone())));
        self.workspace.prune_focus_against_document();
        Ok(revision)
    }
}
