use crate::{
    StudioRuntimeHostEffectId, StudioRuntimeTimerHandleSlot, StudioWindowHostId,
    StudioWindowNativeTimerBinding, StudioWindowTimerDriverAckResult,
    StudioWindowTimerDriverTransition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiNativeTimerOperation {
    Arm {
        window_id: StudioWindowHostId,
        previous_binding: Option<StudioWindowNativeTimerBinding>,
        slot: StudioRuntimeTimerHandleSlot,
    },
    Keep {
        window_id: StudioWindowHostId,
        binding: StudioWindowNativeTimerBinding,
    },
    Rearm {
        window_id: StudioWindowHostId,
        previous_binding: Option<StudioWindowNativeTimerBinding>,
        next_slot: StudioRuntimeTimerHandleSlot,
    },
    Clear {
        window_id: StudioWindowHostId,
        previous_binding: Option<StudioWindowNativeTimerBinding>,
    },
    IgnoreStale {
        window_id: StudioWindowHostId,
        current_binding: Option<StudioWindowNativeTimerBinding>,
        stale_effect_id: StudioRuntimeHostEffectId,
    },
    Transfer {
        from_window_id: StudioWindowHostId,
        to_window_id: StudioWindowHostId,
        binding: Option<StudioWindowNativeTimerBinding>,
        requested_slot: StudioRuntimeTimerHandleSlot,
    },
    Park {
        from_window_id: StudioWindowHostId,
        binding: Option<StudioWindowNativeTimerBinding>,
        requested_slot: StudioRuntimeTimerHandleSlot,
    },
    RestoreParked {
        window_id: StudioWindowHostId,
        binding: Option<StudioWindowNativeTimerBinding>,
        requested_slot: StudioRuntimeTimerHandleSlot,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StudioGuiNativeTimerEffects {
    pub operations: Vec<StudioGuiNativeTimerOperation>,
    pub acks: Vec<StudioWindowTimerDriverAckResult>,
}

impl StudioGuiNativeTimerEffects {
    pub fn from_driver(
        transitions: &[StudioWindowTimerDriverTransition],
        acks: &[StudioWindowTimerDriverAckResult],
    ) -> Self {
        Self {
            operations: transitions
                .iter()
                .cloned()
                .map(StudioGuiNativeTimerOperation::from_transition)
                .collect(),
            acks: acks.to_vec(),
        }
    }
}

impl StudioGuiNativeTimerOperation {
    pub fn from_transition(transition: StudioWindowTimerDriverTransition) -> Self {
        match transition {
            StudioWindowTimerDriverTransition::ArmNativeTimer {
                window_id,
                previous_binding,
                slot,
            } => Self::Arm {
                window_id,
                previous_binding,
                slot,
            },
            StudioWindowTimerDriverTransition::KeepNativeTimer { window_id, binding } => {
                Self::Keep { window_id, binding }
            }
            StudioWindowTimerDriverTransition::RearmNativeTimer {
                window_id,
                previous_binding,
                next_slot,
            } => Self::Rearm {
                window_id,
                previous_binding,
                next_slot,
            },
            StudioWindowTimerDriverTransition::ClearNativeTimer {
                window_id,
                previous_binding,
            } => Self::Clear {
                window_id,
                previous_binding,
            },
            StudioWindowTimerDriverTransition::IgnoreStale {
                window_id,
                current_binding,
                stale_effect_id,
            } => Self::IgnoreStale {
                window_id,
                current_binding,
                stale_effect_id,
            },
            StudioWindowTimerDriverTransition::TransferNativeTimer {
                from_window_id,
                to_window_id,
                binding,
                requested_slot,
            } => Self::Transfer {
                from_window_id,
                to_window_id,
                binding,
                requested_slot,
            },
            StudioWindowTimerDriverTransition::ParkNativeTimer {
                from_window_id,
                binding,
                requested_slot,
            } => Self::Park {
                from_window_id,
                binding,
                requested_slot,
            },
            StudioWindowTimerDriverTransition::RestoreParkedTimer {
                window_id,
                binding,
                requested_slot,
            } => Self::RestoreParked {
                window_id,
                binding,
                requested_slot,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use crate::{
        EntitlementSessionLifecycleEvent, EntitlementSessionTimerArm, EntitlementSessionTimerReason,
        StudioGuiNativeTimerEffects, StudioGuiNativeTimerOperation, StudioRuntimeTimerHandleSlot,
        StudioWindowNativeTimerBinding, StudioWindowTimerDriverAckResult,
        StudioWindowTimerDriverAckStatus, StudioWindowTimerDriverTransition,
    };

    fn slot(effect_id: u64, seconds: u64) -> StudioRuntimeTimerHandleSlot {
        StudioRuntimeTimerHandleSlot {
            effect_id,
            timer: EntitlementSessionTimerArm {
                due_at: UNIX_EPOCH + Duration::from_secs(seconds),
                delay: Duration::from_secs(seconds),
                reason: EntitlementSessionTimerReason::ScheduledCheck,
                event: EntitlementSessionLifecycleEvent::TimerElapsed,
            },
        }
    }

    #[test]
    fn gui_native_timer_effects_map_driver_transitions_to_gui_operations() {
        let transitions = vec![
            StudioWindowTimerDriverTransition::ArmNativeTimer {
                window_id: 7,
                previous_binding: None,
                slot: slot(1, 30),
            },
            StudioWindowTimerDriverTransition::TransferNativeTimer {
                from_window_id: 7,
                to_window_id: 9,
                binding: Some(StudioWindowNativeTimerBinding {
                    handle_id: 100,
                    slot: slot(2, 60),
                }),
                requested_slot: slot(3, 90),
            },
        ];
        let acks = vec![StudioWindowTimerDriverAckResult {
            window_id: 7,
            handle_id: 100,
            status: StudioWindowTimerDriverAckStatus::Applied,
        }];

        let effects = StudioGuiNativeTimerEffects::from_driver(&transitions, &acks);

        assert_eq!(
            effects.operations,
            vec![
                StudioGuiNativeTimerOperation::Arm {
                    window_id: 7,
                    previous_binding: None,
                    slot: slot(1, 30),
                },
                StudioGuiNativeTimerOperation::Transfer {
                    from_window_id: 7,
                    to_window_id: 9,
                    binding: Some(StudioWindowNativeTimerBinding {
                        handle_id: 100,
                        slot: slot(2, 60),
                    }),
                    requested_slot: slot(3, 90),
                },
            ]
        );
        assert_eq!(effects.acks, acks);
    }
}
