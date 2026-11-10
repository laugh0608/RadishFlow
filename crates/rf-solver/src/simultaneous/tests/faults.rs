use super::super::execution::Execution;
use super::*;
use rf_unitops::{
    BuiltinUnitKind, UnitOperation, UnitOperationInputs, UnitOperationOutputs,
    UnitOperationServices,
};
use std::cell::Cell;

fn with_execution<T>(flow: &Flowsheet, f: impl FnOnce(&mut Execution<'_, '_>) -> T) -> T {
    let plan = assemble(flow, &options()).unwrap();
    with_plan(&plan, f)
}
fn with_plan<T>(plan: &ModularPlan, f: impl FnOnce(&mut Execution<'_, '_>) -> T) -> T {
    let thermo = binary_hydrocarbon_provider();
    let mut control = Uninterrupted;
    let mut run = Execution {
        plan,
        services: UnitOperationServices {
            thermo: Some(&thermo),
            flash_solver: Some(&PlaceholderTpFlashSolver),
        },
        operations: BTreeMap::new(),
        control: &mut control,
        streams: BTreeMap::new(),
        blocks: Vec::new(),
        active_units: Vec::new(),
        active_coupling: None,
        costs: ModularCosts::default(),
        progress: ModularProgress {
            stage: ModularStage::Initializing,
            block: None,
            unit: None,
            column: None,
        },
        global_scales: BTreeMap::new(),
    };
    run.initialize().unwrap();
    f(&mut run)
}

#[derive(Clone, Copy)]
enum Fault {
    Unbalanced,
    Nonfinite,
    Identity,
    AlwaysFail,
}
struct FaultyUnit {
    inner: Box<dyn UnitOperation>,
    fault: Fault,
    calls: Cell<usize>,
}
impl UnitOperation for FaultyUnit {
    fn kind(&self) -> BuiltinUnitKind {
        self.inner.kind()
    }
    fn run(
        &self,
        s: &UnitOperationServices<'_>,
        i: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        self.calls.set(self.calls.get() + 1);
        if matches!(self.fault, Fault::AlwaysFail) {
            return Err(failure("injected_local", "late local failure"));
        }
        let mut result = self.inner.run(s, i)?;
        if self.calls.get() == 2 {
            let mut output = result.stream("outlet_a").expect("splitter").clone();
            match self.fault {
                Fault::Unbalanced => output.total_molar_flow_mol_s *= 1.1,
                Fault::Nonfinite => output.total_molar_flow_mol_s = f64::NAN,
                Fault::Identity => output.id = "foreign".into(),
                Fault::AlwaysFail => unreachable!(),
            }
            result.insert_material_stream("outlet_a", output);
        }
        Ok(result)
    }
}
#[test]
fn final_evaluation_faults_cannot_publish_candidate_or_continue_downstream() {
    for (fault, kind) in [
        (Fault::Unbalanced, ModularFailureKind::Balance),
        (Fault::Nonfinite, ModularFailureKind::Boundary),
        (Fault::Identity, ModularFailureKind::LocalEvaluation),
    ] {
        with_execution(&case(0.6, 0.25), |run| {
            let id = UnitId::new("sa");
            let inner = run.operations.remove(&id).unwrap();
            run.operations.insert(
                id.clone(),
                Box::new(FaultyUnit {
                    inner,
                    fault,
                    calls: Cell::new(0),
                }),
            );
            let error = run.execute().unwrap_err();
            assert_eq!(error.kind, kind);
            assert_eq!(run.blocks.len(), 2);
            assert_eq!(run.costs.verification_evaluations, 1);
            assert_eq!(run.costs.calls_by_unit[&id], 2);
            assert!(run.active_coupling.as_ref().unwrap().linear.is_some());
        });
    }
}
#[test]
fn a_wrong_declared_response_is_caught_by_real_module_verification() {
    let flow = case(0.6, 0.25);
    let mut plan = assemble(&flow, &options()).unwrap();
    let mut fake = flow.units[&UnitId::new("sa")].clone();
    fake.parameters.outlet_a_split_fraction = Some(0.5);
    let bad = AffineComponentResponse::for_unit(&fake, 300.0, 650_000.0, &plan.components).unwrap();
    let block = plan
        .blocks
        .iter_mut()
        .find(|b| !b.couplings.is_empty())
        .unwrap();
    *block
        .responses
        .iter_mut()
        .find(|r| r.unit_id() == &fake.id)
        .unwrap() = bad;
    with_plan(&plan, |run| {
        let e = run.execute().unwrap_err();
        assert_eq!(e.kind, ModularFailureKind::Verification);
        assert_eq!(run.blocks.len(), 2);
        assert!(
            run.active_coupling
                .as_ref()
                .unwrap()
                .connections
                .iter()
                .any(|c| c.residual_mol_s.abs() > 1e-10)
        );
    });
}
#[test]
fn all_declared_safe_points_cancel_with_no_success_and_can_be_run_again() {
    let flow = case(0.6, 0.25);
    for target in [
        ModularStage::AssemblyStarted,
        ModularStage::AssemblyCompleted,
        ModularStage::Initializing,
        ModularStage::BlockStarted,
        ModularStage::Anchor,
        ModularStage::UnitStarted,
        ModularStage::UnitCompleted,
        ModularStage::Response,
        ModularStage::Factorization,
        ModularStage::ConditionSolve,
        ModularStage::LinearVerification,
        ModularStage::Verification,
        ModularStage::BlockCompleted,
        ModularStage::Completing,
    ] {
        let mut observed = false;
        struct Stop<'a> {
            target: ModularStage,
            observed: &'a mut bool,
        }
        impl ModularControl for Stop<'_> {
            fn checkpoint(&mut self, p: ModularProgress) -> ControlFlow<()> {
                if p.stage == self.target {
                    *self.observed = true;
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            }
        }
        let mut stop = Stop {
            target,
            observed: &mut observed,
        };
        let error = SimultaneousModularSolver
            .solve_controlled(
                &crate::SolverServices {
                    thermo: &binary_hydrocarbon_provider(),
                    flash_solver: &PlaceholderTpFlashSolver,
                },
                &flow,
                &options(),
                &mut stop,
            )
            .unwrap_err();
        assert!(observed);
        assert_eq!(error.kind, ModularFailureKind::Cancelled);
        assert_eq!(error.progress.stage, target);
        if target == ModularStage::Response {
            assert_eq!(error.costs.response_evaluations, 0);
        }
    }
    assert!(solve(&flow).is_ok());
}

