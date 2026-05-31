use super::*;

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12,
        "expected {actual} to equal {expected}"
    );
}

fn assert_stream_fraction(
    stream: &radishflow_studio::StudioGuiWindowStreamResultModel,
    component_id: &str,
    expected: f64,
) {
    let row = stream
        .composition_rows
        .iter()
        .find(|row| row.component_id == component_id)
        .unwrap_or_else(|| panic!("expected {component_id} composition row"));
    assert_close(row.fraction, expected);
}

fn assert_snapshot_step_links(
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    unit_id: &str,
    consumed_stream_id: &str,
    produced_stream_ids: &[&str],
) {
    let step = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == unit_id)
        .unwrap_or_else(|| panic!("expected solve step for unit {unit_id}"));
    assert!(
        step.consumed_stream_results
            .iter()
            .any(|stream| stream.stream_id == consumed_stream_id),
        "expected unit {unit_id} to consume stream {consumed_stream_id}"
    );
    for produced_stream_id in produced_stream_ids {
        assert!(
            step.produced_stream_results
                .iter()
                .any(|stream| stream.stream_id == *produced_stream_id),
            "expected unit {unit_id} to produce stream {produced_stream_id}"
        );
    }
}

fn assert_result_inspector_unit_streams(
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    inspected_stream_id: &str,
    unit_id: &str,
    consumed_stream_id: &str,
    produced_stream_ids: &[&str],
) {
    let result =
        snapshot.result_inspector_with_unit(Some(inspected_stream_id), None, Some(unit_id));
    let selected_unit = result
        .selected_unit
        .as_ref()
        .unwrap_or_else(|| panic!("expected selected result unit {unit_id}"));
    assert!(
        selected_unit
            .consumed_stream_results
            .iter()
            .any(|stream| stream.stream_id == consumed_stream_id),
        "expected result inspector unit {unit_id} to consume stream {consumed_stream_id}"
    );
    for produced_stream_id in produced_stream_ids {
        assert!(
            selected_unit
                .produced_stream_results
                .iter()
                .any(|stream| stream.stream_id == *produced_stream_id),
            "expected result inspector unit {unit_id} to produce stream {produced_stream_id}"
        );
    }
}

fn snapshot_stream<'a>(
    snapshot: &'a radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
) -> &'a radishflow_studio::StudioGuiWindowStreamResultModel {
    snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == stream_id)
        .unwrap_or_else(|| panic!("expected stream result {stream_id}"))
}

fn assert_flash_outlet_results(
    snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    temperature_k: f64,
    pressure_pa: f64,
    expected_total_flow_mol_s: f64,
) {
    let flash_liquid = snapshot_stream(snapshot, "stream-flash-1-liquid");
    let flash_vapor = snapshot_stream(snapshot, "stream-flash-1-vapor");
    assert_eq!(flash_liquid.temperature_k, temperature_k);
    assert_eq!(flash_liquid.pressure_pa, pressure_pa);
    assert_eq!(flash_vapor.temperature_k, temperature_k);
    assert_eq!(flash_vapor.pressure_pa, pressure_pa);
    assert_close(
        flash_liquid.total_molar_flow_mol_s + flash_vapor.total_molar_flow_mol_s,
        expected_total_flow_mol_s,
    );
    assert!(
        flash_liquid.molar_enthalpy_j_per_mol.is_some()
            || flash_vapor.molar_enthalpy_j_per_mol.is_some(),
        "expected at least one flash outlet to carry molar enthalpy"
    );
}

fn commit_stream_field(app: &mut ReadyAppState, stream_id: &str, field: &str, raw_value: &str) {
    app.dispatch_ui_command(format!("inspector.focus_stream:{stream_id}"));
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(&format!(
            "stream:{stream_id}:{field}"
        )),
        raw_value,
    );
    app.dispatch_inspector_field_draft_commit(
        radishflow_studio::inspector_draft_commit_command_id(&format!(
            "stream:{stream_id}:{field}"
        )),
    );
}

fn normalize_stream_composition(app: &mut ReadyAppState, stream_id: &str, drafts: &[(&str, &str)]) {
    app.dispatch_ui_command(format!("inspector.focus_stream:{stream_id}"));
    for (component_id, raw_value) in drafts {
        app.dispatch_inspector_field_draft_update(
            radishflow_studio::inspector_draft_update_command_id(&format!(
                "stream:{stream_id}:overall_mole_fraction:{component_id}"
            )),
            *raw_value,
        );
    }
    let detail = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .active_inspector_detail
        .expect("expected active stream inspector");
    app.dispatch_inspector_composition_normalize(
        detail
            .property_composition_normalize_command_id
            .expect("expected stream composition normalize command"),
    );
}

#[derive(Clone, Copy)]
struct SingleInletFlashAuthoringCase {
    case_name: &'static str,
    begin_unit_command: &'static str,
    unit_id: &'static str,
    connect_unit_inlet_suggestion: &'static str,
    create_unit_outlet_suggestion: &'static str,
    unit_outlet_stream_id: &'static str,
    unit_outlet_temperature_k: Option<f64>,
    unit_outlet_pressure_pa: f64,
    flash_temperature_k: f64,
    flash_pressure_pa: f64,
}

fn author_single_inlet_flash_case_from_blank(
    app: &mut ReadyAppState,
    case: SingleInletFlashAuthoringCase,
) {
    select_builtin_binary_hydrocarbon_basis(app);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(app, "local.feed.create_outlet.feed-1");

    app.dispatch_ui_command(case.begin_unit_command);
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(180.0, 40.0));
    accept_canvas_suggestion_by_id(app, case.connect_unit_inlet_suggestion);
    accept_canvas_suggestion_by_id(app, case.create_unit_outlet_suggestion);

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(320.0, 40.0));
    accept_canvas_suggestion_by_id(
        app,
        &format!(
            "local.flash_drum.connect_inlet.flash-1.{}",
            case.unit_outlet_stream_id
        ),
    );
    accept_canvas_suggestion_by_id(app, "local.flash_drum.create_outlet.flash-1.liquid");
    accept_canvas_suggestion_by_id(app, "local.flash_drum.create_outlet.flash-1.vapor");

    normalize_stream_composition(
        app,
        "stream-feed-1-outlet",
        &[("methane", "0.2"), ("ethane", "0.6")],
    );
    commit_unit_parameter(app, "feed-1", "unit:feed-1:outlet_temperature_k", "310");
    commit_unit_parameter(app, "feed-1", "unit:feed-1:outlet_pressure_pa", "130000");
    if let Some(temperature_k) = case.unit_outlet_temperature_k {
        commit_unit_parameter(
            app,
            case.unit_id,
            &format!("unit:{}:outlet_temperature_k", case.unit_id),
            &temperature_k.to_string(),
        );
    }
    commit_unit_parameter(
        app,
        case.unit_id,
        &format!("unit:{}:outlet_pressure_pa", case.unit_id),
        &case.unit_outlet_pressure_pa.to_string(),
    );
    commit_unit_parameter(
        app,
        "flash-1",
        "unit:flash-1:outlet_temperature_k",
        &case.flash_temperature_k.to_string(),
    );
    commit_unit_parameter(
        app,
        "flash-1",
        "unit:flash-1:outlet_pressure_pa",
        &case.flash_pressure_pa.to_string(),
    );
}

