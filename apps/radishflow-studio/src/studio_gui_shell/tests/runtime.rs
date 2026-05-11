use super::*;
use radishflow_studio::{
    StudioGuiWindowInspectorTargetDetailModel, StudioGuiWindowSolveSnapshotModel,
    StudioGuiWindowStreamResultModel, StudioGuiWindowUnitExecutionResultModel,
    test_support::{
        apply_stream_state_and_composition, build_binary_hydrocarbon_lite_provider,
        build_synthetic_provider, solve_snapshot_model_from_project_with_provider_and_edit,
        stream_target_detail_model, unit_target_detail_model,
    },
};
use rf_flash::estimate_bubble_dew_window;

const REFERENCE_TEMPERATURE_K: f64 = 300.0;
const REFERENCE_PRESSURE_PA: f64 = 100_000.0;
const BOUNDARY_DELTA_K: f64 = 0.001;

fn render_runtime_area_texts(
    app: &mut ReadyAppState,
    render: impl FnMut(&mut ReadyAppState, &mut egui::Ui),
) -> Vec<String> {
    render_runtime_area_texts_with_open_sections(app, &[], render)
}

fn open_collapsing_sections(ui: &mut egui::Ui, open_section_labels: &[&str]) {
    for label in open_section_labels {
        let id = ui.make_persistent_id(egui::Id::new(label));
        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
        state.set_open(true);
        state.store(ui.ctx());
    }
}

