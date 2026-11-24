use super::*;
use rf_types::units::{DisplayUnitSet, MeasurementUnit, QuantityKind};

fn mixer_project() -> StoredProjectFile {
    serde_json::from_str(include_str!(
        "../../../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap()
}

fn solved(project: &StoredProjectFile) -> (ReadyAppState, PathBuf) {
    let (config, path) = blank_workspace_config();
    write_project_file(&path, project).unwrap();
    let mut app = ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("run_panel.run_manual");
    (app, path)
}

fn comparison_texts(app: &mut ReadyAppState, window: &StudioGuiWindowModel) -> Vec<String> {
    let ctx = egui::Context::default();
    ctx.memory_mut(|memory| memory.set_everything_is_visible(true));
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            assert!(app.render_results_comparison(ui, window));
        });
    });
    collect_output_texts(&output)
}

#[test]
fn comparison_reads_mixer_ports_from_the_workspace_that_owns_the_current_result() {
    let mut project = mixer_project();
    let mut previous = None;
    for (base_port, peer_port) in [("inlet_a", "inlet_b"), ("inlet_b", "inlet_a")] {
        let mixer = project
            .document
            .flowsheet
            .units
            .get_mut(&UnitId::new("mixer-1"))
            .unwrap();
        for port in &mut mixer.ports {
            if port.name == base_port {
                port.stream_id = Some(StreamId::new("stream-feed-a"));
            } else if port.name == peer_port {
                port.stream_id = Some(StreamId::new("stream-feed-b"));
            }
        }
        let (mut app, path) = solved(&project);
        let window = app.platform_host.snapshot().window_model();
        let snapshot = window.runtime.latest_solve_snapshot.as_ref().unwrap();
        assert_eq!(
            snapshot_stream(snapshot, "stream-mix-out").molar_flow_text,
            "5 mol/s"
        );
        let review = snapshot
            .result_inspector_with_comparison(Some("stream-feed-a"), Some("stream-feed-b"))
            .comparison
            .unwrap();
        assert_eq!(
            (review.base_label.as_str(), review.compared_label.as_str()),
            ("Feed A", "Feed B")
        );
        assert_eq!(
            review.base_ports.source.as_deref(),
            Some("Feed A · feed-a:outlet")
        );
        assert_eq!(
            review.compared_ports.source.as_deref(),
            Some("Feed B · feed-b:outlet")
        );
        assert_eq!(
            review.base_ports.sinks,
            [format!("Mixer · mixer-1:{base_port}")]
        );
        assert_eq!(
            review.compared_ports.sinks,
            [format!("Mixer · mixer-1:{peer_port}")]
        );
        assert_eq!(review.summary_rows[2].delta_text, "+1 mol/s");
        app.result_inspector
            .select_stream(&snapshot.snapshot_id, "stream-feed-a");
        app.result_inspector
            .select_comparison_stream(&snapshot.snapshot_id, "stream-feed-b");
        let cells = comparison_texts(&mut app, &window);
        assert!(
            cells.contains(&review.base_ports.sinks[0])
                && cells.contains(&review.compared_ports.sinks[0])
        );
        if let Some(old) = previous.as_ref() {
            let cells = comparison_texts(&mut app, old);
            assert!(cells.iter().any(|cell| cell.contains("Run again")));
            assert!(!cells.iter().any(|cell| cell.contains("mixer-1:inlet")));
        }
        app.dispatch_ui_command("inspector.focus_stream:stream-feed-a");
        app.dispatch_ui_command("canvas.disconnect_selected_stream_sink");
        let edited = app.platform_host.snapshot().window_model();
        assert!(edited.runtime.latest_solve_snapshot.is_none());
        assert!(edited.runtime.stale_solve_snapshot.is_some());
        let cells = comparison_texts(&mut app, &edited);
        assert!(!cells.iter().any(|cell| cell.contains("mixer-1:inlet")));
        previous = Some(window);
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn comparison_table_copy_and_export_preserve_zero_flow_and_absent_flash_enthalpy() {
    let mut project = mixer_project();
    project
        .document
        .flowsheet
        .units
        .get_mut(&UnitId::new("flash-1"))
        .unwrap()
        .parameters
        .outlet_temperature_k = Some(298.0);
    let (mut app, path) = solved(&project);
    let current = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap();
    let mut converted = DisplayUnitSet::engineering();
    converted
        .set_unit(
            QuantityKind::MolarEnthalpy,
            MeasurementUnit::KilojoulePerMole,
        )
        .unwrap();
    for (units, zero, flowing, enthalpy_unit) in [
        (DisplayUnitSet::si(), "0 mol/s", "5 mol/s", " J/mol"),
        (converted, "0 kmol/h", "18 kmol/h", " kJ/mol"),
    ] {
        app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(units))
            .unwrap();
        let snapshot = app
            .platform_host
            .snapshot()
            .window_model()
            .runtime
            .latest_solve_snapshot
            .unwrap();
        assert_eq!(snapshot.result_guard, current.result_guard);
        let review = snapshot
            .result_inspector_with_comparison(Some("stream-vapor"), Some("stream-liquid"))
            .comparison
            .unwrap();
        let flow = &review.summary_rows[2];
        assert_eq!((&*flow.base_value, &*flow.compared_value), (zero, flowing));
        let enthalpy = &review.summary_rows[3];
        assert_eq!((&*enthalpy.base_value, &*enthalpy.delta_text), ("-", "-"));
        assert!(enthalpy.compared_value.ends_with(enthalpy_unit));
        let table = render_bottom_results_table_texts(&mut app);
        let row = table
            .iter()
            .position(|text| text == "Vapor Outlet")
            .unwrap();
        assert_eq!(table[row + 3], zero);
        assert_eq!(table[row + 4], "-");

        let report = path.with_extension("txt");
        app.export_solve_snapshot_to_path(&current, report.clone());
        let exported = fs::read_to_string(&report).unwrap();
        let fields: Vec<_> = exported
            .lines()
            .find(|line| line.starts_with("stream-vapor\t"))
            .unwrap()
            .split('\t')
            .collect();
        assert_eq!(&fields[4..6], &[zero, "-"]);
        let output = egui::Context::default().run(egui::RawInput::default(), |ctx| {
            app.copy_solve_snapshot_to_clipboard(ctx, &current);
        });
        assert!(output.platform_output.commands.iter().any(
            |command| matches!(command, egui::OutputCommand::CopyText(text) if text == &exported)
        ));
        fs::remove_file(report).unwrap();
    }
    fs::remove_file(path).unwrap();
}