#[test]
fn canvas_viewport_navigation_records_inspector_focus_commands() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("inspector.focus_unit:flash-1");

    let unit_focus = app
        .canvas_viewport_navigation
        .active_anchor
        .as_ref()
        .expect("expected unit viewport focus");
    assert_eq!(unit_focus.anchor_label, "unit-slot-1");
    assert!(unit_focus.pending_scroll);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str()
        )),
        Some((
            RunPanelNoticeLevel::Info,
            "located",
            "Canvas object located"
        ))
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line == "canvas object located: Unit flash-1 -> unit-slot-1")
    );
    assert_eq!(app.last_area_focus, Some(StudioGuiWindowAreaId::Canvas));
    assert!(
        app.canvas_viewport_navigation
            .take_pending_scroll_for_anchor("unit-slot-1")
    );
    assert!(
        !app.canvas_viewport_navigation
            .take_pending_scroll_for_anchor("unit-slot-1")
    );

    app.dispatch_ui_command("inspector.focus_stream:stream-feed");

    let stream_focus = app
        .canvas_viewport_navigation
        .active_anchor
        .as_ref()
        .expect("expected stream viewport focus");
    assert_eq!(stream_focus.anchor_label, "stream-feed:0");
    assert!(stream_focus.pending_scroll);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn canvas_pending_edit_commit_records_created_unit_focus_feedback() {
    let (config, project_path) = flash_drum_local_rules_config();
    let layout_path = studio_layout_path_for_project(&project_path);
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(144.0, 88.0));

    let window = app.platform_host.snapshot().window_model();
    let focus = window
        .canvas
        .widget
        .view()
        .viewport
        .focus
        .as_ref()
        .expect("expected created unit focus");
    assert_eq!(focus.kind_label, "Unit");
    assert_eq!(focus.target_id, "flash-2");
    assert_eq!(focus.command_id, "inspector.focus_unit:flash-2");
    assert_eq!(
        window
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| {
                (
                    target.kind_label,
                    target.target_id.as_str(),
                    target.command_id.as_str(),
                )
            }),
        Some(("Unit", "flash-2", "inspector.focus_unit:flash-2"))
    );

    let active_anchor = app
        .canvas_viewport_navigation
        .active_anchor
        .as_ref()
        .expect("expected created unit canvas anchor");
    assert_eq!(active_anchor.anchor_label, focus.anchor_label);
    assert!(active_anchor.pending_scroll);
    assert_eq!(app.last_area_focus, Some(StudioGuiWindowAreaId::Canvas));
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str(),
            result.target.command_id.as_str(),
            result.anchor_label.as_deref()
        )),
        Some((
            RunPanelNoticeLevel::Info,
            "created",
            "Canvas unit created",
            "inspector.focus_unit:flash-2",
            Some(focus.anchor_label.as_str())
        ))
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line
                == &format!(
                    "canvas unit created: Unit flash-2 -> {}",
                    focus.anchor_label
                ))
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    let surface = app
        .canvas_command_result_command_surface()
        .expect("expected canvas command result command surface");
    assert_eq!(surface.status_label, "created");
    assert_eq!(surface.target_command_id, "inspector.focus_unit:flash-2");
    assert!(surface.matches_query("canvas result created flash-2"));
    assert!(!surface.matches_query("stream-feed"));

    let _ = std::fs::remove_file(layout_path);
    let _ = std::fs::remove_file(project_path);
}

#[test]
fn canvas_placement_palette_commit_matrix_records_created_unit_feedback() {
    let cases = [
        ("canvas.begin_place_unit.feed", "feed", "feed-"),
        ("canvas.begin_place_unit.mixer", "mixer", "mixer-"),
        ("canvas.begin_place_unit.heater", "heater", "heater-"),
        ("canvas.begin_place_unit.cooler", "cooler", "cooler-"),
        ("canvas.begin_place_unit.valve", "valve", "valve-"),
        ("canvas.begin_place_unit.flash_drum", "flash_drum", "flash-"),
    ];

    for (command_id, expected_kind, expected_prefix) in cases {
        let (config, project_path) = blank_workspace_config();
        let mut app = ready_app_state(&config);

        app.dispatch_ui_command(command_id);
        app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));

        let window = app.platform_host.snapshot().window_model();
        let focus = window
            .canvas
            .widget
            .view()
            .viewport
            .focus
            .as_ref()
            .unwrap_or_else(|| panic!("{command_id} should focus the created unit"));
        assert_eq!(focus.kind_label, "Unit", "{command_id}");
        assert!(
            focus.target_id.starts_with(expected_prefix),
            "{command_id} should allocate unit id with prefix `{expected_prefix}`"
        );
        assert_eq!(
            focus.command_id,
            format!("inspector.focus_unit:{}", focus.target_id),
            "{command_id}"
        );

        let created_unit = window
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == focus.target_id)
            .unwrap_or_else(|| panic!("{command_id} should expose created unit block"));
        assert_eq!(created_unit.kind, expected_kind, "{command_id}");
        assert!(
            created_unit.port_count > 0,
            "{command_id} should expose canonical ports"
        );

        assert_eq!(
            window
                .runtime
                .active_inspector_target
                .as_ref()
                .map(|target| target.command_id.as_str()),
            Some(focus.command_id.as_str()),
            "{command_id}"
        );
        assert_eq!(
            app.canvas_viewport_navigation
                .active_anchor
                .as_ref()
                .map(|anchor| (anchor.anchor_label.as_str(), anchor.pending_scroll)),
            Some((focus.anchor_label.as_str(), true)),
            "{command_id}"
        );
        assert_eq!(
            app.canvas_command_result.as_ref().map(|result| (
                result.level,
                result.status_label,
                result.title.as_str(),
                result.target.target_id.as_str(),
                result.target.command_id.as_str(),
                result.anchor_label.as_deref(),
            )),
            Some((
                RunPanelNoticeLevel::Info,
                "created",
                "Canvas unit created",
                focus.target_id.as_str(),
                focus.command_id.as_str(),
                Some(focus.anchor_label.as_str()),
            )),
            "{command_id}"
        );
        assert!(
            app.canvas_command_result_command_surface()
                .expect("expected canvas command result command surface")
                .matches_query(&format!("created {}", focus.target_id)),
            "{command_id}"
        );
        assert!(
            app.platform_host
                .snapshot()
                .runtime
                .workspace_document
                .has_unsaved_changes,
            "{command_id}"
        );

        let _ = fs::remove_file(studio_layout_path_for_project(&project_path));
        let _ = fs::remove_file(project_path);
    }
}

#[test]
fn canvas_feed_to_flash_minimal_path_surfaces_local_connection_suggestions() {
    let mut app = ready_app_state(&lease_expiring_config());

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));

    let after_feed = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_feed.canvas.focused_suggestion_id.as_deref(),
        Some("local.feed.create_outlet.feed-2")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_feed_outlet = app.platform_host.snapshot().window_model();
    assert_eq!(after_feed_outlet.canvas.suggestion_count, 0);

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(220.0, 40.0));

    let after_flash = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_flash.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-2.stream-feed-2-outlet")
    );
    assert_eq!(after_flash.canvas.suggestion_count, 1);

    app.dispatch_ui_command("canvas.accept_focused");
    let after_inlet = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_inlet.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.create_outlet.flash-2.liquid")
    );
    assert_eq!(after_inlet.canvas.suggestion_count, 2);

    app.dispatch_ui_command("canvas.accept_focused");
    let after_liquid = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_liquid.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.create_outlet.flash-2.vapor")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let completed = app.platform_host.snapshot().window_model();
    assert_eq!(completed.canvas.suggestion_count, 0);
    assert!(completed.runtime.workspace_document.has_unsaved_changes);
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-feed-2-outlet")
    );
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-flash-2-liquid")
    );
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-flash-2-vapor")
    );
}

