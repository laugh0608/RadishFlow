use super::*;

#[test]
fn invalid_v2_presentation_does_not_replace_the_open_workspace_or_results() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    let before = app.platform_host.snapshot().window_model();
    let recent = app.project_open.recent_projects.clone();
    let path = test_preferences_path("invalid-project-presentation").with_extension("rfproj.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let invalid =
        include_str!("../../../../../crates/rf-store/tests/fixtures/project-v2-engineering.json")
            .replace("\"celsius\"", "\"future-unit\"");
    std::fs::write(&path, &invalid).unwrap();
    app.project_open.path_input = path.display().to_string();
    app.open_project_from_input();
    let after = app.platform_host.snapshot().window_model();
    assert_eq!(
        after.runtime.workspace_document,
        before.runtime.workspace_document
    );
    assert_eq!(
        after.runtime.latest_solve_snapshot,
        before.runtime.latest_solve_snapshot
    );
    assert_eq!(app.project_open.recent_projects, recent);
    let notice = app.project_open.notice.as_ref().unwrap();
    assert_eq!(notice.level, ProjectOpenNoticeLevel::Error);
    assert!(notice.detail.contains("unknown unit ID"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), invalid);
    std::fs::remove_file(path).unwrap();
}
