use std::time::SystemTime;

use rf_types::{RfError, RfResult};

use crate::{
    StudioAppHostState, StudioAppHostUiCommandModel, StudioGuiCanvasInteractionAction,
    StudioGuiCanvasState, StudioGuiCommandRegistry, StudioGuiFocusContext, StudioGuiHost,
    StudioGuiHostCanvasInteractionResult, StudioGuiHostCommand, StudioGuiHostCommandOutcome,
    StudioGuiHostLifecycleEvent, StudioGuiHostUiCommandDispatchResult,
    StudioGuiHostWindowLayoutUpdateResult, StudioGuiNativeTimerRuntime, StudioGuiShortcut,
    StudioGuiShortcutIgnoreReason, StudioGuiShortcutRoute, StudioGuiSnapshot,
    StudioGuiWindowDropTargetQuery, StudioGuiWindowLayoutMutation, StudioGuiWindowModel,
    StudioRuntimeConfig, StudioRuntimeTrigger, StudioWindowHostId,
};

#[cfg(test)]
mod command_surface_tests;
#[cfg(test)]
mod interaction_tests;
#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod timer_tests;

#[derive(Debug, Clone, PartialEq)]
pub enum StudioGuiEvent {
    OpenWindowRequested,
    CloseWindowRequested {
        window_id: StudioWindowHostId,
    },
    WindowTriggerRequested {
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    },
    WindowForegrounded {
        window_id: StudioWindowHostId,
    },
    UiCommandRequested {
        command_id: String,
    },
    InspectorFieldDraftUpdateRequested {
        command_id: String,
        raw_value: String,
    },
    InspectorFieldDraftCommitRequested {
        command_id: String,
    },
    InspectorFieldDraftDiscardRequested {
        command_id: String,
    },
    InspectorFieldDraftBatchCommitRequested {
        command_id: String,
    },
    InspectorFieldDraftBatchDiscardRequested {
        command_id: String,
    },
    InspectorCompositionNormalizeRequested {
        command_id: String,
    },
    InspectorCompositionComponentAddRequested {
        command_id: String,
    },
    InspectorCompositionComponentRemoveRequested {
        command_id: String,
    },
    CanvasSuggestionAcceptRequested,
    CanvasSuggestionAcceptByIdRequested {
        suggestion_id: rf_ui::CanvasSuggestionId,
    },
    CanvasSuggestionRejectRequested,
    CanvasSuggestionFocusNextRequested,
    CanvasSuggestionFocusPreviousRequested,
    CanvasPendingEditCommitRequested {
        position: rf_ui::CanvasPoint,
    },
    CanvasUnitLayoutMoveRequested {
        unit_id: rf_types::UnitId,
        position: rf_ui::CanvasPoint,
    },
    WindowLayoutMutationRequested {
        window_id: Option<StudioWindowHostId>,
        mutation: StudioGuiWindowLayoutMutation,
    },
    WindowDropTargetQueryRequested {
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    },
    WindowDropTargetPreviewRequested {
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    },
    WindowDropTargetPreviewCleared {
        window_id: Option<StudioWindowHostId>,
    },
    WindowDropTargetApplyRequested {
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    },
    ShortcutPressed {
        shortcut: StudioGuiShortcut,
        focus_context: StudioGuiFocusContext,
    },
    NativeTimerElapsed {
        window_id: Option<StudioWindowHostId>,
        handle_id: crate::StudioWindowNativeTimerHandleId,
    },
    LoginCompleted,
    NetworkRestored,
    EntitlementTimerElapsed,
    RunPanelRecoveryRequested,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiDriverDispatch {
    pub event: StudioGuiEvent,
    pub outcome: StudioGuiDriverOutcome,
    pub snapshot: StudioGuiSnapshot,
    pub window: StudioGuiWindowModel,
    pub state: StudioAppHostState,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub command_registry: StudioGuiCommandRegistry,
    pub canvas: StudioGuiCanvasState,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StudioGuiDriverOutcome {
    HostCommand(StudioGuiHostCommandOutcome),
    CanvasInteraction(StudioGuiHostCanvasInteractionResult),
    WindowLayoutUpdated(StudioGuiHostWindowLayoutUpdateResult),
    IgnoredNativeTimerElapsed {
        window_id: Option<StudioWindowHostId>,
        handle_id: crate::StudioWindowNativeTimerHandleId,
    },
    IgnoredShortcut {
        shortcut: StudioGuiShortcut,
        reason: StudioGuiShortcutIgnoreReason,
    },
}

pub struct StudioGuiDriver {
    host: StudioGuiHost,
    native_timer_runtime: StudioGuiNativeTimerRuntime,
}

impl StudioGuiDriver {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            host: StudioGuiHost::new(config)?,
            native_timer_runtime: StudioGuiNativeTimerRuntime::default(),
        })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.host.state()
    }

    pub fn ui_commands(&self) -> StudioAppHostUiCommandModel {
        self.host.ui_commands()
    }

    pub fn command_registry(&self) -> StudioGuiCommandRegistry {
        self.host.command_registry()
    }

    pub fn canvas_state(&self) -> StudioGuiCanvasState {
        self.host.canvas_state()
    }

    pub fn host(&self) -> &StudioGuiHost {
        &self.host
    }

    pub fn native_timer_runtime(&self) -> &StudioGuiNativeTimerRuntime {
        &self.native_timer_runtime
    }

    pub fn next_due_native_timer_at(&self) -> Option<SystemTime> {
        self.native_timer_runtime.next_due_at()
    }

    pub fn snapshot(&self) -> StudioGuiSnapshot {
        self.host.snapshot()
    }

    pub fn window_model(&self) -> StudioGuiWindowModel {
        self.snapshot().window_model()
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.host.window_model_for_window(window_id)
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<rf_ui::CanvasSuggestion>) {
        self.host.replace_canvas_suggestions(suggestions);
    }

    pub fn begin_canvas_place_unit(
        &mut self,
        unit_kind: impl Into<String>,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        self.host.begin_canvas_place_unit(unit_kind)
    }

    pub fn commit_canvas_pending_edit_at(
        &mut self,
        position: rf_ui::CanvasPoint,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        self.host.commit_canvas_pending_edit_at(position)
    }

    pub fn drain_due_native_timer_events(
        &mut self,
        now: SystemTime,
    ) -> RfResult<Vec<StudioGuiDriverDispatch>> {
        let due_events = self.native_timer_runtime.drain_due_events(now);
        let mut dispatches = Vec::with_capacity(due_events.len());
        for _ in due_events {
            dispatches.push(self.dispatch_event(StudioGuiEvent::EntitlementTimerElapsed)?);
        }
        Ok(dispatches)
    }

    pub fn dispatch_event(&mut self, event: StudioGuiEvent) -> RfResult<StudioGuiDriverDispatch> {
        let command_registry = self.host.command_registry();
        let outcome = match route_driver_event(&event, &command_registry) {
            DriverRoute::HostCommand(command) => {
                StudioGuiDriverOutcome::HostCommand(self.host.execute_command(command)?)
            }
            DriverRoute::CanvasInteraction(action) => {
                match self
                    .host
                    .execute_command(StudioGuiHostCommand::DispatchCanvasInteraction { action })?
                {
                    StudioGuiHostCommandOutcome::CanvasInteracted(result) => {
                        StudioGuiDriverOutcome::CanvasInteraction(result)
                    }
                    other => {
                        return Err(RfError::invalid_input(format!(
                            "driver expected canvas interaction outcome, got {other:?}"
                        )));
                    }
                }
            }
            DriverRoute::IgnoredShortcut { shortcut, reason } => {
                StudioGuiDriverOutcome::IgnoredShortcut { shortcut, reason }
            }
            DriverRoute::WindowLayoutMutation {
                window_id,
                mutation,
            } => StudioGuiDriverOutcome::WindowLayoutUpdated(
                self.host.update_window_layout(window_id, mutation)?,
            ),
            DriverRoute::NativeTimerElapsed {
                window_id,
                handle_id,
            } => {
                if self
                    .native_timer_runtime
                    .consume_elapsed_event(window_id, handle_id)
                    .is_some()
                {
                    StudioGuiDriverOutcome::HostCommand(self.host.execute_command(
                        StudioGuiHostCommand::DispatchLifecycleEvent {
                            event: StudioGuiHostLifecycleEvent::TimerElapsed,
                        },
                    )?)
                } else {
                    StudioGuiDriverOutcome::IgnoredNativeTimerElapsed {
                        window_id,
                        handle_id,
                    }
                }
            }
        };
        self.apply_native_timer_effects_from_outcome(&outcome);
        let ui_commands = surfaced_ui_commands(&outcome).unwrap_or_else(|| self.host.ui_commands());
        let canvas = surfaced_canvas_state(&outcome).unwrap_or_else(|| self.host.canvas_state());
        let snapshot = self.host.snapshot();
        let window = self
            .host
            .window_model_for_window(layout_scope_window_id(&outcome));
        Ok(StudioGuiDriverDispatch {
            event,
            outcome,
            snapshot,
            window,
            state: self.host.state().clone(),
            ui_commands,
            command_registry: self.host.command_registry(),
            canvas,
        })
    }

    fn apply_native_timer_effects_from_outcome(&mut self, outcome: &StudioGuiDriverOutcome) {
        let effects = match outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => Some(&opened.native_timers),
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowDispatched(
                dispatch,
            )) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::LifecycleDispatched(lifecycle),
            ) => lifecycle
                .dispatch
                .as_ref()
                .map(|dispatch| &dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
                ),
            ) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction { .. }
                    | StudioGuiHostUiCommandDispatchResult::ExecutedCanvasUnitLayoutMove { .. },
                ),
            ) => None,
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::InspectorDraftUpdated(dispatch),
            ) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::InspectorDraftCommitted(dispatch),
            ) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::InspectorDraftDiscarded(dispatch),
            ) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::InspectorDraftBatchCommitted(dispatch),
            ) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::InspectorDraftBatchDiscarded(dispatch),
            ) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::InspectorCompositionNormalized(dispatch),
            ) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::InspectorCompositionComponentAdded(dispatch),
            ) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::InspectorCompositionComponentRemoved(dispatch),
            ) => Some(&dispatch.native_timers),
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(
                closed,
            )) => Some(&closed.native_timers),
            StudioGuiDriverOutcome::HostCommand(_)
            | StudioGuiDriverOutcome::CanvasInteraction(_)
            | StudioGuiDriverOutcome::WindowLayoutUpdated(_)
            | StudioGuiDriverOutcome::IgnoredNativeTimerElapsed { .. }
            | StudioGuiDriverOutcome::IgnoredShortcut { .. } => None,
        };

        if let Some(effects) = effects {
            self.native_timer_runtime.apply_effects(effects);
        }
    }
}

