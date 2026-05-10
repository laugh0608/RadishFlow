use crate::{
    StudioGuiCanvasPlaceUnitKind, StudioGuiCanvasPresentation, StudioGuiCanvasState,
    StudioGuiEvent, StudioGuiShortcut, StudioGuiShortcutKey, StudioGuiShortcutModifier,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiCanvasActionId {
    BeginPlaceUnit(StudioGuiCanvasPlaceUnitKind),
    AcceptFocused,
    RejectFocused,
    FocusNext,
    FocusPrevious,
    CancelPendingEdit,
    MoveSelectedUnit(StudioGuiCanvasUnitLayoutNudgeDirection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StudioGuiCanvasUnitLayoutNudgeDirection {
    Left,
    Right,
    Up,
    Down,
}

impl StudioGuiCanvasUnitLayoutNudgeDirection {
    pub const fn all() -> &'static [Self] {
        &[Self::Left, Self::Up, Self::Down, Self::Right]
    }

    pub const fn command_id(self) -> &'static str {
        match self {
            Self::Left => "canvas.move_selected_unit.left",
            Self::Right => "canvas.move_selected_unit.right",
            Self::Up => "canvas.move_selected_unit.up",
            Self::Down => "canvas.move_selected_unit.down",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Left => "Move left",
            Self::Right => "Move right",
            Self::Up => "Move up",
            Self::Down => "Move down",
        }
    }

    pub const fn detail(self) -> &'static str {
        match self {
            Self::Left => "Move the selected unit left in the canvas layout sidecar",
            Self::Right => "Move the selected unit right in the canvas layout sidecar",
            Self::Up => "Move the selected unit up in the canvas layout sidecar",
            Self::Down => "Move the selected unit down in the canvas layout sidecar",
        }
    }

    pub const fn search_term(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasRenderableAction {
    pub id: StudioGuiCanvasActionId,
    pub command_id: String,
    pub label: String,
    pub detail: String,
    pub enabled: bool,
    pub shortcut: Option<StudioGuiShortcut>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StudioGuiCanvasWidgetEvent {
    Requested {
        action_id: StudioGuiCanvasActionId,
        event: StudioGuiEvent,
    },
    SuggestionRequested {
        suggestion_id: String,
        event: StudioGuiEvent,
    },
    Disabled {
        action_id: StudioGuiCanvasActionId,
    },
    SuggestionDisabled {
        suggestion_id: String,
    },
    Missing {
        action_id: StudioGuiCanvasActionId,
    },
    SuggestionMissing {
        suggestion_id: String,
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
        let selected_unit = presentation
            .view
            .current_selection
            .as_ref()
            .filter(|selection| selection.kind_label == "Unit");
        let mut actions = presentation
            .view
            .place_unit_palette
            .options
            .iter()
            .map(|option| StudioGuiCanvasRenderableAction {
                id: StudioGuiCanvasActionId::BeginPlaceUnit(option.kind),
                command_id: option.command_id.clone(),
                label: option.label.clone(),
                detail: option.detail.clone(),
                enabled: option.enabled,
                shortcut: None,
            })
            .collect::<Vec<_>>();

        actions.extend([
            StudioGuiCanvasRenderableAction {
                id: StudioGuiCanvasActionId::AcceptFocused,
                command_id: canvas_command_id(StudioGuiCanvasActionId::AcceptFocused).to_string(),
                label: "Accept suggestion".to_string(),
                detail: "Apply the currently focused canvas suggestion".to_string(),
                enabled: can_accept,
                shortcut: Some(StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                }),
            },
            StudioGuiCanvasRenderableAction {
                id: StudioGuiCanvasActionId::RejectFocused,
                command_id: canvas_command_id(StudioGuiCanvasActionId::RejectFocused).to_string(),
                label: "Reject suggestion".to_string(),
                detail: "Dismiss the currently focused canvas suggestion".to_string(),
                enabled: has_focus,
                shortcut: Some(StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Escape,
                }),
            },
            StudioGuiCanvasRenderableAction {
                id: StudioGuiCanvasActionId::FocusNext,
                command_id: canvas_command_id(StudioGuiCanvasActionId::FocusNext).to_string(),
                label: "Next suggestion".to_string(),
                detail: "Move focus to the next available canvas suggestion".to_string(),
                enabled: can_cycle_focus,
                shortcut: Some(StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                    key: StudioGuiShortcutKey::Tab,
                }),
            },
            StudioGuiCanvasRenderableAction {
                id: StudioGuiCanvasActionId::FocusPrevious,
                command_id: canvas_command_id(StudioGuiCanvasActionId::FocusPrevious).to_string(),
                label: "Previous suggestion".to_string(),
                detail: "Move focus to the previous available canvas suggestion".to_string(),
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
                command_id: canvas_command_id(StudioGuiCanvasActionId::CancelPendingEdit)
                    .to_string(),
                label: "Cancel pending edit".to_string(),
                detail: "Cancel the current canvas edit intent".to_string(),
                enabled: can_cancel_pending_edit,
                shortcut: None,
            },
        ]);
        actions.extend(StudioGuiCanvasUnitLayoutNudgeDirection::all().iter().map(
            |direction| {
                let detail = match selected_unit {
                    Some(selection) => format!(
                        "{} `{}` by one layout step; if no sidecar position exists, pin it from its current transient grid slot first.",
                        direction.detail(),
                        selection.target_id
                    ),
                    None => format!("{}; select a unit first.", direction.detail()),
                };
                StudioGuiCanvasRenderableAction {
                    id: StudioGuiCanvasActionId::MoveSelectedUnit(*direction),
                    command_id: direction.command_id().to_string(),
                    label: direction.label().to_string(),
                    detail,
                    enabled: selected_unit.is_some(),
                    shortcut: None,
                }
            },
        ));

        Self {
            presentation,
            actions,
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

    pub fn activate_suggestion(&self, suggestion_id: &str) -> StudioGuiCanvasWidgetEvent {
        match self
            .presentation
            .view
            .suggestions
            .iter()
            .find(|suggestion| suggestion.id == suggestion_id)
        {
            Some(suggestion) if !suggestion.explicit_accept_enabled => {
                StudioGuiCanvasWidgetEvent::SuggestionDisabled {
                    suggestion_id: suggestion_id.to_string(),
                }
            }
            Some(_) => StudioGuiCanvasWidgetEvent::SuggestionRequested {
                suggestion_id: suggestion_id.to_string(),
                event: StudioGuiEvent::CanvasSuggestionAcceptByIdRequested {
                    suggestion_id: rf_ui::CanvasSuggestionId::new(suggestion_id),
                },
            },
            None => StudioGuiCanvasWidgetEvent::SuggestionMissing {
                suggestion_id: suggestion_id.to_string(),
            },
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
        StudioGuiCanvasActionId::BeginPlaceUnit(kind) => kind.command_id(),
        StudioGuiCanvasActionId::AcceptFocused => "canvas.accept_focused",
        StudioGuiCanvasActionId::RejectFocused => "canvas.reject_focused",
        StudioGuiCanvasActionId::FocusNext => "canvas.focus_next",
        StudioGuiCanvasActionId::FocusPrevious => "canvas.focus_previous",
        StudioGuiCanvasActionId::CancelPendingEdit => "canvas.cancel_pending_edit",
        StudioGuiCanvasActionId::MoveSelectedUnit(direction) => direction.command_id(),
    }
}

pub(crate) fn canvas_action_id_from_command_id(
    command_id: &str,
) -> Option<StudioGuiCanvasActionId> {
    if let Some(kind) = StudioGuiCanvasPlaceUnitKind::from_command_id(command_id) {
        return Some(StudioGuiCanvasActionId::BeginPlaceUnit(kind));
    }

    match command_id {
        "canvas.accept_focused" => Some(StudioGuiCanvasActionId::AcceptFocused),
        "canvas.reject_focused" => Some(StudioGuiCanvasActionId::RejectFocused),
        "canvas.focus_next" => Some(StudioGuiCanvasActionId::FocusNext),
        "canvas.focus_previous" => Some(StudioGuiCanvasActionId::FocusPrevious),
        "canvas.cancel_pending_edit" => Some(StudioGuiCanvasActionId::CancelPendingEdit),
        "canvas.move_selected_unit.left" => Some(StudioGuiCanvasActionId::MoveSelectedUnit(
            StudioGuiCanvasUnitLayoutNudgeDirection::Left,
        )),
        "canvas.move_selected_unit.right" => Some(StudioGuiCanvasActionId::MoveSelectedUnit(
            StudioGuiCanvasUnitLayoutNudgeDirection::Right,
        )),
        "canvas.move_selected_unit.up" => Some(StudioGuiCanvasActionId::MoveSelectedUnit(
            StudioGuiCanvasUnitLayoutNudgeDirection::Up,
        )),
        "canvas.move_selected_unit.down" => Some(StudioGuiCanvasActionId::MoveSelectedUnit(
            StudioGuiCanvasUnitLayoutNudgeDirection::Down,
        )),
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
        StudioGuiCanvasActionId, StudioGuiCanvasInteractionAction, StudioGuiCanvasPlaceUnitKind,
        StudioGuiCanvasUnitLayoutNudgeDirection, StudioGuiCanvasWidgetEvent, StudioGuiDriver,
        StudioGuiDriverOutcome, StudioGuiEvent, StudioRuntimeConfig,
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
        let project = crate::test_support::build_flash_drum_local_rules_project_json();
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

        assert_eq!(
            widget
                .actions
                .iter()
                .filter_map(|action| match action.id {
                    StudioGuiCanvasActionId::BeginPlaceUnit(kind) =>
                        Some((kind, action.command_id.as_str(), action.enabled,)),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![
                (
                    StudioGuiCanvasPlaceUnitKind::Feed,
                    "canvas.begin_place_unit.feed",
                    true
                ),
                (
                    StudioGuiCanvasPlaceUnitKind::Mixer,
                    "canvas.begin_place_unit.mixer",
                    true
                ),
                (
                    StudioGuiCanvasPlaceUnitKind::Heater,
                    "canvas.begin_place_unit.heater",
                    true
                ),
                (
                    StudioGuiCanvasPlaceUnitKind::Cooler,
                    "canvas.begin_place_unit.cooler",
                    true
                ),
                (
                    StudioGuiCanvasPlaceUnitKind::Valve,
                    "canvas.begin_place_unit.valve",
                    true
                ),
                (
                    StudioGuiCanvasPlaceUnitKind::FlashDrum,
                    "canvas.begin_place_unit.flash_drum",
                    true
                ),
            ]
        );
        assert!(
            widget
                .action(StudioGuiCanvasActionId::BeginPlaceUnit(
                    StudioGuiCanvasPlaceUnitKind::FlashDrum,
                ))
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
                .action(StudioGuiCanvasActionId::BeginPlaceUnit(
                    StudioGuiCanvasPlaceUnitKind::FlashDrum,
                ))
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
                .action(StudioGuiCanvasActionId::BeginPlaceUnit(
                    StudioGuiCanvasPlaceUnitKind::FlashDrum,
                ))
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
            widget.activate(StudioGuiCanvasActionId::BeginPlaceUnit(
                StudioGuiCanvasPlaceUnitKind::FlashDrum,
            )),
            StudioGuiCanvasWidgetEvent::Requested {
                action_id: StudioGuiCanvasActionId::BeginPlaceUnit(
                    StudioGuiCanvasPlaceUnitKind::FlashDrum,
                ),
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
    fn canvas_widget_exposes_selected_unit_layout_nudge_actions() {
        let canvas = crate::StudioGuiCanvasState {
            units: vec![crate::StudioGuiCanvasUnitState {
                unit_id: rf_types::UnitId::new("feed-1"),
                name: "Feed".to_string(),
                kind: "feed".to_string(),
                layout_position: None,
                ports: Vec::new(),
                port_count: 0,
                connected_port_count: 0,
                is_active_inspector_target: true,
            }],
            ..crate::StudioGuiCanvasState::default()
        };
        let widget = canvas.widget();

        for direction in StudioGuiCanvasUnitLayoutNudgeDirection::all() {
            let action = widget
                .action(StudioGuiCanvasActionId::MoveSelectedUnit(*direction))
                .expect("expected selected unit layout nudge action");
            assert_eq!(action.command_id, direction.command_id());
            assert_eq!(action.label, direction.label());
            assert!(action.enabled);
            assert!(action.detail.contains("feed-1"));
            assert!(
                action
                    .detail
                    .contains("pin it from its current transient grid slot")
            );
        }
        assert_eq!(
            widget.activate(StudioGuiCanvasActionId::MoveSelectedUnit(
                StudioGuiCanvasUnitLayoutNudgeDirection::Left,
            )),
            StudioGuiCanvasWidgetEvent::Requested {
                action_id: StudioGuiCanvasActionId::MoveSelectedUnit(
                    StudioGuiCanvasUnitLayoutNudgeDirection::Left,
                ),
                event: StudioGuiEvent::UiCommandRequested {
                    command_id: "canvas.move_selected_unit.left".to_string(),
                },
            }
        );
    }

    #[test]
    fn canvas_widget_maps_explicit_suggestion_acceptance_to_driver_event() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
        let widget = driver.canvas_state().widget();
        let suggestion = widget
            .view()
            .suggestions
            .iter()
            .find(|suggestion| suggestion.id == "local.flash_drum.create_outlet.flash-1.liquid")
            .expect("expected liquid outlet suggestion");

        assert!(suggestion.explicit_accept_enabled);
        assert_eq!(
            suggestion.explicit_accept_command_id,
            "canvas.accept_suggestion.local.flash_drum.create_outlet.flash-1.liquid"
        );

        let event = match widget.activate_suggestion(&suggestion.id) {
            StudioGuiCanvasWidgetEvent::SuggestionRequested { event, .. } => event,
            other => panic!("expected explicit suggestion request, got {other:?}"),
        };
        let dispatch = driver
            .dispatch_event(event)
            .expect("expected explicit suggestion dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::CanvasInteraction(result) => {
                assert_eq!(
                    result.action,
                    StudioGuiCanvasInteractionAction::AcceptById {
                        suggestion_id: rf_ui::CanvasSuggestionId::new(
                            "local.flash_drum.create_outlet.flash-1.liquid"
                        ),
                    }
                );
                assert_eq!(
                    result
                        .accepted
                        .as_ref()
                        .map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
            }
            other => panic!("expected direct canvas interaction outcome, got {other:?}"),
        }

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
