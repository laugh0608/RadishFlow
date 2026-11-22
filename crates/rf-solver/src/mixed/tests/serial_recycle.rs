use super::*;
use crate::partitioned::{EvaluationPhase, TearSpecification};
use rf_store::project_v4::{ProjectV4Read, parse};
use rf_types::DiagnosticPortTarget;
use std::cell::Cell;

fn case() -> (Flowsheet, MixedOptions) {
    let ProjectV4Read::Editable(project) = parse(include_str!(
        "../../../../../examples/flowsheets/fixed-tp-serial-recycle-eo-reference.rfproj.json"
    ))
    .unwrap() else {
        panic!("EO reference must be readable");
    };
    let flow = project.document.flowsheet;
    let mut options = MixedOptions {
        boundary: FixedRecycleBoundary {
            temperature_k: 300.0,
            pressure_pa: 650_000.0,
        },
        blocks: [
            "feed-a",
            "feed-b",
            "mixer-makeup-b",
            "splitter-recycle-a",
            "splitter-recycle-b",
        ]
        .into_iter()
        .map(|id| MixedBlock {
            id: id.into(),
            members: [id.into()].into(),
            method: MixedBlockMethod::Local,
        })
        .collect(),
        recycle: None,
    };
    let regions = ["a", "b"].map(|side| {
        let mixer = UnitId::new(format!("mixer-{side}"));
        options.blocks.push(MixedBlock {
            id: MixedBlockId::new(format!("equations-{side}")),
            members: [
                mixer.clone(),
                UnitId::new(format!("splitter-primary-{side}")),
            ]
            .into(),
            method: MixedBlockMethod::EquationOriented { scales: None },
        });
        MixedRecycleOptions::new(TearSpecification {
            stream_id: StreamId::new(format!("s-r{side}")),
            source: DiagnosticPortTarget::new(format!("splitter-recycle-{side}"), "outlet_a"),
            sink: DiagnosticPortTarget::new(mixer, "inlet_b"),
            initial_component_flows_mol_s: None,
            component_scales_mol_s: None,
        })
    });
    options.recycle = Some(Box::new(MixedRecycleConfiguration::Serial(regions)));
    (flow, options)
}

fn regions_mut(options: &mut MixedOptions) -> &mut [MixedRecycleOptions; 2] {
    let Some(MixedRecycleConfiguration::Serial(regions)) = options.recycle.as_deref_mut() else {
        panic!("serial fixture");
    };
    regions
}

#[test]
fn assembly_uses_ports_and_uncut_regions_and_canonicalizes_plan_order() {
    let (flow, mut options) = case();
    let plan = assemble(&flow, &options).unwrap();
    assert_eq!(
        plan.blocks()
            .iter()
            .map(|b| b.id.as_str())
            .collect::<Vec<_>>(),
        [
            "feed-a",
            "equations-a",
            "splitter-recycle-a",
            "feed-b",
            "mixer-makeup-b",
            "equations-b",
            "splitter-recycle-b"
        ]
    );
    for id in ["equations-a", "equations-b"] {
        let layout = plan.equation_layout(&id.into()).unwrap();
        assert_eq!((layout.variables.len(), layout.equations.len()), (12, 12));
    }
    options.blocks.reverse();
    regions_mut(&mut options).reverse();
    assert_eq!(assemble(&flow, &options).unwrap(), plan);
    assert_eq!(
        options.referenced_stream_ids(),
        ["s-ra".into(), "s-rb".into()].into()
    );

    let mut wrong = options.clone();
    regions_mut(&mut wrong)[0].tear.source.port_name = "outlet_b".into();
    let error = assemble(&flow, &wrong).unwrap_err();
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.mixed.tear_binding")
    );
    assert!(
        error
            .context()
            .related_stream_ids()
            .contains(&"s-rb".into())
    );

    let mut duplicate = options.clone();
    let regions = regions_mut(&mut duplicate);
    regions[0] = regions[1].clone();
    assert_eq!(
        assemble(&flow, &duplicate)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("solver.mixed.duplicate_tear")
    );

    let mut outside = options.clone();
    let tear = &mut regions_mut(&mut outside)[0].tear;
    tear.stream_id = "s-ab".into();
    tear.source = DiagnosticPortTarget::new("splitter-recycle-a", "outlet_b");
    tear.sink = DiagnosticPortTarget::new("mixer-makeup-b", "inlet_a");
    assert_eq!(
        assemble(&flow, &outside)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("solver.mixed.tear_role")
    );

    let mut missing = options.clone();
    missing.recycle = Some(Box::new(MixedRecycleConfiguration::Single(
        regions_mut(&mut missing)[1].clone(),
    )));
    assert_eq!(
        assemble(&flow, &missing)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("solver.mixed.cross_block_cycle")
    );

    let mut reverse = flow.clone();
    reverse
        .units
        .get_mut(&"mixer-a".into())
        .unwrap()
        .ports
        .iter_mut()
        .find(|p| p.name == "inlet_a")
        .unwrap()
        .stream_id = Some("s-qb".into());
    assert_eq!(
        assemble(&reverse, &options)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("solver.mixed.serial_regions")
    );

    let mut bad_initial = options;
    regions_mut(&mut bad_initial)[0]
        .tear
        .initial_component_flows_mol_s = Some([("methane".into(), 1.0)].into());
    assert!(assemble(&flow, &bad_initial).is_err());
}

