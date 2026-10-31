//! QS1 fixed-T/P partitioned sequential modular core. No persistence, application
//! task identity, or automatic strategy selection is implied by this API.
mod checks;
mod execution;
mod plan;

use crate::SolverServices;
use rf_flowsheet::MaterialConnection;
use rf_model::{Flowsheet, MaterialStreamState};
use rf_types::{ComponentId, DiagnosticPortTarget, RfError, RfResult, StreamId, UnitId};
use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::time::Duration;

pub use plan::assemble_partitioned_plan;

pub const ABSOLUTE_FLOW_TOLERANCE: f64 = 1e-10;
pub const SCALED_FLOW_TOLERANCE: f64 = 1e-10;
pub const TEMPERATURE_TOLERANCE_K: f64 = 1e-9;
pub const PRESSURE_TOLERANCE_PA: f64 = 1e-6;
pub const MAX_ITERATION_EVALUATIONS: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedRecycleBoundary {
    pub temperature_k: f64,
    pub pressure_pa: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TearSpecification {
    pub stream_id: StreamId,
    pub source: DiagnosticPortTarget,
    pub sink: DiagnosticPortTarget,
    /// None means all zero. A supplied vector must bind every registered component.
    pub initial_component_flows_mol_s: Option<BTreeMap<ComponentId, f64>>,
    /// None freezes each component scale to the evaluated fresh total molar flow.
    pub component_scales_mol_s: Option<BTreeMap<ComponentId, f64>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecycleSolveOptions {
    pub boundary: FixedRecycleBoundary,
    pub tears: Vec<TearSpecification>,
    pub max_iteration_evaluations: usize,
}

impl RecycleSolveOptions {
    /// Validate committed configuration without requiring a completed flowsheet.
    pub fn validate_configuration(&self) -> RfResult<()> {
        if !self.boundary.temperature_k.is_finite()
            || self.boundary.temperature_k <= 0.0
            || !self.boundary.pressure_pa.is_finite()
            || self.boundary.pressure_pa <= 0.0
            || !(1..=MAX_ITERATION_EVALUATIONS).contains(&self.max_iteration_evaluations)
        {
            return Err(failure(
                "invalid_options",
                "positive finite fixed T/P and iteration budget in 1..=500 are required",
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for tear in &self.tears {
            if tear.stream_id.as_str().trim().is_empty() || !seen.insert(&tear.stream_id) {
                return Err(
                    failure("duplicate_tear", "tear stream must be nonblank and unique")
                        .with_related_stream_id(tear.stream_id.clone()),
                );
            }
            for target in [&tear.source, &tear.sink] {
                if target.unit_id.as_str().trim().is_empty() || target.port_name.trim().is_empty() {
                    return Err(failure("tear_binding", "blank unit or port binding"));
                }
            }
            for (values, positive) in [
                (&tear.initial_component_flows_mol_s, false),
                (&tear.component_scales_mol_s, true),
            ] {
                if let Some(values) = values {
                    checks::validate_vector(
                        &values.keys().cloned().collect::<Vec<_>>(),
                        values,
                        positive,
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn new(boundary: FixedRecycleBoundary, tears: Vec<TearSpecification>) -> Self {
        Self {
            boundary,
            tears,
            max_iteration_evaluations: MAX_ITERATION_EVALUATIONS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    Acyclic,
    Recycle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionBlock {
    pub members: Vec<UnitId>,
    pub kind: BlockKind,
    recycle: Option<RecyclePlan>,
}

#[derive(Debug, Clone, PartialEq)]
struct RecyclePlan {
    tear: TearSpecification,
    fresh_stream: StreamId,
    product_stream: StreamId,
    mixer: UnitId,
    splitter: UnitId,
}

/// Owns a frozen engineering input. Callers cannot edit the assembled plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PartitionedSolvePlan {
    flowsheet: Flowsheet,
    connections: Vec<MaterialConnection>,
    components: Vec<ComponentId>,
    options: RecycleSolveOptions,
    blocks: Vec<ExecutionBlock>,
}

impl PartitionedSolvePlan {
    pub fn blocks(&self) -> &[ExecutionBlock] {
        &self.blocks
    }
    pub fn connections(&self) -> &[MaterialConnection] {
        &self.connections
    }
    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }
    pub fn options(&self) -> &RecycleSolveOptions {
        &self.options
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalUnitEvaluation {
    pub unit_id: UnitId,
    pub consumed_streams: Vec<MaterialStreamState>,
    pub produced_streams: Vec<MaterialStreamState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationPhase {
    Iteration,
    Verification,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResidualSample {
    pub phase: EvaluationPhase,
    pub evaluation: usize,
    pub max_absolute_mol_s: f64,
    pub max_scaled: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecycleTolerances {
    pub component_absolute_flow_mol_s: f64,
    pub scaled_flow: f64,
    pub total_absolute_flow_mol_s: f64,
    pub temperature_k: f64,
    pub pressure_pa: f64,
}
impl RecycleTolerances {
    fn for_components(count: usize) -> Self {
        Self {
            component_absolute_flow_mol_s: ABSOLUTE_FLOW_TOLERANCE,
            scaled_flow: SCALED_FLOW_TOLERANCE,
            total_absolute_flow_mol_s: count as f64 * ABSOLUTE_FLOW_TOLERANCE,
            temperature_k: TEMPERATURE_TOLERANCE_K,
            pressure_pa: PRESSURE_TOLERANCE_PA,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZeroFlowCompositionRule {
    EvaluatedFreshBoundary(StreamId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecycleBlockReport {
    pub boundary: FixedRecycleBoundary,
    pub zero_flow_composition_rule: ZeroFlowCompositionRule,
    pub tolerances: RecycleTolerances,
    pub tear: TearSpecification,
    pub iteration_evaluations: usize,
    pub verification_evaluations: usize,
    pub component_scales_mol_s: BTreeMap<ComponentId, f64>,
    pub assumed: MaterialStreamState,
    pub returned: MaterialStreamState,
    pub residuals_mol_s: BTreeMap<ComponentId, f64>,
    /// At most 500 iteration samples and one independent verification sample.
    pub residual_trace: Vec<ResidualSample>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockReport {
    pub members: Vec<UnitId>,
    pub kind: BlockKind,
    /// Only the final evaluation, including the actual injected inlet, is retained.
    pub final_units: Vec<LocalUnitEvaluation>,
    pub recycle: Option<RecycleBlockReport>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SolveCosts {
    pub operation_constructions: usize,
    pub local_unit_calls: usize,
    pub calls_by_unit: BTreeMap<UnitId, usize>,
    pub thermo_calls: usize,
    pub flash_calls: usize,
    pub iteration_evaluations: usize,
    pub verification_evaluations: usize,
    pub assembly_elapsed: Duration,
    pub initialization_elapsed: Duration,
    pub elapsed: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartitionedSolveResult {
    pub streams: BTreeMap<StreamId, MaterialStreamState>,
    pub blocks: Vec<BlockReport>,
    pub costs: SolveCosts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionedFailureKind {
    Assembly,
    Initialization,
    LocalEvaluation,
    Boundary,
    Balance,
    NoFiniteSteadyState,
    IterationLimit,
    Verification,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartitionedSolveFailure {
    pub kind: PartitionedFailureKind,
    pub error: RfError,
    /// Diagnostic evidence only: never a partially publishable solve result.
    pub completed_blocks: Vec<BlockReport>,
    pub stage: PartitionedProgress,
    pub active_units: Vec<LocalUnitEvaluation>,
    pub active_recycle: Option<RecycleBlockReport>,
    pub costs: SolveCosts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionedStage {
    AssemblyStarted,
    AssemblyCompleted,
    Initializing,
    BlockStarted,
    IterationStarted,
    IterationCompleted,
    VerificationStarted,
    VerificationCompleted,
    UnitStarted,
    UnitCompleted,
    Completing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartitionedProgress {
    pub evaluation: Option<usize>,
    pub residual: Option<ResidualSample>,
    pub phase: Option<EvaluationPhase>,
    pub stage: PartitionedStage,
    pub block: Option<usize>,
    pub unit: Option<UnitId>,
}

pub trait PartitionedSolverControl {
    /// Complete local calls are indivisible; no cancellation inside thermo/flash.
    fn checkpoint(&mut self, progress: PartitionedProgress) -> ControlFlow<()>;
}
impl<F: FnMut(PartitionedProgress) -> ControlFlow<()>> PartitionedSolverControl for F {
    fn checkpoint(&mut self, progress: PartitionedProgress) -> ControlFlow<()> {
        self(progress)
    }
}

#[derive(Debug, Default)]
pub struct PartitionedSequentialSolver;

impl PartitionedSequentialSolver {
    pub fn solve(
        &self,
        services: &SolverServices<'_>,
        flowsheet: &Flowsheet,
        options: &RecycleSolveOptions,
    ) -> Result<PartitionedSolveResult, Box<PartitionedSolveFailure>> {
        self.solve_controlled(services, flowsheet, options, &mut |_| {
            ControlFlow::Continue(())
        })
    }

    pub fn solve_controlled(
        &self,
        services: &SolverServices<'_>,
        flowsheet: &Flowsheet,
        options: &RecycleSolveOptions,
        control: &mut impl PartitionedSolverControl,
    ) -> Result<PartitionedSolveResult, Box<PartitionedSolveFailure>> {
        execution::solve(services, flowsheet, options, control)
    }
}

fn failure(code: &str, message: impl Into<String>) -> RfError {
    RfError::invalid_input(message).with_diagnostic_code(format!("solver.partitioned.{code}"))
}

#[cfg(test)]
mod tests;

impl std::fmt::Display for PartitionedSolveFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.kind, self.error)
    }
}
impl std::error::Error for PartitionedSolveFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
