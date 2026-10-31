use super::*;
use crate::{FlowsheetSolver, SequentialModularSolver, SolverServices};
use rf_flash::PlaceholderTpFlashSolver;
use rf_model::{Component, Flowsheet, MaterialStreamState};
use rf_types::{ComponentId, units::QuantityKind};
use rf_unitops::{
    build_feed_node, build_mixer_node, component_balance::ComponentBalanceCapability,
};

fn fixture() -> (Flowsheet, MixerBalanceSpecification) {
    let mut flow = Flowsheet::new("QS0 fixed-feed mixer");
    for id in ["component-a", "component-b"] {
        flow.insert_component(Component::new(id, id)).unwrap();
    }
    for (id, total, a, b) in [
        ("a", 2.0, 0.8, 0.2),
        ("b", 3.0, 0.2, 0.8),
        ("out", 0.0, 0.0, 0.0),
    ] {
        flow.insert_stream(MaterialStreamState::from_tpzf(
            id,
            id,
            300.0,
            100_000.0,
            total,
            [
                (ComponentId::new("component-a"), a),
                (ComponentId::new("component-b"), b),
            ]
            .into_iter()
            .collect(),
        ))
        .unwrap();
    }
    for unit in [
        build_feed_node("feed-a", "Feed A", "a"),
        build_feed_node("feed-b", "Feed B", "b"),
        build_mixer_node("mix", "Mixer", "a", "b", "out"),
    ] {
        flow.insert_unit(unit).unwrap();
    }
    let spec = MixerBalanceSpecification::new(
        "mix",
        MixerBalanceScales {
            inlet_a_mol_s: 2.0,
            inlet_b_mol_s: 3.0,
            outlet_mol_s: 5.0,
        },
    );
    (flow, spec)
}

const EXACT: [f64; 6] = [1.6, 0.4, 0.6, 2.4, 2.2, 2.8];

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} != {expected}"
    );
}

#[test]
fn assembly_preserves_domain_bindings_units_and_independent_arithmetic() {
    let (flow, spec) = fixture();
    let before = flow.clone();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    assert_eq!(flow, before);
    assert_eq!(problem.variables().len(), 6);
    assert_eq!(problem.equations().len(), 6);
    let i = problem
        .variable_index(MixerBalancePort::InletB, &"component-a".into())
        .unwrap();
    assert_eq!(i, 2);
    let variable = &problem.variables()[i];
    assert_eq!(variable.quantity(), QuantityKind::MolarFlow);
    assert_eq!(variable.binding.unit_id.as_str(), "mix");
    assert_eq!(variable.binding.stream_id.as_str(), "b");
    assert_eq!(variable.scale_mol_s, 3.0);
    let ComponentBalanceEquation::Connection { source, target } = &problem.equations()[i].equation
    else {
        panic!("connection row")
    };
    assert_eq!(source.unit_id.as_str(), "feed-b");
    assert_eq!(source.port, ComponentBalancePort::FeedOutlet);
    assert_eq!(target, &variable.binding);
    assert!(
        problem
            .equations()
            .iter()
            .all(|r| r.quantity() == QuantityKind::MolarFlow)
    );
    let values = problem.check_solution(&EXACT).unwrap();
    close(values.total_molar_flow_mol_s, 5.0, 1e-10);
    close(
        values.component_molar_flows_mol_s[&"component-a".into()],
        2.2,
        1e-10,
    );
    close(
        values.component_molar_flows_mol_s[&"component-b".into()],
        2.8,
        1e-10,
    );
    close(values.mole_fractions[&"component-a".into()], 0.44, 1e-12);
    close(values.mole_fractions[&"component-b".into()], 0.56, 1e-12);
    assert_eq!(flow, before);
}

#[test]
fn frozen_problem_and_repeated_trial_evaluation_never_touch_source_or_each_other() {
    let (mut flow, mut spec) = fixture();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    let expected = problem.evaluate_si(&EXACT).unwrap();
    let initial = [1.0; 6];
    let trial = problem.evaluate_si(&initial).unwrap();
    assert!(trial.scaled_residuals.iter().any(|v| v.abs() > 0.1));
    flow.streams
        .get_mut(&"a".into())
        .unwrap()
        .total_molar_flow_mol_s = 99.0;
    spec.scales.outlet_mol_s = 999.0;
    let changed = flow.clone();
    for _ in 0..20 {
        assert_eq!(problem.evaluate_si(&EXACT).unwrap(), expected);
        assert_eq!(problem.evaluate_si(&initial).unwrap(), trial);
    }
    assert!(matches!(
        problem.check_solution(&initial),
        Err(ComponentBalanceError::UnsatisfiedEquation { .. })
    ));
    assert_eq!(initial, [1.0; 6]);
    assert_eq!(flow, changed);
}