fn layout_scope_window_id(outcome: &StudioGuiDriverOutcome) -> Option<StudioWindowHostId> {
    match outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            Some(opened.registration.window_id)
        }
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowDispatched(
            dispatch,
        )) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
            lifecycle,
        )) => lifecycle
            .dispatch
            .as_ref()
            .map(|dispatch| dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            crate::StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        )) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftUpdated(dispatch),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftCommitted(dispatch),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftDiscarded(dispatch),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftBatchCommitted(dispatch),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftBatchDiscarded(dispatch),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorCompositionNormalized(dispatch),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorCompositionComponentAdded(dispatch),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorCompositionComponentRemoved(dispatch),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            crate::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                target_window_id,
                ..
            },
        )) => *target_window_id,
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            crate::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasUnitLayoutMove {
                target_window_id,
                ..
            },
        )) => *target_window_id,
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            crate::StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                target_window_id, ..
            },
        )) => *target_window_id,
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            crate::StudioGuiHostUiCommandDispatchResult::IgnoredMissing { .. },
        ))
        | StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetQueried(
                crate::StudioGuiHostWindowDropTargetQueryResult {
                    target_window_id: None,
                    ..
                },
            ),
        )
        | StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(
                crate::StudioGuiHostWindowDropTargetQueryResult {
                    target_window_id: None,
                    ..
                },
            ),
        )
        | StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(
                crate::StudioGuiHostWindowDropPreviewClearResult {
                    target_window_id: None,
                    ..
                },
            ),
        )
        | StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetApplied(
                crate::StudioGuiHostWindowDropTargetApplyResult {
                    target_window_id: None,
                    ..
                },
            ),
        )
        | StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::CanvasInteracted(_))
        | StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::CanvasUnitLayoutMoved(_),
        )
        | StudioGuiDriverOutcome::WindowLayoutUpdated(
            crate::StudioGuiHostWindowLayoutUpdateResult {
                target_window_id: None,
                ..
            },
        )
        | StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(_))
        | StudioGuiDriverOutcome::CanvasInteraction(_)
        | StudioGuiDriverOutcome::IgnoredNativeTimerElapsed { .. }
        | StudioGuiDriverOutcome::IgnoredShortcut { .. } => None,
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetQueried(
                crate::StudioGuiHostWindowDropTargetQueryResult {
                    target_window_id: Some(window_id),
                    ..
                },
            ),
        ) => Some(*window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(
                crate::StudioGuiHostWindowDropTargetQueryResult {
                    target_window_id: Some(window_id),
                    ..
                },
            ),
        ) => Some(*window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(
                crate::StudioGuiHostWindowDropPreviewClearResult {
                    target_window_id: Some(window_id),
                    ..
                },
            ),
        ) => Some(*window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetApplied(
                crate::StudioGuiHostWindowDropTargetApplyResult {
                    target_window_id: Some(window_id),
                    ..
                },
            ),
        ) => Some(*window_id),
        StudioGuiDriverOutcome::WindowLayoutUpdated(
            crate::StudioGuiHostWindowLayoutUpdateResult {
                target_window_id: Some(window_id),
                ..
            },
        ) => Some(*window_id),
    }
}

