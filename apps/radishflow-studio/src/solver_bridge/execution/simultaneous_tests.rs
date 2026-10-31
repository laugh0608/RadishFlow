use super::plan_contract_tests::execute;
use super::*;
use crate::qs1_test_support::select;
use crate::qs2_test_support::{cross_case, explicit_plan, plan};
use rf_ui::{FlowResultRecord, result_query::FlowResultView};

#[test]
fn simultaneous_formal_run_publishes_real_evidence_and_rejects_old_guards() {
    for explicit in [false, true] {
        let mut app = cross_case(0.6, 0.25);
        if explicit {
            let p = explicit_plan(&app);
            select(&mut app, p);
        }
        let mut done = execute(&mut app);
        assert_eq!(
            done.execution_outcome(),
            WorkspaceExecutionOutcome::Succeeded
        );
        assert_eq!(
            complete_workspace_solve(&mut app, &mut done),
            FlowRunDisposition::Applied
        );
        let FlowResultView::Current {
            snapshot: FlowResultRecord::Simultaneous(s),
            origin,
        } = app.workspace.flow_result()
        else {
            panic!("simultaneous current")
        };
        assert_eq!(s.result.blocks.len(), 3);
        let c = &s.result.costs;
        assert_eq!(
            (
                c.local_unit_calls,
                c.response_evaluations,
                c.factorizations,
                c.triangular_solves
            ),
            (10, 4, 1, 9)
        );
        // Independent fraction benchmark, component identities rather than vector positions.
        for (id, methane, ethane) in [
            ("s-ma", 35.0 / 17.0, 20.0 / 17.0),
            ("s-rab", 21.0 / 17.0, 12.0 / 17.0),
            ("s-mb", 156.0 / 85.0, 264.0 / 85.0),
            ("s-rba", 39.0 / 85.0, 66.0 / 85.0),
            ("s-pa", 14.0 / 17.0, 8.0 / 17.0),
            ("s-pb", 117.0 / 85.0, 198.0 / 85.0),
        ] {
            let stream = &s.result.streams[&id.into()];
            for (component, expected) in [("methane", methane), ("ethane", ethane)] {
                let actual = stream.total_molar_flow_mol_s
                    * stream.overall_mole_fractions[&component.into()];
                assert!(
                    (actual - expected).abs() < 1e-9,
                    "{id}/{component}: {actual} != {expected}"
                );
            }
        }
        let p = origin.execution().unwrap().plan.as_ref().unwrap();
        assert_eq!(
            p.modular.as_ref().unwrap().core_method,
            Some(rf_solver::simultaneous::METHOD)
        );
        let wire = crate::modular_result_projection::result(&s.result).unwrap();
        assert_eq!(wire["blocks"][2]["coupling"]["linear"]["dimension"], 8);
        let guard =
            crate::FlowResultGuard::from_snapshot(rf_ui::latest_snapshot(&app.workspace).unwrap())
                .unwrap();
        select(&mut app, FlowPlanConfiguration::Acyclic);
        select(&mut app, plan());
        assert!(matches!(
            app.workspace.flow_result(),
            FlowResultView::Stale { .. }
        ));
        assert!(!guard.is_current(&app.workspace));
        assert!(matches!(
            complete_workspace_solve(&mut app, &mut done),
            FlowRunDisposition::Discarded(_)
        ));
    }
}

#[test]
fn simultaneous_late_success_failure_and_wrong_payload_cannot_publish() {
    for ratios in [(0.4, 0.3), (1.0, 1.0)] {
        let mut app = cross_case(ratios.0, ratios.1);
        let mut done = execute(&mut app);
        select(&mut app, FlowPlanConfiguration::Acyclic);
        select(&mut app, plan());
        assert!(matches!(
            complete_workspace_solve(&mut app, &mut done),
            FlowRunDisposition::Discarded(_)
        ));
        assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
        assert!(app.workspace.simultaneous_failure_evidence().is_none());
    }
    let mut app = cross_case(0.4, 0.3);
    let mut part = crate::qs1_test_support::recycle_case(0.6);
    let mut done = execute(&mut app);
    let mut other = execute(&mut part);
    std::mem::swap(&mut done.outcome, &mut other.outcome);
    for (app, done) in [(&mut app, &mut done), (&mut part, &mut other)] {
        complete_workspace_solve(app, done);
        assert!(matches!(
            app.workspace.flow_result(),
            FlowResultView::Failed { .. }
        ));
        assert_eq!(
            done.completion_result()
                .unwrap_err()
                .context()
                .diagnostic_code(),
            Some("workspace.run.plan_payload_mismatch")
        );
    }
}