#[test]
fn canvas_feed_to_flash_explicit_suggestion_selection_can_run_after_parameters() {
    let mut app = ready_app_state(&lease_expiring_config());

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-2");

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(220.0, 40.0));
    let after_flash = app.platform_host.snapshot().window_model();
    assert!(
        after_flash
            .canvas
            .widget
            .view()
            .suggestions
            .iter()
            .all(|suggestion| !suggestion.id.contains("create_outlet")),
        "flash outlet suggestions must wait until the flash inlet is bound"
    );
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.flash_drum.connect_inlet.flash-2.stream-feed-2-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-2.vapor");
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-2.liquid");

    commit_unit_parameter(
        &mut app,
        "flash-2",
        "unit:flash-2:outlet_temperature_k",
        "300",
    );
    commit_unit_parameter(
        &mut app,
        "flash-2",
        "unit:flash-2:outlet_pressure_pa",
        "85000",
    );

    app.dispatch_ui_command("run_panel.run_manual");
    let completed = app.platform_host.snapshot().window_model();
    assert_eq!(
        completed.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(completed.runtime.control_state.pending_reason, None);
    assert_eq!(completed.canvas.suggestion_count, 0);
}

#[test]
fn blank_project_selects_thermo_basis_saves_reopens_and_runs_feed_flash_path() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);

    let opened_blank = app.platform_host.snapshot().window_model();
    assert_eq!(opened_blank.runtime.workspace_document.revision, 0);
    assert!(!opened_blank.runtime.workspace_document.has_unsaved_changes);
    assert_eq!(
        opened_blank
            .runtime
            .workspace_document
            .property_package_id
            .as_deref(),
        None
    );
    assert!(
        opened_blank
            .runtime
            .workspace_document
            .project_component_choices
            .iter()
            .all(|choice| !choice.selected)
    );

    select_builtin_binary_hydrocarbon_basis(&mut app);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .as_deref(),
        Some("local.feed.create_outlet.feed-1")
    );
    app.dispatch_ui_command("canvas.accept_focused");

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(220.0, 40.0));
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .as_deref(),
        Some("local.flash_drum.connect_inlet.flash-1.stream-feed-1-outlet")
    );
    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");

    normalize_stream_composition(
        &mut app,
        "stream-feed-1-outlet",
        &[("methane", "0.3"), ("ethane", "0.7")],
    );
    commit_stream_field(&mut app, "stream-feed-1-outlet", "temperature_k", "308");
    commit_stream_field(&mut app, "stream-feed-1-outlet", "pressure_pa", "125000");
    commit_stream_field(
        &mut app,
        "stream-feed-1-outlet",
        "total_molar_flow_mol_s",
        "1.5",
    );
    commit_unit_parameter(
        &mut app,
        "flash-1",
        "unit:flash-1:outlet_temperature_k",
        "300",
    );
    commit_unit_parameter(
        &mut app,
        "flash-1",
        "unit:flash-1:outlet_pressure_pa",
        "85000",
    );

    app.dispatch_ui_command("run_panel.run_manual");
    let solved = app.platform_host.snapshot().window_model();
    assert_eq!(
        solved.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(solved.runtime.control_state.pending_reason, None);

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved blank project");
    assert_eq!(saved.document.flowsheet.components.len(), 2);
    assert_eq!(
        stored_unit_port_stream_id(&saved, "feed-1", "outlet"),
        Some("stream-feed-1-outlet")
    );
    assert_eq!(
        stored_unit_port_stream_id(&saved, "flash-1", "inlet"),
        Some("stream-feed-1-outlet")
    );
    assert_eq!(
        stored_unit_port_stream_id(&saved, "flash-1", "liquid"),
        Some("stream-flash-1-liquid")
    );
    assert_eq!(
        stored_unit_port_stream_id(&saved, "flash-1", "vapor"),
        Some("stream-flash-1-vapor")
    );
    let feed_stream = saved
        .document
        .flowsheet
        .streams
        .get(&rf_types::StreamId::new("stream-feed-1-outlet"))
        .expect("expected feed source stream");
    assert_eq!(feed_stream.temperature_k, 308.0);
    assert_eq!(feed_stream.pressure_pa, 125_000.0);
    assert_eq!(feed_stream.total_molar_flow_mol_s, 1.5);
    assert_eq!(feed_stream.overall_mole_fractions.len(), 2);
    assert_eq!(
        feed_stream
            .overall_mole_fractions
            .get(&rf_types::ComponentId::new("methane"))
            .copied(),
        Some(0.3)
    );
    assert_eq!(
        feed_stream
            .overall_mole_fractions
            .get(&rf_types::ComponentId::new("ethane"))
            .copied(),
        Some(0.7)
    );
    let flash_unit = saved
        .document
        .flowsheet
        .units
        .get(&UnitId::new("flash-1"))
        .expect("expected flash unit");
    assert_eq!(flash_unit.parameters.outlet_temperature_k, Some(300.0));
    assert_eq!(flash_unit.parameters.outlet_pressure_pa, Some(85000.0));

    app.open_project(project_path.clone(), "project");
    let reopened = app.platform_host.snapshot().window_model();
    assert_eq!(
        reopened.runtime.workspace_document.revision,
        saved.document.revision
    );
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);

    app.dispatch_ui_command("run_panel.run_manual");
    let rerun = app.platform_host.snapshot().window_model();
    assert_eq!(
        rerun.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(rerun.runtime.control_state.pending_reason, None);
    let snapshot = rerun
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected feed flash rerun solve snapshot");
    let feed = snapshot_stream(snapshot, "stream-feed-1-outlet");
    assert_eq!(feed.temperature_k, 308.0);
    assert_eq!(feed.pressure_pa, 125_000.0);
    assert_eq!(feed.total_molar_flow_mol_s, 1.5);
    assert_stream_fraction(feed, "methane", 0.3);
    assert_stream_fraction(feed, "ethane", 0.7);
    assert_snapshot_step_links(
        snapshot,
        "flash-1",
        "stream-feed-1-outlet",
        &["stream-flash-1-liquid", "stream-flash-1-vapor"],
    );
    assert_flash_outlet_results(snapshot, 300.0, 85_000.0, feed.total_molar_flow_mol_s);

    let _ = fs::remove_file(project_path);
}

