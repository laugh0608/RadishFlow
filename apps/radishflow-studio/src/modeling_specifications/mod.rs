//! Read-only application specification checks for the six existing steady-state
//! built-ins. No window, IO, property calculation, edit, or new wire protocol.

mod checks;

use rf_flowsheet::pressure_context::DocumentPressureContext;
use rf_flowsheet::specifications::{FeedInputField, FeedInputValue, UnitParameter};
use rf_model::Flowsheet;
use rf_types::{ComponentId, RfError, StreamId, UnitId};
use rf_ui::{DocumentId, FlowsheetDocument};

pub use rf_flowsheet::specifications::SPECIFICATION_CONTRACT_VERSION;

/// Supplied by the owner of an open document. Reopening or replacing that
/// document requires a new generation even when persisted ID/revision match.
/// A1-I1 only captures it; the N1 lifecycle / UI consumer belongs to A1-I2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelingCheckSessionId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificationReportIdentity {
    pub document_id: DocumentId,
    pub session: ModelingCheckSessionId,
    pub input_revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecificationAnalysisMethod {
    BuiltinSteadyStateInputRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecificationRule {
    Structure,
    ProjectComposition,
    FeedInputs,
    CompositionValues,
    ExplicitParameters,
    ParameterValues,
    PressureRelationship,
    AnalysisCoverage,
}

impl SpecificationRule {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Structure => "A1-R01",
            Self::ProjectComposition => "A1-R02",
            Self::FeedInputs => "A1-R03",
            Self::CompositionValues => "A1-R04",
            Self::ExplicitParameters => "A1-R05",
            Self::ParameterValues => "A1-R06",
            Self::PressureRelationship => "A1-R07",
            Self::AnalysisCoverage => "A1-R08",
        }
    }

    pub const fn version(self) -> u32 {
        SPECIFICATION_CONTRACT_VERSION
    }
}

/// Where the existing public run contract enforces this fact. In particular a
/// preflight pressure conflict must not silently change `failed` into `blocked`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecificationEnforcement {
    AuthoringReadiness,
    Solver,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecificationProblem {
    EmptyFlowsheet,
    Topology(RfError),
    MissingProjectComponents,
    ComponentOutsideProject {
        stream_id: StreamId,
        component_id: ComponentId,
    },
    InvalidFeedInput {
        unit_id: UnitId,
        stream_id: StreamId,
        field: FeedInputField,
        value: f64,
    },
    MissingFeedComposition {
        unit_id: UnitId,
        stream_id: StreamId,
    },
    InvalidMoleFraction {
        stream_id: StreamId,
        component_id: ComponentId,
        value: f64,
    },
    InvalidCompositionTotal {
        stream_id: StreamId,
        sum: f64,
    },
    UnnormalizedComposition {
        stream_id: StreamId,
        sum: f64,
    },
    MissingParameter {
        unit_id: UnitId,
        parameter: UnitParameter,
    },
    InvalidParameter {
        unit_id: UnitId,
        parameter: UnitParameter,
        value: f64,
    },
    OutletPressureAboveInlet {
        unit_id: UnitId,
        pressure_pa: f64,
        context: DocumentPressureContext,
    },
}

impl SpecificationProblem {
    pub fn rule(&self) -> SpecificationRule {
        match self {
            Self::EmptyFlowsheet | Self::Topology(_) => SpecificationRule::Structure,
            Self::MissingProjectComponents | Self::ComponentOutsideProject { .. } => {
                SpecificationRule::ProjectComposition
            }
            Self::InvalidFeedInput { .. } | Self::MissingFeedComposition { .. } => {
                SpecificationRule::FeedInputs
            }
            Self::InvalidMoleFraction { .. }
            | Self::InvalidCompositionTotal { .. }
            | Self::UnnormalizedComposition { .. } => SpecificationRule::CompositionValues,
            Self::MissingParameter { .. } => SpecificationRule::ExplicitParameters,
            Self::InvalidParameter { .. } => SpecificationRule::ParameterValues,
            Self::OutletPressureAboveInlet { .. } => SpecificationRule::PressureRelationship,
        }
    }

