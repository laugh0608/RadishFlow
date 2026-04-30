use std::collections::BTreeMap;

use rf_types::{RfError, RfResult};
use rf_ui::{AppLogEntry, CanvasSuggestion, CanvasSuggestionId};

use crate::studio_gui_layout_store::{
    load_persisted_window_layouts, save_persisted_window_layouts,
};
use crate::{
    StudioAppHostCloseEffects, StudioAppHostController, StudioAppHostDispatchEffects,
    StudioAppHostGlobalEventResult, StudioAppHostProjection, StudioAppHostState,
    StudioAppHostUiCommandDispatchResult, StudioAppHostUiCommandModel,
    StudioAppHostWindowDispatchResult, StudioAppWindowHostGlobalEvent,
    StudioCanvasInteractionAction, StudioGuiCommandRegistry,
    StudioGuiInspectorTargetDetailSnapshot, StudioGuiInspectorTargetFieldSnapshot,
    StudioGuiInspectorTargetFieldValidationSnapshot,
    StudioGuiInspectorTargetFieldValueKindSnapshot, StudioGuiInspectorTargetPortSnapshot,
    StudioGuiInspectorTargetSummaryRowSnapshot, StudioGuiNativeTimerEffects,
    StudioGuiRuntimeSnapshot, StudioGuiSnapshot, StudioGuiWindowDropPreviewState,
    StudioGuiWindowDropTarget, StudioGuiWindowDropTargetQuery, StudioGuiWindowLayoutMutation,
    StudioGuiWindowLayoutPersistenceState, StudioGuiWindowLayoutState, StudioGuiWindowModel,
    StudioRuntimeConfig, StudioRuntimeTrigger, StudioWindowHostId, StudioWindowHostRegistration,
    studio_gui_canvas_widget::canvas_action_id_from_command_id,
};

#[cfg(test)]
use crate::{
    StudioGuiWindowAreaId, StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion,
    StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
    StudioRuntimeEntitlementSessionEvent, StudioWindowHostRetirement,
};

mod core;
mod dispatch;
mod helpers;
mod layout;

