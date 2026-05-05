use std::collections::BTreeMap;

use rf_flash::PlaceholderTpFlashSolver;
use rf_model::{Component, Composition, Flowsheet, MaterialStreamState, UnitNode, UnitPort};
use rf_store::parse_project_file_json;
use rf_thermo::{AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem};
use rf_types::{
    ComponentId, DiagnosticPortTarget, PhaseLabel, PortDirection, PortKind, RfError, StreamId,
    UnitId,
};
use rf_unitops::{
    UnitOperationOutputs, build_cooler_node, build_feed_node, build_flash_drum_node,
    build_heater_node, build_mixer_node, build_valve_node,
};

use super::{
    FlowsheetSolver, SequentialModularSolver, SolveDiagnosticSeverity, SolveFailureContext,
    SolveStatus, SolverDiagnosticCode, SolverServices, find_port, instantiate_operation,
    materialized_output_stream, port_stream_id, resolved_stream_for_port, solver_step_error,
    solver_step_execution_error, solver_step_lookup_error, stream_for_port,
};

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
    );
}

fn binary_composition(first: f64, second: f64) -> Composition {
    [
        (ComponentId::new("component-a"), first),
        (ComponentId::new("component-b"), second),
    ]
    .into_iter()
    .collect()
}

fn build_stream(
    id: &str,
    name: &str,
    temperature_k: f64,
    pressure_pa: f64,
    total_molar_flow_mol_s: f64,
    composition: Composition,
) -> MaterialStreamState {
    MaterialStreamState::from_tpzf(
        id,
        name,
        temperature_k,
        pressure_pa,
        total_molar_flow_mol_s,
        composition,
    )
}

