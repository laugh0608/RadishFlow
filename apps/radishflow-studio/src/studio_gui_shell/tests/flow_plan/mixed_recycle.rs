use super::*;
use radishflow_studio::{PropertyPackageCacheRoot, StudioProjectOpen, read_studio_project};
use rf_solver::{
    mixed::{MixedBlock, MixedBlockMethod, MixedRecycleOptions},
    partitioned::TearSpecification,
};
use rf_types::{DiagnosticPortTarget, units::MeasurementUnit};
use rf_ui::result_query::FlowResultView;

const M1: &[u8] = include_bytes!(
    "../../../../../../examples/flowsheets/fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json"
);

fn setup() -> (PropertyPackageCacheRoot, PathBuf, ReadyAppState) {
    let temp = PropertyPackageCacheRoot::temporary("m2-shell").unwrap();
    let path = temp.path().join("m1.rfproj.json");
    fs::write(&path, M1).unwrap();
    let mut config = synced_workspace_config();
    config.project_path = path.clone();
    let mut app = ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    app.screen = StudioShellScreen::Run;
    app.enable_background_runs();
    (temp, path, app)
}

fn make_recycle(dag: &FlowPlanConfiguration) -> FlowPlanConfiguration {
    let FlowPlanConfiguration::Mixed(mut options) = dag.clone() else {
        panic!("M1 required")
    };
    options
        .blocks
        .iter_mut()
        .find(|b| b.id.as_str() == "equations")
        .unwrap()
        .members = ["mixer-1".into(), "splitter-1".into()].into();
    for id in ["mixer-2", "splitter-2"] {
        options.blocks.push(MixedBlock {
            id: id.into(),
            members: [id.into()].into(),
            method: MixedBlockMethod::Local,
        });
    }
    options.recycle = Some(Box::new(
        rf_solver::mixed::MixedRecycleConfiguration::Single(MixedRecycleOptions {
            tear: TearSpecification {
                stream_id: "s-rba".into(),
                source: DiagnosticPortTarget::new("splitter-2", "outlet_a"),
                sink: DiagnosticPortTarget::new("mixer-1", "inlet_b"),
                initial_component_flows_mol_s: Some(
                    [("methane".into(), 10.0), ("ethane".into(), 1.0)].into(),
                ),
                component_scales_mol_s: None,
            },
            max_iteration_evaluations: 500,
        }),
    ));
    FlowPlanConfiguration::Mixed(options)
}

fn apply(app: &mut ReadyAppState, plan: FlowPlanConfiguration) {
    app.background_action(Action::ConfigureFlowPlan {
        expected: FlowPlanEditContext::capture(&app.platform_host.modeling_inputs().workspace),
        access: FlowEditAccess::Editable,
        command: FlowPlanCommand::Apply(plan),
    });
}

fn run_id(app: &ReadyAppState) -> rf_ui::FlowRunId {
    let FlowResultView::Current { origin, .. } =
        app.platform_host.modeling_inputs().workspace.flow_result()
    else {
        panic!("verified Current required")
    };
    origin.execution().unwrap().run_id.clone()
}

