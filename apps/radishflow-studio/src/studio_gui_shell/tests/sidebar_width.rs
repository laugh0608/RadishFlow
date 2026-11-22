use super::*;

#[test]
fn sidebar_widths_follow_layout_after_cached_frames_and_viewport_changes() {
    let mut app = ready_app_state(&synced_workspace_config());
    app.right_sidebar_tab = StudioShellRightSidebarTab::Variables;
    let ctx = egui::Context::default();
    let mut window = app.platform_host.snapshot().window_model();

    // Keep one context: constructing a fresh context for each frame hides the
    // stale SidePanel width that survives workspace/layout changes in the app.
    for (viewport_width, weights, expected_left, expected_right) in [
        (1280.0, [24, 52, 24], 280.0, 360.0),
        (1280.0, [24, 36, 40], 280.0, 400.0),
        (1200.0, [20, 40, 40], 240.0, 384.0),
        (1600.0, [24, 36, 40], 280.0, 420.0),
        (1280.0, [24, 52, 24], 280.0, 360.0),
    ] {
        for (region, weight) in window.layout_state.region_weights.iter_mut().zip(weights) {
            region.weight = weight;
        }
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(viewport_width, 889.0),
                )),
                ..Default::default()
            },
            |ctx| {
                let mut hovered = false;
                app.render_left_sidebar(ctx, &window, &mut hovered);
                app.render_right_sidebar(ctx, &window, &mut hovered);
            },
        );
        for (id, expected) in [
            ("studio.left_sidebar", expected_left),
            ("studio.right_sidebar", expected_right),
        ] {
            let rect = egui::containers::panel::PanelState::load(&ctx, egui::Id::new(id))
                .expect("rendered sidebar")
                .rect;
            assert_eq!(rect.width(), expected, "{id} at {viewport_width}");
        }
    }
}
