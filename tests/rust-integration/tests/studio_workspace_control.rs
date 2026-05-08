use std::fs;
use std::path::Path;

use radishflow_studio::{
    RunPanelWidgetDispatchOutcome, StudioAppAuthCacheContext, StudioAppCommand, StudioAppFacade,
    StudioAppResultDispatch, StudioWorkspaceRunOutcome, WorkspaceControlAction,
    WorkspaceRunCommand, WorkspaceRunPackageSelection, apply_run_panel_recovery_action,
    dispatch_run_panel_primary_action_with_auth_cache,
    dispatch_workspace_control_action_with_auth_cache,
};
use rf_rust_integration::{
    sample_auth_cache_index, timestamp, unique_temp_path, write_cached_package,
};
use rf_store::parse_project_file_json;
use rf_types::{PhaseEquilibriumRegion, StreamId, UnitId};
use rf_ui::{
    AppState, DocumentCommand, DocumentMetadata, EntitlementSnapshot, FlowsheetDocument,
    InspectorTarget, PropertyPackageManifest, PropertyPackageSource, RunPanelRecoveryActionKind,
    RunStatus, SimulationMode,
};

fn app_state_from_project(
    project_json: &str,
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
) -> AppState {
    let project = parse_project_file_json(project_json).expect("expected project parse");
    AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(document_id, title, timestamp(created_at_seconds)),
    ))
}

fn material_port_stream_id(app_state: &AppState, unit_id: &str, port_name: &str) -> Option<String> {
    app_state
        .workspace
        .document
        .flowsheet
        .units
        .get(&unit_id.into())
        .and_then(|unit| unit.ports.iter().find(|port| port.name == port_name))
        .and_then(|port| port.stream_id.as_ref())
        .map(|stream_id| stream_id.as_str().to_string())
}

fn stream_exists(app_state: &AppState, stream_id: &str) -> bool {
    app_state
        .workspace
        .document
        .flowsheet
        .streams
        .contains_key(&stream_id.into())
}

fn port_target_stream_id(app_state: &AppState, unit_id: &str, port_name: &str) -> Option<String> {
    material_port_stream_id(app_state, unit_id, port_name)
}

fn find_snapshot_stream<'a>(
    snapshot: &'a rf_ui::SolveSnapshot,
    stream_id: &str,
) -> &'a rf_ui::StreamStateSnapshot {
    snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == StreamId::new(stream_id))
        .expect("expected snapshot stream")
}

fn assert_two_phase_window_spans_ui_stream(stream: &rf_ui::StreamStateSnapshot) {
    let window = stream
        .bubble_dew_window
        .as_ref()
        .expect("expected bubble/dew window");

    assert_eq!(window.phase_region, PhaseEquilibriumRegion::TwoPhase);
    assert!(window.dew_pressure_pa < stream.pressure_pa);
    assert!(window.bubble_pressure_pa > stream.pressure_pa);
    assert!(window.bubble_temperature_k < stream.temperature_k);
    assert!(window.dew_temperature_k > stream.temperature_k);
}

fn sample_entitlement_snapshot(package_ids: &[&str]) -> EntitlementSnapshot {
    EntitlementSnapshot {
        schema_version: 1,
        subject_id: "user-123".to_string(),
        tenant_id: Some("tenant-radish".to_string()),
        issued_at: timestamp(100),
        expires_at: timestamp(200),
        offline_lease_expires_at: Some(timestamp(300)),
        features: std::collections::BTreeSet::from(["thermo.workspace.run".to_string()]),
        allowed_package_ids: package_ids.iter().map(|item| item.to_string()).collect(),
    }
}

fn sample_manifest(package_id: &str) -> PropertyPackageManifest {
    let mut manifest = PropertyPackageManifest::new(
        package_id,
        "2026.03.1",
        PropertyPackageSource::RemoteDerivedPackage,
    );
    manifest.hash = "sha256:test".to_string();
    manifest.size_bytes = 128;
    manifest.expires_at = Some(timestamp(300));
    manifest
}

