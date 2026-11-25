use std::ops::ControlFlow;
use std::sync::{Condvar, Mutex};

use crate::outlet_temperature::{
    TemperatureCondition, TemperatureObservation, TemperatureReading, UnavailableTemperature,
};
use rf_types::UnitId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitRunPhase {
    Running,
    PauseRequested,
    Paused {
        step: usize,
        total: usize,
        unit: UnitId,
    },
    Stepping,
    Cancelling,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitRunControl {
    /// The last released step. A unit entry can be released only once per run.
    pub last_released: usize,
    pub phase: UnitRunPhase,
    pub observation: Option<TemperatureObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitRunCommand {
    Pause { last_released: usize },
    Step { step: usize },
    Continue { step: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitControlReply {
    Accepted,
    AlreadyRequested,
    Stale,
    NotPaused,
    Cancelling,
    Finished,
    UnknownRun,
    Unsupported,
}

pub(super) struct UnitControlGate {
    state: Mutex<UnitRunControl>,
    wake: Condvar,
}

impl UnitControlGate {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(UnitRunControl {
                last_released: 0,
                observation: None,
                phase: UnitRunPhase::Running,
            }),
            wake: Condvar::new(),
        }
    }
    pub fn snapshot(&self) -> UnitRunControl {
        self.state.lock().expect("unit control poisoned").clone()
    }
    pub fn arm(&self, condition: TemperatureCondition) {
        let mut state = self.state.lock().expect("unit control poisoned");
        assert!(state.observation.is_none(), "run condition already frozen");
        state.observation = Some(TemperatureObservation::new(condition));
    }

    /// Returns true when the solver must stop at its next cancellation checkpoint.
    pub fn observe(&self, event: &rf_solver::SequentialExecutionEvent<'_>) -> bool {
        let mut state = self.state.lock().expect("unit control poisoned");
        if state.phase == UnitRunPhase::Cancelling {
            return true;
        }
        let Some(watch) = &mut state.observation else {
            return false;
        };
        if watch.reading != TemperatureReading::Pending {
            return false;
        }
        watch.observe(event);
        match watch.reading {
            TemperatureReading::Measured { step, actual }
                if step < watch.total && actual > watch.condition.threshold.kelvin() =>
            {
                if state.phase == UnitRunPhase::Running {
                    state.phase = UnitRunPhase::PauseRequested;
                }
                false
            }
            TemperatureReading::Unavailable { .. } => {
                state.phase = UnitRunPhase::Cancelling;
                true
            }
            _ => false,
        }
    }

    pub fn completing(&self) -> ControlFlow<()> {
        let mut state = self.state.lock().expect("unit control poisoned");
        if state.phase == UnitRunPhase::Cancelling {
            return ControlFlow::Break(());
        }
        if let Some(watch) = &mut state.observation
            && watch.reading == TemperatureReading::Pending
        {
            watch.reading = TemperatureReading::Unavailable {
                step: None,
                reason: UnavailableTemperature::TargetNotReached,
            };
            state.phase = UnitRunPhase::Cancelling;
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    }
    pub fn request(&self, command: UnitRunCommand) -> UnitControlReply {
        use UnitControlReply as Reply;
        let mut state = self.state.lock().expect("unit control poisoned");
        match state.phase {
            UnitRunPhase::Finished => return Reply::Finished,
            UnitRunPhase::Cancelling => return Reply::Cancelling,
            _ => {}
        }
        match command {
            UnitRunCommand::Pause { last_released } => {
                if last_released != state.last_released {
                    return Reply::Stale;
                }
                if state.phase != UnitRunPhase::Running {
                    return Reply::AlreadyRequested;
                }
                state.phase = UnitRunPhase::PauseRequested;
            }
            UnitRunCommand::Step { step } | UnitRunCommand::Continue { step } => {
                if step <= state.last_released {
                    return Reply::Stale;
                }
                let UnitRunPhase::Paused { step: stopped, .. } = state.phase else {
                    return Reply::NotPaused;
                };
                if step != stopped {
                    return Reply::Stale;
                }
                state.last_released = step;
                state.phase = match command {
                    UnitRunCommand::Step { .. } => UnitRunPhase::Stepping,
                    _ => UnitRunPhase::Running,
                };
                self.wake.notify_one();
            }
        }
        Reply::Accepted
    }
    pub fn before_unit(&self, step: usize, total: usize, unit: &UnitId) -> ControlFlow<()> {
        let mut state = self.state.lock().expect("unit control poisoned");
        if matches!(
            state.phase,
            UnitRunPhase::PauseRequested | UnitRunPhase::Stepping
        ) {
            state.phase = UnitRunPhase::Paused {
                step,
                total,
                unit: unit.clone(),
            };
            state = self
                .wake
                .wait_while(state, |s| matches!(s.phase, UnitRunPhase::Paused { .. }))
                .expect("unit control poisoned");
        }
        if matches!(
            state.phase,
            UnitRunPhase::Cancelling | UnitRunPhase::Finished
        ) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
    pub fn cancel(&self) {
        let mut state = self.state.lock().expect("unit control poisoned");
        if state.phase != UnitRunPhase::Finished {
            state.phase = UnitRunPhase::Cancelling;
        }
        self.wake.notify_one();
    }
    pub fn finish(&self) {
        self.state.lock().expect("unit control poisoned").phase = UnitRunPhase::Finished;
        self.wake.notify_one();
    }
}
