use std::time::{Duration, SystemTime, UNIX_EPOCH};

use radishflow_studio::{StudioSolveRequest, solve_workspace_with_property_package};
use rf_rust_integration::build_binary_demo_package_provider;
use rf_store::parse_project_file_json;
use rf_thermo::InMemoryPropertyPackageProvider;
use rf_types::{StreamId, UnitId};
use rf_ui::{
    AppState, DocumentMetadata, FlowsheetDocument, RunPanelRecoveryActionKind, RunStatus,
};

fn timestamp(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

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
fn studio_solver_bridge_maps_project_snapshot_into_app_state_end_to_end() {
    let provider = build_binary_demo_package_provider();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        "doc-studio-success",
        "Studio Success Demo",
        10,
    );

    solve_workspace_with_property_package(
        &mut app_state,
        &provider,
        &StudioSolveRequest::new("binary-hydrocarbon-lite-v1", "snapshot-success-1", 1),
    )
    .expect("expected solve");

    assert_eq!(app_state.workspace.solve_session.status, RunStatus::Converged);
    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
        Some("snapshot-success-1")
    );
    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_summary.as_deref(),
        Some("solved flowsheet with 3 unit(s), 4 diagnostic entry(ies), and 4 resulting stream(s)")
    );
    assert!(app_state.workspace.run_panel.notice.is_none());

    let snapshot = app_state
        .workspace
        .snapshot_history
        .back()
        .expect("expected stored snapshot");
    assert_eq!(snapshot.id.as_str(), "snapshot-success-1");
    assert_eq!(snapshot.status, RunStatus::Converged);
    assert_eq!(
        snapshot.summary.primary_code.as_deref(),
        Some("solver.execution_order")
    );
    assert_eq!(
        snapshot.summary.related_unit_ids,
        vec![
            UnitId::new("feed-1"),
            UnitId::new("heater-1"),
            UnitId::new("flash-1"),
        ]
    );
    assert_eq!(snapshot.steps.len(), 3);
    assert_eq!(snapshot.steps[1].unit_id, UnitId::new("heater-1"));
    assert_eq!(snapshot.steps[1].streams.len(), 1);
    assert_eq!(snapshot.steps[1].streams[0].label, "stream-heated");
}

#[test]
fn studio_solver_bridge_records_solver_failure_notice_and_target_unit_end_to_end() {
    let provider = build_binary_demo_package_provider();
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
        DocumentMetadata::new("doc-studio-failure", "Studio Failure Demo", timestamp(20)),
    ));

    let error = solve_workspace_with_property_package(
        &mut app_state,
        &provider,
        &StudioSolveRequest::new("binary-hydrocarbon-lite-v1", "snapshot-failure-1", 1),
    )
    .expect_err("expected solve failure");

    assert!(error.message().contains("solver.step.execution:"));
    assert_eq!(app_state.workspace.solve_session.status, RunStatus::Error);
    let summary = app_state
        .workspace
        .solve_session
        .latest_diagnostic
        .as_ref()
        .expect("expected failure summary");
    assert_eq!(summary.primary_code.as_deref(), Some("solver.step.execution"));
    assert_eq!(summary.related_unit_ids, vec![UnitId::new("valve-1")]);

    let notice = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .expect("expected run panel notice");
    assert_eq!(notice.title, "Unit execution failed");
    assert_eq!(
        notice
            .recovery_action
            .as_ref()
            .map(|action| action.kind),
        Some(RunPanelRecoveryActionKind::InspectExecutionInputs)
    );
    assert_eq!(
        notice
            .recovery_action
            .as_ref()
            .and_then(|action| action.target_unit_id.as_ref()),
        Some(&UnitId::new("valve-1"))
    );
}

