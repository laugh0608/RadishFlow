use super::*;
use egui::accesskit::{Action, ActionRequest, NodeId, Role};

fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}
fn focus(target: NodeId) -> egui::Event {
    egui::Event::AccessKitActionRequest(ActionRequest {
        action: Action::Focus,
        target,
        data: None,
    })
}
fn node(output: &egui::FullOutput, role: Role, label: &str) -> NodeId {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| {
            node.role() == role && node.label().is_some_and(|text| text.starts_with(label))
        })
        .unwrap_or_else(|| panic!("missing {role:?} {label}: {:?}", text_rects(output)))
        .0
}
fn frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(width, 1600.0),
        )),
        focused: true,
        time: Some(ctx.cumulative_pass_nr() as f64 * 0.1),
        events,
        ..Default::default()
    });
    app.dispatch_shortcuts(ctx);
    egui::CentralPanel::default().show(ctx, |ui| app.render_variable_browser_content(ui));
    ctx.end_pass()
}
fn select_all(ctx: &egui::Context, text: &str) {
    let id = ctx
        .memory(|memory| memory.focused())
        .expect("numeric input focus");
    let mut state = egui::TextEdit::load_state(ctx, id).expect("text state");
    state
        .cursor
        .set_char_range(Some(egui::text::CCursorRange::two(
            egui::text::CCursor::new(0),
            egui::text::CCursor::new(text.chars().count()),
        )));
    state.store(ctx, id);
}
fn begin(app: &mut ReadyAppState, ctx: &egui::Context, width: f32) {
    frame(app, ctx, width, vec![]);
    let output = frame(app, ctx, width, vec![]);
    let button = node(&output, Role::Button, "Edit");
    frame(app, ctx, width, vec![focus(button)]);
    frame(
        app,
        ctx,
        width,
        vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    frame(app, ctx, width, vec![]);
}

#[test]
fn browser_tab_to_units_keeps_an_untouched_explicit_edit_session() {
    let mut app = app();
    app.locale = StudioShellLocale::En;
    let selection = select(
        &mut app,
        ObjectId::Stream("stream-feed".into()),
        VariableField::MolarFlow,
    );
    app.variable_browser.mode = BrowserMode::Side;
    let original = app.platform_host.document().clone();
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    begin(&mut app, &ctx, 400.0);
    let generation = field(&app, &selection).generation;
    assert!(generation.is_some());
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
    );
    let output = frame(&mut app, &ctx, 400.0, vec![]);
    assert_eq!(field(&app, &selection).generation, generation);
    assert_eq!(
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .focus,
        node(&output, Role::ComboBox, "Input value input unit")
    );
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    assert!(ctx.memory(|memory| memory.any_popup_open()));
    assert_eq!(app.platform_host.document(), &original);
}

