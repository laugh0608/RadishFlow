use super::*;
use rf_model::engineering::{InputBinding, SourceRecord};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskImpact {
    pub task_id: EquipmentTaskId,
    pub input_changed: bool,
    pub evidence_changed: bool,
}

#[derive(Default)]
struct Dependencies {
    basis: BTreeSet<BasisEntryId>,
    sources: BTreeSet<SourceId>,
}
impl Dependencies {
    fn add_task(&mut self, inputs: &EngineeringInputs, task: &SeparatorSizingTask) {
        self.sources
            .extend(task.case.applicability_source_ids.iter().cloned());
        for binding in task
            .case
            .inputs
            .values()
            .chain(task.axial_sections.values())
        {
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
fn task_recipe(task: &SeparatorSizingTask) -> SeparatorSizingTask {
    let mut recipe = task.clone();
    recipe.revision = 0;
    recipe.tag.clear();
    recipe.name.clear();
    recipe.case.name.clear();
    recipe.case.purpose.clear();
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
fn task_evidence_changed(
    old: Option<&SeparatorSizingTask>,
    new: Option<&SeparatorSizingTask>,
) -> bool {
    let (Some(old), Some(new)) = (old, new) else {
        return true;
    };
    old.case.id != new.case.id
        || old.method != new.method
        || old.case.fluid_system != new.case.fluid_system
        || old.case.applicability != new.case.applicability
        || old.case.applicability_source_ids != new.case.applicability_source_ids
        || !old
            .case
            .inputs
            .iter()
            .map(|(field, binding)| (field, binding_evidence(binding)))
            .eq(new
                .case
                .inputs
                .iter()
                .map(|(field, binding)| (field, binding_evidence(binding))))
        || !old
            .axial_sections
            .iter()
            .map(|(section, binding)| (section, binding_evidence(binding)))
            .eq(new
                .axial_sections
                .iter()
                .map(|(section, binding)| (section, binding_evidence(binding))))
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
        .map(|r| r.id.clone())
        .collect::<BTreeSet<_>>();
    let mut impacts = Vec::new();
    for id in ids {
        let old = before.task(&id);
        let new = after.task(&id);
        let mut input_changed = old.map(task_recipe) != new.map(task_recipe);
        let mut evidence_changed = task_evidence_changed(old, new);
        // Both sides matter when a binding or dependency has just been removed.
        let mut dependencies = Dependencies::default();
        if let Some(task) = old {
            dependencies.add_task(before, task);
        }
        if let Some(task) = new {
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
                task_id: id,
                input_changed,
                evidence_changed,
            });
        }
    }
    impacts
}
