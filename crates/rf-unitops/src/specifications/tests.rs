use super::*;

#[test]
fn six_builtins_declare_their_actual_authoring_contracts() {
    use BuiltinUnitKind::*;
    use OutletPressureRule::*;
    use UnitParameter::*;
    for (kind, fields, required, rule) in [
        (
            Feed,
            vec![OutletTemperature, OutletPressure],
            vec![],
            Unrestricted,
        ),
        (
            Heater,
            vec![OutletTemperature, OutletPressure],
            vec![OutletTemperature, OutletPressure],
            AtMostInlet,
        ),
        (
            Cooler,
            vec![OutletTemperature, OutletPressure],
            vec![OutletTemperature, OutletPressure],
            AtMostInlet,
        ),
        (
            FlashDrum,
            vec![OutletTemperature, OutletPressure],
            vec![OutletTemperature, OutletPressure],
            Unrestricted,
        ),
        (
            Mixer,
            vec![OutletPressure],
            vec![OutletPressure],
            AtMostLowestInlet,
        ),
        (
            Valve,
            vec![OutletPressure],
            vec![OutletPressure],
            AtMostInlet,
        ),
    ] {
        let spec = input_specification(kind);
        assert_eq!(spec.parameters, fields, "{kind:?}");
        assert_eq!(spec.required_explicit_parameters, required, "{kind:?}");
        assert_eq!(spec.outlet_pressure_rule, rule, "{kind:?}");
    }
}

#[test]
fn scalar_parameters_require_positive_finite_values_without_temperature_direction_rules() {
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::FlashDrum,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::Valve,
    ] {
        for parameter in [
            UnitParameter::OutletTemperature,
            UnitParameter::OutletPressure,
        ] {
            if parameter == UnitParameter::OutletTemperature
                && matches!(kind, BuiltinUnitKind::Mixer | BuiltinUnitKind::Valve)
            {
                assert_eq!(
                    check_parameter_value(kind, parameter, 300.0, None),
                    Err(ParameterViolation::Unsupported)
                );
                continue;
            }
            for value in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert_eq!(
                    check_parameter_value(kind, parameter, value, None),
                    Err(ParameterViolation::NotPositiveFinite)
                );
            }
            for value in [0.001, 285.0, 345.0, 100_000.0] {
                assert_eq!(check_parameter_value(kind, parameter, value, None), Ok(()));
            }
        }
    }
}

fn inlet(port: &'static str, stream: &str, pressure_pa: f64) -> InletPressure {
    InletPressure {
        port,
        stream_id: stream.into(),
        pressure_pa,
    }
}

#[test]
fn pressure_equality_is_allowed_and_flash_does_not_inherit_an_inlet_upper_bound() {
    let limit = InletPressureLimit {
        pressure_pa: 100_000.0,
        limiting_stream_id: "feed".into(),
    };
    for kind in [
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::Mixer,
    ] {
        for pressure in [90_000.0, 100_000.0] {
            assert_eq!(
                check_parameter_value(kind, UnitParameter::OutletPressure, pressure, Some(&limit)),
                Ok(())
            );
        }
        assert_eq!(
            check_parameter_value(kind, UnitParameter::OutletPressure, 100_001.0, Some(&limit)),
            Err(ParameterViolation::AboveInletPressure)
        );
    }
    assert_eq!(
        check_parameter_value(
            BuiltinUnitKind::FlashDrum,
            UnitParameter::OutletPressure,
            150_000.0,
            Some(&limit)
        ),
        Ok(())
    );
}

