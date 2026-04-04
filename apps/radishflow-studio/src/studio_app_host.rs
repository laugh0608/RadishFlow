use rf_types::RfResult;

use crate::{
    StudioAppWindowHostClose, StudioAppWindowHostCommand, StudioAppWindowHostCommandOutcome,
    StudioAppWindowHostDispatch, StudioAppWindowHostGlobalEvent, StudioAppWindowHostManager,
    StudioRuntimeConfig, StudioRuntimeTimerHandleSlot, StudioRuntimeTrigger, StudioWindowHostId,
    StudioWindowHostRegistration, StudioWindowHostRole,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppHostCommand {
    OpenWindow,
    DispatchWindowTrigger {
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    },
    FocusWindow {
        window_id: StudioWindowHostId,
    },
    DispatchGlobalEvent {
        event: StudioAppWindowHostGlobalEvent,
    },
    CloseWindow {
        window_id: StudioWindowHostId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppHostCommandOutcome {
    WindowOpened(StudioWindowHostRegistration),
    WindowDispatched(StudioAppWindowHostDispatch),
    WindowClosed(StudioAppWindowHostClose),
    IgnoredGlobalEvent {
        event: StudioAppWindowHostGlobalEvent,
    },
    IgnoredClose {
        window_id: StudioWindowHostId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostWindowSnapshot {
    pub window_id: StudioWindowHostId,
    pub role: StudioWindowHostRole,
    pub is_foreground: bool,
    pub entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostSnapshot {
    pub registered_windows: Vec<StudioWindowHostId>,
    pub windows: Vec<StudioAppHostWindowSnapshot>,
    pub foreground_window_id: Option<StudioWindowHostId>,
    pub entitlement_timer_owner_window_id: Option<StudioWindowHostId>,
    pub parked_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostOutput {
    pub outcome: StudioAppHostCommandOutcome,
    pub snapshot: StudioAppHostSnapshot,
}

pub struct StudioAppHost {
    window_host_manager: StudioAppWindowHostManager,
}

impl StudioAppHost {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            window_host_manager: StudioAppWindowHostManager::new(config)?,
        })
    }

    pub fn window_host_manager(&self) -> &StudioAppWindowHostManager {
        &self.window_host_manager
    }

    pub fn snapshot(&self) -> StudioAppHostSnapshot {
        let registered_windows = self.window_host_manager.registered_windows();
        let foreground_window_id = self.window_host_manager.foreground_window_id();
        let entitlement_timer_owner_window_id = self
            .window_host_manager
            .session()
            .host_port()
            .entitlement_timer_owner();
        let windows = registered_windows
            .iter()
            .copied()
            .map(|window_id| {
                let role = if entitlement_timer_owner_window_id == Some(window_id) {
                    StudioWindowHostRole::EntitlementTimerOwner
                } else {
                    StudioWindowHostRole::Observer
                };
                let entitlement_timer = self
                    .window_host_manager
                    .session()
                    .host_port()
                    .window_state(window_id)
                    .and_then(|state| state.entitlement_timer().cloned());

                StudioAppHostWindowSnapshot {
                    window_id,
                    role,
                    is_foreground: foreground_window_id == Some(window_id),
                    entitlement_timer,
                }
            })
            .collect();

        StudioAppHostSnapshot {
            registered_windows,
            windows,
            foreground_window_id,
            entitlement_timer_owner_window_id,
            parked_entitlement_timer: self
                .window_host_manager
                .session()
                .host_port()
                .parked_entitlement_timer()
                .cloned(),
        }
    }

    pub fn execute_command(
        &mut self,
        command: StudioAppHostCommand,
    ) -> RfResult<StudioAppHostOutput> {
        let outcome = self
            .window_host_manager
            .execute_command(map_command(command))
            .map(map_outcome)?;

        Ok(StudioAppHostOutput {
            outcome,
            snapshot: self.snapshot(),
        })
    }
}

fn map_command(command: StudioAppHostCommand) -> StudioAppWindowHostCommand {
    match command {
        StudioAppHostCommand::OpenWindow => StudioAppWindowHostCommand::OpenWindow,
        StudioAppHostCommand::DispatchWindowTrigger { window_id, trigger } => {
            StudioAppWindowHostCommand::DispatchTrigger { window_id, trigger }
        }
        StudioAppHostCommand::FocusWindow { window_id } => {
            StudioAppWindowHostCommand::FocusWindow { window_id }
        }
        StudioAppHostCommand::DispatchGlobalEvent { event } => {
            StudioAppWindowHostCommand::DispatchGlobalEvent { event }
        }
        StudioAppHostCommand::CloseWindow { window_id } => {
            StudioAppWindowHostCommand::CloseWindow { window_id }
        }
    }
}

fn map_outcome(outcome: StudioAppWindowHostCommandOutcome) -> StudioAppHostCommandOutcome {
    match outcome {
        StudioAppWindowHostCommandOutcome::WindowOpened(registration) => {
            StudioAppHostCommandOutcome::WindowOpened(registration)
        }
        StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch)
        }
        StudioAppWindowHostCommandOutcome::WindowClosed(close) => {
            StudioAppHostCommandOutcome::WindowClosed(close)
        }
        StudioAppWindowHostCommandOutcome::IgnoredGlobalEvent { event } => {
            StudioAppHostCommandOutcome::IgnoredGlobalEvent { event }
        }
        StudioAppWindowHostCommandOutcome::IgnoredClose { window_id } => {
            StudioAppHostCommandOutcome::IgnoredClose { window_id }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        StudioAppHost, StudioAppHostCommand, StudioAppHostCommandOutcome,
        StudioAppWindowHostGlobalEvent, StudioRuntimeEntitlementPreflight,
        StudioRuntimeEntitlementSeed, StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger,
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
    fn app_host_returns_snapshot_with_window_open_and_focus_updates() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");

        let first = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open");
        let first_window = match &first.outcome {
            StudioAppHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        assert_eq!(
            first_window.role,
            StudioWindowHostRole::EntitlementTimerOwner
        );
        assert_eq!(
            first.snapshot.registered_windows,
            vec![first_window.window_id]
        );
        assert_eq!(
            first.snapshot.foreground_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(
            first.snapshot.entitlement_timer_owner_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(
            first.snapshot.windows,
            vec![crate::StudioAppHostWindowSnapshot {
                window_id: first_window.window_id,
                role: StudioWindowHostRole::EntitlementTimerOwner,
                is_foreground: true,
                entitlement_timer: None,
            }]
        );

        let second = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected second window open");
        let second_window = match &second.outcome {
            StudioAppHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        assert_eq!(second_window.role, StudioWindowHostRole::Observer);
        assert_eq!(
            second.snapshot.registered_windows,
            vec![first_window.window_id, second_window.window_id]
        );
        assert_eq!(
            second.snapshot.foreground_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(second.snapshot.windows.len(), 2);
        assert_eq!(
            second.snapshot.windows[0].role,
            StudioWindowHostRole::EntitlementTimerOwner
        );
        assert!(second.snapshot.windows[0].is_foreground);
        assert_eq!(
            second.snapshot.windows[1].role,
            StudioWindowHostRole::Observer
        );
        assert!(!second.snapshot.windows[1].is_foreground);

        let focused = app_host
            .execute_command(StudioAppHostCommand::FocusWindow {
                window_id: second_window.window_id,
            })
            .expect("expected focus command");
        assert_eq!(
            focused.snapshot.foreground_window_id,
            Some(second_window.window_id)
        );
        assert_eq!(
            focused.snapshot.entitlement_timer_owner_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(
            focused.snapshot.windows[1],
            crate::StudioAppHostWindowSnapshot {
                window_id: second_window.window_id,
                role: StudioWindowHostRole::Observer,
                is_foreground: true,
                entitlement_timer: None,
            }
        );
    }

    #[test]
    fn app_host_snapshot_tracks_parked_timer_across_last_close_and_reopen() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
        let first = match app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open")
            .outcome
        {
            StudioAppHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let triggered = app_host
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
                window_id: first.window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer trigger");
        assert_eq!(
            triggered.snapshot.entitlement_timer_owner_window_id,
            Some(first.window_id)
        );
        assert_eq!(triggered.snapshot.windows.len(), 1);
        assert_eq!(
            triggered.snapshot.windows[0]
                .entitlement_timer
                .as_ref()
                .map(|slot| slot.effect_id),
            Some(1)
        );

        let closed = app_host
            .execute_command(StudioAppHostCommand::CloseWindow {
                window_id: first.window_id,
            })
            .expect("expected close command");
        match &closed.outcome {
            StudioAppHostCommandOutcome::WindowClosed(close) => {
                assert_eq!(
                    close.shutdown.host_shutdown.retirement,
                    StudioWindowHostRetirement::Parked {
                        parked_entitlement_timer: close
                            .shutdown
                            .host_shutdown
                            .cleared_entitlement_timer
                            .clone(),
                    }
                );
            }
            other => panic!("expected window closed outcome, got {other:?}"),
        }
        assert!(closed.snapshot.registered_windows.is_empty());
        assert!(closed.snapshot.windows.is_empty());
        assert!(closed.snapshot.foreground_window_id.is_none());
        assert!(closed.snapshot.entitlement_timer_owner_window_id.is_none());
        assert_eq!(
            closed
                .snapshot
                .parked_entitlement_timer
                .as_ref()
                .map(|slot| slot.effect_id),
            Some(1)
        );

        let reopened = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected reopen");
        assert_eq!(reopened.snapshot.parked_entitlement_timer, None);
        assert_eq!(reopened.snapshot.registered_windows.len(), 1);
        assert_eq!(
            reopened.snapshot.windows[0]
                .entitlement_timer
                .as_ref()
                .map(|slot| slot.effect_id),
            Some(1)
        );
        assert!(matches!(
            reopened.outcome,
            StudioAppHostCommandOutcome::WindowOpened(_)
        ));
    }

    #[test]
    fn app_host_surfaces_ignored_global_events_with_stable_snapshot() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");

        let output = app_host
            .execute_command(StudioAppHostCommand::DispatchGlobalEvent {
                event: StudioAppWindowHostGlobalEvent::NetworkRestored,
            })
            .expect("expected ignored global event");

        assert_eq!(
            output.outcome,
            StudioAppHostCommandOutcome::IgnoredGlobalEvent {
                event: StudioAppWindowHostGlobalEvent::NetworkRestored,
            }
        );
        assert!(output.snapshot.registered_windows.is_empty());
        assert!(output.snapshot.windows.is_empty());
        assert!(output.snapshot.foreground_window_id.is_none());
    }
}
