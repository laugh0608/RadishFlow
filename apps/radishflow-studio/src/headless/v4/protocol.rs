//! Strict v4 increments, kept separate from historical DTO vocabularies.
use super::super::v3::{
    actions::{self as old, Port, RevisionRef},
    protocol::{self as common, *},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum PlanRevision {
    Number(u64),
    Step { step: String, output: PlanOutput },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanOutput {
    Plan,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentValue {
    pub component_id: String,
    pub value_mol_s: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Initialization {
    ZeroComponentFlows,
    ComponentFlows { values: Vec<ComponentValue> },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scaling {
    EvaluatedFreshTotal,
    ComponentValues { values: Vec<ComponentValue> },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tear {
    pub stream: IdentityRef,
    pub source: Port,
    pub sink: Port,
    pub initialization: Initialization,
    pub residual_scaling: Scaling,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", deny_unknown_fields)]
pub enum Plan {
    #[serde(rename = "radishflow.acyclic-sm/1")]
    Acyclic {},
    #[serde(rename = "radishflow.partitioned-sm/1")]
    Partitioned {
        acceptance_contract_version: u32,
        temperature_k: f64,
        pressure_pa: f64,
        max_iteration_evaluations: usize,
        tears: Vec<Tear>,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitKind {
    Feed,
    Mixer,
    Heater,
    Cooler,
    Valve,
    FlashDrum,
    Splitter,
}
impl From<UnitKind> for rf_ui::BuiltinUnitKind {
    fn from(v: UnitKind) -> Self {
        match v {
            UnitKind::Feed => Self::Feed,
            UnitKind::Mixer => Self::Mixer,
            UnitKind::Heater => Self::Heater,
            UnitKind::Cooler => Self::Cooler,
            UnitKind::Valve => Self::Valve,
            UnitKind::FlashDrum => Self::FlashDrum,
            UnitKind::Splitter => Self::Splitter,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSection {
    Inputs,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FractionField {
    OutletASplitFraction,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FractionTarget {
    pub object: ObjectRef,
    pub section: InputSection,
    pub field: FractionField,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NewAction {
    ConfigureFlowPlan {
        expected_plan_revision: PlanRevision,
        plan: Plan,
    },
    RunFlow {
        expected_revision: RevisionRef,
        expected_plan_revision: PlanRevision,
    },
    CreateUnit {
        expected_revision: RevisionRef,
        unit_kind: UnitKind,
    },
    Write {
        expected_revision: RevisionRef,
        target: FractionTarget,
        value: f64,
    },
    ConnectExact {
        expected_revision: RevisionRef,
        source: Port,
        sink: Port,
        stream: IdentityRef,
    },
}
#[derive(Debug, Clone)]
pub enum Action {
    New(NewAction),
    Legacy(old::Action),
}
impl Serialize for Action {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::New(a) => a.serialize(s),
            Self::Legacy(a) => a.serialize(s),
        }
    }
}
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        let kind = value
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::custom("action requires kind"))?;
        let new = matches!(
            kind,
            "configure_flow_plan" | "run_flow" | "create_unit" | "connect_exact"
        ) || (kind == "write"
            && value.pointer("/target/field").and_then(Value::as_str)
                == Some("outlet_a_split_fraction"));
        if new {
            serde_json::from_value(value)
                .map(Self::New)
                .map_err(serde::de::Error::custom)
        } else {
            serde_json::from_value(value)
                .map(Self::Legacy)
                .map_err(serde::de::Error::custom)
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub action: Action,
    #[serde(
        default,
        deserialize_with = "super::super::v3::strict::optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub confirmation: Option<old::Confirmation>,
}
impl super::super::engine::StepIdentity for Step {
    fn id(&self) -> &str {
        &self.id
    }
    fn target(&self) -> Option<ObjectRef> {
        match &self.action {
            Action::Legacy(a) => a.target(),
            Action::New(NewAction::Write { target, .. }) => Some(target.object.clone()),
            Action::New(NewAction::ConnectExact { source, .. }) => {
                Some(ObjectRef::Unit(source.unit.clone()))
            }
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanScope {
    FlowPlan,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlanQuery {
    Actions { scope: PlanScope },
    Results { scope: PlanScope },
}
#[derive(Debug, Clone)]
pub enum QueryKind {
    Plan(PlanQuery),
    Legacy(common::QueryKind),
}
impl<'de> Deserialize<'de> for QueryKind {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        if value.get("scope").and_then(Value::as_str) == Some("flow_plan") {
            serde_json::from_value(value)
                .map(Self::Plan)
                .map_err(serde::de::Error::custom)
        } else {
            serde_json::from_value(value)
                .map(Self::Legacy)
                .map_err(serde::de::Error::custom)
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub id: String,
    pub query: QueryKind,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryRequest {
    pub schema_version: u32,
    pub project: PathBuf,
    pub queries: Vec<Query>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecuteRequest {
    pub schema_version: u32,
    pub project: PathBuf,
    pub expected_project: old::ExpectedProject,
    #[serde(default, deserialize_with = "super::super::v3::strict::optional")]
    pub flow_resources: Option<old::FlowResources>,
    pub steps: Vec<Step>,
    pub queries: Vec<Query>,
}