#[test]
fn run_panel_primary_action_executes_workspace_run_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-primary");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        "doc-control-success",
        "Control Success Demo",
        10,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    let outcome =
        dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
            .expect("expected primary action dispatch");

    match outcome.dispatch {
        RunPanelWidgetDispatchOutcome::Executed(outcome) => match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert!(matches!(
                    dispatch.outcome,
                    StudioWorkspaceRunOutcome::Started(_)
                ));
            }
            _ => panic!("expected workspace run dispatch"),
        },
        _ => panic!("expected executed run panel outcome"),
    }
    assert_eq!(outcome.state.control_state.run_status, RunStatus::Converged);
    assert_eq!(
        outcome
            .state
            .control_state
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Run completed")
    );
    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
        Some("doc-control-success-rev-0-seq-1")
    );

    let snapshot = app_state
        .workspace
        .snapshot_history
        .back()
        .expect("expected stored snapshot");
    let heated = find_snapshot_stream(snapshot, "stream-heated");
    assert_two_phase_window_spans_ui_stream(heated);

    let flash_step = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == UnitId::new("flash-1"))
        .expect("expected flash step");
    assert_eq!(flash_step.consumed_streams.len(), 1);
    assert_eq!(
        flash_step.consumed_streams[0].stream_id,
        StreamId::new("stream-heated")
    );
    assert_eq!(
        flash_step.consumed_streams[0].bubble_dew_window,
        heated.bubble_dew_window
    );

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn workspace_control_reports_package_selection_required_end_to_end() {
    let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        "doc-control-blocked",
        "Control Blocked Demo",
        20,
    );
    let context = StudioAppAuthCacheContext::new(Path::new("D:\\cache-root"), &auth_cache_index);

    let outcome = dispatch_workspace_control_action_with_auth_cache(
        &facade,
        &mut app_state,
        &context,
        &WorkspaceControlAction::run_manual(WorkspaceRunPackageSelection::Preferred),
    )
    .expect("expected blocked dispatch");

    match outcome.dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => {
            assert!(matches!(
                dispatch.outcome,
                StudioWorkspaceRunOutcome::Blocked(_)
            ));
        }
        _ => panic!("expected workspace run dispatch"),
    }
    assert_eq!(outcome.control_state.run_status, RunStatus::Idle);
    assert_eq!(
        outcome
            .control_state
            .notice
            .as_ref()
            .map(|notice| (notice.title.as_str(), notice.message.as_str())),
        Some((
            "Package selection required",
            "multiple cached property packages are available; explicit package selection is required"
        ))
    );
}

#[test]
fn workspace_control_reports_entitlement_update_required_end_to_end() {
    let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        "doc-control-entitlement-blocked",
        "Control Entitlement Blocked Demo",
        25,
    );
    app_state.update_entitlement(
        sample_entitlement_snapshot(&["pkg-1"]),
        vec![sample_manifest("pkg-1")],
        timestamp(120),
    );
    let context = StudioAppAuthCacheContext::new(Path::new("D:\\cache-root"), &auth_cache_index);

    let outcome = dispatch_workspace_control_action_with_auth_cache(
        &facade,
        &mut app_state,
        &context,
        &WorkspaceControlAction::run_manual(WorkspaceRunPackageSelection::Explicit(
            "pkg-2".to_string(),
        )),
    )
    .expect("expected blocked dispatch");

    match outcome.dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => {
            assert!(matches!(
                dispatch.outcome,
                StudioWorkspaceRunOutcome::Blocked(_)
            ));
        }
        _ => panic!("expected workspace run dispatch"),
    }
    assert_eq!(outcome.control_state.run_status, RunStatus::Idle);
    assert_eq!(
        outcome
            .control_state
            .notice
            .as_ref()
            .map(|notice| (notice.title.as_str(), notice.message.as_str())),
        Some((
            "Entitlement update required",
            "workspace run package `pkg-2` is not present in entitlement manifests"
        ))
    );
}

#[test]
fn workspace_control_reports_local_cache_repair_notice_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-local-cache");
    let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        "doc-control-local-cache-failed",
        "Control Local Cache Failed Demo",
        28,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    let outcome = dispatch_workspace_control_action_with_auth_cache(
        &facade,
        &mut app_state,
        &context,
        &WorkspaceControlAction::run_manual(WorkspaceRunPackageSelection::Explicit(
            "pkg-1".to_string(),
        )),
    )
    .expect("expected failed dispatch");

    match outcome.dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => {
            assert!(matches!(
                dispatch.outcome,
                StudioWorkspaceRunOutcome::Failed(_)
            ));
        }
        _ => panic!("expected workspace run dispatch"),
    }
    assert_eq!(outcome.control_state.run_status, RunStatus::Error);
    let notice = outcome
        .control_state
        .notice
        .as_ref()
        .expect("expected local cache notice");
    assert_eq!(notice.title, "Local cache unavailable");
    assert!(
        notice
            .message
            .contains("failed to prepare local property package cache")
    );
    assert_eq!(
        notice
            .recovery_action
            .as_ref()
            .map(|action| (action.kind, action.title)),
        Some((
            RunPanelRecoveryActionKind::RepairLocalCache,
            "Repair local cache"
        ))
    );

    fs::remove_dir_all(cache_root).ok();
}

