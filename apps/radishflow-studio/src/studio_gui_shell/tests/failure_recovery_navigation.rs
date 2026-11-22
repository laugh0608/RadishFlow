use super::*;

fn failed_fixture(name: &str) -> (ReadyAppState, PathBuf) {
    let path = std::env::temp_dir().join(format!(
        "rf-b22-{name}-{}.rfproj.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../examples/flowsheets/failures/{name}.rfproj.json"
    ));
    fs::copy(source, &path).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(
        app.platform_host
            .snapshot()
            .window_model()
            .runtime
            .latest_failure
            .is_some()
    );
    (app, path)
}

fn assert_recovery_located(app: &ReadyAppState, unit_id: &str) {
    let result = app
        .canvas_command_result
        .as_ref()
        .expect("recovery must request canvas navigation");
    assert_eq!(result.status_label, "located");
    assert_eq!(result.target.target_id, unit_id);
    assert!(
        app.canvas_viewport_navigation
            .active_anchor
            .as_ref()
            .unwrap()
            .pending_scroll
    );
    assert_eq!(app.right_sidebar_tab, StudioShellRightSidebarTab::Inspector);
}

#[test]
fn failure_recovery_focus_navigates_for_commands_shortcuts_and_widget_events() {
    for route in 0..3 {
        let (mut app, path) = failed_fixture("unbound-inlet-port");
        let before = app.platform_host.snapshot().runtime.workspace_document;
        app.right_sidebar_tab = StudioShellRightSidebarTab::ModuleResults;
        match route {
            0 => app.dispatch_ui_command("run_panel.recover_failure"),
            1 => dispatch_shortcut_for_test(&mut app, egui::Key::F8, egui::Modifiers::NONE),
            _ => app.dispatch_event(StudioGuiEvent::RunPanelRecoveryRequested),
        }
        assert_recovery_located(&app, "heater-1");
        let window = app.platform_host.snapshot().window_model();
        assert_eq!(
            window.runtime.workspace_document, before,
            "focus must not change revision or dirty state"
        );
        assert!(
            window.runtime.latest_failure.is_some(),
            "focus is not a repair"
        );
        assert!(window.runtime.latest_solve_snapshot.is_none());
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn failure_recovery_mutation_navigates_once_and_disabled_retry_does_not_mutate() {
    let (mut app, path) = failed_fixture("unbound-outlet-port");
    let before = app.platform_host.snapshot().runtime.workspace_document;
    app.dispatch_ui_command("run_panel.recover_failure");
    assert_recovery_located(&app, "feed-1");
    let repaired = app.platform_host.snapshot().window_model();
    assert_eq!(
        repaired.runtime.workspace_document.revision,
        before.revision + 1
    );
    assert!(repaired.runtime.workspace_document.has_unsaved_changes);
    assert!(repaired.runtime.latest_failure.is_none());
    assert!(repaired.runtime.latest_solve_snapshot.is_none());
    let navigation = app.canvas_command_result.clone();
    app.dispatch_ui_command("run_panel.recover_failure");
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        repaired.runtime.workspace_document
    );
    assert_eq!(app.canvas_command_result, navigation);
    app.dispatch_ui_command("edit.undo");
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .stream_count,
        before.stream_count
    );
    app.dispatch_ui_command("edit.redo");
    assert_eq!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .stream_count,
        repaired.runtime.workspace_document.stream_count
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn failure_recovery_deleted_target_clears_navigation_without_missing_anchor_warning() {
    let (mut app, path) = failed_fixture("orphan-stream");
    app.dispatch_ui_command("inspector.focus_stream:stream-orphan");
    app.dispatch_ui_command("run_panel.recover_failure");
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .active_inspector_target
            .is_none()
    );
    assert!(app.canvas_command_result.is_none());
    assert!(app.canvas_viewport_navigation.active_anchor.is_none());
    fs::remove_file(path).unwrap();
}

