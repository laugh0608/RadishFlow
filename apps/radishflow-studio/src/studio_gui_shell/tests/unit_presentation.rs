use super::*;

#[test]
fn invalid_v2_presentation_does_not_replace_the_open_workspace_or_results() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    let before = app.platform_host.snapshot().window_model();
    let recent = app.project_open.recent_projects.clone();
    let path = test_preferences_path("invalid-project-presentation").with_extension("rfproj.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let invalid =
        include_str!("../../../../../crates/rf-store/tests/fixtures/project-v2-engineering.json")
            .replace("\"celsius\"", "\"future-unit\"");
    std::fs::write(&path, &invalid).unwrap();
    app.project_open.path_input = path.display().to_string();
    app.open_project_from_input();
    let after = app.platform_host.snapshot().window_model();
    assert_eq!(
        after.runtime.workspace_document,
        before.runtime.workspace_document
    );
    assert_eq!(
        after.runtime.latest_solve_snapshot,
        before.runtime.latest_solve_snapshot
    );
    assert_eq!(app.project_open.recent_projects, recent);
    let notice = app.project_open.notice.as_ref().unwrap();
    assert_eq!(notice.level, ProjectOpenNoticeLevel::Error);
    assert!(notice.detail.contains("unknown unit ID"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), invalid);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn presentation_only_edits_protect_close_and_undo_survives_save() {
    use rf_types::units::DisplayUnitSet;
    use rf_ui::ProjectPresentationCommand;
    let (config, original) = blank_workspace_config();
    let mut app = ready_app_state(&config);
    let before = app.platform_host.document().clone();
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    let save = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .save_state;
    assert!(!save.document_dirty);
    assert!(save.presentation_dirty);
    assert!(!app.close_current_window_for_viewport_request());
    app.cancel_pending_close_window();
    app.save_project();
    assert!(
        !app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .save_state
            .has_unsaved_changes()
    );
    assert_eq!(app.platform_host.document(), &before);
    app.apply_presentation_command(ProjectPresentationCommand::Undo)
        .unwrap();
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .save_state
            .presentation_dirty
    );
    app.apply_presentation_command(ProjectPresentationCommand::Redo)
        .unwrap();
    assert!(
        !app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .save_state
            .presentation_dirty
    );
    assert_eq!(
        read_project_file(&original)
            .unwrap()
            .presentation
            .display_units,
        DisplayUnitSet::engineering()
    );
    fs::remove_file(original).unwrap();
}

#[test]
fn legacy_upgrade_cancel_and_failure_preserve_source_and_configuration() {
    use rf_types::units::DisplayUnitSet;
    use rf_ui::ProjectPresentationCommand;
    let path = test_preferences_path("legacy-upgrade").with_extension("rfproj.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let bytes = include_str!(
        "../../../../../examples/flowsheets/feed-heater-flash-synthetic-demo.rfproj.json"
    );
    fs::write(&path, bytes).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    app.save_project();
    assert!(app.pending_format_upgrade.is_some());
    assert_eq!(fs::read_to_string(&path).unwrap(), bytes);
    let before = app.platform_host.snapshot().runtime.workspace_document;
    app.cancel_pending_save_as_overwrite();
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    let bad = path.parent().unwrap().to_path_buf();
    app.request_save_project_as(bad, false);
    app.confirm_format_upgrade();
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    app.save_project();
    app.confirm_format_upgrade();
    assert_eq!(read_project_file(&path).unwrap().schema_version, 2);
    assert!(
        !app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn personal_defaults_are_isolated_and_corruption_requires_explicit_recovery() {
    use rf_types::units::DisplayUnitSet;
    use rf_ui::ProjectPresentationCommand;
    let preferences = test_preferences_path("personal-units");
    let defaults = preferences.with_file_name(rf_store::UNIT_DEFAULTS_FILE_NAME);
    let mut app =
        ReadyAppState::from_config(&synced_workspace_config(), preferences.clone()).unwrap();
    assert!(!defaults.exists());
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    app.save_unit_default(false);
    assert_eq!(
        rf_store::read_unit_defaults(&defaults).unwrap(),
        Some(DisplayUnitSet::engineering())
    );
    app.confirm_pending_blank_project(); // No pending action is a no-op.
    app.create_blank_project();
    app.confirm_pending_blank_project();
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .presentation
            .display_units(),
        &DisplayUnitSet::engineering()
    );
    assert!(
        !app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .save_state
            .presentation_dirty
    );
    fs::write(&defaults, b"broken defaults").unwrap();
    app.save_unit_default(false);
    assert_eq!(fs::read(&defaults).unwrap(), b"broken defaults");
    assert_eq!(
        app.unit_settings.default_units.as_ref().unwrap(),
        &DisplayUnitSet::engineering()
    );
    app.record_and_persist_recent_project(PathBuf::from("example.rfproj.json"));
    assert_eq!(fs::read(&defaults).unwrap(), b"broken defaults");
    let mut restarted =
        ReadyAppState::from_config(&synced_workspace_config(), preferences.clone()).unwrap();
    assert!(restarted.unit_settings.default_units.is_err());
    let before = restarted.platform_host.document().clone();
    restarted.create_blank_project();
    assert!(restarted.unit_settings.pending_new.is_some());
    assert_eq!(restarted.platform_host.document(), &before);
    assert_eq!(fs::read(&defaults).unwrap(), b"broken defaults");
    restarted.unit_settings.pending_new = None;
    restarted.request_unit_default_recovery();
    assert_eq!(fs::read(&defaults).unwrap(), b"broken defaults");
    restarted.save_unit_default(true);
    assert_eq!(
        restarted.unit_settings.default_units.as_ref().unwrap(),
        &DisplayUnitSet::si()
    );
    assert_eq!(
        rf_store::read_unit_defaults(&defaults).unwrap(),
        Some(DisplayUnitSet::si())
    );
    let backups: Vec<_> = fs::read_dir(preferences.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "bak"))
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(fs::read(backups[0].path()).unwrap(), b"broken defaults");
    fs::remove_dir_all(preferences.parent().unwrap()).unwrap();
}

#[test]
fn saving_presentation_retains_unsubmitted_input_and_current_solve_snapshot() {
    use rf_types::units::DisplayUnitSet;
    use rf_ui::ProjectPresentationCommand;
    let path = test_preferences_path("unit-save-drafts").with_extension("rfproj.json");
    let mut project = feed_heater_flash_binary_hydrocarbon_project();
    project.schema_version = 2;
    write_project_file(&path, &project).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_unit:heater-1");
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id("unit:heater-1:outlet_temperature_k"),
        "unfinished",
    );
    let before = app.platform_host.snapshot();
    assert!(before.runtime.latest_solve_snapshot.is_some());
    assert_eq!(
        before
            .runtime
            .workspace_document
            .save_state
            .pending_input_count,
        1
    );
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    app.save_project();
    let after = app.platform_host.snapshot();
    assert!(!after.runtime.workspace_document.has_unsaved_changes);
    assert_eq!(
        after.runtime.latest_solve_snapshot,
        before.runtime.latest_solve_snapshot
    );
    assert_eq!(
        after.runtime.workspace_document.input_edits,
        before.runtime.workspace_document.input_edits
    );
    assert_eq!(
        after.runtime.workspace_document.revision,
        before.runtime.workspace_document.revision
    );
    assert!(!app.close_current_window_for_viewport_request());
    assert!(!app.save_pending_close_window());
    assert!(app.project_open.pending_close_window_confirmation.is_some());
    assert_eq!(read_project_file(&path).unwrap().document, project.document);
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
