use super::*;
use egui::accesskit::{Action, ActionRequest, NodeId, Role, TreeUpdate};
use rf_types::units::{DisplayUnitSet, MeasurementUnit};
use rf_ui::{NumericEditEvent, ProjectPresentationCommand};

const PRESSURE: &str = "stream:stream-feed:pressure_pa";

fn key(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

fn focus(node: NodeId) -> egui::Event {
    egui::Event::AccessKitActionRequest(ActionRequest {
        action: Action::Focus,
        target: node,
        data: None,
    })
}

fn node(tree: &TreeUpdate, role: Role, label: &str) -> NodeId {
    tree.nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {role:?} {label}"))
        .0
}

fn focused_label(tree: &TreeUpdate) -> Option<&str> {
    tree.nodes
        .iter()
        .find(|(id, _)| *id == tree.focus)
        .and_then(|(_, node)| node.label())
}

fn active_option_label(tree: &TreeUpdate, selector: NodeId) -> Option<&str> {
    let option = tree
        .nodes
        .iter()
        .find(|(id, _)| *id == selector)?
        .1
        .active_descendant()?;
    tree.nodes.iter().find(|(id, _)| *id == option)?.1.label()
}

fn pressure(app: &ReadyAppState) -> rf_ui::NumericFieldPresentation {
    app.platform_host
        .snapshot()
        .runtime
        .active_inspector_detail
        .unwrap()
        .property_fields
        .into_iter()
        .find(|field| field.key == PRESSURE)
        .unwrap()
        .numeric
        .unwrap()
        .unwrap()
}

fn input_frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
) -> TreeUpdate {
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(400.0, 600.0),
        )),
        focused: true,
        events,
        ..Default::default()
    });
    app.dispatch_shortcuts(ctx);
    egui::CentralPanel::default().show(ctx, |ui| {
        app.render_numeric_property_field(ui, PRESSURE, "Pressure", &pressure(app));
    });
    ctx.end_pass().platform_output.accesskit_update.unwrap()
}

fn input_app() -> (ReadyAppState, egui::Context) {
    let mut app = ready_app_state(&synced_workspace_config());
    app.locale = StudioShellLocale::En;
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    app.dispatch_ui_command("run_panel.run_manual");
    app.edit_numeric_field(
        &mut pressure(&app),
        NumericEditEvent::ReplaceText("1.4".into()),
    );
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    (app, ctx)
}

#[test]
fn input_unit_menu_keeps_arrows_inside_then_returns_focus_for_keyboard_apply() {
    let (mut app, ctx) = input_app();
    let before = app.platform_host.snapshot();
    let document = app.platform_host.document().clone();
    let first = input_frame(&mut app, &ctx, vec![]);
    let selector = node(&first, Role::ComboBox, "Pressure input unit");
    input_frame(&mut app, &ctx, vec![focus(selector)]);
    input_frame(&mut app, &ctx, vec![key(egui::Key::Enter)]);
    input_frame(&mut app, &ctx, vec![]);
    let next = input_frame(&mut app, &ctx, vec![key(egui::Key::Tab)]);
    assert_eq!(next.focus, selector);
    assert_eq!(active_option_label(&next, selector), Some("Pa"));
    let previous = input_frame(
        &mut app,
        &ctx,
        vec![egui::Event::Key {
            key: egui::Key::Tab,
            physical_key: Some(egui::Key::Tab),
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::SHIFT,
        }],
    );
    assert_eq!(previous.focus, selector);
    assert_eq!(active_option_label(&previous, selector), Some("bar"));
    input_frame(&mut app, &ctx, vec![key(egui::Key::ArrowDown)]);
    let second = input_frame(&mut app, &ctx, vec![key(egui::Key::ArrowDown)]);
    assert_eq!(second.focus, selector);
    assert_eq!(active_option_label(&second, selector), Some("kPa"));
    let selector_node = &second
        .nodes
        .iter()
        .find(|(id, _)| *id == selector)
        .unwrap()
        .1;
    assert_eq!(selector_node.is_expanded(), Some(true));
    assert_eq!(selector_node.controls().len(), 1);
    let list = &second
        .nodes
        .iter()
        .find(|(id, _)| *id == selector_node.controls()[0])
        .unwrap()
        .1;
    assert_eq!(list.role(), Role::ListBox);
    assert!(
        list.children()
            .contains(&selector_node.active_descendant().unwrap())
    );
    let options: Vec<_> = list
        .children()
        .iter()
        .map(|child| {
            let option = &second.nodes.iter().find(|(id, _)| id == child).unwrap().1;
            assert_eq!(option.role(), Role::ListBoxOption);
            option.label().unwrap()
        })
        .collect();
    assert_eq!(options, ["Pa", "kPa", "MPa", "bar"]);
    assert_eq!(pressure(&app).text, "1.4");

    let selected = input_frame(&mut app, &ctx, vec![key(egui::Key::Enter)]);
    assert!(!ctx.memory(|memory| memory.any_popup_open()));
    assert_eq!(selected.focus, selector);
    assert_eq!(pressure(&app).input_unit, MeasurementUnit::Kilopascal);
    assert_eq!(pressure(&app).text, "140");
    assert_eq!(app.platform_host.document(), &document);
    assert_eq!(
        app.platform_host.snapshot().runtime.latest_solve_snapshot,
        before.runtime.latest_solve_snapshot
    );

    let apply = input_frame(&mut app, &ctx, vec![key(egui::Key::Tab)]);
    assert_eq!(focused_label(&apply), Some("Apply"));
    input_frame(&mut app, &ctx, vec![key(egui::Key::Enter)]);
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")].pressure_pa,
        140_000.0
    );
    assert!(pressure(&app).generation.is_none());
}