fn build_provider() -> PlaceholderThermoProvider {
    let pressure_pa = 100_000.0_f64;
    let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
    first.antoine = Some(AntoineCoefficients::new(
        ((2.0_f64 * pressure_pa) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));
    first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
    second.antoine = Some(AntoineCoefficients::new(
        ((0.5_f64 * pressure_pa) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));
    second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
}

fn build_demo_flowsheet() -> Flowsheet {
    let mut flowsheet = Flowsheet::new("feed-mixer-flash");
    for component in [
        Component::new("component-a", "Component A"),
        Component::new("component-b", "Component B"),
    ] {
        flowsheet
            .insert_component(component)
            .expect("expected component insert");
    }

    for stream in [
        build_stream(
            "stream-feed-a",
            "Feed A",
            300.0,
            120_000.0,
            2.0,
            binary_composition(0.25, 0.75),
        ),
        build_stream(
            "stream-feed-b",
            "Feed B",
            360.0,
            100_000.0,
            3.0,
            binary_composition(0.60, 0.40),
        ),
        build_stream(
            "stream-mix-out",
            "Mixer Outlet",
            330.0,
            100_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-liquid",
            "Liquid Outlet",
            330.0,
            100_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-vapor",
            "Vapor Outlet",
            330.0,
            100_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
    ] {
        flowsheet
            .insert_stream(stream)
            .expect("expected stream insert");
    }

    for unit in [
        build_feed_node("feed-a", "Feed A", "stream-feed-a"),
        build_feed_node("feed-b", "Feed B", "stream-feed-b"),
        build_mixer_node(
            "mixer-1",
            "Mixer",
            "stream-feed-a",
            "stream-feed-b",
            "stream-mix-out",
        ),
        build_flash_drum_node(
            "flash-1",
            "Flash Drum",
            "stream-mix-out",
            "stream-liquid",
            "stream-vapor",
        ),
    ] {
        flowsheet.insert_unit(unit).expect("expected unit insert");
    }

    flowsheet
}

fn build_feed_heater_flash_flowsheet() -> Flowsheet {
    let mut flowsheet = Flowsheet::new("feed-heater-flash");
    for component in [
        Component::new("component-a", "Component A"),
        Component::new("component-b", "Component B"),
    ] {
        flowsheet
            .insert_component(component)
            .expect("expected component insert");
    }

    for stream in [
        build_stream(
            "stream-feed",
            "Feed",
            300.0,
            120_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ),
        build_stream(
            "stream-heated",
            "Heated Outlet",
            345.0,
            95_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-liquid",
            "Liquid Outlet",
            345.0,
            95_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-vapor",
            "Vapor Outlet",
            345.0,
            95_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
    ] {
        flowsheet
            .insert_stream(stream)
            .expect("expected stream insert");
    }

    for unit in [
        build_feed_node("feed-1", "Feed", "stream-feed"),
        build_heater_node("heater-1", "Heater", "stream-feed", "stream-heated"),
        build_flash_drum_node(
            "flash-1",
            "Flash Drum",
            "stream-heated",
            "stream-liquid",
            "stream-vapor",
        ),
    ] {
        flowsheet.insert_unit(unit).expect("expected unit insert");
    }

    flowsheet
}

fn build_feed_mixer_heater_flash_flowsheet() -> Flowsheet {
    let mut flowsheet = Flowsheet::new("feed-mixer-heater-flash");
    for component in [
        Component::new("component-a", "Component A"),
        Component::new("component-b", "Component B"),
    ] {
        flowsheet
            .insert_component(component)
            .expect("expected component insert");
    }

    for stream in [
        build_stream(
            "stream-feed-a",
            "Feed A",
            300.0,
            120_000.0,
            2.0,
            binary_composition(0.25, 0.75),
        ),
        build_stream(
            "stream-feed-b",
            "Feed B",
            360.0,
            100_000.0,
            3.0,
            binary_composition(0.60, 0.40),
        ),
        build_stream(
            "stream-mix-out",
            "Mixer Outlet",
            330.0,
            100_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-heated",
            "Heated Outlet",
            350.0,
            95_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-liquid",
            "Liquid Outlet",
            350.0,
            95_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-vapor",
            "Vapor Outlet",
            350.0,
            95_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
    ] {
        flowsheet
            .insert_stream(stream)
            .expect("expected stream insert");
    }

    for unit in [
        build_feed_node("feed-a", "Feed A", "stream-feed-a"),
        build_feed_node("feed-b", "Feed B", "stream-feed-b"),
        build_mixer_node(
            "mixer-1",
            "Mixer",
            "stream-feed-a",
            "stream-feed-b",
            "stream-mix-out",
        ),
        build_heater_node("heater-1", "Heater", "stream-mix-out", "stream-heated"),
        build_flash_drum_node(
            "flash-1",
            "Flash Drum",
            "stream-heated",
            "stream-liquid",
            "stream-vapor",
        ),
    ] {
        flowsheet.insert_unit(unit).expect("expected unit insert");
    }

    flowsheet
}

fn build_feed_valve_flash_flowsheet() -> Flowsheet {
    let mut flowsheet = Flowsheet::new("feed-valve-flash");
    for component in [
        Component::new("component-a", "Component A"),
        Component::new("component-b", "Component B"),
    ] {
        flowsheet
            .insert_component(component)
            .expect("expected component insert");
    }

    for stream in [
        build_stream(
            "stream-feed",
            "Feed",
            315.0,
            120_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ),
        build_stream(
            "stream-throttled",
            "Valve Outlet",
            300.0,
            90_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-liquid",
            "Liquid Outlet",
            315.0,
            90_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-vapor",
            "Vapor Outlet",
            315.0,
            90_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
    ] {
        flowsheet
            .insert_stream(stream)
            .expect("expected stream insert");
    }

    for unit in [
        build_feed_node("feed-1", "Feed", "stream-feed"),
        build_valve_node("valve-1", "Valve", "stream-feed", "stream-throttled"),
        build_flash_drum_node(
            "flash-1",
            "Flash Drum",
            "stream-throttled",
            "stream-liquid",
            "stream-vapor",
        ),
    ] {
        flowsheet.insert_unit(unit).expect("expected unit insert");
    }

    flowsheet
}

fn build_feed_cooler_flash_flowsheet() -> Flowsheet {
    let mut flowsheet = Flowsheet::new("feed-cooler-flash");
    for component in [
        Component::new("component-a", "Component A"),
        Component::new("component-b", "Component B"),
    ] {
        flowsheet
            .insert_component(component)
            .expect("expected component insert");
    }

    for stream in [
        build_stream(
            "stream-feed",
            "Feed",
            360.0,
            120_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ),
        build_stream(
            "stream-cooled",
            "Cooled Outlet",
            305.0,
            98_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-liquid",
            "Liquid Outlet",
            305.0,
            98_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
        build_stream(
            "stream-vapor",
            "Vapor Outlet",
            305.0,
            98_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        ),
    ] {
        flowsheet
            .insert_stream(stream)
            .expect("expected stream insert");
    }

    for unit in [
        build_feed_node("feed-1", "Feed", "stream-feed"),
        build_cooler_node("cooler-1", "Cooler", "stream-feed", "stream-cooled"),
        build_flash_drum_node(
            "flash-1",
            "Flash Drum",
            "stream-cooled",
            "stream-liquid",
            "stream-vapor",
        ),
    ] {
        flowsheet.insert_unit(unit).expect("expected unit insert");
    }

    flowsheet
}

#[test]
fn sequential_solver_solves_feed_mixer_flash_chain() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };

    let snapshot = SequentialModularSolver
        .solve(&services, &build_demo_flowsheet())
        .expect("expected solve snapshot");

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(
        snapshot.summary.highest_severity,
        SolveDiagnosticSeverity::Info
    );
    assert_eq!(snapshot.summary.diagnostic_count, 5);
    assert_eq!(snapshot.steps.len(), 4);
    assert_eq!(snapshot.steps[0].index, 0);
    assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-a");
    assert_eq!(snapshot.steps[1].unit_id.as_str(), "feed-b");
    assert_eq!(snapshot.steps[2].unit_id.as_str(), "mixer-1");
    assert_eq!(snapshot.steps[3].unit_id.as_str(), "flash-1");
    assert_eq!(snapshot.steps[2].consumed_stream_ids.len(), 2);
    assert!(
        snapshot.steps[2]
            .summary
            .contains("produced 1 outlet stream")
    );
    assert_eq!(snapshot.diagnostics[0].code, "solver.execution_order");
    assert_eq!(snapshot.diagnostics[1].code, "solver.unit_executed");

    let mixer_out = snapshot
        .stream(&"stream-mix-out".into())
        .expect("expected mixer outlet");
    assert_close(mixer_out.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(mixer_out.temperature_k, 336.0, 1e-12);
    assert_close(
        *mixer_out
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.46,
        1e-12,
    );

    let liquid = snapshot
        .stream(&"stream-liquid".into())
        .expect("expected liquid outlet");
    let vapor = snapshot
        .stream(&"stream-vapor".into())
        .expect("expected vapor outlet");
    assert_close(liquid.total_molar_flow_mol_s, 3.099999999994907, 1e-9);
    assert_close(vapor.total_molar_flow_mol_s, 1.900000000005093, 1e-9);
    assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
    assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
}

#[test]
fn sequential_solver_runs_example_project_file() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let project = parse_project_file_json(include_str!(
        "../../../examples/flowsheets/feed-mixer-flash.rfproj.json"
    ))
    .expect("expected example project parse");

    let snapshot = SequentialModularSolver
        .solve(&services, &project.document.flowsheet)
        .expect("expected solve snapshot");

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.summary.related_unit_ids.len(), 4);
    assert_eq!(snapshot.steps.len(), 4);
    assert!(snapshot.stream(&"stream-liquid".into()).is_some());
    assert!(snapshot.stream(&"stream-vapor".into()).is_some());
}

#[test]
fn sequential_solver_solves_feed_heater_flash_chain() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };

    let snapshot = SequentialModularSolver
        .solve(&services, &build_feed_heater_flash_flowsheet())
        .expect("expected solve snapshot");

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.summary.diagnostic_count, 4);
    assert_eq!(snapshot.steps.len(), 3);
    assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-1");
    assert_eq!(snapshot.steps[1].unit_id.as_str(), "heater-1");
    assert_eq!(snapshot.steps[2].unit_id.as_str(), "flash-1");
    assert_eq!(
        snapshot.steps[1].consumed_stream_ids,
        vec!["stream-feed".into()]
    );
    assert!(snapshot.steps[1].summary.contains("heater-1"));

    let heated = snapshot
        .stream(&"stream-heated".into())
        .expect("expected heated outlet");
    assert_close(heated.temperature_k, 345.0, 1e-12);
    assert_close(heated.pressure_pa, 95_000.0, 1e-12);
    assert_close(heated.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *heated
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.35,
        1e-12,
    );

    let liquid = snapshot
        .stream(&"stream-liquid".into())
        .expect("expected liquid outlet");
    let vapor = snapshot
        .stream(&"stream-vapor".into())
        .expect("expected vapor outlet");
    assert!(liquid.total_molar_flow_mol_s > 0.0);
    assert!(vapor.total_molar_flow_mol_s > 0.0);
    assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
    assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
}

#[test]
fn sequential_solver_runs_feed_heater_flash_example_project_file() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let project = parse_project_file_json(include_str!(
        "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
    ))
    .expect("expected example project parse");

    let snapshot = SequentialModularSolver
        .solve(&services, &project.document.flowsheet)
        .expect("expected solve snapshot");

    let heated = snapshot
        .stream(&"stream-heated".into())
        .expect("expected heated outlet");
    assert_close(heated.temperature_k, 345.0, 1e-12);
    assert_close(heated.pressure_pa, 95_000.0, 1e-12);
    assert_eq!(snapshot.steps.len(), 3);
    assert!(snapshot.stream(&"stream-liquid".into()).is_some());
    assert!(snapshot.stream(&"stream-vapor".into()).is_some());
}