fn surfaced_ui_commands(
    outcome: &StudioGuiDriverOutcome,
) -> Option<crate::StudioAppHostUiCommandModel> {
    match outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            Some(opened.ui_commands.clone())
        }
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowDispatched(
            dispatch,
        )) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::CanvasInteracted(
            result,
        )) => Some(result.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::CanvasUnitLayoutMoved(result),
        ) => Some(result.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
            lifecycle,
        )) => Some(lifecycle.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        )) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftUpdated(dispatch),
        ) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftCommitted(dispatch),
        ) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftDiscarded(dispatch),
        ) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftBatchCommitted(dispatch),
        ) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftBatchDiscarded(dispatch),
        ) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorCompositionNormalized(dispatch),
        ) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorCompositionComponentAdded(dispatch),
        ) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorCompositionComponentRemoved(dispatch),
        ) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction { result, .. },
        )) => Some(result.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::ExecutedCanvasUnitLayoutMove { result, .. },
        )) => Some(result.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::IgnoredDisabled { ui_commands, .. }
            | StudioGuiHostUiCommandDispatchResult::IgnoredMissing { ui_commands, .. },
        )) => Some(ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(closed)) => {
            Some(closed.ui_commands.clone())
        }
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetQueried(_)
            | StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(_)
            | StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(_)
            | StudioGuiHostCommandOutcome::WindowDropTargetApplied(_),
        )
        | StudioGuiDriverOutcome::CanvasInteraction(_)
        | StudioGuiDriverOutcome::WindowLayoutUpdated(_)
        | StudioGuiDriverOutcome::IgnoredNativeTimerElapsed { .. }
        | StudioGuiDriverOutcome::IgnoredShortcut { .. } => None,
    }
}

