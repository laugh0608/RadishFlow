//! Execute an explicit steady-state plan using already assembled services.
//! Callers own input capture, preparation, scheduling and result acceptance.
pub use crate::control::UncontrolledSolve;
use crate::{
    ControlledSolveOutcome, SequentialModularSolver, SolverControl, SolverServices,
    equation_oriented::{
        EquationControl, EquationFailure, EquationFailureKind, EquationOrientedSolver,
        EquationProgress, EquationResult,
    },
    flow_plan::FlowPlanConfiguration,
    mixed::{
        MixedControl, MixedFailure, MixedFailureKind, MixedProgress, MixedResult, MixedSolver,
    },
    partitioned::{
        PartitionedFailureKind, PartitionedProgress, PartitionedSequentialSolver,
        PartitionedSolveFailure, PartitionedSolveResult, PartitionedSolverControl,
    },
    simultaneous::{
        ModularControl, ModularFailure, ModularFailureKind, ModularProgress, ModularResult,
        SimultaneousModularSolver,
    },
};
use rf_model::Flowsheet;
use rf_types::{RfError, RfResult};
use std::ops::ControlFlow;

pub trait FlowExecutionControl:
    SolverControl + PartitionedSolverControl + ModularControl + EquationControl + MixedControl
{
}

impl<T> FlowExecutionControl for T where
    T: SolverControl + PartitionedSolverControl + ModularControl + EquationControl + MixedControl
{
}

impl PartitionedSolverControl for UncontrolledSolve {
    fn checkpoint(&mut self, _: PartitionedProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl ModularControl for UncontrolledSolve {
    fn checkpoint(&mut self, _: ModularProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl EquationControl for UncontrolledSolve {
    fn checkpoint(&mut self, _: EquationProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl MixedControl for UncontrolledSolve {
    fn checkpoint(&mut self, _: MixedProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowExecutionStatus {
    Succeeded,
    Failed,
    Cancelled,
}

/// The selected solver, its terminal result, and any method-specific evidence.
/// A method can return diagnostic evidence even when it produces no valid result.
#[derive(Debug)]
pub enum FlowExecutionOutcome {
    Acyclic(RfResult<ControlledSolveOutcome>),
    Partitioned(Result<Box<PartitionedSolveResult>, Box<PartitionedSolveFailure>>),
    Simultaneous(Result<Box<ModularResult>, Box<ModularFailure>>),
    EquationOriented(Result<Box<EquationResult>, Box<EquationFailure>>),
    Mixed(Result<Box<MixedResult>, Box<MixedFailure>>),
}

impl FlowExecutionOutcome {
    pub fn status(&self) -> FlowExecutionStatus {
        use FlowExecutionStatus::{Cancelled, Failed, Succeeded};
        match self {
            Self::Acyclic(Ok(ControlledSolveOutcome::Cancelled)) => Cancelled,
            Self::Partitioned(Err(failure))
                if failure.kind == PartitionedFailureKind::Cancelled =>
            {
                Cancelled
            }
            Self::Simultaneous(Err(failure)) if failure.kind == ModularFailureKind::Cancelled => {
                Cancelled
            }
            Self::EquationOriented(Err(failure))
                if failure.kind == EquationFailureKind::Cancelled =>
            {
                Cancelled
            }
            Self::Mixed(Err(failure)) if failure.kind == MixedFailureKind::Cancelled => Cancelled,
            Self::Acyclic(Ok(_))
            | Self::Partitioned(Ok(_))
            | Self::Simultaneous(Ok(_))
            | Self::EquationOriented(Ok(_))
            | Self::Mixed(Ok(_)) => Succeeded,
            Self::Acyclic(Err(_))
            | Self::Partitioned(Err(_))
            | Self::Simultaneous(Err(_))
            | Self::EquationOriented(Err(_))
            | Self::Mixed(Err(_)) => Failed,
        }
    }

    /// The solver diagnostic, including cancellation evidence where supplied.
    pub fn error(&self) -> Option<&RfError> {
        match self {
            Self::Acyclic(result) => result.as_ref().err(),
            Self::Partitioned(result) => result.as_ref().err().map(|failure| &failure.error),
            Self::Simultaneous(result) => result.as_ref().err().map(|failure| &failure.error),
            Self::EquationOriented(result) => result.as_ref().err().map(|failure| &failure.error),
            Self::Mixed(result) => result.as_ref().err().map(|failure| &failure.error),
        }
    }

    pub fn matches_plan(&self, plan: &FlowPlanConfiguration) -> bool {
        matches!(
            (self, plan),
            (Self::Acyclic(_), FlowPlanConfiguration::Acyclic)
                | (Self::Partitioned(_), FlowPlanConfiguration::Partitioned(_))
                | (
                    Self::Simultaneous(_),
                    FlowPlanConfiguration::Simultaneous(_)
                )
                | (
                    Self::EquationOriented(_),
                    FlowPlanConfiguration::EquationOriented(_)
                )
                | (Self::Mixed(_), FlowPlanConfiguration::Mixed(_))
        )
    }
}

/// Dispatch the selected method without attempting any other plan.
/// Each solver assembles and validates its own inputs at its declared checkpoints.
pub fn execute_flow(
    flowsheet: &Flowsheet,
    plan: &FlowPlanConfiguration,
    services: &SolverServices<'_>,
    control: &mut impl FlowExecutionControl,
) -> FlowExecutionOutcome {
    match plan {
        FlowPlanConfiguration::Acyclic => FlowExecutionOutcome::Acyclic(
            SequentialModularSolver.solve_controlled(services, flowsheet, control),
        ),
        FlowPlanConfiguration::Partitioned(options) => FlowExecutionOutcome::Partitioned(
            PartitionedSequentialSolver
                .solve_controlled(services, flowsheet, options, control)
                .map(Box::new),
        ),
        FlowPlanConfiguration::Simultaneous(options) => FlowExecutionOutcome::Simultaneous(
            SimultaneousModularSolver
                .solve_controlled(services, flowsheet, options, control)
                .map(Box::new),
        ),
        FlowPlanConfiguration::EquationOriented(options) => FlowExecutionOutcome::EquationOriented(
            EquationOrientedSolver
                .solve_controlled(services, flowsheet, options, control)
                .map(Box::new),
        ),
        FlowPlanConfiguration::Mixed(options) => FlowExecutionOutcome::Mixed(
            MixedSolver
                .solve_controlled(services, flowsheet, options, control)
                .map(Box::new),
        ),
    }
}
