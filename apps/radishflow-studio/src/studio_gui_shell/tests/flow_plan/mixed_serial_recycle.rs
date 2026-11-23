use super::*;
use radishflow_studio::PropertyPackageCacheRoot;
use rf_types::units::MeasurementUnit;
use rf_ui::result_query::FlowResultView;

const SAVED: &[u8] = include_bytes!(
    "../../../../../../examples/flowsheets/fixed-tp-mixed-serial-recycle.rfproj.json"
);
const EO: &[u8] = include_bytes!(
    "../../../../../../examples/flowsheets/fixed-tp-serial-recycle-eo-reference.rfproj.json"
);

fn shell(path: &std::path::Path) -> ReadyAppState {
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.into(),
        ..synced_workspace_config()
    });
    app.locale = StudioShellLocale::En;
    app.screen = StudioShellScreen::Run;
    app.enable_background_runs();
    app
}

fn edit(app: &mut ReadyAppState, stream: &str, raw: &str) {
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let mut draft = FlowPlanDraft::new(workspace, workspace.flow_plan.current().clone());
    let FlowPlanConfiguration::Mixed(options) = draft.candidate() else {
        panic!("expected M3")
    };
    let region = options
        .serial_recycle()
        .unwrap()
        .iter()
        .find(|region| region.tear.stream_id.as_str() == stream)
        .unwrap();
    let budget = region.max_iteration_evaluations;
    draft
        .edit_number(
            FlowPlanNumericField::RegionIterationBudget(stream.into()),
            budget as f64,
            MeasurementUnit::One,
        )
        .unwrap()
        .raw = raw.into();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
}

fn results(app: &mut ReadyAppState, ctx: &egui::Context) -> String {
    let out = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900., 8500.),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.render_flow_plan(ui));
        },
    );
    let mut texts = vec![];
    for shape in out.shapes {
        collect_shape_texts(&shape.shape, &mut texts);
    }
    texts.join("\n")
}

#[test]
fn serial_recycle_shell_edits_regions_saves_reopens_and_reviews_actual_handoff() {
    let temp = PropertyPackageCacheRoot::temporary("m3-shell-review").unwrap();
    let path = temp.path().join("serial.rfproj.json");
    fs::write(&path, SAVED).unwrap();
    let mut app = shell(&path);
    let ctx = egui::Context::default();
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
    edit(&mut app, "s-rb", "1e-");
    let text = frame(&mut app, &ctx, 900.).join("\n");
    assert!(text.contains("Two serial recycle regions"));
    assert!(!app.background_command_enabled("run_panel.run_manual"));
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let mut draft = workspace.flow_plan.draft().unwrap().clone();
    assert!(draft.configuration().is_err());
    for (id, before, after) in [("s-ra", 100., 101), ("s-rb", 240., 241)] {
        draft
            .edit_number(
                FlowPlanNumericField::RegionIterationBudget(id.into()),
                before,
                MeasurementUnit::One,
            )
            .unwrap()
            .raw = after.to_string();
    }
    draft
        .edit_number(
            FlowPlanNumericField::InitialComponent {
                stream: "s-rb".into(),
                component: "methane".into(),
            },
            2.,
            MeasurementUnit::MolePerSecond,
        )
        .unwrap()
        .raw = "2.5".into();
    let expected = draft.configuration().unwrap();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
    click_text(&mut app, &ctx, "Apply plan", 0);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .current(),
        &expected
    );
    let copy = temp.path().join("copy.rfproj.json");
    fs::write(&copy, EO).unwrap();
    app.request_save_project_as(copy.clone(), true);
    assert!(app.pending_format_upgrade.is_some());
    let mut changed = EO.to_vec();
    changed.push(b'\n');
    fs::write(&copy, &changed).unwrap();
    assert!(!app.confirm_format_upgrade());
    assert_eq!(fs::read(&copy).unwrap(), changed);
    app.request_save_project_as(copy.clone(), true);
    assert!(app.confirm_format_upgrade());
    app.open_project(copy, "M3 roundtrip");
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .current(),
        &expected
    );
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let text = results(&mut app, &ctx);
    assert!(
        text.contains("Current") && text.contains("Accepted streams (13)"),
        "{text}"
    );
    assert!(
        text.find("Recycle region · equations-a").unwrap()
            < text.find("Recycle region · equations-b").unwrap()
    );
    assert!(text.contains("Evaluated inlet · s-bf") && text.contains("Global total flow residual"));
    let FlowResultView::Current {
        snapshot: rf_ui::FlowResultRecord::Mixed(s),
        ..
    } = app.platform_host.modeling_inputs().workspace.flow_result()
    else {
        panic!("expected serial Current")
    };
    for (id, expected) in [("s-ab", 2.), ("s-bf", 7.)] {
        let boundary = s
            .result
            .boundaries
            .iter()
            .find(|b| b.connection.stream_id.as_str() == id)
            .unwrap();
        for state in [&boundary.producer, &boundary.consumed] {
            assert!((state.total_molar_flow_mol_s - expected).abs() < 1e-9);
        }
    }
    assert_eq!(fs::read(path).unwrap(), SAVED);
}

