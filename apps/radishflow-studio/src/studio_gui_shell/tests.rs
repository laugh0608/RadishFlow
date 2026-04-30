use super::*;
use radishflow_studio::{
    StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed, StudioRuntimeTrigger,
};
use rf_store::read_project_file;
use std::{fs, path::PathBuf, time::UNIX_EPOCH};

fn lease_expiring_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Auto,
        entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
        ..StudioRuntimeConfig::default()
    }
}

fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-local-rules-{timestamp}.rfproj.json"
    ));
    let project =
            include_str!("../../../../examples/flowsheets/feed-heater-flash.rfproj.json")
                .replacen(
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-heated\"",
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-liquid\"",
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                );
    fs::write(&project_path, project).expect("expected local rules project");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..lease_expiring_config()
        },
        project_path,
    )
}

fn synced_workspace_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
        ..StudioRuntimeConfig::default()
    }
}

fn flash_drum_local_rules_synced_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-local-rules-synced-{timestamp}.rfproj.json"
    ));
    let project =
            include_str!("../../../../examples/flowsheets/feed-heater-flash.rfproj.json")
                .replacen(
                    ",\n        \"stream-vapor\": {\n          \"id\": \"stream-vapor\",\n          \"name\": \"Vapor Outlet\",\n          \"temperature_k\": 345.0,\n          \"pressure_pa\": 95000.0,\n          \"total_molar_flow_mol_s\": 0.0,\n          \"overall_mole_fractions\": {\n            \"component-a\": 0.5,\n            \"component-b\": 0.5\n          },\n          \"phases\": []\n        }",
                    "",
                    1,
                )
                .replacen(
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                );
    fs::write(&project_path, project).expect("expected synced local rules project");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..synced_workspace_config()
        },
        project_path,
    )
}

fn unbound_outlet_failure_synced_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples")
            .join("flowsheets")
            .join("failures")
            .join("unbound-outlet-port.rfproj.json"),
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
        trigger: StudioRuntimeTrigger::WidgetAction(rf_ui::RunPanelActionId::RunManual),
    }
}

fn test_preferences_path(name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    std::env::temp_dir()
        .join(format!("radishflow-studio-shell-{name}-{timestamp}"))
        .join("preferences.rfstudio-preferences.json")
}

#[test]
fn insert_neighbors_from_area_ids_returns_previous_and_next_for_middle_target() {
    let area_ids = [
        StudioGuiWindowAreaId::Commands,
        StudioGuiWindowAreaId::Canvas,
        StudioGuiWindowAreaId::Runtime,
    ];

    let (previous, next) = insert_neighbors_from_area_ids(&area_ids, 1);

    assert_eq!(previous, Some(StudioGuiWindowAreaId::Commands));
    assert_eq!(next, Some(StudioGuiWindowAreaId::Runtime));
}

#[test]
fn insert_neighbors_from_area_ids_clamps_to_stack_end() {
    let area_ids = [
        StudioGuiWindowAreaId::Commands,
        StudioGuiWindowAreaId::Canvas,
    ];

    let (previous, next) = insert_neighbors_from_area_ids(&area_ids, 8);

    assert_eq!(previous, Some(StudioGuiWindowAreaId::Commands));
    assert_eq!(next, None);
}

#[test]
fn clamp_overlay_pos_to_rect_keeps_overlay_inside_screen_padding() {
    let screen = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(200.0, 120.0));
    let size = egui::vec2(80.0, 40.0);

    let clamped = clamp_overlay_pos_to_rect(screen, egui::pos2(180.0, 110.0), size);

    assert_eq!(clamped, egui::pos2(112.0, 72.0));
}

