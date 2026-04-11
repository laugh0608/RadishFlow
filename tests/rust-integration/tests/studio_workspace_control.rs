use std::fs;
use std::path::Path;

use radishflow_studio::{
    RunPanelWidgetDispatchOutcome, StudioAppAuthCacheContext, StudioAppFacade,
    StudioAppResultDispatch, StudioWorkspaceRunOutcome, WorkspaceControlAction,
    WorkspaceRunPackageSelection, apply_run_panel_recovery_action,
    dispatch_run_panel_primary_action_with_auth_cache,
    dispatch_workspace_control_action_with_auth_cache,
};
use rf_rust_integration::{sample_auth_cache_index, timestamp, unique_temp_path, write_cached_package};
use rf_store::parse_project_file_json;
use rf_ui::{AppState, DocumentMetadata, FlowsheetDocument, InspectorTarget, RunStatus};

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
        DocumentMetadata::new("doc-control-recovery", "Control Recovery Demo", timestamp(30)),
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
