use super::*;

fn snapshot(app: &ReadyAppState) -> StudioGuiWindowModel {
    app.platform_host.snapshot().window_model()
}

fn request_delete(app: &mut ReadyAppState, id: &str) {
    app.dispatch_ui_command(format!("inspector.focus_unit:{id}"));
    app.dispatch_ui_command("canvas.delete_selected_unit");
    assert!(app.pending_unit_deletion.is_some());
}

fn place(app: &mut ReadyAppState, kind: &str, position: rf_ui::CanvasPoint) -> String {
    app.dispatch_ui_command(format!("canvas.begin_place_unit.{kind}"));
    app.dispatch_canvas_pending_edit_commit(position);
    match app
        .platform_host
        .snapshot()
        .runtime
        .active_inspector_target
        .unwrap()
    {
        rf_ui::InspectorTarget::Unit(id) => id.as_str().to_string(),
        other => panic!("expected new unit, got {other:?}"),
    }
}

#[test]
fn blank_unit_deletion_matrix_confirms_cancels_and_restores_layout_after_other_moves() {
    for kind in ["feed", "heater", "cooler", "valve", "mixer", "flash_drum"] {
        let mut app = ready_app_state(&synced_workspace_config());
        app.create_blank_project();
        select_builtin_binary_hydrocarbon_basis(&mut app);
        let position = rf_ui::CanvasPoint::new(64.0, 85.0);
        let unit_id = place(&mut app, kind, position);
        let other_id = place(&mut app, "feed", rf_ui::CanvasPoint::new(240.0, 80.0));
        request_delete(&mut app, &unit_id);
        let before = snapshot(&app);
        app.pending_unit_deletion = None;
        assert_eq!(snapshot(&app), before, "cancellation must not commit");
        request_delete(&mut app, &unit_id);
        app.confirm_unit_deletion();
        let deleted = snapshot(&app);
        assert_eq!(deleted.runtime.workspace_document.unit_count, 1);
        assert_eq!(
            deleted.runtime.workspace_document.revision,
            before.runtime.workspace_document.revision + 1
        );
        assert!(
            app.platform_host
                .snapshot()
                .runtime
                .active_inspector_target
                .is_none()
        );
        app.confirm_unit_deletion();
        assert_eq!(snapshot(&app), deleted, "duplicate confirmation is ignored");
        app.dispatch_canvas_unit_layout_move(
            UnitId::new(other_id),
            rf_ui::CanvasPoint::new(300.0, 90.0),
        );
        app.dispatch_ui_command("edit.undo");
        let undone = snapshot(&app);
        assert_eq!(undone.runtime.workspace_document.unit_count, 2);
        assert_eq!(
            undone
                .canvas
                .widget
                .view()
                .unit_blocks
                .iter()
                .find(|u| u.unit_id == unit_id)
                .unwrap()
                .layout_position,
            Some(position)
        );
        app.dispatch_ui_command("edit.redo");
        assert_eq!(snapshot(&app).runtime.workspace_document.unit_count, 1);
        let new_id = place(&mut app, kind, rf_ui::CanvasPoint::new(400.0, 80.0));
        assert_ne!(new_id, unit_id);
    }
}

#[test]
fn deletion_confirmation_rejects_changed_selection_revision_and_document() {
    let mut app = ready_app_state(&synced_workspace_config());
    request_delete(&mut app, "heater-1");
    app.dispatch_ui_command("inspector.focus_unit:feed-1");
    let before = snapshot(&app).runtime.workspace_document;
    app.confirm_unit_deletion();
    assert_eq!(snapshot(&app).runtime.workspace_document, before);

    request_delete(&mut app, "heater-1");
    commit_unit_parameter(
        &mut app,
        "heater-1",
        "unit:heater-1:outlet_temperature_k",
        "330",
    );
    let before = snapshot(&app).runtime.workspace_document;
    app.confirm_unit_deletion();
    assert_eq!(snapshot(&app).runtime.workspace_document, before);

    request_delete(&mut app, "heater-1");
    app.create_blank_project();
    app.confirm_pending_blank_project();
    assert!(app.pending_unit_deletion.is_none());
    app.confirm_unit_deletion();
    assert_eq!(snapshot(&app).runtime.workspace_document.unit_count, 0);
}

