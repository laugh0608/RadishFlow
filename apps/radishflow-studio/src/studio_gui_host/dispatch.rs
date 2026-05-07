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

    pub fn begin_canvas_place_unit(
        &mut self,
        unit_kind: impl Into<String>,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        self.dispatch_canvas_interaction(StudioGuiCanvasInteractionAction::BeginPlaceUnit {
            unit_kind: unit_kind.into(),
        })
    }

    pub fn cancel_canvas_pending_edit(&mut self) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        self.dispatch_canvas_interaction(StudioGuiCanvasInteractionAction::CancelPendingEdit)
    }

    pub fn commit_canvas_pending_edit_at(
        &mut self,
        position: rf_ui::CanvasPoint,
    ) -> RfResult<StudioGuiHostCanvasInteractionResult> {
        self.dispatch_canvas_interaction(StudioGuiCanvasInteractionAction::CommitPendingEditAt {
            position,
        })
    }

    pub fn move_canvas_unit_layout(
        &mut self,
        unit_id: rf_types::UnitId,
        position: rf_ui::CanvasPoint,
    ) -> RfResult<StudioGuiHostCanvasUnitLayoutMoveResult> {
        if !self
            .controller
            .document()
            .flowsheet
            .units
            .contains_key(&unit_id)
        {
            return Err(RfError::invalid_input(format!(
                "cannot move canvas layout for missing unit `{}`",
                unit_id.as_str()
            )));
        }

        let previous_position = self.canvas_unit_positions.get(&unit_id).copied();
        self.record_canvas_unit_position(&unit_id, position)?;
        Ok(StudioGuiHostCanvasUnitLayoutMoveResult {
            unit_id,
            previous_position,
            position,
            ui_commands: self.ui_commands(),
            canvas: self.canvas_state(),
        })
    }

    pub fn move_selected_canvas_unit_layout(
        &mut self,
        direction: crate::StudioGuiCanvasUnitLayoutNudgeDirection,
    ) -> RfResult<StudioGuiHostCanvasUnitLayoutMoveResult> {
        let Some(unit) = self
            .canvas_state()
            .units
            .iter()
            .enumerate()
            .find(|(_, unit)| unit.is_active_inspector_target)
            .map(|(layout_slot, unit)| {
                (
                    unit.unit_id.clone(),
                    unit.layout_position
                        .unwrap_or_else(|| transient_canvas_grid_position(layout_slot)),
                )
            })
        else {
            return Err(RfError::invalid_input(
                "cannot move canvas layout because no unit is selected",
            ));
        };

        self.move_canvas_unit_layout(unit.0, nudged_canvas_position(unit.1, direction))
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
                crate::StudioGuiCanvasActionId::BeginPlaceUnit(kind) => {
                    StudioGuiCanvasInteractionAction::BeginPlaceUnit {
                        unit_kind: kind.unit_kind().to_string(),
                    }
                }
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
                crate::StudioGuiCanvasActionId::CancelPendingEdit => {
                    StudioGuiCanvasInteractionAction::CancelPendingEdit
                }
                crate::StudioGuiCanvasActionId::MoveSelectedUnit(direction) => {
                    let result = self.move_selected_canvas_unit_layout(direction)?;
                    return Ok(
                        StudioGuiHostUiCommandDispatchResult::ExecutedCanvasUnitLayoutMove {
                            command_id: action_entry.command_id.to_string(),
                            target_window_id,
                            result,
                        },
                    );
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

    pub fn dispatch_inspector_draft_commit(
        &mut self,
        command_id: &str,
    ) -> RfResult<StudioGuiHostDispatch> {
        let command =
            crate::inspector_draft_commit_command_from_id(command_id).ok_or_else(|| {
                RfError::invalid_input(format!(
                    "inspector draft commit command `{command_id}` is not supported"
                ))
            })?;
        let target_window_id = self.preferred_target_window_id().ok_or_else(|| {
            RfError::invalid_input("open a studio window before committing inspector draft")
        })?;
        let dispatch = self.controller.dispatch_window_trigger(
            target_window_id,
            StudioRuntimeTrigger::InspectorDraftCommit(command),
        )?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn dispatch_inspector_draft_discard(
        &mut self,
        command_id: &str,
    ) -> RfResult<StudioGuiHostDispatch> {
        let command =
            crate::inspector_draft_discard_command_from_id(command_id).ok_or_else(|| {
                RfError::invalid_input(format!(
                    "inspector draft discard command `{command_id}` is not supported"
                ))
            })?;
        let target_window_id = self.preferred_target_window_id().ok_or_else(|| {
            RfError::invalid_input("open a studio window before discarding inspector draft")
        })?;
        let dispatch = self.controller.dispatch_window_trigger(
            target_window_id,
            StudioRuntimeTrigger::InspectorDraftDiscard(command),
        )?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn dispatch_inspector_draft_batch_commit(
        &mut self,
        command_id: &str,
    ) -> RfResult<StudioGuiHostDispatch> {
        let command =
            crate::inspector_draft_batch_commit_command_from_id(command_id).ok_or_else(|| {
                RfError::invalid_input(format!(
                    "inspector draft batch commit command `{command_id}` is not supported"
                ))
            })?;
        let target_window_id = self.preferred_target_window_id().ok_or_else(|| {
            RfError::invalid_input("open a studio window before committing inspector drafts")
        })?;
        let dispatch = self.controller.dispatch_window_trigger(
            target_window_id,
            StudioRuntimeTrigger::InspectorDraftBatchCommit(command),
        )?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn dispatch_inspector_draft_batch_discard(
        &mut self,
        command_id: &str,
    ) -> RfResult<StudioGuiHostDispatch> {
        let command =
            crate::inspector_draft_batch_discard_command_from_id(command_id).ok_or_else(|| {
                RfError::invalid_input(format!(
                    "inspector draft batch discard command `{command_id}` is not supported"
                ))
            })?;
        let target_window_id = self.preferred_target_window_id().ok_or_else(|| {
            RfError::invalid_input("open a studio window before discarding inspector drafts")
        })?;
        let dispatch = self.controller.dispatch_window_trigger(
            target_window_id,
            StudioRuntimeTrigger::InspectorDraftBatchDiscard(command),
        )?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn dispatch_inspector_composition_normalize(
        &mut self,
        command_id: &str,
    ) -> RfResult<StudioGuiHostDispatch> {
        let command = crate::inspector_composition_normalize_command_from_id(command_id)
            .ok_or_else(|| {
                RfError::invalid_input(format!(
                    "inspector composition normalize command `{command_id}` is not supported"
                ))
            })?;
        let target_window_id = self.preferred_target_window_id().ok_or_else(|| {
            RfError::invalid_input("open a studio window before normalizing stream composition")
        })?;
        let dispatch = self.controller.dispatch_window_trigger(
            target_window_id,
            StudioRuntimeTrigger::InspectorCompositionNormalize(command),
        )?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn dispatch_inspector_composition_component_add(
        &mut self,
        command_id: &str,
    ) -> RfResult<StudioGuiHostDispatch> {
        let command = crate::inspector_composition_component_add_command_from_id(command_id)
            .ok_or_else(|| {
                RfError::invalid_input(format!(
                    "inspector composition component add command `{command_id}` is not supported"
                ))
            })?;
        let target_window_id = self.preferred_target_window_id().ok_or_else(|| {
            RfError::invalid_input(
                "open a studio window before adding stream composition component",
            )
        })?;
        let dispatch = self.controller.dispatch_window_trigger(
            target_window_id,
            StudioRuntimeTrigger::InspectorCompositionComponentAdd(command),
        )?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn dispatch_inspector_composition_component_remove(
        &mut self,
        command_id: &str,
    ) -> RfResult<StudioGuiHostDispatch> {
        let command = crate::inspector_composition_component_remove_command_from_id(command_id)
            .ok_or_else(|| {
                RfError::invalid_input(format!(
                    "inspector composition component remove command `{command_id}` is not supported"
                ))
            })?;
        let target_window_id = self.preferred_target_window_id().ok_or_else(|| {
            RfError::invalid_input(
                "open a studio window before removing stream composition component",
            )
        })?;
        let dispatch = self.controller.dispatch_window_trigger(
            target_window_id,
            StudioRuntimeTrigger::InspectorCompositionComponentRemove(command),
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
        committed_edit: Option<rf_ui::CanvasEditCommitResult>,
        accepted: Option<CanvasSuggestion>,
        rejected: Option<CanvasSuggestion>,
        focused: Option<CanvasSuggestion>,
    ) -> StudioGuiHostCanvasInteractionResult {
        let applied_document_change = accepted.is_some() || committed_edit.is_some();
        StudioGuiHostCanvasInteractionResult {
            action,
            applied_target: applied_document_change
                .then(|| self.controller.active_inspector_target())
                .flatten(),
            latest_log_entry: applied_document_change
                .then(|| self.controller.latest_log_entry())
                .flatten(),
            committed_edit,
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
        let result = self
            .controller
            .dispatch_canvas_interaction(action.clone())?;
        if let Some(committed) = result.committed_edit.as_ref() {
            self.record_canvas_unit_position(&committed.unit_id, committed.position)?;
        }
        Ok(self.build_canvas_interaction_result_with_focus(
            action,
            result.committed_edit,
            result.accepted,
            result.rejected,
            result.focused,
        ))
    }

    fn record_canvas_unit_position(
        &mut self,
        unit_id: &rf_types::UnitId,
        position: rf_ui::CanvasPoint,
    ) -> RfResult<()> {
        self.canvas_unit_positions.insert(unit_id.clone(), position);
        let flowsheet = &self.controller.document().flowsheet;
        self.canvas_unit_positions
            .retain(|unit_id, _| flowsheet.units.contains_key(unit_id));
        match self.controller.document_path() {
            Some(project_path) => {
                save_persisted_canvas_unit_positions(project_path, &self.canvas_unit_positions)
            }
            None => Ok(()),
        }
    }
}

fn transient_canvas_grid_position(layout_slot: usize) -> rf_ui::CanvasPoint {
    const LEFT_PADDING: f64 = 18.0;
    const TOP_PADDING: f64 = 72.0;
    const BLOCK_WIDTH: f64 = 156.0;
    const BLOCK_HEIGHT: f64 = 72.0;
    const GAP_X: f64 = 22.0;
    const GAP_Y: f64 = 20.0;
    const FALLBACK_COLUMNS: usize = 3;

    let column = layout_slot % FALLBACK_COLUMNS;
    let row = layout_slot / FALLBACK_COLUMNS;
    rf_ui::CanvasPoint::new(
        LEFT_PADDING + column as f64 * (BLOCK_WIDTH + GAP_X),
        TOP_PADDING + row as f64 * (BLOCK_HEIGHT + GAP_Y),
    )
}

fn nudged_canvas_position(
    position: rf_ui::CanvasPoint,
    direction: crate::StudioGuiCanvasUnitLayoutNudgeDirection,
) -> rf_ui::CanvasPoint {
    const STEP: f64 = 40.0;

    match direction {
        crate::StudioGuiCanvasUnitLayoutNudgeDirection::Left => {
            rf_ui::CanvasPoint::new((position.x - STEP).max(0.0), position.y)
        }
        crate::StudioGuiCanvasUnitLayoutNudgeDirection::Right => {
            rf_ui::CanvasPoint::new(position.x + STEP, position.y)
        }
        crate::StudioGuiCanvasUnitLayoutNudgeDirection::Up => {
            rf_ui::CanvasPoint::new(position.x, (position.y - STEP).max(0.0))
        }
        crate::StudioGuiCanvasUnitLayoutNudgeDirection::Down => {
            rf_ui::CanvasPoint::new(position.x, position.y + STEP)
        }
    }
}
