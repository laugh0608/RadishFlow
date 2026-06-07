use super::*;

#[test]
fn open_example_project_rebuilds_runtime_for_selected_sample() {
    let mut app = ready_app_state(&synced_workspace_config());
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();

    app.dispatch_ui_command("run_panel.run_manual");
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .snapshot_history_count,
        1
    );

    app.open_example_project(target_project.clone());
    let window = app.platform_host.snapshot().window_model();

    assert_eq!(
        window.runtime.workspace_document.title,
        "Feed Valve Flash Binary Hydrocarbon Example"
    );
    assert_eq!(window.runtime.workspace_document.snapshot_history_count, 0);
    assert_eq!(
        window
            .runtime
            .example_projects
            .iter()
            .find(|example| example.is_current)
            .map(|example| example.id),
        Some("feed-valve-flash")
    );
    assert_eq!(
        window.runtime.run_panel.view().primary_action.label,
        "Resume"
    );

    app.dispatch_ui_command("run_panel.run_manual");
    let solved_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        solved_window.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert!(solved_window.runtime.latest_solve_snapshot.is_some());
    assert_eq!(
        app.right_sidebar_tab,
        StudioShellRightSidebarTab::ModuleResults
    );
    assert_eq!(
        app.bottom_drawer_tab,
        StudioShellBottomDrawerTab::ResultsTable
    );

    app.screen = StudioShellScreen::Home;
    app.open_recent_project(target_project);
    let reopened_window = app.platform_host.snapshot().window_model();
    assert_eq!(app.screen, StudioShellScreen::Workbench);
    assert_eq!(
        reopened_window.runtime.workspace_document.title,
        "Feed Valve Flash Binary Hydrocarbon Example"
    );
    assert_eq!(
        reopened_window
            .runtime
            .workspace_document
            .snapshot_history_count,
        0
    );

    app.dispatch_ui_command("run_panel.run_manual");
    let rerun_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        rerun_window.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert!(rerun_window.runtime.latest_solve_snapshot.is_some());
}

#[test]
fn open_project_from_input_rebuilds_runtime_and_records_feedback() {
    let mut app = ready_app_state(&synced_workspace_config());
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-cooler-flash")
        .expect("expected feed cooler example")
        .project_path
        .clone();

    app.project_open.path_input = target_project.display().to_string();
    app.open_project_from_input();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.title,
        "Feed Cooler Flash Binary Hydrocarbon Example"
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Info)
    );
    assert!(
        window
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("opened project"))
    );
    assert_eq!(
        app.project_open.recent_projects.first(),
        Some(&target_project)
    );
}

