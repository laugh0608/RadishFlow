use super::*;
use egui::epaint::{Shape, TextShape};
use rf_types::units::DisplayUnitSet;

#[derive(Clone, Copy, Debug)]
enum Scenario {
    Project,
    View,
    Unavailable,
    Recovery,
    Failed,
    Recovered,
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
                egui::vec2(900.0, 1000.0),
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

fn flatten(shape: &Shape) -> Vec<&Shape> {
    match shape {
        Shape::Vec(children) => children.iter().flat_map(flatten).collect(),
        _ => vec![shape],
    }
}

fn luminance(color: egui::Color32) -> f64 {
    assert_eq!(color.a(), 255, "measurement requires opaque sRGB");
    [color.r(), color.g(), color.b()]
        .into_iter()
        .zip([0.2126, 0.7152, 0.0722])
        .map(|(channel, weight)| {
            let value = f64::from(channel) / 255.0;
            weight
                * if value <= 0.04045 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
        })
        .sum()
}

fn contrast(a: egui::Color32, b: egui::Color32) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn foreground(text: &TextShape) -> egui::Color32 {
    let glyph = text
        .galley
        .rows
        .iter()
        .flat_map(|row| &row.visuals.mesh.vertices)
        .find(|vertex| vertex.uv != egui::epaint::WHITE_UV)
        .expect("painted glyph")
        .color;
    text.override_text_color
        .unwrap_or(if glyph == egui::Color32::PLACEHOLDER {
            text.fallback_color
        } else {
            glyph
        })
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
        .expect("opaque background painted before the text")
}

fn assert_paint(output: &egui::FullOutput, required: &[&str], case: &str) {
    let painted: Vec<_> = output
        .shapes
        .iter()
        .flat_map(|shape| flatten(&shape.shape))
        .collect();
    let tree = output.platform_output.accesskit_update.as_ref().unwrap();
    let disabled: Vec<_> = tree
        .nodes
        .iter()
        .filter_map(|(_, node)| {
            (node.is_disabled() && node.role() == egui::accesskit::Role::Button)
                .then(|| node.bounds())
                .flatten()
        })
        .collect();
    let mut text_seen = String::new();
    let mut pairs = std::collections::BTreeMap::new();
    for (index, shape) in painted.iter().enumerate() {
        let Shape::Text(text) = shape else { continue };
        let rect = text.galley.rect.translate(text.pos.to_vec2());
        text_seen.push_str(&text.galley.job.text);
        text_seen.push('\n');
        // Inactive history/save buttons do not carry the only explanation of state.
        if disabled.iter().any(|bounds| {
            bounds.contains(egui::accesskit::Point::new(
                f64::from(rect.center().x),
                f64::from(rect.center().y),
            ))
        }) {
            continue;
        }
        let fg = foreground(text);
        let bg = background(&painted[..index], rect.center());
        let ratio = contrast(fg, bg);
        assert!(
            ratio >= 4.5,
            "{case}: {:?} {fg:?}/{bg:?} = {ratio:.3}:1",
            text.galley.job.text
        );
        pairs.insert((fg.to_array(), bg.to_array()), ratio);
    }
    for required in required {
        assert!(
            text_seen.contains(required),
            "{case}: missing cue {required:?}\n{text_seen}"
        );
    }
    println!("{case}: text pairs {pairs:?}");
}

#[test]
fn settings_and_recovery_paint_readable_states_without_color_only_cues() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for scenario in [
            Scenario::Project,
            Scenario::View,
            Scenario::Unavailable,
            Scenario::Recovery,
            Scenario::Failed,
            Scenario::Recovered,
        ] {
            let preferences = test_preferences_path("settings-paint");
            fs::create_dir_all(preferences.parent().unwrap()).unwrap();
            let defaults = preferences.with_file_name(rf_store::UNIT_DEFAULTS_FILE_NAME);
            if matches!(scenario, Scenario::Failed) {
                fs::create_dir_all(&defaults).unwrap();
            } else {
                fs::write(&defaults, b"invalid personal defaults").unwrap();
            }
            let mut app =
                ReadyAppState::from_config(&synced_workspace_config(), preferences.clone())
                    .unwrap();
            app.locale = locale;
            app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(
                DisplayUnitSet::engineering(),
            ))
            .unwrap();
            app.dispatch_ui_command("inspector.focus_stream:stream-feed");
            app.dispatch_inspector_field_draft_update(
                radishflow_studio::inspector_draft_update_command_id(
                    "stream:stream-feed:pressure_pa",
                ),
                "1e-",
            );
            app.open_unit_settings();
            app.unit_settings.draft.as_mut().unwrap().1 = DisplayUnitSet::si();
            match scenario {
                Scenario::Project => {}
                Scenario::View => {
                    app.open_view_unit_settings(rf_ui::DisplayUnitViewId(
                        app.current_window_id().unwrap(),
                    ));
                }
                Scenario::Unavailable => {
                    app.unit_settings.draft = None;
                    app.unit_settings.pending_new = Some(None);
                }
                Scenario::Recovery | Scenario::Failed | Scenario::Recovered => {
                    app.request_unit_default_recovery();
                    if !matches!(scenario, Scenario::Recovery) {
                        app.save_unit_default(true);
                    }
                }
            }
            let before = app.platform_host.snapshot().runtime.workspace_document;
            let ctx = egui::Context::default();
            ctx.set_visuals(egui::Visuals::light());
            ctx.style_mut(|style| style.animation_time = 0.0);
            ctx.enable_accesskit();
            fonts::configure_studio_fonts(&ctx);
            let zh = locale == StudioShellLocale::ZhCn;
            let required: Vec<&str> = match scenario {
                Scenario::Project => {
                    if zh {
                        vec![
                            "作用范围",
                            "当前选择尚未应用",
                            "显示设置：待保存",
                            "未提交输入：1",
                            "个人默认未加载",
                            "诊断详情",
                        ]
                    } else {
                        vec![
                            "Applies to this project",
                            "These choices are not applied yet",
                            "Display settings: unsaved",
                            "Unsubmitted inputs: 1",
                            "Personal defaults were not loaded",
                            "Diagnostic details",
                        ]
                    }
                }
                Scenario::View => {
                    if zh {
                        vec![
                            "仅本视图",
                            "本视图选择尚未应用",
                            "项目",
                            "bar",
                            "K",
                            "未提交输入：1",
                        ]
                    } else {
                        vec![
                            "This view:",
                            "View choices not yet applied",
                            "Project",
                            "bar",
                            "K",
                            "Unsubmitted inputs: 1",
                        ]
                    }
                }
                Scenario::Unavailable => {
                    if zh {
                        vec![
                            "个人单位默认不可用",
                            "原默认文件保持不变",
                            "本次使用 SI 新建",
                            "取消",
                        ]
                    } else {
                        vec![
                            "Personal unit defaults unavailable",
                            "existing defaults file is unchanged",
                            "Create this project with SI",
                            "Cancel",
                        ]
                    }
                }
                Scenario::Recovery => {
                    if zh {
                        vec!["恢复个人单位默认", "先备份", "备份并替换默认", "取消"]
                    } else {
                        vec![
                            "Recover personal unit defaults",
                            "Back up the existing defaults file",
                            "Back up and replace defaults",
                            "Cancel",
                        ]
                    }
                }
                Scenario::Failed => {
                    if zh {
                        vec![
                            "! 个人默认保存失败",
                            "内存默认与工程设置保持不变，可重试",
                            "诊断详情",
                            "备份并替换默认",
                        ]
                    } else {
                        vec![
                            "! Could not save personal defaults",
                            "You can retry",
                            "Diagnostic details",
                            "Back up and replace defaults",
                        ]
                    }
                }
                Scenario::Recovered => {
                    if zh {
                        vec![
                            "ℹ 个人默认已保存",
                            "新工程默认已恢复",
                            "原文件备份",
                            ".bak",
                            "当前选择尚未应用",
                        ]
                    } else {
                        vec![
                            "ℹ Personal defaults saved",
                            "Defaults restored for new projects",
                            "Previous file backed up to",
                            ".bak",
                            "These choices are not applied yet",
                        ]
                    }
                }
            };
            frame(&mut app, &ctx, vec![]);
            frame(&mut app, &ctx, vec![]);
            if matches!(scenario, Scenario::View) {
                for key in [
                    egui::Key::Tab,
                    egui::Key::Space,
                    egui::Key::ArrowDown,
                    egui::Key::Enter,
                ] {
                    frame(
                        &mut app,
                        &ctx,
                        vec![egui::Event::Key {
                            key,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        }],
                    );
                    frame(&mut app, &ctx, vec![]);
                }
                // Menu selection keeps focus on the selector. Release it so the
                // first measurement also exercises the selector's resting paint.
                ctx.memory_mut(|memory| {
                    if let Some(id) = memory.focused() {
                        memory.surrender_focus(id);
                    }
                });
            }
            let output = frame(&mut app, &ctx, vec![]);
            let case = format!("{locale:?}/{scenario:?}");
            assert_paint(&output, &required, &case);
            // First Tab focuses a real action. Inspect the painted focus boundary.
            frame(
                &mut app,
                &ctx,
                vec![egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            let output = frame(&mut app, &ctx, vec![]);
            assert_paint(&output, &required, &case);
            let response = ctx
                .read_response(ctx.memory(|memory| memory.focused()).unwrap())
                .unwrap();
            let painted: Vec<_> = output
                .shapes
                .iter()
                .flat_map(|s| flatten(&s.shape))
                .collect();
            let (index, border) = painted
                .iter()
                .enumerate()
                .find_map(|(index, shape)| match shape {
                    Shape::Rect(rect)
                        if rect.rect.contains(response.rect.center())
                            && (rect.rect.width() - response.rect.width()).abs() <= 2.1
                            && rect.stroke.width > 0.0 =>
                    {
                        Some((index, rect))
                    }
                    _ => None,
                })
                .expect("focused action has a visible boundary");
            let outside = background(&painted[..index], border.rect.center());
            let ratio = contrast(border.stroke.color, outside)
                .min(contrast(border.stroke.color, border.fill));
            assert!(ratio >= 3.0, "{case}: focus only {ratio:.3}:1");
            println!("{case}: focus {ratio:.3}:1");
            assert_eq!(
                app.platform_host.snapshot().runtime.workspace_document,
                before
            );
            if matches!(scenario, Scenario::Recovered) {
                assert_eq!(
                    rf_store::read_unit_defaults(&defaults).unwrap(),
                    Some(DisplayUnitSet::engineering())
                );
                let backup = fs::read_dir(preferences.parent().unwrap())
                    .unwrap()
                    .map(|e| e.unwrap().path())
                    .find(|path| path.extension().is_some_and(|ext| ext == "bak"))
                    .unwrap();
                assert_eq!(fs::read(backup).unwrap(), b"invalid personal defaults");
            } else if matches!(scenario, Scenario::Failed) {
                assert!(defaults.is_dir());
            } else {
                assert_eq!(fs::read(&defaults).unwrap(), b"invalid personal defaults");
            }
            fs::remove_dir_all(preferences.parent().unwrap()).unwrap();
        }
    }
}
