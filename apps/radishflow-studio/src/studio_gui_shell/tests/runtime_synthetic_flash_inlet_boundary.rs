use super::*;
use radishflow_studio::{
    StudioGuiWindowInspectorTargetDetailModel, StudioGuiWindowSolveSnapshotModel,
    StudioGuiWindowStreamResultModel,
    test_support::{
        apply_stream_state_and_composition, build_synthetic_provider,
        solve_snapshot_model_from_project_with_provider_and_edit, stream_target_detail_model,
    },
};
use rf_flash::estimate_bubble_dew_window;
use rf_types::PhaseEquilibriumRegion;

const REFERENCE_TEMPERATURE_K: f64 = 300.0;
const REFERENCE_PRESSURE_PA: f64 = 100_000.0;
const BOUNDARY_DELTA_K: f64 = 0.001;
const OVERALL_MOLE_FRACTIONS: [f64; 2] = [0.25, 0.75];

#[derive(Clone, Copy)]
struct SyntheticCase {
    label: &'static str,
    k_values: [f64; 2],
    temperature_k: f64,
    phase_region: PhaseEquilibriumRegion,
    flowing_outlet_stream_id: &'static str,
    flowing_outlet_title: &'static str,
    zero_outlet_stream_id: &'static str,
    zero_outlet_title: &'static str,
}

#[derive(Clone, Copy)]
struct SyntheticChainScenario {
    label: &'static str,
    project_json: &'static str,
    source_stream_ids: &'static [&'static str],
    flash_inlet_stream_id: &'static str,
    flash_inlet_title: &'static str,
    seeded_stream_ids: &'static [&'static str],
}

fn synthetic_cases() -> Vec<SyntheticCase> {
    let liquid_only_k_values = [0.8, 0.6];
    let liquid_only_provider =
        build_synthetic_provider(liquid_only_k_values, REFERENCE_PRESSURE_PA);
    let liquid_only_window = estimate_bubble_dew_window(
        &liquid_only_provider,
        REFERENCE_TEMPERATURE_K,
        REFERENCE_PRESSURE_PA,
        OVERALL_MOLE_FRACTIONS.to_vec(),
    )
    .expect("expected liquid-only synthetic bubble/dew window");

    let vapor_only_k_values = [1.8, 1.3];
    let vapor_only_provider = build_synthetic_provider(vapor_only_k_values, REFERENCE_PRESSURE_PA);
    let vapor_only_window = estimate_bubble_dew_window(
        &vapor_only_provider,
        REFERENCE_TEMPERATURE_K,
        REFERENCE_PRESSURE_PA,
        OVERALL_MOLE_FRACTIONS.to_vec(),
    )
    .expect("expected vapor-only synthetic bubble/dew window");

    vec![
        SyntheticCase {
            label: "synthetic liquid-only bubble-temperature - 0.001 K",
            k_values: liquid_only_k_values,
            temperature_k: liquid_only_window.bubble_temperature_k - BOUNDARY_DELTA_K,
            phase_region: PhaseEquilibriumRegion::LiquidOnly,
            flowing_outlet_stream_id: "stream-liquid",
            flowing_outlet_title: "Liquid Outlet",
            zero_outlet_stream_id: "stream-vapor",
            zero_outlet_title: "Vapor Outlet",
        },
        SyntheticCase {
            label: "synthetic vapor-only dew-temperature + 0.001 K",
            k_values: vapor_only_k_values,
            temperature_k: vapor_only_window.dew_temperature_k + BOUNDARY_DELTA_K,
            phase_region: PhaseEquilibriumRegion::VaporOnly,
            flowing_outlet_stream_id: "stream-vapor",
            flowing_outlet_title: "Vapor Outlet",
            zero_outlet_stream_id: "stream-liquid",
            zero_outlet_title: "Liquid Outlet",
        },
    ]
}

