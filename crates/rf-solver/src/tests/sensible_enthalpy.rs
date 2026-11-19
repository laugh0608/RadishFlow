use super::{assert_close, binary_composition, build_provider};
use crate::{
    ControlledSolveOutcome, FlowsheetSolver, SequentialModularSolver, SolveSnapshot,
    SolverServices, execution::UncontrolledSolve,
};
use rf_flash::PlaceholderTpFlashSolver;
use rf_model::{Component, Flowsheet, MaterialStreamState};
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{PhaseLabel, RfResult};
use rf_unitops::{build_feed_node, build_mixer_node};

const PRESSURE_PA: f64 = 100_000.0;
const FLOW_TOLERANCE_W: f64 = 1e-6;

fn model() -> SensibleEnthalpyModel {
    SensibleEnthalpyModel::new(
        [("component-a".into(), 30.0), ("component-b".into(), 50.0)].into(),
        280.0..=420.0,
        PRESSURE_PA,
    )
    .unwrap()
}

fn two_feed_case() -> Flowsheet {
    let mut flow = Flowsheet::new("Two-feed sensible enthalpy");
    for id in ["component-a", "component-b"] {
        flow.insert_component(Component::new(id, id)).unwrap();
    }
    for (unit, stream, temperature, rate, fraction_a) in [
        ("feed-a", "a-out", 300.0, 2.0, 1.0),
        ("feed-b", "b-out", 360.0, 3.0, 0.0),
    ] {
        flow.insert_stream(MaterialStreamState::from_tpzf(
            stream,
            stream,
            temperature,
            PRESSURE_PA,
            rate,
            binary_composition(fraction_a, 1.0 - fraction_a),
        ))
        .unwrap();
        flow.insert_unit(build_feed_node(unit, unit, stream))
            .unwrap();
    }
    flow
}

fn two_feed_mixer_case() -> Flowsheet {
    let mut flow = two_feed_case();
    flow.insert_stream(MaterialStreamState::from_tpzf(
        "mixed",
        "mixed",
        310.0,
        PRESSURE_PA,
        0.0,
        binary_composition(0.5, 0.5),
    ))
    .unwrap();
    let mut mixer = build_mixer_node("mixer", "mixer", "a-out", "b-out", "mixed");
    mixer.parameters.outlet_pressure_pa = Some(PRESSURE_PA);
    flow.insert_unit(mixer).unwrap();
    flow
}

fn solve(flow: &Flowsheet, model: &SensibleEnthalpyModel) -> RfResult<SolveSnapshot> {
    match SequentialModularSolver.solve_single_phase_controlled(
        model,
        flow,
        &mut UncontrolledSolve,
    )? {
        ControlledSolveOutcome::Completed(snapshot) => Ok(snapshot),
        ControlledSolveOutcome::Cancelled => panic!("uncontrolled solve cannot cancel"),
    }
}

fn enthalpy_flow(stream: &MaterialStreamState) -> f64 {
    let phase = stream
        .phases
        .iter()
        .find(|phase| phase.label == PhaseLabel::Overall)
        .unwrap();
    phase.molar_enthalpy_j_per_mol.unwrap() * stream.total_molar_flow_mol_s
}

#[test]
fn single_phase_feeds_materialize_independent_enthalpy_flows_and_preserve_legacy_execution() {
    let flow = two_feed_case();
    let input = flow.clone();
    let provider = build_provider();
    let legacy = SolverServices {
        thermo: &provider,
        flash_solver: &PlaceholderTpFlashSolver,
    };
    let old_result = SequentialModularSolver.solve(&legacy, &flow).unwrap();

    assert_eq!(SensibleEnthalpyModel::REFERENCE_TEMPERATURE_K, 298.15);
    let result = solve(&flow, &model()).unwrap();
    for (id, expected_h, expected_rate, legacy_rate) in [
        ("a-out", 55.5, 111.0, 135.05),
        ("b-out", 3092.5, 9277.5, 9648.6),
    ] {
        let id = id.into();
        let stream = &result.streams[&id];
        assert_close(enthalpy_flow(stream), expected_rate, FLOW_TOLERANCE_W);
        assert_eq!(stream.temperature_k, input.streams[&id].temperature_k);
        assert_eq!(
            stream.overall_mole_fractions,
            input.streams[&id].overall_mole_fractions
        );
        assert!(stream.bubble_dew_window.is_none());
        assert_eq!(
            stream.phases.iter().map(|p| p.label).collect::<Vec<_>>(),
            [PhaseLabel::Overall, PhaseLabel::Vapor]
        );
        for phase in &stream.phases {
            assert_eq!(phase.phase_fraction, 1.0);
            assert_close(phase.molar_enthalpy_j_per_mol.unwrap(), expected_h, 1e-8);
        }
        assert!(old_result.streams[&id].bubble_dew_window.is_some());
        assert_close(
            enthalpy_flow(&old_result.streams[&id]),
            legacy_rate,
            FLOW_TOLERANCE_W,
        );
    }
    assert_eq!(flow, input);
    assert_eq!(
        SequentialModularSolver.solve(&legacy, &flow).unwrap(),
        old_result
    );
}

