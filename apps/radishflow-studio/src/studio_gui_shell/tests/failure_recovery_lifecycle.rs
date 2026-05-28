use super::*;

#[test]
fn stream_input_failure_recovery_saves_reopens_and_reruns_official_case() {
    let project_path = temporary_failure_project_path("stream-input-recovery");
    let mut project = rf_store::parse_project_file_json(include_str!(
        "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .expect("expected heater flash project fixture");
    project
        .document
        .flowsheet
        .streams
        .get_mut(&rf_types::StreamId::new("stream-feed"))
        .expect("expected feed stream")
        .overall_mole_fractions
        .clear();
    write_project_file(&project_path, &project).expect("expected temp project write");
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Stream input invalid",
            primary_code: "solver.step.stream_input",
            recovery_title: Some("Inspect stream inputs"),
            recovery_target: Some(("Stream", "stream-feed")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_active_inspector_target(&app, "Stream", "stream-feed");

    for component_id in ["methane", "ethane"] {
        let window = app.platform_host.snapshot().window_model();
        let detail = window
            .runtime
            .active_inspector_detail
            .as_ref()
            .expect("expected active stream inspector");
        let command_id = detail
            .property_composition_component_actions
            .iter()
            .find(|action| action.component_id == component_id)
            .unwrap_or_else(|| panic!("expected {component_id} composition action"))
            .action
            .command_id
            .clone();
        app.dispatch_inspector_composition_component_add(command_id);
    }

    for (component_id, raw_value) in [("methane", "0.2"), ("ethane", "0.6")] {
        app.dispatch_inspector_field_draft_update(
            radishflow_studio::inspector_draft_update_command_id(&format!(
                "stream:stream-feed:overall_mole_fraction:{component_id}"
            )),
            raw_value,
        );
    }
    let detail = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .active_inspector_detail
        .expect("expected stream inspector after composition draft");
    let normalize_command_id = detail
        .property_composition_normalize_command_id
        .expect("expected composition normalize command");
    app.dispatch_inspector_composition_normalize(normalize_command_id);

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    let saved_feed = &saved.document.flowsheet.streams[&rf_types::StreamId::new("stream-feed")];
    assert_close(
        saved_feed.overall_mole_fractions[&rf_types::ComponentId::new("methane")],
        0.25,
    );
    assert_close(
        saved_feed.overall_mole_fractions[&rf_types::ComponentId::new("ethane")],
        0.75,
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_converged(&mut app);
    assert_latest_stream_fraction(&app, "stream-feed", "methane", 0.25);
    assert_latest_stream_fraction(&app, "stream-feed", "ethane", 0.75);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn unit_parameter_failure_recovery_saves_reopens_and_reruns_official_case() {
    let project_path = temporary_failure_project_path("unit-parameter-recovery");
    let mut project = rf_store::parse_project_file_json(include_str!(
        "../../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
    ))
    .expect("expected valve flash project fixture");
    project
        .document
        .flowsheet
        .units
        .get_mut(&UnitId::new("valve-1"))
        .expect("expected valve unit")
        .parameters
        .outlet_pressure_pa = Some(730_000.0);
    project
        .document
        .flowsheet
        .streams
        .get_mut(&StreamId::new("stream-throttled"))
        .expect("expected valve outlet stream")
        .pressure_pa = 730_000.0;
    write_project_file(&project_path, &project).expect("expected temp project write");
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unit parameter invalid",
            primary_code: "solver.step.parameter",
            recovery_title: Some("Inspect unit parameters"),
            recovery_target: Some(("Unit", "valve-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_active_inspector_target(&app, "Unit", "valve-1");

    commit_unit_parameter(
        &mut app,
        "valve-1",
        "unit:valve-1:outlet_pressure_pa",
        "650000",
    );
    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        saved.document.flowsheet.units[&UnitId::new("valve-1")]
            .parameters
            .outlet_pressure_pa,
        Some(650_000.0)
    );
    assert_eq!(
        saved.document.flowsheet.streams[&StreamId::new("stream-throttled")].pressure_pa,
        650_000.0
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_converged(&mut app);
    assert_latest_stream_pressure(&app, "stream-throttled", 650_000.0);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn unbound_outlet_failure_recovery_saves_reopens_and_reruns_official_case() {
    let project_path = temporary_failure_project_path("unbound-outlet-recovery");
    write_fixture_project(
        &project_path,
        include_str!("../../../../../examples/flowsheets/failures/unbound-outlet-port.rfproj.json"),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unbound outlet port",
            primary_code: "solver.connection_validation.unbound_outlet_port",
            recovery_title: Some("Create outlet stream"),
            recovery_target: Some(("Unit", "feed-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "feed-1");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "feed-1", "outlet"),
        Some("stream-feed-1-outlet")
    );
    assert!(
        saved
            .document
            .flowsheet
            .streams
            .contains_key(&StreamId::new("stream-feed-1-outlet"))
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_converged(&mut app);
    assert_latest_snapshot_has_stream(&app, "stream-feed-1-outlet");

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn missing_stream_reference_recovery_saves_reopens_and_completes_after_outlet_creation() {
    let project_path = temporary_failure_project_path("missing-stream-reference-recovery");
    write_fixture_project(
        &project_path,
        include_str!(
            "../../../../../examples/flowsheets/failures/missing-stream-reference.rfproj.json"
        ),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Missing stream reference",
            primary_code: "solver.connection_validation.missing_stream_reference",
            recovery_title: Some("Disconnect invalid stream reference"),
            recovery_target: Some(("Unit", "heater-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "heater-1");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "heater-1", "outlet"),
        None
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unbound outlet port",
            primary_code: "solver.connection_validation.unbound_outlet_port",
            recovery_title: Some("Create outlet stream"),
            recovery_target: Some(("Unit", "heater-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "heater-1");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "heater-1", "outlet"),
        Some("stream-heater-1-outlet")
    );
    assert!(
        saved
            .document
            .flowsheet
            .streams
            .contains_key(&StreamId::new("stream-heater-1-outlet"))
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_converged(&mut app);
    assert_latest_snapshot_has_stream(&app, "stream-heater-1-outlet");

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn duplicate_source_recovery_saves_reopens_and_completes_after_outlet_creation() {
    let project_path = temporary_failure_project_path("duplicate-source-recovery");
    write_fixture_project(
        &project_path,
        include_str!(
            "../../../../../examples/flowsheets/failures/duplicate-upstream-source.rfproj.json"
        ),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Duplicate stream source",
            primary_code: "solver.connection_validation.duplicate_upstream_source",
            recovery_title: Some("Disconnect conflicting source"),
            recovery_target: Some(("Unit", "feed-2")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "feed-2");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(stored_unit_port_stream_id(&saved, "feed-2", "outlet"), None);

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unbound outlet port",
            primary_code: "solver.connection_validation.unbound_outlet_port",
            recovery_title: Some("Create outlet stream"),
            recovery_target: Some(("Unit", "feed-2")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "feed-2");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "feed-2", "outlet"),
        Some("stream-feed-2-outlet")
    );
    assert!(
        saved
            .document
            .flowsheet
            .streams
            .contains_key(&StreamId::new("stream-feed-2-outlet"))
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_converged(&mut app);
    assert_latest_snapshot_has_stream(&app, "stream-feed-2-outlet");
    assert_latest_snapshot_has_stream(&app, "stream-out");

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn orphan_stream_recovery_saves_reopens_and_reruns_official_case() {
    let project_path = temporary_failure_project_path("orphan-stream-recovery");
    write_fixture_project(
        &project_path,
        include_str!("../../../../../examples/flowsheets/failures/orphan-stream.rfproj.json"),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Orphan stream",
            primary_code: "solver.connection_validation.orphan_stream",
            recovery_title: Some("Delete orphan stream"),
            recovery_target: Some(("Stream", "stream-orphan")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert!(
        !saved
            .document
            .flowsheet
            .streams
            .contains_key(&StreamId::new("stream-orphan"))
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_converged(&mut app);
    assert_latest_snapshot_missing_stream(&app, "stream-orphan");

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn duplicate_sink_recovery_saves_reopens_and_exposes_unbound_inlet_path() {
    let project_path = temporary_failure_project_path("duplicate-sink-recovery");
    write_fixture_project(
        &project_path,
        include_str!(
            "../../../../../examples/flowsheets/failures/duplicate-downstream-sink.rfproj.json"
        ),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Duplicate stream sink",
            primary_code: "solver.connection_validation.duplicate_downstream_sink",
            recovery_title: Some("Disconnect conflicting sink"),
            recovery_target: Some(("Unit", "mixer-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "mixer-1");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "mixer-1", "inlet_a"),
        None
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unbound inlet port",
            primary_code: "solver.connection_validation.unbound_inlet_port",
            recovery_title: Some("Inspect inlet path"),
            recovery_target: Some(("Unit", "mixer-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_error_after_focus_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "mixer-1");

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn missing_upstream_recovery_saves_reopens_and_exposes_unbound_inlet_path() {
    let project_path = temporary_failure_project_path("missing-upstream-recovery");
    write_fixture_project(
        &project_path,
        include_str!(
            "../../../../../examples/flowsheets/failures/missing-upstream-source.rfproj.json"
        ),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Missing upstream source",
            primary_code: "solver.connection_validation.missing_upstream_source",
            recovery_title: Some("Remove dangling inlet stream"),
            recovery_target: Some(("Unit", "mixer-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "mixer-1");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "mixer-1", "inlet_a"),
        None
    );
    assert!(
        !saved
            .document
            .flowsheet
            .streams
            .contains_key(&StreamId::new("stream-feed-a"))
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unbound inlet port",
            primary_code: "solver.connection_validation.unbound_inlet_port",
            recovery_title: Some("Inspect inlet path"),
            recovery_target: Some(("Unit", "mixer-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_error_after_focus_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "mixer-1");

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn unbound_inlet_recovery_focuses_without_mutating_saved_project() {
    let project_path = temporary_failure_project_path("unbound-inlet-recovery");
    write_fixture_project(
        &project_path,
        include_str!("../../../../../examples/flowsheets/failures/unbound-inlet-port.rfproj.json"),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unbound inlet port",
            primary_code: "solver.connection_validation.unbound_inlet_port",
            recovery_title: Some("Inspect inlet path"),
            recovery_target: Some(("Unit", "heater-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_error_after_focus_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "heater-1");

    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(saved.document.revision, 0);
    assert_eq!(
        stored_unit_port_stream_id(&saved, "heater-1", "inlet"),
        None
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unbound inlet port",
            primary_code: "solver.connection_validation.unbound_inlet_port",
            recovery_title: Some("Inspect inlet path"),
            recovery_target: Some(("Unit", "heater-1")),
        },
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn invalid_port_signature_recovery_saves_reopens_and_reruns_official_case() {
    let project_path = temporary_failure_project_path("invalid-port-signature-recovery");
    write_fixture_project(
        &project_path,
        include_str!(
            "../../../../../examples/flowsheets/failures/invalid-port-signature.rfproj.json"
        ),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Invalid port signature",
            primary_code: "solver.connection_validation.invalid_port_signature",
            recovery_title: Some("Restore canonical ports"),
            recovery_target: Some(("Unit", "feed-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "feed-1");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "feed-1", "outlet"),
        Some("stream-feed")
    );
    assert!(
        saved
            .document
            .flowsheet
            .units
            .get(&UnitId::new("feed-1"))
            .expect("expected feed unit")
            .ports
            .iter()
            .all(|port| port.name != "unexpected")
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_converged(&mut app);
    assert_latest_snapshot_has_stream(&app, "stream-feed");

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn self_loop_recovery_saves_reopens_and_exposes_unbound_inlet_path() {
    let project_path = temporary_failure_project_path("self-loop-recovery");
    write_fixture_project(
        &project_path,
        include_str!("../../../../../examples/flowsheets/failures/self-loop-cycle.rfproj.json"),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Self loop detected",
            primary_code: "solver.topological_ordering.self_loop_cycle",
            recovery_title: Some("Disconnect self-loop inlet"),
            recovery_target: Some(("Unit", "flash-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "flash-1");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(stored_unit_port_stream_id(&saved, "flash-1", "inlet"), None);

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unbound inlet port",
            primary_code: "solver.connection_validation.unbound_inlet_port",
            recovery_title: Some("Inspect inlet path"),
            recovery_target: Some(("Unit", "flash-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_error_after_focus_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "flash-1");

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn two_unit_cycle_recovery_saves_reopens_and_exposes_unbound_inlet_path() {
    let project_path = temporary_failure_project_path("two-unit-cycle-recovery");
    write_fixture_project(
        &project_path,
        include_str!("../../../../../examples/flowsheets/failures/multi-unit-cycle.rfproj.json"),
    );
    let mut app = ready_app_state(&project_config(project_path.clone()));

    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Two-unit cycle detected",
            primary_code: "solver.topological_ordering.two_unit_cycle",
            recovery_title: Some("Disconnect cycle inlet"),
            recovery_target: Some(("Unit", "heater-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_dirty_after_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "heater-1");

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "heater-1", "inlet"),
        None
    );

    reopen_and_assert_clean(&mut app, &project_path);
    run_and_assert_failure(
        &mut app,
        ExpectedFailure {
            title: "Unbound inlet port",
            primary_code: "solver.connection_validation.unbound_inlet_port",
            recovery_title: Some("Inspect inlet path"),
            recovery_target: Some(("Unit", "heater-1")),
        },
    );

    app.dispatch_ui_command("run_panel.recover_failure");
    assert_error_after_focus_recovery(&app);
    assert_active_inspector_target(&app, "Unit", "heater-1");

    let _ = std::fs::remove_file(project_path);
}

#[derive(Debug, Clone, Copy)]
struct ExpectedFailure<'a> {
    title: &'a str,
    primary_code: &'a str,
    recovery_title: Option<&'a str>,
    recovery_target: Option<(&'a str, &'a str)>,
}

fn temporary_failure_project_path(name: &str) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "radishflow-studio-shell-failure-recovery-{name}-{timestamp}.rfproj.json"
    ))
}

fn project_config(project_path: PathBuf) -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        project_path,
        ..synced_workspace_config()
    }
}

fn write_fixture_project(project_path: &std::path::Path, project_json: &str) {
    let project =
        rf_store::parse_project_file_json(project_json).expect("expected failure project fixture");
    write_project_file(project_path, &project).expect("expected temp project write");
}

fn run_and_assert_failure(app: &mut ReadyAppState, expected: ExpectedFailure<'_>) {
    app.dispatch_ui_command("run_panel.run_manual");
    let failed = app.platform_host.snapshot().window_model();
    assert_eq!(
        failed.runtime.control_state.run_status,
        rf_ui::RunStatus::Error
    );
    let failure = failed
        .runtime
        .latest_failure
        .as_ref()
        .expect("expected run failure");
    assert_eq!(failure.title, expected.title);
    assert_eq!(
        failure
            .diagnostic_detail
            .as_ref()
            .and_then(|detail| detail.primary_code.as_deref()),
        Some(expected.primary_code)
    );
    assert_eq!(failure.recovery_title, expected.recovery_title);
    assert_eq!(
        failure
            .recovery_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        expected.recovery_target
    );
}

fn assert_dirty_after_recovery(app: &ReadyAppState) {
    let recovered = app.platform_host.snapshot().window_model();
    assert_eq!(
        recovered.runtime.control_state.run_status,
        rf_ui::RunStatus::Dirty
    );
    assert_eq!(
        recovered.runtime.control_state.pending_reason,
        Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
    );
}

fn assert_error_after_focus_recovery(app: &ReadyAppState) {
    let focused = app.platform_host.snapshot().window_model();
    assert_eq!(
        focused.runtime.control_state.run_status,
        rf_ui::RunStatus::Error
    );
}

fn assert_active_inspector_target(app: &ReadyAppState, kind_label: &str, target_id: &str) {
    let focused = app.platform_host.snapshot().window_model();
    let detail = focused
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected active inspector detail");
    assert_eq!(detail.target.kind_label, kind_label);
    assert_eq!(detail.target.target_id, target_id);
}

fn reopen_and_assert_clean(app: &mut ReadyAppState, project_path: &std::path::Path) {
    app.open_project(project_path.to_path_buf(), "project");
    let reopened = app.platform_host.snapshot().window_model();
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);
}

fn run_and_assert_converged(app: &mut ReadyAppState) {
    app.dispatch_ui_command("run_panel.run_manual");
    let rerun = app.platform_host.snapshot().window_model();
    assert_eq!(
        rerun.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert!(rerun.runtime.latest_failure.is_none());
    assert!(rerun.runtime.latest_solve_snapshot.is_some());
}

fn assert_latest_snapshot_has_stream(app: &ReadyAppState, stream_id: &str) {
    assert!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .latest_solve_snapshot
            .as_ref()
            .expect("expected solve snapshot")
            .streams
            .iter()
            .any(|stream| stream.stream_id == stream_id)
    );
}

fn assert_latest_snapshot_missing_stream(app: &ReadyAppState, stream_id: &str) {
    assert!(
        !app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .latest_solve_snapshot
            .as_ref()
            .expect("expected solve snapshot")
            .streams
            .iter()
            .any(|stream| stream.stream_id == stream_id)
    );
}

fn assert_latest_stream_pressure(app: &ReadyAppState, stream_id: &str, expected: f64) {
    let window = app.platform_host.snapshot().window_model();
    let stream = window
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected solve snapshot")
        .streams
        .iter()
        .find(|stream| stream.stream_id == stream_id)
        .unwrap_or_else(|| panic!("expected {stream_id} stream result"));
    assert_eq!(stream.pressure_pa, expected);
}

fn assert_latest_stream_fraction(
    app: &ReadyAppState,
    stream_id: &str,
    component_id: &str,
    expected: f64,
) {
    let window = app.platform_host.snapshot().window_model();
    let stream = window
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected solve snapshot")
        .streams
        .iter()
        .find(|stream| stream.stream_id == stream_id)
        .unwrap_or_else(|| panic!("expected {stream_id} stream result"));
    let row = stream
        .composition_rows
        .iter()
        .find(|row| row.component_id == component_id)
        .unwrap_or_else(|| panic!("expected {component_id} composition row"));
    assert_close(row.fraction, expected);
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12,
        "expected {actual} to equal {expected}"
    );
}