fn synthetic_chain_scenarios() -> [SyntheticChainScenario; 4] {
    [
        SyntheticChainScenario {
            label: "heater",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-heater-flash-synthetic-demo.rfproj.json"
            ),
            source_stream_ids: &["stream-feed"],
            flash_inlet_stream_id: "stream-heated",
            flash_inlet_title: "Heated Outlet",
            seeded_stream_ids: &["stream-feed", "stream-heated"],
        },
        SyntheticChainScenario {
            label: "cooler",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-cooler-flash-synthetic-demo.rfproj.json"
            ),
            source_stream_ids: &["stream-feed"],
            flash_inlet_stream_id: "stream-cooled",
            flash_inlet_title: "Cooled Outlet",
            seeded_stream_ids: &["stream-feed", "stream-cooled"],
        },
        SyntheticChainScenario {
            label: "valve",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-valve-flash-synthetic-demo.rfproj.json"
            ),
            source_stream_ids: &["stream-feed"],
            flash_inlet_stream_id: "stream-throttled",
            flash_inlet_title: "Valve Outlet",
            seeded_stream_ids: &["stream-feed", "stream-throttled"],
        },
        SyntheticChainScenario {
            label: "mixer",
            project_json: include_str!(
                "../../../../../examples/flowsheets/feed-mixer-flash-synthetic-demo.rfproj.json"
            ),
            source_stream_ids: &["stream-feed-a", "stream-feed-b"],
            flash_inlet_stream_id: "stream-mix-out",
            flash_inlet_title: "Mixer Outlet",
            seeded_stream_ids: &["stream-feed-a", "stream-feed-b"],
        },
    ]
}

fn solve_synthetic_snapshot(
    scenario: &SyntheticChainScenario,
    case: &SyntheticCase,
) -> StudioGuiWindowSolveSnapshotModel {
    let provider = build_synthetic_provider(case.k_values, REFERENCE_PRESSURE_PA);
    solve_snapshot_model_from_project_with_provider_and_edit(
        scenario.project_json,
        &provider,
        |project| {
            for stream_id in scenario.seeded_stream_ids {
                apply_stream_state_and_composition(
                    project,
                    stream_id,
                    OVERALL_MOLE_FRACTIONS,
                    case.temperature_k,
                    REFERENCE_PRESSURE_PA,
                );
            }
        },
    )
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

fn open_collapsing_sections(ui: &mut egui::Ui, open_section_labels: &[&str]) {
    for label in open_section_labels {
        let id = ui.make_persistent_id(label);
        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
        state.set_open(true);
        state.store(ui.ctx());
    }
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

fn render_result_inspector_texts(
    app: &mut ReadyAppState,
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    selected_stream_id: &str,
    open_section_labels: &[&str],
) -> Vec<String> {
    let inspector = snapshot.result_inspector(Some(selected_stream_id));
    render_runtime_area_texts_with_open_sections(app, open_section_labels, |app, ui| {
        app.render_result_inspector(ui, &inspector);
    })
}

fn render_active_inspector_texts(
    app: &mut ReadyAppState,
    active_detail: StudioGuiWindowInspectorTargetDetailModel,
    open_section_labels: &[&str],
) -> Vec<String> {
    let mut window = app.platform_host.snapshot().window_model();
    window.runtime.latest_solve_snapshot = None;
    window.runtime.latest_failure = None;
    window.runtime.active_inspector_target = Some(active_detail.target.clone());
    window.runtime.active_inspector_detail = Some(active_detail);

    render_runtime_area_texts_with_open_sections(app, open_section_labels, |app, ui| {
        app.render_runtime_area_contents(ui, &window, StudioGuiWindowAreaId::Runtime);
    })
}

fn rendered_text_occurrences(texts: &[String], expected: &str) -> usize {
    texts.iter().filter(|text| text.contains(expected)).count()
}

fn snapshot_stream<'a>(
    snapshot: &'a StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
) -> &'a StudioGuiWindowStreamResultModel {
    snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == stream_id)
        .expect("expected snapshot stream")
}

fn phase_region_id(region: PhaseEquilibriumRegion) -> &'static str {
    match region {
        PhaseEquilibriumRegion::LiquidOnly => "liquid_only",
        PhaseEquilibriumRegion::TwoPhase => "two_phase",
        PhaseEquilibriumRegion::VaporOnly => "vapor_only",
    }
}

