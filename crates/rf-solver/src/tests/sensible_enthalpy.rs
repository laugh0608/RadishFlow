use super::{assert_close, binary_composition, build_provider};
use crate::{
    ControlledSolveOutcome, FlowsheetSolver, SequentialModularSolver, SolveSnapshot,
    SolverServices, execution::UncontrolledSolve,
};
use rf_flash::PlaceholderTpFlashSolver;
use rf_model::{Component, Flowsheet, MaterialStreamState};
use rf_store::{
    FileOverwritePolicy, StoredDocumentMetadata, StoredProjectDocument, project_v4 as v4,
};
use rf_thermo::SensibleEnthalpyModel;
use rf_types::{PhaseLabel, RfResult, UnitId};
use rf_unitops::{
    BuiltinUnitKind, build_cooler_node, build_feed_node, build_heater_node, build_mixer_node,
    build_splitter_node,
};
use std::collections::BTreeMap;

mod project_v4;
mod recycle;

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

fn heater_cooler_case(kind: BuiltinUnitKind, temperature_k: Option<f64>) -> Flowsheet {
    let mut flow = two_feed_mixer_case();
    flow.insert_stream(MaterialStreamState::from_tpzf(
        "product",
        "product",
        310.0,
        PRESSURE_PA,
        0.0,
        binary_composition(0.5, 0.5),
    ))
    .unwrap();
    let mut unit = match kind {
        BuiltinUnitKind::Heater => {
            build_heater_node(kind.as_str(), kind.as_str(), "mixed", "product")
        }
        BuiltinUnitKind::Cooler => {
            build_cooler_node(kind.as_str(), kind.as_str(), "mixed", "product")
        }
        _ => panic!("expected Heater/Cooler"),
    };
    unit.parameters.outlet_temperature_k = temperature_k;
    unit.parameters.outlet_pressure_pa = Some(PRESSURE_PA);
    flow.insert_unit(unit).unwrap();
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

fn solve_with_duty(
    flow: &Flowsheet,
    model: &SensibleEnthalpyModel,
    unit_id: &str,
    duty_w: f64,
) -> RfResult<SolveSnapshot> {
    let heat_duties_w = BTreeMap::from([(UnitId::from(unit_id), duty_w)]);
    match SequentialModularSolver.solve_single_phase_with_heat_duties_controlled(
        model,
        flow,
        &heat_duties_w,
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
fn single_phase_splitter_distributes_flows_and_preserves_intensive_state() {
    let mut flow = two_feed_mixer_case();
    for id in ["branch-a", "branch-b"] {
        let mut template = flow.stream(&"mixed".into()).unwrap().clone();
        template.id = id.into();
        flow.insert_stream(template).unwrap();
    }
    let splitter = build_splitter_node("split", "split", "mixed", "branch-a", "branch-b");
    let splitter_id = splitter.id.clone();
    flow.insert_unit(splitter).unwrap();
    let model = model();
    for fraction in [0.0, 0.6, 1.0] {
        flow.units
            .get_mut(&splitter_id)
            .unwrap()
            .parameters
            .outlet_a_split_fraction = Some(fraction);
        let result = solve(&flow, &model).unwrap();
        let inlet = &result.streams[&"mixed".into()];
        for (id, share) in [("branch-a", fraction), ("branch-b", 1.0 - fraction)] {
            let output = &result.streams[&id.into()];
            assert_eq!(output.total_molar_flow_mol_s, share * 5.0);
            assert_eq!(output.temperature_k, inlet.temperature_k);
            assert_eq!(output.pressure_pa, inlet.pressure_pa);
            assert_eq!(output.overall_mole_fractions, inlet.overall_mole_fractions);
            assert_eq!(output.bubble_dew_window, None);
            if share > 0.0 {
                assert_eq!(output.phases, inlet.phases);
                assert_close(enthalpy_flow(output), share * 9388.5, FLOW_TOLERANCE_W);
            } else {
                assert!(output.phases.is_empty());
            }
        }
        let balance = result.steps.last().unwrap().energy_balance.unwrap();
        assert_eq!(balance.heat_duty_w, 0.0);
        assert_close(balance.inlet_enthalpy_flow_w, 9388.5, FLOW_TOLERANCE_W);
        assert_close(balance.outlet_enthalpy_flow_w, 9388.5, FLOW_TOLERANCE_W);
        assert!(balance.residual_w.abs() <= FLOW_TOLERANCE_W);
    }
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

fn assert_heater_cooler_energy_balance(snapshot: &SolveSnapshot, expected_heat_w: f64) {
    let step = snapshot.steps.last().unwrap();
    let balance = step.energy_balance.unwrap();
    let inlet_w = enthalpy_flow(&step.consumed_streams[0]);
    let outlet_w = enthalpy_flow(&step.produced_streams[0]);
    let expected_residual_w = (outlet_w - inlet_w) - balance.heat_duty_w;
    let expected_scale_w = (inlet_w.abs() + outlet_w.abs() + balance.heat_duty_w.abs()).max(1.0);
    assert_eq!(balance.inlet_enthalpy_flow_w, inlet_w);
    assert_eq!(balance.outlet_enthalpy_flow_w, outlet_w);
    assert_eq!(balance.residual_w, expected_residual_w);
    assert_eq!(balance.residual_scale_w, expected_scale_w);
    assert_eq!(
        balance.scaled_residual,
        expected_residual_w / expected_scale_w
    );
    assert!(balance.residual_w.abs() <= FLOW_TOLERANCE_W);
    assert!(balance.scaled_residual.abs() <= 1e-10);
    assert_close(balance.heat_duty_w, expected_heat_w, FLOW_TOLERANCE_W);
    assert_close(outlet_w, 9388.5 + expected_heat_w, FLOW_TOLERANCE_W);

    let feeds_w = enthalpy_flow(&snapshot.streams[&"a-out".into()])
        + enthalpy_flow(&snapshot.streams[&"b-out".into()]);
    assert_close(outlet_w - feeds_w, expected_heat_w, FLOW_TOLERANCE_W);
}

#[test]
fn specified_outlet_temperature_gives_signed_heat_duty_and_preserves_legacy_operation() {
    let model = model();
    let provider = build_provider();
    let legacy = SolverServices {
        thermo: &provider,
        flash_solver: &PlaceholderTpFlashSolver,
    };
    for (kind, temperature_k, expected_heat_w) in [
        (BuiltinUnitKind::Heater, 380.0, 7800.0),
        (BuiltinUnitKind::Cooler, 320.0, -4800.0),
    ] {
        let flow = heater_cooler_case(kind, Some(temperature_k));
        let original = flow.clone();
        let old_result = SequentialModularSolver.solve(&legacy, &flow).unwrap();
        let result = solve(&flow, &model).unwrap();
        let product = &result.streams[&"product".into()];
        assert_eq!(product.temperature_k, temperature_k);
        assert_eq!(product.pressure_pa, PRESSURE_PA);
        assert_eq!(product.total_molar_flow_mol_s, 5.0);
        assert_eq!(product.overall_mole_fractions, binary_composition(0.4, 0.6));
        assert!(product.bubble_dew_window.is_none());
        assert_heater_cooler_energy_balance(&result, expected_heat_w);

        let old_product = &old_result.streams[&"product".into()];
        assert_eq!(old_result.streams[&"mixed".into()].temperature_k, 336.0);
        assert_eq!(old_product.temperature_k, temperature_k);
        assert_eq!(old_product.pressure_pa, PRESSURE_PA);
        assert_eq!(old_product.total_molar_flow_mol_s, 5.0);
        assert!(old_product.bubble_dew_window.is_some());
        assert!(
            old_result
                .steps
                .iter()
                .all(|step| step.energy_balance.is_none())
        );
        assert_eq!(flow, original);
        assert_eq!(
            SequentialModularSolver.solve(&legacy, &flow).unwrap(),
            old_result
        );
    }
}

#[test]
fn thermal_unit_requires_a_heat_specification_and_checks_temperature_domain() {
    let model = model();
    for (temperature_k, expected_diagnostic) in [
        (None, "solver.single_phase.missing_heat_specification"),
        (Some(421.0), "thermo.sensible_enthalpy.outside_domain"),
        (Some(279.0), "thermo.sensible_enthalpy.outside_domain"),
    ] {
        let flow = heater_cooler_case(BuiltinUnitKind::Heater, temperature_k);
        let error = solve(&flow, &model).unwrap_err();
        assert_eq!(error.context().diagnostic_code(), Some(expected_diagnostic));
        assert_eq!(error.context().related_unit_ids(), &["heater".into()]);
        assert_eq!(error.context().related_stream_ids(), &["product".into()]);
    }
}

#[test]
fn specified_heat_duty_recovers_forward_temperature_and_energy_balance() {
    let model = model();
    for (kind, target_k, expected_duty_w) in [
        (BuiltinUnitKind::Heater, 380.0, 7800.0),
        (BuiltinUnitKind::Cooler, 320.0, -4800.0),
        (BuiltinUnitKind::Heater, 420.0, 16200.0),
        (BuiltinUnitKind::Cooler, 280.0, -13200.0),
    ] {
        let mut flow = heater_cooler_case(kind, Some(target_k));
        let forward = solve(&flow, &model).unwrap();
        assert_heater_cooler_energy_balance(&forward, expected_duty_w);
        let specified_duty_w = forward
            .steps
            .last()
            .unwrap()
            .energy_balance
            .unwrap()
            .heat_duty_w;
        flow.units
            .get_mut(&kind.as_str().into())
            .unwrap()
            .parameters
            .outlet_temperature_k = None;
        let original = flow.clone();
        let inverse = solve_with_duty(&flow, &model, kind.as_str(), specified_duty_w).unwrap();
        assert_eq!(flow, original);
        assert_heater_cooler_energy_balance(&inverse, expected_duty_w);
        let product_id = "product".into();
        let product = &inverse.streams[&product_id];
        let forward_product = &forward.streams[&product_id];
        assert_close(product.temperature_k, target_k, 1e-8);
        assert_eq!(product.pressure_pa, forward_product.pressure_pa);
        assert_eq!(
            product.total_molar_flow_mol_s,
            forward_product.total_molar_flow_mol_s
        );
        assert_eq!(
            product.overall_mole_fractions,
            forward_product.overall_mole_fractions
        );
        assert_eq!(
            inverse
                .steps
                .last()
                .unwrap()
                .energy_balance
                .unwrap()
                .heat_duty_w,
            specified_duty_w
        );
    }
}

#[test]
fn heat_duty_domain_handles_rounding_at_the_temperature_endpoint() {
    let model = SensibleEnthalpyModel::new(
        [("component-a".into(), 30.3), ("component-b".into(), 42.0)].into(),
        130.0..=2050.0,
        PRESSURE_PA,
    )
    .unwrap();
    let mut flow = heater_cooler_case(BuiltinUnitKind::Cooler, Some(130.0));
    for (id, temperature_k, flow_mol_s) in [("a-out", 1500.1, 2.1), ("b-out", 1800.11, 3.1)] {
        let feed = flow.streams.get_mut(&id.into()).unwrap();
        feed.temperature_k = temperature_k;
        feed.total_molar_flow_mol_s = flow_mol_s;
    }
    let forward = solve(&flow, &model).unwrap();
    let duty_w = forward
        .steps
        .last()
        .unwrap()
        .energy_balance
        .unwrap()
        .heat_duty_w;
    flow.units
        .get_mut(&"cooler".into())
        .unwrap()
        .parameters
        .outlet_temperature_k = None;
    let inverse = solve_with_duty(&flow, &model, "cooler", duty_w).unwrap();
    assert_close(
        inverse.streams[&"product".into()].temperature_k,
        130.0,
        1e-8,
    );
    let error = solve_with_duty(&flow, &model, "cooler", duty_w.next_down()).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("thermo.sensible_enthalpy.outside_domain")
    );
}

#[test]
fn single_phase_heater_cooler_checks_effective_outlet_pressure() {
    let model = model();
    let mut flow = heater_cooler_case(BuiltinUnitKind::Heater, None);
    let heater_id = "heater".into();
    let product_id = "product".into();
    flow.streams.get_mut(&product_id).unwrap().pressure_pa = PRESSURE_PA - 1.0;
    flow.units
        .get_mut(&heater_id)
        .unwrap()
        .parameters
        .outlet_pressure_pa = None;

    let result = solve_with_duty(&flow, &model, "heater", 7800.0).unwrap();
    assert_eq!(result.streams[&product_id].pressure_pa, PRESSURE_PA);
    assert_heater_cooler_energy_balance(&result, 7800.0);

    flow.units
        .get_mut(&heater_id)
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(PRESSURE_PA - 1.0);
    let error = solve_with_duty(&flow, &model, "heater", 7800.0).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("thermo.sensible_enthalpy.outside_domain")
    );
    assert_eq!(error.context().related_unit_ids(), &[heater_id]);
    assert_eq!(error.context().related_stream_ids(), &[product_id]);
}

#[test]
fn zero_heat_duty_preserves_inlet_state_and_requires_positive_flow() {
    let kind = BuiltinUnitKind::Heater;
    let model = model();
    let mut flow = heater_cooler_case(kind, None);
    let result = solve_with_duty(&flow, &model, kind.as_str(), 0.0).unwrap();
    let inlet = &result.streams[&"mixed".into()];
    let product = &result.streams[&"product".into()];
    assert_eq!(product.temperature_k, inlet.temperature_k);
    assert_eq!(product.phases, inlet.phases);
    assert_heater_cooler_energy_balance(&result, 0.0);

    flow.units.remove(&"mixer".into()).unwrap();
    flow.streams.remove(&"mixed".into()).unwrap();
    let heater = flow.units.get_mut(&kind.as_str().into()).unwrap();
    heater
        .ports
        .iter_mut()
        .find(|port| port.name == "inlet")
        .unwrap()
        .stream_id = Some("a-out".into());
    flow.streams
        .get_mut(&"a-out".into())
        .unwrap()
        .total_molar_flow_mol_s = 0.0;
    for duty_w in [0.0, 7800.0] {
        let error = solve_with_duty(&flow, &model, kind.as_str(), duty_w).unwrap_err();
        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.single_phase.heat_duty_requires_flow")
        );
        assert_eq!(error.context().related_unit_ids(), &["heater".into()]);
        assert_eq!(
            error.context().related_stream_ids(),
            &["a-out".into(), "product".into()]
        );
    }
}

#[test]
fn specified_heat_duty_rejects_out_of_domain_nonfinite_and_conflicting_inputs() {
    let kind = BuiltinUnitKind::Heater;
    let model = model();
    for (temperature_k, duty_w, diagnostic) in [
        (None, 16201.0, "thermo.sensible_enthalpy.outside_domain"),
        (None, -13201.0, "thermo.sensible_enthalpy.outside_domain"),
        (None, f64::NAN, "solver.single_phase.non_finite_heat_duty"),
        (
            Some(380.0),
            7800.0,
            "solver.single_phase.conflicting_heat_specifications",
        ),
    ] {
        let flow = heater_cooler_case(kind, temperature_k);
        let error = solve_with_duty(&flow, &model, kind.as_str(), duty_w).unwrap_err();
        assert_eq!(error.context().diagnostic_code(), Some(diagnostic));
        assert_eq!(error.context().related_unit_ids(), &["heater".into()]);
        assert_eq!(error.context().related_stream_ids(), &["product".into()]);
    }
}

#[test]
fn specified_heat_duty_rejects_unknown_units_and_other_unit_kinds() {
    let model = model();
    let flow = heater_cooler_case(BuiltinUnitKind::Heater, Some(380.0));
    for (id, diagnostic) in [
        ("missing", "solver.single_phase.unknown_heat_duty_target"),
        ("mixer", "solver.single_phase.unsupported_heat_duty_target"),
    ] {
        let error = solve_with_duty(&flow, &model, id, 0.0).unwrap_err();
        assert_eq!(error.context().diagnostic_code(), Some(diagnostic));
        assert_eq!(error.context().related_unit_ids(), &[id.into()]);
    }
}
