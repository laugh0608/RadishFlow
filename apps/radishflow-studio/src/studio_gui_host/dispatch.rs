use super::helpers::{dispatch_from_controller, global_event_from_lifecycle};
use super::*;

impl StudioGuiHost {
    pub fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        self.dispatch_canvas_interaction(StudioGuiCanvasInteractionAction::AcceptFocusedByTab)
    }

    pub fn reject_focused_canvas_suggestion(
        &mut self,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        self.dispatch_canvas_interaction(StudioGuiCanvasInteractionAction::RejectFocused)
    }

    pub fn focus_next_canvas_suggestion(
        &mut self,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        self.dispatch_canvas_interaction(StudioGuiCanvasInteractionAction::FocusNext)
    }

    pub fn focus_previous_canvas_suggestion(
        &mut self,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        self.dispatch_canvas_interaction(StudioGuiCanvasInteractionAction::FocusPrevious)
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
        if let Some(target) = crate::inspector_target_from_command_id(command_id) {
            let Some(target_window_id) = self.preferred_target_window_id() else {
                return Ok(StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                    command_id: command_id.to_string(),
                    detail: "Open a studio window before focusing an inspector target".to_string(),
                    target_window_id: None,
                    ui_commands: self.ui_commands(),
                });
            };
            let dispatch = self.controller.dispatch_window_trigger(
                target_window_id,
                StudioRuntimeTrigger::InspectorTarget(target),
            )?;
            return Ok(StudioGuiHostUiCommandDispatchResult::Executed(
                dispatch_from_controller(dispatch, self.canvas_state()),
            ));
        }

        if let Some(action_id) = canvas_action_id_from_command_id(command_id) {
            let target_window_id = self.preferred_target_window_id();
            let canvas = self.canvas_state();
            if canvas.suggestions.is_empty() {
                return Ok(StudioGuiHostUiCommandDispatchResult::IgnoredMissing {
                    command_id: command_id.to_string(),
                    ui_commands: self.ui_commands(),
                });
            }
            let action_entry = canvas
                .widget()
                .action(action_id)
                .cloned()
                .expect("canvas widget should expose command-backed actions");
            if !action_entry.enabled {
                return Ok(StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                    command_id: action_entry.command_id.to_string(),
                    detail: action_entry.detail.to_string(),
                    target_window_id,
                    ui_commands: self.ui_commands(),
                });
            }
            let action = match action_id {
                crate::StudioGuiCanvasActionId::AcceptFocused => {
                    StudioGuiCanvasInteractionAction::AcceptFocusedByTab
                }
                crate::StudioGuiCanvasActionId::RejectFocused => {
                    StudioGuiCanvasInteractionAction::RejectFocused
                }
                crate::StudioGuiCanvasActionId::FocusNext => {
                    StudioGuiCanvasInteractionAction::FocusNext
                }
                crate::StudioGuiCanvasActionId::FocusPrevious => {
                    StudioGuiCanvasInteractionAction::FocusPrevious
                }
            };
            let mut result = self.dispatch_canvas_interaction(action)?;
            result.ui_commands = self.ui_commands();
            result.canvas = self.canvas_state();
            return Ok(
                StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                    command_id: action_entry.command_id.to_string(),
                    target_window_id,
                    result,
                },
            );
        }

        let registry = self.command_registry();
        let ui_commands = self.ui_commands();
        let Some(command) = registry.command(command_id).cloned() else {
            return Ok(StudioGuiHostUiCommandDispatchResult::IgnoredMissing {
                command_id: command_id.to_string(),
                ui_commands,
            });
        };

        if !command.enabled {
            return Ok(StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                command_id: command.command_id,
                detail: command.detail,
                target_window_id: command.target_window_id,
                ui_commands,
            });
        }

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

    pub fn dispatch_inspector_draft_update(
        &mut self,
        command_id: &str,
        raw_value: impl Into<String>,
    ) -> RfResult<StudioGuiHostDispatch> {
        let command = crate::inspector_draft_update_command_from_id(command_id, raw_value)
            .ok_or_else(|| {
                RfError::invalid_input(format!(
                    "inspector draft update command `{command_id}` is not supported"
                ))
            })?;
        let target_window_id = self.preferred_target_window_id().ok_or_else(|| {
            RfError::invalid_input("open a studio window before updating inspector draft")
        })?;
        let dispatch = self.controller.dispatch_window_trigger(
            target_window_id,
            StudioRuntimeTrigger::InspectorDraftUpdate(command),
        )?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub(super) fn preferred_target_window_id(&self) -> Option<StudioWindowHostId> {
        self.state()
            .foreground_window_id
            .or_else(|| self.state().registered_windows.first().copied())
    }

    pub(super) fn build_canvas_interaction_result_with_focus(
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

    pub(super) fn dispatch_canvas_interaction(
        &mut self,
        action: StudioGuiCanvasInteractionAction,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        let result = self.controller.dispatch_canvas_interaction(action)?;
        Ok(self.build_canvas_interaction_result_with_focus(
            action,
            result.accepted,
            result.rejected,
            result.focused,
        ))
    }
}
