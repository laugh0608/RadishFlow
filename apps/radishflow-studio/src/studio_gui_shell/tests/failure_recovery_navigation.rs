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
