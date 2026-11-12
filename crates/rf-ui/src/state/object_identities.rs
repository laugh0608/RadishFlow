use super::WorkspaceState;
use crate::variable_browser::ObjectId;
use rf_model::Flowsheet;
use rf_types::{FlowObjectIdentities, ObjectCreationId, RfError, RfResult};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct WorkspaceObjectIdentities {
    allocated: FlowObjectIdentities,
    pub(super) saved: FlowObjectIdentities,
}

impl WorkspaceState {
    pub fn object_identities_enabled(&self) -> bool {
        self.object_identities.is_some()
    }

    /// Includes objects recoverable through the current session's history.
    pub fn prepare_object_identities(&self) -> RfResult<FlowObjectIdentities> {
        if self.object_identities_enabled() {
            return Err(RfError::invalid_input(
                "project object identities are already enabled",
            ));
        }
        let flowsheets = std::iter::once(&self.document.flowsheet).chain(
            self.command_history
                .entries
                .iter()
                .flat_map(|entry| entry.before.iter().chain(&entry.after)),
        );
        let mut units = BTreeSet::new();
        let mut streams = BTreeSet::new();
        for flowsheet in flowsheets {
            units.extend(flowsheet.units.keys().cloned());
            streams.extend(flowsheet.streams.keys().cloned());
        }
        let mut used = BTreeSet::new();
        Ok(FlowObjectIdentities {
            units: allocate(units, &mut used)?,
            streams: allocate(streams, &mut used)?,
        })
    }

    pub fn install_saved_object_identities(&mut self, allocated: FlowObjectIdentities) {
        let saved = Self::project_object_identities(&allocated, &self.document.flowsheet);
        self.object_identities = Some(WorkspaceObjectIdentities { allocated, saved });
    }

    pub fn current_object_identities(&self) -> Option<FlowObjectIdentities> {
        self.object_identities.as_ref().map(|identities| {
            Self::project_object_identities(&identities.allocated, &self.document.flowsheet)
        })
    }

    pub fn project_object_identities(
        identities: &FlowObjectIdentities,
        flowsheet: &Flowsheet,
    ) -> FlowObjectIdentities {
        FlowObjectIdentities {
            units: identities
                .units
                .iter()
                .filter(|(id, _)| flowsheet.units.contains_key(*id))
                .map(|(id, creation)| (id.clone(), creation.clone()))
                .collect(),
            streams: identities
                .streams
                .iter()
                .filter(|(id, _)| flowsheet.streams.contains_key(*id))
                .map(|(id, creation)| (id.clone(), creation.clone()))
                .collect(),
        }
    }

    pub fn object_creation_id(&self, object: &ObjectId) -> Option<&ObjectCreationId> {
        let identities = &self.object_identities.as_ref()?.allocated;
        match object {
            ObjectId::Unit(id) if self.document.flowsheet.units.contains_key(id) => {
                identities.units.get(id)
            }
            ObjectId::Stream(id) if self.document.flowsheet.streams.contains_key(id) => {
                identities.streams.get(id)
            }
            _ => None,
        }
    }

    pub fn saved_object_creation_id(&self, object: &ObjectId) -> Option<&ObjectCreationId> {
        let current = self.object_creation_id(object)?;
        let saved = &self.object_identities.as_ref()?.saved;
        let saved = match object {
            ObjectId::Unit(id) => saved.units.get(id),
            ObjectId::Stream(id) => saved.streams.get(id),
        };
        (saved == Some(current)).then_some(current)
    }

    pub(super) fn allocate_created_object_identities(&mut self, next: &Flowsheet) -> RfResult<()> {
        let Some(identities) = &mut self.object_identities else {
            return Ok(());
        };
        let before = &self.document.flowsheet;
        let units: Vec<_> = next
            .units
            .keys()
            .filter(|id| !before.units.contains_key(*id))
            .cloned()
            .collect();
        let streams: Vec<_> = next
            .streams
            .keys()
            .filter(|id| !before.streams.contains_key(*id))
            .cloned()
            .collect();
        if units.is_empty() && streams.is_empty() {
            return Ok(());
        }
        if units
            .iter()
            .any(|id| identities.allocated.units.contains_key(id))
            || streams
                .iter()
                .any(|id| identities.allocated.streams.contains_key(id))
        {
            return Err(RfError::invalid_input(
                "a new flow object cannot reuse an ID from this session's history",
            ));
        }
        let mut used = identities
            .allocated
            .units
            .values()
            .chain(identities.allocated.streams.values())
            .cloned()
            .collect();
        let units = allocate(units, &mut used)?;
        let streams = allocate(streams, &mut used)?;
        identities.allocated.units.extend(units);
        identities.allocated.streams.extend(streams);
        Ok(())
    }
}

fn allocate<K: Ord>(
    keys: impl IntoIterator<Item = K>,
    used: &mut BTreeSet<ObjectCreationId>,
) -> RfResult<BTreeMap<K, ObjectCreationId>> {
    keys.into_iter()
        .map(|key| {
            loop {
                let mut bytes = [0; 16];
                getrandom::fill(&mut bytes).map_err(|error| {
                    RfError::invalid_input(format!(
                        "could not allocate object creation identity: {error}"
                    ))
                    .with_diagnostic_code("project.object_identity.random_source")
                })?;
                let creation = ObjectCreationId::from_random_bytes(bytes);
                if used.insert(creation.clone()) {
                    break Ok((key, creation));
                }
            }
        })
        .collect()
}
