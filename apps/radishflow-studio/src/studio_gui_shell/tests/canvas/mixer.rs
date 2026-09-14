use super::*;

fn author_mixer_flash_from_blank(config: &StudioRuntimeConfig) -> ReadyAppState {
    let mut app = ready_app_state(config);
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

    app
}

#[test]
fn blank_project_mixer_path_saves_reopens_and_reruns() {
    let (config, project_path) = blank_workspace_config();
    let mut app = author_mixer_flash_from_blank(&config);
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
    let snapshot = rerun
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected rerun solve snapshot");
    let rerun_streams = &snapshot.streams;
    let rerun_mixer_outlet = rerun_streams
        .iter()
        .find(|stream| stream.stream_id == "stream-mixer-1-outlet")
        .expect("expected mixer outlet rerun result");
    assert_eq!(rerun_mixer_outlet.total_molar_flow_mol_s, 2.0);
    assert_eq!(rerun_mixer_outlet.pressure_pa, 90_000.0);
    assert_stream_fraction(rerun_mixer_outlet, "methane", 0.5625);
    assert_stream_fraction(rerun_mixer_outlet, "ethane", 0.4375);
    assert_mixer_weighted_result(
        snapshot,
        &["stream-feed-1-outlet", "stream-feed-2-outlet"],
        "stream-mixer-1-outlet",
        90_000.0,
    );
    assert_flash_split_material_balance(
        snapshot,
        "stream-mixer-1-outlet",
        "stream-flash-1-liquid",
        "stream-flash-1-vapor",
    );
    assert_flash_outlet_phase_review_semantics(
        &mut app,
        snapshot,
        "stream-flash-1-liquid",
        "stream-flash-1-vapor",
        Some("flash-1"),
    );
    assert_case_review_summary_covers_flow(
        snapshot,
        &["stream-feed-1-outlet", "stream-feed-2-outlet"],
        &["stream-mixer-1-outlet"],
        &["stream-flash-1-liquid", "stream-flash-1-vapor"],
        &["feed-1", "feed-2", "mixer-1", "flash-1"],
    );
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
    assert_bottom_result_table_contains_streams_and_steps(
        &mut app,
        snapshot,
        &[
            "stream-feed-1-outlet",
            "stream-feed-2-outlet",
            "stream-mixer-1-outlet",
            "stream-flash-1-liquid",
            "stream-flash-1-vapor",
        ],
        &["mixer-1", "flash-1"],
    );
    for stream_id in [
        "stream-feed-1-outlet",
        "stream-feed-2-outlet",
        "stream-mixer-1-outlet",
        "stream-flash-1-liquid",
        "stream-flash-1-vapor",
    ] {
        assert_result_inspector_renders_stream_review_object(
            &mut app,
            snapshot,
            stream_id,
            Some("flash-1"),
        );
    }
    assert_result_inspector_renders_unit_stream_references(
        &mut app,
        snapshot,
        "stream-mixer-1-outlet",
        "mixer-1",
    );
    assert_result_inspector_renders_unit_stream_references(
        &mut app,
        snapshot,
        "stream-mixer-1-outlet",
        "flash-1",
    );

    let export_path = project_path.with_extension("txt");
    app.export_solve_snapshot_to_path(snapshot, export_path.clone());
    let exported = fs::read_to_string(&export_path).expect("expected case author export read");
    assert!(exported.contains("Review\ncategory\titems"));
    assert!(exported.contains("source_streams\tstream-feed-1-outlet"));
    assert!(exported.contains("stream-feed-2-outlet"));
    assert!(exported.contains("intermediate_streams\tstream-mixer-1-outlet"));
    assert!(exported.contains("terminal_streams\tstream-flash-1-liquid"));
    assert!(exported.contains("units\tfeed-1 status=Converged"));
    assert!(exported.contains("mixer-1 status=Converged"));
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
fn mixer_feed_flow_edits_propagate_through_saved_results_and_export() {
    let (config, project_path) = blank_workspace_config();
    let mut app = author_mixer_flash_from_blank(&config);
    for (unit, temperature) in [("feed-1", "300"), ("feed-2", "330")] {
        commit_unit_parameter(
            &mut app,
            unit,
            &format!("unit:{unit}:outlet_temperature_k"),
            temperature,
        );
    }
    normalize_stream_composition(
        &mut app,
        "stream-feed-1-outlet",
        &[("methane", "0.5"), ("ethane", "0.5")],
    );
    normalize_stream_composition(
        &mut app,
        "stream-feed-2-outlet",
        &[("methane", "0.8"), ("ethane", "0.2")],
    );
    let export_path = project_path.with_extension("txt");
    let mut previous = None;
    for (flow, expected_flow, temperature, methane) in
        [("2", 3.0, 310.0, 0.6), ("1", 2.0, 315.0, 0.65)]
    {
        commit_stream_field(
            &mut app,
            "stream-feed-1-outlet",
            "total_molar_flow_mol_s",
            flow,
        );
        if let Some(snapshot) = previous.as_ref() {
            let edited = app.platform_host.snapshot().window_model();
            assert_stale_solve_snapshot_notice(&edited, snapshot);
            app.export_solve_snapshot_to_path(snapshot, export_path.clone());
            assert!(!export_path.exists(), "stale mixture must not be exported");
        }
        app.save_project();
        let saved = read_project_file(&project_path).unwrap();
        assert_eq!(
            saved.document.flowsheet.streams[&StreamId::new("stream-feed-1-outlet")]
                .total_molar_flow_mol_s,
            flow.parse::<f64>().unwrap()
        );
        app.open_project(project_path.clone(), "project");
        assert!(
            app.platform_host
                .snapshot()
                .window_model()
                .runtime
                .latest_solve_snapshot
                .is_none()
        );
        app.dispatch_ui_command("run_panel.run_manual");
        let rerun = app.platform_host.snapshot().window_model();
        assert_eq!(
            rerun.runtime.control_state.run_status,
            rf_ui::RunStatus::Converged
        );
        let snapshot = rerun.runtime.latest_solve_snapshot.unwrap();
        assert_eq!(snapshot.document_revision, saved.document.revision);
        let mixed = snapshot_stream(&snapshot, "stream-mixer-1-outlet");
        assert_close(mixed.total_molar_flow_mol_s, expected_flow);
        assert_close(mixed.temperature_k, temperature);
        assert_stream_fraction(mixed, "methane", methane);
        assert_stream_fraction(mixed, "ethane", 1.0 - methane);
        assert_mixer_weighted_result(
            &snapshot,
            &["stream-feed-1-outlet", "stream-feed-2-outlet"],
            "stream-mixer-1-outlet",
            90_000.0,
        );
        assert_flash_split_material_balance(
            &snapshot,
            "stream-mixer-1-outlet",
            "stream-flash-1-liquid",
            "stream-flash-1-vapor",
        );
        let mixer = snapshot
            .steps
            .iter()
            .find(|step| step.unit_id == "mixer-1")
            .unwrap();
        let flash = snapshot
            .steps
            .iter()
            .find(|step| step.unit_id == "flash-1")
            .unwrap();
        assert_eq!(mixer.produced_stream_results, flash.consumed_stream_results);
        assert_bottom_result_table_contains_streams_and_steps(
            &mut app,
            &snapshot,
            &[
                "stream-feed-1-outlet",
                "stream-feed-2-outlet",
                "stream-mixer-1-outlet",
            ],
            &["mixer-1", "flash-1"],
        );
        let project_bytes = fs::read(&project_path).unwrap();
        app.export_solve_snapshot_to_path(&snapshot, export_path.clone());
        let exported = fs::read_to_string(&export_path).unwrap();
        assert_eq!(exported, snapshot.light_text_export());
        assert!(exported.contains(&format!(
            "T {temperature:.2} K | P 90000 Pa | F {expected_flow:.6} mol/s"
        )));
        assert_eq!(fs::read(&project_path).unwrap(), project_bytes);
        assert!(
            !app.platform_host
                .snapshot()
                .window_model()
                .runtime
                .workspace_document
                .has_unsaved_changes
        );
        fs::remove_file(&export_path).unwrap();
        previous = Some(snapshot);
    }
    let _ = fs::remove_file(studio_layout_path_for_project(&project_path));
    fs::remove_file(project_path).unwrap();
}
