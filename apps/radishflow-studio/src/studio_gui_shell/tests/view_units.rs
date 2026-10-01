use super::*;
use rf_types::units::{DisplayUnitSet, MeasurementUnit as U, QuantityKind as Q};
use rf_ui::variable_browser::{ObjectId, VariableField, VariableId, VariableSection};
use rf_ui::{DisplayUnitViewId, ProjectPresentationCommand as P, ViewDisplayUnits};

fn stream_temperature(app: &ReadyAppState, stream: &str) -> VariableId {
    VariableId {
        document: app.platform_host.document().metadata.document_id.clone(),
        object: ObjectId::Stream(StreamId::new(stream)),
        section: VariableSection::Inputs,
        field: VariableField::Temperature,
    }
}
fn override_temperature(unit: U) -> ViewDisplayUnits {
    let mut units = ViewDisplayUnits::default();
    units.set_unit(Q::AbsoluteTemperature, Some(unit)).unwrap();
    units
}
fn apply_for_window(
    app: &mut ReadyAppState,
    window: u64,
    command: P,
) -> RfResult<radishflow_studio::StudioGuiPlatformExecutedDispatch> {
    app.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
        window_id: window,
        trigger: StudioRuntimeTrigger::ProjectPresentation(command),
    })
}

#[test]
fn inspector_scope_crosses_objects_preserves_drafts_and_leaves_module_display_at_project_units() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let before = app.platform_host.snapshot();
    let view = DisplayUnitViewId(app.current_window_id().unwrap());
    app.apply_presentation_command(P::ApplyView {
        view,
        units: override_temperature(U::Celsius),
    })
    .unwrap();
    let id = stream_temperature(&app, "stream-feed");
    let mut field = app
        .platform_host
        .numeric_field_presentation(&id, Some(view))
        .unwrap();
    assert_eq!(field.text, "26.850000000000023");
    assert_eq!(field.input_unit, U::Celsius);
    assert_eq!(
        app.platform_host
            .numeric_field_presentation(&id, None)
            .unwrap()
            .input_unit,
        U::Kelvin
    );
    app.edit_numeric_field(
        &mut field,
        rf_ui::NumericEditEvent::ReplaceText("1e-".into()),
    );
    assert_eq!(field.input_unit, U::Celsius);
    let pending = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .input_edits;
    app.dispatch_ui_command("inspector.focus_stream:stream-heated");
    assert_eq!(
        app.platform_host
            .numeric_field_presentation(&stream_temperature(&app, "stream-heated"), Some(view))
            .unwrap()
            .input_unit,
        U::Celsius
    );
    app.apply_presentation_command(P::ApplyView {
        view,
        units: ViewDisplayUnits::default(),
    })
    .unwrap();
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    field = app
        .platform_host
        .numeric_field_presentation(&id, Some(view))
        .unwrap();
    assert_eq!(field.input_unit, U::Celsius);
    assert_eq!(field.display_unit, U::Kelvin);
    assert_eq!(field.text, "1e-");
    app.close_inspector_view(view).unwrap();
    let after = app.platform_host.snapshot();
    assert_eq!(after.runtime.workspace_document.input_edits, pending);
    assert_eq!(
        after.runtime.workspace_document.revision,
        before.runtime.workspace_document.revision
    );
    assert_eq!(
        after.runtime.latest_solve_snapshot,
        before.runtime.latest_solve_snapshot
    );
    assert!(!after.runtime.workspace_document.presentation.can_undo());
    assert!(
        !after
            .runtime
            .workspace_document
            .save_state
            .presentation_dirty
    );
    assert_eq!(
        after
            .runtime
            .workspace_document
            .save_state
            .pending_input_count,
        1
    );
}

