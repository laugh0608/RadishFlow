use super::*;
use rf_solver::simultaneous::*;
use rf_types::units::MeasurementUnit;
use rf_ui::result_query::FlowResultView;

fn modular_app() -> ReadyAppState {
    let mut config = synced_workspace_config();
    config.project_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets/fixed-tp-cross-recycle-binary-hydrocarbon.rfproj.json");
    let mut app = ready_app_state(&config);
    assert!(app.read_only_project.is_none());
    app.locale = StudioShellLocale::En;
    app.screen = StudioShellScreen::Run;
    app.enable_background_runs();
    app
}
fn configure(app: &mut ReadyAppState, plan: FlowPlanConfiguration) {
    app.background_action(Action::ConfigureFlowPlan {
        expected: FlowPlanEditContext::capture(&app.platform_host.modeling_inputs().workspace),
        access: FlowEditAccess::Editable,
        command: FlowPlanCommand::Apply(plan),
    });
}
fn explicit(app: &ReadyAppState) -> FlowPlanConfiguration {
    let FlowPlanConfiguration::Simultaneous(mut options) = app
        .platform_host
        .modeling_inputs()
        .workspace
        .flow_plan
        .current()
        .clone()
    else {
        panic!()
    };
    options.scales = Some(
        assemble(&app.platform_host.document().flowsheet, &options)
            .unwrap()
            .blocks()
            .iter()
            .flat_map(|b| &b.couplings)
            .cloned()
            .map(|b| {
                (
                    b,
                    CouplingScale {
                        variable_mol_s: 2.0,
                        residual_mol_s: 3.0,
                    },
                )
            })
            .collect(),
    );
    FlowPlanConfiguration::Simultaneous(options)
}
fn temp(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rf-qs2-gui-{label}-{}.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn modular_gui_default_run_explicit_stale_save_reopen_rerun_and_output() {
    let mut app = modular_app();
    let ctx = egui::Context::default();
    let text = frame(&mut app, &ctx, 900.0).join("\n");
    assert!(text.contains("Fixed-T/P affine modular solve"));
    assert!(text.contains("8 unknowns"));
    assert!(!text.contains("Iteration budget"));
    click_text(&mut app, &ctx, "Run F5", 0);
    drain(&mut app, &ctx);
    let model = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap();
    assert_eq!(
        model
            .simultaneous
            .as_ref()
            .unwrap()
            .result
            .costs
            .local_unit_calls,
        10
    );
    assert_eq!(model.block_count(), Some(3));
    assert_eq!(
        model
            .result_inspector_with_unit(None, None, Some("mixer-1"))
            .unit_options
            .len(),
        6
    );
    let path = temp("export").with_extension("txt");
    app.export_solve_snapshot_to_path(&model, path.clone());
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("Variable scale:"));
    assert!(text.contains("LU factorizations: 1"));
    fs::remove_file(&path).unwrap();
    configure(&mut app, explicit(&modular_app()));
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Stale { .. }
    ));
    app.export_solve_snapshot_to_path(&model, path.clone());
    assert!(!path.exists());
    let saved = temp("save");
    app.request_save_project_as(saved.clone(), false);
    assert!(saved.exists());
    app.open_project(saved.clone(), "test");
    assert!(app.read_only_project.is_none());
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let model = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap();
    let result = &model.simultaneous.as_ref().unwrap().result;
    assert_eq!(
        result
            .blocks
            .last()
            .unwrap()
            .coupling
            .as_ref()
            .unwrap()
            .connections
            .len(),
        8
    );
    assert!((result.streams[&"s-ma".into()].total_molar_flow_mol_s - 55.0 / 17.0).abs() < 1e-10);
    fs::remove_file(saved).unwrap();
}

