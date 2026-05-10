use super::*;
use radishflow_studio::{
    StudioGuiWindowInspectorTargetDetailModel, StudioGuiWindowSolveSnapshotModel,
    StudioGuiWindowStreamResultModel, StudioGuiWindowUnitExecutionResultModel,
    test_support::{
        apply_stream_state_and_composition, build_binary_demo_provider,
        build_binary_hydrocarbon_lite_provider, build_synthetic_provider,
        solve_snapshot_model_from_project_with_provider_and_edit, stream_target_detail_model,
        unit_target_detail_model,
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

fn render_result_inspector_with_unit_texts(
    app: &mut ReadyAppState,
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    selected_stream_id: &str,
    selected_unit_id: &str,
) -> Vec<String> {
    let inspector =
        snapshot.result_inspector_with_unit(Some(selected_stream_id), None, Some(selected_unit_id));
    render_runtime_area_texts(app, |app, ui| app.render_result_inspector(ui, &inspector))
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

fn assert_rendered_stream_summary_surface(
    texts: &[String],
    surface: &str,
    stream: &StudioGuiWindowStreamResultModel,
    temperature_label: &str,
    pressure_label: &str,
    molar_flow_label: &str,
    molar_enthalpy_label: &str,
    related_solve_steps_label: &str,
    related_diagnostics_label: &str,
) {
    for (label, value) in [
        (temperature_label, stream.temperature_text.as_str()),
        (pressure_label, stream.pressure_text.as_str()),
        (molar_flow_label, stream.molar_flow_text.as_str()),
    ] {
        assert_eq!(
            rendered_text_occurrences(texts, label),
            1,
            "expected {surface} to render summary row label `{label}`, rendered texts: {:?}",
            texts
        );
        assert!(
            texts.iter().any(|text| text.contains(value)),
            "expected {surface} to render summary row value `{value}`, rendered texts: {:?}",
            texts
        );
    }

    assert_eq!(
        rendered_text_occurrences(texts, molar_enthalpy_label),
        0,
        "expected {surface} to avoid rendering enthalpy for non-flash intermediate stream `{}`, rendered texts: {:?}",
        stream.stream_id,
        texts
    );
    assert_eq!(
        rendered_text_occurrences(texts, related_solve_steps_label),
        1,
        "expected {surface} to render related solve steps section for `{}`, rendered texts: {:?}",
        stream.stream_id,
        texts
    );
    assert_eq!(
        rendered_text_occurrences(texts, related_diagnostics_label),
        1,
        "expected {surface} to render related diagnostics section for `{}`, rendered texts: {:?}",
        stream.stream_id,
        texts
    );
}

fn assert_rendered_unit_summary_surface(
    texts: &[String],
    surface: &str,
    unit: &StudioGuiWindowUnitExecutionResultModel,
    consumed_streams_label: &str,
    produced_streams_label: &str,
    related_solve_steps_label: &str,
    related_diagnostics_label: &str,
    diagnostic_targets_label: &str,
) {
    assert!(
        texts.iter().any(|text| text.contains(&unit.unit_id)),
        "expected {surface} to render unit id `{}`, rendered texts: {:?}",
        unit.unit_id,
        texts
    );
    assert!(
        texts
            .iter()
            .any(|text| text.contains(&format!("#{}", unit.step_index))),
        "expected {surface} to render unit step index `#{}`, rendered texts: {:?}",
        unit.step_index,
        texts
    );
    assert!(
        texts.iter().any(|text| text.contains(&unit.summary)),
        "expected {surface} to render unit summary `{}`, rendered texts: {:?}",
        unit.summary,
        texts
    );
    for label in [
        consumed_streams_label,
        produced_streams_label,
        related_solve_steps_label,
        related_diagnostics_label,
        diagnostic_targets_label,
    ] {
        assert!(
            texts.iter().any(|text| text.contains(label)),
            "expected {surface} to render section label `{label}`, rendered texts: {:?}",
            texts
        );
    }
    for stream in unit
        .consumed_stream_results
        .iter()
        .chain(unit.produced_stream_results.iter())
    {
        assert!(
            texts.iter().any(|text| text.contains(&stream.summary)),
            "expected {surface} to render unit-related stream summary `{}`, rendered texts: {:?}",
            stream.summary,
            texts
        );
    }
}

fn solve_two_phase_snapshot() -> StudioGuiWindowSolveSnapshotModel {
    solve_snapshot_model_from_project_with_provider_and_edit(
        include_str!("../../../../../examples/flowsheets/feed-heater-flash.rfproj.json"),
        &build_binary_demo_provider(),
        |_| {},
    )
}

fn solve_binary_hydrocarbon_snapshot(project_json: &str) -> StudioGuiWindowSolveSnapshotModel {
    solve_snapshot_model_from_project_with_provider_and_edit(
        project_json,
        &build_binary_hydrocarbon_lite_provider(),
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
fn runtime_panel_renders_bubble_dew_window_for_non_flash_intermediate_streams() {
    let mut app = ready_app_state(&synced_workspace_config());
    let bubble_dew_label = app.locale.text(ShellText::BubbleDewWindow).to_string();

    for (project_json, stream_id, title) in [
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-heated",
            "Heated Outlet",
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-cooled",
            "Cooled Outlet",
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-throttled",
            "Valve Outlet",
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-mix-out",
            "Mixer Outlet",
        ),
    ] {
        let snapshot = solve_binary_hydrocarbon_snapshot(project_json);
        let result = snapshot.result_inspector(Some(stream_id));
        assert!(
            result
                .selected_stream
                .as_ref()
                .is_some_and(|stream| stream.bubble_dew_window.is_some()),
            "expected result inspector model to carry bubble/dew window for non-flash intermediate stream `{stream_id}` before shell rendering"
        );

        let result_texts = render_result_inspector_texts(&mut app, &snapshot, stream_id);
        assert_eq!(
            rendered_text_occurrences(&result_texts, &bubble_dew_label),
            1,
            "expected result inspector to render the bubble/dew window for `{stream_id}`, rendered texts: {:?}",
            result_texts
        );

        let active_texts = render_active_inspector_texts(
            &mut app,
            stream_target_detail_model(&snapshot, stream_id, title),
        );
        assert_eq!(
            rendered_text_occurrences(&active_texts, &bubble_dew_label),
            1,
            "expected active inspector to render the bubble/dew window for `{stream_id}`, rendered texts: {:?}",
            active_texts
        );
    }
}

#[test]
fn runtime_panel_renders_summary_rows_and_context_for_non_flash_intermediate_streams() {
    let mut app = ready_app_state(&synced_workspace_config());
    let temperature_label = format!("T · {}", app.locale.runtime_label("Temperature"));
    let pressure_label = format!("P · {}", app.locale.runtime_label("Pressure"));
    let molar_flow_label = format!("F · {}", app.locale.runtime_label("Molar flow"));
    let molar_enthalpy_label = format!("H · {}", app.locale.runtime_label("Molar enthalpy"));
    let related_solve_steps_label = app.locale.text(ShellText::RelatedSolveSteps).to_string();
    let related_diagnostics_label = app.locale.text(ShellText::RelatedDiagnostics).to_string();

    for (project_json, stream_id, title) in [
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-heated",
            "Heated Outlet",
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-cooled",
            "Cooled Outlet",
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-throttled",
            "Valve Outlet",
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-mix-out",
            "Mixer Outlet",
        ),
    ] {
        let snapshot = solve_binary_hydrocarbon_snapshot(project_json);
        let result = snapshot.result_inspector(Some(stream_id));
        let result_stream = result
            .selected_stream
            .as_ref()
            .expect("expected selected non-flash intermediate stream");

        let result_texts = render_result_inspector_texts(&mut app, &snapshot, stream_id);
        assert_rendered_stream_summary_surface(
            &result_texts,
            "result inspector",
            result_stream,
            &temperature_label,
            &pressure_label,
            &molar_flow_label,
            &molar_enthalpy_label,
            &related_solve_steps_label,
            &related_diagnostics_label,
        );

        let active_detail = stream_target_detail_model(&snapshot, stream_id, title);
        let active_stream = active_detail
            .latest_stream_result
            .clone()
            .expect("expected active stream result");
        assert_eq!(&active_stream, result_stream);

        let active_texts = render_active_inspector_texts(&mut app, active_detail);
        assert_rendered_stream_summary_surface(
            &active_texts,
            "active inspector",
            &active_stream,
            &temperature_label,
            &pressure_label,
            &molar_flow_label,
            &molar_enthalpy_label,
            &related_solve_steps_label,
            &related_diagnostics_label,
        );
    }
}

#[test]
fn runtime_panel_renders_unit_summary_and_context_for_non_flash_intermediate_units() {
    let mut app = ready_app_state(&synced_workspace_config());
    let result_unit_view_label = app.locale.text(ShellText::ResultUnitView).to_string();
    let consumed_streams_label = app
        .locale
        .text(ShellText::InspectorConsumedStreams)
        .to_string();
    let produced_streams_label = app
        .locale
        .text(ShellText::InspectorProducedStreams)
        .to_string();
    let related_solve_steps_label = app.locale.text(ShellText::RelatedSolveSteps).to_string();
    let related_diagnostics_label = app.locale.text(ShellText::RelatedDiagnostics).to_string();
    let diagnostic_targets_label = app.locale.text(ShellText::DiagnosticTargets).to_string();

    for (project_json, selected_stream_id, unit_id, title) in [
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-heated",
            "heater-1",
            "Heater",
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-cooled",
            "cooler-1",
            "Cooler",
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-throttled",
            "valve-1",
            "Valve",
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            "stream-mix-out",
            "mixer-1",
            "Mixer",
        ),
    ] {
        let snapshot = solve_binary_hydrocarbon_snapshot(project_json);
        let inspector =
            snapshot.result_inspector_with_unit(Some(selected_stream_id), None, Some(unit_id));
        let selected_unit = inspector
            .selected_unit
            .as_ref()
            .expect("expected selected non-flash intermediate unit");

        let result_texts = render_result_inspector_with_unit_texts(
            &mut app,
            &snapshot,
            selected_stream_id,
            unit_id,
        );
        assert!(
            result_texts
                .iter()
                .any(|text| text.contains(&result_unit_view_label)),
            "expected result inspector to render unit view section for `{unit_id}`, rendered texts: {:?}",
            result_texts
        );

        let active_detail = unit_target_detail_model(&snapshot, unit_id, title);
        let active_unit = active_detail
            .latest_unit_result
            .clone()
            .expect("expected active unit result");
        assert_eq!(&active_unit, selected_unit);

        let active_texts = render_active_inspector_texts(&mut app, active_detail);
        assert_rendered_unit_summary_surface(
            &active_texts,
            "active inspector unit view",
            &active_unit,
            &consumed_streams_label,
            &produced_streams_label,
            &related_solve_steps_label,
            &related_diagnostics_label,
            &diagnostic_targets_label,
        );
    }
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
