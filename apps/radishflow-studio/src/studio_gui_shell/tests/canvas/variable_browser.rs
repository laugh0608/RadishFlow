use super::*;
use rf_ui::variable_browser::{
    ObjectId, ValueState, VariableBrowser, VariableField, VariableSection, VariableValue,
};

#[test]
fn variable_browser_observes_committed_edits_staleness_reopen_and_inspector_navigation() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);
    author_single_inlet_flash_case_from_blank(
        &mut app,
        SingleInletFlashAuthoringCase {
            case_name: "variable-browser",
            begin_unit_command: "canvas.begin_place_unit.cooler",
            unit_id: "cooler-1",
            connect_unit_inlet_suggestion: "local.cooler.connect_inlet.cooler-1.stream-feed-1-outlet",
            create_unit_outlet_suggestion: "local.cooler.create_outlet.cooler-1",
            unit_outlet_stream_id: "stream-cooler-1-outlet",
            unit_outlet_temperature_k: Some(286.),
            unit_outlet_pressure_pa: 90_000.,
            flash_temperature_k: 280.,
            flash_pressure_pa: 80_000.,
        },
    );
    app.dispatch_ui_command("run_panel.run_manual");
    let snapshot = app.platform_host.snapshot();
    let browser = VariableBrowser::new(
        app.platform_host.document(),
        snapshot.runtime.latest_solve_snapshot.as_ref(),
        snapshot.runtime.stale_solve_snapshot.as_ref(),
    );
    let unit = ObjectId::Unit(UnitId::new("cooler-1"));
    let input = browser
        .variables(&unit)
        .unwrap()
        .into_iter()
        .find(|r| r.id.field == VariableField::OutletTemperature)
        .unwrap();
    let output = browser
        .search("stream-cooler-1-outlet")
        .into_iter()
        .find(|r| {
            r.id.section == VariableSection::Results && r.id.field == VariableField::Temperature
        })
        .unwrap();
    assert_eq!(output.value, Some(VariableValue::Number(286.)));
    let revision = app.platform_host.document().revision;
    app.variable_browser.open = true;
    run_with_key_press(egui::Key::Delete, egui::Modifiers::NONE, |ctx| {
        assert_eq!(app.focus_context(ctx), StudioGuiFocusContext::ModalDialog);
        app.render_variable_browser(ctx);
    });
    assert_eq!(app.platform_host.document().revision, revision);
    app.focus_browser_object(unit.clone());
    assert!(!app.variable_browser.open);
    assert_eq!(app.screen, StudioShellScreen::Workbench);
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
    let field = "unit:cooler-1:outlet_temperature_k";
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(field),
        "0",
    );
    let snapshot = app.platform_host.snapshot();
    let browser = VariableBrowser::new(
        app.platform_host.document(),
        snapshot.runtime.latest_solve_snapshot.as_ref(),
        snapshot.runtime.stale_solve_snapshot.as_ref(),
    );
    assert_eq!(
        browser.read(&input.id).unwrap().value,
        Some(VariableValue::Number(286.))
    );
    assert_eq!(browser.read(&output.id).unwrap().state, ValueState::Valid);
    commit_unit_parameter(&mut app, "cooler-1", field, "275");
    let snapshot = app.platform_host.snapshot();
    let browser = VariableBrowser::new(
        app.platform_host.document(),
        snapshot.runtime.latest_solve_snapshot.as_ref(),
        snapshot.runtime.stale_solve_snapshot.as_ref(),
    );
    assert_eq!(
        browser.read(&input.id).unwrap().value,
        Some(VariableValue::Number(275.))
    );
    assert_eq!(browser.read(&output.id).unwrap().state, ValueState::Stale);
    assert_eq!(browser.read(&output.id).unwrap().value, None);
    app.save_project();
    let saved_bytes = fs::read(&project_path).unwrap();
    app.open_project(project_path.clone(), "project");
    let browser = VariableBrowser::new(app.platform_host.document(), None, None);
    assert_eq!(
        browser.read(&input.id).unwrap().value,
        Some(VariableValue::Number(275.))
    );
    assert_eq!(browser.read(&output.id).unwrap().state, ValueState::Missing);
    app.dispatch_ui_command("run_panel.run_manual");
    let snapshot = app.platform_host.snapshot();
    let browser = VariableBrowser::new(
        app.platform_host.document(),
        snapshot.runtime.latest_solve_snapshot.as_ref(),
        snapshot.runtime.stale_solve_snapshot.as_ref(),
    );
    assert_eq!(
        browser.read(&output.id).unwrap().value,
        Some(VariableValue::Number(275.))
    );
    assert_eq!(fs::read(&project_path).unwrap(), saved_bytes);
    fs::remove_file(project_path).unwrap();
}
