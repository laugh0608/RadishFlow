use super::*;

#[test]
fn overwrite_confirmation_preserves_externally_changed_or_removed_target() {
    let source = equipment_storage::v3_file("overwrite-review", false);
    let target = source.with_file_name("copy.rfproj.json");
    let original = fs::read(&source).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: source.clone(),
        ..synced_workspace_config()
    });
    let before = app.project_departure_checkpoint();
    for changed in [Some([original.as_slice(), b"\n"].concat()), None] {
        fs::copy(&source, &target).unwrap();
        app.request_save_project_as(target.clone(), true);
        assert!(app.project_open.pending_save_as_overwrite.is_some());
        match &changed {
            Some(bytes) => fs::write(&target, bytes).unwrap(),
            None => fs::remove_file(&target).unwrap(),
        }
        app.confirm_pending_save_as_overwrite();
        match changed {
            Some(bytes) => assert_eq!(fs::read(&target).unwrap(), bytes),
            None => assert!(!target.try_exists().unwrap()),
        }
        assert_eq!(app.project_departure_checkpoint(), before);
        assert!(app.project_open.recent_projects.is_empty());
        assert!(app.pending_format_upgrade.is_none());
        assert!(app.project_open.pending_save_as_overwrite.is_none());
        assert!(!studio_layout_path_for_project(&target).exists());
    }
    assert_eq!(fs::read(&source).unwrap(), original);
    fs::remove_dir_all(source.parent().unwrap()).unwrap();
}

#[test]
fn save_as_retry_reviews_a_newly_created_target_before_overwriting() {
    let source = equipment_storage::v3_file("save-as-retry-review", false);
    let target = source.with_file_name("copy.rfproj.json");
    let original = fs::read(&source).unwrap();
    let other = [original.as_slice(), b"\n"].concat();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: source.clone(),
        ..synced_workspace_config()
    });
    let before = app.project_departure_checkpoint();
    fs::create_dir(&target).unwrap();
    app.request_save_project_as(target.clone(), true);
    assert_eq!(
        app.project_open
            .pending_save_as_overwrite
            .as_ref()
            .unwrap()
            .button_label(),
        ShellText::RetrySaveAs
    );
    fs::remove_dir(&target).unwrap();
    fs::write(&target, &other).unwrap();
    app.confirm_pending_save_as_overwrite();
    assert_eq!(
        app.project_open
            .pending_save_as_overwrite
            .as_ref()
            .unwrap()
            .button_label(),
        ShellText::ConfirmSaveAsOverwrite
    );
    app.cancel_pending_save_as_overwrite();
    app.confirm_pending_save_as_overwrite();
    assert_eq!(fs::read(&target).unwrap(), other);
    assert_eq!(app.project_departure_checkpoint(), before);
    assert!(app.project_open.recent_projects.is_empty());
    assert!(!studio_layout_path_for_project(&target).exists());

    app.request_save_project_as(target.clone(), true);
    app.confirm_pending_save_as_overwrite();
    assert_eq!(app.project_open.recent_projects.first(), Some(&target));
    assert_eq!(fs::read(&target).unwrap(), original);
    assert_eq!(fs::read(&source).unwrap(), original);
    fs::remove_dir_all(source.parent().unwrap()).unwrap();
}

#[test]
fn save_as_overwrite_rejects_workspace_edits_and_rechecks_after_upgrade_prompt() {
    let source = equipment_storage::v3_file("save-as-workspace-review", false);
    let target = source.with_file_name("copy.rfproj.json");
    let original = fs::read(&source).unwrap();
    fs::write(&target, &original).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: source.clone(),
        ..synced_workspace_config()
    });
    app.request_save_project_as(target.clone(), true);
    assert!(app.project_open.pending_save_as_overwrite.is_some());
    app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(
        rf_types::units::DisplayUnitSet::engineering(),
    ))
    .unwrap();
    assert!(
        app.platform_host
            .project_save_compatibility(&target)
            .unwrap()
            .is_none()
    );
    let edited = app.project_departure_checkpoint();
    app.confirm_pending_save_as_overwrite();
    assert!(app.pending_format_upgrade.is_none());
    assert_eq!(app.project_departure_checkpoint(), edited);
    assert_eq!(fs::read(&target).unwrap(), original);

    app.equipment_command(rf_ui::engineering::editor::EquipmentEditorCommand::CreateRatingTask);
    let with_rating = app.project_departure_checkpoint();
    app.request_save_project_as(target.clone(), true);
    app.confirm_pending_save_as_overwrite();
    assert!(app.pending_format_upgrade.is_some());
    let changed = [original.as_slice(), b"\n"].concat();
    fs::write(&target, &changed).unwrap();
    assert!(!app.confirm_format_upgrade());
    assert_eq!(app.project_departure_checkpoint(), with_rating);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .is_dirty()
    );
    assert!(app.project_open.recent_projects.is_empty());
    assert!(!studio_layout_path_for_project(&target).exists());
    assert_eq!(fs::read(&target).unwrap(), changed);
    assert_eq!(fs::read(&source).unwrap(), original);
    fs::remove_dir_all(source.parent().unwrap()).unwrap();
}
