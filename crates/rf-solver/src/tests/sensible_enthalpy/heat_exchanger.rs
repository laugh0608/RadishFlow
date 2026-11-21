use super::*;
use rf_unitops::{
    StreamTarget,
    heat_exchanger::{
        HeatExchangerResult, HeatExchangerSide, HeatExchangerSpecification,
        calculate_countercurrent,
    },
    single_phase_enthalpy_flow_w,
};

struct TwoFeedExchanger {
    model: SensibleEnthalpyModel,
    hot: MaterialStreamState,
    cold: MaterialStreamState,
}

impl TwoFeedExchanger {
    fn from_example() -> Self {
        let contents = include_str!(
            "../../../../../examples/flowsheets/two-feed-single-vapor-sensible.rfproj.json"
        );
        let v4::ProjectV4Read::Editable(project) =
            v4::parse_with_support(contents, v4::FlowPlanSupport::SinglePhase).unwrap()
        else {
            panic!("expected editable single-vapor example")
        };
        let v4::StoredFlowPlan::SinglePhase(settings) = project.steady_state_plan else {
            panic!("expected single-vapor settings")
        };
        let model = SensibleEnthalpyModel::new(
            settings.heat_capacities_j_per_mol_k,
            settings.min_temperature_k..=settings.max_temperature_k,
            settings.pressure_pa,
        )
        .unwrap();
        let streams = &project.document.flowsheet.streams;
        Self {
            model,
            hot: streams[&"b-out".into()].clone(),
            cold: streams[&"a-out".into()].clone(),
        }
    }

    fn solve(&self, specification: HeatExchangerSpecification) -> RfResult<HeatExchangerResult> {
        calculate_countercurrent(
            &self.model,
            HeatExchangerSide {
                inlet: &self.hot,
                outlet: &StreamTarget::new("hx-hot", "Hot outlet"),
            },
            HeatExchangerSide {
                inlet: &self.cold,
                outlet: &StreamTarget::new("hx-cold", "Cold outlet"),
            },
            specification,
        )
    }
}

fn assert_benchmark(case: &TwoFeedExchanger, result: &HeatExchangerResult) {
    for (inlet, side, outlet_temperature, heat_duty, in_w, out_w) in [
        (&case.hot, &result.hot, 340.0, -3000.0, 9277.5, 6277.5),
        (&case.cold, &result.cold, 350.0, 3000.0, 111.0, 3111.0),
    ] {
        assert_close(side.outlet.temperature_k, outlet_temperature, 1e-8);
        assert_eq!(side.outlet.pressure_pa, inlet.pressure_pa);
        assert_eq!(
            side.outlet.total_molar_flow_mol_s,
            inlet.total_molar_flow_mol_s
        );
        assert_eq!(
            side.outlet.overall_mole_fractions,
            inlet.overall_mole_fractions
        );
        assert_close(
            single_phase_enthalpy_flow_w(&side.outlet).unwrap(),
            out_w,
            FLOW_TOLERANCE_W,
        );
        assert!(side.max_component_flow_residual_mol_s <= 1e-10);
        let energy = side.energy_balance;
        assert_close(energy.inlet_enthalpy_flow_w, in_w, FLOW_TOLERANCE_W);
        assert_close(energy.outlet_enthalpy_flow_w, out_w, FLOW_TOLERANCE_W);
        assert_close(energy.heat_duty_w, heat_duty, FLOW_TOLERANCE_W);
        assert_close(
            energy.residual_w,
            (single_phase_enthalpy_flow_w(&side.outlet).unwrap() - in_w) - heat_duty,
            1e-10,
        );
        assert!(energy.residual_w.abs() <= FLOW_TOLERANCE_W);
        assert!(energy.scaled_residual.abs() <= 1e-10);
    }
    assert_close(result.heat_duty_w, 3000.0, FLOW_TOLERANCE_W);
    assert_close(result.hot_inlet_approach_k, 10.0, 1e-8);
    assert_close(result.cold_inlet_approach_k, 40.0, 1e-8);
    assert_eq!(result.pinch_heat_duty_w, 3600.0);
    assert_eq!(result.overall_energy_balance.heat_duty_w, 0.0);
    assert_close(
        result.overall_energy_balance.inlet_enthalpy_flow_w,
        9388.5,
        FLOW_TOLERANCE_W,
    );
    assert_close(
        result.overall_energy_balance.outlet_enthalpy_flow_w,
        9388.5,
        FLOW_TOLERANCE_W,
    );
    assert!(result.overall_energy_balance.residual_w.abs() <= FLOW_TOLERANCE_W);
    assert!(result.overall_energy_balance.scaled_residual.abs() <= 1e-10);
}

