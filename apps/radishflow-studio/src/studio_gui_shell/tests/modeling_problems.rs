use super::*;
use radishflow_studio::modeling_specifications::*;
use rf_ui::variable_browser::{ObjectId, VariableField};

fn numeric(app: &ReadyAppState, key: &str) -> rf_ui::NumericFieldPresentation {
    app.platform_host
        .snapshot()
        .runtime
        .active_inspector_detail
        .unwrap()
        .property_fields
        .into_iter()
        .find(|f| f.key == key)
        .unwrap()
        .numeric
        .unwrap()
        .unwrap()
}

fn field_target(unit: &str, field: VariableField) -> SpecificationTarget {
    SpecificationTarget::Field {
        object: ObjectId::Unit(UnitId::new(unit)),
        field,
    }
}

fn commit_source_pressure(app: &mut ReadyAppState, pressure: &str) {
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = "stream:stream-feed:pressure_pa";
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        pressure,
    );
    let command = app
        .platform_host
        .snapshot()
        .runtime
        .active_inspector_detail
        .unwrap()
        .property_fields
        .into_iter()
        .find(|f| f.key == key)
        .unwrap()
        .commit_command_id
        .unwrap();
    app.dispatch_inspector_field_draft_commit(command);
}

#[test]
fn formal_edits_history_and_runs_share_current_problems_without_changing_result_gates() {
    let mut app = ready_app_state(&synced_workspace_config());
    assert_eq!(app.modeling_problems.checks().current_count(), Some(0));
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_some()
    );
    commit_source_pressure(&mut app, "85000");
    assert_eq!(app.modeling_problems.checks().current_count(), Some(1));
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_none()
    );
    let key = app.modeling_problems.checks().entries()[0].key.clone();
    app.dispatch_ui_command("edit.undo");
    assert_eq!(app.modeling_problems.checks().current_count(), Some(0));
    app.dispatch_ui_command("edit.redo");
    assert_eq!(app.modeling_problems.checks().entries()[0].key, key);
    app.open_modeling_problems();
    let before = app.platform_host.document().clone();
    app.modeling_problems
        .recommendations
        .recheck(app.platform_host.modeling_inputs());
    assert_eq!(app.platform_host.document(), &before);
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_none()
    );
    app.dispatch_ui_command("run_panel.run_manual");
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .control_state
            .run_status,
        rf_ui::RunStatus::Error
    );
    assert_eq!(
        app.modeling_problems.checks().current_count(),
        Some(1),
        "runtime presentation does not double-count a preflight problem"
    );
    commit_source_pressure(&mut app, "120000");
    assert_eq!(app.modeling_problems.checks().current_count(), Some(0));
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_none(),
        "a clean check is not a solve"
    );
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_some()
    );
}

#[test]
fn exact_field_navigation_and_return_preserve_incomplete_text_unit_and_session() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("inspector.focus_unit:heater-1");
    let key = "unit:heater-1:outlet_pressure_pa";
    let mut pressure = numeric(&app, key);
    app.edit_numeric_field(
        &mut pressure,
        rf_ui::NumericEditEvent::SelectInputUnit(rf_types::units::MeasurementUnit::Bar),
    );
    app.edit_numeric_field(
        &mut pressure,
        rf_ui::NumericEditEvent::ReplaceText("1e-".into()),
    );
    let before = app.platform_host.snapshot();
    let report = app.modeling_problems.checks().clone();
    assert!(
        app.locate_specification_target(field_target("flash-1", VariableField::OutletTemperature))
    );
    assert!(app.pending_input_action.is_none());
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .input_edits,
        before.runtime.workspace_document.input_edits
    );
    assert_eq!(app.modeling_problems.checks(), &report);
    let target_field = numeric(&app, "unit:flash-1:outlet_temperature_k");
    let ctx = egui::Context::default();
    let mut target_id = egui::Id::NULL;
    let _ = ctx.run(
        egui::RawInput {
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                target_id = ui.make_persistent_id((
                    "numeric-input",
                    &target_field.variable.document,
                    "unit:flash-1:outlet_temperature_k",
                ));
                app.render_numeric_property_field(
                    ui,
                    "unit:flash-1:outlet_temperature_k",
                    "Outlet temperature",
                    &target_field,
                );
            });
        },
    );
    assert!(ctx.memory(|m| m.has_focus(target_id)));
    app.open_modeling_problems();
    assert!(
        app.locate_specification_target(field_target("heater-1", VariableField::OutletPressure))
    );
    let after = numeric(&app, key);
    assert_eq!(after.text, "1e-");
    assert_eq!(after.input_unit, rf_types::units::MeasurementUnit::Bar);
    assert_eq!(after.generation, pressure.generation);
    assert_eq!(
        app.platform_host.document().revision,
        before.runtime.workspace_document.revision
    );
}

