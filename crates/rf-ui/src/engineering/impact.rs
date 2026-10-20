use super::*;
use rf_model::engineering::{InputBinding, SourceRecord};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskImpact {
    pub task_id: EquipmentTaskId,
    pub case_id: EngineeringCaseId,
    pub input_changed: bool,
    pub evidence_changed: bool,
}

#[derive(Default)]
struct Dependencies {
    basis: BTreeSet<BasisEntryId>,
    sources: BTreeSet<SourceId>,
}
impl Dependencies {
    fn add_task(&mut self, inputs: &EngineeringInputs, task: &EquipmentTask) {
        for (_, sources) in commands::task_sources(task) {
            self.sources.extend(sources.iter().cloned());
        }
        for (_, _, binding) in commands::task_bindings(task) {
            match binding {
                InputBinding::Direct { source_id, .. } => {
                    self.sources.insert(source_id.clone());
                }
                InputBinding::BasisRef { basis_entry_id } => {
                    self.basis.insert(basis_entry_id.clone());
                }
                InputBinding::Override {
                    source_id,
                    basis_entry_id,
                    ..
                } => {
                    self.sources.insert(source_id.clone());
                    self.basis.insert(basis_entry_id.clone());
                }
            }
        }
        for entry in &inputs.basis.basis_entries {
            if self.basis.contains(&entry.id) {
                self.sources.insert(entry.source_id.clone());
            }
        }
        loop {
            let previous = self.sources.len();
            for source in &inputs.basis.sources {
                if self.sources.contains(&source.id) {
                    for observation in &source.observations {
                        self.sources
                            .extend(observation.depends_on.iter().map(|r| r.source_id.clone()));
                    }
                }
            }
            if previous == self.sources.len() {
                break;
            }
        }
    }
}
/// The same transitive provenance used for result invalidation also gates draft calculations.
pub(super) fn task_references(
    inputs: &EngineeringInputs,
    task: &EquipmentTaskId,
    record: &EngineeringRecordId,
) -> bool {
    inputs.equipment_task(task).is_some_and(|t| {
        t.case_ids().any(|case| {
            case_references(
                inputs,
                &EquipmentCaseKey {
                    task_id: task.clone(),
                    case_id: case.clone(),
                },
                record,
            )
        })
    })
}
pub(super) fn case_references(
    inputs: &EngineeringInputs,
    key: &EquipmentCaseKey,
    record: &EngineeringRecordId,
) -> bool {
    let Some(task) = case_projection(inputs.equipment_task(&key.task_id), &key.case_id) else {
        return false;
    };
    let mut dependencies = Dependencies::default();
    dependencies.add_task(inputs, &task);
    match record {
        EngineeringRecordId::Task(id) | EngineeringRecordId::Geometry(id) => task.id() == id,
        EngineeringRecordId::Case(id) => key.case_id == *id,
        EngineeringRecordId::Source(id) => dependencies.sources.contains(id),
        EngineeringRecordId::Basis(id) => dependencies.basis.contains(id),
    }
}
fn case_projection(
    task: Option<&EquipmentTask>,
    case: &EngineeringCaseId,
) -> Option<EquipmentTask> {
    let mut task = task?.clone();
    match &mut task {
        EquipmentTask::SeparatorSizing(t) if t.case.id != *case => return None,
        EquipmentTask::SeparatorRating(t) => {
            t.cases.retain(|c| c.id == *case);
            if t.cases.is_empty() {
                return None;
            }
        }
        _ => {}
    }
    Some(task)
}
fn task_recipe(task: &EquipmentTask) -> EquipmentTask {
    let mut recipe = task.clone();
    rating_commands::clear_revisions(&mut recipe);
    match &mut recipe {
        EquipmentTask::SeparatorSizing(t) => {
            t.tag.clear();
            t.name.clear();
            t.case.name.clear();
            t.case.purpose.clear();
        }
        EquipmentTask::SeparatorRating(t) => {
            t.tag.clear();
            t.name.clear();
            for c in &mut t.cases {
                c.name.clear();
                c.purpose.clear();
            }
        }
    }
    recipe
}
#[derive(PartialEq)]
enum BindingEvidence<'a> {
    Direct(&'a SourceId),
    Basis(&'a BasisEntryId),
    Override(&'a BasisEntryId, &'a SourceId, &'a str),
}
fn binding_evidence(binding: &InputBinding) -> BindingEvidence<'_> {
    match binding {
        InputBinding::Direct { source_id, .. } => BindingEvidence::Direct(source_id),
        InputBinding::BasisRef { basis_entry_id } => BindingEvidence::Basis(basis_entry_id),
        InputBinding::Override {
            basis_entry_id,
            source_id,
            reason,
            ..
        } => BindingEvidence::Override(basis_entry_id, source_id, reason),
    }
}
fn task_evidence_changed(old: Option<&EquipmentTask>, new: Option<&EquipmentTask>) -> bool {
    let (Some(old), Some(new)) = (old, new) else {
        return true;
    };
    let declarations_changed = match (old, new) {
        (EquipmentTask::SeparatorSizing(a), EquipmentTask::SeparatorSizing(b)) => {
            a.case.id != b.case.id
                || a.case.fluid_system != b.case.fluid_system
                || a.case.applicability != b.case.applicability
        }
        (EquipmentTask::SeparatorRating(a), EquipmentTask::SeparatorRating(b)) => {
            a.geometry.applicability != b.geometry.applicability
                || !a
                    .cases
                    .iter()
                    .map(|c| (&c.id, &c.fluid_system, &c.applicability))
                    .eq(b
                        .cases
                        .iter()
                        .map(|c| (&c.id, &c.fluid_system, &c.applicability)))
        }
        _ => true,
    };
    declarations_changed
        || old.method() != new.method()
        || commands::task_sources(old) != commands::task_sources(new)
        || !commands::task_bindings(old)
            .into_iter()
            .map(|(field, _, binding)| (field, binding_evidence(binding)))
            .eq(commands::task_bindings(new)
                .into_iter()
                .map(|(field, _, binding)| (field, binding_evidence(binding))))
}
fn source_evidence(source: &SourceRecord) -> SourceRecord {
    let mut evidence = source.clone();
    evidence.revision = 0;
    evidence.title.clear();
    evidence
}
pub(super) fn between(before: &EngineeringInputs, after: &EngineeringInputs) -> Vec<TaskImpact> {
    let ids = before
        .tasks
        .iter()
        .chain(&after.tasks)
        .flat_map(|r| {
            r.case_ids().map(|case| EquipmentCaseKey {
                task_id: r.id().clone(),
                case_id: case.clone(),
            })
        })
        .collect::<BTreeSet<_>>();
    let mut impacts = Vec::new();
    for id in ids {
        let old = case_projection(before.equipment_task(&id.task_id), &id.case_id);
        let new = case_projection(after.equipment_task(&id.task_id), &id.case_id);
        let mut input_changed = old.as_ref().map(task_recipe) != new.as_ref().map(task_recipe);
        let mut evidence_changed = task_evidence_changed(old.as_ref(), new.as_ref());
        // Both sides matter when a binding or dependency has just been removed.
        let mut dependencies = Dependencies::default();
        if let Some(task) = &old {
            dependencies.add_task(before, task);
        }
        if let Some(task) = &new {
            dependencies.add_task(after, task);
        }
        for id in &dependencies.basis {
            let old = before.basis.basis_entries.iter().find(|r| r.id == *id);
            let new = after.basis.basis_entries.iter().find(|r| r.id == *id);
            input_changed |= old.map(|r| r.value) != new.map(|r| r.value);
            if old.map(|r| &r.source_id) != new.map(|r| &r.source_id) {
                input_changed = true;
                evidence_changed = true;
            }
        }
        for id in &dependencies.sources {
            let old = before.basis.sources.iter().find(|r| r.id == *id);
            let new = after.basis.sources.iter().find(|r| r.id == *id);
            evidence_changed |= old.map(source_evidence) != new.map(source_evidence);
            input_changed |= old.map(|r| (&r.applicability, &r.observations))
                != new.map(|r| (&r.applicability, &r.observations));
        }
        if input_changed || evidence_changed {
            impacts.push(TaskImpact {
                task_id: id.task_id,
                case_id: id.case_id,
                input_changed,
                evidence_changed,
            });
        }
    }
    impacts
}
