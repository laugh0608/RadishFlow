use super::*;

#[test]
fn disabled_reconnect_buttons_show_the_reason_on_hover() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    app.select_canvas_stream("inspector.focus_stream:stream-feed");
    app.dispatch_ui_command("canvas.disconnect_selected_stream");
    let window = app.platform_host.snapshot().window_model();
    let ctx = egui::Context::default();
    ctx.style_mut(|style| {
        style.interaction.tooltip_delay = 0.0;
        style.interaction.show_tooltips_only_when_still = false;
        style.animation_time = 0.0;
    });
    let mut frame = |events| {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1440.0, 960.0),
            )),
            events,
            focused: true,
            ..Default::default()
        };
        rendered_text_rects(&ctx.run(input, |ctx| {
            app.render_right_sidebar(ctx, &window, &mut false);
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_canvas_area(ui, &window, StudioGuiWindowAreaId::Canvas);
            });
        }))
    };
    let _ = frame(vec![]);
    let buttons: Vec<_> = frame(vec![])
        .into_iter()
        .filter(|(label, _)| label == "Reconnect stream")
        .map(|(_, rect)| rect.center())
        .collect();
    assert_eq!(
        buttons.len(),
        2,
        "expected an Inspector and a Canvas button"
    );
    for pos in buttons {
        let _ = frame(vec![egui::Event::PointerGone]);
        let _ = frame(vec![egui::Event::PointerMoved(pos)]);
        let _ = frame(vec![]); // Allow the previous tooltip to close and the next to size itself.
        let hovered = frame(vec![]);
        assert!(
            hovered
                .iter()
                .any(|(text, _)| text.contains("no material endpoint to reconnect")),
            "missing rejection reason at {pos:?}"
        );
    }

    let _ = fs::remove_file(project_path);
}

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
    let reconnect = window
        .canvas
        .widget
        .action(radishflow_studio::StudioGuiCanvasActionId::ReconnectSelectedStream)
        .expect("reconnect control for the selected stream");
    assert!(!reconnect.enabled);
    assert!(
        reconnect
            .detail
            .contains("no material endpoint to reconnect")
    );
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

#[test]
fn reconnect_picker_returns_native_focus_and_rechecks_an_occupied_second_choice() {
    use crate::studio_gui_shell::tests::reconnect_picker::{
        click, frame, mixer_with_two_open_ports, node,
    };
    use radishflow_studio::{
        StudioBackgroundRunAction,
        flow_actions::{FlowAction, FlowActionRequest, FlowEditAccess, FlowEditContext},
        modeling_actions::{MaterialConnectionTarget, MaterialPortTarget},
    };
    let mut app = mixer_with_two_open_ports("sink");
    let before = app.platform_host.modeling_inputs().clone();
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let mut tree = frame(&mut app, &ctx, None);
    for index in [0, 1] {
        let entries: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, n)| n.label() == Some("Choose sink port…"))
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(entries.len(), 2);
        frame(&mut app, &ctx, Some(click(entries[index])));
        tree = frame(&mut app, &ctx, None);
        frame(
            &mut app,
            &ctx,
            Some(click(node(&tree, "Mixer · mixer-1:inlet_b"))),
        );
        tree = frame(&mut app, &ctx, None);
        frame(&mut app, &ctx, Some(click(node(&tree, "Cancel"))));
        tree = frame(&mut app, &ctx, None);
        assert_eq!(
            tree.nodes
                .iter()
                .filter(|(_, n)| n.label() == Some("Choose sink port…"))
                .nth(index)
                .unwrap()
                .0,
            tree.focus
        );
        assert_eq!(app.platform_host.modeling_inputs(), &before);
    }
    frame(
        &mut app,
        &ctx,
        Some(click(node(&tree, "Choose sink port…"))),
    );
    tree = frame(&mut app, &ctx, None);
    frame(
        &mut app,
        &ctx,
        Some(click(node(&tree, "Mixer · mixer-1:inlet_b"))),
    );
    let expected = FlowEditContext::capture(&app.platform_host.modeling_inputs().workspace);
    let port = |id: &str, name: &str| MaterialPortTarget {
        unit_id: id.into(),
        port: name.into(),
    };
    app.platform_host
        .background_run_action(StudioBackgroundRunAction::EditFlow {
            access: FlowEditAccess::Editable,
            request: FlowActionRequest {
                expected,
                action: FlowAction::ConnectExact(MaterialConnectionTarget {
                    stream_id: "stream-feed-b".into(),
                    source: port("feed-b", "outlet"),
                    sink: Some(port("mixer-1", "inlet_b")),
                }),
            },
        })
        .unwrap();
    tree = frame(&mut app, &ctx, None);
    let occupied = app.platform_host.modeling_inputs().clone();
    frame(
        &mut app,
        &ctx,
        Some(click(node(&tree, "Confirm reconnection"))),
    );
    tree = frame(&mut app, &ctx, None);
    assert!(tree.nodes.iter().any(|(_, n)| n.value()
        == Some("mixer-1:inlet_b is connected to stream-feed-b")
        || n.label() == Some("mixer-1:inlet_b is connected to stream-feed-b")));
    frame(
        &mut app,
        &ctx,
        Some(click(node(&tree, "Refresh available ports"))),
    );
    tree = frame(&mut app, &ctx, None);
    let confirm = node(&tree, "Confirm reconnection");
    assert!(
        tree.nodes
            .iter()
            .find(|(id, _)| *id == confirm)
            .unwrap()
            .1
            .is_disabled()
    );
    assert_eq!(app.platform_host.modeling_inputs(), &occupied);
    app.dispatch_ui_command("inspector.focus_stream:stream-feed-b");
    app.reconcile_reconnect_picker();
    assert!(app.reconnect_picker.is_none());
}