#[test]
fn analytic_and_scaled_jacobians_match_independent_perturbations_at_multiple_steps() {
    let (flow, spec) = fixture();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    let point = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    for step in [1e-4, 1e-5, 1e-6] {
        for column in 0..6 {
            let mut plus = point;
            let mut minus = point;
            plus[column] += step;
            minus[column] -= step;
            let upper = problem.evaluate_si(&plus).unwrap();
            let lower = problem.evaluate_si(&minus).unwrap();
            let scaled_upper = problem.evaluate_scaled(&plus).unwrap();
            let scaled_lower = problem.evaluate_scaled(&minus).unwrap();
            for row in 0..6 {
                close(
                    (upper.residuals_mol_s[row] - lower.residuals_mol_s[row]) / (2.0 * step),
                    problem.jacobian_si()[row][column],
                    1e-8,
                );
                close(
                    (scaled_upper.scaled_residuals[row] - scaled_lower.scaled_residuals[row])
                        / (2.0 * step),
                    problem.scaled_jacobian()[row][column],
                    1e-8,
                );
            }
        }
    }
    let normalized = EXACT
        .iter()
        .zip(problem.variables())
        .map(|(x, v)| x / v.scale_mol_s)
        .collect::<Vec<_>>();
    let evaluation = problem.evaluate_scaled(&normalized).unwrap();
    assert!(evaluation.scaled_residuals.iter().all(|r| r.abs() <= 1e-14));
    assert_eq!(point, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn material_values_match_existing_sm_with_its_real_thermo_services() {
    let (flow, spec) = fixture();
    let before = flow.clone();
    let provider = crate::tests::build_provider();
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &PlaceholderTpFlashSolver,
    };
    let snapshot = SequentialModularSolver.solve(&services, &flow).unwrap();
    let values = MixerComponentBalanceProblem::assemble(&flow, &spec)
        .unwrap()
        .check_solution(&EXACT)
        .unwrap();
    let actual = snapshot.stream(&"out".into()).unwrap();
    close(
        actual.total_molar_flow_mol_s,
        values.total_molar_flow_mol_s,
        1e-10,
    );
    for (component, fraction) in &values.mole_fractions {
        close(actual.overall_mole_fractions[component], *fraction, 1e-12);
    }
    assert_eq!(flow, before);
}

#[test]
fn missing_connections_reuse_structured_graph_diagnostics() {
    let (mut flow, spec) = fixture();
    flow.units.get_mut(&"mix".into()).unwrap().ports[0].stream_id = None;
    let error = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap_err();
    let ComponentBalanceError::Model(error) = error else {
        panic!("canonical graph error")
    };
    assert_eq!(
        error.context().diagnostic_code(),
        Some("flowsheet.connection_validation.unbound_inlet_port")
    );
    assert!(
        error
            .context()
            .related_port_targets()
            .iter()
            .any(|p| p.unit_id.as_str() == "mix" && p.port_name == "inlet_a")
    );
}

#[test]
fn redundant_and_conflicting_outlet_specs_are_distinct_and_not_silently_dropped() {
    let (flow, mut spec) = fixture();
    spec.outlet_total_molar_flow_mol_s = Some(5.0);
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::RedundantOutletSpecification {
            expected_mol_s: 5.0,
            ..
        })
    ));
    spec.outlet_total_molar_flow_mol_s = Some(6.0);
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::ConflictingOutletSpecification {
            expected_mol_s: 5.0,
            specified_mol_s: 6.0,
            ..
        })
    ));
    for bad in [f64::NAN, f64::INFINITY, -1.0] {
        spec.outlet_total_molar_flow_mol_s = Some(bad);
        assert!(matches!(
            MixerComponentBalanceProblem::assemble(&flow, &spec),
            Err(ComponentBalanceError::InvalidOutletSpecification)
        ));
    }
}

#[test]
fn supported_capabilities_are_material_only_and_other_models_and_topologies_refuse() {
    use rf_unitops::{BuiltinUnitKind, catalog::builtin_model};
    assert_eq!(
        builtin_model(BuiltinUnitKind::Feed).component_balance_capability(),
        Some(ComponentBalanceCapability::FixedFeedBoundary)
    );
    assert_eq!(
        builtin_model(BuiltinUnitKind::Mixer).component_balance_capability(),
        Some(ComponentBalanceCapability::TwoInletMixing)
    );
    for kind in [
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::FlashDrum,
    ] {
        assert_eq!(builtin_model(kind).component_balance_capability(), None);
        let (mut flow, spec) = fixture();
        flow.units.get_mut(&"feed-a".into()).unwrap().kind = kind.as_str().into();
        assert!(matches!(
            MixerComponentBalanceProblem::assemble(&flow, &spec),
            Err(ComponentBalanceError::UnsupportedModel { .. })
        ));
    }
    let (mut flow, spec) = fixture();
    flow.insert_unit(build_feed_node("extra", "Extra", "a"))
        .unwrap();
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::UnsupportedTopology { .. })
    ));
}

