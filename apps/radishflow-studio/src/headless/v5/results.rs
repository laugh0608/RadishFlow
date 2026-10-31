use super::*;
use rf_ui::result_query::FlowResultView;
pub(super) fn flow(app: &AppState, executions: &Executions) -> Result<Value, Failure> {
    let mut value = if matches!(
        app.workspace.flow_plan.current(),
        FlowPlanConfiguration::Simultaneous(_)
    ) {
        old::results::flow(app, executions)
    } else {
        v4::results::flow(app, executions)?
    };
    let origin = match app.workspace.flow_result() {
        FlowResultView::Current {
            snapshot: rf_ui::FlowResultRecord::Simultaneous(s),
            origin,
        } => {
            let result = crate::modular_result_projection::result(&s.result).map_err(domain)?;
            value
                .as_object_mut()
                .expect("result object")
                .extend(result.as_object().expect("result object").clone());
            Some(origin)
        }
        FlowResultView::Current { origin, .. } => Some(origin),
        FlowResultView::Failed { origin, .. } => {
            if let Some(f) = app.workspace.simultaneous_failure_evidence() {
                value["diagnostic_evidence"] =
                    crate::modular_result_projection::failure(f).map_err(domain)?;
            }
            Some(origin)
        }
        FlowResultView::Stale { origin, .. } => origin,
        _ => None,
    };
    if let Some(plan) = origin
        .and_then(|o| o.execution())
        .and_then(|e| e.plan.as_ref())
    {
        let mut evidence = json!({"method":plan.method,"acceptance_contract_version":plan.acceptance_contract_version,"configuration_sha256":plan.configuration_sha256});
        if let Some(m) = &plan.modular {
            evidence["capability"] = json!({"id":m.capability,"version":m.capability_version});
            evidence["response_contract_version"] = json!(m.response_contract_version);
            evidence["core_method"] = json!(m.core_method);
        }
        value["origin"]["execution"]["plan"] = evidence;
    }
    Ok(value)
}
