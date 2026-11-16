use super::*;
use crate::studio_gui_shell::tests::{ready_app_state, synced_workspace_config};
use egui::accesskit::{Action, ActionRequest, TreeUpdate};
use rf_types::{StreamId, units::DisplayUnitSet};
use rf_ui::ProjectPresentationCommand;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_PROJECT: AtomicUsize = AtomicUsize::new(0);
const EXAMPLE: &str = "feed-heater-flash-binary-hydrocarbon.rfproj.json";
const VALVE_EXAMPLE: &str = "feed-valve-flash-binary-hydrocarbon.rfproj.json";

fn example(name: &str) -> (ReadyAppState, PathBuf) {
    let path = std::env::temp_dir().join(format!(
        "rf-v2-stream-layer-{}-{}.rfproj.json",
        std::process::id(),
        NEXT_PROJECT.fetch_add(1, Ordering::Relaxed)
    ));
    let original = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets")
        .join(name);
    std::fs::copy(original, &path).unwrap();
    let app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    (app, path)
}

fn current_layer(app: &ReadyAppState) -> CanvasStreamColoring<'_> {
    CanvasStreamLayer::MolarFlow
        .project(&app.platform_host.modeling_inputs().workspace)
        .unwrap()
}

fn temperature_layer(app: &ReadyAppState) -> CanvasStreamColoring<'_> {
    CanvasStreamLayer::Temperature
        .project(&app.platform_host.modeling_inputs().workspace)
        .unwrap()
}

fn absolute_pressure_layer(app: &ReadyAppState) -> CanvasStreamColoring<'_> {
    CanvasStreamLayer::AbsolutePressure
        .project(&app.platform_host.modeling_inputs().workspace)
        .unwrap()
}

fn commit_unit_field(app: &mut ReadyAppState, unit: &str, field: &str, value: &str) {
    app.dispatch_ui_command(format!("inspector.focus_unit:{unit}"));
    let field = format!("unit:{unit}:{field}");
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(&field),
        value,
    );
    app.dispatch_inspector_field_draft_commit(
        radishflow_studio::inspector_draft_commit_command_id(&field),
    );
}