#[test]
fn modular_gui_unfinished_scale_draft_blocks_run_output_preserves_units_and_missing_binding() {
    let mut app = modular_app();
    let ctx = egui::Context::default();
    configure(&mut app, explicit(&modular_app()));
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let model = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap();
    let w = &app.platform_host.modeling_inputs().workspace;
    let FlowPlanConfiguration::Simultaneous(mut options) = w.flow_plan.current().clone() else {
        panic!()
    };
    let binding = options
        .scales
        .as_ref()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    let mut invalid = binding.clone();
    invalid.source.unit_id = "removed-source-very-long-identity".into();
    options.scales.as_mut().unwrap().insert(
        invalid.clone(),
        CouplingScale {
            variable_mol_s: 1.0,
            residual_mol_s: 1.0,
        },
    );
    let mut draft = FlowPlanDraft::new(w, FlowPlanConfiguration::Simultaneous(options));
    let field = FlowPlanNumericField::CouplingResidualScale(invalid);
    draft
        .edit_number(field.clone(), 1.0, MeasurementUnit::MolePerSecond)
        .unwrap()
        .raw = "1e-".into();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
    for locale in [StudioShellLocale::En, StudioShellLocale::ZhCn] {
        app.locale = locale;
        let text = frame(&mut app, &ctx, 400.0).join("\n");
        assert!(text.contains("1e-"));
        assert!(text.contains("removed-source-very-long-identity"));
        assert!(text.contains("mol/s"));
    }
    assert!(!app.background_command_enabled("run_panel.run_manual"));
    let before = model.snapshot_id.clone();
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(!app.background_busy());
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .latest_solve_snapshot
            .unwrap()
            .snapshot_id,
        before
    );
    let path = temp("blocked").with_extension("txt");
    app.export_solve_snapshot_to_path(&model, path.clone());
    assert!(!path.exists());
    let w = &app.platform_host.modeling_inputs().workspace;
    let d = w.flow_plan.draft().unwrap();
    assert_eq!(d.numbers()[&field].raw, "1e-");
    assert!(d.configuration().is_err());
}

#[test]
fn modular_gui_method_picker_explicit_draft_apply_undo_redo_uses_real_clicks() {
    let mut app = modular_app();
    let ctx = egui::Context::default();
    configure(&mut app, FlowPlanConfiguration::Acyclic);
    click_text(&mut app, &ctx, "Acyclic sequential", 0);
    click_text(&mut app, &ctx, "Fixed-T/P affine modular solve", 0);
    click_text(&mut app, &ctx, "Create explicit scale draft", 0);
    click_text(&mut app, &ctx, "Apply plan", 0);
    let w = &app.platform_host.modeling_inputs().workspace;
    assert!(
        matches!(w.flow_plan.current(),FlowPlanConfiguration::Simultaneous(o) if o.scales.as_ref().unwrap().len()==8)
    );
    click_text(&mut app, &ctx, "Undo plan", 0);
    assert!(matches!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .current(),
        FlowPlanConfiguration::Acyclic
    ));
    click_text(&mut app, &ctx, "Redo plan", 0);
    assert!(matches!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .current(),
        FlowPlanConfiguration::Simultaneous(_)
    ));
}

#[test]
fn modular_gui_same_v4_save_as_capability_prompt_cancel_and_confirm() {
    let mut app = super::app();
    let ctx = egui::Context::default();
    configure(
        &mut app,
        FlowPlanConfiguration::Simultaneous(ModularOptions::new(FixedBoundary {
            temperature_k: 300.0,
            pressure_pa: 650000.0,
        })),
    );
    let path = temp("capability");
    app.request_save_project_as(path.clone(), false);
    assert!(app.pending_format_upgrade.is_some());
    assert!(!path.exists());
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(900.0, 1000.0),
        )),
        ..Default::default()
    };
    let _ = ctx.run(input.clone(), |ctx| {
        app.render_format_upgrade(ctx);
    });
    let out = ctx.run(input, |ctx| {
        app.render_format_upgrade(ctx);
    });
    let mut text = Vec::new();
    for shape in out.shapes {
        collect_shape_texts(&shape.shape, &mut text);
    }
    assert!(text.join("\n").contains("project remains v4"), "{text:?}");
    assert!(
        text.join("\n")
            .contains("radishflow.solver.fixed-tp-affine-modular-lu")
    );
    app.cancel_pending_save_as_overwrite();
    assert!(!path.exists());
    app.request_save_project_as(path.clone(), false);
    app.confirm_format_upgrade();
    assert!(path.exists());
    let opened = radishflow_studio::load_project_app_state(&path).unwrap();
    assert!(matches!(
        opened.workspace.flow_plan.current(),
        FlowPlanConfiguration::Simultaneous(_)
    ));
    fs::remove_file(path).unwrap();
}

