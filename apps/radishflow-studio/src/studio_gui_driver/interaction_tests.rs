use std::fs;

use super::test_support::{
    assert_ignored_shortcut, flash_drum_local_rules_config, lease_expiring_config,
    sample_canvas_suggestion,
};
use super::*;

#[test]
fn gui_driver_ignores_canvas_tab_shortcut_without_canvas_command_binding() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::ShortcutPressed {
            shortcut: StudioGuiShortcut {
                modifiers: Vec::new(),
                key: crate::StudioGuiShortcutKey::Tab,
            },
            focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
        })
        .expect("expected shortcut dispatch");

    assert_eq!(
        dispatch.outcome,
        StudioGuiDriverOutcome::IgnoredShortcut {
            shortcut: StudioGuiShortcut {
                modifiers: Vec::new(),
                key: crate::StudioGuiShortcutKey::Tab,
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
                key: crate::StudioGuiShortcutKey::Tab,
            },
            focus_context: StudioGuiFocusContext::TextInput,
        })
        .expect("expected shortcut dispatch");

    assert_eq!(
        dispatch.outcome,
        StudioGuiDriverOutcome::IgnoredShortcut {
            shortcut: StudioGuiShortcut {
                modifiers: Vec::new(),
                key: crate::StudioGuiShortcutKey::Tab,
            },
            reason: StudioGuiShortcutIgnoreReason::TextInputOwnsShortcut,
        }
    );
}

#[test]
fn gui_driver_reports_ignored_shortcut_when_command_palette_owns_function_key() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let _ = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::ShortcutPressed {
            shortcut: StudioGuiShortcut {
                modifiers: vec![crate::StudioGuiShortcutModifier::Shift],
                key: crate::StudioGuiShortcutKey::F6,
            },
            focus_context: StudioGuiFocusContext::CommandPalette,
        })
        .expect("expected shortcut dispatch");

    assert_ignored_shortcut(
        &dispatch,
        StudioGuiShortcut {
            modifiers: vec![crate::StudioGuiShortcutModifier::Shift],
            key: crate::StudioGuiShortcutKey::F6,
        },
        StudioGuiShortcutIgnoreReason::CommandPaletteOwnsShortcut,
    );
    assert_eq!(
        dispatch.snapshot.runtime.control_state.simulation_mode,
        rf_ui::SimulationMode::Hold
    );
}

#[test]
fn gui_driver_reports_ignored_shortcut_when_modal_dialog_owns_canvas_shortcut() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    driver.replace_canvas_suggestions(vec![sample_canvas_suggestion("sug-high", 0.95)]);

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::ShortcutPressed {
            shortcut: StudioGuiShortcut {
                modifiers: Vec::new(),
                key: crate::StudioGuiShortcutKey::Tab,
            },
            focus_context: StudioGuiFocusContext::ModalDialog,
        })
        .expect("expected shortcut dispatch");

    assert_ignored_shortcut(
        &dispatch,
        StudioGuiShortcut {
            modifiers: Vec::new(),
            key: crate::StudioGuiShortcutKey::Tab,
        },
        StudioGuiShortcutIgnoreReason::ModalDialogOwnsShortcut,
    );
    assert_eq!(
        dispatch
            .canvas
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str()),
        Some("sug-high")
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
                key: crate::StudioGuiShortcutKey::Tab,
            },
            focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
        })
        .expect("expected shortcut dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                command_id,
                result,
                ..
            },
        )) => {
            assert_eq!(command_id, "canvas.accept_focused");
            assert_eq!(result.action, StudioGuiCanvasInteractionAction::AcceptFocusedByTab);
            assert_eq!(
                result
                    .accepted
                    .as_ref()
                    .map(|suggestion| suggestion.id.as_str()),
                Some("sug-high")
            );
            assert_eq!(
                result.applied_target,
                Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new("flash-1")))
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
                result.focused.as_ref().map(|suggestion| suggestion.id.as_str()),
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
                key: crate::StudioGuiShortcutKey::Escape,
            },
            focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
        })
        .expect("expected reject dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                command_id,
                result,
                ..
            },
        )) => {
            assert_eq!(command_id, "canvas.reject_focused");
            assert_eq!(result.action, StudioGuiCanvasInteractionAction::RejectFocused);
            assert_eq!(
                result
                    .rejected
                    .as_ref()
                    .map(|suggestion| suggestion.id.as_str()),
                Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
            );
            assert_eq!(
                result.canvas.focused_suggestion_id.as_ref().map(|id| id.as_str()),
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
                modifiers: vec![crate::StudioGuiShortcutModifier::Ctrl],
                key: crate::StudioGuiShortcutKey::Tab,
            },
            focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
        })
        .expect("expected shortcut dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                command_id,
                result,
                ..
            },
        )) => {
            assert_eq!(command_id, "canvas.focus_next");
            assert_eq!(result.action, StudioGuiCanvasInteractionAction::FocusNext);
            assert_eq!(
                result.focused.as_ref().map(|suggestion| suggestion.id.as_str()),
                Some("local.flash_drum.create_outlet.flash-1.liquid")
            );
        }
        other => panic!("expected executed canvas ui command outcome, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}
