use super::*;
use crate::{build_workspace_solve_request, test_support::*};
use rf_types::UnitId;
use rf_ui::{
    DocumentCommand, DocumentMetadata, FlowRunDiscardReason as Reason, FlowRunDisposition,
    FlowsheetDocument, SimulationMode,
};
use std::time::UNIX_EPOCH;

fn app() -> AppState {
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new("same-document", "Same document", UNIX_EPOCH),
    ))
}

fn prepare(app: &mut AppState, fails: bool) -> PreparedWorkspaceSolve {
    let package = if fails {
        "missing-package"
    } else {
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID
    };
    let request = build_workspace_solve_request(app, package).unwrap();
    prepare_workspace_solve(app, &request).unwrap()
}

fn execute(request: PreparedWorkspaceSolve) -> CompletedWorkspaceSolve {
    request.execute_with_property_package(&build_official_binary_hydrocarbon_in_memory_provider(
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
    ))
}

fn rename(app: &mut AppState, name: &str) {
    let mut flowsheet = app.workspace.document.flowsheet.clone();
    flowsheet
        .units
        .get_mut(&UnitId::new("heater-1"))
        .unwrap()
        .name = name.into();
    app.commit_document_change(
        DocumentCommand::RenameUnit {
            unit_id: UnitId::new("heater-1"),
            new_name: name.into(),
        },
        flowsheet,
        UNIX_EPOCH,
    );
}

fn rejected(app: &mut AppState, completed: &mut CompletedWorkspaceSolve, reason: Reason) {
    let before = app.clone();
    assert_eq!(
        complete_workspace_solve(app, completed),
        FlowRunDisposition::Discarded(reason)
    );
    assert_eq!(
        *app, before,
        "rejection must not publish or mutate a newer attempt"
    );
}

