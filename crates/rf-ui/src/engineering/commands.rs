use super::*;
use rf_model::{
    engineering::{BasisEntry, EngineeringDataError, InputBinding, SourceRecord},
    equipment::{
        EquipmentIdentityError, SeparatorAxialSection, SeparatorInput,
        validate_equipment_identities,
    },
};
use rf_types::units::QuantityKind;

/// Creates assign revision 1. Updates require the current record revision;
/// committed updates and history restoration receive a fresh monotonic revision.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineeringCommand {
    CreateSource(Box<SourceRecord>),
    UpdateSource(Box<SourceRecord>),
    DeleteSource(SourceId),
    CreateBasis(BasisEntry),
    UpdateBasis(BasisEntry),
    DeleteBasis(BasisEntryId),
    CreateTask(Box<SeparatorSizingTask>),
    UpdateTask(Box<SeparatorSizingTask>),
    DeleteTask(EquipmentTaskId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineeringReference {
    BasisSource(BasisEntryId),
    SourceObservation {
        source: SourceId,
        observation: rf_types::SourceObservationId,
    },
    TaskInput {
        task: EquipmentTaskId,
        field: SeparatorInput,
    },
    TaskAxialSection {
        task: EquipmentTaskId,
        section: SeparatorAxialSection,
    },
    TaskApplicability(EquipmentTaskId),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineeringError {
    InvalidBasis(EngineeringDataError),
    InvalidIdentity(EquipmentIdentityError),
    InvalidBinding {
        reference: EngineeringReference,
        error: EngineeringDataError,
    },
    MissingApplicabilitySource {
        task: EquipmentTaskId,
        source: SourceId,
    },
    Referenced {
        references: Vec<EngineeringReference>,
    },
    IdentityUnavailable(EngineeringRecordId),
    MissingRecord(EngineeringRecordId),
    RevisionConflict {
        record: EngineeringRecordId,
        expected: u64,
        supplied: u64,
    },
    CaseIdentityChanged(EquipmentTaskId),
    CounterExhausted,
    StalePreparation,
    StaleCalculation,
}
impl std::fmt::Display for EngineeringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "engineering operation rejected: {self:?}")
    }
}
impl std::error::Error for EngineeringError {}

/// Opaque proposal for one atomic transaction. The impact list is available before
/// publication, and a proposal cannot be applied to a later or foreign session.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedEngineeringEdit {
    identity: EngineeringCheckpoint,
    checkpoint: EngineeringCheckpoint,
    next: EngineeringInputs,
    impacts: Vec<TaskImpact>,
}
impl PreparedEngineeringEdit {
    pub fn impacts(&self) -> &[TaskImpact] {
        &self.impacts
    }
    pub fn inputs(&self) -> &EngineeringInputs {
        &self.next
    }
}

