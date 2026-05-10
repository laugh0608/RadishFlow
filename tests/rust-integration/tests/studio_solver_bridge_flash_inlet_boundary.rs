use std::time::{Duration, SystemTime, UNIX_EPOCH};

use radishflow_studio::{StudioSolveRequest, solve_workspace_with_property_package};
use rf_rust_integration::{
    NearBoundaryCaseKind, NearBoundaryStreamWindowCase, assert_close,
    binary_hydrocarbon_lite_near_boundary_stream_window_cases,
    expected_overall_molar_enthalpy_for_case, near_boundary_component_ids_for_package,
    near_boundary_package_provider_for_case,
    synthetic_single_phase_near_boundary_stream_window_cases,
};
use rf_store::parse_project_file_json;
use rf_types::{ComponentId, PhaseEquilibriumRegion, StreamId, UnitId};
use rf_ui::{AppState, DocumentMetadata, FlowsheetDocument};

fn timestamp(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn app_state_from_project(
    project_json: &str,
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
) -> AppState {
    let project = parse_project_file_json(project_json).expect("expected project parse");
    AppState::new(FlowsheetDocument::new(
        project.document.flowsheet,
        DocumentMetadata::new(document_id, title, timestamp(created_at_seconds)),
    ))
}

fn find_snapshot_stream<'a>(
    snapshot: &'a rf_ui::SolveSnapshot,
    stream_id: &str,
) -> &'a rf_ui::StreamStateSnapshot {
    snapshot
        .streams
        .iter()
        .find(|stream| stream.stream_id == StreamId::new(stream_id))
        .expect("expected snapshot stream")
}

fn apply_case_composition(
    app_state: &mut AppState,
    stream_id: &str,
    package_id: &str,
    overall_mole_fractions: [f64; 2],
) {
    let [first_component_id, second_component_id] =
        near_boundary_component_ids_for_package(package_id);
    let stream = app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&stream_id.into())
        .expect("expected stream");
    stream.overall_mole_fractions.clear();
    stream.overall_mole_fractions.insert(
        ComponentId::new(first_component_id),
        overall_mole_fractions[0],
    );
    stream.overall_mole_fractions.insert(
        ComponentId::new(second_component_id),
        overall_mole_fractions[1],
    );
}

fn apply_case_feed_state(
    app_state: &mut AppState,
    stream_id: &str,
    case: &NearBoundaryStreamWindowCase,
) {
    let stream = app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&stream_id.into())
        .expect("expected stream");
    stream.temperature_k = case.temperature_k;
    stream.pressure_pa = case.pressure_pa;
}

fn assert_near_boundary_window_matches_case(
    stream: &rf_ui::StreamStateSnapshot,
    case: &NearBoundaryStreamWindowCase,
) {
    let window = stream
        .bubble_dew_window
        .as_ref()
        .expect("expected bubble/dew window");
    let [first_component_id, second_component_id] =
        near_boundary_component_ids_for_package(case.package_id);
    let overall_phase = stream
        .phases
        .iter()
        .find(|phase| phase.label == "overall")
        .expect("expected overall phase");

    assert_close(stream.temperature_k, case.temperature_k, 1e-12);
    assert_close(stream.pressure_pa, case.pressure_pa, 1e-9);
    assert_close(
        *stream
            .overall_mole_fractions
            .iter()
            .find(|(component_id, _)| component_id == first_component_id)
            .map(|(_, fraction)| fraction)
            .expect("expected first component"),
        case.overall_mole_fractions[0],
        1e-12,
    );
    assert_close(
        *stream
            .overall_mole_fractions
            .iter()
            .find(|(component_id, _)| component_id == second_component_id)
            .map(|(_, fraction)| fraction)
            .expect("expected second component"),
        case.overall_mole_fractions[1],
        1e-12,
    );
    assert_close(
        overall_phase
            .molar_enthalpy_j_per_mol
            .expect("expected overall phase enthalpy"),
        expected_overall_molar_enthalpy_for_case(case),
        1e-9,
    );
    assert_eq!(
        window.phase_region, case.expected_phase_region,
        "{}",
        case.label
    );
    assert_close(
        window.bubble_pressure_pa,
        case.expected_bubble_pressure_pa,
        1e-6,
    );
    assert_close(window.dew_pressure_pa, case.expected_dew_pressure_pa, 1e-6);
    assert_close(
        window.bubble_temperature_k,
        case.expected_bubble_temperature_k,
        1e-4,
    );
    assert_close(
        window.dew_temperature_k,
        case.expected_dew_temperature_k,
        1e-4,
    );
}