#[test]
fn blank_project_single_inlet_flash_paths_save_reopen_and_rerun() {
    let cases = [
        SingleInletFlashAuthoringCase {
            case_name: "heater",
            begin_unit_command: "canvas.begin_place_unit.heater",
            unit_id: "heater-1",
            connect_unit_inlet_suggestion: "local.heater.connect_inlet.heater-1.stream-feed-1-outlet",
            create_unit_outlet_suggestion: "local.heater.create_outlet.heater-1",
            unit_outlet_stream_id: "stream-heater-1-outlet",
            unit_outlet_temperature_k: Some(358.5),
            unit_outlet_pressure_pa: 90_000.0,
            flash_temperature_k: 300.0,
            flash_pressure_pa: 85_000.0,
        },
        SingleInletFlashAuthoringCase {
            case_name: "cooler",
            begin_unit_command: "canvas.begin_place_unit.cooler",
            unit_id: "cooler-1",
            connect_unit_inlet_suggestion: "local.cooler.connect_inlet.cooler-1.stream-feed-1-outlet",
            create_unit_outlet_suggestion: "local.cooler.create_outlet.cooler-1",
            unit_outlet_stream_id: "stream-cooler-1-outlet",
            unit_outlet_temperature_k: Some(286.0),
            unit_outlet_pressure_pa: 90_000.0,
            flash_temperature_k: 300.0,
            flash_pressure_pa: 85_000.0,
        },
        SingleInletFlashAuthoringCase {
            case_name: "valve",
            begin_unit_command: "canvas.begin_place_unit.valve",
            unit_id: "valve-1",
            connect_unit_inlet_suggestion: "local.valve.connect_inlet.valve-1.stream-feed-1-outlet",
            create_unit_outlet_suggestion: "local.valve.create_outlet.valve-1",
            unit_outlet_stream_id: "stream-valve-1-outlet",
            unit_outlet_temperature_k: None,
            unit_outlet_pressure_pa: 85_000.0,
            flash_temperature_k: 300.0,
            flash_pressure_pa: 80_000.0,
        },
    ];

    for case in cases {
        let (config, project_path) = blank_workspace_config();
        let mut app = ready_app_state(&config);
        author_single_inlet_flash_case_from_blank(&mut app, case);

        app.save_project();
        let saved = read_project_file(&project_path)
            .unwrap_or_else(|_| panic!("expected saved {} project", case.case_name));
        assert_eq!(
            saved.document.flowsheet.property_package_id(),
            Some("binary-hydrocarbon-lite-v1"),
            "{}",
            case.case_name
        );
        assert_eq!(
            saved.document.flowsheet.components.len(),
            2,
            "{}",
            case.case_name
        );
        assert_eq!(
            stored_unit_port_stream_id(&saved, "feed-1", "outlet"),
            Some("stream-feed-1-outlet"),
            "{}",
            case.case_name
        );
        assert_eq!(
            stored_unit_port_stream_id(&saved, case.unit_id, "inlet"),
            Some("stream-feed-1-outlet"),
            "{}",
            case.case_name
        );
        assert_eq!(
            stored_unit_port_stream_id(&saved, case.unit_id, "outlet"),
            Some(case.unit_outlet_stream_id),
            "{}",
            case.case_name
        );
        assert_eq!(
            stored_unit_port_stream_id(&saved, "flash-1", "inlet"),
            Some(case.unit_outlet_stream_id),
            "{}",
            case.case_name
        );
        assert_eq!(
            stored_unit_port_stream_id(&saved, "flash-1", "liquid"),
            Some("stream-flash-1-liquid"),
            "{}",
            case.case_name
        );
        assert_eq!(
            stored_unit_port_stream_id(&saved, "flash-1", "vapor"),
            Some("stream-flash-1-vapor"),
            "{}",
            case.case_name
        );
        let feed_stream = &saved.document.flowsheet.streams[&StreamId::new("stream-feed-1-outlet")];
        assert_close(
            feed_stream.overall_mole_fractions[&rf_types::ComponentId::new("methane")],
            0.25,
        );
        assert_close(
            feed_stream.overall_mole_fractions[&rf_types::ComponentId::new("ethane")],
            0.75,
        );
        assert_eq!(feed_stream.temperature_k, 310.0);
        assert_eq!(feed_stream.pressure_pa, 130_000.0);
        let unit = &saved.document.flowsheet.units[&UnitId::new(case.unit_id)];
        assert_eq!(
            unit.parameters.outlet_pressure_pa,
            Some(case.unit_outlet_pressure_pa),
            "{}",
            case.case_name
        );
        let unit_outlet =
            &saved.document.flowsheet.streams[&StreamId::new(case.unit_outlet_stream_id)];
        assert_eq!(unit_outlet.pressure_pa, case.unit_outlet_pressure_pa);
        if let Some(temperature_k) = case.unit_outlet_temperature_k {
            assert_eq!(unit.parameters.outlet_temperature_k, Some(temperature_k));
            assert_eq!(unit_outlet.temperature_k, temperature_k);
        }

        app.open_project(project_path.clone(), "project");
        let reopened = app.platform_host.snapshot().window_model();
        assert!(!reopened.runtime.workspace_document.has_unsaved_changes);

        app.dispatch_ui_command("run_panel.run_manual");
        let rerun = app.platform_host.snapshot().window_model();
        assert_eq!(
            rerun.runtime.control_state.run_status,
            rf_ui::RunStatus::Converged
        );
        let snapshot = rerun
            .runtime
            .latest_solve_snapshot
            .as_ref()
            .unwrap_or_else(|| panic!("expected {} rerun solve snapshot", case.case_name));
        let feed = snapshot_stream(snapshot, "stream-feed-1-outlet");
        let unit_outlet_result = snapshot_stream(snapshot, case.unit_outlet_stream_id);
        assert_eq!(feed.temperature_k, 310.0);
        assert_eq!(feed.pressure_pa, 130_000.0);
        assert_stream_fraction(feed, "methane", 0.25);
        assert_stream_fraction(feed, "ethane", 0.75);
        assert_eq!(unit_outlet_result.pressure_pa, case.unit_outlet_pressure_pa);
        if let Some(temperature_k) = case.unit_outlet_temperature_k {
            assert_eq!(unit_outlet_result.temperature_k, temperature_k);
        } else {
            assert_eq!(unit_outlet_result.temperature_k, feed.temperature_k);
        }
        assert!(unit_outlet_result.molar_enthalpy_j_per_mol.is_some());

        assert_snapshot_step_links(
            snapshot,
            case.unit_id,
            "stream-feed-1-outlet",
            &[case.unit_outlet_stream_id],
        );
        assert_snapshot_step_links(
            snapshot,
            "flash-1",
            case.unit_outlet_stream_id,
            &["stream-flash-1-liquid", "stream-flash-1-vapor"],
        );
        assert_result_inspector_unit_streams(
            snapshot,
            case.unit_outlet_stream_id,
            case.unit_id,
            "stream-feed-1-outlet",
            &[case.unit_outlet_stream_id],
        );
        assert_result_inspector_unit_streams(
            snapshot,
            case.unit_outlet_stream_id,
            "flash-1",
            case.unit_outlet_stream_id,
            &["stream-flash-1-liquid", "stream-flash-1-vapor"],
        );
        assert_flash_outlet_results(
            snapshot,
            case.flash_temperature_k,
            case.flash_pressure_pa,
            feed.total_molar_flow_mol_s,
        );

        if case.case_name == "heater" {
            let export_path = project_path.with_extension("txt");
            app.export_solve_snapshot_to_path(snapshot, export_path.clone());
            let exported = fs::read_to_string(&export_path).expect("expected heater export read");
            assert!(exported.contains("Units\nunit_id\tstep\tstatus\tsummary"));
            assert!(exported.contains(case.unit_id));
            assert!(exported.contains("flash-1"));
            assert!(exported.contains(case.unit_outlet_stream_id));
            assert!(
                !app.platform_host
                    .snapshot()
                    .window_model()
                    .runtime
                    .workspace_document
                    .has_unsaved_changes,
                "heater result export must not dirty the authored case"
            );
            let _ = fs::remove_file(export_path);
        }

        let _ = fs::remove_file(project_path);
    }
}

