use super::*;
use radishflow_studio::modeling_recommendations::*;
use radishflow_studio::modeling_specifications::SpecificationTarget;

fn normalization(app: &mut ReadyAppState) -> RecommendationKey {
    app.refresh_modeling_problems();
    app.modeling_problems
        .recommendations
        .entries()
        .iter()
        .find(|e| e.recommendation.key.rule == RecommendationRule::NormalizeComposition)
        .unwrap()
        .recommendation
        .key
        .clone()
}

fn fraction(app: &mut ReadyAppState, component: &str, value: &str) {
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let key = format!("stream:stream-feed:overall_mole_fraction:{component}");
    app.dispatch_inspector_field_draft_update(
        radishflow_studio::inspector_draft_update_command_id(&key),
        value,
    );
    app.dispatch_inspector_field_draft_commit(
        radishflow_studio::inspector_draft_commit_command_id(&key),
    );
}

fn unnormalized(app: &mut ReadyAppState) -> RecommendationKey {
    fraction(app, "methane", "0.2");
    fraction(app, "ethane", "0.3");
    normalization(app)
}

fn composition(app: &ReadyAppState) -> Vec<f64> {
    let stream = &app.platform_host.document().flowsheet.streams[&StreamId::new("stream-feed")];
    ["methane", "ethane"]
        .iter()
        .map(|id| stream.overall_mole_fractions[&rf_types::ComponentId::new(*id)])
        .collect()
}

fn texts(shape: &egui::epaint::Shape, out: &mut Vec<(String, egui::Rect)>) {
    match shape {
        egui::epaint::Shape::Text(t) => out.push((
            t.galley.text().into(),
            t.galley.rect.translate(t.pos.to_vec2()),
        )),
        egui::epaint::Shape::Vec(v) => {
            for s in v {
                texts(s, out);
            }
        }
        _ => {}
    }
}

fn panel(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    width: f32,
    events: Vec<egui::Event>,
) -> Vec<(String, egui::Rect)> {
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 2500.),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |ctx| {
            app.dispatch_shortcuts(ctx);
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_recommendation_review(ui);
            });
        },
    );
    let mut rendered = vec![];
    for shape in output.shapes {
        texts(&shape.shape, &mut rendered);
    }
    rendered
}