#[test]
fn two_initial_vectors_verify_real_handoff_and_every_analytical_stream() {
    let (flow, mut options) = case();
    for (initial, counts) in [(None, [14, 36]), (Some([[3.0, 1.0], [2.0, 3.0]]), [15, 35])] {
        for (i, region) in regions_mut(&mut options).iter_mut().enumerate() {
            region.tear.initial_component_flows_mol_s = initial.map(|values| {
                [
                    ("methane".into(), values[i][0]),
                    ("ethane".into(), values[i][1]),
                ]
                .into()
            });
        }
        let result = solve(&flow, &options).unwrap();
        assert_eq!(result.method, SERIAL_RECYCLE_METHOD);
        assert!(result.recycle.is_none());
        assert_eq!(result.streams.len(), 13);
        for (id, methane, ethane) in [
            ("s-fa", 3.0, 2.0),
            ("s-fb", 1.0, 4.0),
            ("s-ma", 18.0 / 5.0, 12.0 / 5.0),
            ("s-sa", 9.0 / 5.0, 6.0 / 5.0),
            ("s-ra", 3.0 / 5.0, 2.0 / 5.0),
            ("s-pa", 9.0 / 5.0, 6.0 / 5.0),
            ("s-ab", 6.0 / 5.0, 4.0 / 5.0),
            ("s-bf", 11.0 / 5.0, 24.0 / 5.0),
            ("s-mb", 22.0 / 5.0, 48.0 / 5.0),
            ("s-sb", 44.0 / 15.0, 32.0 / 5.0),
            ("s-rb", 11.0 / 5.0, 24.0 / 5.0),
            ("s-pb", 22.0 / 15.0, 16.0 / 5.0),
            ("s-qb", 11.0 / 15.0, 8.0 / 5.0),
        ] {
            assert_stream(&result, id, [methane, ethane]);
            let actual = &result.streams[&id.into()];
            assert!(
                (actual.overall_mole_fractions[&"methane".into()] - methane / (methane + ethane))
                    .abs()
                    <= 1e-10
            );
            assert!((actual.temperature_k - 300.0).abs() <= 1e-9);
            assert!((actual.pressure_pa - 650_000.0).abs() <= 1e-6);
        }
        for (index, report) in result.serial_recycles.iter().enumerate() {
            let sweep = report.recycle.last_sweep.as_ref().unwrap();
            assert_eq!(
                sweep.returned,
                result.streams[&report.recycle.tear.stream_id]
            );
            assert_ne!(sweep.assumed, sweep.returned);
            assert_eq!(report.recycle.residual_trace.len(), counts[index] + 1);
            let last = report.recycle.residual_trace.last().unwrap();
            assert_eq!(last.phase, EvaluationPhase::Verification);
            assert!(last.max_absolute_mol_s <= 1e-10 && last.max_scaled <= 1e-10);
            assert_balance(report.balance.as_ref().unwrap());
            let inlet = &result.streams[&report.recycle.fresh_stream];
            assert!(
                report
                    .recycle
                    .component_scales_mol_s
                    .values()
                    .all(|s| *s == inlet.total_molar_flow_mol_s)
            );
        }
        assert_balance(&result.global_material);
        for (id, count) in [
            ("feed-a", 1),
            ("feed-b", 1),
            ("mixer-makeup-b", 1),
            ("mixer-a", counts[0] + 1),
            ("mixer-b", counts[1] + 1),
        ] {
            assert_eq!(result.costs.calls_by_unit[&id.into()], count);
        }
        for id in ["s-ab", "s-bf"] {
            let evidence = result
                .boundaries
                .iter()
                .find(|b| b.connection.stream_id == id.into())
                .unwrap();
            assert_eq!(evidence.producer, result.streams[&id.into()]);
            assert_eq!(evidence.producer, evidence.consumed);
        }
    }
}