#[test]
fn canvas_molar_flow_tracks_the_current_run_and_scales_before_unit_conversion() {
    let (mut app, path) = example(EXAMPLE);
    assert_eq!(
        app.canvas_stream_layer,
        CanvasStreamLayer::ConnectionDirection
    );
    assert!(
        app.canvas_stream_layer
            .project(&app.platform_host.modeling_inputs().workspace)
            .is_none()
    );
    assert!(matches!(
        current_layer(&app).result,
        StreamColorResult::Unavailable(ResultStatus::Missing)
    ));

    app.dispatch_ui_command("run_panel.run_manual");
    let layer = current_layer(&app);
    assert!(matches!(
        layer.result,
        StreamColorResult::Current {
            scale: Some(StreamColorScale {
                minimum: 0.0,
                maximum: 5.0
            }),
            sequence: 1,
            revision: 0,
            ..
        }
    ));
    assert_eq!(layer.stroke("stream-heated").color, HIGH);
    assert_eq!(layer.stroke("stream-liquid").color, LOW);
    assert!(!layer.stroke("stream-liquid").dashed);
    assert_eq!(
        layer
            .stream_readout("stream-liquid", StudioShellLocale::En)
            .as_deref(),
        Some("0 mol/s")
    );
    assert_eq!(
        layer
            .stream_readout("stream-heated", StudioShellLocale::En)
            .as_deref(),
        Some("5 mol/s")
    );

    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    let layer = current_layer(&app);
    assert!(matches!(
        layer.result,
        StreamColorResult::Current {
            scale: Some(StreamColorScale {
                minimum: 0.0,
                maximum: 5.0
            }),
            sequence: 1,
            revision: 0,
            ..
        }
    ));
    assert_eq!(layer.stroke("stream-heated").color, HIGH);
    assert_eq!(layer.stroke("stream-liquid").color, LOW);
    assert_eq!(
        layer
            .stream_readout("stream-heated", StudioShellLocale::En)
            .as_deref(),
        Some("18 kmol/h")
    );

    app.dispatch_ui_command("run_panel.run_manual");
    assert!(matches!(
        current_layer(&app).result,
        StreamColorResult::Current {
            sequence: 2,
            revision: 0,
            ..
        }
    ));

    app.screen = StudioShellScreen::Workbench;
    app.select_canvas_stream("inspector.focus_stream:stream-heated");
    app.dispatch_ui_command("canvas.delete_selected_stream");
    let stale = current_layer(&app);
    assert!(matches!(
        stale.result,
        StreamColorResult::Unavailable(ResultStatus::Stale)
    ));
    assert_eq!(stale.stroke("stream-liquid").color, NEUTRAL);
    assert!(!stale.stroke("stream-liquid").dashed);
    assert_eq!(
        stale.stream_readout("stream-liquid", StudioShellLocale::En),
        None
    );

    app.canvas_stream_layer = CanvasStreamLayer::MolarFlow;
    app.open_project(path.clone(), "molar flow same-file regression");
    assert_eq!(
        app.canvas_stream_layer,
        CanvasStreamLayer::ConnectionDirection
    );
    assert!(matches!(
        current_layer(&app).result,
        StreamColorResult::Unavailable(ResultStatus::Missing)
    ));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn canvas_molar_flow_excludes_missing_and_invalid_values_from_the_range() {
    let (mut app, path) = example(EXAMPLE);
    app.dispatch_ui_command("run_panel.run_manual");
    let mut workspace = app.platform_host.modeling_inputs().workspace.clone();
    let mut snapshot = rf_ui::latest_snapshot(&workspace).unwrap().clone();
    snapshot
        .streams
        .retain(|stream| stream.stream_id.as_str() != "stream-feed");
    for stream in &mut snapshot.streams {
        stream.total_molar_flow_mol_s = match stream.stream_id.as_str() {
            "stream-heated" => f64::INFINITY,
            "stream-vapor" => -5.0,
            "stream-liquid" => 0.0,
            id => panic!("Unexpected stream {id}"),
        };
    }
    workspace.store_snapshot(snapshot, 4);
    let layer = CanvasStreamLayer::MolarFlow.project(&workspace).unwrap();
    assert!(matches!(
        layer.result,
        StreamColorResult::Current {
            scale: Some(StreamColorScale {
                minimum: 0.0,
                maximum: 0.0
            }),
            ..
        }
    ));
    for stream in ["stream-feed", "stream-heated", "stream-vapor"] {
        assert_eq!(layer.stroke(stream).color, NEUTRAL);
        assert!(layer.stroke(stream).dashed);
        assert_eq!(
            layer
                .stream_readout(stream, StudioShellLocale::En)
                .as_deref(),
            Some("No flow value")
        );
    }
    assert_eq!(layer.stroke("stream-liquid").color, LOW);
    assert!(!layer.stroke("stream-liquid").dashed);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn temperature_colors_actual_results_and_isothermal_reruns_in_canonical_units() {
    let (mut app, path) = example(EXAMPLE);
    assert!(matches!(
        temperature_layer(&app).result,
        StreamColorResult::Unavailable(ResultStatus::Missing)
    ));
    app.dispatch_ui_command("run_panel.run_manual");
    let layer = temperature_layer(&app);
    assert!(matches!(
        layer.result,
        StreamColorResult::Current {
            scale: Some(StreamColorScale {
                minimum: 300.0,
                maximum: 345.0
            }),
            sequence: 1,
            revision: 0,
            ..
        }
    ));
    assert_eq!(layer.stroke("stream-feed").color, LOW);
    assert_eq!(layer.stroke("stream-heated").color, HIGH);
    assert_eq!(layer.stroke("stream-liquid").color, NEUTRAL);
    assert!(layer.stroke("stream-liquid").dashed);
    assert_eq!(
        layer
            .stream_readout("stream-liquid", StudioShellLocale::En)
            .as_deref(),
        Some("Zero flow")
    );
    assert_eq!(
        current_layer(&app)
            .stream_readout("stream-liquid", StudioShellLocale::En)
            .as_deref(),
        Some("0 mol/s")
    );

    app.apply_presentation_command(ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    let layer = temperature_layer(&app);
    assert!(matches!(
        layer.result,
        StreamColorResult::Current {
            sequence: 1,
            revision: 0,
            ..
        }
    ));
    for (id, color, display) in [
        ("stream-feed", LOW, "26.85 °C"),
        ("stream-heated", HIGH, "71.85 °C"),
    ] {
        assert_eq!(layer.stroke(id).color, color);
        assert_eq!(
            layer.stream_readout(id, StudioShellLocale::En).as_deref(),
            Some(display)
        );
    }

    app.apply_presentation_command(ProjectPresentationCommand::Apply(DisplayUnitSet::si()))
        .unwrap();
    for unit in ["heater-1", "flash-1"] {
        commit_unit_field(&mut app, unit, "outlet_temperature_k", "300");
        let stale = temperature_layer(&app);
        assert!(matches!(
            stale.result,
            StreamColorResult::Unavailable(ResultStatus::Stale)
        ));
        assert_eq!(stale.stroke("stream-heated").color, NEUTRAL);
        assert!(!stale.stroke("stream-heated").dashed);
        assert_eq!(
            stale.stream_readout("stream-heated", StudioShellLocale::En),
            None
        );
        app.dispatch_ui_command("run_panel.run_manual");
        if unit == "heater-1" {
            let layer = temperature_layer(&app);
            assert_eq!(layer.stroke("stream-heated").color, LOW);
            assert_eq!(layer.stroke("stream-vapor").color, HIGH);
        }
    }

    for sequence in [3, 4] {
        let layer = temperature_layer(&app);
        assert!(matches!(
            layer.result,
            StreamColorResult::Current {
                scale: Some(StreamColorScale { minimum: 300.0, maximum: 300.0 }),
                sequence: actual,
                revision: 2,
                ..
            } if actual == sequence
        ));
        for id in ["stream-feed", "stream-heated", "stream-vapor"] {
            assert_eq!(layer.stroke(id).color, scale_color(0.5));
            assert!(!layer.stroke(id).dashed);
        }
        assert!(layer.stroke("stream-liquid").dashed);
        if sequence == 3 {
            app.dispatch_ui_command("run_panel.run_manual");
        }
    }
    app.canvas_stream_layer = CanvasStreamLayer::Temperature;
    app.open_project(path.clone(), "temperature layer same-file regression");
    assert_eq!(
        app.canvas_stream_layer,
        CanvasStreamLayer::ConnectionDirection
    );
    assert!(matches!(
        temperature_layer(&app).result,
        StreamColorResult::Unavailable(ResultStatus::Missing)
    ));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn temperature_excludes_zero_missing_and_invalid_streams_from_the_scale() {
    let (mut app, path) = example(EXAMPLE);
    app.dispatch_ui_command("run_panel.run_manual");
    let mut workspace = app.platform_host.modeling_inputs().workspace.clone();
    let mut snapshot = rf_ui::latest_snapshot(&workspace).unwrap().clone();
    snapshot
        .streams
        .retain(|stream| stream.stream_id.as_str() != "stream-feed");
    for stream in &mut snapshot.streams {
        match stream.stream_id.as_str() {
            "stream-heated" => stream.temperature_k = f64::INFINITY,
            "stream-liquid" => stream.temperature_k = 1300.0,
            "stream-vapor" => stream.total_molar_flow_mol_s = -5.0,
            id => panic!("Unexpected stream {id}"),
        }
    }
    workspace.store_snapshot(snapshot.clone(), 4);
    let layer = CanvasStreamLayer::Temperature.project(&workspace).unwrap();
    assert!(matches!(
        layer.result,
        StreamColorResult::Current { scale: None, .. }
    ));
    for (id, message) in [
        ("stream-feed", "No temperature"),
        ("stream-heated", "Invalid temperature"),
        ("stream-liquid", "Zero flow"),
        ("stream-vapor", "Invalid molar flow"),
    ] {
        assert!(layer.stroke(id).dashed);
        assert_eq!(layer.stroke(id).color, NEUTRAL);
        assert_eq!(
            layer.stream_readout(id, StudioShellLocale::En).as_deref(),
            Some(message)
        );
    }

    let heated = snapshot
        .streams
        .iter_mut()
        .find(|stream| stream.stream_id.as_str() == "stream-heated")
        .unwrap();
    heated.temperature_k = 280.0;
    workspace.store_snapshot(snapshot, 4);
    let layer = CanvasStreamLayer::Temperature.project(&workspace).unwrap();
    assert!(matches!(
        layer.result,
        StreamColorResult::Current {
            scale: Some(StreamColorScale {
                minimum: 280.0,
                maximum: 280.0
            }),
            ..
        }
    ));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn absolute_pressure_projects_actual_valve_runs_and_preserves_si_colors() {
    let (mut app, path) = example(VALVE_EXAMPLE);
    assert!(matches!(
        absolute_pressure_layer(&app).result,
        StreamColorResult::Unavailable(ResultStatus::Missing)
    ));
    app.dispatch_ui_command("run_panel.run_manual");
    let layer = absolute_pressure_layer(&app);
    assert!(matches!(
        layer.result,
        StreamColorResult::Current {
            scale: Some(StreamColorScale {
                minimum: 650000.0,
                maximum: 700000.0
            }),
            sequence: 1,
            revision: 0,
            ..
        }
    ));
    assert_eq!(layer.stroke("stream-feed").color, HIGH);
    assert_eq!(layer.stroke("stream-throttled").color, LOW);
    assert!(!layer.stroke("stream-liquid").dashed);

    for (unit, inlet, outlet) in [
        (MeasurementUnit::Kilopascal, "700 kPa", "650 kPa"),
        (MeasurementUnit::Megapascal, "0.7 MPa", "0.65 MPa"),
        (MeasurementUnit::Bar, "7 bar", "6.5 bar"),
        (MeasurementUnit::Pascal, "700000 Pa", "650000 Pa"),
    ] {
        let mut units = DisplayUnitSet::si();
        units
            .set_unit(QuantityKind::AbsolutePressure, unit)
            .unwrap();
        app.apply_presentation_command(ProjectPresentationCommand::Apply(units))
            .unwrap();
        let layer = absolute_pressure_layer(&app);
        assert!(matches!(
            layer.result,
            StreamColorResult::Current {
                sequence: 1,
                revision: 0,
                ..
            }
        ));
        for (id, color, display) in [
            ("stream-feed", HIGH, inlet),
            ("stream-throttled", LOW, outlet),
        ] {
            assert_eq!(layer.stroke(id).color, color);
            assert_eq!(
                layer.stream_readout(id, StudioShellLocale::En).as_deref(),
                Some(display)
            );
        }
    }

    commit_unit_field(&mut app, "flash-1", "outlet_temperature_k", "345");
    let stale = absolute_pressure_layer(&app);
    assert!(matches!(
        stale.result,
        StreamColorResult::Unavailable(ResultStatus::Stale)
    ));
    assert_eq!(stale.stroke("stream-throttled").color, NEUTRAL);
    assert!(!stale.stroke("stream-throttled").dashed);
    assert_eq!(
        stale.stream_readout("stream-throttled", StudioShellLocale::En),
        None
    );
    app.dispatch_ui_command("run_panel.run_manual");
    let layer = absolute_pressure_layer(&app);
    assert!(matches!(
        layer.result,
        StreamColorResult::Current {
            sequence: 2,
            revision: 1,
            ..
        }
    ));
    assert_eq!(layer.stroke("stream-liquid").color, LOW);
    assert!(layer.stroke("stream-liquid").dashed);
    assert_eq!(
        layer
            .stream_readout("stream-liquid", StudioShellLocale::En)
            .as_deref(),
        Some("650000 Pa · Zero flow")
    );

    for unit in ["valve-1", "flash-1"] {
        commit_unit_field(&mut app, unit, "outlet_pressure_pa", "700000");
    }
    for sequence in [3, 4] {
        app.dispatch_ui_command("run_panel.run_manual");
        let layer = absolute_pressure_layer(&app);
        assert!(matches!(
            layer.result,
            StreamColorResult::Current {
                scale: Some(StreamColorScale { minimum: 700000.0, maximum: 700000.0 }),
                sequence: actual,
                revision: 3,
                ..
            } if actual == sequence
        ));
        for id in [
            "stream-feed",
            "stream-throttled",
            "stream-liquid",
            "stream-vapor",
        ] {
            assert_eq!(layer.stroke(id).color, scale_color(0.5));
        }
        assert!(layer.stroke("stream-liquid").dashed);
    }

    let (_, other_path) = example(EXAMPLE);
    app.canvas_stream_layer = CanvasStreamLayer::AbsolutePressure;
    app.open_project(other_path.clone(), "pressure layer project switch");
    assert_eq!(
        app.canvas_stream_layer,
        CanvasStreamLayer::ConnectionDirection
    );
    assert!(matches!(
        absolute_pressure_layer(&app).result,
        StreamColorResult::Unavailable(ResultStatus::Missing)
    ));
    std::fs::remove_file(other_path).unwrap();
    std::fs::remove_file(path).unwrap();
}

#[test]
fn absolute_pressure_counts_the_zero_flow_extreme_and_excludes_invalid_values() {
    let (mut app, path) = example(VALVE_EXAMPLE);
    app.dispatch_ui_command("run_panel.run_manual");
    let mut workspace = app.platform_host.modeling_inputs().workspace.clone();
    let mut snapshot = rf_ui::latest_snapshot(&workspace).unwrap().clone();
    snapshot
        .streams
        .retain(|stream| stream.stream_id.as_str() != "stream-vapor");
    let liquid = snapshot
        .streams
        .iter_mut()
        .find(|stream| stream.stream_id.as_str() == "stream-liquid")
        .unwrap();
    liquid.pressure_pa = 600000.0;
    liquid.total_molar_flow_mol_s = 0.0;
    workspace.store_snapshot(snapshot.clone(), 4);
    let layer = CanvasStreamLayer::AbsolutePressure
        .project(&workspace)
        .unwrap();
    assert!(matches!(
        layer.result,
        StreamColorResult::Current {
            scale: Some(StreamColorScale {
                minimum: 600000.0,
                maximum: 700000.0
            }),
            ..
        }
    ));
    assert!(layer.stroke("stream-liquid").dashed);
    assert_eq!(layer.stroke("stream-liquid").color, LOW);
    assert_eq!(layer.stroke("stream-throttled").color, scale_color(0.5));
    assert_eq!(layer.stroke("stream-vapor").color, NEUTRAL);
    assert!(layer.stroke("stream-vapor").dashed);
    assert_eq!(
        layer
            .stream_readout("stream-vapor", StudioShellLocale::En)
            .as_deref(),
        Some("No absolute pressure")
    );

    for invalid in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
        for stream in &mut snapshot.streams {
            stream.pressure_pa = invalid;
        }
        workspace.store_snapshot(snapshot.clone(), 4);
        let layer = CanvasStreamLayer::AbsolutePressure
            .project(&workspace)
            .unwrap();
        assert!(matches!(
            layer.result,
            StreamColorResult::Current { scale: None, .. }
        ));
        assert_eq!(layer.stroke("stream-liquid").color, NEUTRAL);
        assert!(layer.stroke("stream-liquid").dashed);
        assert_eq!(
            layer
                .stream_readout("stream-liquid", StudioShellLocale::En)
                .as_deref(),
            Some("Invalid absolute pressure")
        );
    }
    std::fs::remove_file(path).unwrap();
}

fn frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    width: f32,
    events: Vec<egui::Event>,
) -> TreeUpdate {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 1600.0),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |ctx| {
            let window = app.platform_host.snapshot().window_model();
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_canvas_area(ui, &window, StudioGuiWindowAreaId::Canvas);
            });
        },
    )
    .platform_output
    .accesskit_update
    .unwrap()
}

