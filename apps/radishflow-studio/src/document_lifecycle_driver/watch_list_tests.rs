use super::{
    tests::{app_state_from_example, temp_project_path},
    *,
};
use rf_ui::variable_browser::{
    ObjectId, StableVariableReference, VariableField, VariableId, VariableReferenceError,
    VariableSection, VariableWatchCommand, VariableWatchDirection, VariableWatchOutcome,
};
use std::{fs, time::UNIX_EPOCH};

fn initialized_app(path: &PathBuf) -> AppState {
    let mut app = app_state_from_example(path);
    let prepared = prepare_object_identity_initialization(&app, path).unwrap();
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::InitializeObjectIdentities(Box::new(prepared)),
    )
    .unwrap();
    app
}

fn outlet_temperature(app: &AppState) -> VariableId {
    VariableId {
        document: app.workspace.document.metadata.document_id.clone(),
        object: ObjectId::Unit("heater-1".into()),
        section: VariableSection::Inputs,
        field: VariableField::OutletTemperature,
    }
}

fn add(app: &mut AppState, variable: VariableId) -> VariableWatchOutcome {
    app.workspace
        .edit_variable_watch_list(&VariableWatchCommand::Add(variable))
        .unwrap()
}

fn save_first(app: &mut AppState, path: &Path) {
    let review = project_save_compatibility(app, path).unwrap().unwrap();
    assert!(matches!(
        review,
        ProjectSaveCompatibility::FlowCapabilities {
            watch_list_entries: Some(_),
            ..
        }
    ));
    assert!(dispatch_document_lifecycle(app, StudioDocumentLifecycleCommand::Save).is_err());
    dispatch_document_lifecycle(
        app,
        StudioDocumentLifecycleCommand::SaveUpgraded {
            path: path.to_owned(),
        },
    )
    .unwrap();
}

