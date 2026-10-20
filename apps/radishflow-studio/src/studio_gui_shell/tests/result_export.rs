use super::*;

fn current_snapshot(app: &ReadyAppState) -> radishflow_studio::StudioGuiWindowSolveSnapshotModel {
    app.platform_host
        .snapshot()
        .window_model()
        .runtime
        .latest_solve_snapshot
        .unwrap()
}

fn solved_app() -> ReadyAppState {
    let mut app = ready_app_state(&synced_workspace_config());
    app.dispatch_ui_command("run_panel.run_manual");
    app
}

fn output_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rf-result-export-{name}-{}.txt",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn result_export_rejects_stale_missing_and_replaced_snapshots() {
    let mut app = solved_app();
    let old = current_snapshot(&app);
    app.dispatch_ui_command("run_panel.run_manual");
    let new = current_snapshot(&app);
    assert_ne!(old.snapshot_id, new.snapshot_id);
    let path = output_path("stale");
    for snapshot in [&old, &new] {
        if snapshot == &new {
            app.dispatch_ui_command("canvas.begin_place_unit.heater");
            app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(500.0, 200.0));
            assert!(
                app.platform_host
                    .snapshot()
                    .window_model()
                    .runtime
                    .latest_solve_snapshot
                    .is_none()
            );
        }
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            app.copy_solve_snapshot_to_clipboard(ctx, snapshot)
        });
        assert!(
            !output
                .platform_output
                .commands
                .iter()
                .any(|command| matches!(command, egui::OutputCommand::CopyText(_)))
        );
        app.export_solve_snapshot_to_path(snapshot, path.clone());
        assert!(!path.exists());
        assert_eq!(
            app.project_open.notice.as_ref().unwrap().level,
            ProjectOpenNoticeLevel::Warning
        );
    }
    let (config, project_path) = blank_workspace_config();
    let mut missing = ready_app_state(&config);
    missing.export_solve_snapshot_to_path(&old, path.clone());
    assert!(!path.exists());
    fs::remove_file(project_path).unwrap();
}

#[test]
fn result_export_overwrite_requires_confirmation_and_rechecks_currentness() {
    let mut app = solved_app();
    let snapshot = current_snapshot(&app);
    let path = output_path("overwrite");
    fs::write(&path, "original").unwrap();
    app.export_solve_snapshot_to_path(&snapshot, path.clone());
    assert!(app.pending_result_export.is_some());
    assert_eq!(
        app.focus_context(&egui::Context::default()),
        StudioGuiFocusContext::ModalDialog
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    app.cancel_result_export();
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    app.export_solve_snapshot_to_path(&snapshot, path.clone());
    app.dispatch_ui_command("run_panel.run_manual");
    app.confirm_result_export();
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    app.export_solve_snapshot_to_path(&current_snapshot(&app), path.clone());
    let before = app.platform_host.snapshot().runtime.workspace_document;
    app.confirm_result_export();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        current_snapshot(&app).light_text_export().unwrap()
    );
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    fs::remove_file(path).unwrap();
}

struct UnsupportedResultPicker;
impl ProjectFilePicker for UnsupportedResultPicker {
    fn supports_result_export_dialogs(&self) -> bool {
        false
    }
    fn pick_project_file(&mut self) -> Option<PathBuf> {
        unreachable!()
    }
    fn pick_save_project_file(&mut self) -> Option<PathBuf> {
        unreachable!()
    }
    fn pick_result_export_file(&mut self) -> Option<PathBuf> {
        panic!("unsupported picker must not open")
    }
}

