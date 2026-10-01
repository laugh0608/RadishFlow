use super::*;

fn render_canvas(app: &mut ReadyAppState, frame: &StudioGuiWindowModel) {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            app.render_canvas_area(ui, frame, StudioGuiWindowAreaId::Canvas);
        });
    });
}

#[test]
fn navigation_survives_selection_after_the_frame_snapshot_for_units_and_streams() {
    let (config, path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);
    for command in [
        "inspector.focus_unit:flash-1",
        "inspector.focus_stream:stream-feed",
    ] {
        let old_frame = app.platform_host.snapshot().window_model();
        app.dispatch_ui_command(command);
        render_canvas(&mut app, &old_frame);
        let result = app.canvas_command_result.as_ref().unwrap();
        assert_eq!(result.status_label, "located");
        assert_eq!(result.target.command_id, command);
        let active = app
            .canvas_viewport_navigation
            .active_anchor
            .as_ref()
            .unwrap();
        assert!(
            !active.pending_scroll,
            "current geometry consumes the focus request"
        );
        assert_eq!(
            Some(active.anchor_label.as_str()),
            result.anchor_label.as_deref()
        );
    }
    let _ = fs::remove_file(path);
}

#[test]
fn navigation_relocates_surviving_object_when_topology_changes_layout_slots() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.create_blank_project();
    select_builtin_binary_hydrocarbon_basis(&mut app);
    for (kind, accepts) in [("feed", 1), ("heater", 2), ("flash_drum", 0)] {
        app.dispatch_ui_command(format!("canvas.begin_place_unit.{kind}"));
        app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(100.0, 80.0));
        for _ in 0..accepts {
            app.dispatch_ui_command("canvas.accept_focused");
        }
    }
    let before = app.platform_host.snapshot().window_model();
    let target = before
        .canvas
        .widget
        .view()
        .viewport
        .focus
        .as_ref()
        .unwrap()
        .clone();
    app.dispatch_ui_command("canvas.accept_focused");
    let after = app.platform_host.snapshot().window_model();
    let focus = after.canvas.widget.view().viewport.focus.as_ref().unwrap();
    assert_eq!(focus.target_id, target.target_id);
    assert_ne!(
        focus.anchor_label, target.anchor_label,
        "fixture must exercise slot renumbering"
    );
    app.reconcile_canvas_viewport_navigation(after.canvas.widget.view());
    let result = app.canvas_command_result.as_ref().unwrap();
    assert_eq!(result.status_label, "located");
    assert_eq!(
        result.anchor_label.as_deref(),
        Some(focus.anchor_label.as_str())
    );
}

#[test]
fn clearing_selection_does_not_report_existing_object_as_expired() {
    let (config, path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);
    app.dispatch_ui_command("inspector.focus_unit:flash-1");
    app.dispatch_event(StudioGuiEvent::WindowTriggerRequested {
        window_id: app.current_window_id().unwrap(),
        trigger: StudioRuntimeTrigger::ClearInspectorTarget,
    });
    let window = app.platform_host.snapshot().window_model();
    app.reconcile_canvas_viewport_navigation(window.canvas.widget.view());
    assert!(app.canvas_command_result.is_none());
    assert!(app.canvas_viewport_navigation.active_anchor.is_none());
    let _ = fs::remove_file(path);
}