#[cfg(test)]
mod drop_tests;
#[cfg(test)]
mod interaction_tests;
#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod test_support;

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiHostWindowOpened {
    pub projection: StudioAppHostProjection,
    pub registration: StudioWindowHostRegistration,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub canvas: StudioGuiCanvasState,
    pub native_timers: StudioGuiNativeTimerEffects,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiHostDispatch {
    pub projection: StudioAppHostProjection,
    pub target_window_id: StudioWindowHostId,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub canvas: StudioGuiCanvasState,
    pub effects: StudioAppHostDispatchEffects,
    pub native_timers: StudioGuiNativeTimerEffects,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StudioGuiHostUiCommandDispatchResult {
    Executed(StudioGuiHostDispatch),
    ExecutedCanvasInteraction {
        command_id: String,
        target_window_id: Option<StudioWindowHostId>,
        result: StudioGuiHostCanvasInteractionResult,
    },
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

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiHostGlobalEventDispatch {
    pub projection: StudioAppHostProjection,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub canvas: StudioGuiCanvasState,
    pub dispatch: Option<StudioGuiHostDispatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiHostLifecycleEvent {
    WindowForegrounded { window_id: StudioWindowHostId },
    LoginCompleted,
    NetworkRestored,
    TimerElapsed,
    RunPanelRecoveryRequested,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiHostLifecycleDispatch {
    pub event: StudioGuiHostLifecycleEvent,
    pub projection: StudioAppHostProjection,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub canvas: StudioGuiCanvasState,
    pub dispatch: Option<StudioGuiHostDispatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiHostCloseWindowResult {
    pub projection: StudioAppHostProjection,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub canvas: StudioGuiCanvasState,
    pub close: Option<StudioAppHostCloseEffects>,
    pub native_timers: StudioGuiNativeTimerEffects,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StudioGuiCanvasState {
    pub suggestions: Vec<CanvasSuggestion>,
    pub focused_suggestion_id: Option<CanvasSuggestionId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiHostCanvasInteractionResult {
    pub action: StudioGuiCanvasInteractionAction,
    pub accepted: Option<CanvasSuggestion>,
    pub rejected: Option<CanvasSuggestion>,
    pub focused: Option<CanvasSuggestion>,
    pub applied_target: Option<rf_ui::InspectorTarget>,
    pub latest_log_entry: Option<AppLogEntry>,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub canvas: StudioGuiCanvasState,
}

pub type StudioGuiHostCanvasSuggestionResult = StudioGuiHostCanvasInteractionResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiHostWindowLayoutUpdateResult {
    pub target_window_id: Option<StudioWindowHostId>,
    pub mutation: StudioGuiWindowLayoutMutation,
    pub layout_state: StudioGuiWindowLayoutState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiHostWindowDropTargetQueryResult {
    pub target_window_id: Option<StudioWindowHostId>,
    pub query: StudioGuiWindowDropTargetQuery,
    pub layout_state: StudioGuiWindowLayoutState,
    pub drop_target: Option<StudioGuiWindowDropTarget>,
    pub preview_layout_state: Option<StudioGuiWindowLayoutState>,
    pub preview_window: Option<StudioGuiWindowModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiHostWindowDropTargetApplyResult {
    pub target_window_id: Option<StudioWindowHostId>,
    pub query: StudioGuiWindowDropTargetQuery,
    pub mutation: StudioGuiWindowLayoutMutation,
    pub drop_target: StudioGuiWindowDropTarget,
    pub layout_state: StudioGuiWindowLayoutState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiHostWindowDropPreviewClearResult {
    pub target_window_id: Option<StudioWindowHostId>,
    pub layout_state: StudioGuiWindowLayoutState,
    pub had_preview: bool,
}

pub type StudioGuiCanvasInteractionAction = StudioCanvasInteractionAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiHostCommand {
    OpenWindow,
    DispatchWindowTrigger {
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    },
    DispatchCanvasInteraction {
        action: StudioGuiCanvasInteractionAction,
    },
    DispatchLifecycleEvent {
        event: StudioGuiHostLifecycleEvent,
    },
    DispatchUiCommand {
        command_id: String,
    },
    DispatchInspectorDraftUpdate {
        command_id: String,
        raw_value: String,
    },
    DispatchInspectorDraftCommit {
        command_id: String,
    },
    DispatchInspectorDraftBatchCommit {
        command_id: String,
    },
    QueryWindowDropTarget {
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    },
    SetWindowDropTargetPreview {
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    },
    ClearWindowDropTargetPreview {
        window_id: Option<StudioWindowHostId>,
    },
    ApplyWindowDropTarget {
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    },
    CloseWindow {
        window_id: StudioWindowHostId,
    },
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StudioGuiHostCommandOutcome {
    WindowOpened(StudioGuiHostWindowOpened),
    WindowDispatched(StudioGuiHostDispatch),
    CanvasInteracted(StudioGuiHostCanvasInteractionResult),
    LifecycleDispatched(StudioGuiHostLifecycleDispatch),
    UiCommandDispatched(StudioGuiHostUiCommandDispatchResult),
    InspectorDraftUpdated(StudioGuiHostDispatch),
    InspectorDraftCommitted(StudioGuiHostDispatch),
    InspectorDraftBatchCommitted(StudioGuiHostDispatch),
    WindowDropTargetQueried(StudioGuiHostWindowDropTargetQueryResult),
    WindowDropTargetPreviewUpdated(StudioGuiHostWindowDropTargetQueryResult),
    WindowDropTargetPreviewCleared(StudioGuiHostWindowDropPreviewClearResult),
    WindowDropTargetApplied(StudioGuiHostWindowDropTargetApplyResult),
    WindowClosed(StudioGuiHostCloseWindowResult),
}

pub struct StudioGuiHost {
    controller: StudioAppHostController,
    layout_state_overrides: BTreeMap<String, StudioGuiWindowLayoutPersistenceState>,
    window_drop_previews: BTreeMap<String, StudioGuiWindowDropPreviewState>,
}
