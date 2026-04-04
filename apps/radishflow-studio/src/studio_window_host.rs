use rf_types::RfResult;

use crate::{
    StudioRuntime, StudioRuntimeConfig, StudioRuntimeHostAckResult, StudioRuntimeTimerHandleSlot,
    StudioRuntimeTimerHostCommand, StudioRuntimeTimerHostState, StudioRuntimeTimerHostTransition,
    StudioRuntimeTrigger,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StudioWindowHostState {
    entitlement_timer: StudioRuntimeTimerHostState,
}

impl StudioWindowHostState {
    pub fn entitlement_timer(&self) -> Option<&StudioRuntimeTimerHandleSlot> {
        self.entitlement_timer.entitlement_timer()
    }

    pub fn apply_runtime_output(
        &mut self,
        runtime: &mut StudioRuntime,
        output: &crate::StudioRuntimeOutput,
    ) -> Option<StudioWindowHostEvent> {
        let command = output.entitlement_timer_host_command()?.clone();
        let transition = self.entitlement_timer.apply_command(&command);
        let ack = runtime.acknowledge_entitlement_timer_host_command(&command);

        Some(StudioWindowHostEvent::EntitlementTimerApplied {
            command,
            transition,
            ack,
        })
    }

    pub fn prepare_shutdown(&mut self) -> StudioWindowHostShutdown {
        StudioWindowHostShutdown {
            cleared_entitlement_timer: self.entitlement_timer.clear(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioWindowHostEvent {
    EntitlementTimerApplied {
        command: StudioRuntimeTimerHostCommand,
        transition: StudioRuntimeTimerHostTransition,
        ack: StudioRuntimeHostAckResult,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioWindowHostShutdown {
    pub cleared_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioRuntimeHostPortOutput {
    pub runtime_output: crate::StudioRuntimeOutput,
    pub window_event: Option<StudioWindowHostEvent>,
}

pub struct StudioRuntimeHostPort {
    runtime: StudioRuntime,
    window_state: StudioWindowHostState,
}

impl StudioRuntimeHostPort {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            runtime: StudioRuntime::new(config)?,
            window_state: StudioWindowHostState::default(),
        })
    }

    pub fn runtime(&self) -> &StudioRuntime {
        &self.runtime
    }

    pub fn window_state(&self) -> &StudioWindowHostState {
        &self.window_state
    }

    pub fn dispatch_trigger(
        &mut self,
        trigger: &StudioRuntimeTrigger,
    ) -> RfResult<StudioRuntimeHostPortOutput> {
        let runtime_output = self.runtime.dispatch_trigger_output(trigger)?;
        let window_event = self
            .window_state
            .apply_runtime_output(&mut self.runtime, &runtime_output);

        Ok(StudioRuntimeHostPortOutput {
            runtime_output,
            window_event,
        })
    }

    pub fn prepare_shutdown(&mut self) -> StudioWindowHostShutdown {
        self.window_state.prepare_shutdown()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeHostAckStatus, StudioRuntimeHostPort,
        StudioRuntimeHostPortOutput, StudioRuntimeTimerHostCommand,
        StudioRuntimeTimerHostTransition, StudioRuntimeTrigger, StudioWindowHostEvent,
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

        let output = host_port
            .dispatch_trigger(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ))
            .expect("expected timer elapsed output");

        match timer_event(&output) {
            StudioWindowHostEvent::EntitlementTimerApplied {
                command,
                transition,
                ack,
            } => {
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
                .window_state()
                .entitlement_timer()
                .map(|slot| slot.effect_id),
            Some(1)
        );
        assert!(host_port.runtime().pending_host_effects().is_empty());
    }

    #[test]
    fn runtime_host_port_replaces_window_timer_slot_across_consecutive_events() {
        let mut host_port =
            StudioRuntimeHostPort::new(&lease_expiring_config()).expect("expected host port");

        let _ = host_port
            .dispatch_trigger(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ))
            .expect("expected first output");
        let output = host_port
            .dispatch_trigger(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::NetworkRestored,
            ))
            .expect("expected second output");

        match timer_event(&output) {
            StudioWindowHostEvent::EntitlementTimerApplied {
                command,
                transition,
                ack,
            } => {
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
                .window_state()
                .entitlement_timer()
                .map(|slot| slot.effect_id),
            Some(2)
        );
    }

    #[test]
    fn runtime_host_port_shutdown_clears_window_timer_slot() {
        let mut host_port =
            StudioRuntimeHostPort::new(&lease_expiring_config()).expect("expected host port");

        let _ = host_port
            .dispatch_trigger(&StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ))
            .expect("expected timer elapsed output");

        let shutdown = host_port.prepare_shutdown();

        assert_eq!(
            shutdown
                .cleared_entitlement_timer
                .map(|slot| slot.effect_id),
            Some(1)
        );
        assert!(host_port.window_state().entitlement_timer().is_none());
        assert!(
            host_port
                .prepare_shutdown()
                .cleared_entitlement_timer
                .is_none()
        );
    }

    fn timer_event(output: &StudioRuntimeHostPortOutput) -> &StudioWindowHostEvent {
        output
            .window_event
            .as_ref()
            .expect("expected window host event")
    }
}
