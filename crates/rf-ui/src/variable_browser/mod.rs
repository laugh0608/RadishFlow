//! Read-only engineering discovery. Identities are document-scoped, never display paths.
//! Views borrow authoritative inputs and results; drafts are deliberately excluded.

mod actions;
mod variables;
pub use actions::*;
pub use variables::*;

use crate::{DocumentId, FlowsheetDocument, SolveSnapshot};
use rf_types::{StreamId, UnitId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectId {
    Unit(UnitId),
    Stream(StreamId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDescriptor {
    pub id: ObjectId,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowseError {
    DifferentDocument,
    ObjectMissing(ObjectId),
    VariableMissing(VariableId),
}

pub struct VariableBrowser<'a> {
    pub document: &'a FlowsheetDocument,
    current: Option<&'a SolveSnapshot>,
    stale: Option<&'a SolveSnapshot>,
}

impl<'a> VariableBrowser<'a> {
    /// Snapshots must come from the same workspace as the document.
    /// Revision filtering prevents stale values being presented as current; it is not a
    /// cross-workspace identity check because SolveSnapshot has no document identity.
    pub fn new(
        document: &'a FlowsheetDocument,
        current: Option<&'a SolveSnapshot>,
        stale: Option<&'a SolveSnapshot>,
    ) -> Self {
        Self {
            document,
            current: current.filter(|s| s.document_revision == document.revision),
            stale: stale
                .filter(|s| s.document_revision != document.revision)
                .or(current.filter(|s| s.document_revision != document.revision)),
        }
    }

    pub fn document_id(&self) -> &DocumentId {
        &self.document.metadata.document_id
    }

    /// Object enumeration does not materialize variable values or the whole tree.
    pub fn objects(&self) -> Vec<ObjectDescriptor> {
        self.document
            .flowsheet
            .units
            .values()
            .map(|u| ObjectDescriptor {
                id: ObjectId::Unit(u.id.clone()),
                name: u.name.clone(),
                kind: u.kind.clone(),
            })
            .chain(
                self.document
                    .flowsheet
                    .streams
                    .values()
                    .map(|s| ObjectDescriptor {
                        id: ObjectId::Stream(s.id.clone()),
                        name: s.name.clone(),
                        kind: "material_stream".into(),
                    }),
            )
            .collect()
    }

    pub fn object(&self, id: &ObjectId) -> Result<ObjectDescriptor, BrowseError> {
        match id {
            ObjectId::Unit(id) => self
                .document
                .flowsheet
                .units
                .get(id)
                .map(|u| ObjectDescriptor {
                    id: ObjectId::Unit(id.clone()),
                    name: u.name.clone(),
                    kind: u.kind.clone(),
                }),
            ObjectId::Stream(id) => {
                self.document
                    .flowsheet
                    .streams
                    .get(id)
                    .map(|s| ObjectDescriptor {
                        id: ObjectId::Stream(id.clone()),
                        name: s.name.clone(),
                        kind: "material_stream".into(),
                    })
            }
        }
        .ok_or_else(|| BrowseError::ObjectMissing(id.clone()))
    }

    /// Unit ports reference stream objects, including missing references for diagnostics.
    pub fn stream_references(&self, id: &ObjectId) -> Result<Vec<(String, ObjectId)>, BrowseError> {
        self.object(id)?;
        let ObjectId::Unit(id) = id else {
            return Ok(Vec::new());
        };
        Ok(self.document.flowsheet.units[id]
            .ports
            .iter()
            .filter_map(|p| {
                p.stream_id
                    .as_ref()
                    .map(|s| (p.name.clone(), ObjectId::Stream(s.clone())))
            })
            .collect())
    }

    pub fn read(&self, id: &VariableId) -> Result<VariableDescriptor, BrowseError> {
        if &id.document != self.document_id() {
            return Err(BrowseError::DifferentDocument);
        }
        self.variables(&id.object)?
            .into_iter()
            .find(|v| v.id == *id)
            .ok_or_else(|| BrowseError::VariableMissing(id.clone()))
    }

    /// Search matches names, identities, labels and SI units; returned values retain provenance.
    pub fn search(&self, query: &str) -> Vec<VariableDescriptor> {
        let query = query.trim().to_lowercase();
        let mut matches = Vec::new();
        for object in self.objects() {
            let object_matches = format!("{} {:?} {}", object.name, object.id, object.kind)
                .to_lowercase()
                .contains(&query);
            for variable in self
                .variables(&object.id)
                .expect("enumerated object exists in the borrowed document")
            {
                if object_matches
                    || format!(
                        "{} {:?} {}",
                        variable.label,
                        variable.id.field,
                        variable.unit_symbol()
                    )
                    .to_lowercase()
                    .contains(&query)
                {
                    matches.push(variable);
                }
            }
        }
        matches
    }
}

#[cfg(test)]
mod tests;
