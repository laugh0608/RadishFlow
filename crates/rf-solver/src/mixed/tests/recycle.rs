use super::*;
use crate::{
    equation_oriented::GlobalMaterialEvidence,
    partitioned::{ABSOLUTE_FLOW_TOLERANCE, EvaluationPhase, SCALED_FLOW_TOLERANCE},
    simultaneous::{ModularOptions, SimultaneousModularSolver},
};
use rf_types::DiagnosticPortTarget;
use std::cell::Cell;

fn recycle_case() -> (Flowsheet, MixedOptions) {
    let (flow, mut options) = case();
    options.blocks.last_mut().unwrap().members = ["ma".into(), "sa".into()].into();
    for unit in ["mb", "sb"] {
        options.blocks.push(MixedBlock {
            id: unit.into(),
            members: [unit.into()].into(),
            method: MixedBlockMethod::Local,
        });
    }
    options.recycle = Some(Box::new(MixedRecycleConfiguration::Single(
        MixedRecycleOptions::new(TearSpecification {
            stream_id: "ba".into(),
            source: DiagnosticPortTarget::new("sb", "outlet_a"),
            sink: DiagnosticPortTarget::new("ma", "inlet_b"),
            initial_component_flows_mol_s: None,
            component_scales_mol_s: None,
        }),
    )));
    (flow, options)
}

#[test]
fn explicit_tear_preserves_the_engineering_graph_and_enforces_supported_bindings() {
    let (flow, options) = recycle_case();
    let plan = assemble(&flow, &options).unwrap();
    assert_eq!(plan.flow, flow);
    assert_eq!(
        plan.blocks()
            .iter()
            .map(|b| b.id.as_str())
            .collect::<Vec<_>>(),
        ["feed-a", "feed-b", "equations", "mb", "sb", "heater"]
    );
    let layout = plan.equation_layout(&"equations".into()).unwrap();
    assert_eq!((layout.variables.len(), layout.equations.len()), (12, 12));
    assert!(
        plan.connections()
            .iter()
            .any(|c| c.stream_id == "ba".into())
    );

    let mut no_tear = options.clone();
    no_tear.recycle = None;
    assert_eq!(
        assemble(&flow, &no_tear)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("solver.mixed.cross_block_cycle")
    );
    for (stream, source, sink, code) in [
        (
            "absent",
            ("sb", "outlet_a"),
            ("ma", "inlet_b"),
            "solver.mixed.missing_tear",
        ),
        (
            "ba",
            ("sb", "outlet_b"),
            ("ma", "inlet_b"),
            "solver.mixed.tear_binding",
        ),
        (
            "ab",
            ("sa", "outlet_a"),
            ("mb", "inlet_b"),
            "solver.mixed.tear_role",
        ),
    ] {
        let mut invalid = options.clone();
        let tear = &mut invalid.single_recycle_mut().unwrap().tear;
        tear.stream_id = stream.into();
        tear.source = DiagnosticPortTarget::new(source.0, source.1);
        tear.sink = DiagnosticPortTarget::new(sink.0, sink.1);
        let error = assemble(&flow, &invalid).unwrap_err();
        assert_eq!(error.context().diagnostic_code(), Some(code));
        assert!(
            error
                .context()
                .related_stream_ids()
                .contains(&StreamId::new(stream))
        );
    }
    let mut extra_cycle = flow.clone();
    let mut invalid = options.clone();
    for (unit_id, incoming, outgoing) in [
        ("cycle-a", "other-b", "other-a"),
        ("cycle-b", "other-a", "other-b"),
    ] {
        let mut stream = flow.streams[&"heated".into()].clone();
        stream.id = outgoing.into();
        extra_cycle.insert_stream(stream).unwrap();
        extra_cycle
            .insert_unit(build_heater_node(unit_id, unit_id, incoming, outgoing))
            .unwrap();
        invalid.blocks.push(MixedBlock {
            id: unit_id.into(),
            members: [unit_id.into()].into(),
            method: MixedBlockMethod::Local,
        });
    }
    assert_eq!(
        assemble(&extra_cycle, &invalid)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("solver.mixed.cross_block_cycle")
    );

    let mut invalid = options;
    let init = &mut invalid
        .single_recycle_mut()
        .unwrap()
        .tear
        .initial_component_flows_mol_s;
    *init = Some([("methane".into(), 1.0)].into());
    assert!(assemble(&flow, &invalid).is_err());
    *invalid
        .single_recycle_mut()
        .unwrap()
        .tear
        .initial_component_flows_mol_s
        .as_mut()
        .unwrap()
        .get_mut(&"methane".into())
        .unwrap() = f64::NAN;
    assert!(assemble(&flow, &invalid).is_err());
}

