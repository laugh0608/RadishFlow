use super::*;
use rf_types::units::{DisplayUnitSet, MeasurementUnit};
use rf_ui::variable_browser::{ObjectId, VariableField, VariableId, VariableSection};
use rf_ui::{NumericEditEvent, ProjectPresentationCommand};

fn input(app: &mut ReadyAppState, stream: &str, text: &str) -> rf_ui::NumericFieldPresentation {
    app.dispatch_ui_command(format!("inspector.focus_stream:{stream}"));
    let variable = VariableId {
        document: app.platform_host.document().metadata.document_id.clone(),
        object: ObjectId::Stream(stream.into()),
        section: VariableSection::Inputs,
        field: VariableField::Pressure,
    };
    let mut field = app
        .platform_host
        .numeric_field_presentation(&variable, None)
        .unwrap();
    app.edit_numeric_field(
        &mut field,
        NumericEditEvent::SelectInputUnit(MeasurementUnit::Bar),
    );
    app.edit_numeric_field(&mut field, NumericEditEvent::ReplaceText(text.into()));
    field
}

fn commit_pressure(app: &mut ReadyAppState) {
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = "stream:stream-feed:pressure_pa";
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        "110000",
    );
    app.dispatch_inspector_field_draft_commit(
        radishflow_studio::inspector_draft_commit_command_id(key),
    );
}

#[test]
fn input_departure_history_cancel_confirm_and_three_histories_remain_separate() {
    let mut app = ready_app_state(&synced_workspace_config());
    commit_pressure(&mut app);
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    input(&mut app, "stream-feed", "1e-");
    input(&mut app, "stream-vapor", "-2");
    let before = app.platform_host.snapshot().runtime.workspace_document;
    app.dispatch_ui_command("edit.undo");
    assert!(app.pending_input_action.is_some());
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    app.pending_input_action = None;
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    app.dispatch_ui_command("edit.undo");
    app.confirm_input_action();
    let undone = app.platform_host.snapshot().runtime.workspace_document;
    assert_eq!(undone.revision, before.revision + 1);
    assert_eq!(undone.save_state.pending_input_count, 0);
    assert_eq!(undone.presentation, before.presentation);
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&"stream-feed".into()].pressure_pa,
        120000.0
    );
    app.confirm_input_action();
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        undone
    );
    input(&mut app, "stream-feed", "1e-");
    app.dispatch_ui_command("edit.redo");
    app.confirm_input_action();
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&"stream-feed".into()].pressure_pa,
        110000.0
    );
    app.apply_presentation_command(ProjectPresentationCommand::Undo)
        .unwrap();
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .revision,
        undone.revision + 1
    );
}

#[test]
fn input_departure_late_confirmation_reprompts_and_failed_history_keeps_full_draft() {
    let mut app = ready_app_state(&synced_workspace_config());
    let mut field = input(&mut app, "stream-feed", "1e-");
    app.dispatch_ui_command("edit.undo");
    app.edit_numeric_field(&mut field, NumericEditEvent::ReplaceText("2e-".into()));
    let before = app.platform_host.snapshot().runtime.workspace_document;
    app.confirm_input_action();
    assert!(app.pending_input_action.is_some());
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    app.confirm_input_action(); // No engineering history: reject without publishing discard.
    assert!(app.pending_input_action.is_none());
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    assert_eq!(
        app.project_open.notice.as_ref().unwrap().level,
        ProjectOpenNoticeLevel::Error
    );
}

#[test]
fn input_departure_stream_deletion_is_scoped_and_selection_change_rejects_it() {
    let mut app = ready_app_state(&synced_workspace_config());
    input(&mut app, "stream-vapor", "-2");
    input(&mut app, "stream-feed", "1e-");
    let before = app.platform_host.snapshot().runtime.workspace_document;
    app.dispatch_ui_command("canvas.delete_selected_stream");
    assert!(app.pending_input_action.is_some());
    app.dispatch_ui_command("inspector.focus_stream:stream-vapor");
    app.confirm_input_action();
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    app.dispatch_ui_command("canvas.delete_selected_stream");
    app.confirm_input_action();
    let after = app.platform_host.snapshot().runtime.workspace_document;
    assert!(
        !app.platform_host
            .document()
            .flowsheet
            .streams
            .contains_key(&"stream-feed".into())
    );
    assert_eq!(after.save_state.pending_input_count, 1);
    let rf_ui::DraftValue::Numeric(other) = &after.input_edits["stream:stream-vapor:pressure_pa"]
    else {
        panic!("expected numeric draft");
    };
    assert_eq!(other.raw_text(), "-2");
    assert_eq!(other.input_unit(), MeasurementUnit::Bar);
}

