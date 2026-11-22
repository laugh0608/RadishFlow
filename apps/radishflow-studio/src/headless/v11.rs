//! Project-owned single-vapor plan and accepted energy evidence.
use super::{
    planned::{PlanProtocol, domain},
    v3 as old, v10,
};
use old::{
    protocol::Failure,
    references::{Bindings, Executions},
};
use rf_store::project_v4::{
    FlowPlanSupport, SINGLE_PHASE_METHOD, StoredFlowPlan, StoredSinglePhasePlan,
};
use rf_ui::{AppState, flow_plan::FlowPlanConfiguration, result_query::FlowResultView};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(super) const VERSION: u32 = 11;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", deny_unknown_fields)]
pub(super) enum SinglePhasePlan {
    #[serde(rename = "radishflow.acyclic-single-vapor-sensible/1")]
    Project { single_phase: StoredSinglePhasePlan },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum Plan {
    SinglePhase(SinglePhasePlan),
    Existing(v10::Plan),
}

impl PlanProtocol for Plan {
    const VERSION: u32 = VERSION;

    fn configuration(plan: &Self, refs: &Bindings) -> Result<FlowPlanConfiguration, Failure> {
        match plan {
            Self::SinglePhase(SinglePhasePlan::Project { single_phase }) => {
                crate::flow_plan_actions::flow_plan_configuration(StoredFlowPlan::SinglePhase(
                    single_phase.clone(),
                ))
                .map_err(domain)
            }
            Self::Existing(plan) => v10::Plan::configuration(plan, refs),
        }
    }

    fn wire(configuration: &FlowPlanConfiguration) -> Result<Value, Failure> {
        if !matches!(configuration, FlowPlanConfiguration::SinglePhase(_)) {
            return v10::Plan::wire(configuration);
        }
        let StoredFlowPlan::SinglePhase(single_phase) =
            crate::flow_plan_actions::stored_flow_plan_with_support(
                configuration,
                FlowPlanSupport::SinglePhase,
            )
            .map_err(domain)?
        else {
            unreachable!("single phase conversion")
        };
        Ok(json!(SinglePhasePlan::Project { single_phase }))
    }

    fn query(app: &AppState) -> Result<Value, Failure> {
        let plan = app.workspace.flow_plan.current();
        if !matches!(plan, FlowPlanConfiguration::SinglePhase(_)) {
            return v10::Plan::query(app);
        }
        let error = plan.validate_for(&app.workspace.document.flowsheet).err();
        Ok(json!({
            "plan_revision": app.workspace.flow_plan.revision(),
            "configuration": Self::wire(plan)?,
            "configuration_sha256": crate::flow_plan_actions::runtime_configuration_sha256(plan),
            "ready": error.is_none(),
            "diagnostic": error.map(|e| json!({"code": e.context().diagnostic_code(), "message":e.message(),"unit_ids": e.context().related_unit_ids()})),
            "reference_temperature_k": rf_thermo::SensibleEnthalpyModel::REFERENCE_TEMPERATURE_K,
        }))
    }

    fn flow(app: &AppState, executions: &Executions) -> Result<Value, Failure> {
        if !matches!(
            app.workspace.flow_plan.current(),
            FlowPlanConfiguration::SinglePhase(_)
        ) {
            return v10::Plan::flow(app, executions);
        }
        let mut value = super::v4::results::flow(app, executions)?;
        if let FlowResultView::Current { snapshot, .. } = app.workspace.flow_result()
            && let Some(snapshot) = snapshot.acyclic()
        {
            value["execution_kind"] = json!(SINGLE_PHASE_METHOD);
            value["streams"] = json!(
                snapshot
                    .streams
                    .iter()
                    .map(|s| json!({
                        "id": s.stream_id,
                        "temperature_k": s.temperature_k,
                        "pressure_pa": s.pressure_pa,
                        "total_molar_flow_mol_s": s.total_molar_flow_mol_s,
                    }))
                    .collect::<Vec<_>>()
            );
            value["energy_balances"] = json!(
                snapshot
                    .energy_balances
                    .iter()
                    .map(|(id, b)| json!({
                        "unit_id": id,
                        "inlet_enthalpy_flow_w": b.inlet_enthalpy_flow_w,
                        "outlet_enthalpy_flow_w": b.outlet_enthalpy_flow_w,
                        "heat_duty_w": b.heat_duty_w,
                        "residual_w": b.residual_w,
                        "residual_scale_w": b.residual_scale_w,
                        "scaled_residual": b.scaled_residual,
                    }))
                    .collect::<Vec<_>>()
            );
        }
        Ok(value)
    }
}
