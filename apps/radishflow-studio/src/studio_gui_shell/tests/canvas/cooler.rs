use super::*;

#[test]
fn cooler_temperature_edits_preserve_saved_input_and_refresh_downstream_results() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);
    author_single_inlet_flash_case_from_blank(
        &mut app,
        SingleInletFlashAuthoringCase {
            case_name: "cooler",
            begin_unit_command: "canvas.begin_place_unit.cooler",
            unit_id: "cooler-1",
            connect_unit_inlet_suggestion: "local.cooler.connect_inlet.cooler-1.stream-feed-1-outlet",
            create_unit_outlet_suggestion: "local.cooler.create_outlet.cooler-1",
            unit_outlet_stream_id: "stream-cooler-1-outlet",
            unit_outlet_temperature_k: Some(286.0),
            unit_outlet_pressure_pa: 90_000.0,
            flash_temperature_k: 280.0,
            flash_pressure_pa: 80_000.0,
        },
    );
    app.dispatch_ui_command("run_panel.run_manual");
    let initial = app.platform_host.snapshot().window_model();
    let initial_snapshot = initial.runtime.latest_solve_snapshot.unwrap();
    let initial_revision = initial.runtime.workspace_document.revision;
    let field = "unit:cooler-1:outlet_temperature_k";

    for raw in ["0", "-1", "NaN", "inf", "abc"] {
        app.dispatch_ui_command("inspector.focus_unit:cooler-1");
        app.dispatch_inspector_field_draft_update(
            radishflow_studio::inspector_draft_update_command_id(field),
            raw,
        );
        app.dispatch_inspector_field_draft_commit(
            radishflow_studio::inspector_draft_commit_command_id(field),
        );
        let rejected = app.platform_host.snapshot().window_model();
        assert_eq!(
            rejected.runtime.workspace_document.revision,
            initial_revision
        );
        app.save_project();
        let saved = read_project_file(&project_path).unwrap();
        assert_eq!(saved.document.revision, initial_revision);
        assert_eq!(
            saved.document.flowsheet.units[&UnitId::new("cooler-1")]
                .parameters
                .outlet_temperature_k,
            Some(286.0),
            "invalid draft {raw} must not replace the committed temperature"
        );
        assert_eq!(
            saved.document.flowsheet.streams[&StreamId::new("stream-cooler-1-outlet")]
                .temperature_k,
            286.0
        );
    }

    commit_unit_parameter(&mut app, "cooler-1", field, "275");
    let edited = app.platform_host.snapshot().window_model();
    assert_eq!(
        edited.runtime.workspace_document.revision,
        initial_revision + 1
    );
    assert_stale_solve_snapshot_notice(&edited, &initial_snapshot);
    let export_path = project_path.with_extension("txt");
    app.export_solve_snapshot_to_path(&initial_snapshot, export_path.clone());
    assert!(
        !export_path.exists(),
        "old temperature results cannot be exported"
    );

    app.save_project();
    let saved = read_project_file(&project_path).unwrap();
    assert_eq!(
        saved.document.flowsheet.units[&UnitId::new("cooler-1")]
            .parameters
            .outlet_temperature_k,
        Some(275.0)
    );
    assert_eq!(
        saved.document.flowsheet.streams[&StreamId::new("stream-cooler-1-outlet")].temperature_k,
        275.0
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
    assert_single_inlet_unit_result_consistency(
        &snapshot,
        "stream-feed-1-outlet",
        "stream-cooler-1-outlet",
        275.0,
        90_000.0,
    );
    let cooler = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == "cooler-1")
        .unwrap();
    let flash = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == "flash-1")
        .unwrap();
    assert_eq!(
        cooler.produced_stream_results,
        flash.consumed_stream_results
    );
    assert_ne!(
        snapshot_stream(&snapshot, "stream-cooler-1-outlet").molar_enthalpy_j_per_mol,
        snapshot_stream(&initial_snapshot, "stream-cooler-1-outlet").molar_enthalpy_j_per_mol,
        "rerun must recompute outlet enthalpy after the temperature change"
    );
    assert_flash_split_material_balance(
        &snapshot,
        "stream-cooler-1-outlet",
        "stream-flash-1-liquid",
        "stream-flash-1-vapor",
    );
    let project_bytes = fs::read(&project_path).unwrap();
    app.export_solve_snapshot_to_path(&snapshot, export_path.clone());
    let exported = fs::read_to_string(&export_path).unwrap();
    assert_eq!(exported, snapshot.light_text_export());
    assert!(exported.contains("T 275.00 K | P 90000 Pa | F 1.000000 mol/s"));
    assert_eq!(fs::read(&project_path).unwrap(), project_bytes);
    assert!(
        !app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    fs::remove_file(export_path).unwrap();
    let _ = fs::remove_file(studio_layout_path_for_project(&project_path));
    fs::remove_file(project_path).unwrap();
}