#[test]
fn sequential_solver_solves_feed_mixer_heater_flash_chain() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };

    let snapshot = SequentialModularSolver
        .solve(&services, &build_feed_mixer_heater_flash_flowsheet())
        .expect("expected solve snapshot");

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.summary.diagnostic_count, 6);
    assert_eq!(snapshot.steps.len(), 5);
    assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-a");
    assert_eq!(snapshot.steps[1].unit_id.as_str(), "feed-b");
    assert_eq!(snapshot.steps[2].unit_id.as_str(), "mixer-1");
    assert_eq!(snapshot.steps[3].unit_id.as_str(), "heater-1");
    assert_eq!(snapshot.steps[4].unit_id.as_str(), "flash-1");

    let mixed = snapshot
        .stream(&"stream-mix-out".into())
        .expect("expected mixer outlet");
    assert_close(mixed.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(mixed.temperature_k, 336.0, 1e-12);
    assert_close(
        *mixed
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.46,
        1e-12,
    );

    let heated = snapshot
        .stream(&"stream-heated".into())
        .expect("expected heated outlet");
    assert_close(heated.temperature_k, 350.0, 1e-12);
    assert_close(heated.pressure_pa, 95_000.0, 1e-12);
    assert_close(heated.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *heated
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.46,
        1e-12,
    );
}

