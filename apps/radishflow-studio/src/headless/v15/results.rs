use super::{PLAN_SUPPORT, v3};
use crate::{
    StudioRunId,
    flow_plan_actions::{runtime_configuration_sha256, stored_flow_plan_with_support},
    headless::{
        planned::{domain, plan_diagnostic},
        v4::protocol::Initialization,
        v7::plan::wire_blocks,
    },
    mixed_result_projection,
};
use rf_solver::mixed::SERIAL_RECYCLE_METHOD;
use rf_store::project_v4::{MIXED_SERIAL_RECYCLE_CAPABILITY, StoredFlowPlan};
use rf_types::RfError;
use rf_ui::{AppState, FlowResultRecord, result_query::FlowResultView};
use serde_json::{Value, json};
use v3::{protocol::Failure, references::Executions};

fn capability() -> Value {
    json!({ "id": MIXED_SERIAL_RECYCLE_CAPABILITY, "version": 1 })
}

pub(super) fn plan(app: &AppState) -> Result<Value, Failure> {
    let workspace = &app.workspace;
    let configuration = workspace.flow_plan.current();
    let StoredFlowPlan::MixedSerialRecycle(stored) =
        stored_flow_plan_with_support(configuration, PLAN_SUPPORT).map_err(domain)?
    else {
        return Err(domain(method_mismatch()));
    };
    let ready = configuration.validate_for(&workspace.document.flowsheet);
    let regions = stored.recycle_regions.map(|region| {
        let tear = region.tear;
        json!({
            "max_iteration_evaluations": region.max_iteration_evaluations,
            "tear": {
                "stream_id": tear.stream_id,
                "source": { "unit_id": tear.source.unit_id, "port": tear.source.port },
                "sink": { "unit_id": tear.sink.unit_id, "port": tear.sink.port },
                "initialization": Initialization::from(tear.initialization),
            },
        })
    });
    Ok(json!({
        "plan_revision": workspace.flow_plan.revision(),
        "configuration_sha256": runtime_configuration_sha256(configuration),
        "ready": ready.is_ok(),
        "not_ready": ready.as_ref().err().map(plan_diagnostic),
        "configuration": {
            "method": SERIAL_RECYCLE_METHOD,
            "capability": capability(),
            "acceptance_contract_version": 1,
            "temperature_k": stored.mixed.fixed_boundary.temperature_k,
            "pressure_pa": stored.mixed.fixed_boundary.pressure_pa,
            "blocks": wire_blocks(stored.mixed.blocks)?,
            "scaling": { "kind": "evaluated_region_inlet_and_global_feed_total" },
            "recycle_regions": regions,
        },
    }))
}

pub(super) fn flow(app: &AppState, executions: &Executions) -> Result<Value, Failure> {
    let mut response = v3::results::flow(app, executions);
    let origin = match app.workspace.flow_result() {
        FlowResultView::Current {
            snapshot: FlowResultRecord::Mixed(snapshot),
            origin,
        } => {
            let execution = origin
                .execution()
                .ok_or_else(|| domain(method_mismatch()))?;
            if execution
                .plan
                .as_ref()
                .is_none_or(|plan| plan.method != SERIAL_RECYCLE_METHOD)
                || executions
                    .step(&StudioRunId::Flow(execution.run_id.clone()))
                    .is_none()
            {
                return Err(domain(method_mismatch()));
            }
            let payload =
                mixed_result_projection::serial_recycle_result(&snapshot.result).map_err(domain)?;
            response
                .as_object_mut()
                .expect("flow result object")
                .extend(payload);
            Some(origin)
        }
        FlowResultView::Current { .. } => return Err(domain(method_mismatch())),
        FlowResultView::Failed { origin, .. } => {
            if let Some(failure) = app.workspace.mixed_failure_evidence() {
                response["diagnostic_evidence"] =
                    mixed_result_projection::serial_recycle_failure(failure).map_err(domain)?;
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
            "capability": capability(),
        });
    }
    Ok(response)
}

fn method_mismatch() -> RfError {
    RfError::invalid_input("v15 requires a qualified serial recycle plan and run")
        .with_diagnostic_code("workspace.result.method_mismatch")
}