fn surfaced_canvas_state(outcome: &StudioGuiDriverOutcome) -> Option<StudioGuiCanvasState> {
    match outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            Some(opened.canvas.clone())
        }
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowDispatched(
            dispatch,
        )) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::CanvasInteracted(
            result,
        ))
        | StudioGuiDriverOutcome::CanvasInteraction(result) => Some(result.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::CanvasUnitLayoutMoved(result),
        ) => Some(result.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
            lifecycle,
        )) => Some(lifecycle.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        )) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftUpdated(dispatch),
        ) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftCommitted(dispatch),
        ) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftDiscarded(dispatch),
        ) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftBatchCommitted(dispatch),
        ) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftBatchDiscarded(dispatch),
        ) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorCompositionNormalized(dispatch),
        ) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorCompositionComponentAdded(dispatch),
        ) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorCompositionComponentRemoved(dispatch),
        ) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction { result, .. },
        )) => Some(result.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::ExecutedCanvasUnitLayoutMove { result, .. },
        )) => Some(result.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(closed)) => {
            Some(closed.canvas.clone())
        }
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::UiCommandDispatched(
                StudioGuiHostUiCommandDispatchResult::IgnoredDisabled { .. }
                | StudioGuiHostUiCommandDispatchResult::IgnoredMissing { .. },
            )
            | StudioGuiHostCommandOutcome::WindowDropTargetQueried(_)
            | StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(_)
            | StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(_)
            | StudioGuiHostCommandOutcome::WindowDropTargetApplied(_),
        )
        | StudioGuiDriverOutcome::WindowLayoutUpdated(_)
        | StudioGuiDriverOutcome::IgnoredNativeTimerElapsed { .. }
        | StudioGuiDriverOutcome::IgnoredShortcut { .. } => None,
    }
}

