use std::collections::BTreeSet;

use rf_types::{RfError, RfResult};
use rf_ui::{
    CanvasSuggestion, EntitlementActionId, RunPanelActionId, RunPanelWidgetEvent,
    RunPanelWidgetModel,
};

use crate::{
    StudioRuntimeConfig, StudioRuntimeTrigger, StudioWindowHostId, StudioWindowHostLifecycleEvent,
    StudioWindowHostRetirement, StudioWindowSession, StudioWindowSessionDispatch,
    StudioWindowSessionShutdown,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppWindowHostGlobalEvent {
    LoginCompleted,
    NetworkRestored,
    TimerElapsed,
    RunPanelRecoveryRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppWindowHostUiAction {
    RunManualWorkspace,
    ResumeWorkspace,
    HoldWorkspace,
    ActivateWorkspace,
    RecoverRunPanelFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppWindowHostUiActionDisabledReason {
    NoRegisteredWindow,
    RunManualUnavailable,
    ResumeUnavailable,
    HoldUnavailable,
    ActivateUnavailable,
    NoRunPanelRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppWindowHostUiActionAvailability {
    Enabled {
        target_window_id: StudioWindowHostId,
    },
    Disabled {
        reason: StudioAppWindowHostUiActionDisabledReason,
        target_window_id: Option<StudioWindowHostId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppWindowHostUiActionState {
    pub action: StudioAppWindowHostUiAction,
    pub availability: StudioAppWindowHostUiActionAvailability,
}

impl StudioAppWindowHostUiActionState {
    pub fn enabled(&self) -> bool {
        matches!(
            self.availability,
            StudioAppWindowHostUiActionAvailability::Enabled { .. }
        )
    }

    pub fn target_window_id(&self) -> Option<StudioWindowHostId> {
        match self.availability {
            StudioAppWindowHostUiActionAvailability::Enabled { target_window_id } => {
                Some(target_window_id)
            }
            StudioAppWindowHostUiActionAvailability::Disabled {
                target_window_id, ..
            } => target_window_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioCanvasInteractionAction {
    AcceptFocusedByTab,
    RejectFocused,
    FocusNext,
    FocusPrevious,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioAppWindowHostCanvasInteractionResult {
    pub action: StudioCanvasInteractionAction,
    pub accepted: Option<CanvasSuggestion>,
    pub rejected: Option<CanvasSuggestion>,
    pub focused: Option<CanvasSuggestion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppWindowHostCommand {
    OpenWindow,
    DispatchTrigger {
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    },
    DispatchCanvasInteraction {
        action: StudioCanvasInteractionAction,
    },
    DispatchUiAction {
        action: StudioAppWindowHostUiAction,
    },
    DispatchRunPanelRecoveryAction {
        window_id: StudioWindowHostId,
    },
    DispatchForegroundRunPanelRecoveryAction,
    DispatchForegroundEntitlementPrimaryAction,
    DispatchForegroundEntitlementAction {
        action_id: EntitlementActionId,
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
pub struct StudioAppWindowHostOpenWindow {
    pub window_id: StudioWindowHostId,
    pub role: crate::StudioWindowHostRole,
    pub layout_slot: u16,
    pub restored_entitlement_timer: Option<crate::StudioRuntimeTimerHandleSlot>,
    pub timer_driver_commands: Vec<crate::StudioWindowHostTimerDriverCommand>,
    pub timer_driver_transitions: Vec<crate::StudioWindowTimerDriverTransition>,
    pub timer_driver_acks: Vec<crate::StudioWindowTimerDriverAckResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppWindowHostClose {
    pub window_id: StudioWindowHostId,
    pub shutdown: StudioWindowSessionShutdown,
    pub next_foreground_window_id: Option<StudioWindowHostId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StudioAppWindowHostCommandOutcome {
    WindowOpened(StudioAppWindowHostOpenWindow),
    WindowDispatched(StudioAppWindowHostDispatch),
    CanvasInteracted(StudioAppWindowHostCanvasInteractionResult),
    WindowClosed(StudioAppWindowHostClose),
    IgnoredUiAction,
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

    pub fn refresh_local_canvas_suggestions(&mut self) {
        self.session.refresh_local_canvas_suggestions();
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<rf_ui::CanvasSuggestion>) {
        self.session.replace_canvas_suggestions(suggestions);
    }

    pub fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        self.session.accept_focused_canvas_suggestion_by_tab()
    }

    pub fn reject_focused_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.session.reject_focused_canvas_suggestion()
    }

    pub fn focus_next_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.session.focus_next_canvas_suggestion()
    }

    pub fn focus_previous_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.session.focus_previous_canvas_suggestion()
    }

    pub fn dispatch_canvas_interaction(
        &mut self,
        action: StudioCanvasInteractionAction,
    ) -> RfResult<StudioAppWindowHostCanvasInteractionResult> {
        let (accepted, rejected, focused) = match action {
            StudioCanvasInteractionAction::AcceptFocusedByTab => {
                (self.accept_focused_canvas_suggestion_by_tab()?, None, None)
            }
            StudioCanvasInteractionAction::RejectFocused => {
                (None, self.reject_focused_canvas_suggestion(), None)
            }
            StudioCanvasInteractionAction::FocusNext => {
                (None, None, self.focus_next_canvas_suggestion())
            }
            StudioCanvasInteractionAction::FocusPrevious => {
                (None, None, self.focus_previous_canvas_suggestion())
            }
        };

        Ok(StudioAppWindowHostCanvasInteractionResult {
            action,
            accepted,
            rejected,
            focused,
        })
    }

    pub fn foreground_window_id(&self) -> Option<StudioWindowHostId> {
        self.foreground_window_id
    }

    pub fn registered_windows(&self) -> Vec<StudioWindowHostId> {
        self.registered_windows.iter().copied().collect()
    }

    pub fn ui_action_state(
        &self,
        action: StudioAppWindowHostUiAction,
    ) -> StudioAppWindowHostUiActionState {
        let target_window_id = self
            .foreground_window_id
            .or_else(|| self.registered_windows.iter().next().copied());
        let availability = match (action, target_window_id) {
            (StudioAppWindowHostUiAction::RunManualWorkspace, Some(target_window_id))
                if self.run_panel_action_available(RunPanelActionId::RunManual) =>
            {
                StudioAppWindowHostUiActionAvailability::Enabled { target_window_id }
            }
            (StudioAppWindowHostUiAction::RunManualWorkspace, Some(target_window_id)) => {
                StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::RunManualUnavailable,
                    target_window_id: Some(target_window_id),
                }
            }
            (StudioAppWindowHostUiAction::ResumeWorkspace, Some(target_window_id))
                if self.run_panel_action_available(RunPanelActionId::Resume) =>
            {
                StudioAppWindowHostUiActionAvailability::Enabled { target_window_id }
            }
            (StudioAppWindowHostUiAction::ResumeWorkspace, Some(target_window_id)) => {
                StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::ResumeUnavailable,
                    target_window_id: Some(target_window_id),
                }
            }
            (StudioAppWindowHostUiAction::HoldWorkspace, Some(target_window_id))
                if self.run_panel_action_available(RunPanelActionId::SetHold) =>
            {
                StudioAppWindowHostUiActionAvailability::Enabled { target_window_id }
            }
            (StudioAppWindowHostUiAction::HoldWorkspace, Some(target_window_id)) => {
                StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::HoldUnavailable,
                    target_window_id: Some(target_window_id),
                }
            }
            (StudioAppWindowHostUiAction::ActivateWorkspace, Some(target_window_id))
                if self.run_panel_action_available(RunPanelActionId::SetActive) =>
            {
                StudioAppWindowHostUiActionAvailability::Enabled { target_window_id }
            }
            (StudioAppWindowHostUiAction::ActivateWorkspace, Some(target_window_id)) => {
                StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::ActivateUnavailable,
                    target_window_id: Some(target_window_id),
                }
            }
            (StudioAppWindowHostUiAction::RecoverRunPanelFailure, Some(target_window_id))
                if self.run_panel_recovery_available() =>
            {
                StudioAppWindowHostUiActionAvailability::Enabled { target_window_id }
            }
            (StudioAppWindowHostUiAction::RecoverRunPanelFailure, Some(target_window_id)) => {
                StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::NoRunPanelRecovery,
                    target_window_id: Some(target_window_id),
                }
            }
            (_, None) => StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        };

        StudioAppWindowHostUiActionState {
            action,
            availability,
        }
    }

    pub fn ui_action_states(&self) -> Vec<StudioAppWindowHostUiActionState> {
        vec![
            self.ui_action_state(StudioAppWindowHostUiAction::RunManualWorkspace),
            self.ui_action_state(StudioAppWindowHostUiAction::ResumeWorkspace),
            self.ui_action_state(StudioAppWindowHostUiAction::HoldWorkspace),
            self.ui_action_state(StudioAppWindowHostUiAction::ActivateWorkspace),
            self.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure),
        ]
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
            StudioAppWindowHostCommand::DispatchCanvasInteraction { action } => self
                .dispatch_canvas_interaction(action)
                .map(StudioAppWindowHostCommandOutcome::CanvasInteracted),
            StudioAppWindowHostCommand::DispatchUiAction { action } => {
                match self.dispatch_ui_action(action)? {
                    Some(dispatch) => Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(
                        dispatch,
                    )),
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredUiAction),
                }
            }
            StudioAppWindowHostCommand::DispatchRunPanelRecoveryAction { window_id } => {
                let dispatch = self.dispatch_run_panel_recovery_action(window_id)?;
                Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(
                    dispatch,
                ))
            }
            StudioAppWindowHostCommand::DispatchForegroundRunPanelRecoveryAction => {
                match self.dispatch_foreground_run_panel_recovery_action()? {
                    Some(dispatch) => Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(
                        dispatch,
                    )),
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredUiAction),
                }
            }
            StudioAppWindowHostCommand::DispatchForegroundEntitlementPrimaryAction => {
                match self.dispatch_foreground_entitlement_primary_action()? {
                    Some(dispatch) => Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(
                        dispatch,
                    )),
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredUiAction),
                }
            }
            StudioAppWindowHostCommand::DispatchForegroundEntitlementAction { action_id } => {
                match self.dispatch_foreground_entitlement_action(action_id)? {
                    Some(dispatch) => Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(
                        dispatch,
                    )),
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredUiAction),
                }
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

    pub fn open_window(&mut self) -> StudioAppWindowHostOpenWindow {
        let open = self.session.open_window();
        let registration = open.registration;
        self.registered_windows.insert(registration.window_id);
        if self.foreground_window_id.is_none() {
            self.foreground_window_id = Some(registration.window_id);
        }
        StudioAppWindowHostOpenWindow {
            window_id: registration.window_id,
            role: registration.role,
            layout_slot: registration.layout_slot,
            restored_entitlement_timer: registration.restored_entitlement_timer,
            timer_driver_commands: registration.timer_driver_commands,
            timer_driver_transitions: open.timer_driver_transitions,
            timer_driver_acks: open.timer_driver_acks,
        }
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

    pub fn dispatch_ui_action(
        &mut self,
        action: StudioAppWindowHostUiAction,
    ) -> RfResult<Option<StudioAppWindowHostDispatch>> {
        match action {
            StudioAppWindowHostUiAction::RunManualWorkspace => {
                self.dispatch_foreground_run_panel_action(RunPanelActionId::RunManual)
            }
            StudioAppWindowHostUiAction::ResumeWorkspace => {
                self.dispatch_foreground_run_panel_action(RunPanelActionId::Resume)
            }
            StudioAppWindowHostUiAction::HoldWorkspace => {
                self.dispatch_foreground_run_panel_action(RunPanelActionId::SetHold)
            }
            StudioAppWindowHostUiAction::ActivateWorkspace => {
                self.dispatch_foreground_run_panel_action(RunPanelActionId::SetActive)
            }
            StudioAppWindowHostUiAction::RecoverRunPanelFailure => {
                self.dispatch_foreground_run_panel_recovery_action()
            }
        }
    }

    pub fn dispatch_run_panel_recovery_action(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioAppWindowHostDispatch> {
        self.dispatch_trigger(window_id, &StudioRuntimeTrigger::WidgetRecoveryAction)
    }

    pub fn dispatch_run_panel_action(
        &mut self,
        window_id: StudioWindowHostId,
        action_id: RunPanelActionId,
    ) -> RfResult<StudioAppWindowHostDispatch> {
        self.dispatch_trigger(window_id, &StudioRuntimeTrigger::WidgetAction(action_id))
    }

    pub fn dispatch_foreground_run_panel_action(
        &mut self,
        action_id: RunPanelActionId,
    ) -> RfResult<Option<StudioAppWindowHostDispatch>> {
        let Some(window_id) = self
            .foreground_window_id
            .or_else(|| self.registered_windows.iter().next().copied())
        else {
            return Ok(None);
        };

        self.dispatch_run_panel_action(window_id, action_id)
            .map(Some)
    }

    pub fn dispatch_foreground_run_panel_recovery_action(
        &mut self,
    ) -> RfResult<Option<StudioAppWindowHostDispatch>> {
        let Some(window_id) = self
            .foreground_window_id
            .or_else(|| self.registered_windows.iter().next().copied())
        else {
            return Ok(None);
        };

        self.dispatch_run_panel_recovery_action(window_id).map(Some)
    }

    pub fn dispatch_foreground_entitlement_primary_action(
        &mut self,
    ) -> RfResult<Option<StudioAppWindowHostDispatch>> {
        let Some(window_id) = self
            .foreground_window_id
            .or_else(|| self.registered_windows.iter().next().copied())
        else {
            return Ok(None);
        };

        self.dispatch_trigger(
            window_id,
            &StudioRuntimeTrigger::EntitlementWidgetPrimaryAction,
        )
        .map(Some)
    }

    pub fn dispatch_foreground_entitlement_action(
        &mut self,
        action_id: EntitlementActionId,
    ) -> RfResult<Option<StudioAppWindowHostDispatch>> {
        let Some(window_id) = self
            .foreground_window_id
            .or_else(|| self.registered_windows.iter().next().copied())
        else {
            return Ok(None);
        };

        self.dispatch_trigger(
            window_id,
            &StudioRuntimeTrigger::EntitlementWidgetAction(action_id),
        )
        .map(Some)
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
        if matches!(
            event,
            StudioAppWindowHostGlobalEvent::RunPanelRecoveryRequested
        ) {
            return self.dispatch_ui_action(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
        }

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
            StudioAppWindowHostGlobalEvent::RunPanelRecoveryRequested => unreachable!(
                "run panel recovery requests are routed through ui actions before lifecycle dispatch"
            ),
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
            StudioAppWindowHostGlobalEvent::RunPanelRecoveryRequested => None,
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

    fn run_panel_recovery_available(&self) -> bool {
        self.run_panel_widget().recovery_action().is_some()
    }

    fn run_panel_action_available(&self, action_id: RunPanelActionId) -> bool {
        matches!(
            self.run_panel_widget().activate(action_id),
            RunPanelWidgetEvent::Dispatched { .. }
        )
    }

    fn run_panel_widget(&self) -> RunPanelWidgetModel {
        let run_panel = &self
            .session
            .host_port()
            .runtime()
            .app_state()
            .workspace
            .run_panel;
        RunPanelWidgetModel::from_state(run_panel)
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
        StudioAppWindowHostCanvasInteractionResult, StudioAppWindowHostCommand,
        StudioAppWindowHostCommandOutcome, StudioAppWindowHostGlobalEvent,
        StudioAppWindowHostManager, StudioAppWindowHostUiAction,
        StudioAppWindowHostUiActionAvailability, StudioAppWindowHostUiActionDisabledReason,
        StudioAppWindowHostUiActionState, StudioCanvasInteractionAction,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger, StudioWindowHostRole,
        StudioWindowTimerDriverTransition,
    };
    use rf_ui::{EntitlementActionId, RunPanelActionId};

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

    fn synced_workspace_config() -> crate::StudioRuntimeConfig {
        crate::StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            ..crate::StudioRuntimeConfig::default()
        }
    }

    fn flash_drum_local_rules_synced_config() -> (crate::StudioRuntimeConfig, PathBuf) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-window-host-local-rules-{unique}.rfproj.json"
        ));
        let project_json =
            include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json")
                .replacen(
                    ",\n        \"stream-vapor\": {\n          \"id\": \"stream-vapor\",\n          \"name\": \"Vapor Outlet\",\n          \"temperature_k\": 345.0,\n          \"pressure_pa\": 95000.0,\n          \"total_molar_flow_mol_s\": 0.0,\n          \"overall_mole_fractions\": {\n            \"component-a\": 0.5,\n            \"component-b\": 0.5\n          },\n          \"phases\": []\n        }",
                    "",
                    1,
                )
                .replacen(
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                );
        fs::write(&project_path, project_json).expect("expected local rules project");

        (
            crate::StudioRuntimeConfig {
                project_path: project_path.clone(),
                ..synced_workspace_config()
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
    fn app_window_host_manager_accept_canvas_suggestion_rejoins_automatic_mainline() {
        let (config, project_path) = flash_drum_local_rules_synced_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        manager.refresh_local_canvas_suggestions();
        let window = manager.open_window();

        let activate = manager
            .dispatch_run_panel_action(window.window_id, RunPanelActionId::SetActive)
            .expect("expected activate dispatch");
        match &activate.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                crate::StudioAppResultDispatch::WorkspaceMode(mode) => {
                    assert_eq!(mode.simulation_mode, rf_ui::SimulationMode::Active);
                    assert_eq!(
                        mode.pending_reason,
                        Some(rf_ui::SolvePendingReason::ModeActivated)
                    );
                }
                other => panic!("expected workspace mode dispatch, got {other:?}"),
            },
            other => panic!("expected app command dispatch, got {other:?}"),
        }

        let accepted = manager
            .accept_focused_canvas_suggestion_by_tab()
            .expect("expected canvas suggestion acceptance")
            .expect("expected focused suggestion");
        assert_eq!(
            accepted.id.as_str(),
            "local.flash_drum.create_outlet.flash-1.vapor"
        );

        let app_state = manager.session().host_port().runtime().app_state();
        assert_eq!(
            app_state.workspace.run_panel.run_status,
            rf_ui::RunStatus::Converged
        );
        assert_eq!(app_state.workspace.run_panel.pending_reason, None);
        assert_eq!(
            app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
            Some("example-feed-heater-flash-rev-1-seq-1")
        );
        assert!(
            app_state
                .workspace
                .canvas_interaction
                .suggestions
                .iter()
                .all(|suggestion| {
                    suggestion.id.as_str() != "local.flash_drum.create_outlet.flash-1.vapor"
                }),
            "accepted suggestion should be removed after local rules refresh"
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_window_host_manager_executes_canvas_interaction_through_command_surface() {
        let (config, project_path) = flash_drum_local_rules_synced_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        manager.refresh_local_canvas_suggestions();
        let window = manager.open_window();

        let _ = manager
            .execute_command(StudioAppWindowHostCommand::DispatchTrigger {
                window_id: window.window_id,
                trigger: StudioRuntimeTrigger::WidgetAction(RunPanelActionId::SetActive),
            })
            .expect("expected activate command dispatch");

        let interaction = manager
            .execute_command(StudioAppWindowHostCommand::DispatchCanvasInteraction {
                action: StudioCanvasInteractionAction::AcceptFocusedByTab,
            })
            .expect("expected canvas interaction command");
        match interaction {
            StudioAppWindowHostCommandOutcome::CanvasInteracted(
                StudioAppWindowHostCanvasInteractionResult {
                    action: StudioCanvasInteractionAction::AcceptFocusedByTab,
                    accepted: Some(accepted),
                    rejected: None,
                    focused: None,
                },
            ) => {
                assert_eq!(
                    accepted.id.as_str(),
                    "local.flash_drum.create_outlet.flash-1.vapor"
                );
            }
            other => panic!("expected canvas interaction outcome, got {other:?}"),
        }

        let app_state = manager.session().host_port().runtime().app_state();
        assert_eq!(
            app_state.workspace.run_panel.run_status,
            rf_ui::RunStatus::Converged
        );
        assert_eq!(app_state.workspace.run_panel.pending_reason, None);

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_window_host_manager_dispatches_foreground_run_panel_recovery() {
        let (config, project_path) = solver_failure_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();
        let _ = manager
            .focus_window(second.window_id)
            .expect("expected second window focus");

        let _ = manager
            .dispatch_trigger(
                second.window_id,
                &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");

        let recovery = manager
            .dispatch_foreground_run_panel_recovery_action()
            .expect("expected recovery dispatch")
            .expect("expected foreground recovery dispatch");

        assert_eq!(recovery.target_window_id, second.window_id);
        assert_ne!(recovery.target_window_id, first.window_id);
        match &recovery.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
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
    fn app_window_host_manager_dispatches_foreground_entitlement_primary_action() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();
        let _ = manager
            .focus_window(second.window_id)
            .expect("expected second window focus");

        let dispatch = manager
            .dispatch_foreground_entitlement_primary_action()
            .expect("expected foreground entitlement primary result")
            .expect("expected foreground entitlement primary dispatch");

        assert_eq!(dispatch.target_window_id, second.window_id);
        assert_ne!(dispatch.target_window_id, first.window_id);
        assert_eq!(manager.foreground_window_id(), Some(second.window_id));
        match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioBootstrapDispatch::AppCommand(outcome) => {
                assert!(matches!(
                    outcome.dispatch,
                    crate::StudioAppResultDispatch::Entitlement(_)
                ));
            }
            other => panic!("expected entitlement app command dispatch, got {other:?}"),
        }
    }

    #[test]
    fn app_window_host_manager_dispatches_foreground_entitlement_action() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();
        let _ = manager
            .focus_window(second.window_id)
            .expect("expected second window focus");

        let dispatch = manager
            .dispatch_foreground_entitlement_action(EntitlementActionId::SyncEntitlement)
            .expect("expected foreground entitlement action result")
            .expect("expected foreground entitlement action dispatch");

        assert_eq!(dispatch.target_window_id, second.window_id);
        assert_ne!(dispatch.target_window_id, first.window_id);
        assert_eq!(manager.foreground_window_id(), Some(second.window_id));
        match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioBootstrapDispatch::AppCommand(outcome) => match &outcome.dispatch {
                crate::StudioAppResultDispatch::Entitlement(entitlement) => {
                    assert_eq!(
                        entitlement.action,
                        crate::StudioEntitlementAction::SyncEntitlement
                    );
                }
                other => panic!("expected entitlement dispatch, got {other:?}"),
            },
            other => panic!("expected entitlement app command dispatch, got {other:?}"),
        }
    }

    #[test]
    fn app_window_host_manager_dispatches_run_panel_recovery_via_ui_action() {
        let (config, project_path) = solver_failure_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();
        let _ = manager
            .focus_window(second.window_id)
            .expect("expected second window focus");
        let _ = manager
            .dispatch_trigger(
                second.window_id,
                &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");

        let recovery = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::RecoverRunPanelFailure)
            .expect("expected ui action dispatch")
            .expect("expected routed recovery dispatch");

        assert_eq!(recovery.target_window_id, second.window_id);
        assert_ne!(recovery.target_window_id, first.window_id);
        match &recovery.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
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
    fn app_window_host_manager_dispatches_run_manual_via_ui_action() {
        let (config, project_path) = solver_failure_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        let window = manager.open_window();

        let dispatch = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::RunManualWorkspace)
            .expect("expected ui action dispatch")
            .expect("expected routed run dispatch");

        assert_eq!(dispatch.target_window_id, window.window_id);
        assert_eq!(
            dispatch.dispatch.host_output.runtime_output.trigger,
            StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual)
        );
        match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
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

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_window_host_manager_dispatches_resume_via_ui_action() {
        let (config, project_path) = solver_failure_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        let window = manager.open_window();

        let dispatch = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::ResumeWorkspace)
            .expect("expected ui action dispatch")
            .expect("expected routed resume dispatch");

        assert_eq!(dispatch.target_window_id, window.window_id);
        assert_eq!(
            dispatch.dispatch.host_output.runtime_output.trigger,
            StudioRuntimeTrigger::WidgetAction(RunPanelActionId::Resume)
        );
        match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
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

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_window_host_manager_dispatches_activate_via_ui_action() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
        let window = manager.open_window();

        let dispatch = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::ActivateWorkspace)
            .expect("expected ui action dispatch")
            .expect("expected routed activate dispatch");

        assert_eq!(dispatch.target_window_id, window.window_id);
        assert_eq!(
            dispatch.dispatch.host_output.runtime_output.trigger,
            StudioRuntimeTrigger::WidgetAction(RunPanelActionId::SetActive)
        );
        match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                crate::StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                    assert_eq!(dispatch.simulation_mode, rf_ui::SimulationMode::Active);
                }
                other => panic!("expected workspace mode dispatch, got {other:?}"),
            },
            other => panic!("expected app command dispatch, got {other:?}"),
        }
    }

    #[test]
    fn app_window_host_manager_dispatches_hold_via_ui_action_after_activation() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
        let window = manager.open_window();
        let _ = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::ActivateWorkspace)
            .expect("expected ui action dispatch")
            .expect("expected routed activate dispatch");

        let dispatch = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::HoldWorkspace)
            .expect("expected ui action dispatch")
            .expect("expected routed hold dispatch");

        assert_eq!(dispatch.target_window_id, window.window_id);
        assert_eq!(
            dispatch.dispatch.host_output.runtime_output.trigger,
            StudioRuntimeTrigger::WidgetAction(RunPanelActionId::SetHold)
        );
        match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                crate::StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                    assert_eq!(dispatch.simulation_mode, rf_ui::SimulationMode::Hold);
                }
                other => panic!("expected workspace mode dispatch, got {other:?}"),
            },
            other => panic!("expected app command dispatch, got {other:?}"),
        }
    }

    #[test]
    fn app_window_host_manager_reports_ui_action_states_for_run_panel_commands() {
        let (config, project_path) = solver_failure_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();
        let _ = manager
            .focus_window(second.window_id)
            .expect("expected second window focus");

        let run_manual = manager.ui_action_state(StudioAppWindowHostUiAction::RunManualWorkspace);
        assert_eq!(
            run_manual,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::RunManualWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Enabled {
                    target_window_id: second.window_id,
                },
            }
        );
        assert!(run_manual.enabled());

        let resume = manager.ui_action_state(StudioAppWindowHostUiAction::ResumeWorkspace);
        assert_eq!(
            resume,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::ResumeWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Enabled {
                    target_window_id: second.window_id,
                },
            }
        );
        assert!(resume.enabled());

        let hold = manager.ui_action_state(StudioAppWindowHostUiAction::HoldWorkspace);
        assert_eq!(
            hold,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::HoldWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::HoldUnavailable,
                    target_window_id: Some(second.window_id),
                },
            }
        );
        assert!(!hold.enabled());

        let activate = manager.ui_action_state(StudioAppWindowHostUiAction::ActivateWorkspace);
        assert_eq!(
            activate,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::ActivateWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Enabled {
                    target_window_id: second.window_id,
                },
            }
        );
        assert!(activate.enabled());

        let initial = manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
        assert_eq!(
            initial,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::NoRunPanelRecovery,
                    target_window_id: Some(second.window_id),
                },
            }
        );
        assert!(!initial.enabled());
        assert_eq!(initial.target_window_id(), Some(second.window_id));

        let failed_run = manager
            .dispatch_trigger(
                second.window_id,
                &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");
        assert_eq!(failed_run.target_window_id, second.window_id);

        let recovery = manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
        assert_eq!(
            recovery,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
                availability: StudioAppWindowHostUiActionAvailability::Enabled {
                    target_window_id: second.window_id,
                },
            }
        );
        assert!(recovery.enabled());

        let resume_disabled = manager.ui_action_state(StudioAppWindowHostUiAction::ResumeWorkspace);
        assert_eq!(
            resume_disabled,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::ResumeWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::ResumeUnavailable,
                    target_window_id: Some(second.window_id),
                },
            }
        );
        assert!(!resume_disabled.enabled());

        let states = manager.ui_action_states();
        assert_eq!(
            states,
            vec![
                run_manual.clone(),
                resume_disabled.clone(),
                hold.clone(),
                activate.clone(),
                recovery.clone()
            ]
        );
        assert_ne!(run_manual.target_window_id(), Some(first.window_id));
        assert_ne!(resume_disabled.target_window_id(), Some(first.window_id));
        assert_ne!(hold.target_window_id(), Some(first.window_id));
        assert_ne!(activate.target_window_id(), Some(first.window_id));
        assert_ne!(recovery.target_window_id(), Some(first.window_id));

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_window_host_manager_reports_ui_action_state_for_foreground_recovery() {
        let (config, project_path) = solver_failure_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();
        let _ = manager
            .focus_window(second.window_id)
            .expect("expected second window focus");

        let initial = manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
        assert_eq!(
            initial,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::NoRunPanelRecovery,
                    target_window_id: Some(second.window_id),
                },
            }
        );
        assert!(!initial.enabled());
        assert_eq!(initial.target_window_id(), Some(second.window_id));

        let _ = manager
            .dispatch_trigger(
                second.window_id,
                &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");

        let available =
            manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
        assert_eq!(
            available,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
                availability: StudioAppWindowHostUiActionAvailability::Enabled {
                    target_window_id: second.window_id,
                },
            }
        );
        assert!(available.enabled());

        let states = manager.ui_action_states();
        assert_eq!(
            states,
            vec![
                StudioAppWindowHostUiActionState {
                    action: StudioAppWindowHostUiAction::RunManualWorkspace,
                    availability: StudioAppWindowHostUiActionAvailability::Enabled {
                        target_window_id: second.window_id,
                    },
                },
                StudioAppWindowHostUiActionState {
                    action: StudioAppWindowHostUiAction::ResumeWorkspace,
                    availability: StudioAppWindowHostUiActionAvailability::Disabled {
                        reason: StudioAppWindowHostUiActionDisabledReason::ResumeUnavailable,
                        target_window_id: Some(second.window_id),
                    },
                },
                StudioAppWindowHostUiActionState {
                    action: StudioAppWindowHostUiAction::HoldWorkspace,
                    availability: StudioAppWindowHostUiActionAvailability::Disabled {
                        reason: StudioAppWindowHostUiActionDisabledReason::HoldUnavailable,
                        target_window_id: Some(second.window_id),
                    },
                },
                StudioAppWindowHostUiActionState {
                    action: StudioAppWindowHostUiAction::ActivateWorkspace,
                    availability: StudioAppWindowHostUiActionAvailability::Enabled {
                        target_window_id: second.window_id,
                    },
                },
                available.clone(),
            ]
        );
        assert_ne!(available.target_window_id(), Some(first.window_id));

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_window_host_manager_routes_global_recovery_request_to_foreground_window() {
        let (config, project_path) = solver_failure_config();
        let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
        let first = manager.open_window();
        let second = manager.open_window();
        let _ = manager
            .focus_window(second.window_id)
            .expect("expected second window focus");
        let _ = manager
            .dispatch_trigger(
                second.window_id,
                &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");

        let dispatch = manager
            .dispatch_global_event(StudioAppWindowHostGlobalEvent::RunPanelRecoveryRequested)
            .expect("expected global recovery dispatch")
            .expect("expected routed recovery dispatch");

        assert_eq!(dispatch.target_window_id, second.window_id);
        assert_ne!(dispatch.target_window_id, first.window_id);
        match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
            crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
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
    fn app_window_host_manager_ignores_ui_actions_without_windows() {
        let mut manager =
            StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");

        let run_manual = manager.ui_action_state(StudioAppWindowHostUiAction::RunManualWorkspace);
        assert_eq!(
            run_manual,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::RunManualWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            }
        );
        assert!(!run_manual.enabled());
        assert_eq!(run_manual.target_window_id(), None);

        let resume = manager.ui_action_state(StudioAppWindowHostUiAction::ResumeWorkspace);
        assert_eq!(
            resume,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::ResumeWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            }
        );
        assert!(!resume.enabled());
        assert_eq!(resume.target_window_id(), None);

        let hold = manager.ui_action_state(StudioAppWindowHostUiAction::HoldWorkspace);
        assert_eq!(
            hold,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::HoldWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            }
        );
        assert!(!hold.enabled());
        assert_eq!(hold.target_window_id(), None);

        let activate = manager.ui_action_state(StudioAppWindowHostUiAction::ActivateWorkspace);
        assert_eq!(
            activate,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::ActivateWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            }
        );
        assert!(!activate.enabled());
        assert_eq!(activate.target_window_id(), None);

        let recovery = manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
        assert_eq!(
            recovery,
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            }
        );
        assert!(!recovery.enabled());
        assert_eq!(recovery.target_window_id(), None);

        let run_dispatch = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::RunManualWorkspace)
            .expect("expected ui action result");
        assert!(run_dispatch.is_none());

        let resume_dispatch = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::ResumeWorkspace)
            .expect("expected ui action result");
        assert!(resume_dispatch.is_none());

        let hold_dispatch = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::HoldWorkspace)
            .expect("expected ui action result");
        assert!(hold_dispatch.is_none());

        let activate_dispatch = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::ActivateWorkspace)
            .expect("expected ui action result");
        assert!(activate_dispatch.is_none());

        let recovery_dispatch = manager
            .dispatch_ui_action(StudioAppWindowHostUiAction::RecoverRunPanelFailure)
            .expect("expected ui action result");

        assert!(recovery_dispatch.is_none());
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

        let ignored_action = manager
            .execute_command(StudioAppWindowHostCommand::DispatchUiAction {
                action: StudioAppWindowHostUiAction::RunManualWorkspace,
            })
            .expect("expected ignored ui action");
        assert_eq!(
            ignored_action,
            StudioAppWindowHostCommandOutcome::IgnoredUiAction
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