#[test]
fn mixed_recycle_native_shell_saves_reviews_and_reopens_before_explicit_rerun() {
    let (temp, source, mut app) = setup();
    let ctx = egui::Context::default();
    let dag = app
        .platform_host
        .modeling_inputs()
        .workspace
        .flow_plan
        .current()
        .clone();
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let _m1_run = run_id(&app);
    let mut config = make_recycle(&dag);
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let mut draft = FlowPlanDraft::new(workspace, config.clone());
    let field = FlowPlanNumericField::InitialComponent {
        stream: "s-rba".into(),
        component: "methane".into(),
    };
    draft
        .edit_number(field, 10.0, MeasurementUnit::MolePerSecond)
        .unwrap()
        .raw = "1e-".into();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft.clone()),
    });
    ctx.style_mut(|style| {
        for font in style.text_styles.values_mut() {
            font.size = 20.0;
        }
    });
    for (locale, tear_label) in [
        (StudioShellLocale::ZhCn, "撕裂流股"),
        (StudioShellLocale::En, "Tear stream"),
    ] {
        app.locale = locale;
        let text = frame(&mut app, &ctx, 400.0).join("\n");
        assert!(text.contains(tear_label) && text.contains("1e-"), "{text}");
        assert!(!app.background_command_enabled("run_panel.run_manual"));
    }
    draft
        .edit_number(
            FlowPlanNumericField::IterationBudget,
            500.0,
            MeasurementUnit::One,
        )
        .unwrap()
        .raw = "501".into();
    assert!(draft.configuration().is_err());
    draft
        .edit_number(
            FlowPlanNumericField::IterationBudget,
            500.0,
            MeasurementUnit::One,
        )
        .unwrap()
        .raw = "499".into();
    draft
        .edit_number(
            FlowPlanNumericField::InitialComponent {
                stream: "s-rba".into(),
                component: "methane".into(),
            },
            10.0,
            MeasurementUnit::MolePerSecond,
        )
        .unwrap()
        .raw = "9".into();
    config = draft.configuration().unwrap();
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
    click_text(&mut app, &ctx, "Apply plan", 0);
    let workspace = &app.platform_host.modeling_inputs().workspace;
    assert_eq!(workspace.flow_plan.current(), &config);
    assert!(matches!(
        workspace.flow_result(),
        FlowResultView::Stale { .. }
    ));
    app.dispatch_ui_command("run_panel.set_active");
    app.poll_background_runs(&ctx);
    assert!(!app.background_busy());

    app.save_project();
    assert!(app.pending_format_upgrade.is_some());
    assert_eq!(fs::read(&source).unwrap(), M1);
    app.cancel_pending_save_as_overwrite();
    assert_eq!(fs::read(&source).unwrap(), M1);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .project_save_state()
            .simulation_dirty
    );

    let copy = temp.path().join("m2.rfproj.json");
    fs::write(&copy, M1).unwrap();
    app.request_save_project_as(copy.clone(), true);
    assert!(app.pending_format_upgrade.is_some());
    let mut changed = M1.to_vec();
    changed.push(b'\n');
    fs::write(&copy, &changed).unwrap();
    assert!(!app.confirm_format_upgrade());
    assert_eq!(fs::read(&copy).unwrap(), changed);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .document_path
            .as_deref(),
        Some(source.as_path())
    );
    app.request_save_project_as(copy.clone(), true);
    assert!(app.confirm_format_upgrade());
    let frozen = fs::read(&copy).unwrap();
    assert_eq!(fs::read(&source).unwrap(), M1);
    assert!(matches!(
        read_studio_project(&copy).unwrap(),
        StudioProjectOpen::ReadOnly(_)
    ));

    click_text(&mut app, &ctx, "Run F5", 0);
    drain(&mut app, &ctx);
    let first_run = run_id(&app);
    let evidence = frame(&mut app, &ctx, 400.0).join("\n");
    for label in [
        "Single tear recycle",
        "Independent verification",
        "Component balance",
        "s-rba",
        "Property package",
    ] {
        assert!(evidence.contains(label), "{label}: {evidence}");
    }
    app.save_project();
    assert!(app.pending_format_upgrade.is_none());
    assert_eq!(fs::read(&copy).unwrap(), frozen);
    app.open_project(copy.clone(), "M2 test");
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .current(),
        &config
    );
    assert!(matches!(
        app.platform_host.modeling_inputs().workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    assert_ne!(run_id(&app), first_run);

    apply(&mut app, dag);
    app.save_project();
    assert_eq!(fs::read(&copy).unwrap(), frozen);
    let dag_copy = temp.path().join("m1-copy.rfproj.json");
    app.request_save_project_as(dag_copy.clone(), true);
    assert!(app.confirm_format_upgrade());
    assert!(matches!(
        read_studio_project(&dag_copy).unwrap(),
        StudioProjectOpen::Editable(_)
    ));
    assert_eq!(fs::read(&copy).unwrap(), frozen);
}

#[test]
fn mixed_recycle_shell_cold_open_preserves_read_only_protection_and_reports_failure() {
    let (temp, _, mut first) = setup();
    let mut plan = make_recycle(
        first
            .platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .current(),
    );
    let FlowPlanConfiguration::Mixed(options) = &mut plan else {
        unreachable!()
    };
    options
        .single_recycle_mut()
        .unwrap()
        .max_iteration_evaluations = 1;
    apply(&mut first, plan.clone());
    let path = temp.path().join("budget-one.rfproj.json");
    first.request_save_project_as(path.clone(), true);
    assert!(first.confirm_format_upgrade());

    let mut config = synced_workspace_config();
    config.project_path = path.clone();
    let mut app = ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    app.enable_background_runs();
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .current(),
        &plan
    );
    let ctx = egui::Context::default();
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    let text = frame(&mut app, &ctx, 400.0).join("\n");
    assert!(text.contains("solver.mixed.iteration_limit"), "{text}");
    assert!(text.contains("Last trial and closure residual"), "{text}");
    assert!(!text.contains("Component balance · Passed"));
    let original = fs::read_to_string(&path).unwrap();
    let mut future: serde_json::Value = serde_json::from_str(&original).unwrap();
    let capability = future["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|cap| cap["id"] == rf_store::project_v4::MIXED_RECYCLE_CAPABILITY)
        .unwrap();
    capability["version"] = 2.into();
    let future_path = temp.path().join("future.rfproj.json");
    fs::write(&future_path, future.to_string()).unwrap();
    app.open_project(future_path, "future test");
    assert!(app.read_only_project.is_some());
    assert!(!app.background_command_enabled("run_panel.run_manual"));
    assert_eq!(fs::read_to_string(path).unwrap(), original);
}