#[test]
fn sequential_solver_runs_feed_mixer_heater_flash_example_project_file() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let project = parse_project_file_json(include_str!(
        "../../../examples/flowsheets/feed-mixer-heater-flash.rfproj.json"
    ))
    .expect("expected example project parse");

    let snapshot = SequentialModularSolver
        .solve(&services, &project.document.flowsheet)
        .expect("expected solve snapshot");

    let heated = snapshot
        .stream(&"stream-heated".into())
        .expect("expected heated outlet");
    assert_close(heated.temperature_k, 350.0, 1e-12);
    assert_close(heated.pressure_pa, 95_000.0, 1e-12);
    assert_eq!(snapshot.steps.len(), 5);
    assert!(snapshot.stream(&"stream-liquid".into()).is_some());
    assert!(snapshot.stream(&"stream-vapor".into()).is_some());
}

#[test]
fn sequential_solver_solves_feed_cooler_flash_chain() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };

    let snapshot = SequentialModularSolver
        .solve(&services, &build_feed_cooler_flash_flowsheet())
        .expect("expected solve snapshot");

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.summary.diagnostic_count, 4);
    assert_eq!(snapshot.steps.len(), 3);
    assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-1");
    assert_eq!(snapshot.steps[1].unit_id.as_str(), "cooler-1");
    assert_eq!(snapshot.steps[2].unit_id.as_str(), "flash-1");
    assert_eq!(
        snapshot.steps[1].consumed_stream_ids,
        vec!["stream-feed".into()]
    );
    assert!(snapshot.steps[1].summary.contains("cooler-1"));

    let cooled = snapshot
        .stream(&"stream-cooled".into())
        .expect("expected cooled outlet");
    assert_close(cooled.temperature_k, 305.0, 1e-12);
    assert_close(cooled.pressure_pa, 98_000.0, 1e-12);
    assert_close(cooled.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *cooled
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.35,
        1e-12,
    );

    let liquid = snapshot
        .stream(&"stream-liquid".into())
        .expect("expected liquid outlet");
    let vapor = snapshot
        .stream(&"stream-vapor".into())
        .expect("expected vapor outlet");
    assert!(liquid.total_molar_flow_mol_s > 0.0);
    assert!(vapor.total_molar_flow_mol_s > 0.0);
    assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
    assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
}

#[test]
fn sequential_solver_runs_feed_cooler_flash_example_project_file() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let project = parse_project_file_json(include_str!(
        "../../../examples/flowsheets/feed-cooler-flash.rfproj.json"
    ))
    .expect("expected example project parse");

    let snapshot = SequentialModularSolver
        .solve(&services, &project.document.flowsheet)
        .expect("expected solve snapshot");

    let cooled = snapshot
        .stream(&"stream-cooled".into())
        .expect("expected cooled outlet");
    assert_close(cooled.temperature_k, 305.0, 1e-12);
    assert_close(cooled.pressure_pa, 98_000.0, 1e-12);
    assert_eq!(snapshot.steps.len(), 3);
    assert!(snapshot.stream(&"stream-liquid".into()).is_some());
    assert!(snapshot.stream(&"stream-vapor".into()).is_some());
}

