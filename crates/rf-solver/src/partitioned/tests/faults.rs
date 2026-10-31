use super::*;
use rf_thermo::{
    BubbleDewPressureInput, BubbleDewPressures, BubbleDewTemperatureInput, BubbleDewTemperatures,
    PhaseThermoState, ThermoProvider, ThermoState, ThermoSystem,
};
use rf_types::RfResult;
use std::cell::Cell;

struct ObservableThermo {
    inner: rf_thermo::PlaceholderThermoProvider,
    calls: Cell<usize>,
    fail: Cell<bool>,
}
impl ObservableThermo {
    fn new() -> Self {
        Self {
            inner: binary_hydrocarbon_provider(),
            calls: Cell::new(0),
            fail: Cell::new(false),
        }
    }
    fn observe(&self) -> RfResult<()> {
        self.calls.set(self.calls.get() + 1);
        if self.fail.get() {
            Err(RfError::thermo("injected thermo failure")
                .with_diagnostic_code("test.thermo.failure"))
        } else {
            Ok(())
        }
    }
}
impl ThermoProvider for ObservableThermo {
    fn system(&self) -> &ThermoSystem {
        self.inner.system()
    }
    fn estimate_k_values(&self, state: &ThermoState) -> RfResult<Vec<f64>> {
        self.observe()?;
        self.inner.estimate_k_values(state)
    }
    fn estimate_bubble_dew_pressures(
        &self,
        state: &BubbleDewPressureInput,
    ) -> RfResult<BubbleDewPressures> {
        self.observe()?;
        self.inner.estimate_bubble_dew_pressures(state)
    }
    fn estimate_bubble_dew_temperatures(
        &self,
        state: &BubbleDewTemperatureInput,
    ) -> RfResult<BubbleDewTemperatures> {
        self.observe()?;
        self.inner.estimate_bubble_dew_temperatures(state)
    }
    fn phase_molar_enthalpy(&self, state: &PhaseThermoState) -> RfResult<f64> {
        self.observe()?;
        self.inner.phase_molar_enthalpy(state)
    }
}
#[test]
fn counted_service_calls_are_observed_even_when_a_real_local_call_fails() {
    let (flow, options) = case(0.6);
    let thermo = ObservableThermo::new();
    let result = PartitionedSequentialSolver
        .solve(
            &SolverServices {
                thermo: &thermo,
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &options,
        )
        .unwrap();
    assert_eq!(result.costs.thermo_calls, thermo.calls.get());
    let thermo = ObservableThermo::new();
    thermo.fail.set(true);
    let failure = PartitionedSequentialSolver
        .solve(
            &SolverServices {
                thermo: &thermo,
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &options,
        )
        .unwrap_err();
    assert_eq!(failure.kind, PartitionedFailureKind::LocalEvaluation);
    assert_eq!(
        failure.error.context().diagnostic_code(),
        Some("test.thermo.failure")
    );
    assert_eq!(
        failure.error.context().related_unit_ids(),
        &[UnitId::new("fa")]
    );
    assert_eq!(failure.costs.local_unit_calls, 1);
    assert_eq!(failure.costs.thermo_calls, 1);
    assert_eq!(failure.costs.thermo_calls, thermo.calls.get());
}
#[test]
fn failure_during_independent_verification_never_runs_downstream_or_publishes_candidate() {
    let (flow, options) = case(0.6);
    let thermo = ObservableThermo::new();
    let mut control = |progress: PartitionedProgress| {
        if progress.stage == PartitionedStage::VerificationStarted {
            thermo.fail.set(true);
        }
        ControlFlow::Continue(())
    };
    let failure = PartitionedSequentialSolver
        .solve_controlled(
            &SolverServices {
                thermo: &thermo,
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &options,
            &mut control,
        )
        .unwrap_err();
    assert_eq!(failure.kind, PartitionedFailureKind::LocalEvaluation);
    assert_eq!(failure.costs.iteration_evaluations, 48);
    assert_eq!(failure.costs.verification_evaluations, 1);
    assert_eq!(failure.costs.thermo_calls, thermo.calls.get());
    assert_eq!(failure.completed_blocks.len(), 3);
    assert!(!failure.costs.calls_by_unit.contains_key(&UnitId::new("h1")));
    assert_eq!(failure.active_recycle.unwrap().verification_evaluations, 0);
}
#[test]
fn all_declared_safe_points_cancel_atomically_with_actual_costs() {
    let stages = [
        PartitionedStage::AssemblyStarted,
        PartitionedStage::AssemblyCompleted,
        PartitionedStage::Initializing,
        PartitionedStage::BlockStarted,
        PartitionedStage::UnitStarted,
        PartitionedStage::UnitCompleted,
        PartitionedStage::IterationStarted,
        PartitionedStage::IterationCompleted,
        PartitionedStage::VerificationStarted,
        PartitionedStage::VerificationCompleted,
        PartitionedStage::Completing,
    ];
    for stage in stages {
        let (flow, options) = case(0.6);
        let original = flow.clone();
        let mut control = |progress: PartitionedProgress| {
            if progress.stage == stage {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        };
        let error = PartitionedSequentialSolver
            .solve_controlled(
                &SolverServices {
                    thermo: &binary_hydrocarbon_provider(),
                    flash_solver: &PlaceholderTpFlashSolver,
                },
                &flow,
                &options,
                &mut control,
            )
            .unwrap_err();
        assert_eq!(error.kind, PartitionedFailureKind::Cancelled, "{stage:?}");
        assert_eq!(flow, original);
        assert_eq!(
            error.costs.local_unit_calls,
            error.costs.calls_by_unit.values().sum::<usize>()
        );
        if stage == PartitionedStage::UnitCompleted {
            assert_eq!(error.costs.local_unit_calls, 1);
            assert!(error.costs.thermo_calls > 0);
        }
        if stage == PartitionedStage::VerificationCompleted {
            assert_eq!(error.costs.verification_evaluations, 1);
            assert!(!error.costs.calls_by_unit.contains_key(&UnitId::new("h1")));
        }
        assert!(solve(&flow, &options).is_ok()); // No trial state or cancelled guess leaks.
    }
}
#[test]
fn downstream_failure_keeps_verified_loop_only_as_diagnostic_evidence() {
    let (flow, options) = case(0.6);
    let thermo = ObservableThermo::new();
    let mut control = |progress: PartitionedProgress| {
        if progress.stage == PartitionedStage::UnitStarted
            && progress.unit == Some(UnitId::new("h1"))
        {
            thermo.fail.set(true);
        }
        ControlFlow::Continue(())
    };
    let error = PartitionedSequentialSolver
        .solve_controlled(
            &SolverServices {
                thermo: &thermo,
                flash_solver: &PlaceholderTpFlashSolver,
            },
            &flow,
            &options,
            &mut control,
        )
        .unwrap_err();
    assert_eq!(error.kind, PartitionedFailureKind::LocalEvaluation);
    assert_eq!(error.completed_blocks.len(), 4);
    assert_eq!(error.costs.calls_by_unit[&UnitId::new("h1")], 1);
    assert_eq!(error.costs.verification_evaluations, 1);
    assert_eq!(
        error.completed_blocks[3]
            .recycle
            .as_ref()
            .unwrap()
            .verification_evaluations,
        1
    );
}

#[test]
fn flash_costs_count_nested_calls_and_failed_attempts() {
    use rf_flash::{TpFlashInput, TpFlashResult, TpFlashSolver};
    struct ObservedFlash {
        calls: Cell<usize>,
        fail: bool,
    }
    impl TpFlashSolver for ObservedFlash {
        fn flash(
            &self,
            thermo: &dyn ThermoProvider,
            input: &TpFlashInput,
        ) -> RfResult<TpFlashResult> {
            self.calls.set(self.calls.get() + 1);
            if self.fail {
                return Err(RfError::thermo("injected flash failure"));
            }
            PlaceholderTpFlashSolver.flash(thermo, input)
        }
    }
    let (flow, options) = case(0.6);
    for fail in [false, true] {
        let flash = ObservedFlash {
            calls: Cell::new(0),
            fail,
        };
        let thermo = ObservableThermo::new();
        let outcome = PartitionedSequentialSolver.solve(
            &SolverServices {
                thermo: &thermo,
                flash_solver: &flash,
            },
            &flow,
            &options,
        );
        let costs = if fail {
            let error = outcome.unwrap_err();
            assert_eq!(error.kind, PartitionedFailureKind::LocalEvaluation);
            assert_eq!(error.costs.local_unit_calls, 1);
            assert_eq!(error.costs.flash_calls, 1);
            error.costs
        } else {
            outcome.unwrap().costs
        };
        assert_eq!(costs.flash_calls, flash.calls.get());
        assert_eq!(costs.thermo_calls, thermo.calls.get());
    }
}
#[test]
fn stale_derived_feed_intensities_do_not_become_current_input_evidence() {
    use rf_model::PhaseState;
    use rf_types::PhaseLabel;
    let (mut flow, options) = case(0.6);
    let feed = flow.streams.get_mut(&StreamId::new("s-fa")).unwrap();
    let mut phase = PhaseState::new(
        PhaseLabel::Overall,
        1.0,
        feed.overall_mole_fractions.clone(),
    );
    phase.molar_enthalpy_j_per_mol = Some(f64::NAN);
    feed.phases.push(phase);
    let result = solve(&flow, &options).unwrap();
    assert!(
        result.streams[&StreamId::new("s-fa")]
            .phases
            .iter()
            .all(|p| p.molar_enthalpy_j_per_mol.is_none_or(f64::is_finite))
    );
    assert!(
        flow.streams[&StreamId::new("s-fa")].phases[0]
            .molar_enthalpy_j_per_mol
            .unwrap()
            .is_nan()
    );
}
