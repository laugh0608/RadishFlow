//! Formal plan transactions and explicit project DTO conversion. No solver work or IO.
mod equation_oriented;
mod mixed;
mod simultaneous;
use crate::flow_actions::FlowEditAccess;
use rf_solver::partitioned::*;
use rf_store::project_v4::*;
use rf_types::{DiagnosticPortTarget, RfError, RfResult};
use rf_ui::{AppState, flow_plan::*};

#[derive(Debug, Clone, PartialEq)]
pub struct FlowPlanActionReceipt {
    pub changed: bool,
    pub before_revision: u64,
    pub revision: u64,
    pub context: FlowPlanEditContext,
}
pub fn execute_flow_plan_action(
    app: &mut AppState,
    access: FlowEditAccess,
    expected: &FlowPlanEditContext,
    command: FlowPlanCommand,
) -> RfResult<FlowPlanActionReceipt> {
    if access != FlowEditAccess::Editable {
        return Err(RfError::invalid_input("project is read-only"));
    }
    let before_revision = app.workspace.flow_plan.revision();
    let changed = app.workspace.apply_flow_plan(expected, command)?;
    app.refresh_run_panel_state();
    Ok(FlowPlanActionReceipt {
        changed,
        before_revision,
        revision: app.workspace.flow_plan.revision(),
        context: FlowPlanEditContext::capture(&app.workspace),
    })
}
pub fn stored_flow_plan(plan: &FlowPlanConfiguration) -> RfResult<StoredFlowPlan> {
    stored_flow_plan_with_support(plan, FlowPlanSupport::Mixed)
}

pub(crate) fn stored_flow_plan_with_support(
    plan: &FlowPlanConfiguration,
    support: FlowPlanSupport,
) -> RfResult<StoredFlowPlan> {
    if let FlowPlanConfiguration::Mixed(options) = plan
        && let Some(recycle) = &options.recycle
    {
        if support == FlowPlanSupport::MixedRecycle {
            return Ok(StoredFlowPlan::MixedRecycle(Box::new(
                StoredMixedRecyclePlan {
                    mixed: mixed::stored(options)?,
                    max_iteration_evaluations: recycle.max_iteration_evaluations,
                    tear: stored_tear(&recycle.tear),
                },
            )));
        }
        return Err(RfError::invalid_input(
            "the project consumer does not support mixed cross-block recycle",
        )
        .with_diagnostic_code("workspace.plan.in_memory_only"));
    }
    Ok(match plan {
        FlowPlanConfiguration::Acyclic => StoredFlowPlan::Acyclic,
        FlowPlanConfiguration::Mixed(o) => StoredFlowPlan::Mixed(mixed::stored(o)?),
        FlowPlanConfiguration::EquationOriented(o) => {
            StoredFlowPlan::EquationOriented(equation_oriented::stored(o)?)
        }
        FlowPlanConfiguration::Simultaneous(options) => {
            StoredFlowPlan::Simultaneous(simultaneous::stored(options))
        }
        FlowPlanConfiguration::Partitioned(options) => {
            StoredFlowPlan::Partitioned(StoredRecyclePlan {
                fixed_boundary: StoredFixedBoundary {
                    temperature_k: options.boundary.temperature_k,
                    pressure_pa: options.boundary.pressure_pa,
                },
                max_iteration_evaluations: options.max_iteration_evaluations,
                tears: options.tears.iter().map(stored_tear).collect(),
            })
        }
    })
}

