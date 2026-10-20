use super::paint_contrast::{contrast, shapes};
use super::*;
use crate::studio_gui_shell::state_presentation::StudioStateTokens;
use egui::epaint::{Shape, TextShape};

fn painted(output: &egui::FullOutput) -> Vec<&Shape> {
    output
        .shapes
        .iter()
        .flat_map(|s| shapes(&s.shape))
        .collect()
}

fn text<'a>(painted: &[&'a Shape], prefix: &str) -> &'a TextShape {
    painted
        .iter()
        .find_map(|shape| match shape {
            Shape::Text(text) if text.galley.job.text.starts_with(prefix) => Some(text),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing visible text: {prefix}"))
}

fn background(painted: &[&Shape], point: egui::Pos2) -> egui::Color32 {
    painted
        .iter()
        .rev()
        .find_map(|shape| match shape {
            Shape::Rect(rect) if rect.fill.a() == 255 && rect.rect.contains(point) => {
                Some(rect.fill)
            }
            _ => None,
        })
        .expect("text must have an opaque painted background")
}

fn text_contrast(painted: &[&Shape], text: &TextShape) -> f64 {
    // Use the glyph mesh, since disabled painting can tint glyphs after layout.
    let glyph = text
        .galley
        .rows
        .iter()
        .flat_map(|r| &r.visuals.mesh.vertices)
        .find(|v| v.uv != egui::epaint::WHITE_UV)
        .expect("painted glyph")
        .color;
    let foreground = text
        .override_text_color
        .unwrap_or(if glyph == egui::Color32::PLACEHOLDER {
            text.fallback_color
        } else {
            glyph
        });
    let bg = background(
        painted,
        text.galley.rect.translate(text.pos.to_vec2()).center(),
    );
    let ratio = contrast(foreground, bg);
    assert!(
        ratio >= 4.5,
        "{}: {ratio:.3}:1 ({foreground:?} / {bg:?})",
        text.galley.job.text
    );
    ratio
}

fn frame(app: &mut ReadyAppState, ctx: &egui::Context, key: &str, focus: bool) -> egui::FullOutput {
    let field = numeric_field(app, key);
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 1000.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if focus {
                    let id =
                        ui.make_persistent_id(("numeric-input", &field.variable.document, key));
                    ui.memory_mut(|m| m.request_focus(id));
                }
                app.render_numeric_property_field(ui, key, "Pressure", &field);
            });
        },
    )
}

#[test]
fn numeric_sources_drafts_errors_and_focus_have_readable_independent_cues() {
    for (command, key, source) in [
        (
            "inspector.focus_stream:stream-feed",
            "stream:stream-feed:pressure_pa",
            "Source: specified",
        ),
        (
            "inspector.focus_unit:feed-1",
            "unit:feed-1:outlet_pressure_pa",
            "Source: inherited",
        ),
    ] {
        for raw in [None, Some("1e-"), Some("invalid")] {
            for focus in [false, true] {
                let mut app = ready_app_state(&synced_workspace_config());
                app.locale = StudioShellLocale::En;
                app.dispatch_ui_command(command);
                let mut field = numeric_field(&app, key);
                if let Some(raw) = raw {
                    app.edit_numeric_field(
                        &mut field,
                        rf_ui::NumericEditEvent::ReplaceText(raw.into()),
                    );
                }
                let document = app.platform_host.document().clone();
                let ctx = egui::Context::default();
                ctx.set_visuals(egui::Visuals::light());
                frame(&mut app, &ctx, key, focus);
                let output = frame(&mut app, &ctx, key, focus);
                let painted = painted(&output);
                let field = numeric_field(&app, key);
                for prefix in [
                    "Pressure",
                    source,
                    if field.pending {
                        "Uncommitted"
                    } else {
                        "Committed"
                    },
                ] {
                    text_contrast(&painted, text(&painted, prefix));
                }
                let number = text(&painted, &field.text);
                let number_ratio = text_contrast(&painted, number);
                let number_rect = number.galley.rect.translate(number.pos.to_vec2());
                let input = painted
                    .iter()
                    .find_map(|shape| match shape {
                        Shape::Rect(rect)
                            if rect.fill.a() == 255
                                && rect.stroke.width > 0.0
                                && rect.rect.contains_rect(number_rect) =>
                        {
                            Some(rect)
                        }
                        _ => None,
                    })
                    .expect("numeric input frame");
                assert!(contrast(input.stroke.color, input.fill) >= 3.0);
                if let Some(raw) = raw {
                    text_contrast(
                        &painted,
                        text(
                            &painted,
                            if raw == "1e-" {
                                "Input incomplete"
                            } else {
                                "Invalid number"
                            },
                        ),
                    );
                }
                if raw == Some("invalid") {
                    let error = painted
                        .iter()
                        .find_map(|s| match s {
                            Shape::Rect(r)
                                if r.rect == input.rect
                                    && r.stroke.color == StudioStateTokens::ERROR =>
                            {
                                Some(r)
                            }
                            _ => None,
                        })
                        .expect("error has its own border as well as explanatory text");
                    assert!(contrast(error.stroke.color, input.fill) >= 3.0);
                }
                if focus {
                    let ring = painted
                        .iter()
                        .find_map(|s| match s {
                            Shape::Rect(r)
                                if r.stroke.color == StudioStateTokens::FOCUS
                                    && r.rect.contains_rect(input.rect) =>
                            {
                                Some(r)
                            }
                            _ => None,
                        })
                        .expect("focus remains outside the input/error border");
                    let separator = painted
                        .iter()
                        .find_map(|s| match s {
                            Shape::Rect(r)
                                if r.stroke.color == egui::Color32::WHITE
                                    && r.rect.contains_rect(input.rect) =>
                            {
                                Some(r)
                            }
                            _ => None,
                        })
                        .expect("neutral separation between error and focus");
                    assert!(ring.rect.contains_rect(separator.rect));
                    assert!(contrast(ring.stroke.color, separator.stroke.color) >= 3.0);
                    let source_text = text(&painted, source);
                    assert!(source_text.visual_bounding_rect().bottom() < ring.rect.top());
                    if let Some(raw) = raw {
                        let reason = text(
                            &painted,
                            if raw == "1e-" {
                                "Input incomplete"
                            } else {
                                "Invalid number"
                            },
                        );
                        assert!(reason.visual_bounding_rect().top() > ring.rect.bottom());
                    }
                }
                assert_eq!(app.platform_host.document(), &document);
                println!(
                    "{source}, {raw:?}, focus {focus}: number {number_ratio:.3}:1; state text/borders pass"
                );
            }
        }
    }
}