#[test]
fn automatic_workspace_run_executes_after_document_revision_advances_end_to_end() {
    let cache_root = unique_temp_path("integration-automatic-workspace-run");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        "doc-control-auto-run",
        "Control Automatic Run Demo",
        29,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    let mode = facade
        .execute_with_auth_cache(
            &mut app_state,
            &context,
            &StudioAppCommand::set_workspace_simulation_mode(SimulationMode::Active),
        )
        .expect("expected mode activation");
    match mode.dispatch {
        StudioAppResultDispatch::WorkspaceMode(dispatch) => {
            assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
            assert_eq!(
                dispatch.pending_reason,
                Some(rf_ui::SolvePendingReason::ModeActivated)
            );
        }
        _ => panic!("expected workspace mode dispatch"),
    }

    let mut next_flowsheet = app_state.workspace.document.flowsheet.clone();
    next_flowsheet
        .units
        .get_mut(&"heater-1".into())
        .expect("expected heater unit")
        .name = "Heater Updated".to_string();
    app_state.commit_document_change(
        DocumentCommand::RenameUnit {
            unit_id: "heater-1".into(),
            new_name: "Heater Updated".to_string(),
        },
        next_flowsheet,
        timestamp(30),
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Dirty);
    assert_eq!(
        app_state.workspace.run_panel.pending_reason,
        Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
    );

    let automatic = facade
        .execute_with_auth_cache(
            &mut app_state,
            &context,
            &StudioAppCommand::run_workspace(WorkspaceRunCommand::automatic_preferred()),
        )
        .expect("expected automatic run dispatch");

    match automatic.dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => {
            assert_eq!(
                dispatch.package_id.as_deref(),
                Some("binary-hydrocarbon-lite-v1")
            );
            assert!(matches!(
                dispatch.outcome,
                StudioWorkspaceRunOutcome::Started(_)
            ));
            assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
            assert_eq!(dispatch.pending_reason, None);
            assert_eq!(
                dispatch.latest_snapshot_id.as_deref(),
                Some("doc-control-auto-run-rev-1-seq-1")
            );
        }
        _ => panic!("expected workspace run dispatch"),
    }
    assert_eq!(
        app_state.workspace.run_panel.run_status,
        RunStatus::Converged
    );
    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
        Some("doc-control-auto-run-rev-1-seq-1")
    );

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn automatic_workspace_run_skips_before_package_resolution_when_no_pending_request_end_to_end() {
    let cache_root = unique_temp_path("integration-automatic-workspace-skip");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v2",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        "doc-control-auto-skip",
        "Control Automatic Skip Demo",
        31,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_workspace_control_action_with_auth_cache(
        &facade,
        &mut app_state,
        &context,
        &WorkspaceControlAction::set_mode(SimulationMode::Active),
    )
    .expect("expected active mode dispatch");
    let first = dispatch_workspace_control_action_with_auth_cache(
        &facade,
        &mut app_state,
        &context,
        &WorkspaceControlAction::run_manual(WorkspaceRunPackageSelection::Explicit(
            "binary-hydrocarbon-lite-v1".to_string(),
        )),
    )
    .expect("expected successful explicit run");
    assert_eq!(first.control_state.run_status, RunStatus::Converged);
    assert_eq!(app_state.workspace.run_panel.pending_reason, None);
    assert_eq!(
        app_state.workspace.run_panel.simulation_mode,
        SimulationMode::Active
    );

    let automatic = facade
        .execute_with_auth_cache(
            &mut app_state,
            &context,
            &StudioAppCommand::run_workspace(WorkspaceRunCommand::automatic_preferred()),
        )
        .expect("expected automatic skip dispatch");

    match automatic.dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => {
            assert_eq!(dispatch.package_id, None);
            assert!(matches!(
                dispatch.outcome,
                StudioWorkspaceRunOutcome::Skipped(
                    radishflow_studio::WorkspaceSolveSkipReason::NoPendingRequest
                )
            ));
            assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
            assert_eq!(dispatch.pending_reason, None);
            assert_eq!(
                dispatch.latest_snapshot_id.as_deref(),
                Some("doc-control-auto-skip-rev-0-seq-1")
            );
        }
        _ => panic!("expected workspace run dispatch"),
    }
    assert_eq!(
        app_state.workspace.run_panel.run_status,
        RunStatus::Converged
    );
    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
        Some("doc-control-auto-skip-rev-0-seq-1")
    );

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn workspace_control_failed_rerun_clears_previous_snapshot_summary_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-local-cache-rerun-failure");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        "doc-control-local-cache-rerun-failure",
        "Control Local Cache Rerun Failure Demo",
        28,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    let first =
        dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
            .expect("expected first successful run");
    assert_eq!(first.state.control_state.run_status, RunStatus::Converged);
    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
        Some("doc-control-local-cache-rerun-failure-rev-0-seq-1")
    );

    fs::remove_dir_all(&cache_root).expect("expected temp dir cleanup before rerun");

    let rerun = dispatch_workspace_control_action_with_auth_cache(
        &facade,
        &mut app_state,
        &context,
        &WorkspaceControlAction::run_manual(WorkspaceRunPackageSelection::Explicit(
            "binary-hydrocarbon-lite-v1".to_string(),
        )),
    )
    .expect("expected failed rerun dispatch");

    match rerun.dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => {
            assert!(matches!(
                dispatch.outcome,
                StudioWorkspaceRunOutcome::Failed(_)
            ));
            assert_eq!(dispatch.latest_snapshot_id, None);
            assert_eq!(dispatch.latest_snapshot_summary, None);
        }
        _ => panic!("expected workspace run dispatch"),
    }
    assert_eq!(rerun.control_state.run_status, RunStatus::Error);
    assert_eq!(rerun.control_state.latest_snapshot_id, None);
    assert_eq!(rerun.control_state.latest_snapshot_summary, None);
    assert_eq!(app_state.workspace.run_panel.latest_snapshot_id, None);
    assert_eq!(app_state.workspace.run_panel.latest_snapshot_summary, None);
    assert_eq!(
        app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Local cache unavailable")
    );
}

