use super::*;

struct ProjectFile(std::path::PathBuf);
impl ProjectFile {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("radishflow-vbd04-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        Self(directory.join("review.rfproj.json"))
    }
}
impl Drop for ProjectFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.0.parent().unwrap());
    }
}

#[test]
fn browser_save_keeps_drafts_then_reopen_rejects_old_same_document_controls() {
    let project = ProjectFile::new();
    let mut app = app();
    app.request_save_project_as(project.0.clone(), false);
    app.confirm_format_upgrade();
    assert!(project.0.exists());
    let selection = select(
        &mut app,
        ObjectId::Unit("heater-1".into()),
        VariableField::OutletTemperature,
    );
    let mut retired_begin = field(&app, &selection);
    let mut current = retired_begin.clone();
    assert!(app.edit_numeric_field(&mut current, E::ReplaceText("1e-".into())));
    app.save_project();
    let stored = radishflow_studio::load_project_app_state(&project.0).unwrap();
    assert_eq!(
        stored.workspace.document.flowsheet.units[&"heater-1".into()]
            .parameters
            .outlet_temperature_k,
        Some(345.0)
    );
    assert_eq!(field(&app, &selection).text, "1e-");
    assert!(
        app.project_open
            .notice
            .as_ref()
            .unwrap()
            .detail
            .contains("未提交输入 1")
    );
    assert!(!app.close_current_window_for_viewport_request());
    app.cancel_pending_close_window();
    assert_eq!(field(&app, &selection).text, "1e-");
    assert!(app.edit_numeric_field(&mut current, E::ReplaceText("350".into())));
    let mut retired = current.clone();
    assert!(commit(&mut app, &mut current));
    app.save_project();
    let id = app.platform_host.document().metadata.document_id.clone();
    app.open_project(project.0.clone(), "test");
    app.reconcile_browser_session();
    assert_eq!(app.platform_host.document().metadata.document_id, id);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_result()
            .status(),
        ResultStatus::Missing
    );
    assert_eq!(
        app.platform_host.document().flowsheet.units[&"heater-1".into()]
            .parameters
            .outlet_temperature_k,
        Some(350.0)
    );
    let selection = select(
        &mut app,
        ObjectId::Unit("heater-1".into()),
        VariableField::OutletTemperature,
    );
    let mut fresh = field(&app, &selection);
    // Match the retired control's generation, proving generation alone is insufficient.
    assert!(app.edit_numeric_field(&mut fresh, E::ReplaceText("355".into())));
    assert!(app.edit_numeric_field(&mut fresh, E::ReplaceText("360".into())));
    assert_eq!(fresh.generation, retired.generation);
    let before = app.platform_host.document().clone();
    assert!(!commit(&mut app, &mut retired));
    let command = NumericEditCommand::Begin(retired_begin.variable.clone());
    assert!(!app.dispatch_numeric_command(&mut retired_begin, command));
    assert!(!app.edit_numeric_field(&mut retired, E::ReplaceText("999".into())));
    assert_eq!(app.platform_host.document(), &before);
    assert_eq!(field(&app, &selection).text, "360");
    assert!(commit(&mut app, &mut fresh));
    assert_eq!(
        app.platform_host.document().flowsheet.units[&"heater-1".into()]
            .parameters
            .outlet_temperature_k,
        Some(360.0)
    );
}

#[test]
fn browser_read_only_overlay_rejects_edit_and_preserves_original_file() {
    let project = ProjectFile::new();
    let mut app = app();
    app.request_object_identity_initialization();
    app.prepare_identity_destination(project.0.clone());
    app.confirm_identity_initialization();
    let selection = select(
        &mut app,
        ObjectId::Stream("stream-feed".into()),
        VariableField::Pressure,
    );
    let mut current = field(&app, &selection);
    assert!(app.edit_numeric_field(&mut current, E::ReplaceText("110000".into())));
    let mut opaque: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&project.0).unwrap()).unwrap();
    opaque["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"id": "future.editor", "version": 1}));
    let original = serde_json::to_string(&opaque).unwrap();
    std::fs::write(&project.0, &original).unwrap();
    let before = app.platform_host.document().clone();
    app.open_project(project.0.clone(), "test");
    assert!(app.read_only_project.is_some());
    assert!(presentation(&app, &app.variable_browser, &selection).is_none());
    assert!(!commit(&mut app, &mut current));
    assert_eq!(app.platform_host.document(), &before);
    assert_eq!(std::fs::read_to_string(&project.0).unwrap(), original);
}

