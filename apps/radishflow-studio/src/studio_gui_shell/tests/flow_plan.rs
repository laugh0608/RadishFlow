use super::runtime::collect_shape_texts;
use super::*;
use radishflow_studio::{StudioBackgroundRunAction as Action, flow_actions::FlowEditAccess};
use rf_ui::flow_plan::*;
fn app() -> ReadyAppState {
    let mut config = synced_workspace_config();
    config.project_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json");
    let mut app = ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    app.screen = StudioShellScreen::Run;
    app.enable_background_runs();
    app
}
fn frame(app: &mut ReadyAppState, ctx: &egui::Context, width: f32) -> Vec<String> {
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 3000.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.render_flow_plan(ui));
        },
    );
    let mut text = Vec::new();
    for shape in output.shapes {
        collect_shape_texts(&shape.shape, &mut text);
    }
    text
}
fn drain(app: &mut ReadyAppState, ctx: &egui::Context) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.background_busy() {
        app.poll_background_runs(ctx);
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
}
#[test]
fn gui_run_uses_applied_plan_and_keeps_unfinished_draft_across_narrow_bilingual_frames() {
    let mut app = app();
    let ctx = egui::Context::default();
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let physical = workspace.document.clone();
    let mut draft = FlowPlanDraft::new(workspace, workspace.flow_plan.current().clone());
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650000.0,
            rf_types::units::MeasurementUnit::Kilopascal,
        )
        .unwrap()
        .raw = "1e-".into();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
    for locale in [StudioShellLocale::En, StudioShellLocale::ZhCn] {
        app.locale = locale;
        let text = frame(&mut app, &ctx, 400.0).join("\n");
        assert!(text.contains("1e-"), "{text}");
        assert_eq!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .flow_plan
                .draft()
                .unwrap()
                .numbers()[&FlowPlanNumericField::Pressure]
                .raw,
            "1e-"
        );
    }
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        rf_ui::result_query::FlowResultView::Current {
            snapshot: rf_ui::FlowResultRecord::Partitioned(_),
            ..
        }
    ));
    assert_eq!(app.platform_host.document(), &physical);
    let text = frame(&mut app, &ctx, 400.0).join("\n");
    assert!(text.contains("Current"));
    let w = &app.platform_host.modeling_inputs().workspace;
    assert!(w.flow_plan.draft().is_some());
    let path = std::env::temp_dir().join(format!(
        "rf-plan-gui-save-{}.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    app.request_save_project_as(path.clone(), false);
    assert!(path.exists());
    let reopened = radishflow_studio::load_project_app_state(&path).unwrap();
    assert!(reopened.workspace.flow_plan.draft().is_none());
    assert!(matches!(
        reopened.workspace.flow_result(),
        rf_ui::result_query::FlowResultView::Missing { .. }
    ));
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .draft()
            .is_some()
    );
    fs::remove_file(path).unwrap();
}
#[test]
fn output_capture_and_departure_confirmation_reject_plan_and_raw_draft_changes() {
    let mut app = app();
    let ctx = egui::Context::default();
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let snapshot = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap();
    let path = std::env::temp_dir().join(format!(
        "rf-plan-output-{}.txt",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&path, "existing").unwrap();
    app.export_solve_snapshot_to_path(&snapshot, path.clone());
    assert!(app.pending_result_export.is_some());
    let w = &app.platform_host.modeling_inputs().workspace;
    let expected = FlowPlanEditContext::capture(w);
    app.background_action(Action::ConfigureFlowPlan {
        expected,
        access: FlowEditAccess::Editable,
        command: FlowPlanCommand::Apply(FlowPlanConfiguration::Acyclic),
    });
    app.confirm_result_export();
    assert_eq!(fs::read_to_string(&path).unwrap(), "existing");
    fs::remove_file(path).unwrap();
    let before = app.project_departure_checkpoint();
    let w = &app.platform_host.modeling_inputs().workspace;
    let mut draft = FlowPlanDraft::new(w, w.flow_plan.current().clone());
    draft.replace_candidate(FlowPlanConfiguration::Partitioned(
        rf_solver::partitioned::RecycleSolveOptions::new(
            rf_solver::partitioned::FixedRecycleBoundary {
                temperature_k: 300.0,
                pressure_pa: 650000.0,
            },
            vec![],
        ),
    ));
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft.clone()),
    });
    assert_ne!(before, app.project_departure_checkpoint());
    let before = app.project_departure_checkpoint();
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650000.0,
            rf_types::units::MeasurementUnit::Pascal,
        )
        .unwrap()
        .raw = "6e-".into();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
    assert_ne!(before, app.project_departure_checkpoint());
}