#[test]
fn run_panel_recovery_action_focuses_failed_unit_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let project = parse_project_file_json(include_str!(
        "../../../examples/flowsheets/feed-valve-flash.rfproj.json"
    ))
    .expect("expected project parse");
    let mut flowsheet = project.document.flowsheet;
    flowsheet
        .streams
        .get_mut(&"stream-throttled".into())
        .expect("expected throttled stream")
        .pressure_pa = 130_000.0;
    let mut app_state = AppState::new(FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new(
            "doc-control-recovery",
            "Control Recovery Demo",
            timestamp(30),
        ),
    ));
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected failed primary action dispatch");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Inspect unit inputs");
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("valve-1".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("valve-1".into()))
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"valve-1".into())
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_restores_invalid_port_signature_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-connection-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/invalid-port-signature.rfproj.json"),
        "doc-control-connection-recovery",
        "Control Connection Recovery Demo",
        35,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Restore canonical ports");
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("feed-1".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("feed-1".into()))
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"feed-1".into())
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        port_target_stream_id(&app_state, "feed-1", "outlet").as_deref(),
        Some("stream-feed")
    );
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::RestoreCanonicalUnitPorts {
            unit_id: "feed-1".into(),
        })
    );
    assert!(
        app_state
            .workspace
            .document
            .flowsheet
            .units
            .get(&"feed-1".into())
            .and_then(|unit| unit.ports.iter().find(|port| port.name == "unexpected"))
            .is_none()
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_restores_invalid_port_signature_and_reruns_successfully() {
    let cache_root = unique_temp_path("integration-run-panel-connection-recovery-rerun");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/invalid-port-signature.rfproj.json"),
        "doc-control-connection-recovery-rerun",
        "Control Connection Recovery Rerun Demo",
        35,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Restore canonical ports");
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Dirty);
    assert_eq!(
        app_state.workspace.run_panel.pending_reason,
        Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
    );
    assert_eq!(app_state.workspace.run_panel.latest_snapshot_id, None);

    let rerun =
        dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
            .expect("expected successful rerun after recovery");

    match rerun.dispatch {
        RunPanelWidgetDispatchOutcome::Executed(outcome) => match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert!(matches!(
                    dispatch.outcome,
                    StudioWorkspaceRunOutcome::Started(_)
                ));
                assert_eq!(
                    dispatch.latest_snapshot_id.as_deref(),
                    Some("doc-control-connection-recovery-rerun-rev-1-seq-1")
                );
            }
            _ => panic!("expected workspace run dispatch"),
        },
        _ => panic!("expected executed rerun outcome"),
    }
    assert_eq!(rerun.state.control_state.run_status, RunStatus::Converged);
    assert_eq!(
        rerun.state.control_state.latest_snapshot_id.as_deref(),
        Some("doc-control-connection-recovery-rerun-rev-1-seq-1")
    );
    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
        Some("doc-control-connection-recovery-rerun-rev-1-seq-1")
    );

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_focuses_missing_upstream_source_unit_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-missing-upstream-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/missing-upstream-source.rfproj.json"),
        "doc-control-missing-upstream-recovery",
        "Control Missing Upstream Recovery Demo",
        36,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Remove dangling inlet stream");
    assert_eq!(recovery.action.target_port_name.as_deref(), Some("inlet_a"));
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("mixer-1".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("mixer-1".into()))
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"mixer-1".into())
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        material_port_stream_id(&app_state, "mixer-1", "inlet_a"),
        None
    );
    assert!(!stream_exists(&app_state, "stream-feed-a"));
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPortAndDeleteStream {
            unit_id: "mixer-1".into(),
            port: "inlet_a".to_string(),
            stream_id: "stream-feed-a".into(),
        })
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_disconnects_self_loop_inlet_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-self-loop-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/self-loop-cycle.rfproj.json"),
        "doc-control-self-loop-recovery",
        "Control Self Loop Recovery Demo",
        36,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected self-loop failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Disconnect self-loop inlet");
    assert_eq!(recovery.action.target_port_name.as_deref(), Some("inlet"));
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("flash-1".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("flash-1".into()))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        material_port_stream_id(&app_state, "flash-1", "inlet"),
        None
    );
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPorts {
            unit_id: "flash-1".into(),
            port: "inlet".to_string(),
        })
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"flash-1".into())
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_disconnects_two_unit_cycle_inlet_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-two-unit-cycle-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/multi-unit-cycle.rfproj.json"),
        "doc-control-two-unit-cycle-recovery",
        "Control Two Unit Cycle Recovery Demo",
        36,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected two-unit cycle failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Disconnect cycle inlet");
    assert_eq!(recovery.action.target_port_name.as_deref(), Some("inlet"));
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("heater-1".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("heater-1".into()))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        material_port_stream_id(&app_state, "heater-1", "inlet"),
        None
    );
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPorts {
            unit_id: "heater-1".into(),
            port: "inlet".to_string(),
        })
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"heater-1".into())
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_disconnects_missing_stream_reference_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-missing-stream-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/missing-stream-reference.rfproj.json"),
        "doc-control-missing-stream-recovery",
        "Control Missing Stream Recovery Demo",
        37,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Disconnect invalid stream reference");
    assert_eq!(recovery.action.target_port_name.as_deref(), Some("outlet"));
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("heater-1".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("heater-1".into()))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        material_port_stream_id(&app_state, "heater-1", "outlet"),
        None
    );
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPorts {
            unit_id: "heater-1".into(),
            port: "outlet".to_string(),
        })
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"heater-1".into())
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_disconnects_duplicate_upstream_source_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-duplicate-source-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/duplicate-upstream-source.rfproj.json"),
        "doc-control-duplicate-source-recovery",
        "Control Duplicate Source Recovery Demo",
        38,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Disconnect conflicting source");
    assert_eq!(recovery.action.target_port_name.as_deref(), Some("outlet"));
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("feed-2".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("feed-2".into()))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        material_port_stream_id(&app_state, "feed-2", "outlet"),
        None
    );
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPorts {
            unit_id: "feed-2".into(),
            port: "outlet".to_string(),
        })
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"feed-2".into())
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_disconnects_duplicate_downstream_sink_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-duplicate-sink-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/duplicate-downstream-sink.rfproj.json"),
        "doc-control-duplicate-sink-recovery",
        "Control Duplicate Sink Recovery Demo",
        37,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Disconnect conflicting sink");
    assert_eq!(recovery.action.target_port_name.as_deref(), Some("inlet_a"));
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("mixer-1".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("mixer-1".into()))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        material_port_stream_id(&app_state, "mixer-1", "inlet_a"),
        None
    );
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DisconnectPorts {
            unit_id: "mixer-1".into(),
            port: "inlet_a".to_string(),
        })
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"mixer-1".into())
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_deletes_orphan_stream_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-orphan-stream-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/orphan-stream.rfproj.json"),
        "doc-control-orphan-stream-recovery",
        "Control Orphan Stream Recovery Demo",
        39,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Delete orphan stream");
    assert_eq!(
        recovery
            .action
            .target_stream_id
            .as_ref()
            .map(|stream_id| stream_id.as_str()),
        Some("stream-orphan")
    );
    assert_eq!(recovery.applied_target, None);
    assert_eq!(app_state.workspace.document.revision, 1);
    assert!(!stream_exists(&app_state, "stream-orphan"));
    assert!(app_state.workspace.selection.selected_units.is_empty());
    assert!(app_state.workspace.selection.selected_streams.is_empty());
    assert_eq!(app_state.workspace.drafts.active_target, None);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::DeleteStream {
            stream_id: "stream-orphan".into(),
        })
    );

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_creates_stream_for_unbound_outlet_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-unbound-outlet-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/unbound-outlet-port.rfproj.json"),
        "doc-control-unbound-outlet-recovery",
        "Control Unbound Outlet Recovery Demo",
        40,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Create outlet stream");
    assert_eq!(recovery.action.target_port_name.as_deref(), Some("outlet"));
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("feed-1".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("feed-1".into()))
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        port_target_stream_id(&app_state, "feed-1", "outlet").as_deref(),
        Some("stream-feed-1-outlet")
    );
    assert!(stream_exists(&app_state, "stream-feed-1-outlet"));
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&DocumentCommand::ConnectPorts {
            stream_id: "stream-feed-1-outlet".into(),
            from_unit_id: "feed-1".into(),
            from_port: "outlet".to_string(),
            to_unit_id: None,
            to_port: None,
        })
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"feed-1".into())
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_creates_stream_for_unbound_outlet_and_reruns_successfully() {
    let cache_root = unique_temp_path("integration-run-panel-unbound-outlet-recovery-rerun");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/unbound-outlet-port.rfproj.json"),
        "doc-control-unbound-outlet-recovery-rerun",
        "Control Unbound Outlet Recovery Rerun Demo",
        40,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Create outlet stream");
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Dirty);
    assert_eq!(
        app_state.workspace.run_panel.pending_reason,
        Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
    );
    assert_eq!(app_state.workspace.run_panel.latest_snapshot_id, None);

    let rerun =
        dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
            .expect("expected successful rerun after recovery");

    match rerun.dispatch {
        RunPanelWidgetDispatchOutcome::Executed(outcome) => match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert!(matches!(
                    dispatch.outcome,
                    StudioWorkspaceRunOutcome::Started(_)
                ));
                assert_eq!(
                    dispatch.latest_snapshot_id.as_deref(),
                    Some("doc-control-unbound-outlet-recovery-rerun-rev-1-seq-1")
                );
            }
            _ => panic!("expected workspace run dispatch"),
        },
        _ => panic!("expected executed rerun outcome"),
    }
    assert_eq!(rerun.state.control_state.run_status, RunStatus::Converged);
    assert_eq!(
        rerun.state.control_state.latest_snapshot_id.as_deref(),
        Some("doc-control-unbound-outlet-recovery-rerun-rev-1-seq-1")
    );
    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
        Some("doc-control-unbound-outlet-recovery-rerun-rev-1-seq-1")
    );

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}

