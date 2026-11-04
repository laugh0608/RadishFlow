use super::*;
use rf_ui::result_query::FlowResultView;
pub(super) fn flow(app: &AppState, e: &Executions) -> Result<Value, Failure> {
    if !matches!(
        app.workspace.flow_plan.current(),
        FlowPlanConfiguration::Mixed(_)
    ) {
        return <v6::Plan as PlanProtocol>::flow(app, e);
    }
    let mut v = old::results::flow(app, e);
    let origin = match app.workspace.flow_result() {
        FlowResultView::Current {
            snapshot: rf_ui::FlowResultRecord::Mixed(s),
            origin,
        } => {
            let result = crate::mixed_result_projection::result(&s.result).map_err(domain)?;
            v.as_object_mut()
                .expect("result object")
                .extend(result.as_object().expect("mixed object").clone());
            Some(origin)
        }
        FlowResultView::Failed { origin, .. } => {
            if let Some(f) = app.workspace.mixed_failure_evidence() {
                v["diagnostic_evidence"] =
                    crate::mixed_result_projection::failure(f).map_err(domain)?;
            }
            Some(origin)
        }
        FlowResultView::Stale { origin, .. } => origin,
        _ => None,
    };
    if let Some(p) = origin
        .and_then(|o| o.execution())
        .and_then(|e| e.plan.as_ref())
        && let Some(m) = &p.mixed
    {
        v["origin"]["execution"]["plan"] = json!({"method":p.method,"acceptance_contract_version":p.acceptance_contract_version,"configuration_sha256":p.configuration_sha256,"capability":{"id":m.capability,"version":m.capability_version},"core_method":m.core_method});
    }
    Ok(v)
}
