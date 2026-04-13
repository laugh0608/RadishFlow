use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use rf_types::{RfError, RfResult};

use crate::{
    StudioAppWindowHostCanvasInteractionResult, StudioAppWindowHostClose,
    StudioAppWindowHostCommand, StudioAppWindowHostCommandOutcome, StudioAppWindowHostDispatch,
    StudioAppWindowHostGlobalEvent, StudioAppWindowHostManager, StudioAppWindowHostOpenWindow,
    StudioAppWindowHostUiAction, StudioAppWindowHostUiActionAvailability,
    StudioAppWindowHostUiActionDisabledReason, StudioAppWindowHostUiActionState,
    StudioCanvasInteractionAction, StudioRuntimeConfig, StudioRuntimeHostAckResult,
    StudioRuntimeReport, StudioRuntimeTimerHandleSlot, StudioRuntimeTrigger, StudioWindowHostEvent,
    StudioWindowHostId, StudioWindowHostRegistration, StudioWindowHostRetirement,
    StudioWindowHostRole, StudioWindowSessionDispatch, StudioWindowTimerDriverAckResult,
    StudioWindowTimerDriverTransition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppHostUiAction {
    RunManualWorkspace,
    ResumeWorkspace,
    HoldWorkspace,
    ActivateWorkspace,
    RecoverRunPanelFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppHostUiActionDisabledReason {
    NoRegisteredWindow,
    RunManualUnavailable,
    ResumeUnavailable,
    HoldUnavailable,
    ActivateUnavailable,
    NoRunPanelRecovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppHostUiActionAvailability {
    Enabled {
        target_window_id: StudioWindowHostId,
    },
    Disabled {
        reason: StudioAppHostUiActionDisabledReason,
        target_window_id: Option<StudioWindowHostId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostUiActionState {
    pub action: StudioAppHostUiAction,
    pub availability: StudioAppHostUiActionAvailability,
}

impl StudioAppHostUiActionState {
    pub fn enabled(&self) -> bool {
        matches!(
            self.availability,
            StudioAppHostUiActionAvailability::Enabled { .. }
        )
    }

    pub fn target_window_id(&self) -> Option<StudioWindowHostId> {
        match self.availability {
            StudioAppHostUiActionAvailability::Enabled { target_window_id } => {
                Some(target_window_id)
            }
            StudioAppHostUiActionAvailability::Disabled {
                target_window_id, ..
            } => target_window_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostUiActionModel {
    pub action: Option<StudioAppHostUiAction>,
    pub command_id: &'static str,
    pub group: StudioAppHostUiCommandGroup,
    pub sort_order: u16,
    pub label: &'static str,
    pub enabled: bool,
    pub detail: &'static str,
    pub target_window_id: Option<StudioWindowHostId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppHostUiCommandGroup {
    RunPanel,
    Recovery,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StudioAppHostUiCommandModel {
    pub actions: Vec<StudioAppHostUiActionModel>,
}

impl StudioAppHostUiCommandModel {
    pub fn action(&self, action: StudioAppHostUiAction) -> Option<&StudioAppHostUiActionModel> {
        self.actions
            .iter()
            .find(|candidate| candidate.action == Some(action))
    }

    pub fn command(&self, command_id: &str) -> Option<&StudioAppHostUiActionModel> {
        self.actions
            .iter()
            .find(|candidate| candidate.command_id == command_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppHostCommand {
    OpenWindow,
    DispatchWindowTrigger {
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    },
    DispatchCanvasInteraction {
        action: StudioCanvasInteractionAction,
    },
    DispatchUiAction {
        action: StudioAppHostUiAction,
    },
    DispatchWindowRunPanelRecoveryAction {
        window_id: StudioWindowHostId,
    },
    DispatchForegroundRunPanelRecoveryAction,
    DispatchForegroundEntitlementPrimaryAction,
    DispatchForegroundEntitlementAction {
        action_id: rf_ui::EntitlementActionId,
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

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StudioAppHostCommandOutcome {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostWindowSnapshot {
    pub window_id: StudioWindowHostId,
    pub role: StudioWindowHostRole,
    pub layout_slot: u16,
    pub is_foreground: bool,
    pub entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
}

pub type StudioAppHostWindowState = StudioAppHostWindowSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppHostWindowChange {
    Added {
        current: StudioAppHostWindowSnapshot,
    },
    Removed {
        previous: StudioAppHostWindowSnapshot,
    },
    Updated {
        previous: StudioAppHostWindowSnapshot,
        current: StudioAppHostWindowSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostWindowSelectionChange {
    pub previous: Option<StudioWindowHostId>,
    pub current: Option<StudioWindowHostId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostTimerSlotChange {
    pub previous: Option<StudioRuntimeTimerHandleSlot>,
    pub current: Option<StudioRuntimeTimerHandleSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostSnapshot {
    pub registered_windows: Vec<StudioWindowHostId>,
    pub windows: Vec<StudioAppHostWindowSnapshot>,
    pub ui_actions: Vec<StudioAppHostUiActionState>,
    pub foreground_window_id: Option<StudioWindowHostId>,
    pub entitlement_timer_owner_window_id: Option<StudioWindowHostId>,
    pub parked_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum StudioAppHostEntitlementTimerState {
    #[default]
    Idle,
    Owned {
        owner_window_id: StudioWindowHostId,
        slot: Option<StudioRuntimeTimerHandleSlot>,
    },
    Parked {
        slot: StudioRuntimeTimerHandleSlot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostState {
    pub registered_windows: Vec<StudioWindowHostId>,
    pub windows: Vec<StudioAppHostWindowState>,
    pub ui_actions: Vec<StudioAppHostUiActionState>,
    pub foreground_window_id: Option<StudioWindowHostId>,
    pub entitlement_timer: StudioAppHostEntitlementTimerState,
}

impl StudioAppHostState {
    pub fn from_snapshot(snapshot: &StudioAppHostSnapshot) -> Self {
        Self {
            registered_windows: snapshot.registered_windows.clone(),
            windows: snapshot.windows.clone(),
            ui_actions: snapshot.ui_actions.clone(),
            foreground_window_id: snapshot.foreground_window_id,
            entitlement_timer: entitlement_timer_state_from_snapshot(snapshot),
        }
    }

    pub fn window(&self, window_id: StudioWindowHostId) -> Option<&StudioAppHostWindowState> {
        self.windows
            .iter()
            .find(|window| window.window_id == window_id)
    }

    pub fn ui_action_state(
        &self,
        action: StudioAppHostUiAction,
    ) -> Option<&StudioAppHostUiActionState> {
        self.ui_actions.iter().find(|state| state.action == action)
    }

    pub fn entitlement_timer_owner_window_id(&self) -> Option<StudioWindowHostId> {
        match self.entitlement_timer {
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id, ..
            } => Some(owner_window_id),
            StudioAppHostEntitlementTimerState::Idle
            | StudioAppHostEntitlementTimerState::Parked { .. } => None,
        }
    }

    pub fn parked_entitlement_timer(&self) -> Option<&StudioRuntimeTimerHandleSlot> {
        match &self.entitlement_timer {
            StudioAppHostEntitlementTimerState::Parked { slot } => Some(slot),
            StudioAppHostEntitlementTimerState::Idle
            | StudioAppHostEntitlementTimerState::Owned { .. } => None,
        }
    }

    pub fn ui_command_model(&self) -> StudioAppHostUiCommandModel {
        ui_command_model_from_states(&self.ui_actions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostEntitlementTimerStateChange {
    pub previous: StudioAppHostEntitlementTimerState,
    pub current: StudioAppHostEntitlementTimerState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostProjection {
    pub state: StudioAppHostState,
    pub added_windows: Vec<StudioAppHostWindowState>,
    pub removed_window_ids: Vec<StudioWindowHostId>,
    pub updated_windows: Vec<StudioAppHostWindowState>,
    pub foreground_window_change: Option<StudioAppHostWindowSelectionChange>,
    pub entitlement_timer_change: Option<StudioAppHostEntitlementTimerStateChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostChangeSet {
    pub window_changes: Vec<StudioAppHostWindowChange>,
    pub foreground_window_change: Option<StudioAppHostWindowSelectionChange>,
    pub entitlement_timer_owner_change: Option<StudioAppHostWindowSelectionChange>,
    pub parked_entitlement_timer_change: Option<StudioAppHostTimerSlotChange>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioAppHostOutput {
    pub outcome: StudioAppHostCommandOutcome,
    pub snapshot: StudioAppHostSnapshot,
    pub changes: StudioAppHostChangeSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostStore {
    state: StudioAppHostState,
}

impl StudioAppHostStore {
    pub fn new(initial_state: StudioAppHostState) -> Self {
        Self {
            state: initial_state,
        }
    }

    pub fn from_snapshot(snapshot: &StudioAppHostSnapshot) -> Self {
        Self::new(StudioAppHostState::from_snapshot(snapshot))
    }

    pub fn state(&self) -> &StudioAppHostState {
        &self.state
    }

    pub fn project_output(&self, output: &StudioAppHostOutput) -> StudioAppHostProjection {
        let next_state = StudioAppHostState::from_snapshot(&output.snapshot);
        let mut added_windows = Vec::new();
        let mut removed_window_ids = Vec::new();
        let mut updated_windows = Vec::new();

        for change in &output.changes.window_changes {
            match change {
                StudioAppHostWindowChange::Added { current } => {
                    added_windows.push(current.clone());
                }
                StudioAppHostWindowChange::Removed { previous } => {
                    removed_window_ids.push(previous.window_id);
                }
                StudioAppHostWindowChange::Updated { current, .. } => {
                    updated_windows.push(current.clone());
                }
            }
        }

        let entitlement_timer_change = diff_entitlement_timer_state(
            &self.state.entitlement_timer,
            &next_state.entitlement_timer,
        );

        StudioAppHostProjection {
            state: next_state,
            added_windows,
            removed_window_ids,
            updated_windows,
            foreground_window_change: output.changes.foreground_window_change.clone(),
            entitlement_timer_change,
        }
    }

    pub fn apply_output(&mut self, output: &StudioAppHostOutput) -> StudioAppHostProjection {
        let projection = self.project_output(output);
        self.state = projection.state.clone();
        projection
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostOpenWindowResult {
    pub projection: StudioAppHostProjection,
    pub registration: StudioWindowHostRegistration,
    pub native_timer_transitions: Vec<StudioWindowTimerDriverTransition>,
    pub native_timer_acks: Vec<StudioWindowTimerDriverAckResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostWindowDispatchResult {
    pub projection: StudioAppHostProjection,
    pub target_window_id: StudioWindowHostId,
    pub effects: StudioAppHostDispatchEffects,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum StudioAppHostUiCommandDispatchResult {
    Executed(StudioAppHostWindowDispatchResult),
    IgnoredDisabled {
        command_id: String,
        detail: String,
        target_window_id: Option<StudioWindowHostId>,
    },
    IgnoredMissing {
        command_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostGlobalEventResult {
    pub projection: StudioAppHostProjection,
    pub dispatch: Option<StudioAppHostWindowDispatchResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostCloseWindowResult {
    pub projection: StudioAppHostProjection,
    pub close: Option<StudioAppHostCloseEffects>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioAppHostEntitlementTimerEffect {
    Keep {
        owner_window_id: StudioWindowHostId,
        effect_id: u64,
        slot: StudioRuntimeTimerHandleSlot,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
        ack: StudioRuntimeHostAckResult,
    },
    Arm {
        owner_window_id: StudioWindowHostId,
        effect_id: u64,
        slot: StudioRuntimeTimerHandleSlot,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
        ack: StudioRuntimeHostAckResult,
    },
    Rearm {
        owner_window_id: StudioWindowHostId,
        effect_id: u64,
        previous_slot: Option<StudioRuntimeTimerHandleSlot>,
        next_slot: StudioRuntimeTimerHandleSlot,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
        ack: StudioRuntimeHostAckResult,
    },
    Clear {
        owner_window_id: StudioWindowHostId,
        effect_id: u64,
        previous_slot: Option<StudioRuntimeTimerHandleSlot>,
        follow_up_trigger: Option<StudioRuntimeTrigger>,
        ack: StudioRuntimeHostAckResult,
    },
    IgnoreStale {
        owner_window_id: StudioWindowHostId,
        stale_effect_id: u64,
        current_slot: Option<StudioRuntimeTimerHandleSlot>,
        ack: StudioRuntimeHostAckResult,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostDispatchEffects {
    pub runtime_report: StudioRuntimeReport,
    pub entitlement_timer_effect: Option<StudioAppHostEntitlementTimerEffect>,
    pub native_timer_transitions: Vec<StudioWindowTimerDriverTransition>,
    pub native_timer_acks: Vec<StudioWindowTimerDriverAckResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioAppHostCloseEffects {
    pub window_id: StudioWindowHostId,
    pub cleared_entitlement_timer: Option<StudioRuntimeTimerHandleSlot>,
    pub retirement: StudioWindowHostRetirement,
    pub next_foreground_window_id: Option<StudioWindowHostId>,
    pub native_timer_transitions: Vec<StudioWindowTimerDriverTransition>,
    pub native_timer_acks: Vec<StudioWindowTimerDriverAckResult>,
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

    pub fn refresh_local_canvas_suggestions(&mut self) {
        self.window_host_manager.refresh_local_canvas_suggestions();
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<rf_ui::CanvasSuggestion>) {
        self.window_host_manager
            .replace_canvas_suggestions(suggestions);
    }

    pub fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::AcceptFocusedByTab)
            .map(|result| result.accepted)
    }

    pub fn reject_focused_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::RejectFocused)
            .expect("canvas rejection should not fail")
            .rejected
    }

    pub fn focus_next_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::FocusNext)
            .expect("canvas focus-next should not fail")
            .focused
    }

    pub fn focus_previous_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::FocusPrevious)
            .expect("canvas focus-previous should not fail")
            .focused
    }

    pub fn dispatch_canvas_interaction(
        &mut self,
        action: StudioCanvasInteractionAction,
    ) -> RfResult<StudioAppWindowHostCanvasInteractionResult> {
        self.window_host_manager.dispatch_canvas_interaction(action)
    }

    pub fn latest_log_entry(&self) -> Option<rf_ui::AppLogEntry> {
        self.window_host_manager
            .session()
            .host_port()
            .runtime()
            .app_state()
            .log_feed
            .entries
            .back()
            .cloned()
    }

    pub fn log_entries(&self) -> Vec<rf_ui::AppLogEntry> {
        self.window_host_manager
            .session()
            .host_port()
            .runtime()
            .app_state()
            .log_feed
            .entries
            .iter()
            .cloned()
            .collect()
    }

    pub fn workspace_control_state(&self) -> crate::WorkspaceControlState {
        crate::snapshot_workspace_control_state(
            self.window_host_manager
                .session()
                .host_port()
                .runtime()
                .app_state(),
        )
    }

    pub fn run_panel_widget(&self) -> rf_ui::RunPanelWidgetModel {
        rf_ui::RunPanelWidgetModel::from_state(
            &self
                .window_host_manager
                .session()
                .host_port()
                .runtime()
                .app_state()
                .workspace
                .run_panel,
        )
    }

    pub fn entitlement_host_output(&self) -> Option<crate::EntitlementSessionHostRuntimeOutput> {
        self.window_host_manager
            .session()
            .host_port()
            .runtime()
            .host_runtime()
            .last_output()
    }

    pub fn active_inspector_target(&self) -> Option<rf_ui::InspectorTarget> {
        self.window_host_manager
            .session()
            .host_port()
            .runtime()
            .app_state()
            .workspace
            .drafts
            .active_target
            .clone()
    }

    pub fn canvas_interaction(&self) -> rf_ui::CanvasInteractionState {
        self.window_host_manager
            .session()
            .host_port()
            .runtime()
            .app_state()
            .workspace
            .canvas_interaction
            .clone()
    }

    pub fn document_path(&self) -> Option<&Path> {
        self.window_host_manager
            .session()
            .host_port()
            .runtime()
            .app_state()
            .workspace
            .document_path
            .as_deref()
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
                let window_state = self
                    .window_host_manager
                    .session()
                    .host_port()
                    .window_state(window_id)
                    .expect("expected registered window state");
                let layout_slot = match role {
                    StudioWindowHostRole::EntitlementTimerOwner => 1,
                    StudioWindowHostRole::Observer => window_state
                        .observer_layout_slot()
                        .expect("expected observer layout slot"),
                };
                let entitlement_timer = window_state.entitlement_timer().cloned();

                StudioAppHostWindowSnapshot {
                    window_id,
                    role,
                    layout_slot,
                    is_foreground: foreground_window_id == Some(window_id),
                    entitlement_timer,
                }
            })
            .collect();
        let ui_actions = self
            .window_host_manager
            .ui_action_states()
            .into_iter()
            .map(ui_action_state_from_window_host)
            .collect();

        StudioAppHostSnapshot {
            registered_windows,
            windows,
            ui_actions,
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
        let previous_snapshot = self.snapshot();
        let outcome = self
            .window_host_manager
            .execute_command(map_command(command))
            .map(map_outcome)?;
        let snapshot = self.snapshot();

        Ok(StudioAppHostOutput {
            outcome,
            changes: diff_app_host_snapshots(&previous_snapshot, &snapshot),
            snapshot,
        })
    }
}

impl StudioAppHostSnapshot {
    pub fn ui_command_model(&self) -> StudioAppHostUiCommandModel {
        ui_command_model_from_states(&self.ui_actions)
    }
}

pub struct StudioAppHostController {
    app_host: StudioAppHost,
    store: StudioAppHostStore,
}

impl StudioAppHostController {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        let mut app_host = StudioAppHost::new(config)?;
        app_host.refresh_local_canvas_suggestions();
        let store = StudioAppHostStore::from_snapshot(&app_host.snapshot());

        Ok(Self { app_host, store })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.store.state()
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<rf_ui::CanvasSuggestion>) {
        self.app_host.replace_canvas_suggestions(suggestions);
    }

    pub fn refresh_local_canvas_suggestions(&mut self) {
        self.app_host.refresh_local_canvas_suggestions();
    }

    pub fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::AcceptFocusedByTab)
            .map(|result| result.accepted)
    }

    pub fn reject_focused_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::RejectFocused)
            .expect("canvas rejection should not fail")
            .rejected
    }

    pub fn focus_next_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::FocusNext)
            .expect("canvas focus-next should not fail")
            .focused
    }

    pub fn focus_previous_canvas_suggestion(&mut self) -> Option<rf_ui::CanvasSuggestion> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::FocusPrevious)
            .expect("canvas focus-previous should not fail")
            .focused
    }

    pub fn dispatch_canvas_interaction(
        &mut self,
        action: StudioCanvasInteractionAction,
    ) -> RfResult<StudioAppWindowHostCanvasInteractionResult> {
        let (outcome, _) =
            self.execute_command(StudioAppHostCommand::DispatchCanvasInteraction { action })?;
        let StudioAppHostCommandOutcome::CanvasInteracted(result) = outcome else {
            return Err(RfError::invalid_input(format!(
                "app host controller expected canvas interaction outcome for {action:?}"
            )));
        };
        Ok(result)
    }

    pub fn latest_log_entry(&self) -> Option<rf_ui::AppLogEntry> {
        self.app_host.latest_log_entry()
    }

    pub fn log_entries(&self) -> Vec<rf_ui::AppLogEntry> {
        self.app_host.log_entries()
    }

    pub fn workspace_control_state(&self) -> crate::WorkspaceControlState {
        self.app_host.workspace_control_state()
    }

    pub fn run_panel_widget(&self) -> rf_ui::RunPanelWidgetModel {
        self.app_host.run_panel_widget()
    }

    pub fn entitlement_host_output(&self) -> Option<crate::EntitlementSessionHostRuntimeOutput> {
        self.app_host.entitlement_host_output()
    }

    pub fn active_inspector_target(&self) -> Option<rf_ui::InspectorTarget> {
        self.app_host.active_inspector_target()
    }

    pub fn canvas_interaction(&self) -> rf_ui::CanvasInteractionState {
        self.app_host.canvas_interaction()
    }

    pub fn document_path(&self) -> Option<&Path> {
        self.app_host.document_path()
    }

    pub fn open_window(&mut self) -> RfResult<StudioAppHostOpenWindowResult> {
        let (outcome, projection) = self.execute_command(StudioAppHostCommand::OpenWindow)?;
        let StudioAppHostCommandOutcome::WindowOpened(opened) = outcome else {
            return Err(RfError::invalid_input(
                "app host controller expected window open outcome",
            ));
        };

        Ok(StudioAppHostOpenWindowResult {
            projection,
            registration: registration_from_opened_window(&opened),
            native_timer_transitions: opened.timer_driver_transitions,
            native_timer_acks: opened.timer_driver_acks,
        })
    }

    pub fn dispatch_window_trigger(
        &mut self,
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    ) -> RfResult<StudioAppHostWindowDispatchResult> {
        let (outcome, projection) = self
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger { window_id, trigger })?;
        let StudioAppHostCommandOutcome::WindowDispatched(dispatch) = outcome else {
            return Err(RfError::invalid_input(
                "app host controller expected window dispatch outcome",
            ));
        };

        Ok(StudioAppHostWindowDispatchResult {
            projection,
            target_window_id: dispatch.target_window_id,
            effects: dispatch_effects_from_session(dispatch.dispatch),
        })
    }

    pub fn dispatch_window_run_panel_recovery_action(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioAppHostWindowDispatchResult> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::DispatchWindowRunPanelRecoveryAction {
                window_id,
            })?;
        let StudioAppHostCommandOutcome::WindowDispatched(dispatch) = outcome else {
            return Err(RfError::invalid_input(
                "app host controller expected run panel recovery dispatch outcome",
            ));
        };

        Ok(StudioAppHostWindowDispatchResult {
            projection,
            target_window_id: dispatch.target_window_id,
            effects: dispatch_effects_from_session(dispatch.dispatch),
        })
    }

    pub fn dispatch_ui_action(
        &mut self,
        action: StudioAppHostUiAction,
    ) -> RfResult<Option<StudioAppHostWindowDispatchResult>> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::DispatchUiAction { action })?;

        match outcome {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch) => {
                Ok(Some(StudioAppHostWindowDispatchResult {
                    projection,
                    target_window_id: dispatch.target_window_id,
                    effects: dispatch_effects_from_session(dispatch.dispatch),
                }))
            }
            StudioAppHostCommandOutcome::IgnoredUiAction => Ok(None),
            other => Err(RfError::invalid_input(format!(
                "app host controller expected ui action outcome, got {other:?}"
            ))),
        }
    }

    pub fn dispatch_ui_command(
        &mut self,
        command_id: &str,
    ) -> RfResult<StudioAppHostUiCommandDispatchResult> {
        let Some(command) = self.state().ui_command_model().command(command_id).cloned() else {
            return Ok(StudioAppHostUiCommandDispatchResult::IgnoredMissing {
                command_id: command_id.to_string(),
            });
        };

        if !command.enabled {
            return Ok(StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
                command_id: command.command_id.to_string(),
                detail: command.detail.to_string(),
                target_window_id: command.target_window_id,
            });
        }

        let Some(action) = command.action else {
            return Ok(StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
                command_id: command.command_id.to_string(),
                detail: command.detail.to_string(),
                target_window_id: command.target_window_id,
            });
        };

        match self.dispatch_ui_action(action)? {
            Some(dispatch) => Ok(StudioAppHostUiCommandDispatchResult::Executed(dispatch)),
            None => Err(RfError::invalid_input(format!(
                "app host controller resolved enabled ui command `{}` but dispatch returned no action",
                command.command_id
            ))),
        }
    }

    pub fn dispatch_foreground_run_panel_recovery_action(
        &mut self,
    ) -> RfResult<Option<StudioAppHostWindowDispatchResult>> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::DispatchForegroundRunPanelRecoveryAction)?;

        match outcome {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch) => {
                Ok(Some(StudioAppHostWindowDispatchResult {
                    projection,
                    target_window_id: dispatch.target_window_id,
                    effects: dispatch_effects_from_session(dispatch.dispatch),
                }))
            }
            StudioAppHostCommandOutcome::IgnoredUiAction => Ok(None),
            other => Err(RfError::invalid_input(format!(
                "app host controller expected foreground run panel recovery outcome, got {other:?}"
            ))),
        }
    }

    pub fn dispatch_foreground_entitlement_primary_action(
        &mut self,
    ) -> RfResult<Option<StudioAppHostWindowDispatchResult>> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::DispatchForegroundEntitlementPrimaryAction)?;

        match outcome {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch) => {
                Ok(Some(StudioAppHostWindowDispatchResult {
                    projection,
                    target_window_id: dispatch.target_window_id,
                    effects: dispatch_effects_from_session(dispatch.dispatch),
                }))
            }
            StudioAppHostCommandOutcome::IgnoredUiAction => Ok(None),
            other => Err(RfError::invalid_input(format!(
                "app host controller expected foreground entitlement primary outcome, got {other:?}"
            ))),
        }
    }

    pub fn dispatch_foreground_entitlement_action(
        &mut self,
        action_id: rf_ui::EntitlementActionId,
    ) -> RfResult<Option<StudioAppHostWindowDispatchResult>> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::DispatchForegroundEntitlementAction {
                action_id,
            })?;

        match outcome {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch) => {
                Ok(Some(StudioAppHostWindowDispatchResult {
                    projection,
                    target_window_id: dispatch.target_window_id,
                    effects: dispatch_effects_from_session(dispatch.dispatch),
                }))
            }
            StudioAppHostCommandOutcome::IgnoredUiAction => Ok(None),
            other => Err(RfError::invalid_input(format!(
                "app host controller expected foreground entitlement action outcome, got {other:?}"
            ))),
        }
    }

    pub fn focus_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioAppHostWindowDispatchResult> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::FocusWindow { window_id })?;
        let StudioAppHostCommandOutcome::WindowDispatched(dispatch) = outcome else {
            return Err(RfError::invalid_input(
                "app host controller expected focus dispatch outcome",
            ));
        };

        Ok(StudioAppHostWindowDispatchResult {
            projection,
            target_window_id: dispatch.target_window_id,
            effects: dispatch_effects_from_session(dispatch.dispatch),
        })
    }

    pub fn dispatch_global_event(
        &mut self,
        event: StudioAppWindowHostGlobalEvent,
    ) -> RfResult<StudioAppHostGlobalEventResult> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::DispatchGlobalEvent { event })?;
        let dispatch = match outcome {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch) => {
                Some(StudioAppHostWindowDispatchResult {
                    projection: projection.clone(),
                    target_window_id: dispatch.target_window_id,
                    effects: dispatch_effects_from_session(dispatch.dispatch),
                })
            }
            StudioAppHostCommandOutcome::IgnoredGlobalEvent { .. } => None,
            other => {
                return Err(RfError::invalid_input(format!(
                    "app host controller expected global event outcome, got {other:?}"
                )));
            }
        };

        Ok(StudioAppHostGlobalEventResult {
            projection,
            dispatch,
        })
    }

    pub fn close_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioAppHostCloseWindowResult> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::CloseWindow { window_id })?;
        let close = match outcome {
            StudioAppHostCommandOutcome::WindowClosed(close) => {
                Some(close_effects_from_shutdown(close))
            }
            StudioAppHostCommandOutcome::IgnoredClose { .. } => None,
            other => {
                return Err(RfError::invalid_input(format!(
                    "app host controller expected close outcome, got {other:?}"
                )));
            }
        };

        Ok(StudioAppHostCloseWindowResult { projection, close })
    }

    fn execute_command(
        &mut self,
        command: StudioAppHostCommand,
    ) -> RfResult<(StudioAppHostCommandOutcome, StudioAppHostProjection)> {
        let should_refresh_local_canvas_suggestions = !matches!(
            command,
            StudioAppHostCommand::DispatchCanvasInteraction { .. }
        );
        self.app_host.execute_command(command).map(|output| {
            let projection = self.store.apply_output(&output);
            if should_refresh_local_canvas_suggestions {
                self.app_host.refresh_local_canvas_suggestions();
            }
            (output.outcome, projection)
        })
    }
}

