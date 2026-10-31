//! Read-only discovery over one authoritative workspace. No public path grammar,
//! mutation dispatch, IO, implicit calculation or parallel engineering store.
mod actions;
mod fields;
use crate::engineering::{EngineeringCheckpoint, EquipmentCaseKey, EquipmentResultView};
use crate::variable_browser::{self, ObjectId, VariableBrowser};
use crate::{DocumentId, FlowRunQualification, WorkspaceState};
pub use actions::*;
pub use fields::*;
use rf_model::engineering::{BasisEntry, SourceRecord};
use rf_model::equipment::{
    EquipmentTask, SeparatorCase, SeparatorGivenGeometry, SeparatorRatingCase,
};
use rf_types::{BasisEntryId, EquipmentTaskId, SourceId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectObjectId {
    Flow(ObjectId),
    Equipment(EquipmentTaskId),
    Case(EquipmentCaseKey),
    Geometry(EquipmentTaskId),
    Basis(BasisEntryId),
    Source(SourceId),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectObjectRef {
    pub document: DocumentId,
    pub object: ProjectObjectId,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectObjectKind {
    Unit,
    Stream,
    Sizing,
    Rating,
    Case,
    Geometry,
    Basis,
    Source,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectObjectDescriptor {
    pub reference: ProjectObjectRef,
    pub parent: Option<ProjectObjectRef>,
    pub kind: ProjectObjectKind,
    pub label: String,
}
#[derive(Debug)]
pub enum ProjectRecord<'a> {
    Flow(variable_browser::ObjectDescriptor),
    Equipment(&'a EquipmentTask),
    SizingCase(&'a SeparatorCase),
    RatingCase(&'a SeparatorRatingCase),
    Geometry(&'a SeparatorGivenGeometry),
    Basis(&'a BasisEntry),
    Source(&'a SourceRecord),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectQueryError {
    DifferentDocument,
    InputContextChanged,
    ObjectMissing(ProjectObjectId),
    FieldMissing(ProjectField),
    NotEquipmentCase,
}
/// An optimistic input-read guard, never a public or persistent session token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInputContext {
    flow: FlowRunQualification,
    engineering: EngineeringCheckpoint,
}
impl ProjectInputContext {
    pub fn document_id(&self) -> &DocumentId {
        self.flow.document_id()
    }
    pub fn flow_revision(&self) -> u64 {
        self.flow.revision()
    }
    pub fn validate(&self, workspace: &WorkspaceState) -> Result<(), ProjectQueryError> {
        if self.document_id() != &workspace.document.metadata.document_id {
            return Err(ProjectQueryError::DifferentDocument);
        }
        if !self.flow.same_input(&workspace.flow_run_qualification())
            || self.engineering != workspace.engineering.checkpoint()
        {
            return Err(ProjectQueryError::InputContextChanged);
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub struct ProjectQuery<'a> {
    workspace: &'a WorkspaceState,
}
impl<'a> ProjectQuery<'a> {
    pub fn new(workspace: &'a WorkspaceState) -> Self {
        Self { workspace }
    }
    pub fn with_context(
        workspace: &'a WorkspaceState,
        context: &ProjectInputContext,
    ) -> Result<Self, ProjectQueryError> {
        context.validate(workspace)?;
        Ok(Self::new(workspace))
    }
    pub fn input_context(&self) -> ProjectInputContext {
        ProjectInputContext {
            flow: self.workspace.flow_run_qualification(),
            engineering: self.workspace.engineering.checkpoint(),
        }
    }
    pub fn reference(&self, object: ProjectObjectId) -> ProjectObjectRef {
        ProjectObjectRef {
            document: self.workspace.document.metadata.document_id.clone(),
            object,
        }
    }
    pub fn flow_variables(&self) -> VariableBrowser<'a> {
        VariableBrowser::from_workspace(self.workspace)
    }
    pub fn flow_result(&self) -> crate::result_query::FlowResultView<'a> {
        self.workspace.flow_result()
    }
    pub fn equipment_result(
        &self,
        reference: &ProjectObjectRef,
    ) -> Result<EquipmentResultView<'a>, ProjectQueryError> {
        self.record(reference)?;
        let ProjectObjectId::Case(member) = &reference.object else {
            return Err(ProjectQueryError::NotEquipmentCase);
        };
        Ok(self.workspace.engineering.query_case_result(member))
    }
    pub fn record(
        &self,
        reference: &ProjectObjectRef,
    ) -> Result<ProjectRecord<'a>, ProjectQueryError> {
        if reference.document != self.workspace.document.metadata.document_id {
            return Err(ProjectQueryError::DifferentDocument);
        }
        let inputs = self.workspace.engineering.inputs();
        let record = match &reference.object {
            ProjectObjectId::Flow(id) => self
                .flow_variables()
                .object(id)
                .ok()
                .map(ProjectRecord::Flow),
            ProjectObjectId::Equipment(id) => {
                inputs.equipment_task(id).map(ProjectRecord::Equipment)
            }
            ProjectObjectId::Geometry(id) => inputs
                .rating_task(id)
                .map(|t| ProjectRecord::Geometry(&t.geometry)),
            ProjectObjectId::Case(member) => {
                inputs
                    .equipment_task(&member.task_id)
                    .and_then(|task| match task {
                        EquipmentTask::SeparatorSizing(t) => (t.case.id == member.case_id)
                            .then_some(ProjectRecord::SizingCase(&t.case)),
                        EquipmentTask::SeparatorRating(t) => t
                            .cases
                            .iter()
                            .find(|c| c.id == member.case_id)
                            .map(ProjectRecord::RatingCase),
                    })
            }
            ProjectObjectId::Basis(id) => inputs
                .basis
                .basis_entries
                .iter()
                .find(|v| v.id == *id)
                .map(ProjectRecord::Basis),
            ProjectObjectId::Source(id) => inputs
                .basis
                .sources
                .iter()
                .find(|v| v.id == *id)
                .map(ProjectRecord::Source),
        };
        record.ok_or_else(|| ProjectQueryError::ObjectMissing(reference.object.clone()))
    }
    pub fn object(
        &self,
        reference: &ProjectObjectRef,
    ) -> Result<ProjectObjectDescriptor, ProjectQueryError> {
        let record = self.record(reference)?;
        let (kind, label) = match record {
            ProjectRecord::Flow(v) => (
                match v.id {
                    ObjectId::Unit(_) => ProjectObjectKind::Unit,
                    ObjectId::Stream(_) => ProjectObjectKind::Stream,
                },
                v.name,
            ),
            ProjectRecord::Equipment(t) => (
                match t {
                    EquipmentTask::SeparatorSizing(_) => ProjectObjectKind::Sizing,
                    EquipmentTask::SeparatorRating(_) => ProjectObjectKind::Rating,
                },
                format!("{} · {}", t.tag(), t.name()),
            ),
            ProjectRecord::SizingCase(c) => (ProjectObjectKind::Case, c.name.clone()),
            ProjectRecord::RatingCase(c) => (ProjectObjectKind::Case, c.name.clone()),
            ProjectRecord::Geometry(_) => (
                ProjectObjectKind::Geometry,
                "共同几何 / Shared geometry".into(),
            ),
            ProjectRecord::Basis(b) => (ProjectObjectKind::Basis, b.name.clone()),
            ProjectRecord::Source(s) => (ProjectObjectKind::Source, s.title.clone()),
        };
        let parent = match &reference.object {
            ProjectObjectId::Case(member) => {
                Some(self.reference(ProjectObjectId::Equipment(member.task_id.clone())))
            }
            ProjectObjectId::Geometry(id) => {
                Some(self.reference(ProjectObjectId::Equipment(id.clone())))
            }
            _ => None,
        };
        Ok(ProjectObjectDescriptor {
            reference: reference.clone(),
            parent,
            kind,
            label,
        })
    }
    /// Enumeration does not resolve fields, clone all records or compute results.
    pub fn objects(&self) -> Vec<ProjectObjectDescriptor> {
        let mut ids: Vec<_> = self
            .flow_variables()
            .objects()
            .into_iter()
            .map(|v| ProjectObjectId::Flow(v.id))
            .collect();
        let inputs = self.workspace.engineering.inputs();
        for task in &inputs.tasks {
            ids.push(ProjectObjectId::Equipment(task.id().clone()));
            if task.as_rating().is_some() {
                ids.push(ProjectObjectId::Geometry(task.id().clone()));
            }
            ids.extend(task.case_ids().map(|case| {
                ProjectObjectId::Case(EquipmentCaseKey {
                    task_id: task.id().clone(),
                    case_id: case.clone(),
                })
            }));
        }
        ids.extend(
            inputs
                .basis
                .basis_entries
                .iter()
                .map(|v| ProjectObjectId::Basis(v.id.clone())),
        );
        ids.extend(
            inputs
                .basis
                .sources
                .iter()
                .map(|v| ProjectObjectId::Source(v.id.clone())),
        );
        ids.into_iter()
            .map(|id| {
                self.object(&self.reference(id))
                    .expect("enumerated record exists")
            })
            .collect()
    }
    pub fn search(&self, text: &str) -> Vec<ProjectObjectDescriptor> {
        let text = text.trim().to_lowercase();
        self.objects()
            .into_iter()
            .filter(|v| {
                v.label.to_lowercase().contains(&text)
                    || v.reference
                        .object
                        .identity_text()
                        .to_lowercase()
                        .contains(&text)
            })
            .collect()
    }
}
impl ProjectObjectId {
    fn identity_text(&self) -> String {
        match self {
            Self::Flow(ObjectId::Unit(id)) => id.to_string(),
            Self::Flow(ObjectId::Stream(id)) => id.to_string(),
            Self::Equipment(id) | Self::Geometry(id) => id.to_string(),
            Self::Case(m) => format!("{} {}", m.task_id, m.case_id),
            Self::Basis(id) => id.to_string(),
            Self::Source(id) => id.to_string(),
        }
    }
}
#[cfg(test)]
mod tests;
