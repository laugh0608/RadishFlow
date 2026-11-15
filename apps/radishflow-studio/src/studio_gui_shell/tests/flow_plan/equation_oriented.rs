use super::*;
use rf_solver::equation_oriented::*;
use rf_types::units::MeasurementUnit;
use rf_ui::result_query::FlowResultView;
fn options(inverse: bool) -> EquationOptions {
    let mut o = EquationOptions::new(FixedBoundary {
        temperature_k: 300.0,
        pressure_pa: 650000.0,
    });
    if inverse {
        o.released = Some(ReleasedSplitFraction {
            binding: ParameterBinding {
                unit: "splitter-1".into(),
                field: ParameterField::OutletASplitFraction,
            },
            lower: 0.0,
            upper: 0.95,
            max_iterations: 64,
        });
        o.target = Some(ProductTotalTarget {
            source: PortBinding {
                unit: "splitter-1".into(),
                port: MaterialPort::OutletB,
                stream: "s-pa".into(),
            },
            value_mol_s: 1.5,
        });
    }
    o
}
fn configure(app: &mut ReadyAppState, o: EquationOptions) {
    app.background_action(Action::ConfigureFlowPlan {
        expected: FlowPlanEditContext::capture(&app.platform_host.modeling_inputs().workspace),
        access: FlowEditAccess::Editable,
        command: FlowPlanCommand::Apply(FlowPlanConfiguration::EquationOriented(o)),
    });
}
fn equation_app() -> ReadyAppState {
    let mut config = synced_workspace_config();
    config.project_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets/fixed-tp-cross-recycle-binary-hydrocarbon.rfproj.json");
    let mut app = ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    app.screen = StudioShellScreen::Run;
    app.enable_background_runs();
    configure(&mut app, options(true));
    app
}
fn key(app: &mut ReadyAppState, ctx: &egui::Context, key: egui::Key) {
    output(
        app,
        ctx,
        vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    output(
        app,
        ctx,
        vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}

// CentralPanel's horizontal margins leave 400 points for the plan editor.
const NARROW_PLAN_VIEWPORT: f32 = 416.0;

fn narrow_plan_context() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.style_mut(|style| {
        for font in style.text_styles.values_mut() {
            font.size = 20.0;
        }
    });
    ctx
}

fn focus_narrow_plan_number(app: &mut ReadyAppState, ctx: &egui::Context, raw: &str) -> egui::Id {
    let output = output_at_width(app, ctx, NARROW_PLAN_VIEWPORT, vec![]);
    let mut positions = Vec::new();
    for shape in output.shapes {
        text_position(&shape.shape, raw, &mut positions);
    }
    assert_eq!(positions.len(), 1, "expected one numeric field with {raw}");
    click_at_width(app, ctx, NARROW_PLAN_VIEWPORT, positions[0]);
    ctx.memory(|memory| memory.focused())
        .expect("numeric field focused by click")
}

fn narrow_plan_key(key: egui::Key, pressed: bool, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers,
    }
}