#[test]
fn blank_project_mixer_path_saves_reopens_and_reruns() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);
    select_builtin_binary_hydrocarbon_basis(&mut app);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 140.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-2");

    normalize_stream_composition(
        &mut app,
        "stream-feed-1-outlet",
        &[("methane", "0.2"), ("ethane", "0.6")],
    );
    normalize_stream_composition(
        &mut app,
        "stream-feed-2-outlet",
        &[("methane", "0.7"), ("ethane", "0.1")],
    );

    app.dispatch_ui_command("canvas.begin_place_unit.mixer");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(210.0, 90.0));
    let mixer_suggestions = app.platform_host.snapshot().window_model();
    let mixer_action_labels = mixer_suggestions
        .canvas
        .widget
        .view()
        .suggestions
        .iter()
        .map(|suggestion| (suggestion.id.as_str(), suggestion.action_label))
        .collect::<Vec<_>>();
    assert!(
        mixer_action_labels.contains(&(
            "local.mixer.connect_inlet_a.mixer-1.stream-feed-1-outlet",
            "Connect stream",
        )),
        "expected mixer inlet suggestion to render a connect action label, labels: {mixer_action_labels:?}"
    );
    assert!(
        !mixer_action_labels
            .iter()
            .any(|(id, _)| *id == "local.mixer.create_outlet.mixer-1"),
        "mixer outlet suggestion must wait until both mixer inlets are bound, labels: {mixer_action_labels:?}"
    );
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.mixer.connect_inlet_a.mixer-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.mixer.connect_inlet_b.mixer-1.stream-feed-2-outlet",
    );
    let after_mixer_inlets = app.platform_host.snapshot().window_model();
    assert!(
        after_mixer_inlets
            .canvas
            .widget
            .view()
            .suggestions
            .iter()
            .any(
                |suggestion| suggestion.id == "local.mixer.create_outlet.mixer-1"
                    && suggestion.action_label == "Create stream"
            ),
        "expected mixer outlet suggestion after both inlets are bound"
    );
    accept_canvas_suggestion_by_id(&mut app, "local.mixer.create_outlet.mixer-1");

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(360.0, 90.0));
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.flash_drum.connect_inlet.flash-1.stream-mixer-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.liquid");
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.vapor");

    commit_unit_parameter(
        &mut app,
        "feed-1",
        "unit:feed-1:outlet_temperature_k",
        "305",
    );
    commit_unit_parameter(
        &mut app,
        "feed-1",
        "unit:feed-1:outlet_pressure_pa",
        "130000",
    );
    commit_unit_parameter(
        &mut app,
        "feed-2",
        "unit:feed-2:outlet_temperature_k",
        "315",
    );
    commit_unit_parameter(
        &mut app,
        "feed-2",
        "unit:feed-2:outlet_pressure_pa",
        "120000",
    );
    commit_unit_parameter(
        &mut app,
        "mixer-1",
        "unit:mixer-1:outlet_pressure_pa",
        "90000",
    );
    commit_unit_parameter(
        &mut app,
        "flash-1",
        "unit:flash-1:outlet_temperature_k",
        "300",
    );
    commit_unit_parameter(
        &mut app,
        "flash-1",
        "unit:flash-1:outlet_pressure_pa",
        "85000",
    );

    app.dispatch_ui_command("run_panel.run_manual");
    let solved = app.platform_host.snapshot().window_model();
    assert_eq!(
        solved.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(solved.runtime.control_state.pending_reason, None);
    let solved_mixer_outlet = solved
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected solve snapshot")
        .streams
        .iter()
        .find(|stream| stream.stream_id == "stream-mixer-1-outlet")
        .expect("expected mixer outlet result");
    assert_eq!(solved_mixer_outlet.total_molar_flow_mol_s, 2.0);
    assert_eq!(solved_mixer_outlet.pressure_pa, 90_000.0);
    assert_stream_fraction(solved_mixer_outlet, "methane", 0.5625);
    assert_stream_fraction(solved_mixer_outlet, "ethane", 0.4375);
    let solved_flash_liquid = solved
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected solve snapshot")
        .streams
        .iter()
        .find(|stream| stream.stream_id == "stream-flash-1-liquid")
        .expect("expected flash liquid outlet result");
    assert_eq!(solved_flash_liquid.temperature_k, 300.0);
    assert_eq!(solved_flash_liquid.pressure_pa, 85_000.0);

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved blank mixer project");
    assert_eq!(saved.document.flowsheet.components.len(), 2);
    for unit_id in ["feed-1", "feed-2", "mixer-1", "flash-1"] {
        assert!(
            saved
                .document
                .flowsheet
                .units
                .contains_key(&UnitId::new(unit_id)),
            "expected saved unit {unit_id}"
        );
    }
    for stream_id in [
        "stream-feed-1-outlet",
        "stream-feed-2-outlet",
        "stream-mixer-1-outlet",
        "stream-flash-1-liquid",
        "stream-flash-1-vapor",
    ] {
        assert!(
            saved
                .document
                .flowsheet
                .streams
                .contains_key(&StreamId::new(stream_id)),
            "expected saved stream {stream_id}"
        );
    }
    assert_eq!(
        stored_unit_port_stream_id(&saved, "mixer-1", "inlet_a"),
        Some("stream-feed-1-outlet")
    );
    assert_eq!(
        stored_unit_port_stream_id(&saved, "mixer-1", "inlet_b"),
        Some("stream-feed-2-outlet")
    );
    assert_eq!(
        stored_unit_port_stream_id(&saved, "mixer-1", "outlet"),
        Some("stream-mixer-1-outlet")
    );
    assert_eq!(
        stored_unit_port_stream_id(&saved, "flash-1", "inlet"),
        Some("stream-mixer-1-outlet")
    );
    assert_eq!(
        stored_unit_port_stream_id(&saved, "flash-1", "liquid"),
        Some("stream-flash-1-liquid")
    );
    assert_eq!(
        stored_unit_port_stream_id(&saved, "flash-1", "vapor"),
        Some("stream-flash-1-vapor")
    );
    assert_close(
        saved.document.flowsheet.streams[&StreamId::new("stream-feed-1-outlet")]
            .overall_mole_fractions[&rf_types::ComponentId::new("methane")],
        0.25,
    );
    assert_close(
        saved.document.flowsheet.streams[&StreamId::new("stream-feed-2-outlet")]
            .overall_mole_fractions[&rf_types::ComponentId::new("methane")],
        0.875,
    );
    assert_eq!(
        saved
            .document
            .flowsheet
            .units
            .get(&UnitId::new("feed-1"))
            .expect("expected saved feed-1")
            .parameters
            .outlet_temperature_k,
        Some(305.0)
    );
    assert_eq!(
        saved
            .document
            .flowsheet
            .units
            .get(&UnitId::new("feed-2"))
            .expect("expected saved feed-2")
            .parameters
            .outlet_pressure_pa,
        Some(120_000.0)
    );
    assert_eq!(
        saved
            .document
            .flowsheet
            .units
            .get(&UnitId::new("mixer-1"))
            .expect("expected saved mixer")
            .parameters
            .outlet_pressure_pa,
        Some(90_000.0)
    );
    assert_eq!(
        saved
            .document
            .flowsheet
            .units
            .get(&UnitId::new("flash-1"))
            .expect("expected saved flash")
            .parameters
            .outlet_temperature_k,
        Some(300.0)
    );

    app.open_project(project_path.clone(), "project");
    let reopened = app.platform_host.snapshot().window_model();
    assert_eq!(
        reopened.runtime.workspace_document.revision,
        saved.document.revision
    );
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);

    app.dispatch_ui_command("run_panel.run_manual");
    let rerun = app.platform_host.snapshot().window_model();
    assert_eq!(
        rerun.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(rerun.runtime.control_state.pending_reason, None);
    let rerun_streams = &rerun
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected rerun solve snapshot")
        .streams;
    let rerun_mixer_outlet = rerun_streams
        .iter()
        .find(|stream| stream.stream_id == "stream-mixer-1-outlet")
        .expect("expected mixer outlet rerun result");
    assert_eq!(rerun_mixer_outlet.total_molar_flow_mol_s, 2.0);
    assert_eq!(rerun_mixer_outlet.pressure_pa, 90_000.0);
    assert_stream_fraction(rerun_mixer_outlet, "methane", 0.5625);
    assert_stream_fraction(rerun_mixer_outlet, "ethane", 0.4375);
    assert!(
        rerun_streams
            .iter()
            .any(|stream| stream.stream_id == "stream-flash-1-liquid")
    );
    assert!(
        rerun_streams
            .iter()
            .any(|stream| stream.stream_id == "stream-flash-1-vapor")
    );

    let export_path = project_path.with_extension("txt");
    let snapshot = rerun
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected exported rerun solve snapshot");
    app.export_solve_snapshot_to_path(snapshot, export_path.clone());
    let exported = fs::read_to_string(&export_path).expect("expected case author export read");
    assert!(exported.contains("Units\nunit_id\tstep\tstatus\tsummary"));
    assert!(exported.contains("mixer-1"));
    assert!(exported.contains("flash-1"));
    assert!(exported.contains("stream-mixer-1-outlet"));
    assert!(
        !app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes,
        "result export must not dirty the authored case"
    );

    let _ = fs::remove_file(export_path);
    let _ = fs::remove_file(studio_layout_path_for_project(&project_path));
    let _ = fs::remove_file(project_path);
}