fn dispatch_effects_from_session(
    dispatch: StudioWindowSessionDispatch,
) -> StudioAppHostDispatchEffects {
    StudioAppHostDispatchEffects {
        runtime_report: dispatch.host_output.runtime_output.report,
        entitlement_timer_effect: dispatch
            .host_output
            .window_event
            .map(entitlement_timer_effect_from_window_event),
        native_timer_transitions: dispatch.timer_driver_transitions,
        native_timer_acks: dispatch.timer_driver_acks,
    }
}

fn close_effects_from_shutdown(close: StudioAppWindowHostClose) -> StudioAppHostCloseEffects {
    StudioAppHostCloseEffects {
        window_id: close.window_id,
        cleared_entitlement_timer: close.shutdown.host_shutdown.cleared_entitlement_timer,
        retirement: close.shutdown.host_shutdown.retirement,
        next_foreground_window_id: close.next_foreground_window_id,
        native_timer_transitions: close.shutdown.timer_driver_transitions,
        native_timer_acks: close.shutdown.timer_driver_acks,
    }
}

fn registration_from_opened_window(
    opened: impl Borrow<StudioAppWindowHostOpenWindow>,
) -> StudioWindowHostRegistration {
    let opened = opened.borrow();
    StudioWindowHostRegistration {
        window_id: opened.window_id,
        role: opened.role,
        layout_slot: opened.layout_slot,
        restored_entitlement_timer: opened.restored_entitlement_timer.clone(),
        timer_driver_commands: opened.timer_driver_commands.clone(),
    }
}

