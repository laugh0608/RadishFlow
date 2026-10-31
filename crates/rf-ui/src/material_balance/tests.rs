use super::*;
use crate::{DocumentCommand, DocumentMetadata, FlowsheetDocument, PanelLayoutPreferences};
use rf_model::{Component, MaterialStreamState};
use rf_types::ComponentId;
use std::ops::ControlFlow;

pub(super) fn fixture() -> WorkspaceState {
    let mut flow = Flowsheet::new("material");
    for id in ["a", "b"] {
        flow.insert_component(Component::new(id, id)).unwrap();
    }
    for (id, total, z) in [("in-a", 2.0, 0.8), ("in-b", 3.0, 0.2), ("out", 0.0, 0.5)] {
        flow.insert_stream(MaterialStreamState::from_tpzf(
            id,
            id,
            300.0,
            100000.0,
            total,
            [(ComponentId::new("a"), z), (ComponentId::new("b"), 1.0 - z)]
                .into_iter()
                .collect(),
        ))
        .unwrap();
    }
    for unit in [
        rf_unitops::build_feed_node("fa", "FA", "in-a"),
        rf_unitops::build_feed_node("fb", "FB", "in-b"),
        rf_unitops::build_mixer_node("mix", "Mix", "in-a", "in-b", "out"),
    ] {
        flow.insert_unit(unit).unwrap();
    }
    let mut workspace = WorkspaceState::new(
        FlowsheetDocument::new(
            flow,
            DocumentMetadata::new("doc", "Material", std::time::UNIX_EPOCH),
        ),
        &PanelLayoutPreferences::default(),
    );
    workspace
        .set_material_balance_plan(MaterialBalancePlan::new(MixerBalanceSpecification::new(
            "mix",
            MixerBalanceScales {
                inlet_a_mol_s: 2.0,
                inlet_b_mol_s: 3.0,
                outlet_mol_s: 5.0,
            },
        )))
        .unwrap();
    workspace
}
struct Continue;
impl MaterialBalanceControl for Continue {
    fn checkpoint(&mut self, _: MaterialBalanceProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
fn calculate(workspace: &mut WorkspaceState) {
    let (input, mut receipt) = workspace.prepare_material_balance().unwrap();
    let outcome = input.execute(&mut Continue);
    assert!(matches!(outcome, MaterialBalanceOutcome::Succeeded(_)));
    assert_eq!(
        workspace.complete_material_balance(&mut receipt, outcome),
        MaterialBalanceDisposition::Applied
    );
}

#[test]
fn material_result_has_independent_origin_and_does_not_publish_a_flow_snapshot() {
    let mut w = fixture();
    let before = w.document.clone();
    calculate(&mut w);
    let MaterialBalanceResultView::Current { solution, origin } = w.material_balance_result()
    else {
        panic!("current")
    };
    assert_eq!(origin.plan(), w.material_balance_plan().unwrap());
    assert_eq!(
        origin.qualification().input().revision(),
        w.document.revision
    );
    assert!((solution.values.total_molar_flow_mol_s - 5.0).abs() < 1e-12);
    assert!(w.snapshot_history.is_empty());
    assert_eq!(w.document, before);
}

#[test]
fn plan_edits_and_change_back_cannot_revive_result_or_pending_receipt() {
    let mut w = fixture();
    calculate(&mut w);
    let guard = MaterialBalanceResultGuard::capture(&w).unwrap();
    let original = w.material_balance_plan().unwrap().clone();
    let (input, mut receipt) = w.prepare_material_balance().unwrap();
    let outcome = input.execute(&mut Continue);
    let mut changed = original.clone();
    changed.options.scaled_tolerance = 1e-8;
    w.set_material_balance_plan(changed).unwrap();
    w.set_material_balance_plan(original).unwrap();
    assert_eq!(
        w.complete_material_balance(&mut receipt, outcome),
        MaterialBalanceDisposition::Discarded(MaterialBalanceDiscardReason::ConfigurationChanged)
    );
    assert!(guard.read(&w).is_none());
    calculate(&mut w);
    assert!(guard.read(&w).is_none());
}

#[test]
fn invalid_plan_is_atomic_and_identical_plan_does_not_invalidate_result() {
    let mut w = fixture();
    calculate(&mut w);
    let before = w.clone();
    let plan = w.material_balance_plan().unwrap().clone();
    assert!(!w.set_material_balance_plan(plan.clone()).unwrap());
    assert_eq!(w, before);
    let mut bad = plan;
    bad.options.scaled_tolerance = f64::NAN;
    assert!(w.set_material_balance_plan(bad).is_err());
    assert_eq!(w, before);
}

#[test]
fn accepted_result_becomes_stale_after_a_plan_change_and_contains_no_values_in_stale_view() {
    let mut w = fixture();
    calculate(&mut w);
    let mut plan = w.material_balance_plan().unwrap().clone();
    plan.specification.scales.outlet_mol_s = 10.0;
    w.set_material_balance_plan(plan).unwrap();
    assert!(matches!(
        w.material_balance_result(),
        MaterialBalanceResultView::Stale { .. }
    ));
}

#[test]
fn completions_are_single_use_and_a_superseded_attempt_cannot_clear_the_new_one() {
    let mut w = fixture();
    let (first, mut r1) = w.prepare_material_balance().unwrap();
    let (second, mut r2) = w.prepare_material_balance().unwrap();
    assert_eq!(
        w.complete_material_balance(&mut r1, first.execute(&mut Continue)),
        MaterialBalanceDisposition::Discarded(MaterialBalanceDiscardReason::RevokedOrSuperseded)
    );
    assert!(w.material_balance_is_current(r2.qualification()));
    let outcome = second.execute(&mut Continue);
    assert_eq!(
        w.complete_material_balance(&mut r2, outcome.clone()),
        MaterialBalanceDisposition::Applied
    );
    assert_eq!(
        w.complete_material_balance(&mut r2, outcome),
        MaterialBalanceDisposition::Discarded(MaterialBalanceDiscardReason::AlreadySettled)
    );
}

#[test]
fn cancellation_and_clear_never_restore_previous_success() {
    let mut w = fixture();
    calculate(&mut w);
    let (input, mut receipt) = w.prepare_material_balance().unwrap();
    assert!(matches!(
        w.material_balance_result(),
        MaterialBalanceResultView::Missing
    ));
    assert!(w.revoke_material_balance(receipt.qualification().run_id()));
    assert!(matches!(
        w.complete_material_balance(&mut receipt, input.execute(&mut Continue)),
        MaterialBalanceDisposition::Discarded(_)
    ));
    assert!(matches!(
        w.material_balance_result(),
        MaterialBalanceResultView::Missing
    ));
    calculate(&mut w);
    w.clear_results();
    assert!(matches!(
        w.material_balance_result(),
        MaterialBalanceResultView::Missing
    ));
    assert!(w.material_balance_plan().is_some());
}

#[test]
fn failure_replaces_success_and_cannot_be_read_through_an_old_output_guard() {
    let mut w = fixture();
    calculate(&mut w);
    let guard = MaterialBalanceResultGuard::capture(&w).unwrap();
    let (_, mut receipt) = w.prepare_material_balance().unwrap();
    let error = rf_types::RfError::invalid_input("worker disconnected")
        .with_diagnostic_code("material.worker_disconnected");
    assert_eq!(
        w.complete_material_balance(
            &mut receipt,
            MaterialBalanceOutcome::Failed(Arc::new(MaterialBalanceFailure::Infrastructure(error)))
        ),
        MaterialBalanceDisposition::Applied
    );
    assert!(matches!(
        w.material_balance_result(),
        MaterialBalanceResultView::Failed {
            failure: MaterialBalanceFailure::Infrastructure(_),
            ..
        }
    ));
    assert!(guard.read(&w).is_none());
}

#[test]
fn independent_workspace_with_same_document_and_revision_rejects_old_receipt() {
    let mut w = fixture();
    let (input, mut receipt) = w.prepare_material_balance().unwrap();
    let mut other = fixture();
    assert_eq!(other.document, w.document);
    assert_eq!(
        other.complete_material_balance(&mut receipt, input.execute(&mut Continue)),
        MaterialBalanceDisposition::Discarded(MaterialBalanceDiscardReason::DifferentSession)
    );
}

#[test]
fn document_mutation_and_undo_revision_reject_frozen_results() {
    let workspace = fixture();
    let mut app = crate::AppState::new(workspace.document.clone());
    app.workspace = workspace;
    calculate(&mut app.workspace);
    let guard = MaterialBalanceResultGuard::capture(&app.workspace).unwrap();
    let original = app.workspace.document.flowsheet.clone();
    let (input, mut receipt) = app.workspace.prepare_material_balance().unwrap();
    let outcome = input.execute(&mut Continue);
    let mut next = original.clone();
    next.units.get_mut(&"mix".into()).unwrap().name = "Renamed".into();
    app.workspace.commit_document_change(
        DocumentCommand::RenameUnit {
            unit_id: "mix".into(),
            new_name: "Renamed".into(),
        },
        next,
        std::time::UNIX_EPOCH,
    );
    app.undo_document_command(std::time::UNIX_EPOCH)
        .unwrap()
        .unwrap();
    assert_eq!(app.workspace.document.flowsheet, original);
    assert_eq!(
        app.workspace
            .complete_material_balance(&mut receipt, outcome),
        MaterialBalanceDisposition::Discarded(MaterialBalanceDiscardReason::InputChanged)
    );
    assert!(guard.read(&app.workspace).is_none());
    app.redo_document_command(std::time::UNIX_EPOCH)
        .unwrap()
        .unwrap();
    assert!(guard.read(&app.workspace).is_none());
}

#[test]
fn numerical_drafts_preserve_unfinished_input_and_never_apply_invalid_values() {
    let mut w = fixture();
    let before = w.clone();
    let mut draft = MaterialBalanceDraft::from_plan(&w, w.material_balance_plan().unwrap());
    assert!(draft.is_applied(&w));
    draft.text[2] = "1e-".into();
    assert_eq!(
        draft.plan().unwrap_err().field,
        MaterialBalanceSetting::OutletScale
    );
    assert!(!draft.is_applied(&w));
    assert_eq!(draft.text[2], "1e-");
    assert_eq!(w, before);
    w.material_balance = MaterialBalanceState::default();
    assert!(!draft.is_applied(&w));
    draft.text[2] = "NaN".into();
    assert!(draft.plan().is_err());
    draft.text[2] = "0".into();
    assert!(draft.plan().is_err());
}
#[test]
fn calculation_table_uses_project_units_and_output_guard_covers_execution_and_display_changes() {
    use rf_types::units::DisplayUnitSet;
    let mut w = fixture();
    calculate(&mut w);
    let table = MaterialBalanceTable::capture(&w).unwrap().unwrap();
    assert!(table.tsv(&w).unwrap().contains("2.2 mol/s"));
    w.project_presentation
        .apply(crate::ProjectPresentationCommand::Apply(
            DisplayUnitSet::engineering(),
        ));
    assert!(table.tsv(&w).is_none());
    let converted = MaterialBalanceTable::capture(&w).unwrap().unwrap();
    assert!(converted.tsv(&w).unwrap().contains("7.92 kmol/h"));
    assert!(matches!(
        w.material_balance_result(),
        MaterialBalanceResultView::Current { .. }
    ));
    calculate(&mut w);
    assert!(converted.tsv(&w).is_none());
}