#[test]
fn studio_solver_bridge_records_missing_package_without_solver_code_end_to_end() {
    let provider = InMemoryPropertyPackageProvider::default();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        "doc-studio-missing-package",
        "Studio Missing Package Demo",
        30,
    );

    let error = solve_workspace_with_property_package(
        &mut app_state,
        &provider,
        &StudioSolveRequest::new("missing-package", "snapshot-missing-package-1", 1),
    )
    .expect_err("expected missing package failure");

    assert!(error.message().contains("missing property package"));
    assert_eq!(app_state.workspace.solve_session.status, RunStatus::Error);
    assert!(app_state.workspace.snapshot_history.is_empty());

    let summary = app_state
        .workspace
        .solve_session
        .latest_diagnostic
        .as_ref()
        .expect("expected failure summary");
    assert_eq!(summary.primary_code, None);
    assert!(summary.related_unit_ids.is_empty());

    let notice = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .expect("expected run panel notice");
    assert_eq!(notice.title, "Run failed");
    assert!(notice.recovery_action.is_none());
}

#[test]
fn studio_solver_bridge_records_cycle_failure_context_end_to_end() {
    let provider = build_binary_demo_package_provider();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/multi-unit-cycle.rfproj.json"),
        "doc-studio-cycle-failure",
        "Studio Cycle Failure Demo",
        40,
    );

    let error = solve_workspace_with_property_package(
        &mut app_state,
        &provider,
        &StudioSolveRequest::new("binary-hydrocarbon-lite-v1", "snapshot-cycle-failure-1", 1),
    )
    .expect_err("expected cycle failure");

    assert!(error.message().contains("solver.topological_ordering:"));
    assert_eq!(app_state.workspace.solve_session.status, RunStatus::Error);

    let summary = app_state
        .workspace
        .solve_session
        .latest_diagnostic
        .as_ref()
        .expect("expected failure summary");
    assert_eq!(
        summary.primary_code.as_deref(),
        Some("solver.topological_ordering")
    );
    assert_eq!(
        summary.related_unit_ids,
        vec![UnitId::new("heater-1"), UnitId::new("valve-1")]
    );

    let notice = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .expect("expected run panel notice");
    assert_eq!(notice.title, "Topological ordering failed");
    assert_eq!(
        notice
            .recovery_action
            .as_ref()
            .map(|action| action.kind),
        Some(RunPanelRecoveryActionKind::BreakCycle)
    );
    assert_eq!(
        notice
            .recovery_action
            .as_ref()
            .and_then(|action| action.target_unit_id.as_ref()),
        Some(&UnitId::new("heater-1"))
    );
}

#[test]
fn studio_solver_bridge_records_duplicate_sink_stream_target_end_to_end() {
    let provider = build_binary_demo_package_provider();
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/failures/duplicate-downstream-sink.rfproj.json"),
        "doc-studio-duplicate-sink-failure",
        "Studio Duplicate Sink Failure Demo",
        50,
    );

    let error = solve_workspace_with_property_package(
        &mut app_state,
        &provider,
        &StudioSolveRequest::new("binary-hydrocarbon-lite-v1", "snapshot-duplicate-sink-1", 1),
    )
    .expect_err("expected duplicate sink failure");

    assert!(error.message().contains("solver.connection_validation.duplicate_downstream_sink:"));
    assert_eq!(app_state.workspace.solve_session.status, RunStatus::Error);

    let summary = app_state
        .workspace
        .solve_session
        .latest_diagnostic
        .as_ref()
        .expect("expected failure summary");
    assert_eq!(
        summary.primary_code.as_deref(),
        Some("solver.connection_validation.duplicate_downstream_sink")
    );
    assert_eq!(summary.related_stream_ids, vec![StreamId::new("shared-stream")]);

    let notice = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .expect("expected run panel notice");
    assert_eq!(notice.title, "Duplicate stream sink");
    assert_eq!(
        notice
            .recovery_action
            .as_ref()
            .map(|action| action.kind),
        Some(RunPanelRecoveryActionKind::FixConnections)
    );
    assert_eq!(
        notice
            .recovery_action
            .as_ref()
            .and_then(|action| action.target_stream_id.as_ref()),
        Some(&StreamId::new("shared-stream"))
    );
}