fn assert_stream_window_rendered(
    app: &mut ReadyAppState,
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
    case: &SyntheticCase,
) {
    let stream = snapshot_stream(snapshot, stream_id);
    let window = stream
        .bubble_dew_window
        .as_ref()
        .expect("expected stream bubble/dew window");
    assert_eq!(
        window.phase_region,
        phase_region_id(case.phase_region),
        "{} for stream `{stream_id}`",
        case.label
    );
    let molar_enthalpy_text = stream
        .molar_enthalpy_text
        .as_ref()
        .expect("expected stream molar enthalpy text");

    let bubble_dew_label = app.locale.text(ShellText::BubbleDewWindow).to_string();
    let molar_enthalpy_label = app.locale.runtime_label("Molar enthalpy").into_owned();
    let open_sections = [bubble_dew_label.as_str()];

    let result_texts = render_result_inspector_texts(app, snapshot, stream_id, &open_sections);
    assert_rendered_stream_window_surface(
        &result_texts,
        "result inspector",
        stream_id,
        &bubble_dew_label,
        &molar_enthalpy_label,
        molar_enthalpy_text,
    );

    let active_texts = render_active_inspector_texts(
        app,
        stream_target_detail_model(snapshot, stream_id, title),
        &open_sections,
    );
    assert_rendered_stream_window_surface(
        &active_texts,
        "active inspector",
        stream_id,
        &bubble_dew_label,
        &molar_enthalpy_label,
        molar_enthalpy_text,
    );
}

fn assert_rendered_stream_window_surface(
    texts: &[String],
    surface: &str,
    stream_id: &str,
    bubble_dew_label: &str,
    molar_enthalpy_label: &str,
    molar_enthalpy_text: &str,
) {
    for expected in [bubble_dew_label, molar_enthalpy_label, molar_enthalpy_text] {
        assert_eq!(
            rendered_text_occurrences(texts, expected),
            1,
            "expected {surface} to render `{expected}` once for `{stream_id}`, rendered texts: {:?}",
            texts
        );
    }
}

fn assert_stream_window_absent(
    app: &mut ReadyAppState,
    snapshot: &StudioGuiWindowSolveSnapshotModel,
    stream_id: &str,
    title: &str,
) {
    assert!(
        snapshot_stream(snapshot, stream_id)
            .bubble_dew_window
            .is_none(),
        "expected stream `{stream_id}` to avoid carrying a pseudo bubble/dew window"
    );
    let bubble_dew_label = app.locale.text(ShellText::BubbleDewWindow).to_string();

    let result_texts = render_result_inspector_texts(app, snapshot, stream_id, &[]);
    assert_eq!(
        rendered_text_occurrences(&result_texts, &bubble_dew_label),
        0,
        "expected result inspector to avoid rendering bubble/dew window for `{stream_id}`, rendered texts: {:?}",
        result_texts
    );

    let active_texts = render_active_inspector_texts(
        app,
        stream_target_detail_model(snapshot, stream_id, title),
        &[],
    );
    assert_eq!(
        rendered_text_occurrences(&active_texts, &bubble_dew_label),
        0,
        "expected active inspector to avoid rendering bubble/dew window for `{stream_id}`, rendered texts: {:?}",
        active_texts
    );
}

#[test]
fn runtime_panel_renders_synthetic_single_phase_flash_chain_window_semantics() {
    let mut app = ready_app_state(&synced_workspace_config());

    for scenario in synthetic_chain_scenarios() {
        for case in synthetic_cases() {
            let snapshot = solve_synthetic_snapshot(&scenario, &case);

            for stream_id in scenario.source_stream_ids {
                let title = snapshot_stream(&snapshot, stream_id).label.clone();
                assert_stream_window_rendered(&mut app, &snapshot, stream_id, &title, &case);
            }
            assert_stream_window_rendered(
                &mut app,
                &snapshot,
                scenario.flash_inlet_stream_id,
                scenario.flash_inlet_title,
                &case,
            );
            assert_stream_window_rendered(
                &mut app,
                &snapshot,
                case.flowing_outlet_stream_id,
                case.flowing_outlet_title,
                &case,
            );
            assert_stream_window_absent(
                &mut app,
                &snapshot,
                case.zero_outlet_stream_id,
                case.zero_outlet_title,
            );
            assert!(
                snapshot.steps.iter().any(|step| step.unit_id == "flash-1"
                    && step
                        .consumed_stream_results
                        .iter()
                        .any(|stream| stream.stream_id == scenario.flash_inlet_stream_id)),
                "expected `{}` scenario to keep flash consuming `{}` for {}",
                scenario.label,
                scenario.flash_inlet_stream_id,
                case.label
            );
        }
    }
}
