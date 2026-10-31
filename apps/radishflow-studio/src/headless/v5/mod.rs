//! CLI v5 owns the new method vocabulary; stable envelopes and execution are shared.
mod plan;
mod results;
use super::{planned::*, v3 as old, v4};
use old::{
    protocol::{Code, Failure, IdentityRef, Output, Stage},
    references::{Bindings, Executions},
};
use rf_ui::{AppState, flow_plan::FlowPlanConfiguration};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CouplingScale {
    pub stream: IdentityRef,
    pub source: old::actions::Port,
    pub sink: old::actions::Port,
    pub component_id: String,
    pub variable_mol_s: f64,
    pub residual_mol_s: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scaling {
    EvaluatedFreshTotal,
    ExplicitCouplings { values: Vec<CouplingScale> },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", deny_unknown_fields)]
pub enum ModularPlan {
    #[serde(rename = "radishflow.fixed-tp-affine-modular-lu/1")]
    Simultaneous {
        acceptance_contract_version: u32,
        response_contract_version: u32,
        temperature_k: f64,
        pressure_pa: f64,
        scaling: Scaling,
    },
}
#[derive(Debug, Clone)]
pub enum Plan {
    Existing(v4::protocol::Plan),
    Simultaneous(ModularPlan),
}
impl Serialize for Plan {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Existing(p) => p.serialize(s),
            Self::Simultaneous(p) => p.serialize(s),
        }
    }
}
impl<'de> Deserialize<'de> for Plan {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        if v["method"] == "radishflow.fixed-tp-affine-modular-lu/1" {
            serde_json::from_value(v)
                .map(Self::Simultaneous)
                .map_err(serde::de::Error::custom)
        } else {
            serde_json::from_value(v)
                .map(Self::Existing)
                .map_err(serde::de::Error::custom)
        }
    }
}
impl PlanProtocol for Plan {
    const VERSION: u32 = 5;
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
