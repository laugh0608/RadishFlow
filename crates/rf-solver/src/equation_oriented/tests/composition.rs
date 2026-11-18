use super::{numerical::solve, *};

pub(super) fn composition_options(value: f64) -> EquationOptions {
    let mut o = inverse_options();
    o.target = Some(ProductTarget::MoleFraction(Box::new(
        ProductMoleFractionTarget {
            source: PortBinding {
                unit: "sb".into(),
                port: MaterialPort::OutletB,
                stream: "pb".into(),
            },
            component: "methane".into(),
            value_mol_mol: value,
        },
    )));
    o
}

#[test]
fn cross_recycle_project_meets_the_product_specification_and_material_balance() {
    let project = rf_store::project_v4::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/flowsheets/fixed-tp-cross-recycle-binary-hydrocarbon.rfproj.json"
    )))
    .unwrap();
    let rf_store::project_v4::ProjectV4Read::Editable(project) = project else {
        panic!("editable example")
    };
    let flow = &project.document.flowsheet;
    let before = flow.clone();
    let mut options = composition_options(0.35);
    options.released.as_mut().unwrap().binding.unit = "splitter-1".into();
    let Some(ProductTarget::MoleFraction(t)) = &mut options.target else {
        unreachable!()
    };
    t.source.unit = "splitter-2".into();
    t.source.stream = "s-pb".into();
    let plan = crate::flow_plan::FlowPlanConfiguration::EquationOriented(options);
    plan.validate_configuration().unwrap();
    plan.validate_for(flow).unwrap();
    let outcome = crate::execution::execute_flow(
        flow,
        &plan,
        &crate::SolverServices {
            thermo: &crate::test_fixtures::binary_hydrocarbon_provider(),
            flash_solver: &rf_flash::PlaceholderTpFlashSolver,
        },
        &mut crate::execution::UncontrolledSolve,
    );
    let crate::execution::FlowExecutionOutcome::EquationOriented(Ok(result)) = outcome else {
        panic!("composition inverse failed: {outcome:?}")
    };
    assert_eq!(
        result.acceptance_contract_version,
        PRODUCT_COMPOSITION_CONTRACT_VERSION
    );
    assert_eq!(
        result.equation_contract_version,
        PRODUCT_COMPOSITION_CONTRACT_VERSION
    );
    let alpha = result.evidence.solved_parameter.unwrap().value;
    let Some(VerifiedTarget::MoleFraction(verified)) = result.evidence.verified_target else {
        panic!("expected composition evidence")
    };
    let x = verified.composition;
    let product = &result.streams[&StreamId::new("s-pb")];
    assert!((alpha - 0.5).abs() <= FRACTION_INTERVAL_TOLERANCE);
    assert!((x.total_mol_s - 24.0 / 7.0).abs() < 1e-11);
    assert!((x.component_mol_s - 1.2).abs() < 1e-11);
    assert!((product.overall_mole_fractions[&ComponentId::new("methane")] - 0.35).abs() <= 1e-10);
    assert!(x.residual_mol_mol.abs() <= 1e-10);
    assert!(verified.residual_mol_s.abs() <= ABSOLUTE_TOLERANCE_MOL_S);
    assert!(verified.scaled_residual.abs() <= SCALED_TOLERANCE);
    assert_eq!(result.costs.local_unit_calls, 6);
    assert!(result.evidence.global_material.is_some());
    assert_eq!(result.evidence.full_jacobian.unwrap().dimension, 29);
    for sample in result.evidence.conditional_evaluations {
        let a = sample.fraction.unwrap();
        let measured = sample.composition.unwrap();
        let expected = (0.6 + 1.6 * a) / (3.0 + 2.0 * a);
        assert!((measured.actual_mol_mol - expected).abs() < 1e-14);
        assert!((measured.residual_mol_mol - (expected - 0.35)).abs() < 1e-14);
    }
    assert_eq!(*flow, before);
}

#[test]
fn endpoint_roots_unreachable_targets_and_ineffective_release_are_distinguished() {
    let flow = cross_recycle(0.5, 0.25);
    for (target, endpoint) in [(0.2, 0.0), (106.0 / 245.0, 0.95)] {
        let result = solve(&flow, &composition_options(target)).unwrap();
        assert_eq!(result.evidence.solved_parameter.unwrap().value, endpoint);
        assert_eq!(result.costs.conditional_linear_evaluations, 2);
    }
    let failure = solve(&flow, &composition_options(0.45)).unwrap_err();
    assert_eq!(failure.kind, EquationFailureKind::TargetNotBracketed);
    assert_eq!(failure.costs.local_unit_calls, 0);
    let mut o = composition_options(0.35);
    o.released.as_mut().unwrap().binding.unit = "sb".into();
    let failure = solve(&flow, &o).unwrap_err();
    assert!(
        matches!(
            failure.kind,
            EquationFailureKind::LinearSingular | EquationFailureKind::IllConditioned
        ),
        "{failure}"
    );
    assert_eq!(failure.costs.full_jacobian_evaluations, 1);
    assert_eq!(failure.costs.local_unit_calls, 0);
}

