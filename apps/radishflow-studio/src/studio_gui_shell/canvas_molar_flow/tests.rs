use super::*;
use crate::studio_gui_shell::tests::{ready_app_state, synced_workspace_config};
use egui::accesskit::{Action, ActionRequest, TreeUpdate};
use rf_types::{StreamId, units::DisplayUnitSet};
use rf_ui::ProjectPresentationCommand;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_PROJECT: AtomicUsize = AtomicUsize::new(0);
const EXAMPLE: &str = "feed-heater-flash-binary-hydrocarbon.rfproj.json";

fn example(name: &str) -> (ReadyAppState, PathBuf) {
    let path = std::env::temp_dir().join(format!(
        "rf-v2-molar-flow-{}-{}.rfproj.json",
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

fn current_layer(app: &ReadyAppState) -> CanvasMolarFlow<'_> {
    CanvasStreamLayer::MolarFlow
        .project(&app.platform_host.modeling_inputs().workspace)
        .unwrap()
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
        current_layer(&app),
        CanvasMolarFlow::Unavailable(ResultStatus::Missing)
    ));

    app.dispatch_ui_command("run_panel.run_manual");
    let layer = current_layer(&app);
    assert!(matches!(
        layer,
        CanvasMolarFlow::Current {
            maximum: Some(5.0),
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
        layer,
        CanvasMolarFlow::Current {
            maximum: Some(5.0),
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
        current_layer(&app),
        CanvasMolarFlow::Current {
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
        stale,
        CanvasMolarFlow::Unavailable(ResultStatus::Stale)
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
        current_layer(&app),
        CanvasMolarFlow::Unavailable(ResultStatus::Missing)
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
        layer,
        CanvasMolarFlow::Current {
            maximum: Some(0.0),
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
fn canvas_molar_flow_keyboard_choice_keeps_stream_and_shows_the_current_legend() {
    let (mut app, path) = example(EXAMPLE);
    app.screen = StudioShellScreen::Workbench;
    app.dispatch_ui_command("run_panel.run_manual");
    app.select_canvas_stream("inspector.focus_stream:stream-liquid");
    let document = app.platform_host.document().clone();
    for (width, locale, label, legend) in [
        (
            1280.0,
            StudioShellLocale::En,
            "Molar flow",
            "Molar flow legend",
        ),
        (
            400.0,
            StudioShellLocale::En,
            "Molar flow",
            "Molar flow legend",
        ),
        (400.0, StudioShellLocale::ZhCn, "摩尔流量", "摩尔流量图例"),
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
        let (choice, selector) = first
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
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
        frame(
            &mut app,
            &ctx,
            width,
            [true, false]
                .map(|pressed| egui::Event::Key {
                    key: egui::Key::Enter,
                    physical_key: Some(egui::Key::Enter),
                    pressed,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                })
                .to_vec(),
        );
        let changed = frame(&mut app, &ctx, width, vec![]);
        assert_eq!(app.canvas_stream_layer, CanvasStreamLayer::MolarFlow);
        assert_eq!(
            app.flow_path.selected(),
            Some(&StreamId::new("stream-liquid"))
        );
        assert!(contains(&changed, legend));
        assert!(contains(&changed, "0 mol/s – 5 mol/s"));
        assert_eq!(app.platform_host.document(), &document);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn canvas_molar_flow_failure_keeps_the_existing_diagnostic_action() {
    let (mut app, path) = example("failures/unbound-inlet-port.rfproj.json");
    app.screen = StudioShellScreen::Workbench;
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("run_panel.run_manual");
    app.select_canvas_stream("inspector.focus_stream:stream-heated");
    app.canvas_stream_layer = CanvasStreamLayer::MolarFlow;
    assert!(matches!(
        current_layer(&app),
        CanvasMolarFlow::Unavailable(ResultStatus::Failed)
    ));
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let screen = frame(&mut app, &ctx, 1280.0, vec![]);
    assert!(contains(
        &screen,
        "The run produced no result for coloring. Showing connection direction."
    ));
    assert!(contains(&screen, "View flowsheet run diagnostics"));
    assert!(!contains(&screen, "0 mol/s – 5 mol/s"));
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