#[test]
fn normalization_is_one_explicit_transaction_and_does_not_restore_result_qualification() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_some()
    );
    let key = unnormalized(&mut app);
    let before = app.platform_host.document().clone();
    app.preview_modeling_recommendation(&key);
    assert_eq!(app.platform_host.document(), &before);
    assert_eq!(composition(&app), [0.2, 0.3]);
    app.accept_modeling_recommendation();
    assert_eq!(composition(&app), [0.4, 0.6]);
    assert_eq!(app.platform_host.document().revision, before.revision + 1);
    assert_eq!(app.modeling_problems.checks().current_count(), Some(0));
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .latest_solve_snapshot
            .is_none()
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .stale_solve_snapshot
            .is_some()
    );
    app.accept_modeling_recommendation();
    assert_eq!(app.platform_host.document().revision, before.revision + 1);
    app.dispatch_ui_command("edit.undo");
    assert_eq!(composition(&app), [0.2, 0.3]);
    app.dispatch_ui_command("edit.redo");
    assert_eq!(composition(&app), [0.4, 0.6]);
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
fn canvas_and_problem_entries_share_ignore_restore_and_preview_identity() {
    let (config, path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    let id = app
        .platform_host
        .snapshot()
        .window_model()
        .canvas
        .focused_suggestion_id
        .unwrap();
    let key = app.canvas_recommendation_key(&id).unwrap();
    let before = app.platform_host.document().clone();
    let count = app.modeling_problems.checks().current_count();
    app.dispatch_ui_command("canvas.accept_focused");
    assert_eq!(
        app.modeling_problems
            .recommendation_ui
            .review
            .as_ref()
            .unwrap()
            .preview
            .recommendation()
            .key,
        key
    );
    app.ignore_modeling_recommendation(&key, true);
    app.modeling_problems
        .recommendations
        .recheck(app.platform_host.modeling_inputs());
    assert!(!app.canvas_suggestion_visible(&id));
    let mut projected = app.platform_host.snapshot().window_model();
    app.project_canvas_recommendations(&mut projected);
    assert!(
        !projected
            .canvas
            .widget
            .view()
            .legend
            .items
            .iter()
            .any(|i| i.kind_label == "Suggestion")
    );
    assert!(
        projected
            .flowsheet_context_toolbar
            .sections
            .iter()
            .flat_map(|s| &s.items)
            .filter(|i| i.command_id.as_deref() == Some("canvas.accept_focused"))
            .all(|i| !i.enabled)
    );
    assert_eq!(app.modeling_problems.checks().current_count(), count);
    app.dispatch_ui_command("canvas.accept_focused");
    assert!(app.modeling_problems.recommendation_ui.review.is_none());
    assert_eq!(
        app.modeling_problems.recommendation_ui.notice,
        Some(RecommendationError::Ignored)
    );
    app.ignore_modeling_recommendation(&key, false);
    assert!(app.canvas_suggestion_visible(&id));
    app.preview_modeling_recommendation(&key);
    assert_eq!(app.platform_host.document(), &before);
    app.accept_modeling_recommendation();
    assert_eq!(app.platform_host.document().revision, before.revision + 1);
    fs::remove_file(path).unwrap();
}

#[test]
fn stale_preview_rejects_formal_edits_and_regeneration_is_explicit() {
    let mut app = ready_app_state(&synced_workspace_config());
    let key = unnormalized(&mut app);
    app.preview_modeling_recommendation(&key);
    fraction(&mut app, "methane", "0.25");
    let before = app.platform_host.document().clone();
    app.accept_modeling_recommendation();
    assert!(matches!(
        app.modeling_problems
            .recommendation_ui
            .review
            .as_ref()
            .unwrap()
            .outcome,
        Some(Err(RecommendationError::StaleRevision { .. }))
    ));
    assert_eq!(app.platform_host.document(), &before);
    app.preview_modeling_recommendation(&key);
    app.accept_modeling_recommendation();
    assert_eq!(app.platform_host.document().revision, before.revision + 1);
}

#[test]
fn incomplete_bar_draft_survives_preview_accept_rejection_and_navigation() {
    let mut app = ready_app_state(&synced_workspace_config());
    let key = unnormalized(&mut app);
    app.dispatch_ui_command("inspector.focus_unit:heater-1");
    let mut field = app
        .platform_host
        .snapshot()
        .runtime
        .active_inspector_detail
        .unwrap()
        .property_fields
        .into_iter()
        .find(|f| f.key == "unit:heater-1:outlet_pressure_pa")
        .unwrap()
        .numeric
        .unwrap()
        .unwrap();
    app.edit_numeric_field(
        &mut field,
        rf_ui::NumericEditEvent::SelectInputUnit(rf_types::units::MeasurementUnit::Bar),
    );
    app.edit_numeric_field(
        &mut field,
        rf_ui::NumericEditEvent::ReplaceText("1e-".into()),
    );
    let before = app.platform_host.modeling_inputs().clone();
    app.preview_modeling_recommendation(&key);
    app.accept_modeling_recommendation();
    assert_eq!(
        app.modeling_problems
            .recommendation_ui
            .review
            .as_ref()
            .unwrap()
            .outcome,
        Some(Err(RecommendationError::PendingDrafts))
    );
    assert_eq!(app.platform_host.modeling_inputs(), &before);
    assert!(
        app.locate_specification_target(SpecificationTarget::Composition("stream-feed".into()))
    );
    assert_eq!(
        app.platform_host.modeling_inputs().workspace.drafts.fields,
        before.workspace.drafts.fields
    );
    assert_eq!(app.platform_host.document(), &before.workspace.document);
}

#[test]
fn reopening_same_saved_document_resets_ignore_review_and_old_tokens() {
    let (config, path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    app.dispatch_ui_command("canvas.accept_focused");
    let preview = app
        .modeling_problems
        .recommendation_ui
        .review
        .as_ref()
        .unwrap()
        .preview
        .clone();
    app.ignore_modeling_recommendation(&preview.recommendation().key, true);
    save_project_with_upgrade_consent(&mut app);
    app.open_project(path.clone(), "A2 saved project");
    assert!(app.modeling_problems.recommendation_ui.review.is_none());
    assert!(
        app.modeling_problems
            .recommendations
            .entries()
            .iter()
            .all(|e| e.visibility == RecommendationVisibility::Available)
    );
    assert_eq!(
        app.modeling_problems
            .recommendations
            .validate_preview(app.platform_host.modeling_inputs(), &preview),
        Err(RecommendationError::DifferentSession)
    );
    let saved = fs::read_to_string(&path).unwrap();
    assert!(!saved.contains("recommendation"));
    fs::remove_file(path).unwrap();
}

#[test]
fn bilingual_narrow_and_large_font_previews_show_complete_values_without_writing() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for scale in [1., 2.] {
            let mut app = ready_app_state(&synced_workspace_config());
            app.locale = locale;
            let key = unnormalized(&mut app);
            app.preview_modeling_recommendation(&key);
            let before = app.platform_host.document().clone();
            let ctx = egui::Context::default();
            fonts::configure_studio_fonts(&ctx);
            ctx.style_mut(|style| {
                for font in style.text_styles.values_mut() {
                    font.size *= scale;
                }
            });
            let mut rendered = vec![];
            for _ in 0..3 {
                rendered = panel(&mut app, &ctx, 400., vec![]);
            }
            for expected in ["methane: 0.2 → 0.4", "ethane: 0.3 → 0.6"] {
                assert!(rendered.iter().any(|(s, _)| s == expected), "{rendered:?}");
            }
            for (text, bounds) in rendered {
                assert!(bounds.max.x <= 401., "overflow {text}: {bounds:?}");
            }
            assert_eq!(app.platform_host.document(), &before);
        }
    }
}

