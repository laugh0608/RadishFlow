use rf_solver::simultaneous::{CouplingBinding, CouplingScale, FixedBoundary, ModularOptions};
use rf_store::project_v4::{
    StoredCouplingScale, StoredCouplingScaling, StoredFixedBoundary, StoredModularPlan,
    StoredPortBinding,
};
use rf_types::DiagnosticPortTarget;

pub(super) fn stored(options: &ModularOptions) -> StoredModularPlan {
    StoredModularPlan {
        fixed_boundary: StoredFixedBoundary {
            temperature_k: options.boundary.temperature_k,
            pressure_pa: options.boundary.pressure_pa,
        },
        scaling: match &options.scales {
            None => StoredCouplingScaling::EvaluatedFreshTotal,
            Some(scales) => StoredCouplingScaling::ExplicitCouplings {
                values: scales
                    .iter()
                    .map(|(b, s)| StoredCouplingScale {
                        stream_id: b.stream.clone(),
                        source: StoredPortBinding {
                            unit_id: b.source.unit_id.clone(),
                            port: b.source.port_name.clone(),
                        },
                        sink: StoredPortBinding {
                            unit_id: b.sink.unit_id.clone(),
                            port: b.sink.port_name.clone(),
                        },
                        component_id: b.component.clone(),
                        variable_mol_s: s.variable_mol_s,
                        residual_mol_s: s.residual_mol_s,
                    })
                    .collect(),
            },
        },
    }
}

/// The caller validates the wire DTO before collecting, including duplicate bindings.
pub(super) fn configuration(plan: StoredModularPlan) -> ModularOptions {
    ModularOptions {
        boundary: FixedBoundary {
            temperature_k: plan.fixed_boundary.temperature_k,
            pressure_pa: plan.fixed_boundary.pressure_pa,
        },
        scales: match plan.scaling {
            StoredCouplingScaling::EvaluatedFreshTotal => None,
            StoredCouplingScaling::ExplicitCouplings { values } => Some(
                values
                    .into_iter()
                    .map(|v| {
                        (
                            CouplingBinding {
                                stream: v.stream_id,
                                source: DiagnosticPortTarget::new(v.source.unit_id, v.source.port),
                                sink: DiagnosticPortTarget::new(v.sink.unit_id, v.sink.port),
                                component: v.component_id,
                            },
                            CouplingScale {
                                variable_mol_s: v.variable_mol_s,
                                residual_mol_s: v.residual_mol_s,
                            },
                        )
                    })
                    .collect(),
            ),
        },
    }
}