#[test]
fn unit_parameter_edits_save_reopen_and_rerun_official_examples() {
    let cases = [
        UnitParameterShellCase {
            name: "feed-temperature",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "feed-1",
            draft_key: "unit:feed-1:outlet_temperature_k",
            raw_value: "310",
            expected_value: 310.0,
            outlet_stream_ids: &["stream-feed"],
            field: UnitParameterShellField::OutletTemperatureK,
        },
        UnitParameterShellCase {
            name: "feed-pressure",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "feed-1",
            draft_key: "unit:feed-1:outlet_pressure_pa",
            raw_value: "130000",
            expected_value: 130_000.0,
            outlet_stream_ids: &["stream-feed"],
            field: UnitParameterShellField::OutletPressurePa,
        },
        UnitParameterShellCase {
            name: "heater",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "heater-1",
            draft_key: "unit:heater-1:outlet_temperature_k",
            raw_value: "361.25",
            expected_value: 361.25,
            outlet_stream_ids: &["stream-heated"],
            field: UnitParameterShellField::OutletTemperatureK,
        },
        UnitParameterShellCase {
            name: "heater-pressure",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "heater-1",
            draft_key: "unit:heater-1:outlet_pressure_pa",
            raw_value: "90000",
            expected_value: 90_000.0,
            outlet_stream_ids: &["stream-heated"],
            field: UnitParameterShellField::OutletPressurePa,
        },
        UnitParameterShellCase {
            name: "cooler",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "cooler-1",
            draft_key: "unit:cooler-1:outlet_temperature_k",
            raw_value: "302.75",
            expected_value: 302.75,
            outlet_stream_ids: &["stream-cooled"],
            field: UnitParameterShellField::OutletTemperatureK,
        },
        UnitParameterShellCase {
            name: "cooler-pressure",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "cooler-1",
            draft_key: "unit:cooler-1:outlet_pressure_pa",
            raw_value: "640000",
            expected_value: 640_000.0,
            outlet_stream_ids: &["stream-cooled"],
            field: UnitParameterShellField::OutletPressurePa,
        },
        UnitParameterShellCase {
            name: "mixer-pressure",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "mixer-1",
            draft_key: "unit:mixer-1:outlet_pressure_pa",
            raw_value: "640000",
            expected_value: 640_000.0,
            outlet_stream_ids: &["stream-mix-out"],
            field: UnitParameterShellField::OutletPressurePa,
        },
        UnitParameterShellCase {
            name: "valve",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "valve-1",
            draft_key: "unit:valve-1:outlet_pressure_pa",
            raw_value: "640000",
            expected_value: 640_000.0,
            outlet_stream_ids: &["stream-throttled"],
            field: UnitParameterShellField::OutletPressurePa,
        },
        UnitParameterShellCase {
            name: "flash-temperature",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "flash-1",
            draft_key: "unit:flash-1:outlet_temperature_k",
            raw_value: "335",
            expected_value: 335.0,
            outlet_stream_ids: &["stream-liquid", "stream-vapor"],
            field: UnitParameterShellField::OutletTemperatureK,
        },
        UnitParameterShellCase {
            name: "flash-pressure",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            unit_id: "flash-1",
            draft_key: "unit:flash-1:outlet_pressure_pa",
            raw_value: "680000",
            expected_value: 680_000.0,
            outlet_stream_ids: &["stream-liquid", "stream-vapor"],
            field: UnitParameterShellField::OutletPressurePa,
        },
    ];

    for case in cases {
        let project_path = temporary_project_path(case.name);
        let project = serde_json::from_str(case.project_json)
            .unwrap_or_else(|error| panic!("expected {} project fixture: {error}", case.name));
        write_project_file(&project_path, &project)
            .unwrap_or_else(|error| panic!("expected {} temp project write: {error}", case.name));
        let config = StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..synced_workspace_config()
        };
        let mut app = ready_app_state(&config);

        app.dispatch_ui_command(format!("inspector.focus_unit:{}", case.unit_id));
        app.dispatch_inspector_field_draft_update(
            radishflow_studio::inspector_draft_update_command_id(case.draft_key),
            case.raw_value,
        );
        app.dispatch_inspector_field_draft_commit(
            radishflow_studio::inspector_draft_commit_command_id(case.draft_key),
        );

        let edited = app.platform_host.snapshot().window_model();
        assert!(
            edited.runtime.workspace_document.has_unsaved_changes,
            "{} parameter commit should mark the project dirty",
            case.name
        );

        app.save_project();
        let saved = read_project_file(&project_path)
            .unwrap_or_else(|error| panic!("expected {} saved project read: {error}", case.name));
        assert_saved_unit_parameter(&saved, case);
        assert_saved_outlet_template(&saved, case);

        app.open_project(project_path.clone(), "project");
        let reopened = app.platform_host.snapshot().window_model();
        assert!(
            !reopened.runtime.workspace_document.has_unsaved_changes,
            "{} reopened project should be clean",
            case.name
        );

        app.dispatch_ui_command("run_panel.run_manual");
        let rerun = app.platform_host.snapshot().window_model();
        assert_eq!(
            rerun.runtime.control_state.run_status,
            rf_ui::RunStatus::Converged,
            "{} edited project should still converge after reopen",
            case.name
        );
        let solve_snapshot = rerun
            .runtime
            .latest_solve_snapshot
            .as_ref()
            .unwrap_or_else(|| panic!("expected {} solve snapshot", case.name));
        for outlet_stream_id in case.outlet_stream_ids {
            let outlet = solve_snapshot
                .streams
                .iter()
                .find(|stream| stream.stream_id == *outlet_stream_id)
                .unwrap_or_else(|| {
                    panic!(
                        "expected {} outlet stream result {outlet_stream_id}",
                        case.name
                    )
                });
            match case.field {
                UnitParameterShellField::OutletTemperatureK => {
                    assert_eq!(outlet.temperature_k, case.expected_value, "{}", case.name);
                }
                UnitParameterShellField::OutletPressurePa => {
                    assert_eq!(outlet.pressure_pa, case.expected_value, "{}", case.name);
                }
            }
        }

        let _ = std::fs::remove_file(project_path);
    }
}

