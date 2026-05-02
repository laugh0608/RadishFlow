use crate::{
    StudioGuiCanvasPresentation, StudioGuiCanvasState, StudioGuiEvent, StudioGuiShortcut,
    StudioGuiShortcutKey, StudioGuiShortcutModifier,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiCanvasActionId {
    BeginPlaceFlashDrum,
    AcceptFocused,
    RejectFocused,
    FocusNext,
    FocusPrevious,
    CancelPendingEdit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasRenderableAction {
    pub id: StudioGuiCanvasActionId,
    pub command_id: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
    pub enabled: bool,
    pub shortcut: Option<StudioGuiShortcut>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StudioGuiCanvasWidgetEvent {
    Requested {
        action_id: StudioGuiCanvasActionId,
        event: StudioGuiEvent,
    },
    Disabled {
        action_id: StudioGuiCanvasActionId,
    },
    Missing {
        action_id: StudioGuiCanvasActionId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasWidgetModel {
    pub presentation: StudioGuiCanvasPresentation,
    pub actions: Vec<StudioGuiCanvasRenderableAction>,
}

impl StudioGuiCanvasWidgetModel {
    pub fn from_state(state: &StudioGuiCanvasState) -> Self {
        let presentation = state.presentation();
        let suggestion_count = presentation.view.suggestion_count;
        let focused = presentation
            .view
            .suggestions
            .iter()
            .find(|item| item.is_focused);
        let has_focus = focused.is_some();
        let can_accept = focused
            .as_ref()
            .map(|suggestion| suggestion.tab_accept_enabled)
            .unwrap_or(false);
        let can_cycle_focus = suggestion_count > 1;
        let can_cancel_pending_edit = presentation
            .view
            .pending_edit
            .as_ref()
            .map(|pending| pending.cancel_enabled)
            .unwrap_or(false);
        let can_begin_place_unit = presentation.view.pending_edit.is_none();

        Self {
            presentation,
            actions: vec![
                StudioGuiCanvasRenderableAction {
                    id: StudioGuiCanvasActionId::BeginPlaceFlashDrum,
                    command_id: canvas_command_id(StudioGuiCanvasActionId::BeginPlaceFlashDrum),
                    label: "Place Flash Drum",
                    detail: "Start placing a Flash Drum on the canvas",
                    enabled: can_begin_place_unit,
                    shortcut: None,
                },
                StudioGuiCanvasRenderableAction {
                    id: StudioGuiCanvasActionId::AcceptFocused,
                    command_id: canvas_command_id(StudioGuiCanvasActionId::AcceptFocused),
                    label: "Accept suggestion",
                    detail: "Apply the currently focused canvas suggestion",
                    enabled: can_accept,
                    shortcut: Some(StudioGuiShortcut {
                        modifiers: Vec::new(),
                        key: StudioGuiShortcutKey::Tab,
                    }),
                },
                StudioGuiCanvasRenderableAction {
                    id: StudioGuiCanvasActionId::RejectFocused,
                    command_id: canvas_command_id(StudioGuiCanvasActionId::RejectFocused),
                    label: "Reject suggestion",
                    detail: "Dismiss the currently focused canvas suggestion",
                    enabled: has_focus,
                    shortcut: Some(StudioGuiShortcut {
                        modifiers: Vec::new(),
                        key: StudioGuiShortcutKey::Escape,
                    }),
                },
                StudioGuiCanvasRenderableAction {
                    id: StudioGuiCanvasActionId::FocusNext,
                    command_id: canvas_command_id(StudioGuiCanvasActionId::FocusNext),
                    label: "Next suggestion",
                    detail: "Move focus to the next available canvas suggestion",
                    enabled: can_cycle_focus,
                    shortcut: Some(StudioGuiShortcut {
                        modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                        key: StudioGuiShortcutKey::Tab,
                    }),
                },
                StudioGuiCanvasRenderableAction {
                    id: StudioGuiCanvasActionId::FocusPrevious,
                    command_id: canvas_command_id(StudioGuiCanvasActionId::FocusPrevious),
                    label: "Previous suggestion",
                    detail: "Move focus to the previous available canvas suggestion",
                    enabled: can_cycle_focus,
                    shortcut: Some(StudioGuiShortcut {
                        modifiers: vec![
                            StudioGuiShortcutModifier::Ctrl,
                            StudioGuiShortcutModifier::Shift,
                        ],
                        key: StudioGuiShortcutKey::Tab,
                    }),
                },
                StudioGuiCanvasRenderableAction {
                    id: StudioGuiCanvasActionId::CancelPendingEdit,
                    command_id: canvas_command_id(StudioGuiCanvasActionId::CancelPendingEdit),
                    label: "Cancel pending edit",
                    detail: "Cancel the current canvas edit intent",
                    enabled: can_cancel_pending_edit,
                    shortcut: None,
                },
            ],
        }
    }

    pub fn view(&self) -> &crate::StudioGuiCanvasViewModel {
        &self.presentation.view
    }

    pub fn text(&self) -> &crate::StudioGuiCanvasTextView {
        &self.presentation.text
    }

    pub fn action(&self, id: StudioGuiCanvasActionId) -> Option<&StudioGuiCanvasRenderableAction> {
        self.actions.iter().find(|action| action.id == id)
    }

    pub fn primary_action(&self) -> &StudioGuiCanvasRenderableAction {
        self.action(StudioGuiCanvasActionId::AcceptFocused)
            .expect("canvas widget should expose accept action")
    }

    pub fn activate_primary(&self) -> StudioGuiCanvasWidgetEvent {
        self.activate(self.primary_action().id)
    }

    pub fn activate(&self, id: StudioGuiCanvasActionId) -> StudioGuiCanvasWidgetEvent {
        match self.action(id) {
            Some(action) if !action.enabled => {
                StudioGuiCanvasWidgetEvent::Disabled { action_id: id }
            }
            Some(_) => StudioGuiCanvasWidgetEvent::Requested {
                action_id: id,
                event: action_event(id),
            },
            None => StudioGuiCanvasWidgetEvent::Missing { action_id: id },
        }
    }
}

impl StudioGuiCanvasState {
    pub fn widget(&self) -> StudioGuiCanvasWidgetModel {
        StudioGuiCanvasWidgetModel::from_state(self)
    }
}

fn action_event(action_id: StudioGuiCanvasActionId) -> StudioGuiEvent {
    StudioGuiEvent::UiCommandRequested {
        command_id: canvas_command_id(action_id).to_string(),
    }
}

pub(crate) fn canvas_command_id(action_id: StudioGuiCanvasActionId) -> &'static str {
    match action_id {
        StudioGuiCanvasActionId::BeginPlaceFlashDrum => "canvas.begin_place_unit.flash_drum",
        StudioGuiCanvasActionId::AcceptFocused => "canvas.accept_focused",
        StudioGuiCanvasActionId::RejectFocused => "canvas.reject_focused",
        StudioGuiCanvasActionId::FocusNext => "canvas.focus_next",
        StudioGuiCanvasActionId::FocusPrevious => "canvas.focus_previous",
        StudioGuiCanvasActionId::CancelPendingEdit => "canvas.cancel_pending_edit",
    }
}

pub(crate) fn canvas_action_id_from_command_id(
    command_id: &str,
) -> Option<StudioGuiCanvasActionId> {
    match command_id {
        "canvas.begin_place_unit.flash_drum" => Some(StudioGuiCanvasActionId::BeginPlaceFlashDrum),
        "canvas.accept_focused" => Some(StudioGuiCanvasActionId::AcceptFocused),
        "canvas.reject_focused" => Some(StudioGuiCanvasActionId::RejectFocused),
        "canvas.focus_next" => Some(StudioGuiCanvasActionId::FocusNext),
        "canvas.focus_previous" => Some(StudioGuiCanvasActionId::FocusPrevious),
        "canvas.cancel_pending_edit" => Some(StudioGuiCanvasActionId::CancelPendingEdit),
        _ => None,
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
        StudioGuiCanvasActionId, StudioGuiCanvasInteractionAction, StudioGuiCanvasWidgetEvent,
        StudioGuiDriver, StudioGuiDriverOutcome, StudioGuiEvent, StudioRuntimeConfig,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
    };
    use rf_ui::{
        GhostElement, GhostElementKind, StreamVisualKind, StreamVisualState, SuggestionSource,
    };

    fn lease_expiring_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        }
    }

    fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-studio-canvas-widget-{timestamp}.rfproj.json"
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
            "widget sample",
        )
    }

    #[test]
    fn canvas_widget_enables_full_action_set_for_local_rules_focus() {
        let (config, project_path) = flash_drum_local_rules_config();
        let driver = StudioGuiDriver::new(&config).expect("expected driver");

        let widget = driver.canvas_state().widget();

        assert!(
            widget
                .action(StudioGuiCanvasActionId::BeginPlaceFlashDrum)
                .expect("expected begin place action")
                .enabled
        );
        assert!(widget.primary_action().enabled);
        assert!(
            widget
                .action(StudioGuiCanvasActionId::RejectFocused)
                .expect("expected reject action")
                .enabled
        );
        assert!(
            widget
                .action(StudioGuiCanvasActionId::FocusNext)
                .expect("expected next action")
                .enabled
        );
        assert!(
            widget
                .action(StudioGuiCanvasActionId::FocusPrevious)
                .expect("expected previous action")
                .enabled
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn canvas_widget_disables_accept_and_focus_cycle_when_not_available() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        driver.replace_canvas_suggestions(vec![sample_canvas_suggestion("sug-low", 0.6)]);

        let widget = driver.canvas_state().widget();

        assert!(
            widget
                .action(StudioGuiCanvasActionId::BeginPlaceFlashDrum)
                .expect("expected begin place action")
                .enabled
        );
        assert!(!widget.primary_action().enabled);
        assert!(
            widget
                .action(StudioGuiCanvasActionId::RejectFocused)
                .expect("expected reject action")
                .enabled
        );
        assert!(
            !widget
                .action(StudioGuiCanvasActionId::FocusNext)
                .expect("expected next action")
                .enabled
        );
        assert!(
            !widget
                .action(StudioGuiCanvasActionId::FocusPrevious)
                .expect("expected previous action")
                .enabled
        );
        assert!(
            !widget
                .action(StudioGuiCanvasActionId::CancelPendingEdit)
                .expect("expected cancel action")
                .enabled
        );
    }

    #[test]
    fn canvas_widget_enables_cancel_for_pending_canvas_edit() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        driver
            .begin_canvas_place_unit("Flash Drum")
            .expect("expected begin canvas place unit");

        let widget = driver.canvas_state().widget();

        assert!(
            !widget
                .action(StudioGuiCanvasActionId::BeginPlaceFlashDrum)
                .expect("expected begin place action")
                .enabled
        );
        assert!(
            widget
                .action(StudioGuiCanvasActionId::CancelPendingEdit)
                .expect("expected cancel action")
                .enabled
        );
    }

    #[test]
    fn canvas_widget_maps_actions_to_explicit_driver_events() {
        let (config, project_path) = flash_drum_local_rules_config();
        let driver = StudioGuiDriver::new(&config).expect("expected driver");
        let widget = driver.canvas_state().widget();

        assert_eq!(
            widget.activate(StudioGuiCanvasActionId::BeginPlaceFlashDrum),
            StudioGuiCanvasWidgetEvent::Requested {
                action_id: StudioGuiCanvasActionId::BeginPlaceFlashDrum,
                event: StudioGuiEvent::UiCommandRequested {
                    command_id: "canvas.begin_place_unit.flash_drum".to_string(),
                },
            }
        );
        assert_eq!(
            widget.activate_primary(),
            StudioGuiCanvasWidgetEvent::Requested {
                action_id: StudioGuiCanvasActionId::AcceptFocused,
                event: StudioGuiEvent::UiCommandRequested {
                    command_id: "canvas.accept_focused".to_string(),
                },
            }
        );
        assert_eq!(
            widget.activate(StudioGuiCanvasActionId::FocusNext),
            StudioGuiCanvasWidgetEvent::Requested {
                action_id: StudioGuiCanvasActionId::FocusNext,
                event: StudioGuiEvent::UiCommandRequested {
                    command_id: "canvas.focus_next".to_string(),
                },
            }
        );
        assert_eq!(
            widget.activate(StudioGuiCanvasActionId::FocusPrevious),
            StudioGuiCanvasWidgetEvent::Requested {
                action_id: StudioGuiCanvasActionId::FocusPrevious,
                event: StudioGuiEvent::UiCommandRequested {
                    command_id: "canvas.focus_previous".to_string(),
                },
            }
        );
        assert_eq!(
            widget.activate(StudioGuiCanvasActionId::RejectFocused),
            StudioGuiCanvasWidgetEvent::Requested {
                action_id: StudioGuiCanvasActionId::RejectFocused,
                event: StudioGuiEvent::UiCommandRequested {
                    command_id: "canvas.reject_focused".to_string(),
                },
            }
        );
        assert_eq!(
            widget.activate(StudioGuiCanvasActionId::CancelPendingEdit),
            StudioGuiCanvasWidgetEvent::Disabled {
                action_id: StudioGuiCanvasActionId::CancelPendingEdit,
            }
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn canvas_widget_returns_disabled_event_for_unavailable_accept_action() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        driver.replace_canvas_suggestions(vec![sample_canvas_suggestion("sug-low", 0.6)]);

        let widget = driver.canvas_state().widget();

        assert_eq!(
            widget.activate_primary(),
            StudioGuiCanvasWidgetEvent::Disabled {
                action_id: StudioGuiCanvasActionId::AcceptFocused,
            }
        );
    }

    #[test]
    fn canvas_widget_requested_event_dispatches_through_driver() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        let widget = driver.canvas_state().widget();
        let event = match widget.activate(StudioGuiCanvasActionId::FocusNext) {
            StudioGuiCanvasWidgetEvent::Requested { event, .. } => event,
            other => panic!("expected requested widget event, got {other:?}"),
        };

        let dispatch = driver
            .dispatch_event(event)
            .expect("expected driver dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                crate::StudioGuiHostCommandOutcome::UiCommandDispatched(
                    crate::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
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
            other => panic!("expected canvas ui command outcome, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn canvas_widget_cancel_event_dispatches_through_driver() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        driver
            .begin_canvas_place_unit("Flash Drum")
            .expect("expected begin canvas place unit");
        let widget = driver.canvas_state().widget();
        let event = match widget.activate(StudioGuiCanvasActionId::CancelPendingEdit) {
            StudioGuiCanvasWidgetEvent::Requested { event, .. } => event,
            other => panic!("expected requested widget event, got {other:?}"),
        };

        let dispatch = driver
            .dispatch_event(event)
            .expect("expected driver dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                crate::StudioGuiHostCommandOutcome::UiCommandDispatched(
                    crate::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                        command_id,
                        result,
                        ..
                    },
                ),
            ) => {
                assert_eq!(command_id, "canvas.cancel_pending_edit");
                assert_eq!(
                    result.action,
                    StudioGuiCanvasInteractionAction::CancelPendingEdit
                );
                assert_eq!(result.committed_edit, None);
                assert_eq!(result.canvas.pending_edit, None);
                assert_eq!(dispatch.canvas.pending_edit, None);
            }
            other => panic!("expected canvas ui command outcome, got {other:?}"),
        }

        assert!(
            !dispatch
                .snapshot
                .runtime
                .workspace_document
                .has_unsaved_changes
        );
    }
}
