use super::*;
use crate::{qs1_test_support::*, test_support::*};
pub(super) fn execute(app: &mut AppState) -> CompletedWorkspaceSolve {
    let request =
        crate::build_workspace_solve_request(app, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap();
    prepare_workspace_solve(app, &request)
        .unwrap()
        .execute_with_property_package(&build_official_binary_hydrocarbon_in_memory_provider(
            OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        ))
}
#[test]
fn typed_payloads_cannot_be_published_under_the_other_frozen_plan() {
    let mut part = recycle_case(0.6);
    let mut acyclic = crate::load_project_app_state(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    )))
    .unwrap();
    let mut a = execute(&mut acyclic);
    let mut p = execute(&mut part);
    std::mem::swap(&mut a.outcome, &mut p.outcome);
    for (app, completed) in [(&mut acyclic, &mut a), (&mut part, &mut p)] {
        assert_eq!(
            complete_workspace_solve(app, completed),
            FlowRunDisposition::Applied
        );
        assert!(matches!(
            app.workspace.flow_result(),
            rf_ui::result_query::FlowResultView::Failed { .. }
        ));
        assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
        assert_eq!(
            completed
                .completion_result()
                .unwrap_err()
                .context()
                .diagnostic_code(),
            Some("workspace.run.plan_payload_mismatch")
        );
    }
}
#[test]
fn formal_acyclic_request_cannot_survive_a_plan_change_back_before_preparation() {
    let mut app = recycle_case(0.6);
    let plan = app.workspace.flow_plan.current().clone();
    select(&mut app, FlowPlanConfiguration::Acyclic);
    let request =
        crate::build_workspace_solve_request(&app, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap();
    select(&mut app, plan);
    select(&mut app, FlowPlanConfiguration::Acyclic);
    assert!(prepare_workspace_solve(&mut app, &request).is_err());
    assert!(app.workspace.pending_flow_run().is_none());
}