#[test]
fn property_package_selection_saves_reopens_and_preferred_run_uses_selection() {
    let project_path = temporary_project_path("property-package-selection");
    let mut project = rf_store::parse_project_file_json(include_str!(
        "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .expect("expected heater flash project fixture");
    project
        .document
        .flowsheet
        .set_property_package_id(Some("legacy-package".to_string()))
        .expect("expected legacy package marker");
    write_project_file(&project_path, &project).expect("expected temp project write");
    let config = StudioRuntimeConfig {
        project_path: project_path.clone(),
        ..synced_workspace_config()
    };
    let mut app = ready_app_state(&config);

    let opened = app.platform_host.snapshot().window_model();
    assert_eq!(
        opened
            .runtime
            .workspace_document
            .property_package_id
            .as_deref(),
        Some("legacy-package")
    );
    let command_id = opened
        .runtime
        .workspace_document
        .property_package_choices
        .iter()
        .find(|choice| choice.package_id == "binary-hydrocarbon-lite-v1")
        .expect("expected built-in package choice")
        .command_id
        .clone();

    app.dispatch_ui_command(command_id);
    let selected = app.platform_host.snapshot().window_model();
    assert_eq!(
        selected
            .runtime
            .workspace_document
            .property_package_id
            .as_deref(),
        Some("binary-hydrocarbon-lite-v1")
    );
    assert!(selected.runtime.workspace_document.has_unsaved_changes);

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        saved.document.flowsheet.property_package_id(),
        Some("binary-hydrocarbon-lite-v1")
    );

    app.open_project(project_path.clone(), "project");
    let reopened = app.platform_host.snapshot().window_model();
    assert_eq!(
        reopened
            .runtime
            .workspace_document
            .property_package_id
            .as_deref(),
        Some("binary-hydrocarbon-lite-v1")
    );
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);

    app.dispatch_ui_command("run_panel.run_manual");
    let rerun = app.platform_host.snapshot().window_model();
    assert_eq!(
        rerun.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert!(
        rerun
            .runtime
            .control_state
            .latest_log_entry
            .as_ref()
            .map(|entry| entry
                .message
                .contains("with property package `binary-hydrocarbon-lite-v1`"))
            .unwrap_or(false),
        "Preferred run should use the flowsheet package selection"
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn project_component_selection_saves_reopens_and_feeds_composition_choices() {
    let project_path = temporary_project_path("project-component-selection");
    let mut project = rf_store::parse_project_file_json(include_str!(
        "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .expect("expected heater flash project fixture");
    project.document.flowsheet.components.clear();
    for stream in project.document.flowsheet.streams.values_mut() {
        stream.overall_mole_fractions.clear();
    }
    write_project_file(&project_path, &project).expect("expected temp project write");
    let config = StudioRuntimeConfig {
        project_path: project_path.clone(),
        ..synced_workspace_config()
    };
    let mut app = ready_app_state(&config);

    let opened = app.platform_host.snapshot().window_model();
    for component_id in ["methane", "ethane"] {
        let choice = opened
            .runtime
            .workspace_document
            .project_component_choices
            .iter()
            .find(|choice| choice.component_id == component_id)
            .unwrap_or_else(|| panic!("expected {component_id} component choice"));
        assert!(!choice.selected);
        app.dispatch_ui_command(choice.select_command_id.clone());
    }

    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    for component_id in ["methane", "ethane"] {
        let window = app.platform_host.snapshot().window_model();
        let detail = window
            .runtime
            .active_inspector_detail
            .as_ref()
            .expect("expected active stream inspector");
        let command_id = detail
            .property_composition_component_actions
            .iter()
            .find(|action| action.component_id == component_id)
            .unwrap_or_else(|| panic!("expected {component_id} composition action"))
            .action
            .command_id
            .clone();
        app.dispatch_inspector_composition_component_add(command_id);
    }

    let edited = app.platform_host.snapshot().window_model();
    assert!(edited.runtime.workspace_document.has_unsaved_changes);
    assert!(
        edited
            .runtime
            .workspace_document
            .project_component_choices
            .iter()
            .filter(|choice| choice.selected)
            .map(|choice| choice.component_id.as_str())
            .eq(["methane", "ethane"].into_iter())
    );

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert!(
        saved
            .document
            .flowsheet
            .components
            .contains_key(&rf_types::ComponentId::new("methane"))
    );
    assert!(
        saved
            .document
            .flowsheet
            .components
            .contains_key(&rf_types::ComponentId::new("ethane"))
    );
    let feed = &saved.document.flowsheet.streams[&rf_types::StreamId::new("stream-feed")];
    assert_eq!(
        feed.overall_mole_fractions[&rf_types::ComponentId::new("methane")],
        1.0
    );
    assert_eq!(
        feed.overall_mole_fractions[&rf_types::ComponentId::new("ethane")],
        0.0
    );

    app.open_project(project_path.clone(), "project");
    let reopened = app.platform_host.snapshot().window_model();
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);
    assert!(
        reopened
            .runtime
            .workspace_document
            .project_component_choices
            .iter()
            .filter(|choice| choice.selected)
            .map(|choice| choice.component_id.as_str())
            .eq(["methane", "ethane"].into_iter())
    );

    app.dispatch_ui_command("run_panel.run_manual");
    let rerun = app.platform_host.snapshot().window_model();
    assert_eq!(
        rerun.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn feed_composition_drafts_normalize_save_reopen_and_rerun_official_case() {
    let project_path = temporary_project_path("feed-composition-input");
    let project = rf_store::parse_project_file_json(include_str!(
        "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .expect("expected heater flash project fixture");
    write_project_file(&project_path, &project).expect("expected temp project write");
    let config = StudioRuntimeConfig {
        project_path: project_path.clone(),
        ..synced_workspace_config()
    };
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    for (component_id, raw_value) in [("methane", "0.2"), ("ethane", "0.6")] {
        app.dispatch_inspector_field_draft_update(
            radishflow_studio::inspector_draft_update_command_id(&format!(
                "stream:stream-feed:overall_mole_fraction:{component_id}"
            )),
            raw_value,
        );
    }

    let draft_window = app.platform_host.snapshot().window_model();
    let detail = draft_window
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected feed stream inspector");
    assert_eq!(
        detail
            .property_composition_summary
            .as_ref()
            .map(|summary| summary.status_label),
        Some("Draft")
    );
    let normalize_command_id = detail
        .property_composition_normalize_command_id
        .clone()
        .expect("expected composition normalize command");

    app.dispatch_inspector_composition_normalize(normalize_command_id);
    let normalized = app.platform_host.snapshot().window_model();
    assert!(normalized.runtime.workspace_document.has_unsaved_changes);

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    let saved_feed = &saved.document.flowsheet.streams[&rf_types::StreamId::new("stream-feed")];
    assert_close(
        saved_feed.overall_mole_fractions[&rf_types::ComponentId::new("methane")],
        0.25,
    );
    assert_close(
        saved_feed.overall_mole_fractions[&rf_types::ComponentId::new("ethane")],
        0.75,
    );

    app.open_project(project_path.clone(), "project");
    let reopened = app.platform_host.snapshot().window_model();
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);

    app.dispatch_ui_command("run_panel.run_manual");
    let rerun = app.platform_host.snapshot().window_model();
    assert_eq!(
        rerun.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    let feed_result = rerun
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected solve snapshot")
        .streams
        .iter()
        .find(|stream| stream.stream_id == "stream-feed")
        .expect("expected feed stream result");
    assert_stream_fraction(feed_result, "methane", 0.25);
    assert_stream_fraction(feed_result, "ethane", 0.75);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn selected_stream_reconnect_saves_reopens_and_reruns_official_case() {
    let project_path = temporary_project_path("selected-stream-reconnect");
    let project = feed_heater_flash_binary_hydrocarbon_project();
    write_project_file(&project_path, &project).expect("expected temp project write");
    let config = StudioRuntimeConfig {
        project_path: project_path.clone(),
        ..synced_workspace_config()
    };
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("inspector.focus_stream:stream-heated");
    app.dispatch_ui_command("canvas.disconnect_selected_stream_sink");
    let disconnected = app.platform_host.snapshot().window_model();
    assert!(disconnected.runtime.workspace_document.has_unsaved_changes);
    let reconnect_action = disconnected
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected stream inspector detail")
        .connection_actions
        .iter()
        .find(|action| action.command_id == "canvas.reconnect_selected_stream")
        .expect("expected reconnect action");
    assert!(reconnect_action.enabled);
    assert!(
        reconnect_action
            .hover_text
            .contains("only available material inlet `flash-1:inlet`"),
        "expected unique flash inlet reconnect detail, got {reconnect_action:?}"
    );

    app.dispatch_ui_command("canvas.reconnect_selected_stream");
    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved project read");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "heater-1", "outlet"),
        Some("stream-heated")
    );
    assert_eq!(
        stored_unit_port_stream_id(&saved, "flash-1", "inlet"),
        Some("stream-heated")
    );

    app.open_project(project_path.clone(), "project");
    let reopened = app.platform_host.snapshot().window_model();
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);

    app.dispatch_ui_command("run_panel.run_manual");
    let rerun = app.platform_host.snapshot().window_model();
    assert_eq!(
        rerun.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    let flash_step = rerun
        .runtime
        .latest_solve_snapshot
        .as_ref()
        .expect("expected solve snapshot")
        .steps
        .iter()
        .find(|step| step.unit_id == "flash-1")
        .expect("expected flash step");
    assert!(
        flash_step
            .consumed_stream_results
            .iter()
            .any(|stream| stream.stream_id == "stream-heated"),
        "Flash Drum should consume the reconnected heater outlet stream"
    );

    let _ = std::fs::remove_file(project_path);
}

#[derive(Debug, Clone, Copy)]
struct UnitParameterShellCase {
    name: &'static str,
    project_json: &'static str,
    unit_id: &'static str,
    draft_key: &'static str,
    raw_value: &'static str,
    expected_value: f64,
    outlet_stream_ids: &'static [&'static str],
    field: UnitParameterShellField,
}

#[derive(Debug, Clone, Copy)]
enum UnitParameterShellField {
    OutletTemperatureK,
    OutletPressurePa,
}

fn temporary_project_path(name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected current timestamp")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "radishflow-studio-shell-unit-parameter-{name}-{timestamp}.rfproj.json"
    ))
}

fn assert_stream_fraction(
    stream: &radishflow_studio::StudioGuiWindowStreamResultModel,
    component_id: &str,
    expected: f64,
) {
    let row = stream
        .composition_rows
        .iter()
        .find(|row| row.component_id == component_id)
        .unwrap_or_else(|| panic!("expected {component_id} composition row"));
    assert_close(row.fraction, expected);
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12,
        "expected {actual} to equal {expected}"
    );
}

fn assert_saved_unit_parameter(project: &StoredProjectFile, case: UnitParameterShellCase) {
    let unit = project
        .document
        .flowsheet
        .units
        .get(&UnitId::new(case.unit_id))
        .unwrap_or_else(|| panic!("expected {} unit", case.name));
    match case.field {
        UnitParameterShellField::OutletTemperatureK => {
            assert_eq!(
                unit.parameters.outlet_temperature_k,
                Some(case.expected_value),
                "{}",
                case.name
            );
        }
        UnitParameterShellField::OutletPressurePa => {
            assert_eq!(
                unit.parameters.outlet_pressure_pa,
                Some(case.expected_value),
                "{}",
                case.name
            );
        }
    }
}

fn assert_saved_outlet_template(project: &StoredProjectFile, case: UnitParameterShellCase) {
    for outlet_stream_id in case.outlet_stream_ids {
        let stream = project
            .document
            .flowsheet
            .streams
            .get(&StreamId::new(*outlet_stream_id))
            .unwrap_or_else(|| {
                panic!(
                    "expected {} outlet stream template {outlet_stream_id}",
                    case.name
                )
            });
        match case.field {
            UnitParameterShellField::OutletTemperatureK => {
                assert_eq!(stream.temperature_k, case.expected_value, "{}", case.name);
            }
            UnitParameterShellField::OutletPressurePa => {
                assert_eq!(stream.pressure_pa, case.expected_value, "{}", case.name);
            }
        }
    }
}

#[test]
fn open_project_from_picker_rebuilds_runtime_and_records_recent_project() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("picker-open");
    let base_app =
        ReadyAppState::from_config(&config, preferences_path.clone()).expect("expected base app");
    let target_project = base_app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.open_project_from_picker();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.title,
        "Feed Valve Flash Binary Hydrocarbon Example"
    );
    assert_eq!(
        app.project_open.path_input,
        target_project.display().to_string()
    );
    assert_eq!(
        app.project_open.recent_projects.first(),
        Some(&target_project)
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Info)
    );

    let _ = std::fs::remove_file(preferences_path);
}

#[test]
fn canceling_project_picker_keeps_current_workspace_active() {
    let config = synced_workspace_config();
    let original_title = ready_app_state(&config)
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .workspace_document
        .title;
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        test_preferences_path("picker-cancel"),
        Box::new(TestProjectFilePicker::new(None)),
    )
    .expect("expected app state");

    app.open_project_from_picker();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(window.runtime.workspace_document.title, original_title);
    assert!(app.project_open.recent_projects.is_empty());
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Project picker canceled")
    );
}

