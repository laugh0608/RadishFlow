mod projection;
use projection::*;
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
    SaveDocument,
    UndoDocumentCommand,
    RedoDocumentCommand,
    RunManualWorkspace,
    ResumeWorkspace,
    HoldWorkspace,
    ActivateWorkspace,
    RecoverRunPanelFailure,
    SyncEntitlement,
    RefreshOfflineLease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAppHostUiActionDisabledReason {
    NoRegisteredWindow,
    SaveUnavailable,
    UndoUnavailable,
    RedoUnavailable,
    RunManualUnavailable,
    ResumeUnavailable,
    HoldUnavailable,
    ActivateUnavailable,
    NoRunPanelRecovery,
    SyncEntitlementUnavailable,
    RefreshOfflineLeaseUnavailable,
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
    File,
    Edit,
    RunPanel,
    Recovery,
    Entitlement,
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

#[derive(Debug, Clone, PartialEq)]
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

    pub fn begin_canvas_place_unit(
        &mut self,
        unit_kind: impl Into<String>,
    ) -> rf_ui::CanvasEditIntent {
        self.window_host_manager.begin_canvas_place_unit(unit_kind)
    }

    pub fn cancel_canvas_pending_edit(&mut self) -> Option<rf_ui::CanvasEditIntent> {
        self.window_host_manager.cancel_canvas_pending_edit()
    }

    pub fn commit_canvas_pending_edit_at(
        &mut self,
        position: rf_ui::CanvasPoint,
    ) -> RfResult<Option<rf_ui::CanvasEditCommitResult>> {
        self.window_host_manager
            .commit_canvas_pending_edit_at(position)
    }

    pub fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::AcceptFocusedByTab)
            .map(|result| result.accepted)
    }

    pub fn accept_canvas_suggestion(
        &mut self,
        suggestion_id: rf_ui::CanvasSuggestionId,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::AcceptById {
            suggestion_id,
        })
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

    pub fn inspector_drafts(&self) -> &rf_ui::InspectorDraftState {
        &self
            .window_host_manager
            .session()
            .host_port()
            .runtime()
            .app_state()
            .workspace
            .drafts
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

    pub fn document(&self) -> &rf_ui::FlowsheetDocument {
        &self
            .window_host_manager
            .session()
            .host_port()
            .runtime()
            .app_state()
            .workspace
            .document
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

    pub fn document_last_saved_revision(&self) -> Option<u64> {
        self.window_host_manager
            .session()
            .host_port()
            .runtime()
            .app_state()
            .workspace
            .last_saved_revision
    }

    pub fn document_has_unsaved_changes(&self) -> bool {
        self.document_last_saved_revision() != Some(self.document().revision)
    }

    pub fn latest_solve_snapshot(&self) -> Option<rf_ui::SolveSnapshot> {
        rf_ui::latest_snapshot(
            &self
                .window_host_manager
                .session()
                .host_port()
                .runtime()
                .app_state()
                .workspace,
        )
        .cloned()
    }

    pub fn snapshot_history_count(&self) -> usize {
        self.window_host_manager
            .session()
            .host_port()
            .runtime()
            .app_state()
            .workspace
            .snapshot_history
            .len()
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

    pub fn begin_canvas_place_unit(
        &mut self,
        unit_kind: impl Into<String>,
    ) -> rf_ui::CanvasEditIntent {
        self.app_host.begin_canvas_place_unit(unit_kind)
    }

    pub fn cancel_canvas_pending_edit(&mut self) -> Option<rf_ui::CanvasEditIntent> {
        self.app_host.cancel_canvas_pending_edit()
    }

    pub fn commit_canvas_pending_edit_at(
        &mut self,
        position: rf_ui::CanvasPoint,
    ) -> RfResult<Option<rf_ui::CanvasEditCommitResult>> {
        self.app_host.commit_canvas_pending_edit_at(position)
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

    pub fn accept_canvas_suggestion(
        &mut self,
        suggestion_id: rf_ui::CanvasSuggestionId,
    ) -> RfResult<Option<rf_ui::CanvasSuggestion>> {
        self.dispatch_canvas_interaction(StudioCanvasInteractionAction::AcceptById {
            suggestion_id,
        })
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
        let expected_action = action.clone();
        let (outcome, _) =
            self.execute_command(StudioAppHostCommand::DispatchCanvasInteraction { action })?;
        let StudioAppHostCommandOutcome::CanvasInteracted(result) = outcome else {
            return Err(RfError::invalid_input(format!(
                "app host controller expected canvas interaction outcome for {expected_action:?}"
            )));
        };
        Ok(result)
    }

    fn window_dispatch_result(
        projection: StudioAppHostProjection,
        dispatch: StudioAppWindowHostDispatch,
    ) -> StudioAppHostWindowDispatchResult {
        StudioAppHostWindowDispatchResult {
            projection,
            target_window_id: dispatch.target_window_id,
            effects: dispatch_effects_from_session(dispatch.dispatch),
        }
    }

    fn expect_window_dispatch(
        outcome: StudioAppHostCommandOutcome,
        projection: StudioAppHostProjection,
        expected: &'static str,
    ) -> RfResult<StudioAppHostWindowDispatchResult> {
        let StudioAppHostCommandOutcome::WindowDispatched(dispatch) = outcome else {
            return Err(RfError::invalid_input(expected));
        };

        Ok(Self::window_dispatch_result(projection, dispatch))
    }

    fn expect_optional_window_dispatch(
        outcome: StudioAppHostCommandOutcome,
        projection: StudioAppHostProjection,
        expected: &str,
    ) -> RfResult<Option<StudioAppHostWindowDispatchResult>> {
        match outcome {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch) => {
                Ok(Some(Self::window_dispatch_result(projection, dispatch)))
            }
            StudioAppHostCommandOutcome::IgnoredUiAction => Ok(None),
            other => Err(RfError::invalid_input(format!("{expected}, got {other:?}"))),
        }
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

    pub fn inspector_drafts(&self) -> &rf_ui::InspectorDraftState {
        self.app_host.inspector_drafts()
    }

    pub fn canvas_interaction(&self) -> rf_ui::CanvasInteractionState {
        self.app_host.canvas_interaction()
    }

    pub fn document(&self) -> &rf_ui::FlowsheetDocument {
        self.app_host.document()
    }

    pub fn document_path(&self) -> Option<&Path> {
        self.app_host.document_path()
    }

    pub fn document_last_saved_revision(&self) -> Option<u64> {
        self.app_host.document_last_saved_revision()
    }

    pub fn document_has_unsaved_changes(&self) -> bool {
        self.app_host.document_has_unsaved_changes()
    }

    pub fn latest_solve_snapshot(&self) -> Option<rf_ui::SolveSnapshot> {
        self.app_host.latest_solve_snapshot()
    }

    pub fn snapshot_history_count(&self) -> usize {
        self.app_host.snapshot_history_count()
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
        Self::expect_window_dispatch(
            outcome,
            projection,
            "app host controller expected window dispatch outcome",
        )
    }

    pub fn dispatch_ui_action(
        &mut self,
        action: StudioAppHostUiAction,
    ) -> RfResult<Option<StudioAppHostWindowDispatchResult>> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::DispatchUiAction { action })?;
        Self::expect_optional_window_dispatch(
            outcome,
            projection,
            "app host controller expected ui action outcome",
        )
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

    pub fn focus_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioAppHostWindowDispatchResult> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::FocusWindow { window_id })?;
        Self::expect_window_dispatch(
            outcome,
            projection,
            "app host controller expected focus dispatch outcome",
        )
    }

    pub fn dispatch_global_event(
        &mut self,
        event: StudioAppWindowHostGlobalEvent,
    ) -> RfResult<StudioAppHostGlobalEventResult> {
        let (outcome, projection) =
            self.execute_command(StudioAppHostCommand::DispatchGlobalEvent { event })?;
        let dispatch = match outcome {
            StudioAppHostCommandOutcome::WindowDispatched(dispatch) => {
                Some(Self::window_dispatch_result(projection.clone(), dispatch))
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

#[cfg(test)]
mod tests;