#[test]
fn sequential_solver_solves_feed_valve_flash_chain() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };

    let snapshot = SequentialModularSolver
        .solve(&services, &build_feed_valve_flash_flowsheet())
        .expect("expected solve snapshot");

    assert_eq!(snapshot.status, SolveStatus::Converged);
    assert_eq!(snapshot.summary.diagnostic_count, 4);
    assert_eq!(snapshot.steps.len(), 3);
    assert_eq!(snapshot.steps[0].unit_id.as_str(), "feed-1");
    assert_eq!(snapshot.steps[1].unit_id.as_str(), "valve-1");
    assert_eq!(snapshot.steps[2].unit_id.as_str(), "flash-1");
    assert_eq!(
        snapshot.steps[1].consumed_stream_ids,
        vec!["stream-feed".into()]
    );
    assert!(snapshot.steps[1].summary.contains("valve-1"));

    let throttled = snapshot
        .stream(&"stream-throttled".into())
        .expect("expected valve outlet");
    assert_close(throttled.temperature_k, 315.0, 1e-12);
    assert_close(throttled.pressure_pa, 90_000.0, 1e-12);
    assert_close(throttled.total_molar_flow_mol_s, 5.0, 1e-12);
    assert_close(
        *throttled
            .overall_mole_fractions
            .get(&ComponentId::new("component-a"))
            .expect("expected component-a"),
        0.35,
        1e-12,
    );

    let liquid = snapshot
        .stream(&"stream-liquid".into())
        .expect("expected liquid outlet");
    let vapor = snapshot
        .stream(&"stream-vapor".into())
        .expect("expected vapor outlet");
    assert!(liquid.total_molar_flow_mol_s > 0.0);
    assert!(vapor.total_molar_flow_mol_s > 0.0);
}

#[test]
fn sequential_solver_runs_feed_valve_flash_example_project_file() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let project = parse_project_file_json(include_str!(
        "../../../examples/flowsheets/feed-valve-flash.rfproj.json"
    ))
    .expect("expected example project parse");

    let snapshot = SequentialModularSolver
        .solve(&services, &project.document.flowsheet)
        .expect("expected solve snapshot");

    let throttled = snapshot
        .stream(&"stream-throttled".into())
        .expect("expected valve outlet");
    assert_close(throttled.temperature_k, 315.0, 1e-12);
    assert_close(throttled.pressure_pa, 90_000.0, 1e-12);
    assert_eq!(snapshot.steps.len(), 3);
    assert!(snapshot.stream(&"stream-liquid".into()).is_some());
    assert!(snapshot.stream(&"stream-vapor".into()).is_some());
}

#[test]
fn sequential_solver_reports_step_context_for_unit_execution_failures() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let mut flowsheet = build_feed_valve_flash_flowsheet();
    flowsheet
        .streams
        .get_mut(&"stream-throttled".into())
        .expect("expected throttled stream")
        .pressure_pa = 130_000.0;

    let error = SequentialModularSolver
        .solve(&services, &flowsheet)
        .expect_err("expected valve execution failure");

    assert!(error.message().contains("solver.step.execution:"));
    assert!(
        error
            .message()
            .contains("step 2 unit execution failed for unit `valve-1` (`valve`)")
    );
    assert!(error.message().contains("after consuming [stream-feed]"));
}

#[test]
fn sequential_solver_reports_connection_validation_stage_context() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let mut flowsheet = build_feed_valve_flash_flowsheet();
    flowsheet
        .units
        .get_mut(&"flash-1".into())
        .expect("expected flash unit")
        .ports
        .iter_mut()
        .find(|port| port.name == "inlet")
        .expect("expected inlet port")
        .stream_id = Some("stream-feed".into());

    let error = SequentialModularSolver
        .solve(&services, &flowsheet)
        .expect_err("expected connection validation failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.duplicate_downstream_sink")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("flash-1"), UnitId::new("valve-1")]
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[StreamId::new("stream-feed")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[
            DiagnosticPortTarget::new("flash-1", "inlet"),
            DiagnosticPortTarget::new("valve-1", "inlet"),
        ]
    );
    assert!(
        error
            .message()
            .contains("solver connection validation failed")
    );
    assert!(error.message().contains("consumed by both"));
}

