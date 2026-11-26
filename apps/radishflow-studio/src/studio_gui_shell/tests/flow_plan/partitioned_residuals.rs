use super::*;
use radishflow_studio::PropertyPackageCacheRoot;
use rf_solver::partitioned::{
    EvaluationPhase, PartitionedProgress, PartitionedStage, ResidualSample,
};
use rf_types::units::MeasurementUnit;

#[test]
fn progress_keeps_the_phase_and_units_of_the_completed_sample() {
    use crate::studio_gui_shell::flow_plan::partitioned_residuals::progress_text;
    let mut p = PartitionedProgress {
        stage: PartitionedStage::IterationStarted,
        block: Some(3),
        unit: None,
        phase: Some(EvaluationPhase::Iteration),
        evaluation: Some(1),
        residual: None,
    };
    let unit = MeasurementUnit::KilomolePerHour;
    assert!(progress_text(&p, unit, true).contains("尚无已完成求值"));
    p.stage = PartitionedStage::VerificationStarted;
    p.phase = Some(EvaluationPhase::Verification);
    p.residual = Some(ResidualSample {
        phase: EvaluationPhase::Iteration,
        evaluation: 229,
        max_absolute_mol_s: 9.30455712e-11,
        max_scaled: 1.86091142e-11,
    });
    let text = progress_text(&p, unit, false);
    assert!(text.contains("Block 4 · Independent verification 1 · started"));
    assert!(
        text.contains("Last completed · Iteration 229 · Max absolute residual 3.34964e-10 kmol/h")
    );
    assert!(text.contains("1.86091e-11 (dimensionless)"));
}

#[test]
fn real_recycle_runs_show_qualified_terminal_actions_and_narrow_bilingual_verdicts() {
    let temp = PropertyPackageCacheRoot::temporary("s09-terminal").unwrap();
    let base = include_bytes!(
        "../../../../../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json"
    );
    for (fraction, success) in [(0.90, true), (0.99, false)] {
        let mut project: serde_json::Value = serde_json::from_slice(base).unwrap();
        project["document"]["flowsheet"]["units"]["splitter-1"]["parameters"]["outlet_a_split_fraction"] =
            fraction.into();
        let path = temp.path().join(format!("recycle-{fraction}.json"));
        fs::write(&path, serde_json::to_vec(&project).unwrap()).unwrap();
        let mut app = ready_app_state(&StudioRuntimeConfig {
            project_path: path.clone(),
            ..synced_workspace_config()
        });
        app.enable_background_runs();
        app.screen = StudioShellScreen::Run;
        app.dispatch_ui_command("run_panel.run_manual");
        let ctx = egui::Context::default();
        drain(&mut app, &ctx);
        ctx.style_mut(|style| {
            for font in style.text_styles.values_mut() {
                font.size = 20.0;
            }
            style.animation_time = 0.0;
        });
        for (locale, expected) in [
            (
                StudioShellLocale::ZhCn,
                if success {
                    "候选前一次 · 迭代 228"
                } else {
                    "独立复核未执行"
                },
            ),
            (
                StudioShellLocale::En,
                if success {
                    "Before candidate · Iteration 228"
                } else {
                    "Independent verification not run"
                },
            ),
        ] {
            app.locale = locale;
            let text = frame(&mut app, &ctx, 400.0).join("\n");
            assert!(text.contains(expected), "{text}");
            assert!(text.contains("radishflow.partitioned-sm/1"), "{text}");
            assert!(text.contains(if success {
                "1.03384e-10 mol/s"
            } else {
                "0.0183974 mol/s"
            }));
            assert!(text.contains(if success { "(230)" } else { "(500)" }));
        }
        let label = if success {
            "View formal stream s-recycle"
        } else {
            "Locate tear stream s-recycle"
        };
        click_text(&mut app, &ctx, label, 0);
        assert_eq!(
            app.screen,
            if success {
                StudioShellScreen::Results
            } else {
                StudioShellScreen::Workbench
            }
        );
        let returned = |app: &mut ReadyAppState| {
            let out = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| app.render_recycle_return(ui));
            });
            let mut labels = vec![];
            for shape in out.shapes {
                collect_shape_texts(&shape.shape, &mut labels);
            }
            labels.join("\n")
        };
        assert!(returned(&mut app).contains("Back to recycle residuals"));
        app.dispatch_ui_command("run_panel.run_manual");
        // The existing receipt remains qualified until the new worker is received.
        assert!(returned(&mut app).contains("Back to recycle residuals"));
        drain(&mut app, &ctx);
        assert!(!returned(&mut app).contains("Back to recycle residuals"));
        app.screen = StudioShellScreen::Run;
        click_text(&mut app, &ctx, label, 0);
        assert!(returned(&mut app).contains("Back to recycle residuals"));
        if success {
            app.select_canvas_stream("inspector.focus_stream:s-recycle");
            app.dispatch_ui_command("canvas.delete_selected_stream");
            assert!(matches!(
                app.platform_host.modeling_inputs().workspace.flow_result(),
                rf_ui::result_query::FlowResultView::Stale { .. }
            ));
        } else {
            app.open_project(path, "reopen same recycle document");
            assert!(matches!(
                app.platform_host.modeling_inputs().workspace.flow_result(),
                rf_ui::result_query::FlowResultView::Missing { .. }
            ));
        }
        assert!(!returned(&mut app).contains("Back to recycle residuals"));
    }
}