impl EngineeringSession {
    pub fn prepare(
        &self,
        commands: Vec<EngineeringCommand>,
    ) -> Result<PreparedEngineeringEdit, EngineeringError> {
        let mut next = self.current.clone();
        // Also reserve creations within this batch, even when a later command deletes them.
        let mut reserved = self.revisions.clone();
        let mut deletions = Vec::new();
        for command in commands {
            use EngineeringCommand::*;
            match command {
                CreateSource(record) => {
                    reserve(
                        &mut reserved,
                        EngineeringRecordId::Source(record.id.clone()),
                    )?;
                    next.basis.sources.push(*record);
                }
                UpdateSource(record) => update(
                    &mut next.basis.sources,
                    *record,
                    |r| &r.id,
                    |r| r.revision,
                    EngineeringRecordId::Source,
                )?,
                DeleteSource(id) => {
                    remove(
                        &mut next.basis.sources,
                        &id,
                        |r| &r.id,
                        EngineeringRecordId::Source,
                    )?;
                    deletions.push(EngineeringRecordId::Source(id));
                }
                CreateBasis(record) => {
                    reserve(&mut reserved, EngineeringRecordId::Basis(record.id.clone()))?;
                    next.basis.basis_entries.push(record);
                }
                UpdateBasis(record) => update(
                    &mut next.basis.basis_entries,
                    record,
                    |r| &r.id,
                    |r| r.revision,
                    EngineeringRecordId::Basis,
                )?,
                DeleteBasis(id) => {
                    remove(
                        &mut next.basis.basis_entries,
                        &id,
                        |r| &r.id,
                        EngineeringRecordId::Basis,
                    )?;
                    deletions.push(EngineeringRecordId::Basis(id));
                }
                CreateTask(record) => {
                    reserve(&mut reserved, EngineeringRecordId::Task(record.id.clone()))?;
                    reserve(
                        &mut reserved,
                        EngineeringRecordId::Case(record.case.id.clone()),
                    )?;
                    next.tasks.push(*record);
                }
                UpdateTask(record) => {
                    let previous = next.task(&record.id).ok_or_else(|| {
                        EngineeringError::MissingRecord(EngineeringRecordId::Task(
                            record.id.clone(),
                        ))
                    })?;
                    if previous.case.id != record.case.id {
                        return Err(EngineeringError::CaseIdentityChanged(record.id.clone()));
                    }
                    update(
                        &mut next.tasks,
                        *record,
                        |r| &r.id,
                        |r| r.revision,
                        EngineeringRecordId::Task,
                    )?;
                }
                DeleteTask(id) => {
                    remove(&mut next.tasks, &id, |r| &r.id, EngineeringRecordId::Task)?;
                }
            }
        }
        let references = deletions
            .iter()
            .flat_map(|id| references_to(&next, id))
            .collect::<Vec<_>>();
        if !references.is_empty() {
            return Err(EngineeringError::Referenced { references });
        }
        validate(&next)?;
        let impacts = impact::between(&self.current, &next);
        Ok(PreparedEngineeringEdit {
            identity: self.identity.clone(),
            checkpoint: self.checkpoint.clone(),
            next,
            impacts,
        })
    }

