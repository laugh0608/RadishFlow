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
