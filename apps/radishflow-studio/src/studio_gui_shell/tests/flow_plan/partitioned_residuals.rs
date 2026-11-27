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
            "Locate project stream s-recycle"
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

#[test]
fn serial_recycle_owns_one_receipt_and_routes_each_tear_by_its_block() {
    let temp = PropertyPackageCacheRoot::temporary("q7-serial-terminal").unwrap();
    let project = include_bytes!(
        "../../../../../../examples/flowsheets/fixed-tp-q7-serial-recycle.rfproj.json"
    );
    for (fraction, success) in [(0.25, true), (0.99, false)] {
        let mut data: serde_json::Value = serde_json::from_slice(project).unwrap();
        data["document"]["flowsheet"]["units"]["splitter-2"]["parameters"]["outlet_a_split_fraction"] =
            fraction.into();
        let path = temp.path().join(format!("serial-{fraction}.json"));
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        let mut app = ready_app_state(&StudioRuntimeConfig {
            project_path: path,
            ..synced_workspace_config()
        });
        app.enable_background_runs();
        app.screen = StudioShellScreen::Run;
        app.locale = StudioShellLocale::En;
        app.dispatch_ui_command("run_panel.run_manual");
        let ctx = egui::Context::default();
        drain(&mut app, &ctx);
        ctx.style_mut(|s| {
            s.animation_time = 0.0;
            for f in s.text_styles.values_mut() {
                f.size = 20.0;
            }
        });
        let labels = frame(&mut app, &ctx, 400.).join("\n");
        let a = labels.find("Block 4 · s-recycle").unwrap();
        let b = labels.find("Block 5 · s-recycle-2").unwrap();
        assert!(a < b);
        assert!(labels.contains("Block 4 → s-bleed → Block 5"));
        assert!(labels[a..b].contains("Iteration evaluations 48 · Independent verification 1"));
        let status = if success {
            "Current · 10 formal stream results published"
        } else {
            "Failed · Block 5 · Iteration budget exhausted · No formal result for this run"
        };
        assert_eq!(labels.matches(status).count(), 1);
        assert!(labels.contains(if success {
            "Downstream heater-1 local calls 1"
        } else {
            "Downstream heater-1 did not run"
        }));
        assert!(labels[a..b].contains(if success {
            "Block verification passed"
        } else {
            "Block verification passed · Diagnostic only"
        }));
        assert!(labels[b..].contains(if success {
            "Iteration evaluations 18 · Independent verification 1"
        } else {
            "Iteration evaluations 500 · Independent verification 0"
        }));
        assert!(labels[b..].contains(if success {
            "Block verification passed"
        } else {
            "Independent verification not run"
        }));
        assert_eq!(labels.contains("View formal stream"), success);
        for (id, samples) in [
            ("s-recycle", 49),
            ("s-recycle-2", if success { 19 } else { 500 }),
        ] {
            let header = format!("Block evidence · {id} ({samples})");
            click_text(&mut app, &ctx, &header, 0);
            assert!(
                frame(&mut app, &ctx, 900.).contains(&format!("Full residual trace ({samples})"))
            );
            let action = if success {
                "View formal stream"
            } else {
                "Locate project stream"
            };
            click_text(&mut app, &ctx, &format!("{action} {id}"), 0);
            assert_eq!(
                app.screen,
                if success {
                    StudioShellScreen::Results
                } else {
                    StudioShellScreen::Workbench
                }
            );
            let window = app.platform_host.snapshot().window_model();
            let target = window.runtime.active_inspector_target.as_ref().unwrap();
            assert_eq!(target.target_id, id);
            app.screen = StudioShellScreen::Run;
        }
    }
}

#[test]
fn downstream_balance_failure_shows_publication_then_unit_before_block_evidence() {
    let temp = PropertyPackageCacheRoot::temporary("q10-unit-balance").unwrap();
    let mut data: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../../../examples/flowsheets/fixed-tp-q7-serial-recycle.rfproj.json"
    ))
    .unwrap();
    data["document"]["flowsheet"]["streams"]["s-fb"]["total_molar_flow_mol_s"] = 537793.0.into();
    let path = temp.path().join("downstream-balance.rfproj.json");
    fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path,
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
    });
    for (locale, publication, cause, verdict) in [
        (
            StudioShellLocale::ZhCn,
            "Failed · 本次无正式结果",
            "依赖块 6 · heater-1 · 单元物料衡算超限",
            "本块复核通过 · 仅诊断",
        ),
        (
            StudioShellLocale::En,
            "Failed · No formal result for this run",
            "Block 6 · heater-1 · Unit material balance exceeds tolerance",
            "Block verification passed · Diagnostic only",
        ),
    ] {
        app.locale = locale;
        let labels = frame(&mut app, &ctx, 400.0);
        let headline = labels.iter().position(|text| text == publication).unwrap();
        assert_eq!(labels[headline + 1], cause);
        assert_eq!(
            labels[headline + 2..]
                .iter()
                .filter(|text| *text == verdict)
                .count(),
            2
        );
        assert!(
            !labels
                .iter()
                .any(|text| text.contains("local or overall material balance"))
        );
    }
}