#[test]
fn shell_locale_defaults_to_chinese_and_can_translate_runtime_labels() {
    let locale = StudioShellLocale::default();

    assert_eq!(locale, StudioShellLocale::ZhCn);
    assert_eq!(locale.text(ShellText::Runtime), "运行");
    assert_eq!(locale.runtime_label("Converged").as_ref(), "已收敛");
    assert_eq!(
        locale.workspace_counts("Demo", 2, 3, 1),
        "Demo | 2 个单元 | 3 股流股 | 1 个快照"
    );
    assert_eq!(
        locale.solve_snapshot_counts(3, 4, 1),
        "3 股流股，4 个步骤，1 条诊断"
    );
    assert_eq!(
        locale.snapshot_identity("snapshot-a", 7),
        "快照 snapshot-a，序号 7"
    );
    assert_eq!(locale.text(ShellText::ResultInspector), "结果检查器");
    assert_eq!(locale.text(ShellText::DiagnosticTargets), "诊断目标");
    assert_eq!(
        locale.text(ShellText::StaleStreamSelection),
        "已选流股不在最新快照中。"
    );
    assert_eq!(locale.text(ShellText::LastRunFailed), "最近一次运行失败");
    assert_eq!(locale.text(ShellText::RecoveryTarget), "修复目标");
    assert_eq!(locale.text(ShellText::SuggestedRecovery), "建议修复");
    assert_eq!(locale.text(ShellText::ActiveInspectorTarget), "检查器目标");
    assert_eq!(locale.text(ShellText::InspectorProperties), "属性");
    assert_eq!(locale.runtime_label("Number").as_ref(), "数值");
    assert_eq!(locale.runtime_label("Synced").as_ref(), "已同步");
    assert_eq!(locale.text(ShellText::InspectorPorts), "端口");
    assert_eq!(locale.runtime_label("Unit").as_ref(), "单元");
    assert_eq!(locale.runtime_label("Stream").as_ref(), "流股");
    assert_eq!(
        StudioShellLocale::En.runtime_label("Converged").as_ref(),
        "Converged"
    );
}

#[test]
fn result_inspector_state_tracks_selected_stream_per_snapshot() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    let snapshot = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .expect("expected solve snapshot");

    let default_selected = app
        .result_inspector
        .selected_stream_id_for_snapshot(&snapshot);
    assert_eq!(
        default_selected.as_deref(),
        snapshot
            .streams
            .first()
            .map(|stream| stream.stream_id.as_str())
    );

    app.result_inspector
        .select_stream(&snapshot.snapshot_id, "stream-heated");
    assert_eq!(
        app.result_inspector
            .selected_stream_id_for_snapshot(&snapshot)
            .as_deref(),
        Some("stream-heated")
    );

    let mut next_snapshot = snapshot.clone();
    next_snapshot.snapshot_id = "snapshot-next".to_string();
    assert_eq!(
        app.result_inspector
            .selected_stream_id_for_snapshot(&next_snapshot)
            .as_deref(),
        next_snapshot
            .streams
            .first()
            .map(|stream| stream.stream_id.as_str())
    );
}

#[test]
fn right_sidebar_width_keeps_runtime_panel_readable() {
    let width = region_panel_width_from_values(
        StudioGuiWindowDockRegion::RightSidebar,
        1_280.0,
        100.0,
        24.0,
    );

    assert_eq!(width, 360.0);
}

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
        Some("example-feed-heater-flash-rev-1-seq-1")
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
fn open_example_project_rebuilds_runtime_for_selected_sample() {
    let mut app = ready_app_state(&synced_workspace_config());
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();

    app.dispatch_ui_command("run_panel.run_manual");
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .snapshot_history_count,
        1
    );

    app.open_example_project(target_project);
    let window = app.platform_host.snapshot().window_model();

    assert_eq!(
        window.runtime.workspace_document.title,
        "Feed Valve Flash Example"
    );
    assert_eq!(window.runtime.workspace_document.snapshot_history_count, 0);
    assert_eq!(
        window
            .runtime
            .example_projects
            .iter()
            .find(|example| example.is_current)
            .map(|example| example.id),
        Some("feed-valve-flash")
    );
    assert_eq!(
        window.runtime.run_panel.view().primary_action.label,
        "Resume"
    );

    app.dispatch_ui_command("run_panel.run_manual");
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .control_state
            .run_status,
        rf_ui::RunStatus::Converged
    );
}

#[test]
fn open_project_from_input_rebuilds_runtime_and_records_feedback() {
    let mut app = ready_app_state(&synced_workspace_config());
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "water-ethanol-heater-flash")
        .expect("expected water ethanol example")
        .project_path
        .clone();

    app.project_open.path_input = target_project.display().to_string();
    app.open_project_from_input();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.title,
        "Feed Heater Flash Water Ethanol Example"
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Info)
    );
    assert!(
        window
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("opened project"))
    );
    assert_eq!(
        app.project_open.recent_projects.first(),
        Some(&target_project)
    );
}

