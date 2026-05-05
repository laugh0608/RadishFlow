use std::fs;

use super::test_support::{
    assert_ignored_shortcut, flash_drum_local_rules_config, layout_persistence_config,
    lease_expiring_config, sample_canvas_suggestion, synced_workspace_config,
    unbound_outlet_failure_synced_config,
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
fn gui_driver_reports_ignored_shortcut_when_text_input_owns_undo_redo() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");

    for (key, shortcut_name) in [
        (crate::StudioGuiShortcutKey::Z, "ctrl-z"),
        (crate::StudioGuiShortcutKey::Y, "ctrl-y"),
    ] {
        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: vec![crate::StudioGuiShortcutModifier::Ctrl],
                    key,
                },
                focus_context: StudioGuiFocusContext::TextInput,
            })
            .expect("expected shortcut dispatch");

        assert_ignored_shortcut(
            &dispatch,
            StudioGuiShortcut {
                modifiers: vec![crate::StudioGuiShortcutModifier::Ctrl],
                key,
            },
            StudioGuiShortcutIgnoreReason::TextInputOwnsShortcut,
        );
        assert_eq!(
            dispatch.snapshot.runtime.control_state.simulation_mode,
            rf_ui::SimulationMode::Hold,
            "{shortcut_name} should not leave the text input boundary"
        );
    }
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
fn gui_driver_commits_pending_canvas_edit_from_explicit_position_event() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    driver
        .begin_canvas_place_unit("Flash Drum")
        .expect("expected begin canvas place unit");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::CanvasPendingEditCommitRequested {
            position: rf_ui::CanvasPoint::new(144.0, 88.0),
        })
        .expect("expected pending edit commit dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::CanvasInteraction(result) => {
            assert_eq!(
                result.action,
                StudioGuiCanvasInteractionAction::CommitPendingEditAt {
                    position: rf_ui::CanvasPoint::new(144.0, 88.0),
                }
            );
            let committed = result.committed_edit.expect("expected committed edit");
            assert_eq!(committed.unit_id, rf_types::UnitId::new("flash-2"));
            assert_eq!(
                committed.command,
                rf_ui::DocumentCommand::CreateUnit {
                    unit_id: rf_types::UnitId::new("flash-2"),
                    kind: "flash_drum".to_string(),
                }
            );
            assert_eq!(
                result.applied_target,
                Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                    "flash-2"
                )))
            );
            assert_eq!(result.canvas.pending_edit, None);
            assert_eq!(dispatch.canvas.pending_edit, None);
        }
        other => panic!("expected canvas interaction outcome, got {other:?}"),
    }

    assert!(
        dispatch
            .snapshot
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
}

#[test]
fn gui_driver_moves_canvas_unit_layout_from_explicit_event_without_dirtying_document() {
    let (config, project_path, layout_path) = layout_persistence_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::CanvasUnitLayoutMoveRequested {
            unit_id: rf_types::UnitId::new("feed-1"),
            position: rf_ui::CanvasPoint::new(128.0, 96.0),
        })
        .expect("expected canvas layout move dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::CanvasUnitLayoutMoved(result),
        ) => {
            assert_eq!(result.unit_id, rf_types::UnitId::new("feed-1"));
            assert_eq!(result.previous_position, None);
            assert_eq!(result.position, rf_ui::CanvasPoint::new(128.0, 96.0));
            assert_eq!(
                result
                    .canvas
                    .units
                    .iter()
                    .find(|unit| unit.unit_id == rf_types::UnitId::new("feed-1"))
                    .and_then(|unit| unit.layout_position),
                Some(rf_ui::CanvasPoint::new(128.0, 96.0))
            );
        }
        other => panic!("expected canvas unit layout move outcome, got {other:?}"),
    }
    assert!(
        !dispatch
            .snapshot
            .runtime
            .workspace_document
            .has_unsaved_changes
    );

    let stored = rf_store::read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert!(stored.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "feed-1" && position.x == 128.0 && position.y == 96.0
    }));

    let reopened = StudioGuiDriver::new(&config).expect("expected reopened driver");
    assert_eq!(
        reopened
            .canvas_state()
            .units
            .iter()
            .find(|unit| unit.unit_id == rf_types::UnitId::new("feed-1"))
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(128.0, 96.0))
    );

    let _ = fs::remove_file(layout_path);
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
fn gui_driver_dispatches_result_and_step_diagnostic_actions_through_inspector_focus() {
    let mut driver = StudioGuiDriver::new(&synced_workspace_config()).expect("expected driver");
    driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let solved = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.run_manual".to_string(),
        })
        .expect("expected solve dispatch");
    let snapshot = solved
        .window
        .runtime
        .latest_solve_snapshot
        .clone()
        .expect("expected solved snapshot");

    let result_inspector =
        snapshot.result_inspector_with_unit(Some("stream-heated"), None, Some("heater-1"));
    let result_stream_command = result_inspector
        .diagnostic_actions
        .iter()
        .find(|action| {
            action.source_label == "Selected stream"
                && action.action.command_id == "inspector.focus_stream:stream-heated"
        })
        .map(|action| action.action.command_id.clone())
        .expect("expected selected stream diagnostic action");
    assert_inspector_focus_dispatch(
        &mut driver,
        &result_stream_command,
        ("Stream", "stream-heated"),
    );

    let unit_command = result_inspector
        .unit_diagnostic_actions
        .iter()
        .find(|action| {
            action.source_label == "Selected unit"
                && action.action.command_id == "inspector.focus_unit:heater-1"
        })
        .map(|action| action.action.command_id.clone())
        .expect("expected selected unit diagnostic action");
    assert_inspector_focus_dispatch(&mut driver, &unit_command, ("Unit", "heater-1"));

    let comparison = snapshot
        .result_inspector_with_comparison(Some("stream-feed"), Some("stream-heated"))
        .comparison
        .expect("expected comparison stream focus actions");
    assert_inspector_focus_dispatch(
        &mut driver,
        &comparison.base_stream_focus_action.command_id,
        ("Stream", "stream-feed"),
    );
    assert_inspector_focus_dispatch(
        &mut driver,
        &comparison.compared_stream_focus_action.command_id,
        ("Stream", "stream-heated"),
    );

    let step = snapshot
        .steps
        .iter()
        .find(|step| step.unit_id == "heater-1")
        .expect("expected heater step");
    for action in &step.diagnostic_actions {
        assert!(
            crate::inspector_target_from_command_id(&action.action.command_id).is_some(),
            "solve step diagnostic action should stay on inspector focus command surface: {}",
            action.action.command_id
        );
    }
    let step_stream_command = step
        .diagnostic_actions
        .iter()
        .find(|action| action.action.command_id == "inspector.focus_stream:stream-heated")
        .map(|action| action.action.command_id.clone())
        .expect("expected solve step produced stream action");
    assert_inspector_focus_dispatch(
        &mut driver,
        &step_stream_command,
        ("Stream", "stream-heated"),
    );

    let active_stream_detail = driver
        .window_model()
        .runtime
        .active_inspector_detail
        .expect("expected active stream inspector detail");
    let active_step_unit_command = active_stream_detail
        .diagnostic_actions
        .iter()
        .find(|action| {
            action.source_label == "Solve step"
                && action.action.command_id == "inspector.focus_unit:heater-1"
        })
        .map(|action| action.action.command_id.clone())
        .expect("expected active inspector solve step action");
    assert_inspector_focus_dispatch(&mut driver, &active_step_unit_command, ("Unit", "heater-1"));
}

