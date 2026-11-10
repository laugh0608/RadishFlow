//! Cross-strategy acceptance on the intersection of approved material capabilities.
use crate::SolverServices;
use crate::component_balance::{
    MaterialBalanceControl, MaterialBalanceOptions, MaterialBalanceProgress,
    MaterialBalanceSolveOutcome, MixerBalanceScales, MixerBalanceSpecification,
    MixerComponentBalanceProblem,
};
use crate::equation_oriented::{EquationOptions, EquationOrientedSolver};
use crate::partitioned::{
    FixedRecycleBoundary, PartitionedFailureKind, PartitionedSequentialSolver, RecycleSolveOptions,
    TearSpecification,
};
use crate::simultaneous::{ModularOptions, SimultaneousModularSolver};
use crate::test_fixtures::binary_hydrocarbon_provider;
use rf_flash::PlaceholderTpFlashSolver;
use rf_model::{Component, Flowsheet, MaterialStreamState};
use rf_types::{ComponentId, DiagnosticPortTarget, StreamId};
use rf_unitops::{build_feed_node, build_heater_node, build_mixer_node, build_splitter_node};
use std::{collections::BTreeMap, ops::ControlFlow};

struct ContinueBalance;
impl MaterialBalanceControl for ContinueBalance {
    fn checkpoint(&mut self, _: MaterialBalanceProgress) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

fn boundary() -> FixedRecycleBoundary {
    FixedRecycleBoundary {
        temperature_k: 300.0,
        pressure_pa: 650_000.0,
    }
}

fn case(recycle_fraction: Option<f64>) -> (Flowsheet, RecycleSolveOptions) {
    let mut flow = Flowsheet::new("Common material acceptance");
    for id in ["methane", "ethane"] {
        flow.insert_component(Component::new(id, id)).unwrap();
    }
    for (id, total, a) in [("fa", 2.0, 0.8), ("fb", 3.0, 0.2), ("fresh", 97.0, 0.9)] {
        flow.insert_stream(stream(id, total, a)).unwrap();
    }
    let mut units = vec![
        build_feed_node("fa", "FA", "fa"),
        build_feed_node("fb", "FB", "fb"),
        build_mixer_node("m0", "M0", "fa", "fb", "fresh"),
    ];
    let mut tears = vec![];
    if let Some(fraction) = recycle_fraction {
        for id in ["circulation", "return", "product"] {
            flow.insert_stream(stream(id, 83.0, 0.1)).unwrap();
        }
        units.push(build_mixer_node(
            "m1",
            "M1",
            "fresh",
            "return",
            "circulation",
        ));
        let mut splitter = build_splitter_node("s1", "S1", "circulation", "return", "product");
        splitter.parameters.outlet_a_split_fraction = Some(fraction);
        units.push(splitter);
        tears.push(TearSpecification {
            stream_id: "return".into(),
            source: DiagnosticPortTarget::new("s1", "outlet_a"),
            sink: DiagnosticPortTarget::new("m1", "inlet_b"),
            initial_component_flows_mol_s: None,
            component_scales_mol_s: None,
        });
    }
    for mut unit in units {
        if unit.kind == "mixer" {
            unit.parameters.outlet_pressure_pa = Some(boundary().pressure_pa);
        }
        flow.insert_unit(unit).unwrap();
    }
    (flow, RecycleSolveOptions::new(boundary(), tears))
}

fn stream(id: &str, total: f64, methane: f64) -> MaterialStreamState {
    MaterialStreamState::from_tpzf(
        id,
        id,
        boundary().temperature_k,
        boundary().pressure_pa,
        total,
        [
            ("methane".into(), methane),
            ("ethane".into(), 1.0 - methane),
        ]
        .into(),
    )
}

fn assert_components(
    streams: &BTreeMap<StreamId, MaterialStreamState>,
    id: &str,
    expected: [f64; 2],
) {
    let state = &streams[&StreamId::new(id)];
    for (component, expected) in ["methane", "ethane"].into_iter().zip(expected) {
        let actual = state.total_molar_flow_mol_s
            * state.overall_mole_fractions[&ComponentId::new(component)];
        assert!(
            (actual - expected).abs() < 1e-8,
            "{id}/{component}: {actual} != {expected}"
        );
    }
    assert!((state.temperature_k - boundary().temperature_k).abs() <= 1e-9);
    assert!((state.pressure_pa - boundary().pressure_pa).abs() <= 1e-6);
}

#[test]
fn qs0_through_qs3_agree_on_the_same_acyclic_material_problem() {
    let (flow, options) = case(None);
    let before = flow.clone();
    let provider = binary_hydrocarbon_provider();
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &PlaceholderTpFlashSolver,
    };
    let spec = MixerBalanceSpecification::new(
        "m0",
        MixerBalanceScales {
            inlet_a_mol_s: 2.0,
            inlet_b_mol_s: 3.0,
            outlet_mol_s: 5.0,
        },
    );
    let qs0 = MixerComponentBalanceProblem::assemble(&flow, &spec)
        .unwrap()
        .solve_controlled(&MaterialBalanceOptions::default(), &mut ContinueBalance)
        .unwrap();
    let MaterialBalanceSolveOutcome::Completed(qs0) = qs0 else {
        panic!("completed QS0")
    };
    for (id, expected) in [("methane", 2.2), ("ethane", 2.8)] {
        let actual = qs0.values.total_molar_flow_mol_s * qs0.values.mole_fractions[&id.into()];
        assert!((actual - expected).abs() < 1e-12);
    }
    let qs1 = PartitionedSequentialSolver
        .solve(&services, &flow, &options)
        .unwrap();
    let qs2 = SimultaneousModularSolver
        .solve(&services, &flow, &ModularOptions::new(boundary()))
        .unwrap();
    let qs3 = EquationOrientedSolver
        .solve(&services, &flow, &EquationOptions::new(boundary()))
        .unwrap();
    for streams in [&qs1.streams, &qs2.streams, &qs3.streams] {
        assert_components(streams, "fa", [1.6, 0.4]);
        assert_components(streams, "fb", [0.6, 2.4]);
        assert_components(streams, "fresh", [2.2, 2.8]);
    }
    assert_eq!(flow, before);
}

