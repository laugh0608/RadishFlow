//! QS3 bounded fixed-T/P material equations. No energy or phase-equilibrium closure.
mod assembly;
pub(crate) use assembly::validate_parameters as validate_equation_parameters;
mod bindings;
mod block;
mod evaluation;
mod execution;
mod report;
mod verification;
pub(crate) use block::solve_block_with_factory;
pub use block::{EquationBlockInput, assemble_block};
pub use report::*;
mod structure;
#[cfg(test)]
mod tests;

pub use crate::partitioned::FixedRecycleBoundary as FixedBoundary;
pub use assembly::assemble;
use rf_flowsheet::MaterialConnection;
use rf_model::{Flowsheet, MaterialStreamState};
use rf_types::{ComponentId, RfError, StreamId, UnitId};
use rf_unitops::component_equations::ComponentEquation;
pub use rf_unitops::component_equations::{MaterialEquationRole, MaterialPort};
use std::collections::BTreeMap;

pub const EQUATION_CONTRACT_VERSION: u32 = 1;
pub const ACCEPTANCE_CONTRACT_VERSION: u32 = 1;
/// Core equation and acceptance contract for a terminal component mole fraction.
pub const PRODUCT_COMPOSITION_CONTRACT_VERSION: u32 = 2;
pub const MAX_DIMENSION: usize = 128;
pub const ABSOLUTE_TOLERANCE_MOL_S: f64 = 1e-10;
pub const SCALED_TOLERANCE: f64 = 1e-10;
pub const COMPOSITION_TOLERANCE_MOL_MOL: f64 = 1e-10;
pub const FRACTION_INTERVAL_TOLERANCE: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParameterField {
    OutletASplitFraction,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParameterBinding {
    pub unit: UnitId,
    pub field: ParameterField,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PortBinding {
    pub unit: UnitId,
    pub port: MaterialPort,
    pub stream: StreamId,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VariableBinding {
    PortComponent {
        port: PortBinding,
        component: ComponentId,
    },
    Parameter(ParameterBinding),
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EquationBinding {
    Unit {
        unit: UnitId,
        role: MaterialEquationRole,
        component: ComponentId,
    },
    Connection {
        source: PortBinding,
        sink: PortBinding,
        component: ComponentId,
    },
    Boundary {
        source_unit: UnitId,
        source_port: String,
        sink: PortBinding,
        component: ComponentId,
    },
    ProductTotal {
        source: PortBinding,
    },
    ProductMoleFraction {
        source: PortBinding,
        component: ComponentId,
    },
}
impl EquationBinding {
    fn is_product_target(&self) -> bool {
        matches!(
            self,
            Self::ProductTotal { .. } | Self::ProductMoleFraction { .. }
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ReleasedSplitFraction {
    pub binding: ParameterBinding,
    pub lower: f64,
    pub upper: f64,
    pub max_iterations: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ProductTotalTarget {
    pub source: PortBinding,
    pub value_mol_s: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ProductMoleFractionTarget {
    pub source: PortBinding,
    pub component: ComponentId,
    pub value_mol_mol: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ProductTarget {
    TotalMolarFlow(ProductTotalTarget),
    MoleFraction(Box<ProductMoleFractionTarget>),
}
impl ProductTarget {
    pub fn source(&self) -> &PortBinding {
        match self {
            Self::TotalMolarFlow(t) => &t.source,
            Self::MoleFraction(t) => &t.source,
        }
    }
    pub fn total_flow(&self) -> Option<&ProductTotalTarget> {
        match self {
            Self::TotalMolarFlow(t) => Some(t),
            Self::MoleFraction(_) => None,
        }
    }
    pub fn total_flow_mut(&mut self) -> Option<&mut ProductTotalTarget> {
        match self {
            Self::TotalMolarFlow(t) => Some(t),
            Self::MoleFraction(_) => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ExplicitEquationScales {
    pub variables: BTreeMap<VariableBinding, f64>,
    pub equations: BTreeMap<EquationBinding, f64>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EquationOptions {
    pub boundary: FixedBoundary,
    pub released: Option<ReleasedSplitFraction>,
    pub target: Option<ProductTarget>,
    pub scales: Option<ExplicitEquationScales>,
}
impl EquationOptions {
    pub fn new(boundary: FixedBoundary) -> Self {
        Self {
            boundary,
            released: None,
            target: None,
            scales: None,
        }
    }
    /// Equation and acceptance contract versions for this execution, in that order.
    pub fn contract_versions(&self) -> (u32, u32) {
        if self.requires_product_composition() {
            (
                PRODUCT_COMPOSITION_CONTRACT_VERSION,
                PRODUCT_COMPOSITION_CONTRACT_VERSION,
            )
        } else {
            (EQUATION_CONTRACT_VERSION, ACCEPTANCE_CONTRACT_VERSION)
        }
    }
    pub fn requires_product_composition(&self) -> bool {
        matches!(self.target, Some(ProductTarget::MoleFraction(_)))
            || self.scales.as_ref().is_some_and(|scales| {
                scales
                    .equations
                    .keys()
                    .any(|row| matches!(row, EquationBinding::ProductMoleFraction { .. }))
            })
    }
    /// Shape is independent of live graph bindings and pairing readiness.
    pub fn validate_shape(&self) -> EoResult<()> {
        self.validate_identities()?;
        let b = self.boundary;
        if !b.temperature_k.is_finite()
            || b.temperature_k <= 0.0
            || !b.pressure_pa.is_finite()
            || b.pressure_pa <= 0.0
        {
            return Err(error(
                EquationFailureKind::InvalidSpecification,
                "boundary",
                "positive finite fixed T/P required",
            ));
        }
        if let Some(r) = &self.released
            && (!r.lower.is_finite()
                || !r.upper.is_finite()
                || r.lower < 0.0
                || r.upper > 1.0
                || r.lower >= r.upper
                || !(1..=128).contains(&r.max_iterations))
        {
            return Err(error(
                EquationFailureKind::InvalidSpecification,
                "interval",
                "split interval requires 0 <= lower < upper <= 1 and 1..128 bisections",
            ));
        }
        match &self.target {
            Some(ProductTarget::TotalMolarFlow(t))
                if !t.value_mol_s.is_finite() || t.value_mol_s <= 0.0 =>
            {
                return Err(error(
                    EquationFailureKind::InvalidSpecification,
                    "target",
                    "product flow target must be positive and finite",
                ));
            }
            Some(ProductTarget::MoleFraction(t)) if !(0.0..=1.0).contains(&t.value_mol_mol) => {
                return Err(error(
                    EquationFailureKind::InvalidSpecification,
                    "target",
                    "product mole fraction must be finite and in [0, 1]",
                ));
            }
            _ => {}
        }
        if self.scales.as_ref().is_some_and(|s| {
            s.variables
                .values()
                .chain(s.equations.values())
                .any(|v| !v.is_finite() || *v <= 0.0)
        }) {
            return Err(error(
                EquationFailureKind::InvalidSpecification,
                "scales",
                "scales must be positive and finite",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralRegion {
    pub variables: Vec<VariableBinding>,
    pub equations: Vec<EquationBinding>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralReport {
    pub variable_count: usize,
    pub equation_count: usize,
    pub matching: Vec<(EquationBinding, VariableBinding)>,
    pub unmatched_variables: Vec<VariableBinding>,
    pub unmatched_equations: Vec<EquationBinding>,
    pub underdetermined: StructuralRegion,
    pub overdetermined: StructuralRegion,
}
impl StructuralReport {
    pub fn is_square_matched(&self) -> bool {
        self.variable_count == self.equation_count && self.matching.len() == self.variable_count
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquationFailureKind {
    UnsupportedModel,
    InvalidSpecification,
    InvalidBinding,
    Capacity,
    StructuralUnder,
    StructuralOver,
    StructuralSingular,
    LinearSingular,
    IllConditioned,
    LinearVerification,
    TargetNotBracketed,
    BudgetExceeded,
    Stagnated,
    NonFinite,
    PhysicalDomain,
    UndefinedComposition,
    LocalVerification,
    GlobalVerification,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EquationError {
    pub kind: EquationFailureKind,
    pub error: RfError,
}
impl std::fmt::Display for EquationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.error)
    }
}
impl std::error::Error for EquationError {}
pub type EoResult<T> = Result<T, EquationError>;
fn error(kind: EquationFailureKind, code: &str, message: impl Into<String>) -> EquationError {
    EquationError {
        kind,
        error: RfError::invalid_input(message)
            .with_diagnostic_code(format!("solver.equation_oriented.{code}")),
    }
}
fn mapped(kind: EquationFailureKind, error: RfError) -> EquationError {
    EquationError { kind, error }
}

#[derive(Debug, Clone, PartialEq)]
enum AssembledEquation {
    Unit {
        model: ComponentEquation,
        ports: BTreeMap<MaterialPort, usize>,
        fraction: Option<usize>,
    },
    Connection {
        source: usize,
        sink: usize,
    },
    Boundary {
        sink: usize,
        value: f64,
    },
    ProductTotal {
        components: Vec<usize>,
        target: f64,
    },
    ProductMoleFraction {
        components: Vec<usize>,
        selected: usize,
        target: f64,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct EquationEvaluation {
    pub residuals_mol_s: Vec<f64>,
    pub jacobian_si: Vec<Vec<f64>>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EquationProblem {
    flowsheet: Flowsheet,
    connections: Vec<MaterialConnection>,
    components: Vec<ComponentId>,
    options: EquationOptions,
    variables: Vec<VariableBinding>,
    equations: Vec<EquationBinding>,
    rows: Vec<AssembledEquation>,
    feeds: BTreeMap<UnitId, MaterialStreamState>,
    external_inputs: BTreeMap<StreamId, MaterialStreamState>,
    variable_scales: Vec<f64>,
    residual_scales: Vec<f64>,
    fresh_total_mol_s: f64,
    structure: StructuralReport,
}
impl EquationProblem {
    pub fn variables(&self) -> &[VariableBinding] {
        &self.variables
    }
    pub fn equations(&self) -> &[EquationBinding] {
        &self.equations
    }
    pub fn components(&self) -> &[ComponentId] {
        &self.components
    }
    pub fn options(&self) -> &EquationOptions {
        &self.options
    }
    pub fn variable_scales(&self) -> &[f64] {
        &self.variable_scales
    }
    pub fn residual_scales(&self) -> &[f64] {
        &self.residual_scales
    }
    pub fn fresh_total_mol_s(&self) -> f64 {
        self.fresh_total_mol_s
    }
    pub fn structure(&self) -> &StructuralReport {
        &self.structure
    }
    pub fn require_ready(&self) -> EoResult<()> {
        // A fixed closed outlet can make the structure singular before either
        // bracket endpoint is evaluated. Its composition is already undefined.
        if let Some(ProductTarget::MoleFraction(t)) = &self.options.target {
            let source = &self.flowsheet.units[&t.source.unit];
            let fixed = self
                .options
                .released
                .as_ref()
                .is_none_or(|r| r.binding.unit != source.id);
            if fixed
                && matches!(
                    (t.source.port, source.parameters.outlet_a_split_fraction),
                    (MaterialPort::OutletA, Some(0.0)) | (MaterialPort::OutletB, Some(1.0))
                )
                || self
                    .feeds
                    .get(&source.id)
                    .is_some_and(|s| s.total_molar_flow_mol_s == 0.0)
            {
                let mut e = error(
                    EquationFailureKind::UndefinedComposition,
                    "product_flow",
                    "the fixed terminal source produces zero mol/s",
                );
                e.error = e
                    .error
                    .with_related_stream_id(t.source.stream.clone())
                    .with_related_component_id(t.component.clone());
                return Err(e);
            }
        }
        if !self.structure.is_square_matched() {
            let kind = match (
                self.structure.unmatched_variables.is_empty(),
                self.structure.unmatched_equations.is_empty(),
            ) {
                (false, true) => EquationFailureKind::StructuralUnder,
                (true, false) => EquationFailureKind::StructuralOver,
                _ => EquationFailureKind::StructuralSingular,
            };
            return Err(error(
                kind,
                "structure",
                "equations are not square and fully structurally matched",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EquationStage {
    Assemble,
    StructuralAnalysis,
    LinearEvaluation,
    BracketEndpoint,
    IntervalEvaluation,
    Factorization,
    ConditionSolve,
    FullJacobianCheck,
    FinalUnitVerification,
    GlobalVerification,
    Completing,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquationProgress {
    pub stage: EquationStage,
    pub unit: Option<UnitId>,
    pub row: Option<usize>,
    pub column: Option<usize>,
    pub iteration: Option<usize>,
}
impl EquationProgress {
    fn at(stage: EquationStage) -> Self {
        Self {
            stage,
            unit: None,
            row: None,
            column: None,
            iteration: None,
        }
    }
}
pub trait EquationControl {
    fn checkpoint(&mut self, progress: EquationProgress) -> std::ops::ControlFlow<()>;
}
struct Uninterrupted;
impl EquationControl for Uninterrupted {
    fn checkpoint(&mut self, _: EquationProgress) -> std::ops::ControlFlow<()> {
        std::ops::ControlFlow::Continue(())
    }
}
fn checkpoint(control: &mut impl EquationControl, progress: EquationProgress) -> EoResult<()> {
    if control.checkpoint(progress).is_break() {
        Err(error(
            EquationFailureKind::Cancelled,
            "cancelled",
            "equation execution cancelled",
        ))
    } else {
        Ok(())
    }
}
