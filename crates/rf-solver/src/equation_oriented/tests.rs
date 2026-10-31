mod faults;
mod numerical;
use super::*;
use crate::test_fixtures::cross_recycle;
fn options() -> EquationOptions {
    EquationOptions::new(FixedBoundary {
        temperature_k: 300.0,
        pressure_pa: 650_000.0,
    })
}
fn inverse_options() -> EquationOptions {
    let mut o = options();
    o.released = Some(ReleasedSplitFraction {
        binding: ParameterBinding {
            unit: "sa".into(),
            field: ParameterField::OutletASplitFraction,
        },
        lower: 0.0,
        upper: 0.95,
        max_iterations: 64,
    });
    o.target = Some(ProductTotalTarget {
        source: PortBinding {
            unit: "sa".into(),
            port: MaterialPort::OutletB,
            stream: "pa".into(),
        },
        value_mol_s: 1.5,
    });
    o
}
// Independent elimination of the two mixer balances, not a call to the
// assembly, local residual implementation or any product solver.
fn exact_values(problem: &EquationProblem, alpha: f64, beta: f64) -> Vec<f64> {
    problem
        .variables
        .iter()
        .map(|v| match v {
            VariableBinding::Parameter(_) => alpha,
            VariableBinding::PortComponent { port, component } => {
                let (fa, fb) = if component.as_str() == "methane" {
                    (1.6, 0.6)
                } else {
                    (0.4, 2.4)
                };
                let ma = (fa + beta * fb) / (1.0 - alpha * beta);
                let mb = (fb + alpha * fa) / (1.0 - alpha * beta);
                use MaterialPort::*;
                match (port.unit.as_str(), port.port) {
                    ("fa", Outlet) => fa,
                    ("fb", Outlet) => fb,
                    ("ma", InletA) => fa,
                    ("ma", InletB) => beta * mb,
                    ("ma", Outlet) => ma,
                    ("mb", InletA) => fb,
                    ("mb", InletB) => alpha * ma,
                    ("mb", Outlet) => mb,
                    ("sa", Inlet) => ma,
                    ("sa", OutletA) => alpha * ma,
                    ("sa", OutletB) => (1.0 - alpha) * ma,
                    ("sb", Inlet) => mb,
                    ("sb", OutletA) => beta * mb,
                    ("sb", OutletB) => (1.0 - beta) * mb,
                    _ => panic!("unexpected reference binding"),
                }
            }
        })
        .collect()
}
#[test]
fn forward_and_inverse_have_full_domain_identity_and_correct_independent_residuals() {
    for (o, n, alpha) in [(options(), 28, 0.6), (inverse_options(), 29, 10.0 / 19.0)] {
        let p = assemble(&cross_recycle(0.6, 0.25), &o).unwrap();
        assert_eq!(
            (
                p.variables.len(),
                p.equations.len(),
                p.structure.matching.len()
            ),
            (n, n, n)
        );
        p.require_ready().unwrap();
        assert_eq!(p.fresh_total_mol_s, 5.0);
        let e = p.evaluate(&exact_values(&p, alpha, 0.25)).unwrap();
        assert!(e.residuals_mol_s.iter().all(|r| r.abs() < 1e-14));
        assert!(p.variables.windows(2).all(|v| v[0] < v[1]));
        assert!(p.equations.windows(2).all(|v| v[0] < v[1]));
    }
}
#[test]
fn independent_perturbations_validate_full_bilinear_jacobian() {
    let p = assemble(&cross_recycle(0.6, 0.25), &inverse_options()).unwrap();
    for alpha in [0.1, 10.0 / 19.0, 0.9] {
        let x = exact_values(&p, alpha, 0.25);
        let reference = p.evaluate(&x).unwrap();
        for h in [1e-4, 1e-5, 1e-6] {
            for j in 0..x.len() {
                let mut plus = x.clone();
                let mut minus = x.clone();
                plus[j] += h;
                minus[j] -= h;
                let a = p.evaluate(&plus).unwrap();
                let b = p.evaluate(&minus).unwrap();
                for i in 0..p.equations.len() {
                    let observed = (a.residuals_mol_s[i] - b.residuals_mol_s[i]) / (2.0 * h);
                    assert!(
                        (observed - reference.jacobian_si[i][j]).abs() < 1e-8,
                        "row {i} column {j}"
                    );
                }
            }
        }
    }
}
#[test]
fn pairing_is_readiness_and_reports_the_entire_alternating_region() {
    let flow = cross_recycle(0.6, 0.25);
    let mut under = inverse_options();
    under.target = None;
    let p = assemble(&flow, &under).unwrap();
    assert_eq!(
        (p.structure.variable_count, p.structure.equation_count),
        (29, 28)
    );
    assert_eq!(
        p.require_ready().unwrap_err().kind,
        EquationFailureKind::StructuralUnder
    );
    assert!(!p.structure.underdetermined.variables.is_empty());
    let mut over = inverse_options();
    over.released = None;
    let p = assemble(&flow, &over).unwrap();
    assert_eq!(
        (p.structure.variable_count, p.structure.equation_count),
        (28, 29)
    );
    assert_eq!(
        p.require_ready().unwrap_err().kind,
        EquationFailureKind::StructuralOver
    );
    assert!(!p.structure.overdetermined.equations.is_empty());
}
#[test]
fn structural_match_does_not_claim_numerical_rank_or_consistency() {
    let p = assemble(&cross_recycle(1.0, 1.0), &options()).unwrap();
    p.require_ready().unwrap();
    let mut p = assemble(&cross_recycle(0.6, 0.25), &options()).unwrap();
    // Duplicate an equation at the expense of another independent feed row:
    // total counts remain equal, but both deficient regions must be retained.
    p.rows[0] = p.rows[1].clone();
    let report = structure::analyze(&p, &mut Uninterrupted).unwrap();
    assert!(!report.unmatched_variables.is_empty());
    assert!(!report.unmatched_equations.is_empty());
    assert!(!report.underdetermined.variables.is_empty());
    assert!(!report.overdetermined.equations.is_empty());
}
#[test]
fn released_parameter_can_be_unspecified_and_all_explicit_scales_bind_exactly() {
    let mut flow = cross_recycle(0.6, 0.25);
    flow.units
        .get_mut(&"sa".into())
        .unwrap()
        .parameters
        .outlet_a_split_fraction = None;
    assert!(assemble(&flow, &options()).is_err());
    let mut o = inverse_options();
    let p = assemble(&flow, &o).unwrap();
    o.scales = Some(ExplicitEquationScales {
        variables: p.variables.iter().cloned().map(|v| (v, 2.0)).collect(),
        equations: p.equations.iter().cloned().map(|v| (v, 3.0)).collect(),
    });
    let p = assemble(&flow, &o).unwrap();
    assert!(p.variable_scales.iter().all(|v| *v == 2.0));
    o.scales.as_mut().unwrap().variables.remove(&p.variables[0]);
    assert_eq!(
        assemble(&flow, &o).unwrap_err().kind,
        EquationFailureKind::InvalidBinding
    );
    o.scales
        .as_mut()
        .unwrap()
        .variables
        .insert(p.variables[0].clone(), f64::INFINITY);
    assert_eq!(
        assemble(&flow, &o).unwrap_err().kind,
        EquationFailureKind::InvalidSpecification
    );
}
#[test]
fn stale_target_wrong_model_unanchored_and_capacity_are_rejected_before_dense_allocation() {
    let flow = cross_recycle(0.6, 0.25);
    let mut o = inverse_options();
    o.target.as_mut().unwrap().source.stream = "ab".into();
    assert_eq!(
        assemble(&flow, &o).unwrap_err().kind,
        EquationFailureKind::InvalidBinding
    );
    let mut flow = flow.clone();
    flow.units.get_mut(&"ma".into()).unwrap().kind = "heater".into();
    assert_eq!(
        assemble(&flow, &options()).unwrap_err().kind,
        EquationFailureKind::UnsupportedModel
    );
    let mut flow = cross_recycle(0.6, 0.25);
    for i in 0..8 {
        flow.insert_component(rf_model::Component::new(format!("extra-{i}"), "Extra"))
            .unwrap();
    }
    assert_eq!(
        assemble(&flow, &options()).unwrap_err().kind,
        EquationFailureKind::Capacity
    );
}
#[test]
fn bad_feed_and_boundary_inputs_do_not_get_excluded_or_overridden() {
    for bad in [-1.0, f64::NAN, f64::INFINITY] {
        let mut f = cross_recycle(0.6, 0.25);
        f.streams
            .get_mut(&"fa-out".into())
            .unwrap()
            .total_molar_flow_mol_s = bad;
        assert!(assemble(&f, &options()).is_err());
    }
    let mut f = cross_recycle(0.6, 0.25);
    f.units
        .get_mut(&"fa".into())
        .unwrap()
        .parameters
        .outlet_temperature_k = Some(301.0);
    assert!(assemble(&f, &options()).is_err());
}
#[test]
fn structure_traversal_obeys_cancellation() {
    struct Stop;
    impl EquationControl for Stop {
        fn checkpoint(&mut self, p: EquationProgress) -> std::ops::ControlFlow<()> {
            if p.stage == EquationStage::StructuralAnalysis {
                std::ops::ControlFlow::Break(())
            } else {
                std::ops::ControlFlow::Continue(())
            }
        }
    }
    assert_eq!(
        assembly::assemble_controlled(&cross_recycle(0.6, 0.25), &options(), &mut Stop)
            .unwrap_err()
            .kind,
        EquationFailureKind::Cancelled
    );
}