fn assert_balance(balance: &crate::equation_oriented::GlobalMaterialEvidence) {
    assert!(
        balance
            .component_residuals_mol_s
            .values()
            .all(|v| v.abs() <= 1e-10)
    );
    assert!(balance.total_residual_mol_s.abs() <= 2e-10);
}

#[test]
fn composition_and_upstream_split_perturbations_follow_real_transferred_material() {
    let (flow, options) = case();
    let mut makeup = flow.clone();
    makeup
        .streams
        .get_mut(&"s-fb".into())
        .unwrap()
        .overall_mole_fractions = [("methane".into(), 0.8), ("ethane".into(), 0.2)].into();
    let result = solve(&makeup, &options).unwrap();
    assert_stream(&result, "s-ab", [6.0 / 5.0, 4.0 / 5.0]);
    assert_stream(&result, "s-rb", [26.0 / 5.0, 9.0 / 5.0]);
    let mut split = flow;
    split
        .units
        .get_mut(&"splitter-recycle-a".into())
        .unwrap()
        .parameters
        .outlet_a_split_fraction = Some(2.0 / 3.0);
    let result = solve(&split, &options).unwrap();
    assert_stream(&result, "s-ra", [3.0 / 2.0, 1.0]);
    assert_stream(&result, "s-ab", [3.0 / 4.0, 1.0 / 2.0]);
    assert_stream(&result, "s-rb", [7.0 / 4.0, 9.0 / 2.0]);
}

fn assert_no_downstream_calls(failure: &MixedFailure) {
    for id in [
        "feed-b",
        "mixer-makeup-b",
        "mixer-b",
        "splitter-primary-b",
        "splitter-recycle-b",
    ] {
        assert!(
            !failure.costs.calls_by_unit.contains_key(&id.into()),
            "{id}"
        );
    }
}

#[test]
fn early_exhaustion_and_nonpositive_inlet_never_start_downstream() {
    let (flow, options) = case();
    for region_index in 0..2 {
        let mut limited = options.clone();
        regions_mut(&mut limited)[region_index].max_iteration_evaluations = 1;
        let failure = solve(&flow, &limited).unwrap_err();
        assert_eq!(failure.kind, MixedFailureKind::IterationLimit);
        assert_eq!(failure.serial_recycles.len(), region_index + 1);
        if region_index == 0 {
            assert_no_downstream_calls(&failure);
        } else {
            assert_balance(failure.serial_recycles[0].balance.as_ref().unwrap());
            assert!(failure.serial_recycles[1].balance.is_none());
        }
    }
    let mut zero = flow;
    zero.streams
        .get_mut(&"s-fa".into())
        .unwrap()
        .total_molar_flow_mol_s = 0.0;
    let failure = solve(&zero, &options).unwrap_err();
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("solver.mixed.serial_inlet")
    );
    assert_no_downstream_calls(&failure);
}

#[derive(Clone, Copy)]
enum InjectedFault {
    ShiftSplit,
    TransferPressure,
    ProductLoss,
}

struct FaultyOperation {
    inner: Box<dyn UnitOperation>,
    fault: InjectedFault,
}

impl UnitOperation for FaultyOperation {
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
        match self.fault {
            InjectedFault::TransferPressure => {
                let mut stream = actual.stream("outlet").unwrap().clone();
                stream.pressure_pa += 1e-3;
                outputs.insert_material_stream("outlet", stream);
            }
            InjectedFault::ShiftSplit | InjectedFault::ProductLoss => {
                for (port, sign) in [("outlet_a", 1.0), ("outlet_b", -1.0)] {
                    let mut stream = actual.stream(port).unwrap().clone();
                    match self.fault {
                        InjectedFault::ShiftSplit => stream.total_molar_flow_mol_s += sign * 1e-4,
                        InjectedFault::ProductLoss if port == "outlet_b" => {
                            stream.total_molar_flow_mol_s -= 9e-11
                        }
                        _ => {}
                    }
                    outputs.insert_material_stream(port, stream);
                }
            }
        }
        Ok(outputs)
    }
}