#[test]
fn recent_project_card_uses_the_same_v4_reader_as_open() {
    let mut app = app();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json");
    app.project_open.record_recent_project(path);
    let snapshot = app.platform_host.snapshot();
    app.window_model_with_shell_home(&snapshot);
    let card = &app.project_open.recent_case_tiles[0];
    assert_eq!(card.title, "Fixed-T/P Binary Recycle");
    assert!(card.component_summary.contains("Methane"));
    assert!(card.package_summary.contains("Lite"));
    assert_eq!(card.thumbnail.nodes.len(), 6);
}

fn output(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, 3000.0),
            )),
            modifiers: events
                .iter()
                .rev()
                .find_map(|e| {
                    if let egui::Event::Key { modifiers, .. } = e {
                        Some(*modifiers)
                    } else {
                        None
                    }
                })
                .unwrap_or_default(),
            events,
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.render_flow_plan(ui));
        },
    )
}
fn text_position(shape: &egui::Shape, text: &str, positions: &mut Vec<egui::Pos2>) {
    match shape {
        egui::Shape::Text(t) if t.galley.text() == text => {
            positions.push(t.pos + t.galley.size() / 2.0)
        }
        egui::Shape::Vec(shapes) => {
            for shape in shapes {
                text_position(shape, text, positions)
            }
        }
        _ => {}
    }
}
fn click_at(app: &mut ReadyAppState, ctx: &egui::Context, pos: egui::Pos2) {
    output(
        app,
        ctx,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    output(
        app,
        ctx,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}
fn click_text(app: &mut ReadyAppState, ctx: &egui::Context, text: &str, index: usize) {
    let out = output(app, ctx, vec![]);
    let mut positions = Vec::new();
    for shape in out.shapes {
        text_position(&shape.shape, text, &mut positions);
    }
    assert!(positions.len() > index, "missing text {text}");
    click_at(app, ctx, positions[index]);
}
#[test]
fn gui_explicit_connection_button_builds_the_return_edge_before_running() {
    use radishflow_studio::{
        flow_actions::{FlowAction, FlowActionRequest, FlowEditContext},
        modeling_actions::MaterialPortTarget,
    };
    let mut app = app();
    let ctx = egui::Context::default();
    let expected = FlowEditContext::capture(&app.platform_host.modeling_inputs().workspace);
    app.background_action(Action::EditFlow {
        access: FlowEditAccess::Editable,
        request: FlowActionRequest {
            expected,
            action: FlowAction::Disconnect(MaterialPortTarget {
                unit_id: "mixer-2".into(),
                port: "inlet_b".into(),
            }),
        },
    });
    let revision = app.platform_host.document().revision;
    click_text(&mut app, &ctx, "Modeling and connections", 0);
    click_text(&mut app, &ctx, "—", 0);
    click_text(&mut app, &ctx, "splitter-1.outlet_a", 0);
    click_text(&mut app, &ctx, "—", 0);
    click_text(&mut app, &ctx, "mixer-2.inlet_b", 0);
    let out = output(&mut app, &ctx, vec![]);
    let mut pos = Vec::new();
    for shape in out.shapes {
        text_position(
            &shape.shape,
            "Stream ID (existing stream or new explicit identity)",
            &mut pos,
        );
    }
    click_at(&mut app, &ctx, pos[0] + egui::vec2(0.0, 23.0));
    output(&mut app, &ctx, vec![egui::Event::Text("s-recycle".into())]);
    click_text(&mut app, &ctx, "Confirm connection", 0);
    assert_eq!(app.platform_host.document().revision, revision + 1);
    let w = &app.platform_host.modeling_inputs().workspace;
    assert!(w.flow_plan.current().validate_for(w).is_ok());
    assert!(rf_solver::sequential_execution_order(&w.document.flowsheet).is_err());
    click_text(&mut app, &ctx, "Run F5", 0);
    drain(&mut app, &ctx);
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        rf_ui::result_query::FlowResultView::Current {
            snapshot: rf_ui::FlowResultRecord::Partitioned(_),
            ..
        }
    ));
}

mod simultaneous;

mod equation_oriented;