#[test]
fn sequential_solver_reports_unsupported_unit_kind_during_connection_validation() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let mut flowsheet = Flowsheet::new("unsupported-unit-kind");
    for component in [
        Component::new("component-a", "Component A"),
        Component::new("component-b", "Component B"),
    ] {
        flowsheet
            .insert_component(component)
            .expect("expected component insert");
    }
    flowsheet
        .insert_stream(build_stream(
            "stream-feed",
            "Feed",
            320.0,
            100_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ))
        .expect("expected stream insert");
    flowsheet
        .insert_unit(UnitNode::new(
            "mystery-1",
            "Mystery Unit",
            "pump",
            vec![UnitPort::new(
                "outlet",
                PortDirection::Outlet,
                PortKind::Material,
                Some("stream-feed".into()),
            )],
        ))
        .expect("expected unit insert");

    let error = SequentialModularSolver
        .solve(&services, &flowsheet)
        .expect_err("expected unsupported unit kind failure");

    assert_eq!(error.code().as_str(), "invalid_connection");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.connection_validation.unsupported_unit_kind")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("mystery-1")]
    );
    assert!(error.message().contains(
        "solver.connection_validation.unsupported_unit_kind: solver connection validation failed"
    ));
    assert!(
        error
            .message()
            .contains("canonical built-in port signature")
    );
    assert!(error.message().contains("unsupported kind `pump`"));
}

#[test]
fn sequential_solver_reports_topological_ordering_stage_context() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let mut flowsheet = Flowsheet::new("heater-valve-cycle");
    for component in [
        Component::new("component-a", "Component A"),
        Component::new("component-b", "Component B"),
    ] {
        flowsheet
            .insert_component(component)
            .expect("expected component insert");
    }
    for stream in [
        build_stream(
            "stream-a",
            "Cycle Stream A",
            320.0,
            100_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ),
        build_stream(
            "stream-b",
            "Cycle Stream B",
            300.0,
            95_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ),
    ] {
        flowsheet
            .insert_stream(stream)
            .expect("expected stream insert");
    }
    for unit in [
        build_heater_node("heater-1", "Heater", "stream-b", "stream-a"),
        build_valve_node("valve-1", "Valve", "stream-a", "stream-b"),
    ] {
        flowsheet.insert_unit(unit).expect("expected unit insert");
    }

    let error = SequentialModularSolver
        .solve(&services, &flowsheet)
        .expect_err("expected cycle detection failure");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.topological_ordering.two_unit_cycle")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("heater-1"), UnitId::new("valve-1")]
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[StreamId::new("stream-a"), StreamId::new("stream-b")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[
            DiagnosticPortTarget::new("valve-1", "inlet"),
            DiagnosticPortTarget::new("heater-1", "inlet"),
        ]
    );
    assert!(
        error
            .message()
            .contains("solver topological ordering failed")
    );
    assert!(error.message().contains("form a two-unit cycle"));
    assert!(
        error
            .message()
            .contains("streams `stream-a` and `stream-b`")
    );
}

#[test]
fn sequential_solver_reports_self_loop_as_topological_cycle() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let mut flowsheet = Flowsheet::new("self-loop");
    for component in [
        Component::new("component-a", "Component A"),
        Component::new("component-b", "Component B"),
    ] {
        flowsheet
            .insert_component(component)
            .expect("expected component insert");
    }
    for stream in [
        build_stream(
            "stream-loop",
            "Loop Stream",
            320.0,
            100_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ),
        build_stream(
            "stream-out",
            "Outlet Stream",
            300.0,
            95_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ),
    ] {
        flowsheet
            .insert_stream(stream)
            .expect("expected stream insert");
    }
    flowsheet
        .insert_unit(build_flash_drum_node(
            "flash-1",
            "Flash Drum",
            "stream-loop",
            "stream-loop",
            "stream-out",
        ))
        .expect("expected unit insert");

    let error = SequentialModularSolver
        .solve(&services, &flowsheet)
        .expect_err("expected self-loop cycle failure");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.topological_ordering.self_loop_cycle")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        &[UnitId::new("flash-1")]
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[StreamId::new("stream-loop")]
    );
    assert_eq!(
        error.context().related_port_targets(),
        &[
            DiagnosticPortTarget::new("flash-1", "inlet"),
            DiagnosticPortTarget::new("flash-1", "liquid"),
        ]
    );
    assert!(
        error
            .message()
            .contains("solver topological ordering failed")
    );
    assert!(error.message().contains("forms a self loop"));
    assert!(error.message().contains("stream `stream-loop`"));
}

#[test]
fn instantiate_operation_reports_unit_context_for_unsupported_kind() {
    let flowsheet = Flowsheet::new("unsupported-kind-helper");
    let unit = UnitNode::new("mystery-1", "Mystery Unit", "pump", Vec::new());

    let error = match instantiate_operation(&unit, &flowsheet) {
        Ok(_) => panic!("expected unsupported kind"),
        Err(error) => error,
    };

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(
        error
            .message()
            .contains("unit `mystery-1` (`pump`) uses unsupported solver kind `pump`")
    );
}

