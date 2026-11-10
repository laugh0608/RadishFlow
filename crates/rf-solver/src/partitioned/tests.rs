use super::*;
use crate::test_fixtures::binary_hydrocarbon_provider;
use rf_flash::PlaceholderTpFlashSolver;
use rf_model::{Component, UnitOperationParameters};
use rf_unitops::{build_feed_node, build_heater_node, build_mixer_node, build_splitter_node};

mod boundaries;
mod faults;
mod graph;

fn composition(a: f64, b: f64) -> BTreeMap<ComponentId, f64> {
    [("methane".into(), a), ("ethane".into(), b)].into()
}
fn stream(id: &str, flow: f64, a: f64, b: f64) -> MaterialStreamState {
    MaterialStreamState::from_tpzf(id, id, 300.0, 650_000.0, flow, composition(a, b))
}
fn tear(stream_id: &str, splitter: &str, mixer: &str) -> TearSpecification {
    TearSpecification {
        stream_id: stream_id.into(),
        source: DiagnosticPortTarget::new(splitter, "outlet_a"),
        sink: DiagnosticPortTarget::new(mixer, "inlet_b"),
        initial_component_flows_mol_s: None,
        component_scales_mol_s: None,
    }
}
pub(super) fn case(fraction: f64) -> (Flowsheet, RecycleSolveOptions) {
    let mut flow = Flowsheet::new("QS1 fixed-boundary recycle");
    flow.set_property_package_id(Some("binary-hydrocarbon-lite-v1".into()))
        .unwrap();
    for id in ["methane", "ethane"] {
        flow.insert_component(Component::new(id, id)).unwrap();
    }
    for (id, f, a, b) in [
        ("s-fa", 2.0, 0.8, 0.2),
        ("s-fb", 3.0, 0.2, 0.8),
        // Deliberately wrong outlet templates: local outputs and explicit guesses
        // must be the only trial state source.
        ("s-fresh", 97.0, 0.9, 0.1),
        ("s-circulation", 83.0, 0.7, 0.3),
        ("s-recycle", 79.0, 0.3, 0.7),
        ("s-bleed", 91.0, 0.1, 0.9),
        ("s-product", 101.0, 0.6, 0.4),
    ] {
        flow.insert_stream(stream(id, f, a, b)).unwrap();
    }
    for mut unit in [
        build_feed_node("fa", "FA", "s-fa"),
        build_feed_node("fb", "FB", "s-fb"),
        build_mixer_node("m0", "M0", "s-fa", "s-fb", "s-fresh"),
        build_mixer_node("m1", "M1", "s-fresh", "s-recycle", "s-circulation"),
        build_splitter_node("s1", "S1", "s-circulation", "s-recycle", "s-bleed"),
        build_heater_node("h1", "H1", "s-bleed", "s-product"),
    ] {
        if unit.kind == "mixer" {
            unit.parameters.outlet_pressure_pa = Some(650_000.0);
        }
        if unit.kind == "splitter" {
            unit.parameters.outlet_a_split_fraction = Some(fraction);
        }
        if unit.kind == "heater" {
            unit.parameters = UnitOperationParameters {
                outlet_temperature_k: Some(300.0),
                outlet_pressure_pa: Some(650_000.0),
                ..Default::default()
            };
        }
        flow.insert_unit(unit).unwrap();
    }
    let options = RecycleSolveOptions::new(
        FixedRecycleBoundary {
            temperature_k: 300.0,
            pressure_pa: 650_000.0,
        },
        vec![tear("s-recycle", "s1", "m1")],
    );
    (flow, options)
}
fn solve(
    flow: &Flowsheet,
    options: &RecycleSolveOptions,
) -> Result<PartitionedSolveResult, Box<PartitionedSolveFailure>> {
    PartitionedSequentialSolver.solve(
        &SolverServices {
            thermo: &binary_hydrocarbon_provider(),
            flash_solver: &PlaceholderTpFlashSolver,
        },
        flow,
        options,
    )
}
fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}, tolerance {tolerance}"
    );
}
fn assert_flows(result: &PartitionedSolveResult, id: &str, a: f64, b: f64, tolerance: f64) {
    let state = &result.streams[&StreamId::new(id)];
    close(
        state.total_molar_flow_mol_s * state.overall_mole_fractions[&ComponentId::new("methane")],
        a,
        tolerance,
    );
    close(
        state.total_molar_flow_mol_s * state.overall_mole_fractions[&ComponentId::new("ethane")],
        b,
        tolerance,
    );
    close(state.temperature_k, 300.0, TEMPERATURE_TOLERANCE_K);
    close(state.pressure_pa, 650_000.0, PRESSURE_TOLERANCE_PA);
    if a + b > 0.0 {
        close(
            state.overall_mole_fractions[&ComponentId::new("methane")],
            a / (a + b),
            1e-10,
        );
    }
}
#[test]
fn real_local_recycle_matches_independent_analytic_solution_and_actual_call_counts() {
    let (flow, options) = case(0.6);
    let frozen = flow.clone();
    let result = solve(&flow, &options).unwrap();
    assert_flows(&result, "s-fresh", 2.2, 2.8, 1e-9);
    assert_flows(&result, "s-circulation", 5.5, 7.0, 1e-9);
    assert_flows(&result, "s-recycle", 3.3, 4.2, 1e-9);
    assert_flows(&result, "s-product", 2.2, 2.8, 1e-9);
    let report = result
        .blocks
        .iter()
        .find_map(|b| b.recycle.as_ref())
        .unwrap();
    assert_eq!(report.iteration_evaluations, 48);
    assert_eq!(report.verification_evaluations, 1);
    assert_eq!(report.residual_trace.len(), 49);
    assert_eq!(
        report.residual_trace.last().unwrap().phase,
        EvaluationPhase::Verification
    );
    assert!(report.component_scales_mol_s.values().all(|s| *s == 5.0));
    assert!(checks::converged(report.residual_trace.last().unwrap()));
    assert_eq!(result.costs.local_unit_calls, 102);
    for id in ["fa", "fb", "m0", "h1"] {
        assert_eq!(result.costs.calls_by_unit[&UnitId::new(id)], 1);
    }
    for id in ["m1", "s1"] {
        assert_eq!(result.costs.calls_by_unit[&UnitId::new(id)], 49);
    }
    assert!(result.costs.thermo_calls > 0);
    assert!(result.costs.flash_calls > 0);
    let block = &result.blocks[3];
    let consumed = block.final_units[0]
        .consumed_streams
        .iter()
        .find(|s| s.id == StreamId::new("s-recycle"))
        .unwrap();
    assert_eq!(consumed, &report.assumed);
    assert_eq!(result.streams[&StreamId::new("s-recycle")], report.returned);
    assert_ne!(report.assumed, report.returned); // Do not forge exact closure.
    let repeated = solve(&flow, &options).unwrap();
    assert_eq!(repeated.streams, result.streams);
    assert_eq!(repeated.blocks, result.blocks);
    assert_eq!(repeated.costs.calls_by_unit, result.costs.calls_by_unit);
    assert_eq!(flow, frozen);
}
#[test]
fn nonzero_guess_changes_path_but_not_the_accepted_solution() {
    let (flow, mut options) = case(0.6);
    options.tears[0].initial_component_flows_mol_s = Some(composition(10.0, 1.0));
    let result = solve(&flow, &options).unwrap();
    assert_flows(&result, "s-recycle", 3.3, 4.2, 1e-9);
    assert_flows(&result, "s-product", 2.2, 2.8, 1e-9);
    assert_eq!(result.costs.iteration_evaluations, 49);
    assert_eq!(result.costs.verification_evaluations, 1);
}
#[test]
fn high_recycle_uses_frozen_tolerances_and_fails_when_budget_is_insufficient() {
    for (fraction, iterations, tolerance) in [(0.9, 229, 1e-9), (0.95, 469, 2e-9)] {
        let (flow, options) = case(fraction);
        let result = solve(&flow, &options).unwrap();
        let ratio = fraction / (1.0 - fraction);
        assert_flows(&result, "s-recycle", 2.2 * ratio, 2.8 * ratio, tolerance);
        assert_eq!(result.costs.iteration_evaluations, iterations);
        assert_eq!(result.costs.verification_evaluations, 1);
    }
    let (flow, options) = case(0.99);
    let failure = solve(&flow, &options).unwrap_err();
    assert_eq!(failure.kind, PartitionedFailureKind::IterationLimit);
    assert_eq!(failure.costs.iteration_evaluations, 500);
    assert_eq!(failure.costs.verification_evaluations, 0);
    assert!(!failure.costs.calls_by_unit.contains_key(&UnitId::new("h1")));
    assert_eq!(
        failure
            .active_recycle
            .as_ref()
            .unwrap()
            .residual_trace
            .len(),
        500
    );
    assert_eq!(failure.completed_blocks.len(), 3);
}
fn serial_case(second_fraction: f64) -> (Flowsheet, RecycleSolveOptions) {
    let (mut flow, mut options) = case(0.6);
    flow.units
        .get_mut(&UnitId::new("h1"))
        .unwrap()
        .ports
        .iter_mut()
        .find(|p| p.name == "inlet")
        .unwrap()
        .stream_id = Some("s-product-2".into());
    for id in ["s-circulation-2", "s-recycle-2", "s-product-2"] {
        flow.insert_stream(stream(id, 13.0, 0.1, 0.9)).unwrap();
    }
    let mut mixer = build_mixer_node("m2", "M2", "s-bleed", "s-recycle-2", "s-circulation-2");
    mixer.parameters.outlet_pressure_pa = Some(650_000.0);
    let mut splitter =
        build_splitter_node("s2", "S2", "s-circulation-2", "s-recycle-2", "s-product-2");
    splitter.parameters.outlet_a_split_fraction = Some(second_fraction);
    flow.insert_unit(mixer).unwrap();
    flow.insert_unit(splitter).unwrap();
    options.tears.push(tear("s-recycle-2", "s2", "m2"));
    (flow, options)
}
#[test]
fn serial_cyclic_blocks_resolve_in_dependency_order_and_publish_together() {
    let (flow, options) = serial_case(0.25);
    let result = solve(&flow, &options).unwrap();
    assert_eq!(result.blocks.len(), 6);
    assert_eq!(
        result.blocks[3].members,
        vec![UnitId::new("m1"), UnitId::new("s1")]
    );
    assert_eq!(
        result.blocks[4].members,
        vec![UnitId::new("m2"), UnitId::new("s2")]
    );
    assert_flows(&result, "s-recycle-2", 11.0 / 15.0, 14.0 / 15.0, 1e-9);
    assert_flows(&result, "s-circulation-2", 44.0 / 15.0, 56.0 / 15.0, 1e-9);
    assert_flows(&result, "s-product", 2.2, 2.8, 1e-9);
    let (bad, options) = serial_case(0.99);
    let failed = solve(&bad, &options).unwrap_err();
    assert_eq!(failed.kind, PartitionedFailureKind::IterationLimit);
    assert_eq!(failed.completed_blocks.len(), 4);
    assert!(!failed.costs.calls_by_unit.contains_key(&UnitId::new("h1")));
    assert_eq!(
        failed.active_recycle.unwrap().tear.stream_id,
        StreamId::new("s-recycle-2")
    );
}