fn write_fraction(app: &mut ReadyAppState, unit: &str, value: f64) {
    use rf_ui::variable_browser::*;
    let w = &app.platform_host.modeling_inputs().workspace;
    app.background_action(Action::EditFlow {
        access: FlowEditAccess::Editable,
        request: radishflow_studio::flow_actions::FlowActionRequest {
            expected: radishflow_studio::flow_actions::FlowEditContext::capture(w),
            action: radishflow_studio::flow_actions::FlowAction::Write {
                variable: VariableId {
                    document: w.document.metadata.document_id.clone(),
                    object: ObjectId::Unit(unit.into()),
                    section: VariableSection::Inputs,
                    field: VariableField::OutletASplitFraction,
                },
                value: VariableValue::Number(value),
            },
        },
    });
}
#[test]
fn modular_gui_failed_result_keeps_diagnostics_out_of_current_fields() {
    let mut app = modular_app();
    let ctx = egui::Context::default();
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let old = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap();
    for id in ["splitter-1", "splitter-2"] {
        write_fraction(&mut app, id, 0.999999999999);
    }
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let w = &app.platform_host.modeling_inputs().workspace;
    assert!(matches!(w.flow_result(), FlowResultView::Failed { .. }));
    let failure = w.simultaneous_failure_evidence().unwrap();
    assert_eq!(failure.kind, ModularFailureKind::IllConditioned);
    assert!(
        failure
            .active_coupling
            .as_ref()
            .unwrap()
            .linear
            .as_ref()
            .unwrap()
            .backward_error
            .is_none()
    );
    let text = frame(&mut app, &ctx, 400.0).join("\n");
    assert!(text.contains("Failed"));
    assert!(text.contains("not computed"));
    assert!(!text.contains("Current"));
    let path = temp("failed").with_extension("txt");
    app.export_solve_snapshot_to_path(&old, path.clone());
    assert!(!path.exists());
}

#[test]
fn modular_unknown_capability_stays_entirely_read_only_and_recent_metadata_is_safe() {
    let path = temp("unknown");
    let original =
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../examples/flowsheets/fixed-tp-cross-recycle-binary-hydrocarbon.rfproj.json",
        ))
        .unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&original).unwrap();
    json["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"id":"future.modular","version":1}));
    let bytes = serde_json::to_vec(&json).unwrap();
    fs::write(&path, &bytes).unwrap();
    let mut app = modular_app();
    app.open_project(path.clone(), "test");
    assert!(app.read_only_project.is_some());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::remove_file(path).unwrap();
}