#[test]
fn canvas_unit_positions_persist_through_project_save_and_reopen() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);
    select_builtin_binary_hydrocarbon_basis(&mut app);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(220.0, 40.0));

    let before_save = app.platform_host.snapshot().window_model();
    let before_feed = before_save
        .canvas
        .widget
        .view()
        .unit_blocks
        .iter()
        .find(|unit| unit.unit_id == "feed-1")
        .expect("expected feed block before save");
    let before_flash = before_save
        .canvas
        .widget
        .view()
        .unit_blocks
        .iter()
        .find(|unit| unit.unit_id == "flash-1")
        .expect("expected flash block before save");
    assert_eq!(
        before_feed.layout_position,
        Some(rf_ui::CanvasPoint::new(64.0, 40.0))
    );
    assert_eq!(
        before_flash.layout_position,
        Some(rf_ui::CanvasPoint::new(220.0, 40.0))
    );
    assert_eq!(
        before_save.canvas.widget.view().viewport.layout_label,
        "persisted_positions"
    );

    let layout_path = studio_layout_path_for_project(&project_path);
    let stored_layout = read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert!(stored_layout.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "feed-1" && position.x == 64.0 && position.y == 40.0
    }));
    assert!(stored_layout.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "flash-1" && position.x == 220.0 && position.y == 40.0
    }));

    app.save_project();
    app.open_project(project_path.clone(), "project");

    let reopened = app.platform_host.snapshot().window_model();
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);
    assert_eq!(
        reopened
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(64.0, 40.0))
    );
    assert_eq!(
        reopened
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "flash-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(220.0, 40.0))
    );
    assert_eq!(
        reopened.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-1.stream-feed-1-outlet")
    );

    accept_canvas_suggestion_by_id(
        &mut app,
        "local.flash_drum.connect_inlet.flash-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.liquid");
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.vapor");
    normalize_stream_composition(
        &mut app,
        "stream-feed-1-outlet",
        &[("methane", "0.3"), ("ethane", "0.7")],
    );
    commit_stream_field(&mut app, "stream-feed-1-outlet", "temperature_k", "308");
    commit_stream_field(&mut app, "stream-feed-1-outlet", "pressure_pa", "125000");
    commit_stream_field(
        &mut app,
        "stream-feed-1-outlet",
        "total_molar_flow_mol_s",
        "1.5",
    );
    commit_unit_parameter(
        &mut app,
        "flash-1",
        "unit:flash-1:outlet_temperature_k",
        "300",
    );
    commit_unit_parameter(
        &mut app,
        "flash-1",
        "unit:flash-1:outlet_pressure_pa",
        "85000",
    );
    app.dispatch_ui_command("run_panel.run_manual");
    let solved = app.platform_host.snapshot().window_model();
    assert_eq!(
        solved.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_unit_layout_nudge_commands_move_selected_unit_from_command_surface() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    app.save_project();
    app.dispatch_ui_command("inspector.focus_unit:feed-1");

    let before = app.platform_host.snapshot().window_model();
    let move_right = before.commands.palette_items("move right");
    assert_eq!(move_right.len(), 1);
    assert_eq!(
        move_right[0].command_id,
        "canvas.move_selected_unit.right".to_string()
    );
    assert!(move_right[0].enabled);
    assert_eq!(
        find_command_list_command_id(
            &before.commands.command_list_sections,
            "canvas.move_selected_unit.right"
        ),
        Some("canvas.move_selected_unit.right")
    );

    app.dispatch_ui_command("canvas.move_selected_unit.right");
    let moved = app.platform_host.snapshot().window_model();
    assert_eq!(
        moved
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(104.0, 40.0))
    );
    assert!(
        !moved.runtime.workspace_document.has_unsaved_changes,
        "layout nudge should only update the Studio layout sidecar"
    );
    let result = app
        .canvas_command_result_command_surface()
        .expect("expected canvas move command result");
    assert_eq!(result.status_label, "moved");
    assert_eq!(result.title, "Canvas unit moved");
    assert!(result.detail.contains("moved from sidecar (64.0, 40.0)"));
    assert_eq!(result.target_command_id, "inspector.focus_unit:feed-1");

    let layout_path = studio_layout_path_for_project(&project_path);
    let stored_layout = read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert!(stored_layout.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "feed-1" && position.x == 104.0 && position.y == 40.0
    }));

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_selected_unit_direct_position_move_updates_only_layout_sidecar() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    app.save_project();
    app.dispatch_ui_command("inspector.focus_unit:feed-1");

    app.dispatch_canvas_unit_layout_move(
        rf_types::UnitId::new("feed-1"),
        rf_ui::CanvasPoint::new(180.0, 112.0),
    );

    let moved = app.platform_host.snapshot().window_model();
    assert_eq!(
        moved
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(180.0, 112.0))
    );
    assert!(
        !moved.runtime.workspace_document.has_unsaved_changes,
        "direct canvas unit move should only update the Studio layout sidecar"
    );
    let result = app
        .canvas_command_result_command_surface()
        .expect("expected direct canvas move command result");
    assert_eq!(result.status_label, "moved");
    assert_eq!(result.title, "Canvas unit moved");
    assert!(result.detail.contains("moved from sidecar (64.0, 40.0)"));
    assert_eq!(result.target_command_id, "inspector.focus_unit:feed-1");

    let layout_path = studio_layout_path_for_project(&project_path);
    let stored_layout = read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert!(stored_layout.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "feed-1" && position.x == 180.0 && position.y == 112.0
    }));

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_empty_click_clears_selection_without_moving_selected_unit() {
    let (config, project_path) = blank_workspace_config();
    let layout_path = studio_layout_path_for_project(&project_path);
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    app.save_project();
    app.dispatch_ui_command("inspector.focus_unit:feed-1");

    app.clear_canvas_selection();

    let cleared = app.platform_host.snapshot().window_model();
    assert_eq!(cleared.runtime.active_inspector_target, None);
    assert_eq!(
        cleared
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(64.0, 40.0))
    );
    assert!(
        !cleared.runtime.workspace_document.has_unsaved_changes,
        "clearing canvas selection must not dirty the project"
    );

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_viewport_offset_persists_in_layout_sidecar_without_dirtying_project() {
    let (config, project_path) = blank_workspace_config();
    let layout_path = studio_layout_path_for_project(&project_path);
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    app.save_project();
    let project_before = fs::read_to_string(&project_path).expect("expected saved project file");
    assert!(
        !app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );

    app.update_canvas_viewport_offset(egui::vec2(42.0, -16.0));

    let moved_viewport = app.platform_host.snapshot().window_model();
    assert!(
        !moved_viewport
            .runtime
            .workspace_document
            .has_unsaved_changes,
        "viewport pan must only update the Studio layout sidecar"
    );
    assert_eq!(
        fs::read_to_string(&project_path).expect("expected project file after viewport pan"),
        project_before,
        "viewport pan must not rewrite the project JSON"
    );

    let stored_layout = read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert_eq!(
        stored_layout
            .canvas_viewport
            .as_ref()
            .map(|viewport| (viewport.offset_x, viewport.offset_y)),
        Some((42.0, -16.0))
    );

    app.open_project(project_path.clone(), "project");
    assert_eq!(
        app.canvas_initial_viewport_fit,
        CanvasInitialViewportFitState::restore(egui::vec2(42.0, -16.0))
    );

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_viewport_fit_to_content_resets_sidecar_offset_without_dirtying_project() {
    let (config, project_path) = blank_workspace_config();
    let layout_path = studio_layout_path_for_project(&project_path);
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    app.save_project();
    let project_before = fs::read_to_string(&project_path).expect("expected saved project file");
    app.update_canvas_viewport_offset(egui::vec2(42.0, -16.0));

    let view = app
        .platform_host
        .snapshot()
        .window_model()
        .canvas
        .widget
        .view()
        .clone();
    let viewport_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 280.0));

    let fitted_offset =
        app.fit_canvas_viewport_to_content(viewport_rect, &view.unit_blocks, &view.stream_lines);

    assert_ne!(fitted_offset, egui::vec2(42.0, -16.0));
    let fitted = app.platform_host.snapshot().window_model();
    assert!(
        !fitted.runtime.workspace_document.has_unsaved_changes,
        "viewport fit must only update the Studio layout sidecar"
    );
    assert_eq!(
        fs::read_to_string(&project_path).expect("expected project file after viewport fit"),
        project_before,
        "viewport fit must not rewrite the project JSON"
    );
    let stored_layout = read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert_eq!(
        stored_layout
            .canvas_viewport
            .as_ref()
            .map(|viewport| (viewport.offset_x, viewport.offset_y)),
        Some((fitted_offset.x as f64, fitted_offset.y as f64))
    );
    assert_eq!(
        app.canvas_initial_viewport_fit,
        CanvasInitialViewportFitState::restore(fitted_offset)
    );
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str(),
            result.target.command_id.as_str()
        )),
        Some((
            RunPanelNoticeLevel::Info,
            "viewport_fit",
            "Canvas viewport fit to content",
            "canvas.fit_to_content"
        ))
    );
    assert!(
        app.canvas_command_result_command_surface()
            .expect("expected viewport fit command result")
            .matches_query("canvas result viewport fit")
    );

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_unit_layout_nudge_pins_transient_grid_without_dirtying_project() {
    let (config, project_path) = flash_drum_local_rules_config();
    let layout_path = studio_layout_path_for_project(&project_path);
    let _ = fs::remove_file(&layout_path);
    let project_before = fs::read_to_string(&project_path).expect("expected project file");
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("inspector.focus_unit:feed-1");
    let before = app.platform_host.snapshot().window_model();
    let revision_before = before.runtime.workspace_document.revision;
    let saved_revision_before = before.runtime.workspace_document.last_saved_revision;
    assert!(
        !before.runtime.workspace_document.has_unsaved_changes,
        "fixture should start from a saved project"
    );
    assert_eq!(
        before
            .canvas
            .widget
            .view()
            .current_selection
            .as_ref()
            .and_then(|selection| selection.layout_source_label),
        Some("transient grid")
    );
    assert_eq!(
        before
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        None,
        "fixture should not have a sidecar position before the first nudge"
    );

    app.dispatch_ui_command("canvas.move_selected_unit.right");

    let moved = app.platform_host.snapshot().window_model();
    assert_eq!(moved.runtime.workspace_document.revision, revision_before);
    assert_eq!(
        moved.runtime.workspace_document.last_saved_revision,
        saved_revision_before
    );
    assert!(
        !moved.runtime.workspace_document.has_unsaved_changes,
        "layout nudge must not dirty the project document"
    );
    assert_eq!(
        fs::read_to_string(&project_path).expect("expected project file after nudge"),
        project_before,
        "layout nudge must not rewrite the project JSON"
    );
    assert_eq!(
        moved
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(58.0, 72.0))
    );
    assert_eq!(
        moved
            .canvas
            .widget
            .view()
            .current_selection
            .as_ref()
            .and_then(|selection| selection.layout_source_label),
        Some("sidecar position")
    );
    let result = app
        .canvas_command_result_command_surface()
        .expect("expected canvas move command result");
    assert_eq!(result.status_label, "moved");
    assert!(result.detail.contains("had no sidecar position"));
    assert!(
        result
            .detail
            .contains("pinned from its transient grid slot")
    );
    assert_eq!(result.target_command_id, "inspector.focus_unit:feed-1");

    let stored_layout = read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert!(stored_layout.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "feed-1" && position.x == 58.0 && position.y == 72.0
    }));

    let reopened = ready_app_state(&config)
        .platform_host
        .snapshot()
        .window_model();
    assert_eq!(
        reopened
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(58.0, 72.0)),
        "reopened project should restore the pinned sidecar position"
    );

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_feed_heater_flash_minimal_path_can_run_after_parameters() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .as_deref(),
        Some("local.feed.create_outlet.feed-2")
    );
    app.dispatch_ui_command("canvas.accept_focused");

    app.dispatch_ui_command("canvas.begin_place_unit.heater");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(180.0, 40.0));
    let after_heater = app.platform_host.snapshot().window_model();
    assert_eq!(after_heater.canvas.suggestion_count, 1);
    assert_eq!(
        after_heater.canvas.focused_suggestion_id.as_deref(),
        Some("local.heater.connect_inlet.heater-2.stream-feed-2-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_heater_inlet = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_heater_inlet.canvas.focused_suggestion_id.as_deref(),
        Some("local.heater.create_outlet.heater-2")
    );
    assert_eq!(after_heater_inlet.canvas.suggestion_count, 1);

    app.dispatch_ui_command("canvas.accept_focused");
    let after_heater_outlet = app.platform_host.snapshot().window_model();
    assert_eq!(after_heater_outlet.canvas.suggestion_count, 0);

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(320.0, 40.0));
    let after_flash = app.platform_host.snapshot().window_model();
    assert_eq!(after_flash.canvas.suggestion_count, 1);
    assert_eq!(
        after_flash.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-2.stream-heater-2-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");

    commit_unit_parameter(
        &mut app,
        "heater-2",
        "unit:heater-2:outlet_temperature_k",
        "330",
    );
    commit_unit_parameter(
        &mut app,
        "heater-2",
        "unit:heater-2:outlet_pressure_pa",
        "90000",
    );
    commit_unit_parameter(
        &mut app,
        "flash-2",
        "unit:flash-2:outlet_temperature_k",
        "300",
    );
    commit_unit_parameter(
        &mut app,
        "flash-2",
        "unit:flash-2:outlet_pressure_pa",
        "85000",
    );

    app.dispatch_ui_command("run_panel.run_manual");
    let completed = app.platform_host.snapshot().window_model();
    assert_eq!(
        completed.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(completed.runtime.control_state.pending_reason, None);
    assert!(
        completed
            .runtime
            .control_state
            .latest_snapshot_id
            .as_deref()
            .is_some_and(|snapshot_id| snapshot_id.contains("rev-13-seq-1"))
    );
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-heater-2-outlet")
    );
}