#[test]
fn open_project_from_picker_rebuilds_runtime_and_records_recent_project() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("picker-open");
    let base_app =
        ReadyAppState::from_config(&config, preferences_path.clone()).expect("expected base app");
    let target_project = base_app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.open_project_from_picker();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.title,
        "Feed Valve Flash Example"
    );
    assert_eq!(
        app.project_open.path_input,
        target_project.display().to_string()
    );
    assert_eq!(
        app.project_open.recent_projects.first(),
        Some(&target_project)
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Info)
    );

    let _ = std::fs::remove_file(preferences_path);
}

#[test]
fn canceling_project_picker_keeps_current_workspace_active() {
    let config = synced_workspace_config();
    let original_title = ready_app_state(&config)
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .workspace_document
        .title;
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        test_preferences_path("picker-cancel"),
        Box::new(TestProjectFilePicker::new(None)),
    )
    .expect("expected app state");

    app.open_project_from_picker();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(window.runtime.workspace_document.title, original_title);
    assert!(app.project_open.recent_projects.is_empty());
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Project picker canceled")
    );
}

#[test]
fn save_project_as_from_picker_writes_project_and_records_recent_project() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("save-as-picker");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-save-as-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.save_project_as_from_picker();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.project_path.as_deref(),
        Some(target_project.display().to_string().as_str())
    );
    assert!(!window.runtime.workspace_document.has_unsaved_changes);
    assert_eq!(
        app.project_open.path_input,
        target_project.display().to_string()
    );
    assert_eq!(
        app.project_open.recent_projects.first(),
        Some(&target_project)
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Project saved as")
    );
    assert!(read_project_file(&target_project).is_ok());

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn successful_project_opens_keep_recent_projects_deduped_and_ordered() {
    let mut app = ready_app_state(&synced_workspace_config());
    let examples = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects;
    let valve_project = examples
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    let ethanol_project = examples
        .iter()
        .find(|example| example.id == "water-ethanol-heater-flash")
        .expect("expected water ethanol example")
        .project_path
        .clone();

    app.open_project(valve_project.clone(), "project");
    app.open_project(ethanol_project.clone(), "project");
    app.open_recent_project(valve_project.clone());

    assert_eq!(
        app.project_open.recent_projects,
        vec![valve_project, ethanol_project]
    );
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .title,
        "Feed Valve Flash Example"
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.detail.as_str())
            .unwrap_or(""),
        format!(
            "Opened recent project: {}",
            app.project_open
                .recent_projects
                .first()
                .expect("expected recent project")
                .display()
        )
    );
}

#[test]
fn successful_project_opens_persist_recent_projects_for_next_shell_start() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("recent-projects");
    let mut app =
        ReadyAppState::from_config(&config, preferences_path.clone()).expect("expected app state");
    let examples = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects;
    let valve_project = examples
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    let ethanol_project = examples
        .iter()
        .find(|example| example.id == "water-ethanol-heater-flash")
        .expect("expected water ethanol example")
        .project_path
        .clone();

    app.open_project(valve_project.clone(), "project");
    app.open_project(ethanol_project.clone(), "project");

    let restarted =
        ReadyAppState::from_config(&config, preferences_path.clone()).expect("expected restart");

    assert_eq!(
        restarted.project_open.recent_projects,
        vec![ethanol_project, valve_project]
    );

    let _ = std::fs::remove_file(preferences_path);
}

#[test]
fn open_project_failure_keeps_current_runtime_and_surfaces_error_notice() {
    let mut app = ready_app_state(&synced_workspace_config());
    let original_window = app.platform_host.snapshot().window_model();
    let missing_project = std::env::temp_dir().join("radishflow-missing-project.rfproj.json");

    app.project_open.path_input = missing_project.display().to_string();
    app.open_project_from_input();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.title,
        original_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Error)
    );
    assert!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice
                .detail
                .contains("radishflow-missing-project.rfproj.json"))
            .unwrap_or(false)
    );
    assert!(
        window
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("open project failed"))
    );
    assert!(
        app.project_open.recent_projects.is_empty(),
        "failed project opens should not enter recent projects"
    );
}

