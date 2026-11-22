use super::*;
use egui::accesskit::{Action, ActionRequest, Role};
use rf_types::StreamId;
use rf_ui::result_query::{FlowResultView, ResultStatus};
use rf_ui::variable_browser::{
    VariableField, VariableSection, VariableWatchCommand, VariableWatchList,
};
use rf_ui::{NumericEditEvent, NumericFieldPresentation};
use std::time::{Instant, UNIX_EPOCH};

struct ReviewProject(PathBuf);

impl Drop for ReviewProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.0.parent().unwrap());
    }
}

fn fixture() -> (ReadyAppState, ReviewProject, [VariableId; 2]) {
    use crate::studio_gui_shell::tests::{ready_app_state, synced_workspace_config};
    let config = StudioRuntimeConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"),
        ..synced_workspace_config()
    };
    let mut app = ready_app_state(&config);
    let directory = std::env::temp_dir().join(format!(
        "radishflow-vb2-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    std::fs::create_dir(&directory).unwrap();
    let project = ReviewProject(directory.join("batch.rfproj.json"));
    app.request_object_identity_initialization();
    app.prepare_identity_destination(project.0.clone());
    app.confirm_identity_initialization();
    assert!(project.0.exists());
    app.open_variable_browser();
    app.variable_browser
        .synchronize(&app.platform_host.modeling_inputs().workspace, app.locale);
    app.variable_browser.directory = DirectoryView::WatchList;
    let variables = ["stream-feed-a", "stream-feed-b"].map(|id| VariableId {
        document: app.platform_host.document().metadata.document_id.clone(),
        object: ObjectId::Stream(id.into()),
        section: VariableSection::Inputs,
        field: VariableField::MolarFlow,
    });
    for id in &variables {
        app.edit_browser_watch_list(VariableWatchCommand::Add(id.clone()));
    }
    (app, project, variables)
}

fn field(app: &ReadyAppState, variable: &VariableId) -> NumericFieldPresentation {
    app.platform_host
        .numeric_field_presentation(variable, None)
        .unwrap()
}

fn draft(app: &mut ReadyAppState, variable: &VariableId, raw: &str) {
    let mut field = field(app, variable);
    assert!(app.edit_numeric_field(&mut field, NumericEditEvent::ReplaceText(raw.into())));
}

fn flow(app: &ReadyAppState) -> [f64; 2] {
    ["stream-feed-a", "stream-feed-b"].map(|id| {
        app.platform_host.document().flowsheet.streams[&StreamId::new(id)].total_molar_flow_mol_s
    })
}

fn result(app: &ReadyAppState) -> ResultStatus {
    app.platform_host
        .modeling_inputs()
        .workspace
        .flow_result()
        .status()
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
                egui::vec2(400.0, 1100.0),
            )),
            focused: true,
            events,
            ..Default::default()
        },
        |ctx| {
            app.dispatch_shortcuts(ctx);
            egui::CentralPanel::default().show(ctx, |ui| app.render_variable_browser_content(ui));
        },
    )
}

fn activate(app: &mut ReadyAppState, ctx: &egui::Context, role: Role, label: &str) {
    frame(app, ctx, vec![]);
    let output = frame(app, ctx, vec![]);
    let target = output
        .platform_output
        .accesskit_update
        .unwrap()
        .nodes
        .into_iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {role:?}: {label}"))
        .0;
    frame(
        app,
        ctx,
        vec![egui::Event::AccessKitActionRequest(ActionRequest {
            action: Action::Click,
            target,
            data: None,
        })],
    );
}

