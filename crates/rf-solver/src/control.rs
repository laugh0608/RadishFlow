use std::ops::ControlFlow;

use rf_types::UnitId;

use crate::SolveSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolveStage {
    Validating,
    Planned,
    UnitStarted,
    UnitCompleted,
    Completing,
}

/// Actual sequential execution boundaries, not an estimate of numerical work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveProgress {
    pub stage: SolveStage,
    pub total_units: Option<usize>,
    pub completed_units: usize,
    pub current_unit: Option<UnitId>,
}

/// Cancellation is observed only at complete, declared boundaries. A unit or
/// flash call is never interrupted; a numerical error remains the original Err.
pub trait SolverControl {
    fn checkpoint(&mut self, progress: SolveProgress) -> ControlFlow<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlledSolveOutcome {
    Completed(SolveSnapshot),
    Cancelled,
}

pub(crate) struct UncontrolledSolve;

impl SolverControl for UncontrolledSolve {
    fn checkpoint(&mut self, _: SolveProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
