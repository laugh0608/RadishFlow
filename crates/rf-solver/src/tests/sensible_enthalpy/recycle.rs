use super::*;
use crate::{
    flow_plan::SinglePhaseOptions,
    partitioned::{EvaluationPhase, PartitionedStage},
    single_phase_recycle::{
        SinglePhaseRecycleOptions, SinglePhaseRecycleResult, SinglePhaseRecycleSolver,
        SinglePhaseTearBinding, SinglePhaseTearState,
    },
};
use rf_types::DiagnosticPortTarget;
use std::ops::ControlFlow;

fn properties() -> SinglePhaseOptions {
    SinglePhaseOptions {
        model: model(),
        heat_duties_w: BTreeMap::new(),
    }
}

fn specify_duty(flow: &mut Flowsheet, kind: BuiltinUnitKind, duty_w: f64) -> SinglePhaseOptions {
    let id = UnitId::from(kind.as_str());
    flow.units
        .get_mut(&id)
        .unwrap()
        .parameters
        .outlet_temperature_k = None;
    SinglePhaseOptions {
        model: model(),
        heat_duties_w: [(id, duty_w)].into(),
    }
}

pub(super) fn case(
    kind: BuiltinUnitKind,
    target_temperature_k: f64,
    fraction: f64,
) -> (Flowsheet, SinglePhaseRecycleOptions) {
    let mut flow = heater_cooler_case(kind, Some(target_temperature_k));
    for id in ["cycle-mixed", "conditioned", "recycle"] {
        let mut template = flow.stream(&"product".into()).unwrap().clone();
        template.id = id.into();
        flow.insert_stream(template).unwrap();
    }
    let thermal = flow.units.get_mut(&kind.as_str().into()).unwrap();
    for port in &mut thermal.ports {
        port.stream_id = Some(
            if port.name == "inlet" {
                "cycle-mixed"
            } else {
                "conditioned"
            }
            .into(),
        );
    }
    flow.insert_unit(build_mixer_node(
        "recycle-mixer",
        "Recycle mixer",
        "mixed",
        "recycle",
        "cycle-mixed",
    ))
    .unwrap();
    let mut splitter = build_splitter_node(
        "split",
        "Return/product split",
        "conditioned",
        "recycle",
        "product",
    );
    splitter.parameters.outlet_a_split_fraction = Some(fraction);
    flow.insert_unit(splitter).unwrap();
    let options = SinglePhaseRecycleOptions::new(SinglePhaseTearBinding {
        stream_id: "recycle".into(),
        source: DiagnosticPortTarget::new("split", "outlet_a"),
        sink: DiagnosticPortTarget::new("recycle-mixer", "inlet_b"),
    });
    (flow, options)
}

pub(super) fn assert_benchmark(
    result: &SinglePhaseRecycleResult,
    target_k: f64,
    duty_w: f64,
    expected_product_w: f64,
    expected_recycle_w: f64,
    mixer_k: f64,
) {
    let product = &result.streams[&"product".into()];
    let returned = &result.streams[&"recycle".into()];
    let mixed = &result.streams[&"cycle-mixed".into()];
    for (stream, expect_flow, expect_enthalpy_w) in [
        (product, 5.0, expected_product_w),
        (returned, 7.5, expected_recycle_w),
    ] {
        assert_close(stream.total_molar_flow_mol_s, expect_flow, 1e-9);
        assert_close(stream.temperature_k, target_k, 1e-8);
        assert_eq!(stream.pressure_pa, PRESSURE_PA);
        assert_close(
            stream.overall_mole_fractions[&"component-a".into()],
            0.4,
            1e-10,
        );
        assert_close(
            stream.overall_mole_fractions[&"component-b".into()],
            0.6,
            1e-10,
        );
        assert_close(enthalpy_flow(stream), expect_enthalpy_w, FLOW_TOLERANCE_W);
    }
    assert_close(mixed.temperature_k, mixer_k, 1e-8);
    assert_close(
        result.overall_energy_balance.heat_duty_w,
        duty_w,
        FLOW_TOLERANCE_W,
    );
    assert_close(
        result.overall_energy_balance.residual_w,
        (enthalpy_flow(product) - 9388.5) - result.overall_energy_balance.heat_duty_w,
        1e-10,
    );
    assert!(result.overall_energy_balance.residual_w.abs() <= FLOW_TOLERANCE_W);
    assert!(result.overall_energy_balance.scaled_residual.abs() <= 1e-10);
    assert_eq!(
        result
            .final_units
            .iter()
            .filter(|unit| unit.energy_balance.is_some())
            .count(),
        4
    );
    let accepted = result.trace.last().unwrap();
    assert_eq!(accepted.material.phase, EvaluationPhase::Verification);
    assert_eq!(accepted.material.evaluation, 1);
    assert_eq!(
        result
            .trace
            .iter()
            .filter(|row| row.material.phase == EvaluationPhase::Verification)
            .count(),
        1
    );
    assert!(accepted.material.max_absolute_mol_s <= 1e-10);
    assert!(accepted.material.max_scaled <= 1e-10);
    assert!(accepted.enthalpy_flow_residual_w.abs() <= FLOW_TOLERANCE_W);
    assert!(accepted.scaled_enthalpy_flow_residual.abs() <= 1e-10);
    assert_eq!(result.component_scale_mol_s, 5.0);
    assert_close(result.tear_enthalpy_scale_w, 9388.5, FLOW_TOLERANCE_W);
    assert_eq!(returned, &result.verification.returned);
    assert_eq!(
        &result
            .final_units
            .iter()
            .find(|step| step.unit_id.as_str() == "recycle-mixer")
            .unwrap()
            .consumed_streams[1],
        &result.verification.assumed,
    );
}