#[test]
fn open_project_from_input_requires_confirmation_when_workspace_has_unsaved_changes() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);
    assert_eq!(
        dirty_window.runtime.workspace_document.last_saved_revision,
        Some(0)
    );
    assert_eq!(dirty_window.runtime.workspace_document.revision, 1);

    app.project_open.path_input = target_project.display().to_string();
    app.open_project_from_input();

    let blocked_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blocked_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Warning)
    );
    assert!(app.project_open.pending_confirmation.is_some());

    app.confirm_pending_project_open();
    let opened_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        opened_window.runtime.workspace_document.title,
        "Feed Valve Flash Example"
    );
    assert!(!opened_window.runtime.workspace_document.has_unsaved_changes);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn open_project_from_picker_requires_confirmation_when_workspace_has_unsaved_changes() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let target_project = ready_app_state(&synced_workspace_config())
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        test_preferences_path("picker-unsaved"),
        Box::new(TestProjectFilePicker::new(Some(target_project))),
    )
    .expect("expected app state");

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);

    app.open_project_from_picker();

    let blocked_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blocked_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open
            .pending_confirmation
            .as_ref()
            .map(|request| request.source_label.as_str()),
        Some("project picker")
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Warning)
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn open_recent_project_requires_confirmation_when_workspace_has_unsaved_changes() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    app.project_open
        .record_recent_project(target_project.clone());

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);

    app.open_recent_project(target_project);

    let blocked_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blocked_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open
            .pending_confirmation
            .as_ref()
            .map(|request| request.source_label.as_str()),
        Some("recent project")
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Warning)
    );

    app.confirm_pending_project_open();
    let opened_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        opened_window.runtime.workspace_document.title,
        "Feed Valve Flash Example"
    );
    assert!(!opened_window.runtime.workspace_document.has_unsaved_changes);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn cancel_pending_project_open_keeps_dirty_workspace_active() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    app.project_open.path_input = target_project.display().to_string();
    app.open_project_from_input();

    app.cancel_pending_project_open();

    let canceled_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        canceled_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert!(
        canceled_window
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    assert!(app.project_open.pending_confirmation.is_none());
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Project open canceled")
    );

    let _ = std::fs::remove_file(project_path);
}

fn palette_commands_for_test(commands: &[(&str, bool)]) -> Vec<&'static StudioGuiCommandEntry> {
    commands
        .iter()
        .map(|(command_id, enabled)| {
            let entry = Box::leak(Box::new(StudioGuiCommandEntry {
                command_id: (*command_id).to_string(),
                label: (*command_id).to_string(),
                detail: "test".to_string(),
                enabled: *enabled,
                sort_order: 100,
                target_window_id: None,
                menu_path: vec!["Commands".to_string()],
                search_terms: Vec::new(),
                shortcut: None,
            }));
            &*entry
        })
        .collect()
}

struct CommandSurfaceApps {
    menu_app: ReadyAppState,
    toolbar_app: ReadyAppState,
    list_app: ReadyAppState,
    palette_app: ReadyAppState,
    shortcut_app: ReadyAppState,
}

fn ready_command_surface_apps(config: &StudioRuntimeConfig) -> CommandSurfaceApps {
    CommandSurfaceApps {
        menu_app: ready_app_state(config),
        toolbar_app: ready_app_state(config),
        list_app: ready_app_state(config),
        palette_app: ready_app_state(config),
        shortcut_app: ready_app_state(config),
    }
}

fn ready_failed_command_surface_apps() -> CommandSurfaceApps {
    CommandSurfaceApps {
        menu_app: ready_failed_app_state(),
        toolbar_app: ready_failed_app_state(),
        list_app: ready_failed_app_state(),
        palette_app: ready_failed_app_state(),
        shortcut_app: ready_failed_app_state(),
    }
}

fn command_surface_window(app: &ReadyAppState) -> radishflow_studio::StudioGuiWindowModel {
    let mut window = app.platform_host.snapshot().window_model();
    stabilize_command_surface_window(&mut window);
    window
}

fn stabilize_command_surface_window(window: &mut radishflow_studio::StudioGuiWindowModel) {
    window.runtime.entitlement_host = None;
    window.runtime.platform_timer_lines.clear();
    window.runtime.gui_activity_lines.clear();
}

