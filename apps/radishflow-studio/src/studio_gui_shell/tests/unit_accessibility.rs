use super::*;
use egui::accesskit::{Node, Role, TreeUpdate};
use rf_types::units::DisplayUnitSet;
use rf_ui::{DisplayUnitViewId, ProjectPresentationCommand};

fn accessibility_frame(render: impl FnOnce(&egui::Context)) -> TreeUpdate {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 860.0),
        )),
        ..Default::default()
    });
    render(&ctx);
    ctx.end_pass().platform_output.accesskit_update.unwrap()
}

fn combo<'a>(tree: &'a TreeUpdate, label: &str) -> &'a Node {
    tree.nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::ComboBox && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing accessible unit selector: {label}"))
}

#[test]
fn input_unit_selector_exposes_field_scope_value_and_composition_disabled_state() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = "stream:stream-feed:pressure_pa";
    let mut field = app
        .platform_host
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
        .unwrap();
    for composing in [false, true] {
        if composing {
            app.edit_numeric_field(
                &mut field,
                rf_ui::NumericEditEvent::PreviewComposition("1".into()),
            );
        }
        let tree = accessibility_frame(|ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let id = ui.make_persistent_id(("numeric-input", &field.variable.document, key));
                ui.memory_mut(|memory| memory.request_focus(id));
                app.render_numeric_property_field(ui, key, "Pressure", &field);
            });
        });
        let selector = combo(&tree, "Pressure input unit");
        assert_eq!(selector.value(), Some("Pa"));
        assert_eq!(selector.is_disabled(), composing);
    }
}

#[test]
fn project_and_view_unit_selectors_expose_distinct_scope_and_current_selection() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    app.open_unit_settings();
    let tree = accessibility_frame(|ctx| {
        assert!(app.render_unit_settings(ctx));
    });
    for (quantity, unit) in [
        ("绝对温度", "°C"),
        ("温差", "K"),
        ("绝对压力", "bar"),
        ("摩尔流量", "kmol/h"),
        ("摩尔分数", "mol/mol"),
        ("相摩尔分率", "mol/mol"),
        ("摩尔焓", "J/mol"),
    ] {
        assert_eq!(
            combo(&tree, &format!("项目显示单位：{quantity}")).value(),
            Some(unit)
        );
    }
    app.locale = StudioShellLocale::En;
    app.open_view_unit_settings(DisplayUnitViewId(app.current_window_id().unwrap()));
    let tree = accessibility_frame(|ctx| {
        assert!(app.render_view_unit_settings(ctx));
    });
    for quantity in ["Temperature", "Pressure", "Molar flow"] {
        assert_eq!(
            combo(&tree, &format!("Inspector display unit: {quantity}")).value(),
            Some("Follow project")
        );
    }
}
