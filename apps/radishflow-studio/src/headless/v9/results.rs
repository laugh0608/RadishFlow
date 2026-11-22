use super::{FlowPlanSupport, old};
use crate::{
    flow_plan_actions::{runtime_configuration_sha256, stored_flow_plan_with_support},
    headless::{
        planned::domain,
        v4::protocol::{ComponentValue, Initialization},
        v7::plan::wire_blocks,
    },
    mixed_result_projection,
};
use old::{
    protocol::{Code, Failure, Stage},
    references::Executions,
};
use rf_store::project_v4::{
    MIXED_RECYCLE_CAPABILITY, MIXED_RECYCLE_METHOD, StoredFlowPlan, StoredTearInitialization,
};
use rf_types::RfError;
use rf_ui::{AppState, FlowResultRecord, result_query::FlowResultView};
use serde_json::{Value, json};

pub(in crate::headless) fn plan(app: &AppState) -> Result<Value, Failure> {
    let workspace = &app.workspace;
    let configuration = workspace.flow_plan.current();
    let wire = wire(configuration)?;
    let ready = configuration.validate_for(&workspace.document.flowsheet);
    Ok(json!({
        "plan_revision": workspace.flow_plan.revision(),
        "configuration_sha256": runtime_configuration_sha256(configuration),
        "ready": ready.is_ok(),
        "not_ready": ready.as_ref().err().map(plan_diagnostic),
        "configuration": wire,
    }))
}

pub(in crate::headless) fn wire(
    configuration: &rf_ui::flow_plan::FlowPlanConfiguration,
) -> Result<Value, Failure> {
    let StoredFlowPlan::MixedRecycle(stored) =
        stored_flow_plan_with_support(configuration, FlowPlanSupport::MixedRecycle)
            .map_err(domain)?
    else {
        return Err(Failure::new(
            Stage::Query,
            Code::UnsupportedCapability,
            "v9 requires a mixed-recycle plan",
        ));
    };
    let tear = stored.tear;
    let initialization = match tear.initialization {
        StoredTearInitialization::ZeroComponentFlows => Initialization::ZeroComponentFlows,
        StoredTearInitialization::ComponentFlows { values_mol_s } => {
            Initialization::ComponentFlows {
                values: values_mol_s
                    .into_iter()
                    .map(|value| ComponentValue {
                        component_id: value.component_id.into_inner(),
                        value_mol_s: value.value,
                    })
                    .collect(),
            }
        }
    };
    Ok(json!({
            "method": MIXED_RECYCLE_METHOD,
            "acceptance_contract_version": 1,
            "temperature_k": stored.mixed.fixed_boundary.temperature_k,
            "pressure_pa": stored.mixed.fixed_boundary.pressure_pa,
            "blocks": wire_blocks(stored.mixed.blocks)?,
            "max_iteration_evaluations": stored.max_iteration_evaluations,
            "tear": {
                "stream_id": tear.stream_id,
                "source": { "unit_id": tear.source.unit_id, "port": tear.source.port },
                "sink": { "unit_id": tear.sink.unit_id, "port": tear.sink.port },
                "initialization": initialization,
                "residual_scaling": { "kind": "evaluated_fresh_total" },
            },
    }))
}

pub(super) fn plan_diagnostic(error: &RfError) -> Value {
    let context = error.context();
    json!({
        "code": context.diagnostic_code(),
        "message": error.to_string(),
        "unit_ids": context.related_unit_ids(),
        "stream_ids": context.related_stream_ids(),
        "component_ids": context.related_component_ids(),
        "ports": context.related_port_targets().iter().map(|port| json!({
            "unit_id": port.unit_id,
            "port": port.port_name,
        })).collect::<Vec<_>>(),
    })
}

pub(in crate::headless) fn flow(app: &AppState, executions: &Executions) -> Result<Value, Failure> {
    super::require_m2(app)?;
    let mut response = old::results::flow(app, executions);
    let origin = match app.workspace.flow_result() {
        FlowResultView::Current {
            snapshot: FlowResultRecord::Mixed(snapshot),
            origin,
        } => {
            let payload =
                mixed_result_projection::recycle_result(&snapshot.result).map_err(domain)?;
            let Value::Object(payload) = payload else {
                unreachable!("mixed result object")
            };
            response
                .as_object_mut()
                .expect("flow result object")
                .extend(payload);
            Some(origin)
        }
        FlowResultView::Current { .. } => {
            return Err(domain(
                RfError::invalid_input("mixed-recycle snapshot required")
                    .with_diagnostic_code("workspace.result.method_mismatch"),
            ));
        }
        FlowResultView::Failed { origin, .. } => {
            if let Some(failure) = app.workspace.mixed_failure_evidence() {
                response["diagnostic_evidence"] =
                    mixed_result_projection::recycle_failure(failure).map_err(domain)?;
            }
            Some(origin)
        }
        FlowResultView::Stale { origin, .. } => origin,
        FlowResultView::Missing { .. } => None,
    };
    if let Some(plan) = origin
        .and_then(|o| o.execution())
        .and_then(|e| e.plan.as_ref())
    {
        response["origin"]["execution"]["plan"] = json!({
            "method": plan.method,
            "acceptance_contract_version": plan.acceptance_contract_version,
            "configuration_sha256": plan.configuration_sha256,
            "capability": { "id": MIXED_RECYCLE_CAPABILITY, "version": 1 },
        });
    }
    Ok(response)
}
