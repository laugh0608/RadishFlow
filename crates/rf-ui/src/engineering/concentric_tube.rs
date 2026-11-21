//! Execution against a saved, immutable H3 case. Editable sessions use their own qualification.

use rf_model::{engineering::EngineeringBasis, equipment::ConcentricTubeRatingTask};
use rf_types::RfError;
use rf_unitops::heat_exchanger::concentric_tube::rate_task;
pub use rf_unitops::heat_exchanger::concentric_tube::{
    ConcentricTubeRating, LimitCheck, SideRating,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatingRevisions {
    pub task: u64,
    pub geometry: u64,
    pub case: u64,
}

impl From<&ConcentricTubeRatingTask> for RatingRevisions {
    fn from(task: &ConcentricTubeRatingTask) -> Self {
        Self {
            task: task.revision,
            geometry: task.geometry.revision,
            case: task.case.revision,
        }
    }
}

#[derive(Debug)]
pub enum SavedRatingError {
    RevisionConflict { actual: RatingRevisions },
    Calculation(RfError),
}

/// Execute only for the exact saved input revisions reviewed by the caller.
pub fn rate_saved_case(
    task: &ConcentricTubeRatingTask,
    basis: &EngineeringBasis,
    expected: RatingRevisions,
) -> Result<ConcentricTubeRating, SavedRatingError> {
    let actual = RatingRevisions::from(task);
    if actual != expected {
        return Err(SavedRatingError::RevisionConflict { actual });
    }
    if task.case.flow_association.is_some() {
        return Err(SavedRatingError::Calculation(
            RfError::invalid_input("resolve the saved association against a Current flow result")
                .with_diagnostic_code("equipment.h3.flow.association_unresolved"),
        ));
    }
    rate_task(task, basis).map_err(SavedRatingError::Calculation)
}