#[test]
fn single_phase_feed_reports_missing_heat_capacity_by_component_and_outlet() {
    let missing_b = SensibleEnthalpyModel::new(
        [("component-a".into(), 30.0)].into(),
        280.0..=420.0,
        PRESSURE_PA,
    )
    .unwrap();
    let error = solve(&two_feed_case(), &missing_b).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("thermo.sensible_enthalpy.missing_heat_capacity")
    );
    assert_eq!(
        error.context().related_component_ids(),
        &["component-b".into()]
    );
    assert_eq!(error.context().related_unit_ids(), &["feed-b".into()]);
    assert_eq!(error.context().related_stream_ids(), &["b-out".into()]);
}

#[test]
fn single_phase_feed_checks_the_effective_temperature_and_pressure() {
    let mut flow = two_feed_case();
    for (temperature, pressure) in [
        (279.0, PRESSURE_PA),
        (421.0, PRESSURE_PA),
        (360.0, PRESSURE_PA + 1.0),
    ] {
        let parameters = &mut flow.units.get_mut(&"feed-b".into()).unwrap().parameters;
        parameters.outlet_temperature_k = Some(temperature);
        parameters.outlet_pressure_pa = Some(pressure);
        let error = solve(&flow, &model()).unwrap_err();
        assert_eq!(
            error.context().diagnostic_code(),
            Some("thermo.sensible_enthalpy.outside_domain")
        );
        assert_eq!(error.context().related_stream_ids(), &["b-out".into()]);
    }
    for (temperature, expected_rate) in [(280.0, -2722.5), (420.0, 18277.5)] {
        let parameters = &mut flow.units.get_mut(&"feed-b".into()).unwrap().parameters;
        parameters.outlet_temperature_k = Some(temperature);
        parameters.outlet_pressure_pa = Some(PRESSURE_PA);
        let result = solve(&flow, &model()).unwrap();
        assert_close(
            enthalpy_flow(&result.streams[&"b-out".into()]),
            expected_rate,
            FLOW_TOLERANCE_W,
        );
    }
}

#[test]
fn adiabatic_mixer_matches_independent_temperature_and_conserves_enthalpy_and_components() {
    let flow = two_feed_mixer_case();
    let original = flow.clone();
    let snapshot = solve(&flow, &model()).unwrap();
    let step = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id.as_str() == "mixer")
        .unwrap();
    let outlet = &step.produced_streams[0];
    assert_eq!(outlet, &snapshot.streams[&"mixed".into()]);
    assert_close(outlet.temperature_k, 2400.0 / 7.0, 1e-8);
    assert_eq!(outlet.total_molar_flow_mol_s, 5.0);
    assert_eq!(outlet.pressure_pa, PRESSURE_PA);
    assert_close(
        outlet.overall_mole_fractions[&"component-a".into()] * 5.0,
        2.0,
        1e-10,
    );
    assert_close(
        outlet.overall_mole_fractions[&"component-b".into()] * 5.0,
        3.0,
        1e-10,
    );
    let outlet_w = enthalpy_flow(outlet);
    assert_close(outlet_w, 9388.5, FLOW_TOLERANCE_W);
    let inlet_w: f64 = step.consumed_streams.iter().map(enthalpy_flow).sum();
    let residual_w = (outlet_w - inlet_w).abs();
    assert!(residual_w <= FLOW_TOLERANCE_W);
    assert!(residual_w / (outlet_w.abs() + inlet_w.abs()) <= 1e-10);
    assert!(outlet.bubble_dew_window.is_none());
    assert_eq!(flow, original);

    let provider = build_provider();
    let legacy = SolverServices {
        thermo: &provider,
        flash_solver: &PlaceholderTpFlashSolver,
    };
    let old_result = SequentialModularSolver.solve(&legacy, &flow).unwrap();
    assert_eq!(old_result.streams[&"mixed".into()].temperature_k, 336.0);
    assert!(
        old_result.streams[&"mixed".into()]
            .bubble_dew_window
            .is_some()
    );
}

#[test]
fn adiabatic_mixer_accepts_single_active_feed_and_rejects_zero_total_flow_or_pressure_change() {
    let mut flow = two_feed_mixer_case();
    flow.streams
        .get_mut(&"a-out".into())
        .unwrap()
        .total_molar_flow_mol_s = 0.0;
    let snapshot = solve(&flow, &model()).unwrap();
    let outlet = &snapshot.streams[&"mixed".into()];
    assert_eq!(outlet.temperature_k, 360.0);
    assert_eq!(outlet.total_molar_flow_mol_s, 3.0);
    assert_close(enthalpy_flow(outlet), 9277.5, FLOW_TOLERANCE_W);

    flow.units
        .get_mut(&"mixer".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(PRESSURE_PA - 1.0);
    let error = solve(&flow, &model()).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("thermo.sensible_enthalpy.outside_domain")
    );
    assert_eq!(error.context().related_stream_ids(), &["mixed".into()]);

    flow.streams
        .get_mut(&"b-out".into())
        .unwrap()
        .total_molar_flow_mol_s = 0.0;
    let error = solve(&flow, &model()).unwrap_err();
    assert!(error.message().contains("positive finite value"));
    assert_eq!(error.context().related_unit_ids(), &["mixer".into()]);
}

#[test]
fn adiabatic_mixer_accepts_isothermal_feeds_at_both_domain_endpoints() {
    let mut flow = two_feed_mixer_case();
    for temperature in [280.0, 420.0] {
        for id in ["a-out", "b-out"] {
            flow.streams.get_mut(&id.into()).unwrap().temperature_k = temperature;
        }
        let result = solve(&flow, &model()).unwrap();
        assert_close(
            result.streams[&"mixed".into()].temperature_k,
            temperature,
            1e-8,
        );
    }
}
