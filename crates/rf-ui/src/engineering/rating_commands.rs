//! Rating edits address geometry and cases explicitly. Case membership and method
//! expansion remain outside the version 1 application capability.
use super::*;
use rf_model::equipment::{SeparatorGivenGeometry, SeparatorRatingCase};

pub(super) fn validate_capability(task: &SeparatorRatingTask) -> Result<(), EngineeringError> {
    if task.method != rf_unitops::separator_rating::method() {
        return Err(EngineeringError::UnsupportedRatingMethod(task.id.clone()));
    }
    if task.cases.len() != 1 {
        return Err(EngineeringError::UnsupportedRatingCaseCount {
            task: task.id.clone(),
            actual: task.cases.len(),
        });
    }
    Ok(())
}
pub(super) fn create(
    next: &mut EngineeringInputs,
    reserved: &mut BTreeMap<EngineeringRecordId, u64>,
    task: SeparatorRatingTask,
) -> Result<(), EngineeringError> {
    validate_capability(&task)?;
    for (id, _) in task_revisions(&EquipmentTask::SeparatorRating(task.clone())) {
        commands::reserve(reserved, id)?;
    }
    next.tasks.push(task.into());
    Ok(())
}
fn task_mut<'a>(
    inputs: &'a mut EngineeringInputs,
    id: &EquipmentTaskId,
) -> Result<&'a mut SeparatorRatingTask, EngineeringError> {
    inputs
        .tasks
        .iter_mut()
        .find_map(|t| match t {
            EquipmentTask::SeparatorRating(t) if t.id == *id => Some(t),
            _ => None,
        })
        .ok_or_else(|| EngineeringError::MissingRecord(EngineeringRecordId::Task(id.clone())))
}
fn revision(id: EngineeringRecordId, expected: u64, supplied: u64) -> Result<(), EngineeringError> {
    if expected == supplied {
        Ok(())
    } else {
        Err(EngineeringError::RevisionConflict {
            record: id,
            expected,
            supplied,
        })
    }
}
pub(super) fn labels(
    inputs: &mut EngineeringInputs,
    id: &EquipmentTaskId,
    supplied: u64,
    tag: String,
    name: String,
) -> Result<(), EngineeringError> {
    let t = task_mut(inputs, id)?;
    revision(EngineeringRecordId::Task(id.clone()), t.revision, supplied)?;
    t.tag = tag;
    t.name = name;
    Ok(())
}
pub(super) fn geometry(
    inputs: &mut EngineeringInputs,
    id: &EquipmentTaskId,
    geometry: SeparatorGivenGeometry,
) -> Result<(), EngineeringError> {
    let t = task_mut(inputs, id)?;
    revision(
        EngineeringRecordId::Geometry(id.clone()),
        t.geometry.revision,
        geometry.revision,
    )?;
    t.geometry = geometry;
    Ok(())
}
pub(super) fn case(
    inputs: &mut EngineeringInputs,
    id: &EquipmentTaskId,
    case: SeparatorRatingCase,
) -> Result<(), EngineeringError> {
    let t = task_mut(inputs, id)?;
    let old = t
        .cases
        .iter_mut()
        .find(|c| c.id == case.id)
        .ok_or_else(|| {
            EngineeringError::MissingRecord(EngineeringRecordId::Case(case.id.clone()))
        })?;
    revision(
        EngineeringRecordId::Case(case.id.clone()),
        old.revision,
        case.revision,
    )?;
    *old = case;
    Ok(())
}

pub(super) fn clear_revisions(task: &mut EquipmentTask) {
    task.set_revision(0);
    if let EquipmentTask::SeparatorRating(t) = task {
        t.geometry.revision = 0;
        for c in &mut t.cases {
            c.revision = 0;
        }
    }
}
pub(super) fn task_revisions(task: &EquipmentTask) -> Vec<(EngineeringRecordId, u64)> {
    let mut records = vec![(
        EngineeringRecordId::Task(task.id().clone()),
        task.revision(),
    )];
    match task {
        EquipmentTask::SeparatorSizing(t) => {
            records.push((EngineeringRecordId::Case(t.case.id.clone()), 0))
        }
        EquipmentTask::SeparatorRating(t) => {
            records.push((
                EngineeringRecordId::Geometry(t.id.clone()),
                t.geometry.revision,
            ));
            records.extend(
                t.cases
                    .iter()
                    .map(|c| (EngineeringRecordId::Case(c.id.clone()), c.revision)),
            );
        }
    }
    records
}
impl EngineeringSession {
    pub(super) fn publish_task(
        &mut self,
        task: &mut EquipmentTask,
    ) -> Result<(), EngineeringError> {
        // Work from owned old records so advancing revision counters cannot alias inputs.
        let previous = self.current.equipment_task(task.id()).cloned();
        let mut old_content = previous.clone();
        if let Some(t) = &mut old_content {
            clear_revisions(t);
        }
        let mut new_content = task.clone();
        clear_revisions(&mut new_content);
        let revision = self.next_revision(
            EngineeringRecordId::Task(task.id().clone()),
            old_content.as_ref() != Some(&new_content),
        )?;
        task.set_revision(revision);
        match task {
            EquipmentTask::SeparatorSizing(t) => {
                self.revisions
                    .entry(EngineeringRecordId::Case(t.case.id.clone()))
                    .or_insert(0);
            }
            EquipmentTask::SeparatorRating(t) => {
                let old = previous.as_ref().and_then(EquipmentTask::as_rating);
                let mut geometry = t.geometry.clone();
                geometry.revision = old.map_or(0, |r| r.geometry.revision);
                t.geometry.revision = self.next_revision(
                    EngineeringRecordId::Geometry(t.id.clone()),
                    old.map(|r| &r.geometry) != Some(&geometry),
                )?;
                for case in &mut t.cases {
                    let old_case = old.and_then(|r| r.cases.iter().find(|c| c.id == case.id));
                    let mut normalized = case.clone();
                    normalized.revision = old_case.map_or(0, |c| c.revision);
                    case.revision = self.next_revision(
                        EngineeringRecordId::Case(case.id.clone()),
                        old_case != Some(&normalized),
                    )?;
                }
            }
        }
        Ok(())
    }
}