fn entitlement_timer_effect_from_window_event(
    event: StudioWindowHostEvent,
) -> StudioAppHostEntitlementTimerEffect {
    match event {
        StudioWindowHostEvent::EntitlementTimerApplied {
            window_id,
            command,
            transition,
            ack,
        } => match transition {
            crate::StudioRuntimeTimerHostTransition::KeepTimer {
                slot,
                follow_up_trigger,
            } => StudioAppHostEntitlementTimerEffect::Keep {
                owner_window_id: window_id,
                effect_id: command.effect_id(),
                slot,
                follow_up_trigger,
                ack,
            },
            crate::StudioRuntimeTimerHostTransition::ArmTimer {
                slot,
                follow_up_trigger,
            } => StudioAppHostEntitlementTimerEffect::Arm {
                owner_window_id: window_id,
                effect_id: command.effect_id(),
                slot,
                follow_up_trigger,
                ack,
            },
            crate::StudioRuntimeTimerHostTransition::RearmTimer {
                previous,
                next,
                follow_up_trigger,
            } => StudioAppHostEntitlementTimerEffect::Rearm {
                owner_window_id: window_id,
                effect_id: command.effect_id(),
                previous_slot: previous,
                next_slot: next,
                follow_up_trigger,
                ack,
            },
            crate::StudioRuntimeTimerHostTransition::ClearTimer {
                previous,
                follow_up_trigger,
            } => StudioAppHostEntitlementTimerEffect::Clear {
                owner_window_id: window_id,
                effect_id: command.effect_id(),
                previous_slot: previous,
                follow_up_trigger,
                ack,
            },
            crate::StudioRuntimeTimerHostTransition::IgnoreStale {
                current,
                stale_effect_id,
            } => StudioAppHostEntitlementTimerEffect::IgnoreStale {
                owner_window_id: window_id,
                stale_effect_id,
                current_slot: current,
                ack,
            },
        },
    }
}

fn diff_app_host_snapshots(
    previous: &StudioAppHostSnapshot,
    current: &StudioAppHostSnapshot,
) -> StudioAppHostChangeSet {
    let previous_windows = snapshot_windows_by_id(&previous.windows);
    let current_windows = snapshot_windows_by_id(&current.windows);
    let mut window_ids = BTreeSet::new();
    window_ids.extend(previous_windows.keys().copied());
    window_ids.extend(current_windows.keys().copied());

    let mut window_changes = Vec::new();
    for window_id in window_ids {
        match (
            previous_windows.get(&window_id),
            current_windows.get(&window_id),
        ) {
            (None, Some(current)) => window_changes.push(StudioAppHostWindowChange::Added {
                current: current.clone(),
            }),
            (Some(previous), None) => window_changes.push(StudioAppHostWindowChange::Removed {
                previous: previous.clone(),
            }),
            (Some(previous), Some(current)) if previous != current => {
                window_changes.push(StudioAppHostWindowChange::Updated {
                    previous: previous.clone(),
                    current: current.clone(),
                })
            }
            (Some(_), Some(_)) | (None, None) => {}
        }
    }

    StudioAppHostChangeSet {
        window_changes,
        foreground_window_change: diff_window_selection(
            previous.foreground_window_id,
            current.foreground_window_id,
        ),
        entitlement_timer_owner_change: diff_window_selection(
            previous.entitlement_timer_owner_window_id,
            current.entitlement_timer_owner_window_id,
        ),
        parked_entitlement_timer_change: diff_timer_slot(
            previous.parked_entitlement_timer.as_ref(),
            current.parked_entitlement_timer.as_ref(),
        ),
    }
}

fn snapshot_windows_by_id(
    windows: &[StudioAppHostWindowSnapshot],
) -> BTreeMap<StudioWindowHostId, StudioAppHostWindowSnapshot> {
    windows
        .iter()
        .cloned()
        .map(|window| (window.window_id, window))
        .collect()
}