#[test]
fn equation_gui_invalid_enter_refocuses_submitted_field_and_preserves_drafts() {
    let spec = options(true);
    let target = FlowPlanNumericField::ProductTarget(spec.target.unwrap().source);
    let budget = FlowPlanNumericField::BisectionBudget(spec.released.unwrap().binding);
    for (field, target_raw, budget_raw, submitted_raw) in [
        ("product target", "1e-", "63", "1e-"),
        ("bisection budget", "1.6", "129", "129"),
    ] {
        let mut app = equation_app();
        let ctx = narrow_plan_context();
        app.dispatch_ui_command("run_panel.run_manual");
        drain(&mut app, &ctx);
        let workspace = &app.platform_host.modeling_inputs().workspace;
        let before = FlowPlanEditContext::capture(workspace);
        let mut draft = FlowPlanDraft::new(workspace, workspace.flow_plan.current().clone());
        draft
            .edit_number(target.clone(), 1.5, MeasurementUnit::KilomolePerHour)
            .unwrap()
            .raw = target_raw.into();
        draft
            .edit_number(budget.clone(), 64.0, MeasurementUnit::One)
            .unwrap()
            .raw = budget_raw.into();
        app.background_action(Action::FlowPlanDraft {
            access: FlowEditAccess::Editable,
            draft: Some(draft),
        });
        let focused = focus_narrow_plan_number(&mut app, &ctx, submitted_raw);
        let draft = app
            .platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .draft()
            .unwrap()
            .clone();
        for pressed in [true, false] {
            output_at_width(
                &mut app,
                &ctx,
                NARROW_PLAN_VIEWPORT,
                vec![narrow_plan_key(
                    egui::Key::Enter,
                    pressed,
                    egui::Modifiers::NONE,
                )],
            );
        }
        assert_eq!(
            ctx.memory(|memory| memory.focused()),
            Some(focused),
            "after Enter on {field}"
        );
        let workspace = &app.platform_host.modeling_inputs().workspace;
        assert_eq!(workspace.flow_plan.draft(), Some(&draft));
        assert_eq!(FlowPlanEditContext::capture(workspace), before);
        assert!(matches!(
            workspace.flow_result(),
            FlowResultView::Current { .. }
        ));
        assert!(!app.background_command_enabled("run_panel.run_manual"));
    }
}

#[test]
fn equation_gui_first_numeric_edit_keeps_focus_and_corrected_enter_applies() {
    let mut app = equation_app();
    let ctx = narrow_plan_context();
    let focused = focus_narrow_plan_number(&mut app, &ctx, "1.5");
    let replace = |app: &mut ReadyAppState, text: &str| {
        output_at_width(
            app,
            &ctx,
            NARROW_PLAN_VIEWPORT,
            vec![
                narrow_plan_key(egui::Key::A, true, egui::Modifiers::COMMAND),
                egui::Event::Text(text.into()),
                narrow_plan_key(egui::Key::A, false, egui::Modifiers::COMMAND),
            ],
        );
        output_at_width(app, &ctx, NARROW_PLAN_VIEWPORT, vec![]);
    };
    replace(&mut app, "1e-");
    assert_eq!(
        ctx.memory(|memory| memory.focused()),
        Some(focused),
        "after draft controls appear"
    );
    replace(&mut app, "1.6");
    output_at_width(
        &mut app,
        &ctx,
        NARROW_PLAN_VIEWPORT,
        vec![narrow_plan_key(
            egui::Key::Enter,
            true,
            egui::Modifiers::NONE,
        )],
    );
    let plan = &app.platform_host.modeling_inputs().workspace.flow_plan;
    assert!(plan.draft().is_none());
    assert!(
        matches!(plan.current(), FlowPlanConfiguration::EquationOriented(options) if options.target.as_ref().unwrap().value_mol_s == 1.6)
    );
    assert_ne!(ctx.memory(|memory| memory.focused()), Some(focused));
}

#[test]
fn equation_gui_english_chrome_leaves_room_for_narrow_plan() {
    let mut app = equation_app();
    let ctx = narrow_plan_context();
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let snapshot = app.platform_host.snapshot();
    let mut window = snapshot.window_model();
    for (width, weights) in [(1027.0, [12, 25, 23]), (1116.0, [28, 26, 46])] {
        for (region, weight) in window.layout_state.region_weights.iter_mut().zip(weights) {
            region.weight = weight;
        }
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 857.0),
                )),
                ..Default::default()
            },
            |ctx| {
                let mut hovered = false;
                app.render_top_bar(ctx, &snapshot.app_host_state.windows, &window, &mut hovered);
                app.render_left_sidebar(ctx, &window, &mut hovered);
                app.render_right_sidebar(ctx, &window, &mut hovered);
                app.render_bottom_status_bar(ctx, &window);
                app.render_bottom_drawer(ctx, &window);
                let center = ctx.available_rect();
                assert!((410.0..=420.0).contains(&center.width()), "{center:?}");
                assert!(
                    center.height() >= 5.0 * 24.0,
                    "editor viewport at {width}: {center:?}"
                );
                app.render_center_stage(ctx, &window, &mut hovered);
            },
        );
    }
}

