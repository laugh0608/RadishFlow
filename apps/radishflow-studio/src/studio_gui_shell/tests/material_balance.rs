use super::runtime::collect_shape_texts;
use super::*;
use radishflow_studio::{
    StudioBackgroundRunAction as Action, flow_actions::FlowEditContext,
    run_actions::StudioRunAccess,
};
use rf_solver::component_balance::MaterialBalancePlan;
use rf_ui::material_balance::MaterialBalanceResultView;
use std::time::{Duration, Instant};
fn app() -> ReadyAppState {
    let mut config = synced_workspace_config();
    config.project_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets/feed-mixer-material-balance.rfproj.json");
    let mut app = ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    app.screen = StudioShellScreen::Run;
    app.enable_background_runs();
    app
}
fn frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    width: f32,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, 2600.0),
            )),
            events,
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| app.render_material_balance(ui));
        },
    )
}
fn texts(output: &egui::FullOutput) -> Vec<String> {
    let mut values = vec![];
    for shape in &output.shapes {
        collect_shape_texts(&shape.shape, &mut values);
    }
    values
}
fn find(shape: &egui::Shape, label: &str) -> Option<egui::Pos2> {
    match shape {
        egui::Shape::Text(t) if t.galley.text() == label => Some(t.pos + t.galley.size() / 2.0),
        egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| find(s, label)),
        _ => None,
    }
}
fn click(app: &mut ReadyAppState, ctx: &egui::Context, label: &str) {
    let output = frame(app, ctx, 900.0, vec![]);
    let pos = output
        .shapes
        .iter()
        .find_map(|s| find(&s.shape, label))
        .unwrap_or_else(|| panic!("missing button {label}: {:?}", texts(&output)));
    frame(
        app,
        ctx,
        900.0,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        ctx,
        900.0,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}
fn drain(app: &mut ReadyAppState, ctx: &egui::Context) {
    let until = Instant::now() + Duration::from_secs(10);
    while app.background_busy() {
        app.poll_background_runs(ctx);
        assert!(Instant::now() < until);
        std::thread::yield_now();
    }
}
#[test]
fn gui_buttons_apply_settings_submit_real_worker_and_render_qualified_material_result() {
    let mut app = app();
    let ctx = egui::Context::default();
    let before = app.platform_host.document().clone();
    click(&mut app, &ctx, "Apply settings");
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .material_balance_plan()
            .is_some()
    );
    click(&mut app, &ctx, "Calculate material balance");
    drain(&mut app, &ctx);
    let out = frame(&mut app, &ctx, 900.0, vec![]);
    let labels = texts(&out).join("\n");
    assert!(labels.contains("Current material result"), "{labels}");
    assert!(labels.contains("0.44"), "{labels}");
    assert_eq!(app.platform_host.document(), &before);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .snapshot_history
            .is_empty()
    );
}
#[test]
fn unfinished_numeric_draft_survives_narrow_bilingual_render_without_showing_old_values() {
    let mut app = app();
    let ctx = egui::Context::default();
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let plan =
        MaterialBalancePlan::for_mixer(&workspace.document.flowsheet, "mixer-1".into()).unwrap();
    let expected = FlowEditContext::capture(workspace);
    app.background_action(Action::ConfigureMaterial {
        expected: expected.clone(),
        access: StudioRunAccess::Allowed,
        plan: Box::new(plan),
    });
    app.background_action(Action::RunMaterial {
        expected,
        access: StudioRunAccess::Allowed,
    });
    drain(&mut app, &ctx);
    frame(&mut app, &ctx, 400.0, vec![]);
    app.material.draft.as_mut().unwrap().text[2] = "1e-".into();
    for locale in [StudioShellLocale::En, StudioShellLocale::ZhCn] {
        app.locale = locale;
        let out = frame(&mut app, &ctx, 400.0, vec![]);
        let labels = texts(&out).join("\n");
        assert!(labels.contains("1e-"));
        assert!(!labels.contains("0.44"));
        assert_eq!(app.material.draft.as_ref().unwrap().text[2], "1e-");
    }
    assert!(matches!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .material_balance_result(),
        MaterialBalanceResultView::Current { .. }
    ));
}
