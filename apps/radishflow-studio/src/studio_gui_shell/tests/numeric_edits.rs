use super::*;

mod field_matrix;

#[test]
fn legacy_si_inspector_commands_keep_explicit_units() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let before = app.platform_host.snapshot();
    let key = "stream:stream-feed:pressure_pa";
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        "1.2 bar",
    );
    let invalid = app.platform_host.snapshot();
    let field = invalid
        .runtime
        .active_inspector_detail
        .as_ref()
        .unwrap()
        .property_fields
        .iter()
        .find(|field| field.key == key)
        .unwrap();
    assert!(field.commit_command_id.is_none());
    assert_eq!(field.current_value, "1.2 bar");
    assert_eq!(field.label, "Pressure (Pa)");
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        "110000 Pa",
    );
    let edited = app.platform_host.snapshot();
    let field = edited
        .runtime
        .active_inspector_detail
        .as_ref()
        .unwrap()
        .property_fields
        .iter()
        .find(|field| field.key == key)
        .unwrap();
    assert_eq!(field.label, "Pressure (Pa)");
    assert_eq!(field.current_value, "110000 Pa");
    assert_eq!(
        edited.runtime.workspace_document.revision,
        before.runtime.workspace_document.revision
    );
    assert_eq!(
        edited.runtime.latest_solve_snapshot,
        before.runtime.latest_solve_snapshot
    );
    app.dispatch_inspector_field_draft_commit(field.commit_command_id.as_ref().unwrap().clone());
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")].pressure_pa,
        110000.0
    );
    let committed = app.platform_host.snapshot();
    assert_eq!(
        committed
            .runtime
            .workspace_document
            .save_state
            .pending_input_count,
        0
    );
    assert!(committed.runtime.latest_solve_snapshot.is_none());
    app.dispatch_ui_command("edit.undo");
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")].pressure_pa,
        120000.0
    );
}

fn numeric_field(app: &ReadyAppState, key: &str) -> rf_ui::NumericFieldPresentation {
    app.platform_host
        .snapshot()
        .runtime
        .active_inspector_detail
        .unwrap()
        .property_fields
        .into_iter()
        .find(|field| field.key == key)
        .unwrap()
        .numeric
        .unwrap()
        .unwrap()
}

fn key_event(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn numeric_frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    key: &str,
    focus: bool,
    events: Vec<egui::Event>,
) -> egui::Id {
    let field = numeric_field(app, key);
    let mut input_id = egui::Id::NULL;
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(400.0, 600.0),
        )),
        focused: true,
        time: Some(ctx.cumulative_pass_nr() as f64 * 0.1),
        events,
        ..Default::default()
    });
    app.dispatch_shortcuts(ctx);
    egui::CentralPanel::default().show(ctx, |ui| {
        input_id = ui.make_persistent_id(("numeric-input", &field.variable.document, key));
        if focus {
            ui.memory_mut(|memory| memory.request_focus(input_id));
        }
        app.render_numeric_property_field(ui, key, "Pressure", &field);
    });
    let _ = ctx.end_pass();
    input_id
}

fn select_all(ctx: &egui::Context, id: egui::Id, text: &str) {
    let mut state = egui::TextEdit::load_state(ctx, id).unwrap();
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::two(
            egui::text::CCursor::new(0),
            egui::text::CCursor::new(text.chars().count()),
        )));
    state.store(ctx, id);
}

#[test]
fn native_numeric_paste_switch_undo_and_enter_keep_a_single_history_owner() {
    use rf_types::units::MeasurementUnit as U;
    use rf_ui::NumericEditEvent as E;
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = "stream:stream-feed:pressure_pa";
    let ctx = egui::Context::default();
    let before = app.platform_host.snapshot();
    let id = numeric_frame(&mut app, &ctx, key, true, vec![]);
    let original = numeric_field(&app, key).text;
    select_all(&ctx, id, &original);
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![egui::Event::Paste("1.1 bar".into())],
    );
    let mut field = numeric_field(&app, key);
    assert_eq!(field.input_unit, U::Bar);
    assert!(field.can_apply);
    assert!(app.edit_numeric_field(&mut field, E::SelectInputUnit(U::Kilopascal)));
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(egui::Key::Z, egui::Modifiers::COMMAND)],
    );
    let field = numeric_field(&app, key);
    assert_eq!(field.text, "1.1 bar");
    assert_eq!(field.input_unit, U::Bar);
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(egui::Key::Z, egui::Modifiers::COMMAND)],
    );
    let field = numeric_field(&app, key);
    assert_eq!(field.text, original);
    assert_eq!(field.input_unit, U::Pascal);
    assert!(!field.pending);
    assert_eq!(
        app.platform_host.snapshot().runtime.latest_solve_snapshot,
        before.runtime.latest_solve_snapshot
    );
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(
            egui::Key::Z,
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        )],
    );
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    assert!(
        (app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")].pressure_pa
            - 110000.0)
            .abs()
            < 1e-8
    );
    assert!(numeric_field(&app, key).generation.is_none());
    app.dispatch_ui_command("edit.undo");
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")].pressure_pa,
        120000.0
    );
}