#[test]
fn input_departure_open_failure_and_stale_new_or_close_preserve_inputs() {
    let mut app = ready_app_state(&synced_workspace_config());
    input(&mut app, "stream-feed", "1e-");
    let before = app.platform_host.snapshot().runtime.workspace_document;
    app.request_open_project(
        test_preferences_path("missing-input-departure").with_extension("rfproj.json"),
        "missing file",
    );
    app.confirm_pending_project_open();
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    app.create_blank_project();
    input(&mut app, "stream-vapor", "2e-");
    let edited = app.platform_host.snapshot().runtime.workspace_document;
    app.confirm_pending_blank_project();
    assert!(app.project_open.pending_blank_project_confirmation);
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        edited
    );
    app.cancel_pending_blank_project();
    assert!(!app.close_current_window_for_viewport_request());
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    assert!(!app.confirm_pending_close_window());
    assert!(app.project_open.pending_close_window_confirmation.is_some());
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .input_edits,
        edited.input_edits
    );
    app.cancel_pending_close_window();
}

#[test]
fn input_departure_keyboard_undo_uses_the_same_confirmation() {
    let mut app = ready_app_state(&synced_workspace_config());
    commit_pressure(&mut app);
    input(&mut app, "stream-feed", "1e-");
    let before = app.platform_host.snapshot().runtime.workspace_document;
    run_with_key_press(
        egui::Key::Z,
        egui::Modifiers {
            command: true,
            mac_cmd: cfg!(target_os = "macos"),
            ctrl: !cfg!(target_os = "macos"),
            ..egui::Modifiers::NONE
        },
        |ctx| app.dispatch_shortcuts(ctx),
    );
    assert!(app.pending_input_action.is_some());
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
}

#[test]
fn input_departure_repair_command_shortcut_and_widget_share_confirmation() {
    for route in 0..3 {
        let path = test_preferences_path("repair-input-confirmation").with_extension("rfproj.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/flowsheets/failures/unbound-outlet-port.rfproj.json"),
            &path,
        )
        .unwrap();
        let mut app = ready_app_state(&StudioRuntimeConfig {
            project_path: path.clone(),
            ..synced_workspace_config()
        });
        app.dispatch_ui_command("run_panel.run_manual");
        assert!(
            app.platform_host
                .snapshot()
                .runtime
                .run_panel
                .recovery_action()
                .unwrap()
                .mutation
                .is_some()
        );
        app.dispatch_ui_command("inspector.focus_unit:feed-1");
        app.dispatch_inspector_field_draft_update(
            radishflow_studio::inspector_draft_update_command_id("unit:feed-1:name"),
            "尚未采用的名称",
        );
        let before = app.platform_host.snapshot().runtime.workspace_document;
        match route {
            0 => app.dispatch_ui_command("run_panel.recover_failure"),
            1 => run_with_key_press(egui::Key::F8, egui::Modifiers::NONE, |ctx| {
                app.dispatch_shortcuts(ctx)
            }),
            _ => app.dispatch_event(StudioGuiEvent::RunPanelRecoveryRequested),
        }
        assert!(app.pending_input_action.is_some());
        assert_eq!(
            app.platform_host.snapshot().runtime.workspace_document,
            before
        );
        app.confirm_input_action();
        let after = app.platform_host.snapshot().runtime.workspace_document;
        assert_eq!(after.revision, before.revision + 1);
        assert_eq!(after.save_state.pending_input_count, 0);
        app.dispatch_ui_command("edit.undo");
        assert_eq!(
            app.platform_host
                .snapshot()
                .runtime
                .workspace_document
                .stream_count,
            before.stream_count
        );
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn input_departure_unapplied_settings_block_project_replacement() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.open_unit_settings();
    app.unit_settings.draft.as_mut().unwrap().1 = DisplayUnitSet::engineering();
    let before = app.platform_host.snapshot().runtime.workspace_document;
    app.create_blank_project();
    assert!(app.unit_settings.draft.is_some());
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    assert!(!app.close_current_window_for_viewport_request());
    app.request_open_project(synced_workspace_config().project_path, "test");
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
}

#[test]
fn input_departure_deleting_clean_stream_keeps_other_objects_name_draft() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("inspector.focus_stream:stream-vapor");
    let key = "stream:stream-vapor:name";
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        "Vapor draft",
    );
    let before = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .input_edits[key]
        .clone();
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    app.dispatch_ui_command("canvas.delete_selected_stream");
    assert!(app.pending_input_action.is_none());
    assert!(
        !app.platform_host
            .document()
            .flowsheet
            .streams
            .contains_key(&"stream-feed".into())
    );
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .input_edits[key],
        before
    );
}
