//! Execute an explicit steady-state plan using already assembled services.
//! Callers own input capture, preparation, scheduling and result acceptance.
pub use crate::control::UncontrolledSolve;
use crate::{
    ControlledSolveOutcome, SequentialModularSolver, SolverControl, SolverServices,
    equation_oriented::{
        EquationControl, EquationFailure, EquationFailureKind, EquationOrientedSolver,
        EquationProgress, EquationResult,
    },
    flow_plan::{FlowPlanConfiguration, SinglePhaseOptions, SinglePhaseRecycleConfiguration},
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
    single_phase_recycle::{
        SinglePhaseRecycleFailure, SinglePhaseRecycleResult, SinglePhaseRecycleSolver,
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
    SinglePhase(RfResult<ControlledSolveOutcome>),
    SinglePhaseRecycle(Result<Box<SinglePhaseRecycleResult>, Box<SinglePhaseRecycleFailure>>),
    Partitioned(Result<Box<PartitionedSolveResult>, Box<PartitionedSolveFailure>>),
    Simultaneous(Result<Box<ModularResult>, Box<ModularFailure>>),
    EquationOriented(Result<Box<EquationResult>, Box<EquationFailure>>),
    Mixed(Result<Box<MixedResult>, Box<MixedFailure>>),
}

impl FlowExecutionOutcome {
    pub fn status(&self) -> FlowExecutionStatus {
        use FlowExecutionStatus::{Cancelled, Failed, Succeeded};
        match self {
            Self::Acyclic(Ok(ControlledSolveOutcome::Cancelled))
            | Self::SinglePhase(Ok(ControlledSolveOutcome::Cancelled)) => Cancelled,
            Self::SinglePhaseRecycle(Err(failure)) if failure.is_cancelled() => Cancelled,
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
            | Self::SinglePhase(Ok(_))
            | Self::SinglePhaseRecycle(Ok(_))
            | Self::Partitioned(Ok(_))
            | Self::Simultaneous(Ok(_))
            | Self::EquationOriented(Ok(_))
            | Self::Mixed(Ok(_)) => Succeeded,
            Self::Acyclic(Err(_))
            | Self::SinglePhase(Err(_))
            | Self::SinglePhaseRecycle(Err(_))
            | Self::Partitioned(Err(_))
            | Self::Simultaneous(Err(_))
            | Self::EquationOriented(Err(_))
            | Self::Mixed(Err(_)) => Failed,
        }
    }

    /// The solver diagnostic, including cancellation evidence where supplied.
    pub fn error(&self) -> Option<&RfError> {
        match self {
            Self::Acyclic(result) | Self::SinglePhase(result) => result.as_ref().err(),
            Self::SinglePhaseRecycle(result) => result.as_ref().err().map(|failure| &failure.error),
            Self::Partitioned(result) => result.as_ref().err().map(|failure| &failure.error),
            Self::Simultaneous(result) => result.as_ref().err().map(|failure| &failure.error),
            Self::EquationOriented(result) => result.as_ref().err().map(|failure| &failure.error),
            Self::Mixed(result) => result.as_ref().err().map(|failure| &failure.error),
        }
    }

    pub fn matches_plan(&self, plan: &FlowPlanConfiguration) -> bool {
        if let (Self::EquationOriented(outcome), FlowPlanConfiguration::EquationOriented(options)) =
            (self, plan)
        {
            return match outcome {
                Ok(result) => {
                    (
                        result.equation_contract_version,
                        result.acceptance_contract_version,
                    ) == options.contract_versions()
                        && result
                            .evidence
                            .definition
                            .as_ref()
                            .is_some_and(|d| &d.options == options)
                }
                Err(failure) => failure
                    .evidence
                    .definition
                    .as_ref()
                    .is_none_or(|d| &d.options == options),
            };
        }
        if let (Self::Mixed(outcome), FlowPlanConfiguration::Mixed(options)) = (self, plan) {
            let method = match outcome {
                Ok(result) => result.method,
                Err(failure) => failure.method,
            };
            if method != options.method() {
                return false;
            }
            if let (Some(regions), Ok(result)) = (options.serial_recycle(), outcome) {
                return result.serial_recycles.len() == regions.len()
                    && regions.iter().all(|region| {
                        result.serial_recycles.iter().any(|report| {
                            report.recycle.tear == region.tear
                                && report.recycle.max_iteration_evaluations
                                    == region.max_iteration_evaluations
                                && report.balance.is_some()
                        })
                    });
            }
            return true;
        }
        if let (
            Self::SinglePhaseRecycle(outcome),
            FlowPlanConfiguration::SinglePhaseRecycle(options),
        ) = (self, plan)
        {
            return match outcome {
                Ok(result) => result.tear == options.recycle.tear,
                Err(_) => true,
            };
        }
        matches!(
            (self, plan),
            (Self::Acyclic(_), FlowPlanConfiguration::Acyclic)
                | (Self::SinglePhase(_), FlowPlanConfiguration::SinglePhase(_))
                | (Self::Partitioned(_), FlowPlanConfiguration::Partitioned(_))
                | (
                    Self::Simultaneous(_),
                    FlowPlanConfiguration::Simultaneous(_)
                )
        )
    }
}

pub fn execute_single_phase_recycle(
    flowsheet: &Flowsheet,
    options: &SinglePhaseRecycleConfiguration,
    control: &mut impl PartitionedSolverControl,
) -> FlowExecutionOutcome {
    FlowExecutionOutcome::SinglePhaseRecycle(
        SinglePhaseRecycleSolver
            .solve_controlled(flowsheet, &options.properties, &options.recycle, control)
            .map(Box::new),
    )
}

pub fn execute_single_phase(
    flowsheet: &Flowsheet,
    options: &SinglePhaseOptions,
    control: &mut impl SolverControl,
) -> FlowExecutionOutcome {
    FlowExecutionOutcome::SinglePhase(
        SequentialModularSolver.solve_single_phase_with_heat_duties_controlled(
            &options.model,
            flowsheet,
            &options.heat_duties_w,
            control,
        ),
    )
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
        FlowPlanConfiguration::SinglePhase(options) => {
            execute_single_phase(flowsheet, options, control)
        }
        FlowPlanConfiguration::SinglePhaseRecycle(options) => {
            execute_single_phase_recycle(flowsheet, options, control)
        }
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