#[test]
fn watch_list_saves_reopens_and_forks_without_dirtying_physical_inputs() {
    let path = temp_project_path("watch-save");
    let copy = temp_project_path("watch-copy");
    let mut app = initialized_app(&path);
    let variable = outlet_temperature(&app);
    let original = fs::read(&path).unwrap();
    let qualification = app.flow_run_qualification();
    let history = app.workspace.command_history.clone();
    assert_eq!(add(&mut app, variable.clone()), VariableWatchOutcome::Added);
    let generation = app.workspace.variable_watch_list.generation();
    let first = app.workspace.variable_watch_list.entries()[0].clone();
    assert_eq!(
        add(&mut app, variable.clone()),
        VariableWatchOutcome::AlreadyWatched {
            reference: first.clone()
        }
    );
    assert_eq!(app.workspace.variable_watch_list.generation(), generation);
    assert!(app.workspace.project_save_state().watch_list_dirty);
    assert!(
        app.workspace
            .project_save_state()
            .needs_close_confirmation()
    );
    assert!(app.flow_run_qualification().same_input(&qualification));
    assert_eq!(app.workspace.command_history, history);
    assert_eq!(fs::read(&path).unwrap(), original);

    save_first(&mut app, &path);
    assert!(!app.workspace.project_save_state().watch_list_dirty);
    let original = fs::read(&path).unwrap();
    let mut reopened = load_project_app_state(&path).unwrap();
    assert_eq!(
        reopened.workspace.variable_watch_list.entries(),
        std::slice::from_ref(&first)
    );
    let reference: StableVariableReference = first.parse().unwrap();
    assert_eq!(reference.resolve(&reopened.workspace).unwrap(), variable);
    let second_variable = VariableId {
        field: VariableField::Name,
        ..variable
    };
    assert_eq!(
        add(&mut reopened, second_variable),
        VariableWatchOutcome::Added
    );
    let second = reopened.workspace.variable_watch_list.entries()[1].clone();
    assert_eq!(
        reopened
            .workspace
            .edit_variable_watch_list(&VariableWatchCommand::Move {
                reference: second.clone(),
                direction: VariableWatchDirection::Up,
            })
            .unwrap(),
        VariableWatchOutcome::Moved { position: 1 }
    );
    dispatch_document_lifecycle(
        &mut reopened,
        StudioDocumentLifecycleCommand::SaveAs { path: copy.clone() },
    )
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(
        reopened.workspace.document_path.as_deref(),
        Some(copy.as_path())
    );
    let mut copied = load_project_app_state(&copy).unwrap();
    assert_eq!(
        copied.workspace.variable_watch_list.entries(),
        &[second.clone(), first.clone()]
    );
    assert!(!copied.workspace.project_save_state().watch_list_dirty);
    assert_eq!(
        copied.workspace.document.metadata.document_id,
        reopened.workspace.document.metadata.document_id
    );

    assert!(matches!(
        project_save_compatibility(&copied, &path).unwrap(),
        Some(ProjectSaveCompatibility::FlowCapabilities {
            replaces_target_variable_watch_list: true,
            ..
        })
    ));
    assert!(
        dispatch_document_lifecycle(
            &mut copied,
            StudioDocumentLifecycleCommand::SaveAs { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(
        copied
            .workspace
            .edit_variable_watch_list(&VariableWatchCommand::Remove(second))
            .unwrap(),
        VariableWatchOutcome::Removed {
            next: Some(first.clone())
        }
    );
    assert_eq!(
        copied
            .workspace
            .edit_variable_watch_list(&VariableWatchCommand::Remove(first))
            .unwrap(),
        VariableWatchOutcome::Removed { next: None }
    );
    dispatch_document_lifecycle(&mut copied, StudioDocumentLifecycleCommand::Save).unwrap();
    assert_eq!(
        load_project_app_state(&copy)
            .unwrap()
            .workspace
            .variable_watch_list
            .entries_for_save(),
        Some([].as_slice())
    );
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::remove_file(path).unwrap();
    fs::remove_file(copy).unwrap();
}

#[test]
fn watch_list_preserves_invalid_entries_and_recovers_only_original_identity() {
    let path = temp_project_path("watch-invalid");
    let mut app = initialized_app(&path);
    let variable = outlet_temperature(&app);
    add(&mut app, variable);
    save_first(&mut app, &path);
    let original_reference = app.workspace.variable_watch_list.entries()[0].clone();
    let reference: StableVariableReference = original_reference.parse().unwrap();
    app.delete_unit(&"heater-1".into(), UNIX_EPOCH).unwrap();
    assert_eq!(
        reference.resolve(&app.workspace),
        Err(VariableReferenceError::ObjectMissing)
    );
    assert!(!app.workspace.project_save_state().watch_list_dirty);
    app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
    assert!(reference.resolve(&app.workspace).is_ok());
    app.redo_document_command(UNIX_EPOCH).unwrap().unwrap();
    dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).unwrap();
    let mut reopened = load_project_app_state(&path).unwrap();
    let recreated = reopened
        .create_builtin_unit(rf_ui::BuiltinUnitKind::Heater, UNIX_EPOCH)
        .unwrap();
    assert_eq!(recreated.unit_id.as_str(), "heater-1");
    assert_eq!(
        reference.resolve(&reopened.workspace),
        Err(VariableReferenceError::ObjectRecreated)
    );
    let mut opaque = reopened.workspace.variable_watch_list.entries().to_vec();
    opaque.push("rfvar:2/unknown".into());
    opaque.push(" broken text ".into());
    reopened.workspace.variable_watch_list =
        rf_ui::variable_browser::VariableWatchList::from_saved(Some(opaque.clone()));
    dispatch_document_lifecycle(&mut reopened, StudioDocumentLifecycleCommand::Save).unwrap();
    assert_eq!(
        load_project_app_state(&path)
            .unwrap()
            .workspace
            .variable_watch_list
            .entries(),
        opaque
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn watch_list_can_save_a_new_object_and_can_discard_unpublished_first_use() {
    let path = temp_project_path("watch-unsaved-object");
    let mut app = initialized_app(&path);
    let created = app
        .create_builtin_unit(rf_ui::BuiltinUnitKind::Heater, UNIX_EPOCH)
        .unwrap();
    let variable = VariableId {
        object: ObjectId::Unit(created.unit_id),
        ..outlet_temperature(&app)
    };
    assert_eq!(
        StableVariableReference::for_saved_variable(&app.workspace, variable.clone()),
        Err(VariableReferenceError::ObjectNotSaved)
    );
    assert_eq!(add(&mut app, variable.clone()), VariableWatchOutcome::Added);
    let text = app.workspace.variable_watch_list.entries()[0].clone();
    app.workspace
        .edit_variable_watch_list(&VariableWatchCommand::Remove(text))
        .unwrap();
    assert!(!app.workspace.project_save_state().watch_list_dirty);
    assert!(
        app.workspace
            .variable_watch_list
            .entries_for_save()
            .is_none()
    );
    assert!(project_save_compatibility(&app, &path).unwrap().is_none());

    add(&mut app, variable.clone());
    save_first(&mut app, &path);
    let reopened = load_project_app_state(&path).unwrap();
    let reference: StableVariableReference = reopened.workspace.variable_watch_list.entries()[0]
        .parse()
        .unwrap();
    assert_eq!(reference.resolve(&reopened.workspace).unwrap(), variable);
    fs::remove_file(path).unwrap();
}
