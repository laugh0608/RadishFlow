use super::protocol::*;
use crate::StudioRunId;
use rf_ui::{
    engineering::EquipmentCaseKey, project_query::ProjectObjectId, variable_browser::ObjectId,
};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct BoundOutput {
    pub id: String,
    pub revision: u64,
}
#[derive(Default, Clone)]
pub struct Bindings(pub BTreeMap<String, BTreeMap<Output, BoundOutput>>);
impl Bindings {
    pub fn resolve(&self, r: &IdentityRef, kind: Output) -> Result<String, Failure> {
        match r {
            IdentityRef::Id { id } if !id.trim().is_empty() => Ok(id.clone()),
            IdentityRef::Id { .. } => Err(Failure::new(
                Stage::Request,
                Code::InvalidRequest,
                "object id must not be blank",
            )),
            IdentityRef::Step { step, output } => {
                if *output != kind {
                    return Err(Failure::new(
                        Stage::Step,
                        Code::ReferenceKindMismatch,
                        "step output has a different object kind",
                    ));
                }
                self.0
                    .get(step)
                    .and_then(|values| values.get(output))
                    .map(|v| v.id.clone())
                    .ok_or_else(|| {
                        Failure::new(
                            Stage::Step,
                            Code::UnknownReference,
                            "step output is not available",
                        )
                    })
            }
        }
    }
    pub fn object(&self, r: &ObjectRef) -> Result<ProjectObjectId, Failure> {
        Ok(match r {
            ObjectRef::Unit(id) => {
                ProjectObjectId::Flow(ObjectId::Unit(self.resolve(id, Output::Unit)?.into()))
            }
            ObjectRef::Stream(id) => {
                ProjectObjectId::Flow(ObjectId::Stream(self.resolve(id, Output::Stream)?.into()))
            }
            ObjectRef::Equipment(id) => {
                ProjectObjectId::Equipment(self.resolve(id, Output::Task)?.into())
            }
            ObjectRef::Case { task, case } => ProjectObjectId::Case(EquipmentCaseKey {
                task_id: self.resolve(task, Output::Task)?.into(),
                case_id: self.resolve(case, Output::Case)?.into(),
            }),
            ObjectRef::Geometry(id) => {
                ProjectObjectId::Geometry(self.resolve(id, Output::Task)?.into())
            }
            ObjectRef::Basis(id) => ProjectObjectId::Basis(self.resolve(id, Output::Basis)?.into()),
            ObjectRef::Source(id) => {
                ProjectObjectId::Source(self.resolve(id, Output::Source)?.into())
            }
        })
    }
}

pub fn object_ref(id: &ProjectObjectId) -> ObjectRef {
    match id {
        ProjectObjectId::Flow(ObjectId::Unit(id)) => ObjectRef::Unit(IdentityRef::id(id)),
        ProjectObjectId::Flow(ObjectId::Stream(id)) => ObjectRef::Stream(IdentityRef::id(id)),
        ProjectObjectId::Equipment(id) => ObjectRef::Equipment(IdentityRef::id(id)),
        ProjectObjectId::Case(key) => ObjectRef::Case {
            task: IdentityRef::id(&key.task_id),
            case: IdentityRef::id(&key.case_id),
        },
        ProjectObjectId::Geometry(id) => ObjectRef::Geometry(IdentityRef::id(id)),
        ProjectObjectId::Basis(id) => ObjectRef::Basis(IdentityRef::id(id)),
        ProjectObjectId::Source(id) => ObjectRef::Source(IdentityRef::id(id)),
    }
}

/// Public execution references are request-local step names, never pointer keys.
#[derive(Default)]
pub struct Executions(pub Vec<(StudioRunId, String)>);
impl Executions {
    pub fn step(&self, id: &StudioRunId) -> Option<&str> {
        self.0
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, step)| step.as_str())
    }
}

impl Bindings {
    /// A receipt version is valid only for its actual owner, not another record
    /// which happens to have the same numeric revision.
    pub fn revision(
        &self,
        reference: &super::actions::RevisionRef,
        kind: Output,
        id: &str,
        actual: u64,
    ) -> Result<u64, Failure> {
        let supplied = match reference {
            super::actions::RevisionRef::Number(n) => *n,
            super::actions::RevisionRef::Step { step, output } => {
                if *output != kind {
                    return Err(Failure::new(
                        Stage::Step,
                        Code::ReferenceKindMismatch,
                        "revision owner kind differs",
                    ));
                }
                let value = self
                    .0
                    .get(step)
                    .and_then(|v| v.get(output))
                    .ok_or_else(|| {
                        Failure::new(
                            Stage::Step,
                            Code::UnknownReference,
                            "revision output is unavailable",
                        )
                    })?;
                if value.id != id {
                    return Err(Failure::new(
                        Stage::Step,
                        Code::ReferenceKindMismatch,
                        "revision belongs to a different record",
                    ));
                }
                value.revision
            }
        };
        if supplied != actual {
            let mut failure =
                Failure::new(Stage::Step, Code::RecordConflict, "record revision differs");
            failure.details = Some(Box::new(
                serde_json::json!({"owner":kind,"id":id,"expected":supplied,"actual":actual}),
            ));
            return Err(failure);
        }
        Ok(supplied)
    }
}
