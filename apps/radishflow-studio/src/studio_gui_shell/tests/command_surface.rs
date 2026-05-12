use super::*;

#[test]
fn command_surface_interactions_converge_to_same_window_state_for_activate_workspace() {
    let mut apps = ready_command_surface_apps(&lease_expiring_config());
    let initial_window = shared_command_surface_initial_window(&apps);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &initial_window,
        "run_panel.set_active",
        "activate",
        egui::Key::F6,
        egui::Modifiers {
            shift: true,
            ..egui::Modifiers::NONE
        },
    );

    let menu_window = command_surface_window(&apps.menu_app);

    assert!(!apps.palette_app.command_palette.open);
    assert_eq!(
        menu_window.runtime.control_state.simulation_mode,
        SimulationMode::Active
    );
    assert_command_surface_windows_equal(&apps, &menu_window);
}

#[test]
fn command_surface_interactions_converge_to_same_window_state_for_resume_workspace() {
    let mut apps = ready_failed_command_surface_apps();
    let failed_window = shared_command_surface_initial_window(&apps);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &failed_window,
        "run_panel.recover_failure",
        "diagnostic",
        egui::Key::F8,
        egui::Modifiers::NONE,
    );
    let recovered_window = command_surface_window(&apps.menu_app);
    assert_command_surface_windows_equal(&apps, &recovered_window);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &recovered_window,
        "run_panel.resume_workspace",
        "resume",
        egui::Key::F5,
        egui::Modifiers {
            shift: true,
            ..egui::Modifiers::NONE
        },
    );

    let menu_window = command_surface_window(&apps.menu_app);

    assert!(!apps.palette_app.command_palette.open);
    assert_eq!(
        menu_window.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(menu_window.runtime.control_state.pending_reason, None);
    assert_eq!(
        menu_window
            .runtime
            .control_state
            .latest_snapshot_id
            .as_deref(),
        Some("example-unbound-outlet-port-rev-1-seq-1")
    );
    assert_eq!(
        menu_window.runtime.run_panel.view().status_label,
        "Converged"
    );
    assert_command_surface_windows_equal(&apps, &menu_window);
}

#[test]
fn command_surface_interactions_converge_to_same_window_state_for_hold_workspace() {
    let mut apps = ready_command_surface_apps(&lease_expiring_config());
    let initial_window = shared_command_surface_initial_window(&apps);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &initial_window,
        "run_panel.set_active",
        "activate",
        egui::Key::F6,
        egui::Modifiers {
            shift: true,
            ..egui::Modifiers::NONE
        },
    );
    let active_window = command_surface_window(&apps.menu_app);
    assert_command_surface_windows_equal(&apps, &active_window);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &active_window,
        "run_panel.set_hold",
        "hold",
        egui::Key::F6,
        egui::Modifiers::NONE,
    );

    let menu_window = command_surface_window(&apps.menu_app);

    assert!(!apps.palette_app.command_palette.open);
    assert_eq!(
        menu_window.runtime.control_state.simulation_mode,
        SimulationMode::Hold
    );
    assert_eq!(
        menu_window.runtime.control_state.pending_reason,
        Some(rf_ui::SolvePendingReason::ModeActivated)
    );
    assert!(!menu_window.runtime.control_state.can_set_hold);
    assert!(menu_window.runtime.control_state.can_set_active);
    assert_eq!(menu_window.runtime.run_panel.view().mode_label, "Hold");
    assert_command_surface_windows_equal(&apps, &menu_window);
}

