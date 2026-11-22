use super::*;
use crate::studio_gui_shell::locale::tr;
use egui::accesskit::{Action, ActionRequest, NodeId, Role, TreeUpdate};
use radishflow_studio::{StudioProjectOpen, StudioReadOnlyProject};
use rf_store::{FileOverwritePolicy, project_v3, project_v4};
use rf_ui::engineering::{
    EquipmentCaseKey,
    concentric_tube::{LimitCheck, RatingRevisions},
};
use serde_json::{Value, json};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "rf-studio-h3-review-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn project(&self, version: u32, edit: impl FnOnce(&mut Value)) -> PathBuf {
        let example = if version == 3 {
            include_str!(
                "../../../../../examples/flowsheets/standalone-separator-basis.rfproj.json"
            )
        } else {
            include_str!(
                "../../../../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json"
            )
        };
        let mut root: Value = serde_json::from_str(example).unwrap();
        root["engineering"] = serde_json::from_str(include_str!(
            "../../../tests/fixtures/h3-review-engineering.json"
        ))
        .unwrap();
        if version == 4 {
            root["requiredCapabilities"]
                .as_array_mut()
                .unwrap()
                .push(json!({
                    "id":project_v4::CONCENTRIC_TUBE_RATING_CAPABILITY,"version":1
                }));
        }
        edit(&mut root);
        let path = self.0.join(format!("review-{version}.rfproj.json"));
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&root).unwrap()),
        )
        .unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn key(task: &rf_model::equipment::ConcentricTubeRatingTask) -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: task.id.clone(),
        case_id: task.case.id.clone(),
    }
}

#[test]
fn h3_review_opens_both_envelopes_rates_exact_cases_and_preserves_file() {
    let fixture = Fixture::new();
    for version in [3, 4] {
        let path = fixture.project(version, |_| {});
        let original = fs::read(&path).unwrap();
        let StudioProjectOpen::ReadOnly(view) = read_studio_gui_project(&path).unwrap() else {
            panic!("review must be detached from the editable workspace");
        };
        assert_eq!(view.schema_version(), version);
        assert_eq!(view.original_json().as_bytes(), original);
        let StudioReadOnlyProject::H3(review) = view.as_ref() else {
            panic!("expected typed H3 review")
        };
        assert_eq!(review.file_sha256(), rf_store::sha256_hex(&original));
        let cases: Vec<_> = review.cases().collect();
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].case.name, cases[1].case.name);
        for (task, duty, ua, hot, cold, dp, limit) in [
            (
                cases[0],
                5_521.707_636_437_1,
                874.553_423_114_86,
                297.914_262_742_51,
                294.982_508_040_12,
                23_679.130_040_479,
                LimitCheck::Passed,
            ),
            (
                cases[1],
                5_877.489_522_370_7,
                945.093_778_801_02,
                297.576_906_873_93,
                294.710_466_075_30,
                34_962.547_685_264,
                LimitCheck::Failed,
            ),
        ] {
            let result = review.rate(&key(task), task.into()).unwrap().unwrap();
            assert!((result.heat_duty_w - duty).abs() <= 1e-6);
            assert!((result.ua_w_per_k - ua).abs() <= 1e-6);
            assert!((result.hot.outlet.temperature_k - hot).abs() <= 1e-8);
            assert!((result.cold.outlet.temperature_k - cold).abs() <= 1e-8);
            assert!((result.hot.passage.pressure_drop_pa - 9_799.896_961_973_8).abs() <= 1e-4);
            assert!((result.cold.passage.pressure_drop_pa - dp).abs() <= 1e-4);
            assert_eq!(result.checks.cold_pressure_drop, limit);
            assert_eq!(result.checks.heat_duty, LimitCheck::Passed);
            assert_eq!(result.checks.hot_pressure_drop, LimitCheck::Passed);
        }
        let stale = RatingRevisions {
            case: cases[0].case.revision + 1,
            ..cases[0].into()
        };
        assert!(review.rate(&key(cases[0]), stale).unwrap().is_err());
        let mixed = EquipmentCaseKey {
            task_id: cases[0].id.clone(),
            case_id: cases[1].case.id.clone(),
        };
        assert!(review.rate(&mixed, cases[0].into()).is_none());

        let ordinary = radishflow_studio::read_studio_project(&path).unwrap();
        assert!(
            matches!(ordinary, StudioProjectOpen::ReadOnly(other) if !matches!(*other, StudioReadOnlyProject::H3(_)))
        );
        assert!(if version == 3 {
            let source = project_v3::parse(include_str!(
                "../../../../../examples/flowsheets/standalone-separator-basis.rfproj.json"
            ))
            .unwrap();
            project_v3::write_upgraded(&path, &source, FileOverwritePolicy::Allow).is_err()
        } else {
            let source = project_v4::parse(include_str!(
                "../../../../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json"
            ))
            .unwrap();
            project_v4::prepare_write(&path, &source, FileOverwritePolicy::Allow).is_err()
        });
        assert_eq!(fs::read(&path).unwrap(), original);
    }
}