fn key(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

fn open_batch(app: &mut ReadyAppState, ctx: &egui::Context) {
    activate(app, ctx, Role::CheckBox, "Select Feed A flow");
    activate(app, ctx, Role::CheckBox, "Select Feed B flow");
    activate(app, ctx, Role::Button, "Edit selected flows");
    assert!(app.variable_browser.watch.batch.open);
}

#[test]
fn watch_batch_checks_then_applies_and_retires_old_background_run() {
    let (mut app, project, ids) = fixture();
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("run_panel.run_manual");
    assert_eq!(result(&app), ResultStatus::Current);
    app.enable_background_runs();
    app.dispatch_ui_command("run_panel.run_manual");
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    open_batch(&mut app, &ctx);
    draft(&mut app, &ids[0], "36 kmol/h");
    draft(&mut app, &ids[1], "1e-");
    activate(&mut app, &ctx, Role::Button, "Check 2 drafts");
    assert!(matches!(
        app.variable_browser.watch.batch.rejected,
        Some(FeedMolarFlowBatchError::InvalidDrafts(_))
    ));
    assert_eq!(flow(&app), [2.0, 3.0]);
    assert_eq!(result(&app), ResultStatus::Current);
    activate(&mut app, &ctx, Role::Button, "Edit first failed item");
    let update = frame(&mut app, &ctx, vec![])
        .platform_output
        .accesskit_update
        .unwrap();
    assert!(update.nodes.iter().any(|(id, node)| {
        *id == update.focus
            && node
                .label()
                .is_some_and(|label| label.starts_with("Feed B total molar flow"))
    }));
    frame(&mut app, &ctx, vec![key(egui::Key::Tab)]);
    assert_eq!(field(&app, &ids[0]).text, "36 kmol/h");
    assert_eq!(field(&app, &ids[1]).text, "1e-");
    activate(&mut app, &ctx, Role::Button, "Edit first failed item");
    frame(&mut app, &ctx, vec![key(egui::Key::Escape)]);
    assert!(app.variable_browser.watch.batch.open);
    assert_eq!(field(&app, &ids[1]).text, "1e-");
    activate(&mut app, &ctx, Role::Button, "Edit first failed item");
    let mut second = field(&app, &ids[1]);
    assert!(app.edit_numeric_field(&mut second, NumericEditEvent::ReplaceText("5".into())));
    frame(&mut app, &ctx, vec![key(egui::Key::Enter)]);
    assert_eq!(flow(&app), [2.0, 3.0]);
    activate(&mut app, &ctx, Role::Button, "Check 2 drafts");
    assert!(app.variable_browser.watch.batch.checked.is_some());
    draft(&mut app, &ids[1], "6");
    frame(&mut app, &ctx, vec![]);
    assert!(app.variable_browser.watch.batch.checked.is_none());
    draft(&mut app, &ids[1], "5");
    activate(&mut app, &ctx, Role::Button, "Check 2 drafts");
    activate(&mut app, &ctx, Role::Button, "Apply 2 items");
    assert_eq!(flow(&app), [10.0, 5.0]);
    assert_eq!(app.platform_host.document().revision, 1);
    assert_eq!(result(&app), ResultStatus::Stale);
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    while app.background_busy() {
        assert!(Instant::now() < deadline);
        app.poll_background_runs(&ctx);
        std::thread::yield_now();
    }
    {
        let owner = app.background_runs.as_ref().unwrap().borrow();
        assert_eq!(owner.history().len(), 1);
        assert!(!owner.history()[0].applied);
    }
    app.dispatch_ui_command("edit.undo");
    assert_eq!(flow(&app), [2.0, 3.0]);
    app.dispatch_ui_command("edit.redo");
    assert_eq!(flow(&app), [10.0, 5.0]);
    app.save_project();
    app.confirm_format_upgrade();
    let saved = radishflow_studio::load_project_app_state(&project.0).unwrap();
    let workspace = &app.platform_host.modeling_inputs().workspace;
    assert_eq!(
        saved.workspace.document.flowsheet,
        workspace.document.flowsheet
    );
    assert_eq!(
        saved.workspace.variable_watch_list.entries(),
        workspace.variable_watch_list.entries()
    );
    draft(&mut app, &ids[0], "12");
    draft(&mut app, &ids[1], "6");
    let checked = app
        .platform_host
        .modeling_inputs()
        .workspace
        .check_feed_molar_flow_batch(&ids)
        .unwrap();
    app.open_project(project.0.clone(), "reopen batch test");
    assert_eq!(
        apply(&mut app, checked).unwrap(),
        Err(FeedMolarFlowBatchError::StaleCheck)
    );
    draft(&mut app, &ids[0], "12");
    draft(&mut app, &ids[1], "6");
    let checked = app
        .platform_host
        .modeling_inputs()
        .workspace
        .check_feed_molar_flow_batch(&ids)
        .unwrap();
    let mut read_only: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&project.0).unwrap()).unwrap();
    read_only["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "id": "test.read-only", "version": 1
        }));
    let read_only_path = project.0.with_file_name("readonly.rfproj.json");
    std::fs::write(&read_only_path, read_only.to_string()).unwrap();
    app.open_project(read_only_path, "read-only batch test");
    assert!(app.read_only_project.is_some());
    let before = app.platform_host.modeling_inputs().clone();
    assert!(apply(&mut app, checked).is_err());
    assert_eq!(app.platform_host.modeling_inputs(), &before);
}

