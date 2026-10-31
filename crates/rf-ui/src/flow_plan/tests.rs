use super::*;
use crate::{
    AppState, DiagnosticSeverity, DiagnosticSummary, DocumentMetadata, FlowRunDiscardReason,
    FlowRunDisposition, FlowsheetDocument, SolveSnapshot,
    result_query::{FlowResultUnavailable, FlowResultView},
};
use rf_model::Flowsheet;
use rf_solver::partitioned::FixedRecycleBoundary;
fn app() -> AppState {
    AppState::new(FlowsheetDocument::new(
        Flowsheet::new("plan"),
        DocumentMetadata::new("plan-document", "Plan", std::time::UNIX_EPOCH),
    ))
}
fn partitioned() -> FlowPlanConfiguration {
    FlowPlanConfiguration::Partitioned(RecycleSolveOptions::new(
        FixedRecycleBoundary {
            temperature_k: 300.0,
            pressure_pa: 650_000.0,
        },
        vec![],
    ))
}
fn apply(app: &mut AppState, command: FlowPlanCommand) -> bool {
    app.workspace
        .apply_flow_plan(&FlowPlanEditContext::capture(&app.workspace), command)
        .unwrap()
}
#[test]
fn plan_transactions_preserve_input_identity_and_saved_value_but_never_reuse_generation() {
    let mut app = app();
    app.mark_saved("plan.json");
    let input = app.flow_run_qualification();
    let context = FlowPlanEditContext::capture(&app.workspace);
    assert!(apply(&mut app, FlowPlanCommand::Apply(partitioned())));
    assert!(input.same_input(&app.flow_run_qualification()));
    assert_eq!(app.workspace.document.revision, 0);
    assert!(app.workspace.project_save_state().simulation_dirty);
    assert!(!context.is_current(&app.workspace));
    assert!(!apply(&mut app, FlowPlanCommand::Apply(partitioned())));
    assert_eq!(app.workspace.flow_plan.revision(), 1);
    assert!(apply(&mut app, FlowPlanCommand::Undo));
    assert_eq!(
        app.workspace.flow_plan.current(),
        &FlowPlanConfiguration::Acyclic
    );
    assert!(!app.workspace.flow_plan.is_dirty());
    assert!(!context.is_current(&app.workspace));
    assert!(apply(&mut app, FlowPlanCommand::Redo));
    assert_eq!(app.workspace.flow_plan.revision(), 3);
    let capture = app.workspace.flow_plan.current().clone();
    app.workspace.flow_plan.record_saved(capture);
    assert!(!app.workspace.project_save_state().has_unsaved_changes());
    assert!(
        app.workspace
            .apply_flow_plan(&context, FlowPlanCommand::RestoreSaved)
            .is_err()
    );
}
#[test]
fn accepted_results_and_pending_receipts_are_both_invalidated_including_after_restore() {
    let mut app = app();
    let snapshot = SolveSnapshot::new(
        "snapshot",
        0,
        1,
        RunStatus::Converged,
        DiagnosticSummary::new(0, DiagnosticSeverity::Info, "ok"),
    );
    app.workspace.store_snapshot(snapshot, 5);
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Current { .. }
    ));
    let mut receipt = app.prepare_flow_run().into_receipt();
    apply(&mut app, FlowPlanCommand::Apply(partitioned()));
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Stale {
            reason: FlowResultUnavailable::PlanChanged,
            ..
        }
    ));
    apply(&mut app, FlowPlanCommand::RestoreSaved);
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Stale {
            reason: FlowResultUnavailable::PlanChanged,
            ..
        }
    ));
    assert_eq!(
        app.accept_flow_run(&mut receipt),
        FlowRunDisposition::Discarded(FlowRunDiscardReason::PlanChanged)
    );
    assert_eq!(
        app.accept_flow_run(&mut receipt),
        FlowRunDisposition::Discarded(FlowRunDiscardReason::AlreadySettled)
    );
}
#[test]
fn cloned_transactions_share_plan_context_but_independently_owned_workspaces_do_not() {
    let mut first = app();
    let context = FlowPlanEditContext::capture(&first.workspace);
    let mut transaction = first.clone();
    assert!(context.is_current(&transaction.workspace));
    apply(&mut transaction, FlowPlanCommand::Apply(partitioned()));
    assert!(context.is_current(&first.workspace));
    first.workspace.reset_flow_run_session();
    assert!(!context.is_current(&first.workspace));
    assert!(!context.is_current(&app().workspace));
}
#[test]
fn numeric_drafts_keep_raw_text_and_input_unit_without_committing_or_saving_them() {
    use rf_types::units::MeasurementUnit;
    let mut app = app();
    let mut draft = FlowPlanDraft::new(&app.workspace, partitioned());
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650_000.0,
            MeasurementUnit::Kilopascal,
        )
        .unwrap()
        .raw = "1e-".into();
    app.workspace.flow_plan.set_draft(draft);
    assert_eq!(app.workspace.flow_plan.pending_count(), 1);
    assert!(!app.workspace.flow_plan.is_dirty());
    assert!(app.workspace.apply_flow_plan_draft().is_err());
    assert_eq!(
        app.workspace.flow_plan.draft().unwrap().numbers()[&FlowPlanNumericField::Pressure].raw,
        "1e-"
    );
    let mut draft = app.workspace.flow_plan.draft().unwrap().clone();
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650_000.0,
            MeasurementUnit::Kilopascal,
        )
        .unwrap()
        .raw = "650".into();
    app.workspace.flow_plan.set_draft(draft);
    assert!(app.workspace.apply_flow_plan_draft().unwrap());
    assert!(app.workspace.flow_plan.draft().is_none());
    let FlowPlanConfiguration::Partitioned(options) = app.workspace.flow_plan.current() else {
        unreachable!()
    };
    assert_eq!(options.boundary.pressure_pa, 650_000.0);
}
#[test]
fn invalid_configuration_and_stale_drafts_leave_the_formal_plan_unchanged() {
    let mut app = app();
    let mut invalid = partitioned();
    let FlowPlanConfiguration::Partitioned(options) = &mut invalid else {
        unreachable!()
    };
    options.max_iteration_evaluations = 501;
    assert!(
        app.workspace
            .apply_flow_plan(
                &FlowPlanEditContext::capture(&app.workspace),
                FlowPlanCommand::Apply(invalid)
            )
            .is_err()
    );
    assert_eq!(app.workspace.flow_plan.revision(), 0);
    let draft = FlowPlanDraft::new(&app.workspace, partitioned());
    apply(&mut app, FlowPlanCommand::Apply(partitioned()));
    app.workspace.flow_plan.set_draft(draft);
    assert!(app.workspace.apply_flow_plan_draft().is_err());
    assert!(app.workspace.flow_plan.draft().is_some());
}

#[test]
fn plan_context_equality_ignores_physical_revision_and_acceptance_epoch() {
    let mut app = app();
    let expected = FlowPlanEditContext::capture(&app.workspace);
    app.workspace.revoke_flow_run_acceptance();
    assert_eq!(expected, FlowPlanEditContext::capture(&app.workspace));
    app.create_builtin_unit(crate::BuiltinUnitKind::Feed, std::time::UNIX_EPOCH)
        .unwrap();
    assert!(expected.is_current(&app.workspace));
    assert_eq!(expected, FlowPlanEditContext::capture(&app.workspace));
}
