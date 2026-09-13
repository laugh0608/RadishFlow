use super::*;

fn rename(app: &mut ReadyAppState, name: &str) {
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id("unit:heater-1:name"),
        name,
    );
    app.dispatch_inspector_field_draft_commit(
        radishflow_studio::inspector_draft_commit_command_id("unit:heater-1:name"),
    );
}

#[test]
fn unit_rename_updates_presentations_preserves_topology_and_reopens_for_rerun() {
    let path = test_preferences_path("rename-unit").with_extension("rfproj.json");
    let original = feed_heater_flash_binary_hydrocarbon_project();
    write_project_file(&path, &original).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    app.dispatch_canvas_unit_layout_move(
        UnitId::new("heater-1"),
        rf_ui::CanvasPoint::new(210.0, 90.0),
    );
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_unit:heater-1");
    let before = app.platform_host.snapshot().window_model();
    rename(&mut app, "  ");
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .revision,
        before.runtime.workspace_document.revision
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_some()
    );
    let name = "主加热器 A";
    rename(&mut app, name);
    let renamed = app.platform_host.snapshot().window_model();
    assert_eq!(
        renamed.runtime.workspace_document.revision,
        before.runtime.workspace_document.revision + 1
    );
    assert!(renamed.runtime.latest_solve_snapshot.is_none());
    assert!(renamed.runtime.stale_solve_snapshot.is_some());
    let view = renamed.canvas.widget.view();
    assert_eq!(
        view.unit_blocks
            .iter()
            .find(|u| u.unit_id == "heater-1")
            .unwrap()
            .name,
        name
    );
    assert_eq!(
        view.object_list
            .items
            .iter()
            .find(|u| u.target_id == "heater-1")
            .unwrap()
            .label,
        name
    );
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .active_inspector_detail
            .as_ref()
            .unwrap()
            .title,
        name
    );
    app.dispatch_ui_command("edit.undo");
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_none()
    );
    app.dispatch_ui_command("edit.redo");
    app.save_project();
    let saved = read_project_file(&path).unwrap();
    let mut expected = original.document.flowsheet;
    expected
        .units
        .get_mut(&UnitId::new("heater-1"))
        .unwrap()
        .name = name.into();
    assert_eq!(saved.document.flowsheet, expected);
    app.open_project(path.clone(), "renamed project");
    let reopened = app.platform_host.snapshot().window_model();
    let unit = reopened
        .canvas
        .widget
        .view()
        .unit_blocks
        .iter()
        .find(|u| u.unit_id == "heater-1")
        .unwrap();
    assert_eq!(unit.name, name);
    assert_eq!(
        unit.layout_position,
        Some(rf_ui::CanvasPoint::new(210.0, 90.0))
    );
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_some()
    );
    let _ = fs::remove_file(studio_layout_path_for_project(&path));
    let _ = fs::remove_file(path);
}
