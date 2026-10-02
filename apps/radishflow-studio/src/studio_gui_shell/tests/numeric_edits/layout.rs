use super::*;
use rf_types::units::DisplayUnitSet;
use rf_ui::{NumericEditEvent, ProjectPresentationCommand};

fn assert_text_fits(shape: &egui::epaint::Shape, clip: egui::Rect, case: &str) {
    match shape {
        egui::epaint::Shape::Text(text) => {
            let rect = text.galley.rect.translate(text.pos.to_vec2());
            assert!(
                rect.left() >= clip.left() - 0.5 && rect.right() <= clip.right() + 0.5,
                "{case}: {:?} overflows {clip:?}: {rect:?}",
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

#[test]
fn numeric_fields_keep_units_states_and_actions_inside_narrow_columns() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for width in [400.0, 280.0] {
            for font_scale in [1.0, 1.5, 2.0] {
                for draft in [
                    None,
                    Some(NumericEditEvent::ReplaceText("1.4".into())),
                    Some(NumericEditEvent::ReplaceText("1e-".into())),
                    Some(NumericEditEvent::ReplaceText("invalid".into())),
                    Some(NumericEditEvent::PreviewComposition("1".into())),
                ] {
                    let mut app = ready_app_state(&synced_workspace_config());
                    app.locale = locale;
                    app.apply_presentation_command(ProjectPresentationCommand::Apply(
                        DisplayUnitSet::engineering(),
                    ))
                    .unwrap();
                    app.dispatch_ui_command("inspector.focus_unit:feed-1");
                    let key = "unit:feed-1:outlet_pressure_pa";
                    let mut field = numeric_field(&app, key);
                    if let Some(event) = &draft {
                        app.edit_numeric_field(&mut field, event.clone());
                    }
                    let ctx = egui::Context::default();
                    fonts::configure_studio_fonts(&ctx);
                    ctx.style_mut(|style| {
                        for font in style.text_styles.values_mut() {
                            font.size *= font_scale;
                        }
                    });
                    let case =
                        format!("{locale:?}, width {width}, font {font_scale}, draft {draft:?}");
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width, 1600.0),
                            )),
                            ..Default::default()
                        },
                        |ctx| {
                            egui::CentralPanel::default().show(ctx, |ui| {
                                if matches!(draft, Some(NumericEditEvent::PreviewComposition(_))) {
                                    let id = ui.make_persistent_id((
                                        "numeric-input",
                                        &field.variable.document,
                                        key,
                                    ));
                                    ui.memory_mut(|memory| memory.request_focus(id));
                                }
                                app.render_numeric_property_field(
                                    ui,
                                    key,
                                    if locale == StudioShellLocale::ZhCn {
                                        "源压力"
                                    } else {
                                        "Source pressure"
                                    },
                                    &field,
                                );
                            });
                        },
                    );
                    for shape in &output.shapes {
                        assert_text_fits(&shape.shape, shape.clip_rect, &case);
                    }
                }
            }
        }
    }
}