#[test]
fn qs1_through_qs3_share_recycle_oracles_but_keep_their_original_failure_limits() {
    let provider = binary_hydrocarbon_provider();
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &PlaceholderTpFlashSolver,
    };
    for fraction in [0.0, 0.6, 0.95, 0.99] {
        let (flow, options) = case(Some(fraction));
        let before = flow.clone();
        let qs1 = PartitionedSequentialSolver.solve(&services, &flow, &options);
        let qs2 = SimultaneousModularSolver
            .solve(&services, &flow, &ModularOptions::new(boundary()))
            .unwrap();
        let qs3 = EquationOrientedSolver
            .solve(&services, &flow, &EquationOptions::new(boundary()))
            .unwrap();
        // Independent balance: m=f/(1-alpha), r=alpha*m, product=f.
        for streams in [&qs2.streams, &qs3.streams]
            .into_iter()
            .chain(qs1.as_ref().ok().map(|r| &r.streams))
        {
            assert_components(streams, "fresh", [2.2, 2.8]);
            assert_components(
                streams,
                "circulation",
                [2.2 / (1.0 - fraction), 2.8 / (1.0 - fraction)],
            );
            assert_components(
                streams,
                "return",
                [
                    2.2 * fraction / (1.0 - fraction),
                    2.8 * fraction / (1.0 - fraction),
                ],
            );
            assert_components(streams, "product", [2.2, 2.8]);
        }
        match qs1 {
            Ok(result) => {
                assert!(fraction < 0.99);
                eprintln!("alpha={fraction} QS1 success costs={:?}", result.costs);
            }
            Err(failure) => {
                assert_eq!(fraction, 0.99);
                assert_eq!(failure.kind, PartitionedFailureKind::IterationLimit);
                assert_eq!(failure.costs.iteration_evaluations, 500);
                eprintln!(
                    "alpha={fraction} QS1 {:?} costs={:?}",
                    failure.kind, failure.costs
                );
            }
        }
        eprintln!("alpha={fraction} QS2 success costs={:?}", qs2.costs);
        eprintln!("alpha={fraction} QS3 success costs={:?}", qs3.costs);
        assert_eq!(flow, before);
    }
    let (flow, options) = case(Some(1.0));
    let before = flow.clone();
    let qs1 = PartitionedSequentialSolver
        .solve(&services, &flow, &options)
        .unwrap_err();
    let qs2 = SimultaneousModularSolver
        .solve(&services, &flow, &ModularOptions::new(boundary()))
        .unwrap_err();
    let qs3 = EquationOrientedSolver
        .solve(&services, &flow, &EquationOptions::new(boundary()))
        .unwrap_err();
    assert_eq!(qs1.kind, PartitionedFailureKind::NoFiniteSteadyState);
    assert_eq!(
        qs2.kind,
        crate::simultaneous::ModularFailureKind::NoFiniteSteadyState
    );
    assert_eq!(
        qs3.kind,
        crate::equation_oriented::EquationFailureKind::LinearSingular
    );
    assert_eq!(flow, before);
}

#[test]
fn local_only_downstream_model_has_an_independent_modular_reference_but_no_whole_flow_eo() {
    let mut flow = crate::test_fixtures::cross_recycle(0.6, 0.25);
    flow.insert_stream(stream("heated-product", 101.0, 0.9))
        .unwrap();
    let mut heater = build_heater_node("h1", "H1", "pa", "heated-product");
    heater.parameters.outlet_temperature_k = Some(boundary().temperature_k);
    heater.parameters.outlet_pressure_pa = Some(boundary().pressure_pa);
    flow.insert_unit(heater).unwrap();
    let before = flow.clone();
    let provider = binary_hydrocarbon_provider();
    let services = SolverServices {
        thermo: &provider,
        flash_solver: &PlaceholderTpFlashSolver,
    };
    let modular = SimultaneousModularSolver
        .solve(&services, &flow, &ModularOptions::new(boundary()))
        .unwrap();
    assert_components(&modular.streams, "pa", [14.0 / 17.0, 8.0 / 17.0]);
    assert_components(
        &modular.streams,
        "heated-product",
        [14.0 / 17.0, 8.0 / 17.0],
    );
    assert_components(&modular.streams, "pb", [117.0 / 85.0, 198.0 / 85.0]);
    let failure = EquationOrientedSolver
        .solve(&services, &flow, &EquationOptions::new(boundary()))
        .unwrap_err();
    assert_eq!(
        failure.kind,
        crate::equation_oriented::EquationFailureKind::UnsupportedModel
    );
    assert!(
        failure
            .error
            .context()
            .related_unit_ids()
            .contains(&"h1".into())
    );
    assert_eq!(failure.costs.local_unit_calls, 0);
    assert_eq!(flow, before);
}
