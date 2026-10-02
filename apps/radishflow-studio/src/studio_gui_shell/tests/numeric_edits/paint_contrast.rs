use super::*;

fn luminance(color: egui::Color32) -> f64 {
    assert_eq!(
        color.a(),
        255,
        "this measurement requires opaque sRGB colors"
    );
    [color.r(), color.g(), color.b()]
        .into_iter()
        .zip([0.2126, 0.7152, 0.0722])
        .map(|(channel, weight)| {
            let srgb = f64::from(channel) / 255.0;
            weight
                * if srgb <= 0.04045 {
                    srgb / 12.92
                } else {
                    ((srgb + 0.055) / 1.055).powf(2.4)
                }
        })
        .sum()
}

fn contrast(a: egui::Color32, b: egui::Color32) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn shapes(shape: &egui::epaint::Shape) -> Vec<&egui::epaint::Shape> {
    match shape {
        egui::epaint::Shape::Vec(children) => children.iter().flat_map(shapes).collect(),
        _ => vec![shape],
    }
}

#[test]
fn selected_numeric_text_meets_contrast_on_its_actual_painted_selection() {
    for raw in [None, Some("1e-"), Some("invalid")] {
        let mut app = ready_app_state(&synced_workspace_config());
        app.dispatch_ui_command("inspector.focus_stream:stream-feed");
        let key = "stream:stream-feed:pressure_pa";
        if let Some(raw) = raw {
            let mut field = numeric_field(&app, key);
            app.edit_numeric_field(&mut field, rf_ui::NumericEditEvent::ReplaceText(raw.into()));
        }
        let ctx = egui::Context::default();
        ctx.set_visuals(egui::Visuals::light());
        let id = numeric_frame(&mut app, &ctx, key, true, vec![]);
        let field = numeric_field(&app, key);
        select_all(&ctx, id, &field.text);
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 600.0),
                )),
                focused: true,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    app.render_numeric_property_field(ui, key, "Pressure", &field);
                });
            },
        );
        let painted = output
            .shapes
            .iter()
            .flat_map(|shape| shapes(&shape.shape))
            .collect::<Vec<_>>();
        let text = painted
            .iter()
            .find_map(|shape| match shape {
                egui::epaint::Shape::Text(text) if text.galley.job.text == field.text => Some(text),
                _ => None,
            })
            .expect("numeric text must be painted");
        let foreground = text.galley.job.sections[0].format.color;
        // egui inserts the selection background into the galley mesh behind the glyphs.
        // Read that mesh, so a default-theme regression cannot hide behind token-only tests.
        let background = text
            .galley
            .rows
            .iter()
            .flat_map(|row| &row.visuals.mesh.vertices)
            .find(|vertex| vertex.uv == egui::epaint::WHITE_UV)
            .expect("selected text must have a painted background")
            .color;
        let ratio = contrast(foreground, background);
        assert!(
            ratio >= 4.5,
            "{raw:?}: selected numeric text is only {ratio:.3}:1 ({foreground:?} / {background:?})"
        );
        println!("{raw:?}: selected numeric text {ratio:.3}:1");
    }
}

#[test]
fn unit_selector_paints_a_distinguishable_boundary_in_each_interaction_state() {
    let ctx = egui::Context::default();
    ctx.set_visuals(egui::Visuals::light());
    let mut button: Option<egui::Response> = None;
    for state in ["rest", "hover", "focus", "open"] {
        let events = match (state, &button) {
            ("hover", Some(response)) => vec![egui::Event::PointerMoved(response.rect.center())],
            ("open", _) => vec![key_event(egui::Key::Space, egui::Modifiers::NONE)],
            _ => vec![egui::Event::PointerGone],
        };
        if state == "focus" {
            button.as_ref().unwrap().request_focus();
        }
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 600.0),
                )),
                focused: true,
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    button = Some(
                        crate::studio_gui_shell::unit_selector::unit_selector(
                            ui,
                            "test-pressure-unit",
                            "Pressure unit",
                            0,
                            &[(0, "Pa"), (1, "bar")],
                        )
                        .response,
                    );
                });
            },
        );
        let button = button.as_ref().unwrap();
        let border = output
            .shapes
            .iter()
            .flat_map(|shape| shapes(&shape.shape))
            .find_map(|shape| match shape {
                egui::epaint::Shape::Rect(rect)
                    if rect.rect.contains(button.rect.center())
                        && (rect.rect.width() - button.rect.width()).abs() < 4.0 =>
                {
                    Some(rect)
                }
                _ => None,
            })
            .expect("unit selector frame must be painted");
        assert!(
            border.stroke.width > 0.0,
            "{state}: unit selector has no boundary"
        );
        let ratio = contrast(border.stroke.color, border.fill);
        assert!(
            ratio >= 3.0,
            "{state}: unit selector boundary is only {ratio:.3}:1"
        );
        let text = output
            .shapes
            .iter()
            .flat_map(|shape| shapes(&shape.shape))
            .find_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text == "Pa" && button.rect.contains(text.pos) =>
                {
                    Some(text)
                }
                _ => None,
            })
            .expect("current unit must be visible in the selector");
        let section_color = text.galley.job.sections[0].format.color;
        let foreground =
            text.override_text_color
                .unwrap_or(if section_color == egui::Color32::PLACEHOLDER {
                    text.fallback_color
                } else {
                    section_color
                });
        let text_ratio = contrast(foreground, border.fill);
        assert!(
            text_ratio >= 4.5,
            "{state}: unit text is only {text_ratio:.3}:1"
        );
        println!("{state}: unit selector boundary {ratio:.3}:1, unit text {text_ratio:.3}:1");
        if state == "open" {
            assert!(ctx.memory(|memory| memory.any_popup_open()));
        }
    }
}