fn stored_tear(tear: &TearSpecification) -> StoredTear {
    StoredTear {
        stream_id: tear.stream_id.clone(),
        source: StoredPortBinding {
            unit_id: tear.source.unit_id.clone(),
            port: tear.source.port_name.clone(),
        },
        sink: StoredPortBinding {
            unit_id: tear.sink.unit_id.clone(),
            port: tear.sink.port_name.clone(),
        },
        initialization: match &tear.initial_component_flows_mol_s {
            None => StoredTearInitialization::ZeroComponentFlows,
            Some(values) => StoredTearInitialization::ComponentFlows {
                values_mol_s: values
                    .iter()
                    .map(|(id, &value)| StoredComponentValue {
                        component_id: id.clone(),
                        value,
                    })
                    .collect(),
            },
        },
        residual_scaling: match &tear.component_scales_mol_s {
            None => StoredResidualScaling::EvaluatedFreshTotal,
            Some(values) => StoredResidualScaling::ComponentValues {
                values_mol_s: values
                    .iter()
                    .map(|(id, &value)| StoredComponentValue {
                        component_id: id.clone(),
                        value,
                    })
                    .collect(),
            },
        },
    }
}

pub(crate) fn runtime_configuration_sha256(plan: &FlowPlanConfiguration) -> String {
    let bytes = match plan {
        FlowPlanConfiguration::Mixed(options) if let Some(recycle) = &options.recycle => {
            // An execution fingerprint, independent of the project format.
            serde_json::to_vec(&(
                options.method(),
                mixed::stored(options).expect("validated workspace block scales"),
                stored_tear(&recycle.tear),
                recycle.max_iteration_evaluations,
            ))
        }
        _ => serde_json::to_vec(&stored_flow_plan(plan).expect("persistable plan")),
    }
    .expect("validated frozen plan is serializable");
    rf_store::sha256_hex(&bytes)
}

pub fn flow_plan_configuration(plan: StoredFlowPlan) -> RfResult<FlowPlanConfiguration> {
    plan.validate()?;
    let value = match plan {
        StoredFlowPlan::Acyclic => FlowPlanConfiguration::Acyclic,
        StoredFlowPlan::Mixed(o) => FlowPlanConfiguration::Mixed(mixed::configuration(o)?),
        StoredFlowPlan::MixedRecycle(plan) => {
            let mut options = mixed::configuration(plan.mixed)?;
            options.recycle = Some(Box::new(rf_solver::mixed::MixedRecycleOptions {
                tear: tear_configuration(plan.tear),
                max_iteration_evaluations: plan.max_iteration_evaluations,
            }));
            FlowPlanConfiguration::Mixed(options)
        }
        StoredFlowPlan::EquationOriented(p) => {
            FlowPlanConfiguration::EquationOriented(equation_oriented::configuration(p)?)
        }
        StoredFlowPlan::Simultaneous(plan) => {
            FlowPlanConfiguration::Simultaneous(simultaneous::configuration(plan))
        }
        StoredFlowPlan::Partitioned(plan) => {
            FlowPlanConfiguration::Partitioned(RecycleSolveOptions {
                boundary: FixedRecycleBoundary {
                    temperature_k: plan.fixed_boundary.temperature_k,
                    pressure_pa: plan.fixed_boundary.pressure_pa,
                },
                max_iteration_evaluations: plan.max_iteration_evaluations,
                tears: plan.tears.into_iter().map(tear_configuration).collect(),
            })
        }
    };
    value.validate_configuration()?;
    Ok(value)
}

fn tear_configuration(tear: StoredTear) -> TearSpecification {
    TearSpecification {
        stream_id: tear.stream_id,
        source: DiagnosticPortTarget::new(tear.source.unit_id, tear.source.port),
        sink: DiagnosticPortTarget::new(tear.sink.unit_id, tear.sink.port),
        initial_component_flows_mol_s: match tear.initialization {
            StoredTearInitialization::ZeroComponentFlows => None,
            StoredTearInitialization::ComponentFlows { values_mol_s } => Some(
                values_mol_s
                    .into_iter()
                    .map(|v| (v.component_id, v.value))
                    .collect(),
            ),
        },
        component_scales_mol_s: match tear.residual_scaling {
            StoredResidualScaling::EvaluatedFreshTotal => None,
            StoredResidualScaling::ComponentValues { values_mol_s } => Some(
                values_mol_s
                    .into_iter()
                    .map(|v| (v.component_id, v.value))
                    .collect(),
            ),
        },
    }
}