fn assert_horizontal_fit(shape: &egui::Shape, clip: egui::Rect, case: &str) {
    match shape {
        egui::Shape::Text(t) => {
            let r = t.galley.rect.translate(t.pos.to_vec2());
            assert!(
                r.left() >= clip.left() - 1.0 && r.right() <= clip.right() + 1.0,
                "{case}: {:?} outside {clip:?}: {r:?}",
                t.galley.text()
            );
        }
        egui::Shape::Vec(v) => {
            for s in v {
                assert_horizontal_fit(s, clip, case);
            }
        }
        _ => {}
    }
}
#[test]
fn modular_400px_bilingual_large_font_layouts_keep_text_within_horizontal_bounds() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for scale in [1.0, 2.0] {
            let mut app = modular_app();
            app.locale = locale;
            let ctx = egui::Context::default();
            fonts::configure_studio_fonts(&ctx);
            ctx.style_mut(|s| {
                for font in s.text_styles.values_mut() {
                    font.size *= scale;
                }
            });
            app.dispatch_ui_command("run_panel.run_manual");
            drain(&mut app, &ctx);
            for explicit_scales in [false, true] {
                if explicit_scales {
                    configure(&mut app, explicit(&modular_app()));
                }
                for _ in 0..2 {
                    let out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(400.0, 18000.0),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::CentralPanel::default().show(ctx, |ui| app.render_flow_plan(ui));
                        },
                    );
                    let case = format!("{locale:?} scale {scale} explicit {explicit_scales}");
                    for shape in out.shapes {
                        assert_horizontal_fit(&shape.shape, shape.clip_rect, &case);
                    }
                }
            }
        }
    }
}
#[test]
fn modular_keyboard_tab_shift_tab_and_escape_preserve_the_plan_draft() {
    let mut app = modular_app();
    let ctx = egui::Context::default();
    let w = &app.platform_host.modeling_inputs().workspace;
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
    output(&mut app, &ctx, vec![]);
    let key = |key, shift| egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            shift,
            ..Default::default()
        },
    };
    output(&mut app, &ctx, vec![key(egui::Key::Tab, false)]);
    let first = ctx.memory(|m| m.focused());
    assert!(first.is_some());
    output(&mut app, &ctx, vec![key(egui::Key::Tab, false)]);
    let second = ctx.memory(|m| m.focused());
    assert_ne!(first, second);
    output(&mut app, &ctx, vec![key(egui::Key::Tab, true)]);
    output(&mut app, &ctx, vec![]);
    assert_eq!(ctx.memory(|m| m.focused()), first);
    output(&mut app, &ctx, vec![key(egui::Key::Escape, false)]);
    assert_eq!(app.screen, StudioShellScreen::Workbench);
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

#[test]
fn modular_cancel_waits_for_exit_and_preserves_previous_current_result() {
    let mut app = modular_app();
    let ctx = egui::Context::default();
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let old = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap()
        .snapshot_id;
    app.dispatch_ui_command("run_panel.run_manual");
    let activity = app
        .background_runs
        .as_ref()
        .unwrap()
        .borrow()
        .activity()
        .unwrap();
    app.background_action(Action::Cancel(activity.id));
    assert!(app.background_busy());
    assert!(!app.background_command_enabled("run_panel.run_manual"));
    let out = ctx.run(egui::RawInput::default(), |ctx| {
        app.render_background_activity(ctx)
    });
    let mut text = Vec::new();
    for shape in out.shapes {
        collect_shape_texts(&shape.shape, &mut text);
    }
    assert!(text.join("\n").contains("Cancelling"));
    drain(&mut app, &ctx);
    assert_eq!(
        app.background_runs
            .as_ref()
            .unwrap()
            .borrow()
            .history()
            .back()
            .unwrap()
            .outcome,
        radishflow_studio::StudioRunOutcome::Cancelled
    );
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .latest_solve_snapshot
            .unwrap()
            .snapshot_id,
        old
    );
}

#[test]
fn modular_gui_exact_connection_repairs_cross_recycle_before_run() {
    use radishflow_studio::{flow_actions::*, modeling_actions::MaterialPortTarget};
    let mut app = modular_app();
    let ctx = egui::Context::default();
    let expected = FlowEditContext::capture(&app.platform_host.modeling_inputs().workspace);
    app.background_action(Action::EditFlow {
        access: FlowEditAccess::Editable,
        request: FlowActionRequest {
            expected,
            action: FlowAction::Disconnect(MaterialPortTarget {
                unit_id: "mixer-1".into(),
                port: "inlet_b".into(),
            }),
        },
    });
    click_text(&mut app, &ctx, "Modeling and connections", 0);
    click_text(&mut app, &ctx, "—", 0);
    click_text(&mut app, &ctx, "splitter-2.outlet_a", 0);
    click_text(&mut app, &ctx, "—", 0);
    click_text(&mut app, &ctx, "mixer-1.inlet_b", 0);
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
    output(&mut app, &ctx, vec![egui::Event::Text("s-rba".into())]);
    click_text(&mut app, &ctx, "Confirm connection", 0);
    click_text(&mut app, &ctx, "Run F5", 0);
    drain(&mut app, &ctx);
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Current {
            snapshot: rf_ui::FlowResultRecord::Simultaneous(_),
            ..
        }
    ));
}
