use std::collections::BTreeMap;

use crate::{
    StudioRuntimeHostEffectId, StudioRuntimeTimerHandleSlot, StudioWindowHostId,
    StudioWindowHostTimerDriverCommand,
};

pub type StudioWindowNativeTimerHandleId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWindowNativeTimerBinding {
    pub handle_id: StudioWindowNativeTimerHandleId,
    pub slot: StudioRuntimeTimerHandleSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioWindowTimerDriverTransition {
    ArmNativeTimer {
        window_id: StudioWindowHostId,
        previous_binding: Option<StudioWindowNativeTimerBinding>,
        slot: StudioRuntimeTimerHandleSlot,
    },
    KeepNativeTimer {
        window_id: StudioWindowHostId,
        binding: StudioWindowNativeTimerBinding,
    },
    RearmNativeTimer {
        window_id: StudioWindowHostId,
        previous_binding: Option<StudioWindowNativeTimerBinding>,
        next_slot: StudioRuntimeTimerHandleSlot,
    },
    ClearNativeTimer {
        window_id: StudioWindowHostId,
        previous_binding: Option<StudioWindowNativeTimerBinding>,
    },
    IgnoreStale {
        window_id: StudioWindowHostId,
        current_binding: Option<StudioWindowNativeTimerBinding>,
        stale_effect_id: StudioRuntimeHostEffectId,
    },
    TransferNativeTimer {
        from_window_id: StudioWindowHostId,
        to_window_id: StudioWindowHostId,
        binding: Option<StudioWindowNativeTimerBinding>,
        requested_slot: StudioRuntimeTimerHandleSlot,
    },
    ParkNativeTimer {
        from_window_id: StudioWindowHostId,
        binding: Option<StudioWindowNativeTimerBinding>,
        requested_slot: StudioRuntimeTimerHandleSlot,
    },
    RestoreParkedTimer {
        window_id: StudioWindowHostId,
        binding: Option<StudioWindowNativeTimerBinding>,
        requested_slot: StudioRuntimeTimerHandleSlot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWindowPendingTimerBinding {
    pub window_id: StudioWindowHostId,
    pub slot: StudioRuntimeTimerHandleSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioWindowTimerDriverAckStatus {
    Applied,
    NoPendingBinding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWindowTimerDriverAckResult {
    pub window_id: StudioWindowHostId,
    pub handle_id: StudioWindowNativeTimerHandleId,
    pub status: StudioWindowTimerDriverAckStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StudioWindowTimerDriverState {
    window_bindings: BTreeMap<StudioWindowHostId, StudioWindowNativeTimerBinding>,
    pending_bindings: BTreeMap<StudioWindowHostId, StudioWindowPendingTimerBinding>,
    parked_binding: Option<StudioWindowNativeTimerBinding>,
}

impl StudioWindowTimerDriverState {
    pub fn window_binding(
        &self,
        window_id: StudioWindowHostId,
    ) -> Option<&StudioWindowNativeTimerBinding> {
        self.window_bindings.get(&window_id)
    }

    pub fn parked_binding(&self) -> Option<&StudioWindowNativeTimerBinding> {
        self.parked_binding.as_ref()
    }

    pub fn pending_binding(
        &self,
        window_id: StudioWindowHostId,
    ) -> Option<&StudioWindowPendingTimerBinding> {
        self.pending_bindings.get(&window_id)
    }

    pub fn apply_command(
        &mut self,
        command: &StudioWindowHostTimerDriverCommand,
    ) -> StudioWindowTimerDriverTransition {
        match command {
            StudioWindowHostTimerDriverCommand::Arm { window_id, slot } => {
                self.pending_bindings.remove(window_id);
                let previous_binding = self.window_bindings.remove(window_id);
                self.pending_bindings.insert(
                    *window_id,
                    StudioWindowPendingTimerBinding {
                        window_id: *window_id,
                        slot: slot.clone(),
                    },
                );
                StudioWindowTimerDriverTransition::ArmNativeTimer {
                    window_id: *window_id,
                    previous_binding,
                    slot: slot.clone(),
                }
            }
            StudioWindowHostTimerDriverCommand::Rearm {
                window_id,
                next_slot,
                ..
            } => {
                self.pending_bindings.remove(window_id);
                let previous_binding = self.window_bindings.remove(window_id);
                self.pending_bindings.insert(
                    *window_id,
                    StudioWindowPendingTimerBinding {
                        window_id: *window_id,
                        slot: next_slot.clone(),
                    },
                );
                StudioWindowTimerDriverTransition::RearmNativeTimer {
                    window_id: *window_id,
                    previous_binding,
                    next_slot: next_slot.clone(),
                }
            }
            StudioWindowHostTimerDriverCommand::Keep { window_id, slot } => {
                if let Some(binding) = self.window_bindings.get_mut(window_id) {
                    binding.slot = slot.clone();
                    return StudioWindowTimerDriverTransition::KeepNativeTimer {
                        window_id: *window_id,
                        binding: binding.clone(),
                    };
                }

                self.pending_bindings.insert(
                    *window_id,
                    StudioWindowPendingTimerBinding {
                        window_id: *window_id,
                        slot: slot.clone(),
                    },
                );
                StudioWindowTimerDriverTransition::ArmNativeTimer {
                    window_id: *window_id,
                    previous_binding: None,
                    slot: slot.clone(),
                }
            }
            StudioWindowHostTimerDriverCommand::Clear {
                window_id,
                previous_slot: _,
            } => {
                self.pending_bindings.remove(window_id);
                let previous_binding = self.window_bindings.remove(window_id);
                StudioWindowTimerDriverTransition::ClearNativeTimer {
                    window_id: *window_id,
                    previous_binding,
                }
            }
            StudioWindowHostTimerDriverCommand::IgnoreStale {
                window_id,
                stale_effect_id,
                ..
            } => StudioWindowTimerDriverTransition::IgnoreStale {
                window_id: *window_id,
                current_binding: self.window_bindings.get(window_id).cloned(),
                stale_effect_id: *stale_effect_id,
            },
            StudioWindowHostTimerDriverCommand::Transfer {
                from_window_id,
                to_window_id,
                slot,
            } => {
                self.pending_bindings.remove(from_window_id);
                self.pending_bindings.remove(to_window_id);
                let binding = self
                    .window_bindings
                    .remove(from_window_id)
                    .map(|mut binding| {
                        binding.slot = slot.clone();
                        self.window_bindings.insert(*to_window_id, binding.clone());
                        binding
                    });

                if binding.is_none() {
                    self.pending_bindings.insert(
                        *to_window_id,
                        StudioWindowPendingTimerBinding {
                            window_id: *to_window_id,
                            slot: slot.clone(),
                        },
                    );
                    return StudioWindowTimerDriverTransition::ArmNativeTimer {
                        window_id: *to_window_id,
                        previous_binding: self.window_bindings.remove(to_window_id),
                        slot: slot.clone(),
                    };
                }

                StudioWindowTimerDriverTransition::TransferNativeTimer {
                    from_window_id: *from_window_id,
                    to_window_id: *to_window_id,
                    binding,
                    requested_slot: slot.clone(),
                }
            }
            StudioWindowHostTimerDriverCommand::Park {
                from_window_id,
                slot,
            } => {
                self.pending_bindings.remove(from_window_id);
                let binding = self
                    .window_bindings
                    .remove(from_window_id)
                    .map(|mut binding| {
                        binding.slot = slot.clone();
                        self.parked_binding = Some(binding.clone());
                        binding
                    });
                if binding.is_none() {
                    self.parked_binding = None;
                }
                StudioWindowTimerDriverTransition::ParkNativeTimer {
                    from_window_id: *from_window_id,
                    binding,
                    requested_slot: slot.clone(),
                }
            }
            StudioWindowHostTimerDriverCommand::RestoreParked { window_id, slot } => {
                self.pending_bindings.remove(window_id);
                let binding = self.parked_binding.take().map(|mut binding| {
                    binding.slot = slot.clone();
                    self.window_bindings.insert(*window_id, binding.clone());
                    binding
                });

                if binding.is_none() {
                    self.pending_bindings.insert(
                        *window_id,
                        StudioWindowPendingTimerBinding {
                            window_id: *window_id,
                            slot: slot.clone(),
                        },
                    );
                    return StudioWindowTimerDriverTransition::ArmNativeTimer {
                        window_id: *window_id,
                        previous_binding: self.window_bindings.remove(window_id),
                        slot: slot.clone(),
                    };
                }

                StudioWindowTimerDriverTransition::RestoreParkedTimer {
                    window_id: *window_id,
                    binding,
                    requested_slot: slot.clone(),
                }
            }
        }
    }

    pub fn acknowledge_native_timer(
        &mut self,
        window_id: StudioWindowHostId,
        handle_id: StudioWindowNativeTimerHandleId,
    ) -> StudioWindowTimerDriverAckResult {
        let Some(pending) = self.pending_bindings.remove(&window_id) else {
            return StudioWindowTimerDriverAckResult {
                window_id,
                handle_id,
                status: StudioWindowTimerDriverAckStatus::NoPendingBinding,
            };
        };

        self.window_bindings.insert(
            window_id,
            StudioWindowNativeTimerBinding {
                handle_id,
                slot: pending.slot,
            },
        );

        StudioWindowTimerDriverAckResult {
            window_id,
            handle_id,
            status: StudioWindowTimerDriverAckStatus::Applied,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use crate::{
        EntitlementSessionTimerArm, EntitlementSessionTimerReason, StudioRuntimeTimerHandleSlot,
        StudioWindowHostTimerDriverCommand, StudioWindowTimerDriverAckStatus,
        StudioWindowTimerDriverState, StudioWindowTimerDriverTransition,
    };

    fn slot(effect_id: u64, seconds: u64) -> StudioRuntimeTimerHandleSlot {
        StudioRuntimeTimerHandleSlot {
            effect_id,
            timer: EntitlementSessionTimerArm {
                due_at: UNIX_EPOCH + Duration::from_secs(seconds),
                delay: Duration::from_secs(seconds),
                reason: EntitlementSessionTimerReason::ScheduledCheck,
                event: crate::EntitlementSessionLifecycleEvent::TimerElapsed,
            },
        }
    }

    #[test]
    fn timer_driver_state_arms_and_acknowledges_new_native_handle() {
        let mut state = StudioWindowTimerDriverState::default();
        let command = StudioWindowHostTimerDriverCommand::Arm {
            window_id: 7,
            slot: slot(1, 30),
        };

        let transition = state.apply_command(&command);

        assert!(matches!(
            transition,
            StudioWindowTimerDriverTransition::ArmNativeTimer {
                window_id: 7,
                previous_binding: None,
                ..
            }
        ));
        assert_eq!(
            state
                .pending_binding(7)
                .map(|binding| binding.slot.effect_id),
            Some(1)
        );
        assert_eq!(
            state.acknowledge_native_timer(7, 100).status,
            StudioWindowTimerDriverAckStatus::Applied
        );
        assert_eq!(
            state.window_binding(7).map(|binding| binding.handle_id),
            Some(100)
        );
    }

    #[test]
    fn timer_driver_state_keeps_existing_native_handle_without_new_ack() {
        let mut state = StudioWindowTimerDriverState::default();
        let _ = state.apply_command(&StudioWindowHostTimerDriverCommand::Arm {
            window_id: 7,
            slot: slot(1, 30),
        });
        let _ = state.acknowledge_native_timer(7, 100);

        let transition = state.apply_command(&StudioWindowHostTimerDriverCommand::Keep {
            window_id: 7,
            slot: slot(2, 60),
        });

        assert!(matches!(
            transition,
            StudioWindowTimerDriverTransition::KeepNativeTimer { window_id: 7, binding }
            if binding.handle_id == 100 && binding.slot.effect_id == 2
        ));
        assert!(state.pending_binding(7).is_none());
    }

    #[test]
    fn timer_driver_state_transfers_existing_binding_between_windows() {
        let mut state = StudioWindowTimerDriverState::default();
        let _ = state.apply_command(&StudioWindowHostTimerDriverCommand::Arm {
            window_id: 7,
            slot: slot(1, 30),
        });
        let _ = state.acknowledge_native_timer(7, 100);

        let transition = state.apply_command(&StudioWindowHostTimerDriverCommand::Transfer {
            from_window_id: 7,
            to_window_id: 9,
            slot: slot(2, 60),
        });

        assert!(matches!(
            transition,
            StudioWindowTimerDriverTransition::TransferNativeTimer {
                from_window_id: 7,
                to_window_id: 9,
                binding: Some(binding),
                requested_slot,
            } if binding.handle_id == 100 && requested_slot.effect_id == 2
        ));
        assert!(state.window_binding(7).is_none());
        assert_eq!(
            state.window_binding(9).map(|binding| binding.handle_id),
            Some(100)
        );
    }

    #[test]
    fn timer_driver_state_parks_and_restores_existing_binding() {
        let mut state = StudioWindowTimerDriverState::default();
        let _ = state.apply_command(&StudioWindowHostTimerDriverCommand::Arm {
            window_id: 7,
            slot: slot(1, 30),
        });
        let _ = state.acknowledge_native_timer(7, 100);

        let park = state.apply_command(&StudioWindowHostTimerDriverCommand::Park {
            from_window_id: 7,
            slot: slot(2, 60),
        });
        assert!(matches!(
            park,
            StudioWindowTimerDriverTransition::ParkNativeTimer {
                from_window_id: 7,
                binding: Some(binding),
                requested_slot,
            } if binding.handle_id == 100 && requested_slot.effect_id == 2
        ));
        assert_eq!(
            state.parked_binding().map(|binding| binding.handle_id),
            Some(100)
        );

        let restore = state.apply_command(&StudioWindowHostTimerDriverCommand::RestoreParked {
            window_id: 9,
            slot: slot(3, 90),
        });
        assert!(matches!(
            restore,
            StudioWindowTimerDriverTransition::RestoreParkedTimer {
                window_id: 9,
                binding: Some(binding),
                requested_slot,
            } if binding.handle_id == 100 && requested_slot.effect_id == 3
        ));
        assert!(state.parked_binding().is_none());
        assert_eq!(
            state.window_binding(9).map(|binding| binding.handle_id),
            Some(100)
        );
    }

    #[test]
    fn timer_driver_state_rearms_when_restore_has_no_native_binding() {
        let mut state = StudioWindowTimerDriverState::default();

        let transition = state.apply_command(&StudioWindowHostTimerDriverCommand::RestoreParked {
            window_id: 9,
            slot: slot(3, 90),
        });

        assert!(matches!(
            transition,
            StudioWindowTimerDriverTransition::ArmNativeTimer { window_id: 9, previous_binding: None, slot }
            if slot.effect_id == 3
        ));
        assert_eq!(
            state
                .pending_binding(9)
                .map(|binding| binding.slot.effect_id),
            Some(3)
        );
    }
}