#[test]
fn execution_freezes_formal_inputs_and_selected_package_without_borrowing_live_state() {
    let mut app = app();
    let baseline = execute(prepare(&mut app.clone(), false));
    let mut options =
        build_workspace_solve_request(&app, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap();
    let prepared = prepare_workspace_solve(&mut app, &options).unwrap();
    let run_id = prepared.run_id().clone();
    let original = prepared.flowsheet().clone();
    options.package_id = "missing-package".into();
    options.snapshot_id = "changed-after-prepare".into();
    rename(&mut app, "Changed after prepare");
    assert_eq!(prepared.flowsheet(), &original);
    assert_eq!(
        prepared.request().package_id,
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID
    );
    assert_eq!(prepared.revision(), 0);
    let mut completed = execute(prepared);
    assert_eq!(completed.run_id(), &run_id);
    assert_eq!(completed.result().unwrap(), baseline.result().unwrap());
    assert_eq!(completed.provenance(), baseline.provenance());
    assert_eq!(
        completed.package_descriptor(),
        baseline.package_descriptor()
    );
    rejected(&mut app, &mut completed, Reason::InputChanged);
}

#[test]
fn reopen_and_explicit_independent_owner_reject_same_document_and_revision() {
    for fork in [false, true] {
        let mut original = app();
        let mut completed = execute(prepare(&mut original, false));
        let mut other = if fork {
            let mut fork = original.clone();
            fork.workspace.reset_flow_run_session();
            fork
        } else {
            app()
        };
        assert_eq!(original.workspace.document, other.workspace.document);
        let pending = prepare(&mut other, false);
        rejected(&mut other, &mut completed, Reason::DifferentSession);
        assert_eq!(other.workspace.pending_flow_run(), Some(pending.run_id()));
    }
}

#[test]
fn unchanged_transaction_clone_retains_receipt_and_consumes_only_once() {
    let mut original = app();
    let mut completed = execute(prepare(&mut original, false));
    let mut committed = original.clone();
    assert_eq!(
        complete_workspace_solve(&mut committed, &mut completed),
        FlowRunDisposition::Applied
    );
    rejected(&mut committed, &mut completed, Reason::AlreadySettled);
    // Moving the same completion back to the pre-commit state cannot publish twice.
    rejected(&mut original, &mut completed, Reason::AlreadySettled);
}

#[test]
fn equal_revision_divergent_transactions_never_share_input_qualification() {
    let mut left = app();
    let mut right = left.clone();
    rename(&mut left, "left");
    rename(&mut right, "right");
    assert_eq!(
        left.workspace.document.revision,
        right.workspace.document.revision
    );
    let mut completed = execute(prepare(&mut left, false));
    let pending = prepare(&mut right, false);
    rejected(&mut right, &mut completed, Reason::InputChanged);
    assert_eq!(right.workspace.pending_flow_run(), Some(pending.run_id()));
}

#[test]
fn undo_redo_never_revive_receipts_even_when_values_return() {
    let mut app = app();
    let mut before_edit = execute(prepare(&mut app, false));
    let original = app.workspace.document.flowsheet.clone();
    rename(&mut app, "changed");
    let mut before_undo = execute(prepare(&mut app, false));
    app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
    assert_eq!(app.workspace.document.flowsheet, original);
    rejected(&mut app, &mut before_edit, Reason::InputChanged);
    let mut before_redo = execute(prepare(&mut app, false));
    app.redo_document_command(UNIX_EPOCH).unwrap().unwrap();
    rejected(&mut app, &mut before_undo, Reason::InputChanged);
    rejected(&mut app, &mut before_redo, Reason::InputChanged);
}

#[test]
fn old_success_and_failure_cannot_change_new_pending_or_completed_state() {
    for fails in [false, true] {
        for old_first in [false, true] {
            let mut app = app();
            let mut old = execute(prepare(&mut app, fails));
            assert_eq!(old.result().is_err(), fails);
            rename(&mut app, "new-input");
            app.set_simulation_mode(SimulationMode::Active);
            let mut current = execute(prepare(&mut app, false));
            if old_first {
                rejected(&mut app, &mut old, Reason::InputChanged);
            }
            assert_eq!(
                complete_workspace_solve(&mut app, &mut current),
                FlowRunDisposition::Applied
            );
            if !old_first {
                rejected(&mut app, &mut old, Reason::InputChanged);
            }
            assert_eq!(app.workspace.solve_session.mode, SimulationMode::Active);
            assert_eq!(app.workspace.solve_session.status, RunStatus::Converged);
            assert_eq!(app.workspace.snapshot_history.len(), 1);
            assert_eq!(app.workspace.snapshot_history[0].document_revision, 1);
            rejected(&mut app, &mut current, Reason::AlreadySettled);
        }
    }
}

#[test]
fn attempts_have_distinct_ids_even_with_identical_snapshot_names() {
    let mut app = app();
    let first = prepare(&mut app, false);
    let second = prepare(&mut app, false);
    assert_eq!(first.request(), second.request());
    assert_ne!(first.run_id(), second.run_id());
    let mut old = execute(first);
    rejected(&mut app, &mut old, Reason::Superseded);
    let mut current = execute(second);
    assert_eq!(
        complete_workspace_solve(&mut app, &mut current),
        FlowRunDisposition::Applied
    );
}

#[test]
fn clear_results_revokes_unreceived_success_and_failure_at_same_revision() {
    for fails in [false, true] {
        let mut app = app();
        let mut old = execute(prepare(&mut app, fails));
        let revision = app.workspace.document.revision;
        app.workspace.clear_results();
        assert_eq!(app.workspace.document.revision, revision);
        let new_request = prepare(&mut app, false);
        assert_ne!(old.run_id(), new_request.run_id());
        rejected(&mut app, &mut old, Reason::AcceptanceRevoked);
        let mut current = execute(new_request);
        assert_eq!(
            complete_workspace_solve(&mut app, &mut current),
            FlowRunDisposition::Applied
        );
    }
}

#[test]
fn revoking_acceptance_keeps_prior_results_and_does_not_dirty_document() {
    let mut app = app();
    let mut prior = execute(prepare(&mut app, false));
    complete_workspace_solve(&mut app, &mut prior);
    app.mark_saved("saved.rfproj.json");
    let mut unreceived = execute(prepare(&mut app, false));
    let document = app.workspace.document.clone();
    let history = app.workspace.snapshot_history.clone();
    app.workspace.revoke_flow_run_acceptance();
    assert_eq!(app.workspace.document, document);
    assert_eq!(app.workspace.snapshot_history, history);
    assert_eq!(app.workspace.last_saved_revision, Some(document.revision));
    rejected(&mut app, &mut unreceived, Reason::AcceptanceRevoked);
}

#[test]
fn logout_revokes_unreceived_results_but_hold_and_save_as_do_not() {
    let mut app = app();
    let mut completed = execute(prepare(&mut app, false));
    app.set_simulation_mode(SimulationMode::Hold);
    app.mark_saved("another-path.rfproj.json");
    assert_eq!(
        complete_workspace_solve(&mut app, &mut completed),
        FlowRunDisposition::Applied
    );
    let mut next = execute(prepare(&mut app, false));
    app.clear_auth_session();
    rejected(&mut app, &mut next, Reason::AcceptanceRevoked);
}

#[test]
fn current_failure_uses_existing_diagnostics_and_is_not_republished() {
    let mut app = app();
    app.set_simulation_mode(SimulationMode::Active);
    let mut completed = execute(prepare(&mut app, true));
    assert_eq!(
        completed.result().unwrap_err().context().diagnostic_code(),
        Some(WORKSPACE_RUN_DIAGNOSTIC_PROPERTY_PACKAGE_MISSING)
    );
    assert_eq!(
        complete_workspace_solve(&mut app, &mut completed),
        FlowRunDisposition::Applied
    );
    assert_eq!(app.workspace.solve_session.status, RunStatus::Error);
    assert_eq!(app.workspace.solve_session.mode, SimulationMode::Hold);
    assert_eq!(app.workspace.pending_flow_run(), None);
    rejected(&mut app, &mut completed, Reason::AlreadySettled);
}

#[test]
fn invalid_options_leave_pending_and_all_application_state_unchanged() {
    let mut app = app();
    let pending = prepare(&mut app, false);
    let before = app.clone();
    assert!(
        prepare_workspace_solve(&mut app, &StudioSolveRequest::new("", "snapshot", 1)).is_err()
    );
    assert_eq!(app, before);
    assert_eq!(app.workspace.pending_flow_run(), Some(pending.run_id()));
}

#[test]
fn snapshot_sequence_exhaustion_is_structured_and_atomic() {
    let mut app = app();
    let mut completed = execute(prepare(&mut app, false));
    complete_workspace_solve(&mut app, &mut completed);
    app.workspace.snapshot_history.back_mut().unwrap().sequence = u64::MAX;
    let before = app.clone();
    let error =
        build_workspace_solve_request(&app, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("workspace.run.snapshot_sequence_exhausted")
    );
    assert_eq!(app, before);
    assert!(
        crate::WorkspaceSolveService::new()
            .run_with_property_package(
                &mut app,
                &build_official_binary_hydrocarbon_in_memory_provider(
                    OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID
                ),
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            )
            .is_err()
    );
    assert_eq!(app, before);
}

#[test]
fn presentation_and_uncommitted_drafts_preserve_qualification_and_survive_publication() {
    let mut app = app();
    let mut completed = execute(prepare(&mut app, false));
    let target = rf_ui::InspectorTarget::Unit("heater-1".into());
    app.focus_inspector_target(target.clone());
    app.update_unit_inspector_draft(
        &"heater-1".into(),
        rf_ui::UnitInspectorDraftField::OutletTemperatureK,
        "400",
    )
    .unwrap();
    app.workspace
        .project_presentation
        .apply(rf_ui::ProjectPresentationCommand::Apply(
            rf_types::units::DisplayUnitSet::engineering(),
        ));
    app.preferences.locale = rf_ui::LocaleCode::new("en-US");
    app.workspace.panels.inspector_open = false;
    let drafts = app.workspace.drafts.clone();
    let presentation = app.workspace.project_presentation.clone();
    assert_eq!(
        complete_workspace_solve(&mut app, &mut completed),
        FlowRunDisposition::Applied
    );
    assert_eq!(app.workspace.document.revision, 0);
    assert_eq!(app.workspace.snapshot_history[0].document_revision, 0);
    assert_eq!(app.workspace.drafts, drafts);
    assert_eq!(app.workspace.drafts.active_target, Some(target));
    assert_eq!(app.workspace.project_presentation, presentation);
}

#[test]
fn committed_inspector_changes_revoke_qualification_for_name_and_numeric_paths() {
    for (field, value) in [
        (rf_ui::UnitInspectorDraftField::Name, "New heater"),
        (rf_ui::UnitInspectorDraftField::OutletTemperatureK, "400"),
    ] {
        let mut app = app();
        let mut completed = execute(prepare(&mut app, false));
        let unit = UnitId::new("heater-1");
        app.focus_inspector_target(rf_ui::InspectorTarget::Unit(unit.clone()));
        app.update_unit_inspector_draft(&unit, field.clone(), value)
            .unwrap();
        app.commit_unit_inspector_draft(&unit, field, UNIX_EPOCH)
            .unwrap()
            .unwrap();
        rejected(&mut app, &mut completed, Reason::InputChanged);
    }
}

#[test]
fn package_component_topology_and_delete_restore_revoke_qualification() {
    for change in 0..4 {
        let mut app = app();
        let mut completed = execute(prepare(&mut app, false));
        match change {
            0 => {
                app.set_flowsheet_property_package_id(Some("another-package".into()), UNIX_EPOCH)
                    .unwrap();
            }
            1 => {
                app.add_flowsheet_component(rf_model::Component::new("extra", "Extra"), UNIX_EPOCH)
                    .unwrap();
            }
            2 => {
                app.disconnect_stream_connections(&"stream-feed".into(), UNIX_EPOCH)
                    .unwrap();
            }
            _ => {
                app.delete_unit(&"heater-1".into(), UNIX_EPOCH).unwrap();
                app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
            }
        }
        rejected(&mut app, &mut completed, Reason::InputChanged);
    }
}

#[test]
fn replay_after_history_pruning_and_clear_never_reuses_execution_identity() {
    let mut app = app();
    let mut first = execute(prepare(&mut app, false));
    complete_workspace_solve(&mut app, &mut first);
    let first_id = first.run_id().clone();
    for _ in 0..10 {
        let mut next = execute(prepare(&mut app, false));
        assert_ne!(next.run_id(), &first_id);
        complete_workspace_solve(&mut app, &mut next);
        app.workspace.apply_snapshot_history_limit(1);
    }
    rejected(&mut app, &mut first, Reason::AlreadySettled);
    app.workspace.clear_results();
    let next = prepare(&mut app, false);
    assert_eq!(next.request().sequence, 1);
    assert_ne!(next.run_id(), &first_id);
}

#[test]
fn solver_failure_retains_unit_context_but_never_marks_a_new_input_failed() {
    let mut app = app();
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&"heater-1".into())
        .unwrap()
        .parameters
        .outlet_temperature_k = Some(-1.0);
    let mut completed = execute(prepare(&mut app, false));
    let error = completed.result().unwrap_err();
    let context = SolveFailureContext::from_error(&error);
    assert!(context.related_unit_ids.contains(&"heater-1".into()));
    assert!(completed.package_descriptor().is_some());
    rename(&mut app, "new input");
    app.set_simulation_mode(SimulationMode::Active);
    rejected(&mut app, &mut completed, Reason::InputChanged);
}