#[test]
fn simultaneous_failure_preserves_partial_diagnostics_and_nonfinite_values() {
    let mut app = cross_case(1.0, 1.0);
    let mut done = execute(&mut app);
    let SolveOutcome::SimultaneousFailed(f) = &mut done.outcome else {
        panic!("singular failure")
    };
    assert_eq!(f.completed_blocks.len(), 2);
    let linear = f.active_coupling.as_mut().unwrap().linear.get_or_insert(
        rf_solver::simultaneous::LinearEvidence {
            dimension: 8,
            matrix_inf_norm: 0.0,
            pivot_threshold: 0.0,
            row_swaps: vec![],
            rcond_indicator: None,
            backward_error: None,
        },
    );
    linear.rcond_indicator = Some(f64::NAN);
    linear.backward_error = Some(f64::INFINITY);
    complete_workspace_solve(&mut app, &mut done);
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Failed { .. }
    ));
    let f = app.workspace.simultaneous_failure_evidence().unwrap();
    let wire = crate::modular_result_projection::failure(f).unwrap();
    assert_eq!(wire["error_code"], f.error.code().as_str());
    assert_eq!(
        wire["active_coupling"]["linear"]["rcond_indicator"]["non_finite"],
        "nan"
    );
    assert_eq!(
        wire["active_coupling"]["linear"]["backward_error"]["non_finite"],
        "positive_infinity"
    );
    assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
}

struct StopAt(rf_solver::simultaneous::ModularStage);
impl SolverControl for StopAt {
    fn checkpoint(&mut self, _: SolveProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl PartitionedSolverControl for StopAt {
    fn checkpoint(&mut self, _: PartitionedProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl ModularControl for StopAt {
    fn checkpoint(&mut self, p: ModularProgress) -> ControlFlow<()> {
        if p.stage == self.0 {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
impl EquationControl for StopAt {
    fn checkpoint(&mut self, _: EquationProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl WorkspaceSolveControl for StopAt {
    fn before_load(&mut self) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
    fn ready(&mut self) -> RfResult<ControlFlow<()>> {
        Ok(ControlFlow::Continue(()))
    }
}
#[test]
fn simultaneous_core_cancellation_keeps_current_result_and_exposes_only_terminal_diagnostics() {
    use rf_solver::simultaneous::ModularStage;
    for stage in [
        ModularStage::AssemblyStarted,
        ModularStage::Anchor,
        ModularStage::Factorization,
        ModularStage::ConditionSolve,
        ModularStage::Verification,
        ModularStage::Completing,
    ] {
        let mut app = cross_case(0.6, 0.25);
        let mut success = execute(&mut app);
        complete_workspace_solve(&mut app, &mut success);
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
            &mut StopAt(stage),
        );
        let mut done = CompletedWorkspaceSolve::from_output(receipt, output);
        assert_eq!(
            done.execution_outcome(),
            WorkspaceExecutionOutcome::Cancelled
        );
        let f = done.simultaneous_failure_evidence().unwrap();
        assert_eq!(f.progress.stage, stage);
        assert!(crate::modular_result_projection::failure(f).is_ok());
        assert_eq!(
            complete_workspace_solve(&mut app, &mut done),
            FlowRunDisposition::Applied
        );
        assert_eq!(app.workspace.snapshot_history, before);
        assert!(matches!(
            app.workspace.flow_result(),
            FlowResultView::Current { .. }
        ));
        assert!(app.workspace.simultaneous_failure_evidence().is_none());
    }
}
#[test]
fn simultaneous_load_failure_never_claims_core_execution_or_package_provenance() {
    let mut app = cross_case(0.6, 0.25);
    let request = crate::build_workspace_solve_request(
        &app,
        crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
    )
    .unwrap();
    let mut done = prepare_workspace_solve(&mut app, &request)
        .unwrap()
        .execute_with_property_package(
            &rf_property_package::InMemoryPropertyPackageProvider::default(),
        );
    complete_workspace_solve(&mut app, &mut done);
    let FlowResultView::Failed { origin, .. } = app.workspace.flow_result() else {
        panic!("failed")
    };
    let e = origin.execution().unwrap();
    assert!(e.package.is_none());
    assert_eq!(e.solver_implementation, "not-invoked");
    assert_eq!(
        e.plan
            .as_ref()
            .unwrap()
            .modular
            .as_ref()
            .unwrap()
            .core_method,
        None
    );
    assert!(app.workspace.simultaneous_failure_evidence().is_none());
}