#[test]
fn h3_review_keeps_unknown_or_mixed_future_content_opaque() {
    let fixture = Fixture::new();
    for version in [3, 4] {
        let path = fixture.project(version, |root| {
            root["engineering"]["tasks"][1]["method"]["version"] = json!(9);
        });
        let bytes = fs::read(&path).unwrap();
        let StudioProjectOpen::ReadOnly(view) = read_studio_gui_project(&path).unwrap() else {
            panic!()
        };
        assert!(!matches!(view.as_ref(), StudioReadOnlyProject::H3(_)));
        assert_eq!(view.original_json().as_bytes(), bytes);
    }

    let path = fixture.project(4, |root| {
        root["requiredCapabilities"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":"example.unknown","version":1}));
    });
    assert!(
        matches!(read_studio_gui_project(&path).unwrap(), StudioProjectOpen::ReadOnly(view) if matches!(*view, StudioReadOnlyProject::V4(_)))
    );
}

fn button_node(tree: &TreeUpdate, label: &str) -> NodeId {
    tree.nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::Button && node.label().is_some_and(|name| name.contains(label))
        })
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("missing {label:?}, nodes: {:?}", texts(tree)))
}

fn texts(tree: &TreeUpdate) -> Vec<&str> {
    tree.nodes
        .iter()
        .flat_map(|(_, node)| [node.label(), node.value()].into_iter().flatten())
        .collect()
}

fn press(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn focus(target: NodeId) -> egui::Event {
    egui::Event::AccessKitActionRequest(ActionRequest {
        target,
        action: Action::Focus,
        data: None,
    })
}

fn frame(app: &mut ReadyAppState, ctx: &egui::Context, events: Vec<egui::Event>) -> TreeUpdate {
    review_frame(app, ctx, events, egui::vec2(400.0, 820.0))
}

fn review_frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    events: Vec<egui::Event>,
    size: egui::Vec2,
) -> TreeUpdate {
    let modifiers = events
        .iter()
        .find_map(|event| match event {
            egui::Event::Key { modifiers, .. } => Some(*modifiers),
            _ => None,
        })
        .unwrap_or_default();
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
        events,
        modifiers,
        focused: true,
        ..Default::default()
    });
    app.dispatch_shortcuts(ctx);
    app.render_read_only_project(ctx);
    let output = ctx.end_pass();
    for shape in output.shapes {
        super::equipment_rating_editor::fits(
            &shape.shape,
            shape.clip_rect,
            &[],
            "H3 review viewport",
        );
    }
    output
        .platform_output
        .accesskit_update
        .expect("review must expose its accessibility tree")
}

#[test]
fn h3_review_desktop_shows_the_action_and_rating_without_scrolling() {
    let fixture = Fixture::new();
    let path = fixture.project(4, |_| {});
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let viewport = egui::vec2(1280.0, 720.0);
    let mut app = ready_app_state(&synced_workspace_config());
    app.locale = StudioShellLocale::En;
    app.open_project(path, "h3 desktop test");
    review_frame(&mut app, &ctx, vec![], viewport);
    let tree = review_frame(&mut app, &ctx, vec![], viewport);
    let rate = button_node(&tree, "Rate selected case");
    let bounds = tree
        .nodes
        .iter()
        .find(|(id, _)| *id == rate)
        .unwrap()
        .1
        .bounds()
        .unwrap();
    assert!(bounds.y1 < 200.0, "{bounds:?}");
    review_frame(&mut app, &ctx, vec![focus(rate)], viewport);
    review_frame(
        &mut app,
        &ctx,
        vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
        viewport,
    );
    let tree = review_frame(&mut app, &ctx, vec![], viewport);
    let bounds = tree
        .nodes
        .iter()
        .find(|(_, node)| {
            node.value()
                .is_some_and(|value| value.contains("23679.13 Pa"))
        })
        .and_then(|(_, node)| node.bounds())
        .expect("annulus pressure drop must have screen bounds");
    assert!(
        bounds.y0 >= 0.0 && bounds.y1 <= f64::from(viewport.y),
        "{bounds:?}"
    );
}