#[test]
fn both_initial_vectors_match_fraction_references_and_real_qs2() {
    let (flow, mut options) = recycle_case();
    let before = flow.clone();
    let reference = SimultaneousModularSolver
        .solve(
            &crate::SolverServices {
                thermo: &binary_hydrocarbon_provider(),
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &ModularOptions::new(options.boundary),
        )
        .unwrap();
    for (initial, evaluations) in [(None, 13), (Some([10.0, 1.0]), 15)] {
        options
            .single_recycle_mut()
            .unwrap()
            .tear
            .initial_component_flows_mol_s =
            initial.map(|[a, b]| [("methane".into(), a), ("ethane".into(), b)].into());
        let result = solve(&flow, &options).unwrap();
        assert_eq!(result.method, RECYCLE_METHOD);
        assert_eq!(result.blocks.len(), 6);
        assert_eq!(result.streams.len(), flow.streams.len());
        for (id, expected) in [
            ("fa-out", [8.0 / 5.0, 2.0 / 5.0]),
            ("fb-out", [3.0 / 5.0, 12.0 / 5.0]),
            ("ma-out", [35.0 / 17.0, 20.0 / 17.0]),
            ("mb-out", [156.0 / 85.0, 264.0 / 85.0]),
            ("ab", [21.0 / 17.0, 12.0 / 17.0]),
            ("ba", [39.0 / 85.0, 66.0 / 85.0]),
            ("pa", [14.0 / 17.0, 8.0 / 17.0]),
            ("pb", [117.0 / 85.0, 198.0 / 85.0]),
            ("heated", [14.0 / 17.0, 8.0 / 17.0]),
        ] {
            assert_stream(&result, id, expected);
            let stream_id = StreamId::new(id);
            let actual = &result.streams[&stream_id];
            let qs2 = &reference.streams[&stream_id];
            for (component, flow) in ["methane", "ethane"].into_iter().zip(expected) {
                let component = ComponentId::new(component);
                let x = actual.overall_mole_fractions[&component];
                assert!((x - flow / expected.iter().sum::<f64>()).abs() <= 1e-10);
                let n_qs2 = qs2.total_molar_flow_mol_s * qs2.overall_mole_fractions[&component];
                assert!((x * actual.total_molar_flow_mol_s - n_qs2).abs() <= 1e-9);
            }
        }
        let evidence = result.recycle.as_ref().unwrap();
        assert_eq!(evidence.fresh_stream, "fa-out".into());
        assert!(evidence.component_scales_mol_s.values().all(|v| *v == 5.0));
        assert_eq!(evidence.residual_trace.len(), evaluations + 1);
        let verified = evidence.residual_trace.last().unwrap();
        assert_eq!(verified.phase, EvaluationPhase::Verification);
        assert!(verified.max_absolute_mol_s <= ABSOLUTE_FLOW_TOLERANCE);
        assert!(verified.max_scaled <= SCALED_FLOW_TOLERANCE);
        let last = evidence.last_sweep.as_ref().unwrap();
        let edge = result
            .boundaries
            .iter()
            .find(|b| b.connection.stream_id == "ba".into())
            .unwrap();
        assert_eq!(edge.producer, result.streams[&"ba".into()]);
        assert_eq!(edge.consumed, last.assumed);
        assert_eq!(edge.producer, last.returned);
        assert_ne!(
            edge.producer.total_molar_flow_mol_s,
            edge.consumed.total_molar_flow_mol_s
        );
        let mixer = result
            .blocks
            .iter()
            .flat_map(|b| b.evidence.final_units())
            .find(|unit| unit.unit_id == "ma".into())
            .unwrap();
        assert!(mixer.consumed_streams.contains(&edge.consumed));
        let GlobalMaterialEvidence {
            component_residuals_mol_s,
            total_residual_mol_s,
            ..
        } = &result.global_material;
        assert!(component_residuals_mol_s.values().all(|r| r.abs() <= 1e-10));
        assert!(total_residual_mol_s.abs() <= 2e-10);
        for unit in ["fa", "fb", "h1"] {
            assert_eq!(result.costs.calls_by_unit[&unit.into()], 1);
        }
        for unit in ["ma", "sa", "mb", "sb"] {
            assert_eq!(result.costs.calls_by_unit[&unit.into()], evaluations + 1);
        }
    }
    assert_eq!(flow, before);
}

#[test]
fn iteration_limit_returns_last_real_residual_without_running_downstream() {
    let (flow, mut options) = recycle_case();
    options
        .single_recycle_mut()
        .unwrap()
        .max_iteration_evaluations = 1;
    let failure = solve(&flow, &options).unwrap_err();
    assert_eq!(failure.kind, MixedFailureKind::IterationLimit);
    assert!(!failure.costs.calls_by_unit.contains_key(&"h1".into()));
    let report = failure.recycle.unwrap();
    assert_eq!(report.residual_trace.len(), 1);
    let sweep = report.last_sweep.unwrap();
    assert_eq!(sweep.assumed.total_molar_flow_mol_s, 0.0);
    assert_eq!(
        sweep.assumed.overall_mole_fractions,
        flow.streams[&"fa-out".into()].overall_mole_fractions
    );
    assert!((sweep.residuals_mol_s[&"methane".into()] - 0.39).abs() < 1e-12);
    assert!((sweep.residuals_mol_s[&"ethane".into()] - 0.66).abs() < 1e-12);
}

struct SplitFault {
    inner: Box<dyn UnitOperation>,
    pressure_drift: bool,
}

impl UnitOperation for SplitFault {
    fn kind(&self) -> rf_unitops::BuiltinUnitKind {
        self.inner.kind()
    }

    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        let actual = self.inner.run(services, inputs)?;
        let mut outputs = UnitOperationOutputs::new();
        for (port, sign) in [("outlet_a", 1.0), ("outlet_b", -1.0)] {
            let mut stream = actual.stream(port).unwrap().clone();
            if self.pressure_drift {
                stream.pressure_pa += 1e-3;
            } else {
                // A conservative but incorrect split must be caught by tear verification.
                stream.total_molar_flow_mol_s += sign * 1e-4;
            }
            outputs.insert_material_stream(port, stream);
        }
        Ok(outputs)
    }
}