#[test]
fn zero_flow_has_no_product_composition_and_trace_flow_cannot_mask_wrong_fraction() {
    let mut o = composition_options(0.35);
    o.released.as_mut().unwrap().lower = 0.6;
    let failure = solve(&cross_recycle(0.6, 1.0), &o).unwrap_err();
    assert_eq!(failure.kind, EquationFailureKind::UndefinedComposition);
    assert!(failure.evidence.conditional_evaluations.is_empty());
    let mut endpoint = composition_options(0.35);
    let release = endpoint.released.as_mut().unwrap();
    release.binding.unit = "sb".into();
    release.upper = 1.0;
    let failure = solve(&cross_recycle(0.5, 0.25), &endpoint).unwrap_err();
    assert_eq!(failure.kind, EquationFailureKind::UndefinedComposition);
    assert_eq!(failure.evidence.conditional_evaluations.len(), 2);
    assert!(failure.evidence.conditional_evaluations[0].completed);
    assert!(!failure.evidence.conditional_evaluations[1].completed);
    let failure = solve(&cross_recycle(0.6, 1.0 - 1e-12), &o).unwrap_err();
    assert_eq!(failure.kind, EquationFailureKind::TargetNotBracketed);
    let first = &failure.evidence.conditional_evaluations[0];
    let x = first.composition.unwrap();
    assert!(x.total_mol_s > 0.0 && x.total_mol_s < 2e-11);
    assert!(first.target_residual_mol_s.unwrap().abs() < ABSOLUTE_TOLERANCE_MOL_S);
    assert!(x.residual_mol_mol > 0.02);
}

#[test]
fn fraction_domain_terminal_identity_and_component_registry_are_checked() {
    let flow = cross_recycle(0.6, 0.25);
    for value in [-0.1, 1.01, f64::NAN, f64::INFINITY] {
        assert_eq!(
            assemble(&flow, &composition_options(value))
                .unwrap_err()
                .kind,
            EquationFailureKind::InvalidSpecification
        );
    }
    for (unit, port, stream, component) in [
        ("sb", MaterialPort::OutletB, "pb", "unknown"),
        ("sb", MaterialPort::OutletB, "pb", " "),
        ("sb", MaterialPort::OutletA, "ba", "methane"),
        ("sa", MaterialPort::OutletB, "pb", "methane"),
        ("sb", MaterialPort::OutletA, "pb", "methane"),
    ] {
        let mut o = composition_options(0.35);
        let Some(ProductTarget::MoleFraction(t)) = &mut o.target else {
            unreachable!()
        };
        t.source = PortBinding {
            unit: unit.into(),
            port,
            stream: stream.into(),
        };
        t.component = component.into();
        assert_eq!(
            assemble(&flow, &o).unwrap_err().kind,
            EquationFailureKind::InvalidBinding
        );
    }
}

#[test]
fn target_jacobian_and_structural_incidence_match_the_specified_fraction() {
    let flow = cross_recycle(0.6, 0.25);
    for target in [0.0, 0.35, 1.0] {
        let p = assemble(&flow, &composition_options(target)).unwrap();
        let values = exact_values(&p, 0.6, 0.25);
        let row = p
            .equations
            .iter()
            .position(EquationBinding::is_product_target)
            .unwrap();
        let evaluation = p.evaluate(&values).unwrap();
        let incidence = &p.incidence().unwrap()[row];
        for (i, variable) in p.variables.iter().enumerate() {
            let expected = match variable {
                VariableBinding::PortComponent { port, component }
                    if port.unit.as_str() == "sb" && port.port == MaterialPort::OutletB =>
                {
                    if component.as_str() == "methane" {
                        1.0 - target
                    } else {
                        -target
                    }
                }
                _ => 0.0,
            };
            assert_eq!(evaluation.jacobian_si[row][i], expected);
            assert_eq!(incidence.contains(&i), expected != 0.0);
            let mut perturbed = values.clone();
            perturbed[i] += 1e-6;
            let difference = (p.evaluate(&perturbed).unwrap().residuals_mol_s[row]
                - evaluation.residuals_mol_s[row])
                / 1e-6;
            assert!((difference - expected).abs() < 1e-9);
        }
    }
}

#[test]
fn explicit_target_scale_binds_the_selected_component_of_the_same_product() {
    let flow = cross_recycle(0.6, 0.25);
    let mut o = composition_options(0.65);
    let Some(ProductTarget::MoleFraction(t)) = &mut o.target else {
        unreachable!()
    };
    t.component = "ethane".into();
    let source = t.source.clone();
    let p = assemble(&flow, &o).unwrap();
    let mut scales = ExplicitEquationScales {
        variables: p.variables().iter().cloned().map(|v| (v, 1.0)).collect(),
        equations: p.equations().iter().cloned().map(|e| (e, 5.0)).collect(),
    };
    let ethane = EquationBinding::ProductMoleFraction {
        source: source.clone(),
        component: "ethane".into(),
    };
    *scales.equations.get_mut(&ethane).unwrap() = 0.125;
    o.scales = Some(scales.clone());
    let result = solve(&flow, &o).unwrap();
    assert!(
        (result.evidence.solved_parameter.unwrap().value - 0.5).abs()
            <= FRACTION_INTERVAL_TOLERANCE
    );
    let Some(VerifiedTarget::MoleFraction(target)) = result.evidence.verified_target else {
        panic!("expected ethane evidence")
    };
    assert!((target.composition.component_mol_s - 78.0 / 35.0).abs() < 1e-11);
    assert_eq!(target.scaled_residual, target.residual_mol_s / 0.125);
    scales.equations.remove(&ethane);
    scales.equations.insert(
        EquationBinding::ProductMoleFraction {
            source,
            component: "methane".into(),
        },
        0.125,
    );
    o.scales = Some(scales);
    assert_eq!(
        assemble(&flow, &o).unwrap_err().kind,
        EquationFailureKind::InvalidBinding
    );
}