    pub fn enforcement(&self) -> SpecificationEnforcement {
        match self {
            Self::Topology(_)
            | Self::InvalidParameter { .. }
            | Self::OutletPressureAboveInlet { .. } => SpecificationEnforcement::Solver,
            _ => SpecificationEnforcement::AuthoringReadiness,
        }
    }

    pub fn severity(&self) -> rf_ui::DiagnosticSeverity {
        rf_ui::DiagnosticSeverity::Error
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecificationCheckScope {
    Topology,
    ProjectComponents,
    FeedInputs,
    StreamCompositions,
    UnitParameters(UnitId),
    InletPressure(UnitId),
    StructuralDegreesOfFreedom,
    NumericalSolvability,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpecificationCoverage {
    /// The applicable predicates were evaluated, not necessarily passed.
    Evaluated,
    NotCheckedTopology,
    NotCheckedMissingOrInvalidParameter,
    NotCheckedInputPressure(DocumentPressureContext),
    AwaitingExecution(DocumentPressureContext),
    UnsupportedUnitKind(String),
    NotAnalyzed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpecificationCheckCoverage {
    pub scope: SpecificationCheckScope,
    pub coverage: SpecificationCoverage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FeedSpecificationInput {
    pub unit_id: UnitId,
    pub stream_id: StreamId,
    pub field: FeedInputField,
    pub input: FeedInputValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelingSpecificationReport {
    pub identity: SpecificationReportIdentity,
    pub method: SpecificationAnalysisMethod,
    pub contract_version: u32,
    /// Typed reasons carry stable object/field targets and canonical SI evidence.
    /// UI templates and legacy run messages are consumer projections.
    pub problems: Vec<SpecificationProblem>,
    pub coverage: Vec<SpecificationCheckCoverage>,
    /// Includes valid inputs too, so an effective Feed override remains explainable.
    pub feed_inputs: Vec<FeedSpecificationInput>,
}

/// Same document / session yields the same ordered facts. No snapshots, display
/// preferences, drafts, caches, or solver services participate in this query.
pub fn check_modeling_specifications(
    document: &FlowsheetDocument,
    session: ModelingCheckSessionId,
) -> ModelingSpecificationReport {
    let checks = check_inputs(&document.flowsheet);
    ModelingSpecificationReport {
        identity: SpecificationReportIdentity {
            document_id: document.metadata.document_id.clone(),
            session,
            input_revision: document.revision,
        },
        method: SpecificationAnalysisMethod::BuiltinSteadyStateInputRules,
        contract_version: SPECIFICATION_CONTRACT_VERSION,
        problems: checks.problems,
        coverage: checks.coverage,
        feed_inputs: checks.feed_inputs,
    }
}

#[derive(Default)]
pub(crate) struct InputChecks {
    pub problems: Vec<SpecificationProblem>,
    pub coverage: Vec<SpecificationCheckCoverage>,
    pub feed_inputs: Vec<FeedSpecificationInput>,
}

impl InputChecks {
    fn coverage(&mut self, scope: SpecificationCheckScope, coverage: SpecificationCoverage) {
        self.coverage
            .push(SpecificationCheckCoverage { scope, coverage });
    }

    /// Retain the old application's first-problem projection, including deferring
    /// topology errors to the solver before any authoring blocker is returned.
    pub(crate) fn first_readiness_problem(&self) -> Option<&SpecificationProblem> {
        if self
            .problems
            .iter()
            .any(|problem| matches!(problem, SpecificationProblem::Topology(_)))
        {
            return None;
        }
        self.problems
            .iter()
            .find(|problem| problem.enforcement() == SpecificationEnforcement::AuthoringReadiness)
    }
}

pub(crate) fn check_inputs(flowsheet: &Flowsheet) -> InputChecks {
    checks::check_inputs(flowsheet)
}

#[cfg(test)]
mod tests;
