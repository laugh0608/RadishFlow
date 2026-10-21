//! Immutable matrix evidence and qualification, including absent and failed rows.
use super::*;
use rf_unitops::separator_rating::{
    self, RatingFailure, RatingMatrixError, SeparatorRatingResult, SeparatorRatingSummary,
};

#[derive(Debug, Clone, PartialEq)]
pub struct RatingMatrixRow {
    pub member: EquipmentCaseKey,
    pub status: EquipmentResultStatus,
    pub result: Option<SeparatorRatingResult>,
    pub failure: Option<RatingFailure>,
    pub staleness: Option<ResultStaleness>,
}
#[derive(Debug, Clone, PartialEq)]
struct RowQualification {
    key: results::CalculationKey,
    status: EquipmentResultStatus,
    calculation: Option<CalculationId>,
    pending: Option<CalculationId>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct RatingMatrixCapture {
    task: SeparatorRatingTask,
    rows: Vec<RatingMatrixRow>,
    summary: SeparatorRatingSummary,
    identity: EngineeringCheckpoint,
    membership: EngineeringCheckpoint,
    qualification: BTreeMap<EquipmentCaseKey, RowQualification>,
}
impl RatingMatrixCapture {
    pub fn task(&self) -> &SeparatorRatingTask {
        &self.task
    }
    pub fn rows(&self) -> &[RatingMatrixRow] {
        &self.rows
    }
    pub fn summary(&self) -> &SeparatorRatingSummary {
        &self.summary
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum RatingMatrixCaptureError {
    MissingTask,
    Summary(RatingMatrixError),
}
impl std::fmt::Display for RatingMatrixCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "matrix capture rejected: {self:?}")
    }
}
impl std::error::Error for RatingMatrixCaptureError {}
impl EngineeringSession {
    fn matrix_qualification(
        &self,
        task: &SeparatorRatingTask,
    ) -> BTreeMap<EquipmentCaseKey, RowQualification> {
        task.cases
            .iter()
            .map(|case| {
                let member = EquipmentCaseKey {
                    task_id: task.id.clone(),
                    case_id: case.id.clone(),
                };
                let row = RowQualification {
                    key: self
                        .calculation_key(&member)
                        .expect("validated session member"),
                    status: self.case_result_status(&member),
                    calculation: self.case_calculation_id(&member).cloned(),
                    pending: self.pending.get(&member).cloned(),
                };
                (member, row)
            })
            .collect()
    }
    pub fn capture_rating_matrix(
        &self,
        task: &EquipmentTaskId,
    ) -> Result<RatingMatrixCapture, RatingMatrixCaptureError> {
        let task = self
            .current
            .rating_task(task)
            .ok_or(RatingMatrixCaptureError::MissingTask)?;
        let rows: Vec<_> = task
            .cases
            .iter()
            .map(|case| {
                let member = EquipmentCaseKey {
                    task_id: task.id.clone(),
                    case_id: case.id.clone(),
                };
                RatingMatrixRow {
                    status: self.case_result_status(&member),
                    result: self.current_rating(&member).cloned(),
                    failure: self.current_rating_failure(&member).cloned(),
                    staleness: self.case_staleness(&member),
                    member,
                }
            })
            .collect();
        let summary = separator_rating::summarize(
            task,
            &rows
                .iter()
                .filter_map(|r| r.result.as_ref())
                .collect::<Vec<_>>(),
        )
        .map_err(RatingMatrixCaptureError::Summary)?;
        Ok(RatingMatrixCapture {
            task: task.clone(),
            rows,
            summary,
            identity: self.identity.clone(),
            membership: self.membership[&task.id].clone(),
            qualification: self.matrix_qualification(task),
        })
    }
    /// Names and display order retain captured values. Every membership change,
    /// input/evidence generation and execution transition revokes the capture.
    /// Consumers separately recheck drafts and presentation units before output.
    pub fn rating_matrix_is_current(&self, capture: &RatingMatrixCapture) -> bool {
        self.identity == capture.identity
            && self.membership.get(&capture.task.id) == Some(&capture.membership)
            && self
                .current
                .rating_task(&capture.task.id)
                .is_some_and(|t| self.matrix_qualification(t) == capture.qualification)
    }
}

#[cfg(test)]
mod tests;
