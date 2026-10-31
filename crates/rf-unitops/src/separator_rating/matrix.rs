//! Pure projection of explicitly supplied, caller-qualified row results. Session
//! freshness belongs to rf-ui; matching inputs alone never grants Current status.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Absolute distance from the maximum unrounded dimensionless load. This is a
/// numerical tie rule, not engineering equivalence or an uncertainty allowance.
pub const CONTROL_TIE_TOLERANCE: f64 = 1e-10;

#[derive(Debug, Clone, PartialEq)]
pub struct ControllingCases {
    pub maximum_load_fraction: f64,
    /// Stable identity order, independent of display or calculation order.
    pub case_ids: Vec<EngineeringCaseId>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixCoverage {
    Complete,
    Incomplete,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorRatingSummary {
    /// All required members in the task's declared order, including absent rows.
    pub required_case_ids: Vec<EngineeringCaseId>,
    pub evaluated_case_ids: Vec<EngineeringCaseId>,
    pub coverage: MatrixCoverage,
    pub gas: Option<ControllingCases>,
    pub liquid: Option<ControllingCases>,
    /// Conclusion of the supplied subset only. None when no results are supplied.
    pub subset_conclusion: Option<ConstraintStatus>,
}
impl SeparatorRatingSummary {
    pub fn all_cases_within_criteria(&self) -> bool {
        self.coverage == MatrixCoverage::Complete
            && self.subset_conclusion == Some(ConstraintStatus::WithinCriteria)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RatingMatrixError {
    InvalidTask(RatingFailure),
    ForeignTask(EngineeringCaseId),
    MethodMismatch(EngineeringCaseId),
    UnknownCase(EngineeringCaseId),
    DuplicateCase(EngineeringCaseId),
    GeometryMismatch(EngineeringCaseId),
    CaseInputMismatch(EngineeringCaseId),
}
impl std::fmt::Display for RatingMatrixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "separator matrix rejected: {self:?}")
    }
}
impl std::error::Error for RatingMatrixError {}

/// Summarize a subset already qualified by its owning application session.
/// Missing/failed/stale rows are omitted from `results`, never from the task's
/// required membership. This function performs no calculations or freshness
/// inference; the caller must also qualify source/basis and execution identities.
pub fn summarize(
    task: &SeparatorRatingTask,
    results: &[&SeparatorRatingResult],
) -> Result<SeparatorRatingSummary, RatingMatrixError> {
    evidence::validate_task(task, true).map_err(RatingMatrixError::InvalidTask)?;
    let mut rows = BTreeMap::new();
    let mut common_geometry: Option<&RatingInputCapture> = None;
    for result in results {
        let capture = result.capture();
        let input = &capture.input;
        let id = &input.case.id;
        if input.task_id != task.id {
            return Err(RatingMatrixError::ForeignTask(id.clone()));
        }
        if input.method != task.method {
            return Err(RatingMatrixError::MethodMismatch(id.clone()));
        }
        let expected = task
            .cases
            .iter()
            .find(|case| case.id == *id)
            .ok_or_else(|| RatingMatrixError::UnknownCase(id.clone()))?;
        if input.geometry != task.geometry
            || common_geometry.is_some_and(|previous| {
                previous.geometry_values_m != capture.geometry_values_m
                    || previous.axial_lengths_m != capture.axial_lengths_m
            })
        {
            return Err(RatingMatrixError::GeometryMismatch(id.clone()));
        }
        // Names, purpose and label-only revisions do not change numerical inputs.
        if input.case.inputs != expected.inputs
            || input.case.fluid_system != expected.fluid_system
            || input.case.applicability != expected.applicability
            || input.case.applicability_source_ids != expected.applicability_source_ids
        {
            return Err(RatingMatrixError::CaseInputMismatch(id.clone()));
        }
        if rows.insert(id.clone(), result.performance()).is_some() {
            return Err(RatingMatrixError::DuplicateCase(id.clone()));
        }
        common_geometry = Some(capture);
    }
    let required_case_ids: Vec<_> = task.cases.iter().map(|c| c.id.clone()).collect();
    let evaluated_case_ids = required_case_ids
        .iter()
        .filter(|id| rows.contains_key(*id))
        .cloned()
        .collect();
    let subset_conclusion = rows.values().map(|p| p.conclusion()).reduce(|a, b| {
        use ConstraintStatus::*;
        match (a, b) {
            (Exceeded, _) | (_, Exceeded) => Exceeded,
            (BoundaryReview, _) | (_, BoundaryReview) => BoundaryReview,
            _ => WithinCriteria,
        }
    });
    Ok(SeparatorRatingSummary {
        coverage: if rows.len() == required_case_ids.len() {
            MatrixCoverage::Complete
        } else {
            MatrixCoverage::Incomplete
        },
        required_case_ids,
        evaluated_case_ids,
        gas: control(rows.iter().map(|(id, p)| (id, p.gas.load_fraction))),
        liquid: control(rows.iter().map(|(id, p)| (id, p.liquid.load_fraction))),
        subset_conclusion,
    })
}
fn control<'a>(
    rows: impl Iterator<Item = (&'a EngineeringCaseId, f64)> + Clone,
) -> Option<ControllingCases> {
    let maximum_load_fraction = rows.clone().map(|(_, load)| load).reduce(f64::max)?;
    let case_ids = rows
        .filter(|(_, load)| (load - maximum_load_fraction).abs() <= CONTROL_TIE_TOLERANCE)
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Some(ControllingCases {
        maximum_load_fraction,
        case_ids,
    })
}