#[test]
fn browser_commit_keeps_watch_references_and_order_across_run_and_reopen() {
    use rf_ui::variable_browser::VariableWatchCommand;
    let project = ProjectFile::new();
    let mut app = app();
    app.request_object_identity_initialization();
    app.prepare_identity_destination(project.0.clone());
    app.confirm_identity_initialization();
    let selection = select(
        &mut app,
        ObjectId::Unit("heater-1".into()),
        VariableField::OutletTemperature,
    );
    let result = flow_selection(
        &app,
        ObjectId::Stream("stream-heated".into()),
        VariableSection::Results,
        VariableField::Temperature,
    );
    for target in [&result, &selection] {
        app.edit_browser_watch_list(VariableWatchCommand::Add(
            references::selected_variable(target).unwrap(),
        ));
    }
    let order = app
        .platform_host
        .modeling_inputs()
        .workspace
        .variable_watch_list
        .entries()
        .to_vec();
    app.dispatch_ui_command("run_panel.run_manual");
    let mut current = field(&app, &selection);
    assert!(app.edit_numeric_field(&mut current, E::ReplaceText("350".into())));
    assert!(commit(&mut app, &mut current));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_result()
            .status(),
        ResultStatus::Stale
    );
    app.dispatch_ui_command("run_panel.run_manual");
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_result()
            .status(),
        ResultStatus::Current
    );
    let value = read_selection(
        &app.platform_host.modeling_inputs().workspace,
        &result,
        app.locale,
    )
    .unwrap();
    assert!(value.value.contains("350"), "{}", value.value);
    app.save_project();
    app.confirm_format_upgrade();
    app.open_project(project.0.clone(), "test");
    let workspace = &app.platform_host.modeling_inputs().workspace;
    assert_eq!(workspace.variable_watch_list.entries(), order);
    assert_eq!(workspace.flow_result().status(), ResultStatus::Missing);
    assert_eq!(
        workspace.document.flowsheet.units[&"heater-1".into()]
            .parameters
            .outlet_temperature_k,
        Some(350.0)
    );
    for reference in workspace.variable_watch_list.entries() {
        reference
            .parse::<StableVariableReference>()
            .unwrap()
            .resolve(workspace)
            .unwrap();
    }
}

#[test]
fn browser_commit_retires_inflight_result_before_completion_is_polled() {
    let mut app = app();
    app.dispatch_ui_command("run_panel.run_manual");
    app.enable_background_runs();
    app.dispatch_ui_command("run_panel.run_manual");
    let old_task = app
        .background_runs
        .as_ref()
        .unwrap()
        .borrow()
        .activity()
        .unwrap()
        .id;
    let selection = select(
        &mut app,
        ObjectId::Unit("heater-1".into()),
        VariableField::OutletTemperature,
    );
    let mut current = field(&app, &selection);
    app.edit_numeric_field(&mut current, E::ReplaceText("350".into()));
    assert!(commit(&mut app, &mut current));
    let ctx = egui::Context::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while app.background_busy() {
        assert!(std::time::Instant::now() < deadline);
        app.poll_background_runs(&ctx);
        std::thread::yield_now();
    }
    let owner = app.background_runs.as_ref().unwrap().borrow();
    let old = owner
        .history()
        .iter()
        .find(|item| item.activity.id == old_task)
        .unwrap();
    assert!(!old.applied, "old input completion must not become current");
    assert_eq!(
        owner.history().len(),
        1,
        "Hold mode requires an explicit new run"
    );
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_result()
            .status(),
        ResultStatus::Stale
    );
}