#[test]
fn run_panel_recovery_action_focuses_unbound_inlet_port_end_to_end() {
    let cache_root = unique_temp_path("integration-run-panel-unbound-inlet-recovery");
    let mut auth_cache_index = sample_auth_cache_index(&[]);
    write_cached_package(
        &cache_root,
        &mut auth_cache_index,
        "binary-hydrocarbon-lite-v1",
    );
    let facade = StudioAppFacade::new();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/unbound-inlet-port.rfproj.json"),
        "doc-control-unbound-inlet-recovery",
        "Control Unbound Inlet Recovery Demo",
        41,
    );
    let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

    dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
        .expect("expected connection validation failure");

    let recovery =
        apply_run_panel_recovery_action(&mut app_state).expect("expected recovery action");

    assert_eq!(recovery.action.title, "Inspect inlet path");
    assert_eq!(recovery.action.target_port_name.as_deref(), Some("inlet"));
    assert_eq!(
        recovery.applied_target,
        Some(InspectorTarget::Unit("heater-1".into()))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit("heater-1".into()))
    );
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(
        material_port_stream_id(&app_state, "heater-1", "inlet"),
        None
    );
    assert!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .is_none()
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&"heater-1".into())
    );
    assert!(app_state.workspace.panels.inspector_open);

    fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
}