#[test]
fn create_blank_project_opens_untitled_blank_workspace_without_picker() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("blank-untitled");
    let unused_picker_target = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-unused-blank-picker-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(
            unused_picker_target.clone(),
        ))),
    )
    .expect("expected app state");

    app.create_blank_project();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(window.runtime.workspace_document.title, "Blank Project");
    assert_eq!(window.runtime.workspace_document.project_path, None);
    assert!(
        window.runtime.workspace_document.has_unsaved_changes,
        "untitled blank projects still need Save As even before semantic edits"
    );
    assert_eq!(
        window
            .runtime
            .workspace_document
            .property_package_id
            .as_deref(),
        None
    );
    assert!(
        window
            .runtime
            .workspace_document
            .project_component_choices
            .iter()
            .all(|choice| !choice.selected),
        "blank projects should expose component choices without preselecting them"
    );
    assert_eq!(window.runtime.workspace_document.unit_count, 0);
    assert_eq!(window.runtime.workspace_document.stream_count, 0);
    assert_eq!(app.project_open.path_input, "");
    assert!(
        app.project_open.recent_projects.is_empty(),
        "untitled blank projects should not enter recent projects before Save As"
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Blank project created")
    );
    assert!(
        !unused_picker_target.exists(),
        "creating an untitled blank project should not open the save picker or create its target"
    );

    let _ = std::fs::remove_file(preferences_path);
}