fn shared_command_surface_initial_window(
    apps: &CommandSurfaceApps,
) -> radishflow_studio::StudioGuiWindowModel {
    let initial_window = command_surface_window(&apps.menu_app);
    assert_eq!(command_surface_window(&apps.toolbar_app), initial_window);
    assert_eq!(command_surface_window(&apps.list_app), initial_window);
    assert_eq!(command_surface_window(&apps.palette_app), initial_window);
    assert_eq!(command_surface_window(&apps.shortcut_app), initial_window);
    initial_window
}

fn assert_command_surface_windows_equal(
    apps: &CommandSurfaceApps,
    expected: &radishflow_studio::StudioGuiWindowModel,
) {
    assert_eq!(command_surface_window(&apps.menu_app), *expected);
    assert_eq!(command_surface_window(&apps.toolbar_app), *expected);
    assert_eq!(command_surface_window(&apps.list_app), *expected);
    assert_eq!(command_surface_window(&apps.palette_app), *expected);
    assert_eq!(command_surface_window(&apps.shortcut_app), *expected);
}

fn dispatch_enabled_command_surface_interactions(
    apps: &mut CommandSurfaceApps,
    window: &radishflow_studio::StudioGuiWindowModel,
    command_id: &str,
    palette_query: &str,
    shortcut_key: egui::Key,
    shortcut_modifiers: egui::Modifiers,
) {
    let menu_command = find_menu_command(&window.commands.menu_tree, command_id)
        .cloned()
        .expect("expected menu command");
    let toolbar_command_id = find_toolbar_command_id(&window.commands.toolbar_sections, command_id)
        .expect("expected toolbar command");
    let list_command_id =
        find_command_list_command_id(&window.commands.command_list_sections, command_id)
            .expect("expected command list command");

    apps.menu_app.dispatch_menu_command(&menu_command);
    apps.toolbar_app.dispatch_ui_command(toolbar_command_id);
    apps.list_app.dispatch_ui_command(list_command_id);

    apps.palette_app.command_palette.open();
    apps.palette_app.command_palette.query = palette_query.to_string();
    let commands = apps
        .palette_app
        .platform_host
        .snapshot()
        .window_model()
        .commands;
    assert_eq!(
        selected_palette_item_command_id(&commands.palette_items(palette_query), 0),
        Some(command_id.to_string())
    );
    run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
        assert!(
            apps.palette_app
                .handle_command_palette_keyboard(ctx, &commands)
        );
    });
    dispatch_shortcut_for_test(&mut apps.shortcut_app, shortcut_key, shortcut_modifiers);
}

fn dispatch_disabled_command_surface_interactions(
    apps: &mut CommandSurfaceApps,
    window: &radishflow_studio::StudioGuiWindowModel,
    command_id: &str,
    palette_query: &str,
    shortcut_key: egui::Key,
    shortcut_modifiers: egui::Modifiers,
) {
    let menu_command = find_menu_command(&window.commands.menu_tree, command_id)
        .cloned()
        .expect("expected menu command");
    assert!(
        !menu_command.enabled,
        "expected menu command to stay disabled before dispatch"
    );

    let toolbar_command = find_toolbar_command(&window.commands.toolbar_sections, command_id)
        .cloned()
        .expect("expected toolbar command");
    assert!(
        !toolbar_command.enabled,
        "expected toolbar command to stay disabled before dispatch"
    );

    let list_command =
        find_command_list_command(&window.commands.command_list_sections, command_id)
            .cloned()
            .expect("expected command list command");
    assert!(
        !list_command.enabled,
        "expected command list command to stay disabled before dispatch"
    );

    apps.menu_app.dispatch_menu_command(&menu_command);
    apps.toolbar_app
        .dispatch_ui_command(&toolbar_command.command_id);
    apps.list_app.dispatch_ui_command(&list_command.command_id);

    apps.palette_app.command_palette.open();
    apps.palette_app.command_palette.query = palette_query.to_string();
    let commands = apps
        .palette_app
        .platform_host
        .snapshot()
        .window_model()
        .commands;
    let palette_items = commands.palette_items(palette_query);
    assert_eq!(palette_items.len(), 1);
    assert_eq!(palette_items[0].command_id, command_id.to_string());
    assert!(
        !palette_items[0].enabled,
        "expected palette item to stay disabled before dispatch"
    );
    assert_eq!(selected_palette_item_command_id(&palette_items, 0), None);
    run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
        assert!(
            apps.palette_app
                .handle_command_palette_keyboard(ctx, &commands)
        );
    });
    dispatch_shortcut_for_test(&mut apps.shortcut_app, shortcut_key, shortcut_modifiers);
}

