use super::*;
use radishflow_studio::StudioGuiShortcutPlatform;

fn primary_modifiers() -> egui::Modifiers {
    egui::Modifiers {
        command: true,
        mac_cmd: cfg!(target_os = "macos"),
        ctrl: !cfg!(target_os = "macos"),
        ..egui::Modifiers::NONE
    }
}

#[test]
fn platform_shortcuts_preserve_event_modifiers_and_deduplicate_repeats() {
    let command = egui::Modifiers {
        command: true,
        mac_cmd: true,
        shift: true,
        ..egui::Modifiers::NONE
    };
    let event = egui::Event::Key {
        key: egui::Key::Z,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: command,
    };
    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput {
        // Modifier release later in the same frame must not change the chord.
        modifiers: egui::Modifiers::NONE,
        events: vec![
            event.clone(),
            event,
            egui::Event::Key {
                key: egui::Key::Z,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        ..Default::default()
    });
    assert_eq!(
        ctx.input(collect_shortcuts),
        vec![StudioGuiShortcutPlatform::MacOs.redo_shortcut()]
    );
    let _ = ctx.end_pass();
}

#[test]
fn platform_shortcuts_keep_physical_control_and_extra_modifiers() {
    for (mods, expected) in [
        (
            egui::Modifiers {
                command: true,
                mac_cmd: true,
                ..egui::Modifiers::NONE
            },
            vec![StudioGuiShortcutModifier::Primary],
        ),
        (
            egui::Modifiers {
                command: true,
                ctrl: true,
                ..egui::Modifiers::NONE
            },
            vec![StudioGuiShortcutModifier::Primary],
        ),
        (
            egui::Modifiers {
                command: true,
                mac_cmd: true,
                ctrl: true,
                ..egui::Modifiers::NONE
            },
            vec![
                StudioGuiShortcutModifier::Primary,
                StudioGuiShortcutModifier::Ctrl,
            ],
        ),
    ] {
        run_with_key_press(egui::Key::Z, mods, |ctx| {
            assert_eq!(
                ctx.input(collect_shortcuts),
                vec![StudioGuiShortcut {
                    modifiers: expected,
                    key: StudioGuiShortcutKey::Z,
                }]
            );
        });
    }
    run_with_key_press(egui::Key::Z, egui::Modifiers::CTRL, |ctx| {
        assert!(
            ctx.input(collect_shortcuts).is_empty(),
            "physical Control alone on macOS is not Command"
        );
    });
    for mods in [
        egui::Modifiers::CTRL,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..egui::Modifiers::NONE
        },
    ] {
        run_with_key_press(egui::Key::Tab, mods, |ctx| {
            assert_eq!(
                ctx.input(collect_shortcuts),
                vec![StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                    key: StudioGuiShortcutKey::Tab,
                }]
            );
        });
    }
    run_with_key_press(
        egui::Key::Tab,
        egui::Modifiers {
            mac_cmd: true,
            command: true,
            ..egui::Modifiers::NONE
        },
        |ctx| {
            assert_eq!(
                ctx.input(collect_shortcuts)[0].modifiers,
                vec![StudioGuiShortcutModifier::Primary]
            );
        },
    );
}

#[test]
fn platform_shortcuts_history_commits_once_and_save_reopens() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(100.0, 100.0));
    app.dispatch_ui_command("canvas.begin_place_unit.heater");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(300.0, 100.0));
    let before = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .clone();
    assert_eq!(before.unit_count, 2);
    let primary = primary_modifiers();
    let redo = StudioGuiShortcutPlatform::current().redo_shortcut();
    let redo_key = if redo.key == StudioGuiShortcutKey::Z {
        egui::Key::Z
    } else {
        egui::Key::Y
    };
    let redo_mods = egui::Modifiers {
        shift: redo.modifiers.contains(&StudioGuiShortcutModifier::Shift),
        ..primary
    };
    for (key, mods, count, revision_delta) in [
        (egui::Key::Z, primary, 1, 1),
        (redo_key, redo_mods, 2, 2),
        (egui::Key::Z, primary, 1, 3),
        (egui::Key::Y, primary, 2, 4),
    ] {
        dispatch_shortcut_for_test(&mut app, key, mods);
        let document = app.platform_host.snapshot().runtime.workspace_document;
        assert_eq!(document.unit_count, count);
        assert_eq!(document.revision, before.revision + revision_delta);
    }
    let stable = app.platform_host.snapshot().runtime.workspace_document;
    run_with_key_press_and_focus(
        egui::Key::Z,
        primary,
        egui::Id::new("history-text"),
        |ctx| {
            app.dispatch_shortcuts(ctx);
        },
    );
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        stable
    );
    dispatch_shortcut_for_test(&mut app, egui::Key::S, primary);
    let saved = read_project_file(&project_path).unwrap();
    assert_eq!(saved.document.revision, stable.revision);
    assert_eq!(saved.document.flowsheet.units.len(), 2);
    let reopened = ready_app_state(&config);
    assert_eq!(
        reopened
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .unit_count,
        2
    );
    let _ = fs::remove_file(studio_layout_path_for_project(&project_path));
    fs::remove_file(project_path).unwrap();
}

#[test]
fn platform_shortcuts_pending_document_dialogs_block_history() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(100.0, 100.0));
    let before = app.platform_host.snapshot().runtime.workspace_document;
    for kind in 0..4 {
        match kind {
            0 => app.project_open.pending_blank_project_confirmation = true,
            1 => {
                app.project_open.pending_save_as_overwrite =
                    Some(PathBuf::from("unused.rfproj.json"))
            }
            2 => app.project_open.pending_close_window_confirmation = app.current_window_id(),
            _ => app.command_palette.open = true,
        }
        dispatch_shortcut_for_test(&mut app, egui::Key::Z, primary_modifiers());
        assert_eq!(
            app.platform_host.snapshot().runtime.workspace_document,
            before
        );
        app.project_open.pending_blank_project_confirmation = false;
        app.project_open.pending_save_as_overwrite = None;
        app.project_open.pending_close_window_confirmation = None;
        app.command_palette.open = false;
    }
}
