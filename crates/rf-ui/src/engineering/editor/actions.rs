//! Typed formal actions. Text editing stays in `EquipmentEditorCommand`.
use super::*;
use rf_model::equipment::{SeparatorGivenGeometry, SeparatorRatingCase};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentActionContext {
    document: crate::DocumentId,
    session: EngineeringCheckpoint,
    input: EngineeringCheckpoint,
}
impl EquipmentActionContext {
    pub fn capture(session: &EngineeringSession) -> Self {
        Self {
            document: session.document_id.clone(),
            session: session.identity.clone(),
            input: session.checkpoint(),
        }
    }
    pub fn validate(&self, session: &EngineeringSession) -> Result<(), EquipmentActionError> {
        if self.document != session.document_id {
            return Err(EquipmentActionError::DifferentDocument);
        }
        if self.session != session.identity {
            return Err(EquipmentActionError::DifferentSession);
        }
        if self.input != session.checkpoint() {
            return Err(EquipmentActionError::StaleInput);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentEditAccess {
    Editable,
    ReadOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentActionError {
    ReadOnly,
    DifferentDocument,
    DifferentSession,
    StaleInput,
    Edit(EquipmentEditError),
}
impl From<EquipmentEditError> for EquipmentActionError {
    fn from(value: EquipmentEditError) -> Self {
        Self::Edit(value)
    }
}
impl std::fmt::Display for EquipmentActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "equipment action rejected: {self:?}")
    }
}
impl std::error::Error for EquipmentActionError {}

/// Formal records carry SI values and their current revisions. Creation uses
/// the editor's identity allocator; callers never supply a new record identity.
#[derive(Debug, Clone, PartialEq)]
pub enum EquipmentAction {
    CreateSizingTask,
    CreateRatingTask,
    CreateSource,
    CreateBasis {
        name: String,
        value: rf_model::engineering::EngineeringValue,
        source_id: SourceId,
    },
    UpdateSizingTask(Box<SeparatorSizingTask>),
    UpdateSource(Box<SourceRecord>),
    UpdateBasis(BasisEntry),
    UpdateRatingLabels {
        task: EquipmentTaskId,
        revision: u64,
        tag: String,
        name: String,
    },
    UpdateRatingGeometry {
        task: EquipmentTaskId,
        geometry: Box<SeparatorGivenGeometry>,
    },
    UpdateRatingCase {
        task: EquipmentTaskId,
        case: Box<SeparatorRatingCase>,
    },
    AddRatingCase {
        task: EquipmentTaskId,
        copy_from: Option<EngineeringCaseId>,
    },
    ReorderRatingCases {
        task: EquipmentTaskId,
        cases: Vec<EngineeringCaseId>,
    },
    Delete(EquipmentEditorTarget),
    History {
        redo: bool,
    },
    Confirm(EngineeringCheckpoint),
    CancelConfirmation,
    CheckSizing(EquipmentTaskId),
    CheckRating(EquipmentCaseKey),
    PreviewAdoption {
        source: EquipmentTaskId,
        target: GeometryAdoptionDestination,
        adjustments: GeometryAdjustments,
    },
    ConfirmAdoption(EngineeringCheckpoint),
    CancelAdoption,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentActionOutcome {
    Committed,
    Unchanged,
    ConfirmationRequired {
        key: EngineeringCheckpoint,
        discarded_drafts: Vec<EquipmentEditorTarget>,
    },
    AdoptionConfirmationRequired {
        key: EngineeringCheckpoint,
    },
    SizingChecked {
        task: EquipmentTaskId,
        issues: Vec<SizingIssue>,
    },
    RatingChecked {
        member: EquipmentCaseKey,
        issues: Vec<separator_rating::RatingIssue>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentActionReceipt {
    pub before: EquipmentActionContext,
    pub after: EquipmentActionContext,
    pub outcome: EquipmentActionOutcome,
    pub target: Option<EquipmentEditorTarget>,
    pub impacts: Vec<TaskImpact>,
}

impl EquipmentEditorState {
    /// Preflight executes identical validation on isolated state, without issuing
    /// a confirmation token that could subsequently be committed.
    pub fn preflight_action(
        &self,
        session: &EngineeringSession,
        units: &DisplayUnitSet,
        access: EquipmentEditAccess,
        context: &EquipmentActionContext,
        action: EquipmentAction,
    ) -> Result<(), EquipmentActionError> {
        self.clone()
            .execute_action(&mut session.clone(), units, access, context, action)
            .map(|_| ())
    }

    pub fn execute_action(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        access: EquipmentEditAccess,
        context: &EquipmentActionContext,
        action: EquipmentAction,
    ) -> Result<EquipmentActionReceipt, EquipmentActionError> {
        if access != EquipmentEditAccess::Editable {
            return Err(EquipmentActionError::ReadOnly);
        }
        context.validate(session)?;
        let before = EquipmentActionContext::capture(session);
        let old_inputs = session.inputs().clone();
        let mut editor = self.clone();
        let mut formal = session.clone();
        if !matches!(
            action,
            EquipmentAction::Confirm(_)
                | EquipmentAction::ConfirmAdoption(_)
                | EquipmentAction::CancelConfirmation
                | EquipmentAction::CancelAdoption
        ) {
            editor.pending = None;
            editor.adoption_preview = None;
        }
        let (target, check) = editor.perform_action(&mut formal, units, action)?;
        let mut impacts = super::super::impact::between(&old_inputs, formal.inputs());
        let outcome = if let Some(check) = check {
            check
        } else if let Some(pending) = &editor.pending {
            impacts = pending.impacts.clone();
            EquipmentActionOutcome::ConfirmationRequired {
                key: pending.key(),
                discarded_drafts: pending.discarded_drafts.clone(),
            }
        } else if let Some(preview) = &editor.adoption_preview {
            impacts = preview.proposal().impacts().to_vec();
            EquipmentActionOutcome::AdoptionConfirmationRequired { key: preview.key() }
        } else if before == EquipmentActionContext::capture(&formal) {
            EquipmentActionOutcome::Unchanged
        } else {
            EquipmentActionOutcome::Committed
        };
        let receipt = EquipmentActionReceipt {
            before,
            after: EquipmentActionContext::capture(&formal),
            outcome,
            target,
            impacts,
        };
        editor.error = None;
        *self = editor;
        *session = formal;
        Ok(receipt)
    }

    /// Shared by validated GUI drafts and typed program inputs. No public flag
    /// can bypass the program entry's draft guard.
    pub(super) fn submit_input(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        target: EquipmentEditorTarget,
        command: EngineeringCommand,
    ) -> Result<(), EquipmentEditError> {
        let prepared = session.prepare(vec![command])?;
        if !matches!(
            target,
            EquipmentEditorTarget::Task(_)
                | EquipmentEditorTarget::RatingTask(_)
                | EquipmentEditorTarget::RatingCase(_)
        ) && !prepared.impacts().is_empty()
        {
            self.set_pending(session, PendingOperation::Apply { prepared, target });
        } else {
            session.commit(prepared)?;
            self.reload_target(session, units, &target)?;
        }
        Ok(())
    }

    fn perform_action(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        action: EquipmentAction,
    ) -> Result<
        (
            Option<EquipmentEditorTarget>,
            Option<EquipmentActionOutcome>,
        ),
        EquipmentEditError,
    > {
        use EquipmentAction::*;
        let update = match action {
            UpdateSizingTask(record) => Some((
                EquipmentEditorTarget::Task(record.id.clone()),
                EngineeringCommand::UpdateTask(record),
            )),
            UpdateSource(record) => Some((
                EquipmentEditorTarget::Source(record.id.clone()),
                EngineeringCommand::UpdateSource(record),
            )),
            UpdateBasis(record) => Some((
                EquipmentEditorTarget::Basis(record.id.clone()),
                EngineeringCommand::UpdateBasis(record),
            )),
            CreateBasis {
                name,
                value,
                source_id,
            } => {
                let record = BasisEntry {
                    id: self.next_id(session, "basis")?.into(),
                    revision: 0,
                    name,
                    value,
                    source_id,
                };
                Some((
                    EquipmentEditorTarget::Basis(record.id.clone()),
                    EngineeringCommand::CreateBasis(record),
                ))
            }
            UpdateRatingLabels {
                task,
                revision,
                tag,
                name,
            } => Some((
                EquipmentEditorTarget::RatingTask(task.clone()),
                EngineeringCommand::UpdateRatingLabels {
                    task,
                    revision,
                    tag,
                    name,
                },
            )),
            UpdateRatingGeometry { task, geometry } => Some((
                EquipmentEditorTarget::RatingGeometry(task.clone()),
                EngineeringCommand::UpdateRatingGeometry { task, geometry },
            )),
            UpdateRatingCase { task, case } => Some((
                EquipmentEditorTarget::RatingCase(EquipmentCaseKey {
                    task_id: task.clone(),
                    case_id: case.id.clone(),
                }),
                EngineeringCommand::UpdateRatingCase { task, case },
            )),
            other => {
                let command = match other {
                    CreateSizingTask => EquipmentEditorCommand::CreateTask,
                    CreateRatingTask => EquipmentEditorCommand::CreateRatingTask,
                    CreateSource => EquipmentEditorCommand::CreateSource,
                    AddRatingCase { task, copy_from } => {
                        EquipmentEditorCommand::AddRatingCase { task, copy_from }
                    }
                    ReorderRatingCases { task, cases } => {
                        EquipmentEditorCommand::ReorderRatingCases { task, cases }
                    }
                    Delete(target) => EquipmentEditorCommand::Delete(target),
                    History { redo } => EquipmentEditorCommand::History { redo },
                    Confirm(key) => EquipmentEditorCommand::Confirm(key),
                    CancelConfirmation => EquipmentEditorCommand::CancelConfirmation,
                    ConfirmAdoption(key) => EquipmentEditorCommand::ConfirmGeometryAdoption(key),
                    CancelAdoption => EquipmentEditorCommand::CancelGeometryAdoption,
                    CheckSizing(task) => {
                        self.execute(session, units, EquipmentEditorCommand::Check(task.clone()))?;
                        return Ok((
                            Some(EquipmentEditorTarget::Task(task.clone())),
                            Some(EquipmentActionOutcome::SizingChecked {
                                issues: self.checked_issues(&task, session).unwrap().to_vec(),
                                task,
                            }),
                        ));
                    }
                    CheckRating(member) => {
                        self.execute(
                            session,
                            units,
                            EquipmentEditorCommand::CheckRating(member.clone()),
                        )?;
                        return Ok((
                            Some(EquipmentEditorTarget::RatingCase(member.clone())),
                            Some(EquipmentActionOutcome::RatingChecked {
                                issues: self
                                    .checked_rating_issues(&member, session)
                                    .unwrap()
                                    .to_vec(),
                                member,
                            }),
                        ));
                    }
                    PreviewAdoption {
                        source,
                        target,
                        adjustments,
                    } => {
                        self.preview_typed_adoption(session, source, target, adjustments)?;
                        return Ok((
                            self.adoption_preview.as_ref().map(|preview| {
                                EquipmentEditorTarget::RatingGeometry(
                                    preview.proposal().target().task_id().clone(),
                                )
                            }),
                            None,
                        ));
                    }
                    _ => unreachable!(),
                };
                let target = match &command {
                    EquipmentEditorCommand::Confirm(_) => {
                        self.pending
                            .as_ref()
                            .and_then(|pending| match &pending.operation {
                                PendingOperation::AddCase { target, .. }
                                | PendingOperation::Apply { target, .. }
                                | PendingOperation::Delete { target, .. } => Some(target.clone()),
                                PendingOperation::History { .. } => None,
                            })
                    }
                    EquipmentEditorCommand::ConfirmGeometryAdoption(_) => {
                        self.adoption_preview.as_ref().map(|preview| {
                            EquipmentEditorTarget::RatingGeometry(
                                preview.proposal().target().task_id().clone(),
                            )
                        })
                    }
                    _ => command.error_target(),
                };
                let creates = matches!(
                    command,
                    EquipmentEditorCommand::CreateTask
                        | EquipmentEditorCommand::CreateRatingTask
                        | EquipmentEditorCommand::CreateSource
                        | EquipmentEditorCommand::AddRatingCase { .. }
                );
                self.execute(session, units, command)?;
                let target = if creates {
                    self.pending
                        .as_ref()
                        .and_then(|pending| match &pending.operation {
                            PendingOperation::AddCase { target, .. } => Some(target.clone()),
                            _ => None,
                        })
                        .or_else(|| self.active.clone())
                } else {
                    target
                };
                return Ok((target, None));
            }
        };
        let (target, command) = update.expect("formal input");
        if self.drafts.get(&target).is_some_and(|d| d.is_pending()) {
            return Err(EquipmentEditError::PendingDrafts);
        }
        let related_pending = match &target {
            EquipmentEditorTarget::Task(id)
            | EquipmentEditorTarget::RatingTask(id)
            | EquipmentEditorTarget::RatingGeometry(id) => self.pending_for_task(id, session),
            EquipmentEditorTarget::RatingCase(key) => self.pending_for_case(key, session),
            _ => false,
        };
        if related_pending {
            return Err(EquipmentEditError::PendingDrafts);
        }
        self.submit_input(session, units, target.clone(), command)?;
        Ok((Some(target), None))
    }
}

#[cfg(test)]
mod tests;
