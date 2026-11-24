use super::basic::render_property_page_texts;
use super::*;

#[test]
fn component_catalog_partial_binary_projects_reopen_but_still_require_the_full_thermo_composition_to_solve()
 {
    let (config, path) = blank_workspace_config();
    for kept in ["methane", "ethane"] {
        let mut project = feed_heater_flash_binary_hydrocarbon_project();
        let sheet = &mut project.document.flowsheet;
        sheet.components.retain(|id, _| id.as_str() == kept);
        for stream in sheet.streams.values_mut() {
            stream.overall_mole_fractions = [(kept.into(), 1.0)].into();
        }
        write_project_file(&path, &project).unwrap();
        let mut app = ready_app_state(&config);
        assert_eq!(
            app.platform_host.document().flowsheet,
            project.document.flowsheet
        );
        app.dispatch_ui_command("run_panel.run_manual");
        let session = &app.platform_host.modeling_inputs().workspace.solve_session;
        let diagnostic = session.latest_diagnostic.as_ref().unwrap();
        assert!(
            diagnostic
                .primary_message
                .contains("thermo system expects 2"),
            "{diagnostic:?}"
        );
        assert!(
            app.platform_host
                .snapshot()
                .window_model()
                .runtime
                .latest_solve_snapshot
                .is_none()
        );
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn component_catalog_matches_a_saved_read_only_component_by_its_id_without_a_formula() {
    let config = StudioRuntimeConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/flowsheets/two-feed-single-vapor-sensible.rfproj.json"),
        ..synced_workspace_config()
    };
    let mut app = ready_app_state(&config);
    app.component_filter = "COMPONENT-A".into();
    let texts = render_property_page_texts(&mut app);
    for expected in ["Component A", "component-a", "此组分由工程文件定义，只读。"] {
        assert!(texts.iter().any(|text| text == expected), "{texts:?}");
    }
    assert!(!texts.iter().any(|text| text == "Component B"));
    let window = app.platform_host.snapshot().window_model();
    let component = window
        .property_page
        .component_choices
        .iter()
        .find(|choice| choice.component_id == "component-a")
        .unwrap();
    assert!(component.selected && !component.remove_enabled && component.formula.is_none());
}

#[test]
fn component_catalog_filter_preserves_project_and_resets_on_reopen() {
    let (config, path) = blank_workspace_config();
    let mut app = ready_app_state(&config);
    let before = app.platform_host.document().clone();
    for (query, visible) in [
        (" ch4 ", vec!["Methane"]),
        ("EtHaNe", vec!["Ethane", "Methane"]),
        ("th", vec!["Methane", "Ethane"]),
        ("x-no-hit", vec![]),
        ("", vec!["Methane", "Ethane"]),
    ] {
        app.component_filter = query.to_owned();
        let texts = render_property_page_texts(&mut app);
        let listed: Vec<_> = texts
            .iter()
            .map(String::as_str)
            .filter(|s| matches!(*s, "Methane" | "Ethane"))
            .collect();
        assert_eq!(listed, visible, "{query}");
        assert_eq!(
            texts.iter().any(|text| text == "没有匹配的组分"),
            visible.is_empty()
        );
        assert_eq!(app.platform_host.document(), &before);
    }
    app.component_filter = "CH4".into();
    app.dispatch_ui_command("project.property_package.select:binary-hydrocarbon-lite-v1");
    app.dispatch_ui_command("project.component.select:methane");
    let selected = app.platform_host.document().clone();
    app.dispatch_ui_command("project.component.select:methane");
    assert_eq!(app.platform_host.document(), &selected);
    assert!(
        app.platform_host
            .snapshot()
            .window_model()
            .property_page
            .flowsheet_modeling_enabled
    );
    app.dispatch_ui_command("project.component.remove:methane");
    assert!(app.platform_host.document().flowsheet.components.is_empty());
    app.dispatch_ui_command("edit.undo");
    assert_eq!(app.platform_host.document().flowsheet, selected.flowsheet);
    app.dispatch_ui_command("edit.redo");
    assert!(app.platform_host.document().flowsheet.components.is_empty());
    app.dispatch_ui_command("edit.undo");
    save_project_with_upgrade_consent(&mut app);
    let saved = app.platform_host.document().flowsheet.clone();
    app.open_project(path.clone(), "test");
    assert!(app.component_filter.is_empty());
    assert_eq!(saved, selected.flowsheet);
    assert_eq!(app.platform_host.document().flowsheet, saved);
    fs::remove_file(path).unwrap();
}

#[test]
fn component_catalog_lists_actual_references_and_rejects_removal_without_invalidating_result() {
    let (config, path) = blank_workspace_config();
    let mut project: StoredProjectFile = serde_json::from_str(include_str!(
        "../../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    let feed = project
        .document
        .flowsheet
        .streams
        .get_mut(&StreamId::new("stream-feed-a"))
        .unwrap();
    let methane = rf_types::ComponentId::new("methane");
    feed.overall_mole_fractions.insert(methane, 0.0);
    feed.overall_mole_fractions.insert("ethane".into(), 1.0);
    write_project_file(&path, &project).unwrap();
    let mut app = ready_app_state(&config);
    app.dispatch_ui_command("run_panel.run_manual");
    let before = app.platform_host.snapshot().window_model();
    let choice = before
        .property_page
        .component_choices
        .iter()
        .find(|c| c.component_id == "methane")
        .unwrap();
    let expected = [
        "Feed A (stream-feed-a)",
        "Feed B (stream-feed-b)",
        "Liquid Outlet (stream-liquid)",
        "Mixer Outlet (stream-mix-out)",
        "Vapor Outlet (stream-vapor)",
    ];
    assert!(!choice.remove_enabled);
    app.component_filter = "CH4".into();
    let texts = render_property_page_texts(&mut app).join("\n");
    for reference in expected {
        assert!(texts.contains(reference), "{reference}: {texts}");
    }
    app.dispatch_ui_command(&choice.remove_command_id);
    let after = app.platform_host.snapshot().window_model();
    assert_eq!(
        after.runtime.workspace_document,
        before.runtime.workspace_document
    );
    assert!(before.runtime.latest_solve_snapshot.is_some());
    assert_eq!(
        after.runtime.latest_solve_snapshot,
        before.runtime.latest_solve_snapshot
    );
    fs::remove_file(path).unwrap();
}
