use super::super::tests::{app, flow_selection, synchronize, text_rects};
use super::*;
use rf_types::units::MeasurementUnit as U;
use rf_ui::{NumericEditEvent as E, result_query::ResultStatus};

mod keyboard;
mod lifecycle;

fn select(app: &mut ReadyAppState, object: ObjectId, field: VariableField) -> BrowserSelection {
    app.open_variable_browser();
    synchronize(app);
    let selection = flow_selection(app, object, VariableSection::Inputs, field);
    app.variable_browser.selected = Some(selection.clone());
    app.variable_browser.detail_page = true;
    selection
}

fn field(app: &ReadyAppState, selection: &BrowserSelection) -> NumericFieldPresentation {
    presentation(app, &app.variable_browser, selection)
        .unwrap()
        .1
}

fn commit(app: &mut ReadyAppState, field: &mut NumericFieldPresentation) -> bool {
    let command = NumericEditCommand::Commit {
        variable: field.variable.clone(),
        generation: field.generation.unwrap(),
    };
    app.dispatch_numeric_command(field, command)
}

fn cancel(app: &mut ReadyAppState, field: &mut NumericFieldPresentation) {
    let command = NumericEditCommand::Cancel {
        variable: field.variable.clone(),
        generation: field.generation.unwrap(),
    };
    assert!(app.dispatch_numeric_command(field, command));
}

#[test]
fn browser_edit_scope_uses_live_kind_connections_and_field_identity() {
    let mut app = app();
    for (object, fields) in [
        (
            ObjectId::Unit("heater-1".into()),
            vec![
                VariableField::OutletTemperature,
                VariableField::OutletPressure,
            ],
        ),
        (
            ObjectId::Stream("stream-feed".into()),
            vec![
                VariableField::Temperature,
                VariableField::Pressure,
                VariableField::MolarFlow,
            ],
        ),
    ] {
        for value in fields {
            let selection = select(&mut app, object.clone(), value);
            assert!(presentation(&app, &app.variable_browser, &selection).is_some());
        }
    }
    for (object, value, section) in [
        (
            ObjectId::Unit("feed-1".into()),
            VariableField::OutletTemperature,
            VariableSection::Inputs,
        ),
        (
            ObjectId::Unit("flash-1".into()),
            VariableField::OutletPressure,
            VariableSection::Inputs,
        ),
        (
            ObjectId::Unit("heater-1".into()),
            VariableField::Name,
            VariableSection::Inputs,
        ),
        (
            ObjectId::Stream("stream-heated".into()),
            VariableField::Temperature,
            VariableSection::Inputs,
        ),
        (
            ObjectId::Stream("stream-feed".into()),
            VariableField::Temperature,
            VariableSection::Results,
        ),
        (
            ObjectId::Stream("stream-feed".into()),
            VariableField::MoleFraction("methane".into()),
            VariableSection::Inputs,
        ),
    ] {
        let selection = flow_selection(&app, object, section, value);
        assert!(presentation(&app, &app.variable_browser, &selection).is_none());
    }
    let selection = select(
        &mut app,
        ObjectId::Stream("stream-feed".into()),
        VariableField::Pressure,
    );
    let mut workspace = app.platform_host.modeling_inputs().workspace.clone();
    for port in &mut workspace
        .document
        .flowsheet
        .units
        .get_mut(&"feed-1".into())
        .unwrap()
        .ports
    {
        port.stream_id = None;
    }
    assert!(
        field_key(&workspace, &selection).is_none(),
        "an unconnected stream is outside this slice"
    );
    let heater = flow_selection(
        &app,
        ObjectId::Unit("heater-1".into()),
        VariableSection::Inputs,
        VariableField::OutletTemperature,
    );
    workspace
        .document
        .flowsheet
        .units
        .get_mut(&"heater-1".into())
        .unwrap()
        .kind = "feed".into();
    assert!(field_key(&workspace, &heater).is_none());
    let stale = VariableBrowserState::default();
    assert!(presentation(&app, &stale, &selection).is_none());
}