#[test]
fn thermal_or_unconsumed_parameter_specs_refuse_without_claiming_full_mixer_equivalence() {
    let (mut flow, spec) = fixture();
    flow.streams.get_mut(&"b".into()).unwrap().temperature_k = 301.0;
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::UnsupportedThermalBoundary { .. })
    ));
    let (mut flow, spec) = fixture();
    flow.units
        .get_mut(&"mix".into())
        .unwrap()
        .parameters
        .outlet_temperature_k = Some(300.0);
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::UnsupportedSpecification { .. })
    ));
    let (mut flow, spec) = fixture();
    flow.units
        .get_mut(&"mix".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(90_000.0);
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::UnsupportedSpecification { .. })
    ));
    flow.units
        .get_mut(&"mix".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(100_000.0);
    assert!(MixerComponentBalanceProblem::assemble(&flow, &spec).is_ok());
}

#[test]
fn unknown_components_empty_catalog_and_invalid_boundary_values_refuse() {
    let (mut flow, spec) = fixture();
    flow.components.clear();
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::NoComponents)
    ));
    let (mut flow, spec) = fixture();
    flow.streams
        .get_mut(&"a".into())
        .unwrap()
        .overall_mole_fractions
        .insert("alien".into(), 1.0);
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::UnknownComponent { .. })
    ));
    for bad in [-1.0, f64::NAN, f64::INFINITY] {
        let (mut flow, spec) = fixture();
        flow.streams
            .get_mut(&"a".into())
            .unwrap()
            .total_molar_flow_mol_s = bad;
        let ComponentBalanceError::Model(error) =
            MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap_err()
        else {
            panic!("boundary validation")
        };
        assert_eq!(error.context().related_stream_ids(), &["a".into()]);
        assert!(
            error
                .context()
                .related_port_targets()
                .iter()
                .any(|p| p.port_name == "inlet_a")
        );
    }
    for bad in [-1.0, f64::NAN, f64::INFINITY] {
        let (mut flow, spec) = fixture();
        flow.streams
            .get_mut(&"a".into())
            .unwrap()
            .overall_mole_fractions
            .insert("component-a".into(), bad);
        assert!(matches!(
            MixerComponentBalanceProblem::assemble(&flow, &spec),
            Err(ComponentBalanceError::Model(_))
        ));
    }
    let (mut flow, spec) = fixture();
    for stream in flow.streams.values_mut() {
        stream.total_molar_flow_mol_s = 0.0;
    }
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::InvalidOutletTotal { .. })
    ));
}

#[test]
fn registered_component_cardinality_defines_the_problem_and_sparse_absence_is_zero() {
    let (mut flow, spec) = fixture();
    flow.insert_component(Component::new("component-c", "Inactive component"))
        .unwrap();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    assert_eq!(problem.variables().len(), 9);
    assert_eq!(problem.equations().len(), 9);
    let values = problem
        .check_solution(&[1.6, 0.4, 0.0, 0.6, 2.4, 0.0, 2.2, 2.8, 0.0])
        .unwrap();
    assert_eq!(
        values.component_molar_flows_mol_s[&"component-c".into()],
        0.0
    );
    assert_eq!(
        problem.variable_index(MixerBalancePort::Outlet, &"component-c".into()),
        Some(8)
    );
    assert_eq!(
        problem.variable_index(MixerBalancePort::Outlet, &"absent".into()),
        None
    );
}

#[test]
fn local_sm_weight_normalization_and_sparse_zero_components_are_preserved() {
    let (mut flow, spec) = fixture();
    let a = flow.streams.get_mut(&"a".into()).unwrap();
    a.overall_mole_fractions = [("component-a".into(), 8.0), ("component-b".into(), 2.0)]
        .into_iter()
        .collect();
    let b = flow.streams.get_mut(&"b".into()).unwrap();
    b.overall_mole_fractions = [("component-b".into(), 10.0)].into_iter().collect();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    let values = problem
        .check_solution(&[1.6, 0.4, 0.0, 3.0, 1.6, 3.4])
        .unwrap();
    close(values.mole_fractions[&"component-a".into()], 0.32, 1e-12);
    close(values.mole_fractions[&"component-b".into()], 0.68, 1e-12);
    flow.streams
        .get_mut(&"a".into())
        .unwrap()
        .total_molar_flow_mol_s = 0.0;
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    assert!(
        problem
            .check_solution(&[0.0, 0.0, 0.0, 3.0, 0.0, 3.0])
            .is_ok()
    );
}

