use super::*;
use rf_types::DiagnosticPortTarget;

fn serial_app() -> ReadyAppState {
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/flowsheets/fixed-tp-q7-serial-recycle.rfproj.json"),
        ..synced_workspace_config()
    });
    app.screen = StudioShellScreen::Run;
    app.locale = StudioShellLocale::En;
    app.enable_background_runs();
    app
}

fn plan_action(app: &mut ReadyAppState, command: FlowPlanCommand) {
    app.background_action(Action::ConfigureFlowPlan {
        expected: FlowPlanEditContext::capture(&app.platform_host.modeling_inputs().workspace),
        access: FlowEditAccess::Editable,
        command,
    });
}

fn edit_draft(app: &mut ReadyAppState, candidate: FlowPlanConfiguration) {
    let draft = FlowPlanDraft::new(&app.platform_host.modeling_inputs().workspace, candidate);
    app.background_action(Action::FlowPlanDraft {
        access: FlowEditAccess::Editable,
        draft: Some(draft),
    });
}

#[test]
fn applied_readiness_owns_the_revision_and_targets_across_opposite_drafts() {
    let mut app = serial_app();
    let ctx = egui::Context::default();
    ctx.style_mut(|style| {
        for font in style.text_styles.values_mut() {
            font.size = 20.0;
        }
    });
    let saved = app
        .platform_host
        .modeling_inputs()
        .workspace
        .flow_plan
        .current()
        .clone();
    let mut bad = saved.clone();
    let FlowPlanConfiguration::Partitioned(options) = &mut bad else {
        unreachable!()
    };
    let tear = &mut options.tears[1];
    tear.stream_id = "s-bleed".into();
    tear.source = DiagnosticPortTarget::new("splitter-1", "outlet_b");
    tear.sink = DiagnosticPortTarget::new("mixer-3", "inlet_a");
    options.tears.reverse();

    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    edit_draft(&mut app, bad);
    let text = frame(&mut app, &ctx, 400.0).join("\n");
    for label in [
        "Draft is not applied",
        "Applied plan",
        "Plan revision 0",
        "Topology and bindings ready",
        "Current · 10",
    ] {
        assert!(text.contains(label), "{label}");
    }
    assert!(!text.contains("Source port  splitter-1.outlet_b"));

    apply_draft_by_keyboard(&mut app, &ctx);
    edit_draft(&mut app, saved);
    app.locale = StudioShellLocale::ZhCn;
    let text = frame(&mut app, &ctx, 400.0).join("\n");
    for label in [
        "草稿未应用",
        "已应用计划",
        "计划修订 1",
        "拓扑与绑定未就绪",
        "撕裂流股位于循环块外。",
        "流股  s-bleed",
        "源端口  splitter-1.outlet_b",
        "汇端口  mixer-3.inlet_a",
        "Stale ·",
    ] {
        assert!(text.contains(label), "{label}");
    }
    assert!(!text.contains("拓扑与绑定已就绪"));
    let tree = output_at_width(&mut app, &ctx, 400.0, vec![])
        .platform_output
        .accesskit_update
        .unwrap();
    let targets = [
        "流股  s-bleed",
        "源端口  splitter-1.outlet_b",
        "汇端口  mixer-3.inlet_a",
    ]
    .map(|label| {
        tree.nodes
            .iter()
            .find(|(_, node)| node.value() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .expect("visible diagnostic target")
    });
    assert!(targets.iter().all(|r| r.x0 >= 0.0 && r.x1 <= 400.0));
    assert!(targets.windows(2).all(|rows| rows[0].y1 <= rows[1].y0));
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    assert!(
        frame(&mut app, &ctx, 400.0)
            .iter()
            .any(|s| s.starts_with("Failed ·"))
    );

    plan_action(&mut app, FlowPlanCommand::RestoreSaved);
    let text = frame(&mut app, &ctx, 400.0).join("\n");
    for label in ["计划修订 2", "拓扑与绑定已就绪", "Stale ·"] {
        assert!(text.contains(label), "{label}");
    }
    assert!(!text.contains("流股  s-bleed"));
    app.dispatch_ui_command("run_panel.run_manual");
    drain(&mut app, &ctx);
    assert!(
        frame(&mut app, &ctx, 400.0)
            .iter()
            .any(|s| s.starts_with("Current · 10"))
    );
}

fn apply_draft_by_keyboard(app: &mut ReadyAppState, ctx: &egui::Context) {
    use egui::accesskit::{Action, ActionRequest};
    ctx.enable_accesskit();
    let mut frame = |events| {
        output_at_width(app, ctx, 400.0, events)
            .platform_output
            .accesskit_update
            .unwrap()
    };
    let tree = frame(vec![]);
    let apply = tree
        .nodes
        .iter()
        .find(|(_, n)| n.label() == Some("Apply plan"))
        .unwrap()
        .0;
    let tree = frame(vec![egui::Event::AccessKitActionRequest(ActionRequest {
        target: apply,
        action: Action::Focus,
        data: None,
    })]);
    assert_eq!(tree.focus, apply);
    for (key, expected, disabled) in [
        (egui::Key::Enter, "Apply plan", true),
        (egui::Key::Enter, "Apply plan", true),
        (egui::Key::Tab, "Restore saved", false),
    ] {
        for pressed in [true, false] {
            frame(vec![egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }]);
        }
        let update = frame(vec![]);
        let (_, focused) = update
            .nodes
            .iter()
            .find(|(id, _)| *id == update.focus)
            .unwrap();
        assert_eq!(focused.label(), Some(expected));
        assert_eq!(focused.is_disabled(), disabled);
    }
}

#[test]
fn terminal_tear_keeps_the_original_reason_and_only_its_confirmed_port() {
    let mut app = serial_app();
    let mut plan = app
        .platform_host
        .modeling_inputs()
        .workspace
        .flow_plan
        .current()
        .clone();
    let FlowPlanConfiguration::Partitioned(options) = &mut plan else {
        unreachable!()
    };
    let tear = &mut options.tears[1];
    tear.stream_id = "s-product".into();
    tear.source = DiagnosticPortTarget::new("heater-1", "outlet");
    plan_action(&mut app, FlowPlanCommand::Apply(plan));
    let text = frame(&mut app, &egui::Context::default(), 400.0).join("\n");
    for label in [
        "Topology and bindings not ready",
        "terminal stream cannot be a tear",
        "Stream  s-product",
        "Source port  heater-1.outlet",
    ] {
        assert!(text.contains(label), "{label}");
    }
    assert!(!text.contains("Sink port  "));
}
