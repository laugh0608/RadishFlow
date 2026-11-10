use super::*;
use crate::{flow_actions::FlowEditContext, qs1_test_support::*, test_support::*};
use rf_ui::{
    FlowResultRecord, FlowRunDiscardReason, FlowRunDisposition, flow_plan::*,
    result_query::FlowResultView,
};
fn execute(app: &mut AppState) -> CompletedWorkspaceSolve {
    let request =
        crate::build_workspace_solve_request(app, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap();
    prepare_workspace_solve(app, &request)
        .unwrap()
        .execute_with_property_package(&build_official_binary_hydrocarbon_in_memory_provider(
            OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        ))
}
#[test]
fn formal_recycle_run_has_true_blocks_io_and_repeatable_results() {
    let mut app = recycle_case(0.6);
    assert!(
        crate::studio_modeling_readiness::studio_workspace_modeling_run_blocker(&app.workspace)
            .is_none()
    );
    let legacy = StudioSolveRequest::new(OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID, "legacy", 1);
    assert!(prepare_workspace_solve(&mut app, &legacy).is_err());
    let mut completed = execute(&mut app);
    assert_eq!(completed.completion_result(), Ok(()));
    assert_eq!(
        complete_workspace_solve(&mut app, &mut completed),
        FlowRunDisposition::Applied
    );
    let FlowResultView::Current {
        snapshot: FlowResultRecord::Partitioned(first),
        origin,
    } = app.workspace.flow_result()
    else {
        panic!("current partitioned result")
    };
    let first = first.clone();
    assert_eq!(first.result.blocks.len(), 5);
    assert_eq!(first.result.costs.local_unit_calls, 102);
    let report = first
        .result
        .blocks
        .iter()
        .find_map(|b| b.recycle.as_ref())
        .unwrap();
    assert_eq!(
        (
            report.iteration_evaluations,
            report.verification_evaluations
        ),
        (48, 1)
    );
    let rec = &first.result.streams[&"s-recycle".into()];
    assert!((rec.total_molar_flow_mol_s - 7.5).abs() < 1e-9);
    assert!(
        (rec.total_molar_flow_mol_s * rec.overall_mole_fractions[&"methane".into()] - 3.3).abs()
            < 1e-9
    );
    let consumed = first.final_units[&"mixer-2".into()]
        .consumed_streams
        .iter()
        .find(|s| s.stream_id.as_str() == "s-recycle")
        .unwrap();
    assert_eq!(
        consumed.total_molar_flow_mol_s,
        report.assumed.total_molar_flow_mol_s
    );
    assert_ne!(
        report.assumed.total_molar_flow_mol_s,
        report.returned.total_molar_flow_mol_s
    );
    let evidence = origin.execution().unwrap();
    assert_eq!(
        evidence.plan.as_ref().unwrap().method,
        "radishflow.partitioned-sm/1"
    );
    assert!(evidence.package.is_some());
    let review = crate::FlowResultReview::capture(&app.workspace);
    let physical = FlowEditContext::capture(&app.workspace);
    let mut second = execute(&mut app);
    complete_workspace_solve(&mut app, &mut second);
    let FlowResultView::Current {
        snapshot: FlowResultRecord::Partitioned(second),
        ..
    } = app.workspace.flow_result()
    else {
        panic!()
    };
    assert_eq!(first.result.streams, second.result.streams);
    assert_eq!(first.result.blocks, second.result.blocks);
    assert!(
        review
            .guard
            .as_ref()
            .is_some_and(|g| !g.is_current(&app.workspace))
    );
    let plan = app.workspace.flow_plan.current().clone();
    select(&mut app, FlowPlanConfiguration::Acyclic);
    select(&mut app, plan);
    assert!(physical.validate(&app.workspace).is_ok());
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Stale { .. }
    ));
}
#[test]
fn plan_changes_reject_late_completion_and_failure_never_exposes_fields() {
    let mut app = recycle_case(0.6);
    let mut completion = execute(&mut app);
    let plan = app.workspace.flow_plan.current().clone();
    select(&mut app, FlowPlanConfiguration::Acyclic);
    select(&mut app, plan);
    assert_eq!(
        complete_workspace_solve(&mut app, &mut completion),
        FlowRunDisposition::Discarded(FlowRunDiscardReason::PlanChanged)
    );
    for (alpha, expected) in [
        (
            0.99,
            rf_solver::partitioned::PartitionedFailureKind::IterationLimit,
        ),
        (
            1.0,
            rf_solver::partitioned::PartitionedFailureKind::NoFiniteSteadyState,
        ),
    ] {
        let mut app = recycle_case(alpha);
        let mut completion = execute(&mut app);
        assert!(completion.completion_result().is_err());
        complete_workspace_solve(&mut app, &mut completion);
        assert!(matches!(
            app.workspace.flow_result(),
            FlowResultView::Failed { .. }
        ));
        assert_eq!(
            app.workspace.partitioned_failure_evidence().unwrap().kind,
            expected
        );
        assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
    }
}
