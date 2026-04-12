use crate::{
    StudioGuiCanvasActionId, StudioGuiCommandRegistry, StudioGuiShortcut,
    studio_gui_canvas_widget::canvas_command_id,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiFocusContext {
    Global,
    Canvas,
    CanvasSuggestionFocused,
    InspectorPanel,
    TextInput,
    CommandPalette,
    ModalDialog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiShortcutRoute {
    DispatchCommandId {
        command_id: String,
    },
    Ignored {
        reason: StudioGuiShortcutIgnoreReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiShortcutIgnoreReason {
    NoBindingFound,
    TextInputOwnsShortcut,
    CommandPaletteOwnsShortcut,
    ModalDialogOwnsShortcut,
    NoCanvasSuggestionFocused,
}

pub fn route_shortcut(
    registry: &StudioGuiCommandRegistry,
    shortcut: &StudioGuiShortcut,
    focus_context: StudioGuiFocusContext,
) -> StudioGuiShortcutRoute {
    if is_canvas_accept_shortcut(shortcut) {
        return match focus_context {
            StudioGuiFocusContext::CanvasSuggestionFocused => dispatch_canvas_command_shortcut(
                registry,
                shortcut,
                canvas_command_id(StudioGuiCanvasActionId::AcceptFocused),
            ),
            StudioGuiFocusContext::TextInput => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::TextInputOwnsShortcut,
            },
            StudioGuiFocusContext::CommandPalette => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::CommandPaletteOwnsShortcut,
            },
            StudioGuiFocusContext::ModalDialog => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::ModalDialogOwnsShortcut,
            },
            StudioGuiFocusContext::Global
            | StudioGuiFocusContext::Canvas
            | StudioGuiFocusContext::InspectorPanel => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::NoCanvasSuggestionFocused,
            },
        };
    }

    if is_canvas_focus_next_shortcut(shortcut) {
        return match focus_context {
            StudioGuiFocusContext::CanvasSuggestionFocused => dispatch_canvas_command_shortcut(
                registry,
                shortcut,
                canvas_command_id(StudioGuiCanvasActionId::FocusNext),
            ),
            StudioGuiFocusContext::TextInput => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::TextInputOwnsShortcut,
            },
            StudioGuiFocusContext::CommandPalette => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::CommandPaletteOwnsShortcut,
            },
            StudioGuiFocusContext::ModalDialog => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::ModalDialogOwnsShortcut,
            },
            StudioGuiFocusContext::Global
            | StudioGuiFocusContext::Canvas
            | StudioGuiFocusContext::InspectorPanel => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::NoCanvasSuggestionFocused,
            },
        };
    }

    if is_canvas_focus_previous_shortcut(shortcut) {
        return match focus_context {
            StudioGuiFocusContext::CanvasSuggestionFocused => dispatch_canvas_command_shortcut(
                registry,
                shortcut,
                canvas_command_id(StudioGuiCanvasActionId::FocusPrevious),
            ),
            StudioGuiFocusContext::TextInput => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::TextInputOwnsShortcut,
            },
            StudioGuiFocusContext::CommandPalette => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::CommandPaletteOwnsShortcut,
            },
            StudioGuiFocusContext::ModalDialog => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::ModalDialogOwnsShortcut,
            },
            StudioGuiFocusContext::Global
            | StudioGuiFocusContext::Canvas
            | StudioGuiFocusContext::InspectorPanel => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::NoCanvasSuggestionFocused,
            },
        };
    }

    if is_canvas_reject_shortcut(shortcut) {
        return match focus_context {
            StudioGuiFocusContext::CanvasSuggestionFocused => dispatch_canvas_command_shortcut(
                registry,
                shortcut,
                canvas_command_id(StudioGuiCanvasActionId::RejectFocused),
            ),
            StudioGuiFocusContext::TextInput => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::TextInputOwnsShortcut,
            },
            StudioGuiFocusContext::CommandPalette => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::CommandPaletteOwnsShortcut,
            },
            StudioGuiFocusContext::ModalDialog => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::ModalDialogOwnsShortcut,
            },
            StudioGuiFocusContext::Global
            | StudioGuiFocusContext::Canvas
            | StudioGuiFocusContext::InspectorPanel => StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::NoCanvasSuggestionFocused,
            },
        };
    }

    match focus_context {
        StudioGuiFocusContext::CommandPalette => StudioGuiShortcutRoute::Ignored {
            reason: StudioGuiShortcutIgnoreReason::CommandPaletteOwnsShortcut,
        },
        StudioGuiFocusContext::ModalDialog => StudioGuiShortcutRoute::Ignored {
            reason: StudioGuiShortcutIgnoreReason::ModalDialogOwnsShortcut,
        },
        StudioGuiFocusContext::Global
        | StudioGuiFocusContext::Canvas
        | StudioGuiFocusContext::CanvasSuggestionFocused
        | StudioGuiFocusContext::InspectorPanel
        | StudioGuiFocusContext::TextInput => registry
            .find_by_shortcut(shortcut)
            .map(|entry| StudioGuiShortcutRoute::DispatchCommandId {
                command_id: entry.command_id.clone(),
            })
            .unwrap_or(StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::NoBindingFound,
            }),
    }
}