fn find_menu_command<'a>(
    nodes: &'a [StudioGuiCommandMenuNode],
    command_id: &str,
) -> Option<&'a StudioGuiCommandMenuCommandModel> {
    for node in nodes {
        if let Some(command) = node.command.as_ref() {
            if command.command_id == command_id {
                return Some(command);
            }
        }
        if let Some(command) = find_menu_command(&node.children, command_id) {
            return Some(command);
        }
    }
    None
}

fn find_toolbar_command<'a>(
    sections: &'a [StudioGuiWindowToolbarSectionModel],
    command_id: &str,
) -> Option<&'a radishflow_studio::StudioGuiWindowToolbarItemModel> {
    sections
        .iter()
        .flat_map(|section| section.items.iter())
        .find(|command| command.command_id == command_id)
}

fn find_toolbar_command_id<'a>(
    sections: &'a [StudioGuiWindowToolbarSectionModel],
    command_id: &str,
) -> Option<&'a str> {
    find_toolbar_command(sections, command_id).map(|command| command.command_id.as_str())
}

fn find_command_list_command<'a>(
    sections: &'a [radishflow_studio::StudioGuiWindowCommandListSectionModel],
    command_id: &str,
) -> Option<&'a radishflow_studio::StudioGuiWindowCommandListItemModel> {
    sections
        .iter()
        .flat_map(|section| section.items.iter())
        .find(|command| command.command_id == command_id)
}

fn find_command_list_command_id<'a>(
    sections: &'a [radishflow_studio::StudioGuiWindowCommandListSectionModel],
    command_id: &str,
) -> Option<&'a str> {
    find_command_list_command(sections, command_id).map(|command| command.command_id.as_str())
}

fn ready_failed_app_state() -> ReadyAppState {
    let mut app = ready_app_state(&unbound_outlet_failure_synced_config());
    app.dispatch_ui_command("run_panel.run_manual");

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.control_state.run_status,
        rf_ui::RunStatus::Error
    );
    assert!(
        find_menu_command(&window.commands.menu_tree, "run_panel.recover_failure")
            .map(|command| command.enabled)
            .unwrap_or(false),
        "expected recovery command to be enabled after failed run"
    );

    app
}

fn ready_app_state(config: &StudioRuntimeConfig) -> ReadyAppState {
    ReadyAppState::from_config(config, test_preferences_path("default"))
        .expect("expected app state")
}

struct TestProjectFilePicker {
    selected_project: Option<PathBuf>,
}

impl TestProjectFilePicker {
    fn new(selected_project: Option<PathBuf>) -> Self {
        Self { selected_project }
    }
}

impl ProjectFilePicker for TestProjectFilePicker {
    fn pick_project_file(&mut self) -> Option<PathBuf> {
        self.selected_project.take()
    }

    fn pick_save_project_file(&mut self) -> Option<PathBuf> {
        self.selected_project.take()
    }
}

fn dispatch_shortcut_for_test(app: &mut ReadyAppState, key: egui::Key, modifiers: egui::Modifiers) {
    run_with_key_press(key, modifiers, |ctx| app.dispatch_shortcuts(ctx));
}

fn run_with_key_press<R>(
    key: egui::Key,
    modifiers: egui::Modifiers,
    run: impl FnOnce(&egui::Context) -> R,
) -> R {
    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 720.0),
        )),
        focused: true,
        modifiers,
        events: vec![egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers,
        }],
        ..Default::default()
    });
    let output = run(&ctx);
    let _ = ctx.end_pass();
    output
}

fn run_with_key_press_and_focus<R>(
    key: egui::Key,
    modifiers: egui::Modifiers,
    focused_id: egui::Id,
    run: impl FnOnce(&egui::Context) -> R,
) -> R {
    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 720.0),
        )),
        focused: true,
        modifiers,
        events: vec![egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers,
        }],
        ..Default::default()
    });
    ctx.memory_mut(|mem| mem.request_focus(focused_id));
    let output = run(&ctx);
    let _ = ctx.end_pass();
    output
}
