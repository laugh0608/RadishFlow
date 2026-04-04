use std::collections::BTreeMap;

use rf_types::{RfError, RfResult};

use crate::{
    StudioRuntime, StudioRuntimeConfig, StudioRuntimeHostAckResult, StudioRuntimeTimerHandleSlot,
    StudioRuntimeTimerHostCommand, StudioRuntimeTimerHostState, StudioRuntimeTimerHostTransition,
    StudioRuntimeTrigger,
};

pub type StudioWindowHostId = u64;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StudioWindowHostState {
    entitlement_timer: StudioRuntimeTimerHostState,
}

impl StudioWindowHostState {
    pub fn entitlement_timer(&self) -> Option<&StudioRuntimeTimerHandleSlot> {
        self.entitlement_timer.entitlement_timer()
    }

    fn apply_timer_command(
        &mut self,
        command: &StudioRuntimeTimerHostCommand,
    ) -> StudioRuntimeTimerHostTransition {
        self.entitlement_timer.apply_command(command)
    }

    fn restore_entitlement_timer(&mut self, slot: StudioRuntimeTimerHandleSlot) {
        self.entitlement_timer.restore(slot);
    }

    pub fn prepare_shutdown(&mut self) -> StudioWindowHostShutdown {
        StudioWindowHostShutdown {
            window_id: 0,
            was_entitlement_timer_owner: false,
            cleared_entitlement_timer: self.entitlement_timer.clear(),
            retirement: StudioWindowHostRetirement::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioWindowHostEvent {
    EntitlementTimerApplied {
        window_id: StudioWindowHostId,
        command: StudioRuntimeTimerHostCommand,
        transition: StudioRuntimeTimerHostTransition,
        ack: StudioRuntimeHostAckResult,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioWindowHostRole {
    EntitlementTimerOwner,
    Observer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWindowHostRegistration {
    pub window_id: StudioWindowHostId,
    pub role: StudioWindowHostRole,
    pub restored_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioWindowHostRetirement {
    None,
    Transferred {
        new_owner_window_id: StudioWindowHostId,
        restored_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
    },
    Parked {
        parked_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWindowHostShutdown {
    pub window_id: StudioWindowHostId,
    pub was_entitlement_timer_owner: bool,
    pub cleared_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
    pub retirement: StudioWindowHostRetirement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRuntimeHostPortOutput {
    pub runtime_output: crate::StudioRuntimeOutput,
    pub window_event: Option<StudioWindowHostEvent>,
}

pub struct StudioRuntimeHostPort {
    runtime: StudioRuntime,
    next_window_id: StudioWindowHostId,
    entitlement_timer_owner: Option<StudioWindowHostId>,
    parked_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
    windows: BTreeMap<StudioWindowHostId, StudioWindowHostState>,
}

impl StudioRuntimeHostPort {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            runtime: StudioRuntime::new(config)?,
            next_window_id: 1,
            entitlement_timer_owner: None,
            parked_entitlement_timer: None,
            windows: BTreeMap::new(),
        })
    }

    pub fn runtime(&self) -> &StudioRuntime {
        &self.runtime
    }

    pub fn entitlement_timer_owner(&self) -> Option<StudioWindowHostId> {
        self.entitlement_timer_owner
    }

    pub fn parked_entitlement_timer(&self) -> Option<&StudioRuntimeTimerHandleSlot> {
        self.parked_entitlement_timer.as_ref()
    }

    pub fn window_state(&self, window_id: StudioWindowHostId) -> Option<&StudioWindowHostState> {
        self.windows.get(&window_id)
    }

    pub fn open_window(&mut self) -> StudioWindowHostRegistration {
        let window_id = self.allocate_window_id();
        let role = if self.entitlement_timer_owner.is_none() {
            self.entitlement_timer_owner = Some(window_id);
            StudioWindowHostRole::EntitlementTimerOwner
        } else {
            StudioWindowHostRole::Observer
        };
        let mut state = StudioWindowHostState::default();
        let restored_entitlement_timer =
            if matches!(role, StudioWindowHostRole::EntitlementTimerOwner) {
                self.parked_entitlement_timer.take().inspect(|slot| {
                    state.restore_entitlement_timer(slot.clone());
                })
            } else {
                None
            };
        self.windows.insert(window_id, state);

        StudioWindowHostRegistration {
            window_id,
            role,
            restored_entitlement_timer,
        }
    }

    pub fn dispatch_trigger(
        &mut self,
        window_id: StudioWindowHostId,
        trigger: &StudioRuntimeTrigger,
    ) -> RfResult<StudioRuntimeHostPortOutput> {
        if !self.windows.contains_key(&window_id) {
            return Err(RfError::invalid_input(format!(
                "window host `{window_id}` is not registered"
            )));
        }

        let runtime_output = self.runtime.dispatch_trigger_output(trigger)?;
        let window_event = self.apply_runtime_output(window_id, &runtime_output);

        Ok(StudioRuntimeHostPortOutput {
            runtime_output,
            window_event,
        })
    }

    pub fn close_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> Option<StudioWindowHostShutdown> {
        let mut state = self.windows.remove(&window_id)?;
        let mut shutdown = state.prepare_shutdown();
        shutdown.window_id = window_id;
        shutdown.was_entitlement_timer_owner = self.entitlement_timer_owner == Some(window_id);

        if shutdown.was_entitlement_timer_owner {
            if let Some(new_owner_window_id) = self.windows.keys().next().copied() {
                if let Some(slot) = shutdown.cleared_entitlement_timer.clone() {
                    self.windows
                        .get_mut(&new_owner_window_id)
                        .expect("expected replacement window to exist")
                        .restore_entitlement_timer(slot.clone());
                }
                self.entitlement_timer_owner = Some(new_owner_window_id);
                shutdown.retirement = StudioWindowHostRetirement::Transferred {
                    new_owner_window_id,
                    restored_entitlement_timer: shutdown.cleared_entitlement_timer.clone(),
                };
            } else {
                self.entitlement_timer_owner = None;
                self.parked_entitlement_timer = shutdown.cleared_entitlement_timer.clone();
                shutdown.retirement = StudioWindowHostRetirement::Parked {
                    parked_entitlement_timer: shutdown.cleared_entitlement_timer.clone(),
                };
            }
        }

        Some(shutdown)
    }

    fn apply_runtime_output(
        &mut self,
        window_id: StudioWindowHostId,
        output: &crate::StudioRuntimeOutput,
    ) -> Option<StudioWindowHostEvent> {
        let command = output.entitlement_timer_host_command()?.clone();
        let owner_window_id = self
            .entitlement_timer_owner
            .unwrap_or_else(|| self.ensure_entitlement_timer_owner(window_id));
        let transition = self
            .windows
            .get_mut(&owner_window_id)
            .expect("expected timer owner window to exist")
            .apply_timer_command(&command);
        let ack = self
            .runtime
            .acknowledge_entitlement_timer_host_command(&command);

        Some(StudioWindowHostEvent::EntitlementTimerApplied {
            window_id: owner_window_id,
            command,
            transition,
            ack,
        })
    }

    fn ensure_entitlement_timer_owner(
        &mut self,
        fallback_window_id: StudioWindowHostId,
    ) -> StudioWindowHostId {
        let owner_window_id = self
            .windows
            .keys()
            .next()
            .copied()
            .unwrap_or(fallback_window_id);
        self.entitlement_timer_owner = Some(owner_window_id);

        if let Some(slot) = self.parked_entitlement_timer.take() {
            self.windows
                .get_mut(&owner_window_id)
                .expect("expected timer owner window to exist")
                .restore_entitlement_timer(slot);
        }

        owner_window_id
    }

    fn allocate_window_id(&mut self) -> StudioWindowHostId {
        let window_id = self.next_window_id;
        self.next_window_id += 1;
        window_id
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeHostAckStatus, StudioRuntimeHostPort,
        StudioRuntimeHostPortOutput, StudioRuntimeTimerHostCommand,
        StudioRuntimeTimerHostTransition, StudioRuntimeTrigger, StudioWindowHostEvent,
        StudioWindowHostRetirement, StudioWindowHostRole,
    };

    fn lease_expiring_config() -> crate::StudioRuntimeConfig {
        crate::StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..crate::StudioRuntimeConfig::default()
        }
    }

    #[test]
    fn runtime_host_port_applies_entitlement_timer_command_into_window_state() {
        let mut host_port =
            StudioRuntimeHostPort::new(&lease_expiring_config()).expect("expected host port");
        let window = host_port.open_window();

        let output = host_port
            .dispatch_trigger(
                window.window_id,
                &StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected timer elapsed output");

        match timer_event(&output) {
            StudioWindowHostEvent::EntitlementTimerApplied {
                window_id,
                command,
                transition,
                ack,
            } => {
                assert_eq!(*window_id, window.window_id);
                assert!(matches!(
                    command,
                    StudioRuntimeTimerHostCommand::RearmTimer { .. }
                ));
                assert!(matches!(
                    transition,
                    StudioRuntimeTimerHostTransition::RearmTimer { .. }
                ));
                assert_eq!(ack.status, StudioRuntimeHostAckStatus::Applied);
            }
        }
        assert_eq!(
            host_port
                .window_state(window.window_id)
                .expect("expected window state")
                .entitlement_timer()
                .map(|slot| slot.effect_id),
            Some(1)
        );
        assert!(host_port.runtime().pending_host_effects().is_empty());
    }

    #[test]
    fn first_window_becomes_timer_owner_and_second_window_is_observer() {
        let mut host_port =
            StudioRuntimeHostPort::new(&lease_expiring_config()).expect("expected host port");
        let first = host_port.open_window();
        let second = host_port.open_window();

        assert_eq!(first.role, StudioWindowHostRole::EntitlementTimerOwner);
        assert_eq!(second.role, StudioWindowHostRole::Observer);
        assert_eq!(host_port.entitlement_timer_owner(), Some(first.window_id));
    }

    #[test]
    fn runtime_host_port_keeps_timer_owner_stable_across_secondary_window_dispatches() {
        let mut host_port =
            StudioRuntimeHostPort::new(&lease_expiring_config()).expect("expected host port");
        let first = host_port.open_window();
        let second = host_port.open_window();

        let _ = host_port
            .dispatch_trigger(
                first.window_id,
                &StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected first output");
        let output = host_port
            .dispatch_trigger(
                second.window_id,
                &StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::NetworkRestored,
                ),
            )
            .expect("expected second output");

        match timer_event(&output) {
            StudioWindowHostEvent::EntitlementTimerApplied {
                window_id,
                command,
                transition,
                ack,
            } => {
                assert_eq!(*window_id, first.window_id);
                assert!(matches!(
                    command,
                    StudioRuntimeTimerHostCommand::KeepTimer { effect_id: 2, .. }
                ));
                assert!(matches!(
                    transition,
                    StudioRuntimeTimerHostTransition::KeepTimer { .. }
                ));
                assert_eq!(ack.status, StudioRuntimeHostAckStatus::Applied);
            }
        }
        assert_eq!(
            host_port
                .window_state(first.window_id)
                .expect("expected owner window state")
                .entitlement_timer()
                .map(|slot| slot.effect_id),
            Some(2)
        );
        assert!(
            host_port
                .window_state(second.window_id)
                .expect("expected observer window state")
                .entitlement_timer()
                .is_none()
        );
        assert_eq!(host_port.entitlement_timer_owner(), Some(first.window_id));
    }

    #[test]
    fn runtime_host_port_transfers_timer_slot_to_remaining_window_when_owner_closes() {
        let mut host_port =
            StudioRuntimeHostPort::new(&lease_expiring_config()).expect("expected host port");
        let first = host_port.open_window();
        let second = host_port.open_window();

        let _ = host_port
            .dispatch_trigger(
                first.window_id,
                &StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected timer elapsed output");

        let shutdown = host_port
            .close_window(first.window_id)
            .expect("expected first window shutdown");
        let cleared_entitlement_timer = shutdown.cleared_entitlement_timer.clone();

        assert_eq!(
            cleared_entitlement_timer
                .as_ref()
                .map(|slot| slot.effect_id),
            Some(1)
        );
        assert!(shutdown.was_entitlement_timer_owner);
        assert_eq!(
            shutdown.retirement,
            StudioWindowHostRetirement::Transferred {
                new_owner_window_id: second.window_id,
                restored_entitlement_timer: cleared_entitlement_timer,
            }
        );
        assert_eq!(host_port.entitlement_timer_owner(), Some(second.window_id));
        assert!(
            host_port
                .window_state(second.window_id)
                .expect("expected replacement owner state")
                .entitlement_timer()
                .is_some()
        );
    }

    #[test]
    fn runtime_host_port_parks_timer_slot_and_restores_it_when_new_owner_opens() {
        let mut host_port =
            StudioRuntimeHostPort::new(&lease_expiring_config()).expect("expected host port");
        let first = host_port.open_window();

        let _ = host_port
            .dispatch_trigger(
                first.window_id,
                &StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected timer elapsed output");

        let shutdown = host_port
            .close_window(first.window_id)
            .expect("expected first window shutdown");

        assert_eq!(
            shutdown.retirement,
            StudioWindowHostRetirement::Parked {
                parked_entitlement_timer: shutdown.cleared_entitlement_timer.clone(),
            }
        );
        assert!(host_port.entitlement_timer_owner().is_none());
        assert_eq!(
            host_port
                .parked_entitlement_timer()
                .map(|slot| slot.effect_id),
            Some(1)
        );

        let reopened = host_port.open_window();

        assert_eq!(reopened.role, StudioWindowHostRole::EntitlementTimerOwner);
        assert_eq!(
            reopened
                .restored_entitlement_timer
                .as_ref()
                .map(|slot| slot.effect_id),
            Some(1)
        );
        assert_eq!(
            host_port.entitlement_timer_owner(),
            Some(reopened.window_id)
        );
        assert!(host_port.parked_entitlement_timer().is_none());
        assert_eq!(
            host_port
                .window_state(reopened.window_id)
                .expect("expected reopened window state")
                .entitlement_timer()
                .map(|slot| slot.effect_id),
            Some(1)
        );
    }

    fn timer_event(output: &StudioRuntimeHostPortOutput) -> &StudioWindowHostEvent {
        output
            .window_event
            .as_ref()
            .expect("expected window host event")
    }
}