#[test]
fn single_phase_recycle_specified_temperature_matches_independent_steady_states() {
    for (kind, temperature, duty, product_w, recycle_w, mixer_k) in [
        (
            BuiltinUnitKind::Heater,
            380.0,
            7800.0,
            17188.5,
            25782.75,
            2556.0 / 7.0,
        ),
        (
            BuiltinUnitKind::Cooler,
            320.0,
            -4800.0,
            4588.5,
            6882.75,
            2304.0 / 7.0,
        ),
    ] {
        let (flow, mut options) = case(kind, temperature, 0.6);
        let original = flow.clone();
        for initial in [
            None,
            Some(SinglePhaseTearState {
                component_flows_mol_s: [("component-a".into(), 6.0), ("component-b".into(), 1.0)]
                    .into(),
                enthalpy_flow_w: 21125.5,
            }),
        ] {
            options.initial_state = initial;
            let result = SinglePhaseRecycleSolver
                .solve_controlled(&flow, &properties(), &options, &mut UncontrolledSolve)
                .unwrap();
            assert_benchmark(&result, temperature, duty, product_w, recycle_w, mixer_k);
            assert_eq!(flow, original);
        }
    }
}

#[test]
fn single_phase_recycle_zero_flow_initialization_and_budget_are_observable() {
    let (flow, mut options) = case(BuiltinUnitKind::Heater, 380.0, 0.6);
    options.max_iteration_evaluations = 1;
    let failure = SinglePhaseRecycleSolver
        .solve_controlled(&flow, &properties(), &options, &mut UncontrolledSolve)
        .unwrap_err();
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("solver.single_phase_recycle.iteration_limit")
    );
    assert_eq!(failure.trace.len(), 1);
    let eval = failure.last_evaluation.unwrap();
    assert_eq!(eval.assumed.total_molar_flow_mol_s, 0.0);
    assert!(eval.assumed.phases.is_empty());
    assert_close(eval.assumed.temperature_k, 2400.0 / 7.0, 1e-8);
    assert_eq!(
        eval.assumed.overall_mole_fractions,
        binary_composition(0.4, 0.6)
    );
    assert_eq!(eval.returned.total_molar_flow_mol_s, 3.0);
    assert_close(
        eval.residual.enthalpy_flow_residual_w,
        10313.1,
        FLOW_TOLERANCE_W,
    );

    let (flow, options) = case(BuiltinUnitKind::Heater, 380.0, 0.0);
    let result = SinglePhaseRecycleSolver
        .solve_controlled(&flow, &properties(), &options, &mut UncontrolledSolve)
        .unwrap();
    assert_eq!(result.trace.len(), 2);
    assert_eq!(
        result.streams[&"recycle".into()].total_molar_flow_mol_s,
        0.0
    );
    assert!(result.streams[&"recycle".into()].phases.is_empty());
    assert_close(
        result.verification.assumed.temperature_k,
        2400.0 / 7.0,
        1e-8,
    );
    assert_close(
        enthalpy_flow(&result.streams[&"product".into()]),
        17188.5,
        FLOW_TOLERANCE_W,
    );
}