#[test]
fn browser_and_inspector_share_drafts_and_both_formal_input_transactions() {
    for (object, value, raw, unit, expected) in [
        (
            ObjectId::Unit("heater-1".into()),
            VariableField::OutletTemperature,
            "76.85",
            U::Celsius,
            350.0,
        ),
        (
            ObjectId::Stream("stream-feed".into()),
            VariableField::MolarFlow,
            "36",
            U::KilomolePerHour,
            10.0,
        ),
        (
            ObjectId::Unit("heater-1".into()),
            VariableField::OutletPressure,
            "0.9",
            U::Bar,
            90000.0,
        ),
        (
            ObjectId::Stream("stream-feed".into()),
            VariableField::Pressure,
            "1.1",
            U::Bar,
            110000.0,
        ),
        (
            ObjectId::Stream("stream-feed".into()),
            VariableField::Temperature,
            "30",
            U::Celsius,
            303.15,
        ),
    ] {
        let mut browser = app();
        let mut inspector = app();
        for app in [&mut browser, &mut inspector] {
            app.dispatch_ui_command("run_panel.run_manual");
        }
        let original = browser.platform_host.document().flowsheet.clone();
        let revision = browser.platform_host.document().revision;
        let selection = select(&mut browser, object.clone(), value.clone());
        let mut browser_field = field(&browser, &selection);
        let mut inspector_field = inspector
            .platform_host
            .numeric_field_presentation(&browser_field.variable, None)
            .unwrap();
        for (app, field) in [
            (&mut browser, &mut browser_field),
            (&mut inspector, &mut inspector_field),
        ] {
            assert!(app.edit_numeric_field(field, E::SelectInputUnit(unit)));
            assert!(app.edit_numeric_field(field, E::ReplaceText(raw.into())));
            assert_eq!(app.platform_host.document().revision, revision);
            assert_eq!(
                app.platform_host
                    .modeling_inputs()
                    .workspace
                    .flow_result()
                    .status(),
                ResultStatus::Current
            );
            assert!(commit(app, field));
            assert!(field.generation.is_none());
        }
        assert_eq!(
            browser.platform_host.document().flowsheet,
            inspector.platform_host.document().flowsheet
        );
        let workspace = &browser.platform_host.modeling_inputs().workspace;
        let descriptor =
            rf_ui::variable_browser::VariableBrowser::new(&workspace.document, None, None)
                .read(&browser_field.variable)
                .unwrap();
        let Some(rf_ui::variable_browser::VariableValue::Number(actual)) = descriptor.value else {
            panic!("numeric input")
        };
        assert!((actual - expected).abs() <= expected.abs() * 1e-12);
        assert_eq!(workspace.document.revision, revision + 1);
        assert_eq!(workspace.flow_result().status(), ResultStatus::Stale);
        if value == VariableField::OutletTemperature {
            assert_eq!(
                workspace.document.flowsheet.streams[&"stream-heated".into()].temperature_k,
                350.0
            );
        }
        browser.dispatch_ui_command("edit.undo");
        assert_eq!(
            browser.platform_host.document().flowsheet,
            original,
            "one input = one history entry"
        );
        browser.dispatch_ui_command("edit.redo");
        assert_eq!(
            browser.platform_host.document().flowsheet,
            inspector.platform_host.document().flowsheet
        );
    }
}

#[test]
fn browser_draft_failure_cancel_and_same_value_keep_formal_state() {
    let mut app = app();
    app.dispatch_ui_command("run_panel.run_manual");
    let selection = select(
        &mut app,
        ObjectId::Unit("heater-1".into()),
        VariableField::OutletPressure,
    );
    let original = app.platform_host.document().clone();
    let qualification = app
        .platform_host
        .modeling_inputs()
        .workspace
        .flow_run_qualification();
    let mut current = field(&app, &selection);
    assert!(app.edit_numeric_field(&mut current, E::SelectInputUnit(U::Bar)));
    assert!(app.edit_numeric_field(&mut current, E::ReplaceText("1.3".into())));
    assert!(!current.can_apply);
    assert!(!commit(&mut app, &mut current));
    assert_eq!(current.text, "1.3");
    assert_eq!(current.input_unit, U::Bar);
    assert!(current.issue.is_some());
    assert!(app.edit_numeric_field(&mut current, E::ReplaceText("1e-".into())));
    app.variable_browser.directory = DirectoryView::Variables;
    app.variable_browser.mode = BrowserMode::Side;
    app.variable_browser.query = "missing".into();
    app.variable_browser.detail_page = false;
    synchronize(&mut app);
    let inspector = app
        .platform_host
        .numeric_field_presentation(&current.variable, None)
        .unwrap();
    assert_eq!(inspector.text, "1e-");
    assert_eq!(inspector.input_unit, U::Bar);
    assert_eq!(inspector.generation, current.generation);
    assert_eq!(app.platform_host.document(), &original);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_run_qualification(),
        qualification
    );
    cancel(&mut app, &mut current);
    assert_eq!(app.platform_host.document(), &original);
    assert!(app.edit_numeric_field(&mut current, E::ReplaceText("95000 Pa".into())));
    assert!(commit(&mut app, &mut current));
    assert_eq!(app.platform_host.document(), &original);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_result()
            .status(),
        ResultStatus::Current
    );
}
