use rf_flash::PlaceholderTpFlashSolver;
use rf_rust_integration::{
    NearBoundaryCaseKind, NearBoundaryStreamWindowCase, SYNTHETIC_LIQUID_ONLY_PACKAGE_ID,
    SYNTHETIC_VAPOR_ONLY_PACKAGE_ID, assert_close, build_demo_antoine_coefficients,
    expected_overall_molar_enthalpy_for_case, near_boundary_component_ids_for_package,
    synthetic_single_phase_near_boundary_stream_window_cases,
};
use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolveStatus, SolverServices};
use rf_store::parse_project_file_json;
use rf_thermo::{PlaceholderThermoProvider, ThermoComponent, ThermoSystem};
use rf_types::{ComponentId, PhaseEquilibriumRegion, PhaseLabel, StreamId, UnitId};

fn build_synthetic_near_boundary_provider(package_id: &str) -> PlaceholderThermoProvider {
    let k_values = match package_id {
        SYNTHETIC_LIQUID_ONLY_PACKAGE_ID => [0.8, 0.6],
        SYNTHETIC_VAPOR_ONLY_PACKAGE_ID => [1.8, 1.3],
        _ => panic!("unexpected synthetic near-boundary package id `{package_id}`"),
    };
    let pressure_pa = 100_000.0_f64;

    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(build_demo_antoine_coefficients(k_values[0], pressure_pa));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(build_demo_antoine_coefficients(k_values[1], pressure_pa));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
}

fn solve_example_with_case<F>(
    project_json: &str,
    case: &NearBoundaryStreamWindowCase,
    edit_project: F,
) -> rf_solver::SolveSnapshot
where
    F: FnOnce(&mut rf_store::StoredProjectFile),
{
    let provider = build_synthetic_near_boundary_provider(case.package_id);
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let mut project =
        parse_project_file_json(project_json).expect("expected example project parse");
    edit_project(&mut project);

    SequentialModularSolver
        .solve(&services, &project.document.flowsheet)
        .expect("expected solve snapshot")
}

fn apply_case_composition(
    project: &mut rf_store::StoredProjectFile,
    stream_id: &str,
    case: &NearBoundaryStreamWindowCase,
) {
    let [first_component_id, second_component_id] =
        near_boundary_component_ids_for_package(case.package_id);
    let stream = project
        .document
        .flowsheet
        .streams
        .get_mut(&stream_id.into())
        .expect("expected stream");
    stream.overall_mole_fractions.clear();
    stream.overall_mole_fractions.insert(
        ComponentId::new(first_component_id),
        case.overall_mole_fractions[0],
    );
    stream.overall_mole_fractions.insert(
        ComponentId::new(second_component_id),
        case.overall_mole_fractions[1],
    );
}

fn apply_case_state(
    project: &mut rf_store::StoredProjectFile,
    stream_id: &str,
    case: &NearBoundaryStreamWindowCase,
) {
    let stream = project
        .document
        .flowsheet
        .streams
        .get_mut(&stream_id.into())
        .expect("expected stream");
    stream.temperature_k = case.temperature_k;
    stream.pressure_pa = case.pressure_pa;
}

fn assert_step_consumes_snapshot_stream(
    snapshot: &rf_solver::SolveSnapshot,
    unit_id: &str,
    stream: &rf_model::MaterialStreamState,
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
            .find(|phase| phase.label == PhaseLabel::Overall)
            .and_then(|phase| phase.molar_enthalpy_j_per_mol)
            .is_some(),
        "{}",
        case_label
    );
    assert!(stream.bubble_dew_window.is_some(), "{}", case_label);
}

fn assert_near_boundary_window_matches_case(
    stream: &rf_model::MaterialStreamState,
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
        .find(|phase| phase.label == PhaseLabel::Overall)
        .expect("expected overall phase");

    assert_close(stream.temperature_k, case.temperature_k, 1e-12);
    assert_close(stream.pressure_pa, case.pressure_pa, 1e-9);
    assert_close(
        *stream
            .overall_mole_fractions
            .get(&ComponentId::new(first_component_id))
            .expect("expected first component"),
        case.overall_mole_fractions[0],
        1e-12,
    );
    assert_close(
        *stream
            .overall_mole_fractions
            .get(&ComponentId::new(second_component_id))
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
    snapshot: &rf_solver::SolveSnapshot,
    inlet: &rf_model::MaterialStreamState,
    case_label: &str,
) {
    let flash_step = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == UnitId::new("flash-1"))
        .expect("expected flash step");
    assert_eq!(flash_step.consumed_streams.len(), 1, "{}", case_label);
    assert_eq!(&flash_step.consumed_streams[0], inlet, "{}", case_label);
}