fn render_runtime_area_texts_with_open_sections(
    app: &mut ReadyAppState,
    open_section_labels: &[&str],
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
                open_collapsing_sections(ui, open_section_labels);
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

fn render_stream_result_inspector_texts(
    app: &mut ReadyAppState,
    scope_id: impl Into<String>,
    stream: &StudioGuiWindowStreamResultModel,
    open_section_labels: &[&str],
) -> Vec<String> {
    let scope_id = scope_id.into();
    render_runtime_area_texts(app, |app, ui| {
        ui.push_id(scope_id.clone(), |ui| {
            open_collapsing_sections(ui, open_section_labels);
            app.render_stream_result_inspector(ui, stream);
        });
    })
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

fn render_result_inspector_comparison_texts(
    app: &mut ReadyAppState,
    comparison: &radishflow_studio::StudioGuiWindowResultInspectorComparisonModel,
) -> Vec<String> {
    render_runtime_area_texts(app, |app, ui| {
        app.render_result_inspector_comparison(ui, comparison);
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

struct StreamSummaryLabels<'a> {
    temperature: &'a str,
    pressure: &'a str,
    molar_flow: &'a str,
    molar_enthalpy: &'a str,
    related_solve_steps: &'a str,
    related_diagnostics: &'a str,
}

fn assert_rendered_stream_summary_surface(
    texts: &[String],
    surface: &str,
    stream: &StudioGuiWindowStreamResultModel,
    labels: &StreamSummaryLabels<'_>,
) {
    for (label, value) in [
        (labels.temperature, stream.temperature_text.as_str()),
        (labels.pressure, stream.pressure_text.as_str()),
        (labels.molar_flow, stream.molar_flow_text.as_str()),
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

    if let Some(molar_enthalpy_text) = stream.molar_enthalpy_text.as_deref() {
        assert_eq!(
            rendered_text_occurrences(texts, labels.molar_enthalpy),
            1,
            "expected {surface} to render enthalpy for stream `{}`, rendered texts: {:?}",
            stream.stream_id,
            texts
        );
        assert!(
            texts.iter().any(|text| text.contains(molar_enthalpy_text)),
            "expected {surface} to render enthalpy value `{molar_enthalpy_text}` for stream `{}`, rendered texts: {:?}",
            stream.stream_id,
            texts
        );
    } else {
        assert_eq!(
            rendered_text_occurrences(texts, labels.molar_enthalpy),
            0,
            "expected {surface} to avoid rendering enthalpy for stream `{}`, rendered texts: {:?}",
            stream.stream_id,
            texts
        );
    }
    assert_eq!(
        rendered_text_occurrences(texts, labels.related_solve_steps),
        1,
        "expected {surface} to render related solve steps section for `{}`, rendered texts: {:?}",
        stream.stream_id,
        texts
    );
    assert_eq!(
        rendered_text_occurrences(texts, labels.related_diagnostics),
        1,
        "expected {surface} to render related diagnostics section for `{}`, rendered texts: {:?}",
        stream.stream_id,
        texts
    );
}

struct UnitSummaryLabels<'a> {
    consumed_streams: &'a str,
    produced_streams: &'a str,
    related_solve_steps: &'a str,
    related_diagnostics: &'a str,
    diagnostic_targets: &'a str,
}

fn assert_rendered_unit_summary_surface(
    texts: &[String],
    surface: &str,
    unit: &StudioGuiWindowUnitExecutionResultModel,
    labels: &UnitSummaryLabels<'_>,
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
        labels.consumed_streams,
        labels.produced_streams,
        labels.related_solve_steps,
        labels.related_diagnostics,
        labels.diagnostic_targets,
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

fn assert_rendered_flash_outlet_surface(
    texts: &[String],
    surface: &str,
    stream: &StudioGuiWindowStreamResultModel,
    molar_enthalpy_label: &str,
    phase_results_label: &str,
) {
    let molar_enthalpy_text = stream
        .molar_enthalpy_text
        .as_ref()
        .expect("expected flash outlet enthalpy text");
    assert_eq!(
        rendered_text_occurrences(texts, molar_enthalpy_label),
        1,
        "expected {surface} to render enthalpy summary row for `{}`, rendered texts: {:?}",
        stream.stream_id,
        texts
    );
    assert!(
        texts.iter().any(|text| text.contains(molar_enthalpy_text)),
        "expected {surface} to render enthalpy summary value `{molar_enthalpy_text}` for `{}`, rendered texts: {:?}",
        stream.stream_id,
        texts
    );
    assert!(
        texts.iter().any(|text| text.contains(phase_results_label)),
        "expected {surface} to render phase results section for `{}`, rendered texts: {:?}",
        stream.stream_id,
        texts
    );

    for row in &stream.phase_rows {
        assert!(
            texts.iter().any(|text| text.contains(&row.label)),
            "expected {surface} to render phase label `{}` for `{}`, rendered texts: {:?}",
            row.label,
            stream.stream_id,
            texts
        );
        assert!(
            texts
                .iter()
                .any(|text| text.contains(&row.phase_fraction_text)),
            "expected {surface} to render phase fraction `{}` for `{}`, rendered texts: {:?}",
            row.phase_fraction_text,
            stream.stream_id,
            texts
        );
        assert!(
            texts.iter().any(|text| text.contains(&row.molar_flow_text)),
            "expected {surface} to render phase molar flow `{}` for `{}`, rendered texts: {:?}",
            row.molar_flow_text,
            stream.stream_id,
            texts
        );
        if let Some(molar_enthalpy_text) = row.molar_enthalpy_text.as_ref() {
            assert!(
                texts.iter().any(|text| text.contains(molar_enthalpy_text)),
                "expected {surface} to render phase enthalpy `{molar_enthalpy_text}` for `{}`, rendered texts: {:?}",
                stream.stream_id,
                texts
            );
        }
    }
}

fn assert_rendered_comparison_surface(
    texts: &[String],
    surface: &str,
    comparison: &radishflow_studio::StudioGuiWindowResultInspectorComparisonModel,
) {
    assert!(
        texts
            .iter()
            .any(|text| text.contains(&comparison.base_stream_id)),
        "expected {surface} to render base stream id `{}`, rendered texts: {:?}",
        comparison.base_stream_id,
        texts
    );
    assert!(
        texts
            .iter()
            .any(|text| text.contains(&comparison.compared_stream_id)),
        "expected {surface} to render compared stream id `{}`, rendered texts: {:?}",
        comparison.compared_stream_id,
        texts
    );

    for row in &comparison.summary_rows {
        if row.label == "H" {
            for value in [&row.base_value, &row.compared_value, &row.delta_text] {
                assert!(
                    texts.iter().any(|text| text.contains(value)),
                    "expected {surface} to render comparison enthalpy summary value `{value}`, rendered texts: {:?}",
                    texts
                );
            }
        }
    }

    for row in &comparison.phase_rows {
        assert!(
            texts.iter().any(|text| text.contains(&row.phase_label)),
            "expected {surface} to render comparison phase label `{}`, rendered texts: {:?}",
            row.phase_label,
            texts
        );
        for value in [
            &row.base_fraction_text,
            &row.compared_fraction_text,
            &row.base_molar_flow_text,
            &row.compared_molar_flow_text,
            &row.base_molar_enthalpy_text,
            &row.compared_molar_enthalpy_text,
        ] {
            if value != "-" {
                assert!(
                    texts.iter().any(|text| text.contains(value)),
                    "expected {surface} to render comparison phase value `{value}`, rendered texts: {:?}",
                    texts
                );
            }
        }
        for delta in [
            &row.fraction_delta_text,
            &row.molar_flow_delta_text,
            &row.molar_enthalpy_delta_text,
        ] {
            if delta != "-" {
                assert!(
                    texts.iter().any(|text| text.contains(delta)),
                    "expected {surface} to render comparison delta `{delta}`, rendered texts: {:?}",
                    texts
                );
            }
        }
    }
}

fn solve_two_phase_snapshot() -> StudioGuiWindowSolveSnapshotModel {
    solve_binary_hydrocarbon_snapshot(include_str!(
        "../../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
    ))
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
        include_str!(
            "../../../../../examples/flowsheets/feed-mixer-flash-synthetic-demo.rfproj.json"
        ),
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
        include_str!(
            "../../../../../examples/flowsheets/feed-mixer-flash-synthetic-demo.rfproj.json"
        ),
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
fn runtime_panel_renders_bubble_dew_window_for_source_streams() {
    let mut app = ready_app_state(&synced_workspace_config());
    let bubble_dew_label = app.locale.text(ShellText::BubbleDewWindow).to_string();

    for (project_json, stream_ids) in [
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
            &["stream-feed"][..],
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
            ),
            &["stream-feed"][..],
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
            ),
            &["stream-feed"][..],
        ),
        (
            include_str!(
                "../../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
            ),
            &["stream-feed-a", "stream-feed-b"][..],
        ),
    ] {
        let snapshot = solve_binary_hydrocarbon_snapshot(project_json);

        for stream_id in stream_ids {
            let title = snapshot
                .streams
                .iter()
                .find(|stream| stream.stream_id == *stream_id)
                .expect("expected snapshot stream")
                .label
                .clone();
            let result = snapshot.result_inspector(Some(stream_id));
            assert!(
                result
                    .selected_stream
                    .as_ref()
                    .is_some_and(|stream| stream.bubble_dew_window.is_some()),
                "expected result inspector model to carry bubble/dew window for source stream `{stream_id}` before shell rendering"
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
                stream_target_detail_model(&snapshot, stream_id, title.as_str()),
            );
            assert_eq!(
                rendered_text_occurrences(&active_texts, &bubble_dew_label),
                1,
                "expected active inspector to render the bubble/dew window for `{stream_id}`, rendered texts: {:?}",
                active_texts
            );
        }
    }
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
    let labels = StreamSummaryLabels {
        temperature: &temperature_label,
        pressure: &pressure_label,
        molar_flow: &molar_flow_label,
        molar_enthalpy: &molar_enthalpy_label,
        related_solve_steps: &related_solve_steps_label,
        related_diagnostics: &related_diagnostics_label,
    };

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
        assert!(
            result_stream.molar_enthalpy_text.is_some(),
            "expected snapshot stream `{stream_id}` to materialize enthalpy before shell rendering"
        );

        let result_texts = render_result_inspector_texts(&mut app, &snapshot, stream_id);
        assert_rendered_stream_summary_surface(
            &result_texts,
            "result inspector",
            result_stream,
            &labels,
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
            &labels,
        );
    }
}

#[test]
fn runtime_panel_renders_official_two_phase_flash_enthalpy_and_comparison_rows() {
    let mut app = ready_app_state(&synced_workspace_config());
    let molar_enthalpy_label = format!("H · {}", app.locale.runtime_label("Molar enthalpy"));
    let phase_results_label = app.locale.text(ShellText::PhaseResults).to_string();
    let stream_comparison_label = app.locale.text(ShellText::StreamComparison).to_string();
    let expanded_phase_sections = [phase_results_label.as_str()];
    let snapshot = solve_binary_hydrocarbon_snapshot(include_str!(
        "../../../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
    ));

    let liquid_overview_texts = render_result_inspector_texts(&mut app, &snapshot, "stream-liquid");
    assert_eq!(
        rendered_text_occurrences(&liquid_overview_texts, &stream_comparison_label),
        1,
        "expected liquid result inspector to expose stream comparison section, rendered texts: {:?}",
        liquid_overview_texts
    );

    let liquid_result = snapshot.result_inspector(Some("stream-liquid"));
    let liquid_stream = liquid_result
        .selected_stream
        .as_ref()
        .expect("expected selected liquid outlet stream");
    let liquid_result_texts = render_stream_result_inspector_texts(
        &mut app,
        format!(
            "result-inspector:selected-stream:{}:{}",
            snapshot.snapshot_id, liquid_stream.stream_id
        ),
        liquid_stream,
        &expanded_phase_sections,
    );
    assert_rendered_flash_outlet_surface(
        &liquid_result_texts,
        "result inspector liquid outlet",
        liquid_stream,
        &molar_enthalpy_label,
        &phase_results_label,
    );

    let liquid_active_detail =
        stream_target_detail_model(&snapshot, "stream-liquid", "Liquid Outlet");
    let liquid_active_scope_id = format!(
        "active-inspector-latest-stream:{}",
        liquid_active_detail.target.command_id
    );
    let liquid_active_stream = liquid_active_detail
        .latest_stream_result
        .clone()
        .expect("expected active liquid outlet stream result");
    let liquid_active_texts = render_stream_result_inspector_texts(
        &mut app,
        liquid_active_scope_id,
        &liquid_active_stream,
        &expanded_phase_sections,
    );
    assert_rendered_flash_outlet_surface(
        &liquid_active_texts,
        "active inspector liquid outlet",
        &liquid_active_stream,
        &molar_enthalpy_label,
        &phase_results_label,
    );

    let vapor_result = snapshot.result_inspector(Some("stream-vapor"));
    let vapor_stream = vapor_result
        .selected_stream
        .as_ref()
        .expect("expected selected vapor outlet stream");
    let vapor_result_texts = render_stream_result_inspector_texts(
        &mut app,
        format!(
            "result-inspector:selected-stream:{}:{}",
            snapshot.snapshot_id, vapor_stream.stream_id
        ),
        vapor_stream,
        &expanded_phase_sections,
    );
    assert_rendered_flash_outlet_surface(
        &vapor_result_texts,
        "result inspector vapor outlet",
        vapor_stream,
        &molar_enthalpy_label,
        &phase_results_label,
    );

    let vapor_active_detail = stream_target_detail_model(&snapshot, "stream-vapor", "Vapor Outlet");
    let vapor_active_scope_id = format!(
        "active-inspector-latest-stream:{}",
        vapor_active_detail.target.command_id
    );
    let vapor_active_stream = vapor_active_detail
        .latest_stream_result
        .clone()
        .expect("expected active vapor outlet stream result");
    let vapor_active_texts = render_stream_result_inspector_texts(
        &mut app,
        vapor_active_scope_id,
        &vapor_active_stream,
        &expanded_phase_sections,
    );
    assert_rendered_flash_outlet_surface(
        &vapor_active_texts,
        "active inspector vapor outlet",
        &vapor_active_stream,
        &molar_enthalpy_label,
        &phase_results_label,
    );

    let comparison_inspector =
        snapshot.result_inspector_with_comparison(Some("stream-liquid"), Some("stream-vapor"));
    let comparison = comparison_inspector
        .comparison
        .as_ref()
        .expect("expected phase outlet comparison model");
    let comparison_texts = render_result_inspector_comparison_texts(&mut app, comparison);
    assert_rendered_comparison_surface(
        &comparison_texts,
        "result inspector comparison",
        comparison,
    );
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
    let labels = UnitSummaryLabels {
        consumed_streams: &consumed_streams_label,
        produced_streams: &produced_streams_label,
        related_solve_steps: &related_solve_steps_label,
        related_diagnostics: &related_diagnostics_label,
        diagnostic_targets: &diagnostic_targets_label,
    };

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
            &labels,
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
