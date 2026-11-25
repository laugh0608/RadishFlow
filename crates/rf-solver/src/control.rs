use std::ops::ControlFlow;

use rf_types::{RfError, UnitId};

use crate::{SolveSnapshot, UnitSolveStep};

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

    /// Observe actual execution without introducing a cancellation boundary.
    fn observe_sequential(&mut self, _event: SequentialExecutionEvent<'_>) {}
}

/// Borrowed evidence from the acyclic solver. Step numbers are one-based.
#[derive(Debug)]
pub enum SequentialExecutionEvent<'a> {
    Validating,
    Planned(&'a [UnitId]),
    StepStarted {
        number: usize,
        unit: &'a UnitId,
    },
    StepCompleted {
        number: usize,
        step: &'a UnitSolveStep,
    },
    StepFailed {
        number: usize,
        unit: &'a UnitId,
        error: &'a RfError,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlledSolveOutcome {
    Completed(SolveSnapshot),
    Cancelled,
}

/// Synchronous execution without progress observation or cancellation.
pub struct UncontrolledSolve;

impl SolverControl for UncontrolledSolve {
    fn checkpoint(&mut self, _: SolveProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
