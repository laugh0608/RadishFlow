use crate::{StudioGuiCommandRegistry, StudioGuiShortcut};

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
    RequestCanvasSuggestionAccept,
    RequestCanvasSuggestionReject,
    RequestCanvasSuggestionFocusNext,
    RequestCanvasSuggestionFocusPrevious,
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
            StudioGuiFocusContext::CanvasSuggestionFocused => {
                StudioGuiShortcutRoute::RequestCanvasSuggestionAccept
            }
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
            StudioGuiFocusContext::CanvasSuggestionFocused => {
                StudioGuiShortcutRoute::RequestCanvasSuggestionFocusNext
            }
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
            StudioGuiFocusContext::CanvasSuggestionFocused => {
                StudioGuiShortcutRoute::RequestCanvasSuggestionFocusPrevious
            }
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
            StudioGuiFocusContext::CanvasSuggestionFocused => {
                StudioGuiShortcutRoute::RequestCanvasSuggestionReject
            }
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

#[cfg(test)]
mod tests {
    use crate::{
        StudioAppHostUiActionModel, StudioAppHostUiCommandGroup, StudioAppHostUiCommandModel,
        StudioGuiCommandRegistry, StudioGuiFocusContext, StudioGuiShortcut,
        StudioGuiShortcutIgnoreReason, StudioGuiShortcutKey, StudioGuiShortcutModifier,
        StudioGuiShortcutRoute, route_shortcut,
    };

    fn registry() -> StudioGuiCommandRegistry {
        StudioGuiCommandRegistry::from_model(&StudioAppHostUiCommandModel {
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
        })
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
            StudioGuiShortcutRoute::RequestCanvasSuggestionAccept
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

        assert_eq!(route, StudioGuiShortcutRoute::RequestCanvasSuggestionFocusNext);
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
            StudioGuiShortcutRoute::RequestCanvasSuggestionFocusPrevious
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

        assert_eq!(route, StudioGuiShortcutRoute::RequestCanvasSuggestionReject);
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