#[test]
fn cancellation_inside_linear_columns_retains_only_attempted_work() {
    struct StopColumn(ModularStage);
    impl ModularControl for StopColumn {
        fn checkpoint(&mut self, p: ModularProgress) -> ControlFlow<()> {
            if p.stage == self.0 && p.column == Some(3) {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }
    }
    for stage in [
        ModularStage::Response,
        ModularStage::Factorization,
        ModularStage::ConditionSolve,
    ] {
        let error = SimultaneousModularSolver
            .solve_controlled(
                &crate::SolverServices {
                    thermo: &binary_hydrocarbon_provider(),
                    flash_solver: &PlaceholderTpFlashSolver,
                },
                &case(0.6, 0.25),
                &options(),
                &mut StopColumn(stage),
            )
            .unwrap_err();
        assert_eq!(error.kind, ModularFailureKind::Cancelled);
        assert_eq!(error.progress.column, Some(3));
        assert_eq!(error.costs.local_unit_calls, 6);
        assert_eq!(error.costs.verification_evaluations, 0);
        if stage == ModularStage::ConditionSolve {
            assert_eq!(error.costs.triangular_solves, 3);
            let evidence = error.active_coupling.unwrap().linear.unwrap();
            assert_eq!(evidence.rcond_indicator, None);
        }
    }
}
#[test]
fn repeated_nonzero_assumptions_use_one_frozen_vector_and_remain_affine() {
    let flow = case(0.6, 0.25);
    let frozen = flow.clone();
    with_execution(&flow, |run| {
        // Evaluate the two independent feeds, then probe the four-module block directly.
        let scales = run.global_scales.clone();
        for id in ["fa", "fb"] {
            run.evaluate_unit(&UnitId::new(id), &BTreeMap::new(), &scales)
                .unwrap();
        }
        let block = run.plan.blocks.last().unwrap().clone();
        let n = block.couplings.len();
        let rules = block
            .couplings
            .iter()
            .map(|c| {
                let mixer = if flow.units[&c.sink.unit_id].kind == "mixer" {
                    &c.sink.unit_id
                } else {
                    &c.source.unit_id
                };
                (c.stream.clone(), block.fresh_by_mixer[mixer].clone())
            })
            .collect();
        let (r0, _) = run
            .evaluate_couplings(&block, &vec![0.0; n], &rules, &scales)
            .unwrap();
        let unit_scales = vec![
            CouplingScale {
                variable_mol_s: 1.0,
                residual_mol_s: 1.0
            };
            n
        ];
        let j = run.jacobian(&block, &unit_scales).unwrap();
        for values in [
            (0..n).map(|i| i as f64 + 1.0).collect::<Vec<_>>(),
            vec![0.25; n],
        ] {
            let (actual, units) = run
                .evaluate_couplings(&block, &values, &rules, &scales)
                .unwrap();
            let (repeat, repeated_units) = run
                .evaluate_couplings(&block, &values, &rules, &scales)
                .unwrap();
            assert_eq!(actual, repeat);
            assert_eq!(units, repeated_units);
            let mut reverse = block.clone();
            reverse.members.reverse();
            let (reordered, _) = run
                .evaluate_couplings(&reverse, &values, &rules, &scales)
                .unwrap();
            assert_eq!(actual, reordered);
            for i in 0..n {
                let expected = r0[i] + j[i].iter().zip(&values).map(|(a, x)| a * x).sum::<f64>();
                close(actual[i], expected, 1e-12);
            }
        }
        for invalid in [
            vec![-1.0; n],
            vec![f64::NAN; n],
            vec![f64::INFINITY; n],
            vec![f64::MAX; n],
        ] {
            assert!(
                run.evaluate_couplings(&block, &invalid, &rules, &scales)
                    .is_err()
            );
        }
    });
    assert_eq!(flow, frozen);
}

#[test]
fn downstream_failure_preserves_completed_block_only_as_diagnostic() {
    let mut flow = case(0.6, 0.25);
    flow.insert_stream(stream("heated", 90.0, 0.2, 0.8))
        .unwrap();
    let mut h = rf_unitops::build_heater_node("h", "H", "pa", "heated");
    h.parameters.outlet_temperature_k = Some(300.0);
    h.parameters.outlet_pressure_pa = Some(650_000.0);
    flow.insert_unit(h).unwrap();
    with_execution(&flow, |run| {
        let id = UnitId::new("h");
        let inner = run.operations.remove(&id).unwrap();
        run.operations.insert(
            id.clone(),
            Box::new(FaultyUnit {
                inner,
                fault: Fault::AlwaysFail,
                calls: Cell::new(0),
            }),
        );
        let e = run.execute().unwrap_err();
        assert_eq!(e.kind, ModularFailureKind::LocalEvaluation);
        assert_eq!(run.blocks.len(), 3);
        assert!(run.blocks.last().unwrap().coupling.is_some());
        assert_eq!(run.costs.local_unit_calls, 11);
        assert_eq!(e.error.context().related_unit_ids(), &[id]);
    });
}

struct SwitchThermo {
    inner: rf_thermo::PlaceholderThermoProvider,
    fail: Cell<bool>,
    calls: Cell<usize>,
}
impl SwitchThermo {
    fn observe(&self) -> RfResult<()> {
        self.calls.set(self.calls.get() + 1);
        if self.fail.get() {
            Err(failure("injected_thermo", "observed thermo failure"))
        } else {
            Ok(())
        }
    }
}
impl rf_thermo::ThermoProvider for SwitchThermo {
    fn system(&self) -> &rf_thermo::ThermoSystem {
        self.inner.system()
    }
    fn estimate_k_values(&self, s: &rf_thermo::ThermoState) -> RfResult<Vec<f64>> {
        self.observe()?;
        self.inner.estimate_k_values(s)
    }
    fn estimate_bubble_dew_pressures(
        &self,
        s: &rf_thermo::BubbleDewPressureInput,
    ) -> RfResult<rf_thermo::BubbleDewPressures> {
        self.observe()?;
        self.inner.estimate_bubble_dew_pressures(s)
    }
    fn estimate_bubble_dew_temperatures(
        &self,
        s: &rf_thermo::BubbleDewTemperatureInput,
    ) -> RfResult<rf_thermo::BubbleDewTemperatures> {
        self.observe()?;
        self.inner.estimate_bubble_dew_temperatures(s)
    }
    fn phase_molar_enthalpy(&self, s: &rf_thermo::PhaseThermoState) -> RfResult<f64> {
        self.observe()?;
        self.inner.phase_molar_enthalpy(s)
    }
}
#[test]
fn thermo_failures_count_actual_attempts_during_preprocessing_and_verification() {
    for late in [false, true] {
        let thermo = SwitchThermo {
            inner: binary_hydrocarbon_provider(),
            fail: Cell::new(!late),
            calls: Cell::new(0),
        };
        struct Switch<'a>(&'a Cell<bool>);
        impl ModularControl for Switch<'_> {
            fn checkpoint(&mut self, p: ModularProgress) -> ControlFlow<()> {
                if p.stage == ModularStage::Verification {
                    self.0.set(true);
                }
                ControlFlow::Continue(())
            }
        }
        let mut control = Switch(&thermo.fail);
        let error = SimultaneousModularSolver
            .solve_controlled(
                &crate::SolverServices {
                    thermo: &thermo,
                    flash_solver: &PlaceholderTpFlashSolver,
                },
                &case(0.6, 0.25),
                &options(),
                &mut control,
            )
            .unwrap_err();
        assert_eq!(error.kind, ModularFailureKind::LocalEvaluation);
        assert_eq!(error.costs.thermo_calls, thermo.calls.get());
        if late {
            assert_eq!(error.costs.verification_evaluations, 1);
            assert_eq!(error.costs.local_unit_calls, 7);
            assert_eq!(error.completed_blocks.len(), 2);
        } else {
            assert_eq!(error.costs.local_unit_calls, 1);
            assert_eq!(error.costs.thermo_calls, 1);
        }
    }
}
