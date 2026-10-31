//! Persisted EO declarations only; no numerical workspace, candidate or run token.
use super::{StoredFixedBoundary, StoredPortBinding};
use rf_types::{ComponentId, RfError, RfResult, StreamId, UnitId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const EQUATION_ORIENTED_METHOD: &str = "radishflow.fixed-tp-material-eo";
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredMaterialPort {
    Inlet,
    InletA,
    InletB,
    Outlet,
    OutletA,
    OutletB,
}
impl StoredMaterialPort {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inlet => "inlet",
            Self::InletA => "inlet_a",
            Self::InletB => "inlet_b",
            Self::Outlet => "outlet",
            Self::OutletA => "outlet_a",
            Self::OutletB => "outlet_b",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredEquationParameter {
    OutletASplitFraction,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredMaterialEquationRole {
    FeedBoundary,
    MixerBalance,
    SplitA,
    SplitB,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StoredEquationVariableBinding {
    PortComponent {
        unit_id: UnitId,
        port: StoredMaterialPort,
        stream_id: StreamId,
        component_id: ComponentId,
    },
    Parameter {
        unit_id: UnitId,
        field: StoredEquationParameter,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StoredEquationBinding {
    Unit {
        unit_id: UnitId,
        role: StoredMaterialEquationRole,
        component_id: ComponentId,
    },
    Connection {
        stream_id: StreamId,
        source: StoredPortBinding,
        sink: StoredPortBinding,
        component_id: ComponentId,
    },
    ProductTotal {
        stream_id: StreamId,
        source: StoredPortBinding,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredEquationScale<B> {
    pub binding: B,
    pub scale_si: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredEquationScaling {
    EvaluatedFreshTotal,
    ExplicitEquations {
        variables: Vec<StoredEquationScale<StoredEquationVariableBinding>>,
        equations: Vec<StoredEquationScale<StoredEquationBinding>>,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredFractionInterval {
    pub lower: f64,
    pub upper: f64,
    pub max_iterations: usize,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StoredEquationSpecification {
    ReleaseSplitFraction {
        unit_id: UnitId,
        field: StoredEquationParameter,
        interval: StoredFractionInterval,
    },
    ProductTotalMolarFlow {
        stream_id: StreamId,
        source: StoredPortBinding,
        value_mol_s: f64,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub struct StoredEquationPlan {
    pub fixed_boundary: StoredFixedBoundary,
    pub specifications: Vec<StoredEquationSpecification>,
    pub scaling: StoredEquationScaling,
}
fn nonblank(values: &[&str]) -> bool {
    values.iter().all(|s| !s.trim().is_empty())
}
fn port_valid(p: &StoredPortBinding) -> bool {
    nonblank(&[p.unit_id.as_str()])
        && [
            "inlet", "inlet_a", "inlet_b", "outlet", "outlet_a", "outlet_b",
        ]
        .contains(&p.port.as_str())
}
impl StoredEquationVariableBinding {
    fn valid(&self) -> bool {
        match self {
            Self::PortComponent {
                unit_id,
                stream_id,
                component_id,
                ..
            } => nonblank(&[unit_id.as_str(), stream_id.as_str(), component_id.as_str()]),
            Self::Parameter { unit_id, .. } => nonblank(&[unit_id.as_str()]),
        }
    }
}
impl StoredEquationBinding {
    fn valid(&self) -> bool {
        match self {
            Self::Unit {
                unit_id,
                component_id,
                ..
            } => nonblank(&[unit_id.as_str(), component_id.as_str()]),
            Self::Connection {
                stream_id,
                source,
                sink,
                component_id,
            } => {
                nonblank(&[stream_id.as_str(), component_id.as_str()])
                    && port_valid(source)
                    && port_valid(sink)
            }
            Self::ProductTotal { stream_id, source } => {
                nonblank(&[stream_id.as_str()]) && port_valid(source)
            }
        }
    }
}
impl StoredEquationPlan {
    pub fn validate(&self) -> RfResult<()> {
        let invalid = || {
            RfError::invalid_input(
                "simulation: invalid EO shape, identity, interval, target or scale",
            )
        };
        let b = &self.fixed_boundary;
        if !b.temperature_k.is_finite()
            || b.temperature_k <= 0.0
            || !b.pressure_pa.is_finite()
            || b.pressure_pa <= 0.0
        {
            return Err(invalid());
        }
        let (mut releases, mut targets) = (0, 0);
        for spec in &self.specifications {
            let valid = match spec {
                StoredEquationSpecification::ReleaseSplitFraction {
                    unit_id, interval, ..
                } => {
                    releases += 1;
                    nonblank(&[unit_id.as_str()])
                        && interval.lower.is_finite()
                        && interval.upper.is_finite()
                        && interval.lower >= 0.0
                        && interval.upper <= 1.0
                        && interval.lower < interval.upper
                        && (1..=128).contains(&interval.max_iterations)
                }
                StoredEquationSpecification::ProductTotalMolarFlow {
                    stream_id,
                    source,
                    value_mol_s,
                } => {
                    targets += 1;
                    nonblank(&[stream_id.as_str()])
                        && port_valid(source)
                        && value_mol_s.is_finite()
                        && *value_mol_s > 0.0
                }
            };
            if !valid || releases > 1 || targets > 1 {
                return Err(invalid());
            }
        }
        if let StoredEquationScaling::ExplicitEquations {
            variables,
            equations,
        } = &self.scaling
        {
            let mut seen = BTreeSet::new();
            for v in variables {
                if !v.binding.valid()
                    || !v.scale_si.is_finite()
                    || v.scale_si <= 0.0
                    || !seen.insert(&v.binding)
                {
                    return Err(invalid());
                }
            }
            let mut seen = BTreeSet::new();
            for v in equations {
                if !v.binding.valid()
                    || !v.scale_si.is_finite()
                    || v.scale_si <= 0.0
                    || !seen.insert(&v.binding)
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
    pub fn canonicalized(&self) -> Self {
        let mut plan = self.clone();
        plan.specifications.sort_by_key(|s| match s {
            StoredEquationSpecification::ReleaseSplitFraction { .. } => 0,
            StoredEquationSpecification::ProductTotalMolarFlow { .. } => 1,
        });
        if let StoredEquationScaling::ExplicitEquations {
            variables,
            equations,
        } = &mut plan.scaling
        {
            variables.sort_by(|a, b| a.binding.cmp(&b.binding));
            equations.sort_by(|a, b| a.binding.cmp(&b.binding));
        }
        plan
    }
}
