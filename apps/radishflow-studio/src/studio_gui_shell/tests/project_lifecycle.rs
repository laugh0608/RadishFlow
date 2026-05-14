use super::*;

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
        "Feed Valve Flash Binary Hydrocarbon Example"
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
        "Feed Valve Flash Binary Hydrocarbon Example"
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
fn create_blank_project_opens_untitled_blank_workspace_without_picker() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("blank-untitled");
    let unused_picker_target = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-unused-blank-picker-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(
            unused_picker_target.clone(),
        ))),
    )
    .expect("expected app state");

    app.create_blank_project();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(window.runtime.workspace_document.title, "Blank Project");
    assert_eq!(window.runtime.workspace_document.project_path, None);
    assert!(
        window.runtime.workspace_document.has_unsaved_changes,
        "bootstrap should mark the default blank thermo basis as an explicit unsaved project edit"
    );
    assert_eq!(window.runtime.workspace_document.unit_count, 0);
    assert_eq!(window.runtime.workspace_document.stream_count, 0);
    assert_eq!(app.project_open.path_input, "");
    assert!(
        app.project_open.recent_projects.is_empty(),
        "untitled blank projects should not enter recent projects before Save As"
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Blank project created")
    );
    assert!(
        !unused_picker_target.exists(),
        "creating an untitled blank project should not open the save picker or create its target"
    );

    let _ = std::fs::remove_file(preferences_path);
}

#[test]
fn saving_untitled_blank_project_uses_save_as_picker() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("blank-save-as");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-untitled-save-as-{}.rfproj.json",
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

    app.create_blank_project();
    app.save_project();

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
fn saving_untitled_blank_project_allows_creating_another_blank_project() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("blank-save-then-new");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-untitled-save-then-new-{}.rfproj.json",
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

    app.create_blank_project();
    app.save_project();
    assert!(
        !app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );

    app.create_blank_project();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(window.runtime.workspace_document.title, "Blank Project");
    assert_eq!(window.runtime.workspace_document.project_path, None);
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Blank project created")
    );

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn saving_untitled_blank_project_allows_opening_example_project() {
    let config = synced_workspace_config();
    let example_project = config.project_path.clone();
    let preferences_path = test_preferences_path("blank-save-then-example");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-untitled-save-then-example-{}.rfproj.json",
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

    app.create_blank_project();
    app.save_project();
    assert!(
        !app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );

    app.open_example_project(example_project.clone());

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.project_path.as_deref(),
        Some(example_project.display().to_string().as_str())
    );
    assert!(!window.runtime.workspace_document.has_unsaved_changes);
    assert!(
        app.project_open.pending_confirmation.is_none(),
        "clean saved workspace should not require discard confirmation before opening another project"
    );

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn create_blank_project_requires_clean_workspace() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        test_preferences_path("blank-unsaved"),
        Box::new(TestProjectFilePicker::new(None)),
    )
    .expect("expected app state");
    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);

    app.create_blank_project();

    let blocked_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blocked_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Warning)
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Blank project blocked")
    );

    let _ = std::fs::remove_file(project_path);
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
fn save_project_as_from_picker_requires_confirmation_before_overwrite() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("save-as-overwrite");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-save-as-overwrite-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    fs::write(&target_project, "existing project placeholder").expect("expected target seed");
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.save_project_as_from_picker();

    assert_eq!(
        fs::read_to_string(&target_project).expect("expected target read"),
        "existing project placeholder"
    );
    assert_eq!(
        app.project_open.pending_save_as_overwrite.as_deref(),
        Some(target_project.as_path())
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Confirm overwrite")
    );

    app.confirm_pending_save_as_overwrite();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.project_path.as_deref(),
        Some(target_project.display().to_string().as_str())
    );
    assert!(app.project_open.pending_save_as_overwrite.is_none());
    assert!(read_project_file(&target_project).is_ok());

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn failed_confirmed_save_as_keeps_workspace_state_and_retry_target() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("save-as-overwrite-failure");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-save-as-overwrite-failure-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    fs::create_dir_all(&target_project).expect("expected directory target");
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");
    let original_window = app.platform_host.snapshot().window_model();
    let original_document = original_window.runtime.workspace_document.clone();

    app.save_project_as_from_picker();
    assert_eq!(
        app.project_open.pending_save_as_overwrite.as_deref(),
        Some(target_project.as_path())
    );

    app.confirm_pending_save_as_overwrite();

    let failed_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        failed_window.runtime.workspace_document.project_path,
        original_document.project_path
    );
    assert_eq!(
        failed_window.runtime.workspace_document.last_saved_revision,
        original_document.last_saved_revision
    );
    assert_eq!(
        failed_window.runtime.workspace_document.has_unsaved_changes,
        original_document.has_unsaved_changes
    );
    assert!(app.project_open.recent_projects.is_empty());
    assert_eq!(
        app.project_open.pending_save_as_overwrite.as_deref(),
        Some(target_project.as_path()),
        "failed overwrite save-as should keep the retry/cancel target visible"
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Save As failed")
    );
    assert!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.detail.contains("Current workspace remains open"))
            .unwrap_or(false)
    );
    assert!(
        failed_window
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("save as failed"))
    );
    assert!(target_project.is_dir());

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_dir_all(target_project);
}

#[test]
fn cancel_pending_save_as_overwrite_keeps_existing_file() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("save-as-overwrite-cancel");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-save-as-overwrite-cancel-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    fs::write(&target_project, "existing project placeholder").expect("expected target seed");
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.save_project_as_from_picker();
    app.cancel_pending_save_as_overwrite();

    assert_eq!(
        fs::read_to_string(&target_project).expect("expected target read"),
        "existing project placeholder"
    );
    assert!(app.project_open.pending_save_as_overwrite.is_none());
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Save As canceled")
    );

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
        "Feed Valve Flash Binary Hydrocarbon Example"
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
        "Feed Valve Flash Binary Hydrocarbon Example"
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
        "Feed Valve Flash Binary Hydrocarbon Example"
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
