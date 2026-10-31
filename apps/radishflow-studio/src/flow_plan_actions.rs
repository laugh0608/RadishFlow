//! Formal plan transactions and explicit project DTO conversion. No solver work or IO.
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
pub fn stored_flow_plan(plan: &FlowPlanConfiguration) -> StoredFlowPlan {
    match plan {
        FlowPlanConfiguration::Acyclic => StoredFlowPlan::Acyclic,
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
                tears: options
                    .tears
                    .iter()
                    .map(|tear| StoredTear {
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
                    })
                    .collect(),
            })
        }
    }
}
pub fn flow_plan_configuration(plan: StoredFlowPlan) -> RfResult<FlowPlanConfiguration> {
    plan.validate()?;
    let value = match plan {
        StoredFlowPlan::Acyclic => FlowPlanConfiguration::Acyclic,
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
                tears: plan
                    .tears
                    .into_iter()
                    .map(|t| TearSpecification {
                        stream_id: t.stream_id,
                        source: DiagnosticPortTarget::new(t.source.unit_id, t.source.port),
                        sink: DiagnosticPortTarget::new(t.sink.unit_id, t.sink.port),
                        initial_component_flows_mol_s: match t.initialization {
                            StoredTearInitialization::ZeroComponentFlows => None,
                            StoredTearInitialization::ComponentFlows { values_mol_s } => Some(
                                values_mol_s
                                    .into_iter()
                                    .map(|v| (v.component_id, v.value))
                                    .collect(),
                            ),
                        },
                        component_scales_mol_s: match t.residual_scaling {
                            StoredResidualScaling::EvaluatedFreshTotal => None,
                            StoredResidualScaling::ComponentValues { values_mol_s } => Some(
                                values_mol_s
                                    .into_iter()
                                    .map(|v| (v.component_id, v.value))
                                    .collect(),
                            ),
                        },
                    })
                    .collect(),
            })
        }
    };
    value.validate_configuration()?;
    Ok(value)
}
