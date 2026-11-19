use super::*;
use crate::{StudioProjectOpen, read_studio_project_with_support};
use rf_solver::equation_oriented::{EquationFailureKind, EquationOptions, ProductTarget};
use rf_store::project_v4::{FlowPlanSupport, StoredFlowPlan};
use rf_ui::result_query::{ExecutedFlowPlan, FlowResultOrigin, ResultStatus};
use serde_json::Value;
use std::path::Path;

fn reopened_case() -> AppState {
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/flowsheets/fixed-tp-product-composition-binary-hydrocarbon.rfproj.json"
    ));
    let StudioProjectOpen::Editable(app) =
        read_studio_project_with_support(path, FlowPlanSupport::ProductComposition).unwrap()
    else {
        panic!("editable composition project")
    };
    assert_eq!(app.workspace.flow_result().status(), ResultStatus::Missing);
    *app
}

fn equation_options(app: &AppState) -> EquationOptions {
    let FlowPlanConfiguration::EquationOriented(options) = app.workspace.flow_plan.current() else {
        panic!("expected EO plan")
    };
    options.clone()
}

fn origin_plan(origin: &FlowResultOrigin) -> &ExecutedFlowPlan {
    let plan = origin.execution().unwrap().plan.as_ref().unwrap();
    let eo = plan.equation_oriented.as_ref().unwrap();
    assert_eq!(
        eo.capability,
        rf_store::project_v4::EQUATION_ORIENTED_CAPABILITY
    );
    assert_eq!(eo.capability_version, 2);
    assert_eq!(eo.equation_contract_version, 2);
    assert_eq!(plan.acceptance_contract_version, 2);
    plan
}

fn number(value: &Value) -> f64 {
    value
        .as_f64()
        .unwrap_or_else(|| panic!("expected evidence number, got {value}"))
}

#[test]
fn reopened_project_publishes_versioned_producer_evidence_and_adopts_only_on_request() {
    let mut app = reopened_case();
    let input = app.workspace.document.clone();
    let plan = app.workspace.flow_plan.clone();
    run(&mut app);

    let FlowResultView::Current {
        snapshot: FlowResultRecord::EquationOriented(s),
        origin,
    } = app.workspace.flow_result()
    else {
        panic!("current EO result")
    };
    let execution_plan = origin_plan(origin);
    let stored = crate::flow_plan_actions::stored_flow_plan_with_support(
        plan.current(),
        FlowPlanSupport::ProductComposition,
    )
    .unwrap();
    assert!(matches!(&stored, StoredFlowPlan::EquationOriented(o) if o.contract_version() == 2));
    let configuration_bytes = serde_json::to_vec(&stored).unwrap();
    assert_eq!(
        execution_plan.configuration_sha256,
        rf_store::sha256_hex(&configuration_bytes)
    );
    assert_eq!(
        execution_plan
            .equation_oriented
            .as_ref()
            .unwrap()
            .core_backend,
        Some(s.result.backend)
    );
    assert!(origin.execution().unwrap().package.is_some());

    let value = crate::equation_result_projection::result(&s.result).unwrap();
    assert_eq!(value["equation_contract_version"], 2);
    assert_eq!(value["acceptance_contract_version"], 2);
    let evidence = &value["evidence"];
    assert_eq!(
        evidence["definition"]["configuration"],
        serde_json::to_value(stored).unwrap()
    );
    let target = &evidence["verified_target"];
    assert_eq!(target["kind"], "product_mole_fraction");
    assert_eq!(target["component_id"], "methane");
    assert_eq!(
        target["source"],
        serde_json::json!({"unit_id": "splitter-2", "port": "outlet_b", "stream_id": "s-pb"})
    );
    assert_eq!(target["target_mol_mol"], 0.35);
    let composition = &target["composition"];
    assert!((number(&composition["total_mol_s"]) - 24.0 / 7.0).abs() < 1e-11);
    assert!((number(&composition["component_mol_s"]) - 1.2).abs() < 1e-11);
    let producer = &s.result.streams[&"s-pb".into()];
    assert_eq!(
        number(&composition["actual_mol_mol"]),
        producer.overall_mole_fractions[&"methane".into()]
    );
    assert!(number(&composition["residual_mol_mol"]).abs() <= 1e-10);
    assert!(number(&target["residual_mol_s"]).abs() <= 1e-10);
    assert!(number(&target["scaled_residual"]).abs() <= 1e-10);
    let trial = &evidence["conditional_evaluations"][0];
    assert!((number(&trial["composition"]["actual_mol_mol"]) - 0.2).abs() < 1e-14);
    assert!((number(&trial["composition"]["residual_mol_mol"]) + 0.15).abs() < 1e-14);
    assert_eq!(app.workspace.document, input);

    let preview = preview_equation_adoption(&app, EDIT, false).unwrap();
    assert_eq!(preview.stored_value(), Some(0.6));
    assert!((preview.solved_value() - 0.5).abs() < 1e-12);
    let receipt = adopt_equation_candidate(&mut app, EDIT, false, &preview, UNIX_EPOCH).unwrap();
    assert_eq!(receipt.outcome, FlowEditOutcome::Committed);
    assert_eq!(app.workspace.document.revision, input.revision + 1);
    assert_eq!(app.workspace.flow_plan, plan);
    assert_eq!(app.workspace.flow_result().status(), ResultStatus::Stale);
}

