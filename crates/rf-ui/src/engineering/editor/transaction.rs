use super::*;
use rf_model::{engineering::*, equipment::*};
use rf_types::units::QuantityKind;

impl EquipmentEditorState {
    pub(super) fn execute(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        command: EquipmentEditorCommand,
    ) -> Result<(), EquipmentEditError> {
        use EquipmentEditorCommand::*;
        match command {
            CreateTask => {
                let id = self.next_id(session, "task")?;
                let case_id = self.next_id(session, "case")?;
                session.apply(vec![EngineeringCommand::CreateTask(Box::new(
                    SeparatorSizingTask {
                        id: id.clone().into(),
                        revision: 0,
                        tag: String::new(),
                        name: String::new(),
                        method: separator_sizing::method(),
                        case: SeparatorCase {
                            id: case_id.into(),
                            name: String::new(),
                            purpose: String::new(),
                            fluid_system: String::new(),
                            applicability_source_ids: vec![],
                            inputs: BTreeMap::new(),
                            applicability: SeparatorApplicability::default(),
                        },
                        axial_sections: BTreeMap::new(),
                    },
                ))])?;
                self.select(session, units, EquipmentEditorTarget::Task(id.into()))?;
            }
            CreateSource => {
                let id = self.next_id(session, "source")?;
                session.apply(vec![EngineeringCommand::CreateSource(Box::new(
                    SourceRecord {
                        id: id.clone().into(),
                        revision: 0,
                        kind: SourceKind::Unknown,
                        title: String::new(),
                        reference: String::new(),
                        locator: String::new(),
                        publication_version: String::new(),
                        accessed_on: None,
                        adoption_note: String::new(),
                        review: SourceReview::Unreviewed,
                        applicability: SourceApplicability {
                            fluid_system: String::new(),
                            phase: SourcePhase::NotApplicable,
                            operating_conditions: None,
                            scope: String::new(),
                        },
                        observations: vec![],
                    },
                ))])?;
                self.select(session, units, EquipmentEditorTarget::Source(id.into()))?;
            }
            NewBasis => {
                let target = EquipmentEditorTarget::Basis(self.next_id(session, "basis")?.into());
                let value = EquipmentDraft::Basis(BasisDraft {
                    name: String::new(),
                    quantity: QuantityKind::Duration,
                    value: QuantityDraft::empty(units.unit_for(QuantityKind::Duration)),
                    source: None,
                });
                let generation = self.bump()?;
                self.drafts.insert(
                    target.clone(),
                    EquipmentDraftRecord {
                        baseline: RecordBaseline::Basis(None),
                        original: value.clone(),
                        value,
                        generation,
                    },
                );
                self.active = Some(target);
            }
            Select(target) => self.select(session, units, target)?,
            Change {
                target,
                generation,
                value,
            } => {
                let record = self
                    .drafts
                    .get(&target)
                    .ok_or(EquipmentEditError::MissingEditor)?;
                if record.generation != generation {
                    return Err(EquipmentEditError::StaleEditor);
                }
                if std::mem::discriminant(&record.value) != std::mem::discriminant(value.as_ref()) {
                    return Err(EquipmentEditError::DraftTypeMismatch);
                }
                if record.value != *value {
                    let generation = self.bump()?;
                    let record = self.drafts.get_mut(&target).expect("checked");
                    record.value = *value;
                    record.generation = generation;
                }
            }
            Apply(target) => {
                let command = self.draft_command(session, &target)?;
                let prepared = session.prepare(vec![command])?;
                // Shared records are always reviewed with their affected tasks before commit.
                if !matches!(target, EquipmentEditorTarget::Task(_))
                    && !prepared.impacts().is_empty()
                {
                    self.set_pending(session, PendingOperation::Apply { prepared, target });
                } else {
                    session.commit(prepared)?;
                    self.reload_target(session, units, &target)?;
                }
            }
            Discard(target) => {
                self.reload_target(session, units, &target)?;
            }
            Delete(target) => {
                let command = match &target {
                    EquipmentEditorTarget::Task(id) => EngineeringCommand::DeleteTask(id.clone()),
                    EquipmentEditorTarget::Source(id) => {
                        EngineeringCommand::DeleteSource(id.clone())
                    }
                    EquipmentEditorTarget::Basis(id) => EngineeringCommand::DeleteBasis(id.clone()),
                };
                let prepared = session.prepare(vec![command])?;
                self.set_pending(session, PendingOperation::Delete { prepared, target });
            }
            History { redo } => {
                if self.pending_count() != 0 {
                    self.set_pending(session, PendingOperation::History { redo });
                } else {
                    self.restore_history(session, units, redo)?;
                }
            }
            Confirm(key) => {
                let pending = self
                    .pending
                    .clone()
                    .ok_or(EquipmentEditError::StaleEditor)?;
                if pending.key != key
                    || pending.editor_checkpoint != self.checkpoint
                    || pending.formal_checkpoint != session.checkpoint()
                {
                    return Err(EquipmentEditError::StaleEditor);
                }
                match pending.operation {
                    PendingOperation::Apply { prepared, target } => {
                        session.commit(prepared)?;
                        self.reload_target(session, units, &target)?;
                    }
                    PendingOperation::Delete { prepared, target } => {
                        session.commit(prepared)?;
                        self.drafts.remove(&target);
                        if self.active.as_ref() == Some(&target) {
                            self.active = None;
                        }
                        self.bump()?;
                    }
                    PendingOperation::History { redo } => {
                        self.restore_history(session, units, redo)?
                    }
                }
                self.pending = None;
            }
            CancelConfirmation => self.pending = None,
            Check(task) => {
                if self.pending_for_task(&task, session) {
                    return Err(EquipmentEditError::PendingDrafts);
                }
                let record = session
                    .inputs()
                    .task(&task)
                    .ok_or(EquipmentEditError::MissingTask)?;
                self.checks.insert(
                    task,
                    InputCheck {
                        formal: session.checkpoint(),
                        issues: separator_sizing::check(record, &session.inputs().basis),
                    },
                );
            }
            Calculate(task) => {
                if self.pending_for_task(&task, session) {
                    return Err(EquipmentEditError::PendingDrafts);
                }
                session.calculate(&task)?;
                self.checks.remove(&task);
            }
        }
        Ok(())
    }
    fn next_id(
        &mut self,
        session: &EngineeringSession,
        prefix: &str,
    ) -> Result<String, EquipmentEditError> {
        loop {
            self.next_identity = self
                .next_identity
                .checked_add(1)
                .ok_or(EquipmentEditError::CounterExhausted)?;
            let id = format!("{prefix}-ui-{}", self.next_identity);
            if !session.revisions.keys().any(|r| match r {
                EngineeringRecordId::Task(v) => v.as_str() == id,
                EngineeringRecordId::Case(v) => v.as_str() == id,
                EngineeringRecordId::Source(v) => v.as_str() == id,
                EngineeringRecordId::Basis(v) => v.as_str() == id,
            }) {
                return Ok(id);
            }
        }
    }
    fn select(
        &mut self,
        session: &EngineeringSession,
        units: &DisplayUnitSet,
        target: EquipmentEditorTarget,
    ) -> Result<(), EquipmentEditError> {
        // Refresh clean records after an external formal edit; never overwrite a pending draft.
        if self.drafts.get(&target).is_none_or(|d| !d.is_pending()) {
            let (baseline, value) = match &target {
                EquipmentEditorTarget::Task(id) => {
                    let record = session
                        .inputs()
                        .task(id)
                        .ok_or(EquipmentEditError::MissingTask)?;
                    (
                        RecordBaseline::Task(record.clone()),
                        EquipmentDraft::Task(SeparatorTaskDraft::from_record(record, units)),
                    )
                }
                EquipmentEditorTarget::Source(id) => {
                    let record = session
                        .inputs()
                        .basis
                        .sources
                        .iter()
                        .find(|r| r.id == *id)
                        .ok_or(EquipmentEditError::MissingEditor)?;
                    (
                        RecordBaseline::Source(record.clone()),
                        EquipmentDraft::Source(SourceDraft::from_record(record, units)),
                    )
                }
                EquipmentEditorTarget::Basis(id) => {
                    let record = session
                        .inputs()
                        .basis
                        .basis_entries
                        .iter()
                        .find(|r| r.id == *id)
                        .ok_or(EquipmentEditError::MissingEditor)?;
                    (
                        RecordBaseline::Basis(Some(record.clone())),
                        EquipmentDraft::Basis(BasisDraft::from_record(record, units)),
                    )
                }
            };
            self.next_generation = self
                .next_generation
                .checked_add(1)
                .ok_or(EquipmentEditError::CounterExhausted)?;
            self.drafts.insert(
                target.clone(),
                EquipmentDraftRecord {
                    baseline,
                    original: value.clone(),
                    value,
                    generation: self.next_generation,
                },
            );
        }
        self.active = Some(target);
        Ok(())
    }
    fn reload_target(
        &mut self,
        session: &EngineeringSession,
        units: &DisplayUnitSet,
        target: &EquipmentEditorTarget,
    ) -> Result<(), EquipmentEditError> {
        self.drafts.remove(target);
        self.bump()?;
        let exists = match target {
            EquipmentEditorTarget::Task(id) => session.inputs().task(id).is_some(),
            EquipmentEditorTarget::Source(id) => {
                session.inputs().basis.sources.iter().any(|r| r.id == *id)
            }
            EquipmentEditorTarget::Basis(id) => session
                .inputs()
                .basis
                .basis_entries
                .iter()
                .any(|r| r.id == *id),
        };
        if self.active.as_ref() == Some(target) {
            if exists {
                self.select(session, units, target.clone())?;
            } else {
                self.active = None;
            }
        }
        Ok(())
    }
    fn draft_command(
        &self,
        session: &EngineeringSession,
        target: &EquipmentEditorTarget,
    ) -> Result<EngineeringCommand, EquipmentEditError> {
        let draft = self
            .drafts
            .get(target)
            .ok_or(EquipmentEditError::MissingEditor)?;
        match (&draft.baseline, &draft.value, target) {
            (
                RecordBaseline::Task(base),
                EquipmentDraft::Task(value),
                EquipmentEditorTarget::Task(id),
            ) => {
                if session.inputs().task(id) != Some(base) {
                    return Err(EquipmentEditError::StaleEditor);
                }
                Ok(EngineeringCommand::UpdateTask(Box::new(
                    value.record(base)?,
                )))
            }
            (
                RecordBaseline::Source(base),
                EquipmentDraft::Source(value),
                EquipmentEditorTarget::Source(id),
            ) => {
                if session.inputs().basis.sources.iter().find(|r| r.id == *id) != Some(base) {
                    return Err(EquipmentEditError::StaleEditor);
                }
                Ok(EngineeringCommand::UpdateSource(Box::new(
                    value.record(base)?,
                )))
            }
            (
                RecordBaseline::Basis(base),
                EquipmentDraft::Basis(value),
                EquipmentEditorTarget::Basis(id),
            ) => {
                if session
                    .inputs()
                    .basis
                    .basis_entries
                    .iter()
                    .find(|r| r.id == *id)
                    != base.as_ref()
                {
                    return Err(EquipmentEditError::StaleEditor);
                }
                let record = value.record(id.clone(), base.as_ref().map_or(0, |b| b.revision))?;
                Ok(if base.is_some() {
                    EngineeringCommand::UpdateBasis(record)
                } else {
                    EngineeringCommand::CreateBasis(record)
                })
            }
            _ => Err(EquipmentEditError::DraftTypeMismatch),
        }
    }
    fn set_pending(&mut self, session: &EngineeringSession, operation: PendingOperation) {
        let impacts = match &operation {
            PendingOperation::Apply { prepared, .. }
            | PendingOperation::Delete { prepared, .. } => prepared.impacts().to_vec(),
            PendingOperation::History { .. } => vec![],
        };
        let discarded_drafts = match &operation {
            PendingOperation::Apply { .. } => vec![],
            PendingOperation::Delete { target, .. } => self
                .pending_targets()
                .into_iter()
                .filter(|t| t == target)
                .collect(),
            PendingOperation::History { .. } => self.pending_targets(),
        };
        self.pending = Some(EquipmentConfirmation {
            key: EngineeringCheckpoint::fresh(),
            editor_checkpoint: self.checkpoint(),
            formal_checkpoint: session.checkpoint(),
            operation,
            impacts,
            discarded_drafts,
        });
    }
    fn restore_history(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        redo: bool,
    ) -> Result<(), EquipmentEditError> {
        if redo {
            session.redo()?;
        } else {
            session.undo()?;
        }
        let active = self.active.take();
        self.drafts.clear();
        self.bump()?;
        if let Some(target) = active {
            let exists = match &target {
                EquipmentEditorTarget::Task(id) => session.inputs().task(id).is_some(),
                EquipmentEditorTarget::Source(id) => {
                    session.inputs().basis.sources.iter().any(|r| r.id == *id)
                }
                EquipmentEditorTarget::Basis(id) => session
                    .inputs()
                    .basis
                    .basis_entries
                    .iter()
                    .any(|r| r.id == *id),
            };
            if exists {
                self.select(session, units, target)?;
            }
        }
        Ok(())
    }
}