fn is_canvas_accept_shortcut(shortcut: &StudioGuiShortcut) -> bool {
    shortcut.modifiers.is_empty() && matches!(shortcut.key, crate::StudioGuiShortcutKey::Tab)
}

fn is_canvas_focus_next_shortcut(shortcut: &StudioGuiShortcut) -> bool {
    has_exact_modifiers(shortcut, &[crate::StudioGuiShortcutModifier::Ctrl])
        && matches!(shortcut.key, crate::StudioGuiShortcutKey::Tab)
}

fn is_canvas_focus_previous_shortcut(shortcut: &StudioGuiShortcut) -> bool {
    has_exact_modifiers(
        shortcut,
        &[
            crate::StudioGuiShortcutModifier::Ctrl,
            crate::StudioGuiShortcutModifier::Shift,
        ],
    ) && matches!(shortcut.key, crate::StudioGuiShortcutKey::Tab)
}

fn is_canvas_reject_shortcut(shortcut: &StudioGuiShortcut) -> bool {
    shortcut.modifiers.is_empty() && matches!(shortcut.key, crate::StudioGuiShortcutKey::Escape)
}

fn has_exact_modifiers(
    shortcut: &StudioGuiShortcut,
    expected: &[crate::StudioGuiShortcutModifier],
) -> bool {
    let mut actual = shortcut.modifiers.clone();
    actual.sort();

    let mut expected = expected.to_vec();
    expected.sort();

    actual == expected
}