fn narrow_context() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    fonts::configure_studio_fonts(&ctx);
    ctx.style_mut(|style| {
        for fonts in style.text_styles.values_mut() {
            fonts.size = fonts.size.max(20.0);
        }
    });
    ctx
}

#[test]
fn linked_h3_review_supports_narrow_keyboard_recalculation_in_both_languages() {
    let fixture = Fixture::new();
    let path = fixture.0.join("linked.rfproj.json");
    let original =
        include_str!("../../../../../examples/flowsheets/h3-linked-water-50.rfproj.json");
    fs::write(&path, original).unwrap();
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        let ctx = narrow_context();
        let mut app = ready_app_state(&synced_workspace_config());
        app.locale = locale;
        let original_document = app.platform_host.document().metadata.document_id.clone();
        app.open_project(path.clone(), "linked review test");
        frame(&mut app, &ctx, vec![]);
        let opened = frame(&mut app, &ctx, vec![]);
        assert!(texts(&opened).contains(&tr(locale, "待运行", "Ready to run")));
        let run = button_node(
            &opened,
            tr(locale, "运行并复核当前工况", "Run and review this case"),
        );
        frame(&mut app, &ctx, vec![focus(run)]);
        let tree = frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        assert_eq!(
            tree.focus,
            button_node(&tree, tr(locale, "取消复核", "Cancel review"))
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let tree = frame(&mut app, &ctx, vec![]);
            let text = texts(&tree);
            if text.contains(&"5877.49 W") {
                assert!(text.contains(&tr(locale, "1 项超限", "1 limit exceeded")));
                assert_eq!(tree.focus, run);
                break;
            }
            assert!(std::time::Instant::now() < deadline, "{text:?}");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(
            app.check_project_event_access(&StudioGuiEvent::UiCommandRequested {
                command_id: "file.save".into(),
            })
            .is_err()
        );
        frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Escape, egui::Modifiers::NONE)],
        );
        assert!(app.read_only_project.is_none());
        assert_eq!(
            app.platform_host.document().metadata.document_id,
            original_document
        );
    }
    assert_eq!(fs::read_to_string(path).unwrap(), original);
}

#[test]
fn h3_review_narrow_keyboard_selects_rates_and_reopens_in_both_languages() {
    let fixture = Fixture::new();
    let path = fixture.project(4, |_| {});
    let bytes = fs::read(&path).unwrap();
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        let ctx = narrow_context();
        let mut app = ready_app_state(&synced_workspace_config());
        app.locale = locale;
        let previous = app.platform_host.document().metadata.document_id.clone();
        app.open_project(path.clone(), "h3 review test");
        frame(&mut app, &ctx, vec![]);
        let tree = frame(&mut app, &ctx, vec![]);
        let first = button_node(&tree, "E-301");
        let second = button_node(&tree, "E-302");
        let rate = button_node(&tree, tr(locale, "复算当前工况", "Rate selected case"));
        let tree = frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::F5, egui::Modifiers::NONE)],
        );
        assert!(texts(&tree).contains(&tr(locale, "待复算", "Not yet rated")));
        frame(&mut app, &ctx, vec![focus(first)]);
        let tree = frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Tab, egui::Modifiers::NONE)],
        );
        assert_eq!(tree.focus, second);
        frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        let tree = frame(&mut app, &ctx, vec![]);
        assert!(texts(&tree).iter().any(|s| s.contains("50 mol/s")));
        assert!(texts(&tree).contains(&tr(locale, "待复算", "Not yet rated")));
        // The selected case has its own stable action ID.
        let second_rate = button_node(&tree, tr(locale, "复算当前工况", "Rate selected case"));
        assert_ne!(rate, second_rate);
        let tree = frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Tab, egui::Modifiers::NONE)],
        );
        assert_eq!(tree.focus, second_rate);
        frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        let tree = frame(&mut app, &ctx, vec![]);
        assert!(texts(&tree).iter().any(|s| s.contains("5877.49")));
        assert!(texts(&tree).iter().any(|s| s.contains(tr(
            locale,
            "1 项超限",
            "1 limit exceeded"
        ))));
        frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Tab, egui::Modifiers::SHIFT)],
        );
        let tree = frame(&mut app, &ctx, vec![]);
        assert_eq!(tree.focus, second);
        frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Tab, egui::Modifiers::SHIFT)],
        );
        let tree = frame(&mut app, &ctx, vec![]);
        assert_eq!(tree.focus, first);
        frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        let tree = frame(&mut app, &ctx, vec![]);
        assert!(texts(&tree).contains(&tr(locale, "待复算", "Not yet rated")));
        assert!(!texts(&tree).iter().any(|s| s.contains("5877.49")));
        let rate = button_node(&tree, tr(locale, "复算当前工况", "Rate selected case"));
        frame(&mut app, &ctx, vec![focus(rate)]);
        frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        let tree = frame(&mut app, &ctx, vec![]);
        assert!(texts(&tree).iter().any(|s| s.contains("5521.71")));
        assert!(!texts(&tree).iter().any(|s| s.contains("5877.49")));
        assert!(
            app.check_project_event_access(&StudioGuiEvent::UiCommandRequested {
                command_id: "file.save".into()
            })
            .is_err()
        );
        app.request_save_project_as(fixture.0.join("must-not-exist.rfproj.json"), false);
        frame(
            &mut app,
            &ctx,
            vec![press(egui::Key::Escape, egui::Modifiers::NONE)],
        );
        assert!(app.read_only_project.is_none());
        assert_eq!(app.platform_host.document().metadata.document_id, previous);
        app.open_project(path.clone(), "h3 reopen");
        let tree = frame(&mut app, &ctx, vec![]);
        assert!(texts(&tree).contains(&tr(locale, "待复算", "Not yet rated")));
        assert!(
            !texts(&tree)
                .iter()
                .any(|s| s.contains("5521.71") || s.contains("5877.49"))
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(!fixture.0.join("must-not-exist.rfproj.json").exists());
    }
}