#[test]
fn navigation_rejects_removed_targets_and_reopening_resets_check_identity() {
    let mut app = ready_app_state(&synced_workspace_config());
    let before = app.modeling_problems.checks().identity().clone();
    assert!(
        !app.locate_specification_target(field_target(
            "missing-unit",
            VariableField::OutletPressure
        ))
    );
    app.open_example_project(synced_workspace_config().project_path);
    assert_eq!(
        app.modeling_problems.checks().identity().document_id,
        before.document_id
    );
    assert_ne!(
        app.modeling_problems.checks().identity().session,
        before.session
    );
}

#[test]
fn saving_and_reopening_rebuilds_missing_and_repaired_specifications_without_cache() {
    let path = std::env::temp_dir().join(format!(
        "radishflow-a1-missing-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut project = feed_heater_flash_binary_hydrocarbon_project();
    project
        .document
        .flowsheet
        .units
        .get_mut(&UnitId::new("heater-1"))
        .unwrap()
        .parameters
        .outlet_temperature_k = None;
    write_project_file(&path, &project).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    assert_eq!(app.modeling_problems.checks().current_count(), Some(1));
    save_project_with_upgrade_consent(&mut app);
    let before = app.modeling_problems.checks().identity().clone();
    app.open_project(path.clone(), "project");
    assert_ne!(
        app.modeling_problems.checks().identity().session,
        before.session
    );
    assert_eq!(app.modeling_problems.checks().current_count(), Some(1));
    assert!(
        app.locate_specification_target(field_target("heater-1", VariableField::OutletTemperature))
    );
    let key = "unit:heater-1:outlet_temperature_k";
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(key),
        "345",
    );
    assert_eq!(
        app.modeling_problems.checks().current_count(),
        Some(1),
        "a draft does not complete a specification"
    );
    app.dispatch_inspector_field_draft_commit(
        radishflow_studio::inspector_draft_commit_command_id(key),
    );
    assert_eq!(app.modeling_problems.checks().current_count(), Some(0));
    save_project_with_upgrade_consent(&mut app);
    let saved = read_project_file(&path).unwrap();
    assert_eq!(
        saved.document.flowsheet.units[&UnitId::new("heater-1")]
            .parameters
            .outlet_temperature_k,
        Some(345.)
    );
    let raw = fs::read_to_string(&path).unwrap();
    assert!(!raw.contains("specification") && !raw.contains("checkGeneration"));
    app.open_project(path.clone(), "project");
    assert_eq!(app.modeling_problems.checks().current_count(), Some(0));
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_none()
    );
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_some()
    );
    fs::remove_file(path).unwrap();
}

fn panel(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 2000.),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let window = app.platform_host.snapshot().window_model();
                app.render_modeling_problems(ui, &window);
            });
        },
    )
}