    pub fn commit(
        &mut self,
        prepared: PreparedEngineeringEdit,
    ) -> Result<Vec<TaskImpact>, EngineeringError> {
        if self.identity != prepared.identity || self.checkpoint != prepared.checkpoint {
            return Err(EngineeringError::StalePreparation);
        }
        if same_content(&self.current, &prepared.next) {
            return Ok(Vec::new());
        }
        let before = self.current.clone();
        // Stage counters and input publication too: an overflow must leave everything untouched.
        let mut staged = self.clone();
        staged.publish_inputs(prepared.next, &prepared.impacts)?;
        staged.undo.push(HistoryEntry {
            before,
            after: staged.current.clone(),
        });
        staged.redo.clear();
        *self = staged;
        Ok(prepared.impacts)
    }
    pub fn apply(
        &mut self,
        commands: Vec<EngineeringCommand>,
    ) -> Result<Vec<TaskImpact>, EngineeringError> {
        let prepared = self.prepare(commands)?;
        self.commit(prepared)
    }
    pub fn undo(&mut self) -> Result<Vec<TaskImpact>, EngineeringError> {
        self.restore_history(false)
    }
    pub fn redo(&mut self) -> Result<Vec<TaskImpact>, EngineeringError> {
        self.restore_history(true)
    }
    fn restore_history(&mut self, redo: bool) -> Result<Vec<TaskImpact>, EngineeringError> {
        let mut staged = self.clone();
        let Some(entry) = (if redo {
            &mut staged.redo
        } else {
            &mut staged.undo
        })
        .pop() else {
            return Ok(Vec::new());
        };
        let next = if redo {
            entry.after.clone()
        } else {
            entry.before.clone()
        };
        let impacts = impact::between(&staged.current, &next);
        staged.publish_inputs(next, &impacts)?;
        (if redo {
            &mut staged.undo
        } else {
            &mut staged.redo
        })
        .push(entry);
        *self = staged;
        Ok(impacts)
    }
    fn publish_inputs(
        &mut self,
        mut next: EngineeringInputs,
        impacts: &[TaskImpact],
    ) -> Result<(), EngineeringError> {
        for record in &mut next.basis.sources {
            let previous = self
                .current
                .basis
                .sources
                .iter()
                .find(|r| r.id == record.id);
            let mut normalized = record.clone();
            normalized.revision = previous.map_or(0, |r| r.revision);
            record.revision = self.next_revision(
                EngineeringRecordId::Source(record.id.clone()),
                previous != Some(&normalized),
            )?;
        }
        for record in &mut next.basis.basis_entries {
            let previous = self
                .current
                .basis
                .basis_entries
                .iter()
                .find(|r| r.id == record.id);
            let mut normalized = record.clone();
            normalized.revision = previous.map_or(0, |r| r.revision);
            record.revision = self.next_revision(
                EngineeringRecordId::Basis(record.id.clone()),
                previous != Some(&normalized),
            )?;
        }
        for record in &mut next.tasks {
            let previous = self.current.task(&record.id);
            let mut normalized = record.clone();
            normalized.revision = previous.map_or(0, |r| r.revision);
            record.revision = self.next_revision(
                EngineeringRecordId::Task(record.id.clone()),
                previous != Some(&normalized),
            )?;
            self.revisions
                .entry(EngineeringRecordId::Case(record.case.id.clone()))
                .or_insert(0);
        }
        for impact in impacts {
            let generation = self.generations.entry(impact.task_id.clone()).or_default();
            if impact.input_changed {
                generation.input = increment(generation.input)?;
            }
            if impact.evidence_changed {
                generation.evidence = increment(generation.evidence)?;
            }
            self.eligibility
                .insert(impact.task_id.clone(), EngineeringCheckpoint::fresh());
            self.pending.remove(&impact.task_id);
        }
        self.current = next;
        self.checkpoint = EngineeringCheckpoint::fresh();
        Ok(())
    }
    fn next_revision(
        &mut self,
        id: EngineeringRecordId,
        changed: bool,
    ) -> Result<u64, EngineeringError> {
        let revision = self.revisions.entry(id).or_insert(0);
        if changed {
            *revision = increment(*revision)?;
        }
        Ok(*revision)
    }
}
fn increment(value: u64) -> Result<u64, EngineeringError> {
    value
        .checked_add(1)
        .ok_or(EngineeringError::CounterExhausted)
}
fn reserve(
    reserved: &mut BTreeMap<EngineeringRecordId, u64>,
    id: EngineeringRecordId,
) -> Result<(), EngineeringError> {
    if reserved.insert(id.clone(), 0).is_some() {
        Err(EngineeringError::IdentityUnavailable(id))
    } else {
        Ok(())
    }
}
fn update<T, I: PartialEq + Clone>(
    records: &mut [T],
    record: T,
    id: impl Fn(&T) -> &I,
    revision: impl Fn(&T) -> u64,
    record_id: impl Fn(I) -> EngineeringRecordId,
) -> Result<(), EngineeringError> {
    let previous = records
        .iter_mut()
        .find(|r| id(r) == id(&record))
        .ok_or_else(|| EngineeringError::MissingRecord(record_id(id(&record).clone())))?;
    if revision(previous) != revision(&record) {
        return Err(EngineeringError::RevisionConflict {
            record: record_id(id(&record).clone()),
            expected: revision(previous),
            supplied: revision(&record),
        });
    }
    *previous = record;
    Ok(())
}
fn remove<T, I: PartialEq + Clone>(
    records: &mut Vec<T>,
    identity: &I,
    id: impl Fn(&T) -> &I,
    record_id: impl Fn(I) -> EngineeringRecordId,
) -> Result<(), EngineeringError> {
    let index = records
        .iter()
        .position(|r| id(r) == identity)
        .ok_or_else(|| EngineeringError::MissingRecord(record_id(identity.clone())))?;
    records.remove(index);
    Ok(())
}

