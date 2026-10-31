use super::*;
use crate::test_fixtures::binary_hydrocarbon_provider;
use rf_flash::PlaceholderTpFlashSolver;
fn solve(flow: &Flowsheet, o: &EquationOptions) -> Result<EquationResult, Box<EquationFailure>> {
    EquationOrientedSolver.solve(
        &crate::SolverServices {
            thermo: &binary_hydrocarbon_provider(),
            flash_solver: &PlaceholderTpFlashSolver,
        },
        flow,
        o,
    )
}
#[test]
fn real_forward_and_inverse_match_fraction_oracles_and_keep_input_immutable() {
    let flow = cross_recycle(0.6, 0.25);
    let before = flow.clone();
    for (o, alpha, backend) in [
        (options(), 0.6, FORWARD_BACKEND),
        (inverse_options(), 10.0 / 19.0, INVERSE_BACKEND),
    ] {
        let result = solve(&flow, &o).unwrap();
        let p = assemble(&flow, &o).unwrap();
        let reference = exact_values(&p, alpha, 0.25);
        let actual = result.evidence.verified_values_si.as_ref().unwrap();
        for (a, b) in actual.iter().zip(reference) {
            assert!((a - b).abs() < 1e-11, "{a} != {b}");
        }
        assert_eq!(result.backend, backend);
        assert_eq!(
            result.evidence.full_jacobian.as_ref().unwrap().dimension,
            actual.len()
        );
        assert_eq!(result.evidence.final_units.len(), 6);
        assert_eq!(result.costs.local_unit_calls, 6);
        assert_eq!(result.costs.operation_constructions, 6);
        assert_eq!(result.costs.full_jacobian_evaluations, 1);
        assert_eq!(
            result.costs.factorizations,
            result.costs.conditional_linear_evaluations + 1
        );
        assert!(result.costs.thermo_calls > 0);
        assert!(result.costs.flash_calls > 0);
        assert_eq!(
            result.evidence.conditional_evaluations.len(),
            result.costs.conditional_linear_evaluations
        );
        assert!(
            result
                .evidence
                .conditional_evaluations
                .iter()
                .all(|s| s.completed && s.linear.as_ref().unwrap().dimension == 28)
        );
        if o.released.is_some() {
            let solved = result.evidence.solved_parameter.as_ref().unwrap();
            assert_eq!(solved.stored_value, Some(0.6));
            assert!((solved.value - 10.0 / 19.0).abs() < 1e-12);
            assert!(
                result
                    .evidence
                    .verified_target
                    .unwrap()
                    .residual_mol_s
                    .abs()
                    < 1e-10
            );
            assert_eq!(
                result.costs.target_evaluations,
                result.costs.conditional_linear_evaluations
            );
            assert!(
                result
                    .evidence
                    .candidate_residuals_mol_s
                    .as_ref()
                    .unwrap()
                    .iter()
                    .any(|r| *r != 0.0)
            );
        } else {
            assert_eq!(result.costs.conditional_linear_evaluations, 1);
            assert_eq!(result.costs.target_evaluations, 0);
        }
    }
    assert_eq!(flow, before);
}
#[test]
fn forward_agrees_with_existing_modular_strategy_without_using_it_as_backend() {
    let flow = cross_recycle(0.6, 0.25);
    let eo = solve(&flow, &options()).unwrap();
    let qs2 = crate::simultaneous::SimultaneousModularSolver
        .solve(
            &crate::SolverServices {
                thermo: &binary_hydrocarbon_provider(),
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &crate::simultaneous::ModularOptions::new(options().boundary),
        )
        .unwrap();
    for (id, s) in &eo.streams {
        let other = &qs2.streams[id];
        assert!((s.total_molar_flow_mol_s - other.total_molar_flow_mol_s).abs() < 1e-11);
        for c in flow.components.keys() {
            assert!((s.overall_mole_fractions[c] - other.overall_mole_fractions[c]).abs() < 1e-11);
        }
    }
}
#[test]
fn bracket_endpoints_unbracketed_budget_and_nonisolated_candidate_are_distinct() {
    let flow = cross_recycle(0.6, 0.25);
    for (target, expected) in [(11.0 / 4.0, 0.0), (11.0 / 61.0, 0.95)] {
        let mut o = inverse_options();
        o.target.as_mut().unwrap().value_mol_s = target;
        let r = solve(&flow, &o).unwrap();
        assert_eq!(r.evidence.solved_parameter.unwrap().value, expected);
        assert_eq!(r.costs.conditional_linear_evaluations, 2);
    }
    let mut o = inverse_options();
    o.target.as_mut().unwrap().value_mol_s = 3.0;
    let e = solve(&flow, &o).unwrap_err();
    assert_eq!(e.kind, EquationFailureKind::TargetNotBracketed);
    assert_eq!(e.costs.local_unit_calls, 0);
    assert_eq!(e.evidence.conditional_evaluations.len(), 2);
    let mut o = inverse_options();
    o.released.as_mut().unwrap().max_iterations = 1;
    assert_eq!(
        solve(&flow, &o).unwrap_err().kind,
        EquationFailureKind::BudgetExceeded
    );
    let mut o = inverse_options();
    o.target.as_mut().unwrap().value_mol_s = 5.0;
    let e = solve(&cross_recycle(0.6, 1.0), &o).unwrap_err();
    assert_eq!(e.kind, EquationFailureKind::LinearSingular);
    assert_eq!(e.costs.full_jacobian_evaluations, 1);
    assert_eq!(e.costs.local_unit_calls, 0);
}
#[test]
fn singular_endpoint_is_not_skipped_and_zero_products_have_physical_composition_sources() {
    let mut o = inverse_options();
    o.released.as_mut().unwrap().upper = 1.0;
    let e = solve(&cross_recycle(0.6, 1.0), &o).unwrap_err();
    assert_eq!(e.kind, EquationFailureKind::LinearSingular);
    assert_eq!(e.evidence.conditional_evaluations.len(), 2);
    assert_eq!(e.costs.full_jacobian_evaluations, 0);
    let r = solve(&cross_recycle(1.0, 0.25), &options()).unwrap();
    assert_eq!(r.streams[&"pa".into()].total_molar_flow_mol_s, 0.0);
    let zero = PortBinding {
        unit: "sa".into(),
        port: MaterialPort::OutletB,
        stream: "pa".into(),
    };
    assert_eq!(
        r.evidence.zero_flow_sources[&zero].port,
        MaterialPort::Inlet
    );
    assert_eq!(r.evidence.zero_flow_sources[&zero].unit, UnitId::new("sa"));
    assert_eq!(
        r.streams[&"pa".into()].overall_mole_fractions,
        r.streams[&"ma-out".into()].overall_mole_fractions
    );
}
#[test]
fn explicit_double_scales_preserve_physics_and_wrong_scale_condition_refuses() {
    let flow = cross_recycle(0.6, 0.25);
    let mut o = inverse_options();
    let p = assemble(&flow, &o).unwrap();
    o.scales = Some(ExplicitEquationScales {
        variables: p
            .variables
            .iter()
            .enumerate()
            .map(|(i, v)| {
                (
                    v.clone(),
                    if matches!(v, VariableBinding::Parameter(_)) {
                        0.5
                    } else {
                        1.0 + (i % 3) as f64
                    },
                )
            })
            .collect(),
        equations: p
            .equations
            .iter()
            .enumerate()
            .map(|(i, e)| (e.clone(), 2.0 + (i % 4) as f64))
            .collect(),
    });
    let r = solve(&flow, &o).unwrap();
    assert!((r.evidence.solved_parameter.unwrap().value - 10.0 / 19.0).abs() < 1e-12);
    *o.scales
        .as_mut()
        .unwrap()
        .variables
        .get_mut(&p.variables[0])
        .unwrap() = 1e-20;
    let e = solve(&flow, &o).unwrap_err();
    assert!(matches!(
        e.kind,
        EquationFailureKind::LinearSingular | EquationFailureKind::IllConditioned
    ));
    assert_eq!(e.costs.local_unit_calls, 0);
}
#[test]
fn zero_composition_follows_nonselective_chain_and_acyclic_case_is_supported() {
    let mut flow = cross_recycle(1.0, 0.25);
    for id in ["pc", "pd"] {
        flow.insert_stream(MaterialStreamState::from_tpzf(
            id,
            id,
            900.0,
            10.0,
            90.0,
            [("methane".into(), 0.5), ("ethane".into(), 0.5)].into(),
        ))
        .unwrap();
    }
    let mut sc = rf_unitops::build_splitter_node("sc", "SC", "pa", "pc", "pd");
    sc.parameters.outlet_a_split_fraction = Some(0.4);
    flow.insert_unit(sc).unwrap();
    let r = solve(&flow, &options()).unwrap();
    for id in ["pc", "pd"] {
        assert_eq!(r.streams[&StreamId::new(id)].total_molar_flow_mol_s, 0.0);
        for c in flow.components.keys() {
            assert!(
                (r.streams[&StreamId::new(id)].overall_mole_fractions[c]
                    - r.streams[&"ma-out".into()].overall_mole_fractions[c])
                    .abs()
                    < 1e-14
            );
        }
    }
    let mut acyclic = Flowsheet::new("single feed");
    acyclic.components = flow.components.clone();
    acyclic
        .insert_stream(flow.streams[&"fa-out".into()].clone())
        .unwrap();
    acyclic
        .insert_unit(flow.units[&"fa".into()].clone())
        .unwrap();
    let r = solve(&acyclic, &options()).unwrap();
    assert_eq!(r.streams.len(), 1);
    assert_eq!(r.costs.local_unit_calls, 1);
}