#[test]
fn canvas_tab_only_opens_review_and_escape_returns_without_accepting_or_ignoring() {
    let (config, path) = flash_drum_local_rules_synced_config();
    let mut app = ready_app_state(&config);
    app.last_area_focus = Some(StudioGuiWindowAreaId::Canvas);
    let before = app.platform_host.document().clone();
    dispatch_shortcut_for_test(&mut app, egui::Key::Tab, egui::Modifiers::NONE);
    assert!(app.modeling_problems.recommendation_ui.review.is_some());
    let ctx = egui::Context::default();
    fonts::configure_studio_fonts(&ctx);
    panel(&mut app, &ctx, 400., vec![]);
    panel(
        &mut app,
        &ctx,
        400.,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(app.modeling_problems.recommendation_ui.review.is_none());
    assert!(
        app.modeling_problems
            .recommendations
            .entries()
            .iter()
            .all(|e| e.visibility == RecommendationVisibility::Available)
    );
    assert_eq!(app.platform_host.document(), &before);
    fs::remove_file(path).unwrap();
}

fn key_event(key: egui::Key) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        })
        .collect()
}

#[test]
fn opening_enter_cannot_accept_then_tab_and_explicit_space_commit_once() {
    let mut app = ready_app_state(&synced_workspace_config());
    let key = unnormalized(&mut app);
    app.preview_modeling_recommendation(&key);
    let before = app.platform_host.document().clone();
    let ctx = egui::Context::default();
    fonts::configure_studio_fonts(&ctx);
    panel(&mut app, &ctx, 500., key_event(egui::Key::Enter));
    assert_eq!(app.platform_host.document(), &before);
    assert!(app.modeling_problems.recommendation_ui.review.is_some());
    panel(&mut app, &ctx, 500., vec![]);
    panel(&mut app, &ctx, 500., key_event(egui::Key::Tab));
    panel(&mut app, &ctx, 500., key_event(egui::Key::Space));
    assert_eq!(composition(&app), [0.4, 0.6]);
    assert_eq!(app.platform_host.document().revision, before.revision + 1);
    panel(&mut app, &ctx, 500., key_event(egui::Key::Enter));
    assert_eq!(app.platform_host.document().revision, before.revision + 1);
}