#[test]
fn result_export_distinguishes_cancel_unsupported_and_write_failure() {
    let mut app = solved_app();
    let snapshot = current_snapshot(&app);
    let before = app.platform_host.snapshot().runtime.workspace_document;
    app.project_file_picker = Box::new(TestProjectFilePicker::new(None));
    app.export_solve_snapshot_from_picker(&snapshot);
    let canceled = app.project_open.notice.as_ref().unwrap().clone();
    assert_eq!(canceled.level, ProjectOpenNoticeLevel::Info);
    app.project_file_picker = Box::new(UnsupportedResultPicker);
    app.export_solve_snapshot_from_picker(&snapshot);
    let unsupported = app.project_open.notice.as_ref().unwrap();
    assert_eq!(unsupported.level, ProjectOpenNoticeLevel::Warning);
    assert_ne!(unsupported.title, canceled.title);
    let path = output_path("directory");
    fs::create_dir(&path).unwrap();
    fs::write(path.join("keep"), "original").unwrap();
    app.export_solve_snapshot_to_path(&snapshot, path.clone());
    assert_eq!(
        app.project_open.notice.as_ref().unwrap().level,
        ProjectOpenNoticeLevel::Error
    );
    assert_eq!(fs::read_to_string(path.join("keep")).unwrap(), "original");
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn result_export_normalizes_extension_before_overwrite_confirmation() {
    let mut app = solved_app();
    let snapshot = current_snapshot(&app);
    let path = output_path("中文 空格");
    fs::write(&path, "original").unwrap();
    app.project_file_picker = Box::new(TestProjectFilePicker::new(Some(path.with_extension(""))));
    app.export_solve_snapshot_from_picker(&snapshot);
    assert!(app.pending_result_export.is_some());
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    app.cancel_result_export();
    fs::remove_file(path).unwrap();
    for name in ["结果.TXT", "result.txt"] {
        assert_eq!(
            project_picker::ensure_text_file_extension(PathBuf::from(name)),
            PathBuf::from(name)
        );
    }
}

#[test]
fn result_export_uses_fresh_payload_and_records_revision_without_changing_project() {
    let mut app = solved_app();
    let current = current_snapshot(&app);
    let mut rendered = current.clone();
    rendered.summary = "outdated render payload".to_string();
    let before = app.platform_host.snapshot().runtime.workspace_document;
    let path = output_path("current");
    app.export_solve_snapshot_to_path(&rendered, path.clone());
    let text = fs::read_to_string(&path).unwrap();
    assert_eq!(text, current.light_text_export().unwrap());
    assert!(text.contains(&format!("document_revision: {}", before.revision)));
    for unit in [" K", " Pa", " mol/s", " J/mol"] {
        assert!(text.contains(unit), "missing {unit}");
    }
    assert_eq!(
        app.platform_host.snapshot().runtime.workspace_document,
        before
    );
    fs::remove_file(path).unwrap();
}

#[test]
fn pending_export_keeps_invocation_units_and_new_copy_uses_current_project_units() {
    use rf_types::units::DisplayUnitSet;
    use rf_ui::ProjectPresentationCommand as P;
    let mut app = solved_app();
    let si = current_snapshot(&app);
    app.apply_presentation_command(P::Apply(DisplayUnitSet::engineering()))
        .unwrap();
    let captured = current_snapshot(&app).light_text_export().unwrap();
    let path = output_path("captured-units");
    fs::write(&path, "original").unwrap();
    // Even a prior render payload must use authoritative project units at invocation.
    app.export_solve_snapshot_to_path(&si, path.clone());
    app.apply_presentation_command(P::Apply(DisplayUnitSet::si()))
        .unwrap();
    app.confirm_result_export();
    assert_eq!(fs::read_to_string(&path).unwrap(), captured);
    let ctx = egui::Context::default();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        app.copy_solve_snapshot_to_clipboard(ctx, &si);
    });
    let expected = si.light_text_export().unwrap();
    assert!(output.platform_output.commands.iter().any(
        |command| matches!(command, egui::OutputCommand::CopyText(text) if text == &expected)
    ));
    fs::remove_file(path).unwrap();
}

#[test]
fn pending_export_rejects_document_edits_and_is_cleared_on_project_switch() {
    let mut app = solved_app();
    let snapshot = current_snapshot(&app);
    let path = output_path("revision-project");
    fs::write(&path, "original").unwrap();
    app.export_solve_snapshot_to_path(&snapshot, path.clone());
    app.dispatch_ui_command("canvas.begin_place_unit.heater");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(500.0, 200.0));
    app.confirm_result_export();
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");

    let mut app = solved_app();
    app.export_solve_snapshot_to_path(&current_snapshot(&app), path.clone());
    let (config, project) = blank_workspace_config();
    app.open_project(config.project_path, "test");
    assert!(app.pending_result_export.is_none());
    app.confirm_result_export();
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    fs::remove_file(path).unwrap();
    fs::remove_file(project).unwrap();
}

#[test]
fn conversion_failure_preserves_file_and_clipboard_without_invalidating_si_result() {
    use rf_types::units::{DisplayUnitSet, MeasurementUnit as U, QuantityKind as Q};
    let project_path = output_path("overflow-input").with_extension("rfproj.json");
    let mut project = feed_heater_flash_binary_hydrocarbon_project();
    project
        .document
        .flowsheet
        .streams
        .get_mut(&StreamId::new("stream-feed"))
        .unwrap()
        .total_molar_flow_mol_s = 1e305;
    write_project_file(&project_path, &project).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: project_path.clone(),
        ..synced_workspace_config()
    });
    app.dispatch_ui_command("run_panel.run_manual");
    let si = current_snapshot(&app);
    assert!(si.presentation_errors.is_empty());
    let before = app.platform_host.snapshot().runtime.latest_solve_snapshot;
    let mut units = DisplayUnitSet::si();
    units.set_unit(Q::MolarFlow, U::MolePerHour).unwrap();
    app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(units))
        .unwrap();
    assert!(!current_snapshot(&app).presentation_errors.is_empty());
    let path = output_path("overflow-protected");
    fs::write(&path, "original").unwrap();
    app.export_solve_snapshot_to_path(&si, path.clone());
    assert!(app.pending_result_export.is_none());
    assert_eq!(fs::read_to_string(&path).unwrap(), "original");
    // A failed conversion must not open even a supported picker.
    struct MustNotOpenPicker;
    impl ProjectFilePicker for MustNotOpenPicker {
        fn supports_result_export_dialogs(&self) -> bool {
            true
        }
        fn pick_project_file(&mut self) -> Option<PathBuf> {
            unreachable!()
        }
        fn pick_save_project_file(&mut self) -> Option<PathBuf> {
            unreachable!()
        }
        fn pick_result_export_file(&mut self) -> Option<PathBuf> {
            panic!("conversion must precede picker")
        }
    }
    app.project_file_picker = Box::new(MustNotOpenPicker);
    app.export_solve_snapshot_from_picker(&si);
    let ctx = egui::Context::default();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        app.copy_solve_snapshot_to_clipboard(ctx, &si);
    });
    assert!(
        !output
            .platform_output
            .commands
            .iter()
            .any(|command| matches!(command, egui::OutputCommand::CopyText(_)))
    );
    assert_eq!(
        app.project_open.notice.as_ref().unwrap().level,
        ProjectOpenNoticeLevel::Error
    );
    assert_eq!(
        app.platform_host.snapshot().runtime.latest_solve_snapshot,
        before
    );
    fs::remove_file(path).unwrap();
    fs::remove_file(project_path).unwrap();
}
