use super::{protocol::*, *};
use rf_solver::partitioned::*;
use rf_types::DiagnosticPortTarget;
use rf_ui::flow_plan::FlowPlanConfiguration;
use std::collections::{BTreeMap, BTreeSet};
fn values(values: &[ComponentValue]) -> Result<BTreeMap<rf_types::ComponentId, f64>, Failure> {
    let mut seen = BTreeSet::new();
    values
        .iter()
        .map(|v| {
            if v.component_id.trim().is_empty()
                || !seen.insert(&v.component_id)
                || !v.value_mol_s.is_finite()
            {
                return Err(invalid("invalid or duplicate component vector entry"));
            }
            Ok((v.component_id.clone().into(), v.value_mol_s))
        })
        .collect()
}
pub fn configuration(plan: &Plan, refs: &Bindings) -> Result<FlowPlanConfiguration, Failure> {
    let result = match plan {
        Plan::Acyclic {} => FlowPlanConfiguration::Acyclic,
        Plan::Partitioned {
            acceptance_contract_version,
            temperature_k,
            pressure_pa,
            max_iteration_evaluations,
            tears,
        } => {
            if *acceptance_contract_version != 1 {
                return Err(Failure::new(
                    Stage::Step,
                    Code::UnsupportedCapability,
                    "unsupported recycle acceptance contract",
                ));
            }
            FlowPlanConfiguration::Partitioned(RecycleSolveOptions {
                boundary: FixedRecycleBoundary {
                    temperature_k: *temperature_k,
                    pressure_pa: *pressure_pa,
                },
                max_iteration_evaluations: *max_iteration_evaluations,
                tears: tears
                    .iter()
                    .map(|t| {
                        Ok(TearSpecification {
                            stream_id: refs.resolve(&t.stream, Output::Stream)?.into(),
                            source: DiagnosticPortTarget::new(
                                refs.resolve(&t.source.unit, Output::Unit)?,
                                t.source.port.clone(),
                            ),
                            sink: DiagnosticPortTarget::new(
                                refs.resolve(&t.sink.unit, Output::Unit)?,
                                t.sink.port.clone(),
                            ),
                            initial_component_flows_mol_s: match &t.initialization {
                                Initialization::ZeroComponentFlows => None,
                                Initialization::ComponentFlows { values: v } => Some(values(v)?),
                            },
                            component_scales_mol_s: match &t.residual_scaling {
                                Scaling::EvaluatedFreshTotal => None,
                                Scaling::ComponentValues { values: v } => Some(values(v)?),
                            },
                        })
                    })
                    .collect::<Result<_, Failure>>()?,
            })
        }
    };
    result.validate_configuration().map_err(domain)?;
    Ok(result)
}
pub fn wire(plan: &FlowPlanConfiguration) -> Plan {
    match plan {
        FlowPlanConfiguration::Acyclic => Plan::Acyclic {},
        FlowPlanConfiguration::Partitioned(o) => Plan::Partitioned {
            acceptance_contract_version: 1,
            temperature_k: o.boundary.temperature_k,
            pressure_pa: o.boundary.pressure_pa,
            max_iteration_evaluations: o.max_iteration_evaluations,
            tears: o
                .tears
                .iter()
                .map(|t| Tear {
                    stream: IdentityRef::id(&t.stream_id),
                    source: old::actions::Port {
                        unit: IdentityRef::id(&t.source.unit_id),
                        port: t.source.port_name.clone(),
                    },
                    sink: old::actions::Port {
                        unit: IdentityRef::id(&t.sink.unit_id),
                        port: t.sink.port_name.clone(),
                    },
                    initialization: t
                        .initial_component_flows_mol_s
                        .as_ref()
                        .map(|v| Initialization::ComponentFlows { values: vector(v) })
                        .unwrap_or(Initialization::ZeroComponentFlows),
                    residual_scaling: t
                        .component_scales_mol_s
                        .as_ref()
                        .map(|v| Scaling::ComponentValues { values: vector(v) })
                        .unwrap_or(Scaling::EvaluatedFreshTotal),
                })
                .collect(),
        },
    }
}
fn vector(v: &BTreeMap<rf_types::ComponentId, f64>) -> Vec<ComponentValue> {
    v.iter()
        .map(|(id, value)| ComponentValue {
            component_id: id.to_string(),
            value_mol_s: *value,
        })
        .collect()
}
pub fn query(app: &rf_ui::AppState) -> Value {
    let w = &app.workspace;
    let ready = w.flow_plan.current().validate_for(w);
    json!({"plan_revision":w.flow_plan.revision(),"configuration":wire(w.flow_plan.current()),"ready":ready.is_ok(),"not_ready":ready.err().map(|e|json!({"code":e.context().diagnostic_code(),"message":e.to_string()})),"simulation_dirty":w.flow_plan.is_dirty(),"methods":["radishflow.acyclic-sm/1","radishflow.partitioned-sm/1"],"in_memory_required_project_schema_version":crate::project_save_version(app)})
}