#[test]
fn find_port_reports_unit_context_for_missing_port() {
    let unit = build_feed_node("feed-1", "Feed", "stream-feed");

    let error = find_port(&unit, "missing-port").expect_err("expected missing port");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(
        error
            .message()
            .contains("unit `feed-1` (`feed`) does not define expected port `missing-port`")
    );
}

#[test]
fn port_stream_id_reports_unit_port_context_for_missing_stream_id() {
    let unit = UnitNode::new(
        "heater-1",
        "Heater",
        "heater",
        vec![UnitPort::new(
            "outlet",
            PortDirection::Outlet,
            PortKind::Material,
            None,
        )],
    );

    let error = port_stream_id(&unit, "outlet").expect_err("expected missing connected stream id");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(
        error
            .message()
            .contains("unit `heater-1` (`heater`) port `outlet` is missing a connected stream id")
    );
}

#[test]
fn stream_for_port_reports_unit_port_context_for_missing_stream_reference() {
    let mut flowsheet = Flowsheet::new("missing-stream-helper");
    let unit = build_feed_node("feed-1", "Feed", "stream-missing");

    flowsheet
        .insert_unit(unit.clone())
        .expect("expected unit insert");

    let error = stream_for_port(&unit, "outlet", &flowsheet).expect_err("expected missing stream");

    assert_eq!(error.code().as_str(), "missing_entity");
    assert!(error.message().contains(
        "unit `feed-1` (`feed`) port `outlet` references missing stream `stream-missing`"
    ));
}

#[test]
fn resolved_stream_for_port_reports_unit_port_context_for_unsolved_inlet() {
    let unit = build_heater_node("heater-1", "Heater", "stream-feed", "stream-heated");
    let solved_streams = BTreeMap::new();

    let error = resolved_stream_for_port(&unit, "inlet", &solved_streams)
        .expect_err("expected unsolved inlet stream");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(
        error
            .message()
            .contains("unit `heater-1` (`heater`) port `inlet` requires inlet stream `stream-feed` to be solved before this step")
    );
}

#[test]
fn solver_step_lookup_error_uses_stable_template() {
    let error = solver_step_lookup_error(
        3,
        &UnitId::new("flash-1"),
        RfError::missing_entity("unit", "flash-1"),
    );

    assert_eq!(error.code().as_str(), "missing_entity");
    assert_eq!(
        error.message(),
        "solver.step.lookup: solver step 3 unit lookup failed for `flash-1`: missing unit `flash-1`"
    );
}

#[test]
fn solver_step_error_uses_stable_template() {
    let unit = build_heater_node("heater-1", "Heater", "stream-feed", "stream-heated");
    let error = solver_step_error(
        2,
        &unit,
        SolverDiagnosticCode::StepSpec,
        RfError::invalid_input("port mismatch"),
    );

    assert_eq!(error.code().as_str(), "invalid_input");
    assert_eq!(
        error.message(),
        "solver.step.spec: solver step 2 unit spec validation failed for unit `heater-1` (`heater`): port mismatch"
    );
}

#[test]
fn solver_step_execution_error_uses_stable_code_and_template() {
    let unit = build_valve_node("valve-1", "Valve", "stream-feed", "stream-throttled");
    let error = solver_step_execution_error(
        2,
        &unit,
        &[StreamId::new("stream-feed")],
        RfError::invalid_input("valve outlet pressure cannot exceed inlet pressure"),
    );

    assert_eq!(error.code().as_str(), "invalid_input");
    assert_eq!(
        error.message(),
        "solver.step.execution: solver step 2 unit execution failed for unit `valve-1` (`valve`) after consuming [stream-feed]: valve outlet pressure cannot exceed inlet pressure"
    );
    assert_eq!(
        error.context().related_stream_ids(),
        &[StreamId::new("stream-feed")]
    );
}

#[test]
fn materialized_output_stream_reports_step_context_for_missing_port() {
    let unit = build_flash_drum_node(
        "flash-1",
        "Flash Drum",
        "stream-feed",
        "stream-liquid",
        "stream-vapor",
    );
    let outputs = UnitOperationOutputs::new();

    let error = materialized_output_stream(4, &unit, "liquid", &outputs)
        .expect_err("expected missing produced outlet port");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert_eq!(
        error.message(),
        "solver.step.materialization: solver step 4 output materialization failed for unit `flash-1` (`flash_drum`): missing produced outlet port `liquid`"
    );
}