enum DriverRoute {
    HostCommand(StudioGuiHostCommand),
    CanvasInteraction(StudioGuiCanvasInteractionAction),
    WindowLayoutMutation {
        window_id: Option<StudioWindowHostId>,
        mutation: StudioGuiWindowLayoutMutation,
    },
    NativeTimerElapsed {
        window_id: Option<StudioWindowHostId>,
        handle_id: crate::StudioWindowNativeTimerHandleId,
    },
    IgnoredShortcut {
        shortcut: StudioGuiShortcut,
        reason: StudioGuiShortcutIgnoreReason,
    },
}

fn route_driver_event(event: &StudioGuiEvent, registry: &StudioGuiCommandRegistry) -> DriverRoute {
    match event {
        StudioGuiEvent::OpenWindowRequested => {
            DriverRoute::HostCommand(StudioGuiHostCommand::OpenWindow)
        }
        StudioGuiEvent::CloseWindowRequested { window_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::CloseWindow {
                window_id: *window_id,
            })
        }
        StudioGuiEvent::WindowTriggerRequested { window_id, trigger } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchWindowTrigger {
                window_id: *window_id,
                trigger: trigger.clone(),
            })
        }
        StudioGuiEvent::WindowForegrounded { window_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::WindowForegrounded {
                    window_id: *window_id,
                },
            })
        }
        StudioGuiEvent::UiCommandRequested { command_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchUiCommand {
                command_id: command_id.clone(),
            })
        }
        StudioGuiEvent::InspectorFieldDraftUpdateRequested {
            command_id,
            raw_value,
        } => DriverRoute::HostCommand(StudioGuiHostCommand::DispatchInspectorDraftUpdate {
            command_id: command_id.clone(),
            raw_value: raw_value.clone(),
        }),
        StudioGuiEvent::InspectorFieldDraftCommitRequested { command_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchInspectorDraftCommit {
                command_id: command_id.clone(),
            })
        }
        StudioGuiEvent::InspectorFieldDraftDiscardRequested { command_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchInspectorDraftDiscard {
                command_id: command_id.clone(),
            })
        }
        StudioGuiEvent::InspectorFieldDraftBatchCommitRequested { command_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchInspectorDraftBatchCommit {
                command_id: command_id.clone(),
            })
        }
        StudioGuiEvent::InspectorFieldDraftBatchDiscardRequested { command_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchInspectorDraftBatchDiscard {
                command_id: command_id.clone(),
            })
        }
        StudioGuiEvent::InspectorCompositionNormalizeRequested { command_id } => {
            DriverRoute::HostCommand(
                StudioGuiHostCommand::DispatchInspectorCompositionNormalize {
                    command_id: command_id.clone(),
                },
            )
        }
        StudioGuiEvent::InspectorCompositionComponentAddRequested { command_id } => {
            DriverRoute::HostCommand(
                StudioGuiHostCommand::DispatchInspectorCompositionComponentAdd {
                    command_id: command_id.clone(),
                },
            )
        }
        StudioGuiEvent::InspectorCompositionComponentRemoveRequested { command_id } => {
            DriverRoute::HostCommand(
                StudioGuiHostCommand::DispatchInspectorCompositionComponentRemove {
                    command_id: command_id.clone(),
                },
            )
        }
        StudioGuiEvent::CanvasSuggestionAcceptRequested => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::AcceptFocusedByTab)
        }
        StudioGuiEvent::CanvasSuggestionAcceptByIdRequested { suggestion_id } => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::AcceptById {
                suggestion_id: suggestion_id.clone(),
            })
        }
        StudioGuiEvent::CanvasSuggestionRejectRequested => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::RejectFocused)
        }
        StudioGuiEvent::CanvasSuggestionFocusNextRequested => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::FocusNext)
        }
        StudioGuiEvent::CanvasSuggestionFocusPreviousRequested => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::FocusPrevious)
        }
        StudioGuiEvent::CanvasPendingEditCommitRequested { position } => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::CommitPendingEditAt {
                position: *position,
            })
        }
        StudioGuiEvent::CanvasUnitLayoutMoveRequested { unit_id, position } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::MoveCanvasUnitLayout {
                unit_id: unit_id.clone(),
                position: *position,
            })
        }
        StudioGuiEvent::WindowLayoutMutationRequested {
            window_id,
            mutation,
        } => DriverRoute::WindowLayoutMutation {
            window_id: *window_id,
            mutation: mutation.clone(),
        },
        StudioGuiEvent::WindowDropTargetQueryRequested { window_id, query } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::QueryWindowDropTarget {
                window_id: *window_id,
                query: *query,
            })
        }
        StudioGuiEvent::WindowDropTargetPreviewRequested { window_id, query } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::SetWindowDropTargetPreview {
                window_id: *window_id,
                query: *query,
            })
        }
        StudioGuiEvent::WindowDropTargetPreviewCleared { window_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::ClearWindowDropTargetPreview {
                window_id: *window_id,
            })
        }
        StudioGuiEvent::WindowDropTargetApplyRequested { window_id, query } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::ApplyWindowDropTarget {
                window_id: *window_id,
                query: *query,
            })
        }
        StudioGuiEvent::ShortcutPressed {
            shortcut,
            focus_context,
        } => match crate::route_shortcut(registry, shortcut, *focus_context) {
            StudioGuiShortcutRoute::DispatchCommandId { command_id } => {
                DriverRoute::HostCommand(StudioGuiHostCommand::DispatchUiCommand { command_id })
            }
            StudioGuiShortcutRoute::Ignored { reason } => DriverRoute::IgnoredShortcut {
                shortcut: shortcut.clone(),
                reason,
            },
        },
        StudioGuiEvent::NativeTimerElapsed {
            window_id,
            handle_id,
        } => DriverRoute::NativeTimerElapsed {
            window_id: *window_id,
            handle_id: *handle_id,
        },
        StudioGuiEvent::LoginCompleted => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::LoginCompleted,
            })
        }
        StudioGuiEvent::NetworkRestored => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::NetworkRestored,
            })
        }
        StudioGuiEvent::EntitlementTimerElapsed => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::TimerElapsed,
            })
        }
        StudioGuiEvent::RunPanelRecoveryRequested => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::RunPanelRecoveryRequested,
            })
        }
    }
}
