use super::*;
use radishflow_studio::{
    StudioGuiWindowInspectorTargetDetailModel, StudioGuiWindowSolveSnapshotModel,
    test_support::{
        apply_stream_state_and_composition, build_binary_demo_provider, build_synthetic_provider,
        solve_snapshot_model_from_project_with_provider_and_edit, stream_target_detail_model,
    },
};
use rf_flash::estimate_bubble_dew_window;

const REFERENCE_TEMPERATURE_K: f64 = 300.0;
const REFERENCE_PRESSURE_PA: f64 = 100_000.0;
const BOUNDARY_DELTA_K: f64 = 0.001;

fn render_runtime_area_texts(
    app: &mut ReadyAppState,
    mut render: impl FnMut(&mut ReadyAppState, &mut egui::Ui),
) -> Vec<String> {
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 6000.0),
            )),
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(app, ui);
            });
        },
    );

    let mut texts = Vec::new();
    for clipped_shape in &output.shapes {
        collect_shape_texts(&clipped_shape.shape, &mut texts);
    }
    texts
}

fn render_result_inspector_texts(
    app: &mut ReadyAppState,
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    selected_stream_id: &str,
) -> Vec<String> {
    let inspector = snapshot.result_inspector(Some(selected_stream_id));
    render_runtime_area_texts(app, |app, ui| app.render_result_inspector(ui, &inspector))
}

fn render_active_inspector_texts(
    app: &mut ReadyAppState,
    active_detail: StudioGuiWindowInspectorTargetDetailModel,
) -> Vec<String> {
    let mut window = app.platform_host.snapshot().window_model();
    window.runtime.latest_solve_snapshot = None;
    window.runtime.latest_failure = None;
    window.runtime.active_inspector_target = Some(active_detail.target.clone());
    window.runtime.active_inspector_detail = Some(active_detail);

    render_runtime_area_texts(app, |app, ui| {
        app.render_runtime_area_contents(ui, &window, StudioGuiWindowAreaId::Runtime);
    })
}

fn render_full_runtime_panel_texts(
    app: &mut ReadyAppState,
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    selected_stream_id: &str,
    active_detail: StudioGuiWindowInspectorTargetDetailModel,
) -> Vec<String> {
    app.result_inspector
        .select_stream(&snapshot.snapshot_id, selected_stream_id);

    let mut window = app.platform_host.snapshot().window_model();
    window.runtime.latest_solve_snapshot = Some(snapshot.clone());
    window.runtime.latest_failure = None;
    window.runtime.active_inspector_target = Some(active_detail.target.clone());
    window.runtime.active_inspector_detail = Some(active_detail);

    render_runtime_area_texts(app, |app, ui| {
        app.render_runtime_area_contents(ui, &window, StudioGuiWindowAreaId::Runtime);
    })
}

fn collect_shape_texts(shape: &egui::epaint::Shape, texts: &mut Vec<String>) {
    match shape {
        egui::epaint::Shape::Text(text) => texts.push(text.galley.job.text.clone()),
        egui::epaint::Shape::Vec(shapes) => {
            for shape in shapes {
                collect_shape_texts(shape, texts);
            }
        }
        _ => {}
    }
}

fn rendered_text_occurrences(texts: &[String], expected: &str) -> usize {
    texts.iter().filter(|text| text.contains(expected)).count()
}

