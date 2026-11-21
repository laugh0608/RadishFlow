//! Saved single-vapor recycle plans and acceptance evidence.
use super::{
    planned::{PlanProtocol, domain},
    v3 as old, v11,
};
use old::{
    protocol::Failure,
    references::{Bindings, Executions},
};
use rf_model::MaterialStreamState;
use rf_solver::{
    UnitEnergyBalance, partitioned::EvaluationPhase,
    single_phase_recycle::SinglePhaseRecycleResidual,
};
use rf_store::project_v4::{
    FlowPlanSupport, SINGLE_PHASE_RECYCLE_METHOD, StoredFlowPlan, StoredSinglePhaseRecyclePlan,
};
use rf_ui::{
    AppState, FlowResultRecord, flow_plan::FlowPlanConfiguration, result_query::FlowResultView,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(super) const VERSION: u32 = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", deny_unknown_fields)]
pub(super) enum RecyclePlan {
    #[serde(rename = "radishflow.single-vapor-sensible-recycle/1")]
    Project {
        single_phase_recycle: Box<StoredSinglePhaseRecyclePlan>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum Plan {
    Recycle(RecyclePlan),
    Existing(v11::Plan),
}

impl PlanProtocol for Plan {
    const VERSION: u32 = VERSION;

    fn configuration(plan: &Self, refs: &Bindings) -> Result<FlowPlanConfiguration, Failure> {
        match plan {
            Self::Recycle(RecyclePlan::Project {
                single_phase_recycle,
            }) => crate::flow_plan_actions::flow_plan_configuration(
                StoredFlowPlan::SinglePhaseRecycle(single_phase_recycle.clone()),
            )
            .map_err(domain),
            Self::Existing(plan) => v11::Plan::configuration(plan, refs),
        }
    }

    fn wire(configuration: &FlowPlanConfiguration) -> Result<Value, Failure> {
        if !matches!(configuration, FlowPlanConfiguration::SinglePhaseRecycle(_)) {
            return v11::Plan::wire(configuration);
        }
        let StoredFlowPlan::SinglePhaseRecycle(single_phase_recycle) =
            crate::flow_plan_actions::stored_flow_plan_with_support(
                configuration,
                FlowPlanSupport::SinglePhaseRecycle,
            )
            .map_err(domain)?
        else {
            unreachable!("single phase recycle conversion")
        };
        Ok(json!(RecyclePlan::Project {
            single_phase_recycle
        }))
    }

    fn query(app: &AppState) -> Result<Value, Failure> {
        let plan = app.workspace.flow_plan.current();
        if !matches!(plan, FlowPlanConfiguration::SinglePhaseRecycle(_)) {
            return v11::Plan::query(app);
        }
        let error = plan.validate_for(&app.workspace.document.flowsheet).err();
        Ok(json!({
            "plan_revision": app.workspace.flow_plan.revision(),
            "configuration": Self::wire(plan)?,
            "configuration_sha256": crate::flow_plan_actions::runtime_configuration_sha256(plan),
            "ready": error.is_none(),
            "diagnostic": error.map(|e| json!({
                "code": e.context().diagnostic_code(), "message": e.message(),
                "unit_ids": e.context().related_unit_ids(), "stream_ids": e.context().related_stream_ids()
            })),
            "reference_temperature_k": rf_thermo::SensibleEnthalpyModel::REFERENCE_TEMPERATURE_K,
        }))
    }

    fn flow(app: &AppState, executions: &Executions) -> Result<Value, Failure> {
        if !matches!(
            app.workspace.flow_plan.current(),
            FlowPlanConfiguration::SinglePhaseRecycle(_)
        ) {
            return v11::Plan::flow(app, executions);
        }
        let mut value = old::results::flow(app, executions);
        match app.workspace.flow_result() {
            FlowResultView::Current {
                snapshot: FlowResultRecord::SinglePhaseRecycle(snapshot),
                origin,
            } => {
                super::v4::results::evidence(&mut value, origin);
                let result = &snapshot.result;
                let tear = &result.tear;
                let verification = &result.verification;
                value["execution_kind"] = json!(SINGLE_PHASE_RECYCLE_METHOD);
                value["streams"] = json!(result.streams.values().map(stream).collect::<Vec<_>>());
                value["energy_balances"] = json!(
                    result
                        .final_units
                        .iter()
                        .filter_map(|unit| {
                            unit.energy_balance
                                .as_ref()
                                .map(|b| json!({"unit_id":unit.unit_id, "balance":balance(b)}))
                        })
                        .collect::<Vec<_>>()
                );
                value["overall_energy_balance"] = balance(&result.overall_energy_balance);
                value["tear"] = json!({
                    "stream_id": tear.stream_id,
                    "source": {"unit_id":tear.source.unit_id,"port":tear.source.port_name},
                    "sink": {"unit_id":tear.sink.unit_id,"port":tear.sink.port_name},
                    "assumed": stream(&verification.assumed),
                    "returned": stream(&verification.returned),
                    "component_residuals_mol_s": verification.component_residuals_mol_s,
                    "component_scale_mol_s": result.component_scale_mol_s,
                    "enthalpy_scale_w": result.tear_enthalpy_scale_w,
                    "verification": residual(&verification.residual),
                    "residual_trace": result.trace.iter().map(residual).collect::<Vec<_>>(),
                });
            }
            FlowResultView::Failed { origin, .. } => {
                super::v4::results::evidence(&mut value, origin)
            }
            _ => {}
        }
        Ok(value)
    }
}

fn stream(stream: &MaterialStreamState) -> Value {
    json!({
        "id":stream.id, "temperature_k":stream.temperature_k, "pressure_pa":stream.pressure_pa,
        "total_molar_flow_mol_s":stream.total_molar_flow_mol_s, "mole_fractions":stream.overall_mole_fractions,
        "enthalpy_flow_w":rf_solver::single_phase_recycle::single_phase_enthalpy_flow_w(stream).expect("verified stream enthalpy"),
    })
}

fn residual(r: &SinglePhaseRecycleResidual) -> Value {
    json!({
        "phase": match r.material.phase { EvaluationPhase::Iteration => "iteration", EvaluationPhase::Verification => "verification" },
        "evaluation":r.material.evaluation,
        "max_absolute_mol_s":r.material.max_absolute_mol_s,
        "max_scaled":r.material.max_scaled,
        "enthalpy_flow_residual_w":r.enthalpy_flow_residual_w,
        "scaled_enthalpy_flow_residual":r.scaled_enthalpy_flow_residual,
    })
}

fn balance(b: &UnitEnergyBalance) -> Value {
    json!({
        "inlet_enthalpy_flow_w":b.inlet_enthalpy_flow_w,
        "outlet_enthalpy_flow_w":b.outlet_enthalpy_flow_w,
        "heat_duty_w":b.heat_duty_w,
        "residual_w":b.residual_w,
        "residual_scale_w":b.residual_scale_w,
        "scaled_residual":b.scaled_residual,
    })
}
