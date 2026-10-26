//! Device actions use the same editor transaction boundary as the application.
use super::{
    actions::{Action, RevisionRef},
    inputs,
    protocol::*,
    references::{Bindings, BoundOutput},
};
use rf_model::equipment::EquipmentTask;
use rf_ui::{
    AppState,
    engineering::{EquipmentCaseKey, editor::*},
    project_query::{ProjectObjectId, ProjectQuery, ProjectRecord},
};
use std::collections::BTreeMap;

pub fn missing() -> Failure {
    Failure::new(
        Stage::Step,
        Code::ObjectMissing,
        "equipment record is missing or has a different kind",
    )
}
pub fn target(
    app: &AppState,
    object: &ObjectRef,
    refs: &Bindings,
) -> Result<EquipmentEditorTarget, Failure> {
    let q = ProjectQuery::new(&app.workspace);
    let r = q.reference(refs.object(object)?);
    let record = q.record(&r).map_err(super::query::query_error)?;
    Ok(match (&r.object, record) {
        (
            ProjectObjectId::Equipment(id),
            ProjectRecord::Equipment(EquipmentTask::SeparatorSizing(_)),
        ) => EquipmentEditorTarget::Task(id.clone()),
        (
            ProjectObjectId::Equipment(id),
            ProjectRecord::Equipment(EquipmentTask::SeparatorRating(_)),
        ) => EquipmentEditorTarget::RatingTask(id.clone()),
        (ProjectObjectId::Geometry(id), _) => EquipmentEditorTarget::RatingGeometry(id.clone()),
        (ProjectObjectId::Case(key), ProjectRecord::RatingCase(_)) => {
            EquipmentEditorTarget::RatingCase(key.clone())
        }
        (ProjectObjectId::Source(id), _) => EquipmentEditorTarget::Source(id.clone()),
        (ProjectObjectId::Basis(id), _) => EquipmentEditorTarget::Basis(id.clone()),
        _ => {
            return Err(Failure::new(
                Stage::Step,
                Code::UnsupportedCapability,
                "this record has no device edit operation",
            ));
        }
    })
}
pub fn object(target: &EquipmentEditorTarget) -> ObjectRef {
    match target {
        EquipmentEditorTarget::Task(id) | EquipmentEditorTarget::RatingTask(id) => {
            ObjectRef::Equipment(IdentityRef::id(id))
        }
        EquipmentEditorTarget::RatingGeometry(id) => ObjectRef::Geometry(IdentityRef::id(id)),
        EquipmentEditorTarget::RatingCase(key) => ObjectRef::Case {
            task: IdentityRef::id(&key.task_id),
            case: IdentityRef::id(&key.case_id),
        },
        EquipmentEditorTarget::Source(id) => ObjectRef::Source(IdentityRef::id(id)),
        EquipmentEditorTarget::Basis(id) => ObjectRef::Basis(IdentityRef::id(id)),
    }
}
pub fn version(app: &AppState, target: &EquipmentEditorTarget) -> Option<(Output, BoundOutput)> {
    let i = app.workspace.engineering.inputs();
    let (kind, id, revision) = match target {
        EquipmentEditorTarget::Task(id) | EquipmentEditorTarget::RatingTask(id) => (
            Output::Task,
            id.to_string(),
            i.equipment_task(id)?.revision(),
        ),
        EquipmentEditorTarget::RatingGeometry(id) => (
            Output::Geometry,
            id.to_string(),
            i.rating_task(id)?.geometry.revision,
        ),
        EquipmentEditorTarget::RatingCase(key) => (
            Output::Case,
            key.case_id.to_string(),
            i.rating_task(&key.task_id)?
                .cases
                .iter()
                .find(|c| c.id == key.case_id)?
                .revision,
        ),
        EquipmentEditorTarget::Source(id) => (
            Output::Source,
            id.to_string(),
            i.basis.sources.iter().find(|s| s.id == *id)?.revision,
        ),
        EquipmentEditorTarget::Basis(id) => (
            Output::Basis,
            id.to_string(),
            i.basis.basis_entries.iter().find(|b| b.id == *id)?.revision,
        ),
    };
    Some((kind, BoundOutput { id, revision }))
}
fn guard(
    app: &AppState,
    target: &EquipmentEditorTarget,
    expected: &RevisionRef,
    refs: &Bindings,
) -> Result<(), Failure> {
    let (kind, record) = version(app, target).ok_or_else(missing)?;
    refs.revision(expected, kind, &record.id, record.revision)?;
    Ok(())
}
pub fn prepare(
    app: &AppState,
    action: &Action,
    refs: &Bindings,
) -> Result<EquipmentAction, Failure> {
    let i = app.workspace.engineering.inputs();
    let task_id = |r: &IdentityRef| -> Result<rf_types::EquipmentTaskId, Failure> {
        refs.resolve(r, Output::Task).map(Into::into)
    };
    Ok(match action {
        Action::CreateSizingTask {} => EquipmentAction::CreateSizingTask,
        Action::CreateRatingTask {} => EquipmentAction::CreateRatingTask,
        Action::CreateSource {} => EquipmentAction::CreateSource,
        Action::CreateBasis { data } => EquipmentAction::CreateBasis {
            name: data.name.clone(),
            value: data.value.clone().into(),
            source_id: refs.resolve(&data.source, Output::Source)?.into(),
        },
        Action::UpdateSizingTask {
            task,
            expected_revision,
            data,
        } => {
            let id = task_id(task)?;
            guard(
                app,
                &EquipmentEditorTarget::Task(id.clone()),
                expected_revision,
                refs,
            )?;
            EquipmentAction::UpdateSizingTask(Box::new(inputs::sizing(
                i.task(&id).ok_or_else(missing)?,
                data,
                refs,
            )?))
        }
        Action::UpdateSource {
            source,
            expected_revision,
            data,
        } => {
            let id: rf_types::SourceId = refs.resolve(source, Output::Source)?.into();
            guard(
                app,
                &EquipmentEditorTarget::Source(id.clone()),
                expected_revision,
                refs,
            )?;
            let old = i
                .basis
                .sources
                .iter()
                .find(|r| r.id == id)
                .ok_or_else(missing)?;
            EquipmentAction::UpdateSource(Box::new(inputs::source(old, data, refs)?))
        }
        Action::UpdateBasis {
            basis,
            expected_revision,
            data,
        } => {
            let id: rf_types::BasisEntryId = refs.resolve(basis, Output::Basis)?.into();
            guard(
                app,
                &EquipmentEditorTarget::Basis(id.clone()),
                expected_revision,
                refs,
            )?;
            let old = i
                .basis
                .basis_entries
                .iter()
                .find(|r| r.id == id)
                .ok_or_else(missing)?;
            EquipmentAction::UpdateBasis(inputs::basis(old, data, refs)?)
        }
        Action::UpdateRatingLabels {
            task,
            expected_revision,
            data,
        } => {
            let id = task_id(task)?;
            let target = EquipmentEditorTarget::RatingTask(id.clone());
            guard(app, &target, expected_revision, refs)?;
            EquipmentAction::UpdateRatingLabels {
                task: id.clone(),
                revision: i.rating_task(&id).ok_or_else(missing)?.revision,
                tag: data.tag.clone(),
                name: data.name.clone(),
            }
        }
        Action::UpdateRatingGeometry {
            task,
            expected_revision,
            data,
        } => {
            let id = task_id(task)?;
            guard(
                app,
                &EquipmentEditorTarget::RatingGeometry(id.clone()),
                expected_revision,
                refs,
            )?;
            EquipmentAction::UpdateRatingGeometry {
                task: id.clone(),
                geometry: Box::new(inputs::geometry(
                    &i.rating_task(&id).ok_or_else(missing)?.geometry,
                    data,
                    refs,
                )?),
            }
        }
        Action::UpdateRatingCase {
            task,
            case,
            expected_revision,
            data,
        } => {
            let id = task_id(task)?;
            let case_id: rf_types::EngineeringCaseId = refs.resolve(case, Output::Case)?.into();
            guard(
                app,
                &EquipmentEditorTarget::RatingCase(EquipmentCaseKey {
                    task_id: id.clone(),
                    case_id: case_id.clone(),
                }),
                expected_revision,
                refs,
            )?;
            let old = i
                .rating_task(&id)
                .and_then(|t| t.cases.iter().find(|c| c.id == case_id))
                .ok_or_else(missing)?;
            EquipmentAction::UpdateRatingCase {
                task: id,
                case: Box::new(inputs::rating_case(old, data, refs)?),
            }
        }
        Action::AddRatingCase {
            task,
            expected_revision,
            copy_from,
        } => {
            let id = task_id(task)?;
            guard(
                app,
                &EquipmentEditorTarget::RatingTask(id.clone()),
                expected_revision,
                refs,
            )?;
            EquipmentAction::AddRatingCase {
                task: id,
                copy_from: copy_from
                    .as_ref()
                    .map(|r| refs.resolve(r, Output::Case).map(Into::into))
                    .transpose()?,
            }
        }
        Action::ReorderRatingCases {
            task,
            expected_revision,
            cases,
        } => {
            let id = task_id(task)?;
            guard(
                app,
                &EquipmentEditorTarget::RatingTask(id.clone()),
                expected_revision,
                refs,
            )?;
            EquipmentAction::ReorderRatingCases {
                task: id,
                cases: cases
                    .iter()
                    .map(|r| refs.resolve(r, Output::Case).map(Into::into))
                    .collect::<Result<_, _>>()?,
            }
        }
        Action::DeleteEquipmentRecord {
            target: r,
            expected_revision,
        } => {
            let target = target(app, r, refs)?;
            guard(app, &target, expected_revision, refs)?;
            EquipmentAction::Delete(target)
        }
        Action::CheckSizing { task } => EquipmentAction::CheckSizing(task_id(task)?),
        Action::CheckRating { task, case } => EquipmentAction::CheckRating(EquipmentCaseKey {
            task_id: task_id(task)?,
            case_id: refs.resolve(case, Output::Case)?.into(),
        }),
        _ => {
            return Err(Failure::new(
                Stage::Step,
                Code::UnsupportedCapability,
                "not a device input action",
            ));
        }
    })
}
pub fn apply(
    app: &mut AppState,
    action: EquipmentAction,
) -> Result<EquipmentActionReceipt, Failure> {
    let w = &mut app.workspace;
    let context = EquipmentActionContext::capture(&w.engineering);
    w.equipment_editor
        .execute_action(
            &mut w.engineering,
            &rf_types::units::DisplayUnitSet::si(),
            EquipmentEditAccess::Editable,
            &context,
            action,
        )
        .map_err(super::errors::equipment)
}
pub fn outputs(
    app: &AppState,
    target: Option<&EquipmentEditorTarget>,
) -> BTreeMap<Output, BoundOutput> {
    let mut out = BTreeMap::new();
    let Some(target) = target else { return out };
    if let Some((kind, record)) = version(app, target) {
        out.insert(kind, record);
    }
    let i = app.workspace.engineering.inputs();
    match target {
        EquipmentEditorTarget::Task(id) => {
            if let Some(t) = i.task(id) {
                out.insert(
                    Output::Case,
                    BoundOutput {
                        id: t.case.id.to_string(),
                        revision: t.revision,
                    },
                );
            }
        }
        EquipmentEditorTarget::RatingTask(id) | EquipmentEditorTarget::RatingGeometry(id) => {
            if let Some(t) = i.rating_task(id) {
                out.insert(
                    Output::Task,
                    BoundOutput {
                        id: id.to_string(),
                        revision: t.revision,
                    },
                );
                out.insert(
                    Output::Geometry,
                    BoundOutput {
                        id: id.to_string(),
                        revision: t.geometry.revision,
                    },
                );
                if let [case] = t.cases.as_slice() {
                    out.insert(
                        Output::Case,
                        BoundOutput {
                            id: case.id.to_string(),
                            revision: case.revision,
                        },
                    );
                }
            }
        }
        EquipmentEditorTarget::RatingCase(key) => {
            if let Some(t) = i.rating_task(&key.task_id) {
                out.insert(
                    Output::Task,
                    BoundOutput {
                        id: key.task_id.to_string(),
                        revision: t.revision,
                    },
                );
            }
        }
        _ => {}
    }
    out
}
