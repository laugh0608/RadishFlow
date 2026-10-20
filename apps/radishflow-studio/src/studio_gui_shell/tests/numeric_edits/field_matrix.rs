use super::*;
use rf_types::units::{DisplayUnitSet, MeasurementUnit};

#[derive(Clone, Copy)]
enum Quantity {
    Temperature,
    Pressure,
    Flow,
}

impl Quantity {
    fn engineering_unit(self) -> MeasurementUnit {
        match self {
            Self::Temperature => MeasurementUnit::Celsius,
            Self::Pressure => MeasurementUnit::Bar,
            Self::Flow => MeasurementUnit::KilomolePerHour,
        }
    }

    fn stream_value(self, stream: &rf_model::MaterialStreamState) -> f64 {
        match self {
            Self::Temperature => stream.temperature_k,
            Self::Pressure => stream.pressure_pa,
            Self::Flow => stream.total_molar_flow_mol_s,
        }
    }
}

#[test]
fn non_si_field_matrix_commits_undoes_saves_reopens_and_reruns_in_both_locales() {
    use Quantity::{Flow, Pressure, Temperature};
    // Independent SI expectations cover every currently editable numeric field role.
    let cases = [
        (
            "heater",
            "stream:stream-feed:temperature_k",
            "36.85",
            310.0,
            Temperature,
            &["stream-feed"][..],
        ),
        (
            "heater",
            "stream:stream-feed:pressure_pa",
            "1.3",
            130_000.0,
            Pressure,
            &["stream-feed"],
        ),
        (
            "heater",
            "stream:stream-feed:total_molar_flow_mol_s",
            "21.6",
            6.0,
            Flow,
            &["stream-feed"],
        ),
        (
            "heater",
            "unit:feed-1:outlet_temperature_k",
            "36.85",
            310.0,
            Temperature,
            &["stream-feed"],
        ),
        (
            "heater",
            "unit:feed-1:outlet_pressure_pa",
            "1.3",
            130_000.0,
            Pressure,
            &["stream-feed"],
        ),
        (
            "heater",
            "unit:heater-1:outlet_temperature_k",
            "88.1",
            361.25,
            Temperature,
            &["stream-heated"],
        ),
        (
            "heater",
            "unit:heater-1:outlet_pressure_pa",
            "0.9",
            90_000.0,
            Pressure,
            &["stream-heated"],
        ),
        (
            "cooler",
            "unit:cooler-1:outlet_temperature_k",
            "-10",
            263.15,
            Temperature,
            &["stream-cooled"],
        ),
        (
            "cooler",
            "unit:cooler-1:outlet_pressure_pa",
            "6.4",
            640_000.0,
            Pressure,
            &["stream-cooled"],
        ),
        (
            "mixer",
            "unit:mixer-1:outlet_pressure_pa",
            "6.4",
            640_000.0,
            Pressure,
            &["stream-mix-out"],
        ),
        (
            "valve",
            "unit:valve-1:outlet_pressure_pa",
            "6.4",
            640_000.0,
            Pressure,
            &["stream-throttled"],
        ),
        (
            "heater",
            "unit:flash-1:outlet_temperature_k",
            "61.85",
            335.0,
            Temperature,
            &["stream-liquid", "stream-vapor"],
        ),
        (
            "heater",
            "unit:flash-1:outlet_pressure_pa",
            "6.8",
            680_000.0,
            Pressure,
            &["stream-liquid", "stream-vapor"],
        ),
    ];
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for (fixture, key, text, expected, quantity, outlets) in cases {
            let preferences = test_preferences_path("numeric-field-matrix");
            let root = preferences.parent().unwrap();
            let path = root.join("case.rfproj.json");
            let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/flowsheets")
                .join(format!(
                    "feed-{fixture}-flash-binary-hydrocarbon.rfproj.json"
                ));
            let mut project = read_project_file(fixture_path).unwrap();
            project.schema_version = 2;
            project.presentation.display_units = DisplayUnitSet::engineering();
            write_project_file(&path, &project).unwrap();
            let config = StudioRuntimeConfig {
                project_path: path.clone(),
                ..synced_workspace_config()
            };
            let mut app = ReadyAppState::from_config(&config, preferences.clone()).unwrap();
            app.locale = locale;
            app.dispatch_ui_command("run_panel.run_manual");
            let mut identity = key.split(':');
            let object_kind = identity.next().unwrap();
            let object_id = identity.next().unwrap();
            app.dispatch_ui_command(format!("inspector.focus_{object_kind}:{object_id}"));
            let before = app.platform_host.snapshot();
            assert!(before.runtime.latest_solve_snapshot.is_some(), "{key}");
            let ctx = egui::Context::default();
            let id = numeric_frame(&mut app, &ctx, key, true, vec![]);
            let field = numeric_field(&app, key);
            assert_eq!(field.input_unit, quantity.engineering_unit(), "{key}");
            select_all(&ctx, id, &field.text);
            numeric_frame(
                &mut app,
                &ctx,
                key,
                false,
                vec![egui::Event::Paste(text.into())],
            );
            let draft = app.platform_host.snapshot();
            assert_eq!(
                draft.runtime.workspace_document.revision,
                before.runtime.workspace_document.revision,
                "{key}"
            );
            assert_eq!(
                draft.runtime.latest_solve_snapshot, before.runtime.latest_solve_snapshot,
                "{key}"
            );
            assert!(numeric_field(&app, key).can_apply, "{key}");
            numeric_frame(
                &mut app,
                &ctx,
                key,
                false,
                vec![key_event(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            let committed = app.platform_host.document().clone();
            assert_eq!(committed.revision, project.document.revision + 1, "{key}");
            assert!(numeric_field(&app, key).generation.is_none(), "{key}");
            assert!(
                app.platform_host
                    .snapshot()
                    .runtime
                    .latest_solve_snapshot
                    .is_none(),
                "{key}"
            );
            for stream in outlets {
                assert_si(
                    quantity.stream_value(&committed.flowsheet.streams[&StreamId::new(*stream)]),
                    expected,
                    key,
                );
            }
            if object_kind == "unit" {
                let parameters = &committed.flowsheet.units[&UnitId::new(object_id)].parameters;
                let value = match quantity {
                    Temperature => parameters.outlet_temperature_k.unwrap(),
                    Pressure => parameters.outlet_pressure_pa.unwrap(),
                    Flow => unreachable!("units expose no independent molar-flow parameter"),
                };
                assert_si(value, expected, key);
            }
            app.dispatch_ui_command("edit.undo");
            assert_eq!(
                app.platform_host.document().flowsheet,
                project.document.flowsheet,
                "{key}"
            );
            app.dispatch_ui_command("edit.redo");
            assert_eq!(
                app.platform_host.document().flowsheet,
                committed.flowsheet,
                "{key}"
            );
            app.save_project();
            let saved = read_project_file(&path).unwrap();
            assert_eq!(saved.document.flowsheet, committed.flowsheet, "{key}");
            assert_eq!(
                saved.presentation.display_units,
                DisplayUnitSet::engineering()
            );
            app.open_project(path, "numeric field matrix");
            assert!(
                !app.platform_host
                    .snapshot()
                    .runtime
                    .workspace_document
                    .has_unsaved_changes,
                "{key}"
            );
            app.dispatch_ui_command("run_panel.run_manual");
            let rerun = app.platform_host.snapshot();
            assert_eq!(
                rerun.runtime.control_state.run_status,
                rf_ui::RunStatus::Converged,
                "{key}"
            );
            let result = rerun.runtime.latest_solve_snapshot.unwrap();
            for outlet in outlets {
                let stream = result
                    .streams
                    .iter()
                    .find(|stream| stream.stream_id.as_str() == *outlet)
                    .unwrap();
                let value = match quantity {
                    Temperature => stream.temperature_k,
                    Pressure => stream.pressure_pa,
                    Flow => stream.total_molar_flow_mol_s,
                };
                assert_si(value, expected, key);
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
}

fn assert_si(actual: f64, expected: f64, field: &str) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{field}: {actual} != {expected}"
    );
}
