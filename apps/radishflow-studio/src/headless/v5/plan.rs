use super::*;
use rf_solver::simultaneous::{CouplingBinding, FixedBoundary, ModularOptions};
use rf_types::DiagnosticPortTarget;
use std::collections::BTreeMap;
pub(super) fn configuration(
    plan: &Plan,
    refs: &Bindings,
) -> Result<FlowPlanConfiguration, Failure> {
    let Plan::Simultaneous(ModularPlan::Simultaneous {
        acceptance_contract_version,
        response_contract_version,
        temperature_k,
        pressure_pa,
        scaling,
    }) = plan
    else {
        let Plan::Existing(p) = plan else {
            unreachable!()
        };
        return v4::plan::configuration(p, refs);
    };
    if *acceptance_contract_version != 1 || *response_contract_version != 1 {
        return Err(Failure::new(
            Stage::Step,
            Code::UnsupportedCapability,
            "unsupported modular contract version",
        ));
    }
    let mut options = ModularOptions::new(FixedBoundary {
        temperature_k: *temperature_k,
        pressure_pa: *pressure_pa,
    });
    if let Scaling::ExplicitCouplings { values } = scaling {
        let mut scales = BTreeMap::new();
        for v in values {
            let binding = CouplingBinding {
                stream: refs.resolve(&v.stream, Output::Stream)?.into(),
                source: DiagnosticPortTarget::new(
                    refs.resolve(&v.source.unit, Output::Unit)?,
                    v.source.port.clone(),
                ),
                sink: DiagnosticPortTarget::new(
                    refs.resolve(&v.sink.unit, Output::Unit)?,
                    v.sink.port.clone(),
                ),
                component: v.component_id.clone().into(),
            };
            let scale = rf_solver::simultaneous::CouplingScale {
                variable_mol_s: v.variable_mol_s,
                residual_mol_s: v.residual_mol_s,
            };
            if scales.insert(binding, scale).is_some() {
                return Err(invalid("duplicate coupling scale binding"));
            }
        }
        options.scales = Some(scales);
    }
    let result = FlowPlanConfiguration::Simultaneous(options);
    result.validate_configuration().map_err(domain)?;
    Ok(result)
}
pub(super) fn wire(plan: &FlowPlanConfiguration) -> Result<Value, Failure> {
    let FlowPlanConfiguration::Simultaneous(o) = plan else {
        return <v4::protocol::Plan as PlanProtocol>::wire(plan);
    };
    let scaling = o
        .scales
        .as_ref()
        .map(|scales| Scaling::ExplicitCouplings {
            values: scales
                .iter()
                .map(|(b, s)| CouplingScale {
                    stream: IdentityRef::id(&b.stream),
                    source: old::actions::Port {
                        unit: IdentityRef::id(&b.source.unit_id),
                        port: b.source.port_name.clone(),
                    },
                    sink: old::actions::Port {
                        unit: IdentityRef::id(&b.sink.unit_id),
                        port: b.sink.port_name.clone(),
                    },
                    component_id: b.component.to_string(),
                    variable_mol_s: s.variable_mol_s,
                    residual_mol_s: s.residual_mol_s,
                })
                .collect(),
        })
        .unwrap_or(Scaling::EvaluatedFreshTotal);
    serde_json::to_value(ModularPlan::Simultaneous {
        acceptance_contract_version: 1,
        response_contract_version: 1,
        temperature_k: o.boundary.temperature_k,
        pressure_pa: o.boundary.pressure_pa,
        scaling,
    })
    .map_err(old::invalid_request)
}
pub(super) fn query(app: &AppState) -> Result<Value, Failure> {
    let w = &app.workspace;
    let ready = w.flow_plan.current().validate_for(w);
    let preview = if let FlowPlanConfiguration::Simultaneous(o) = w.flow_plan.current() {
        rf_solver::simultaneous::assemble(&w.document.flowsheet,o).ok().map(|p| p.blocks().iter().enumerate().map(|(i,b)|json!({"index":i,"members":b.members.iter().map(|id|id.as_str()).collect::<Vec<_>>(),"couplings":b.couplings.iter().map(crate::modular_result_projection::binding).collect::<Vec<_>>()})).collect::<Vec<_>>())
    } else {
        None
    };
    Ok(
        json!({"plan_revision":w.flow_plan.revision(),"configuration":wire(w.flow_plan.current())?,"ready":ready.is_ok(),"not_ready":ready.err().map(|e|json!({"code":e.context().diagnostic_code(),"message":e.to_string()})),"simulation_dirty":w.flow_plan.is_dirty(),"methods":["radishflow.acyclic-sm/1","radishflow.partitioned-sm/1","radishflow.fixed-tp-affine-modular-lu/1"],"in_memory_required_project_schema_version":crate::project_save_version(app),"in_memory_required_capabilities":required_capabilities(app),"coupling_preview":preview}),
    )
}
