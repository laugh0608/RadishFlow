use super::*;
use rf_types::ComponentId;

fn inlet() -> MaterialStreamState {
    let mut stream = MaterialStreamState::from_tpzf(
        "in",
        "Fresh",
        300.0,
        650_000.0,
        5.0,
        [(ComponentId::new("a"), 0.44), (ComponentId::new("b"), 0.56)].into(),
    );
    let mut phase = PhaseState::new(
        PhaseLabel::Overall,
        1.0,
        stream.overall_mole_fractions.clone(),
    );
    phase.molar_enthalpy_j_per_mol = Some(123.0);
    stream.phases.push(phase);
    stream.bubble_dew_window = Some(rf_model::BubbleDewWindow::new(
        rf_types::PhaseEquilibriumRegion::TwoPhase,
        700_000.0,
        600_000.0,
        290.0,
        310.0,
    ));
    stream
}
fn operation(fraction: f64) -> Splitter {
    Splitter::new(
        StreamTarget::new("a", "Return"),
        StreamTarget::new("b", "Product"),
        fraction,
    )
}
#[test]
fn split_conserves_components_and_preserves_intensive_state_at_both_endpoints() {
    for fraction in [0.0, 0.6, 1.0] {
        for flow in [0.0, 5.0] {
            let mut source = inlet();
            source.total_molar_flow_mol_s = flow;
            let inputs = UnitOperationInputs::new().with_material_stream("inlet", source.clone());
            let result = operation(fraction)
                .run(&UnitOperationServices::default(), &inputs)
                .unwrap();
            let a = result.stream("outlet_a").unwrap();
            let b = result.stream("outlet_b").unwrap();
            assert_eq!(a.id, StreamId::new("a"));
            assert_eq!(b.name, "Product");
            for output in [a, b] {
                assert_eq!(output.temperature_k, source.temperature_k);
                assert_eq!(output.pressure_pa, source.pressure_pa);
                assert_eq!(output.overall_mole_fractions, source.overall_mole_fractions);
                assert_eq!(output.phases, source.phases);
                assert_eq!(output.bubble_dew_window, source.bubble_dew_window);
            }
            for (id, z) in &source.overall_mole_fractions {
                let returned = a.total_molar_flow_mol_s * a.overall_mole_fractions[id]
                    + b.total_molar_flow_mol_s * b.overall_mole_fractions[id];
                assert!((returned - flow * z).abs() <= 1e-14);
            }
            assert_eq!(a.total_molar_flow_mol_s, flow * fraction);
            assert_eq!(b.total_molar_flow_mol_s, flow * (1.0 - fraction));
            assert_eq!(inputs.require_stream("inlet").unwrap(), &source);
        }
    }
}
#[test]
fn missing_intensities_are_not_filled_from_services_or_templates() {
    let mut source = inlet();
    source.phases.clear();
    let result = operation(0.6)
        .run(
            &UnitOperationServices::default(),
            &UnitOperationInputs::new().with_material_stream("inlet", source),
        )
        .unwrap();
    assert!(result.stream("outlet_a").unwrap().phases.is_empty());
}
#[test]
fn bad_fractions_and_inlet_states_fail_without_defaults() {
    for fraction in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        assert!(
            operation(fraction)
                .run(
                    &UnitOperationServices::default(),
                    &UnitOperationInputs::new().with_material_stream("inlet", inlet())
                )
                .is_err()
        );
    }
    for field in 0..4 {
        let mut source = inlet();
        match field {
            0 => source.temperature_k = 0.0,
            1 => source.pressure_pa = f64::NAN,
            2 => source.total_molar_flow_mol_s = -1.0,
            _ => {
                source.overall_mole_fractions.insert("a".into(), 0.2);
            }
        }
        assert!(
            operation(0.6)
                .run(
                    &UnitOperationServices::default(),
                    &UnitOperationInputs::new().with_material_stream("inlet", source)
                )
                .is_err()
        );
    }
    let node = build_splitter_node("s", "Splitter", "in", "a", "b");
    let error = crate::catalog::instantiate_builtin(&node, &BTreeMap::new())
        .err()
        .unwrap();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("unitops.splitter.missing_fraction")
    );
    assert_eq!(
        crate::specifications::UnitParameter::OutletASplitFraction.quantity_kind(),
        rf_types::units::QuantityKind::DimensionlessRatio
    );
}