pub(super) fn validate(inputs: &EngineeringInputs) -> Result<(), EngineeringError> {
    inputs
        .basis
        .validate()
        .map_err(EngineeringError::InvalidBasis)?;
    validate_equipment_identities(&inputs.tasks).map_err(EngineeringError::InvalidIdentity)?;
    for task in &inputs.tasks {
        for (field, binding) in &task.case.inputs {
            inputs
                .basis
                .resolve(field.quantity(), binding)
                .map_err(|error| EngineeringError::InvalidBinding {
                    reference: EngineeringReference::TaskInput {
                        task: task.id.clone(),
                        field: *field,
                    },
                    error,
                })?;
        }
        for (section, binding) in &task.axial_sections {
            inputs
                .basis
                .resolve(QuantityKind::Length, binding)
                .map_err(|error| EngineeringError::InvalidBinding {
                    reference: EngineeringReference::TaskAxialSection {
                        task: task.id.clone(),
                        section: *section,
                    },
                    error,
                })?;
        }
        for source in &task.case.applicability_source_ids {
            if !inputs.basis.sources.iter().any(|r| r.id == *source) {
                return Err(EngineeringError::MissingApplicabilitySource {
                    task: task.id.clone(),
                    source: source.clone(),
                });
            }
        }
    }
    Ok(())
}

fn binding_references(binding: &InputBinding, id: &EngineeringRecordId) -> bool {
    match (binding, id) {
        (
            InputBinding::Direct { source_id, .. } | InputBinding::Override { source_id, .. },
            EngineeringRecordId::Source(id),
        ) => source_id == id,
        (
            InputBinding::BasisRef { basis_entry_id }
            | InputBinding::Override { basis_entry_id, .. },
            EngineeringRecordId::Basis(id),
        ) => basis_entry_id == id,
        _ => false,
    }
}
fn references_to(
    inputs: &EngineeringInputs,
    id: &EngineeringRecordId,
) -> Vec<EngineeringReference> {
    let mut found = Vec::new();
    if let EngineeringRecordId::Source(id) = id {
        for entry in &inputs.basis.basis_entries {
            if entry.source_id == *id {
                found.push(EngineeringReference::BasisSource(entry.id.clone()));
            }
        }
        for source in &inputs.basis.sources {
            for observation in &source.observations {
                if observation.depends_on.iter().any(|r| r.source_id == *id) {
                    found.push(EngineeringReference::SourceObservation {
                        source: source.id.clone(),
                        observation: observation.id.clone(),
                    });
                }
            }
        }
    }
    for task in &inputs.tasks {
        for (field, binding) in &task.case.inputs {
            if binding_references(binding, id) {
                found.push(EngineeringReference::TaskInput {
                    task: task.id.clone(),
                    field: *field,
                });
            }
        }
        for (section, binding) in &task.axial_sections {
            if binding_references(binding, id) {
                found.push(EngineeringReference::TaskAxialSection {
                    task: task.id.clone(),
                    section: *section,
                });
            }
        }
        if let EngineeringRecordId::Source(id) = id
            && task.case.applicability_source_ids.contains(id)
        {
            found.push(EngineeringReference::TaskApplicability(task.id.clone()));
        }
    }
    found
}
pub(super) fn record_revisions(inputs: &EngineeringInputs) -> BTreeMap<EngineeringRecordId, u64> {
    inputs
        .basis
        .sources
        .iter()
        .map(|r| (EngineeringRecordId::Source(r.id.clone()), r.revision))
        .chain(
            inputs
                .basis
                .basis_entries
                .iter()
                .map(|r| (EngineeringRecordId::Basis(r.id.clone()), r.revision)),
        )
        .chain(inputs.tasks.iter().flat_map(|r| {
            [
                (EngineeringRecordId::Task(r.id.clone()), r.revision),
                (EngineeringRecordId::Case(r.case.id.clone()), 0),
            ]
        }))
        .collect()
}
pub(super) fn same_content(a: &EngineeringInputs, b: &EngineeringInputs) -> bool {
    fn normalized(inputs: &EngineeringInputs) -> EngineeringInputs {
        let mut value = inputs.clone();
        for r in &mut value.basis.sources {
            r.revision = 0;
        }
        for r in &mut value.basis.basis_entries {
            r.revision = 0;
        }
        for r in &mut value.tasks {
            r.revision = 0;
        }
        value
    }
    normalized(a) == normalized(b)
}
