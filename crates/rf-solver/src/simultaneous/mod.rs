//! QS2 fixed-T/P affine simultaneous modular blocks. No persistence or app plan.
mod assembly;
mod dense;
mod evaluation;
mod execution;
#[cfg(test)]
mod tests;

pub use crate::partitioned::{FixedRecycleBoundary as FixedBoundary, LocalUnitEvaluation};
pub use assembly::assemble;
use rf_flowsheet::MaterialConnection;
use rf_model::{Flowsheet, MaterialStreamState};
use rf_types::{ComponentId, DiagnosticPortTarget, RfError, RfResult, StreamId, UnitId};
pub use rf_unitops::component_response::{AffineComponentCapability, AffineComponentResponse};
use std::{collections::BTreeMap, ops::ControlFlow, time::Duration};

pub const METHOD: &str = "rf-solver.fixed-tp-affine-modular-lu/1";
pub const ACCEPTANCE_VERSION: u32 = 1;
pub const RESPONSE_VERSION: u32 = rf_unitops::component_response::RESPONSE_VERSION;
pub const MAX_UNKNOWNS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CouplingBinding {
    pub stream: StreamId,
    pub source: DiagnosticPortTarget,
    pub sink: DiagnosticPortTarget,
    pub component: ComponentId,
}
impl Ord for CouplingBinding {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            &self.stream,
            &self.source.unit_id,
            &self.source.port_name,
            &self.sink.unit_id,
            &self.sink.port_name,
            &self.component,
        )
            .cmp(&(
                &other.stream,
                &other.source.unit_id,
                &other.source.port_name,
                &other.sink.unit_id,
                &other.sink.port_name,
                &other.component,
            ))
    }
}
impl PartialOrd for CouplingBinding {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct CouplingScale {
    pub variable_mol_s: f64,
    pub residual_mol_s: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ModularOptions {
    pub boundary: FixedBoundary,
    /// Complete exact mapping, or evaluated block fresh-total scales by default.
    pub scales: Option<BTreeMap<CouplingBinding, CouplingScale>>,
}
impl ModularOptions {
    pub fn new(boundary: FixedBoundary) -> Self {
        Self {
            boundary,
            scales: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ModularBlock {
    pub members: Vec<UnitId>,
    pub couplings: Vec<CouplingBinding>,
    pub responses: Vec<AffineComponentResponse>,
    pub fresh_by_mixer: BTreeMap<UnitId, StreamId>,
    pub products: Vec<StreamId>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ModularPlan {
    flowsheet: Flowsheet,
    connections: Vec<MaterialConnection>,
    components: Vec<ComponentId>,
    options: ModularOptions,
    blocks: Vec<ModularBlock>,
}
impl ModularPlan {
    pub fn blocks(&self) -> &[ModularBlock] {
        &self.blocks
    }
    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }
    pub fn options(&self) -> &ModularOptions {
        &self.options
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModularStage {
    AssemblyStarted,
    AssemblyCompleted,
    Initializing,
    BlockStarted,
    Anchor,
    UnitStarted,
    UnitCompleted,
    Response,
    Factorization,
    ConditionSolve,
    LinearVerification,
    Verification,
    BlockCompleted,
    Completing,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModularProgress {
    pub stage: ModularStage,
    pub block: Option<usize>,
    pub unit: Option<UnitId>,
    pub column: Option<usize>,
}
pub trait ModularControl {
    fn checkpoint(&mut self, progress: ModularProgress) -> ControlFlow<()>;
}
struct Uninterrupted;
impl ModularControl for Uninterrupted {
    fn checkpoint(&mut self, _: ModularProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModularCosts {
    pub operation_constructions: usize,
    pub local_unit_calls: usize,
    pub calls_by_unit: BTreeMap<UnitId, usize>,
    pub thermo_calls: usize,
    pub flash_calls: usize,
    pub response_evaluations: usize,
    pub anchor_evaluations: usize,
    pub verification_evaluations: usize,
    pub factorizations: usize,
    pub triangular_solves: usize,
    pub assembly_elapsed: Duration,
    pub initialization_elapsed: Duration,
    pub anchor_elapsed: Duration,
    pub response_elapsed: Duration,
    pub linear_elapsed: Duration,
    pub verification_elapsed: Duration,
    pub elapsed: Duration,
}
pub use crate::dense_linear::LinearEvidence;
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionEvidence {
    pub binding: CouplingBinding,
    pub scale: CouplingScale,
    pub assumed_mol_s: f64,
    pub produced_mol_s: f64,
    pub residual_mol_s: f64,
    pub scaled_residual: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CoupledEvidence {
    pub boundary: FixedBoundary,
    pub response_version: u32,
    pub acceptance_version: u32,
    pub responses: Vec<AffineComponentResponse>,
    pub zero_flow_fresh_boundaries: BTreeMap<StreamId, StreamId>,
    pub anchor_residuals_mol_s: Vec<f64>,
    pub connections: Vec<ConnectionEvidence>,
    pub linear: Option<LinearEvidence>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ModularBlockReport {
    pub members: Vec<UnitId>,
    pub final_units: Vec<LocalUnitEvaluation>,
    pub coupling: Option<CoupledEvidence>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ModularResult {
    pub method: &'static str,
    pub streams: BTreeMap<StreamId, MaterialStreamState>,
    pub blocks: Vec<ModularBlockReport>,
    pub costs: ModularCosts,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModularFailureKind {
    Assembly,
    Initialization,
    LocalEvaluation,
    Boundary,
    Balance,
    Response,
    NoFiniteSteadyState,
    Singular,
    IllConditioned,
    LinearVerification,
    Verification,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ModularFailure {
    pub kind: ModularFailureKind,
    pub error: RfError,
    /// Diagnostic only. No partial result or successful stream table is exposed.
    pub completed_blocks: Vec<ModularBlockReport>,
    pub active_units: Vec<LocalUnitEvaluation>,
    pub active_coupling: Option<CoupledEvidence>,
    pub progress: ModularProgress,
    pub costs: ModularCosts,
}
impl std::fmt::Display for ModularFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.error)
    }
}
impl std::error::Error for ModularFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct SimultaneousModularSolver;
impl SimultaneousModularSolver {
    pub fn solve(
        &self,
        services: &crate::SolverServices<'_>,
        flowsheet: &Flowsheet,
        options: &ModularOptions,
    ) -> Result<ModularResult, Box<ModularFailure>> {
        self.solve_controlled(services, flowsheet, options, &mut Uninterrupted)
    }
    pub fn solve_controlled(
        &self,
        services: &crate::SolverServices<'_>,
        flowsheet: &Flowsheet,
        options: &ModularOptions,
        control: &mut impl ModularControl,
    ) -> Result<ModularResult, Box<ModularFailure>> {
        execution::solve(services, flowsheet, options, control)
    }
}
fn failure(code: &str, message: impl Into<String>) -> RfError {
    RfError::invalid_input(message).with_diagnostic_code(format!("solver.simultaneous.{code}"))
}
#[derive(Debug)]
struct RunError {
    kind: ModularFailureKind,
    error: RfError,
}
type RunResult<T> = Result<T, RunError>;
fn classify<T>(kind: ModularFailureKind, result: RfResult<T>) -> RunResult<T> {
    result.map_err(|mut error| {
        if let Some(suffix) = error
            .context()
            .diagnostic_code()
            .and_then(|code| code.strip_prefix("solver.partitioned."))
        {
            let code = format!("solver.simultaneous.{suffix}");
            error = error.with_diagnostic_code(code);
        }
        RunError { kind, error }
    })
}
