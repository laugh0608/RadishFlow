use super::*;
use radishflow_studio::{
    StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed, StudioRuntimeTrigger,
};
use rf_store::{
    StoredDocumentMetadata, StoredProjectFile, read_project_file, read_studio_layout_file,
    studio_layout_path_for_project, write_project_file,
};
use std::{fs, path::PathBuf, time::UNIX_EPOCH};

const OFFICIAL_HEATER_BINARY_HYDROCARBON_AUTORUN_SNAPSHOT_ID: &str =
    "example-feed-heater-flash-binary-hydrocarbon-rev-1-seq-1";

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
    let project = include_str!(
        "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    )
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

fn blank_workspace_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-blank-project-{timestamp}.rfproj.json"
    ));
    let project = StoredProjectFile::new(
        rf_model::Flowsheet::new("Blank Project"),
        StoredDocumentMetadata::new("blank-doc", "Blank Project", UNIX_EPOCH),
    );
    write_project_file(&project_path, &project).expect("expected blank project write");

    (
        StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..synced_workspace_config()
        },
        project_path,
    )
}

fn flash_drum_local_rules_synced_config() -> (StudioRuntimeConfig, PathBuf) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-local-rules-synced-{timestamp}.rfproj.json"
    ));
    let project = include_str!(
        "../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    )
    .replacen(
        ",\n        \"stream-vapor\": {\n          \"id\": \"stream-vapor\",\n          \"name\": \"Vapor Outlet\",\n          \"temperature_k\": 345.0,\n          \"pressure_pa\": 95000.0,\n          \"total_molar_flow_mol_s\": 0.0,\n          \"overall_mole_fractions\": {\n            \"methane\": 0.5,\n            \"ethane\": 0.5\n          },\n          \"phases\": []\n        }",
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
        untitled_blank_project: None,
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

mod basic;
mod canvas;
mod command_palette;
mod command_surface;
mod project_lifecycle;
mod runtime;
mod runtime_synthetic_flash_inlet_boundary;

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

fn accept_canvas_suggestion_by_id(app: &mut ReadyAppState, suggestion_id: &str) {
    app.dispatch_event(StudioGuiEvent::CanvasSuggestionAcceptByIdRequested {
        suggestion_id: rf_ui::CanvasSuggestionId::new(suggestion_id),
    });
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
