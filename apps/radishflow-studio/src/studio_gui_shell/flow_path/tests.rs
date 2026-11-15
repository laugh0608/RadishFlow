use super::*;
use crate::studio_gui_shell::tests::{ready_app_state, synced_workspace_config};
use egui::accesskit::{Action, ActionRequest, NodeId, TreeUpdate};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_EXAMPLE: AtomicU64 = AtomicU64::new(0);

fn example(path: &str) -> (ReadyAppState, PathBuf) {
    let project = std::env::temp_dir().join(format!(
        "rf-v1-flow-path-{}-{}.rfproj.json",
        std::process::id(),
        NEXT_EXAMPLE.fetch_add(1, Ordering::Relaxed),
    ));
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets")
        .join(path);
    std::fs::copy(source, &project).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: project.clone(),
        ..synced_workspace_config()
    });
    app.screen = StudioShellScreen::Workbench;
    (app, project)
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
                egui::vec2(width, 980.0),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |ctx| {
            if width > 400.0 {
                app.update(ctx);
            } else {
                let window = app.platform_host.snapshot().window_model();
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.render_flow_path_return(ui);
                    if app.screen == StudioShellScreen::Results {
                        app.render_bottom_results_table(ui, &window);
                    } else {
                        app.render_flow_path(ui, &window);
                    }
                });
            }
        },
    )
    .platform_output
    .accesskit_update
    .unwrap()
}

fn button(tree: &TreeUpdate, label: &str) -> NodeId {
    tree.nodes
        .iter()
        .find(|(_, node)| {
            node.role() == egui::accesskit::Role::Button && node.label() == Some(label)
        })
        .unwrap_or_else(|| panic!("missing button: {label}"))
        .0
}

fn activate(app: &mut ReadyAppState, ctx: &egui::Context, width: f32, id: NodeId) {
    let focused = frame(
        app,
        ctx,
        width,
        vec![egui::Event::AccessKitActionRequest(ActionRequest {
            action: Action::Focus,
            target: id,
            data: None,
        })],
    );
    assert_eq!(focused.focus, id);
    frame(
        app,
        ctx,
        width,
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: Some(egui::Key::Enter),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            })
            .collect(),
    );
}

