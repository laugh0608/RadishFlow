use super::*;
use rf_solver::{
    flow_plan::{SinglePhaseOptions, SinglePhaseRecycleConfiguration},
    single_phase_recycle::{
        SinglePhaseRecycleOptions, SinglePhaseTearBinding, SinglePhaseTearState,
    },
};

pub(super) fn stored_properties(options: &SinglePhaseOptions) -> StoredSinglePhasePlan {
    let (min_temperature_k, max_temperature_k) = options.model.temperature_range_k().into_inner();
    StoredSinglePhasePlan {
        min_temperature_k,
        max_temperature_k,
        pressure_pa: options.model.pressure_pa(),
        heat_capacities_j_per_mol_k: options.model.heat_capacities_j_per_mol_k().clone(),
        heat_duties_w: options.heat_duties_w.clone(),
    }
}

pub(super) fn stored_recycle(
    options: &SinglePhaseRecycleConfiguration,
) -> StoredSinglePhaseRecyclePlan {
    let recycle = &options.recycle;
    let port = |target: &DiagnosticPortTarget| StoredPortBinding {
        unit_id: target.unit_id.clone(),
        port: target.port_name.clone(),
    };
    StoredSinglePhaseRecyclePlan {
        single_phase: stored_properties(&options.properties),
        tear: StoredSinglePhaseTear {
            stream_id: recycle.tear.stream_id.clone(),
            source: port(&recycle.tear.source),
            sink: port(&recycle.tear.sink),
            initialization: match &recycle.initial_state {
                None => StoredSinglePhaseTearInitialization::ZeroFlowsAndEnthalpy,
                Some(state) => StoredSinglePhaseTearInitialization::ComponentFlowsAndEnthalpy {
                    component_flows_mol_s: state.component_flows_mol_s.clone(),
                    enthalpy_flow_w: state.enthalpy_flow_w,
                },
            },
        },
        max_iteration_evaluations: recycle.max_iteration_evaluations,
    }
}

pub(super) fn properties(plan: StoredSinglePhasePlan) -> RfResult<SinglePhaseOptions> {
    SinglePhaseOptions::new(
        plan.heat_capacities_j_per_mol_k,
        plan.min_temperature_k..=plan.max_temperature_k,
        plan.pressure_pa,
        plan.heat_duties_w,
    )
}

pub(super) fn recycle(
    plan: StoredSinglePhaseRecyclePlan,
) -> RfResult<SinglePhaseRecycleConfiguration> {
    let tear = plan.tear;
    let port =
        |binding: StoredPortBinding| DiagnosticPortTarget::new(binding.unit_id, binding.port);
    Ok(SinglePhaseRecycleConfiguration {
        properties: properties(plan.single_phase)?,
        recycle: SinglePhaseRecycleOptions {
            tear: SinglePhaseTearBinding {
                stream_id: tear.stream_id,
                source: port(tear.source),
                sink: port(tear.sink),
            },
            initial_state: match tear.initialization {
                StoredSinglePhaseTearInitialization::ZeroFlowsAndEnthalpy => None,
                StoredSinglePhaseTearInitialization::ComponentFlowsAndEnthalpy {
                    component_flows_mol_s,
                    enthalpy_flow_w,
                } => Some(SinglePhaseTearState {
                    component_flows_mol_s,
                    enthalpy_flow_w,
                }),
            },
            max_iteration_evaluations: plan.max_iteration_evaluations,
        },
    })
}