#[test]
fn failure_recovery_toolbar_distinguishes_focus_and_mutation_and_clears_after_repair() {
    for (fixture, expected, action) in [
        (
            "unbound-inlet-port",
            "定位问题（不修改模型）",
            "Inspect inlet path",
        ),
        (
            "unbound-outlet-port",
            "修复模型（可撤销）",
            "Create outlet stream",
        ),
    ] {
        let (mut app, path) = failed_fixture(fixture);
        app.screen = StudioShellScreen::Run;
        let texts = super::basic::render_top_bar_texts(&mut app);
        for label in [expected, action] {
            assert!(
                texts.iter().any(|text| text.contains(label)),
                "missing {label}: {texts:?}"
            );
        }
        app.dispatch_ui_command("run_panel.recover_failure");
        let texts = super::basic::render_top_bar_texts(&mut app);
        assert_eq!(
            texts.iter().any(|text| text.contains(action)),
            fixture == "unbound-inlet-port"
        );
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn failure_recovery_keyboard_runs_switch_to_failure_or_current_results() {
    let (mut failed, path) = failed_fixture("unbound-inlet-port");
    failed.bottom_drawer_tab = StudioShellBottomDrawerTab::ResultsTable;
    failed.right_sidebar_tab = StudioShellRightSidebarTab::ModuleResults;
    dispatch_shortcut_for_test(&mut failed, egui::Key::F5, egui::Modifiers::NONE);
    assert_eq!(
        failed.bottom_drawer_tab,
        StudioShellBottomDrawerTab::Messages
    );
    assert_eq!(
        failed.right_sidebar_tab,
        StudioShellRightSidebarTab::Inspector
    );
    let mut solved = ready_app_state(&synced_workspace_config());
    dispatch_shortcut_for_test(&mut solved, egui::Key::F5, egui::Modifiers::NONE);
    assert!(
        solved
            .platform_host
            .snapshot()
            .window_model()
            .runtime
            .latest_solve_snapshot
            .is_some()
    );
    assert_eq!(
        solved.bottom_drawer_tab,
        StudioShellBottomDrawerTab::ResultsTable
    );
    assert_eq!(
        solved.right_sidebar_tab,
        StudioShellRightSidebarTab::ModuleResults
    );
    fs::remove_file(path).unwrap();
}

fn diagnostic_frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::accesskit::TreeUpdate {
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 980.0),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |ctx| {
            if width > 400.0 {
                app.update(ctx);
            } else {
                let snapshot = app.platform_host.snapshot();
                let window = snapshot.window_model();
                app.render_top_bar(ctx, &snapshot.app_host_state.windows, &window, &mut false);
                app.prepare_modeling_problem_field_layout(width, 20.0);
                app.render_bottom_drawer(ctx, &window);
            }
        },
    );
    output.platform_output.accesskit_update.unwrap()
}

fn diagnostic_button(
    tree: &egui::accesskit::TreeUpdate,
    label: &str,
    last: bool,
) -> egui::accesskit::NodeId {
    tree.nodes
        .iter()
        .filter(|(_, node)| {
            node.role() == egui::accesskit::Role::Button && node.label() == Some(label)
        })
        .min_by(|(_, a), (_, b)| {
            let order = a.bounds().unwrap().y0.total_cmp(&b.bounds().unwrap().y0);
            if last { order.reverse() } else { order }
        })
        .unwrap_or_else(|| panic!("Missing diagnostic button: {label}"))
        .0
}

fn activate_diagnostic_button(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    width: f32,
    target: egui::accesskit::NodeId,
) {
    use egui::accesskit::{Action, ActionRequest};

    let focused = diagnostic_frame(
        app,
        ctx,
        width,
        vec![egui::Event::AccessKitActionRequest(ActionRequest {
            target,
            action: Action::Focus,
            data: None,
        })],
    );
    assert_eq!(focused.focus, target);
    diagnostic_frame(
        app,
        ctx,
        width,
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: Some(egui::Key::Enter),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            })
            .collect(),
    );
}

