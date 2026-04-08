use std::time::SystemTime;

use rf_types::RfResult;

use crate::{
    StudioAppHostState, StudioAppHostUiCommandModel, StudioGuiCanvasState,
    StudioGuiCommandRegistry, StudioGuiDriver, StudioGuiDriverDispatch, StudioGuiDriverOutcome,
    StudioGuiEvent, StudioGuiSnapshot, StudioGuiWindowModel, StudioWindowHostId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerRequest {
    Arm {
        due_at: SystemTime,
    },
    Rearm {
        previous_due_at: SystemTime,
        due_at: SystemTime,
    },
    Clear {
        previous_due_at: SystemTime,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformDispatch {
    pub event: StudioGuiEvent,
    pub outcome: StudioGuiDriverOutcome,
    pub snapshot: StudioGuiSnapshot,
    pub window: StudioGuiWindowModel,
    pub state: StudioAppHostState,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub command_registry: StudioGuiCommandRegistry,
    pub canvas: StudioGuiCanvasState,
    pub native_timer_request: Option<StudioGuiPlatformTimerRequest>,
}

pub struct StudioGuiPlatformHost {
    driver: StudioGuiDriver,
    current_due_at: Option<SystemTime>,
}

impl StudioGuiPlatformHost {
    pub fn new(config: &crate::StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            driver: StudioGuiDriver::new(config)?,
            current_due_at: None,
        })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.driver.state()
    }

    pub fn snapshot(&self) -> StudioGuiSnapshot {
        self.driver.snapshot()
    }

    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.current_due_at
    }

    pub fn dispatch_event(&mut self, event: StudioGuiEvent) -> RfResult<StudioGuiPlatformDispatch> {
        let previous_due_at = self.current_due_at;
        let dispatch = self.driver.dispatch_event(event)?;
        let next_due_at = self.driver.next_due_native_timer_at();
        self.current_due_at = next_due_at;
        Ok(platform_dispatch_from_driver(
            dispatch,
            plan_platform_timer_request(previous_due_at, next_due_at),
        ))
    }

    pub fn dispatch_native_timer_elapsed(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        handle_id: crate::StudioWindowNativeTimerHandleId,
    ) -> RfResult<StudioGuiPlatformDispatch> {
        self.dispatch_event(StudioGuiEvent::NativeTimerElapsed {
            window_id,
            handle_id,
        })
    }

    pub fn dispatch_due_native_timer_events(
        &mut self,
        now: SystemTime,
    ) -> RfResult<Vec<StudioGuiPlatformDispatch>> {
        let due_dispatches = self.driver.drain_due_native_timer_events(now)?;
        let mut platform_dispatches = Vec::with_capacity(due_dispatches.len());
        for dispatch in due_dispatches {
            let previous_due_at = self.current_due_at;
            let next_due_at = self.driver.next_due_native_timer_at();
            self.current_due_at = next_due_at;
            platform_dispatches.push(platform_dispatch_from_driver(
                dispatch,
                plan_platform_timer_request(previous_due_at, next_due_at),
            ));
        }
        Ok(platform_dispatches)
    }
}

fn platform_dispatch_from_driver(
    dispatch: StudioGuiDriverDispatch,
    native_timer_request: Option<StudioGuiPlatformTimerRequest>,
) -> StudioGuiPlatformDispatch {
    StudioGuiPlatformDispatch {
        event: dispatch.event,
        outcome: dispatch.outcome,
        snapshot: dispatch.snapshot,
        window: dispatch.window,
        state: dispatch.state,
        ui_commands: dispatch.ui_commands,
        command_registry: dispatch.command_registry,
        canvas: dispatch.canvas,
        native_timer_request,
    }
}

fn plan_platform_timer_request(
    previous_due_at: Option<SystemTime>,
    next_due_at: Option<SystemTime>,
) -> Option<StudioGuiPlatformTimerRequest> {
    match (previous_due_at, next_due_at) {
        (None, Some(due_at)) => Some(StudioGuiPlatformTimerRequest::Arm { due_at }),
        (Some(previous_due_at), Some(due_at)) if previous_due_at != due_at => {
            Some(StudioGuiPlatformTimerRequest::Rearm {
                previous_due_at,
                due_at,
            })
        }
        (Some(previous_due_at), None) => {
            Some(StudioGuiPlatformTimerRequest::Clear { previous_due_at })
        }
        (Some(_), Some(_)) | (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        StudioGuiEvent, StudioGuiPlatformHost, StudioGuiPlatformTimerRequest, StudioRuntimeConfig,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
    };

    fn lease_expiring_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        }
    }

    #[test]
    fn platform_host_reports_arm_request_when_native_timer_first_appears() {
        let mut host =
            StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
        let opened = host
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            crate::StudioGuiDriverOutcome::HostCommand(
                crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
            ) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };

        let dispatched = host
            .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                    crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer trigger dispatch");

        assert!(matches!(
            dispatched.native_timer_request,
            Some(StudioGuiPlatformTimerRequest::Arm { .. })
        ));
        assert!(host.next_native_timer_due_at().is_some());
    }

    #[test]
    fn platform_host_reports_rearm_after_due_timer_dispatch() {
        let mut host =
            StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
        let opened = host
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            crate::StudioGuiDriverOutcome::HostCommand(
                crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
            ) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let first = host
            .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                    crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected first timer trigger");
        assert!(matches!(
            first.native_timer_request,
            Some(StudioGuiPlatformTimerRequest::Arm { .. })
        ));
        let due_at = host
            .next_native_timer_due_at()
            .expect("expected scheduled timer due at");

        let due_dispatches = host
            .dispatch_due_native_timer_events(due_at)
            .expect("expected due timer dispatches");

        assert!(!due_dispatches.is_empty());
        assert!(due_dispatches.iter().all(|dispatch| matches!(
            dispatch.native_timer_request,
            Some(StudioGuiPlatformTimerRequest::Rearm { .. }) | None
        )));
    }
}