#[test]
fn command_surface_interactions_converge_to_same_window_state_for_canvas_focus_next() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut apps = ready_command_surface_apps(&config);
    let initial_window = shared_command_surface_initial_window(&apps);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &initial_window,
        "canvas.focus_next",
        "next suggestion",
        egui::Key::Tab,
        egui::Modifiers {
            ctrl: true,
            ..egui::Modifiers::NONE
        },
    );

    let menu_window = command_surface_window(&apps.menu_app);

    assert!(!apps.palette_app.command_palette.open);
    assert_eq!(
        menu_window.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.create_outlet.flash-1.liquid")
    );
    assert_eq!(
        menu_window
            .canvas
            .widget
            .view()
            .focused_suggestion_id
            .as_deref(),
        Some("local.flash_drum.create_outlet.flash-1.liquid")
    );
    assert_command_surface_windows_equal(&apps, &menu_window);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn command_surface_interactions_converge_to_same_window_state_for_canvas_reject_focused() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut apps = ready_command_surface_apps(&config);
    let initial_window = shared_command_surface_initial_window(&apps);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &initial_window,
        "canvas.reject_focused",
        "reject",
        egui::Key::Escape,
        egui::Modifiers::NONE,
    );

    let menu_window = command_surface_window(&apps.menu_app);

    assert!(!apps.palette_app.command_palette.open);
    assert_eq!(menu_window.canvas.suggestion_count, 3);
    assert_eq!(
        menu_window.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.create_outlet.flash-1.liquid")
    );
    assert_eq!(
        menu_window
            .canvas
            .widget
            .view()
            .focused_suggestion_id
            .as_deref(),
        Some("local.flash_drum.create_outlet.flash-1.liquid")
    );
    assert_command_surface_windows_equal(&apps, &menu_window);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn command_surface_interactions_converge_to_same_window_state_for_canvas_accept_focused() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut apps = ready_command_surface_apps(&config);
    let initial_window = shared_command_surface_initial_window(&apps);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &initial_window,
        "run_panel.set_active",
        "activate",
        egui::Key::F6,
        egui::Modifiers {
            shift: true,
            ..egui::Modifiers::NONE
        },
    );
    let active_window = command_surface_window(&apps.menu_app);
    assert_command_surface_windows_equal(&apps, &active_window);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &active_window,
        "canvas.accept_focused",
        "accept",
        egui::Key::Tab,
        egui::Modifiers::NONE,
    );

    let menu_window = command_surface_window(&apps.menu_app);

    assert!(!apps.palette_app.command_palette.open);
    assert_eq!(
        menu_window.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(menu_window.runtime.control_state.pending_reason, None);
    assert_eq!(
        menu_window
            .runtime
            .control_state
            .latest_snapshot_id
            .as_deref(),
        Some(OFFICIAL_HEATER_BINARY_HYDROCARBON_AUTORUN_SNAPSHOT_ID)
    );
    assert_eq!(
        menu_window.runtime.run_panel.view().status_label,
        "Converged"
    );
    assert_command_surface_windows_equal(&apps, &menu_window);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn command_surface_interactions_converge_to_same_window_state_for_run_panel_recovery() {
    let mut apps = ready_failed_command_surface_apps();
    let failed_window = shared_command_surface_initial_window(&apps);

    dispatch_enabled_command_surface_interactions(
        &mut apps,
        &failed_window,
        "run_panel.recover_failure",
        "diagnostic",
        egui::Key::F8,
        egui::Modifiers::NONE,
    );

    let menu_window = command_surface_window(&apps.menu_app);

    assert!(!apps.palette_app.command_palette.open);
    assert_eq!(
        menu_window.runtime.control_state.run_status,
        rf_ui::RunStatus::Dirty
    );
    assert_eq!(
        menu_window.runtime.control_state.pending_reason,
        Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
    );
    assert_eq!(
        menu_window.runtime.run_panel.view().primary_action.label,
        "Resume"
    );
    assert_command_surface_windows_equal(&apps, &menu_window);
}

#[test]
fn disabled_command_surface_interactions_do_not_change_window_state_for_run_panel_recovery() {
    let mut apps = ready_command_surface_apps(&lease_expiring_config());
    let initial_window = shared_command_surface_initial_window(&apps);

    dispatch_disabled_command_surface_interactions(
        &mut apps,
        &initial_window,
        "run_panel.recover_failure",
        "diagnostic",
        egui::Key::F8,
        egui::Modifiers::NONE,
    );

    assert_command_surface_windows_equal(&apps, &initial_window);
    assert!(apps.palette_app.command_palette.open);
    assert_eq!(apps.palette_app.command_palette.query, "diagnostic");
}

#[test]
fn dispatch_shortcuts_does_not_leak_host_shortcuts_while_palette_is_open() {
    let mut app = ready_app_state(&lease_expiring_config());
    app.command_palette.open();

    run_with_key_press(
        egui::Key::F6,
        egui::Modifiers {
            shift: true,
            ..egui::Modifiers::NONE
        },
        |ctx| app.dispatch_shortcuts(ctx),
    );

    assert!(app.command_palette.open);
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .control_state
            .simulation_mode,
        SimulationMode::Hold
    );
}