#[test]
fn h3_review_narrow_diagnostics_locate_and_return_in_both_languages() {
    let fixture = Fixture::new();
    for missing in [true, false] {
        let path = fixture.project(3, |root| {
            let tasks = root["engineering"]["tasks"].as_array_mut().unwrap();
            tasks.truncate(1);
            if missing {
                tasks[0]["geometry"]
                    .as_object_mut()
                    .unwrap()
                    .remove("activeLengthM");
            } else {
                tasks[0]["case"]["cold"]["inlet"]["pressurePa"] = json!(150_000.0);
            }
        });
        let bytes = fs::read(&path).unwrap();
        for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
            let ctx = narrow_context();
            let mut app = ready_app_state(&synced_workspace_config());
            app.locale = locale;
            app.open_project(path.clone(), "h3 diagnostic test");
            frame(&mut app, &ctx, vec![]);
            let tree = frame(&mut app, &ctx, vec![]);
            let rate = button_node(&tree, tr(locale, "复算当前工况", "Rate selected case"));
            frame(&mut app, &ctx, vec![focus(rate)]);
            frame(
                &mut app,
                &ctx,
                vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            let tree = frame(&mut app, &ctx, vec![]);
            let code = if missing {
                "missing_input"
            } else {
                "inlet_outside_domain"
            };
            assert!(texts(&tree).iter().any(|s| s.contains(code)));
            assert!(!texts(&tree).iter().any(|s| s.contains("5521.71")));
            let locate = button_node(&tree, tr(locale, "查看输入字段", "Show input field"));
            frame(&mut app, &ctx, vec![focus(locate)]);
            frame(
                &mut app,
                &ctx,
                vec![press(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            let tree = frame(&mut app, &ctx, vec![]);
            let focus = tree
                .nodes
                .iter()
                .find(|(id, _)| *id == tree.focus)
                .map(|(_, node)| node);
            let expected = if missing {
                tr(locale, "有效换热长度", "Active length")
            } else {
                tr(
                    locale,
                    "入口绝压 · 1.5 bar",
                    "Inlet absolute pressure · 1.5 bar",
                )
            };
            assert!(
                focus.is_some_and(|node| node
                    .label()
                    .or(node.value())
                    .is_some_and(|s| s.contains(expected))),
                "{focus:?}"
            );
            let bounds = focus.unwrap().bounds().unwrap();
            assert!(bounds.y0 >= 0.0 && bounds.y1 <= 820.0, "{bounds:?}");
            frame(
                &mut app,
                &ctx,
                vec![press(egui::Key::Escape, egui::Modifiers::NONE)],
            );
            let tree = frame(&mut app, &ctx, vec![]);
            assert_eq!(tree.focus, locate);
            assert!(app.read_only_project.is_some());
            frame(
                &mut app,
                &ctx,
                vec![press(egui::Key::Escape, egui::Modifiers::NONE)],
            );
            assert!(app.read_only_project.is_none());
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }
}
