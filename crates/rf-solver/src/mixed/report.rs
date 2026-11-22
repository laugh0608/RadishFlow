use super::*;
use crate::equation_oriented::{
    EquationFailure, EquationProgress, EquationResult, GlobalMaterialEvidence,
};
use crate::partitioned::{EvaluationPhase, LocalUnitEvaluation, ResidualSample};
use rf_flowsheet::MaterialConnection;
use rf_model::MaterialStreamState;
use rf_types::{ComponentId, StreamId};
use std::{ops::ControlFlow, time::Duration};

#[derive(Debug, Clone, PartialEq)]
pub enum MixedBlockEvidence {
    Local(LocalUnitEvaluation),
    EquationOriented(Box<EquationResult>),
}
impl MixedBlockEvidence {
    pub fn final_units(&self) -> &[LocalUnitEvaluation] {
        match self {
            Self::Local(u) => std::slice::from_ref(u),
            Self::EquationOriented(r) => &r.evidence.final_units,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct MixedBlockReport {
    pub id: MixedBlockId,
    pub members: BTreeSet<UnitId>,
    pub evidence: MixedBlockEvidence,
    pub elapsed: Duration,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MixedBoundaryEvidence {
    pub connection: MaterialConnection,
    pub producer: MaterialStreamState,
    pub consumed: MaterialStreamState,
    pub residuals_mol_s: BTreeMap<ComponentId, f64>,
    pub scaled_residuals: BTreeMap<ComponentId, f64>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MixedCosts {
    pub operation_constructions: usize,
    pub local_unit_calls: usize,
    pub calls_by_unit: BTreeMap<UnitId, usize>,
    pub thermo_calls: usize,
    pub flash_calls: usize,
    pub assembly_elapsed: Duration,
    pub elapsed: Duration,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MixedResult {
    pub method: &'static str,
    pub streams: BTreeMap<StreamId, MaterialStreamState>,
    pub blocks: Vec<MixedBlockReport>,
    pub boundaries: Vec<MixedBoundaryEvidence>,
    pub global_material: GlobalMaterialEvidence,
    pub costs: MixedCosts,
    pub recycle: Option<MixedRecycleReport>,
    pub serial_recycles: Vec<MixedRecycleRegionReport>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixedRecycleRegionReport {
    pub equation_block: MixedBlockId,
    pub recycle: MixedRecycleReport,
    pub balance: Option<GlobalMaterialEvidence>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixedRecycleReport {
    pub tear: TearSpecification,
    pub fresh_stream: StreamId,
    pub max_iteration_evaluations: usize,
    pub component_scales_mol_s: BTreeMap<ComponentId, f64>,
    pub residual_trace: Vec<ResidualSample>,
    pub last_sweep: Option<MixedRecycleSweep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixedRecycleSweep {
    pub assumed: MaterialStreamState,
    pub returned: MaterialStreamState,
    pub residuals_mol_s: BTreeMap<ComponentId, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MixedRecycleProgress {
    pub phase: EvaluationPhase,
    pub evaluation: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixedStage {
    Assembly,
    BlockStarted,
    Equation,
    Boundary,
    BlockCompleted,
    RegionVerification,
    GlobalVerification,
    Completing,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixedProgress {
    pub stage: MixedStage,
    pub block: Option<MixedBlockId>,
    pub unit: Option<UnitId>,
    pub equation: Option<EquationProgress>,
    pub recycle: Option<MixedRecycleProgress>,
    pub recycle_region: Option<MixedBlockId>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixedFailureKind {
    Assembly,
    LocalEvaluation,
    Equation,
    Boundary,
    IterationLimit,
    RecycleVerification,
    GlobalVerification,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MixedFailure {
    pub method: &'static str,
    pub kind: MixedFailureKind,
    pub error: RfError,
    pub progress: MixedProgress,
    pub completed_blocks: Vec<MixedBlockReport>,
    pub boundaries: Vec<MixedBoundaryEvidence>,
    pub active_units: Vec<LocalUnitEvaluation>,
    pub equation_failure: Option<Box<EquationFailure>>,
    pub costs: MixedCosts,
    pub recycle: Option<MixedRecycleReport>,
    pub serial_recycles: Vec<MixedRecycleRegionReport>,
}
impl std::fmt::Display for MixedFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.error)
    }
}
impl std::error::Error for MixedFailure {}
pub trait MixedControl {
    fn checkpoint(&mut self, progress: MixedProgress) -> ControlFlow<()>;
}
impl<F: FnMut(MixedProgress) -> ControlFlow<()>> MixedControl for F {
    fn checkpoint(&mut self, p: MixedProgress) -> ControlFlow<()> {
        self(p)
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct MixedSolver;
impl MixedSolver {
    pub fn solve(
        &self,
        services: &crate::SolverServices<'_>,
        flow: &Flowsheet,
        options: &MixedOptions,
    ) -> Result<MixedResult, Box<MixedFailure>> {
        self.solve_controlled(services, flow, options, &mut |_| ControlFlow::Continue(()))
    }
    pub fn solve_controlled(
        &self,
        services: &crate::SolverServices<'_>,
        flow: &Flowsheet,
        options: &MixedOptions,
        control: &mut impl MixedControl,
    ) -> Result<MixedResult, Box<MixedFailure>> {
        execution::solve(
            services,
            flow,
            options,
            control,
            &mut rf_unitops::catalog::instantiate_builtin,
        )
    }
}