fn diff_window_selection(
    previous: Option<StudioWindowHostId>,
    current: Option<StudioWindowHostId>,
) -> Option<StudioAppHostWindowSelectionChange> {
    if previous == current {
        return None;
    }

    Some(StudioAppHostWindowSelectionChange { previous, current })
}

fn diff_timer_slot(
    previous: Option<&StudioRuntimeTimerHandleSlot>,
    current: Option<&StudioRuntimeTimerHandleSlot>,
) -> Option<StudioAppHostTimerSlotChange> {
    if previous == current {
        return None;
    }

    Some(StudioAppHostTimerSlotChange {
        previous: previous.cloned(),
        current: current.cloned(),
    })
}

fn entitlement_timer_state_from_snapshot(
    snapshot: &StudioAppHostSnapshot,
) -> StudioAppHostEntitlementTimerState {
    if let Some(owner_window_id) = snapshot.entitlement_timer_owner_window_id {
        let slot = snapshot
            .windows
            .iter()
            .find(|window| window.window_id == owner_window_id)
            .and_then(|window| window.entitlement_timer.clone());
        return StudioAppHostEntitlementTimerState::Owned {
            owner_window_id,
            slot,
        };
    }

    if let Some(slot) = snapshot.parked_entitlement_timer.clone() {
        return StudioAppHostEntitlementTimerState::Parked { slot };
    }

    StudioAppHostEntitlementTimerState::Idle
}

fn diff_entitlement_timer_state(
    previous: &StudioAppHostEntitlementTimerState,
    current: &StudioAppHostEntitlementTimerState,
) -> Option<StudioAppHostEntitlementTimerStateChange> {
    if previous == current {
        return None;
    }

    Some(StudioAppHostEntitlementTimerStateChange {
        previous: previous.clone(),
        current: current.clone(),
    })
}