fn assert_flash_outlet_window_semantics_match_case(
    snapshot: &rf_solver::SolveSnapshot,
    case: &NearBoundaryStreamWindowCase,
) {
    let liquid = snapshot
        .stream(&StreamId::new("stream-liquid"))
        .expect("expected liquid outlet");
    let vapor = snapshot
        .stream(&StreamId::new("stream-vapor"))
        .expect("expected vapor outlet");

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

fn assert_synthetic_near_boundary_cases_across_chain<F>(
    project_json: &str,
    source_consumer_unit_id: &str,
    source_stream_ids: &[&str],
    flash_inlet_stream_id: &str,
    kind: NearBoundaryCaseKind,
    edit_project: F,
) where
    F: Fn(&mut rf_store::StoredProjectFile, &NearBoundaryStreamWindowCase),
{
    for case in synthetic_single_phase_near_boundary_stream_window_cases()
        .into_iter()
        .filter(|case| case.kind == kind)
    {
        let snapshot =
            solve_example_with_case(project_json, &case, |project| edit_project(project, &case));

        assert_eq!(snapshot.status, SolveStatus::Converged, "{}", case.label);
        for stream_id in source_stream_ids {
            let source_stream = snapshot
                .stream(&StreamId::new(*stream_id))
                .expect("expected source stream");
            assert_step_consumes_snapshot_stream(
                &snapshot,
                source_consumer_unit_id,
                source_stream,
                &case.label,
            );
        }
        let inlet = snapshot
            .stream(&StreamId::new(flash_inlet_stream_id))
            .expect("expected flash inlet stream");
        assert_near_boundary_window_matches_case(inlet, &case);
        assert_flash_consumed_stream_matches_inlet(&snapshot, inlet, &case.label);
        assert_flash_outlet_window_semantics_match_case(&snapshot, &case);
    }
}

#[test]
fn synthetic_heater_flash_near_boundary_pressure_cases_preserve_inlet_and_outlet_windows_end_to_end()
 {
    assert_synthetic_near_boundary_cases_across_chain(
        include_str!("../../../examples/flowsheets/feed-heater-flash-synthetic-demo.rfproj.json"),
        "heater-1",
        &["stream-feed"],
        "stream-heated",
        NearBoundaryCaseKind::Pressure,
        |project, case| {
            apply_case_composition(project, "stream-feed", case);
            apply_case_state(project, "stream-feed", case);
            apply_case_state(project, "stream-heated", case);
        },
    );
}

#[test]
fn synthetic_heater_flash_near_boundary_temperature_cases_preserve_inlet_and_outlet_windows_end_to_end()
 {
    assert_synthetic_near_boundary_cases_across_chain(
        include_str!("../../../examples/flowsheets/feed-heater-flash-synthetic-demo.rfproj.json"),
        "heater-1",
        &["stream-feed"],
        "stream-heated",
        NearBoundaryCaseKind::Temperature,
        |project, case| {
            apply_case_composition(project, "stream-feed", case);
            apply_case_state(project, "stream-feed", case);
            apply_case_state(project, "stream-heated", case);
        },
    );
}

#[test]
fn synthetic_cooler_flash_near_boundary_pressure_cases_preserve_inlet_and_outlet_windows_end_to_end()
 {
    assert_synthetic_near_boundary_cases_across_chain(
        include_str!("../../../examples/flowsheets/feed-cooler-flash-synthetic-demo.rfproj.json"),
        "cooler-1",
        &["stream-feed"],
        "stream-cooled",
        NearBoundaryCaseKind::Pressure,
        |project, case| {
            apply_case_composition(project, "stream-feed", case);
            apply_case_state(project, "stream-feed", case);
            apply_case_state(project, "stream-cooled", case);
        },
    );
}

#[test]
fn synthetic_cooler_flash_near_boundary_temperature_cases_preserve_inlet_and_outlet_windows_end_to_end()
 {
    assert_synthetic_near_boundary_cases_across_chain(
        include_str!("../../../examples/flowsheets/feed-cooler-flash-synthetic-demo.rfproj.json"),
        "cooler-1",
        &["stream-feed"],
        "stream-cooled",
        NearBoundaryCaseKind::Temperature,
        |project, case| {
            apply_case_composition(project, "stream-feed", case);
            apply_case_state(project, "stream-feed", case);
            apply_case_state(project, "stream-cooled", case);
        },
    );
}

#[test]
fn synthetic_valve_flash_near_boundary_pressure_cases_preserve_inlet_and_outlet_windows_end_to_end()
{
    assert_synthetic_near_boundary_cases_across_chain(
        include_str!("../../../examples/flowsheets/feed-valve-flash-synthetic-demo.rfproj.json"),
        "valve-1",
        &["stream-feed"],
        "stream-throttled",
        NearBoundaryCaseKind::Pressure,
        |project, case| {
            apply_case_composition(project, "stream-feed", case);
            apply_case_state(project, "stream-feed", case);
            apply_case_state(project, "stream-throttled", case);
        },
    );
}

#[test]
fn synthetic_valve_flash_near_boundary_temperature_cases_preserve_inlet_and_outlet_windows_end_to_end()
 {
    assert_synthetic_near_boundary_cases_across_chain(
        include_str!("../../../examples/flowsheets/feed-valve-flash-synthetic-demo.rfproj.json"),
        "valve-1",
        &["stream-feed"],
        "stream-throttled",
        NearBoundaryCaseKind::Temperature,
        |project, case| {
            apply_case_composition(project, "stream-feed", case);
            apply_case_state(project, "stream-feed", case);
            apply_case_state(project, "stream-throttled", case);
        },
    );
}
