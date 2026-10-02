use super::*;
use egui::accesskit::{Node, Role, TreeUpdate};
use rf_types::units::DisplayUnitSet;
use rf_ui::{DisplayUnitViewId, ProjectPresentationCommand};

pub(super) fn accessibility_frame(render: impl FnOnce(&egui::Context)) -> TreeUpdate {
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
    let tree = accessibility_frame(|ctx| {
        assert!(app.render_unit_settings(ctx));
    });
    for (quantity, unit) in [
        ("Absolute temperature", "°C"),
        ("Temperature difference", "K"),
        ("Absolute pressure", "bar"),
        ("Molar flow", "kmol/h"),
        ("Mole fraction", "mol/mol"),
        ("Molar phase fraction", "mol/mol"),
        ("Molar enthalpy", "J/mol"),
    ] {
        assert_eq!(
            combo(&tree, &format!("Project display unit: {quantity}")).value(),
            Some(unit)
        );
    }
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

#[test]
fn numeric_errors_are_localized_without_reinterpreting_or_committing_the_input() {
    use rf_types::units::MeasurementUnit;
    use rf_ui::NumericEditEvent;
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        let mut app = ready_app_state(&synced_workspace_config());
        app.locale = locale;
        app.dispatch_ui_command("inspector.focus_stream:stream-feed");
        app.dispatch_ui_command("run_panel.run_manual");
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
        app.edit_numeric_field(
            &mut field,
            NumericEditEvent::SelectInputUnit(MeasurementUnit::Kilopascal),
        );
        let document = app.platform_host.document().clone();
        let snapshot = app.platform_host.snapshot().runtime.latest_solve_snapshot;
        for (raw, zh, en) in [
            (
                "1.4 bar",
                "输入单位冲突：已选择 kPa，文本后缀为 bar",
                "Unit conflict: selected kPa, but the text uses bar",
            ),
            ("oops", "数值格式无效", "Invalid number"),
            (
                "1 mystery",
                "无法识别单位“mystery”",
                "Unknown unit “mystery”",
            ),
            (
                "1 kelvin",
                "K 不能用于绝对压力",
                "K cannot be used for Absolute pressure",
            ),
            ("1 K", "单位“K”的含义不明确", "Unit “K” is ambiguous"),
            (
                "1 barg",
                "bar(g) 的换算需要参考压力",
                "Converting bar(g) requires a reference pressure",
            ),
            ("NaN", "请输入有限数值", "Enter a finite number"),
            (
                "1e308 kPa",
                "单位换算结果超出数值范围",
                "The converted value is outside the numeric range",
            ),
            ("-1", "无法应用此值", "Cannot apply this value"),
            ("1e-", "输入未完成", "Input incomplete"),
        ] {
            app.edit_numeric_field(&mut field, NumericEditEvent::ReplaceText(raw.into()));
            let tree = accessibility_frame(|ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.render_numeric_property_field(
                        ui,
                        key,
                        if locale == StudioShellLocale::ZhCn {
                            "压力"
                        } else {
                            "Pressure"
                        },
                        &field,
                    );
                });
            });
            let input = tree
                .nodes
                .iter()
                .map(|(_, node)| node)
                .find(|node| node.role() == Role::TextInput)
                .unwrap();
            let label = input.label().unwrap();
            assert!(
                label.contains(if locale == StudioShellLocale::ZhCn {
                    zh
                } else {
                    en
                }),
                "{raw}: {label}"
            );
            assert!(
                !label.contains("UnitConflict")
                    && !label.contains("Conversion(")
                    && !label.contains("Rejected(")
            );
            assert_eq!(input.value(), Some(raw));
            assert_eq!(field.input_unit, MeasurementUnit::Kilopascal);
            assert!(!field.can_apply);
            let projected = app
                .platform_host
                .snapshot()
                .runtime
                .active_inspector_detail
                .unwrap()
                .property_fields
                .into_iter()
                .find(|field| field.key == key)
                .unwrap();
            assert_eq!(
                projected.constraint_text, None,
                "errors must not be duplicated in the constraint slot"
            );
            assert_eq!(app.platform_host.document(), &document);
            assert_eq!(
                app.platform_host.snapshot().runtime.latest_solve_snapshot,
                snapshot
            );
        }
    }
}

#[test]
fn numeric_accessible_name_separates_unit_pending_state_and_issue() {
    for (locale, label, pending, incomplete) in [
        (StudioShellLocale::ZhCn, "压力", "未提交", "输入未完成"),
        (
            StudioShellLocale::En,
            "Pressure",
            "Uncommitted",
            "Input incomplete",
        ),
    ] {
        let mut app = ready_app_state(&synced_workspace_config());
        app.locale = locale;
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
        for edited in [false, true] {
            if edited {
                app.edit_numeric_field(
                    &mut field,
                    rf_ui::NumericEditEvent::ReplaceText("1e-".into()),
                );
            }
            let tree = accessibility_frame(|ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.render_numeric_property_field(ui, key, label, &field);
                });
            });
            let expected = if edited {
                format!("{label} Pa · {pending} · {incomplete}")
            } else {
                format!("{label} Pa")
            };
            let input = tree
                .nodes
                .iter()
                .map(|(_, node)| node)
                .find(|node| node.role() == Role::TextInput)
                .unwrap();
            assert_eq!(input.label(), Some(expected.as_str()));
            assert_eq!(input.value(), Some(field.text.as_str()));
        }
    }
}