fn map_command(command: StudioAppHostCommand) -> StudioAppWindowHostCommand {
    match command {
        StudioAppHostCommand::OpenWindow => StudioAppWindowHostCommand::OpenWindow,
        StudioAppHostCommand::DispatchWindowTrigger { window_id, trigger } => {
            StudioAppWindowHostCommand::DispatchTrigger { window_id, trigger }
        }
        StudioAppHostCommand::DispatchCanvasInteraction { action } => {
            StudioAppWindowHostCommand::DispatchCanvasInteraction { action }
        }
        StudioAppHostCommand::DispatchUiAction { action } => {
            StudioAppWindowHostCommand::DispatchUiAction {
                action: map_ui_action(action),
            }
        }
        StudioAppHostCommand::DispatchWindowRunPanelRecoveryAction { window_id } => {
            StudioAppWindowHostCommand::DispatchRunPanelRecoveryAction { window_id }
        }
        StudioAppHostCommand::DispatchForegroundRunPanelRecoveryAction => {
            StudioAppWindowHostCommand::DispatchForegroundRunPanelRecoveryAction
        }
        StudioAppHostCommand::DispatchForegroundEntitlementPrimaryAction => {
            StudioAppWindowHostCommand::DispatchForegroundEntitlementPrimaryAction
        }
        StudioAppHostCommand::DispatchForegroundEntitlementAction { action_id } => {
            StudioAppWindowHostCommand::DispatchForegroundEntitlementAction { action_id }
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

fn map_ui_action(action: StudioAppHostUiAction) -> StudioAppWindowHostUiAction {
    match action {
        StudioAppHostUiAction::RunManualWorkspace => {
            StudioAppWindowHostUiAction::RunManualWorkspace
        }
        StudioAppHostUiAction::ResumeWorkspace => StudioAppWindowHostUiAction::ResumeWorkspace,
        StudioAppHostUiAction::HoldWorkspace => StudioAppWindowHostUiAction::HoldWorkspace,
        StudioAppHostUiAction::ActivateWorkspace => StudioAppWindowHostUiAction::ActivateWorkspace,
        StudioAppHostUiAction::RecoverRunPanelFailure => {
            StudioAppWindowHostUiAction::RecoverRunPanelFailure
        }
    }
}

fn ui_action_state_from_window_host(
    state: StudioAppWindowHostUiActionState,
) -> StudioAppHostUiActionState {
    StudioAppHostUiActionState {
        action: match state.action {
            StudioAppWindowHostUiAction::RunManualWorkspace => {
                StudioAppHostUiAction::RunManualWorkspace
            }
            StudioAppWindowHostUiAction::ResumeWorkspace => StudioAppHostUiAction::ResumeWorkspace,
            StudioAppWindowHostUiAction::HoldWorkspace => StudioAppHostUiAction::HoldWorkspace,
            StudioAppWindowHostUiAction::ActivateWorkspace => {
                StudioAppHostUiAction::ActivateWorkspace
            }
            StudioAppWindowHostUiAction::RecoverRunPanelFailure => {
                StudioAppHostUiAction::RecoverRunPanelFailure
            }
        },
        availability: match state.availability {
            StudioAppWindowHostUiActionAvailability::Enabled { target_window_id } => {
                StudioAppHostUiActionAvailability::Enabled { target_window_id }
            }
            StudioAppWindowHostUiActionAvailability::Disabled {
                reason,
                target_window_id,
            } => StudioAppHostUiActionAvailability::Disabled {
                reason: match reason {
                    StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow => {
                        StudioAppHostUiActionDisabledReason::NoRegisteredWindow
                    }
                    StudioAppWindowHostUiActionDisabledReason::RunManualUnavailable => {
                        StudioAppHostUiActionDisabledReason::RunManualUnavailable
                    }
                    StudioAppWindowHostUiActionDisabledReason::ResumeUnavailable => {
                        StudioAppHostUiActionDisabledReason::ResumeUnavailable
                    }
                    StudioAppWindowHostUiActionDisabledReason::HoldUnavailable => {
                        StudioAppHostUiActionDisabledReason::HoldUnavailable
                    }
                    StudioAppWindowHostUiActionDisabledReason::ActivateUnavailable => {
                        StudioAppHostUiActionDisabledReason::ActivateUnavailable
                    }
                    StudioAppWindowHostUiActionDisabledReason::NoRunPanelRecovery => {
                        StudioAppHostUiActionDisabledReason::NoRunPanelRecovery
                    }
                },
                target_window_id,
            },
        },
    }
}

fn ui_command_model_from_states(
    states: &[StudioAppHostUiActionState],
) -> StudioAppHostUiCommandModel {
    let mut actions: Vec<_> = states
        .iter()
        .cloned()
        .map(ui_action_model_from_state)
        .collect();
    actions.extend(placeholder_ui_command_models());
    actions.sort_by_key(|action| (ui_command_group_sort_key(action.group), action.sort_order));

    StudioAppHostUiCommandModel { actions }
}

fn ui_action_model_from_state(state: StudioAppHostUiActionState) -> StudioAppHostUiActionModel {
    let (command_id, group, sort_order, label) = match state.action {
        StudioAppHostUiAction::RunManualWorkspace => (
            "run_panel.run_manual",
            StudioAppHostUiCommandGroup::RunPanel,
            100,
            "Run workspace",
        ),
        StudioAppHostUiAction::ResumeWorkspace => (
            "run_panel.resume_workspace",
            StudioAppHostUiCommandGroup::RunPanel,
            110,
            "Resume workspace",
        ),
        StudioAppHostUiAction::HoldWorkspace => (
            "run_panel.set_hold",
            StudioAppHostUiCommandGroup::RunPanel,
            120,
            "Hold workspace",
        ),
        StudioAppHostUiAction::ActivateWorkspace => (
            "run_panel.set_active",
            StudioAppHostUiCommandGroup::RunPanel,
            130,
            "Activate workspace",
        ),
        StudioAppHostUiAction::RecoverRunPanelFailure => (
            "run_panel.recover_failure",
            StudioAppHostUiCommandGroup::Recovery,
            200,
            "Recover run panel failure",
        ),
    };
    let (enabled, detail, target_window_id) = match state.availability {
        StudioAppHostUiActionAvailability::Enabled { target_window_id } => {
            let detail = match state.action {
                StudioAppHostUiAction::RunManualWorkspace => {
                    "Dispatch the current manual run action in the target window"
                }
                StudioAppHostUiAction::ResumeWorkspace => {
                    "Dispatch the current resume action in the target window"
                }
                StudioAppHostUiAction::HoldWorkspace => {
                    "Dispatch the current hold action in the target window"
                }
                StudioAppHostUiAction::ActivateWorkspace => {
                    "Dispatch the current activate action in the target window"
                }
                StudioAppHostUiAction::RecoverRunPanelFailure => {
                    "Apply the current run panel recovery action in the target window"
                }
            };

            (true, detail, Some(target_window_id))
        }
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
            target_window_id,
        } => {
            let detail = match state.action {
                StudioAppHostUiAction::RunManualWorkspace => {
                    "Open a studio window before running the workspace"
                }
                StudioAppHostUiAction::ResumeWorkspace => {
                    "Open a studio window before resuming the workspace"
                }
                StudioAppHostUiAction::HoldWorkspace => {
                    "Open a studio window before holding the workspace"
                }
                StudioAppHostUiAction::ActivateWorkspace => {
                    "Open a studio window before activating the workspace"
                }
                StudioAppHostUiAction::RecoverRunPanelFailure => {
                    "Open a studio window before requesting run panel recovery"
                }
            };

            (false, detail, target_window_id)
        }
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::RunManualUnavailable,
            target_window_id,
        } => (
            false,
            "Manual run is currently unavailable in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::ResumeUnavailable,
            target_window_id,
        } => (
            false,
            "Resume is currently unavailable in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::HoldUnavailable,
            target_window_id,
        } => (
            false,
            "Hold is currently unavailable in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::ActivateUnavailable,
            target_window_id,
        } => (
            false,
            "Activate is currently unavailable in the target window",
            target_window_id,
        ),
        StudioAppHostUiActionAvailability::Disabled {
            reason: StudioAppHostUiActionDisabledReason::NoRunPanelRecovery,
            target_window_id,
        } => (
            false,
            "No run panel recovery action is currently available in the target window",
            target_window_id,
        ),
    };

    StudioAppHostUiActionModel {
        action: Some(state.action),
        command_id,
        group,
        sort_order,
        label,
        enabled,
        detail,
        target_window_id,
    }
}

fn placeholder_ui_command_models() -> Vec<StudioAppHostUiActionModel> {
    Vec::new()
}

fn ui_command_group_sort_key(group: StudioAppHostUiCommandGroup) -> u16 {
    match group {
        StudioAppHostUiCommandGroup::RunPanel => 100,
        StudioAppHostUiCommandGroup::Recovery => 200,
    }
}

fn map_outcome(outcome: StudioAppWindowHostCommandOutcome) -> StudioAppHostCommandOutcome {
    match outcome {
        StudioAppWindowHostCommandOutcome::WindowOpened(opened) => {
            StudioAppHostCommandOutcome::WindowOpened(opened)
        }
        StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch)
        }
        StudioAppWindowHostCommandOutcome::CanvasInteracted(result) => {
            StudioAppHostCommandOutcome::CanvasInteracted(result)
        }
        StudioAppWindowHostCommandOutcome::WindowClosed(close) => {
            StudioAppHostCommandOutcome::WindowClosed(close)
        }
        StudioAppWindowHostCommandOutcome::IgnoredUiAction => {
            StudioAppHostCommandOutcome::IgnoredUiAction
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
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        StudioAppHost, StudioAppHostCommand, StudioAppHostCommandOutcome, StudioAppHostController,
        StudioAppHostEntitlementTimerEffect, StudioAppHostEntitlementTimerState,
        StudioAppHostStore, StudioAppHostTimerSlotChange, StudioAppHostUiAction,
        StudioAppHostUiActionAvailability, StudioAppHostUiActionDisabledReason,
        StudioAppHostUiActionModel, StudioAppHostUiActionState,
        StudioAppHostUiCommandDispatchResult, StudioAppHostUiCommandGroup,
        StudioAppHostWindowChange, StudioAppHostWindowSelectionChange,
        StudioAppWindowHostGlobalEvent, StudioCanvasInteractionAction,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger, StudioWindowHostRetirement,
        StudioWindowHostRole,
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
        let project_path =
            std::env::temp_dir().join(format!("radishflow-app-host-recovery-{unique}.rfproj.json"));
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

    fn flash_drum_local_rules_synced_config() -> (crate::StudioRuntimeConfig, PathBuf) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-app-host-local-rules-{unique}.rfproj.json"
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
                entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
                entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
                ..crate::StudioRuntimeConfig::default()
            },
            project_path,
        )
    }

    #[test]
    fn app_host_returns_snapshot_with_window_open_and_focus_updates() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");

        let first = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open");
        let first_window = match &first.outcome {
            StudioAppHostCommandOutcome::WindowOpened(opened) => {
                super::registration_from_opened_window(opened)
            }
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
                layout_slot: 1,
                is_foreground: true,
                entitlement_timer: None,
            }]
        );
        assert_eq!(
            first.snapshot.ui_actions,
            vec![
                StudioAppHostUiActionState {
                    action: StudioAppHostUiAction::RunManualWorkspace,
                    availability: StudioAppHostUiActionAvailability::Enabled {
                        target_window_id: first_window.window_id,
                    },
                },
                StudioAppHostUiActionState {
                    action: StudioAppHostUiAction::ResumeWorkspace,
                    availability: StudioAppHostUiActionAvailability::Enabled {
                        target_window_id: first_window.window_id,
                    },
                },
                StudioAppHostUiActionState {
                    action: StudioAppHostUiAction::HoldWorkspace,
                    availability: StudioAppHostUiActionAvailability::Disabled {
                        reason: StudioAppHostUiActionDisabledReason::HoldUnavailable,
                        target_window_id: Some(first_window.window_id),
                    },
                },
                StudioAppHostUiActionState {
                    action: StudioAppHostUiAction::ActivateWorkspace,
                    availability: StudioAppHostUiActionAvailability::Enabled {
                        target_window_id: first_window.window_id,
                    },
                },
                StudioAppHostUiActionState {
                    action: StudioAppHostUiAction::RecoverRunPanelFailure,
                    availability: StudioAppHostUiActionAvailability::Disabled {
                        reason: StudioAppHostUiActionDisabledReason::NoRunPanelRecovery,
                        target_window_id: Some(first_window.window_id),
                    },
                },
            ]
        );
        assert_eq!(
            first.changes.window_changes,
            vec![StudioAppHostWindowChange::Added {
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: first_window.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    layout_slot: 1,
                    is_foreground: true,
                    entitlement_timer: None,
                },
            }]
        );
        assert_eq!(
            first.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: None,
                current: Some(first_window.window_id),
            })
        );
        assert_eq!(
            first.changes.entitlement_timer_owner_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: None,
                current: Some(first_window.window_id),
            })
        );
        assert_eq!(first.changes.parked_entitlement_timer_change, None);

        let second = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected second window open");
        let second_window = match &second.outcome {
            StudioAppHostCommandOutcome::WindowOpened(opened) => {
                super::registration_from_opened_window(opened)
            }
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
        assert_eq!(
            second.changes.window_changes,
            vec![StudioAppHostWindowChange::Added {
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: second_window.window_id,
                    role: StudioWindowHostRole::Observer,
                    layout_slot: 1,
                    is_foreground: false,
                    entitlement_timer: None,
                },
            }]
        );
        assert_eq!(second.changes.foreground_window_change, None);
        assert_eq!(second.changes.entitlement_timer_owner_change, None);

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
                layout_slot: 1,
                is_foreground: true,
                entitlement_timer: None,
            }
        );
        let focused_owner_timer = focused.snapshot.windows[0].entitlement_timer.clone();
        assert_eq!(
            focused.changes.window_changes,
            vec![
                StudioAppHostWindowChange::Updated {
                    previous: crate::StudioAppHostWindowSnapshot {
                        window_id: first_window.window_id,
                        role: StudioWindowHostRole::EntitlementTimerOwner,
                        layout_slot: 1,
                        is_foreground: true,
                        entitlement_timer: None,
                    },
                    current: crate::StudioAppHostWindowSnapshot {
                        window_id: first_window.window_id,
                        role: StudioWindowHostRole::EntitlementTimerOwner,
                        layout_slot: 1,
                        is_foreground: false,
                        entitlement_timer: focused_owner_timer,
                    },
                },
                StudioAppHostWindowChange::Updated {
                    previous: crate::StudioAppHostWindowSnapshot {
                        window_id: second_window.window_id,
                        role: StudioWindowHostRole::Observer,
                        layout_slot: 1,
                        is_foreground: false,
                        entitlement_timer: None,
                    },
                    current: crate::StudioAppHostWindowSnapshot {
                        window_id: second_window.window_id,
                        role: StudioWindowHostRole::Observer,
                        layout_slot: 1,
                        is_foreground: true,
                        entitlement_timer: None,
                    },
                },
            ]
        );
        assert_eq!(
            focused.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first_window.window_id),
                current: Some(second_window.window_id),
            })
        );
        assert_eq!(focused.changes.entitlement_timer_owner_change, None);
    }

    #[test]
    fn app_host_snapshot_tracks_parked_timer_across_last_close_and_reopen() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
        let first = match app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open")
            .outcome
        {
            StudioAppHostCommandOutcome::WindowOpened(opened) => {
                super::registration_from_opened_window(opened)
            }
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
        let triggered_timer = triggered.snapshot.windows[0]
            .entitlement_timer
            .clone()
            .expect("expected timer slot");
        assert_eq!(
            triggered.changes.window_changes,
            vec![StudioAppHostWindowChange::Updated {
                previous: crate::StudioAppHostWindowSnapshot {
                    window_id: first.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    layout_slot: 1,
                    is_foreground: true,
                    entitlement_timer: None,
                },
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: first.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    layout_slot: 1,
                    is_foreground: true,
                    entitlement_timer: Some(triggered_timer.clone()),
                },
            }]
        );
        assert_eq!(triggered.changes.foreground_window_change, None);
        assert_eq!(triggered.changes.entitlement_timer_owner_change, None);

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
        let parked_timer = closed
            .snapshot
            .parked_entitlement_timer
            .clone()
            .expect("expected parked timer");
        assert_eq!(
            closed.changes.window_changes,
            vec![StudioAppHostWindowChange::Removed {
                previous: crate::StudioAppHostWindowSnapshot {
                    window_id: first.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    layout_slot: 1,
                    is_foreground: true,
                    entitlement_timer: Some(triggered_timer.clone()),
                },
            }]
        );
        assert_eq!(
            closed.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first.window_id),
                current: None,
            })
        );
        assert_eq!(
            closed.changes.entitlement_timer_owner_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first.window_id),
                current: None,
            })
        );
        assert_eq!(
            closed.changes.parked_entitlement_timer_change,
            Some(StudioAppHostTimerSlotChange {
                previous: None,
                current: Some(parked_timer.clone()),
            })
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
        let restored_timer = reopened.snapshot.windows[0]
            .entitlement_timer
            .clone()
            .expect("expected restored timer");
        assert!(matches!(
            reopened.outcome,
            StudioAppHostCommandOutcome::WindowOpened(_)
        ));
        assert_eq!(
            reopened.changes.window_changes,
            vec![StudioAppHostWindowChange::Added {
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: reopened.snapshot.windows[0].window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    layout_slot: 1,
                    is_foreground: true,
                    entitlement_timer: Some(restored_timer.clone()),
                },
            }]
        );
        assert_eq!(
            reopened.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: None,
                current: Some(reopened.snapshot.windows[0].window_id),
            })
        );
        assert_eq!(
            reopened.changes.entitlement_timer_owner_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: None,
                current: Some(reopened.snapshot.windows[0].window_id),
            })
        );
        assert_eq!(
            reopened.changes.parked_entitlement_timer_change,
            Some(StudioAppHostTimerSlotChange {
                previous: Some(parked_timer),
                current: None,
            })
        );
    }

    #[test]
    fn app_host_change_set_captures_owner_transfer_on_close() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
        let first = match app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open")
            .outcome
        {
            StudioAppHostCommandOutcome::WindowOpened(opened) => {
                super::registration_from_opened_window(opened)
            }
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let second = match app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected second window open")
            .outcome
        {
            StudioAppHostCommandOutcome::WindowOpened(opened) => {
                super::registration_from_opened_window(opened)
            }
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let _ = app_host
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
                window_id: first.window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer trigger");

        let closed = app_host
            .execute_command(StudioAppHostCommand::CloseWindow {
                window_id: first.window_id,
            })
            .expect("expected owner close");
        let transferred_timer = closed.snapshot.windows[0]
            .entitlement_timer
            .clone()
            .expect("expected transferred timer");

        assert_eq!(
            closed.changes.window_changes,
            vec![
                StudioAppHostWindowChange::Removed {
                    previous: crate::StudioAppHostWindowSnapshot {
                        window_id: first.window_id,
                        role: StudioWindowHostRole::EntitlementTimerOwner,
                        layout_slot: 1,
                        is_foreground: true,
                        entitlement_timer: Some(transferred_timer.clone()),
                    },
                },
                StudioAppHostWindowChange::Updated {
                    previous: crate::StudioAppHostWindowSnapshot {
                        window_id: second.window_id,
                        role: StudioWindowHostRole::Observer,
                        layout_slot: 1,
                        is_foreground: false,
                        entitlement_timer: None,
                    },
                    current: crate::StudioAppHostWindowSnapshot {
                        window_id: second.window_id,
                        role: StudioWindowHostRole::EntitlementTimerOwner,
                        layout_slot: 1,
                        is_foreground: true,
                        entitlement_timer: Some(transferred_timer),
                    },
                },
            ]
        );
        assert_eq!(
            closed.changes.foreground_window_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first.window_id),
                current: Some(second.window_id),
            })
        );
        assert_eq!(
            closed.changes.entitlement_timer_owner_change,
            Some(StudioAppHostWindowSelectionChange {
                previous: Some(first.window_id),
                current: Some(second.window_id),
            })
        );
        assert_eq!(closed.changes.parked_entitlement_timer_change, None);
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
        assert!(output.changes.window_changes.is_empty());
        assert_eq!(output.changes.foreground_window_change, None);
        assert_eq!(output.changes.entitlement_timer_owner_change, None);
        assert_eq!(output.changes.parked_entitlement_timer_change, None);
    }

    #[test]
    fn app_host_store_projects_output_into_single_state_boundary() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
        let mut store = StudioAppHostStore::from_snapshot(&app_host.snapshot());

        let first_open = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open");
        let first_projection = store.apply_output(&first_open);
        let first_window = first_projection
            .added_windows
            .first()
            .expect("expected first added window");

        assert_eq!(
            first_projection.state.registered_windows,
            vec![first_window.window_id]
        );
        assert_eq!(
            first_projection.state.foreground_window_id,
            Some(first_window.window_id)
        );
        assert_eq!(
            first_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: first_window.window_id,
                slot: None,
            }
        );
        assert_eq!(first_projection.removed_window_ids, Vec::<u64>::new());
        assert!(first_projection.updated_windows.is_empty());

        let second_open = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected second window open");
        let second_projection = store.apply_output(&second_open);
        let second_window = second_projection
            .added_windows
            .first()
            .expect("expected second added window");

        assert_eq!(
            second_projection.state.registered_windows,
            vec![first_window.window_id, second_window.window_id]
        );
        assert_eq!(
            second_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: first_window.window_id,
                slot: None,
            }
        );

        let focused = app_host
            .execute_command(StudioAppHostCommand::FocusWindow {
                window_id: second_window.window_id,
            })
            .expect("expected focus command");
        let focused_projection = store.apply_output(&focused);

        assert_eq!(
            focused_projection.state.foreground_window_id,
            Some(second_window.window_id)
        );
        assert_eq!(focused_projection.added_windows, Vec::new());
        assert_eq!(focused_projection.removed_window_ids, Vec::<u64>::new());
        assert_eq!(focused_projection.updated_windows.len(), 2);
        assert_eq!(
            focused_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: first_window.window_id,
                slot: focused_projection
                    .state
                    .window(first_window.window_id)
                    .and_then(|window| window.entitlement_timer.clone()),
            }
        );
        assert!(focused_projection.entitlement_timer_change.is_some());
    }

    #[test]
    fn app_host_store_collapses_owner_and_parked_timer_semantics() {
        let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
        let mut store = StudioAppHostStore::from_snapshot(&app_host.snapshot());

        let opened = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected window open");
        let opened_projection = store.apply_output(&opened);
        let window_id = opened_projection
            .added_windows
            .first()
            .expect("expected opened window")
            .window_id;

        let triggered = app_host
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
                window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer trigger");
        let triggered_projection = store.apply_output(&triggered);
        let owned_slot = triggered_projection
            .state
            .window(window_id)
            .and_then(|window| window.entitlement_timer.clone())
            .expect("expected owned timer slot");

        assert_eq!(
            triggered_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: window_id,
                slot: Some(owned_slot.clone()),
            }
        );

        let closed = app_host
            .execute_command(StudioAppHostCommand::CloseWindow { window_id })
            .expect("expected close command");
        let closed_projection = store.apply_output(&closed);

        assert!(closed_projection.state.windows.is_empty());
        assert_eq!(
            closed_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Parked {
                slot: owned_slot.clone(),
            }
        );
        assert_eq!(closed_projection.removed_window_ids, vec![window_id]);

        let reopened = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected reopen");
        let reopened_projection = store.apply_output(&reopened);
        let reopened_window_id = reopened_projection
            .added_windows
            .first()
            .expect("expected reopened window")
            .window_id;

        assert_eq!(
            reopened_projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: reopened_window_id,
                slot: reopened_projection
                    .state
                    .window(reopened_window_id)
                    .and_then(|window| window.entitlement_timer.clone()),
            }
        );
        assert!(
            reopened_projection
                .state
                .parked_entitlement_timer()
                .is_none()
        );
    }

    #[test]
    fn app_host_controller_advances_state_through_typed_command_methods() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let opened = controller.open_window().expect("expected window open");
        assert_eq!(
            opened.projection.state.registered_windows,
            vec![opened.registration.window_id]
        );
        assert_eq!(
            controller.state().foreground_window_id,
            Some(opened.registration.window_id)
        );

        let dispatched = controller
            .dispatch_window_trigger(
                opened.registration.window_id,
                StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected trigger dispatch");
        let owner_slot = dispatched
            .projection
            .state
            .window(opened.registration.window_id)
            .and_then(|window| window.entitlement_timer.clone())
            .expect("expected timer slot");

        assert_eq!(dispatched.target_window_id, opened.registration.window_id);
        assert!(matches!(
            dispatched.effects.entitlement_timer_effect,
            Some(StudioAppHostEntitlementTimerEffect::Rearm {
                owner_window_id,
                effect_id: 1,
                ..
            }) if owner_window_id == opened.registration.window_id
        ));
        assert!(matches!(
            dispatched.effects.native_timer_transitions.as_slice(),
            [crate::StudioWindowTimerDriverTransition::RearmNativeTimer { window_id, .. }]
            if *window_id == opened.registration.window_id
        ));
        assert_eq!(dispatched.effects.native_timer_acks.len(), 1);
        assert_eq!(
            controller.state().entitlement_timer,
            StudioAppHostEntitlementTimerState::Owned {
                owner_window_id: opened.registration.window_id,
                slot: Some(owner_slot),
            }
        );
    }

    #[test]
    fn app_host_controller_dispatches_run_panel_recovery_through_typed_method() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");

        let run = controller
            .dispatch_window_trigger(
                opened.registration.window_id,
                StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");
        match &run.effects.runtime_report.dispatch {
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

        let recovery = controller
            .dispatch_window_run_panel_recovery_action(opened.registration.window_id)
            .expect("expected recovery dispatch");

        assert_eq!(recovery.target_window_id, opened.registration.window_id);
        match &recovery.effects.runtime_report.dispatch {
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
        assert_eq!(
            controller.state().foreground_window_id,
            Some(opened.registration.window_id)
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_controller_dispatches_foreground_run_panel_recovery_action() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let first = controller
            .open_window()
            .expect("expected first window open");
        let second = controller
            .open_window()
            .expect("expected second window open");
        let _ = controller
            .focus_window(second.registration.window_id)
            .expect("expected second window focus");

        let _ = controller
            .dispatch_window_trigger(
                second.registration.window_id,
                StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");

        let recovery = controller
            .dispatch_foreground_run_panel_recovery_action()
            .expect("expected foreground recovery call")
            .expect("expected foreground recovery dispatch");

        assert_eq!(recovery.target_window_id, second.registration.window_id);
        assert_ne!(recovery.target_window_id, first.registration.window_id);
        match &recovery.effects.runtime_report.dispatch {
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
        assert_eq!(
            controller.state().foreground_window_id,
            Some(second.registration.window_id)
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_controller_dispatches_foreground_entitlement_primary_action() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
        let first = controller
            .open_window()
            .expect("expected first window open");
        let second = controller
            .open_window()
            .expect("expected second window open");
        let _ = controller
            .focus_window(second.registration.window_id)
            .expect("expected second window focus");

        let dispatch = controller
            .dispatch_foreground_entitlement_primary_action()
            .expect("expected foreground entitlement primary call")
            .expect("expected foreground entitlement primary dispatch");

        assert_eq!(dispatch.target_window_id, second.registration.window_id);
        assert_ne!(dispatch.target_window_id, first.registration.window_id);
        assert_eq!(
            controller.state().foreground_window_id,
            Some(second.registration.window_id)
        );
        match &dispatch.effects.runtime_report.dispatch {
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
    fn app_host_controller_dispatches_foreground_entitlement_action() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
        let first = controller
            .open_window()
            .expect("expected first window open");
        let second = controller
            .open_window()
            .expect("expected second window open");
        let _ = controller
            .focus_window(second.registration.window_id)
            .expect("expected second window focus");

        let dispatch = controller
            .dispatch_foreground_entitlement_action(EntitlementActionId::SyncEntitlement)
            .expect("expected foreground entitlement action call")
            .expect("expected foreground entitlement action dispatch");

        assert_eq!(dispatch.target_window_id, second.registration.window_id);
        assert_ne!(dispatch.target_window_id, first.registration.window_id);
        assert_eq!(
            controller.state().foreground_window_id,
            Some(second.registration.window_id)
        );
        match &dispatch.effects.runtime_report.dispatch {
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
    fn app_host_controller_dispatches_run_panel_recovery_via_ui_action() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let first = controller
            .open_window()
            .expect("expected first window open");
        let second = controller
            .open_window()
            .expect("expected second window open");
        let _ = controller
            .focus_window(second.registration.window_id)
            .expect("expected second window focus");
        let _ = controller
            .dispatch_window_trigger(
                second.registration.window_id,
                StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");

        let recovery = controller
            .dispatch_ui_action(StudioAppHostUiAction::RecoverRunPanelFailure)
            .expect("expected ui action call")
            .expect("expected ui action dispatch");

        assert_eq!(recovery.target_window_id, second.registration.window_id);
        assert_ne!(recovery.target_window_id, first.registration.window_id);
        match &recovery.effects.runtime_report.dispatch {
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
        assert_eq!(
            controller.state().foreground_window_id,
            Some(second.registration.window_id)
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_controller_dispatches_run_manual_via_ui_action() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");

        let run = controller
            .dispatch_ui_action(StudioAppHostUiAction::RunManualWorkspace)
            .expect("expected ui action call")
            .expect("expected ui action dispatch");

        assert_eq!(run.target_window_id, opened.registration.window_id);
        match &run.effects.runtime_report.dispatch {
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
    fn app_host_controller_dispatches_resume_via_ui_action() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");

        let resume = controller
            .dispatch_ui_action(StudioAppHostUiAction::ResumeWorkspace)
            .expect("expected ui action call")
            .expect("expected ui action dispatch");

        assert_eq!(resume.target_window_id, opened.registration.window_id);
        match &resume.effects.runtime_report.dispatch {
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
    fn app_host_controller_dispatches_activate_via_ui_action() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");

        let activate = controller
            .dispatch_ui_action(StudioAppHostUiAction::ActivateWorkspace)
            .expect("expected ui action call")
            .expect("expected ui action dispatch");

        assert_eq!(activate.target_window_id, opened.registration.window_id);
        match &activate.effects.runtime_report.dispatch {
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
    fn app_host_controller_dispatches_hold_via_ui_action_after_activation() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");
        let _ = controller
            .dispatch_ui_action(StudioAppHostUiAction::ActivateWorkspace)
            .expect("expected ui action call")
            .expect("expected ui action dispatch");

        let hold = controller
            .dispatch_ui_action(StudioAppHostUiAction::HoldWorkspace)
            .expect("expected ui action call")
            .expect("expected ui action dispatch");

        assert_eq!(hold.target_window_id, opened.registration.window_id);
        match &hold.effects.runtime_report.dispatch {
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
    fn app_host_snapshot_tracks_ui_action_availability_for_recovery() {
        let (config, project_path) = solver_failure_config();
        let mut app_host = StudioAppHost::new(&config).expect("expected app host");
        let first = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected first window open");
        let first_window = match &first.outcome {
            StudioAppHostCommandOutcome::WindowOpened(opened) => {
                super::registration_from_opened_window(opened)
            }
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let second = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected second window open");
        let second_window = match &second.outcome {
            StudioAppHostCommandOutcome::WindowOpened(opened) => {
                super::registration_from_opened_window(opened)
            }
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let focused = app_host
            .execute_command(StudioAppHostCommand::FocusWindow {
                window_id: second_window.window_id,
            })
            .expect("expected focus command");
        assert_eq!(
            focused
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::RunManualWorkspace)
                .expect("expected run manual ui action state"),
            &StudioAppHostUiActionState {
                action: StudioAppHostUiAction::RunManualWorkspace,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: second_window.window_id,
                },
            }
        );
        assert_eq!(
            focused
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::ResumeWorkspace)
                .expect("expected resume ui action state"),
            &StudioAppHostUiActionState {
                action: StudioAppHostUiAction::ResumeWorkspace,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: second_window.window_id,
                },
            }
        );
        assert_eq!(
            focused
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::HoldWorkspace)
                .expect("expected hold ui action state"),
            &StudioAppHostUiActionState {
                action: StudioAppHostUiAction::HoldWorkspace,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::HoldUnavailable,
                    target_window_id: Some(second_window.window_id),
                },
            }
        );
        assert_eq!(
            focused
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::ActivateWorkspace)
                .expect("expected activate ui action state"),
            &StudioAppHostUiActionState {
                action: StudioAppHostUiAction::ActivateWorkspace,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: second_window.window_id,
                },
            }
        );
        assert_eq!(
            focused
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::RecoverRunPanelFailure)
                .expect("expected recovery ui action state"),
            &StudioAppHostUiActionState {
                action: StudioAppHostUiAction::RecoverRunPanelFailure,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::NoRunPanelRecovery,
                    target_window_id: Some(second_window.window_id),
                },
            }
        );

        let failed_run = app_host
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
                window_id: second_window.window_id,
                trigger: StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            })
            .expect("expected failed run dispatch");
        assert_eq!(
            failed_run
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::RecoverRunPanelFailure)
                .expect("expected recovery ui action state"),
            &StudioAppHostUiActionState {
                action: StudioAppHostUiAction::RecoverRunPanelFailure,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: second_window.window_id,
                },
            }
        );
        assert_eq!(
            failed_run
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::ResumeWorkspace)
                .expect("expected resume ui action state"),
            &StudioAppHostUiActionState {
                action: StudioAppHostUiAction::ResumeWorkspace,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::ResumeUnavailable,
                    target_window_id: Some(second_window.window_id),
                },
            }
        );
        assert_eq!(
            failed_run
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::HoldWorkspace)
                .expect("expected hold ui action state"),
            &StudioAppHostUiActionState {
                action: StudioAppHostUiAction::HoldWorkspace,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::HoldUnavailable,
                    target_window_id: Some(second_window.window_id),
                },
            }
        );
        assert_eq!(
            failed_run
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::ActivateWorkspace)
                .expect("expected activate ui action state"),
            &StudioAppHostUiActionState {
                action: StudioAppHostUiAction::ActivateWorkspace,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: second_window.window_id,
                },
            }
        );
        assert_ne!(
            failed_run
                .snapshot
                .ui_actions
                .iter()
                .find(|state| state.action == StudioAppHostUiAction::RecoverRunPanelFailure)
                .and_then(|state| state.target_window_id()),
            Some(first_window.window_id)
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_state_derives_ui_command_model_from_availability() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let disabled = controller.state().ui_command_model();
        assert_eq!(
            disabled.action(StudioAppHostUiAction::RunManualWorkspace),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::RunManualWorkspace),
                command_id: "run_panel.run_manual",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 100,
                label: "Run workspace",
                enabled: false,
                detail: "Open a studio window before running the workspace",
                target_window_id: None,
            })
        );
        assert_eq!(
            disabled.action(StudioAppHostUiAction::ResumeWorkspace),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::ResumeWorkspace),
                command_id: "run_panel.resume_workspace",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 110,
                label: "Resume workspace",
                enabled: false,
                detail: "Open a studio window before resuming the workspace",
                target_window_id: None,
            })
        );
        assert_eq!(
            disabled.action(StudioAppHostUiAction::HoldWorkspace),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::HoldWorkspace),
                command_id: "run_panel.set_hold",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 120,
                label: "Hold workspace",
                enabled: false,
                detail: "Open a studio window before holding the workspace",
                target_window_id: None,
            })
        );
        assert_eq!(
            disabled.action(StudioAppHostUiAction::ActivateWorkspace),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::ActivateWorkspace),
                command_id: "run_panel.set_active",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 130,
                label: "Activate workspace",
                enabled: false,
                detail: "Open a studio window before activating the workspace",
                target_window_id: None,
            })
        );
        assert_eq!(
            disabled.action(StudioAppHostUiAction::RecoverRunPanelFailure),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::RecoverRunPanelFailure),
                command_id: "run_panel.recover_failure",
                group: StudioAppHostUiCommandGroup::Recovery,
                sort_order: 200,
                label: "Recover run panel failure",
                enabled: false,
                detail: "Open a studio window before requesting run panel recovery",
                target_window_id: None,
            })
        );
        assert_eq!(
            disabled.command("run_panel.recover_failure"),
            disabled.action(StudioAppHostUiAction::RecoverRunPanelFailure)
        );

        let opened = controller.open_window().expect("expected window open");
        let no_recovery = opened.projection.state.ui_command_model();
        assert_eq!(
            no_recovery.actions[0],
            StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::RunManualWorkspace),
                command_id: "run_panel.run_manual",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 100,
                label: "Run workspace",
                enabled: true,
                detail: "Dispatch the current manual run action in the target window",
                target_window_id: Some(opened.registration.window_id),
            }
        );
        assert_eq!(
            no_recovery.actions[1],
            StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::ResumeWorkspace),
                command_id: "run_panel.resume_workspace",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 110,
                label: "Resume workspace",
                enabled: true,
                detail: "Dispatch the current resume action in the target window",
                target_window_id: Some(opened.registration.window_id),
            }
        );
        assert_eq!(
            no_recovery.actions[2],
            StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::HoldWorkspace),
                command_id: "run_panel.set_hold",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 120,
                label: "Hold workspace",
                enabled: false,
                detail: "Hold is currently unavailable in the target window",
                target_window_id: Some(opened.registration.window_id),
            }
        );
        assert_eq!(
            no_recovery.actions[3],
            StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::ActivateWorkspace),
                command_id: "run_panel.set_active",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 130,
                label: "Activate workspace",
                enabled: true,
                detail: "Dispatch the current activate action in the target window",
                target_window_id: Some(opened.registration.window_id),
            }
        );
        assert_eq!(
            no_recovery.action(StudioAppHostUiAction::RecoverRunPanelFailure),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::RecoverRunPanelFailure),
                command_id: "run_panel.recover_failure",
                group: StudioAppHostUiCommandGroup::Recovery,
                sort_order: 200,
                label: "Recover run panel failure",
                enabled: false,
                detail: "No run panel recovery action is currently available in the target window",
                target_window_id: Some(opened.registration.window_id),
            })
        );
        assert_eq!(
            no_recovery.command("run_panel.run_manual"),
            no_recovery.action(StudioAppHostUiAction::RunManualWorkspace)
        );
        assert_eq!(
            no_recovery.command("run_panel.resume_workspace"),
            no_recovery.action(StudioAppHostUiAction::ResumeWorkspace)
        );
        assert_eq!(
            no_recovery.command("run_panel.set_hold"),
            no_recovery.action(StudioAppHostUiAction::HoldWorkspace)
        );
        assert_eq!(
            no_recovery.command("run_panel.set_active"),
            no_recovery.action(StudioAppHostUiAction::ActivateWorkspace)
        );
    }

    #[test]
    fn app_host_controller_routes_global_recovery_event_to_foreground_window() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let first = controller
            .open_window()
            .expect("expected first window open");
        let second = controller
            .open_window()
            .expect("expected second window open");
        let _ = controller
            .focus_window(second.registration.window_id)
            .expect("expected second window focus");
        let _ = controller
            .dispatch_window_trigger(
                second.registration.window_id,
                StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");

        let dispatch = controller
            .dispatch_global_event(StudioAppWindowHostGlobalEvent::RunPanelRecoveryRequested)
            .expect("expected global recovery event");
        let recovery = dispatch.dispatch.expect("expected recovery dispatch");

        assert_eq!(recovery.target_window_id, second.registration.window_id);
        assert_ne!(recovery.target_window_id, first.registration.window_id);
        match &recovery.effects.runtime_report.dispatch {
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
        assert_eq!(
            controller.state().foreground_window_id,
            Some(second.registration.window_id)
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_controller_ignores_foreground_run_panel_recovery_without_windows() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let recovery = controller
            .dispatch_foreground_run_panel_recovery_action()
            .expect("expected optional foreground recovery");

        assert!(recovery.is_none());
    }

    #[test]
    fn app_host_controller_ignores_ui_actions_without_windows() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        assert_eq!(
            controller
                .state()
                .ui_action_state(StudioAppHostUiAction::RunManualWorkspace),
            Some(&StudioAppHostUiActionState {
                action: StudioAppHostUiAction::RunManualWorkspace,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            })
        );
        assert_eq!(
            controller
                .state()
                .ui_action_state(StudioAppHostUiAction::ResumeWorkspace),
            Some(&StudioAppHostUiActionState {
                action: StudioAppHostUiAction::ResumeWorkspace,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            })
        );
        assert_eq!(
            controller
                .state()
                .ui_action_state(StudioAppHostUiAction::HoldWorkspace),
            Some(&StudioAppHostUiActionState {
                action: StudioAppHostUiAction::HoldWorkspace,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            })
        );
        assert_eq!(
            controller
                .state()
                .ui_action_state(StudioAppHostUiAction::ActivateWorkspace),
            Some(&StudioAppHostUiActionState {
                action: StudioAppHostUiAction::ActivateWorkspace,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            })
        );
        assert_eq!(
            controller
                .state()
                .ui_action_state(StudioAppHostUiAction::RecoverRunPanelFailure),
            Some(&StudioAppHostUiActionState {
                action: StudioAppHostUiAction::RecoverRunPanelFailure,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                    target_window_id: None,
                },
            })
        );

        let run_manual = controller
            .dispatch_ui_action(StudioAppHostUiAction::RunManualWorkspace)
            .expect("expected optional ui action result");
        assert!(run_manual.is_none());

        let resume = controller
            .dispatch_ui_action(StudioAppHostUiAction::ResumeWorkspace)
            .expect("expected optional ui action result");
        assert!(resume.is_none());

        let hold = controller
            .dispatch_ui_action(StudioAppHostUiAction::HoldWorkspace)
            .expect("expected optional ui action result");
        assert!(hold.is_none());

        let activate = controller
            .dispatch_ui_action(StudioAppHostUiAction::ActivateWorkspace)
            .expect("expected optional ui action result");
        assert!(activate.is_none());

        let recovery = controller
            .dispatch_ui_action(StudioAppHostUiAction::RecoverRunPanelFailure)
            .expect("expected optional ui action result");

        assert!(recovery.is_none());

        let entitlement_primary = controller
            .dispatch_foreground_entitlement_primary_action()
            .expect("expected optional entitlement primary result");
        assert!(entitlement_primary.is_none());

        let entitlement_action = controller
            .dispatch_foreground_entitlement_action(EntitlementActionId::SyncEntitlement)
            .expect("expected optional entitlement action result");
        assert!(entitlement_action.is_none());
    }

    #[test]
    fn app_host_snapshot_derives_enabled_ui_command_model_after_failed_run() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");
        let _ = controller
            .dispatch_window_trigger(
                opened.registration.window_id,
                StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");

        let model = controller.state().ui_command_model();
        assert_eq!(
            model.action(StudioAppHostUiAction::RunManualWorkspace),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::RunManualWorkspace),
                command_id: "run_panel.run_manual",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 100,
                label: "Run workspace",
                enabled: true,
                detail: "Dispatch the current manual run action in the target window",
                target_window_id: Some(opened.registration.window_id),
            })
        );
        assert_eq!(
            model.action(StudioAppHostUiAction::ResumeWorkspace),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::ResumeWorkspace),
                command_id: "run_panel.resume_workspace",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 110,
                label: "Resume workspace",
                enabled: false,
                detail: "Resume is currently unavailable in the target window",
                target_window_id: Some(opened.registration.window_id),
            })
        );
        assert_eq!(
            model.action(StudioAppHostUiAction::HoldWorkspace),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::HoldWorkspace),
                command_id: "run_panel.set_hold",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 120,
                label: "Hold workspace",
                enabled: false,
                detail: "Hold is currently unavailable in the target window",
                target_window_id: Some(opened.registration.window_id),
            })
        );
        assert_eq!(
            model.action(StudioAppHostUiAction::ActivateWorkspace),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::ActivateWorkspace),
                command_id: "run_panel.set_active",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 130,
                label: "Activate workspace",
                enabled: true,
                detail: "Dispatch the current activate action in the target window",
                target_window_id: Some(opened.registration.window_id),
            })
        );
        assert_eq!(
            model.action(StudioAppHostUiAction::RecoverRunPanelFailure),
            Some(&StudioAppHostUiActionModel {
                action: Some(StudioAppHostUiAction::RecoverRunPanelFailure),
                command_id: "run_panel.recover_failure",
                group: StudioAppHostUiCommandGroup::Recovery,
                sort_order: 200,
                label: "Recover run panel failure",
                enabled: true,
                detail: "Apply the current run panel recovery action in the target window",
                target_window_id: Some(opened.registration.window_id),
            })
        );
        assert_eq!(model.actions.len(), 5);

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_executes_canvas_interaction_through_command_surface() {
        let (config, project_path) = flash_drum_local_rules_synced_config();
        let mut app_host = StudioAppHost::new(&config).expect("expected app host");
        app_host.refresh_local_canvas_suggestions();
        let opened = app_host
            .execute_command(StudioAppHostCommand::OpenWindow)
            .expect("expected window open");
        let window_id = match opened.outcome {
            StudioAppHostCommandOutcome::WindowOpened(opened) => opened.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let _ = app_host
            .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
                window_id,
                trigger: StudioRuntimeTrigger::WidgetAction(RunPanelActionId::SetActive),
            })
            .expect("expected activate command");

        let interaction = app_host
            .execute_command(StudioAppHostCommand::DispatchCanvasInteraction {
                action: StudioCanvasInteractionAction::AcceptFocusedByTab,
            })
            .expect("expected canvas interaction command");
        match interaction.outcome {
            StudioAppHostCommandOutcome::CanvasInteracted(result) => {
                assert_eq!(
                    result.action,
                    StudioCanvasInteractionAction::AcceptFocusedByTab
                );
                assert_eq!(
                    result
                        .accepted
                        .as_ref()
                        .map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.vapor")
                );
            }
            other => panic!("expected canvas interaction outcome, got {other:?}"),
        }
        assert_eq!(
            app_host.workspace_control_state().run_status,
            rf_ui::RunStatus::Converged
        );
        assert_eq!(app_host.workspace_control_state().pending_reason, None);

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_controller_reports_disabled_run_manual_command_without_windows() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let dispatch = controller
            .dispatch_ui_command("run_panel.run_manual")
            .expect("expected ui command result");

        assert_eq!(
            dispatch,
            StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
                command_id: "run_panel.run_manual".to_string(),
                detail: "Open a studio window before running the workspace".to_string(),
                target_window_id: None,
            }
        );
    }

    #[test]
    fn app_host_controller_reports_disabled_resume_command_without_windows() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let dispatch = controller
            .dispatch_ui_command("run_panel.resume_workspace")
            .expect("expected ui command result");

        assert_eq!(
            dispatch,
            StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
                command_id: "run_panel.resume_workspace".to_string(),
                detail: "Open a studio window before resuming the workspace".to_string(),
                target_window_id: None,
            }
        );
    }

    #[test]
    fn app_host_controller_reports_disabled_hold_command_without_windows() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let dispatch = controller
            .dispatch_ui_command("run_panel.set_hold")
            .expect("expected ui command result");

        assert_eq!(
            dispatch,
            StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
                command_id: "run_panel.set_hold".to_string(),
                detail: "Open a studio window before holding the workspace".to_string(),
                target_window_id: None,
            }
        );
    }

    #[test]
    fn app_host_controller_reports_disabled_activate_command_without_windows() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let dispatch = controller
            .dispatch_ui_command("run_panel.set_active")
            .expect("expected ui command result");

        assert_eq!(
            dispatch,
            StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
                command_id: "run_panel.set_active".to_string(),
                detail: "Open a studio window before activating the workspace".to_string(),
                target_window_id: None,
            }
        );
    }

    #[test]
    fn app_host_controller_dispatches_ui_command_by_command_id() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");

        let dispatch = controller
            .dispatch_ui_command("run_panel.run_manual")
            .expect("expected ui command dispatch");

        match dispatch {
            StudioAppHostUiCommandDispatchResult::Executed(run) => {
                assert_eq!(run.target_window_id, opened.registration.window_id);
                match &run.effects.runtime_report.dispatch {
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
            }
            other => panic!("expected executed ui command dispatch, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_controller_dispatches_activate_ui_command_by_command_id() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");

        let dispatch = controller
            .dispatch_ui_command("run_panel.set_active")
            .expect("expected ui command dispatch");

        match dispatch {
            StudioAppHostUiCommandDispatchResult::Executed(activate) => {
                assert_eq!(activate.target_window_id, opened.registration.window_id);
                match &activate.effects.runtime_report.dispatch {
                    crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                        crate::StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                            assert_eq!(dispatch.simulation_mode, rf_ui::SimulationMode::Active);
                        }
                        other => panic!("expected workspace mode dispatch, got {other:?}"),
                    },
                    other => panic!("expected app command dispatch, got {other:?}"),
                }
            }
            other => panic!("expected executed ui command dispatch, got {other:?}"),
        }
    }

    #[test]
    fn app_host_controller_dispatches_resume_ui_command_by_command_id() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");

        let dispatch = controller
            .dispatch_ui_command("run_panel.resume_workspace")
            .expect("expected ui command dispatch");

        match dispatch {
            StudioAppHostUiCommandDispatchResult::Executed(resume) => {
                assert_eq!(resume.target_window_id, opened.registration.window_id);
                match &resume.effects.runtime_report.dispatch {
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
            }
            other => panic!("expected executed ui command dispatch, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_controller_dispatches_hold_ui_command_by_command_id_after_activation() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");
        let _ = controller
            .dispatch_ui_command("run_panel.set_active")
            .expect("expected activate command dispatch");

        let dispatch = controller
            .dispatch_ui_command("run_panel.set_hold")
            .expect("expected ui command dispatch");

        match dispatch {
            StudioAppHostUiCommandDispatchResult::Executed(hold) => {
                assert_eq!(hold.target_window_id, opened.registration.window_id);
                match &hold.effects.runtime_report.dispatch {
                    crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                        crate::StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                            assert_eq!(dispatch.simulation_mode, rf_ui::SimulationMode::Hold);
                        }
                        other => panic!("expected workspace mode dispatch, got {other:?}"),
                    },
                    other => panic!("expected app command dispatch, got {other:?}"),
                }
            }
            other => panic!("expected executed ui command dispatch, got {other:?}"),
        }
    }

    #[test]
    fn app_host_controller_reports_disabled_ui_command_by_command_id() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");

        let dispatch = controller
            .dispatch_ui_command("run_panel.recover_failure")
            .expect("expected ui command result");

        assert_eq!(
            dispatch,
            StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
                command_id: "run_panel.recover_failure".to_string(),
                detail: "No run panel recovery action is currently available in the target window"
                    .to_string(),
                target_window_id: Some(opened.registration.window_id),
            }
        );
    }

    #[test]
    fn app_host_controller_dispatches_recovery_ui_command_by_command_id() {
        let (config, project_path) = solver_failure_config();
        let mut controller = StudioAppHostController::new(&config).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");
        let _ = controller
            .dispatch_window_trigger(
                opened.registration.window_id,
                StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
            )
            .expect("expected failed run dispatch");

        let dispatch = controller
            .dispatch_ui_command("run_panel.recover_failure")
            .expect("expected ui command dispatch");

        match dispatch {
            StudioAppHostUiCommandDispatchResult::Executed(recovery) => {
                assert_eq!(recovery.target_window_id, opened.registration.window_id);
                match &recovery.effects.runtime_report.dispatch {
                    crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
                        assert_eq!(outcome.action.title, "Inspect unit inputs");
                    }
                    other => panic!("expected run panel recovery dispatch, got {other:?}"),
                }
            }
            other => panic!("expected executed ui command dispatch, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn app_host_controller_reports_missing_ui_command_by_command_id() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let dispatch = controller
            .dispatch_ui_command("run_panel.unknown")
            .expect("expected ui command result");

        assert_eq!(
            dispatch,
            StudioAppHostUiCommandDispatchResult::IgnoredMissing {
                command_id: "run_panel.unknown".to_string(),
            }
        );
    }

    #[test]
    fn app_host_controller_returns_optional_results_for_ignored_cases() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

        let ignored_global = controller
            .dispatch_global_event(StudioAppWindowHostGlobalEvent::NetworkRestored)
            .expect("expected ignored global event");
        assert!(ignored_global.dispatch.is_none());
        assert!(ignored_global.projection.state.windows.is_empty());

        let opened = controller.open_window().expect("expected window open");
        let closed = controller
            .close_window(opened.registration.window_id)
            .expect("expected close");
        assert!(closed.close.is_some());

        let ignored_close = controller
            .close_window(opened.registration.window_id)
            .expect("expected ignored close");
        assert!(ignored_close.close.is_none());
        assert!(ignored_close.projection.state.windows.is_empty());
    }

    #[test]
    fn app_host_controller_maps_close_side_effects_into_gui_facing_summary() {
        let mut controller =
            StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
        let opened = controller.open_window().expect("expected window open");
        let _ = controller
            .dispatch_window_trigger(
                opened.registration.window_id,
                StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            )
            .expect("expected timer trigger");

        let closed = controller
            .close_window(opened.registration.window_id)
            .expect("expected close");
        let close = closed.close.expect("expected close effects");

        assert_eq!(close.window_id, opened.registration.window_id);
        assert!(matches!(
            close.retirement,
            StudioWindowHostRetirement::Parked {
                parked_entitlement_timer: Some(_)
            }
        ));
        assert!(matches!(
            close.native_timer_transitions.as_slice(),
            [crate::StudioWindowTimerDriverTransition::ParkNativeTimer { from_window_id, .. }]
            if *from_window_id == opened.registration.window_id
        ));
        assert!(close.native_timer_acks.is_empty());
        assert!(matches!(
            closed.projection.state.entitlement_timer,
            StudioAppHostEntitlementTimerState::Parked { .. }
        ));
    }
}
