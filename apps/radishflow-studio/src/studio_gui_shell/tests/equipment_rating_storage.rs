use super::*;
use rf_ui::engineering::editor::EquipmentEditorCommand;

fn pending_rating(name: &str) -> (ReadyAppState, PathBuf, Vec<u8>) {
    let path = super::equipment_storage::v3_file(name, false);
    let original = fs::read(&path).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    // Exercise the application command; the new UI mode is intentionally not exposed yet.
    app.equipment_command(EquipmentEditorCommand::CreateRatingTask);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs()
            .has_rating()
    );
    (app, path, original)
}

#[test]
fn rating_compatibility_cancel_and_confirm_preserve_mixed_tasks_and_reopen_missing() {
    let (mut app, path, original) = pending_rating("rating-shell-consent");
    app.save_project();
    assert!(app.pending_format_upgrade.is_some());
    app.cancel_pending_save_as_overwrite();
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .is_dirty()
    );
    app.save_project();
    app.confirm_format_upgrade();
    assert!(app.pending_format_upgrade.is_none());
    let rf_store::project_v3::ProjectV3Read::Editable(project) =
        rf_store::project_v3::read(&path).unwrap()
    else {
        panic!()
    };
    assert_eq!(project.engineering.as_ref().unwrap().tasks.len(), 2);
    assert!(project.has_rating());
    app.open_project(path.clone(), "rating reopen");
    let workspace = &app.platform_host.modeling_inputs().workspace;
    assert!(
        workspace
            .engineering
            .inputs()
            .task(&"task-v3".into())
            .is_some()
    );
    let rating = workspace
        .engineering
        .inputs()
        .tasks
        .iter()
        .find_map(|t| t.as_rating())
        .unwrap();
    let key = rf_ui::engineering::EquipmentCaseKey {
        task_id: rating.id.clone(),
        case_id: rating.cases[0].id.clone(),
    };
    assert_eq!(
        workspace.engineering.case_result_status(&key),
        rf_ui::engineering::EquipmentResultStatus::Missing
    );
    app.save_project();
    assert!(app.pending_format_upgrade.is_none());
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn rating_confirmation_rejects_formal_changes_target_changes_and_new_target_creation() {
    let (mut app, path, original) = pending_rating("rating-shell-stale");
    app.save_project();
    app.equipment_command(EquipmentEditorCommand::CreateSource);
    app.confirm_format_upgrade();
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .is_dirty()
    );
    app.save_project();
    let mut changed: serde_json::Value = serde_json::from_slice(&original).unwrap();
    changed["document"]["metadata"]["title"] = serde_json::json!("Changed by another writer");
    let changed = changed.to_string();
    fs::write(&path, &changed).unwrap();
    app.confirm_format_upgrade();
    assert_eq!(fs::read_to_string(&path).unwrap(), changed);
    assert!(
        app.project_open
            .notice
            .as_ref()
            .unwrap()
            .detail
            .contains("变化")
    );
    let copy = path.with_file_name("new-target.rfproj.json");
    app.request_save_project_as(copy.clone(), false);
    assert!(app.pending_format_upgrade.is_some());
    fs::write(&copy, &original).unwrap();
    app.confirm_format_upgrade();
    assert_eq!(fs::read(&copy).unwrap(), original);
    app.request_save_project_as(copy.clone(), false);
    app.confirm_format_upgrade();
    assert!(
        radishflow_studio::load_project_app_state(&copy)
            .unwrap()
            .workspace
            .engineering
            .inputs()
            .has_rating()
    );
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn known_rating_save_as_to_older_v3_target_prompts_even_after_original_was_saved() {
    let (mut app, path, original) = pending_rating("rating-shell-other-target");
    app.save_project();
    app.confirm_format_upgrade();
    let other = path.with_file_name("older-v3.rfproj.json");
    fs::write(&other, &original).unwrap();
    app.request_save_project_as(other.clone(), false);
    assert!(app.pending_format_upgrade.is_some());
    app.cancel_pending_save_as_overwrite();
    assert_eq!(fs::read(&other).unwrap(), original);
    app.request_save_project_as(other.clone(), false);
    app.confirm_format_upgrade();
    assert!(
        radishflow_studio::load_project_app_state(&other)
            .unwrap()
            .workspace
            .engineering
            .inputs()
            .has_rating()
    );
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn rating_save_target_error_preserves_retry_target_inputs_and_original_file() {
    let (mut app, path, original) = pending_rating("rating-shell-target-failure");
    let target = path.with_file_name("directory-target");
    fs::create_dir(&target).unwrap();
    let before = app.platform_host.modeling_inputs().workspace.clone();
    app.request_save_project_as(target.clone(), true);
    app.confirm_pending_save_as_overwrite();
    assert!(app.pending_format_upgrade.is_none());
    assert_eq!(
        app.project_open.pending_save_as_overwrite.as_ref(),
        Some(&target)
    );
    assert_eq!(
        app.project_open.notice.as_ref().unwrap().title,
        "Save As failed"
    );
    assert_eq!(&app.platform_host.modeling_inputs().workspace, &before);
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(target.is_dir());
    app.cancel_pending_save_as_overwrite();
    assert!(app.project_open.pending_save_as_overwrite.is_none());
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn rating_compatibility_modal_is_readable_in_both_languages_and_defaults_to_cancel() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for key in [egui::Key::Enter, egui::Key::Escape] {
            let (mut app, path, original) = pending_rating("rating-shell-modal");
            app.locale = locale;
            app.save_project();
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            fonts::configure_studio_fonts(&ctx);
            let mut labels = String::new();
            for event in [None, None, Some(key), None] {
                let out = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(400.0, 900.0),
                        )),
                        focused: true,
                        events: event
                            .map(|key| egui::Event::Key {
                                key,
                                physical_key: Some(key),
                                pressed: true,
                                repeat: false,
                                modifiers: egui::Modifiers::NONE,
                            })
                            .into_iter()
                            .collect(),
                        ..Default::default()
                    },
                    |ctx| app.update(ctx),
                );
                labels.push_str(
                    &out.platform_output
                        .accesskit_update
                        .unwrap()
                        .nodes
                        .iter()
                        .flat_map(|(_, n)| [n.label(), n.value()].into_iter().flatten())
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
            assert!(
                labels.contains(if locale == StudioShellLocale::ZhCn {
                    "确认校核任务兼容性"
                } else {
                    "Confirm rating compatibility"
                }),
                "{labels}"
            );
            assert!(
                labels.contains(if locale == StudioShellLocale::ZhCn {
                    "工程仍为 v3"
                } else {
                    "The project remains v3"
                }),
                "{labels}"
            );
            assert!(!labels.contains("v4"));
            assert!(app.pending_format_upgrade.is_none());
            assert_eq!(fs::read(&path).unwrap(), original);
            assert!(
                app.platform_host
                    .modeling_inputs()
                    .workspace
                    .engineering
                    .is_dirty()
            );
            fs::remove_dir_all(path.parent().unwrap()).unwrap();
        }
    }
}