#[test]
fn flow_path_reads_raw_current_flow_and_uses_one_result_identity_for_playback() {
    let (mut app, path) = example("feed-heater-flash-binary-hydrocarbon.rfproj.json");
    let heated = StreamId::new("stream-heated");
    let liquid = StreamId::new("stream-liquid");
    app.select_canvas_stream("inspector.focus_stream:stream-heated");
    assert_eq!(
        FlowPathReading::from_workspace(&app.platform_host.modeling_inputs().workspace, &heated),
        FlowPathReading::Unavailable(ResultStatus::Missing)
    );

    app.dispatch_ui_command("run_panel.run_manual");
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let positive = FlowPathReading::from_workspace(workspace, &heated);
    let zero = FlowPathReading::from_workspace(workspace, &liquid);
    let FlowPathReading::Current {
        amount,
        revision,
        sequence,
    } = positive
    else {
        panic!("expected current Heated Outlet")
    };
    assert_eq!(amount, 5.0);
    assert_eq!(
        zero,
        FlowPathReading::Current {
            amount: 0.0,
            revision,
            sequence
        }
    );
    assert_eq!(
        rf_ui::format_quantity(
            amount,
            QuantityKind::MolarFlow,
            rf_types::units::MeasurementUnit::KilomolePerHour
        )
        .unwrap(),
        "18 kmol/h"
    );
    assert_eq!(
        positive.playback_unavailable(StudioShellLocale::En, true, MotionPreference::Allowed),
        None
    );
    for blocked in [MotionPreference::Reduced, MotionPreference::Unknown] {
        assert!(
            positive
                .playback_unavailable(StudioShellLocale::En, true, blocked)
                .is_some()
        );
    }
    assert!(
        positive
            .playback_unavailable(StudioShellLocale::En, false, MotionPreference::Allowed)
            .is_some()
    );
    assert!(
        zero.playback_unavailable(StudioShellLocale::En, true, MotionPreference::Allowed)
            .unwrap()
            .contains("Zero flow")
    );
    assert_eq!(
        FlowPathReading::from_workspace(workspace, &StreamId::new("unseen")),
        FlowPathReading::NoStreamFlow
    );

    let guard = app
        .flow_path_guard(FlowPathAction::Results, &heated)
        .unwrap();
    app.flow_path.playing = Some(FlowPathPlayback {
        guard,
        started: 1.0,
    });
    assert!(app.flow_path.phase(1.4).is_some());
    app.dispatch_ui_command("run_panel.run_manual");
    app.reconcile_flow_path();
    let result =
        FlowPathReading::from_workspace(&app.platform_host.modeling_inputs().workspace, &heated);
    assert!(matches!(result, FlowPathReading::Current {
        revision: current_revision, sequence: current_sequence, ..
    } if current_revision == revision && current_sequence > sequence));
    assert_eq!(app.flow_path.phase(1.5), None);

    let guard = app
        .flow_path_guard(FlowPathAction::Results, &heated)
        .unwrap();
    app.flow_path.playing = Some(FlowPathPlayback {
        guard,
        started: 2.0,
    });
    app.select_canvas_stream("inspector.focus_stream:stream-liquid");
    assert_eq!(app.flow_path.selected(), Some(&liquid));
    assert_eq!(app.flow_path.phase(2.1), None);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn flow_path_keyboard_enters_the_selected_result_and_returns_to_the_origin() {
    for (width, locale, enter, back) in [
        (
            1280.0,
            StudioShellLocale::ZhCn,
            "在流程结果中查看",
            "← 返回二维流向与路径",
        ),
        (
            400.0,
            StudioShellLocale::ZhCn,
            "在流程结果中查看",
            "← 返回二维流向与路径",
        ),
        (
            400.0,
            StudioShellLocale::En,
            "View in flowsheet results",
            "← Back to flow direction and path",
        ),
    ] {
        let (mut app, path) = example("feed-heater-flash-binary-hydrocarbon.rfproj.json");
        app.locale = locale;
        app.dispatch_ui_command("run_panel.run_manual");
        app.select_canvas_stream("inspector.focus_stream:stream-heated");
        app.dispatch_ui_command("inspector.focus_unit:heater-1");
        app.dispatch_inspector_field_draft_update(
            radishflow_studio::inspector_draft_update_command_id(
                "unit:heater-1:outlet_pressure_pa",
            ),
            "1e-",
        );
        app.bottom_drawer_tab = StudioShellBottomDrawerTab::RunLog;
        app.bottom_drawer_tab = StudioShellBottomDrawerTab::RunLog;
        let viewport = CanvasInitialViewportFitState::restore(egui::vec2(-36.0, 16.0));
        app.canvas_initial_viewport_fit = viewport;
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        fonts::configure_studio_fonts(&ctx);
        if width == 400.0 {
            ctx.style_mut(|style| {
                style
                    .text_styles
                    .get_mut(&egui::TextStyle::Body)
                    .unwrap()
                    .size = 20.0;
                style
                    .text_styles
                    .get_mut(&egui::TextStyle::Button)
                    .unwrap()
                    .size = 20.0;
            });
        }
        frame(&mut app, &ctx, width, vec![]);
        let tree = frame(&mut app, &ctx, width, vec![]);
        let origin = button(&tree, enter);
        if width == 400.0 {
            let bounds = tree
                .nodes
                .iter()
                .find(|(id, _)| *id == origin)
                .unwrap()
                .1
                .bounds()
                .unwrap();
            assert!(
                bounds.x0 >= 0.0 && bounds.x1 <= f64::from(width),
                "{bounds:?}"
            );
        }
        let before = app.platform_host.snapshot().runtime.workspace_document;
        assert_eq!(before.save_state.pending_input_count, 1);
        activate(&mut app, &ctx, width, origin);
        assert_eq!(app.screen, StudioShellScreen::Results);
        let tree = frame(&mut app, &ctx, width, vec![]);
        assert!(
            tree.nodes
                .iter()
                .any(|(id, node)| *id == tree.focus && node.label() == Some("Heated Outlet"))
        );
        app.canvas_initial_viewport_fit =
            CanvasInitialViewportFitState::restore(egui::vec2(8.0, 40.0));
        activate(&mut app, &ctx, width, button(&tree, back));
        let tree = frame(&mut app, &ctx, width, vec![]);
        assert_eq!(tree.focus, button(&tree, enter), "{width}");
        assert_eq!(
            app.flow_path.selected(),
            Some(&StreamId::new("stream-heated"))
        );
        assert_eq!(app.bottom_drawer_tab, StudioShellBottomDrawerTab::RunLog);
        assert_eq!(app.canvas_initial_viewport_fit, viewport);
        assert_eq!(
            app.platform_host.snapshot().runtime.workspace_document,
            before
        );
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn flow_path_failure_opens_existing_diagnostics_and_clears_on_same_file_reopen() {
    let (mut app, path) = example("failures/unbound-inlet-port.rfproj.json");
    app.dispatch_ui_command("run_panel.run_manual");
    app.select_canvas_stream("inspector.focus_stream:stream-heated");
    let id = StreamId::new("stream-heated");
    let before = app.platform_host.snapshot().runtime.workspace_document;
    assert_eq!(
        FlowPathReading::from_workspace(&app.platform_host.modeling_inputs().workspace, &id),
        FlowPathReading::Unavailable(ResultStatus::Failed)
    );
    let original_drawer = app.bottom_drawer_tab;
    app.open_flow_path_visit(&id, FlowPathAction::Diagnostics);
    assert_eq!(
        app.bottom_drawer_tab,
        StudioShellBottomDrawerTab::Diagnostics
    );
    assert!(app.modeling_problems.runtime_details());
    app.return_from_flow_path_visit(0);
    assert_eq!(app.flow_path.selected(), Some(&id));
    assert_eq!(app.bottom_drawer_tab, original_drawer);
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    app.open_flow_path_visit(&id, FlowPathAction::Diagnostics);
    app.open_project(path.clone(), "flow path same-file regression");
    assert!(app.flow_path.selected().is_none());
    assert!(app.flow_path.visit.is_none());
    assert!(app.flow_path.return_focus.is_none());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn flow_path_return_focuses_an_explanation_when_results_change_or_the_stream_is_removed() {
    let (mut app, path) = example("feed-heater-flash-binary-hydrocarbon.rfproj.json");
    let id = StreamId::new("stream-heated");
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("run_panel.run_manual");
    app.select_canvas_stream("inspector.focus_stream:stream-heated");
    app.open_flow_path_visit(&id, FlowPathAction::Results);
    app.dispatch_ui_command("run_panel.run_manual");
    app.return_from_flow_path_visit(0);

    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let tree = frame(&mut app, &ctx, 400.0, vec![]);
    assert!(tree.nodes.iter().any(|(id, node)| *id == tree.focus
        && node.value() == Some("The flow result changed. Returned to the current canvas.")));
    assert_eq!(app.flow_path.selected(), Some(&id));

    app.open_flow_path_visit(&id, FlowPathAction::Results);
    app.dispatch_ui_command("canvas.delete_selected_stream");
    app.reconcile_flow_path();
    assert!(
        !app.platform_host
            .document()
            .flowsheet
            .streams
            .contains_key(&id)
    );
    assert_eq!(
        FlowPathReading::from_workspace(&app.platform_host.modeling_inputs().workspace, &id),
        FlowPathReading::Unavailable(ResultStatus::Stale),
    );
    app.return_from_flow_path_visit(0);
    let tree = frame(&mut app, &ctx, 400.0, vec![]);
    assert!(tree.nodes.iter().any(|(id, node)| *id == tree.focus
        && node.value() == Some("The originating stream was removed.")));
    assert_eq!(app.flow_path.selected(), None);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn a_large_flow_only_schedules_preview_repaints_while_playing_on_a_visible_canvas() {
    let (mut app, path) = example("feed-heater-flash-binary-hydrocarbon.rfproj.json");
    let mut project: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let flowsheet = &mut project["document"]["flowsheet"];
    let base_units = flowsheet["units"].as_object().unwrap().clone();
    let base_streams = flowsheet["streams"].as_object().unwrap().clone();
    for copy in 1..24 {
        for (name, stream) in &base_streams {
            let mut stream = stream.clone();
            let id = format!("{name}-copy-{copy}");
            stream["id"] = serde_json::json!(id);
            flowsheet["streams"][&id] = stream;
        }
        for (name, unit) in &base_units {
            let mut unit = unit.clone();
            let id = format!("{name}-copy-{copy}");
            unit["id"] = serde_json::json!(id);
            for port in unit["ports"].as_array_mut().unwrap() {
                if let Some(stream) = port["stream_id"].as_str() {
                    port["stream_id"] = serde_json::json!(format!("{stream}-copy-{copy}"));
                }
            }
            flowsheet["units"][&id] = unit;
        }
    }
    std::fs::write(&path, serde_json::to_vec(&project).unwrap()).unwrap();
    app.open_project(path.clone(), "large flow path fixture");
    app.screen = StudioShellScreen::Workbench;
    app.dispatch_ui_command("run_panel.run_manual");
    let id = StreamId::new("stream-heated");
    let guard = app.flow_path_guard(FlowPathAction::Results, &id).unwrap();
    app.flow_path.select(app.current_window_id(), id);
    app.canvas_initial_viewport_fit = CanvasInitialViewportFitState::restore(egui::Vec2::ZERO);
    assert_eq!(app.platform_host.document().flowsheet.streams.len(), 96);

    fn delay(app: &mut ReadyAppState, clipped: bool) -> Duration {
        let ctx = egui::Context::default();
        let mut delay = Duration::ZERO;
        for frame in 0..4 {
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1800.0, 720.0),
                    )),
                    time: Some(f64::from(frame) / 30.0),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if clipped {
                            ui.set_clip_rect(egui::Rect::NOTHING);
                        }
                        app.render_canvas_drop_surface(ui);
                    });
                },
            );
            delay = output.viewport_output[&egui::ViewportId::ROOT].repaint_delay;
        }
        delay
    }

    assert_eq!(delay(&mut app, false), Duration::MAX);
    app.flow_path.playing = Some(FlowPathPlayback {
        guard,
        started: 0.0,
    });
    let playing_delay = delay(&mut app, false);
    assert!(
        playing_delay > Duration::ZERO && playing_delay <= Duration::from_millis(33),
        "{playing_delay:?}"
    );
    assert_eq!(delay(&mut app, true), Duration::MAX);
    app.screen = StudioShellScreen::Results;
    assert_eq!(delay(&mut app, false), Duration::MAX);
    std::fs::remove_file(path).unwrap();
}
