use super::plan_contract_tests::execute;
use super::*;
use crate::{
    qs1_test_support::select,
    qs4_test_support::{case, explicit, recycle_case},
};
use rf_solver::mixed::MixedStage;
use rf_ui::{FlowResultRecord, result_query::FlowResultView};
#[test]
fn mixed_formal_result_is_atomic_sourced_and_invalidated_by_plan_aba() {
    for scales in [false, true] {
        let mut app = case();
        if scales {
            let p = explicit(&app);
            select(&mut app, p);
        }
        let input = app.workspace.document.flowsheet.clone();
        let mut done = execute(&mut app);
        assert_eq!(
            done.execution_outcome(),
            WorkspaceExecutionOutcome::Succeeded,
            "{:?}",
            done.completion_result()
        );
        assert_eq!(
            complete_workspace_solve(&mut app, &mut done),
            FlowRunDisposition::Applied
        );
        let FlowResultView::Current {
            snapshot: FlowResultRecord::Mixed(s),
            origin,
        } = app.workspace.flow_result()
        else {
            panic!()
        };
        let p = origin.execution().unwrap().plan.as_ref().unwrap();
        assert_eq!(
            p.mixed.as_ref().unwrap().core_method,
            Some(rf_solver::mixed::METHOD)
        );
        assert!(p.equation_oriented.is_none());
        assert!(p.modular.is_none());
        let v = crate::mixed_result_projection::result(&s.result).unwrap();
        assert_eq!(v["blocks"].as_array().unwrap().len(), 4);
        assert_eq!(v["boundaries"].as_array().unwrap().len(), 3);
        assert_eq!(v["costs"]["local_unit_calls"], 7);
        for b in v["boundaries"].as_array().unwrap() {
            assert_eq!(b["producer"], b["consumed"]);
        }
        let mut nonfinite = (*s.result).clone();
        nonfinite.global_material.total_residual_mol_s = f64::NAN;
        assert!(crate::mixed_result_projection::result(&nonfinite).is_err());
        assert_eq!(app.workspace.document.flowsheet, input);
        let mut done = execute(&mut app);
        let plan = app.workspace.flow_plan.current().clone();
        select(&mut app, FlowPlanConfiguration::Acyclic);
        select(&mut app, plan);
        assert!(matches!(
            complete_workspace_solve(&mut app, &mut done),
            FlowRunDisposition::Discarded(_)
        ));
        assert!(!matches!(
            app.workspace.flow_result(),
            FlowResultView::Current { .. }
        ));
    }
}
#[test]
fn mixed_wrong_payload_and_invalid_preflight_never_publish_results() {
    let mut app = case();
    let mut done = execute(&mut app);
    let mut eo = crate::qs3_test_support::case(false);
    let mut other = execute(&mut eo);
    std::mem::swap(&mut done.outcome, &mut other.outcome);
    complete_workspace_solve(&mut app, &mut done);
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Failed { .. }
    ));
    assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
    let mut app = case();
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&"heater-1".into())
        .unwrap()
        .parameters
        .outlet_temperature_k = Some(310.0);
    let request = crate::build_workspace_solve_request(
        &app,
        crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
    )
    .unwrap();
    assert!(prepare_workspace_solve(&mut app, &request).is_err());
    assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
}
struct Stop(MixedStage);
impl SolverControl for Stop {
    fn checkpoint(&mut self, _: SolveProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl PartitionedSolverControl for Stop {
    fn checkpoint(&mut self, _: PartitionedProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl ModularControl for Stop {
    fn checkpoint(&mut self, _: ModularProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl EquationControl for Stop {
    fn checkpoint(&mut self, _: EquationProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl MixedControl for Stop {
    fn checkpoint(&mut self, p: MixedProgress) -> ControlFlow<()> {
        if p.stage == self.0 {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
impl WorkspaceSolveControl for Stop {
    fn before_load(&mut self) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
    fn ready(&mut self) -> RfResult<ControlFlow<()>> {
        Ok(ControlFlow::Continue(()))
    }
}
#[test]
fn mixed_cancel_at_outer_stages_keeps_previous_current_and_diagnostics_separate() {
    for stage in [
        MixedStage::Assembly,
        MixedStage::BlockStarted,
        MixedStage::Equation,
        MixedStage::Boundary,
        MixedStage::BlockCompleted,
        MixedStage::GlobalVerification,
        MixedStage::Completing,
    ] {
        let mut app = case();
        let mut first = execute(&mut app);
        complete_workspace_solve(&mut app, &mut first);
        let before = app.workspace.snapshot_history.clone();
        let request = crate::build_workspace_solve_request(
            &app,
            crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        )
        .unwrap();
        let (receipt, job) = prepare_workspace_solve(&mut app, &request)
            .unwrap()
            .into_job();
        let provider = crate::test_support::build_official_binary_hydrocarbon_in_memory_provider(
            crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        );
        let output = job.execute(
            |id| {
                provider
                    .load_package(id, SystemTime::now())
                    .and_then(prepare_local_context)
                    .map_err(WorkspacePackageLoadError::Package)
            },
            false,
            &mut Stop(stage),
        );
        let mut done = CompletedWorkspaceSolve::from_output(receipt, output);
        assert_eq!(
            done.execution_outcome(),
            WorkspaceExecutionOutcome::Cancelled
        );
        let f = done.mixed_failure_evidence().unwrap();
        assert_eq!(f.progress.stage, stage);
        assert!(crate::mixed_result_projection::failure(f).is_ok());
        complete_workspace_solve(&mut app, &mut done);
        assert_eq!(app.workspace.snapshot_history, before);
        assert!(matches!(
            app.workspace.flow_result(),
            FlowResultView::Current { .. }
        ));
        assert!(app.workspace.mixed_failure_evidence().is_none());
    }
}

#[test]
fn mixed_recycle_accepts_verified_success_once_and_reports_current_numeric_failure() {
    let mut successful = recycle_case(500);
    let mut done = execute(&mut successful);
    assert_eq!(
        done.execution_outcome(),
        WorkspaceExecutionOutcome::Succeeded
    );
    assert_eq!(
        complete_workspace_solve(&mut successful, &mut done),
        FlowRunDisposition::Applied
    );
    let FlowResultView::Current {
        snapshot: FlowResultRecord::Mixed(snapshot),
        origin,
    } = successful.workspace.flow_result()
    else {
        panic!("verified M2 result must be Current");
    };
    assert_eq!(
        snapshot
            .result
            .recycle
            .as_ref()
            .unwrap()
            .residual_trace
            .len(),
        14
    );
    assert!(crate::mixed_result_projection::result(&snapshot.result).is_err());
    let origin = origin.execution().unwrap().plan.as_ref().unwrap();
    assert_eq!(origin.method, rf_solver::mixed::RECYCLE_METHOD);
    assert!(
        origin.mixed.is_none(),
        "the runtime method is independent of a project capability"
    );
    let history = successful.workspace.snapshot_history.clone();
    assert!(matches!(
        complete_workspace_solve(&mut successful, &mut done),
        FlowRunDisposition::Discarded(_)
    ));
    assert_eq!(successful.workspace.snapshot_history, history);

    let mut failing = recycle_case(1);
    let mut failed = execute(&mut failing);
    assert_eq!(
        failed.execution_outcome(),
        WorkspaceExecutionOutcome::Failed
    );
    assert_eq!(
        complete_workspace_solve(&mut failing, &mut failed),
        FlowRunDisposition::Applied
    );
    let FlowResultView::Failed { diagnostic, origin } = failing.workspace.flow_result() else {
        panic!("iteration exhaustion must become Failed for the current task");
    };
    assert_eq!(
        diagnostic.primary_code.as_deref(),
        Some("solver.mixed.iteration_limit")
    );
    assert_ne!(
        origin
            .execution()
            .unwrap()
            .plan
            .as_ref()
            .unwrap()
            .configuration_sha256,
        crate::flow_plan_actions::runtime_configuration_sha256(
            successful.workspace.flow_plan.current()
        )
    );
    assert!(rf_ui::latest_snapshot(&failing.workspace).is_none());
    assert!(
        crate::mixed_result_projection::failure(
            failing.workspace.mixed_failure_evidence().unwrap()
        )
        .is_err()
    );
    assert_eq!(
        failing
            .workspace
            .mixed_failure_evidence()
            .unwrap()
            .recycle
            .as_ref()
            .unwrap()
            .residual_trace
            .len(),
        1,
    );
}

#[test]
fn mixed_recycle_rejects_plan_aba_and_a_late_failure_after_reopen() {
    let mut app = recycle_case(500);
    let mut prior = execute(&mut app);
    complete_workspace_solve(&mut app, &mut prior);
    let mut late = execute(&mut app);
    let plan = app.workspace.flow_plan.current().clone();
    select(&mut app, FlowPlanConfiguration::Acyclic);
    select(&mut app, plan);
    assert!(matches!(
        complete_workspace_solve(&mut app, &mut late),
        FlowRunDisposition::Discarded(_)
    ));
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Stale { .. }
    ));

    let mut failing = recycle_case(1);
    let mut late = execute(&mut failing);
    failing.workspace.reset_flow_run_session();
    assert!(matches!(
        complete_workspace_solve(&mut failing, &mut late),
        FlowRunDisposition::Discarded(_)
    ));
    assert!(matches!(
        failing.workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
    assert!(failing.workspace.mixed_failure_evidence().is_none());
}

#[test]
fn dag_and_recycle_payloads_are_distinct_inside_the_mixed_method_family() {
    let mut dag = case();
    let mut cyclic = recycle_case(500);
    let mut left = execute(&mut dag);
    let mut right = execute(&mut cyclic);
    assert_eq!(
        left.execution_outcome(),
        WorkspaceExecutionOutcome::Succeeded
    );
    assert_eq!(
        right.execution_outcome(),
        WorkspaceExecutionOutcome::Succeeded
    );
    std::mem::swap(&mut left.outcome, &mut right.outcome);
    for (app, completed) in [(&mut dag, &mut left), (&mut cyclic, &mut right)] {
        assert_eq!(
            complete_workspace_solve(app, completed),
            FlowRunDisposition::Applied
        );
        assert_eq!(
            completed
                .completion_result()
                .unwrap_err()
                .context()
                .diagnostic_code(),
            Some("workspace.run.plan_payload_mismatch")
        );
        assert!(matches!(
            app.workspace.flow_result(),
            FlowResultView::Failed { .. }
        ));
        assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
    }
}

#[test]
fn mixed_recycle_cannot_be_projected_as_the_saved_dag() {
    let app = recycle_case(500);
    assert_eq!(
        crate::flow_plan_actions::stored_flow_plan(app.workspace.flow_plan.current())
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("workspace.plan.in_memory_only")
    );
}