#[test]
fn solve_failure_context_extracts_step_execution_code_and_unit() {
    let error = solver_step_execution_error(
        2,
        &build_valve_node("valve-1", "Valve", "stream-feed", "stream-throttled"),
        &[StreamId::new("stream-feed")],
        RfError::invalid_input("valve outlet pressure cannot exceed inlet pressure"),
    );

    let context = SolveFailureContext::from_error(&error);

    assert_eq!(
        context.primary_code.as_deref(),
        Some("solver.step.execution")
    );
    assert_eq!(context.related_unit_ids, vec![UnitId::new("valve-1")]);
    assert_eq!(
        context.related_stream_ids,
        vec![StreamId::new("stream-feed")]
    );
    assert!(context.related_port_targets.is_empty());
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.step.execution")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        [UnitId::new("valve-1")].as_slice()
    );
    assert_eq!(
        error.context().related_stream_ids(),
        [StreamId::new("stream-feed")].as_slice()
    );
}

#[test]
fn solve_failure_context_extracts_step_lookup_unit_id() {
    let error = solver_step_lookup_error(
        3,
        &UnitId::new("flash-1"),
        RfError::missing_entity("unit", "flash-1"),
    );

    let context = SolveFailureContext::from_error(&error);

    assert_eq!(context.primary_code.as_deref(), Some("solver.step.lookup"));
    assert_eq!(context.related_unit_ids, vec![UnitId::new("flash-1")]);
    assert!(context.related_stream_ids.is_empty());
    assert!(context.related_port_targets.is_empty());
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.step.lookup")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        [UnitId::new("flash-1")].as_slice()
    );
    assert!(error.context().related_stream_ids().is_empty());
}

#[test]
fn solve_failure_context_extracts_cycle_unit_ids_from_wrapped_message() {
    let context = SolveFailureContext::from_message(
        "flowsheet solve failed with package `binary-hydrocarbon-lite-v1`: solver.topological_ordering.two_unit_cycle: solver topological ordering failed: units `heater-1` and `valve-1` form a two-unit cycle through streams `stream-a` and `stream-b`; `valve-1.inlet` and `heater-1.inlet` currently depend on each other in opposite directions",
    );

    assert_eq!(
        context.primary_code.as_deref(),
        Some("solver.topological_ordering.two_unit_cycle")
    );
    assert_eq!(
        context.related_unit_ids,
        vec![UnitId::new("heater-1"), UnitId::new("valve-1")]
    );
    assert_eq!(
        context.related_stream_ids,
        vec![StreamId::new("stream-a"), StreamId::new("stream-b")]
    );
    assert!(context.related_port_targets.is_empty());
}

#[test]
fn solve_failure_context_extracts_stream_ids_from_wrapped_message() {
    let context = SolveFailureContext::from_message(
        "flowsheet solve failed with package `binary-hydrocarbon-lite-v1`: solver.connection_validation.duplicate_downstream_sink: solver connection validation failed: stream `shared-stream` is consumed by both `mixer-1.inlet_a` and `flash-1.inlet`",
    );

    assert_eq!(
        context.primary_code.as_deref(),
        Some("solver.connection_validation.duplicate_downstream_sink")
    );
    assert_eq!(
        context.related_stream_ids,
        vec![StreamId::new("shared-stream")]
    );
}

#[test]
fn cycle_detection_error_carries_related_units_in_error_context() {
    let provider = build_provider();
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &flash_solver,
    };
    let mut flowsheet = Flowsheet::new("heater-valve-cycle");
    for component in [
        Component::new("component-a", "Component A"),
        Component::new("component-b", "Component B"),
    ] {
        flowsheet
            .insert_component(component)
            .expect("expected component insert");
    }
    for stream in [
        build_stream(
            "stream-a",
            "Cycle Stream A",
            320.0,
            100_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ),
        build_stream(
            "stream-b",
            "Cycle Stream B",
            300.0,
            95_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        ),
    ] {
        flowsheet
            .insert_stream(stream)
            .expect("expected stream insert");
    }
    for unit in [
        build_heater_node("heater-1", "Heater", "stream-b", "stream-a"),
        build_valve_node("valve-1", "Valve", "stream-a", "stream-b"),
    ] {
        flowsheet.insert_unit(unit).expect("expected unit insert");
    }

    let error = SequentialModularSolver
        .solve(&services, &flowsheet)
        .expect_err("expected cycle detection failure");

    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.topological_ordering.two_unit_cycle")
    );
    assert_eq!(
        error.context().related_unit_ids(),
        [UnitId::new("heater-1"), UnitId::new("valve-1")].as_slice()
    );
}