#[test]
fn saving_untitled_blank_project_uses_save_as_picker() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("blank-save-as");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-untitled-save-as-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.create_blank_project();
    app.save_project();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.project_path.as_deref(),
        Some(target_project.display().to_string().as_str())
    );
    assert!(!window.runtime.workspace_document.has_unsaved_changes);
    assert_eq!(
        app.project_open.path_input,
        target_project.display().to_string()
    );
    assert_eq!(
        app.project_open.recent_projects.first(),
        Some(&target_project)
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Project saved as")
    );
    assert!(read_project_file(&target_project).is_ok());

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn saving_untitled_blank_project_allows_creating_another_blank_project() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("blank-save-then-new");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-untitled-save-then-new-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.create_blank_project();
    app.save_project();
    assert!(
        !app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );

    app.create_blank_project();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(window.runtime.workspace_document.title, "Blank Project");
    assert_eq!(window.runtime.workspace_document.project_path, None);
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Blank project created")
    );

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn saving_untitled_blank_project_allows_opening_example_project() {
    let config = synced_workspace_config();
    let example_project = config.project_path.clone();
    let preferences_path = test_preferences_path("blank-save-then-example");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-untitled-save-then-example-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.create_blank_project();
    app.save_project();
    assert!(
        !app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );

    app.open_example_project(example_project.clone());

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.project_path.as_deref(),
        Some(example_project.display().to_string().as_str())
    );
    assert!(!window.runtime.workspace_document.has_unsaved_changes);
    assert!(
        app.project_open.pending_confirmation.is_none(),
        "clean saved workspace should not require discard confirmation before opening another project"
    );

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn create_blank_project_requires_confirmation_when_workspace_has_unsaved_changes() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        test_preferences_path("blank-unsaved"),
        Box::new(TestProjectFilePicker::new(None)),
    )
    .expect("expected app state");
    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);

    app.create_blank_project();

    let blocked_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blocked_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Warning)
    );
    assert!(app.project_open.pending_blank_project_confirmation);
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("未保存更改")
    );

    app.confirm_pending_blank_project();

    let blank_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blank_window.runtime.workspace_document.title,
        "Blank Project"
    );
    assert_eq!(blank_window.runtime.workspace_document.project_path, None);
    assert!(blank_window.runtime.workspace_document.has_unsaved_changes);
    assert!(app.project_open.pending_confirmation.is_none());
    assert!(!app.project_open.pending_blank_project_confirmation);
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Blank project created")
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn cancel_pending_blank_project_keeps_dirty_workspace_active() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);

    app.create_blank_project();
    app.cancel_pending_blank_project();

    let canceled_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        canceled_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert!(
        canceled_window
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    assert!(!app.project_open.pending_blank_project_confirmation);
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("已取消新建项目")
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn closing_dirty_workspace_requires_explicit_confirmation() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);

    assert!(!app.close_current_window_for_viewport_request());

    let blocked_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blocked_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert_eq!(app.logical_window_count(), 1);
    assert!(app.project_open.pending_close_window_confirmation.is_some());
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        None,
        "close confirmation should be rendered as a dialog instead of a top notice"
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn cancel_pending_close_keeps_dirty_workspace_active() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);
    assert!(!app.close_current_window_for_viewport_request());

    app.cancel_pending_close_window();

    let canceled_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        canceled_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert!(
        canceled_window
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    assert!(app.project_open.pending_close_window_confirmation.is_none());
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("已取消关闭")
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn discard_pending_close_closes_dirty_workspace() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.accept_focused");
    assert!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    assert!(!app.close_current_window_for_viewport_request());

    assert!(app.confirm_pending_close_window());

    assert_eq!(app.logical_window_count(), 0);
    assert!(app.project_open.pending_close_window_confirmation.is_none());

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn save_pending_close_writes_existing_dirty_project_before_close() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.accept_focused");
    assert!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    assert!(!app.close_current_window_for_viewport_request());

    assert!(app.save_pending_close_window());

    assert_eq!(app.logical_window_count(), 0);
    assert!(app.project_open.pending_close_window_confirmation.is_none());
    let saved = read_project_file(&project_path).expect("expected saved project");
    assert_eq!(
        stored_unit_port_stream_id(&saved, "flash-1", "vapor"),
        Some("stream-flash-1-vapor")
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn save_project_as_from_picker_writes_project_and_records_recent_project() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("save-as-picker");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-save-as-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.save_project_as_from_picker();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.project_path.as_deref(),
        Some(target_project.display().to_string().as_str())
    );
    assert!(!window.runtime.workspace_document.has_unsaved_changes);
    assert_eq!(
        app.project_open.path_input,
        target_project.display().to_string()
    );
    assert_eq!(
        app.project_open.recent_projects.first(),
        Some(&target_project)
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Project saved as")
    );
    assert!(read_project_file(&target_project).is_ok());

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn save_project_as_from_picker_requires_confirmation_before_overwrite() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("save-as-overwrite");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-save-as-overwrite-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    fs::write(&target_project, "existing project placeholder").expect("expected target seed");
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.save_project_as_from_picker();

    assert_eq!(
        fs::read_to_string(&target_project).expect("expected target read"),
        "existing project placeholder"
    );
    assert_eq!(
        app.project_open.pending_save_as_overwrite.as_deref(),
        Some(target_project.as_path())
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Confirm overwrite")
    );

    app.confirm_pending_save_as_overwrite();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.project_path.as_deref(),
        Some(target_project.display().to_string().as_str())
    );
    assert!(app.project_open.pending_save_as_overwrite.is_none());
    assert!(read_project_file(&target_project).is_ok());

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn failed_confirmed_save_as_keeps_workspace_state_and_retry_target() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("save-as-overwrite-failure");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-save-as-overwrite-failure-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    fs::create_dir_all(&target_project).expect("expected directory target");
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");
    let original_window = app.platform_host.snapshot().window_model();
    let original_document = original_window.runtime.workspace_document.clone();

    app.save_project_as_from_picker();
    assert_eq!(
        app.project_open.pending_save_as_overwrite.as_deref(),
        Some(target_project.as_path())
    );

    app.confirm_pending_save_as_overwrite();

    let failed_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        failed_window.runtime.workspace_document.project_path,
        original_document.project_path
    );
    assert_eq!(
        failed_window.runtime.workspace_document.last_saved_revision,
        original_document.last_saved_revision
    );
    assert_eq!(
        failed_window.runtime.workspace_document.has_unsaved_changes,
        original_document.has_unsaved_changes
    );
    assert!(app.project_open.recent_projects.is_empty());
    assert_eq!(
        app.project_open.pending_save_as_overwrite.as_deref(),
        Some(target_project.as_path()),
        "failed overwrite save-as should keep the retry/cancel target visible"
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Save As failed")
    );
    assert!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.detail.contains("Current workspace remains open"))
            .unwrap_or(false)
    );
    assert!(
        failed_window
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("save as failed"))
    );
    assert!(target_project.is_dir());

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_dir_all(target_project);
}