#[test]
fn mixer_needs_both_valid_inlets_and_tracks_the_minimum_deterministically() {
    let rule = OutletPressureRule::AtMostLowestInlet;
    let mut inlets = vec![inlet("inlet_a", "a", 150_000.0)];
    assert_eq!(inlet_pressure_limit(rule, &inlets), None);
    inlets.push(inlet("inlet_b", "b", 120_000.0));
    let limit = inlet_pressure_limit(rule, &inlets).unwrap();
    assert_eq!(limit.pressure_pa, 120_000.0);
    assert_eq!(limit.limiting_stream_id.as_str(), "b");
    inlets[0].pressure_pa = 120_000.0;
    assert_eq!(
        inlet_pressure_limit(rule, &inlets)
            .unwrap()
            .limiting_stream_id
            .as_str(),
        "a"
    );
    inlets[0].pressure_pa = 110_000.0;
    assert_eq!(
        inlet_pressure_limit(rule, &inlets).unwrap().pressure_pa,
        110_000.0
    );
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        inlets[1].pressure_pa = invalid;
        assert_eq!(inlet_pressure_limit(rule, &inlets), None);
    }
    inlets[1].pressure_pa = 120_000.0;
    inlets.push(inlet("inlet_a", "duplicate", 80_000.0));
    assert_eq!(inlet_pressure_limit(rule, &inlets), None);
}

#[test]
fn feed_override_is_effective_even_when_invalid_and_never_replaces_flow() {
    let stream =
        MaterialStreamState::from_tpzf("s", "Feed", 300.0, 100_000.0, 2.0, Composition::new());
    let mut parameters = UnitOperationParameters::default();
    assert_eq!(
        effective_feed_input(&parameters, &stream, FeedInputField::Temperature),
        FeedInputValue {
            value: 300.0,
            source: FeedInputSource::StreamInput
        }
    );
    parameters.outlet_temperature_k = Some(350.0);
    parameters.outlet_pressure_pa = Some(-1.0);
    assert_eq!(
        effective_feed_input(&parameters, &stream, FeedInputField::Temperature),
        FeedInputValue {
            value: 350.0,
            source: FeedInputSource::UnitParameter(UnitParameter::OutletTemperature)
        }
    );
    assert_eq!(
        effective_feed_input(&parameters, &stream, FeedInputField::Pressure),
        FeedInputValue {
            value: -1.0,
            source: FeedInputSource::UnitParameter(UnitParameter::OutletPressure)
        }
    );
    assert_eq!(
        effective_feed_input(&parameters, &stream, FeedInputField::MolarFlow).value,
        2.0
    );
    assert_eq!(stream.pressure_pa, 100_000.0);
}

fn composition(a: f64, b: f64) -> Composition {
    [("a".into(), a), ("b".into(), b)].into()
}

#[test]
fn composition_distinguishes_bad_fractions_totals_and_explicit_normalization() {
    let policy = CompositionPolicy::SpecifiedMoleFractions;
    assert_eq!(
        check_composition(&Composition::new(), policy),
        CompositionCheck::Empty
    );
    for invalid in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
        assert_eq!(
            check_composition(&composition(invalid, 0.5), policy),
            CompositionCheck::InvalidFractions(vec!["a".into()])
        );
    }
    assert_eq!(
        check_composition(&composition(0.0, 0.0), policy),
        CompositionCheck::InvalidTotal { sum: 0.0 }
    );
    assert_eq!(
        check_composition(&composition(0.2, 0.3), policy),
        CompositionCheck::Unnormalized { sum: 0.5 }
    );
    assert!(matches!(
        check_composition(&composition(0.5, 0.5 + 0.5e-9), policy),
        CompositionCheck::Valid { .. }
    ));
    assert!(matches!(
        check_composition(&composition(0.5, 0.5 + 2e-9), policy),
        CompositionCheck::Unnormalized { .. }
    ));
    assert_eq!(
        check_composition(&composition(0.0, 1.0), policy),
        CompositionCheck::Valid { sum: 1.0 }
    );
}

#[test]
fn execution_weights_keep_existing_normalization_and_overflow_contract() {
    let policy = CompositionPolicy::NonnegativeWeights;
    let weights = composition(2.0, 3.0);
    assert_eq!(
        check_composition(&weights, policy),
        CompositionCheck::Valid { sum: 5.0 }
    );
    let stream = MaterialStreamState::from_tpzf("s", "weights", 300.0, 100_000.0, 0.0, weights);
    assert_eq!(
        crate::normalized_composition(&stream).unwrap(),
        composition(0.4, 0.6)
    );
    assert_eq!(crate::validated_total_flow(&stream).unwrap(), 0.0);
    assert!(matches!(
        check_composition(&composition(f64::MAX, f64::MAX), policy),
        CompositionCheck::InvalidTotal { .. }
    ));
}
