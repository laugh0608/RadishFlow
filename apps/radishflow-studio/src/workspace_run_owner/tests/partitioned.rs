use super::*;
use crate::qs1_test_support::{recycle_case, select};
use rf_ui::flow_plan::FlowPlanConfiguration;
#[test]
fn ready_plan_change_cancels_and_joins_before_releasing_the_slot() {
    let mut owner = WorkspaceRunOwner::new();
    let mut app = recycle_case(0.6);
    let id = spawn(&mut owner, &mut app, load(), &ThreadSpawner);
    wait_ready(&owner);
    let plan = app.workspace.flow_plan.current().clone();
    select(&mut app, FlowPlanConfiguration::Acyclic);
    select(&mut app, plan);
    assert!(!owner.is_idle());
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.run_id, id);
    assert_eq!(
        summary.cancellation,
        Some(WorkspaceRunCancelReason::QualificationChanged)
    );
    assert_eq!(
        summary.disposition,
        FlowRunDisposition::Discarded(FlowRunDiscardReason::PlanChanged)
    );
    assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
    assert!(owner.is_idle());
    spawn(&mut owner, &mut app, load(), &ThreadSpawner);
    let summary = drain(&mut owner, &mut app);
    assert_eq!(summary.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert!(matches!(
        app.workspace.flow_result(),
        rf_ui::result_query::FlowResultView::Current {
            snapshot: rf_ui::FlowResultRecord::Partitioned(_),
            ..
        }
    ));
}