fn dispatch_canvas_command_shortcut(
    registry: &StudioGuiCommandRegistry,
    shortcut: &StudioGuiShortcut,
    command_id: &str,
) -> StudioGuiShortcutRoute {
    match registry.find_by_shortcut(shortcut) {
        Some(entry) if entry.command_id == command_id => {
            StudioGuiShortcutRoute::DispatchCommandId {
                command_id: command_id.to_string(),
            }
        }
        _ => StudioGuiShortcutRoute::Ignored {
            reason: StudioGuiShortcutIgnoreReason::NoBindingFound,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        StudioAppHostUiActionModel, StudioAppHostUiCommandGroup, StudioAppHostUiCommandModel,
        StudioGuiCommandRegistry, StudioGuiFocusContext, StudioGuiShortcut,
        StudioGuiShortcutIgnoreReason, StudioGuiShortcutKey, StudioGuiShortcutModifier,
        StudioGuiShortcutRoute, route_shortcut,
    };

    fn registry() -> StudioGuiCommandRegistry {
        let mut canvas = crate::StudioGuiCanvasState::default();
        canvas.suggestions = vec![rf_ui::CanvasSuggestion::new(
            rf_ui::CanvasSuggestionId::new("sug-a"),
            rf_ui::SuggestionSource::LocalRules,
            0.9,
            rf_ui::GhostElement {
                kind: rf_ui::GhostElementKind::Connection,
                target_unit_id: rf_types::UnitId::new("flash-1"),
                visual_kind: rf_ui::StreamVisualKind::Material,
                visual_state: rf_ui::StreamVisualState::Suggested,
            },
            "test",
        )];
        canvas.focused_suggestion_id = Some(rf_ui::CanvasSuggestionId::new("sug-a"));

        StudioGuiCommandRegistry::from_surfaces(
            &StudioAppHostUiCommandModel {
                actions: vec![StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.run_manual",
                    group: StudioAppHostUiCommandGroup::RunPanel,
                    sort_order: 100,
                    label: "Run workspace",
                    enabled: true,
                    detail: "Run",
                    target_window_id: Some(1),
                }],
            },
            &canvas,
            Some(1),
        )
    }

    #[test]
    fn shortcut_router_dispatches_bound_function_shortcuts_from_text_input() {
        let route = route_shortcut(
            &registry(),
            &StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::F5,
            },
            StudioGuiFocusContext::TextInput,
        );

        assert_eq!(
            route,
            StudioGuiShortcutRoute::DispatchCommandId {
                command_id: "run_panel.run_manual".to_string(),
            }
        );
    }

    #[test]
    fn shortcut_router_routes_tab_to_canvas_accept_when_suggestion_is_focused() {
        let route = route_shortcut(
            &registry(),
            &StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::Tab,
            },
            StudioGuiFocusContext::CanvasSuggestionFocused,
        );

        assert_eq!(
            route,
            StudioGuiShortcutRoute::DispatchCommandId {
                command_id: "canvas.accept_focused".to_string(),
            }
        );
    }

    #[test]
    fn shortcut_router_routes_ctrl_tab_to_canvas_focus_next() {
        let route = route_shortcut(
            &registry(),
            &StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                key: StudioGuiShortcutKey::Tab,
            },
            StudioGuiFocusContext::CanvasSuggestionFocused,
        );

        assert_eq!(
            route,
            StudioGuiShortcutRoute::DispatchCommandId {
                command_id: "canvas.focus_next".to_string(),
            }
        );
    }

    #[test]
    fn shortcut_router_routes_ctrl_shift_tab_to_canvas_focus_previous() {
        let route = route_shortcut(
            &registry(),
            &StudioGuiShortcut {
                modifiers: vec![
                    StudioGuiShortcutModifier::Ctrl,
                    StudioGuiShortcutModifier::Shift,
                ],
                key: StudioGuiShortcutKey::Tab,
            },
            StudioGuiFocusContext::CanvasSuggestionFocused,
        );

        assert_eq!(
            route,
            StudioGuiShortcutRoute::DispatchCommandId {
                command_id: "canvas.focus_previous".to_string(),
            }
        );
    }

    #[test]
    fn shortcut_router_routes_escape_to_canvas_reject_when_suggestion_is_focused() {
        let route = route_shortcut(
            &registry(),
            &StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::Escape,
            },
            StudioGuiFocusContext::CanvasSuggestionFocused,
        );

        assert_eq!(
            route,
            StudioGuiShortcutRoute::DispatchCommandId {
                command_id: "canvas.reject_focused".to_string(),
            }
        );
    }

    #[test]
    fn shortcut_router_blocks_tab_inside_text_input() {
        let route = route_shortcut(
            &registry(),
            &StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::Tab,
            },
            StudioGuiFocusContext::TextInput,
        );

        assert_eq!(
            route,
            StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::TextInputOwnsShortcut,
            }
        );
    }

    #[test]
    fn shortcut_router_ignores_unbound_shortcut_when_no_binding_exists() {
        let route = route_shortcut(
            &registry(),
            &StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                key: StudioGuiShortcutKey::F8,
            },
            StudioGuiFocusContext::Global,
        );

        assert_eq!(
            route,
            StudioGuiShortcutRoute::Ignored {
                reason: StudioGuiShortcutIgnoreReason::NoBindingFound,
            }
        );
    }
}