#[test]
fn invalid_trial_vectors_and_overflow_have_typed_domain_targets() {
    let (flow, spec) = fixture();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    assert!(matches!(
        problem.evaluate_si(&EXACT[..5]),
        Err(ComponentBalanceError::PointSize {
            expected: 6,
            actual: 5
        })
    ));
    assert!(matches!(
        problem.evaluate_scaled(&[]),
        Err(ComponentBalanceError::PointSize { .. })
    ));
    for bad in [-1.0, f64::NAN, f64::INFINITY] {
        let mut point = EXACT;
        point[2] = bad;
        let ComponentBalanceError::InvalidTrialValue { binding, .. } =
            problem.evaluate_si(&point).unwrap_err()
        else {
            panic!("invalid component flow")
        };
        assert_eq!(binding.stream_id.as_str(), "b");
        assert_eq!(binding.component_id.as_str(), "component-a");
    }
    assert!(matches!(
        problem.evaluate_scaled(&[f64::MAX; 6]),
        Err(ComponentBalanceError::InvalidTrialValue { .. })
    ));
    assert!(matches!(
        problem.evaluate_si(&[f64::MAX, 0.0, f64::MAX, 0.0, 0.0, 0.0]),
        Err(ComponentBalanceError::NonFiniteEquation { .. })
    ));
    assert!(matches!(
        problem.check_solution(&[0.0; 6]),
        Err(ComponentBalanceError::UnsatisfiedEquation { .. })
    ));
    assert!(problem.check_solution(&EXACT).is_ok());
}

#[test]
fn invalid_scales_and_nonfinite_scaled_derivatives_or_residuals_refuse() {
    let (flow, mut spec) = fixture();
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        spec.scales.inlet_a_mol_s = bad;
        assert!(matches!(
            MixerComponentBalanceProblem::assemble(&flow, &spec),
            Err(ComponentBalanceError::InvalidScale {
                port: MixerBalancePort::InletA
            })
        ));
    }
    spec.scales.inlet_a_mol_s = f64::MAX;
    spec.scales.outlet_mol_s = f64::MIN_POSITIVE;
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::NonFiniteScaling { .. })
    ));
    spec.scales = MixerBalanceScales {
        inlet_a_mol_s: f64::MIN_POSITIVE,
        inlet_b_mol_s: f64::MIN_POSITIVE,
        outlet_mol_s: f64::MIN_POSITIVE,
    };
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    assert!(matches!(
        problem.evaluate_si(&[f64::MAX; 6]),
        Err(ComponentBalanceError::NonFiniteScaling { .. })
    ));
}

#[test]
fn large_scales_cannot_hide_specification_conflicts_or_a_wrong_material_solution() {
    let (flow, mut spec) = fixture();
    spec.scales = MixerBalanceScales {
        inlet_a_mol_s: 1e20,
        inlet_b_mol_s: 1e20,
        outlet_mol_s: 1e20,
    };
    spec.outlet_total_molar_flow_mol_s = Some(6.0);
    assert!(matches!(
        MixerComponentBalanceProblem::assemble(&flow, &spec),
        Err(ComponentBalanceError::ConflictingOutletSpecification { .. })
    ));
    spec.outlet_total_molar_flow_mol_s = None;
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    let trial = [1.0; 6];
    assert!(
        problem
            .evaluate_si(&trial)
            .unwrap()
            .scaled_residuals
            .iter()
            .all(|v| v.abs() < COMPONENT_BALANCE_TOLERANCE)
    );
    assert!(matches!(
        problem.check_solution(&trial),
        Err(ComponentBalanceError::UnsatisfiedEquation { .. })
    ));
    assert!(problem.check_solution(&EXACT).is_ok());
}

#[test]
fn port_binding_controls_boundary_values_and_outlet_templates_are_not_specifications() {
    let (mut flow, spec) = fixture();
    let mixer = flow.units.get_mut(&"mix".into()).unwrap();
    mixer.ports[0].stream_id = Some("b".into());
    mixer.ports[1].stream_id = Some("a".into());
    let outlet = flow.streams.get_mut(&"out".into()).unwrap();
    outlet.total_molar_flow_mol_s = 12345.0;
    outlet.overall_mole_fractions.clear();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    let ComponentBalanceEquation::Connection { source, target } = &problem.equations()[0].equation
    else {
        panic!("inlet connection")
    };
    assert_eq!(source.unit_id.as_str(), "feed-b");
    assert_eq!(target.stream_id.as_str(), "b");
    let values = problem
        .check_solution(&[0.6, 2.4, 1.6, 0.4, 2.2, 2.8])
        .unwrap();
    close(values.total_molar_flow_mol_s, 5.0, 1e-10);
    close(
        flow.streams[&"out".into()].total_molar_flow_mol_s,
        12345.0,
        0.0,
    );
}
