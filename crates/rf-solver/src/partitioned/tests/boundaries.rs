use super::*;
use crate::component_balance::MixerBalanceScales;
use rf_thermo::{PlaceholderThermoProvider, ThermoProvider};
#[test]
fn zero_split_keeps_structural_cycle_and_explicit_zero_flow_composition_rule() {
    let (flow, options) = case(0.0);
    let plan = assemble_partitioned_plan(&flow, &options).unwrap();
    assert_eq!(plan.blocks[3].kind, BlockKind::Recycle);
    let result = solve(&flow, &options).unwrap();
    let report = result.blocks[3].recycle.as_ref().unwrap();
    assert_eq!(report.iteration_evaluations, 1);
    assert_eq!(report.verification_evaluations, 1);
    assert_eq!(report.assumed.total_molar_flow_mol_s, 0.0);
    assert_eq!(
        report.assumed.overall_mole_fractions,
        result.streams[&StreamId::new("s-fresh")].overall_mole_fractions
    );
    assert_flows(&result, "s-product", 2.2, 2.8, 1e-9);
}
#[test]
fn one_zero_feed_and_registered_absent_component_preserve_dimension() {
    let (mut flow, mut options) = case(0.6);
    flow.streams
        .get_mut(&StreamId::new("s-fa"))
        .unwrap()
        .total_molar_flow_mol_s = 0.0;
    let result = solve(&flow, &options).unwrap();
    assert_flows(&result, "s-product", 0.6, 2.4, 1e-9);
    let mut system = binary_hydrocarbon_provider().system().clone();
    let mut third = system.components[0].clone();
    third.id = "unused".into();
    system.components.push(third);
    flow.insert_component(Component::new("unused", "Unused"))
        .unwrap();
    let provider = PlaceholderThermoProvider::new(system);
    // Defaults expand to the frozen registry; explicit vectors cannot omit it.
    let result = PartitionedSequentialSolver
        .solve(
            &SolverServices {
                thermo: &provider,
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &options,
        )
        .unwrap();
    let report = result.blocks[3].recycle.as_ref().unwrap();
    assert_eq!(report.residuals_mol_s.len(), 3);
    assert_eq!(report.residuals_mol_s[&ComponentId::new("unused")], 0.0);
    options.tears[0].initial_component_flows_mol_s = Some(composition(0.0, 0.0));
    assert!(assemble_partitioned_plan(&flow, &options).is_err());
}
#[test]
fn full_recycle_and_nonpositive_fresh_flow_are_explicit_failures() {
    let (flow, options) = case(1.0);
    let error = solve(&flow, &options).unwrap_err();
    assert_eq!(error.kind, PartitionedFailureKind::NoFiniteSteadyState);
    assert_eq!(error.costs.iteration_evaluations, 0);
    assert!(!error.costs.calls_by_unit.contains_key(&UnitId::new("h1")));
    let (mut flow, options) = case(0.6);
    for id in ["s-fa", "s-fb"] {
        flow.streams
            .get_mut(&StreamId::new(id))
            .unwrap()
            .total_molar_flow_mol_s = 0.0;
    }
    let error = solve(&flow, &options).unwrap_err();
    assert_eq!(error.kind, PartitionedFailureKind::Initialization);
    assert_eq!(error.costs.local_unit_calls, 0);
}
#[test]
fn invalid_component_guesses_and_scales_do_not_start_local_models() {
    for value in [-1.0, f64::NAN, f64::INFINITY] {
        let (flow, mut options) = case(0.6);
        options.tears[0].initial_component_flows_mol_s = Some(composition(value, 0.0));
        let error = solve(&flow, &options).unwrap_err();
        assert_eq!(error.kind, PartitionedFailureKind::Assembly);
        assert_eq!(
            error.error.context().related_component_ids(),
            &[ComponentId::new("methane")]
        );
        assert_eq!(error.costs.local_unit_calls, 0);
    }
    for value in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        let (flow, mut options) = case(0.6);
        options.tears[0].component_scales_mol_s = Some(composition(value, 5.0));
        assert_eq!(
            solve(&flow, &options).unwrap_err().kind,
            PartitionedFailureKind::Assembly
        );
    }
    let (flow, mut options) = case(0.6);
    options.tears[0].initial_component_flows_mol_s = Some(composition(f64::MAX, f64::MAX));
    let error = solve(&flow, &options).unwrap_err();
    assert_eq!(
        error.error.context().diagnostic_code(),
        Some("solver.partitioned.flow_overflow")
    );
}
#[test]
fn fixed_boundary_rejects_temperature_pressure_and_unsupported_specifications() {
    for (temperature, pressure) in [
        (301.0, 650_000.0),
        (300.0, 650_001.0),
        (f64::NAN, 650_000.0),
    ] {
        let (mut flow, options) = case(0.6);
        let stream = flow.streams.get_mut(&StreamId::new("s-fa")).unwrap();
        stream.temperature_k = temperature;
        stream.pressure_pa = pressure;
        assert!(solve(&flow, &options).is_err());
    }
    let (mut flow, options) = case(0.6);
    flow.units
        .get_mut(&UnitId::new("s1"))
        .unwrap()
        .parameters
        .outlet_temperature_k = Some(300.0);
    assert_eq!(
        solve(&flow, &options).unwrap_err().kind,
        PartitionedFailureKind::Assembly
    );
    for budget in [0, 501] {
        let (flow, mut options) = case(0.6);
        options.max_iteration_evaluations = budget;
        assert!(assemble_partitioned_plan(&flow, &options).is_err());
    }
}
#[test]
fn small_explicit_scales_enforce_the_scaled_test_in_addition_to_raw_defect() {
    let (flow, mut options) = case(0.6);
    options.tears[0].component_scales_mol_s = Some(composition(0.01, 0.01));
    let result = solve(&flow, &options).unwrap();
    assert!(result.costs.iteration_evaluations > 48);
    let report = result.blocks[3].recycle.as_ref().unwrap();
    assert!(checks::converged(report.residual_trace.last().unwrap()));
    assert!(report.residuals_mol_s.values().all(|v| v.abs() <= 1e-12));
}