fn assert_flash_consumed_stream_matches_inlet(
    snapshot: &rf_ui::SolveSnapshot,
    inlet_stream: &rf_ui::StreamStateSnapshot,
    case: &NearBoundaryStreamWindowCase,
) {
    let flash_step = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == UnitId::new("flash-1"))
        .expect("expected flash step");
    assert_eq!(flash_step.consumed_streams.len(), 1, "{}", case.label);
    assert_eq!(
        &flash_step.consumed_streams[0], inlet_stream,
        "{}",
        case.label
    );
}

fn assert_step_consumes_snapshot_stream(
    snapshot: &rf_ui::SolveSnapshot,
    unit_id: &str,
    stream: &rf_ui::StreamStateSnapshot,
    case_label: &str,
) {
    let step = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == UnitId::new(unit_id))
        .expect("expected consumer step");
    assert!(
        step.consumed_streams
            .iter()
            .any(|candidate| candidate == stream),
        "{}",
        case_label
    );
    assert!(
        stream
            .phases
            .iter()
            .find(|phase| phase.label == "overall")
            .and_then(|phase| phase.molar_enthalpy_j_per_mol)
            .is_some(),
        "{}",
        case_label
    );
    assert!(stream.bubble_dew_window.is_some(), "{}", case_label);
}

fn assert_flash_outlet_window_semantics_match_case(
    snapshot: &rf_ui::SolveSnapshot,
    case: &NearBoundaryStreamWindowCase,
) {
    let liquid = find_snapshot_stream(snapshot, "stream-liquid");
    let vapor = find_snapshot_stream(snapshot, "stream-vapor");

    match case.expected_phase_region {
        PhaseEquilibriumRegion::LiquidOnly => {
            assert!(liquid.total_molar_flow_mol_s > 0.0, "{}", case.label);
            assert_close(vapor.total_molar_flow_mol_s, 0.0, 1e-12);
            assert_eq!(
                liquid
                    .bubble_dew_window
                    .as_ref()
                    .expect("expected liquid outlet bubble/dew window")
                    .phase_region,
                PhaseEquilibriumRegion::LiquidOnly,
                "{}",
                case.label
            );
            assert!(
                vapor.bubble_dew_window.is_none(),
                "expected vapor outlet bubble/dew window to be absent for {}",
                case.label
            );
        }
        PhaseEquilibriumRegion::TwoPhase => {
            assert!(liquid.total_molar_flow_mol_s > 0.0, "{}", case.label);
            assert!(vapor.total_molar_flow_mol_s > 0.0, "{}", case.label);

            let liquid_window = liquid
                .bubble_dew_window
                .as_ref()
                .expect("expected liquid outlet bubble/dew window");
            assert_eq!(
                liquid_window.phase_region,
                PhaseEquilibriumRegion::TwoPhase,
                "{}",
                case.label
            );
            assert_close(liquid_window.bubble_pressure_pa, liquid.pressure_pa, 1e-6);
            assert_close(
                liquid_window.bubble_temperature_k,
                liquid.temperature_k,
                1e-4,
            );

            let vapor_window = vapor
                .bubble_dew_window
                .as_ref()
                .expect("expected vapor outlet bubble/dew window");
            assert_eq!(
                vapor_window.phase_region,
                PhaseEquilibriumRegion::TwoPhase,
                "{}",
                case.label
            );
            assert_close(vapor_window.dew_pressure_pa, vapor.pressure_pa, 1e-6);
            assert_close(vapor_window.dew_temperature_k, vapor.temperature_k, 1e-4);
        }
        PhaseEquilibriumRegion::VaporOnly => {
            assert_close(liquid.total_molar_flow_mol_s, 0.0, 1e-12);
            assert!(vapor.total_molar_flow_mol_s > 0.0, "{}", case.label);
            assert!(
                liquid.bubble_dew_window.is_none(),
                "expected liquid outlet bubble/dew window to be absent for {}",
                case.label
            );
            assert_eq!(
                vapor
                    .bubble_dew_window
                    .as_ref()
                    .expect("expected vapor outlet bubble/dew window")
                    .phase_region,
                PhaseEquilibriumRegion::VaporOnly,
                "{}",
                case.label
            );
        }
    }
}