#[test]
fn deleting_middle_unit_preserves_external_bindings_saves_reopens_and_undo_reruns() {
    let project_path = test_preferences_path("delete-middle").with_extension("rfproj.json");
    let original = feed_heater_flash_binary_hydrocarbon_project();
    write_project_file(&project_path, &original).unwrap();
    let config = StudioRuntimeConfig {
        project_path: project_path.clone(),
        ..synced_workspace_config()
    };
    let mut app = ready_app_state(&config);
    let position = rf_ui::CanvasPoint::new(321.0, 123.0);
    app.dispatch_canvas_unit_layout_move(UnitId::new("heater-1"), position);
    app.dispatch_ui_command("run_panel.run_manual");
    let previous_snapshot = snapshot(&app).runtime.latest_solve_snapshot.unwrap();
    request_delete(&mut app, "heater-1");
    app.confirm_unit_deletion();
    assert!(snapshot(&app).runtime.latest_solve_snapshot.is_none());
    assert!(snapshot(&app).runtime.stale_solve_snapshot.is_some());
    app.dispatch_ui_command(radishflow_studio::FILE_SAVE_COMMAND_ID);
    let deleted = read_project_file(&project_path).unwrap();
    assert_eq!(
        deleted.document.flowsheet.streams,
        original.document.flowsheet.streams
    );
    let mut expected_units = original.document.flowsheet.units.clone();
    expected_units.remove(&UnitId::new("heater-1"));
    assert_eq!(deleted.document.flowsheet.units, expected_units);
    assert!(
        !read_studio_layout_file(studio_layout_path_for_project(&project_path))
            .unwrap()
            .canvas_unit_positions
            .iter()
            .any(|p| p.unit_id == "heater-1")
    );

    // Restore and save before reopening: history remains session-local.
    app.dispatch_ui_command("edit.undo");
    assert_eq!(
        snapshot(&app)
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|u| u.unit_id == "heater-1")
            .unwrap()
            .layout_position,
        Some(position)
    );
    assert!(snapshot(&app).runtime.latest_solve_snapshot.is_none());
    app.save_project();
    app.open_project(project_path.clone(), "test restored project");
    assert_eq!(
        read_project_file(&project_path).unwrap().document.flowsheet,
        original.document.flowsheet
    );
    assert_eq!(
        snapshot(&app)
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|u| u.unit_id == "heater-1")
            .unwrap()
            .layout_position,
        Some(position)
    );
    app.dispatch_ui_command("run_panel.run_manual");
    let rerun = snapshot(&app).runtime.latest_solve_snapshot.unwrap();
    assert_ne!(rerun.snapshot_id, previous_snapshot.snapshot_id);
    assert_eq!(
        snapshot(&app).runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );

    request_delete(&mut app, "heater-1");
    app.confirm_unit_deletion();
    app.save_project();
    app.open_project(project_path.clone(), "test deleted project");
    assert_eq!(snapshot(&app).runtime.workspace_document.unit_count, 2);
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(
        snapshot(&app).runtime.latest_solve_snapshot.is_none(),
        "broken chain must not claim a current result"
    );
}

#[test]
fn untitled_project_first_save_preserves_only_live_unit_positions() {
    let target = test_preferences_path("untitled-layout").with_extension("rfproj.json");
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &synced_workspace_config(),
        test_preferences_path("untitled-layout-preferences"),
        Box::new(TestProjectFilePicker::new(Some(target.clone()))),
    )
    .unwrap();
    app.create_blank_project();
    select_builtin_binary_hydrocarbon_basis(&mut app);
    let feed = place(&mut app, "feed", rf_ui::CanvasPoint::new(55.0, 88.0));
    let heater = place(&mut app, "heater", rf_ui::CanvasPoint::new(200.0, 90.0));
    request_delete(&mut app, &heater);
    app.confirm_unit_deletion();
    app.save_project();
    assert!(
        !snapshot(&app)
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    app.open_project(target, "test first save");
    let blocks = snapshot(&app).canvas.widget.view().unit_blocks.clone();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].unit_id, feed);
    assert_eq!(
        blocks[0].layout_position,
        Some(rf_ui::CanvasPoint::new(55.0, 88.0))
    );
}

#[test]
fn deleting_unit_discards_invalid_draft_only_after_confirmation() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("inspector.focus_unit:heater-1");
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id("unit:heater-1:outlet_temperature_k"),
        "-1",
    );
    let before = snapshot(&app).runtime.active_inspector_detail.unwrap();
    app.dispatch_ui_command("canvas.delete_selected_unit");
    app.pending_unit_deletion = None;
    assert_eq!(snapshot(&app).runtime.active_inspector_detail, Some(before));
    app.dispatch_ui_command("canvas.delete_selected_unit");
    app.confirm_unit_deletion();
    app.dispatch_ui_command("edit.undo");
    app.dispatch_ui_command("inspector.focus_unit:heater-1");
    assert!(
        snapshot(&app)
            .runtime
            .active_inspector_detail
            .unwrap()
            .property_fields
            .iter()
            .all(|field| field.current_value != "-1")
    );
}

#[test]
fn layout_save_failure_reports_saved_project_and_can_be_retried() {
    let (config, path) = blank_workspace_config();
    let mut app = ready_app_state(&config);
    let sidecar = studio_layout_path_for_project(&path);
    fs::create_dir(&sidecar).unwrap();
    app.save_project();
    assert!(read_project_file(&path).is_ok());
    assert!(
        !snapshot(&app)
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    assert_eq!(
        app.project_open.notice.as_ref().unwrap().title,
        "项目已保存，布局保存失败"
    );
    fs::remove_dir(&sidecar).unwrap();
    app.save_project();
    assert!(read_studio_layout_file(&sidecar).is_ok());
    assert_eq!(
        app.project_open.notice.as_ref().unwrap().title,
        "Project saved"
    );
}
