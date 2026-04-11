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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    EntitlementPrimaryActionRequested,
    EntitlementActionRequested {
        action_id: rf_ui::EntitlementActionId,
    },
    CanvasSuggestionAcceptRequested,
    CanvasSuggestionRejectRequested,
    CanvasSuggestionFocusNextRequested,
    CanvasSuggestionFocusPreviousRequested,
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
                StudioGuiHostCommandOutcome::EntitlementActionDispatched(
                    crate::StudioGuiHostEntitlementDispatchResult::Executed { dispatch, .. },
                ),
            ) => Some(&dispatch.native_timers),
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
                    StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction { .. },
                ),
            ) => None,
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
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            crate::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                target_window_id,
                ..
            },
        )) => *target_window_id,
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::EntitlementActionDispatched(result),
        ) => result.target_window_id(),
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

fn surfaced_ui_commands(outcome: &StudioGuiDriverOutcome) -> Option<crate::StudioAppHostUiCommandModel> {
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
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
            lifecycle,
        )) => Some(lifecycle.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        )) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction { result, .. },
        )) => Some(result.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::IgnoredDisabled { ui_commands, .. }
            | StudioGuiHostUiCommandDispatchResult::IgnoredMissing { ui_commands, .. },
        )) => Some(ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::EntitlementActionDispatched(
                crate::StudioGuiHostEntitlementDispatchResult::Executed { dispatch, .. },
            ),
        ) => Some(dispatch.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(
            closed,
        )) => Some(closed.ui_commands.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::EntitlementActionDispatched(
                crate::StudioGuiHostEntitlementDispatchResult::IgnoredDisabled { .. }
                | crate::StudioGuiHostEntitlementDispatchResult::IgnoredMissing { .. },
            ),
        )
        | StudioGuiDriverOutcome::HostCommand(
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
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
            lifecycle,
        )) => Some(lifecycle.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        )) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction { result, .. },
        )) => Some(result.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::EntitlementActionDispatched(
                crate::StudioGuiHostEntitlementDispatchResult::Executed { dispatch, .. },
            ),
        ) => Some(dispatch.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(
            closed,
        )) => Some(closed.canvas.clone()),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::UiCommandDispatched(
                StudioGuiHostUiCommandDispatchResult::IgnoredDisabled { .. }
                | StudioGuiHostUiCommandDispatchResult::IgnoredMissing { .. },
            )
            | StudioGuiHostCommandOutcome::EntitlementActionDispatched(
                crate::StudioGuiHostEntitlementDispatchResult::IgnoredDisabled { .. }
                | crate::StudioGuiHostEntitlementDispatchResult::IgnoredMissing { .. },
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
        StudioGuiEvent::EntitlementPrimaryActionRequested => DriverRoute::HostCommand(
            StudioGuiHostCommand::DispatchForegroundEntitlementPrimaryAction,
        ),
        StudioGuiEvent::EntitlementActionRequested { action_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchForegroundEntitlementAction {
                action_id: *action_id,
            })
        }
        StudioGuiEvent::CanvasSuggestionAcceptRequested => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::AcceptFocusedByTab)
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        StudioGuiCanvasInteractionAction, StudioGuiDriver, StudioGuiDriverOutcome, StudioGuiEvent,
        StudioGuiFocusContext, StudioGuiHostCommandOutcome,
        StudioGuiHostEntitlementDispatchResult, StudioGuiHostUiCommandDispatchResult,
        StudioGuiShortcut, StudioGuiShortcutIgnoreReason, StudioGuiShortcutKey,
        StudioGuiShortcutModifier, StudioGuiWindowAreaId, StudioGuiWindowDockPlacement,
        StudioGuiWindowDockRegion, StudioGuiWindowDropTargetQuery, StudioGuiWindowLayoutMutation,
        StudioRuntimeConfig, StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeTrigger,
    };
    use rf_ui::{
        EntitlementActionId, GhostElement, GhostElementKind, StreamVisualKind, StreamVisualState,
        SuggestionSource,
    };

    fn lease_expiring_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        }
    }

    fn synced_workspace_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            ..StudioRuntimeConfig::default()
        }
    }

    fn flash_drum_local_rules_synced_config() -> (StudioRuntimeConfig, PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-studio-local-rules-synced-{timestamp}.rfproj.json"
        ));
        let project =
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
        fs::write(&project_path, project).expect("expected synced local rules project");

        (
            StudioRuntimeConfig {
                project_path: project_path.clone(),
                ..synced_workspace_config()
            },
            project_path,
        )
    }

    fn unbound_outlet_failure_synced_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("examples")
                .join("flowsheets")
                .join("failures")
                .join("unbound-outlet-port.rfproj.json"),
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            trigger: StudioRuntimeTrigger::WidgetAction(rf_ui::RunPanelActionId::RunManual),
        }
    }

    fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-studio-local-rules-{timestamp}.rfproj.json"
        ));
        let project =
            include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json")
                .replacen(
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-heated\"",
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-liquid\"",
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                );
        fs::write(&project_path, project).expect("expected local rules project");

        (
            StudioRuntimeConfig {
                project_path: project_path.clone(),
                ..lease_expiring_config()
            },
            project_path,
        )
    }

    fn sample_canvas_suggestion(id: &str, confidence: f32) -> rf_ui::CanvasSuggestion {
        rf_ui::CanvasSuggestion::new(
            rf_ui::CanvasSuggestionId::new(id),
            SuggestionSource::LocalRules,
            confidence,
            GhostElement {
                kind: GhostElementKind::Connection,
                target_unit_id: rf_types::UnitId::new("flash-1"),
                visual_kind: StreamVisualKind::Material,
                visual_state: StreamVisualState::Suggested,
            },
            "accept by tab",
        )
    }

    #[test]
    fn gui_driver_opens_window_and_refreshes_command_state() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => {
                assert_eq!(opened.registration.window_id, 1);
            }
            other => panic!("expected window opened outcome, got {other:?}"),
        }
        assert_eq!(dispatch.state.windows.len(), 1);
        assert_eq!(dispatch.snapshot.app_host_state.windows.len(), 1);
        assert_eq!(dispatch.window.header.registered_window_count, 1);
        assert_eq!(
            dispatch.window.layout().default_focus_area,
            crate::StudioGuiWindowAreaId::Commands
        );
        assert_eq!(
            dispatch
                .command_registry
                .sections
                .first()
                .and_then(|section| section.commands.first())
                .and_then(|command| command.target_window_id),
            Some(1)
        );
        assert!(dispatch.canvas.suggestions.is_empty());
        assert_eq!(dispatch.snapshot.canvas.view().suggestion_count, 0);
    }

    #[test]
    fn gui_driver_routes_ui_command_request_through_single_event_entry() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let open = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match open.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "run_panel.set_active".to_string(),
            })
            .expect("expected ui command dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(executed),
                ),
            ) => {
                assert_eq!(executed.target_window_id, window_id);
            }
            other => panic!("expected executed ui command outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_routes_entitlement_primary_action_through_single_event_entry() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let open = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match open.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::EntitlementPrimaryActionRequested)
            .expect("expected entitlement primary action dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::EntitlementActionDispatched(
                    StudioGuiHostEntitlementDispatchResult::Executed {
                        action_id,
                        dispatch,
                    },
                ),
            ) => {
                assert_eq!(action_id, EntitlementActionId::RefreshOfflineLease);
                assert_eq!(dispatch.target_window_id, window_id);
            }
            other => panic!("expected executed entitlement primary action outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_routes_entitlement_action_through_single_event_entry() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let open = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match open.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::EntitlementActionRequested {
                action_id: EntitlementActionId::SyncEntitlement,
            })
            .expect("expected entitlement action dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::EntitlementActionDispatched(
                    StudioGuiHostEntitlementDispatchResult::Executed {
                        action_id,
                        dispatch,
                    },
                ),
            ) => {
                assert_eq!(action_id, EntitlementActionId::SyncEntitlement);
                assert_eq!(dispatch.target_window_id, window_id);
            }
            other => panic!("expected executed entitlement action outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_stably_ignores_entitlement_action_without_registered_window() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::EntitlementActionRequested {
                action_id: EntitlementActionId::SyncEntitlement,
            })
            .expect("expected entitlement action result");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::EntitlementActionDispatched(
                    StudioGuiHostEntitlementDispatchResult::IgnoredDisabled {
                        action_id,
                        detail,
                        target_window_id,
                    },
                ),
            ) => {
                assert_eq!(action_id, EntitlementActionId::SyncEntitlement);
                assert_eq!(
                    detail,
                    "Open a window before dispatching entitlement actions"
                );
                assert_eq!(target_window_id, None);
            }
            other => panic!("expected ignored entitlement action outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_surfaces_local_rules_canvas_state_from_project() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let initial_canvas = driver.canvas_state();
        assert_eq!(initial_canvas.suggestions.len(), 3);
        assert_eq!(
            initial_canvas
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
        );

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        assert_eq!(dispatch.canvas.suggestions.len(), 3);
        assert_eq!(dispatch.snapshot.canvas.view().suggestion_count, 3);
        assert_eq!(
            dispatch
                .snapshot
                .runtime
                .run_panel
                .view()
                .primary_action
                .label,
            "Resume"
        );
        assert_eq!(
            dispatch
                .canvas
                .suggestions
                .iter()
                .map(|suggestion| suggestion.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "local.flash_drum.connect_inlet.flash-1.stream-heated",
                "local.flash_drum.create_outlet.flash-1.liquid",
                "local.flash_drum.create_outlet.flash-1.vapor",
            ]
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_dispatch_snapshot_aggregates_gui_facing_state() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        assert!(
            dispatch.snapshot.command_registry.sections.len() >= 1,
            "expected at least one command section in gui snapshot"
        );
        assert_eq!(
            dispatch.snapshot.canvas.primary_action().label,
            "Accept suggestion"
        );
        assert_eq!(dispatch.window.canvas.suggestion_count, 3);
        assert_eq!(
            dispatch.window.layout().default_focus_area,
            crate::StudioGuiWindowAreaId::Canvas
        );
        assert_eq!(
            dispatch
                .snapshot
                .command_registry
                .sections
                .first()
                .map(|section| section.title),
            Some("Run Panel")
        );
        assert_eq!(
            dispatch.snapshot.runtime.control_state.run_status,
            rf_ui::RunStatus::Idle
        );
        assert_eq!(
            dispatch
                .window
                .runtime
                .run_panel
                .view()
                .primary_action
                .label,
            "Resume"
        );
        assert!(dispatch.snapshot.runtime.entitlement_host.is_some());
        assert!(
            !dispatch
                .snapshot
                .runtime
                .run_panel
                .view()
                .primary_action
                .label
                .is_empty()
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_routes_network_restored_without_open_windows() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::NetworkRestored)
            .expect("expected lifecycle dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::LifecycleDispatched(lifecycle),
            ) => {
                assert!(lifecycle.dispatch.is_none());
            }
            other => panic!("expected lifecycle outcome, got {other:?}"),
        }
        assert!(dispatch.state.windows.is_empty());
    }

    #[test]
    fn gui_driver_routes_shortcut_into_ui_command_dispatch() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let _ = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Shift],
                    key: StudioGuiShortcutKey::F6,
                },
                focus_context: StudioGuiFocusContext::Global,
            })
            .expect("expected shortcut dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(executed),
                ),
            ) => match &executed.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::WorkspaceMode(mode) => {
                        assert_eq!(mode.simulation_mode, rf_ui::SimulationMode::Active);
                    }
                    other => panic!("expected workspace mode dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            },
            other => panic!("expected executed shortcut outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_automatic_runs_after_canvas_write_when_workspace_is_active() {
        let (config, project_path) = flash_drum_local_rules_synced_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        assert_eq!(
            driver
                .canvas_state()
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("local.flash_drum.create_outlet.flash-1.vapor")
        );
        let _ = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        let activate = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Shift],
                    key: StudioGuiShortcutKey::F6,
                },
                focus_context: StudioGuiFocusContext::Global,
            })
            .expect("expected activate shortcut dispatch");

        match &activate.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(executed),
                ),
            ) => match &executed.effects.runtime_report.dispatch {
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
            },
            other => panic!("expected executed activate shortcut outcome, got {other:?}"),
        }
        assert_eq!(
            driver
                .canvas_state()
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("local.flash_drum.create_outlet.flash-1.vapor")
        );

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
            })
            .expect("expected canvas acceptance dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                        command_id,
                        result,
                        ..
                    },
                ),
            ) => {
                assert_eq!(command_id, "canvas.accept_focused");
                assert_eq!(
                    result
                        .accepted
                        .as_ref()
                        .map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.vapor")
                );
                assert_eq!(
                    result
                        .latest_log_entry
                        .as_ref()
                        .map(|entry| entry.message.as_str()),
                    Some(
                        "Solved document revision 1 with property package `binary-hydrocarbon-lite-v1` into snapshot `example-feed-heater-flash-rev-1-seq-1`"
                    )
                );
            }
            other => panic!("expected executed canvas ui command outcome, got {other:?}"),
        }
        assert_eq!(
            dispatch.snapshot.runtime.control_state.run_status,
            rf_ui::RunStatus::Converged
        );
        assert_eq!(dispatch.snapshot.runtime.control_state.pending_reason, None);
        assert_eq!(
            dispatch
                .snapshot
                .runtime
                .control_state
                .latest_snapshot_id
                .as_deref(),
            Some("example-feed-heater-flash-rev-1-seq-1")
        );
        assert_eq!(
            dispatch.snapshot.runtime.run_panel.view().status_label,
            "Converged"
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_recovery_then_resume_rejoins_automatic_mainline() {
        let mut driver =
            StudioGuiDriver::new(&unbound_outlet_failure_synced_config()).expect("expected driver");
        let _ = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        let failed = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "run_panel.run_manual".to_string(),
            })
            .expect("expected failed run dispatch");
        match failed.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(executed),
                ),
            ) => match &executed.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::WorkspaceRun(run) => {
                        assert!(matches!(
                            run.outcome,
                            crate::StudioWorkspaceRunOutcome::Failed(_)
                        ));
                        assert_eq!(run.simulation_mode, rf_ui::SimulationMode::Hold);
                    }
                    other => panic!("expected workspace run dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            },
            other => panic!("expected executed failed run outcome, got {other:?}"),
        }

        let recovery = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::F8,
                },
                focus_context: StudioGuiFocusContext::Global,
            })
            .expect("expected recovery shortcut dispatch");
        match recovery.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(executed),
                ),
            ) => match &executed.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
                    assert_eq!(outcome.action.title, "Create outlet stream");
                    assert_eq!(
                        outcome.applied_target,
                        Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                            "feed-1"
                        )))
                    );
                }
                other => panic!("expected recovery dispatch, got {other:?}"),
            },
            other => panic!("expected executed recovery outcome, got {other:?}"),
        }
        assert_eq!(
            recovery.snapshot.runtime.control_state.run_status,
            rf_ui::RunStatus::Dirty
        );
        assert_eq!(
            recovery.snapshot.runtime.control_state.pending_reason,
            Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
        );
        assert_eq!(
            recovery.snapshot.runtime.control_state.simulation_mode,
            rf_ui::SimulationMode::Hold
        );
        assert_eq!(
            recovery
                .snapshot
                .runtime
                .run_panel
                .view()
                .primary_action
                .label,
            "Resume"
        );

        let resumed = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Shift],
                    key: StudioGuiShortcutKey::F5,
                },
                focus_context: StudioGuiFocusContext::Global,
            })
            .expect("expected resume shortcut dispatch");
        match resumed.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(executed),
                ),
            ) => match &executed.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::WorkspaceRun(run) => {
                        assert!(matches!(
                            run.outcome,
                            crate::StudioWorkspaceRunOutcome::Started(_)
                        ));
                        assert_eq!(run.simulation_mode, rf_ui::SimulationMode::Active);
                        assert_eq!(run.pending_reason, None);
                        assert_eq!(run.run_status, rf_ui::RunStatus::Converged);
                        assert_eq!(
                            run.latest_snapshot_id.as_deref(),
                            Some("example-unbound-outlet-port-rev-1-seq-1")
                        );
                    }
                    other => panic!("expected workspace run dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            },
            other => panic!("expected executed resume outcome, got {other:?}"),
        }
        assert_eq!(
            resumed.snapshot.runtime.control_state.run_status,
            rf_ui::RunStatus::Converged
        );
        assert_eq!(
            resumed
                .snapshot
                .runtime
                .control_state
                .latest_snapshot_id
                .as_deref(),
            Some("example-unbound-outlet-port-rev-1-seq-1")
        );
        assert_eq!(
            resumed.snapshot.runtime.run_panel.view().status_label,
            "Converged"
        );
    }

    #[test]
    fn gui_driver_ignores_canvas_tab_shortcut_without_canvas_command_binding() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
            })
            .expect("expected shortcut dispatch");

        assert_eq!(
            dispatch.outcome,
            StudioGuiDriverOutcome::IgnoredShortcut {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                reason: StudioGuiShortcutIgnoreReason::NoBindingFound,
            }
        );
    }

    #[test]
    fn gui_driver_reports_ignored_shortcut_when_text_input_owns_tab() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                focus_context: StudioGuiFocusContext::TextInput,
            })
            .expect("expected shortcut dispatch");

        assert_eq!(
            dispatch.outcome,
            StudioGuiDriverOutcome::IgnoredShortcut {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                reason: StudioGuiShortcutIgnoreReason::TextInputOwnsShortcut,
            }
        );
    }

    #[test]
    fn gui_driver_accepts_focused_canvas_suggestion_by_tab() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        driver.replace_canvas_suggestions(vec![sample_canvas_suggestion("sug-high", 0.95)]);

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
            })
            .expect("expected shortcut dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                        command_id,
                        result,
                        ..
                    },
                ),
            ) => {
                assert_eq!(command_id, "canvas.accept_focused");
                assert_eq!(
                    result.action,
                    StudioGuiCanvasInteractionAction::AcceptFocusedByTab
                );
                assert_eq!(
                    result
                        .accepted
                        .as_ref()
                        .map(|suggestion| suggestion.id.as_str()),
                    Some("sug-high")
                );
                assert_eq!(
                    result.applied_target,
                    Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                        "flash-1"
                    )))
                );
                assert_eq!(
                    result
                        .latest_log_entry
                        .as_ref()
                        .map(|entry| entry.message.as_str()),
                    Some("Accepted canvas suggestion `sug-high` from local rules for unit flash-1")
                );
            }
            other => panic!("expected executed canvas ui command outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_focuses_next_canvas_suggestion_from_explicit_event() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::CanvasSuggestionFocusNextRequested)
            .expect("expected focus-next dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::CanvasInteraction(result) => {
                assert_eq!(result.action, StudioGuiCanvasInteractionAction::FocusNext);
                assert_eq!(
                    result
                        .focused
                        .as_ref()
                        .map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
                assert_eq!(
                    dispatch
                        .canvas
                        .focused_suggestion_id
                        .as_ref()
                        .map(|id| id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
            }
            other => panic!("expected canvas interaction outcome, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_rejects_focused_canvas_suggestion_from_shortcut() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Escape,
                },
                focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
            })
            .expect("expected reject dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                        command_id,
                        result,
                        ..
                    },
                ),
            ) => {
                assert_eq!(command_id, "canvas.reject_focused");
                assert_eq!(
                    result.action,
                    StudioGuiCanvasInteractionAction::RejectFocused
                );
                assert_eq!(
                    result
                        .rejected
                        .as_ref()
                        .map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
                );
                assert_eq!(
                    result
                        .canvas
                        .focused_suggestion_id
                        .as_ref()
                        .map(|id| id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
                assert_eq!(
                    dispatch
                        .canvas
                        .focused_suggestion_id
                        .as_ref()
                        .map(|id| id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
            }
            other => panic!("expected executed canvas ui command outcome, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_routes_ctrl_tab_to_canvas_focus_next() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                    key: StudioGuiShortcutKey::Tab,
                },
                focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
            })
            .expect("expected shortcut dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                        command_id,
                        result,
                        ..
                    },
                ),
            ) => {
                assert_eq!(command_id, "canvas.focus_next");
                assert_eq!(result.action, StudioGuiCanvasInteractionAction::FocusNext);
                assert_eq!(
                    result
                        .focused
                        .as_ref()
                        .map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
            }
            other => panic!("expected executed canvas ui command outcome, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_updates_window_layout_and_preserves_per_window_overrides() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        let first = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected first open dispatch");
        let first_window_id = match first.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected first window opened outcome, got {other:?}"),
        };
        let second = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected second open dispatch");
        let second_window_id = match second.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected second window opened outcome, got {other:?}"),
        };

        let hidden_runtime = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::SetPanelVisibility {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    visible: false,
                },
            })
            .expect("expected layout visibility update");
        match hidden_runtime.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result
                        .layout_state
                        .panel(StudioGuiWindowAreaId::Runtime)
                        .map(|panel| panel.visible),
                    Some(false)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let collapsed_commands = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                    area_id: StudioGuiWindowAreaId::Commands,
                    collapsed: true,
                },
            })
            .expect("expected layout collapsed update");
        match collapsed_commands.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result
                        .layout_state
                        .panel(StudioGuiWindowAreaId::Commands)
                        .map(|panel| panel.collapsed),
                    Some(true)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let weighted = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::SetRegionWeight {
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    weight: 31,
                },
            })
            .expect("expected layout weight update");
        match weighted.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result
                        .layout_state
                        .region_weight(StudioGuiWindowDockRegion::RightSidebar)
                        .map(|region| region.weight),
                    Some(31)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let centered = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::SetCenterArea {
                    area_id: StudioGuiWindowAreaId::Runtime,
                },
            })
            .expect("expected layout center update");
        match centered.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result.layout_state.center_area,
                    StudioGuiWindowAreaId::Runtime
                );
                assert_eq!(
                    result
                        .layout_state
                        .panel(StudioGuiWindowAreaId::Runtime)
                        .map(|panel| panel.visible),
                    Some(true)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let reordered = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::SetPanelOrder {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    order: 5,
                },
            })
            .expect("expected layout order update");
        match reordered.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result
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
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let moved = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                    area_id: StudioGuiWindowAreaId::Commands,
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            })
            .expect("expected layout dock region update");
        match moved.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result
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
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let stacked = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::StackPanelWith {
                    area_id: StudioGuiWindowAreaId::Commands,
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            })
            .expect("expected layout stack update");
        let stacked_window_layout = stacked.window.layout();
        match &stacked.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result
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
                assert_eq!(
                    result
                        .layout_state
                        .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
                    Some(StudioGuiWindowAreaId::Commands)
                );
                assert_eq!(
                    stacked_window_layout
                        .panel(StudioGuiWindowAreaId::Commands)
                        .map(|panel| panel.display_mode),
                    Some(crate::StudioGuiWindowPanelDisplayMode::ActiveTab)
                );
                assert_eq!(
                    stacked_window_layout
                        .panel(StudioGuiWindowAreaId::Runtime)
                        .map(|panel| panel.display_mode),
                    Some(crate::StudioGuiWindowPanelDisplayMode::InactiveTab)
                );
                assert_eq!(
                    stacked_window_layout
                        .stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
                        .map(|group| group.tabbed),
                    Some(true)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let activated_next = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::ActivateNextPanelInStack {
                    area_id: StudioGuiWindowAreaId::Commands,
                },
            })
            .expect("expected activate-next update");
        match &activated_next.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(
                    result
                        .layout_state
                        .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
                    Some(StudioGuiWindowAreaId::Runtime)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let reordered_stack = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::MovePanelWithinStack {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Commands,
                    },
                },
            })
            .expect("expected stack reorder update");
        match &reordered_stack.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(
                    result
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
                assert_eq!(
                    result
                        .layout_state
                        .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
                    Some(StudioGuiWindowAreaId::Runtime)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let unstacked = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
                    area_id: StudioGuiWindowAreaId::Commands,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            })
            .expect("expected unstack update");
        match &unstacked.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(
                    result
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
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let first_window = driver.window_model_for_window(Some(first_window_id));
        let second_window = driver.window_model_for_window(Some(second_window_id));

        assert_eq!(
            first_window
                .layout_state
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| panel.visible),
            Some(true)
        );
        assert_eq!(
            second_window
                .layout_state
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| panel.visible),
            Some(true)
        );
        assert_eq!(
            second_window
                .layout_state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| {
                    (
                        panel.collapsed,
                        panel.dock_region,
                        panel.stack_group,
                        panel.order,
                    )
                }),
            Some((true, StudioGuiWindowDockRegion::RightSidebar, 10, 10))
        );
        assert_eq!(
            first_window
                .layout_state
                .region_weight(StudioGuiWindowDockRegion::RightSidebar)
                .map(|region| region.weight),
            Some(24)
        );
        assert_eq!(
            second_window
                .layout_state
                .region_weight(StudioGuiWindowDockRegion::RightSidebar)
                .map(|region| region.weight),
            Some(31)
        );
        assert_eq!(
            first_window.layout_state.center_area,
            StudioGuiWindowAreaId::Canvas
        );
        assert_eq!(
            second_window.layout_state.center_area,
            StudioGuiWindowAreaId::Runtime
        );
        assert_eq!(
            second_window
                .layout_state
                .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
                .into_iter()
                .map(|panel| (
                    panel.area_id,
                    panel.dock_region,
                    panel.stack_group,
                    panel.order
                ))
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
            second_window
                .layout_state
                .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
            Some(StudioGuiWindowAreaId::Commands)
        );
        assert_eq!(
            second_window
                .layout_state
                .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 20),
            Some(StudioGuiWindowAreaId::Runtime)
        );

        let layout_path = rf_store::studio_layout_path_for_project(&project_path);
        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_routes_drop_target_queries_without_mutating_layout_state() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };

        let queried = driver
            .dispatch_event(StudioGuiEvent::WindowDropTargetQueryRequested {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::Stack {
                    area_id: StudioGuiWindowAreaId::Commands,
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            })
            .expect("expected drop target query dispatch");

        match queried.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::WindowDropTargetQueried(result),
            ) => {
                let target = result.drop_target.expect("expected stack preview target");
                assert_eq!(result.target_window_id, Some(window_id));
                assert_eq!(target.target_stack_group, 10);
                assert_eq!(target.target_tab_index, 0);
                assert_eq!(
                    result.preview_layout_state.as_ref().and_then(|layout| {
                        layout
                            .panel(StudioGuiWindowAreaId::Commands)
                            .map(|panel| (panel.dock_region, panel.stack_group, panel.order))
                    }),
                    Some((StudioGuiWindowDockRegion::RightSidebar, 10, 10))
                );
                assert_eq!(
                    result.preview_window.as_ref().and_then(|window| {
                        window
                            .layout_state
                            .panel(StudioGuiWindowAreaId::Runtime)
                            .map(|panel| (panel.dock_region, panel.stack_group, panel.order))
                    }),
                    Some((StudioGuiWindowDockRegion::RightSidebar, 10, 20))
                );
                assert_eq!(
                    result
                        .layout_state
                        .panel(StudioGuiWindowAreaId::Commands)
                        .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
                    Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
                );
            }
            other => panic!("expected drop target query outcome, got {other:?}"),
        }

        assert_eq!(
            queried
                .window
                .layout_state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
        );
        assert_eq!(queried.window.drop_preview, None);
        let window = driver.window_model_for_window(Some(window_id));
        assert_eq!(window.drop_preview, None);
        assert_eq!(
            window
                .layout_state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
        );
        assert_eq!(
            window
                .layout_state
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::RightSidebar, 10, 30))
        );

        let layout_path = rf_store::studio_layout_path_for_project(&project_path);
        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_routes_drop_preview_updates_through_single_event_entry() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };

        let previewed = driver
            .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::Stack {
                    area_id: StudioGuiWindowAreaId::Commands,
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            })
            .expect("expected drop preview dispatch");

        match previewed.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(result),
            ) => {
                assert_eq!(result.target_window_id, Some(window_id));
                assert!(result.drop_target.is_some());
                assert!(result.preview_layout_state.is_some());
            }
            other => panic!("expected drop preview outcome, got {other:?}"),
        }

        let preview = previewed
            .window
            .drop_preview
            .as_ref()
            .expect("expected drop preview in dispatch window");
        assert_eq!(
            preview.overlay.drag_area_id,
            StudioGuiWindowAreaId::Commands
        );
        assert_eq!(
            preview.overlay.target_dock_region,
            StudioGuiWindowDockRegion::RightSidebar
        );
        assert_eq!(preview.overlay.target_stack_group, 10);
        assert_eq!(preview.overlay.target_tab_index, 0);
        assert_eq!(
            preview.overlay.target_stack_area_ids,
            vec![
                StudioGuiWindowAreaId::Commands,
                StudioGuiWindowAreaId::Runtime
            ]
        );
        assert_eq!(
            preview.overlay.target_stack_active_area_id,
            StudioGuiWindowAreaId::Commands
        );
        assert_eq!(preview.drop_target.target_stack_group, 10);
        assert_eq!(
            preview
                .preview_layout
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::RightSidebar, 10, 10))
        );
        assert_eq!(
            preview.changed_area_ids,
            vec![
                StudioGuiWindowAreaId::Commands,
                StudioGuiWindowAreaId::Runtime
            ]
        );
        assert_eq!(
            preview
                .preview_layout_state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::RightSidebar, 10, 10))
        );
        assert_eq!(
            previewed
                .window
                .layout_state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
        );
        assert!(
            driver
                .window_model_for_window(Some(window_id))
                .drop_preview
                .is_some()
        );

        let layout_path = rf_store::studio_layout_path_for_project(&project_path);
        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_clears_drop_preview_through_single_event_entry() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let _ = driver
            .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect("expected drop preview dispatch");

        let cleared = driver
            .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewCleared {
                window_id: Some(window_id),
            })
            .expect("expected drop preview clear dispatch");

        match cleared.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(result),
            ) => {
                assert_eq!(result.target_window_id, Some(window_id));
                assert!(result.had_preview);
            }
            other => panic!("expected drop preview clear outcome, got {other:?}"),
        }
        assert_eq!(cleared.window.drop_preview, None);
        assert_eq!(
            driver.window_model_for_window(Some(window_id)).drop_preview,
            None
        );

        let layout_path = rf_store::studio_layout_path_for_project(&project_path);
        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_applies_drop_target_queries_through_single_event_entry() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let _ = driver
            .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect("expected drop preview dispatch");

        let applied = driver
            .dispatch_event(StudioGuiEvent::WindowDropTargetApplyRequested {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect("expected drop target apply dispatch");

        match applied.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::WindowDropTargetApplied(result),
            ) => {
                assert_eq!(result.target_window_id, Some(window_id));
                assert_eq!(
                    result.mutation,
                    StudioGuiWindowDropTargetQuery::DockRegion {
                        area_id: StudioGuiWindowAreaId::Runtime,
                        dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                        placement: StudioGuiWindowDockPlacement::Start,
                    }
                    .layout_mutation()
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

        assert_eq!(
            applied
                .window
                .layout_state
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
            Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
        );
        assert_eq!(applied.window.drop_preview, None);

        let layout_path = rf_store::studio_layout_path_for_project(&project_path);
        let _ = fs::remove_file(layout_path);
        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_rejects_layout_update_for_unknown_window() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let error = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(99),
                mutation: StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                    area_id: StudioGuiWindowAreaId::Commands,
                    collapsed: true,
                },
            })
            .expect_err("expected invalid layout target");

        assert_eq!(error.code().as_str(), "invalid_input");
    }

    #[test]
    fn gui_driver_rejects_drop_target_query_for_unknown_window() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let error = driver
            .dispatch_event(StudioGuiEvent::WindowDropTargetQueryRequested {
                window_id: Some(99),
                query: StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                },
            })
            .expect_err("expected invalid drop target query target");

        assert_eq!(error.code().as_str(), "invalid_input");
    }

    #[test]
    fn gui_driver_rejects_inapplicable_drop_target_apply() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };

        let error = driver
            .dispatch_event(StudioGuiEvent::WindowDropTargetApplyRequested {
                window_id: Some(window_id),
                query: StudioGuiWindowDropTargetQuery::Unstack {
                    area_id: StudioGuiWindowAreaId::Commands,
                    placement: StudioGuiWindowDockPlacement::End,
                },
            })
            .expect_err("expected invalid drop target apply");

        assert_eq!(error.code().as_str(), "invalid_input");
    }

    #[test]
    fn gui_driver_tracks_parked_timer_restore_when_reopening_owner_window() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let first_window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };

        let _ = driver
            .dispatch_event(StudioGuiEvent::EntitlementTimerElapsed)
            .expect("expected timer elapsed dispatch");
        let closed = driver
            .dispatch_event(StudioGuiEvent::CloseWindowRequested {
                window_id: first_window_id,
            })
            .expect("expected close dispatch");
        match closed.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(
                closed,
            )) => {
                assert!(
                    closed
                        .native_timers
                        .operations
                        .iter()
                        .any(|operation| matches!(
                            operation,
                            crate::StudioGuiNativeTimerOperation::Park { .. }
                        ))
                );
            }
            other => panic!("expected window closed outcome, got {other:?}"),
        }
        assert!(driver.native_timer_runtime().parked_binding().is_some());

        let reopened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected reopen dispatch");
        let second_window_id = match reopened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => {
                assert!(
                    opened
                        .native_timers
                        .operations
                        .iter()
                        .any(|operation| matches!(
                            operation,
                            crate::StudioGuiNativeTimerOperation::RestoreParked { .. }
                        ))
                );
                opened.registration.window_id
            }
            other => panic!("expected reopened window outcome, got {other:?}"),
        };

        assert!(driver.native_timer_runtime().parked_binding().is_none());
        assert!(
            driver
                .native_timer_runtime()
                .window_binding(second_window_id)
                .is_some()
        );
    }

    #[test]
    fn gui_driver_drains_due_native_timer_events_through_lifecycle_entry() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let _ = driver
            .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                    crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer trigger");

        let due_at = driver
            .next_due_native_timer_at()
            .expect("expected scheduled native timer");
        let due_dispatches = driver
            .drain_due_native_timer_events(due_at)
            .expect("expected due timer dispatch");

        assert_eq!(due_dispatches.len(), 1);
        assert!(matches!(
            due_dispatches[0].outcome,
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
                _
            ))
        ));
        assert!(driver.next_due_native_timer_at().is_some());
    }

    #[test]
    fn gui_driver_routes_native_timer_callback_for_current_binding() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let _ = driver
            .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                    crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer trigger");
        let binding = driver
            .native_timer_runtime()
            .window_binding(window_id)
            .cloned()
            .expect("expected native timer binding");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::NativeTimerElapsed {
                window_id: Some(window_id),
                handle_id: binding.handle_id,
            })
            .expect("expected native timer callback dispatch");

        assert!(matches!(
            dispatch.outcome,
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
                _
            ))
        ));
    }

    #[test]
    fn gui_driver_ignores_unknown_native_timer_callback_handle() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let _ = driver
            .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                    crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected first timer trigger");
        let current_handle_id = driver
            .native_timer_runtime()
            .window_binding(window_id)
            .map(|binding| binding.handle_id)
            .expect("expected initial binding");

        let ignored = driver
            .dispatch_event(StudioGuiEvent::NativeTimerElapsed {
                window_id: Some(window_id),
                handle_id: current_handle_id + 999,
            })
            .expect("expected unknown native timer callback");

        assert!(matches!(
            ignored.outcome,
            StudioGuiDriverOutcome::IgnoredNativeTimerElapsed {
                window_id: Some(_),
                ..
            }
        ));
    }
}