fn app_state_for_binary_heater_boundary_case(
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
    case: &NearBoundaryStreamWindowCase,
) -> AppState {
    let mut app_state = app_state_from_project(
        include_str!(
            "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ),
        document_id,
        title,
        created_at_seconds,
    );
    apply_case_composition(
        &mut app_state,
        "stream-feed",
        case.package_id,
        case.overall_mole_fractions,
    );
    app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-feed".into())
        .expect("expected feed stream")
        .pressure_pa = 700_000.0;
    apply_case_feed_state(&mut app_state, "stream-heated", case);
    app_state
}

fn app_state_for_binary_mixer_boundary_case(
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
    case: &NearBoundaryStreamWindowCase,
) -> AppState {
    let mut app_state = app_state_from_project(
        include_str!(
            "../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
        ),
        document_id,
        title,
        created_at_seconds,
    );
    for stream_id in ["stream-feed-a", "stream-feed-b"] {
        apply_case_composition(
            &mut app_state,
            stream_id,
            case.package_id,
            case.overall_mole_fractions,
        );
        apply_case_feed_state(&mut app_state, stream_id, case);
    }
    app_state
}

fn app_state_for_binary_cooler_boundary_case(
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
    case: &NearBoundaryStreamWindowCase,
) -> AppState {
    let mut app_state = app_state_from_project(
        include_str!(
            "../../../examples/flowsheets/feed-cooler-flash-binary-hydrocarbon.rfproj.json"
        ),
        document_id,
        title,
        created_at_seconds,
    );
    apply_case_composition(
        &mut app_state,
        "stream-feed",
        case.package_id,
        case.overall_mole_fractions,
    );
    app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-feed".into())
        .expect("expected feed stream")
        .pressure_pa = 700_000.0;
    apply_case_feed_state(&mut app_state, "stream-cooled", case);
    app_state
}

fn app_state_for_binary_valve_boundary_case(
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
    case: &NearBoundaryStreamWindowCase,
) -> AppState {
    let mut app_state = app_state_from_project(
        include_str!(
            "../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
        ),
        document_id,
        title,
        created_at_seconds,
    );
    apply_case_composition(
        &mut app_state,
        "stream-feed",
        case.package_id,
        case.overall_mole_fractions,
    );
    let feed = app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-feed".into())
        .expect("expected feed stream");
    feed.temperature_k = case.temperature_k;
    feed.pressure_pa = 700_000.0;
    let throttled = app_state
        .workspace
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-throttled".into())
        .expect("expected throttled stream");
    throttled.temperature_k = case.temperature_k;
    throttled.pressure_pa = case.pressure_pa;
    app_state
}

fn app_state_for_synthetic_mixer_boundary_case(
    document_id: &str,
    title: &str,
    created_at_seconds: u64,
    case: &NearBoundaryStreamWindowCase,
) -> AppState {
    let mut app_state = app_state_from_project(
        include_str!("../../../examples/flowsheets/feed-mixer-flash.rfproj.json"),
        document_id,
        title,
        created_at_seconds,
    );
    for stream_id in ["stream-feed-a", "stream-feed-b"] {
        apply_case_composition(
            &mut app_state,
            stream_id,
            case.package_id,
            case.overall_mole_fractions,
        );
        apply_case_feed_state(&mut app_state, stream_id, case);
    }
    app_state
}

fn assert_near_boundary_cases_across_chain<F>(
    cases: Vec<NearBoundaryStreamWindowCase>,
    snapshot_prefix: &str,
    source_consumer_unit_id: &str,
    source_stream_ids: &[&str],
    flash_inlet_stream_id: &str,
    build_app_state: F,
) where
    F: Fn(usize, &NearBoundaryStreamWindowCase) -> AppState,
{
    for (index, case) in cases.into_iter().enumerate() {
        let provider = near_boundary_package_provider_for_case(&case);
        let mut app_state = build_app_state(index, &case);

        solve_workspace_with_property_package(
            &mut app_state,
            &provider,
            &StudioSolveRequest::new(case.package_id, format!("{snapshot_prefix}-{index}"), 1),
        )
        .expect("expected solve");

        let snapshot = app_state
            .workspace
            .snapshot_history
            .back()
            .expect("expected stored snapshot");
        for stream_id in source_stream_ids {
            let source_stream = find_snapshot_stream(snapshot, stream_id);
            assert_step_consumes_snapshot_stream(
                snapshot,
                source_consumer_unit_id,
                source_stream,
                &case.label,
            );
        }
        let inlet_stream = find_snapshot_stream(snapshot, flash_inlet_stream_id);
        assert_near_boundary_window_matches_case(inlet_stream, &case);
        assert_flash_consumed_stream_matches_inlet(snapshot, inlet_stream, &case);
        assert_flash_outlet_window_semantics_match_case(snapshot, &case);
    }
}

#[test]
fn studio_solver_bridge_preserves_pressure_near_boundary_windows_across_binary_heater_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        binary_hydrocarbon_lite_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Pressure)
            .collect(),
        "snapshot-studio-binary-heater-pressure",
        "heater-1",
        &["stream-feed"],
        "stream-heated",
        |index, case| {
            app_state_for_binary_heater_boundary_case(
                &format!("doc-studio-binary-heater-pressure-{index}"),
                &case.label,
                360 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn studio_solver_bridge_preserves_temperature_near_boundary_windows_across_binary_heater_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        binary_hydrocarbon_lite_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Temperature)
            .collect(),
        "snapshot-studio-binary-heater-temperature",
        "heater-1",
        &["stream-feed"],
        "stream-heated",
        |index, case| {
            app_state_for_binary_heater_boundary_case(
                &format!("doc-studio-binary-heater-temperature-{index}"),
                &case.label,
                380 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn studio_solver_bridge_preserves_pressure_near_boundary_windows_across_binary_mixer_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        binary_hydrocarbon_lite_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Pressure)
            .collect(),
        "snapshot-studio-binary-mixer-pressure",
        "mixer-1",
        &["stream-feed-a", "stream-feed-b"],
        "stream-mix-out",
        |index, case| {
            app_state_for_binary_mixer_boundary_case(
                &format!("doc-studio-binary-mixer-pressure-{index}"),
                &case.label,
                400 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn studio_solver_bridge_preserves_temperature_near_boundary_windows_across_binary_mixer_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        binary_hydrocarbon_lite_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Temperature)
            .collect(),
        "snapshot-studio-binary-mixer-temperature",
        "mixer-1",
        &["stream-feed-a", "stream-feed-b"],
        "stream-mix-out",
        |index, case| {
            app_state_for_binary_mixer_boundary_case(
                &format!("doc-studio-binary-mixer-temperature-{index}"),
                &case.label,
                440 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn studio_solver_bridge_preserves_pressure_near_boundary_windows_across_binary_cooler_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        binary_hydrocarbon_lite_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Pressure)
            .collect(),
        "snapshot-studio-binary-cooler-pressure",
        "cooler-1",
        &["stream-feed"],
        "stream-cooled",
        |index, case| {
            app_state_for_binary_cooler_boundary_case(
                &format!("doc-studio-binary-cooler-pressure-{index}"),
                &case.label,
                560 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn studio_solver_bridge_preserves_temperature_near_boundary_windows_across_binary_cooler_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        binary_hydrocarbon_lite_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Temperature)
            .collect(),
        "snapshot-studio-binary-cooler-temperature",
        "cooler-1",
        &["stream-feed"],
        "stream-cooled",
        |index, case| {
            app_state_for_binary_cooler_boundary_case(
                &format!("doc-studio-binary-cooler-temperature-{index}"),
                &case.label,
                600 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn studio_solver_bridge_preserves_pressure_near_boundary_windows_across_binary_valve_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        binary_hydrocarbon_lite_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Pressure)
            .collect(),
        "snapshot-studio-binary-valve-pressure",
        "valve-1",
        &["stream-feed"],
        "stream-throttled",
        |index, case| {
            app_state_for_binary_valve_boundary_case(
                &format!("doc-studio-binary-valve-pressure-{index}"),
                &case.label,
                640 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn studio_solver_bridge_preserves_temperature_near_boundary_windows_across_binary_valve_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        binary_hydrocarbon_lite_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Temperature)
            .collect(),
        "snapshot-studio-binary-valve-temperature",
        "valve-1",
        &["stream-feed"],
        "stream-throttled",
        |index, case| {
            app_state_for_binary_valve_boundary_case(
                &format!("doc-studio-binary-valve-temperature-{index}"),
                &case.label,
                680 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn studio_solver_bridge_preserves_synthetic_single_phase_pressure_near_boundary_windows_across_mixer_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        synthetic_single_phase_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Pressure)
            .collect(),
        "snapshot-studio-synthetic-mixer-pressure",
        "mixer-1",
        &["stream-feed-a", "stream-feed-b"],
        "stream-mix-out",
        |index, case| {
            app_state_for_synthetic_mixer_boundary_case(
                &format!("doc-studio-synthetic-mixer-pressure-{index}"),
                &case.label,
                480 + index as u64,
                case,
            )
        },
    );
}

#[test]
fn studio_solver_bridge_preserves_synthetic_single_phase_temperature_near_boundary_windows_across_mixer_flash_inlet_end_to_end()
 {
    assert_near_boundary_cases_across_chain(
        synthetic_single_phase_near_boundary_stream_window_cases()
            .into_iter()
            .filter(|case| case.kind == NearBoundaryCaseKind::Temperature)
            .collect(),
        "snapshot-studio-synthetic-mixer-temperature",
        "mixer-1",
        &["stream-feed-a", "stream-feed-b"],
        "stream-mix-out",
        |index, case| {
            app_state_for_synthetic_mixer_boundary_case(
                &format!("doc-studio-synthetic-mixer-temperature-{index}"),
                &case.label,
                520 + index as u64,
                case,
            )
        },
    );
}
