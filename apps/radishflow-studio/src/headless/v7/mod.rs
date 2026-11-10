//! Mixed-plan vocabulary is isolated from v1-v6 consumers.
mod plan;
mod results;
use super::{
    planned::{PlanProtocol, domain, invalid, required_capabilities},
    v3 as old, v6,
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
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BlockEquation {
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
    Boundary {
        stream: IdentityRef,
        source: old::actions::Port,
        sink: old::actions::Port,
        component_id: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scaling {
    EvaluatedFreshTotal,
    ExplicitEquations {
        variables: Vec<v6::Scale<v6::Variable>>,
        equations: Vec<v6::Scale<BlockEquation>>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BlockMethod {
    Local,
    EquationOriented {
        equation_contract_version: u32,
        scaling: Scaling,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    pub id: String,
    pub members: Vec<IdentityRef>,
    pub method: BlockMethod,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", deny_unknown_fields)]
pub enum MixedPlan {
    #[serde(rename = "radishflow.fixed-tp-mixed-material/1")]
    Mixed {
        acceptance_contract_version: u32,
        temperature_k: f64,
        pressure_pa: f64,
        blocks: Vec<Block>,
    },
}
#[derive(Debug, Clone)]
pub enum Plan {
    Existing(v6::Plan),
    Mixed(MixedPlan),
}
impl Serialize for Plan {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Existing(p) => p.serialize(s),
            Self::Mixed(p) => p.serialize(s),
        }
    }
}
impl<'de> Deserialize<'de> for Plan {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(d)?;
        if v["method"] == "radishflow.fixed-tp-mixed-material/1" {
            serde_json::from_value(v)
                .map(Self::Mixed)
                .map_err(serde::de::Error::custom)
        } else {
            serde_json::from_value(v)
                .map(Self::Existing)
                .map_err(serde::de::Error::custom)
        }
    }
}
impl PlanProtocol for Plan {
    const VERSION: u32 = 7;
    fn configuration(p: &Self, r: &Bindings) -> Result<FlowPlanConfiguration, Failure> {
        plan::configuration(p, r)
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
