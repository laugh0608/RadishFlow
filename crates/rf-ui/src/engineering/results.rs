use super::*;
use rf_model::equipment::EquipmentMethod;
use rf_unitops::separator_sizing::{self, SeparatorCandidate, SizingFailure};

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
    task: EquipmentTaskId,
    case: EngineeringCaseId,
    method: EquipmentMethod,
    generation: ResultGeneration,
    eligibility: EngineeringCheckpoint,
}

/// Frozen inputs with an unforgeable qualification key. Execution has no IO or
/// session mutation. Completion must return through the owning session.
#[derive(Debug)]
pub struct CalculationRequest {
    key: CalculationKey,
    request: CalculationId,
    task: SeparatorSizingTask,
    basis: EngineeringBasis,
}
impl CalculationRequest {
    pub fn task(&self) -> &SeparatorSizingTask {
        &self.task
    }
    pub fn basis(&self) -> &EngineeringBasis {
        &self.basis
    }
    pub fn execute(self) -> CompletedCalculation {
        let outcome = separator_sizing::calculate(&self.task, &self.basis);
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
    outcome: Result<SeparatorCandidate, SizingFailure>,
}
#[derive(Debug, Clone, PartialEq)]
pub(super) struct CalculationRecord {
    id: CalculationId,
    key: CalculationKey,
    outcome: Result<SeparatorCandidate, SizingFailure>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentResultStatus {
    Missing,
    Current,
    Failed,
    Stale,
}

impl EngineeringSession {
    pub fn generation(&self, task: &EquipmentTaskId) -> Option<ResultGeneration> {
        self.current.task(task)?;
        self.generations.get(task).copied()
    }
    fn calculation_key(&self, task: &EquipmentTaskId) -> Option<CalculationKey> {
        let record = self.current.task(task)?;
        Some(CalculationKey {
            document: self.document_id.clone(),
            session: self.identity.clone(),
            task: task.clone(),
            case: record.case.id.clone(),
            method: record.method.clone(),
            generation: *self.generations.get(task)?,
            eligibility: self.eligibility.get(task)?.clone(),
        })
    }
    pub fn begin_calculation(
        &mut self,
        task: &EquipmentTaskId,
    ) -> Result<CalculationRequest, EngineeringError> {
        let key = self.calculation_key(task).ok_or_else(|| {
            EngineeringError::MissingRecord(EngineeringRecordId::Task(task.clone()))
        })?;
        let request = CalculationId(EngineeringCheckpoint::fresh());
        self.pending.insert(task.clone(), request.clone());
        Ok(CalculationRequest {
            key,
            request,
            task: self.current.task(task).expect("key checked task").clone(),
            basis: self.current.basis.clone(),
        })
    }
    pub fn complete_calculation(
        &mut self,
        completed: CompletedCalculation,
    ) -> Result<EquipmentResultStatus, EngineeringError> {
        if self.calculation_key(&completed.key.task).as_ref() != Some(&completed.key)
            || self.pending.get(&completed.key.task) != Some(&completed.request)
        {
            return Err(EngineeringError::StaleCalculation);
        }
        let id = completed.key.task.clone();
        self.pending.remove(&id);
        self.results.insert(
            id.clone(),
            CalculationRecord {
                id: completed.request,
                key: completed.key,
                outcome: completed.outcome,
            },
        );
        Ok(self.result_status(&id))
    }
    pub fn calculate(
        &mut self,
        task: &EquipmentTaskId,
    ) -> Result<EquipmentResultStatus, EngineeringError> {
        let completed = self.begin_calculation(task)?.execute();
        self.complete_calculation(completed)
    }
    pub fn result_status(&self, task: &EquipmentTaskId) -> EquipmentResultStatus {
        let Some(record) = self.results.get(task) else {
            return EquipmentResultStatus::Missing;
        };
        if self.calculation_key(task).as_ref() != Some(&record.key) {
            return EquipmentResultStatus::Stale;
        }
        if record.outcome.is_ok() {
            EquipmentResultStatus::Current
        } else {
            EquipmentResultStatus::Failed
        }
    }
    pub fn calculation_id(&self, task: &EquipmentTaskId) -> Option<&CalculationId> {
        self.results.get(task).map(|record| &record.id)
    }
    pub fn staleness(&self, task: &EquipmentTaskId) -> Option<ResultStaleness> {
        if self.result_status(task) != EquipmentResultStatus::Stale {
            return None;
        }
        let record = self.results.get(task)?;
        let generation = self.generations.get(task)?;
        Some(ResultStaleness {
            input_changed: generation.input != record.key.generation.input,
            evidence_changed: generation.evidence != record.key.generation.evidence,
            task_missing: self.current.task(task).is_none(),
        })
    }
    /// Sole access to a candidate eligible for current display/export. Labels may
    /// have changed since capture; current labels come from inputs().task(id).
    pub fn current_candidate(&self, task: &EquipmentTaskId) -> Option<&SeparatorCandidate> {
        if self.result_status(task) != EquipmentResultStatus::Current {
            return None;
        }
        self.results.get(task)?.outcome.as_ref().ok()
    }
    pub fn current_failure(&self, task: &EquipmentTaskId) -> Option<&SizingFailure> {
        if self.result_status(task) != EquipmentResultStatus::Failed {
            return None;
        }
        self.results.get(task)?.outcome.as_ref().err()
    }
    /// Historical evidence only; the caller must label it with result_status().
    pub fn historical_outcome(
        &self,
        task: &EquipmentTaskId,
    ) -> Option<&Result<SeparatorCandidate, SizingFailure>> {
        self.results.get(task).map(|r| &r.outcome)
    }
    /// Cache clearing is not an input edit. Pending completion is revoked too.
    pub fn clear_results(&mut self) {
        self.results.clear();
        self.pending.clear();
    }
}