#[test]
fn canvas_feed_mixer_flash_minimal_path_can_run_after_parameters() {
    let mut app = ready_app_state(&synced_workspace_config());

    for point in [
        rf_ui::CanvasPoint::new(64.0, 40.0),
        rf_ui::CanvasPoint::new(64.0, 140.0),
    ] {
        app.dispatch_ui_command("canvas.begin_place_unit.feed");
        app.dispatch_canvas_pending_edit_commit(point);
        app.dispatch_ui_command("canvas.accept_focused");
    }

    app.dispatch_ui_command("canvas.begin_place_unit.mixer");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(210.0, 90.0));
    let after_mixer = app.platform_host.snapshot().window_model();
    assert_eq!(after_mixer.canvas.suggestion_count, 2);
    assert_eq!(
        after_mixer.canvas.focused_suggestion_id.as_deref(),
        Some("local.mixer.connect_inlet_a.mixer-1.stream-feed-2-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_mixer_inlet_a = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_mixer_inlet_a.canvas.focused_suggestion_id.as_deref(),
        Some("local.mixer.connect_inlet_b.mixer-1.stream-feed-3-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_mixer_inlet_b = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_mixer_inlet_b.canvas.focused_suggestion_id.as_deref(),
        Some("local.mixer.create_outlet.mixer-1")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_mixer_outlet = app.platform_host.snapshot().window_model();
    assert_eq!(after_mixer_outlet.canvas.suggestion_count, 0);

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(360.0, 90.0));
    let after_flash = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_flash.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-2.stream-mixer-1-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");

    commit_unit_parameter(
        &mut app,
        "mixer-1",
        "unit:mixer-1:outlet_pressure_pa",
        "90000",
    );
    commit_unit_parameter(
        &mut app,
        "flash-2",
        "unit:flash-2:outlet_temperature_k",
        "300",
    );
    commit_unit_parameter(
        &mut app,
        "flash-2",
        "unit:flash-2:outlet_pressure_pa",
        "85000",
    );

    app.dispatch_ui_command("run_panel.run_manual");
    let completed = app.platform_host.snapshot().window_model();
    assert_eq!(
        completed.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(completed.runtime.control_state.pending_reason, None);
    assert!(
        completed
            .runtime
            .control_state
            .latest_snapshot_id
            .as_deref()
            .is_some_and(|snapshot_id| snapshot_id.contains("rev-15-seq-1"))
    );
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-mixer-1-outlet")
    );
}

#[test]
fn canvas_pending_edit_commit_reports_missing_pending_edit_through_command_result() {
    let mut app = ready_app_state(&lease_expiring_config());

    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(12.0, 24.0));

    assert_eq!(app.canvas_viewport_navigation.active_anchor, None);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str(),
            result.target.kind_label,
            result.target.target_id.as_str(),
            result.target.command_id.as_str(),
        )),
        Some((
            RunPanelNoticeLevel::Warning,
            "pending_edit_unavailable",
            "Canvas pending edit unavailable",
            "Edit",
            "pending_edit",
            "canvas.commit_pending_edit_at",
        ))
    );
    assert!(
        app.canvas_command_result
            .as_ref()
            .map(|result| result.detail.contains("no pending edit was active"))
            .unwrap_or(false)
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line
                == "Canvas pending edit unavailable: Edit pending_edit (Pending canvas edit)")
    );
    assert!(
        !app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    let surface = app
        .canvas_command_result_command_surface()
        .expect("expected canvas command result command surface");
    assert_eq!(surface.status_label, "pending_edit_unavailable");
    assert_eq!(surface.target_command_id, "canvas.commit_pending_edit_at");
    assert!(surface.matches_query("pending edit unavailable"));
}