#[test]
fn final_upstream_recheck_and_transfer_failures_stop_before_b() {
    let (flow, options) = case();
    for fault in [InjectedFault::ShiftSplit, InjectedFault::TransferPressure] {
        let final_upstream_sweep = Cell::new(false);
        let failure = execution::solve(
            &crate::SolverServices {
                thermo: &binary_hydrocarbon_provider(),
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &options,
            &mut |progress: MixedProgress| {
                final_upstream_sweep.set(
                    progress
                        .recycle
                        .is_some_and(|r| r.phase == EvaluationPhase::Verification)
                        && progress.recycle_region == Some("equations-a".into()),
                );
                ControlFlow::Continue(())
            },
            &mut |unit, streams| {
                let inner = rf_unitops::catalog::instantiate_builtin(unit, streams)?;
                let inject = match fault {
                    InjectedFault::ShiftSplit => {
                        unit.id.as_str() == "splitter-recycle-a" && final_upstream_sweep.get()
                    }
                    InjectedFault::TransferPressure => unit.id.as_str() == "mixer-makeup-b",
                    InjectedFault::ProductLoss => unreachable!(),
                };
                Ok(if inject {
                    Box::new(FaultyOperation { inner, fault })
                } else {
                    inner
                })
            },
        )
        .unwrap_err();
        assert_eq!(failure.serial_recycles.len(), 1);
        assert!(!failure.costs.calls_by_unit.contains_key(&"mixer-b".into()));
        match fault {
            InjectedFault::ShiftSplit => {
                assert_no_downstream_calls(&failure);
                assert_eq!(failure.kind, MixedFailureKind::RecycleVerification);
                assert!(failure.serial_recycles[0].balance.is_none());
                let sample = failure.serial_recycles[0]
                    .recycle
                    .residual_trace
                    .last()
                    .unwrap();
                assert_eq!(sample.phase, EvaluationPhase::Verification);
                assert!(sample.max_absolute_mol_s > 1e-10);
            }
            InjectedFault::TransferPressure => {
                assert_balance(failure.serial_recycles[0].balance.as_ref().unwrap());
                assert_eq!(failure.progress.block, Some("mixer-makeup-b".into()));
                assert_eq!(
                    failure.error.context().diagnostic_code(),
                    Some("solver.partitioned.fixed_boundary")
                );
            }
            InjectedFault::ProductLoss => unreachable!(),
        }
    }
}

#[test]
fn conservation_gates_reject_accumulated_drift_after_tear_closure() {
    let (flow, options) = case();
    for (other, expected) in [
        ("splitter-recycle-a", MixedFailureKind::RecycleVerification),
        ("splitter-primary-b", MixedFailureKind::GlobalVerification),
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
                let inner = rf_unitops::catalog::instantiate_builtin(unit, streams)?;
                Ok(
                    if ["splitter-primary-a", other].contains(&unit.id.as_str()) {
                        Box::new(FaultyOperation {
                            inner,
                            fault: InjectedFault::ProductLoss,
                        })
                    } else {
                        inner
                    },
                )
            },
        )
        .unwrap_err();
        assert_eq!(failure.kind, expected);
        assert_eq!(
            failure.error.context().diagnostic_code(),
            Some("solver.partitioned.component_balance")
        );
        let reports = &failure.serial_recycles;
        for report in reports {
            assert!(
                report
                    .recycle
                    .residual_trace
                    .last()
                    .unwrap()
                    .max_absolute_mol_s
                    <= 1e-10
            );
        }
        if expected == MixedFailureKind::RecycleVerification {
            assert_eq!(failure.progress.stage, MixedStage::RegionVerification);
            assert_no_downstream_calls(&failure);
            assert_eq!(reports.len(), 1);
            assert!(
                reports[0]
                    .balance
                    .as_ref()
                    .unwrap()
                    .component_residuals_mol_s
                    .values()
                    .any(|v| v.abs() > 1e-10)
            );
        } else {
            assert_eq!(reports.len(), 2);
            for report in reports {
                assert_balance(report.balance.as_ref().unwrap());
            }
        }
    }
}

#[test]
fn cancel_at_region_verification_or_transfer_never_completes() {
    let (flow, options) = case();
    for (stage, block) in [
        (MixedStage::RegionVerification, "equations-a"),
        (MixedStage::BlockStarted, "mixer-makeup-b"),
        (MixedStage::RegionVerification, "equations-b"),
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
                        && progress
                            .block
                            .as_ref()
                            .is_some_and(|id| id.as_str() == block)
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
        if block != "equations-b" {
            assert!(!failure.costs.calls_by_unit.contains_key(&"mixer-b".into()));
        }
    }
}