#[test]
fn input_unit_menu_escape_restores_selector_without_changing_the_candidate() {
    let (mut app, ctx) = input_app();
    let before = pressure(&app);
    let first = input_frame(&mut app, &ctx, vec![]);
    let selector = node(&first, Role::ComboBox, "Pressure input unit");
    input_frame(&mut app, &ctx, vec![focus(selector)]);
    input_frame(&mut app, &ctx, vec![key(egui::Key::Enter)]);
    input_frame(&mut app, &ctx, vec![]);
    input_frame(&mut app, &ctx, vec![key(egui::Key::ArrowDown)]);
    let escaped = input_frame(&mut app, &ctx, vec![key(egui::Key::Escape)]);
    assert!(!ctx.memory(|memory| memory.any_popup_open()));
    assert_eq!(escaped.focus, selector);
    assert_eq!(pressure(&app), before);
    input_frame(&mut app, &ctx, vec![key(egui::Key::Escape)]);
    assert!(!pressure(&app).pending);
    assert!(pressure(&app).issue.is_none());
    let returned = input_frame(&mut app, &ctx, vec![]);
    assert_eq!(focused_label(&returned), Some("Pressure bar"));
}

fn settings_frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    view: bool,
    events: Vec<egui::Event>,
) -> TreeUpdate {
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 860.0),
        )),
        focused: true,
        events,
        ..Default::default()
    });
    if view {
        app.render_view_unit_settings(ctx);
    } else {
        app.render_unit_settings(ctx);
    }
    ctx.end_pass().platform_output.accesskit_update.unwrap()
}

#[test]
fn display_unit_menus_keep_keyboard_selection_local_until_settings_are_applied() {
    for view in [false, true] {
        let (mut app, ctx) = input_app();
        let field = pressure(&app);
        let document = app.platform_host.document().clone();
        let snapshot = app.platform_host.snapshot().runtime.latest_solve_snapshot;
        if view {
            app.open_view_unit_settings(rf_ui::DisplayUnitViewId(app.current_window_id().unwrap()));
        } else {
            app.open_unit_settings();
        }
        let label = if view {
            "Inspector display unit: Pressure"
        } else {
            "Project display unit: Absolute pressure"
        };
        settings_frame(&mut app, &ctx, view, vec![]);
        let first = settings_frame(&mut app, &ctx, view, vec![]);
        let selector = node(&first, Role::ComboBox, label);
        settings_frame(&mut app, &ctx, view, vec![focus(selector)]);
        settings_frame(&mut app, &ctx, view, vec![key(egui::Key::Enter)]);
        settings_frame(&mut app, &ctx, view, vec![]);
        settings_frame(&mut app, &ctx, view, vec![key(egui::Key::Home)]);
        settings_frame(&mut app, &ctx, view, vec![key(egui::Key::ArrowDown)]);
        if view {
            settings_frame(&mut app, &ctx, view, vec![key(egui::Key::ArrowDown)]);
        }
        let picked = settings_frame(&mut app, &ctx, view, vec![key(egui::Key::Enter)]);
        assert!(!ctx.memory(|memory| memory.any_popup_open()));
        assert_eq!(picked.focus, selector);
        let updated = settings_frame(&mut app, &ctx, view, vec![]);
        assert_eq!(
            updated
                .nodes
                .iter()
                .find(|(id, _)| *id == selector)
                .unwrap()
                .1
                .value(),
            Some("kPa")
        );
        assert_eq!(app.platform_host.document(), &document);
        assert_eq!(pressure(&app), field);
        assert_eq!(
            app.platform_host.snapshot().runtime.latest_solve_snapshot,
            snapshot
        );

        settings_frame(&mut app, &ctx, view, vec![key(egui::Key::Enter)]);
        settings_frame(&mut app, &ctx, view, vec![key(egui::Key::End)]);
        let canceled = settings_frame(&mut app, &ctx, view, vec![key(egui::Key::Escape)]);
        assert!(!ctx.memory(|memory| memory.any_popup_open()));
        assert!(app.unit_settings.is_open(), "Escape closes only the menu");
        assert_eq!(canceled.focus, selector);
        let apply = node(
            &canceled,
            Role::Button,
            if view {
                "Apply to this view"
            } else {
                "Apply display settings"
            },
        );
        settings_frame(&mut app, &ctx, view, vec![focus(apply)]);
        settings_frame(&mut app, &ctx, view, vec![key(egui::Key::Enter)]);
        assert!(!app.unit_settings.is_open());
        assert_eq!(app.platform_host.document(), &document);
        assert_eq!(pressure(&app).text, field.text);
        assert_eq!(pressure(&app).input_unit, field.input_unit);
        let inspector_field = app
            .platform_host
            .numeric_field_presentation(
                &field.variable,
                Some(rf_ui::DisplayUnitViewId(app.current_window_id().unwrap())),
            )
            .unwrap();
        assert_eq!(inspector_field.display_unit, MeasurementUnit::Kilopascal);
        assert_eq!(
            app.platform_host.snapshot().runtime.latest_solve_snapshot,
            snapshot
        );
    }
}