#[test]
fn canvas_pending_edit_commit_reports_dispatch_error_through_command_result() {
    let mut app = ready_app_state(&lease_expiring_config());

    app.record_canvas_pending_edit_commit_error(
        rf_ui::CanvasPoint::new(12.0, 24.0),
        "[invalid_input] canvas place unit intent uses unsupported unit kind `Pump`",
    );

    assert_eq!(app.canvas_viewport_navigation.active_anchor, None);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str(),
            result.target.kind_label,
            result.target.target_id.as_str(),
            result.target.command_id.as_str(),
        )),
        Some((
            RunPanelNoticeLevel::Error,
            "pending_edit_failed",
            "Canvas pending edit failed",
            "Edit",
            "pending_edit",
            "canvas.commit_pending_edit_at",
        ))
    );
    assert!(
        app.canvas_command_result
            .as_ref()
            .map(|result| result.detail.contains("unsupported unit kind `Pump`"))
            .unwrap_or(false)
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line
                == "Canvas pending edit failed: Edit pending_edit (Pending canvas edit)")
    );
}

#[test]
fn canvas_viewport_navigation_reports_missing_inspector_target() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("inspector.focus_unit:missing-unit");

    assert_eq!(app.canvas_viewport_navigation.active_anchor, None);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str()
        )),
        Some((
            RunPanelNoticeLevel::Error,
            "dispatch_failed",
            "Canvas object navigation failed"
        ))
    );
    assert!(
        app.canvas_command_result
            .as_ref()
            .map(|result| result.detail.contains("missing-unit"))
            .unwrap_or(false)
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("Canvas object navigation failed: Unit missing-unit"))
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn canvas_viewport_navigation_reconciles_against_current_presentation_focus() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);
    app.dispatch_ui_command("inspector.focus_unit:flash-1");
    assert!(app.canvas_viewport_navigation.active_anchor.is_some());

    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let window = app.platform_host.snapshot().window_model();
    app.reconcile_canvas_viewport_navigation(window.canvas.widget.view().viewport.focus.as_ref());

    assert_eq!(
        app.canvas_viewport_navigation
            .active_anchor
            .as_ref()
            .map(|focus| focus.anchor_label.as_str()),
        Some("stream-feed:0")
    );

    app.reconcile_canvas_viewport_navigation(None);

    assert_eq!(app.canvas_viewport_navigation.active_anchor, None);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str()
        )),
        Some((
            RunPanelNoticeLevel::Warning,
            "anchor_expired",
            "Canvas navigation anchor expired"
        ))
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn right_sidebar_width_keeps_runtime_panel_readable() {
    let width = region_panel_width_from_values(
        StudioGuiWindowDockRegion::RightSidebar,
        1_280.0,
        100.0,
        24.0,
    );

    assert_eq!(width, 360.0);
}