#[test]
fn cached_preparation_errors_are_guarded_without_changing_legacy_status_semantics() {
    use rf_store::{
        StoredCredentialReference, StoredPropertyPackageRecord, StoredPropertyPackageSource,
    };
    let root = std::env::temp_dir().join(format!(
        "radishflow-af3a-missing-cache-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(!root.exists());
    for invalid_index in [false, true] {
        for revoked in [false, true] {
            let mut app = app();
            let mut index = StoredAuthCacheIndex::new(
                "https://id.radish.local",
                "user",
                StoredCredentialReference::new("studio", "user"),
            );
            let mut record = StoredPropertyPackageRecord::new(
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
                "2026.03.1",
                StoredPropertyPackageSource::RemoteDerivedPackage,
                "sha256:missing",
                128,
                UNIX_EPOCH,
            );
            record.expires_at = Some(SystemTime::now() + std::time::Duration::from_secs(3600));
            index.property_packages.push(record.clone());
            if invalid_index {
                index.property_packages.push(record);
            }
            let mut completed = prepare(&mut app, false).execute_from_auth_cache(&root, &index);
            assert_eq!(
                completed.result().unwrap_err().context().diagnostic_code(),
                Some(WORKSPACE_RUN_DIAGNOSTIC_LOCAL_CACHE_UNAVAILABLE)
            );
            if revoked {
                app.workspace.revoke_flow_run_acceptance();
                rejected(&mut app, &mut completed, Reason::AcceptanceRevoked);
            } else {
                let before = app.workspace.solve_session.clone();
                assert_eq!(
                    complete_workspace_solve(&mut app, &mut completed),
                    FlowRunDisposition::Applied
                );
                assert_eq!(app.workspace.solve_session, before);
                assert!(app.workspace.pending_flow_run().is_none());
                assert_eq!(app.log_feed.entries.len(), usize::from(!invalid_index));
                rejected(&mut app, &mut completed, Reason::AlreadySettled);
            }
        }
    }
    assert!(!root.exists(), "preparation must remain read-only");
}

#[test]
fn accepted_flow_query_carries_actual_execution_and_survives_cancelled_rerun() {
    use rf_ui::result_query::{FlowResultView, ResultDataOrigin};
    let mut app = app();
    let mut completed = execute(prepare(&mut app, false));
    let run_id = completed.run_id().clone();
    let fingerprint = completed.provenance().unwrap().payload_fingerprint.clone();
    assert_eq!(
        complete_workspace_solve(&mut app, &mut completed),
        FlowRunDisposition::Applied
    );
    let FlowResultView::Current { origin, .. } = app.workspace.flow_result() else {
        panic!("current")
    };
    let original = origin.clone();
    let execution = origin.execution().unwrap();
    assert_eq!(execution.run_id, run_id);
    let package = execution.package.as_ref().unwrap();
    assert_eq!(package.package_id, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID);
    assert_eq!(package.payload_fingerprint, fingerprint);
    assert_eq!(package.origin, ResultDataOrigin::InMemory);
    let mut pending = execute(prepare(&mut app, false));
    app.workspace.revoke_flow_run(pending.run_id());
    rejected(&mut app, &mut pending, Reason::AcceptanceRevoked);
    let FlowResultView::Current { origin, .. } = app.workspace.flow_result() else {
        panic!("current survives cancelled rerun")
    };
    assert_eq!(*origin, original);
}

#[test]
fn flow_query_rejects_foreign_snapshots_and_keeps_stale_origin_without_values() {
    use rf_ui::result_query::{FlowResultUnavailable, FlowResultView};
    let mut app = app();
    let mut completed = execute(prepare(&mut app, false));
    complete_workspace_solve(&mut app, &mut completed);
    let FlowResultView::Current { snapshot, origin } = app.workspace.flow_result() else {
        panic!("current")
    };
    let snapshot = snapshot.clone();
    let original = origin.clone();
    let mut other = app.clone();
    other.workspace.reset_flow_run_session();
    other.store_snapshot(snapshot);
    assert!(matches!(
        other.workspace.flow_result(),
        FlowResultView::Missing {
            reason: FlowResultUnavailable::DifferentSession
        }
    ));
    rename(&mut app, "Edited");
    let FlowResultView::Stale {
        origin: Some(origin),
        ..
    } = app.workspace.flow_result()
    else {
        panic!("stale")
    };
    assert_eq!(*origin, original);
    app.workspace.clear_results();
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Missing {
            reason: FlowResultUnavailable::NoResult
        }
    ));
}