#[test]
fn unreachable_cycles_unbound_ports_and_zero_positive_feed_total_refuse() {
    let mut flow = cross_recycle(0.6, 0.25);
    for id in ["x", "y", "xp", "yp"] {
        flow.insert_stream(MaterialStreamState::from_tpzf(
            id,
            id,
            300.0,
            650_000.0,
            0.0,
            [("methane".into(), 1.0)].into(),
        ))
        .unwrap();
    }
    for (id, input, output, product) in [("sx", "y", "x", "xp"), ("sy", "x", "y", "yp")] {
        let mut unit = rf_unitops::build_splitter_node(id, id, input, output, product);
        unit.parameters.outlet_a_split_fraction = Some(0.5);
        flow.insert_unit(unit).unwrap();
    }
    assert!(
        assemble(&flow, &options())
            .unwrap_err()
            .error
            .message()
            .contains("reachable")
    );
    let mut flow = cross_recycle(0.6, 0.25);
    flow.units
        .get_mut(&"sa".into())
        .unwrap()
        .ports
        .iter_mut()
        .find(|p| p.name == "outlet_b")
        .unwrap()
        .stream_id = None;
    assert_eq!(
        assemble(&flow, &options()).unwrap_err().kind,
        EquationFailureKind::InvalidBinding
    );
    let mut flow = cross_recycle(0.6, 0.25);
    for id in ["fa-out", "fb-out"] {
        flow.streams
            .get_mut(&StreamId::new(id))
            .unwrap()
            .total_molar_flow_mol_s = 0.0;
    }
    assert!(
        assemble(&flow, &options())
            .unwrap_err()
            .error
            .message()
            .contains("positive")
    );
}