#[test]
fn invalid_or_missing_physical_fraction_is_never_defaulted() {
    for fraction in [
        None,
        Some(-0.1),
        Some(1.1),
        Some(f64::NAN),
        Some(f64::INFINITY),
    ] {
        let (mut flow, options) = case(0.6);
        flow.units
            .get_mut(&UnitId::new("s1"))
            .unwrap()
            .parameters
            .outlet_a_split_fraction = fraction;
        let failure = solve(&flow, &options).unwrap_err();
        assert_eq!(failure.kind, PartitionedFailureKind::Assembly);
        assert_eq!(failure.costs.local_unit_calls, 0);
        assert_eq!(
            failure.error.context().related_unit_ids(),
            &[UnitId::new("s1")]
        );
    }
}
#[test]
fn either_canonical_mixer_inlet_can_be_explicitly_bound_as_the_return() {
    let (mut flow, mut options) = case(0.6);
    for port in &mut flow.units.get_mut(&UnitId::new("m1")).unwrap().ports {
        match port.name.as_str() {
            "inlet_a" => port.stream_id = Some("s-recycle".into()),
            "inlet_b" => port.stream_id = Some("s-fresh".into()),
            _ => {}
        }
    }
    options.tears[0].sink.port_name = "inlet_a".into();
    let result = solve(&flow, &options).unwrap();
    assert_flows(&result, "s-product", 2.2, 2.8, 1e-9);
    assert_flows(&result, "s-recycle", 3.3, 4.2, 1e-9);
}
#[test]
fn fixed_task_does_not_expand_the_qs0_material_template() {
    use crate::component_balance::{MixerBalanceSpecification, MixerComponentBalanceProblem};
    let (flow, options) = case(0.6);
    assert!(solve(&flow, &options).is_ok());
    assert!(
        MixerComponentBalanceProblem::assemble(
            &flow,
            &MixerBalanceSpecification::new(
                "m0",
                MixerBalanceScales {
                    inlet_a_mol_s: 5.0,
                    inlet_b_mol_s: 5.0,
                    outlet_mol_s: 5.0
                }
            )
        )
        .is_err()
    );
}

#[test]
fn total_flow_acceptance_propagates_component_tolerances_instead_of_using_a_percentage() {
    let (flow, options) = case(0.6);
    let plan = assemble_partitioned_plan(&flow, &options).unwrap();
    let input = stream("input", 5.0, 0.44, 0.56);
    let scales = composition(5.0, 5.0);
    for (defect, accepted) in [(1e-10, true), (3e-10, false)] {
        let mut output = input.clone();
        output.id = "output".into();
        output.total_molar_flow_mol_s += defect;
        // Keep each component flow exactly constant; the redundant total must
        // still be checked when fractions are within normalization tolerance.
        for z in output.overall_mole_fractions.values_mut() {
            *z *= 5.0 / output.total_molar_flow_mol_s;
        }
        assert_eq!(
            checks::balance(&plan, std::slice::from_ref(&input), &[output], &scales).is_ok(),
            accepted
        );
    }
}