#[test]
fn native_ime_confirmation_enter_does_not_apply_engineering_input() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = "stream:stream-feed:pressure_pa";
    let ctx = egui::Context::default();
    let id = numeric_frame(&mut app, &ctx, key, true, vec![]);
    let original = numeric_field(&app, key).text;
    select_all(&ctx, id, &original);
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![
            egui::Event::Ime(egui::ImeEvent::Enabled),
            egui::Event::Ime(egui::ImeEvent::Preedit("110000".into())),
        ],
    );
    let field = numeric_field(&app, key);
    assert!(field.composing && field.pending && !field.can_apply);
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![
            key_event(egui::Key::Enter, egui::Modifiers::NONE),
            egui::Event::Ime(egui::ImeEvent::Commit("110000".into())),
        ],
    );
    let field = numeric_field(&app, key);
    assert!(!field.composing);
    assert_eq!(field.text, "110000");
    assert!(field.can_apply);
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")].pressure_pa,
        120000.0
    );
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    assert!(
        (app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")].pressure_pa
            - 110000.0)
            .abs()
            < 1e-8
    );
}

#[test]
fn native_typing_coalesces_and_project_display_switch_preserves_incomplete_draft() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = "stream:stream-feed:pressure_pa";
    let ctx = egui::Context::default();
    let id = numeric_frame(&mut app, &ctx, key, true, vec![]);
    let original = numeric_field(&app, key).text;
    select_all(&ctx, id, &original);
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![egui::Event::Text("1".into())],
    );
    for ch in ["1", "0", "0", "0", "0"] {
        numeric_frame(
            &mut app,
            &ctx,
            key,
            false,
            vec![egui::Event::Text(ch.into())],
        );
    }
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(egui::Key::Z, egui::Modifiers::COMMAND)],
    );
    assert_eq!(
        numeric_field(&app, key).text,
        "1",
        "selection replacement is its own atomic edit"
    );
    select_all(&ctx, id, "1");
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![egui::Event::Text("1e-".into())],
    );
    app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(
        rf_types::units::DisplayUnitSet::engineering(),
    ))
    .unwrap();
    numeric_frame(&mut app, &ctx, key, false, vec![]);
    let mut field = numeric_field(&app, key);
    assert_eq!(field.text, "1e-");
    assert_eq!(field.input_unit, rf_types::units::MeasurementUnit::Pascal);
    assert_eq!(field.display_unit, rf_types::units::MeasurementUnit::Bar);
    let generation = field.generation;
    assert!(!app.edit_numeric_field(
        &mut field,
        rf_ui::NumericEditEvent::SelectInputUnit(rf_types::units::MeasurementUnit::Bar)
    ));
    assert_eq!(field.generation, generation);
    assert_eq!(field.text, "1e-");
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(egui::Key::Escape, egui::Modifiers::NONE)],
    );
    let field = numeric_field(&app, key);
    assert!(!field.pending);
    assert_eq!(field.text, "1.2");
    assert_eq!(field.input_unit, rf_types::units::MeasurementUnit::Bar);
}

#[test]
fn native_tab_preserves_redo_after_undo_to_the_committed_value() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = "stream:stream-feed:pressure_pa";
    let ctx = egui::Context::default();
    let id = numeric_frame(&mut app, &ctx, key, true, vec![]);
    select_all(&ctx, id, &numeric_field(&app, key).text);
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![egui::Event::Paste("110000".into())],
    );
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(egui::Key::Z, egui::Modifiers::COMMAND)],
    );
    assert!(numeric_field(&app, key).can_redo);
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(egui::Key::Tab, egui::Modifiers::NONE)],
    );
    assert!(numeric_field(&app, key).can_redo);
    numeric_frame(&mut app, &ctx, key, true, vec![]);
    numeric_frame(
        &mut app,
        &ctx,
        key,
        false,
        vec![key_event(
            egui::Key::Z,
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        )],
    );
    assert_eq!(numeric_field(&app, key).text, "110000");
}
