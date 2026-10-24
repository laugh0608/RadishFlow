//! Application-owned device editor. Formal inputs, text drafts, checks and history
//! remain distinct; all shell actions use the same command boundary.
mod adoption;
mod drafts;
pub use adoption::{
    DimensionAdjustmentDraft, GeometryAdoptionDestination, GeometryAdoptionDraft,
    GeometryAdoptionPreview, PreparedEditorGeometryAdoption,
};
mod membership;
mod rating_drafts;
mod rating_editor;
mod transaction;
use super::*;
pub use drafts::*;
pub use rating_drafts::*;
use rf_model::{
    engineering::{BasisEntry, SourceRecord},
    equipment::SeparatorSizingTask,
};
use rf_types::units::{ConversionError, DisplayUnitSet};
pub use rf_unitops::{separator_rating, separator_sizing};
use separator_sizing::SizingIssue;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EquipmentEditorTarget {
    Task(EquipmentTaskId),
    RatingTask(EquipmentTaskId),
    RatingGeometry(EquipmentTaskId),
    RatingCase(EquipmentCaseKey),
    Source(SourceId),
    Basis(BasisEntryId),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentDraft {
    Task(SeparatorTaskDraft),
    RatingTask(RatingLabelsDraft),
    RatingGeometry(RatingGeometryDraft),
    RatingCase(RatingCaseDraft),
    Source(SourceDraft),
    Basis(BasisDraft),
}
#[derive(Debug, Clone, PartialEq)]
enum RecordBaseline {
    Task(SeparatorSizingTask),
    RatingTask(SeparatorRatingTask),
    RatingGeometry(rf_model::equipment::SeparatorGivenGeometry),
    RatingCase(rf_model::equipment::SeparatorRatingCase),
    Source(SourceRecord),
    Basis(Option<BasisEntry>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct EquipmentDraftRecord {
    baseline: RecordBaseline,
    original: EquipmentDraft,
    value: EquipmentDraft,
    generation: u64,
}
impl EquipmentDraftRecord {
    pub fn value(&self) -> &EquipmentDraft {
        &self.value
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn is_pending(&self) -> bool {
        self.value != self.original || matches!(self.baseline, RecordBaseline::Basis(None))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentEditError {
    Domain(EngineeringError),
    Number(crate::NumericParseError),
    Conversion(ConversionError),
    IncompleteNumber,
    SelectSource,
    SelectBasis,
    OverrideReason,
    AccessDate,
    ObservationValue,
    Field(EquipmentField, Box<EquipmentEditError>),
    RatingGeometryField(RatingGeometryField, Box<EquipmentEditError>),
    RatingInput(
        rf_model::equipment::SeparatorRatingInput,
        Box<EquipmentEditError>,
    ),
    UnsupportedEditorAction,
    MissingEditor,
    StaleEditor,
    DraftTypeMismatch,
    PendingDrafts,
    MissingTask,
    CounterExhausted,
}
impl std::fmt::Display for EquipmentEditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "equipment edit rejected: {self:?}")
    }
}
impl std::error::Error for EquipmentEditError {}
impl From<EngineeringError> for EquipmentEditError {
    fn from(e: EngineeringError) -> Self {
        Self::Domain(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentEditorCommand {
    PreviewGeometryAdoption(Box<GeometryAdoptionDraft>),
    ConfirmGeometryAdoption(EngineeringCheckpoint),
    CancelGeometryAdoption,
    CreateTask,
    CreateRatingTask,
    AddRatingCase {
        task: EquipmentTaskId,
        copy_from: Option<EngineeringCaseId>,
    },
    ReorderRatingCases {
        task: EquipmentTaskId,
        cases: Vec<EngineeringCaseId>,
    },
    CreateSource,
    NewBasis,
    Select(EquipmentEditorTarget),
    Change {
        target: EquipmentEditorTarget,
        generation: u64,
        value: Box<EquipmentDraft>,
    },
    Apply(EquipmentEditorTarget),
    Discard(EquipmentEditorTarget),
    Delete(EquipmentEditorTarget),
    History {
        redo: bool,
    },
    Confirm(EngineeringCheckpoint),
    CancelConfirmation,
    Check(EquipmentTaskId),
    Calculate(EquipmentTaskId),
    CheckRating(EquipmentCaseKey),
    CalculateRating(EquipmentCaseKey),
}
#[derive(Debug, Clone, PartialEq)]
enum PendingOperation {
    AddCase {
        prepared: PreparedEngineeringEdit,
        target: EquipmentEditorTarget,
    },
    Apply {
        prepared: PreparedEngineeringEdit,
        target: EquipmentEditorTarget,
    },
    Delete {
        prepared: PreparedEngineeringEdit,
        target: EquipmentEditorTarget,
    },
    History {
        redo: bool,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct EquipmentConfirmation {
    key: EngineeringCheckpoint,
    editor_checkpoint: EngineeringCheckpoint,
    formal_checkpoint: EngineeringCheckpoint,
    operation: PendingOperation,
    impacts: Vec<TaskImpact>,
    discarded_drafts: Vec<EquipmentEditorTarget>,
}
impl EquipmentConfirmation {
    pub fn key(&self) -> EngineeringCheckpoint {
        self.key.clone()
    }
    pub fn impacts(&self) -> &[TaskImpact] {
        &self.impacts
    }
    pub fn discarded_drafts(&self) -> &[EquipmentEditorTarget] {
        &self.discarded_drafts
    }
    pub fn is_delete(&self) -> bool {
        matches!(self.operation, PendingOperation::Delete { .. })
    }
    pub fn upgrades_rating_method(&self) -> bool {
        matches!(self.operation, PendingOperation::AddCase { .. })
    }
    pub fn deleted_rating_case(&self) -> Option<&EquipmentCaseKey> {
        match &self.operation {
            PendingOperation::Delete {
                target: EquipmentEditorTarget::RatingCase(key),
                ..
            } => Some(key),
            _ => None,
        }
    }
    pub fn is_history(&self) -> bool {
        matches!(self.operation, PendingOperation::History { .. })
    }
}
#[derive(Debug, Clone, PartialEq)]
struct InputCheck {
    formal: EngineeringCheckpoint,
    issues: Vec<SizingIssue>,
}
#[derive(Debug, Clone, PartialEq)]
struct RatingInputCheck {
    session: EngineeringCheckpoint,
    formal: ResultGeneration,
    issues: Vec<rf_unitops::separator_rating::RatingIssue>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EquipmentEditorState {
    active: Option<EquipmentEditorTarget>,
    drafts: BTreeMap<EquipmentEditorTarget, EquipmentDraftRecord>,
    checkpoint: EngineeringCheckpoint,
    next_generation: u64,
    next_identity: u64,
    pending: Option<EquipmentConfirmation>,
    adoption_preview: Option<GeometryAdoptionPreview>,
    checks: BTreeMap<EquipmentTaskId, InputCheck>,
    rating_checks: BTreeMap<EquipmentCaseKey, RatingInputCheck>,
    error: Option<EquipmentEditError>,
    target_errors: BTreeMap<EquipmentEditorTarget, EquipmentEditError>,
}
impl Default for EquipmentEditorState {
    fn default() -> Self {
        Self {
            active: None,
            drafts: BTreeMap::new(),
            checkpoint: EngineeringCheckpoint::fresh(),
            next_generation: 0,
            next_identity: 0,
            pending: None,
            adoption_preview: None,
            checks: BTreeMap::new(),
            rating_checks: BTreeMap::new(),
            error: None,
            target_errors: BTreeMap::new(),
        }
    }
}
impl EquipmentEditorState {
    /// Apply the same draft gate as inline calculation, then freeze formal inputs.
    /// The caller must reserve its worker slot before invoking this method.
    pub fn prepare_calculation(
        &mut self,
        session: &mut EngineeringSession,
        command: &EquipmentEditorCommand,
    ) -> Result<CalculationRequest, EquipmentEditError> {
        let result = match command {
            EquipmentEditorCommand::Calculate(task) => {
                if self.pending_for_task(task, session) {
                    Err(EquipmentEditError::PendingDrafts)
                } else {
                    session.begin_calculation(task).map_err(Into::into)
                }
            }
            EquipmentEditorCommand::CalculateRating(member) => {
                if self.pending_for_case(member, session) {
                    Err(EquipmentEditError::PendingDrafts)
                } else {
                    session.begin_rating(member).map_err(Into::into)
                }
            }
            _ => Err(EquipmentEditError::UnsupportedEditorAction),
        };
        self.error = result.as_ref().err().cloned();
        if result.is_ok() {
            match command {
                EquipmentEditorCommand::Calculate(task) => {
                    self.checks.remove(task);
                }
                EquipmentEditorCommand::CalculateRating(member) => {
                    self.rating_checks.remove(member);
                }
                _ => unreachable!(),
            }
        }
        result
    }
    pub fn active(&self) -> Option<&EquipmentEditorTarget> {
        self.active.as_ref()
    }
    pub fn draft(&self, target: &EquipmentEditorTarget) -> Option<&EquipmentDraftRecord> {
        self.drafts.get(target)
    }
    pub fn checkpoint(&self) -> EngineeringCheckpoint {
        self.checkpoint.clone()
    }
    pub fn confirmation(&self) -> Option<&EquipmentConfirmation> {
        self.pending.as_ref()
    }
    pub fn error(&self) -> Option<&EquipmentEditError> {
        self.error
            .as_ref()
            .or_else(|| self.active.as_ref().and_then(|t| self.target_errors.get(t)))
    }
    pub fn pending_targets(&self) -> Vec<EquipmentEditorTarget> {
        self.drafts
            .iter()
            .filter(|(_, d)| d.is_pending())
            .map(|(t, _)| t.clone())
            .collect()
    }
    pub fn pending_count(&self) -> usize {
        self.drafts.values().filter(|d| d.is_pending()).count()
    }
    pub fn pending_for_task(&self, task: &EquipmentTaskId, session: &EngineeringSession) -> bool {
        self.drafts.iter().any(|(target, draft)| {
            if !draft.is_pending() {
                return false;
            }
            let record = match target {
                EquipmentEditorTarget::Task(id)
                | EquipmentEditorTarget::RatingTask(id)
                | EquipmentEditorTarget::RatingGeometry(id) => return id == task,
                EquipmentEditorTarget::RatingCase(key) => return key.task_id == *task,
                EquipmentEditorTarget::Source(id) => EngineeringRecordId::Source(id.clone()),
                EquipmentEditorTarget::Basis(id) => EngineeringRecordId::Basis(id.clone()),
            };
            super::impact::task_references(session.inputs(), task, &record)
        })
    }
    pub fn pending_for_case(
        &self,
        member: &EquipmentCaseKey,
        session: &EngineeringSession,
    ) -> bool {
        self.drafts.iter().any(|(target, draft)| {
            if !draft.is_pending() {
                return false;
            }
            let record = match target {
                EquipmentEditorTarget::Task(id) | EquipmentEditorTarget::RatingTask(id) => {
                    EngineeringRecordId::Task(id.clone())
                }
                EquipmentEditorTarget::RatingGeometry(id) => {
                    EngineeringRecordId::Geometry(id.clone())
                }
                EquipmentEditorTarget::RatingCase(key) => {
                    return key == member;
                }
                EquipmentEditorTarget::Source(id) => EngineeringRecordId::Source(id.clone()),
                EquipmentEditorTarget::Basis(id) => EngineeringRecordId::Basis(id.clone()),
            };
            super::impact::case_references(session.inputs(), member, &record)
        })
    }
    pub fn checked_rating_issues(
        &self,
        member: &EquipmentCaseKey,
        session: &EngineeringSession,
    ) -> Option<&[rf_unitops::separator_rating::RatingIssue]> {
        self.rating_checks
            .get(member)
            .filter(|c| {
                c.session == session.identity && Some(c.formal) == session.case_generation(member)
            })
            .map(|c| c.issues.as_slice())
    }
    pub fn checked_issues(
        &self,
        task: &EquipmentTaskId,
        session: &EngineeringSession,
    ) -> Option<&[SizingIssue]> {
        self.checks
            .get(task)
            .filter(|c| c.formal == session.checkpoint())
            .map(|c| c.issues.as_slice())
    }
    /// Dispatch is atomic, including draft disposal after successful history/commit.
    /// Error presentation changes, but rejected operations retain all user input.
    pub fn dispatch(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        command: EquipmentEditorCommand,
    ) -> Result<(), EquipmentEditError> {
        let target = command.error_target().or_else(|| self.active.clone());
        let selecting = matches!(command, EquipmentEditorCommand::Select(_));
        let mut staged = self.clone();
        let mut formal = session.clone();
        match staged.execute(&mut formal, units, command) {
            Ok(()) => {
                staged.error = None;
                if !selecting && let Some(target) = &target {
                    staged.target_errors.remove(target);
                }
                *self = staged;
                *session = formal;
                Ok(())
            }
            Err(error) => {
                self.error = Some(error.clone());
                if let Some(target) = target {
                    self.target_errors.insert(target, error.clone());
                }
                Err(error)
            }
        }
    }
    fn bump(&mut self) -> Result<u64, EquipmentEditError> {
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or(EquipmentEditError::CounterExhausted)?;
        self.checkpoint = EngineeringCheckpoint::fresh();
        Ok(self.next_generation)
    }
}

#[cfg(test)]
mod rating_tests;
#[cfg(test)]
mod tests;
