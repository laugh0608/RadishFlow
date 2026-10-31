use super::plan_contract_tests::execute;
use super::*;
use crate::equation_actions::*;
use crate::flow_actions::{FlowEditAccess, FlowEditOutcome};
use crate::qs1_test_support::select;
use crate::qs3_test_support::{case, explicit, options};
use rf_ui::{FlowResultRecord, result_query::FlowResultView};
use std::time::UNIX_EPOCH;
const EDIT: FlowEditAccess = FlowEditAccess::Editable;
fn run(app: &mut AppState) {
    let mut done = execute(app);
    assert_eq!(
        done.execution_outcome(),
        WorkspaceExecutionOutcome::Succeeded,
        "{:?}",
        done.completion_result()
    );
    assert_eq!(
        complete_workspace_solve(app, &mut done),
        FlowRunDisposition::Applied
    );
}
#[test]
fn equation_formal_result_projection_and_single_adoption_transaction() {
    for explicit_scales in [false, true] {
        let mut app = case(true);
        app.workspace.solve_session.mode = rf_ui::SimulationMode::Active;
        if explicit_scales {
            let p = explicit(&app, true);
            select(&mut app, p);
        }
        let input = app.workspace.document.flowsheet.clone();
        let plan = app.workspace.flow_plan.clone();
        run(&mut app);
        assert_eq!(app.workspace.document.flowsheet, input);
        let FlowResultView::Current {
            snapshot: FlowResultRecord::EquationOriented(s),
            origin,
        } = app.workspace.flow_result()
        else {
            panic!("current EO")
        };
        assert_eq!(
            origin
                .execution()
                .unwrap()
                .plan
                .as_ref()
                .unwrap()
                .equation_oriented
                .as_ref()
                .unwrap()
                .core_backend,
            Some(rf_solver::equation_oriented::INVERSE_BACKEND)
        );
        let v = crate::equation_result_projection::result(&s.result).unwrap();
        assert_eq!(
            v["evidence"]["definition"]["structure"]["variable_count"],
            29
        );
        assert_eq!(v["evidence"]["full_jacobian"]["dimension"], 29);
        assert_eq!(
            v["evidence"]["conditional_evaluations"][0]["linear"]["dimension"],
            28
        );
        assert_eq!(v["evidence"]["final_units"].as_array().unwrap().len(), 6);
        let preview = preview_equation_adoption(&app, EDIT, false).unwrap();
        assert_eq!(preview.stored_value(), Some(0.6));
        assert!((preview.solved_value() - 10.0 / 19.0).abs() < 1e-10);
        let revision = app.workspace.document.revision;
        let receipt =
            adopt_equation_candidate(&mut app, EDIT, false, &preview, UNIX_EPOCH).unwrap();
        assert_eq!(receipt.outcome, FlowEditOutcome::Committed);
        assert_eq!(app.workspace.document.revision, revision + 1);
        assert_eq!(app.workspace.flow_plan, plan);
        assert_eq!(
            app.workspace.solve_session.pending_reason,
            Some(rf_ui::SolvePendingReason::CandidateAdopted)
        );
        assert!(!crate::StudioRunOwner::default().should_attempt_automatic(&app));
        assert!(matches!(
            app.workspace.flow_result(),
            FlowResultView::Stale { .. }
        ));
        assert!(adopt_equation_candidate(&mut app, EDIT, false, &preview, UNIX_EPOCH).is_err());
        app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
        assert_eq!(app.workspace.document.flowsheet, input);
        assert!(adopt_equation_candidate(&mut app, EDIT, false, &preview, UNIX_EPOCH).is_err());
        run(&mut app);
        let preview = preview_equation_adoption(&app, EDIT, false).unwrap();
        adopt_equation_candidate(&mut app, EDIT, false, &preview, UNIX_EPOCH).unwrap();
        // Re-running the inverse plan yields the same candidate: validated no-op.
        run(&mut app);
        let preview = preview_equation_adoption(&app, EDIT, false).unwrap();
        let revision = app.workspace.document.revision;
        assert_eq!(
            adopt_equation_candidate(&mut app, EDIT, false, &preview, UNIX_EPOCH)
                .unwrap()
                .outcome,
            FlowEditOutcome::Unchanged
        );
        assert_eq!(app.workspace.document.revision, revision);
        select(
            &mut app,
            FlowPlanConfiguration::EquationOriented(options(false)),
        );
        run(&mut app);
        let FlowResultView::Current {
            snapshot: FlowResultRecord::EquationOriented(s),
            ..
        } = app.workspace.flow_result()
        else {
            panic!()
        };
        assert!((s.result.streams[&"s-pa".into()].total_molar_flow_mol_s - 1.5).abs() < 1e-10);
        assert!(preview_equation_adoption(&app, EDIT, false).is_err());
    }
}
#[test]
fn equation_structure_and_adoption_guards_reject_plan_aba_busy_readonly_and_drafts() {
    let mut app = case(true);
    let structure = inspect_equation_structure(&app.workspace).unwrap();
    assert_eq!(structure.definition.structure.variable_count, 29);
    run(&mut app);
    let preview = preview_equation_adoption(&app, EDIT, false).unwrap();
    let before = app.workspace.clone();
    assert!(
        adopt_equation_candidate(
            &mut app,
            FlowEditAccess::ReadOnly,
            false,
            &preview,
            UNIX_EPOCH
        )
        .is_err()
    );
    assert!(adopt_equation_candidate(&mut app, EDIT, true, &preview, UNIX_EPOCH).is_err());
    assert_eq!(app.workspace, before);
    let mut draft = rf_ui::flow_plan::FlowPlanDraft::new(
        &app.workspace,
        FlowPlanConfiguration::EquationOriented(options(true)),
    );
    draft
        .edit_number(
            rf_ui::flow_plan::FlowPlanNumericField::Pressure,
            650000.0,
            rf_types::units::MeasurementUnit::Pascal,
        )
        .unwrap()
        .raw = "1e-".into();
    app.workspace.flow_plan.set_draft(draft);
    assert!(preview_equation_adoption(&app, EDIT, false).is_err());
    app.workspace.flow_plan.discard_draft();
    select(
        &mut app,
        FlowPlanConfiguration::EquationOriented(options(false)),
    );
    select(
        &mut app,
        FlowPlanConfiguration::EquationOriented(options(true)),
    );
    assert!(!structure.is_current(&app.workspace));
    assert!(adopt_equation_candidate(&mut app, EDIT, false, &preview, UNIX_EPOCH).is_err());
}
#[test]
fn equation_failure_evidence_and_late_completion_never_publish_values() {
    let mut app = case(true);
    let mut o = options(true);
    o.target.as_mut().unwrap().value_mol_s = 3.0;
    select(&mut app, FlowPlanConfiguration::EquationOriented(o));
    let mut done = execute(&mut app);
    assert_eq!(done.execution_outcome(), WorkspaceExecutionOutcome::Failed);
    let SolveOutcome::EquationFailed(f) = &mut done.outcome else {
        panic!()
    };
    f.evidence.conditional_evaluations[0].target_residual_mol_s = Some(f64::NAN);
    complete_workspace_solve(&mut app, &mut done);
    let f = app.workspace.equation_failure_evidence().unwrap();
    let v = crate::equation_result_projection::failure(f).unwrap();
    assert_eq!(v["failure_kind"], "target_not_bracketed");
    assert_eq!(
        v["evidence"]["conditional_evaluations"][0]["target_residual_mol_s"]["non_finite"],
        "nan"
    );
    assert!(v.get("streams").is_none());
    assert!(preview_equation_adoption(&app, EDIT, false).is_err());
    for inverse in [false, true] {
        let mut app = case(inverse);
        let mut done = execute(&mut app);
        select(&mut app, FlowPlanConfiguration::Acyclic);
        select(
            &mut app,
            FlowPlanConfiguration::EquationOriented(options(inverse)),
        );
        assert!(matches!(
            complete_workspace_solve(&mut app, &mut done),
            FlowRunDisposition::Discarded(_)
        ));
        assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
    }
}
#[test]
fn equation_structure_incomplete_pairs_and_released_missing_input_are_distinct() {
    let mut app = case(true);
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&"splitter-1".into())
        .unwrap()
        .parameters
        .outlet_a_split_fraction = None;
    assert!(
        app.workspace
            .flow_plan
            .current()
            .validate_for(&app.workspace)
            .is_ok()
    );
    run(&mut app);
    for release_only in [true, false] {
        let mut app = case(true);
        let mut o = options(true);
        if release_only {
            o.target = None;
        } else {
            o.released = None;
        }
        select(&mut app, FlowPlanConfiguration::EquationOriented(o));
        let s = inspect_equation_structure(&app.workspace).unwrap();
        assert!(!s.definition.structure.is_square_matched());
        assert!(
            app.workspace
                .flow_plan
                .current()
                .validate_for(&app.workspace)
                .is_err()
        );
    }
}