#[test]
fn cancel_pending_save_as_overwrite_keeps_existing_file() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("save-as-overwrite-cancel");
    let target_project = std::env::temp_dir().join(format!(
        "radishflow-studio-shell-save-as-overwrite-cancel-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos()
    ));
    fs::write(&target_project, "existing project placeholder").expect("expected target seed");
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        preferences_path.clone(),
        Box::new(TestProjectFilePicker::new(Some(target_project.clone()))),
    )
    .expect("expected app state");

    app.save_project_as_from_picker();
    app.cancel_pending_save_as_overwrite();

    assert_eq!(
        fs::read_to_string(&target_project).expect("expected target read"),
        "existing project placeholder"
    );
    assert!(app.project_open.pending_save_as_overwrite.is_none());
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Save As canceled")
    );

    let _ = std::fs::remove_file(preferences_path);
    let _ = std::fs::remove_file(target_project);
}

#[test]
fn successful_project_opens_keep_recent_projects_deduped_and_ordered() {
    let mut app = ready_app_state(&synced_workspace_config());
    let examples = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects;
    let valve_project = examples
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    let cooler_project = examples
        .iter()
        .find(|example| example.id == "feed-cooler-flash")
        .expect("expected feed cooler example")
        .project_path
        .clone();

    app.open_project(valve_project.clone(), "project");
    app.open_project(cooler_project.clone(), "project");
    app.open_recent_project(valve_project.clone());

    assert_eq!(
        app.project_open.recent_projects,
        vec![valve_project, cooler_project]
    );
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .workspace_document
            .title,
        "Feed Valve Flash Binary Hydrocarbon Example"
    );
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.detail.as_str())
            .unwrap_or(""),
        format!(
            "已打开最近项目: {}",
            app.project_open
                .recent_projects
                .first()
                .expect("expected recent project")
                .display()
        )
    );
}