#[test]
fn heat_exchanger_specified_duty_matches_the_two_feed_balance() {
    let case = TwoFeedExchanger::from_example();
    let result = case
        .solve(HeatExchangerSpecification::HeatDutyW(3000.0))
        .unwrap();
    assert_benchmark(&case, &result);
}

#[test]
fn heat_exchanger_hot_outlet_specification_recovers_the_forward_result() {
    use HeatExchangerSpecification::{HeatDutyW, HotOutletTemperatureK};
    let case = TwoFeedExchanger::from_example();
    let forward = case.solve(HeatDutyW(3000.0)).unwrap();
    let inverse = case.solve(HotOutletTemperatureK(340.0)).unwrap();
    assert_benchmark(&case, &inverse);
    assert_eq!(inverse, forward);
    assert_eq!(
        case.solve(HotOutletTemperatureK(360.0)).unwrap(),
        case.solve(HeatDutyW(0.0)).unwrap()
    );
    for (temperature, diagnostic) in [
        (336.0, "unit.heat_exchanger.pinch_limit"),
        (335.0, "unit.heat_exchanger.above_pinch"),
        (361.0, "unit.heat_exchanger.reverse_heat_transfer"),
        (279.0, "thermo.sensible_enthalpy.outside_domain"),
        (
            f64::INFINITY,
            "unit.heat_exchanger.non_finite_specification",
        ),
    ] {
        let error = case.solve(HotOutletTemperatureK(temperature)).unwrap_err();
        assert_eq!(error.context().diagnostic_code(), Some(diagnostic));
    }
}

#[test]
fn heat_exchanger_cold_outlet_specification_recovers_the_forward_result() {
    use HeatExchangerSpecification::{ColdOutletTemperatureK, HeatDutyW};
    let case = TwoFeedExchanger::from_example();
    let forward = case.solve(HeatDutyW(3000.0)).unwrap();
    let inverse = case.solve(ColdOutletTemperatureK(350.0)).unwrap();
    assert_benchmark(&case, &inverse);
    assert_eq!(inverse, forward);
    assert_eq!(
        case.solve(ColdOutletTemperatureK(300.0)).unwrap(),
        case.solve(HeatDutyW(0.0)).unwrap()
    );
    for (temperature, diagnostic) in [
        (360.0, "unit.heat_exchanger.pinch_limit"),
        (361.0, "unit.heat_exchanger.above_pinch"),
        (299.0, "unit.heat_exchanger.reverse_heat_transfer"),
        (421.0, "thermo.sensible_enthalpy.outside_domain"),
        (f64::NAN, "unit.heat_exchanger.non_finite_specification"),
    ] {
        let error = case.solve(ColdOutletTemperatureK(temperature)).unwrap_err();
        assert_eq!(error.context().diagnostic_code(), Some(diagnostic));
    }
}

#[test]
fn heat_exchanger_requires_exactly_one_specification() {
    use HeatExchangerSpecification::*;
    assert_eq!(
        HeatExchangerSpecification::from_inputs(Some(3000.0), None, None).unwrap(),
        HeatDutyW(3000.0)
    );
    assert_eq!(
        HeatExchangerSpecification::from_inputs(None, Some(340.0), None).unwrap(),
        HotOutletTemperatureK(340.0)
    );
    assert_eq!(
        HeatExchangerSpecification::from_inputs(None, None, Some(350.0)).unwrap(),
        ColdOutletTemperatureK(350.0)
    );
    assert_eq!(
        HeatExchangerSpecification::from_inputs(None, None, None)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("unit.heat_exchanger.missing_specification")
    );
    for (duty, hot, cold) in [
        (Some(3000.0), Some(340.0), None),
        (Some(3000.0), None, Some(350.0)),
        (None, Some(340.0), Some(350.0)),
        (Some(3000.0), Some(340.0), Some(350.0)),
    ] {
        assert_eq!(
            HeatExchangerSpecification::from_inputs(duty, hot, cold)
                .unwrap_err()
                .context()
                .diagnostic_code(),
            Some("unit.heat_exchanger.conflicting_specifications")
        );
    }
}