#[test]
fn single_phase_recycle_specified_heat_duty_matches_both_temperature_cases() {
    for (kind, temperature, duty, product_w, recycle_w, mixer_k) in [
        (
            BuiltinUnitKind::Heater,
            380.0,
            7800.0,
            17188.5,
            25782.75,
            2556.0 / 7.0,
        ),
        (
            BuiltinUnitKind::Cooler,
            320.0,
            -4800.0,
            4588.5,
            6882.75,
            2304.0 / 7.0,
        ),
    ] {
        let (mut flow, options) = case(kind, temperature, 0.6);
        let forward = SinglePhaseRecycleSolver
            .solve_controlled(&flow, &properties(), &options, &mut UncontrolledSolve)
            .unwrap();
        let settings = specify_duty(&mut flow, kind, duty);
        let original = flow.clone();
        let inverse = SinglePhaseRecycleSolver
            .solve_controlled(&flow, &settings, &options, &mut UncontrolledSolve)
            .unwrap();
        assert_benchmark(&inverse, temperature, duty, product_w, recycle_w, mixer_k);
        for stream in ["product", "recycle", "cycle-mixed"] {
            let id = stream.into();
            assert_close(
                inverse.streams[&id].temperature_k,
                forward.streams[&id].temperature_k,
                1e-8,
            );
            assert_close(
                enthalpy_flow(&inverse.streams[&id]),
                enthalpy_flow(&forward.streams[&id]),
                FLOW_TOLERANCE_W,
            );
        }
        assert_eq!(flow, original);
    }
}

#[test]
fn single_phase_recycle_requires_enthalpy_closure_even_with_exact_tear_component_flows() {
    let kind = BuiltinUnitKind::Heater;
    let (mut flow, mut options) = case(kind, 380.0, 0.6);
    let settings = specify_duty(&mut flow, kind, 7800.0);
    options.max_iteration_evaluations = 1;
    options.initial_state = Some(SinglePhaseTearState {
        component_flows_mol_s: [("component-a".into(), 3.0), ("component-b".into(), 4.5)].into(),
        enthalpy_flow_w: 582.75,
    });
    let failure = SinglePhaseRecycleSolver
        .solve_controlled(&flow, &settings, &options, &mut UncontrolledSolve)
        .unwrap_err();
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("solver.single_phase_recycle.iteration_limit")
    );
    assert!(failure.trace[0].material.max_absolute_mol_s <= 1e-10);
    assert_close(
        failure.trace[0].enthalpy_flow_residual_w,
        10080.0,
        FLOW_TOLERANCE_W,
    );
}

#[test]
fn single_phase_recycle_heat_duty_covers_zero_high_recycle_and_domain_limits() {
    let kind = BuiltinUnitKind::Heater;
    for (alpha, duty_w, expected_k) in [
        (0.6, 0.0, 2400.0 / 7.0),
        (0.95, 7800.0, 380.0),
        (0.6, -13200.0, 280.0),
        (0.6, 16200.0, 420.0),
    ] {
        let (mut flow, options) = case(kind, 380.0, alpha);
        let settings = specify_duty(&mut flow, kind, duty_w);
        let result = SinglePhaseRecycleSolver
            .solve_controlled(&flow, &settings, &options, &mut UncontrolledSolve)
            .unwrap();
        let product = &result.streams[&"product".into()];
        let recycle = &result.streams[&"recycle".into()];
        assert_close(product.temperature_k, expected_k, 1e-8);
        assert_close(product.total_molar_flow_mol_s, 5.0, 1e-9);
        assert_close(
            recycle.total_molar_flow_mol_s,
            5.0 * alpha / (1.0 - alpha),
            5e-9,
        );
        let expected_product_w = 9388.5 + duty_w;
        assert_close(enthalpy_flow(product), expected_product_w, FLOW_TOLERANCE_W);
        assert_close(
            enthalpy_flow(recycle),
            expected_product_w * alpha / (1.0 - alpha),
            FLOW_TOLERANCE_W / (1.0 - alpha),
        );
        assert!(result.overall_energy_balance.residual_w.abs() <= FLOW_TOLERANCE_W);
        assert!(result.trace.len() <= 501);
    }
}