struct StopAt(rf_solver::equation_oriented::EquationStage);
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
    fn checkpoint(&mut self, _: ModularProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
impl EquationControl for StopAt {
    fn checkpoint(&mut self, p: EquationProgress) -> ControlFlow<()> {
        if p.stage == self.0 {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
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
fn equation_cancellation_at_every_core_stage_preserves_current_result() {
    use rf_solver::equation_oriented::EquationStage::*;
    for stage in [
        Assemble,
        StructuralAnalysis,
        LinearEvaluation,
        BracketEndpoint,
        IntervalEvaluation,
        Factorization,
        ConditionSolve,
        FullJacobianCheck,
        FinalUnitVerification,
        GlobalVerification,
        Completing,
    ] {
        let mut app = case(true);
        run(&mut app);
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
        assert_eq!(
            done.equation_failure_evidence().unwrap().progress.stage,
            stage
        );
        assert!(
            crate::equation_result_projection::failure(done.equation_failure_evidence().unwrap())
                .is_ok()
        );
        complete_workspace_solve(&mut app, &mut done);
        assert_eq!(app.workspace.snapshot_history, before);
        assert!(matches!(
            app.workspace.flow_result(),
            FlowResultView::Current { .. }
        ));
        assert!(app.workspace.equation_failure_evidence().is_none());
    }
}
#[test]
fn equation_preparation_failure_has_no_core_or_package_provenance() {
    let mut app = case(true);
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
        panic!()
    };
    let e = origin.execution().unwrap();
    assert!(e.package.is_none());
    assert_eq!(e.solver_implementation, "not-invoked");
    assert_eq!(
        e.plan
            .as_ref()
            .unwrap()
            .equation_oriented
            .as_ref()
            .unwrap()
            .core_backend,
        None
    );
    assert!(app.workspace.equation_failure_evidence().is_none());
}
#[test]
fn equation_replaced_payload_is_rejected_before_publication() {
    let mut app = case(true);
    let mut other = crate::qs2_test_support::cross_case(0.6, 0.25);
    let mut a = execute(&mut app);
    let mut b = execute(&mut other);
    std::mem::swap(&mut a.outcome, &mut b.outcome);
    for (app, done) in [(&mut app, &mut a), (&mut other, &mut b)] {
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
        assert!(app.workspace.equation_failure_evidence().is_none());
    }
}