#[test]
fn closing_one_window_prunes_its_undo_and_redo_without_resetting_other_view_or_project_history() {
    let mut app = ready_app_state(&synced_workspace_config());
    let a = DisplayUnitViewId(app.current_window_id().unwrap());
    app.dispatch_event_result(StudioGuiEvent::OpenWindowRequested)
        .unwrap();
    let b = DisplayUnitViewId(
        app.platform_host
            .snapshot()
            .app_host_state
            .windows
            .iter()
            .find(|window| window.window_id != a.0)
            .unwrap()
            .window_id,
    );
    apply_for_window(
        &mut app,
        a.0,
        P::ApplyView {
            view: a,
            units: override_temperature(U::Celsius),
        },
    )
    .unwrap();
    apply_for_window(&mut app, a.0, P::Apply(DisplayUnitSet::engineering())).unwrap();
    apply_for_window(
        &mut app,
        b.0,
        P::ApplyView {
            view: b,
            units: override_temperature(U::Kelvin),
        },
    )
    .unwrap();
    // Another window cannot initialize or replace this Inspector's scoped preferences.
    let before = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .presentation;
    assert!(apply_for_window(&mut app, b.0, P::CloseView(a)).is_err());
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .presentation,
        before
    );
    let id = stream_temperature(&app, "stream-feed");
    assert!(
        app.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id: b.0,
            trigger: StudioRuntimeTrigger::NumericEdit(rf_ui::NumericEditCommand::BeginInView {
                variable: id,
                view: a
            })
        })
        .is_err()
    );
    app.dispatch_event_result(StudioGuiEvent::CloseWindowRequested { window_id: a.0 })
        .unwrap();
    let state = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .presentation;
    assert!(state.view_units(a).is_empty());
    assert_eq!(
        state.view_units(b).unit_for(Q::AbsoluteTemperature),
        Some(U::Kelvin)
    );
    apply_for_window(&mut app, b.0, P::Undo).unwrap(); // b moves to redo
    app.dispatch_event_result(StudioGuiEvent::CloseWindowRequested { window_id: b.0 })
        .unwrap();
    let state = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .presentation;
    assert!(state.view_units(b).is_empty());
    assert!(!state.can_redo());
    assert!(state.can_undo()); // the project history survived
    app.dispatch_event_result(StudioGuiEvent::OpenWindowRequested)
        .unwrap();
    app.apply_presentation_command(P::Undo).unwrap();
    let state = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .presentation;
    assert_eq!(state.display_units(), &DisplayUnitSet::si());
    assert!(!state.can_undo());
}

#[test]
fn saving_and_reopening_stores_project_units_without_view_overrides() {
    let path = test_preferences_path("view-units-save").with_extension("rfproj.json");
    let mut project = feed_heater_flash_binary_hydrocarbon_project();
    project.schema_version = 2;
    write_project_file(&path, &project).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    let view = DisplayUnitViewId(app.current_window_id().unwrap());
    app.apply_presentation_command(P::ApplyView {
        view,
        units: override_temperature(U::Celsius),
    })
    .unwrap();
    let save = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .save_state;
    assert!(!save.has_unsaved_changes());
    assert!(!save.needs_close_confirmation());
    app.save_project();
    assert_eq!(read_project_file(&path).unwrap(), project);
    app.apply_presentation_command(P::Apply(DisplayUnitSet::engineering()))
        .unwrap();
    app.apply_presentation_command(P::ApplyView {
        view,
        units: override_temperature(U::Kelvin),
    })
    .unwrap();
    app.save_project();
    let reopened = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    let state = reopened
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .presentation;
    assert_eq!(state.display_units(), &DisplayUnitSet::engineering());
    assert!(state.view_units(view).is_empty());
    assert!(!state.can_undo());
    assert!(!state.is_dirty());
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn canceling_view_settings_is_not_a_presentation_change() {
    let mut app = ready_app_state(&synced_workspace_config());
    let view = DisplayUnitViewId(app.current_window_id().unwrap());
    let before = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .presentation;
    app.open_view_unit_settings(view);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        assert!(app.render_view_unit_settings(ctx));
    });
    let _ = ctx.run(
        egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: Some(egui::Key::Escape),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ctx| {
            assert!(app.render_view_unit_settings(ctx));
        },
    );
    assert!(app.unit_settings.view_draft.is_none());
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .presentation,
        before
    );
}

#[test]
fn failed_open_keeps_view_state_and_successful_project_replacement_clears_it() {
    let mut app = ready_app_state(&synced_workspace_config());
    let view = DisplayUnitViewId(app.current_window_id().unwrap());
    app.apply_presentation_command(P::ApplyView {
        view,
        units: override_temperature(U::Celsius),
    })
    .unwrap();
    app.open_view_unit_settings(view);
    let before = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .presentation;
    let missing = test_preferences_path("missing-view-project").with_extension("rfproj.json");
    app.open_project(missing, "test");
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .presentation,
        before
    );
    assert!(app.unit_settings.view_draft.is_some());
    app.close_inspector_view(view).unwrap();
    assert!(app.unit_settings.closed_inspectors.contains(&view));
    let (config, project) = blank_workspace_config();
    app.open_project(config.project_path, "test");
    assert!(app.unit_settings.closed_inspectors.is_empty());
    assert!(app.unit_settings.view_draft.is_none());
    let presentation = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .presentation;
    assert!(presentation.view_units(view).is_empty());
    assert!(!presentation.can_undo());
    fs::remove_file(project).unwrap();
}
