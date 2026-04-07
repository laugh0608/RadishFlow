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
    StudioAppHostWindowDispatchResult, StudioAppWindowHostGlobalEvent, StudioGuiCommandRegistry,
    StudioGuiNativeTimerEffects, StudioGuiRuntimeSnapshot, StudioGuiSnapshot,
    StudioGuiWindowDropPreviewState, StudioGuiWindowDropTarget,
    StudioGuiWindowDropTargetQuery, StudioGuiWindowLayoutMutation,
    StudioGuiWindowLayoutPersistenceState, StudioGuiWindowLayoutState, StudioGuiWindowModel,
    StudioRuntimeConfig, StudioRuntimeTrigger, StudioWindowHostId, StudioWindowHostRegistration,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiHostWindowOpened {
    pub projection: StudioAppHostProjection,
    pub registration: StudioWindowHostRegistration,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub canvas: StudioGuiCanvasState,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiCanvasInteractionAction {
    AcceptFocusedByTab,
    RejectFocused,
    FocusNext,
    FocusPrevious,
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
pub enum StudioGuiHostCommandOutcome {
    WindowOpened(StudioGuiHostWindowOpened),
    WindowDispatched(StudioGuiHostDispatch),
    LifecycleDispatched(StudioGuiHostLifecycleDispatch),
    UiCommandDispatched(StudioGuiHostUiCommandDispatchResult),
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

impl StudioGuiHost {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        let controller = StudioAppHostController::new(config)?;
        let layout_state_overrides = match controller.document_path() {
            Some(project_path) => load_persisted_window_layouts(project_path)?,
            None => BTreeMap::new(),
        };

        Ok(Self {
            controller,
            layout_state_overrides,
            window_drop_previews: BTreeMap::new(),
        })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.controller.state()
    }

    pub fn ui_commands(&self) -> StudioAppHostUiCommandModel {
        self.state().ui_command_model()
    }

    pub fn canvas_state(&self) -> StudioGuiCanvasState {
        let canvas = self.controller.canvas_interaction();
        StudioGuiCanvasState {
            suggestions: canvas.suggestions,
            focused_suggestion_id: canvas.focused_suggestion_id,
        }
    }

    pub fn command_registry(&self) -> StudioGuiCommandRegistry {
        StudioGuiCommandRegistry::from_model(&self.ui_commands())
    }

    pub fn snapshot(&self) -> StudioGuiSnapshot {
        let mut snapshot = StudioGuiSnapshot::new(
            self.state().clone(),
            self.ui_commands(),
            self.command_registry(),
            self.canvas_state().widget(),
            StudioGuiRuntimeSnapshot {
                control_state: self.controller.workspace_control_state(),
                run_panel: self.controller.run_panel_widget(),
                entitlement_host: self.controller.entitlement_host_output(),
                log_entries: self.controller.log_entries(),
            },
            self.window_drop_previews.clone(),
        );
        snapshot.layout_state = self.layout_state_for_window_from_snapshot(&snapshot, None);
        snapshot
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        let snapshot = self.snapshot();
        let mut window = snapshot.window_model_for_window(window_id);
        window.layout_state = self.layout_state_for_window_from_snapshot(&snapshot, window_id);
        window
    }

    pub fn refresh_local_canvas_suggestions(&mut self) {
        self.controller.refresh_local_canvas_suggestions();
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<CanvasSuggestion>) {
        self.controller.replace_canvas_suggestions(suggestions);
    }

    pub fn update_window_layout(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        mutation: StudioGuiWindowLayoutMutation,
    ) -> RfResult<StudioGuiHostWindowLayoutUpdateResult> {
        self.validate_registered_window_for_layout(window_id, "layout updates")?;
        let snapshot = self.snapshot();
        let layout_state = self
            .layout_state_for_window_from_snapshot(&snapshot, window_id)
            .applying_mutation(&mutation);
        self.clear_window_drop_preview_for_scope(&layout_state.scope.layout_key);
        if let Some(legacy_layout_key) = layout_state.scope.legacy_layout_key() {
            self.layout_state_overrides.remove(&legacy_layout_key);
            self.window_drop_previews.remove(&legacy_layout_key);
        }
        self.layout_state_overrides.insert(
            layout_state.scope.layout_key.clone(),
            layout_state.persistence_state(),
        );
        self.persist_window_layouts()?;

        Ok(StudioGuiHostWindowLayoutUpdateResult {
            target_window_id: layout_state.scope.window_id,
            mutation,
            layout_state,
        })
    }

    pub fn query_window_drop_target(
        &self,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    ) -> RfResult<StudioGuiHostWindowDropTargetQueryResult> {
        self.validate_registered_window_for_layout(window_id, "drop target queries")?;
        let snapshot = self.snapshot();
        let layout_state = self.layout_state_for_window_from_snapshot(&snapshot, window_id);
        let drop_target = layout_state.drop_target_for_query(&query);
        let preview_layout_state = layout_state.preview_layout_state_for_query(&query);
        let preview_window = preview_layout_state.as_ref().map(|preview_layout_state| {
            snapshot
                .window_model_for_window(layout_state.scope.window_id)
                .with_layout_state(preview_layout_state.clone())
        });

        Ok(StudioGuiHostWindowDropTargetQueryResult {
            target_window_id: layout_state.scope.window_id,
            query,
            layout_state,
            drop_target,
            preview_layout_state,
            preview_window,
        })
    }

    pub fn set_window_drop_target_preview(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    ) -> RfResult<StudioGuiHostWindowDropTargetQueryResult> {
        let query_result = self.query_window_drop_target(window_id, query)?;
        self.clear_window_drop_preview_for_scope(&query_result.layout_state.scope.layout_key);
        if let (Some(drop_target), Some(preview_layout_state)) = (
            query_result.drop_target.clone(),
            query_result.preview_layout_state.clone(),
        ) {
            self.window_drop_previews.insert(
                query_result.layout_state.scope.layout_key.clone(),
                StudioGuiWindowDropPreviewState {
                    query,
                    drop_target,
                    preview_layout_state,
                },
            );
        }
        Ok(query_result)
    }

    pub fn clear_window_drop_target_preview(
        &mut self,
        window_id: Option<StudioWindowHostId>,
    ) -> RfResult<StudioGuiHostWindowDropPreviewClearResult> {
        self.validate_registered_window_for_layout(window_id, "drop preview updates")?;
        let snapshot = self.snapshot();
        let layout_state = self.layout_state_for_window_from_snapshot(&snapshot, window_id);
        let mut had_preview =
            self.clear_window_drop_preview_for_scope(&layout_state.scope.layout_key);
        if let Some(legacy_layout_key) = layout_state.scope.legacy_layout_key() {
            had_preview |= self.clear_window_drop_preview_for_scope(&legacy_layout_key);
        }
        Ok(StudioGuiHostWindowDropPreviewClearResult {
            target_window_id: layout_state.scope.window_id,
            layout_state,
            had_preview,
        })
    }

    pub fn apply_window_drop_target(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    ) -> RfResult<StudioGuiHostWindowDropTargetApplyResult> {
        let query_result = self.query_window_drop_target(window_id, query)?;
        let drop_target = query_result.drop_target.ok_or_else(|| {
            RfError::invalid_input(format!(
                "drop target query `{query:?}` is not applicable for the current layout state"
            ))
        })?;
        let mutation = query.layout_mutation();
        let update = self.update_window_layout(window_id, mutation.clone())?;

        Ok(StudioGuiHostWindowDropTargetApplyResult {
            target_window_id: update.target_window_id,
            query,
            mutation,
            drop_target,
            layout_state: update.layout_state,
        })
    }

    pub fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        let accepted = self.controller.accept_focused_canvas_suggestion_by_tab()?;
        Ok(self.build_canvas_interaction_result(
            StudioGuiCanvasInteractionAction::AcceptFocusedByTab,
            accepted,
            None,
        ))
    }

    pub fn reject_focused_canvas_suggestion(
        &mut self,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        let rejected = self.controller.reject_focused_canvas_suggestion();
        Ok(self.build_canvas_interaction_result(
            StudioGuiCanvasInteractionAction::RejectFocused,
            None,
            rejected,
        ))
    }

    pub fn focus_next_canvas_suggestion(
        &mut self,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        let focused = self.controller.focus_next_canvas_suggestion();
        Ok(self.build_canvas_interaction_result_with_focus(
            StudioGuiCanvasInteractionAction::FocusNext,
            None,
            None,
            focused,
        ))
    }

    pub fn focus_previous_canvas_suggestion(
        &mut self,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        let focused = self.controller.focus_previous_canvas_suggestion();
        Ok(self.build_canvas_interaction_result_with_focus(
            StudioGuiCanvasInteractionAction::FocusPrevious,
            None,
            None,
            focused,
        ))
    }

    pub fn execute_command(
        &mut self,
        command: StudioGuiHostCommand,
    ) -> RfResult<StudioGuiHostCommandOutcome> {
        match command {
            StudioGuiHostCommand::OpenWindow => self
                .open_window()
                .map(StudioGuiHostCommandOutcome::WindowOpened),
            StudioGuiHostCommand::DispatchWindowTrigger { window_id, trigger } => self
                .dispatch_window_trigger(window_id, trigger)
                .map(StudioGuiHostCommandOutcome::WindowDispatched),
            StudioGuiHostCommand::DispatchLifecycleEvent { event } => self
                .dispatch_lifecycle_event(event)
                .map(StudioGuiHostCommandOutcome::LifecycleDispatched),
            StudioGuiHostCommand::DispatchUiCommand { command_id } => self
                .dispatch_ui_command(&command_id)
                .map(StudioGuiHostCommandOutcome::UiCommandDispatched),
            StudioGuiHostCommand::QueryWindowDropTarget { window_id, query } => self
                .query_window_drop_target(window_id, query)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetQueried),
            StudioGuiHostCommand::SetWindowDropTargetPreview { window_id, query } => self
                .set_window_drop_target_preview(window_id, query)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated),
            StudioGuiHostCommand::ClearWindowDropTargetPreview { window_id } => self
                .clear_window_drop_target_preview(window_id)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared),
            StudioGuiHostCommand::ApplyWindowDropTarget { window_id, query } => self
                .apply_window_drop_target(window_id, query)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetApplied),
            StudioGuiHostCommand::CloseWindow { window_id } => self
                .close_window(window_id)
                .map(StudioGuiHostCommandOutcome::WindowClosed),
        }
    }

    pub fn open_window(&mut self) -> RfResult<StudioGuiHostWindowOpened> {
        let opened = self.controller.open_window()?;
        Ok(StudioGuiHostWindowOpened {
            ui_commands: ui_commands_from_projection(&opened.projection),
            canvas: self.canvas_state(),
            projection: opened.projection,
            registration: opened.registration,
        })
    }

    pub fn dispatch_window_trigger(
        &mut self,
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    ) -> RfResult<StudioGuiHostDispatch> {
        let dispatch = self
            .controller
            .dispatch_window_trigger(window_id, trigger)?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn focus_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioGuiHostDispatch> {
        let dispatch = self.controller.focus_window(window_id)?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
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
                    canvas: dispatch.canvas.clone(),
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
                    canvas: result.canvas,
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
                    dispatch_from_controller(dispatch, self.canvas_state()),
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
        Ok(global_event_from_controller(result, self.canvas_state()))
    }

    pub fn close_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioGuiHostCloseWindowResult> {
        if self.state().window(window_id).is_some() {
            let snapshot = self.snapshot();
            let layout_state =
                self.layout_state_for_window_from_snapshot(&snapshot, Some(window_id));
            self.clear_window_drop_preview_for_scope(&layout_state.scope.layout_key);
            if let Some(legacy_layout_key) = layout_state.scope.legacy_layout_key() {
                self.clear_window_drop_preview_for_scope(&legacy_layout_key);
            }
        }
        let closed = self.controller.close_window(window_id)?;
        Ok(StudioGuiHostCloseWindowResult {
            ui_commands: ui_commands_from_projection(&closed.projection),
            canvas: self.canvas_state(),
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

impl StudioGuiHost {
    fn persist_window_layouts(&self) -> RfResult<()> {
        match self.controller.document_path() {
            Some(project_path) => {
                save_persisted_window_layouts(project_path, &self.layout_state_overrides)
            }
            None => Ok(()),
        }
    }

    fn layout_state_for_window_from_snapshot(
        &self,
        snapshot: &StudioGuiSnapshot,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowLayoutState {
        let derived = StudioGuiWindowLayoutState::from_snapshot_for_window(snapshot, window_id);
        self.layout_state_overrides
            .get(&derived.scope.layout_key)
            .or_else(|| {
                derived
                    .scope
                    .legacy_layout_key()
                    .as_ref()
                    .and_then(|legacy_layout_key| {
                        self.layout_state_overrides.get(legacy_layout_key)
                    })
            })
            .map(|persisted| derived.merged_with_persisted(persisted))
            .unwrap_or(derived)
    }

    fn validate_registered_window_for_layout(
        &self,
        window_id: Option<StudioWindowHostId>,
        action: &str,
    ) -> RfResult<()> {
        if let Some(window_id) = window_id.filter(|window_id| self.state().window(*window_id).is_none()) {
            return Err(RfError::invalid_input(format!(
                "window host `{window_id}` is not registered for {action}"
            )));
        }
        Ok(())
    }

    fn clear_window_drop_preview_for_scope(&mut self, layout_key: &str) -> bool {
        self.window_drop_previews.remove(layout_key).is_some()
    }

    fn build_canvas_interaction_result(
        &self,
        action: StudioGuiCanvasInteractionAction,
        accepted: Option<CanvasSuggestion>,
        rejected: Option<CanvasSuggestion>,
    ) -> StudioGuiHostCanvasInteractionResult {
        let focused = self
            .controller
            .canvas_interaction()
            .focused_suggestion()
            .cloned();
        self.build_canvas_interaction_result_with_focus(action, accepted, rejected, focused)
    }

    fn build_canvas_interaction_result_with_focus(
        &self,
        action: StudioGuiCanvasInteractionAction,
        accepted: Option<CanvasSuggestion>,
        rejected: Option<CanvasSuggestion>,
        focused: Option<CanvasSuggestion>,
    ) -> StudioGuiHostCanvasInteractionResult {
        StudioGuiHostCanvasInteractionResult {
            action,
            applied_target: accepted
                .as_ref()
                .and_then(|_| self.controller.active_inspector_target()),
            latest_log_entry: accepted
                .as_ref()
                .and_then(|_| self.controller.latest_log_entry()),
            accepted,
            rejected,
            focused,
            ui_commands: self.ui_commands(),
            canvas: self.canvas_state(),
        }
    }
}

fn dispatch_from_controller(
    dispatch: StudioAppHostWindowDispatchResult,
    canvas: StudioGuiCanvasState,
) -> StudioGuiHostDispatch {
    let native_timers = StudioGuiNativeTimerEffects::from_driver(
        &dispatch.effects.native_timer_transitions,
        &dispatch.effects.native_timer_acks,
    );
    StudioGuiHostDispatch {
        ui_commands: ui_commands_from_projection(&dispatch.projection),
        canvas,
        projection: dispatch.projection,
        target_window_id: dispatch.target_window_id,
        effects: dispatch.effects,
        native_timers,
    }
}

fn global_event_from_controller(
    result: StudioAppHostGlobalEventResult,
    canvas: StudioGuiCanvasState,
) -> StudioGuiHostGlobalEventDispatch {
    StudioGuiHostGlobalEventDispatch {
        ui_commands: ui_commands_from_projection(&result.projection),
        canvas: canvas.clone(),
        projection: result.projection.clone(),
        dispatch: result
            .dispatch
            .map(|dispatch| dispatch_from_controller(dispatch, canvas)),
    }
}

fn ui_commands_from_projection(
    projection: &StudioAppHostProjection,
) -> StudioAppHostUiCommandModel {
    projection.state.ui_command_model()
}

fn global_event_from_lifecycle(
    event: StudioGuiHostLifecycleEvent,
) -> StudioAppWindowHostGlobalEvent {
    match event {
        StudioGuiHostLifecycleEvent::WindowForegrounded { .. } => {
            unreachable!(
                "window foregrounding is routed through focus_window before global mapping"
            )
        }
        StudioGuiHostLifecycleEvent::LoginCompleted => {
            StudioAppWindowHostGlobalEvent::LoginCompleted
        }
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

    use rf_store::{
        read_studio_layout_file, studio_layout_path_for_project, write_studio_layout_file,
    };
    use rf_ui::RunPanelActionId;

    use crate::{
        StudioGuiHost, StudioGuiHostCommand, StudioGuiHostCommandOutcome,
        StudioGuiHostLifecycleEvent, StudioGuiHostUiCommandDispatchResult,
        StudioGuiNativeTimerEffects, StudioGuiWindowAreaId, StudioGuiWindowDockPlacement,
        StudioGuiWindowDockRegion, StudioGuiWindowDropTargetQuery,
        StudioGuiWindowLayoutMutation, StudioRuntimeConfig,
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
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-studio-gui-host-failure-{timestamp}.rfproj.json"
        ));
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

    fn layout_persistence_config() -> (StudioRuntimeConfig, PathBuf, PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-studio-layout-persistence-{timestamp}.rfproj.json"
        ));
        let project = include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json");
        fs::write(&project_path, project).expect("expected persistence project");
        let layout_path = studio_layout_path_for_project(&project_path);

        (
            StudioRuntimeConfig {
                project_path: project_path.clone(),
                ..lease_expiring_config()
            },
            project_path,
            layout_path,
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
    fn gui_host_queries_drop_target_through_explicit_command_surface() {
        let (config, project_path, layout_path) = layout_persistence_config();
        let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
        let window_id = match gui_host
            .execute_command(StudioGuiHostCommand::OpenWindow)
            .expect("expected window open")
        {
            StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };

        let queried = gui_host
            .execute_command(StudioGuiHostCommand::QueryWindowDropTarget {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect("expected drop target query");

        match queried {
            StudioGuiHostCommandOutcome::WindowDropTargetQueried(result) => {
                assert_eq!(result.target_window_id, Some(window_id));
                assert_eq!(
                    result.query,
                    StudioGuiWindowDropTargetQuery::DockRegion {
                        area_id: StudioGuiWindowAreaId::Runtime,
                        dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                        placement: StudioGuiWindowDockPlacement::Start,
                    }
                );
                assert_eq!(
                    result.drop_target.as_ref().map(|target| target.dock_region),
                    Some(StudioGuiWindowDockRegion::LeftSidebar)
                );
                assert_eq!(
                    result.preview_layout_state.as_ref().map(|layout| {
                        layout
                            .panel(StudioGuiWindowAreaId::Runtime)
                            .map(|panel| (panel.dock_region, panel.stack_group, panel.order))
                    }),
                    Some(Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10)))
                );
                assert_eq!(
                    result.preview_window.as_ref().and_then(|window| {
                        window
                            .layout_state
                            .panel(StudioGuiWindowAreaId::Runtime)
                            .map(|panel| (panel.dock_region, panel.stack_group, panel.order))
                    }),
                    Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
                );
                assert_eq!(
                    result
                        .layout_state
                        .panel(StudioGuiWindowAreaId::Runtime)
                        .map(|panel| (panel.dock_region, panel.stack_group)),
                    Some((StudioGuiWindowDockRegion::RightSidebar, 10))
                );
            }
            other => panic!("expected drop target query outcome, got {other:?}"),
        }
        assert_eq!(gui_host.window_model_for_window(Some(window_id)).drop_preview, None);

        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_host_sets_drop_preview_and_surfaces_it_through_window_model() {
        let (config, project_path, layout_path) = layout_persistence_config();
        let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
        let window_id = match gui_host
            .execute_command(StudioGuiHostCommand::OpenWindow)
            .expect("expected window open")
        {
            StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };

        let previewed = gui_host
            .execute_command(StudioGuiHostCommand::SetWindowDropTargetPreview {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect("expected drop preview update");

        match previewed {
            StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(result) => {
                assert_eq!(result.target_window_id, Some(window_id));
                assert!(result.drop_target.is_some());
                assert!(result.preview_layout_state.is_some());
            }
            other => panic!("expected drop preview update outcome, got {other:?}"),
        }

        let window = gui_host.window_model_for_window(Some(window_id));
        let preview = window.drop_preview.expect("expected drop preview in window model");
        assert_eq!(
            preview.drop_target.dock_region,
            StudioGuiWindowDockRegion::LeftSidebar
        );
        assert_eq!(
            preview
                .preview_layout_state
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
        );
        assert_eq!(
            preview
                .preview_layout
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
        );
        assert_eq!(preview.overlay.drag_area_id, StudioGuiWindowAreaId::Runtime);
        assert_eq!(
            preview.overlay.target_dock_region,
            StudioGuiWindowDockRegion::LeftSidebar
        );
        assert_eq!(preview.overlay.target_stack_group, 10);
        assert_eq!(preview.overlay.target_tab_index, 0);
        assert_eq!(
            preview.overlay.target_stack_area_ids,
            vec![StudioGuiWindowAreaId::Runtime]
        );
        assert_eq!(
            preview.overlay.target_stack_active_area_id,
            StudioGuiWindowAreaId::Runtime
        );
        assert_eq!(
            preview.changed_area_ids,
            vec![StudioGuiWindowAreaId::Commands, StudioGuiWindowAreaId::Runtime]
        );
        assert_eq!(
            window
                .layout_state
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::RightSidebar, 10, 30))
        );
        assert!(gui_host
            .snapshot()
            .window_model_for_window(Some(window_id))
            .drop_preview
            .is_some());

        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_host_clears_drop_preview_through_explicit_command_surface() {
        let (config, project_path, layout_path) = layout_persistence_config();
        let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
        let window_id = match gui_host
            .execute_command(StudioGuiHostCommand::OpenWindow)
            .expect("expected window open")
        {
            StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let _ = gui_host
            .execute_command(StudioGuiHostCommand::SetWindowDropTargetPreview {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect("expected drop preview update");

        let cleared = gui_host
            .execute_command(StudioGuiHostCommand::ClearWindowDropTargetPreview {
                window_id: Some(window_id),
            })
            .expect("expected drop preview clear");

        match cleared {
            StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(result) => {
                assert_eq!(result.target_window_id, Some(window_id));
                assert!(result.had_preview);
            }
            other => panic!("expected drop preview clear outcome, got {other:?}"),
        }
        assert_eq!(gui_host.window_model_for_window(Some(window_id)).drop_preview, None);

        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_host_applies_drop_target_through_explicit_command_surface() {
        let (config, project_path, layout_path) = layout_persistence_config();
        let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
        let window_id = match gui_host
            .execute_command(StudioGuiHostCommand::OpenWindow)
            .expect("expected window open")
        {
            StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let _ = gui_host
            .execute_command(StudioGuiHostCommand::SetWindowDropTargetPreview {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect("expected drop preview update");

        let applied = gui_host
            .execute_command(StudioGuiHostCommand::ApplyWindowDropTarget {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect("expected drop target apply");

        match applied {
            StudioGuiHostCommandOutcome::WindowDropTargetApplied(result) => {
                assert_eq!(result.target_window_id, Some(window_id));
                assert_eq!(
                    result.drop_target.dock_region,
                    StudioGuiWindowDockRegion::LeftSidebar
                );
                assert_eq!(
                    result
                        .layout_state
                        .panel(StudioGuiWindowAreaId::Runtime)
                        .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
                    Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
                );
            }
            other => panic!("expected drop target apply outcome, got {other:?}"),
        }
        assert_eq!(gui_host.window_model_for_window(Some(window_id)).drop_preview, None);

        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_host_returns_no_preview_window_for_inapplicable_drop_query() {
        let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
        let window_id = match gui_host
            .execute_command(StudioGuiHostCommand::OpenWindow)
            .expect("expected window open")
        {
            StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };

        let queried = gui_host
            .execute_command(StudioGuiHostCommand::QueryWindowDropTarget {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::Unstack {
                    area_id: StudioGuiWindowAreaId::Commands,
                    placement: StudioGuiWindowDockPlacement::End,
                },
            })
            .expect("expected drop target query");

        match queried {
            StudioGuiHostCommandOutcome::WindowDropTargetQueried(result) => {
                assert_eq!(result.drop_target, None);
                assert_eq!(result.preview_layout_state, None);
                assert_eq!(result.preview_window, None);
            }
            other => panic!("expected drop target query outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_host_rejects_inapplicable_drop_target_apply() {
        let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
        let window_id = match gui_host
            .execute_command(StudioGuiHostCommand::OpenWindow)
            .expect("expected window open")
        {
            StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };

        let error = gui_host
            .execute_command(StudioGuiHostCommand::ApplyWindowDropTarget {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::Unstack {
                    area_id: StudioGuiWindowAreaId::Commands,
                    placement: StudioGuiWindowDockPlacement::End,
                },
            })
            .expect_err("expected invalid drop apply");

        assert_eq!(error.code().as_str(), "invalid_input");
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

    #[test]
    fn gui_host_persists_window_layout_overrides_into_project_sidecar() {
        let (config, project_path, layout_path) = layout_persistence_config();
        let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
        let opened = gui_host.open_window().expect("expected window open");

        let updated = gui_host
            .update_window_layout(
                Some(opened.registration.window_id),
                StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                    area_id: StudioGuiWindowAreaId::Commands,
                    collapsed: true,
                },
            )
            .expect("expected layout update");
        assert_eq!(
            updated
                .layout_state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| panel.collapsed),
            Some(true)
        );

        let second_update = gui_host
            .update_window_layout(
                Some(opened.registration.window_id),
                StudioGuiWindowLayoutMutation::SetRegionWeight {
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    weight: 33,
                },
            )
            .expect("expected region weight update");
        assert_eq!(
            second_update
                .layout_state
                .region_weight(StudioGuiWindowDockRegion::RightSidebar)
                .map(|region| region.weight),
            Some(33)
        );

        let third_update = gui_host
            .update_window_layout(
                Some(opened.registration.window_id),
                StudioGuiWindowLayoutMutation::SetCenterArea {
                    area_id: StudioGuiWindowAreaId::Runtime,
                },
            )
            .expect("expected center area update");
        assert_eq!(
            third_update.layout_state.center_area,
            StudioGuiWindowAreaId::Runtime
        );

        let fourth_update = gui_host
            .update_window_layout(
                Some(opened.registration.window_id),
                StudioGuiWindowLayoutMutation::SetPanelOrder {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    order: 5,
                },
            )
            .expect("expected panel order update");
        assert_eq!(
            fourth_update
                .layout_state
                .panels
                .iter()
                .map(|panel| (panel.area_id, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (StudioGuiWindowAreaId::Commands, 10),
                (StudioGuiWindowAreaId::Canvas, 20),
                (StudioGuiWindowAreaId::Runtime, 5),
            ]
        );

        let fifth_update = gui_host
            .update_window_layout(
                Some(opened.registration.window_id),
                StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                    area_id: StudioGuiWindowAreaId::Commands,
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            )
            .expect("expected panel dock region update");
        assert_eq!(
            fifth_update
                .layout_state
                .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
                .into_iter()
                .map(|panel| (panel.area_id, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (StudioGuiWindowAreaId::Commands, 10),
                (StudioGuiWindowAreaId::Runtime, 10),
            ]
        );

        let sixth_update = gui_host
            .update_window_layout(
                Some(opened.registration.window_id),
                StudioGuiWindowLayoutMutation::StackPanelWith {
                    area_id: StudioGuiWindowAreaId::Commands,
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            )
            .expect("expected panel stack update");
        assert_eq!(
            sixth_update
                .layout_state
                .panels_in_stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
                .into_iter()
                .map(|panel| (panel.area_id, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (StudioGuiWindowAreaId::Commands, 10),
                (StudioGuiWindowAreaId::Runtime, 20),
            ]
        );

        let seventh_update = gui_host
            .update_window_layout(
                Some(opened.registration.window_id),
                StudioGuiWindowLayoutMutation::ActivateNextPanelInStack {
                    area_id: StudioGuiWindowAreaId::Commands,
                },
            )
            .expect("expected stack cycle update");
        assert_eq!(
            seventh_update
                .layout_state
                .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
            Some(StudioGuiWindowAreaId::Runtime)
        );

        let eighth_update = gui_host
            .update_window_layout(
                Some(opened.registration.window_id),
                StudioGuiWindowLayoutMutation::MovePanelWithinStack {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Commands,
                    },
                },
            )
            .expect("expected stack reorder update");
        assert_eq!(
            eighth_update
                .layout_state
                .panels_in_stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
                .into_iter()
                .map(|panel| (panel.area_id, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (StudioGuiWindowAreaId::Runtime, 10),
                (StudioGuiWindowAreaId::Commands, 20),
            ]
        );

        let ninth_update = gui_host
            .update_window_layout(
                Some(opened.registration.window_id),
                StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
                    area_id: StudioGuiWindowAreaId::Commands,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            )
            .expect("expected panel unstack update");
        assert_eq!(
            ninth_update
                .layout_state
                .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
                .into_iter()
                .map(|panel| (panel.area_id, panel.stack_group, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (StudioGuiWindowAreaId::Commands, 10, 10),
                (StudioGuiWindowAreaId::Runtime, 20, 10),
            ]
        );

        let stored = read_studio_layout_file(&layout_path).expect("expected stored layout sidecar");
        assert_eq!(stored.entries.len(), 1);
        assert_eq!(stored.entries[0].layout_key, "studio.window.owner.slot-1");
        assert_eq!(stored.entries[0].center_area, "runtime");
        let mut stored_panels = stored.entries[0]
            .panels
            .iter()
            .map(|panel| {
                (
                    panel.area_id.as_str(),
                    panel.dock_region.as_str(),
                    panel.stack_group,
                    panel.order,
                )
            })
            .collect::<Vec<_>>();
        stored_panels.sort_unstable();
        assert_eq!(
            stored_panels,
            vec![
                ("canvas", "center-stage", 10, 20),
                ("commands", "right-sidebar", 10, 10),
                ("runtime", "right-sidebar", 20, 10),
            ]
        );
        assert_eq!(stored.entries[0].stack_groups.len(), 3);
        assert_eq!(
            stored.entries[0]
                .stack_groups
                .iter()
                .find(|group| {
                    group.dock_region == "right-sidebar" && group.stack_group == 10
                })
                .map(|group| group.active_area_id.as_str()),
            Some("commands")
        );
        assert_eq!(
            stored.entries[0]
                .stack_groups
                .iter()
                .find(|group| {
                    group.dock_region == "right-sidebar" && group.stack_group == 20
                })
                .map(|group| group.active_area_id.as_str()),
            Some("runtime")
        );

        drop(gui_host);

        let mut reloaded = StudioGuiHost::new(&config).expect("expected reloaded gui host");
        let reopened = reloaded.open_window().expect("expected reopened window");
        let window = reloaded.window_model_for_window(Some(reopened.registration.window_id));

        assert_eq!(
            window
                .layout_state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| panel.collapsed),
            Some(true)
        );
        assert_eq!(
            window
                .layout_state
                .region_weight(StudioGuiWindowDockRegion::RightSidebar)
                .map(|region| region.weight),
            Some(33)
        );
        assert_eq!(
            window.layout_state.center_area,
            StudioGuiWindowAreaId::Runtime
        );
        assert_eq!(
            window
                .layout_state
                .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
                .into_iter()
                .map(|panel| (panel.area_id, panel.dock_region, panel.stack_group, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (
                    StudioGuiWindowAreaId::Commands,
                    StudioGuiWindowDockRegion::RightSidebar,
                    10,
                    10,
                ),
                (
                    StudioGuiWindowAreaId::Runtime,
                    StudioGuiWindowDockRegion::RightSidebar,
                    20,
                    10,
                ),
            ]
        );
        assert_eq!(
            window
                .layout_state
                .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
            Some(StudioGuiWindowAreaId::Commands)
        );
        assert_eq!(
            window
                .layout_state
                .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 20),
            Some(StudioGuiWindowAreaId::Runtime)
        );

        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_host_loads_legacy_window_layout_key_for_current_owner_scope() {
        let (config, project_path, layout_path) = layout_persistence_config();
        write_studio_layout_file(
            &layout_path,
            &rf_store::StoredStudioLayoutFile::new(vec![rf_store::StoredStudioWindowLayoutEntry {
                layout_key: "studio.window.owner.1".to_string(),
                center_area: "canvas".to_string(),
                panels: vec![rf_store::StoredStudioLayoutPanelState {
                    area_id: "commands".to_string(),
                    dock_region: "left-sidebar".to_string(),
                    stack_group: 10,
                    order: 10,
                    visible: true,
                    collapsed: true,
                }],
                stack_groups: Vec::new(),
                region_weights: vec![rf_store::StoredStudioLayoutRegionWeight {
                    dock_region: "right-sidebar".to_string(),
                    weight: 35,
                }],
            }]),
        )
        .expect("expected legacy layout sidecar");

        let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
        let opened = gui_host.open_window().expect("expected window open");
        let window = gui_host.window_model_for_window(Some(opened.registration.window_id));

        assert_eq!(window.layout_state.scope.layout_slot, Some(1));
        assert_eq!(
            window.layout_state.scope.layout_key,
            "studio.window.owner.slot-1"
        );
        assert_eq!(
            window
                .layout_state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| panel.collapsed),
            Some(true)
        );
        assert_eq!(
            window
                .layout_state
                .region_weight(StudioGuiWindowDockRegion::RightSidebar)
                .map(|region| region.weight),
            Some(35)
        );

        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }
}
