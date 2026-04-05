use std::collections::BTreeSet;

use rf_types::{RfError, RfResult};

use crate::{
    StudioRuntimeConfig, StudioRuntimeTrigger, StudioWindowHostId, StudioWindowHostLifecycleEvent,
    StudioWindowHostRegistration, StudioWindowHostRetirement, StudioWindowSession,
    StudioWindowSessionDispatch, StudioWindowSessionShutdown,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppWindowHostGlobalEvent {
    LoginCompleted,
    NetworkRestored,
    TimerElapsed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppWindowHostCommand {
    OpenWindow,
    DispatchTrigger {
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    },
    DispatchRunPanelRecoveryAction {
        window_id: StudioWindowHostId,
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
pub struct StudioAppWindowHostDispatch {
    pub target_window_id: StudioWindowHostId,
    pub dispatch: StudioWindowSessionDispatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppWindowHostClose {
    pub window_id: StudioWindowHostId,
    pub shutdown: StudioWindowSessionShutdown,
    pub next_foreground_window_id: Option<StudioWindowHostId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppWindowHostCommandOutcome {
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

pub struct StudioAppWindowHostManager {
    session: StudioWindowSession,
    registered_windows: BTreeSet<StudioWindowHostId>,
    foreground_window_id: Option<StudioWindowHostId>,
}

impl StudioAppWindowHostManager {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            session: StudioWindowSession::new(config)?,
            registered_windows: BTreeSet::new(),
            foreground_window_id: None,
        })
    }

    pub fn session(&self) -> &StudioWindowSession {
        &self.session
    }

    pub fn foreground_window_id(&self) -> Option<StudioWindowHostId> {
        self.foreground_window_id
    }

    pub fn registered_windows(&self) -> Vec<StudioWindowHostId> {
        self.registered_windows.iter().copied().collect()
    }

    pub fn execute_command(
        &mut self,
        command: StudioAppWindowHostCommand,
    ) -> RfResult<StudioAppWindowHostCommandOutcome> {
        match command {
            StudioAppWindowHostCommand::OpenWindow => Ok(
                StudioAppWindowHostCommandOutcome::WindowOpened(self.open_window()),
            ),
            StudioAppWindowHostCommand::DispatchTrigger { window_id, trigger } => {
                let dispatch = self.dispatch_trigger(window_id, &trigger)?;
                Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(
                    dispatch,
                ))
            }
            StudioAppWindowHostCommand::DispatchRunPanelRecoveryAction { window_id } => {
                let dispatch = self.dispatch_run_panel_recovery_action(window_id)?;
                Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(
                    dispatch,
                ))
            }
            StudioAppWindowHostCommand::FocusWindow { window_id } => {
                let dispatch = self.focus_window(window_id)?;
                Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(
                    dispatch,
                ))
            }
            StudioAppWindowHostCommand::DispatchGlobalEvent { event } => {
                match self.dispatch_global_event(event)? {
                    Some(dispatch) => Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(
                        dispatch,
                    )),
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredGlobalEvent { event }),
                }
            }
            StudioAppWindowHostCommand::CloseWindow { window_id } => {
                match self.close_window(window_id) {
                    Some(close) => Ok(StudioAppWindowHostCommandOutcome::WindowClosed(close)),
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredClose { window_id }),
                }
            }
        }
    }

    pub fn open_window(&mut self) -> StudioWindowHostRegistration {
        let registration = self.session.open_window();
        self.registered_windows.insert(registration.window_id);
        if self.foreground_window_id.is_none() {
            self.foreground_window_id = Some(registration.window_id);
        }
        registration
    }

    pub fn dispatch_trigger(
        &mut self,
        window_id: StudioWindowHostId,
        trigger: &StudioRuntimeTrigger,
    ) -> RfResult<StudioAppWindowHostDispatch> {
        self.ensure_registered_window(window_id)?;
        let dispatch = self.session.dispatch_trigger(window_id, trigger)?;

        Ok(StudioAppWindowHostDispatch {
            target_window_id: window_id,
            dispatch,
        })
    }

    pub fn dispatch_run_panel_recovery_action(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioAppWindowHostDispatch> {
        self.dispatch_trigger(window_id, &StudioRuntimeTrigger::WidgetRecoveryAction)
    }

    pub fn focus_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioAppWindowHostDispatch> {
        self.ensure_registered_window(window_id)?;
        self.foreground_window_id = Some(window_id);
        let dispatch = self.session.dispatch_lifecycle_event(
            window_id,
            StudioWindowHostLifecycleEvent::WindowForegrounded,
        )?;

        Ok(StudioAppWindowHostDispatch {
            target_window_id: window_id,
            dispatch,
        })
    }

    pub fn dispatch_global_event(
        &mut self,
        event: StudioAppWindowHostGlobalEvent,
    ) -> RfResult<Option<StudioAppWindowHostDispatch>> {
        let Some(target_window_id) = self.resolve_global_event_target(event) else {
            return Ok(None);
        };

        let lifecycle_event = match event {
            StudioAppWindowHostGlobalEvent::LoginCompleted => {
                StudioWindowHostLifecycleEvent::LoginCompleted
            }
            StudioAppWindowHostGlobalEvent::NetworkRestored => {
                StudioWindowHostLifecycleEvent::NetworkRestored
            }
            StudioAppWindowHostGlobalEvent::TimerElapsed => {
                StudioWindowHostLifecycleEvent::TimerElapsed
            }
        };
        let dispatch = self
            .session
            .dispatch_lifecycle_event(target_window_id, lifecycle_event)?;

        Ok(Some(StudioAppWindowHostDispatch {
            target_window_id,
            dispatch,
        }))
    }

    pub fn close_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> Option<StudioAppWindowHostClose> {
        self.registered_windows.remove(&window_id);
        let shutdown = self.session.close_window(window_id)?;

        if self.foreground_window_id == Some(window_id) {
            self.foreground_window_id = match shutdown.host_shutdown.retirement {
                StudioWindowHostRetirement::Transferred {
                    new_owner_window_id,
                    ..
                } => Some(new_owner_window_id),
                StudioWindowHostRetirement::None | StudioWindowHostRetirement::Parked { .. } => {
                    self.registered_windows.iter().next().copied()
                }
            };
        }

        Some(StudioAppWindowHostClose {
            window_id,
            shutdown,
            next_foreground_window_id: self.foreground_window_id,
        })
    }

    fn resolve_global_event_target(
        &self,
        event: StudioAppWindowHostGlobalEvent,
    ) -> Option<StudioWindowHostId> {
        if self.registered_windows.is_empty() {
            return None;
        }

        match event {
            StudioAppWindowHostGlobalEvent::TimerElapsed => self
                .session
                .host_port()
                .entitlement_timer_owner()
                .or(self.foreground_window_id)
                .or_else(|| self.registered_windows.iter().next().copied()),
            StudioAppWindowHostGlobalEvent::LoginCompleted
            | StudioAppWindowHostGlobalEvent::NetworkRestored => self
                .foreground_window_id
                .or_else(|| self.registered_windows.iter().next().copied()),
        }
    }

    fn ensure_registered_window(&self, window_id: StudioWindowHostId) -> RfResult<()> {
        if self.registered_windows.contains(&window_id) {
            return Ok(());
        }

        Err(RfError::invalid_input(format!(
            "window host `{window_id}` is not registered with app host manager"
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        StudioAppWindowHostCommand, StudioAppWindowHostCommandOutcome,
        StudioAppWindowHostGlobalEvent, StudioAppWindowHostManager,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger, StudioWindowHostRole,
        StudioWindowTimerDriverTransition,
    };
    use rf_ui::RunPanelActionId;

    fn lease_expiring_config() -> crate::StudioRuntimeConfig {
        crate::StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..crate::StudioRuntimeConfig::default()
        }
    }

    fn solver_failure_config() -> (crate::StudioRuntimeConfig, PathBuf) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-window-host-recovery-{unique}.rfproj.json"
        ));
        let project_json = include_str!("../../../examples/flowsheets/feed-valve-flash.rfproj.json")
            .replacen(
                "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 90000.0,",
                "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 130000.0,",
                1,
            );
        fs::write(&project_path, project_json).expect("expected temporary failure project");

        (
            crate::StudioRuntimeConfig {
                project_path: project_path.clone(),
                entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
                entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
                trigger: crate::StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            },
            project_path,
        )
    }

    #[test]
    fn app_window_host_manager_tracks_foreground_window_across_open_and_close() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();

        assert_eq!(first.role, StudioWindowHostRole::EntitlementTimerOwner);
        assert_eq!(manager.foreground_window_id(), Some(first.window_id));
        assert_eq!(
            manager.registered_windows(),
            vec![first.window_id, second.window_id]
        );

        let close = manager
            .close_window(first.window_id)
            .expect("expected first window close");

        assert_eq!(close.next_foreground_window_id, Some(second.window_id));
        assert_eq!(manager.foreground_window_id(), Some(second.window_id));
    }

    #[test]
    fn app_window_host_manager_focuses_window_through_single_entry() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();

        let dispatch = manager
            .focus_window(second.window_id)
            .expect("expected focus dispatch");

        assert_eq!(dispatch.target_window_id, second.window_id);
        assert_eq!(manager.foreground_window_id(), Some(second.window_id));
        assert_eq!(
            dispatch.dispatch.host_output.runtime_output.trigger,
            StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::WindowForegrounded
            )
        );
        assert_eq!(
            manager.session().host_port().entitlement_timer_owner(),
            Some(first.window_id)
        );
    }

    #[test]
    fn app_window_host_manager_routes_global_timer_elapsed_to_current_owner() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();
        let _ = manager
            .dispatch_trigger(
                first.window_id,
                &StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected first timer dispatch");
        let _ = manager
            .focus_window(second.window_id)
            .expect("expected second window focus");

        let dispatch = manager
            .dispatch_global_event(StudioAppWindowHostGlobalEvent::TimerElapsed)
            .expect("expected global timer dispatch")
            .expect("expected routed timer dispatch");

        assert_eq!(dispatch.target_window_id, first.window_id);
        assert!(matches!(
            dispatch.dispatch.timer_driver_transitions.as_slice(),
            [StudioWindowTimerDriverTransition::KeepNativeTimer { window_id, .. }]
            if *window_id == first.window_id
        ));
    }

    #[test]
    fn app_window_host_manager_routes_global_network_restored_to_foreground_window() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();
        let _ = manager
            .focus_window(second.window_id)
            .expect("expected second window focus");

        let dispatch = manager
            .dispatch_global_event(StudioAppWindowHostGlobalEvent::NetworkRestored)
            .expect("expected global network dispatch")
            .expect("expected routed network dispatch");

        assert_eq!(dispatch.target_window_id, second.window_id);
        assert_eq!(
            dispatch.dispatch.host_output.runtime_output.trigger,
            StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::NetworkRestored
            )
        );
        assert_eq!(manager.foreground_window_id(), Some(second.window_id));
        assert_eq!(
            manager.registered_windows(),
            vec![first.window_id, second.window_id]
        );
    }

    #[test]
    fn app_window_host_manager_ignores_global_events_without_windows() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");

        let dispatch = manager
            .dispatch_global_event(StudioAppWindowHostGlobalEvent::NetworkRestored)
            .expect("expected global network dispatch");

        assert!(dispatch.is_none());
    }

    #[test]
    fn app_window_host_manager_executes_commands_through_single_entry() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
        let first = match manager
            .execute_command(StudioAppWindowHostCommand::OpenWindow)
            .expect("expected first window open")
        {
            StudioAppWindowHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let second = match manager
            .execute_command(StudioAppWindowHostCommand::OpenWindow)
            .expect("expected second window open")
        {
            StudioAppWindowHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        assert_eq!(first.role, StudioWindowHostRole::EntitlementTimerOwner);
        assert_eq!(second.role, StudioWindowHostRole::Observer);

        let focus = manager
            .execute_command(StudioAppWindowHostCommand::FocusWindow {
                window_id: second.window_id,
            })
            .expect("expected focus command");
        match focus {
            StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
                assert_eq!(dispatch.target_window_id, second.window_id);
            }
            other => panic!("expected focus dispatch outcome, got {other:?}"),
        }
        assert_eq!(manager.foreground_window_id(), Some(second.window_id));

        let trigger = manager
            .execute_command(StudioAppWindowHostCommand::DispatchTrigger {
                window_id: first.window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected trigger command");
        match trigger {
            StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
                assert_eq!(dispatch.target_window_id, first.window_id);
            }
            other => panic!("expected trigger dispatch outcome, got {other:?}"),
        }

        let global = manager
            .execute_command(StudioAppWindowHostCommand::DispatchGlobalEvent {
                event: StudioAppWindowHostGlobalEvent::TimerElapsed,
            })
            .expect("expected global event command");
        match global {
            StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
                assert_eq!(dispatch.target_window_id, first.window_id);
            }
            other => panic!("expected global dispatch outcome, got {other:?}"),
        }

        let close = manager
            .execute_command(StudioAppWindowHostCommand::CloseWindow {
                window_id: first.window_id,
            })
            .expect("expected close command");
        match close {
            StudioAppWindowHostCommandOutcome::WindowClosed(close) => {
                assert_eq!(close.window_id, first.window_id);
                assert_eq!(close.next_foreground_window_id, Some(second.window_id));
            }
            other => panic!("expected close outcome, got {other:?}"),
        }
    }

    #[test]
    fn app_window_host_manager_dispatches_run_panel_recovery_through_typed_entry() {
        let (config, project_path) = solver_failure_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        let window = manager.open_window();

        let run = manager
            .dispatch_trigger(
                window.window_id,
                &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");
        match &run.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                crate::StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                    assert!(matches!(
                        dispatch.outcome,
                        crate::StudioWorkspaceRunOutcome::Failed(_)
                    ));
                }
                other => panic!("expected workspace run dispatch, got {other:?}"),
            },
            other => panic!("expected app command dispatch, got {other:?}"),
        }

        let recovery = manager
            .dispatch_run_panel_recovery_action(window.window_id)
            .expect("expected run panel recovery dispatch");

        assert_eq!(recovery.target_window_id, window.window_id);
        match &recovery.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
                assert_eq!(outcome.action.title, "Inspect unit inputs");
                assert_eq!(
                    outcome.applied_target,
                    Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                        "valve-1"
                    )))
                );
            }
            other => panic!("expected run panel recovery dispatch, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_window_host_manager_command_entry_surfaces_ignored_cases() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");

        let ignored_global = manager
            .execute_command(StudioAppWindowHostCommand::DispatchGlobalEvent {
                event: StudioAppWindowHostGlobalEvent::NetworkRestored,
            })
            .expect("expected ignored global event");
        assert_eq!(
            ignored_global,
            StudioAppWindowHostCommandOutcome::IgnoredGlobalEvent {
                event: StudioAppWindowHostGlobalEvent::NetworkRestored,
            }
        );

        let window = match manager
            .execute_command(StudioAppWindowHostCommand::OpenWindow)
            .expect("expected window open")
        {
            StudioAppWindowHostCommandOutcome::WindowOpened(registration) => registration,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let _ = manager
            .execute_command(StudioAppWindowHostCommand::CloseWindow {
                window_id: window.window_id,
            })
            .expect("expected first close");
        let ignored_close = manager
            .execute_command(StudioAppWindowHostCommand::CloseWindow {
                window_id: window.window_id,
            })
            .expect("expected ignored close");
        assert_eq!(
            ignored_close,
            StudioAppWindowHostCommandOutcome::IgnoredClose {
                window_id: window.window_id,
            }
        );
    }
}
