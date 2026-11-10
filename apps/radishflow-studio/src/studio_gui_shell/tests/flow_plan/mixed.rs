use super::*;
use rf_types::units::MeasurementUnit;
use rf_ui::result_query::FlowResultView;
fn mixed_app() -> ReadyAppState {
    let mut config = synced_workspace_config();
    config.project_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets/fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json");
    let mut app = ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    app.screen = StudioShellScreen::Run;
    app.enable_background_runs();
    app
}
#[test]
fn mixed_gui_run_block_evidence_plan_stale_save_reopen_and_rerun() {
    let mut app = mixed_app();
    let ctx = egui::Context::default();
    let text = frame(&mut app, &ctx, 900.0).join("\n");
    assert!(text.contains("Fixed-T/P mixed material"), "{text}");
    assert!(text.contains("24 variables / 24 equations"));
    assert!(text.contains("feed-a → feed-b → equations → heater"));
    click_text(&mut app, &ctx, "Run F5", 0);
    drain(&mut app, &ctx);
    let model = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap();
    assert!(model.mixed.is_some());
    assert_eq!(model.final_unit_results().unwrap().len(), 7);
    let exported = model.light_text_export().unwrap();
    assert!(exported.contains("Boundary s-pa"));
    assert!(exported.contains("core_method: radishflow.fixed-tp-mixed-material/1"));
    let text = frame(&mut app, &ctx, 900.0).join("\n");
    assert!(text.contains("Complete mixed-partition result"));
    assert!(text.contains("3 actual block boundaries"));
    let FlowPlanConfiguration::Mixed(mut o) = app
        .platform_host
        .modeling_inputs()
        .workspace
        .flow_plan
        .current()
        .clone()
    else {
        panic!()
    };
    o.blocks.reverse();
    let rev = app
        .platform_host
        .modeling_inputs()
        .workspace
        .flow_plan
        .revision();
    app.background_action(Action::ConfigureFlowPlan {
        expected: FlowPlanEditContext::capture(&app.platform_host.modeling_inputs().workspace),
        access: FlowEditAccess::Editable,
        command: FlowPlanCommand::Apply(FlowPlanConfiguration::Mixed(o.clone())),
    });
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .revision(),
        rev
    );
    o.blocks
        .iter_mut()
        .find(|b| b.id.as_str() == "heater")
        .unwrap()
        .id = "heater-final".into();
    app.background_action(Action::ConfigureFlowPlan {
        expected: FlowPlanEditContext::capture(&app.platform_host.modeling_inputs().workspace),
        access: FlowEditAccess::Editable,
        command: FlowPlanCommand::Apply(FlowPlanConfiguration::Mixed(o)),
    });
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Stale { .. }
    ));
    let rejected = std::env::temp_dir().join(format!(
        "rf-qs4-old-output-{}.txt",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    app.export_solve_snapshot_to_path(&model, rejected.clone());
    assert!(!rejected.exists());
    let path = std::env::temp_dir().join(format!(
        "rf-qs4-gui-{}.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    app.request_save_project_as(path.clone(), false);
    assert!(path.exists());
    app.open_project(path.clone(), "QS4 test");
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Current { .. }
    ));
    fs::remove_file(path).unwrap();
}
#[test]
fn mixed_gui_bilingual_narrow_drafts_preserve_raw_units_and_block_f5() {
    let mut app = mixed_app();
    let ctx = egui::Context::default();
    let w = &app.platform_host.modeling_inputs().workspace;
    let physical = w.document.clone();
    let mut draft = FlowPlanDraft::new(w, w.flow_plan.current().clone());
    draft
        .edit_number(
            FlowPlanNumericField::Pressure,
            650000.0,
            MeasurementUnit::Kilopascal,
        )
        .unwrap()
        .raw = "1e-".into();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
    ctx.style_mut(|s| {
        for f in s.text_styles.values_mut() {
            f.size = 20.0;
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
            d.numbers()[&FlowPlanNumericField::Pressure].unit,
            MeasurementUnit::Kilopascal
        );
    }
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(!app.background_busy());
    assert_eq!(app.platform_host.document(), &physical);
    click_text(&mut app, &ctx, "Discard draft", 0);
    assert!(app.background_command_enabled("run_panel.run_manual"));
}
