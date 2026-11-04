use super::*;
use crate::equation_oriented::{EquationBinding, EquationBlockInput, assemble_block};
use crate::test_fixtures::{binary_hydrocarbon_provider, cross_recycle};
use rf_flash::PlaceholderTpFlashSolver;
use rf_model::MaterialStreamState;
use rf_types::{ComponentId, StreamId};
use rf_unitops::{
    UnitOperation, UnitOperationInputs, UnitOperationOutputs, UnitOperationServices,
    build_heater_node,
};
use std::ops::ControlFlow;

pub(super) fn case() -> (Flowsheet, MixedOptions) {
    let mut flow = cross_recycle(0.6, 0.25);
    flow.insert_stream(MaterialStreamState::from_tpzf(
        "heated",
        "Heated",
        300.0,
        650000.0,
        99.0,
        [("methane".into(), 0.9), ("ethane".into(), 0.1)].into(),
    ))
    .unwrap();
    let mut heater = build_heater_node("h1", "H1", "pa", "heated");
    heater.parameters.outlet_temperature_k = Some(300.0);
    heater.parameters.outlet_pressure_pa = Some(650000.0);
    flow.insert_unit(heater).unwrap();
    let mut blocks: Vec<_> = [("feed-a", "fa"), ("feed-b", "fb"), ("heater", "h1")]
        .into_iter()
        .map(|(id, unit)| MixedBlock {
            id: id.into(),
            members: [unit.into()].into(),
            method: MixedBlockMethod::Local,
        })
        .collect();
    blocks.push(MixedBlock {
        id: "equations".into(),
        members: ["ma", "mb", "sa", "sb"]
            .into_iter()
            .map(UnitId::new)
            .collect(),
        method: MixedBlockMethod::EquationOriented { scales: None },
    });
    (
        flow,
        MixedOptions {
            boundary: FixedRecycleBoundary {
                temperature_k: 300.0,
                pressure_pa: 650000.0,
            },
            blocks,
        },
    )
}
fn solve(flow: &Flowsheet, o: &MixedOptions) -> Result<MixedResult, Box<MixedFailure>> {
    MixedSolver.solve(
        &crate::SolverServices {
            thermo: &binary_hydrocarbon_provider(),
            flash_solver: &PlaceholderTpFlashSolver,
        },
        flow,
        o,
    )
}
fn assert_stream(r: &MixedResult, id: &str, expected: [f64; 2]) {
    let s = &r.streams[&StreamId::new(id)];
    for (c, n) in ["methane", "ethane"].into_iter().zip(expected) {
        assert!(
            (s.total_molar_flow_mol_s * s.overall_mole_fractions[&ComponentId::new(c)] - n).abs()
                < 1e-9,
            "{id} / {c}"
        );
    }
}
#[test]
fn explicit_dag_and_external_equations_keep_original_boundary_identities() {
    let (flow, o) = case();
    let before = flow.clone();
    let plan = assemble(&flow, &o).unwrap();
    assert_eq!(
        plan.blocks()
            .iter()
            .map(|b| b.id.as_str())
            .collect::<Vec<_>>(),
        ["feed-a", "feed-b", "equations", "heater"]
    );
    let members = &o.blocks.last().unwrap().members;
    let inputs = [
        ("fa-out".into(), flow.streams[&"fa-out".into()].clone()),
        ("fb-out".into(), flow.streams[&"fb-out".into()].clone()),
    ]
    .into();
    let input = EquationBlockInput {
        flowsheet: &flow,
        members,
        inputs: &inputs,
    };
    let p = assemble_block(
        &input,
        &o.blocks
            .last()
            .unwrap()
            .equation_options(o.boundary)
            .unwrap(),
    )
    .unwrap();
    assert_eq!((p.variables().len(), p.equations().len()), (24, 24));
    p.require_ready().unwrap();
    assert_eq!(
        p.equations()
            .iter()
            .filter(|e| matches!(e, EquationBinding::Boundary { .. }))
            .count(),
        4
    );
    assert!(!p.variables().iter().any(|v|matches!(v,crate::equation_oriented::VariableBinding::PortComponent{port,..} if port.unit==UnitId::new("fa"))));
    let mut reverse = o.clone();
    reverse.blocks.reverse();
    assert_eq!(assemble(&flow, &reverse).unwrap(), plan);
    assert_eq!(flow, before);
}
#[test]
fn mixed_m1_matches_independent_fractions_and_whole_modular_reference() {
    let (flow, o) = case();
    let before = flow.clone();
    let r = solve(&flow, &o).unwrap();
    for (id, a, b) in [
        ("ma-out", 35.0 / 17.0, 20.0 / 17.0),
        ("mb-out", 156.0 / 85.0, 264.0 / 85.0),
        ("ab", 21.0 / 17.0, 12.0 / 17.0),
        ("ba", 39.0 / 85.0, 66.0 / 85.0),
        ("pa", 14.0 / 17.0, 8.0 / 17.0),
        ("heated", 14.0 / 17.0, 8.0 / 17.0),
        ("pb", 117.0 / 85.0, 198.0 / 85.0),
    ] {
        assert_stream(&r, id, [a, b]);
    }
    assert_eq!(r.blocks.len(), 4);
    assert_eq!(r.boundaries.len(), 3);
    assert_eq!(r.costs.local_unit_calls, 7);
    assert_eq!(r.costs.operation_constructions, 7);
    assert!(r.costs.thermo_calls > 0 && r.costs.flash_calls > 0);
    for b in &r.boundaries {
        assert_eq!(b.producer, b.consumed);
        assert!(b.residuals_mol_s.values().all(|r| r.abs() <= 1e-10));
    }
    assert!(
        r.global_material
            .component_residuals_mol_s
            .values()
            .all(|v| v.abs() <= 1e-10)
    );
    let modular = crate::simultaneous::SimultaneousModularSolver
        .solve(
            &crate::SolverServices {
                thermo: &binary_hydrocarbon_provider(),
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &crate::simultaneous::ModularOptions::new(o.boundary),
        )
        .unwrap();
    for (id, s) in &modular.streams {
        assert_stream(
            &r,
            id.as_str(),
            [
                s.total_molar_flow_mol_s * s.overall_mole_fractions[&"methane".into()],
                s.total_molar_flow_mol_s * s.overall_mole_fractions[&"ethane".into()],
            ],
        );
    }
    assert_eq!(flow, before);
}
#[test]
fn partitions_boundary_values_scales_and_capabilities_are_never_silently_repaired() {
    let (flow, o) = case();
    for change in 0..6 {
        let mut bad = o.clone();
        match change {
            0 => {
                bad.blocks.pop();
            }
            1 => {
                bad.blocks[0].members.insert("ma".into());
            }
            2 => {
                bad.blocks[0].id = bad.blocks[1].id.clone();
            }
            3 => {
                bad.blocks[0].members = ["missing".into()].into();
            }
            4 => {
                bad.blocks[2].method = MixedBlockMethod::EquationOriented { scales: None };
            }
            _ => {
                bad.blocks.last_mut().unwrap().members.remove(&"ma".into());
                bad.blocks.push(MixedBlock {
                    id: "split-scc".into(),
                    members: ["ma".into()].into(),
                    method: MixedBlockMethod::Local,
                });
            }
        }
        assert!(assemble(&flow, &bad).is_err(), "change {change}");
    }
    let members = &o.blocks.last().unwrap().members;
    for fault in 0..4 {
        let mut inputs: BTreeMap<_, _> = [
            ("fa-out".into(), flow.streams[&"fa-out".into()].clone()),
            ("fb-out".into(), flow.streams[&"fb-out".into()].clone()),
        ]
        .into();
        match fault {
            0 => {
                inputs.remove(&"fa-out".into());
            }
            1 => inputs.get_mut(&"fa-out".into()).unwrap().id = "wrong".into(),
            2 => inputs.get_mut(&"fa-out".into()).unwrap().temperature_k = 301.0,
            _ => inputs
                .get_mut(&"fa-out".into())
                .unwrap()
                .overall_mole_fractions
                .insert("unknown".into(), 0.0)
                .map(|_| ())
                .unwrap_or(()),
        };
        assert!(
            assemble_block(
                &EquationBlockInput {
                    flowsheet: &flow,
                    members,
                    inputs: &inputs
                },
                &o.blocks
                    .last()
                    .unwrap()
                    .equation_options(o.boundary)
                    .unwrap()
            )
            .is_err()
        );
    }
}
#[test]
fn cancellation_at_outer_and_nested_safety_points_returns_only_diagnostics() {
    let (flow, o) = case();
    for stage in [
        MixedStage::Assembly,
        MixedStage::BlockStarted,
        MixedStage::Equation,
        MixedStage::Boundary,
        MixedStage::BlockCompleted,
        MixedStage::GlobalVerification,
        MixedStage::Completing,
    ] {
        let e = MixedSolver
            .solve_controlled(
                &crate::SolverServices {
                    thermo: &binary_hydrocarbon_provider(),
                    flash_solver: &PlaceholderTpFlashSolver,
                },
                &flow,
                &o,
                &mut |p: MixedProgress| {
                    if p.stage == stage {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    }
                },
            )
            .unwrap_err();
        assert_eq!(e.kind, MixedFailureKind::Cancelled, "{stage:?}");
    }
}
struct Faulty {
    inner: Box<dyn UnitOperation>,
    wrong_identity: bool,
    delta: f64,
}
impl UnitOperation for Faulty {
    fn kind(&self) -> rf_unitops::BuiltinUnitKind {
        self.inner.kind()
    }
    fn run(
        &self,
        s: &UnitOperationServices<'_>,
        i: &UnitOperationInputs,
    ) -> rf_types::RfResult<UnitOperationOutputs> {
        let o = self.inner.run(s, i)?;
        let mut out = UnitOperationOutputs::new();
        let mut value = o.stream("outlet").unwrap().clone();
        if self.wrong_identity {
            value.id = "foreign".into();
        } else {
            value.total_molar_flow_mol_s += self.delta;
        }
        out.insert_material_stream("outlet", value);
        Ok(out)
    }
}
#[test]
fn upstream_and_late_local_faults_never_publish_partial_success() {
    let (flow, o) = case();
    let before = flow.clone();
    for (target, wrong_identity) in [("fa", true), ("fa", false), ("h1", true), ("h1", false)] {
        let e = execution::solve(
            &crate::SolverServices {
                thermo: &binary_hydrocarbon_provider(),
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &o,
            &mut |_| ControlFlow::Continue(()),
            &mut |u, streams| {
                let inner = rf_unitops::catalog::instantiate_builtin(u, streams)?;
                Ok(if u.id == UnitId::new(target) {
                    Box::new(Faulty {
                        inner,
                        wrong_identity,
                        delta: 0.01,
                    }) as Box<dyn UnitOperation>
                } else {
                    inner
                })
            },
        )
        .unwrap_err();
        assert_eq!(e.kind, MixedFailureKind::LocalEvaluation);
        if target == "fa" {
            assert_eq!(e.costs.local_unit_calls, 1);
            assert!(e.completed_blocks.is_empty());
        } else {
            assert_eq!(e.completed_blocks.len(), 3);
            assert_eq!(e.costs.local_unit_calls, 7);
        }
    }
    assert_eq!(flow, before);
}

#[test]
fn accumulated_local_roundoff_must_still_pass_global_material_verification() {
    let (mut flow, mut o) = case();
    for (id, input, output) in [("h2", "heated", "heated-2"), ("h3", "heated-2", "heated-3")] {
        let mut stream = flow.streams[&StreamId::new("heated")].clone();
        stream.id = output.into();
        flow.insert_stream(stream).unwrap();
        let mut heater = build_heater_node(id, id, input, output);
        heater.parameters.outlet_temperature_k = Some(300.0);
        heater.parameters.outlet_pressure_pa = Some(650000.0);
        flow.insert_unit(heater).unwrap();
        o.blocks.push(MixedBlock {
            id: id.into(),
            members: [id.into()].into(),
            method: MixedBlockMethod::Local,
        });
    }
    let e = execution::solve(
        &crate::SolverServices {
            thermo: &binary_hydrocarbon_provider(),
            flash_solver: &PlaceholderTpFlashSolver,
        },
        &flow,
        &o,
        &mut |_| ControlFlow::Continue(()),
        &mut |u, streams| {
            let inner = rf_unitops::catalog::instantiate_builtin(u, streams)?;
            Ok(if u.kind == "heater" {
                Box::new(Faulty {
                    inner,
                    wrong_identity: false,
                    delta: 1e-10,
                }) as Box<dyn UnitOperation>
            } else {
                inner
            })
        },
    )
    .unwrap_err();
    assert_eq!(e.kind, MixedFailureKind::GlobalVerification);
    assert_eq!(e.completed_blocks.len(), 6);
    assert_eq!(e.costs.local_unit_calls, 9);
    assert_eq!(e.progress.stage, MixedStage::GlobalVerification);
}
