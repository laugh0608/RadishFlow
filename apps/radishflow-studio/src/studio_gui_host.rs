use rf_types::RfResult;
use rf_ui::{AppLogEntry, CanvasSuggestion};

use crate::{
    StudioAppHostCloseEffects, StudioAppHostController, StudioAppHostDispatchEffects,
    StudioAppHostGlobalEventResult, StudioAppHostProjection, StudioAppHostState,
    StudioAppHostUiCommandDispatchResult, StudioAppHostUiCommandModel,
    StudioAppHostWindowDispatchResult, StudioAppWindowHostGlobalEvent, StudioGuiCommandRegistry,
    StudioGuiNativeTimerEffects, StudioRuntimeConfig, StudioRuntimeTrigger, StudioWindowHostId,
    StudioWindowHostRegistration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiHostWindowOpened {
    pub projection: StudioAppHostProjection,
    pub registration: StudioWindowHostRegistration,
    pub ui_commands: StudioAppHostUiCommandModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiHostDispatch {
    pub projection: StudioAppHostProjection,
    pub target_window_id: StudioWindowHostId,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub effects: StudioAppHostDispatchEffects,
    pub native_timers: StudioGuiNativeTimerEffects,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiHostUiCommandDispatchResult {
    Executed(StudioGuiHostDispatch),
    IgnoredDisabled {
        command_id: String,
        detail: String,
        target_window_id: Option<StudioWindowHostId>,
        ui_commands: StudioAppHostUiCommandModel,
    },
    IgnoredMissing {
        command_id: String,
        ui_commands: StudioAppHostUiCommandModel,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiHostGlobalEventDispatch {
    pub projection: StudioAppHostProjection,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub dispatch: Option<StudioGuiHostDispatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiHostLifecycleEvent {
    WindowForegrounded {
        window_id: StudioWindowHostId,
    },
    LoginCompleted,
    NetworkRestored,
    TimerElapsed,
    RunPanelRecoveryRequested,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiHostLifecycleDispatch {
    pub event: StudioGuiHostLifecycleEvent,
    pub projection: StudioAppHostProjection,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub dispatch: Option<StudioGuiHostDispatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiHostCloseWindowResult {
    pub projection: StudioAppHostProjection,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub close: Option<StudioAppHostCloseEffects>,
    pub native_timers: StudioGuiNativeTimerEffects,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiHostCanvasSuggestionResult {
    pub accepted: Option<CanvasSuggestion>,
    pub applied_target: Option<rf_ui::InspectorTarget>,
    pub latest_log_entry: Option<AppLogEntry>,
    pub ui_commands: StudioAppHostUiCommandModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiHostCommand {
    OpenWindow,
    DispatchWindowTrigger {
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    },
    DispatchLifecycleEvent {
        event: StudioGuiHostLifecycleEvent,
    },
    DispatchUiCommand {
        command_id: String,
    },
    CloseWindow {
        window_id: StudioWindowHostId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiHostCommandOutcome {
    WindowOpened(StudioGuiHostWindowOpened),
    WindowDispatched(StudioGuiHostDispatch),
    LifecycleDispatched(StudioGuiHostLifecycleDispatch),
    UiCommandDispatched(StudioGuiHostUiCommandDispatchResult),
    WindowClosed(StudioGuiHostCloseWindowResult),
}

pub struct StudioGuiHost {
    controller: StudioAppHostController,
}

impl StudioGuiHost {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            controller: StudioAppHostController::new(config)?,
        })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.controller.state()
    }

    pub fn ui_commands(&self) -> StudioAppHostUiCommandModel {
        self.state().ui_command_model()
    }

    pub fn command_registry(&self) -> StudioGuiCommandRegistry {
        StudioGuiCommandRegistry::from_model(&self.ui_commands())
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<CanvasSuggestion>) {
        self.controller.replace_canvas_suggestions(suggestions);
    }

    pub fn accept_focused_canvas_suggestion_by_tab(&mut self) -> StudioGuiHostCanvasSuggestionResult {
        StudioGuiHostCanvasSuggestionResult {
            accepted: self.controller.accept_focused_canvas_suggestion_by_tab(),
            applied_target: self.controller.active_inspector_target(),
            latest_log_entry: self.controller.latest_log_entry(),
            ui_commands: self.ui_commands(),
        }
    }

    pub fn execute_command(
        &mut self,
        command: StudioGuiHostCommand,
    ) -> RfResult<StudioGuiHostCommandOutcome> {
        match command {
            StudioGuiHostCommand::OpenWindow => {
                self.open_window().map(StudioGuiHostCommandOutcome::WindowOpened)
            }
            StudioGuiHostCommand::DispatchWindowTrigger { window_id, trigger } => self
                .dispatch_window_trigger(window_id, trigger)
                .map(StudioGuiHostCommandOutcome::WindowDispatched),
            StudioGuiHostCommand::DispatchLifecycleEvent { event } => self
                .dispatch_lifecycle_event(event)
                .map(StudioGuiHostCommandOutcome::LifecycleDispatched),
            StudioGuiHostCommand::DispatchUiCommand { command_id } => self
                .dispatch_ui_command(&command_id)
                .map(StudioGuiHostCommandOutcome::UiCommandDispatched),
            StudioGuiHostCommand::CloseWindow { window_id } => self
                .close_window(window_id)
                .map(StudioGuiHostCommandOutcome::WindowClosed),
        }
    }

    pub fn open_window(&mut self) -> RfResult<StudioGuiHostWindowOpened> {
        let opened = self.controller.open_window()?;
        Ok(StudioGuiHostWindowOpened {
            ui_commands: ui_commands_from_projection(&opened.projection),
            projection: opened.projection,
            registration: opened.registration,
        })
    }

    pub fn dispatch_window_trigger(
        &mut self,
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    ) -> RfResult<StudioGuiHostDispatch> {
        self.controller
            .dispatch_window_trigger(window_id, trigger)
            .map(dispatch_from_controller)
    }

    pub fn focus_window(&mut self, window_id: StudioWindowHostId) -> RfResult<StudioGuiHostDispatch> {
        self.controller
            .focus_window(window_id)
            .map(dispatch_from_controller)
    }

    pub fn dispatch_lifecycle_event(
        &mut self,
        event: StudioGuiHostLifecycleEvent,
    ) -> RfResult<StudioGuiHostLifecycleDispatch> {
        match event {
            StudioGuiHostLifecycleEvent::WindowForegrounded { window_id } => {
                let dispatch = self.focus_window(window_id)?;
                Ok(StudioGuiHostLifecycleDispatch {
                    event,
                    projection: dispatch.projection.clone(),
                    ui_commands: dispatch.ui_commands.clone(),
                    dispatch: Some(dispatch),
                })
            }
            StudioGuiHostLifecycleEvent::LoginCompleted
            | StudioGuiHostLifecycleEvent::NetworkRestored
            | StudioGuiHostLifecycleEvent::TimerElapsed
            | StudioGuiHostLifecycleEvent::RunPanelRecoveryRequested => {
                let result = self.dispatch_global_event(global_event_from_lifecycle(event))?;
                Ok(StudioGuiHostLifecycleDispatch {
                    event,
                    projection: result.projection,
                    ui_commands: result.ui_commands,
                    dispatch: result.dispatch,
                })
            }
        }
    }

    pub fn dispatch_ui_command(
        &mut self,
        command_id: &str,
    ) -> RfResult<StudioGuiHostUiCommandDispatchResult> {
        let ui_commands = self.ui_commands();
        match self.controller.dispatch_ui_command(command_id)? {
            StudioAppHostUiCommandDispatchResult::Executed(dispatch) => {
                Ok(StudioGuiHostUiCommandDispatchResult::Executed(
                    dispatch_from_controller(dispatch),
                ))
            }
            StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
                command_id,
                detail,
                target_window_id,
            } => Ok(StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                command_id,
                detail,
                target_window_id,
                ui_commands,
            }),
            StudioAppHostUiCommandDispatchResult::IgnoredMissing { command_id } => {
                Ok(StudioGuiHostUiCommandDispatchResult::IgnoredMissing {
                    command_id,
                    ui_commands,
                })
            }
        }
    }

    pub fn dispatch_global_event(
        &mut self,
        event: StudioAppWindowHostGlobalEvent,
    ) -> RfResult<StudioGuiHostGlobalEventDispatch> {
        let result = self.controller.dispatch_global_event(event)?;
        Ok(global_event_from_controller(result))
    }

    pub fn close_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioGuiHostCloseWindowResult> {
        let closed = self.controller.close_window(window_id)?;
        Ok(StudioGuiHostCloseWindowResult {
            ui_commands: ui_commands_from_projection(&closed.projection),
            projection: closed.projection,
            native_timers: closed
                .close
                .as_ref()
                .map(|close| {
                    StudioGuiNativeTimerEffects::from_driver(
                        &close.native_timer_transitions,
                        &close.native_timer_acks,
                    )
                })
                .unwrap_or_default(),
            close: closed.close,
        })
    }
}

fn dispatch_from_controller(dispatch: StudioAppHostWindowDispatchResult) -> StudioGuiHostDispatch {
    let native_timers = StudioGuiNativeTimerEffects::from_driver(
        &dispatch.effects.native_timer_transitions,
        &dispatch.effects.native_timer_acks,
    );
    StudioGuiHostDispatch {
        ui_commands: ui_commands_from_projection(&dispatch.projection),
        projection: dispatch.projection,
        target_window_id: dispatch.target_window_id,
        effects: dispatch.effects,
        native_timers,
    }
}

fn global_event_from_controller(
    result: StudioAppHostGlobalEventResult,
) -> StudioGuiHostGlobalEventDispatch {
    StudioGuiHostGlobalEventDispatch {
        ui_commands: ui_commands_from_projection(&result.projection),
        projection: result.projection.clone(),
        dispatch: result.dispatch.map(dispatch_from_controller),
    }
}

fn ui_commands_from_projection(projection: &StudioAppHostProjection) -> StudioAppHostUiCommandModel {
    projection.state.ui_command_model()
}

fn global_event_from_lifecycle(event: StudioGuiHostLifecycleEvent) -> StudioAppWindowHostGlobalEvent {
    match event {
        StudioGuiHostLifecycleEvent::WindowForegrounded { .. } => {
            unreachable!("window foregrounding is routed through focus_window before global mapping")
        }
        StudioGuiHostLifecycleEvent::LoginCompleted => StudioAppWindowHostGlobalEvent::LoginCompleted,
        StudioGuiHostLifecycleEvent::NetworkRestored => {
            StudioAppWindowHostGlobalEvent::NetworkRestored
        }
        StudioGuiHostLifecycleEvent::TimerElapsed => StudioAppWindowHostGlobalEvent::TimerElapsed,
        StudioGuiHostLifecycleEvent::RunPanelRecoveryRequested => {
            StudioAppWindowHostGlobalEvent::RunPanelRecoveryRequested
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use rf_ui::RunPanelActionId;

    use crate::{
        StudioGuiHost, StudioGuiHostCommand, StudioGuiHostCommandOutcome,
        StudioGuiHostLifecycleEvent, StudioGuiHostUiCommandDispatchResult,
        StudioGuiNativeTimerEffects, StudioRuntimeConfig,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger, StudioWindowHostRetirement,
    };

    fn lease_expiring_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        }
    }

    fn solver_failure_config() -> (StudioRuntimeConfig, PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let project_path = std::env::temp_dir()
            .join(format!("radishflow-studio-gui-host-failure-{timestamp}.rfproj.json"));
        let project =
            include_str!("../../../examples/flowsheets/feed-valve-flash.rfproj.json").replacen(
                "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 90000.0,",
                "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 130000.0,",
                1,
            );
        fs::write(&project_path, project).expect("expected failure project");

        (
            StudioRuntimeConfig {
                project_path: project_path.clone(),
                trigger: StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
                ..lease_expiring_config()
            },
            project_path,
        )
    }

    #[test]
    fn gui_host_surfaces_ui_commands_for_disabled_command_dispatch() {
        let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");

        let dispatch = gui_host
            .dispatch_ui_command("run_panel.run_manual")
            .expect("expected gui host ui command result");

        match dispatch {
            StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                command_id,
                detail,
                target_window_id,
                ui_commands,
            } => {
                assert_eq!(command_id, "run_panel.run_manual");
                assert_eq!(target_window_id, None);
                assert_eq!(detail, "Open a studio window before running the workspace");
                assert_eq!(
                    ui_commands
                        .command("run_panel.run_manual")
                        .expect("expected run command model")
                        .enabled,
                    false
                );
            }
            other => panic!("expected disabled ui command result, got {other:?}"),
        }
    }

    #[test]
    fn gui_host_dispatches_ui_command_and_refreshes_command_registry() {
        let (config, project_path) = solver_failure_config();
        let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
        let opened = gui_host.open_window().expect("expected window open");

        let dispatch = gui_host
            .dispatch_ui_command("run_panel.run_manual")
            .expect("expected gui host ui command dispatch");

        match dispatch {
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch) => {
                assert_eq!(dispatch.target_window_id, opened.registration.window_id);
                assert_eq!(
                    dispatch
                        .ui_commands
                        .command("run_panel.recover_failure")
                        .expect("expected recovery command model")
                        .enabled,
                    true
                );
                assert_eq!(
                    dispatch.native_timers,
                    StudioGuiNativeTimerEffects::from_driver(
                        &dispatch.effects.native_timer_transitions,
                        &dispatch.effects.native_timer_acks,
                    )
                );
                match &dispatch.effects.runtime_report.dispatch {
                    crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                        crate::StudioAppResultDispatch::WorkspaceRun(run) => {
                            assert!(matches!(
                                run.outcome,
                                crate::StudioWorkspaceRunOutcome::Failed(_)
                            ));
                        }
                        other => panic!("expected workspace run dispatch, got {other:?}"),
                    },
                    other => panic!("expected app command dispatch, got {other:?}"),
                }
            }
            other => panic!("expected executed ui command result, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_host_preserves_timer_retirement_summary_on_close() {
        let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
        let opened = gui_host.open_window().expect("expected window open");
        let _ = gui_host
            .dispatch_window_trigger(
                opened.registration.window_id,
                StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected timer trigger");

        let closed = gui_host
            .close_window(opened.registration.window_id)
            .expect("expected close result");
        let close = closed.close.expect("expected close summary");

        assert!(matches!(
            close.retirement,
            StudioWindowHostRetirement::Parked {
                parked_entitlement_timer: Some(_)
            }
        ));
        assert!(matches!(
            closed.native_timers.operations.as_slice(),
            [crate::StudioGuiNativeTimerOperation::Park { from_window_id, .. }]
            if *from_window_id == opened.registration.window_id
        ));
        assert_eq!(closed.projection.state.windows.len(), 0);
        assert_eq!(
            closed
                .ui_commands
                .command("run_panel.run_manual")
                .expect("expected run command")
                .enabled,
            false
        );
    }

    #[test]
    fn gui_host_routes_window_foregrounded_lifecycle_event_to_target_window() {
        let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
        let first = gui_host.open_window().expect("expected first window");
        let second = gui_host.open_window().expect("expected second window");

        let dispatch = gui_host
            .dispatch_lifecycle_event(StudioGuiHostLifecycleEvent::WindowForegrounded {
                window_id: second.registration.window_id,
            })
            .expect("expected lifecycle dispatch");

        let routed = dispatch.dispatch.expect("expected foreground dispatch");
        assert_eq!(routed.target_window_id, second.registration.window_id);
        assert_ne!(routed.target_window_id, first.registration.window_id);
        assert_eq!(
            dispatch.projection.state.foreground_window_id,
            Some(second.registration.window_id)
        );
        assert_eq!(
            dispatch
                .ui_commands
                .command("run_panel.run_manual")
                .and_then(|command| command.target_window_id),
            Some(second.registration.window_id)
        );
        match &routed.effects.runtime_report.dispatch {
            crate::StudioRuntimeDispatch::EntitlementSessionEvent(outcome) => {
                assert_eq!(outcome.event, crate::EntitlementSessionEvent::TimerElapsed);
            }
            other => panic!("expected entitlement session event dispatch, got {other:?}"),
        }
    }

    #[test]
    fn gui_host_routes_timer_elapsed_lifecycle_event_to_timer_owner_window() {
        let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
        let first = gui_host.open_window().expect("expected first window");
        let second = gui_host.open_window().expect("expected second window");
        let _ = gui_host
            .dispatch_window_trigger(
                first.registration.window_id,
                StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected first timer dispatch");
        let _ = gui_host
            .dispatch_lifecycle_event(StudioGuiHostLifecycleEvent::WindowForegrounded {
                window_id: second.registration.window_id,
            })
            .expect("expected foreground update");

        let dispatch = gui_host
            .dispatch_lifecycle_event(StudioGuiHostLifecycleEvent::TimerElapsed)
            .expect("expected timer elapsed lifecycle dispatch");

        let routed = dispatch.dispatch.expect("expected routed timer dispatch");
        assert_eq!(routed.target_window_id, first.registration.window_id);
        assert_ne!(routed.target_window_id, second.registration.window_id);
        assert!(matches!(
            routed.native_timers.operations.as_slice(),
            [crate::StudioGuiNativeTimerOperation::Keep { window_id, .. }]
            if *window_id == first.registration.window_id
        ));
        assert_eq!(
            dispatch.projection.state.foreground_window_id,
            Some(second.registration.window_id)
        );
    }

    #[test]
    fn gui_host_execute_command_routes_open_ui_command_and_close() {
        let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");

        let opened = gui_host
            .execute_command(StudioGuiHostCommand::OpenWindow)
            .expect("expected open command");
        let window_id = match opened {
            StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let command_dispatch = gui_host
            .execute_command(StudioGuiHostCommand::DispatchUiCommand {
                command_id: "run_panel.set_active".to_string(),
            })
            .expect("expected ui command dispatch");
        match command_dispatch {
            StudioGuiHostCommandOutcome::UiCommandDispatched(
                StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
            ) => {
                assert_eq!(dispatch.target_window_id, window_id);
            }
            other => panic!("expected executed ui command outcome, got {other:?}"),
        }

        let closed = gui_host
            .execute_command(StudioGuiHostCommand::CloseWindow { window_id })
            .expect("expected close command");
        match closed {
            StudioGuiHostCommandOutcome::WindowClosed(closed) => {
                assert!(closed.close.is_some());
                assert!(closed.projection.state.windows.is_empty());
            }
            other => panic!("expected window closed outcome, got {other:?}"),
        }
    }
}
