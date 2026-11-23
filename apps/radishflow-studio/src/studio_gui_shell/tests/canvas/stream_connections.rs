use super::*;

#[test]
fn disconnected_stream_can_be_reselected_without_a_canvas_line() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);
    let stream_command = "inspector.focus_stream:stream-feed";
    app.dispatch_ui_command(stream_command);
    app.dispatch_ui_command("canvas.disconnect_selected_stream");
    let window = app.platform_host.snapshot().window_model();
    let view = window.canvas.widget.view();
    app.reconcile_canvas_viewport_navigation(view);
    assert!(app.canvas_viewport_navigation.active_anchor.is_none());
    assert!(app.canvas_command_result.is_none());
    let command = window
        .commands
        .command_list_sections
        .iter()
        .flat_map(|section| &section.items)
        .find(|entry| entry.command_id == stream_command)
        .expect("disconnected stream can also be inspected from the command list");
    assert!(command.enabled);
    assert_eq!(command.label, "Inspect Stream Feed");
    assert!(command.detail.contains("No connected ports"));

    app.dispatch_ui_command("inspector.focus_unit:feed-1");
    let item = view
        .object_list
        .items
        .iter()
        .find(|item| item.command_id == stream_command)
        .expect("project tree still offers the disconnected stream");
    app.select_canvas_stream(&item.command_id);
    let window = app.platform_host.snapshot().window_model();
    let view = window.canvas.widget.view();
    assert_eq!(
        view.current_selection.as_ref().unwrap().target_id,
        "stream-feed"
    );
    assert!(view.viewport.focus.is_none());
    assert!(app.canvas_viewport_navigation.active_anchor.is_none());
    assert!(app.canvas_command_result.is_none());
    assert_eq!(
        app.platform_host.snapshot().runtime.active_inspector_target,
        Some(rf_ui::InspectorTarget::Stream(StreamId::new("stream-feed")))
    );

    app.dispatch_ui_command("edit.undo");
    app.select_canvas_stream(stream_command);
    assert_eq!(
        app.canvas_command_result.as_ref().unwrap().status_label,
        "located"
    );
    assert!(app.canvas_viewport_navigation.active_anchor.is_some());

    let _ = fs::remove_file(project_path);
}