#[test]
fn single_phase_recycle_rejects_incompatible_heat_duties_and_tear_initialization() {
    let kind = BuiltinUnitKind::Heater;
    let (mut flow, mut options) = case(kind, 380.0, 0.6);
    let conflicting = SinglePhaseOptions {
        model: model(),
        heat_duties_w: [("heater".into(), 7800.0)].into(),
    };
    let failure = SinglePhaseRecycleSolver
        .solve_controlled(&flow, &conflicting, &options, &mut UncontrolledSolve)
        .unwrap_err();
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("solver.single_phase.conflicting_heat_specifications")
    );
    assert!(failure.trace.is_empty());
    for duty_w in [-13200.000001, 16200.000001] {
        let settings = specify_duty(&mut flow, kind, duty_w);
        let failure = SinglePhaseRecycleSolver
            .solve_controlled(&flow, &settings, &options, &mut UncontrolledSolve)
            .unwrap_err();
        assert_eq!(
            failure.error.context().diagnostic_code(),
            Some("thermo.sensible_enthalpy.outside_domain")
        );
        assert_eq!(failure.stage.unit, Some("heater".into()));
        assert!(failure.trace.is_empty());
    }
    let settings = specify_duty(&mut flow, kind, 7800.0);
    options.initial_state = Some(SinglePhaseTearState {
        component_flows_mol_s: [("component-a".into(), 0.0), ("component-b".into(), 0.0)].into(),
        enthalpy_flow_w: 1.0,
    });
    let failure = SinglePhaseRecycleSolver
        .solve_controlled(&flow, &settings, &options, &mut UncontrolledSolve)
        .unwrap_err();
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("solver.single_phase_recycle.invalid_initial_state")
    );
    assert_eq!(failure.stage.stage, PartitionedStage::Initializing);
}

#[test]
fn single_phase_recycle_validates_graph_identity_before_evaluation() {
    let (mut flow, options) = case(BuiltinUnitKind::Heater, 380.0, 0.6);
    flow.units.get_mut(&"split".into()).unwrap().id = "different-unit".into();
    let failure = SinglePhaseRecycleSolver
        .solve_controlled(&flow, &properties(), &options, &mut UncontrolledSolve)
        .unwrap_err();
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("solver.single_phase_recycle.identity")
    );
    assert_eq!(failure.stage.stage, PartitionedStage::AssemblyStarted);
    assert!(failure.trace.is_empty());
}

#[test]
fn single_phase_recycle_rejects_full_return_wrong_binding_and_cancellation() {
    let (flow, mut options) = case(BuiltinUnitKind::Heater, 380.0, 1.0);
    let failure = SinglePhaseRecycleSolver
        .solve_controlled(&flow, &properties(), &options, &mut UncontrolledSolve)
        .unwrap_err();
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("solver.single_phase_recycle.no_finite_steady_state")
    );
    assert!(failure.trace.is_empty());
    options.tear.sink.port_name = "inlet_a".into();
    let failure = SinglePhaseRecycleSolver
        .solve_controlled(&flow, &properties(), &options, &mut UncontrolledSolve)
        .unwrap_err();
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("solver.single_phase_recycle.invalid_tear")
    );
    assert_eq!(failure.stage.stage, PartitionedStage::AssemblyStarted);

    let (flow, options) = case(BuiltinUnitKind::Heater, 380.0, 0.6);
    let failure = SinglePhaseRecycleSolver
        .solve_controlled(
            &flow,
            &properties(),
            &options,
            &mut |progress: crate::partitioned::PartitionedProgress| {
                if progress.stage == PartitionedStage::VerificationStarted {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        )
        .unwrap_err();
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("solver.single_phase_recycle.cancelled")
    );
    assert_eq!(failure.stage.stage, PartitionedStage::VerificationStarted);
    assert!(
        failure
            .trace
            .iter()
            .all(|row| row.material.phase == EvaluationPhase::Iteration)
    );
}