#[test]
fn boundary_drift_and_failed_final_sweep_are_distinct_and_stop_downstream() {
    let (flow, options) = recycle_case();
    for drift in [false, true] {
        let verification = Cell::new(false);
        let failure = execution::solve(
            &crate::SolverServices {
                thermo: &binary_hydrocarbon_provider(),
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &options,
            &mut |p: MixedProgress| {
                verification.set(
                    p.recycle
                        .is_some_and(|p| p.phase == EvaluationPhase::Verification),
                );
                ControlFlow::Continue(())
            },
            &mut |u, streams| {
                let inner = rf_unitops::catalog::instantiate_builtin(u, streams)?;
                Ok(if u.id == "sb".into() && (drift || verification.get()) {
                    Box::new(SplitFault {
                        inner,
                        pressure_drift: drift,
                    })
                } else {
                    inner
                })
            },
        )
        .unwrap_err();
        assert!(!failure.costs.calls_by_unit.contains_key(&"h1".into()));
        if drift {
            assert_eq!(
                failure.error.context().diagnostic_code(),
                Some("solver.partitioned.fixed_boundary")
            );
            assert_eq!(failure.progress.unit, Some("sb".into()));
        } else {
            assert_eq!(failure.kind, MixedFailureKind::RecycleVerification);
            let report = failure.recycle.unwrap();
            let sample = report.residual_trace.last().unwrap();
            assert_eq!(sample.phase, EvaluationPhase::Verification);
            assert!(sample.max_absolute_mol_s > 1e-10);
        }
    }
}

#[test]
fn local_faults_and_global_balance_still_gate_publication_after_recycle() {
    let (mut flow, mut options) = recycle_case();
    for (id, incoming, outgoing) in [("h2", "heated", "heated-2"), ("h3", "heated-2", "heated-3")] {
        let mut stream = flow.streams[&"heated".into()].clone();
        stream.id = outgoing.into();
        flow.insert_stream(stream).unwrap();
        let mut unit = build_heater_node(id, id, incoming, outgoing);
        unit.parameters.outlet_temperature_k = Some(300.0);
        unit.parameters.outlet_pressure_pa = Some(650_000.0);
        flow.insert_unit(unit).unwrap();
        options.blocks.push(MixedBlock {
            id: id.into(),
            members: [id.into()].into(),
            method: MixedBlockMethod::Local,
        });
    }
    for (target, expected) in [
        (Some("mb"), MixedFailureKind::LocalEvaluation),
        (Some("h1"), MixedFailureKind::LocalEvaluation),
        (None, MixedFailureKind::GlobalVerification),
    ] {
        let failure = execution::solve(
            &crate::SolverServices {
                thermo: &binary_hydrocarbon_provider(),
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &options,
            &mut |_| ControlFlow::Continue(()),
            &mut |unit, streams| {
                if target == Some(unit.id.as_str()) {
                    return Err(RfError::invalid_input("test local fault")
                        .with_related_unit_id(unit.id.clone()));
                }
                let inner = rf_unitops::catalog::instantiate_builtin(unit, streams)?;
                Ok(if target.is_none() && unit.kind == "heater" {
                    Box::new(Faulty {
                        inner,
                        wrong_identity: false,
                        delta: 1e-10,
                    })
                } else {
                    inner
                })
            },
        )
        .unwrap_err();
        assert_eq!(failure.kind, expected);
        if target == Some("mb") {
            assert!(!failure.costs.calls_by_unit.contains_key(&"h1".into()));
        } else {
            let trace = &failure.recycle.as_ref().unwrap().residual_trace;
            assert_eq!(trace.last().unwrap().phase, EvaluationPhase::Verification);
            assert!(trace.last().unwrap().max_absolute_mol_s <= ABSOLUTE_FLOW_TOLERANCE);
        }
        if target.is_none() {
            assert_eq!(failure.progress.stage, MixedStage::GlobalVerification);
            assert_eq!(failure.costs.calls_by_unit[&"h3".into()], 1);
        }
    }
}

#[test]
fn cycle_and_downstream_checkpoints_are_cooperatively_cancelled() {
    let (flow, options) = recycle_case();
    for (stage, phase, block) in [
        (
            MixedStage::BlockStarted,
            Some(EvaluationPhase::Iteration),
            None,
        ),
        (
            MixedStage::Equation,
            Some(EvaluationPhase::Iteration),
            Some("equations"),
        ),
        (
            MixedStage::Boundary,
            Some(EvaluationPhase::Iteration),
            Some("mb"),
        ),
        (
            MixedStage::BlockCompleted,
            Some(EvaluationPhase::Verification),
            Some("sb"),
        ),
        (MixedStage::BlockStarted, None, Some("heater")),
        (MixedStage::GlobalVerification, None, None),
        (MixedStage::Completing, None, None),
    ] {
        let failure = MixedSolver
            .solve_controlled(
                &crate::SolverServices {
                    thermo: &binary_hydrocarbon_provider(),
                    flash_solver: &PlaceholderTpFlashSolver,
                },
                &flow,
                &options,
                &mut |progress: MixedProgress| {
                    if progress.stage == stage
                        && progress.recycle.map(|p| p.phase) == phase
                        && progress.block.as_ref().map(MixedBlockId::as_str) == block
                    {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                },
            )
            .unwrap_err();
        assert_eq!(failure.kind, MixedFailureKind::Cancelled);
        assert_eq!(failure.progress.stage, stage);
    }
}