#[test]
fn successful_project_opens_persist_recent_projects_for_next_shell_start() {
    let config = synced_workspace_config();
    let preferences_path = test_preferences_path("recent-projects");
    let mut app =
        ReadyAppState::from_config(&config, preferences_path.clone()).expect("expected app state");
    let examples = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects;
    let valve_project = examples
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    let cooler_project = examples
        .iter()
        .find(|example| example.id == "feed-cooler-flash")
        .expect("expected feed cooler example")
        .project_path
        .clone();

    app.open_project(valve_project.clone(), "project");
    app.open_project(cooler_project.clone(), "project");

    let restarted =
        ReadyAppState::from_config(&config, preferences_path.clone()).expect("expected restart");

    assert_eq!(
        restarted.project_open.recent_projects,
        vec![cooler_project, valve_project]
    );

    let _ = std::fs::remove_file(preferences_path);
}

#[test]
fn open_project_failure_keeps_current_runtime_and_surfaces_error_notice() {
    let mut app = ready_app_state(&synced_workspace_config());
    let original_window = app.platform_host.snapshot().window_model();
    let missing_project = std::env::temp_dir().join("radishflow-missing-project.rfproj.json");

    app.project_open.path_input = missing_project.display().to_string();
    app.open_project_from_input();

    let window = app.platform_host.snapshot().window_model();
    assert_eq!(
        window.runtime.workspace_document.title,
        original_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Error)
    );
    assert!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice
                .detail
                .contains("radishflow-missing-project.rfproj.json"))
            .unwrap_or(false)
    );
    assert!(
        window
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("open project failed"))
    );
    assert!(
        app.project_open.recent_projects.is_empty(),
        "failed project opens should not enter recent projects"
    );
}

