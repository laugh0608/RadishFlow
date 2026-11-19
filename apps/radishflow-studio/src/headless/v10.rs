//! EO composition vocabulary for the planned CLI, isolated from legacy requests.
use super::{
    planned::{PlanProtocol, domain},
    v3 as old, v6, v7, v9,
};
use old::{
    protocol::{Failure, IdentityRef, Output},
    references::{Bindings, Executions},
};
use rf_store::project_v4::{
    FlowPlanSupport, StoredEquationBinding, StoredEquationSpecification, StoredFlowPlan,
};
use rf_ui::{AppState, flow_plan::FlowPlanConfiguration};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Value, json};
use v6::plan as eo;

pub(super) const VERSION: u32 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum CompositionSpecification {
    ProductMoleFraction {
        stream: IdentityRef,
        source: old::actions::Port,
        component_id: String,
        value_mol_mol: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum CompositionEquation {
    ProductMoleFraction {
        stream: IdentityRef,
        source: old::actions::Port,
        component_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum Specification {
    Composition(CompositionSpecification),
    Existing(v6::Specification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum Equation {
    Composition(CompositionEquation),
    Existing(v6::Equation),
}

#[derive(Debug, Clone)]
pub(super) enum Plan {
    EquationOriented(v6::EquationPlan<Specification, Equation>),
    Existing(v7::Plan),
}

impl Serialize for Plan {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Existing(plan) => plan.serialize(serializer),
            Self::EquationOriented(plan) => plan.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Plan {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value["method"] == v6::METHOD {
            serde_json::from_value(value).map(Self::EquationOriented)
        } else {
            serde_json::from_value(value).map(Self::Existing)
        }
        .map_err(serde::de::Error::custom)
    }
}

impl PlanProtocol for Plan {
    const VERSION: u32 = VERSION;

    fn configuration(plan: &Self, refs: &Bindings) -> Result<FlowPlanConfiguration, Failure> {
        match plan {
            Self::Existing(plan) => v7::Plan::configuration(plan, refs),
            Self::EquationOriented(plan) => eo::equation_configuration(
                plan,
                refs,
                FlowPlanSupport::ProductComposition,
                stored_specification,
                stored_equation,
            ),
        }
    }

    fn wire(plan: &FlowPlanConfiguration) -> Result<Value, Failure> {
        match plan {
            FlowPlanConfiguration::EquationOriented(_) => {
                let StoredFlowPlan::EquationOriented(stored) =
                    crate::flow_plan_actions::stored_flow_plan_with_support(
                        plan,
                        FlowPlanSupport::ProductComposition,
                    )
                    .map_err(domain)?
                else {
                    unreachable!("EO configuration conversion");
                };
                eo::wire_equation_plan(stored, wire_specification, wire_equation)
            }
            FlowPlanConfiguration::Mixed(o) if o.recycle.is_some() => v9::results::wire(plan),
            _ => v7::Plan::wire(plan),
        }
    }

    fn query(app: &AppState) -> Result<Value, Failure> {
        let plan = app.workspace.flow_plan.current();
        match plan {
            FlowPlanConfiguration::EquationOriented(_) => {
                let mut value = eo::equation_query(app, Self::wire)?;
                value["configuration_sha256"] =
                    json!(crate::flow_plan_actions::runtime_configuration_sha256(plan));
                value["equation_capability"] = json!({
                    "models": ["feed", "mixer", "splitter"],
                    "fixed_temperature_pressure": true,
                    "max_variables": rf_solver::equation_oriented::MAX_DIMENSION,
                    "max_equations": rf_solver::equation_oriented::MAX_DIMENSION,
                    "max_released_split_fractions": 1,
                    "targets": ["terminal_product_total_molar_flow", "terminal_product_mole_fraction"],
                    "forward_backend": "direct_lu",
                    "inverse_backend": "conditional_lu_bisection",
                    "energy_or_phase_equilibrium": false,
                });
                Ok(value)
            }
            FlowPlanConfiguration::Mixed(o) if o.recycle.is_some() => v9::results::plan(app),
            _ => v7::Plan::query(app),
        }
    }

    fn flow(app: &AppState, executions: &Executions) -> Result<Value, Failure> {
        match app.workspace.flow_plan.current() {
            FlowPlanConfiguration::Mixed(o) if o.recycle.is_some() => {
                v9::results::flow(app, executions)
            }
            _ => v7::Plan::flow(app, executions),
        }
    }
}

fn stored_specification(
    spec: &Specification,
    refs: &Bindings,
) -> Result<StoredEquationSpecification, Failure> {
    match spec {
        Specification::Existing(spec) => eo::stored_specification(spec, refs),
        Specification::Composition(CompositionSpecification::ProductMoleFraction {
            stream,
            source,
            component_id,
            value_mol_mol,
        }) => Ok(StoredEquationSpecification::ProductMoleFraction {
            stream_id: refs.resolve(stream, Output::Stream)?.into(),
            source: eo::stored_port(source, refs)?,
            component_id: component_id.clone().into(),
            value_mol_mol: *value_mol_mol,
        }),
    }
}

fn stored_equation(row: &Equation, refs: &Bindings) -> Result<StoredEquationBinding, Failure> {
    match row {
        Equation::Existing(row) => eo::stored_equation(row, refs),
        Equation::Composition(CompositionEquation::ProductMoleFraction {
            stream,
            source,
            component_id,
        }) => Ok(StoredEquationBinding::ProductMoleFraction {
            stream_id: refs.resolve(stream, Output::Stream)?.into(),
            source: eo::stored_port(source, refs)?,
            component_id: component_id.clone().into(),
        }),
    }
}

fn wire_specification(spec: StoredEquationSpecification) -> Result<Specification, Failure> {
    match spec {
        StoredEquationSpecification::ProductMoleFraction {
            stream_id,
            source,
            component_id,
            value_mol_mol,
        } => Ok(Specification::Composition(
            CompositionSpecification::ProductMoleFraction {
                stream: IdentityRef::id(&stream_id),
                source: eo::wire_port(&source),
                component_id: component_id.into_inner(),
                value_mol_mol,
            },
        )),
        existing => eo::wire_specification(existing).map(Specification::Existing),
    }
}

fn wire_equation(row: StoredEquationBinding) -> Result<Equation, Failure> {
    match row {
        StoredEquationBinding::ProductMoleFraction {
            stream_id,
            source,
            component_id,
        } => Ok(Equation::Composition(
            CompositionEquation::ProductMoleFraction {
                stream: IdentityRef::id(&stream_id),
                source: eo::wire_port(&source),
                component_id: component_id.into_inner(),
            },
        )),
        existing => eo::wire_equation(existing).map(Equation::Existing),
    }
}