fn solve_two_phase_snapshot() -> StudioGuiWindowSolveSnapshotModel {
    solve_snapshot_model_from_project_with_provider_and_edit(
        include_str!("../../../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        &build_binary_demo_provider(),
        |_| {},
    )
}

fn solve_liquid_only_snapshot() -> (
    StudioGuiWindowSolveSnapshotModel,
    StudioGuiWindowInspectorTargetDetailModel,
) {
    let overall_mole_fractions = [0.25, 0.75];
    let provider = build_synthetic_provider([0.8, 0.6], REFERENCE_PRESSURE_PA);
    let boundary_window = estimate_bubble_dew_window(
        &provider,
        REFERENCE_TEMPERATURE_K,
        REFERENCE_PRESSURE_PA,
        overall_mole_fractions.to_vec(),
    )
    .expect("expected liquid-only boundary window");
    let snapshot = solve_snapshot_model_from_project_with_provider_and_edit(
        include_str!("../../../../../examples/flowsheets/feed-mixer-flash.rfproj.json"),
        &provider,
        |project| {
            for stream_id in ["stream-feed-a", "stream-feed-b"] {
                apply_stream_state_and_composition(
                    project,
                    stream_id,
                    overall_mole_fractions,
                    boundary_window.bubble_temperature_k - BOUNDARY_DELTA_K,
                    REFERENCE_PRESSURE_PA,
                );
            }
        },
    );
    let detail = stream_target_detail_model(&snapshot, "stream-vapor", "Vapor Outlet");
    (snapshot, detail)
}

fn solve_vapor_only_snapshot() -> (
    StudioGuiWindowSolveSnapshotModel,
    StudioGuiWindowInspectorTargetDetailModel,
) {
    let overall_mole_fractions = [0.25, 0.75];
    let provider = build_synthetic_provider([1.8, 1.3], REFERENCE_PRESSURE_PA);
    let boundary_window = estimate_bubble_dew_window(
        &provider,
        REFERENCE_TEMPERATURE_K,
        REFERENCE_PRESSURE_PA,
        overall_mole_fractions.to_vec(),
    )
    .expect("expected vapor-only boundary window");
    let snapshot = solve_snapshot_model_from_project_with_provider_and_edit(
        include_str!("../../../../../examples/flowsheets/feed-mixer-flash.rfproj.json"),
        &provider,
        |project| {
            for stream_id in ["stream-feed-a", "stream-feed-b"] {
                apply_stream_state_and_composition(
                    project,
                    stream_id,
                    overall_mole_fractions,
                    boundary_window.dew_temperature_k + BOUNDARY_DELTA_K,
                    REFERENCE_PRESSURE_PA,
                );
            }
        },
    );
    let detail = stream_target_detail_model(&snapshot, "stream-liquid", "Liquid Outlet");
    (snapshot, detail)
}

#[test]
fn runtime_panel_renders_bubble_dew_window_for_flowing_flash_outlets() {
    let mut app = ready_app_state(&synced_workspace_config());
    let bubble_dew_label = app.locale.text(ShellText::BubbleDewWindow).to_string();

    let two_phase_snapshot = solve_two_phase_snapshot();
    let two_phase_result = two_phase_snapshot.result_inspector(Some("stream-liquid"));
    assert!(
        two_phase_result
            .selected_stream
            .as_ref()
            .is_some_and(|stream| stream.bubble_dew_window.is_some()),
        "expected two-phase result inspector model to carry bubble/dew window before shell rendering"
    );
    let two_phase_result_texts =
        render_result_inspector_texts(&mut app, &two_phase_snapshot, "stream-liquid");
    assert_eq!(
        rendered_text_occurrences(&two_phase_result_texts, &bubble_dew_label),
        1,
        "expected result inspector to render the bubble/dew window for a two-phase outlet, rendered texts: {:?}",
        two_phase_result_texts
    );
    let two_phase_active_texts = render_active_inspector_texts(
        &mut app,
        stream_target_detail_model(&two_phase_snapshot, "stream-liquid", "Liquid Outlet"),
    );
    assert_eq!(
        rendered_text_occurrences(&two_phase_active_texts, &bubble_dew_label),
        1,
        "expected active inspector to render the bubble/dew window for a two-phase outlet, rendered texts: {:?}",
        two_phase_active_texts
    );

    let (liquid_only_snapshot, _) = solve_liquid_only_snapshot();
    let liquid_only_result_texts =
        render_result_inspector_texts(&mut app, &liquid_only_snapshot, "stream-liquid");
    assert_eq!(
        rendered_text_occurrences(&liquid_only_result_texts, &bubble_dew_label),
        1,
        "expected result inspector to keep the bubble/dew window for the flowing liquid-only outlet"
    );
    let liquid_only_active_texts = render_active_inspector_texts(
        &mut app,
        stream_target_detail_model(&liquid_only_snapshot, "stream-liquid", "Liquid Outlet"),
    );
    assert_eq!(
        rendered_text_occurrences(&liquid_only_active_texts, &bubble_dew_label),
        1,
        "expected active inspector to keep the bubble/dew window for the flowing liquid-only outlet"
    );

    let (vapor_only_snapshot, _) = solve_vapor_only_snapshot();
    let vapor_only_result_texts =
        render_result_inspector_texts(&mut app, &vapor_only_snapshot, "stream-vapor");
    assert_eq!(
        rendered_text_occurrences(&vapor_only_result_texts, &bubble_dew_label),
        1,
        "expected result inspector to keep the bubble/dew window for the flowing vapor-only outlet"
    );
    let vapor_only_active_texts = render_active_inspector_texts(
        &mut app,
        stream_target_detail_model(&vapor_only_snapshot, "stream-vapor", "Vapor Outlet"),
    );
    assert_eq!(
        rendered_text_occurrences(&vapor_only_active_texts, &bubble_dew_label),
        1,
        "expected active inspector to keep the bubble/dew window for the flowing vapor-only outlet"
    );
}

#[test]
fn runtime_panel_hides_bubble_dew_window_for_zero_flow_single_phase_flash_outlets() {
    let mut app = ready_app_state(&synced_workspace_config());
    let bubble_dew_label = app.locale.text(ShellText::BubbleDewWindow).to_string();

    let (liquid_only_snapshot, liquid_only_detail) = solve_liquid_only_snapshot();
    let liquid_only_result_texts =
        render_result_inspector_texts(&mut app, &liquid_only_snapshot, "stream-vapor");
    assert_eq!(
        rendered_text_occurrences(&liquid_only_result_texts, &bubble_dew_label),
        0,
        "expected result inspector to render no bubble/dew window section for the zero-flow vapor outlet"
    );
    let liquid_only_active_texts = render_active_inspector_texts(&mut app, liquid_only_detail);
    assert_eq!(
        rendered_text_occurrences(&liquid_only_active_texts, &bubble_dew_label),
        0,
        "expected active inspector to render no bubble/dew window section for the zero-flow vapor outlet"
    );

    let (vapor_only_snapshot, vapor_only_detail) = solve_vapor_only_snapshot();
    let vapor_only_result_texts =
        render_result_inspector_texts(&mut app, &vapor_only_snapshot, "stream-liquid");
    assert_eq!(
        rendered_text_occurrences(&vapor_only_result_texts, &bubble_dew_label),
        0,
        "expected result inspector to render no bubble/dew window section for the zero-flow liquid outlet"
    );
    let vapor_only_texts = render_active_inspector_texts(&mut app, vapor_only_detail);
    assert_eq!(
        rendered_text_occurrences(&vapor_only_texts, &bubble_dew_label),
        0,
        "expected active inspector to render no bubble/dew window section for the zero-flow liquid outlet"
    );
}

#[test]
fn runtime_panel_can_render_same_stream_in_result_and_active_inspectors() {
    let mut app = ready_app_state(&synced_workspace_config());
    let bubble_dew_label = app.locale.text(ShellText::BubbleDewWindow).to_string();
    let snapshot = solve_two_phase_snapshot();
    let texts = render_full_runtime_panel_texts(
        &mut app,
        &snapshot,
        "stream-liquid",
        stream_target_detail_model(&snapshot, "stream-liquid", "Liquid Outlet"),
    );

    assert_eq!(
        rendered_text_occurrences(&texts, &bubble_dew_label),
        2,
        "expected the runtime panel to render bubble/dew window sections in both result and active inspector surfaces, rendered texts: {:?}",
        texts
    );
    assert!(
        !texts
            .iter()
            .any(|text| text.contains("widget ID") || text.contains("Grid ID")),
        "expected no egui duplicate-id diagnostics after namespacing repeated stream inspector rendering, rendered texts: {:?}",
        texts
    );
}