#[test]
fn browser_keyboard_unit_menu_escape_and_draft_return_share_u2() {
    for (width, mode) in [(1200.0, BrowserMode::Expanded), (400.0, BrowserMode::Side)] {
        let mut app = app();
        app.locale = StudioShellLocale::En;
        let selection = select(
            &mut app,
            ObjectId::Unit("heater-1".into()),
            VariableField::OutletTemperature,
        );
        app.variable_browser.mode = mode;
        let original = app.platform_host.document().clone();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        begin(&mut app, &ctx, width);
        select_all(&ctx, "345");
        let output = frame(&mut app, &ctx, width, vec![egui::Event::Text("350".into())]);
        let selector = node(&output, Role::ComboBox, "Input value input unit");
        frame(&mut app, &ctx, width, vec![focus(selector)]);
        frame(
            &mut app,
            &ctx,
            width,
            vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        assert!(ctx.memory(|memory| memory.any_popup_open()));
        frame(&mut app, &ctx, width, vec![]);
        frame(
            &mut app,
            &ctx,
            width,
            vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
        );
        assert!(!ctx.memory(|memory| memory.any_popup_open()));
        assert_eq!(field(&app, &selection).text, "350");
        assert!(app.variable_browser.open);
        frame(
            &mut app,
            &ctx,
            width,
            vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        frame(&mut app, &ctx, width, vec![]);
        frame(
            &mut app,
            &ctx,
            width,
            vec![key(egui::Key::ArrowDown, egui::Modifiers::NONE)],
        );
        let output = frame(
            &mut app,
            &ctx,
            width,
            vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        assert_eq!(field(&app, &selection).input_unit, U::Celsius);
        let celsius = field(&app, &selection).text;
        assert!((celsius.parse::<f64>().unwrap() - 76.85).abs() < 1e-12);
        assert_eq!(app.platform_host.document(), &original);
        assert_eq!(
            output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .focus,
            node(&output, Role::TextInput, "Input value")
        );
        select_all(&ctx, &celsius);
        frame(&mut app, &ctx, width, vec![egui::Event::Text("1e-".into())]);
        frame(
            &mut app,
            &ctx,
            width,
            vec![key(egui::Key::Tab, egui::Modifiers::NONE)],
        );
        assert_eq!(field(&app, &selection).text, "1e-");
        assert!(!field(&app, &selection).can_apply);
        app.variable_browser.detail_page = false;
        app.variable_browser.directory = DirectoryView::Variables;
        app.variable_browser.query = "not-found".into();
        frame(&mut app, &ctx, width, vec![]);
        app.variable_browser.query.clear();
        app.variable_browser.detail_page = true;
        let output = frame(&mut app, &ctx, width, vec![]);
        assert_eq!(field(&app, &selection).text, "1e-");
        assert_eq!(field(&app, &selection).input_unit, U::Celsius);
        let input = node(&output, Role::TextInput, "Input value");
        frame(&mut app, &ctx, width, vec![focus(input)]);
        frame(
            &mut app,
            &ctx,
            width,
            vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
        );
        assert!(field(&app, &selection).generation.is_none());
        assert!(app.variable_browser.open);
        assert!(app.variable_browser.detail_page);
        assert_eq!(app.platform_host.document(), &original);
    }
}

#[test]
fn browser_enter_applies_once_and_history_shortcuts_route_by_focus() {
    let mut app = app();
    app.locale = StudioShellLocale::En;
    let selection = select(
        &mut app,
        ObjectId::Unit("heater-1".into()),
        VariableField::OutletTemperature,
    );
    let revision = app.platform_host.document().revision;
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    begin(&mut app, &ctx, 400.0);
    select_all(&ctx, "345");
    frame(&mut app, &ctx, 400.0, vec![egui::Event::Text("350".into())]);
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![key(egui::Key::Z, egui::Modifiers::COMMAND)],
    );
    assert_eq!(field(&app, &selection).text, "345");
    assert_eq!(app.platform_host.document().revision, revision);
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![key(
            egui::Key::Z,
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        )],
    );
    assert_eq!(field(&app, &selection).text, "350");
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    assert_eq!(app.platform_host.document().revision, revision + 1);
    assert!(field(&app, &selection).generation.is_none());
    let output = frame(&mut app, &ctx, 400.0, vec![]);
    let button = node(&output, Role::Button, "Edit");
    frame(&mut app, &ctx, 400.0, vec![focus(button)]);
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![key(egui::Key::Z, egui::Modifiers::COMMAND)],
    );
    assert_eq!(
        app.platform_host.document().flowsheet.units[&"heater-1".into()]
            .parameters
            .outlet_temperature_k,
        Some(345.0)
    );
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![key(
            egui::Key::Z,
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        )],
    );
    assert_eq!(
        app.platform_host.document().flowsheet.units[&"heater-1".into()]
            .parameters
            .outlet_temperature_k,
        Some(350.0)
    );
}

#[test]
fn browser_ime_enter_finishes_composition_before_a_separate_apply() {
    let mut app = app();
    app.locale = StudioShellLocale::En;
    let selection = select(
        &mut app,
        ObjectId::Stream("stream-feed".into()),
        VariableField::Pressure,
    );
    app.variable_browser.mode = BrowserMode::Side;
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    begin(&mut app, &ctx, 400.0);
    let original = app.platform_host.document().clone();
    select_all(&ctx, &field(&app, &selection).text);
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![
            egui::Event::Ime(egui::ImeEvent::Enabled),
            egui::Event::Ime(egui::ImeEvent::Preedit("110000".into())),
        ],
    );
    assert!(field(&app, &selection).composing);
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![key(egui::Key::Escape, egui::Modifiers::NONE)],
    );
    assert!(app.variable_browser.detail_page && app.variable_browser.open);
    assert!(field(&app, &selection).generation.is_some());
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![
            key(egui::Key::Enter, egui::Modifiers::NONE),
            egui::Event::Ime(egui::ImeEvent::Commit("110000".into())),
        ],
    );
    assert_eq!(field(&app, &selection).text, "110000");
    assert_eq!(app.platform_host.document(), &original);
    frame(
        &mut app,
        &ctx,
        400.0,
        vec![key(egui::Key::Enter, egui::Modifiers::NONE)],
    );
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&"stream-feed".into()].pressure_pa,
        110000.0
    );
}

#[test]
fn browser_editing_and_rejection_fit_400_pixels_in_both_locales_at_double_font() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        let mut app = app();
        app.locale = locale;
        let selection = select(
            &mut app,
            ObjectId::Unit("heater-1".into()),
            VariableField::OutletPressure,
        );
        app.variable_browser.mode = BrowserMode::Side;
        let mut current = field(&app, &selection);
        app.edit_numeric_field(&mut current, E::SelectInputUnit(U::Bar));
        app.edit_numeric_field(&mut current, E::ReplaceText("1.3".into()));
        let ctx = egui::Context::default();
        ctx.style_mut(|style| {
            for font in style.text_styles.values_mut() {
                font.size *= 2.0;
            }
        });
        frame(&mut app, &ctx, 400.0, vec![]);
        let output = frame(&mut app, &ctx, 400.0, vec![]);
        for (text, rect) in text_rects(&output) {
            assert!(
                rect.left() >= -1.0 && rect.right() <= 401.0,
                "{locale:?}: {text} {rect:?}"
            );
        }
        assert_eq!(field(&app, &selection).text, "1.3");
        assert_eq!(field(&app, &selection).input_unit, U::Bar);
    }
}
