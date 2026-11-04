//! Numerical-layer fault injection, not an additional supported engineering model.
use super::*;
use rf_unitops::component_equations::{ComponentEquation, SplitFraction};

#[test]
fn singular_midpoint_is_reported_without_skipping_or_using_an_endpoint_result() {
    let flow = crate::test_fixtures::cross_recycle(0.6, 0.25);
    let mut options = super::super::tests::inverse_options();
    let released = options.released.as_mut().unwrap();
    released.lower = 0.0;
    released.upper = 1.0;
    let mut problem = assemble(&flow, &options).unwrap();
    let variable = |unit: &str, port: MaterialPort| {
        problem.variables.iter().find(|v|matches!(v,VariableBinding::PortComponent {port:p,component} if p.unit==UnitId::new(unit) && p.port==port && component.as_str()=="methane")).unwrap().clone()
    };
    let parameter = problem
        .variables
        .iter()
        .find(|v| matches!(v, VariableBinding::Parameter(_)))
        .unwrap()
        .clone();
    let variables = vec![
        variable("fa", MaterialPort::Outlet),
        variable("sa", MaterialPort::OutletB),
        variable("sb", MaterialPort::OutletA),
        variable("sb", MaterialPort::OutletB),
        parameter,
    ];
    let target = problem
        .equations
        .iter()
        .find(|e| matches!(e, EquationBinding::ProductTotal { .. }))
        .unwrap()
        .clone();
    let unit_row = |unit: &str, role: MaterialEquationRole| EquationBinding::Unit {
        unit: unit.into(),
        role,
        component: "methane".into(),
    };
    problem.variables = variables;
    problem.equations = vec![
        unit_row("fa", MaterialEquationRole::FeedBoundary),
        unit_row("sa", MaterialEquationRole::SplitB),
        unit_row("sb", MaterialEquationRole::SplitA),
        unit_row("mb", MaterialEquationRole::MixerBalance),
        target,
    ];
    problem.variable_scales = vec![1.0; 5];
    problem.residual_scales = vec![1.0; 5];
    // x=1; t=(1-alpha)x; u=alpha*v; v=u+u; target t=1/2.
    // The duplicated numerical inlet injects det=1-2*alpha. It cannot be
    // assembled from a valid single-producer engineering connection graph.
    problem.rows = vec![
        AssembledEquation::Unit {
            model: ComponentEquation {
                role: MaterialEquationRole::FeedBoundary,
                feed_mol_s: Some(1.0),
                split: None,
            },
            ports: [(MaterialPort::Outlet, 0)].into(),
            fraction: None,
        },
        AssembledEquation::Unit {
            model: ComponentEquation {
                role: MaterialEquationRole::SplitB,
                feed_mol_s: None,
                split: Some(SplitFraction::Released),
            },
            ports: [(MaterialPort::Inlet, 0), (MaterialPort::OutletB, 1)].into(),
            fraction: Some(4),
        },
        AssembledEquation::Unit {
            model: ComponentEquation {
                role: MaterialEquationRole::SplitA,
                feed_mol_s: None,
                split: Some(SplitFraction::Released),
            },
            ports: [(MaterialPort::Inlet, 3), (MaterialPort::OutletA, 2)].into(),
            fraction: Some(4),
        },
        AssembledEquation::Unit {
            model: ComponentEquation {
                role: MaterialEquationRole::MixerBalance,
                feed_mol_s: None,
                split: None,
            },
            ports: [
                (MaterialPort::InletA, 2),
                (MaterialPort::InletB, 2),
                (MaterialPort::Outlet, 3),
            ]
            .into(),
            fraction: None,
        },
        AssembledEquation::ProductTotal {
            components: vec![1],
            target: 0.5,
        },
    ];
    let mut control = Uninterrupted;
    let mut execution = Execution {
        control: &mut control,
        progress: EquationProgress::at(EquationStage::Assemble),
        evidence: EquationEvidence::default(),
        costs: EquationCosts::default(),
        stage_started: Instant::now(),
    };
    let error = execution.solve_equations(&problem).unwrap_err();
    assert_eq!(error.kind, EquationFailureKind::LinearSingular);
    assert_eq!(execution.costs.conditional_linear_evaluations, 3);
    assert_eq!(execution.costs.target_evaluations, 2);
    assert_eq!(execution.costs.interval_updates, 0);
    assert_eq!(execution.costs.local_unit_calls, 0);
    let samples = &execution.evidence.conditional_evaluations;
    assert_eq!(samples[0].target_residual_mol_s, Some(0.5));
    assert_eq!(samples[1].target_residual_mol_s, Some(-0.5));
    assert!(samples[..2].iter().all(|s| s.completed));
    assert_eq!(samples[2].fraction, Some(0.5));
    assert_eq!(samples[2].interval, Some((0.0, 1.0)));
    assert!(!samples[2].completed);
    assert!(samples[2].target_residual_mol_s.is_none());
    assert!(execution.evidence.candidate_values_si.is_none());
    assert!(execution.evidence.final_units.is_empty());
}