fn contains(tree: &TreeUpdate, text: &str) -> bool {
    tree.nodes
        .iter()
        .any(|(_, node)| [node.label(), node.value()].contains(&Some(text)))
}

#[test]
fn canvas_stream_layer_keyboard_choice_keeps_stream_and_shows_the_current_legend() {
    let (mut app, path) = example(EXAMPLE);
    app.screen = StudioShellScreen::Workbench;
    app.dispatch_ui_command("run_panel.run_manual");
    app.select_canvas_stream("inspector.focus_stream:stream-liquid");
    let document = app.platform_host.document().clone();
    for (width, locale, layer, label, legend, range) in [
        (
            1280.0,
            StudioShellLocale::En,
            CanvasStreamLayer::MolarFlow,
            "Molar flow",
            "Molar flow legend",
            "0 mol/s – 5 mol/s",
        ),
        (
            400.0,
            StudioShellLocale::En,
            CanvasStreamLayer::MolarFlow,
            "Molar flow",
            "Molar flow legend",
            "0 mol/s – 5 mol/s",
        ),
        (
            400.0,
            StudioShellLocale::ZhCn,
            CanvasStreamLayer::MolarFlow,
            "摩尔流量",
            "摩尔流量图例",
            "0 mol/s – 5 mol/s",
        ),
        (
            400.0,
            StudioShellLocale::En,
            CanvasStreamLayer::Temperature,
            "Stream temperature",
            "Stream temperature legend",
            "300 K – 345 K",
        ),
        (
            400.0,
            StudioShellLocale::ZhCn,
            CanvasStreamLayer::Temperature,
            "流股温度",
            "流股温度图例",
            "300 K – 345 K",
        ),
        (
            400.0,
            StudioShellLocale::En,
            CanvasStreamLayer::AbsolutePressure,
            "Stream absolute pressure",
            "Stream absolute pressure legend",
            "95000 Pa – 120000 Pa",
        ),
        (
            400.0,
            StudioShellLocale::ZhCn,
            CanvasStreamLayer::AbsolutePressure,
            "流股绝对压力",
            "流股绝对压力图例",
            "95000 Pa – 120000 Pa",
        ),
    ] {
        app.locale = locale;
        app.canvas_stream_layer = CanvasStreamLayer::ConnectionDirection;
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        if width == 400.0 {
            ctx.style_mut(|style| {
                for font in style.text_styles.values_mut() {
                    font.size = 20.0;
                }
            });
        }
        let first = frame(&mut app, &ctx, width, vec![]);
        assert!(!contains(&first, legend));
        let selector_label = localized(locale, "图层", "Layer");
        let (choice, selector) = first
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::ComboBox
                    && node.label() == Some(selector_label)
            })
            .unwrap();
        let bounds = selector.bounds().unwrap();
        assert!(
            bounds.x0 >= 0.0 && bounds.x1 <= f64::from(width),
            "{bounds:?}"
        );
        let focus = frame(
            &mut app,
            &ctx,
            width,
            vec![egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Focus,
                target: *choice,
                data: None,
            })],
        );
        assert_eq!(focus.focus, *choice);
        let press = |app: &mut ReadyAppState, key| {
            frame(
                app,
                &ctx,
                width,
                [true, false]
                    .map(|pressed| egui::Event::Key {
                        key,
                        physical_key: Some(key),
                        pressed,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    })
                    .to_vec(),
            )
        };
        press(&mut app, egui::Key::Enter);
        let open = frame(&mut app, &ctx, width, vec![]);
        assert!(contains(&open, label));
        let steps = match layer {
            CanvasStreamLayer::MolarFlow => 1,
            CanvasStreamLayer::Temperature => 2,
            CanvasStreamLayer::AbsolutePressure => 3,
            CanvasStreamLayer::ConnectionDirection => unreachable!(),
        };
        for _ in 0..steps {
            press(&mut app, egui::Key::ArrowDown);
        }
        assert_eq!(
            app.canvas_stream_layer,
            CanvasStreamLayer::ConnectionDirection
        );
        press(&mut app, egui::Key::Enter);
        let changed = frame(&mut app, &ctx, width, vec![]);
        assert_eq!(changed.focus, *choice);
        let (_, selected) = changed.nodes.iter().find(|(id, _)| id == choice).unwrap();
        let bounds = selected.bounds().unwrap();
        assert!(
            bounds.x0 >= 0.0 && bounds.x1 <= f64::from(width),
            "{bounds:?}"
        );
        assert_eq!(app.canvas_stream_layer, layer);
        assert_eq!(
            app.flow_path.selected(),
            Some(&StreamId::new("stream-liquid"))
        );
        assert!(contains(&changed, legend));
        assert!(contains(&changed, range));
        if layer == CanvasStreamLayer::AbsolutePressure {
            assert!(changed.nodes.iter().any(|(_, node)| {
                node.role() == egui::accesskit::Role::Label
                    && node.value()
                        == Some(localized(
                            locale,
                            "Liquid Outlet · 95000 Pa · 零流量",
                            "Liquid Outlet · 95000 Pa · Zero flow",
                        ))
            }));
        }
        assert_eq!(app.platform_host.document(), &document);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn canvas_stream_layer_failure_keeps_the_existing_diagnostic_action() {
    let (mut app, path) = example("failures/unbound-inlet-port.rfproj.json");
    app.screen = StudioShellScreen::Workbench;
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("run_panel.run_manual");
    app.select_canvas_stream("inspector.focus_stream:stream-heated");
    for (layer, range) in [
        (CanvasStreamLayer::MolarFlow, "0 mol/s – 5 mol/s"),
        (CanvasStreamLayer::Temperature, "300 K – 345 K"),
        (CanvasStreamLayer::AbsolutePressure, "95000 Pa – 120000 Pa"),
    ] {
        app.canvas_stream_layer = layer;
        assert!(matches!(
            layer
                .project(&app.platform_host.modeling_inputs().workspace)
                .unwrap()
                .result,
            StreamColorResult::Unavailable(ResultStatus::Failed)
        ));
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let screen = frame(&mut app, &ctx, 1280.0, vec![]);
        assert!(contains(
            &screen,
            "The run produced no result for coloring. Showing connection direction."
        ));
        assert!(contains(&screen, "View flowsheet run diagnostics"));
        assert!(!contains(&screen, range));
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn narrow_canvas_scrolls_the_flow_legend_and_selected_stream_into_view() {
    for locale in [StudioShellLocale::En, StudioShellLocale::ZhCn] {
        let (mut app, path) = example(EXAMPLE);
        app.screen = StudioShellScreen::Workbench;
        app.locale = locale;
        app.canvas_stream_layer = CanvasStreamLayer::MolarFlow;
        app.dispatch_ui_command("run_panel.run_manual");
        app.select_canvas_stream("inspector.focus_stream:stream-heated");

        let ctx = egui::Context::default();
        ctx.all_styles_mut(|style| {
            for font in style.text_styles.values_mut() {
                font.size = 20.0;
            }
        });
        let frame = |app: &mut ReadyAppState, time| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(416.0, 420.0),
                    )),
                    time: Some(time),
                    ..Default::default()
                },
                |ctx| {
                    let window = app.platform_host.snapshot().window_model();
                    egui::CentralPanel::default().show(ctx, |ui| {
                        app.render_canvas_area(ui, &window, StudioGuiWindowAreaId::Canvas);
                    });
                },
            )
        };
        for step in 0..16 {
            let _ = frame(&mut app, f64::from(step) * 0.1);
        }
        let out = frame(&mut app, 1.6);
        let range = "0 mol/s – 5 mol/s";
        let visible_text = |label| {
            out.shapes.iter().any(|clipped| {
                matches!(&clipped.shape, egui::Shape::Text(text) if text.galley.text() == label)
                    && clipped
                        .clip_rect
                        .expand(1.0)
                        .contains_rect(clipped.shape.visual_bounding_rect())
            })
        };
        assert!(visible_text(range), "range was clipped in {locale:?}");
        assert!(
            visible_text(localized(
                locale,
                "当前快照 · 运行 #1 · r0",
                "Current snapshot · Run #1 · r0",
            )),
            "snapshot identity was clipped in {locale:?}"
        );
        assert!(
            visible_text(localized(locale, "摩尔流量", "Molar flow")),
            "scrolling the plot hid the layer choice in {locale:?}"
        );
        let colored: Vec<_> = out.shapes.iter().filter(|clipped| {
            matches!(&clipped.shape, egui::Shape::LineSegment { stroke, .. } if stroke.color == HIGH)
        }).collect();
        assert!(
            colored.iter().any(|clipped| clipped
                .clip_rect
                .intersects(clipped.shape.visual_bounding_rect())),
            "{locale:?}: {colored:?}"
        );
        std::fs::remove_file(path).unwrap();
    }
}