#[test]
fn composing_input_explains_disabled_actions_and_keeps_unit_readable() {
    for (locale, hint) in [
        (StudioShellLocale::ZhCn, "输入法正在处理文本"),
        (StudioShellLocale::En, "Input method is composing"),
    ] {
        let mut app = ready_app_state(&synced_workspace_config());
        app.locale = locale;
        app.dispatch_ui_command("inspector.focus_stream:stream-feed");
        let key = "stream:stream-feed:pressure_pa";
        let ctx = egui::Context::default();
        fonts::configure_studio_fonts(&ctx);
        ctx.enable_accesskit();
        numeric_frame(&mut app, &ctx, key, true, vec![]);
        let mut field = numeric_field(&app, key);
        app.edit_numeric_field(
            &mut field,
            rf_ui::NumericEditEvent::PreviewComposition("1".into()),
        );
        let document = app.platform_host.document().clone();
        let output = frame(&mut app, &ctx, key, true);
        let painted = painted(&output);
        let explanation = text(&painted, hint);
        assert!(explanation.galley.job.text.contains("Pa"));
        text_contrast(&painted, explanation);
        let tree = output.platform_output.accesskit_update.unwrap();
        let input = tree
            .nodes
            .iter()
            .map(|(_, n)| n)
            .find(|n| n.role() == egui::accesskit::Role::TextInput)
            .unwrap();
        assert!(input.label().unwrap().contains(hint));
        let combo = tree
            .nodes
            .iter()
            .map(|(_, n)| n)
            .find(|n| n.role() == egui::accesskit::Role::ComboBox)
            .unwrap();
        assert!(combo.is_disabled());
        assert_eq!(combo.value(), Some("Pa"));
        assert!(!numeric_field(&app, key).can_apply);
        assert_eq!(app.platform_host.document(), &document);
        app.edit_numeric_field(&mut field, rf_ui::NumericEditEvent::CancelComposition);
        let output = frame(&mut app, &ctx, key, true);
        assert!(!painted_text_contains(&output, hint));
    }
}

fn painted_text_contains(output: &egui::FullOutput, needle: &str) -> bool {
    painted(output)
        .iter()
        .any(|s| matches!(s, Shape::Text(t) if t.galley.job.text.contains(needle)))
}

#[test]
fn save_summary_and_notice_severity_are_readable_without_color() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let mut field = numeric_field(&app, "stream:stream-feed:pressure_pa");
    app.edit_numeric_field(
        &mut field,
        rf_ui::NumericEditEvent::ReplaceText("1e-".into()),
    );
    let state = app
        .platform_host
        .snapshot()
        .runtime
        .workspace_document
        .save_state;
    let ctx = egui::Context::default();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_project_save_state(ui, &state, StudioShellLocale::En);
            for level in [
                ProjectOpenNoticeLevel::Info,
                ProjectOpenNoticeLevel::Warning,
                ProjectOpenNoticeLevel::Error,
            ] {
                render_project_notice(
                    ui,
                    &ProjectOpenNotice {
                        level,
                        title: "Save notice".into(),
                        detail: "The current input stays in the editor.".into(),
                    },
                );
            }
        });
    });
    let painted = painted(&output);
    for prefix in [
        "Project content:",
        "Display settings:",
        "Unsubmitted inputs: 1",
        "ℹ ",
        "⚠ ",
        "! ",
    ] {
        let ratio = text_contrast(&painted, text(&painted, prefix));
        println!("{prefix}: {ratio:.3}:1");
    }
}
