use super::*;

#[derive(Clone, Copy, Debug)]
enum Dialog {
    Project,
    View,
    UnavailableDefaults,
    RecoveryFailure,
}

impl Dialog {
    fn id(self) -> egui::Id {
        egui::Id::new(match self {
            Self::Project => "project-display-units",
            Self::View => "inspector-display-units",
            Self::UnavailableDefaults => "invalid-unit-default-new-project",
            Self::RecoveryFailure => "recover-unit-defaults",
        })
    }
}

fn frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1024.0, 665.0),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |ctx| {
            assert!(app.render_unit_settings(ctx));
        },
    )
}

#[test]
fn unit_settings_keep_dialog_content_within_small_viewports() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for font_scale in [1.0, 1.5, 2.0] {
            for dialog in [
                Dialog::Project,
                Dialog::View,
                Dialog::UnavailableDefaults,
                Dialog::RecoveryFailure,
            ] {
                let preferences = test_preferences_path("unit-settings-layout");
                fs::create_dir_all(preferences.parent().unwrap()).unwrap();
                let defaults = preferences.with_file_name(rf_store::UNIT_DEFAULTS_FILE_NAME);
                if matches!(
                    dialog,
                    Dialog::UnavailableDefaults | Dialog::RecoveryFailure
                ) {
                    fs::create_dir_all(&defaults).unwrap();
                }
                let mut app =
                    ReadyAppState::from_config(&synced_workspace_config(), preferences.clone())
                        .unwrap();
                app.locale = locale;
                app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(
                    rf_types::units::DisplayUnitSet::engineering(),
                ))
                .unwrap();
                app.dispatch_ui_command("inspector.focus_stream:stream-feed");
                app.dispatch_inspector_field_draft_update(
                    radishflow_studio::inspector_draft_update_command_id(
                        "stream:stream-feed:pressure_pa",
                    ),
                    "1e-",
                );
                match dialog {
                    Dialog::Project => {
                        app.open_unit_settings();
                        app.unit_settings.draft.as_mut().unwrap().1 =
                            rf_types::units::DisplayUnitSet::si();
                    }
                    Dialog::View => app.open_view_unit_settings(rf_ui::DisplayUnitViewId(
                        app.current_window_id().unwrap(),
                    )),
                    Dialog::UnavailableDefaults => app.unit_settings.pending_new = Some(None),
                    Dialog::RecoveryFailure => {
                        app.open_unit_settings();
                        app.request_unit_default_recovery();
                        app.save_unit_default(true);
                        assert!(defaults.is_dir());
                    }
                }
                let before = app.platform_host.snapshot().runtime.workspace_document;
                let ctx = egui::Context::default();
                ctx.set_visuals(egui::Visuals::light());
                fonts::configure_studio_fonts(&ctx);
                ctx.style_mut(|style| {
                    style.scroll_animation = egui::style::ScrollAnimation::none();
                    for font in style.text_styles.values_mut() {
                        font.size *= font_scale;
                    }
                });
                let case = format!("{locale:?}, font {font_scale}, {dialog:?}");
                frame(&mut app, &ctx, vec![]);
                frame(&mut app, &ctx, vec![]);
                let output = frame(&mut app, &ctx, vec![]);
                assert_dialog_fits(&ctx, dialog, &output, &case);
                if matches!(dialog, Dialog::Project) {
                    // The grid must allocate room for its labels rather than wrapping
                    // each quantity into the default minimum column width.
                    for shape in &output.shapes {
                        if let egui::epaint::Shape::Text(text) = &shape.shape
                            && rf_types::units::DisplayUnitSet::si().entries().any(
                                |(quantity, _)| {
                                    text.galley.job.text == locale.quantity_name(quantity)
                                },
                            )
                        {
                            assert_eq!(
                                text.galley.rows.len(),
                                1,
                                "{case}: {:?}",
                                text.galley.job.text
                            );
                        }
                    }
                }
                // Tab must reveal controls below the viewport, not focus clipped actions.
                let mut visited = std::collections::HashSet::new();
                for _ in 0..24 {
                    frame(
                        &mut app,
                        &ctx,
                        vec![egui::Event::Key {
                            key: egui::Key::Tab,
                            physical_key: Some(egui::Key::Tab),
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        }],
                    );
                    frame(&mut app, &ctx, vec![]);
                    frame(&mut app, &ctx, vec![]);
                    let output = frame(&mut app, &ctx, vec![]);
                    assert_dialog_fits(&ctx, dialog, &output, &case);
                    if let Some(id) = ctx.memory(|memory| memory.focused()) {
                        let response = ctx.read_response(id).unwrap();
                        let visible = response.interact_rect;
                        assert!(
                            visible.contains_rect(response.rect),
                            "{case}: focused control clipped: {:?}, visible {visible:?}",
                            response.rect
                        );
                        visited.insert(id);
                    }
                }
                assert!(visited.len() >= 2, "{case}: no keyboard navigation");
                assert_eq!(
                    app.platform_host.snapshot().runtime.workspace_document,
                    before
                );
                fs::remove_dir_all(preferences.parent().unwrap()).unwrap();
            }
        }
    }
}

fn assert_dialog_fits(ctx: &egui::Context, dialog: Dialog, output: &egui::FullOutput, case: &str) {
    let rect = ctx.memory(|memory| memory.area_rect(dialog.id())).unwrap();
    assert!(
        ctx.screen_rect().contains_rect(rect),
        "{case}: dialog exceeds viewport: {rect:?}"
    );
    for shape in &output.shapes {
        assert_text_fits(&shape.shape, shape.clip_rect, case);
    }
}

fn assert_text_fits(shape: &egui::epaint::Shape, clip: egui::Rect, case: &str) {
    match shape {
        egui::epaint::Shape::Text(text) => {
            let rect = text.galley.rect.translate(text.pos.to_vec2());
            assert!(
                rect.left() >= clip.left() - 0.5 && rect.right() <= clip.right() + 0.5,
                "{case}: horizontal clipping: {:?}, {rect:?}, {clip:?}",
                text.galley.job.text
            );
        }
        egui::epaint::Shape::Vec(shapes) => {
            for shape in shapes {
                assert_text_fits(shape, clip, case);
            }
        }
        _ => {}
    }
}
