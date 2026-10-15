use super::*;
use rf_types::units::{DisplayUnitSet, MeasurementUnit, QuantityKind};

fn v3_file(name: &str, unknown: bool) -> PathBuf {
    let path = test_preferences_path(name).with_extension("rfproj.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        include_bytes!(
            "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ),
    )
    .unwrap();
    let mut app = radishflow_studio::load_project_app_state(&path).unwrap();
    let task = rf_model::equipment::SeparatorSizingTask {
        id: "task-v3".into(),
        revision: 0,
        tag: "V-101".into(),
        name: "Incomplete equipment".into(),
        method: rf_model::equipment::EquipmentMethod {
            id: "radishflow.separator.vertical-sb-working-volume".into(),
            version: 1,
        },
        case: rf_model::equipment::SeparatorCase {
            id: "case-v3".into(),
            name: "Draft".into(),
            purpose: "Lifecycle test".into(),
            fluid_system: String::new(),
            applicability_source_ids: vec![],
            inputs: Default::default(),
            applicability: Default::default(),
        },
        axial_sections: Default::default(),
    };
    app.workspace
        .engineering
        .apply(vec![rf_ui::engineering::EngineeringCommand::CreateTask(
            Box::new(task),
        )])
        .unwrap();
    radishflow_studio::dispatch_document_lifecycle(
        &mut app,
        radishflow_studio::StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    if unknown {
        let text = fs::read_to_string(&path).unwrap().replace(
            "radishflow.separator.vertical-sb-working-volume",
            "future.method",
        );
        fs::write(&path, text).unwrap();
    }
    path
}
#[test]
fn studio_known_v3_opens_saves_as_and_reopens_without_dropping_equipment() {
    let path = v3_file("studio-known-v3", false);
    let copy = path.with_file_name("copy.rfproj.json");
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    assert!(app.read_only_project.is_none());
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .presentation
            .source_file_version(),
        Some(3)
    );
    app.request_save_project_as(copy.clone(), false);
    assert!(app.pending_format_upgrade.is_none());
    assert!(copy.is_file());
    app.open_project(copy.clone(), "v3 test");
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .project_path
            .as_deref(),
        Some(copy.to_str().unwrap())
    );
    let rf_store::project_v3::ProjectV3Read::Editable(project) =
        rf_store::project_v3::read(&copy).unwrap()
    else {
        unreachable!()
    };
    assert_eq!(project.engineering.unwrap().tasks.len(), 1);
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
#[test]
fn v3_upgrade_cancel_stale_confirmation_and_success_preserve_formal_content() {
    let path = test_preferences_path("studio-v3-upgrade").with_extension("rfproj.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let original = include_bytes!(
        "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    );
    fs::write(&path, original).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    let mut units = DisplayUnitSet::si();
    units
        .set_unit(QuantityKind::Duration, MeasurementUnit::Minute)
        .unwrap();
    app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(units.clone()))
        .unwrap();
    app.save_project();
    assert!(app.pending_format_upgrade.is_some());
    app.cancel_pending_save_as_overwrite();
    assert_eq!(fs::read(&path).unwrap(), original);
    app.save_project();
    units
        .set_unit(QuantityKind::Length, MeasurementUnit::Millimetre)
        .unwrap();
    app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(units.clone()))
        .unwrap();
    app.confirm_format_upgrade();
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(
        app.project_open
            .notice
            .as_ref()
            .unwrap()
            .detail
            .contains("变化")
    );
    app.save_project();
    app.confirm_format_upgrade();
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .presentation
            .source_file_version(),
        Some(3)
    );
    assert!(
        !app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .save_state
            .presentation_dirty
    );
    app.open_project(path.clone(), "v3 test");
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .presentation
            .display_units(),
        &units
    );
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
#[test]
fn unknown_project_inspection_blocks_edit_run_save_shortcuts_and_keeps_existing_workspace() {
    let path = v3_file("studio-unknown-v3", true);
    let text = fs::read(&path).unwrap();
    let mut app = ready_app_state(&synced_workspace_config());
    let before = app.platform_host.snapshot().window_model();
    app.open_project(path.clone(), "unknown method");
    assert!(app.read_only_project.is_some());
    for key in [egui::Key::S, egui::Key::Z, egui::Key::Enter] {
        run_with_key_press(key, egui::Modifiers::COMMAND, |ctx| app.update(ctx));
    }
    assert_eq!(app.platform_host.snapshot().window_model(), before);
    assert_eq!(fs::read(&path).unwrap(), text);
    assert!(app.pending_format_upgrade.is_none());
    run_with_key_press(egui::Key::Escape, egui::Modifiers::NONE, |ctx| {
        app.update(ctx)
    });
    assert!(app.read_only_project.is_none());
    assert_eq!(app.platform_host.snapshot().window_model(), before);
    let startup = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    assert!(startup.read_only_project.is_some());
    assert_eq!(fs::read(&path).unwrap(), text);
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
