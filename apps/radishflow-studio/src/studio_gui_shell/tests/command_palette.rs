use super::*;

#[test]
fn command_palette_state_open_close_and_selection_reset_cleanly() {
    let mut state = CommandPaletteState {
        open: false,
        query: "recover".to_string(),
        selected_index: 3,
        focus_query_input: false,
    };

    state.open();

    assert!(state.open);
    assert!(state.query.is_empty());
    assert_eq!(state.selected_index, 0);
    assert!(state.focus_query_input);

    state.close();

    assert!(!state.open);
    assert!(state.query.is_empty());
    assert_eq!(state.selected_index, 0);
    assert!(!state.focus_query_input);
}

#[test]
fn command_palette_state_moves_selection_within_bounds() {
    let commands = palette_commands_for_test(&[
        ("run_panel.run_manual", true),
        ("run_panel.recover_failure", false),
        ("run_panel.resume_workspace", true),
    ]);
    let mut state = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 0,
        focus_query_input: false,
    };

    state.move_selection(1, &commands);
    assert_eq!(state.selected_index, 2);

    state.move_selection(1, &commands);
    assert_eq!(state.selected_index, 2);

    state.move_selection(-5, &commands);
    assert_eq!(state.selected_index, 0);

    let empty_commands: [&StudioGuiCommandEntry; 0] = [];
    state.move_selection(1, &empty_commands);
    assert_eq!(state.selected_index, 0);
}

#[test]
fn command_palette_state_syncs_disabled_selection_to_nearest_enabled_command() {
    let commands = palette_commands_for_test(&[
        ("run_panel.run_manual", false),
        ("run_panel.resume_workspace", false),
        ("run_panel.set_active", true),
        ("run_panel.recover_failure", false),
    ]);
    let mut state = CommandPaletteState {
        open: true,
        query: String::new(),
        selected_index: 1,
        focus_query_input: false,
    };

    state.sync_selection(&commands);

    assert_eq!(state.selected_index, 2);
}

#[test]
fn selected_palette_command_id_ignores_disabled_entries() {
    let commands = palette_commands_for_test(&[
        ("run_panel.run_manual", false),
        ("run_panel.resume_workspace", true),
    ]);

    assert_eq!(
        selected_palette_command_id(&commands, 0),
        Some("run_panel.resume_workspace".to_string())
    );
}

#[test]
fn focus_context_prioritizes_command_palette_over_canvas_focus() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);
    assert!(
        app.platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .is_some()
    );

    app.command_palette.open();

    run_with_key_press(egui::Key::F5, egui::Modifiers::NONE, |ctx| {
        assert_eq!(
            app.focus_context(ctx),
            StudioGuiFocusContext::CommandPalette
        );
    });

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn focus_context_prioritizes_text_input_over_canvas_focus() {
    let (config, project_path) = flash_drum_local_rules_config();
    let app = ready_app_state(&config);
    assert!(
        app.platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .is_some()
    );

    run_with_key_press_and_focus(
        egui::Key::F5,
        egui::Modifiers::NONE,
        egui::Id::new("studio.test_input"),
        |ctx| {
            assert_eq!(app.focus_context(ctx), StudioGuiFocusContext::TextInput);
        },
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn focus_context_reports_canvas_when_canvas_was_last_interacted_area() {
    let mut app = ready_app_state(&lease_expiring_config());
    app.last_area_focus = Some(StudioGuiWindowAreaId::Canvas);

    run_with_key_press(egui::Key::F5, egui::Modifiers::NONE, |ctx| {
        assert_eq!(app.focus_context(ctx), StudioGuiFocusContext::Canvas);
    });
}

#[test]
fn focus_context_keeps_global_when_non_canvas_area_was_last_interacted() {
    let mut app = ready_app_state(&lease_expiring_config());
    app.last_area_focus = Some(StudioGuiWindowAreaId::Runtime);

    run_with_key_press(egui::Key::F5, egui::Modifiers::NONE, |ctx| {
        assert_eq!(app.focus_context(ctx), StudioGuiFocusContext::Global);
    });
}

#[test]
fn command_palette_toggle_shortcut_opens_and_closes_palette() {
    let mut app = ready_app_state(&lease_expiring_config());
    assert!(!app.command_palette.open);

    run_with_key_press(
        egui::Key::K,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..egui::Modifiers::NONE
        },
        |ctx| {
            assert!(app.handle_command_palette_toggle_shortcut(ctx));
        },
    );
    assert!(app.command_palette.open);
    assert!(app.command_palette.focus_query_input);

    run_with_key_press(
        egui::Key::K,
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..egui::Modifiers::NONE
        },
        |ctx| {
            assert!(app.handle_command_palette_toggle_shortcut(ctx));
        },
    );
    assert!(!app.command_palette.open);
}

#[test]
fn command_palette_enter_executes_selected_command_and_closes_palette() {
    let mut app = ready_app_state(&lease_expiring_config());
    app.command_palette.open();
    app.command_palette.query = "activate".to_string();

    let commands = app.platform_host.snapshot().window_model().commands;
    run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
        assert!(app.handle_command_palette_keyboard(ctx, &commands));
    });

    assert!(!app.command_palette.open);
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .control_state
            .simulation_mode,
        SimulationMode::Active
    );
}
