use super::*;

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
