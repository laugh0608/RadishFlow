//! CLI v6 owns EO vocabulary; historical protocol capabilities stay fixed.
pub(super) mod adoption;
mod plan;
mod results;
use super::{
    planned::{PlanProtocol, domain, invalid, required_capabilities},
    v3 as old, v5,
};
use old::{
    protocol::{Failure, IdentityRef, Output},
    references::{Bindings, Executions},
};
use rf_store::project_v4::*;
use rf_ui::{AppState, flow_plan::FlowPlanConfiguration};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    pub lower: f64,
    pub upper: f64,
    pub max_iterations: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Specification {
    ReleaseSplitFraction {
        unit: IdentityRef,
        field: StoredEquationParameter,
        interval: Interval,
    },
    ProductTotalMolarFlow {
        stream: IdentityRef,
        source: old::actions::Port,
        value_mol_s: f64,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Variable {
    PortComponent {
        unit: IdentityRef,
        port: StoredMaterialPort,
        stream: IdentityRef,
        component_id: String,
    },
    Parameter {
        unit: IdentityRef,
        field: StoredEquationParameter,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Equation {
    Unit {
        unit: IdentityRef,
        role: StoredMaterialEquationRole,
        component_id: String,
    },
    Connection {
        stream: IdentityRef,
        source: old::actions::Port,
        sink: old::actions::Port,
        component_id: String,
    },
    ProductTotal {
        stream: IdentityRef,
        source: old::actions::Port,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scale<B> {
    pub binding: B,
    pub scale_si: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scaling {
    EvaluatedFreshTotal,
    ExplicitEquations {
        variables: Vec<Scale<Variable>>,
        equations: Vec<Scale<Equation>>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", deny_unknown_fields)]
pub enum EquationPlan {
    #[serde(rename = "radishflow.fixed-tp-material-eo/1")]
    EquationOriented {
        equation_contract_version: u32,
        acceptance_contract_version: u32,
        temperature_k: f64,
        pressure_pa: f64,
        specifications: Vec<Specification>,
        scaling: Scaling,
    },
}
#[derive(Debug, Clone)]
pub enum Plan {
    Existing(v5::Plan),
    EquationOriented(EquationPlan),
}
impl Serialize for Plan {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Existing(p) => p.serialize(s),
            Self::EquationOriented(p) => p.serialize(s),
        }
    }
}
impl<'de> Deserialize<'de> for Plan {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        if value["method"] == "radishflow.fixed-tp-material-eo/1" {
            serde_json::from_value(value)
                .map(Self::EquationOriented)
                .map_err(serde::de::Error::custom)
        } else {
            serde_json::from_value(value)
                .map(Self::Existing)
                .map_err(serde::de::Error::custom)
        }
    }
}
impl PlanProtocol for Plan {
    const VERSION: u32 = 6;
    fn configuration(p: &Self, refs: &Bindings) -> Result<FlowPlanConfiguration, Failure> {
        plan::configuration(p, refs)
    }
    fn wire(p: &FlowPlanConfiguration) -> Result<Value, Failure> {
        plan::wire(p)
    }
    fn query(app: &AppState) -> Result<Value, Failure> {
        plan::query(app)
    }
    fn flow(app: &AppState, e: &Executions) -> Result<Value, Failure> {
        results::flow(app, e)
    }
}