#[test]
fn watch_batch_active_runs_once_on_committed_flows() {
    let (mut app, _project, ids) = fixture();
    app.locale = StudioShellLocale::En;
    app.dispatch_ui_command("run_panel.run_manual");
    app.enable_background_runs();
    app.dispatch_ui_command("run_panel.set_active");
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    open_batch(&mut app, &ctx);
    draft(&mut app, &ids[0], "36 kmol/h");
    draft(&mut app, &ids[1], "5");
    activate(&mut app, &ctx, Role::Button, "Check 2 drafts");
    app.poll_background_runs(&ctx);
    assert!(!app.background_busy());
    assert_eq!(flow(&app), [2.0, 3.0]);
    activate(&mut app, &ctx, Role::Button, "Apply 2 items");
    assert_eq!(result(&app), ResultStatus::Stale);

    app.poll_background_runs(&ctx);
    assert!(app.background_busy());
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    while app.background_busy() {
        assert!(Instant::now() < deadline);
        app.poll_background_runs(&ctx);
        std::thread::yield_now();
    }
    app.poll_background_runs(&ctx);
    assert!(!app.background_busy());
    let owner = app.background_runs.as_ref().unwrap().borrow();
    assert_eq!(owner.history().len(), 1);
    assert!(owner.history()[0].applied);
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let FlowResultView::Current { snapshot, .. } = workspace.flow_result() else {
        panic!("automatic run should publish the committed batch");
    };
    assert_eq!(snapshot.document_revision, 1);
    let mixer = snapshot.unit_result(&"mixer-1".into()).unwrap();
    for (id, expected) in [("stream-feed-a", 10.0), ("stream-feed-b", 5.0)] {
        let feed = mixer
            .consumed_streams
            .iter()
            .find(|stream| stream.stream_id.as_str() == id)
            .unwrap();
        assert_eq!(feed.total_molar_flow_mol_s, expected);
    }
}

#[test]
fn stale_watch_reference_cannot_select_a_recreated_stream() {
    let (app, _project, ids) = fixture();
    let mut workspace = app.platform_host.modeling_inputs().workspace.clone();
    let mut entries = workspace.variable_watch_list.entries().to_vec();
    let old_reference = entries[0]
        .split('/')
        .enumerate()
        .map(|(index, segment)| {
            if index == 4 {
                "f68b48c3-618b-4b88-99d5-c558a1305375"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/");
    entries.push(old_reference.clone());
    workspace.variable_watch_list = VariableWatchList::from_saved(Some(entries.clone()));
    assert!(eligible(&workspace, &entries[0]));
    assert!(!eligible(&workspace, &old_reference));

    let mut batch = FeedMolarFlowBatchView::default();
    batch.select(&old_reference, true);
    batch.select(&entries[1], true);
    batch.retain_watched(&workspace);
    assert!(matches!(
        batch.rejected,
        Some(FeedMolarFlowBatchError::StaleCheck)
    ));
    assert_eq!(batch.variables(&workspace), ids[1..]);
}

#[test]
fn batch_check_and_failure_controls_fit_400_pixels_in_both_languages() {
    for locale in [StudioShellLocale::En, StudioShellLocale::ZhCn] {
        for scale in [1.0, 2.0] {
            let (mut app, _project, ids) = fixture();
            app.locale = locale;
            app.variable_browser.mode = BrowserMode::Side;
            let ctx = egui::Context::default();
            crate::studio_gui_shell::fonts::configure_studio_fonts(&ctx);
            ctx.style_mut(|style| {
                for font in style.text_styles.values_mut() {
                    font.size *= scale;
                }
            });
            ctx.enable_accesskit();
            let zh = locale == StudioShellLocale::ZhCn;
            activate(
                &mut app,
                &ctx,
                Role::CheckBox,
                if zh {
                    "选取 Feed A 的流量"
                } else {
                    "Select Feed A flow"
                },
            );
            activate(
                &mut app,
                &ctx,
                Role::CheckBox,
                if zh {
                    "选取 Feed B 的流量"
                } else {
                    "Select Feed B flow"
                },
            );
            activate(
                &mut app,
                &ctx,
                Role::Button,
                if zh {
                    "编辑所选流量"
                } else {
                    "Edit selected flows"
                },
            );
            draft(&mut app, &ids[0], "36 kmol/h");
            draft(&mut app, &ids[1], "1e-");
            activate(
                &mut app,
                &ctx,
                Role::Button,
                if zh {
                    "检查 2 项草稿"
                } else {
                    "Check 2 drafts"
                },
            );
            let output = frame(&mut app, &ctx, vec![]);
            for shape in output.shapes {
                crate::studio_gui_shell::tests::assert_text_fits(
                    &shape.shape,
                    shape.clip_rect,
                    &["36 kmol/h".into(), "1e-".into()],
                    "batch draft review",
                );
            }
            assert!(matches!(
                app.variable_browser.watch.batch.rejected,
                Some(FeedMolarFlowBatchError::InvalidDrafts(_))
            ));
        }
    }
}