#[test]
fn loaded_rating_is_visible_and_preserved_without_exposing_the_unreviewed_editor() {
    let path = test_preferences_path("rating-shell-loaded").with_extension("rfproj.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        include_str!("../../../../../crates/rf-store/tests/fixtures/project-v3-rating.json"),
    )
    .unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    app.locale = StudioShellLocale::ZhCn;
    app.enter_equipment();
    let before = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .inputs()
        .clone();
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    fonts::configure_studio_fonts(&ctx);
    let mut labels = String::new();
    for _ in 0..2 {
        let out = ctx.run(egui::RawInput::default(), |ctx| app.update(ctx));
        labels.push_str(
            &out.platform_output
                .accesskit_update
                .unwrap()
                .nodes
                .iter()
                .flat_map(|(_, n)| [n.label(), n.value()].into_iter().flatten())
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    assert!(labels.contains("已有几何校核"), "{labels}");
    assert!(labels.contains("将随工程完整保存"), "{labels}");
    assert!(!labels.contains("计算尺寸"));
    app.save_project();
    assert!(app.pending_format_upgrade.is_none());
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs(),
        &before
    );
    assert_eq!(
        radishflow_studio::load_project_app_state(&path)
            .unwrap()
            .workspace
            .engineering
            .inputs(),
        &before
    );
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
