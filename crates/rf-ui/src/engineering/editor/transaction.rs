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
            PreviewGeometryAdoption(draft) => self.preview_geometry_adoption(session, *draft)?,
            ConfirmGeometryAdoption(key) => self.confirm_geometry_adoption(session, units, key)?,
            CancelGeometryAdoption => self.adoption_preview = None,
            CreateRatingTask => self.create_rating(session, units)?,
            AddRatingCase { task, copy_from } => {
                self.add_rating_case(session, units, task, copy_from)?
            }
            ReorderRatingCases { task, cases } => {
                session.apply(vec![EngineeringCommand::ReorderRatingCases { task, cases }])?;
            }
            CheckRating(member) => {
                if self.pending_for_case(&member, session) {
                    return Err(EquipmentEditError::PendingDrafts);
                }
                let task = session
                    .inputs()
                    .rating_task(&member.task_id)
                    .ok_or(EquipmentEditError::MissingTask)?;
                if !task.cases.iter().any(|c| c.id == member.case_id) {
                    return Err(EquipmentEditError::MissingEditor);
                }
                self.rating_checks.insert(
                    member.clone(),
                    RatingInputCheck {
                        session: session.identity.clone(),
                        formal: session
                            .case_generation(&member)
                            .ok_or(EquipmentEditError::MissingEditor)?,
                        issues: rf_unitops::separator_rating::check_case(
                            task,
                            &member.case_id,
                            &session.inputs().basis,
                        ),
                    },
                );
            }
            CalculateRating(member) => {
                let request = self.prepare_calculation(session, &CalculateRating(member))?;
                session.complete_calculation(request.execute())?;
            }
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
                self.submit_input(session, units, target, command)?;
            }
            Discard(target) => {
                self.reload_target(session, units, &target)?;
            }
            Delete(target) => {
                let command = match &target {
                    EquipmentEditorTarget::Task(id) | EquipmentEditorTarget::RatingTask(id) => {
                        EngineeringCommand::DeleteTask(id.clone())
                    }
                    EquipmentEditorTarget::RatingCase(key) => {
                        EngineeringCommand::DeleteRatingCase(key.clone())
                    }
                    EquipmentEditorTarget::RatingGeometry(_) => {
                        return Err(EquipmentEditError::UnsupportedEditorAction);
                    }
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
                    PendingOperation::AddCase { prepared, target } => {
                        session.commit(prepared)?;
                        self.select(session, units, target)?;
                        self.bump()?;
                    }
                    PendingOperation::Apply { prepared, target } => {
                        session.commit(prepared)?;
                        self.reload_target(session, units, &target)?;
                    }
                    PendingOperation::Delete { prepared, target } => {
                        session.commit(prepared)?;
                        self.drafts
                            .retain(|candidate, _| !Self::deleted_with(candidate, &target));
                        self.target_errors
                            .retain(|candidate, _| !Self::deleted_with(candidate, &target));
                        if self
                            .active
                            .as_ref()
                            .is_some_and(|candidate| Self::deleted_with(candidate, &target))
                        {
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
                let request = self.prepare_calculation(session, &Calculate(task))?;
                session.complete_calculation(request.execute())?;
            }
        }
        Ok(())
    }
    pub(super) fn next_id(
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
                EngineeringRecordId::Task(v) | EngineeringRecordId::Geometry(v) => v.as_str() == id,
                EngineeringRecordId::Case(v) => v.as_str() == id,
                EngineeringRecordId::Source(v) => v.as_str() == id,
                EngineeringRecordId::Basis(v) => v.as_str() == id,
            }) {
                return Ok(id);
            }
        }
    }
    pub(super) fn select(
        &mut self,
        session: &EngineeringSession,
        units: &DisplayUnitSet,
        target: EquipmentEditorTarget,
    ) -> Result<(), EquipmentEditError> {
        // Refresh clean records after an external formal edit; never overwrite a pending draft.
        if self.drafts.get(&target).is_none_or(|d| !d.is_pending()) {
            let (baseline, value) = match &target {
                EquipmentEditorTarget::RatingTask(_)
                | EquipmentEditorTarget::RatingGeometry(_)
                | EquipmentEditorTarget::RatingCase(_) => {
                    Self::rating_record(session, units, &target)?
                }
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
    pub(super) fn reload_target(
        &mut self,
        session: &EngineeringSession,
        units: &DisplayUnitSet,
        target: &EquipmentEditorTarget,
    ) -> Result<(), EquipmentEditError> {
        self.drafts.remove(target);
        self.bump()?;
        let exists = Self::target_exists(session, target);
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
                RecordBaseline::RatingTask(base),
                EquipmentDraft::RatingTask(value),
                EquipmentEditorTarget::RatingTask(id),
            ) => {
                if session.inputs().rating_task(id) != Some(base) {
                    return Err(EquipmentEditError::StaleEditor);
                }
                Ok(EngineeringCommand::UpdateRatingLabels {
                    task: id.clone(),
                    revision: base.revision,
                    tag: value.tag.clone(),
                    name: value.name.clone(),
                })
            }
            (
                RecordBaseline::RatingGeometry(base),
                EquipmentDraft::RatingGeometry(value),
                EquipmentEditorTarget::RatingGeometry(id),
            ) => {
                if session.inputs().rating_task(id).map(|t| &t.geometry) != Some(base) {
                    return Err(EquipmentEditError::StaleEditor);
                }
                Ok(EngineeringCommand::UpdateRatingGeometry {
                    task: id.clone(),
                    geometry: Box::new(value.record(base)?),
                })
            }
            (
                RecordBaseline::RatingCase(base),
                EquipmentDraft::RatingCase(value),
                EquipmentEditorTarget::RatingCase(key),
            ) => {
                if session
                    .inputs()
                    .rating_task(&key.task_id)
                    .and_then(|t| t.cases.iter().find(|c| c.id == key.case_id))
                    != Some(base)
                {
                    return Err(EquipmentEditError::StaleEditor);
                }
                Ok(EngineeringCommand::UpdateRatingCase {
                    task: key.task_id.clone(),
                    case: Box::new(value.record(base)?),
                })
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
    pub(super) fn set_pending(
        &mut self,
        session: &EngineeringSession,
        operation: PendingOperation,
    ) {
        let impacts = match &operation {
            PendingOperation::AddCase { prepared, .. }
            | PendingOperation::Apply { prepared, .. }
            | PendingOperation::Delete { prepared, .. } => prepared.impacts().to_vec(),
            PendingOperation::History { .. } => vec![],
        };
        let discarded_drafts = match &operation {
            PendingOperation::Apply { .. } | PendingOperation::AddCase { .. } => vec![],
            PendingOperation::Delete { target, .. } => self
                .pending_targets()
                .into_iter()
                .filter(|t| Self::deleted_with(t, target))
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
    fn deleted_with(candidate: &EquipmentEditorTarget, deleted: &EquipmentEditorTarget) -> bool {
        if let EquipmentEditorTarget::RatingTask(id) = deleted {
            match candidate {
                EquipmentEditorTarget::RatingTask(other)
                | EquipmentEditorTarget::RatingGeometry(other) => other == id,
                EquipmentEditorTarget::RatingCase(key) => key.task_id == *id,
                _ => false,
            }
        } else {
            candidate == deleted
        }
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
        self.target_errors.clear();
        self.bump()?;
        if let Some(target) = active {
            let exists = Self::target_exists(session, &target);
            if exists {
                self.select(session, units, target)?;
            }
        }
        Ok(())
    }
}
