    use super::*;
    use radishflow_studio::{
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed, StudioRuntimeTrigger,
    };
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
        assert_eq!(menu_window.runtime.control_state.run_status, rf_ui::RunStatus::Converged);
        assert_eq!(menu_window.runtime.control_state.pending_reason, None);
        assert_eq!(
            menu_window.runtime.control_state.latest_snapshot_id.as_deref(),
            Some("example-unbound-outlet-port-rev-1-seq-1")
        );
        assert_eq!(menu_window.runtime.run_panel.view().status_label, "Converged");
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
        assert_eq!(menu_window.runtime.control_state.run_status, rf_ui::RunStatus::Converged);
        assert_eq!(menu_window.runtime.control_state.pending_reason, None);
        assert_eq!(
            menu_window.runtime.control_state.latest_snapshot_id.as_deref(),
            Some("example-feed-heater-flash-rev-1-seq-1")
        );
        assert_eq!(menu_window.runtime.run_panel.view().status_label, "Converged");
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
        assert_eq!(menu_window.runtime.control_state.run_status, rf_ui::RunStatus::Dirty);
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
        let commands = apps.palette_app.platform_host.snapshot().window_model().commands;
        assert_eq!(
            selected_palette_item_command_id(&commands.palette_items(palette_query), 0),
            Some(command_id.to_string())
        );
        run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
            assert!(apps.palette_app.handle_command_palette_keyboard(ctx, &commands));
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
        apps.toolbar_app.dispatch_ui_command(&toolbar_command.command_id);
        apps.list_app.dispatch_ui_command(&list_command.command_id);

        apps.palette_app.command_palette.open();
        apps.palette_app.command_palette.query = palette_query.to_string();
        let commands = apps.palette_app.platform_host.snapshot().window_model().commands;
        let palette_items = commands.palette_items(palette_query);
        assert_eq!(palette_items.len(), 1);
        assert_eq!(palette_items[0].command_id, command_id.to_string());
        assert!(
            !palette_items[0].enabled,
            "expected palette item to stay disabled before dispatch"
        );
        assert_eq!(selected_palette_item_command_id(&palette_items, 0), None);
        run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
            assert!(apps.palette_app.handle_command_palette_keyboard(ctx, &commands));
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
        assert_eq!(window.runtime.control_state.run_status, rf_ui::RunStatus::Error);
        assert!(
            find_menu_command(&window.commands.menu_tree, "run_panel.recover_failure")
                .map(|command| command.enabled)
                .unwrap_or(false),
            "expected recovery command to be enabled after failed run"
        );

        app
    }

    fn ready_app_state(config: &StudioRuntimeConfig) -> ReadyAppState {
        let mut app = ReadyAppState {
            platform_host: StudioGuiPlatformHost::new(config).expect("expected platform host"),
            platform_timer_executor: EguiPlatformTimerExecutor::default(),
            last_error: None,
            command_palette: CommandPaletteState::default(),
            last_area_focus: None,
            drag_session: None,
            active_drop_preview: None,
            drop_preview_overlay_anchor: None,
            last_viewport_focused: None,
        };
        app.dispatch_event(StudioGuiEvent::OpenWindowRequested);
        app
    }

    fn dispatch_shortcut_for_test(
        app: &mut ReadyAppState,
        key: egui::Key,
        modifiers: egui::Modifiers,
    ) {
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