#[test]
fn run_diagnostic_keyboard_return_restores_the_original_action_and_draft() {
    for (screen, locale, width, drawer) in [
        (
            StudioShellScreen::Run,
            StudioShellLocale::ZhCn,
            1280.0,
            false,
        ),
        (
            StudioShellScreen::Results,
            StudioShellLocale::En,
            1280.0,
            true,
        ),
        (
            StudioShellScreen::Workbench,
            StudioShellLocale::ZhCn,
            400.0,
            true,
        ),
        (StudioShellScreen::Run, StudioShellLocale::En, 400.0, true),
    ] {
        let (mut app, path) = failed_fixture("unbound-inlet-port");
        app.screen = screen;
        app.locale = locale;
        if screen == StudioShellScreen::Workbench {
            app.bottom_drawer_tab = StudioShellBottomDrawerTab::Diagnostics;
            app.modeling_problems.set_runtime_details(true);
        }
        app.dispatch_ui_command("inspector.focus_unit:heater-1");
        let mut draft = super::modeling_problems::numeric(&app, "unit:heater-1:outlet_pressure_pa");
        assert!(app.edit_numeric_field(
            &mut draft,
            rf_ui::NumericEditEvent::ReplaceText("1e-".into())
        ));
        let before = app.platform_host.snapshot().runtime.workspace_document;
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        fonts::configure_studio_fonts(&ctx);
        if width == 400.0 {
            ctx.style_mut(|style| {
                style
                    .text_styles
                    .get_mut(&egui::TextStyle::Body)
                    .unwrap()
                    .size = 20.0;
                style
                    .text_styles
                    .get_mut(&egui::TextStyle::Button)
                    .unwrap()
                    .size = 20.0;
            });
        }
        diagnostic_frame(&mut app, &ctx, width, vec![]);
        let mut tree = diagnostic_frame(&mut app, &ctx, width, vec![]);
        if screen != StudioShellScreen::Workbench {
            let shortcut = match locale {
                StudioShellLocale::ZhCn => "运行诊断",
                StudioShellLocale::En => "Run diagnostics",
            };
            let opener = diagnostic_button(&tree, shortcut, false);
            activate_diagnostic_button(&mut app, &ctx, width, opener);
            assert_eq!(app.screen, screen);
            assert_eq!(
                app.bottom_drawer_tab,
                StudioShellBottomDrawerTab::Diagnostics
            );
            assert!(app.modeling_problems.runtime_details());
            tree = diagnostic_frame(&mut app, &ctx, width, vec![]);
        }
        let origin = diagnostic_button(&tree, "Inspect inlet path", drawer);
        let origin_bounds = tree
            .nodes
            .iter()
            .find(|(id, _)| *id == origin)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        assert!(origin_bounds.x0 >= 0.0 && origin_bounds.x1 <= f64::from(width));
        activate_diagnostic_button(&mut app, &ctx, width, origin);
        assert!(app.at_run_diagnostic_target());
        assert_eq!(app.screen, StudioShellScreen::Workbench);
        assert_eq!(
            app.platform_host
                .snapshot()
                .runtime
                .workspace_document
                .input_edits,
            before.input_edits
        );
        let tree = diagnostic_frame(&mut app, &ctx, width, vec![]);
        assert!(
            tree.nodes
                .iter()
                .any(|(id, node)| *id == tree.focus && node.value() == Some("→ inlet")),
            "focus must land on the structured port target for {screen:?}, {locale:?}, {width}"
        );
        let return_label = match locale {
            StudioShellLocale::ZhCn => "← 返回原运行诊断",
            StudioShellLocale::En => "← Back to originating run diagnostic",
        };
        let return_button = diagnostic_button(&tree, return_label, false);
        activate_diagnostic_button(&mut app, &ctx, width, return_button);
        assert_eq!(app.screen, screen);
        assert_eq!(
            app.bottom_drawer_tab,
            StudioShellBottomDrawerTab::Diagnostics
        );
        assert!(app.modeling_problems.runtime_details());
        let tree = diagnostic_frame(&mut app, &ctx, width, vec![]);
        assert_eq!(tree.focus, origin);
        assert_eq!(
            app.platform_host.snapshot().runtime.workspace_document,
            before
        );
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn run_diagnostic_return_rejects_an_old_attempt_and_same_file_reopen() {
    use crate::studio_gui_shell::flow_diagnostics::RunDiagnosticSurface;

    let (mut app, path) = failed_fixture("unbound-inlet-port");
    app.screen = StudioShellScreen::Run;
    app.bottom_drawer_tab = StudioShellBottomDrawerTab::Diagnostics;
    app.modeling_problems.set_runtime_details(true);
    app.locale = StudioShellLocale::En;
    let action = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_failure
        .unwrap()
        .recovery_action
        .unwrap();
    app.activate_run_diagnostic(RunDiagnosticSurface::Drawer, egui::Id::NULL, &action, None);
    assert!(app.at_run_diagnostic_target());
    app.dispatch_ui_command("run_panel.run_manual");
    assert!(!action.result_is_current(&app.platform_host.modeling_inputs().workspace));

    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let tree = diagnostic_frame(&mut app, &ctx, 1280.0, vec![]);
    let back = diagnostic_button(&tree, "← Back to originating run diagnostic", false);
    activate_diagnostic_button(&mut app, &ctx, 1280.0, back);
    let tree = diagnostic_frame(&mut app, &ctx, 1280.0, vec![]);
    assert_eq!(app.screen, StudioShellScreen::Run);
    assert_eq!(
        app.bottom_drawer_tab,
        StudioShellBottomDrawerTab::Diagnostics
    );
    assert!(tree.nodes.iter().any(|(id, node)| *id == tree.focus
        && node.value()
            == Some("The run or inputs changed. The original diagnostic is no longer current.")));

    let current = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_failure
        .unwrap()
        .recovery_action
        .unwrap();
    app.activate_run_diagnostic(RunDiagnosticSurface::Drawer, egui::Id::NULL, &current, None);
    assert!(app.at_run_diagnostic_target());
    app.open_project(path.clone(), "same file");
    assert!(app.run_diagnostic_navigation.is_none());
    fs::remove_file(path).unwrap();
}

#[test]
fn run_diagnostic_return_reports_a_removed_target_at_the_original_list() {
    use crate::studio_gui_shell::flow_diagnostics::RunDiagnosticSurface;

    let (mut app, path) = failed_fixture("unbound-inlet-port");
    app.screen = StudioShellScreen::Results;
    app.bottom_drawer_tab = StudioShellBottomDrawerTab::ResultsTable;
    app.locale = StudioShellLocale::En;
    let origin = app
        .platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_failure
        .unwrap();
    let action = origin.recovery_target.unwrap().action;
    app.activate_run_diagnostic(RunDiagnosticSurface::Drawer, egui::Id::NULL, &action, None);
    assert!(app.at_run_diagnostic_target());
    app.request_unit_deletion();
    app.confirm_unit_deletion();
    assert!(
        !app.platform_host
            .document()
            .flowsheet
            .units
            .contains_key(&UnitId::new("heater-1"))
    );

    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    let tree = diagnostic_frame(&mut app, &ctx, 1280.0, vec![]);
    let back = diagnostic_button(&tree, "← Back to originating run diagnostic", false);
    activate_diagnostic_button(&mut app, &ctx, 1280.0, back);
    let tree = diagnostic_frame(&mut app, &ctx, 1280.0, vec![]);
    assert_eq!(app.screen, StudioShellScreen::Results);
    assert_eq!(
        app.bottom_drawer_tab,
        StudioShellBottomDrawerTab::ResultsTable
    );
    assert!(tree.nodes.iter().any(|(id, node)| *id == tree.focus
        && node.value() == Some("The original diagnostic target was removed.")));
    fs::remove_file(path).unwrap();
}