#[test]
fn failure_query_is_qualified_and_late_failure_cannot_replace_new_evidence() {
    use rf_ui::result_query::FlowResultView;
    let mut app = app();
    let mut failed = execute(prepare(&mut app, true));
    let failed_id = failed.run_id().clone();
    complete_workspace_solve(&mut app, &mut failed);
    let FlowResultView::Failed { origin, .. } = app.workspace.flow_result() else {
        panic!("failed")
    };
    assert_eq!(origin.execution().unwrap().run_id, failed_id);
    assert!(origin.execution().unwrap().package.is_none());
    let review = crate::FlowResultReview::capture(&app.workspace);
    let guard = review.guard.as_ref().unwrap();
    assert!(guard.is_current(&app.workspace));
    let rows = crate::flow_origin_lines(review.origin.as_ref());
    assert!(
        rows.iter()
            .any(|(key, value)| key == "package" && value == "not loaded during preparation")
    );
    assert!(!rows.iter().any(|(key, _)| key == "payload_fingerprint"));
    rename(&mut app, "New input");
    assert!(!guard.is_current(&app.workspace));
    let mut good = execute(prepare(&mut app, false));
    complete_workspace_solve(&mut app, &mut good);
    let before = app.clone();
    complete_workspace_solve(&mut app, &mut failed);
    assert_eq!(app, before);
    assert!(matches!(
        app.workspace.flow_result(),
        FlowResultView::Current { .. }
    ));
}

