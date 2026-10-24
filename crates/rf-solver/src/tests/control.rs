use super::*;
use crate::{ControlledSolveOutcome, SolveProgress, SolveStage, SolverControl};
use std::{cell::Cell, ops::ControlFlow};

#[derive(Default)]
struct RecordingControl {
    events: Vec<SolveProgress>,
    stop_at: Option<usize>,
}
impl SolverControl for RecordingControl {
    fn checkpoint(&mut self, progress: SolveProgress) -> ControlFlow<()> {
        self.events.push(progress);
        if self.stop_at == Some(self.events.len() - 1) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

#[test]
fn controlled_and_legacy_solve_have_identical_numeric_outputs_and_diagnostics() {
    let thermo = build_provider();
    let flash = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &thermo,
        flash_solver: &flash,
    };
    for flowsheet in [
        build_demo_flowsheet(),
        build_feed_mixer_heater_flash_flowsheet(),
        build_feed_valve_flash_flowsheet(),
        build_feed_cooler_flash_flowsheet(),
    ] {
        let expected = SequentialModularSolver
            .solve(&services, &flowsheet)
            .unwrap();
        let mut control = RecordingControl::default();
        assert_eq!(
            SequentialModularSolver
                .solve_controlled(&services, &flowsheet, &mut control)
                .unwrap(),
            ControlledSolveOutcome::Completed(expected.clone())
        );
        assert_eq!(control.events.first().unwrap().total_units, None);
        assert_eq!(
            control.events.last().unwrap().completed_units,
            expected.steps.len()
        );
        assert_eq!(control.events.last().unwrap().stage, SolveStage::Completing);
        let units = control
            .events
            .iter()
            .filter(|e| e.stage == SolveStage::UnitCompleted)
            .map(|e| e.current_unit.clone().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            units,
            expected
                .steps
                .iter()
                .map(|s| s.unit_id.clone())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn cancellation_at_every_declared_boundary_never_returns_a_partial_success() {
    let thermo = build_provider();
    let flash = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &thermo,
        flash_solver: &flash,
    };
    let flowsheet = build_feed_mixer_heater_flash_flowsheet();
    let mut reference = RecordingControl::default();
    SequentialModularSolver
        .solve_controlled(&services, &flowsheet, &mut reference)
        .unwrap();
    for stop_at in 0..reference.events.len() {
        let mut control = RecordingControl {
            stop_at: Some(stop_at),
            ..Default::default()
        };
        assert_eq!(
            SequentialModularSolver
                .solve_controlled(&services, &flowsheet, &mut control)
                .unwrap(),
            ControlledSolveOutcome::Cancelled
        );
        assert_eq!(control.events, reference.events[..=stop_at]);
    }
}

#[test]
fn graph_errors_keep_their_order_and_context_in_controlled_execution() {
    let thermo = build_provider();
    let flash = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &thermo,
        flash_solver: &flash,
    };
    let mut flow = build_feed_heater_flash_flowsheet();
    flow.units
        .get_mut(&"heater-1".into())
        .unwrap()
        .parameters
        .outlet_temperature_k = Some(-1.0);
    let expected = SequentialModularSolver.solve(&services, &flow).unwrap_err();
    let actual = SequentialModularSolver
        .solve_controlled(&services, &flow, &mut RecordingControl::default())
        .unwrap_err();
    assert_eq!(actual, expected);
}

#[test]
fn cancellation_inside_a_unit_waits_for_return_and_does_not_replace_its_error() {
    struct CancellingFlash<'a> {
        flag: &'a Cell<bool>,
        fail: bool,
    }
    impl rf_flash::TpFlashSolver for CancellingFlash<'_> {
        fn flash(
            &self,
            thermo: &dyn rf_thermo::ThermoProvider,
            input: &rf_flash::TpFlashInput,
        ) -> rf_types::RfResult<rf_flash::TpFlashResult> {
            self.flag.set(true);
            if self.fail {
                Err(RfError::invalid_input(
                    "numeric failure during cancellation",
                ))
            } else {
                PlaceholderTpFlashSolver.flash(thermo, input)
            }
        }
    }
    struct Control<'a>(&'a Cell<bool>);
    impl SolverControl for Control<'_> {
        fn checkpoint(&mut self, _: SolveProgress) -> ControlFlow<()> {
            if self.0.get() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }
    }
    for fail in [false, true] {
        let flag = Cell::new(false);
        let thermo = build_provider();
        let flash = CancellingFlash { flag: &flag, fail };
        let result = SequentialModularSolver.solve_controlled(
            &SolverServices {
                thermo: &thermo,
                flash_solver: &flash,
            },
            &build_demo_flowsheet(),
            &mut Control(&flag),
        );
        assert!(flag.get());
        if fail {
            assert!(
                result
                    .unwrap_err()
                    .message()
                    .contains("numeric failure during cancellation")
            );
        } else {
            assert_eq!(result.unwrap(), ControlledSolveOutcome::Cancelled);
        }
    }
}