#[test]
fn open_project_from_input_requires_confirmation_when_workspace_has_unsaved_changes() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);
    assert_eq!(
        dirty_window.runtime.workspace_document.last_saved_revision,
        Some(0)
    );
    assert_eq!(dirty_window.runtime.workspace_document.revision, 1);

    app.project_open.path_input = target_project.display().to_string();
    app.open_project_from_input();

    let blocked_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blocked_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Warning)
    );
    assert!(app.project_open.pending_confirmation.is_some());

    app.confirm_pending_project_open();
    let opened_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        opened_window.runtime.workspace_document.title,
        "Feed Valve Flash Binary Hydrocarbon Example"
    );
    assert!(!opened_window.runtime.workspace_document.has_unsaved_changes);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn open_project_from_picker_requires_confirmation_when_workspace_has_unsaved_changes() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let target_project = ready_app_state(&synced_workspace_config())
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    let mut app = ReadyAppState::from_config_with_project_file_picker(
        &config,
        test_preferences_path("picker-unsaved"),
        Box::new(TestProjectFilePicker::new(Some(target_project))),
    )
    .expect("expected app state");

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);

    app.open_project_from_picker();

    let blocked_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blocked_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open
            .pending_confirmation
            .as_ref()
            .map(|request| request.source_label.as_str()),
        Some("project picker")
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Warning)
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn open_recent_project_requires_confirmation_when_workspace_has_unsaved_changes() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();
    app.project_open
        .record_recent_project(target_project.clone());

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    assert!(dirty_window.runtime.workspace_document.has_unsaved_changes);

    app.open_recent_project(target_project);

    let blocked_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        blocked_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert_eq!(
        app.project_open
            .pending_confirmation
            .as_ref()
            .map(|request| request.source_label.as_str()),
        Some("recent project")
    );
    assert_eq!(
        app.project_open.notice.as_ref().map(|notice| notice.level),
        Some(ProjectOpenNoticeLevel::Warning)
    );

    app.confirm_pending_project_open();
    let opened_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        opened_window.runtime.workspace_document.title,
        "Feed Valve Flash Binary Hydrocarbon Example"
    );
    assert!(!opened_window.runtime.workspace_document.has_unsaved_changes);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn cancel_pending_project_open_keeps_dirty_workspace_active() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    let target_project = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .example_projects
        .iter()
        .find(|example| example.id == "feed-valve-flash")
        .expect("expected feed valve example")
        .project_path
        .clone();

    app.dispatch_ui_command("canvas.accept_focused");
    let dirty_window = app.platform_host.snapshot().window_model();
    app.project_open.path_input = target_project.display().to_string();
    app.open_project_from_input();

    app.cancel_pending_project_open();

    let canceled_window = app.platform_host.snapshot().window_model();
    assert_eq!(
        canceled_window.runtime.workspace_document.title,
        dirty_window.runtime.workspace_document.title
    );
    assert!(
        canceled_window
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    assert!(app.project_open.pending_confirmation.is_none());
    assert_eq!(
        app.project_open
            .notice
            .as_ref()
            .map(|notice| notice.title.as_str()),
        Some("Project open canceled")
    );

    let _ = std::fs::remove_file(project_path);
}
