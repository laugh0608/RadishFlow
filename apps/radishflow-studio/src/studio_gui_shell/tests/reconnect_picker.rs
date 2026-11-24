use super::*;
use egui::accesskit::{Action, ActionRequest, NodeId, TreeUpdate};

pub(in crate::studio_gui_shell) fn node(tree: &TreeUpdate, label: &str) -> NodeId {
    tree.nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing {label}"))
        .0
}

pub(in crate::studio_gui_shell) fn click(id: NodeId) -> egui::Event {
    egui::Event::AccessKitActionRequest(ActionRequest {
        target: id,
        action: Action::Click,
        data: None,
    })
}

pub(in crate::studio_gui_shell) fn paint_frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    event: Option<egui::Event>,
) -> egui::FullOutput {
    let window = app.platform_host.snapshot().window_model();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1440.0, 1200.0),
            )),
            focused: true,
            events: event.into_iter().collect(),
            ..Default::default()
        },
        |ctx| {
            app.render_right_sidebar(ctx, &window, &mut false);
            egui::CentralPanel::default().show(ctx, |ui| {
                app.render_canvas_area(ui, &window, StudioGuiWindowAreaId::Canvas)
            });
        },
    );
    let tree = output.platform_output.accesskit_update.as_ref().unwrap();
    assert!(
        tree.nodes.iter().any(|(id, _)| *id == tree.focus),
        "native focus must exist in this frame"
    );
    output
}

pub(in crate::studio_gui_shell) fn frame(
    app: &mut ReadyAppState,
    ctx: &egui::Context,
    event: Option<egui::Event>,
) -> TreeUpdate {
    paint_frame(app, ctx, event)
        .platform_output
        .accesskit_update
        .unwrap()
}

pub(in crate::studio_gui_shell) fn mixer_with_two_open_ports(side: &str) -> ReadyAppState {
    let mut config = synced_workspace_config();
    config.project_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json");
    let mut app = ready_app_state(&config);
    app.locale = StudioShellLocale::En;
    for id in ["stream-feed-a", "stream-feed-b"] {
        app.dispatch_ui_command(format!("inspector.focus_stream:{id}"));
        app.dispatch_ui_command(format!("canvas.disconnect_selected_stream_{side}"));
    }
    app.dispatch_ui_command("inspector.focus_stream:stream-feed-a");
    app
}
