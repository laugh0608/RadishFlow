use super::*;
use crate::component_balance::tests::fixture;

struct Control {
    seen: Vec<MaterialBalanceProgress>,
    cancel: Option<usize>,
}
impl MaterialBalanceControl for Control {
    fn checkpoint(&mut self, progress: MaterialBalanceProgress) -> ControlFlow<()> {
        self.seen.push(progress);
        if self.cancel == Some(self.seen.len() - 1) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
fn solve(problem: &MixerComponentBalanceProblem) -> MaterialBalanceSolution {
    let mut control = Control {
        seen: vec![],
        cancel: None,
    };
    let MaterialBalanceSolveOutcome::Completed(result) = problem
        .solve_controlled(&MaterialBalanceOptions::default(), &mut control)
        .unwrap()
    else {
        panic!("completion")
    };
    assert_eq!(control.seen.first().unwrap().completed_rows, 0);
    assert_eq!(
        control.seen.last().unwrap().stage,
        MaterialBalanceStage::Completed
    );
    *result
}

#[test]
fn direct_backend_solves_without_supplied_solution_and_reports_actual_evidence() {
    let (flow, spec) = fixture();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    let result = solve(&problem);
    for (actual, expected) in result
        .point_mol_s
        .iter()
        .zip([1.6, 0.4, 0.6, 2.4, 2.2, 2.8])
    {
        assert!((actual - expected).abs() < 1e-12);
    }
    assert_eq!(result.variables, problem.variables());
    assert_eq!(result.equations, problem.equations());
    assert_eq!(result.solved_rows, 6);
    assert_eq!(result.residual_evaluations, 1);
    assert!(
        result
            .options
            .algorithm
            .implementation()
            .contains("lower-triangular-substitution/1@")
    );
    assert!((result.values.total_molar_flow_mol_s - 5.0).abs() < 1e-12);
    assert!((result.values.mole_fractions[&"component-a".into()] - 0.44).abs() < 1e-12);
    assert_eq!(
        result.evaluation,
        problem.evaluate_si(&result.point_mol_s).unwrap()
    );
}

#[test]
fn cancellation_is_observed_at_every_declared_boundary_without_returning_a_solution() {
    let (flow, spec) = fixture();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    let mut first = Control {
        seen: vec![],
        cancel: None,
    };
    problem
        .solve_controlled(&MaterialBalanceOptions::default(), &mut first)
        .unwrap();
    for boundary in 0..first.seen.len() {
        let mut control = Control {
            seen: vec![],
            cancel: Some(boundary),
        };
        assert!(matches!(
            problem
                .solve_controlled(&MaterialBalanceOptions::default(), &mut control)
                .unwrap(),
            MaterialBalanceSolveOutcome::Cancelled
        ));
        assert_eq!(control.seen.len(), boundary + 1);
    }
    assert_eq!(solve(&problem).point_mol_s, solve(&problem).point_mol_s);
}

#[test]
fn positive_scales_and_sparse_components_preserve_the_physical_solution() {
    let (mut flow, mut spec) = fixture();
    flow.insert_component(rf_model::Component::new("component-c", "Absent"))
        .unwrap();
    for scales in [[1.0, 1.0, 1.0], [0.001, 100.0, 10.0], [1e100, 1e100, 1e100]] {
        spec.scales = MixerBalanceScales {
            inlet_a_mol_s: scales[0],
            inlet_b_mol_s: scales[1],
            outlet_mol_s: scales[2],
        };
        let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
        let result = solve(&problem);
        assert_eq!(result.solved_rows, 9);
        assert_eq!(
            result.values.component_molar_flows_mol_s[&"component-c".into()],
            0.0
        );
        assert!((result.values.total_molar_flow_mol_s - 5.0).abs() < 1e-12);
    }
}

#[test]
fn invalid_tolerances_are_rejected_before_execution() {
    let (flow, spec) = fixture();
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        for option in [
            MaterialBalanceTolerance::AbsoluteMolarFlow,
            MaterialBalanceTolerance::ScaledResidual,
        ] {
            let mut options = MaterialBalanceOptions::default();
            match option {
                MaterialBalanceTolerance::AbsoluteMolarFlow => {
                    options.absolute_tolerance_mol_s = bad
                }
                MaterialBalanceTolerance::ScaledResidual => options.scaled_tolerance = bad,
            }
            let mut control = Control {
                seen: vec![],
                cancel: None,
            };
            assert!(
                matches!(problem.solve_controlled(&options,&mut control), Err(MaterialBalanceSolveError::InvalidTolerance { option: actual }) if actual==option)
            );
            assert!(control.seen.is_empty());
        }
    }
}

#[test]
fn unsupported_linear_structure_never_silently_uses_the_triangular_backend() {
    let (flow, spec) = fixture();
    for diagonal in [0.0, f64::NAN] {
        let mut problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
        problem.scaled_jacobian[0][0] = diagonal;
        let mut control = Control {
            seen: vec![],
            cancel: None,
        };
        assert!(matches!(
            problem.solve_controlled(&MaterialBalanceOptions::default(), &mut control),
            Err(MaterialBalanceSolveError::UnsupportedStructure { .. })
        ));
    }
    let mut problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    problem.scaled_jacobian[0][1] = 0.1;
    let mut control = Control {
        seen: vec![],
        cancel: None,
    };
    assert!(matches!(
        problem.solve_controlled(&MaterialBalanceOptions::default(), &mut control),
        Err(MaterialBalanceSolveError::UnsupportedStructure { .. })
    ));
}

#[test]
fn numerical_underflow_cannot_skip_independent_si_verification() {
    let (flow, mut spec) = fixture();
    spec.scales = MixerBalanceScales {
        inlet_a_mol_s: 1e-200,
        inlet_b_mol_s: 1e-200,
        outlet_mol_s: 1e200,
    };
    let problem = MixerComponentBalanceProblem::assemble(&flow, &spec).unwrap();
    let mut control = Control {
        seen: vec![],
        cancel: None,
    };
    assert!(matches!(
        problem.solve_controlled(&MaterialBalanceOptions::default(), &mut control),
        Err(MaterialBalanceSolveError::Problem(
            ComponentBalanceError::UnsatisfiedEquation { .. }
        ))
    ));
}

#[test]
fn suggested_scales_use_connected_feed_totals_and_zero_requires_an_explicit_scale() {
    let (mut flow, spec) = fixture();
    flow.streams
        .get_mut(&"out".into())
        .unwrap()
        .total_molar_flow_mol_s = 999.0;
    let plan = MaterialBalancePlan::for_mixer(&flow, spec.mixer_id.clone()).unwrap();
    assert_eq!(plan.specification.scales, spec.scales);
    flow.streams
        .get_mut(&"a".into())
        .unwrap()
        .total_molar_flow_mol_s = 0.0;
    let plan = MaterialBalancePlan::for_mixer(&flow, spec.mixer_id).unwrap();
    assert!(plan.validate().is_err());
    assert_eq!(plan.specification.scales.inlet_a_mol_s, 0.0);
}
#[test]
fn readiness_and_assembly_share_the_same_domain_rejections() {
    let (mut flow, spec) = fixture();
    assert!(MixerComponentBalanceProblem::validate_specification(&flow, &spec).is_ok());
    flow.streams.get_mut(&"b".into()).unwrap().temperature_k = 301.0;
    let ready = MixerComponentBalanceProblem::validate_specification(&flow, &spec)
        .unwrap_err()
        .diagnostic();
    let assembly = MixerComponentBalanceProblem::assemble(&flow, &spec)
        .unwrap_err()
        .diagnostic();
    assert_eq!(ready, assembly);
}