#[test]
fn equation_gui_narrow_plan_scrolls_keyboard_focus_into_view() {
    let mut app = equation_app();
    let ctx = narrow_plan_context();
    ctx.style_mut(|style| style.scroll_animation = egui::style::ScrollAnimation::none());
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let mut draft = FlowPlanDraft::new(workspace, workspace.flow_plan.current().clone());
    draft
        .edit_number(
            FlowPlanNumericField::ProductTarget(options(true).target.unwrap().source),
            1.5,
            MeasurementUnit::MolePerSecond,
        )
        .unwrap()
        .raw = "1.6".into();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
    let window = app.platform_host.snapshot().window_model();
    let frame = |app: &mut ReadyAppState, height: f32, events: Vec<egui::Event>| {
        let mut focused = None;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(NARROW_PLAN_VIEWPORT, height),
                )),
                modifiers: events
                    .iter()
                    .find_map(|event| match event {
                        egui::Event::Key { modifiers, .. } => Some(*modifiers),
                        _ => None,
                    })
                    .unwrap_or_default(),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| {
                app.render_center_stage(ctx, &window, &mut false);
                focused = ctx
                    .memory(|memory| memory.focused())
                    .and_then(|id| ctx.read_response(id));
            },
        );
        (output, focused)
    };
    let (output, _) = frame(&mut app, 3000.0, vec![]);
    let mut positions = Vec::new();
    for shape in &output.shapes {
        text_position(&shape.shape, "1.6", &mut positions);
    }
    assert_eq!(positions.len(), 1);
    let pos = positions[0];
    for pressed in [true, false] {
        frame(
            &mut app,
            3000.0,
            vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
    let target = ctx
        .memory(|memory| memory.focused())
        .expect("focused target");
    const HEIGHT: f32 = 300.0;
    let visible_focus = |app: &mut ReadyAppState, name: &str| {
        let response = frame(app, HEIGHT, vec![]).1.expect("focused widget");
        let rect = response.rect;
        assert!(
            rect.top() >= 0.0 && rect.bottom() <= HEIGHT,
            "{name} outside the plan viewport: {rect:?}"
        );
        response.id
    };
    let press_tab = |app: &mut ReadyAppState, shift| {
        let modifiers = if shift {
            egui::Modifiers::SHIFT
        } else {
            egui::Modifiers::NONE
        };
        for pressed in [true, false] {
            frame(
                app,
                HEIGHT,
                vec![narrow_plan_key(egui::Key::Tab, pressed, modifiers)],
            );
        }
        // Backward focus resolves on the next pass, before the scroll settles.
        frame(app, HEIGHT, vec![]);
    };
    press_tab(&mut app, true);
    assert_ne!(visible_focus(&mut app, "bisection budget"), target);
    press_tab(&mut app, false);
    assert_eq!(visible_focus(&mut app, "product target"), target);
    for name in [
        "target unit",
        "unmatched regions",
        "all variables",
        "apply plan",
    ] {
        press_tab(&mut app, false);
        visible_focus(&mut app, name);
    }
    frame(
        &mut app,
        HEIGHT,
        vec![narrow_plan_key(
            egui::Key::Enter,
            true,
            egui::Modifiers::NONE,
        )],
    );
    assert!(
        matches!(app.platform_host.modeling_inputs().workspace.flow_plan.current(), FlowPlanConfiguration::EquationOriented(options) if options.target.as_ref().unwrap().value_mol_s == 1.6)
    );
}

#[test]
fn equation_gui_run_preview_cancel_adopt_stale_and_save_reopen() {
    let mut app = equation_app();
    let ctx = egui::Context::default();
    let text = frame(&mut app, &ctx, 900.0).join("\n");
    assert!(text.contains("Variables 29"), "{text}");
    assert!(text.contains("Matched 29"));
    click_text(&mut app, &ctx, "Run F5", 0);
    drain(&mut app, &ctx);
    let model = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap();
    assert!(model.equation_oriented.is_some());
    assert_eq!(model.final_unit_results().unwrap().len(), 6);
    let before = app.platform_host.document().clone();
    click_text(&mut app, &ctx, "Preview adoption into input…", 0);
    assert!(app.flow_plan.adoption.is_some());
    frame(&mut app, &ctx, 900.0);
    key(&mut app, &ctx, egui::Key::Enter);
    assert!(app.flow_plan.adoption.is_none());
    assert_eq!(app.platform_host.document(), &before);
    click_text(&mut app, &ctx, "Preview adoption into input…", 0);
    frame(&mut app, &ctx, 900.0);
    click_text(&mut app, &ctx, "Confirm adoption", 0);
    assert!(app.flow_plan.adoption.is_none());
    assert_eq!(app.platform_host.document().revision, before.revision + 1);
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Stale { .. }
    ));
    assert!(!app.background_busy());
    assert!(
        matches!(app.platform_host.modeling_inputs().workspace.flow_plan.current(),FlowPlanConfiguration::EquationOriented(o) if o.released.is_some())
    );
    let export = std::env::temp_dir().join(format!(
        "rf-eo-gui-output-{}.txt",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    app.export_solve_snapshot_to_path(&model, export.clone());
    assert!(!export.exists());
    let path = export.with_extension("json");
    app.request_save_project_as(path.clone(), false);
    assert!(app.pending_format_upgrade.is_some());
    assert!(!path.exists());
    app.confirm_format_upgrade();
    assert!(path.exists());
    app.open_project(path.clone(), "EO test");
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
    configure(&mut app, options(false));
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let model = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap();
    assert!(
        (model.equation_oriented.unwrap().result.streams[&"s-pa".into()].total_molar_flow_mol_s
            - 1.5)
            .abs()
            < 1e-10
    );
    fs::remove_file(path).unwrap();
}
#[test]
fn equation_gui_narrow_bilingual_numeric_draft_blocks_run_and_adoption() {
    let mut app = equation_app();
    let ctx = egui::Context::default();
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let w = &app.platform_host.modeling_inputs().workspace;
    let revision = w.document.revision;
    let mut draft = FlowPlanDraft::new(w, w.flow_plan.current().clone());
    let binding = options(true).target.unwrap().source;
    draft
        .edit_number(
            FlowPlanNumericField::ProductTarget(binding.clone()),
            1.5,
            MeasurementUnit::KilomolePerHour,
        )
        .unwrap()
        .raw = "1e-".into();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
    ctx.style_mut(|s| {
        for font in s.text_styles.values_mut() {
            font.size = 20.0;
        }
    });
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        app.locale = locale;
        let text = frame(&mut app, &ctx, 400.0).join("\n");
        assert!(text.contains("1e-"));
        assert!(!app.background_command_enabled("run_panel.run_manual"));
        let d = app
            .platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .draft()
            .unwrap();
        assert_eq!(
            d.numbers()[&FlowPlanNumericField::ProductTarget(binding.clone())].unit,
            MeasurementUnit::KilomolePerHour
        );
    }
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(!app.background_busy());
    assert_eq!(app.platform_host.document().revision, revision);
    assert!(
        radishflow_studio::equation_actions::preview_equation_adoption(
            app.platform_host.modeling_inputs(),
            FlowEditAccess::Editable,
            false
        )
        .is_err()
    );
    app.locale = StudioShellLocale::En;
    click_text(&mut app, &ctx, "Discard draft", 0);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .pending_count(),
        0
    );
}
#[test]
fn equation_gui_incomplete_structure_and_failed_bracket_never_show_current_values() {
    let mut app = equation_app();
    let ctx = egui::Context::default();
    let mut o = options(true);
    o.target = None;
    configure(&mut app, o);
    let text = frame(&mut app, &ctx, 900.0).join("\n");
    assert!(text.contains("Variables 29"));
    assert!(text.contains("Equations 28"));
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(!app.background_busy());
    let mut o = options(true);
    o.target.as_mut().unwrap().value_mol_s = 3.0;
    configure(&mut app, o);
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let text = frame(&mut app, &ctx, 900.0).join("\n");
    assert!(text.contains("Failed"));
    assert!(text.contains("TargetNotBracketed"), "{text}");
    assert!(!text.contains("Current EO condition"));
    let runtime = app.platform_host.snapshot().window_model().runtime;
    assert!(runtime.latest_solve_snapshot.is_none());
}
