use super::*;
use rf_model::equipment::EquipmentMethod;
use rf_unitops::{
    separator_rating::{self, RatingFailure, SeparatorRatingResult},
    separator_sizing::{self, SeparatorCandidate, SizingFailure},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResultGeneration {
    pub input: u64,
    pub evidence: u64,
}
/// Session-local execution identity; never persisted or reused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculationId(EngineeringCheckpoint);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResultStaleness {
    pub input_changed: bool,
    pub evidence_changed: bool,
    pub task_missing: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct CalculationKey {
    document: DocumentId,
    session: EngineeringCheckpoint,
    member: EquipmentCaseKey,
    method: EquipmentMethod,
    generation: ResultGeneration,
    eligibility: EngineeringCheckpoint,
}
#[derive(Debug, Clone, PartialEq)]
enum CalculationOutcome {
    Sizing(Result<SeparatorCandidate, SizingFailure>),
    Rating(Result<SeparatorRatingResult, RatingFailure>),
}
impl CalculationOutcome {
    fn is_ok(&self) -> bool {
        match self {
            Self::Sizing(r) => r.is_ok(),
            Self::Rating(r) => r.is_ok(),
        }
    }
}
/// Frozen inputs with an unforgeable qualification key. Execution has no IO or
/// session mutation. Completion must return through the owning session.
#[derive(Debug)]
pub struct CalculationRequest {
    key: CalculationKey,
    request: CalculationId,
    task: EquipmentTask,
    basis: EngineeringBasis,
}
impl CalculationRequest {
    pub fn task(&self) -> &EquipmentTask {
        &self.task
    }
    pub fn basis(&self) -> &EngineeringBasis {
        &self.basis
    }
    pub fn execute(self) -> CompletedCalculation {
        let outcome = match &self.task {
            EquipmentTask::SeparatorSizing(t) => {
                CalculationOutcome::Sizing(separator_sizing::calculate(t, &self.basis))
            }
            EquipmentTask::SeparatorRating(t) => {
                CalculationOutcome::Rating(separator_rating::calculate(t, &self.basis))
            }
        };
        CompletedCalculation {
            key: self.key,
            request: self.request,
            outcome,
        }
    }
}
#[derive(Debug)]
pub struct CompletedCalculation {
    key: CalculationKey,
    request: CalculationId,
    outcome: CalculationOutcome,
}
#[derive(Debug, Clone, PartialEq)]
pub(super) struct CalculationRecord {
    id: CalculationId,
    key: CalculationKey,
    outcome: CalculationOutcome,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentResultStatus {
    Missing,
    Current,
    Failed,
    Stale,
}

impl EngineeringSession {
    // Both currently supported application methods have exactly one case.
    // Explicit case APIs below keep result identity independent of a UI selection.
    fn member(&self, task: &EquipmentTaskId) -> Option<EquipmentCaseKey> {
        let t = self.current.equipment_task(task)?;
        let mut cases = t.case_ids();
        let case_id = cases.next()?.clone();
        if cases.next().is_some() {
            return None;
        }
        Some(EquipmentCaseKey {
            task_id: task.clone(),
            case_id,
        })
    }
    fn historical_member(&self, task: &EquipmentTaskId) -> Option<EquipmentCaseKey> {
        self.member(task)
            .or_else(|| self.results.keys().find(|k| k.task_id == *task).cloned())
    }
    pub fn generation(&self, task: &EquipmentTaskId) -> Option<ResultGeneration> {
        self.case_generation(&self.member(task)?)
    }
    pub fn case_generation(&self, member: &EquipmentCaseKey) -> Option<ResultGeneration> {
        self.calculation_key(member).map(|k| k.generation)
    }
    fn calculation_key(&self, member: &EquipmentCaseKey) -> Option<CalculationKey> {
        let record = self.current.equipment_task(&member.task_id)?;
        if !record.case_ids().any(|id| *id == member.case_id) {
            return None;
        }
        Some(CalculationKey {
            document: self.document_id.clone(),
            session: self.identity.clone(),
            member: member.clone(),
            method: record.method().clone(),
            generation: *self.generations.get(member)?,
            eligibility: self.eligibility.get(member)?.clone(),
        })
    }
    pub fn begin_calculation(
        &mut self,
        task: &EquipmentTaskId,
    ) -> Result<CalculationRequest, EngineeringError> {
        let member = self.member(task).ok_or_else(|| {
            EngineeringError::MissingRecord(EngineeringRecordId::Task(task.clone()))
        })?;
        self.begin_case_calculation(&member)
    }
    pub fn begin_rating(
        &mut self,
        member: &EquipmentCaseKey,
    ) -> Result<CalculationRequest, EngineeringError> {
        self.current.rating_task(&member.task_id).ok_or_else(|| {
            EngineeringError::MissingRecord(EngineeringRecordId::Task(member.task_id.clone()))
        })?;
        self.begin_case_calculation(member)
    }
    fn begin_case_calculation(
        &mut self,
        member: &EquipmentCaseKey,
    ) -> Result<CalculationRequest, EngineeringError> {
        self.calculation_key(member).ok_or_else(|| {
            EngineeringError::MissingRecord(EngineeringRecordId::Case(member.case_id.clone()))
        })?;
        // Rating reruns immediately revoke the previous row result. Preserve the
        // existing sizing lifecycle, whose current candidate is replaced on completion.
        if self.current.rating_task(&member.task_id).is_some() {
            self.eligibility
                .insert(member.clone(), EngineeringCheckpoint::fresh());
        }
        let key = self.calculation_key(member).expect("checked member");
        let request = CalculationId(EngineeringCheckpoint::fresh());
        self.pending.insert(member.clone(), request.clone());
        Ok(CalculationRequest {
            key,
            request,
            task: self
                .current
                .equipment_task(&member.task_id)
                .expect("checked task")
                .clone(),
            basis: self.current.basis.clone(),
        })
    }
    pub fn complete_calculation(
        &mut self,
        completed: CompletedCalculation,
    ) -> Result<EquipmentResultStatus, EngineeringError> {
        if self.calculation_key(&completed.key.member).as_ref() != Some(&completed.key)
            || self.pending.get(&completed.key.member) != Some(&completed.request)
        {
            return Err(EngineeringError::StaleCalculation);
        }
        let member = completed.key.member.clone();
        self.pending.remove(&member);
        self.results.insert(
            member.clone(),
            CalculationRecord {
                id: completed.request,
                key: completed.key,
                outcome: completed.outcome,
            },
        );
        Ok(self.case_result_status(&member))
    }
    pub fn calculate(
        &mut self,
        task: &EquipmentTaskId,
    ) -> Result<EquipmentResultStatus, EngineeringError> {
        let completed = self.begin_calculation(task)?.execute();
        self.complete_calculation(completed)
    }
    pub fn calculate_rating(
        &mut self,
        member: &EquipmentCaseKey,
    ) -> Result<EquipmentResultStatus, EngineeringError> {
        let completed = self.begin_rating(member)?.execute();
        self.complete_calculation(completed)
    }
    pub fn result_status(&self, task: &EquipmentTaskId) -> EquipmentResultStatus {
        self.historical_member(task)
            .map_or(EquipmentResultStatus::Missing, |k| {
                self.case_result_status(&k)
            })
    }
    pub fn case_result_status(&self, member: &EquipmentCaseKey) -> EquipmentResultStatus {
        let Some(record) = self.results.get(member) else {
            return EquipmentResultStatus::Missing;
        };
        if self.calculation_key(member).as_ref() != Some(&record.key) {
            return EquipmentResultStatus::Stale;
        }
        if record.outcome.is_ok() {
            EquipmentResultStatus::Current
        } else {
            EquipmentResultStatus::Failed
        }
    }
    pub fn calculation_id(&self, task: &EquipmentTaskId) -> Option<&CalculationId> {
        self.case_calculation_id(&self.historical_member(task)?)
    }
    pub fn case_calculation_id(&self, member: &EquipmentCaseKey) -> Option<&CalculationId> {
        self.results.get(member).map(|r| &r.id)
    }
    pub fn staleness(&self, task: &EquipmentTaskId) -> Option<ResultStaleness> {
        self.case_staleness(&self.historical_member(task)?)
    }
    pub fn case_staleness(&self, member: &EquipmentCaseKey) -> Option<ResultStaleness> {
        if self.case_result_status(member) != EquipmentResultStatus::Stale {
            return None;
        }
        let record = self.results.get(member)?;
        let generation = self.generations.get(member)?;
        Some(ResultStaleness {
            input_changed: generation.input != record.key.generation.input,
            evidence_changed: generation.evidence != record.key.generation.evidence,
            task_missing: self.current.equipment_task(&member.task_id).is_none(),
        })
    }
    /// Sole access to sizing evidence eligible for current display/export.
    pub fn current_candidate(&self, task: &EquipmentTaskId) -> Option<&SeparatorCandidate> {
        if self.result_status(task) != EquipmentResultStatus::Current {
            return None;
        }
        self.historical_outcome(task)?.as_ref().ok()
    }
    pub fn current_failure(&self, task: &EquipmentTaskId) -> Option<&SizingFailure> {
        if self.result_status(task) != EquipmentResultStatus::Failed {
            return None;
        }
        self.historical_outcome(task)?.as_ref().err()
    }
    pub fn current_rating(&self, member: &EquipmentCaseKey) -> Option<&SeparatorRatingResult> {
        if self.case_result_status(member) != EquipmentResultStatus::Current {
            return None;
        }
        self.historical_rating(member)?.as_ref().ok()
    }
    pub fn current_rating_failure(&self, member: &EquipmentCaseKey) -> Option<&RatingFailure> {
        if self.case_result_status(member) != EquipmentResultStatus::Failed {
            return None;
        }
        self.historical_rating(member)?.as_ref().err()
    }
    /// Historical evidence only; the caller must label it with result_status().
    pub fn historical_outcome(
        &self,
        task: &EquipmentTaskId,
    ) -> Option<&Result<SeparatorCandidate, SizingFailure>> {
        match &self.results.get(&self.historical_member(task)?)?.outcome {
            CalculationOutcome::Sizing(r) => Some(r),
            _ => None,
        }
    }
    /// Historical evidence only; never grants current display/export eligibility.
    pub fn historical_rating(
        &self,
        member: &EquipmentCaseKey,
    ) -> Option<&Result<SeparatorRatingResult, RatingFailure>> {
        match &self.results.get(member)?.outcome {
            CalculationOutcome::Rating(r) => Some(r),
            _ => None,
        }
    }
    pub fn clear_results(&mut self) {
        self.results.clear();
        self.pending.clear();
    }
}