#[test]
fn heat_exchanger_enforces_zero_duty_and_the_countercurrent_pinch() {
    let mut case = TwoFeedExchanger::from_example();
    let zero = case
        .solve(HeatExchangerSpecification::HeatDutyW(0.0))
        .unwrap();
    assert_eq!(zero.hot.outlet.temperature_k, 360.0);
    assert_eq!(zero.cold.outlet.temperature_k, 300.0);
    assert_eq!(zero.hot.energy_balance.residual_w, 0.0);
    assert_eq!(zero.cold.energy_balance.residual_w, 0.0);
    assert_eq!(zero.overall_energy_balance.residual_w, 0.0);
    for (duty, code) in [
        (3600.0, "unit.heat_exchanger.pinch_limit"),
        (3600.0_f64.next_up(), "unit.heat_exchanger.above_pinch"),
        (-1.0, "unit.heat_exchanger.reverse_heat_transfer"),
        (f64::NAN, "unit.heat_exchanger.non_finite_specification"),
    ] {
        let error = case
            .solve(HeatExchangerSpecification::HeatDutyW(duty))
            .unwrap_err();
        assert_eq!(error.context().diagnostic_code(), Some(code));
    }
    case.hot.temperature_k = 300.0;
    let equal = case
        .solve(HeatExchangerSpecification::HeatDutyW(0.0))
        .unwrap();
    assert_eq!(equal.hot_inlet_approach_k, 0.0);
    assert_eq!(equal.cold_inlet_approach_k, 0.0);
    assert_eq!(
        equal.hot.outlet.temperature_k,
        equal.cold.outlet.temperature_k
    );
    let error = case
        .solve(HeatExchangerSpecification::HeatDutyW(1.0))
        .unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("unit.heat_exchanger.above_pinch")
    );
    case.hot.temperature_k = 299.0;
    let error = case
        .solve(HeatExchangerSpecification::HeatDutyW(0.0))
        .unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("unit.heat_exchanger.reversed_inlets")
    );
}

#[test]
fn heat_exchanger_rejects_invalid_streams_and_locates_the_inlet() {
    let mut case = TwoFeedExchanger::from_example();
    let spec = HeatExchangerSpecification::HeatDutyW(0.0);
    case.cold.total_molar_flow_mol_s = 0.0;
    let zero = case.solve(spec).unwrap_err();
    assert_eq!(
        zero.context().diagnostic_code(),
        Some("unit.heat_exchanger.invalid_flow")
    );
    assert_eq!(
        zero.context().related_stream_ids(),
        std::slice::from_ref(&case.cold.id)
    );
    case.cold.total_molar_flow_mol_s = 2.0;
    case.hot.pressure_pa = 80_000.0;
    let pressure = case.solve(spec).unwrap_err();
    assert_eq!(
        pressure.context().diagnostic_code(),
        Some("thermo.sensible_enthalpy.outside_domain")
    );
    assert_eq!(
        pressure.context().related_stream_ids(),
        std::slice::from_ref(&case.hot.id)
    );
    case.hot.pressure_pa = PRESSURE_PA;
    case.cold.temperature_k = 279.0;
    let temperature = case.solve(spec).unwrap_err();
    assert_eq!(
        temperature.context().diagnostic_code(),
        Some("thermo.sensible_enthalpy.outside_domain")
    );
    assert_eq!(
        temperature.context().related_stream_ids(),
        std::slice::from_ref(&case.cold.id)
    );
    case.cold.temperature_k = 300.0;
    case.model = SensibleEnthalpyModel::new(
        [("component-a".into(), 30.0)].into(),
        280.0..=420.0,
        PRESSURE_PA,
    )
    .unwrap();
    let missing = case.solve(spec).unwrap_err();
    assert_eq!(
        missing.context().diagnostic_code(),
        Some("thermo.sensible_enthalpy.missing_heat_capacity")
    );
    assert_eq!(
        missing.context().related_component_ids(),
        &["component-b".into()]
    );
    assert_eq!(missing.context().related_stream_ids(), &[case.hot.id]);
}