#[test]
fn dispatch_shortcuts_allows_function_keys_from_text_input_context() {
    let mut app = ready_app_state(&lease_expiring_config());

    run_with_key_press_and_focus(
        egui::Key::F6,
        egui::Modifiers {
            shift: true,
            ..egui::Modifiers::NONE
        },
        egui::Id::new("studio.test_input"),
        |ctx| app.dispatch_shortcuts(ctx),
    );

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

#[test]
fn command_palette_items_surface_window_model_results() {
    let app = ready_app_state(&lease_expiring_config());

    let filtered = app
        .platform_host
        .snapshot()
        .window_model()
        .commands
        .palette_items("activate");

    assert_eq!(
        filtered
            .into_iter()
            .map(|item| item.command_id)
            .collect::<Vec<_>>(),
        vec!["run_panel.set_active".to_string()]
    );
}

#[test]
fn command_palette_can_focus_canvas_object_navigation_command() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);
    app.command_palette.open();
    app.command_palette.query = "viewport flash-1".to_string();
    let commands = app.platform_host.snapshot().window_model().commands;

    let palette_items = commands.palette_items(&app.command_palette.query);
    assert_eq!(palette_items.len(), 1);
    assert_eq!(palette_items[0].command_id, "inspector.focus_unit:flash-1");
    assert_eq!(
        palette_items[0].menu_path_text,
        "Canvas > Objects > Unit > Flash Drum"
    );

    let consumed = run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
        app.handle_command_palette_keyboard(ctx, &commands)
    });

    assert!(consumed);
    assert!(!app.command_palette.open);
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Unit", "flash-1"))
    );
    assert_eq!(
        app.canvas_viewport_navigation
            .active_anchor
            .as_ref()
            .map(|focus| focus.anchor_label.as_str()),
        Some("unit-slot-1")
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn command_palette_can_focus_latest_solve_snapshot_result_command() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.command_palette.open();
    app.command_palette.query = "result snapshot stream-vapor".to_string();
    let commands = app.platform_host.snapshot().window_model().commands;

    let palette_items = commands.palette_items(&app.command_palette.query);
    assert!(
        palette_items.iter().any(|item| {
            item.command_id == "inspector.focus_stream:stream-vapor"
                && item.menu_path_text == "Results > Streams > Vapor Outlet"
                && item.detail.contains("SolveSnapshot")
        }),
        "expected command palette to include latest result stream focus command: {palette_items:?}"
    );

    let selected_index = palette_items
        .iter()
        .position(|item| item.command_id == "inspector.focus_stream:stream-vapor")
        .expect("expected result stream palette item");
    app.command_palette.selected_index = selected_index;
    let consumed = run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
        app.handle_command_palette_keyboard(ctx, &commands)
    });

    assert!(consumed);
    assert!(!app.command_palette.open);
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Stream", "stream-vapor"))
    );
}

#[test]
fn results_command_surface_can_focus_latest_solve_snapshot_from_menu_and_list() {
    let mut menu_app = ready_app_state(&synced_workspace_config());
    let mut list_app = ready_app_state(&synced_workspace_config());
    menu_app.dispatch_ui_command("run_panel.run_manual");
    list_app.dispatch_ui_command("run_panel.run_manual");

    let menu_window = command_surface_window(&menu_app);
    let stream_menu_command = find_menu_command_by_path(
        &menu_window.commands.menu_tree,
        &["Results", "Streams", "Vapor Outlet"],
    )
    .cloned()
    .expect("expected result stream menu command");
    assert_eq!(
        stream_menu_command.command_id,
        "inspector.focus_stream:stream-vapor"
    );
    assert!(stream_menu_command.enabled);
    assert!(stream_menu_command.hover_text.contains("SolveSnapshot"));

    menu_app.dispatch_menu_command(&stream_menu_command);
    assert_eq!(
        menu_app
            .platform_host
            .snapshot()
            .window_model()
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Stream", "stream-vapor"))
    );

    let list_window = command_surface_window(&list_app);
    let unit_command_id = list_window
        .commands
        .command_list_sections
        .iter()
        .find(|section| section.title == "Results")
        .and_then(|section| {
            section.items.iter().find(|item| {
                item.command_id == "inspector.focus_unit:flash-1"
                    && item.menu_path_text == "Results > Units > flash-1"
                    && item.detail.contains("SolveSnapshot")
            })
        })
        .map(|item| item.command_id.clone())
        .expect("expected result unit command list item");

    list_app.dispatch_ui_command(unit_command_id);
    assert_eq!(
        list_app
            .platform_host
            .snapshot()
            .window_model()
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Unit", "flash-1"))
    );
}

fn find_menu_command_by_path<'a>(
    nodes: &'a [StudioGuiCommandMenuNode],
    path: &[&str],
) -> Option<&'a StudioGuiCommandMenuCommandModel> {
    let (label, remaining) = path.split_first()?;
    let node = nodes.iter().find(|node| node.label == *label)?;
    if remaining.is_empty() {
        return node.command.as_ref();
    }
    find_menu_command_by_path(&node.children, remaining)
}
