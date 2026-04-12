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
                Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch))
            }
            StudioAppWindowHostCommand::DispatchCanvasInteraction { action } => self
                .dispatch_canvas_interaction(action)
                .map(StudioAppWindowHostCommandOutcome::CanvasInteracted),
            StudioAppWindowHostCommand::DispatchUiAction { action } => {
                match self.dispatch_ui_action(action)? {
                    Some(dispatch) => {
                        Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch))
                    }
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredUiAction),
                }
            }
            StudioAppWindowHostCommand::DispatchRunPanelRecoveryAction { window_id } => {
                let dispatch = self.dispatch_run_panel_recovery_action(window_id)?;
                Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch))
            }
            StudioAppWindowHostCommand::DispatchForegroundRunPanelRecoveryAction => {
                match self.dispatch_foreground_run_panel_recovery_action()? {
                    Some(dispatch) => {
                        Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch))
                    }
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredUiAction),
                }
            }
            StudioAppWindowHostCommand::DispatchForegroundEntitlementPrimaryAction => {
                match self.dispatch_foreground_entitlement_primary_action()? {
                    Some(dispatch) => {
                        Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch))
                    }
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredUiAction),
                }
            }
            StudioAppWindowHostCommand::DispatchForegroundEntitlementAction { action_id } => {
                match self.dispatch_foreground_entitlement_action(action_id)? {
                    Some(dispatch) => {
                        Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch))
                    }
                    None => Ok(StudioAppWindowHostCommandOutcome::IgnoredUiAction),
                }
            }
            StudioAppWindowHostCommand::FocusWindow { window_id } => {
                let dispatch = self.focus_window(window_id)?;
                Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch))
            }
            StudioAppWindowHostCommand::DispatchGlobalEvent { event } => {
                match self.dispatch_global_event(event)? {
                    Some(dispatch) => {
                        Ok(StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch))
                    }
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

        self.dispatch_run_panel_action(window_id, action_id).map(Some)
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

        self.dispatch_trigger(window_id, &StudioRuntimeTrigger::EntitlementWidgetPrimaryAction)
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

        self.dispatch_trigger(window_id, &StudioRuntimeTrigger::EntitlementWidgetAction(action_id))
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
mod tests;