#[test]
fn narrow_destination_has_one_editor_and_navigation_enter_does_not_apply() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.open_modeling_problems();
    app.locate_specification_target(field_target("heater-1", VariableField::OutletPressure));
    let before = app.platform_host.document().clone();
    let ctx = egui::Context::default();
    fonts::configure_studio_fonts(&ctx);
    for frame in 0..4 {
        let window = app.platform_host.snapshot().window_model();
        let output = ctx.run(
            egui::RawInput {
                time: Some(frame as f64 * 0.25),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1280., 860.),
                )),
                focused: true,
                events: if frame == 0 {
                    vec![egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }]
                } else {
                    vec![]
                },
                ..Default::default()
            },
            |ctx| {
                let mut hovered = false;
                app.render_left_sidebar(ctx, &window, &mut hovered);
                app.render_right_sidebar(ctx, &window, &mut hovered);
                app.render_bottom_status_bar(ctx, &window);
                app.render_bottom_drawer(ctx, &window);
            },
        );
        assert!(app.modeling_problem_fields_embedded());
        let mut rendered = vec![];
        for shape in &output.shapes {
            texts(&shape.shape, &mut rendered);
        }
        if frame >= 2 {
            assert_eq!(
                rendered
                    .iter()
                    .filter(|(text, _)| text == "出口压力")
                    .count(),
                1,
                "one visible destination editor after scrolling: {rendered:?}"
            );
        }
    }
    assert_eq!(app.platform_host.document(), &before);
    let field = numeric(&app, "unit:heater-1:outlet_pressure_pa");
    assert!(
        field.issue.is_none(),
        "navigation must not attempt a commit"
    );
    app.open_modeling_problems();
    assert!(!app.modeling_problem_fields_embedded());
}

fn texts(shape: &egui::epaint::Shape, out: &mut Vec<(String, egui::Rect)>) {
    match shape {
        egui::epaint::Shape::Text(t) => out.push((
            t.galley.job.text.clone(),
            t.galley.rect.translate(t.pos.to_vec2()),
        )),
        egui::epaint::Shape::Vec(shapes) => {
            for shape in shapes {
                texts(shape, out);
            }
        }
        _ => (),
    }
}

#[test]
fn problem_selection_is_read_only_and_localized_panel_fits_narrow_and_large_text() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for width in [400., 1100.] {
            for scale in [1., 2.] {
                let mut app = ready_app_state(&synced_workspace_config());
                app.locale = locale;
                commit_source_pressure(&mut app, "85000");
                let before = app.platform_host.snapshot();
                let ctx = egui::Context::default();
                fonts::configure_studio_fonts(&ctx);
                ctx.style_mut(|style| {
                    for font in style.text_styles.values_mut() {
                        font.size *= scale;
                    }
                });
                let mut output = panel(&mut app, &ctx, width, vec![]);
                let mut rendered = Vec::new();
                for s in &output.shapes {
                    texts(&s.shape, &mut rendered);
                }
                let row = rendered
                    .iter()
                    .find(|(text, _)| text.starts_with("! "))
                    .unwrap()
                    .1
                    .center();
                output = panel(
                    &mut app,
                    &ctx,
                    width,
                    vec![
                        egui::Event::PointerMoved(row),
                        egui::Event::PointerButton {
                            pos: row,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        },
                        egui::Event::PointerButton {
                            pos: row,
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers: egui::Modifiers::NONE,
                        },
                    ],
                );
                let _ = output;
                output = panel(&mut app, &ctx, width, vec![]);
                assert_eq!(
                    app.platform_host.snapshot().runtime.active_inspector_target,
                    before.runtime.active_inspector_target
                );
                assert_eq!(
                    app.platform_host.document().revision,
                    before.runtime.workspace_document.revision
                );
                rendered.clear();
                for s in &output.shapes {
                    texts(&s.shape, &mut rendered);
                }
                assert!(rendered.iter().any(|(text, _)| text.contains(
                    if locale == StudioShellLocale::ZhCn {
                        "已提交出口"
                    } else {
                        "Committed outlet"
                    }
                )));
                for (text, rect) in rendered {
                    assert!(
                        rect.left() >= -0.5 && rect.right() <= width + 0.5,
                        "{locale:?} width {width} scale {scale}: {text} outside {rect:?}"
                    );
                }
            }
        }
    }
}