#[test]
fn serial_recycle_shell_failure_only_displays_regions_that_ran() {
    let temp = PropertyPackageCacheRoot::temporary("m3-shell-fail").unwrap();
    let path = temp.path().join("serial.rfproj.json");
    fs::write(&path, SAVED).unwrap();
    for (id, downstream) in [("s-ra", false), ("s-rb", true)] {
        let mut app = shell(&path);
        let ctx = egui::Context::default();
        edit(&mut app, id, "1");
        click_text(&mut app, &ctx, "Apply plan", 0);
        app.dispatch_ui_command("run_panel.run_manual");
        drain(&mut app, &ctx);
        let text = results(&mut app, &ctx);
        assert!(
            text.contains("Failed") && text.contains("Recycle region · equations-a"),
            "{text}"
        );
        assert_eq!(text.contains("Recycle region · equations-b"), downstream);
        assert!(!text.contains("Accepted streams"));
    }
}

#[test]
fn typing_in_downstream_budget_preserves_its_identity_and_source_budget() {
    let temp = PropertyPackageCacheRoot::temporary("m3-shell-text").unwrap();
    let path = temp.path().join("saved.rfproj.json");
    fs::write(&path, SAVED).unwrap();
    let mut app = shell(&path);
    let ctx = egui::Context::default();
    let out = output(&mut app, &ctx, vec![]);
    let mut budgets = vec![];
    for shape in out.shapes {
        text_position(&shape.shape, "Candidate evaluation limit", &mut budgets);
    }
    assert_eq!(budgets.len(), 2);
    click_at(&mut app, &ctx, budgets[1] + egui::vec2(0., 19.));
    output(
        &mut app,
        &ctx,
        vec![
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            },
            egui::Event::Text("37".into()),
        ],
    );
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let draft = workspace.flow_plan.draft().unwrap();
    let field = FlowPlanNumericField::RegionIterationBudget("s-rb".into());
    assert_eq!(draft.numbers()[&field].raw, "37");
    let FlowPlanConfiguration::Mixed(options) = draft.configuration().unwrap() else {
        panic!("M3 plan")
    };
    let regions = options.serial_recycle().unwrap();
    assert_eq!(
        [
            regions[0].max_iteration_evaluations,
            regions[1].max_iteration_evaluations
        ],
        [100, 37]
    );
}

#[test]
fn studio_first_serial_save_confirms_new_capability_and_protects_future_versions() {
    use radishflow_studio::{StudioProjectOpen, read_studio_project};
    use rf_store::project_v4::MIXED_SERIAL_RECYCLE_CAPABILITY;

    let temp = PropertyPackageCacheRoot::temporary("m3-shell-save").unwrap();
    let reference = temp.path().join("eo.rfproj.json");
    let example = temp.path().join("serial.rfproj.json");
    fs::write(&reference, EO).unwrap();
    fs::write(&example, SAVED).unwrap();
    let mut app = shell(&reference);
    let StudioProjectOpen::Editable(loaded) = read_studio_gui_project(&example).unwrap() else {
        panic!("editable M3 sample")
    };
    let plan = loaded.workspace.flow_plan.current().clone();
    app.background_action(Action::ConfigureFlowPlan {
        access: FlowEditAccess::Editable,
        expected: FlowPlanEditContext::capture(&app.platform_host.modeling_inputs().workspace),
        command: FlowPlanCommand::Apply(plan),
    });
    app.save_project();
    assert!(
        matches!(&app.pending_format_upgrade, Some(project_save::PendingFormatUpgrade::FlowPlan { prepared,.. }) if prepared.compatibility().is_some())
    );
    app.cancel_pending_save_as_overwrite();
    assert_eq!(fs::read(&reference).unwrap(), EO);
    app.save_project();
    assert!(app.confirm_format_upgrade());
    assert!(matches!(
        read_studio_project(&reference).unwrap(),
        StudioProjectOpen::ReadOnly(_)
    ));

    let mut future: serde_json::Value =
        serde_json::from_slice(&fs::read(&reference).unwrap()).unwrap();
    let capability = future["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|cap| cap["id"] == MIXED_SERIAL_RECYCLE_CAPABILITY)
        .unwrap();
    capability["version"] = 2.into();
    let bytes = serde_json::to_vec(&future).unwrap();
    fs::write(&example, &bytes).unwrap();
    app.open_project(example.clone(), "future M3");
    assert!(app.read_only_project.is_some());
    assert!(!app.background_command_enabled("run_panel.run_manual"));
    let copy = temp.path().join("future-copy.rfproj.json");
    app.request_save_project_as(copy.clone(), true);
    assert!(!copy.exists());
    assert_eq!(fs::read(example).unwrap(), bytes);
}
