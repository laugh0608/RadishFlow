use std::fmt;

use rf_types::{ComponentId, RfError, StreamId, UnitId, units::QuantityKind};
use rf_unitops::{component_balance::MixerBalancePort, specifications::UnitParameter};

/// Canonical model ports, not user-facing tree paths or numerical indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentBalancePort {
    FeedOutlet,
    Mixer(MixerBalancePort),
}
impl ComponentBalancePort {
    pub const fn name(self) -> &'static str {
        match self {
            Self::FeedOutlet => rf_unitops::FEED_OUTLET_PORT,
            Self::Mixer(port) => port.name(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentFlowBinding {
    pub unit_id: UnitId,
    pub port: ComponentBalancePort,
    pub stream_id: StreamId,
    pub component_id: ComponentId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentFlowVariable {
    pub binding: ComponentFlowBinding,
    pub scale_mol_s: f64,
}
impl ComponentFlowVariable {
    pub const fn quantity(&self) -> QuantityKind {
        QuantityKind::MolarFlow
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentBalanceEquation {
    Connection {
        source: ComponentFlowBinding,
        target: ComponentFlowBinding,
    },
    MixerConservation {
        unit_id: UnitId,
        component_id: ComponentId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentBalanceRow {
    pub equation: ComponentBalanceEquation,
    pub scale_mol_s: f64,
}
impl ComponentBalanceRow {
    pub const fn quantity(&self) -> QuantityKind {
        QuantityKind::MolarFlow
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerBalanceScales {
    pub inlet_a_mol_s: f64,
    pub inlet_b_mol_s: f64,
    pub outlet_mol_s: f64,
}
impl MixerBalanceScales {
    pub const fn for_port(self, port: MixerBalancePort) -> f64 {
        match port {
            MixerBalancePort::InletA => self.inlet_a_mol_s,
            MixerBalancePort::InletB => self.inlet_b_mol_s,
            MixerBalancePort::Outlet => self.outlet_mol_s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixerBalanceSpecification {
    pub mixer_id: UnitId,
    pub scales: MixerBalanceScales,
    /// Fixed feeds already determine this value: extra specifications are rejected.
    pub outlet_total_molar_flow_mol_s: Option<f64>,
}
impl MixerBalanceSpecification {
    pub fn new(mixer_id: impl Into<UnitId>, scales: MixerBalanceScales) -> Self {
        Self {
            mixer_id: mixer_id.into(),
            scales,
            outlet_total_molar_flow_mol_s: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentBalanceEvaluation {
    pub residuals_mol_s: Vec<f64>,
    pub scaled_residuals: Vec<f64>,
}

/// Values of the material subproblem only; no thermal state or run status.
#[derive(Debug, Clone, PartialEq)]
pub struct MixerBalanceValues {
    pub component_molar_flows_mol_s: rf_model::Composition,
    pub total_molar_flow_mol_s: f64,
    pub mole_fractions: rf_model::Composition,
}

#[derive(Debug, PartialEq)]
pub enum ComponentBalanceError {
    Model(RfError),
    UnsupportedModel {
        unit_id: UnitId,
        kind: String,
    },
    UnsupportedTopology {
        mixer_id: UnitId,
    },
    UnsupportedSpecification {
        unit_id: UnitId,
        parameter: UnitParameter,
    },
    UnsupportedThermalBoundary {
        stream_id: StreamId,
    },
    NoComponents,
    UnknownComponent {
        stream_id: StreamId,
        component_id: ComponentId,
    },
    InvalidScale {
        port: MixerBalancePort,
    },
    InvalidOutletSpecification,
    RedundantOutletSpecification {
        mixer_id: UnitId,
        expected_mol_s: f64,
    },
    ConflictingOutletSpecification {
        mixer_id: UnitId,
        expected_mol_s: f64,
        specified_mol_s: f64,
    },
    InvalidOutletTotal {
        mixer_id: UnitId,
    },
    PointSize {
        expected: usize,
        actual: usize,
    },
    InvalidTrialValue {
        binding: ComponentFlowBinding,
        value: f64,
    },
    NonFiniteEquation {
        equation: Box<ComponentBalanceEquation>,
    },
    NonFiniteScaling {
        equation: Box<ComponentBalanceEquation>,
    },
    UnsatisfiedEquation {
        equation: Box<ComponentBalanceEquation>,
        scaled_residual: f64,
        residual_mol_s: f64,
    },
}
impl From<RfError> for ComponentBalanceError {
    fn from(error: RfError) -> Self {
        Self::Model(error)
    }
}
impl fmt::Display for ComponentBalanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(error) => error.fmt(f),
            Self::UnsupportedModel { unit_id, kind } => write!(
                f,
                "unit `{unit_id}` kind `{kind}` has no supported component balance capability"
            ),
            Self::UnsupportedTopology { mixer_id } => write!(
                f,
                "component balance for `{mixer_id}` requires exactly two fixed feeds and one terminal mixer"
            ),
            Self::UnsupportedSpecification { unit_id, parameter } => write!(
                f,
                "unit `{unit_id}` parameter `{}` is outside the fixed material subproblem",
                parameter.key()
            ),
            Self::UnsupportedThermalBoundary { stream_id } => write!(
                f,
                "stream `{stream_id}` must have the same positive finite fixed T/P as the other feed"
            ),
            Self::NoComponents => f.write_str("component balance requires registered components"),
            Self::UnknownComponent {
                stream_id,
                component_id,
            } => write!(
                f,
                "stream `{stream_id}` uses unregistered component `{component_id}`"
            ),
            Self::InvalidScale { port } => write!(
                f,
                "port `{}` requires a positive finite molar flow scale",
                port.name()
            ),
            Self::InvalidOutletSpecification => {
                f.write_str("outlet total specification must be finite and nonnegative")
            }
            Self::RedundantOutletSpecification {
                mixer_id,
                expected_mol_s,
            } => write!(
                f,
                "mixer `{mixer_id}` outlet total {expected_mol_s} mol/s is already fixed by its feeds"
            ),
            Self::ConflictingOutletSpecification {
                mixer_id,
                expected_mol_s,
                specified_mol_s,
            } => write!(
                f,
                "mixer `{mixer_id}` outlet specification {specified_mol_s} mol/s conflicts with feed total {expected_mol_s} mol/s"
            ),
            Self::InvalidOutletTotal { mixer_id } => write!(
                f,
                "mixer `{mixer_id}` requires a positive finite outlet total"
            ),
            Self::PointSize { expected, actual } => write!(
                f,
                "component balance point has {actual} values; expected {expected}"
            ),
            Self::InvalidTrialValue { binding, value } => write!(
                f,
                "component flow at {binding:?} must be finite and nonnegative; received {value}"
            ),
            Self::NonFiniteEquation { equation } => {
                write!(f, "non-finite component residual at {equation:?}")
            }
            Self::NonFiniteScaling { equation } => {
                write!(f, "non-finite component balance scaling at {equation:?}")
            }
            Self::UnsatisfiedEquation {
                equation,
                scaled_residual,
                residual_mol_s,
            } => write!(
                f,
                "component constraint {equation:?} is not satisfied: {residual_mol_s} mol/s, scaled residual {scaled_residual}"
            ),
        }
    }
}
impl std::error::Error for ComponentBalanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Model(error) => Some(error),
            _ => None,
        }
    }
}
