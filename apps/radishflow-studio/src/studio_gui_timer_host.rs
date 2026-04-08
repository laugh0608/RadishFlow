use std::collections::{BTreeMap, BTreeSet};
use std::time::SystemTime;

use crate::{
    StudioRuntimeHostEffectId, StudioRuntimeTimerHandleSlot, StudioWindowHostId,
    StudioWindowNativeTimerBinding, StudioWindowNativeTimerHandleId,
    StudioWindowTimerDriverAckResult, StudioWindowTimerDriverAckStatus,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiNativeTimerDueEvent {
    pub window_id: Option<StudioWindowHostId>,
    pub handle_id: StudioWindowNativeTimerHandleId,
    pub slot: StudioRuntimeTimerHandleSlot,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StudioGuiNativeTimerRuntime {
    window_bindings: BTreeMap<StudioWindowHostId, StudioWindowNativeTimerBinding>,
    parked_binding: Option<StudioWindowNativeTimerBinding>,
    delivered_effect_ids: BTreeSet<StudioRuntimeHostEffectId>,
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

impl StudioGuiNativeTimerRuntime {
    pub fn window_binding(
        &self,
        window_id: StudioWindowHostId,
    ) -> Option<&StudioWindowNativeTimerBinding> {
        self.window_bindings.get(&window_id)
    }

    pub fn parked_binding(&self) -> Option<&StudioWindowNativeTimerBinding> {
        self.parked_binding.as_ref()
    }

    pub fn next_due_at(&self) -> Option<SystemTime> {
        self.pending_bindings()
            .into_iter()
            .map(|binding| binding.slot.timer.due_at)
            .min()
    }

    pub fn apply_effects(&mut self, effects: &StudioGuiNativeTimerEffects) {
        let applied_acks = effects
            .acks
            .iter()
            .filter(|ack| ack.status == StudioWindowTimerDriverAckStatus::Applied)
            .map(|ack| (ack.window_id, ack.handle_id))
            .collect::<BTreeMap<_, _>>();

        for operation in &effects.operations {
            self.apply_operation(operation, &applied_acks);
        }
    }

    pub fn drain_due_events(&mut self, now: SystemTime) -> Vec<StudioGuiNativeTimerDueEvent> {
        let mut due = self
            .window_bindings
            .iter()
            .filter_map(|(window_id, binding)| {
                let effect_id = binding.slot.effect_id;
                (binding.slot.timer.due_at <= now
                    && !self.delivered_effect_ids.contains(&effect_id))
                .then(|| StudioGuiNativeTimerDueEvent {
                    window_id: Some(*window_id),
                    handle_id: binding.handle_id,
                    slot: binding.slot.clone(),
                })
            })
            .collect::<Vec<_>>();

        if let Some(binding) = self.parked_binding.as_ref() {
            let effect_id = binding.slot.effect_id;
            if binding.slot.timer.due_at <= now && !self.delivered_effect_ids.contains(&effect_id) {
                due.push(StudioGuiNativeTimerDueEvent {
                    window_id: None,
                    handle_id: binding.handle_id,
                    slot: binding.slot.clone(),
                });
            }
        }

        for event in &due {
            self.delivered_effect_ids.insert(event.slot.effect_id);
        }

        due
    }

    pub fn consume_elapsed_event(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        handle_id: StudioWindowNativeTimerHandleId,
    ) -> Option<StudioGuiNativeTimerDueEvent> {
        let binding = match window_id {
            Some(window_id) => self.window_bindings.get(&window_id).and_then(|binding| {
                (binding.handle_id == handle_id).then_some((Some(window_id), binding))
            }),
            None => self
                .parked_binding
                .as_ref()
                .and_then(|binding| (binding.handle_id == handle_id).then_some((None, binding))),
        }?;

        let effect_id = binding.1.slot.effect_id;
        if self.delivered_effect_ids.contains(&effect_id) {
            return None;
        }
        self.delivered_effect_ids.insert(effect_id);

        Some(StudioGuiNativeTimerDueEvent {
            window_id: binding.0,
            handle_id,
            slot: binding.1.slot.clone(),
        })
    }

    fn apply_operation(
        &mut self,
        operation: &StudioGuiNativeTimerOperation,
        applied_acks: &BTreeMap<StudioWindowHostId, StudioWindowNativeTimerHandleId>,
    ) {
        match operation {
            StudioGuiNativeTimerOperation::Arm {
                window_id, slot, ..
            }
            | StudioGuiNativeTimerOperation::Rearm {
                window_id,
                next_slot: slot,
                ..
            } => {
                self.parked_binding = None;
                let binding =
                    applied_acks
                        .get(window_id)
                        .map(|handle_id| StudioWindowNativeTimerBinding {
                            handle_id: *handle_id,
                            slot: slot.clone(),
                        });
                self.replace_window_binding(*window_id, binding);
            }
            StudioGuiNativeTimerOperation::Keep { window_id, binding } => {
                self.parked_binding = None;
                self.replace_window_binding(*window_id, Some(binding.clone()));
            }
            StudioGuiNativeTimerOperation::Clear { window_id, .. } => {
                self.remove_window_binding(*window_id);
            }
            StudioGuiNativeTimerOperation::IgnoreStale { .. } => {}
            StudioGuiNativeTimerOperation::Transfer {
                from_window_id,
                to_window_id,
                binding,
                ..
            } => {
                self.window_bindings.remove(from_window_id);
                self.replace_window_binding(*to_window_id, binding.clone());
                self.parked_binding = None;
            }
            StudioGuiNativeTimerOperation::Park {
                from_window_id,
                binding,
                ..
            } => {
                self.window_bindings.remove(from_window_id);
                self.replace_parked_binding(binding.clone());
            }
            StudioGuiNativeTimerOperation::RestoreParked {
                window_id, binding, ..
            } => {
                self.parked_binding = None;
                self.replace_window_binding(*window_id, binding.clone());
            }
        }
    }

    fn pending_bindings(&self) -> Vec<&StudioWindowNativeTimerBinding> {
        let mut bindings = self.window_bindings.values().collect::<Vec<_>>();
        if let Some(binding) = self.parked_binding.as_ref() {
            bindings.push(binding);
        }
        bindings
            .into_iter()
            .filter(|binding| !self.delivered_effect_ids.contains(&binding.slot.effect_id))
            .collect()
    }

    fn replace_window_binding(
        &mut self,
        window_id: StudioWindowHostId,
        next: Option<StudioWindowNativeTimerBinding>,
    ) {
        let previous_effect_id = self
            .window_bindings
            .get(&window_id)
            .map(|binding| binding.slot.effect_id);
        match next {
            Some(binding) => {
                if previous_effect_id != Some(binding.slot.effect_id) {
                    self.delivered_effect_ids.remove(&binding.slot.effect_id);
                }
                self.window_bindings.insert(window_id, binding);
            }
            None => {
                if let Some(previous_effect_id) = previous_effect_id {
                    self.delivered_effect_ids.remove(&previous_effect_id);
                }
                self.window_bindings.remove(&window_id);
            }
        }
    }

    fn remove_window_binding(&mut self, window_id: StudioWindowHostId) {
        if let Some(previous) = self.window_bindings.remove(&window_id) {
            self.delivered_effect_ids.remove(&previous.slot.effect_id);
        }
    }

    fn replace_parked_binding(&mut self, next: Option<StudioWindowNativeTimerBinding>) {
        let previous_effect_id = self
            .parked_binding
            .as_ref()
            .map(|binding| binding.slot.effect_id);
        match next {
            Some(binding) => {
                if previous_effect_id != Some(binding.slot.effect_id) {
                    self.delivered_effect_ids.remove(&binding.slot.effect_id);
                }
                self.parked_binding = Some(binding);
            }
            None => {
                if let Some(previous_effect_id) = previous_effect_id {
                    self.delivered_effect_ids.remove(&previous_effect_id);
                }
                self.parked_binding = None;
            }
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
        EntitlementSessionLifecycleEvent, EntitlementSessionTimerArm,
        EntitlementSessionTimerReason, StudioGuiNativeTimerDueEvent, StudioGuiNativeTimerEffects,
        StudioGuiNativeTimerOperation, StudioGuiNativeTimerRuntime, StudioRuntimeTimerHandleSlot,
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

    #[test]
    fn gui_native_timer_runtime_tracks_bindings_across_effects_and_drains_due_once() {
        let mut runtime = StudioGuiNativeTimerRuntime::default();
        runtime.apply_effects(&StudioGuiNativeTimerEffects {
            operations: vec![StudioGuiNativeTimerOperation::Arm {
                window_id: 7,
                previous_binding: None,
                slot: slot(1, 30),
            }],
            acks: vec![StudioWindowTimerDriverAckResult {
                window_id: 7,
                handle_id: 100,
                status: StudioWindowTimerDriverAckStatus::Applied,
            }],
        });

        assert_eq!(
            runtime.window_binding(7),
            Some(&StudioWindowNativeTimerBinding {
                handle_id: 100,
                slot: slot(1, 30),
            })
        );
        assert_eq!(runtime.next_due_at(), Some(slot(1, 30).timer.due_at));

        assert_eq!(
            runtime.drain_due_events(slot(1, 30).timer.due_at),
            vec![StudioGuiNativeTimerDueEvent {
                window_id: Some(7),
                handle_id: 100,
                slot: slot(1, 30),
            }]
        );
        assert!(
            runtime
                .drain_due_events(slot(1, 30).timer.due_at)
                .is_empty()
        );
    }

    #[test]
    fn gui_native_timer_runtime_tracks_park_and_restore_without_losing_handle() {
        let mut runtime = StudioGuiNativeTimerRuntime::default();
        runtime.apply_effects(&StudioGuiNativeTimerEffects {
            operations: vec![StudioGuiNativeTimerOperation::Keep {
                window_id: 7,
                binding: StudioWindowNativeTimerBinding {
                    handle_id: 100,
                    slot: slot(1, 30),
                },
            }],
            acks: Vec::new(),
        });
        runtime.apply_effects(&StudioGuiNativeTimerEffects {
            operations: vec![StudioGuiNativeTimerOperation::Park {
                from_window_id: 7,
                binding: Some(StudioWindowNativeTimerBinding {
                    handle_id: 100,
                    slot: slot(2, 60),
                }),
                requested_slot: slot(2, 60),
            }],
            acks: Vec::new(),
        });

        assert!(runtime.window_binding(7).is_none());
        assert_eq!(
            runtime.parked_binding(),
            Some(&StudioWindowNativeTimerBinding {
                handle_id: 100,
                slot: slot(2, 60),
            })
        );

        runtime.apply_effects(&StudioGuiNativeTimerEffects {
            operations: vec![StudioGuiNativeTimerOperation::RestoreParked {
                window_id: 9,
                binding: Some(StudioWindowNativeTimerBinding {
                    handle_id: 100,
                    slot: slot(2, 60),
                }),
                requested_slot: slot(2, 60),
            }],
            acks: Vec::new(),
        });

        assert!(runtime.parked_binding().is_none());
        assert_eq!(
            runtime.window_binding(9),
            Some(&StudioWindowNativeTimerBinding {
                handle_id: 100,
                slot: slot(2, 60),
            })
        );
    }

    #[test]
    fn gui_native_timer_runtime_accepts_current_elapsed_callback_and_rejects_stale_repeats() {
        let mut runtime = StudioGuiNativeTimerRuntime::default();
        runtime.apply_effects(&StudioGuiNativeTimerEffects {
            operations: vec![StudioGuiNativeTimerOperation::Keep {
                window_id: 7,
                binding: StudioWindowNativeTimerBinding {
                    handle_id: 100,
                    slot: slot(1, 30),
                },
            }],
            acks: Vec::new(),
        });

        assert_eq!(
            runtime.consume_elapsed_event(Some(7), 100),
            Some(StudioGuiNativeTimerDueEvent {
                window_id: Some(7),
                handle_id: 100,
                slot: slot(1, 30),
            })
        );
        assert_eq!(runtime.consume_elapsed_event(Some(7), 100), None);
        assert_eq!(runtime.consume_elapsed_event(Some(7), 999), None);
    }
}