#[test]
fn gui_driver_dispatches_failure_diagnostic_actions_through_recovery_or_inspector_focus() {
    let mut driver =
        StudioGuiDriver::new(&unbound_outlet_failure_synced_config()).expect("expected driver");
    driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let failed = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.run_manual".to_string(),
        })
        .expect("expected failed run dispatch");
    let failure = failed
        .window
        .runtime
        .latest_failure
        .as_ref()
        .expect("expected visible failure");

    for action in &failure.diagnostic_actions {
        let command_id = action.action.command_id.as_str();
        assert!(
            command_id == "run_panel.recover_failure"
                || crate::inspector_target_from_command_id(command_id).is_some(),
            "failure diagnostic action should stay on recovery or inspector command surface: {command_id}"
        );
    }

    let focus_command = failure
        .diagnostic_actions
        .iter()
        .find(|action| action.action.command_id == "inspector.focus_unit:feed-1")
        .map(|action| action.action.command_id.clone())
        .expect("expected failure focus action");
    assert_inspector_focus_dispatch(&mut driver, &focus_command, ("Unit", "feed-1"));
    assert_eq!(
        driver.window_model().runtime.control_state.run_status,
        rf_ui::RunStatus::Error,
        "focusing a failure target must not apply recovery"
    );

    let recovery_command = failure
        .diagnostic_actions
        .iter()
        .find(|action| action.action.command_id == "run_panel.recover_failure")
        .map(|action| action.action.command_id.clone())
        .expect("expected failure recovery action");
    let recovery = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: recovery_command,
        })
        .expect("expected recovery dispatch");
    match recovery.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
            crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
                assert_eq!(outcome.action.title, "Create outlet stream");
                assert_eq!(
                    outcome.applied_target,
                    Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                        "feed-1"
                    )))
                );
            }
            other => panic!("expected run panel recovery dispatch, got {other:?}"),
        },
        other => panic!("expected executed recovery command, got {other:?}"),
    }
    assert_eq!(
        recovery
            .window
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Unit", "feed-1"))
    );
    assert_eq!(
        recovery.window.runtime.control_state.run_status,
        rf_ui::RunStatus::Dirty
    );
}

fn assert_inspector_focus_dispatch(
    driver: &mut StudioGuiDriver,
    command_id: &str,
    expected_target: (&str, &str),
) {
    assert!(
        crate::inspector_target_from_command_id(command_id).is_some(),
        "expected inspector focus command id, got {command_id}"
    );
    let dispatch = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: command_id.to_string(),
        })
        .expect("expected inspector focus dispatch");
    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(_),
        )) => {}
        other => panic!("expected executed inspector focus command, got {other:?}"),
    }
    assert_eq!(
        dispatch
            .window
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(expected_target)
    );
}