#[test]
fn replacing_the_same_package_asset_never_rewrites_prior_result_evidence() {
    use rf_property_package::LoadedPropertyPackage;
    use rf_ui::result_query::FlowResultView;
    let mut app = app();
    let mut provider = build_official_binary_hydrocarbon_in_memory_provider(
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
    );
    let mut old = prepare(&mut app, false).execute_with_property_package(&provider);
    complete_workspace_solve(&mut app, &mut old);
    let FlowResultView::Current { origin, .. } = app.workspace.flow_result() else {
        panic!("current")
    };
    let original = origin.clone();
    let loaded = provider
        .load_package(OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID, SystemTime::now())
        .unwrap();
    let mut system = loaded.system().clone();
    system.components[0].vapor_heat_capacity_j_per_mol_k = Some(170.0);
    let replacement =
        LoadedPropertyPackage::from_runtime(loaded.manifest().clone(), system, SystemTime::now())
            .unwrap();
    assert_ne!(
        replacement.provenance().payload_fingerprint,
        loaded.provenance().payload_fingerprint
    );
    provider.insert(replacement);
    let FlowResultView::Current { origin, .. } = app.workspace.flow_result() else {
        panic!("current")
    };
    assert_eq!(*origin, original);
    let mut next = prepare(&mut app, false).execute_with_property_package(&provider);
    complete_workspace_solve(&mut app, &mut next);
    let FlowResultView::Current { origin, .. } = app.workspace.flow_result() else {
        panic!("current")
    };
    assert_ne!(
        origin
            .execution()
            .unwrap()
            .package
            .as_ref()
            .unwrap()
            .payload_fingerprint,
        original
            .execution()
            .unwrap()
            .package
            .as_ref()
            .unwrap()
            .payload_fingerprint
    );
    assert_eq!(
        app.workspace
            .snapshot_history
            .front()
            .unwrap()
            .origin
            .as_ref(),
        Some(&original)
    );
}