#[test]
fn template_preview_contains_actual_payload_and_commit_binds_that_stream_atomically() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.create_blank_project();
    select_builtin_binary_hydrocarbon_basis(&mut app);
    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64., 40.));
    app.dispatch_ui_command("canvas.accept_focused");
    let action = app
        .modeling_problems
        .recommendation_ui
        .review
        .as_ref()
        .unwrap()
        .preview
        .recommendation()
        .action
        .clone();
    let RecommendedAction::MaterialConnection(connection) = action else {
        panic!("connection")
    };
    let rf_ui::CanvasSuggestedStreamBinding::Create { stream } = connection.stream else {
        panic!("template")
    };
    let ctx = egui::Context::default();
    fonts::configure_studio_fonts(&ctx);
    let rendered = panel(&mut app, &ctx, 400., vec![]);
    for expected in [
        "298.15 K",
        "101325 Pa",
        "1 mol/s",
        "methane: 0.5",
        "ethane: 0.5",
    ] {
        assert!(
            rendered.iter().any(|(s, _)| s.contains(expected)),
            "missing {expected}: {rendered:?}"
        );
    }
    let before = app.platform_host.document().clone();
    app.accept_modeling_recommendation();
    assert_eq!(
        app.platform_host.document().flowsheet.streams[&stream.id],
        stream
    );
    assert_eq!(app.platform_host.document().revision, before.revision + 1);
    app.dispatch_ui_command("edit.undo");
    assert_eq!(app.platform_host.document().flowsheet, before.flowsheet);
}

#[test]
fn failed_recheck_presentation_keeps_success_and_offers_recheck_without_duplicate_acceptance() {
    use radishflow_studio::modeling_specifications::{ModelingCheckFailure, ModelingCheckSession};
    let mut app = ready_app_state(&synced_workspace_config());
    let key = unnormalized(&mut app);
    app.preview_modeling_recommendation(&key);
    app.accept_modeling_recommendation();
    let before = app.platform_host.document().clone();
    // Supply a failed check result to the UI projection; the transaction/failure
    // boundary itself is tested through accept_with_recheck in the library.
    let mut checks = ModelingCheckSession::new(&before);
    let ticket = checks.begin(&before);
    assert!(checks.finish(&ticket, Err(ModelingCheckFailure::Panicked), &before));
    app.modeling_problems.recommendations = ModelingRecommendationSession::from_checks(checks);
    let ctx = egui::Context::default();
    fonts::configure_studio_fonts(&ctx);
    let rendered = panel(&mut app, &ctx, 400., vec![]);
    assert!(rendered.iter().any(|(s, _)| s.contains("动作已执行")));
    assert!(rendered.iter().any(|(s, _)| s.contains("重检失败")));
    assert!(rendered.iter().any(|(s, _)| s == "重新检查"));
    assert!(!rendered.iter().any(|(s, _)| s == "接受并归一"));
    assert_eq!(app.modeling_problems.checks().current_count(), None);
    app.accept_modeling_recommendation();
    assert_eq!(app.platform_host.document(), &before);
}

#[test]
fn tab_reveals_accept_action_inside_short_preview_scroll_viewport() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.locale = StudioShellLocale::En;
    let key = unnormalized(&mut app);
    app.preview_modeling_recommendation(&key);
    let before = app.platform_host.document().clone();
    let ctx = egui::Context::default();
    fonts::configure_studio_fonts(&ctx);
    let mut rendered = vec![];
    for frame in 0..6 {
        let output = ctx.run(
            egui::RawInput {
                time: Some(frame as f64 * 0.25),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400., 180.),
                )),
                focused: true,
                events: if frame == 2 {
                    key_event(egui::Key::Tab)
                } else {
                    vec![]
                },
                ..Default::default()
            },
            |ctx| {
                app.dispatch_shortcuts(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        app.render_recommendation_review(ui);
                    });
                });
            },
        );
        rendered.clear();
        for shape in output.shapes {
            texts(&shape.shape, &mut rendered);
        }
    }
    let bounds = rendered
        .iter()
        .find(|(s, _)| s == "Accept normalization")
        .unwrap()
        .1;
    assert!(
        bounds.min.y >= 0. && bounds.max.y <= 180.,
        "focused action outside viewport: {bounds:?}"
    );
    assert_eq!(app.platform_host.document(), &before);
}
