use super::*;
pub use crate::dense_linear::LinearEvidence;
pub use crate::partitioned::LocalUnitEvaluation;
use std::time::Duration;

pub const FORWARD_BACKEND: &str = "rf-solver.fixed-tp-material-eo-lu/1";
pub const INVERSE_BACKEND: &str = "rf-solver.fixed-tp-material-eo-conditional-lu-bisection/1";
#[derive(Debug, Clone, PartialEq)]
pub struct EquationDefinition {
    pub variables: Vec<VariableBinding>,
    pub equations: Vec<EquationBinding>,
    pub variable_scales_si: Vec<f64>,
    pub residual_scales_mol_s: Vec<f64>,
    pub structure: StructuralReport,
    pub fresh_total_mol_s: f64,
    pub fixed_split_fractions: BTreeMap<ParameterBinding, f64>,
    pub options: EquationOptions,
}
impl From<&EquationProblem> for EquationDefinition {
    fn from(p: &EquationProblem) -> Self {
        Self {
            variables: p.variables.clone(),
            equations: p.equations.clone(),
            variable_scales_si: p.variable_scales.clone(),
            residual_scales_mol_s: p.residual_scales.clone(),
            structure: p.structure.clone(),
            fresh_total_mol_s: p.fresh_total_mol_s,
            options: p.options.clone(),
            fixed_split_fractions: p
                .flowsheet
                .units
                .values()
                .filter(|u| {
                    u.kind == "splitter"
                        && p.options
                            .released
                            .as_ref()
                            .is_none_or(|r| r.binding.unit != u.id)
                })
                .map(|u| {
                    (
                        ParameterBinding {
                            unit: u.id.clone(),
                            field: ParameterField::OutletASplitFraction,
                        },
                        u.parameters
                            .outlet_a_split_fraction
                            .expect("validated fixed split"),
                    )
                })
                .collect(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalLinearEvaluation {
    pub fraction: Option<f64>,
    pub interval: Option<(f64, f64)>,
    pub linear: Option<LinearEvidence>,
    pub target_residual_mol_s: Option<f64>,
    pub completed: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SolvedSplitFraction {
    pub binding: ParameterBinding,
    pub stored_value: Option<f64>,
    pub value: f64,
    pub interval: (f64, f64),
}
#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedTarget {
    pub target: ProductTotalTarget,
    pub actual_mol_s: f64,
    pub residual_mol_s: f64,
    pub scaled_residual: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMaterialEvidence {
    pub fresh_components_mol_s: BTreeMap<ComponentId, f64>,
    pub product_components_mol_s: BTreeMap<ComponentId, f64>,
    pub component_residuals_mol_s: BTreeMap<ComponentId, f64>,
    pub total_residual_mol_s: f64,
}
/// Diagnostic state is populated only by work actually performed. Failed runs
/// carry this state without a successful stream table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EquationEvidence {
    pub definition: Option<EquationDefinition>,
    pub conditional_evaluations: Vec<ConditionalLinearEvaluation>,
    pub candidate_values_si: Option<Vec<f64>>,
    pub candidate_residuals_mol_s: Option<Vec<f64>>,
    pub full_jacobian: Option<LinearEvidence>,
    pub verified_values_si: Option<Vec<f64>>,
    pub verified_residuals_mol_s: Option<Vec<f64>>,
    pub solved_parameter: Option<SolvedSplitFraction>,
    pub verified_target: Option<VerifiedTarget>,
    pub final_units: Vec<LocalUnitEvaluation>,
    /// Every zero port records its physical composition anchor, never a template.
    pub zero_flow_sources: BTreeMap<PortBinding, PortBinding>,
    pub global_material: Option<GlobalMaterialEvidence>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EquationCosts {
    pub assemblies: usize,
    pub structural_analyses: usize,
    pub structural_checkpoints: usize,
    pub conditional_linear_evaluations: usize,
    pub full_jacobian_evaluations: usize,
    pub factorizations: usize,
    pub triangular_solves: usize,
    pub target_evaluations: usize,
    pub interval_updates: usize,
    pub operation_constructions: usize,
    pub local_unit_calls: usize,
    pub calls_by_unit: BTreeMap<UnitId, usize>,
    pub thermo_calls: usize,
    pub flash_calls: usize,
    pub stage_elapsed: BTreeMap<EquationStage, Duration>,
    pub elapsed: Duration,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EquationResult {
    pub backend: &'static str,
    pub equation_contract_version: u32,
    pub acceptance_contract_version: u32,
    pub streams: BTreeMap<StreamId, MaterialStreamState>,
    pub evidence: EquationEvidence,
    pub costs: EquationCosts,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EquationFailure {
    pub kind: EquationFailureKind,
    pub error: RfError,
    pub backend: Option<&'static str>,
    pub evidence: EquationEvidence,
    pub progress: EquationProgress,
    pub costs: EquationCosts,
}
impl std::fmt::Display for EquationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.error)
    }
}
impl std::error::Error for EquationFailure {}

#[derive(Debug, Default, Clone, Copy)]
pub struct EquationOrientedSolver;
impl EquationOrientedSolver {
    pub fn solve(
        &self,
        services: &crate::SolverServices<'_>,
        flow: &Flowsheet,
        options: &EquationOptions,
    ) -> Result<EquationResult, Box<EquationFailure>> {
        self.solve_controlled(services, flow, options, &mut Uninterrupted)
    }
    pub fn solve_controlled(
        &self,
        services: &crate::SolverServices<'_>,
        flow: &Flowsheet,
        options: &EquationOptions,
        control: &mut impl EquationControl,
    ) -> Result<EquationResult, Box<EquationFailure>> {
        execution::solve(
            services,
            flow,
            options,
            control,
            &mut rf_unitops::catalog::instantiate_builtin,
        )
    }
}
