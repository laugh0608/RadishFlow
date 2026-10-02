use super::*;

#[test]
fn close_confirmation_keeps_keyboard_focus_inside_and_escape_preserves_the_draft() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        let mut app = ready_app_state(&synced_workspace_config());
        app.locale = locale;
        app.dispatch_ui_command("inspector.focus_stream:stream-feed");
        app.dispatch_inspector_field_draft_update(
            radishflow_studio::inspector_draft_update_command_id("stream:stream-feed:pressure_pa"),
            "1e-",
        );
        let before = app.platform_host.snapshot().runtime.workspace_document;
        assert!(!app.close_current_window_for_viewport_request());
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        ctx.set_visuals(egui::Visuals::light());
        let mut frame = |key: Option<egui::Key>| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1024.0, 665.0),
                    )),
                    focused: true,
                    events: key
                        .map(|key| egui::Event::Key {
                            key,
                            physical_key: Some(key),
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        })
                        .into_iter()
                        .collect(),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let _ = ui.button("Background action");
                    });
                    app.render_pending_close_window_dialog(ctx);
                },
            )
        };
        // Settle area sizing and the initial focus before traversing every action twice.
        let _ = frame(None);
        let _ = frame(None);
        let output = frame(None);
        let tree = output.platform_output.accesskit_update.unwrap();
        let focused = tree.nodes.iter().find(|(id, _)| *id == tree.focus).unwrap();
        assert_eq!(
            focused.1.label(),
            Some(locale.text(ShellText::CancelCloseProject))
        );
        for expected in [
            ShellText::SaveAndCloseProject,
            ShellText::DiscardAndCloseProject,
            ShellText::CancelCloseProject,
        ]
        .into_iter()
        .cycle()
        .take(6)
        {
            let _ = frame(Some(egui::Key::Tab));
            // egui completes the last-to-first Tab wrap on the following pass.
            let output = frame(None);
            let tree = output.platform_output.accesskit_update.unwrap();
            let focused = tree.nodes.iter().find(|(id, _)| *id == tree.focus).unwrap();
            assert_eq!(
                focused.1.label(),
                Some(locale.text(expected)),
                "Tab must traverse each action without moving into the background"
            );
        }
        let _ = frame(Some(egui::Key::Escape));
        assert!(app.project_open.pending_close_window_confirmation.is_none());
        assert_eq!(
            app.platform_host.snapshot().runtime.workspace_document,
            before
        );
        assert_eq!(app.logical_window_count(), 1);
    }
}

#[test]
fn command_shortcuts_survive_modifier_release_before_the_frame_and_are_consumed_once() {
    for key in [egui::Key::Q, egui::Key::K] {
        let mut app = ready_app_state(&synced_workspace_config());
        app.dispatch_ui_command("inspector.focus_stream:stream-feed");
        app.dispatch_inspector_field_draft_update(
            radishflow_studio::inspector_draft_update_command_id("stream:stream-feed:pressure_pa"),
            "1e-",
        );
        let before = app.platform_host.snapshot().runtime.workspace_document;
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput {
            focused: true,
            // A fast native key chord can be completely released before the next frame.
            modifiers: egui::Modifiers::NONE,
            events: vec![
                egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                },
                egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed: false,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        });
        match key {
            egui::Key::Q => {
                assert!(app.handle_quit_shortcut(&ctx));
                assert!(app.project_open.pending_close_window_confirmation.is_some());
                assert!(!app.handle_quit_shortcut(&ctx));
            }
            egui::Key::K => {
                assert!(app.handle_command_palette_toggle_shortcut(&ctx));
                assert!(app.command_palette.open);
                assert!(!app.handle_command_palette_toggle_shortcut(&ctx));
                assert!(app.command_palette.open);
            }
            _ => unreachable!(),
        }
        let _ = ctx.end_pass();
        assert_eq!(
            app.platform_host.snapshot().runtime.workspace_document,
            before
        );
        assert_eq!(app.logical_window_count(), 1);
    }
}