#[test]
fn composition_solver_failures_publish_diagnostics_without_a_result_or_candidate() {
    #[derive(Debug, PartialEq)]
    enum Scenario {
        Unbracketed,
        IneffectiveRelease,
        ZeroProduct,
        TraceProduct,
    }
    use Scenario::*;
    for scenario in [Unbracketed, IneffectiveRelease, ZeroProduct, TraceProduct] {
        let mut app = reopened_case();
        let mut options = equation_options(&app);
        let release = options.released.as_mut().unwrap();
        let units = &mut app.workspace.document.flowsheet.units;
        let Some(ProductTarget::MoleFraction(target)) = &mut options.target else {
            panic!("composition target")
        };
        match scenario {
            Unbracketed => target.value_mol_mol = 0.45,
            IneffectiveRelease | ZeroProduct => {
                units
                    .get_mut(&"splitter-1".into())
                    .unwrap()
                    .parameters
                    .outlet_a_split_fraction = Some(0.5);
                release.binding.unit = "splitter-2".into();
                if scenario == ZeroProduct {
                    release.upper = 1.0;
                }
            }
            TraceProduct => {
                units
                    .get_mut(&"splitter-2".into())
                    .unwrap()
                    .parameters
                    .outlet_a_split_fraction = Some(1.0 - 1e-12);
                release.lower = 0.6;
            }
        }
        select(&mut app, FlowPlanConfiguration::EquationOriented(options));
        let mut done = execute(&mut app);
        assert_eq!(done.execution_outcome(), WorkspaceExecutionOutcome::Failed);
        assert_eq!(
            complete_workspace_solve(&mut app, &mut done),
            FlowRunDisposition::Applied
        );
        let FlowResultView::Failed { origin, .. } = app.workspace.flow_result() else {
            panic!("failed composition result for {scenario:?}")
        };
        origin_plan(origin);
        let failure = app.workspace.equation_failure_evidence().unwrap();
        let projected = crate::equation_result_projection::failure(failure).unwrap();
        assert!(projected.get("streams").is_none());
        assert!(projected["evidence"]["verified_target"].is_null());
        assert!(preview_equation_adoption(&app, EDIT, false).is_err());
        assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
        match scenario {
            Unbracketed => assert_eq!(failure.kind, EquationFailureKind::TargetNotBracketed),
            IneffectiveRelease => assert!(matches!(
                failure.kind,
                EquationFailureKind::LinearSingular | EquationFailureKind::IllConditioned
            )),
            ZeroProduct => {
                assert_eq!(failure.kind, EquationFailureKind::UndefinedComposition);
                let trials = projected["evidence"]["conditional_evaluations"]
                    .as_array()
                    .unwrap();
                assert_eq!(trials.last().unwrap()["completed"], false);
                assert!(trials.last().unwrap().get("composition").is_none());
            }
            TraceProduct => {
                assert_eq!(failure.kind, EquationFailureKind::TargetNotBracketed);
                let trial = &projected["evidence"]["conditional_evaluations"][0];
                assert!(number(&trial["target_residual_mol_s"]).abs() < 1e-10);
                assert!(number(&trial["composition"]["residual_mol_mol"]).abs() > 0.02);
            }
        }
    }
}

#[test]
fn cancellation_and_late_completions_do_not_publish_composition_results() {
    let mut app = reopened_case();
    run(&mut app);
    let before = app.workspace.snapshot_history.clone();
    let mut cancelled = cancel_at(
        &mut app,
        rf_solver::equation_oriented::EquationStage::Completing,
    );
    assert_eq!(
        cancelled.execution_outcome(),
        WorkspaceExecutionOutcome::Cancelled
    );
    assert_eq!(
        complete_background_workspace_solve(&mut app, &mut cancelled),
        FlowRunDisposition::Applied
    );
    assert_eq!(app.workspace.snapshot_history, before);
    assert_eq!(app.workspace.flow_result().status(), ResultStatus::Current);
    assert!(app.workspace.equation_failure_evidence().is_none());

    let mut late = execute(&mut app);
    let original = app.workspace.flow_plan.current().clone();
    select(
        &mut app,
        FlowPlanConfiguration::EquationOriented(options(true)),
    );
    select(&mut app, original);
    let before = app.workspace.clone();
    assert!(matches!(
        complete_background_workspace_solve(&mut app, &mut late),
        FlowRunDisposition::Discarded(_)
    ));
    assert_eq!(app.workspace, before);
    assert_eq!(app.workspace.flow_result().status(), ResultStatus::Stale);
    run(&mut app);
    assert_eq!(app.workspace.flow_result().status(), ResultStatus::Current);
}

#[test]
fn receipt_rejects_a_success_payload_with_the_wrong_contract() {
    let mut app = reopened_case();
    let mut done = execute(&mut app);
    let SolveOutcome::Executed(FlowExecutionOutcome::EquationOriented(Ok(result))) =
        &mut done.outcome
    else {
        panic!("successful composition result")
    };
    result.acceptance_contract_version = 1;
    assert_eq!(
        complete_workspace_solve(&mut app, &mut done),
        FlowRunDisposition::Applied
    );
    assert_eq!(
        done.execution_outcome(),
        WorkspaceExecutionOutcome::InfrastructureFailed
    );
    assert_eq!(
        done.completion_result()
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("workspace.run.plan_payload_mismatch")
    );
    assert_eq!(app.workspace.flow_result().status(), ResultStatus::Failed);
    assert!(rf_ui::latest_snapshot(&app.workspace).is_none());
}
