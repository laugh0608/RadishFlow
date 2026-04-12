use std::time::SystemTime;

use crate::{StudioGuiNativeTimerSchedule, StudioGuiPlatformTimerRequest};

pub type StudioGuiPlatformNativeTimerId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiPlatformTimerBinding {
    pub schedule: StudioGuiNativeTimerSchedule,
    pub native_timer_id: StudioGuiPlatformNativeTimerId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerCommand {
    Arm {
        schedule: StudioGuiNativeTimerSchedule,
    },
    Rearm {
        previous: Option<StudioGuiPlatformTimerBinding>,
        schedule: StudioGuiNativeTimerSchedule,
    },
    Clear {
        previous: Option<StudioGuiPlatformTimerBinding>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerStartAckStatus {
    Applied,
    MissingPendingSchedule,
    StalePendingSchedule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiPlatformTimerStartAckResult {
    pub schedule: StudioGuiNativeTimerSchedule,
    pub native_timer_id: StudioGuiPlatformNativeTimerId,
    pub status: StudioGuiPlatformTimerStartAckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerStartFailureStatus {
    Applied,
    MissingPendingSchedule,
    StalePendingSchedule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiPlatformTimerStartFailureResult {
    pub schedule: StudioGuiNativeTimerSchedule,
    pub status: StudioGuiPlatformTimerStartFailureStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerCallbackResolution {
    Dispatch {
        schedule: StudioGuiNativeTimerSchedule,
    },
    IgnoredUnknownNativeTimer {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
    IgnoredStaleNativeTimer {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StudioGuiPlatformTimerDriverState {
    current_binding: Option<StudioGuiPlatformTimerBinding>,
    pending_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformTimerDriverState {
    pub fn current_binding(&self) -> Option<&StudioGuiPlatformTimerBinding> {
        self.current_binding.as_ref()
    }

    pub fn pending_schedule(&self) -> Option<&StudioGuiNativeTimerSchedule> {
        self.pending_schedule.as_ref()
    }

    pub fn current_due_at(&self) -> Option<SystemTime> {
        self.current_binding
            .as_ref()
            .map(|binding| binding.schedule.slot.timer.due_at)
    }

    pub fn apply_request(
        &mut self,
        request: Option<&StudioGuiPlatformTimerRequest>,
    ) -> Option<StudioGuiPlatformTimerCommand> {
        match request {
            Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => {
                self.current_binding = None;
                self.pending_schedule = Some(schedule.clone());
                Some(StudioGuiPlatformTimerCommand::Arm {
                    schedule: schedule.clone(),
                })
            }
            Some(StudioGuiPlatformTimerRequest::Rearm { previous, schedule }) => {
                let previous_binding = self.current_binding.take().filter(|binding| {
                    binding.schedule.window_id == previous.window_id
                        && binding.schedule.handle_id == previous.handle_id
                });
                self.pending_schedule = Some(schedule.clone());
                Some(StudioGuiPlatformTimerCommand::Rearm {
                    previous: previous_binding,
                    schedule: schedule.clone(),
                })
            }
            Some(StudioGuiPlatformTimerRequest::Clear { previous }) => {
                let previous_binding = self.current_binding.take().filter(|binding| {
                    binding.schedule.window_id == previous.window_id
                        && binding.schedule.handle_id == previous.handle_id
                });
                self.pending_schedule = None;
                Some(StudioGuiPlatformTimerCommand::Clear {
                    previous: previous_binding,
                })
            }
            None => None,
        }
    }

    pub fn acknowledge_timer_started(
        &mut self,
        schedule: &StudioGuiNativeTimerSchedule,
        native_timer_id: StudioGuiPlatformNativeTimerId,
    ) -> StudioGuiPlatformTimerStartAckResult {
        let status = match self.pending_schedule.as_ref() {
            Some(pending) if pending == schedule => {
                self.current_binding = Some(StudioGuiPlatformTimerBinding {
                    schedule: schedule.clone(),
                    native_timer_id,
                });
                self.pending_schedule = None;
                StudioGuiPlatformTimerStartAckStatus::Applied
            }
            Some(_) => StudioGuiPlatformTimerStartAckStatus::StalePendingSchedule,
            None => StudioGuiPlatformTimerStartAckStatus::MissingPendingSchedule,
        };

        StudioGuiPlatformTimerStartAckResult {
            schedule: schedule.clone(),
            native_timer_id,
            status,
        }
    }

    pub fn acknowledge_timer_start_failed(
        &mut self,
        schedule: &StudioGuiNativeTimerSchedule,
    ) -> StudioGuiPlatformTimerStartFailureResult {
        let status = match self.pending_schedule.as_ref() {
            Some(pending) if pending == schedule => {
                self.pending_schedule = None;
                StudioGuiPlatformTimerStartFailureStatus::Applied
            }
            Some(_) => StudioGuiPlatformTimerStartFailureStatus::StalePendingSchedule,
            None => StudioGuiPlatformTimerStartFailureStatus::MissingPendingSchedule,
        };

        StudioGuiPlatformTimerStartFailureResult {
            schedule: schedule.clone(),
            status,
        }
    }

    pub fn callback_schedule(
        &self,
        native_timer_id: StudioGuiPlatformNativeTimerId,
    ) -> Option<&StudioGuiNativeTimerSchedule> {
        self.current_binding.as_ref().and_then(|binding| {
            (binding.native_timer_id == native_timer_id).then_some(&binding.schedule)
        })
    }

    pub fn resolve_callback(
        &self,
        native_timer_id: StudioGuiPlatformNativeTimerId,
    ) -> StudioGuiPlatformTimerCallbackResolution {
        match self.current_binding.as_ref() {
            Some(binding) if binding.native_timer_id == native_timer_id => {
                StudioGuiPlatformTimerCallbackResolution::Dispatch {
                    schedule: binding.schedule.clone(),
                }
            }
            Some(_) => StudioGuiPlatformTimerCallbackResolution::IgnoredStaleNativeTimer {
                native_timer_id,
            },
            None if self.pending_schedule.is_some() => {
                StudioGuiPlatformTimerCallbackResolution::IgnoredStaleNativeTimer {
                    native_timer_id,
                }
            }
            None => StudioGuiPlatformTimerCallbackResolution::IgnoredUnknownNativeTimer {
                native_timer_id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use crate::{
        StudioGuiNativeTimerSchedule, StudioGuiPlatformNativeTimerId,
        StudioGuiPlatformTimerBinding, StudioGuiPlatformTimerCallbackResolution,
        StudioGuiPlatformTimerCommand, StudioGuiPlatformTimerDriverState,
        StudioGuiPlatformTimerRequest, StudioRuntimeHostEffectId, StudioRuntimeTimerHandleSlot,
        StudioWindowHostId,
    };

    fn schedule(
        window_id: Option<StudioWindowHostId>,
        handle_id: u64,
        effect_id: StudioRuntimeHostEffectId,
        due_seconds: u64,
    ) -> StudioGuiNativeTimerSchedule {
        StudioGuiNativeTimerSchedule {
            window_id,
            handle_id,
            slot: StudioRuntimeTimerHandleSlot {
                effect_id,
                timer: crate::EntitlementSessionTimerArm {
                    event: crate::EntitlementSessionLifecycleEvent::TimerElapsed,
                    due_at: UNIX_EPOCH + Duration::from_secs(due_seconds),
                    delay: Duration::from_secs(due_seconds),
                    reason: crate::EntitlementSessionTimerReason::ScheduledCheck,
                },
            },
        }
    }

    fn acknowledge(
        state: &mut StudioGuiPlatformTimerDriverState,
        schedule: &StudioGuiNativeTimerSchedule,
        native_timer_id: StudioGuiPlatformNativeTimerId,
    ) {
        let ack = state.acknowledge_timer_started(schedule, native_timer_id);
        assert_eq!(
            ack.status,
            crate::StudioGuiPlatformTimerStartAckStatus::Applied
        );
    }

    #[test]
    fn platform_timer_driver_arms_and_maps_native_timer_callback() {
        let mut state = StudioGuiPlatformTimerDriverState::default();
        let schedule = schedule(Some(7), 41, 1001, 60);

        let command = state.apply_request(Some(&StudioGuiPlatformTimerRequest::Arm {
            schedule: schedule.clone(),
        }));

        assert_eq!(
            command,
            Some(StudioGuiPlatformTimerCommand::Arm {
                schedule: schedule.clone(),
            })
        );
        assert_eq!(state.pending_schedule(), Some(&schedule));
        assert_eq!(state.current_binding(), None);

        acknowledge(&mut state, &schedule, 9001);

        assert_eq!(
            state.current_binding(),
            Some(&StudioGuiPlatformTimerBinding {
                schedule: schedule.clone(),
                native_timer_id: 9001,
            })
        );
        assert_eq!(state.callback_schedule(9001), Some(&schedule));
        assert_eq!(state.current_due_at(), Some(schedule.slot.timer.due_at));
    }

    #[test]
    fn platform_timer_driver_rearms_using_previous_native_timer_binding() {
        let mut state = StudioGuiPlatformTimerDriverState::default();
        let first = schedule(Some(7), 41, 1001, 60);
        let second = schedule(Some(7), 42, 1002, 90);
        acknowledge_after_arm(&mut state, &first, 9001);

        let command = state.apply_request(Some(&StudioGuiPlatformTimerRequest::Rearm {
            previous: first.clone(),
            schedule: second.clone(),
        }));

        assert_eq!(
            command,
            Some(StudioGuiPlatformTimerCommand::Rearm {
                previous: Some(StudioGuiPlatformTimerBinding {
                    schedule: first.clone(),
                    native_timer_id: 9001,
                }),
                schedule: second.clone(),
            })
        );
        assert_eq!(state.callback_schedule(9001), None);
        assert_eq!(state.pending_schedule(), Some(&second));

        acknowledge(&mut state, &second, 9002);
        assert_eq!(state.callback_schedule(9002), Some(&second));
    }

    #[test]
    fn platform_timer_driver_clears_current_native_timer_binding() {
        let mut state = StudioGuiPlatformTimerDriverState::default();
        let schedule = schedule(Some(7), 41, 1001, 60);
        acknowledge_after_arm(&mut state, &schedule, 9001);

        let command = state.apply_request(Some(&StudioGuiPlatformTimerRequest::Clear {
            previous: schedule.clone(),
        }));

        assert_eq!(
            command,
            Some(StudioGuiPlatformTimerCommand::Clear {
                previous: Some(StudioGuiPlatformTimerBinding {
                    schedule: schedule.clone(),
                    native_timer_id: 9001,
                }),
            })
        );
        assert_eq!(state.current_binding(), None);
        assert_eq!(state.callback_schedule(9001), None);
    }

    #[test]
    fn platform_timer_driver_rejects_stale_start_ack() {
        let mut state = StudioGuiPlatformTimerDriverState::default();
        let pending = schedule(Some(7), 42, 1002, 90);
        let stale = schedule(Some(7), 41, 1001, 60);
        let _ = state.apply_request(Some(&StudioGuiPlatformTimerRequest::Arm {
            schedule: pending.clone(),
        }));

        let ack = state.acknowledge_timer_started(&stale, 9001);

        assert_eq!(
            ack.status,
            crate::StudioGuiPlatformTimerStartAckStatus::StalePendingSchedule
        );
        assert_eq!(state.pending_schedule(), Some(&pending));
        assert_eq!(state.current_binding(), None);
    }

    #[test]
    fn platform_timer_driver_clears_pending_schedule_after_start_failure() {
        let mut state = StudioGuiPlatformTimerDriverState::default();
        let pending = schedule(Some(7), 42, 1002, 90);
        let _ = state.apply_request(Some(&StudioGuiPlatformTimerRequest::Arm {
            schedule: pending.clone(),
        }));

        let failure = state.acknowledge_timer_start_failed(&pending);

        assert_eq!(
            failure.status,
            crate::StudioGuiPlatformTimerStartFailureStatus::Applied
        );
        assert_eq!(state.pending_schedule(), None);
        assert_eq!(state.current_binding(), None);
    }

    #[test]
    fn platform_timer_driver_rejects_stale_start_failure_ack() {
        let mut state = StudioGuiPlatformTimerDriverState::default();
        let pending = schedule(Some(7), 42, 1002, 90);
        let stale = schedule(Some(7), 41, 1001, 60);
        let _ = state.apply_request(Some(&StudioGuiPlatformTimerRequest::Arm {
            schedule: pending.clone(),
        }));

        let failure = state.acknowledge_timer_start_failed(&stale);

        assert_eq!(
            failure.status,
            crate::StudioGuiPlatformTimerStartFailureStatus::StalePendingSchedule
        );
        assert_eq!(state.pending_schedule(), Some(&pending));
        assert_eq!(state.current_binding(), None);
    }

    #[test]
    fn platform_timer_driver_resolves_current_native_timer_callback() {
        let mut state = StudioGuiPlatformTimerDriverState::default();
        let schedule = schedule(Some(7), 41, 1001, 60);
        acknowledge_after_arm(&mut state, &schedule, 9001);

        let resolution = state.resolve_callback(9001);

        assert_eq!(
            resolution,
            StudioGuiPlatformTimerCallbackResolution::Dispatch {
                schedule: schedule.clone(),
            }
        );
    }

    #[test]
    fn platform_timer_driver_classifies_unknown_native_timer_callback_without_binding() {
        let state = StudioGuiPlatformTimerDriverState::default();

        let resolution = state.resolve_callback(9001);

        assert_eq!(
            resolution,
            StudioGuiPlatformTimerCallbackResolution::IgnoredUnknownNativeTimer {
                native_timer_id: 9001,
            }
        );
    }

    #[test]
    fn platform_timer_driver_classifies_stale_native_timer_callback_when_rearm_is_pending() {
        let mut state = StudioGuiPlatformTimerDriverState::default();
        let previous = schedule(Some(7), 41, 1001, 60);
        let next = schedule(Some(7), 42, 1002, 90);
        acknowledge_after_arm(&mut state, &previous, 9001);
        let _ = state.apply_request(Some(&StudioGuiPlatformTimerRequest::Rearm {
            previous: previous.clone(),
            schedule: next,
        }));

        let resolution = state.resolve_callback(9001);

        assert_eq!(
            resolution,
            StudioGuiPlatformTimerCallbackResolution::IgnoredStaleNativeTimer {
                native_timer_id: 9001,
            }
        );
    }

    fn acknowledge_after_arm(
        state: &mut StudioGuiPlatformTimerDriverState,
        schedule: &StudioGuiNativeTimerSchedule,
        native_timer_id: StudioGuiPlatformNativeTimerId,
    ) {
        let _ = state.apply_request(Some(&StudioGuiPlatformTimerRequest::Arm {
            schedule: schedule.clone(),
        }));
        acknowledge(state, schedule, native_timer_id);
    }
}
