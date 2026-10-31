use super::*;
use rf_ui::result_query::FlowResultView;
pub(super) fn flow(app: &AppState, executions: &Executions) -> Result<Value, Failure> {
    let mut value = if matches!(
        app.workspace.flow_plan.current(),
        FlowPlanConfiguration::EquationOriented(_)
    ) {
        old::results::flow(app, executions)
    } else {
        <v5::Plan as PlanProtocol>::flow(app, executions)?
    };
    let origin = match app.workspace.flow_result() {
        FlowResultView::Current {
            snapshot: rf_ui::FlowResultRecord::EquationOriented(s),
            origin,
        } => {
            let r = crate::equation_result_projection::result(&s.result).map_err(domain)?;
            value
                .as_object_mut()
                .expect("result object")
                .extend(r.as_object().expect("result object").clone());
            Some(origin)
        }
        FlowResultView::Failed { origin, .. } => {
            if let Some(f) = app.workspace.equation_failure_evidence() {
                value["diagnostic_evidence"] =
                    crate::equation_result_projection::failure(f).map_err(domain)?;
            }
            Some(origin)
        }
        FlowResultView::Stale { origin, .. } => origin,
        _ => None,
    };
    if let Some(p) = origin
        .and_then(|o| o.execution())
        .and_then(|e| e.plan.as_ref())
        && let Some(e) = &p.equation_oriented
    {
        let mut evidence = json!({"method":p.method,"acceptance_contract_version":p.acceptance_contract_version,"configuration_sha256":p.configuration_sha256});
        evidence["capability"] = json!({"id":e.capability,"version":e.capability_version});
        evidence["equation_contract_version"] = json!(e.equation_contract_version);
        evidence["core_backend"] = json!(e.core_backend);
        value["origin"]["execution"]["plan"] = evidence;
    }
    Ok(value)
}
