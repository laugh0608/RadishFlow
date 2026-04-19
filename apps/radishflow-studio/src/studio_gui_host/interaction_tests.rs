use std::fs;

use super::test_support::*;
use super::*;

#[test]
fn gui_host_surfaces_ui_commands_for_disabled_command_dispatch() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");

    let dispatch = gui_host
        .dispatch_ui_command("run_panel.run_manual")
        .expect("expected gui host ui command result");

    match dispatch {
        StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
            command_id,
            detail,
            target_window_id,
            ui_commands,
        } => {
            assert_eq!(command_id, "run_panel.run_manual");
            assert_eq!(target_window_id, None);
            assert_eq!(detail, "Open a studio window before running the workspace");
            assert!(
                !ui_commands
                    .command("run_panel.run_manual")
                    .expect("expected run command model")
                    .enabled
            );
        }
        other => panic!("expected disabled ui command result, got {other:?}"),
    }
}

#[test]
fn gui_host_dispatches_ui_command_and_refreshes_command_registry() {
    let (config, project_path) = solver_failure_config();
    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
    let opened = gui_host.open_window().expect("expected window open");

    let dispatch = gui_host
        .dispatch_ui_command("run_panel.run_manual")
        .expect("expected gui host ui command dispatch");

    match dispatch {
        StudioGuiHostUiCommandDispatchResult::Executed(dispatch) => {
            assert_eq!(dispatch.target_window_id, opened.registration.window_id);
            assert!(
                dispatch
                    .ui_commands
                    .command("run_panel.recover_failure")
                    .expect("expected recovery command model")
                    .enabled
            );
            assert_eq!(
                dispatch.native_timers,
                StudioGuiNativeTimerEffects::from_driver(
                    &dispatch.effects.native_timer_transitions,
                    &dispatch.effects.native_timer_acks,
                )
            );
            match &dispatch.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::WorkspaceRun(run) => {
                        assert!(matches!(
                            run.outcome,
                            crate::StudioWorkspaceRunOutcome::Failed(_)
                        ));
                    }
                    other => panic!("expected workspace run dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            }
        }
        other => panic!("expected executed ui command result, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_host_command_surface_ids_converge_into_equivalent_host_dispatch_paths() {
    let mut surface_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
    let opened = surface_host.open_window().expect("expected window open");
    let window = surface_host.window_model_for_window(Some(opened.registration.window_id));

    let palette_command_id = window
        .commands
        .palette_items("activate")
        .into_iter()
        .map(|item| item.command_id)
        .collect::<Vec<_>>();
    let toolbar_command_id = window
        .commands
        .toolbar_sections
        .iter()
        .flat_map(|section| section.items.iter())
        .find(|item| item.label == "Activate workspace")
        .map(|item| item.command_id.clone())
        .expect("expected activate toolbar command");
    let command_list_command_id = window
        .commands
        .command_list_sections
        .iter()
        .flat_map(|section| section.items.iter())
        .find(|item| item.label == "Activate workspace (Shift+F6)")
        .map(|item| item.command_id.clone())
        .expect("expected activate command list item");
    let menu_command_id =
        find_menu_command_by_label(&window.commands.menu_tree, "Activate workspace (Shift+F6)")
            .map(|item| item.command_id.clone())
            .expect("expected activate menu command");

    assert_eq!(palette_command_id, vec!["run_panel.set_active".to_string()]);
    assert_eq!(toolbar_command_id, "run_panel.set_active");
    assert_eq!(command_list_command_id, "run_panel.set_active");
    assert_eq!(menu_command_id, "run_panel.set_active");

    let dispatched = surface_host
        .dispatch_ui_command(&palette_command_id[0])
        .expect("expected dispatch ui command");
    match &dispatched {
        StudioGuiHostUiCommandDispatchResult::Executed(dispatch) => {
            assert_eq!(dispatch.target_window_id, opened.registration.window_id);
            match &dispatch.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::WorkspaceMode(mode) => {
                        assert_eq!(mode.simulation_mode, rf_ui::SimulationMode::Active);
                    }
                    other => panic!("expected workspace mode dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            }
        }
        other => panic!("expected executed ui command dispatch, got {other:?}"),
    }

    let mut command_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
    let opened = match command_host
        .execute_command(StudioGuiHostCommand::OpenWindow)
        .expect("expected open command")
    {
        StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    let executed = command_host
        .execute_command(StudioGuiHostCommand::DispatchUiCommand {
            command_id: menu_command_id,
        })
        .expect("expected command dispatch");
    match &executed {
        StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        ) => {
            assert_eq!(dispatch.target_window_id, opened);
            match &dispatch.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::WorkspaceMode(mode) => {
                        assert_eq!(mode.simulation_mode, rf_ui::SimulationMode::Active);
                    }
                    other => panic!("expected workspace mode dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            }
        }
        other => panic!("expected executed command outcome, got {other:?}"),
    }

    let dispatched_window = surface_host.window_model_for_window(Some(1));
    let executed_window = command_host.window_model_for_window(Some(1));
    assert_eq!(dispatched_window.commands, executed_window.commands);
    assert_eq!(
        dispatched_window.runtime.control_state,
        executed_window.runtime.control_state
    );
    assert_eq!(
        dispatched_window.runtime.run_panel.view(),
        executed_window.runtime.run_panel.view()
    );
}

#[test]
fn gui_host_executes_canvas_interaction_through_command_surface() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
    let opened = gui_host.open_window().expect("expected window open");

    let _ = gui_host
        .dispatch_ui_command("run_panel.set_active")
        .expect("expected activate dispatch");

    let dispatch = gui_host
        .execute_command(StudioGuiHostCommand::DispatchCanvasInteraction {
            action: StudioGuiCanvasInteractionAction::AcceptFocusedByTab,
        })
        .expect("expected canvas interaction command");
    match dispatch {
        StudioGuiHostCommandOutcome::CanvasInteracted(result) => {
            assert_eq!(
                result.action,
                StudioGuiCanvasInteractionAction::AcceptFocusedByTab
            );
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
            assert!(
                !result
                    .ui_commands
                    .command("run_panel.resume_workspace")
                    .expect("expected resume command")
                    .enabled
            );
        }
        other => panic!("expected canvas interaction outcome, got {other:?}"),
    }
    assert_eq!(
        gui_host.state().foreground_window_id,
        Some(opened.registration.window_id)
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_host_dispatches_canvas_ui_command_by_command_id() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
    let opened = gui_host.open_window().expect("expected window open");

    let _ = gui_host
        .dispatch_ui_command("run_panel.set_active")
        .expect("expected activate dispatch");

    let dispatch = gui_host
        .dispatch_ui_command("canvas.accept_focused")
        .expect("expected canvas ui command");
    match dispatch {
        StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
            command_id,
            target_window_id,
            result,
        } => {
            assert_eq!(command_id, "canvas.accept_focused");
            assert_eq!(target_window_id, Some(opened.registration.window_id));
            assert_eq!(
                result
                    .accepted
                    .as_ref()
                    .map(|suggestion| suggestion.id.as_str()),
                Some("local.flash_drum.create_outlet.flash-1.vapor")
            );
        }
        other => panic!("expected canvas ui command outcome, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_host_canvas_ui_command_focus_persists_for_followup_reject() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");

    let focus = gui_host
        .dispatch_ui_command("canvas.focus_next")
        .expect("expected focus-next dispatch");
    match focus {
        StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction { result, .. } => {
            assert_eq!(
                result
                    .focused
                    .as_ref()
                    .map(|suggestion| suggestion.id.as_str()),
                Some("local.flash_drum.create_outlet.flash-1.liquid")
            );
        }
        other => panic!("expected focus-next canvas ui command outcome, got {other:?}"),
    }

    let reject = gui_host
        .dispatch_ui_command("canvas.reject_focused")
        .expect("expected reject dispatch");
    match reject {
        StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction { result, .. } => {
            assert_eq!(
                result
                    .rejected
                    .as_ref()
                    .map(|suggestion| suggestion.id.as_str()),
                Some("local.flash_drum.create_outlet.flash-1.liquid")
            );
        }
        other => panic!("expected reject canvas ui command outcome, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_host_dispatches_foreground_entitlement_primary_action() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
    let opened = gui_host.open_window().expect("expected window open");

    let dispatch = gui_host
        .execute_command(StudioGuiHostCommand::DispatchUiCommand {
            command_id: "entitlement.refresh_offline_lease".to_string(),
        })
        .expect("expected entitlement primary action dispatch");

    match dispatch {
        StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        ) => {
            assert_eq!(dispatch.target_window_id, opened.registration.window_id);
            match &dispatch.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::Entitlement(entitlement) => {
                        assert_eq!(
                            entitlement.action,
                            crate::StudioEntitlementAction::RefreshOfflineLease
                        );
                    }
                    other => panic!("expected entitlement dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            }
        }
        other => panic!("expected executed entitlement primary action outcome, got {other:?}"),
    }
}

#[test]
fn gui_host_dispatches_foreground_entitlement_secondary_action() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
    let opened = gui_host.open_window().expect("expected window open");

    let dispatch = gui_host
        .execute_command(StudioGuiHostCommand::DispatchUiCommand {
            command_id: "entitlement.sync".to_string(),
        })
        .expect("expected entitlement action dispatch");

    match dispatch {
        StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        ) => {
            assert_eq!(dispatch.target_window_id, opened.registration.window_id);
            match &dispatch.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::Entitlement(entitlement) => {
                        assert_eq!(
                            entitlement.action,
                            crate::StudioEntitlementAction::SyncEntitlement
                        );
                    }
                    other => panic!("expected entitlement dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            }
        }
        other => panic!("expected executed entitlement action outcome, got {other:?}"),
    }
}

#[test]
fn gui_host_stably_ignores_entitlement_action_without_registered_window() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");

    let dispatch = gui_host
        .execute_command(StudioGuiHostCommand::DispatchUiCommand {
            command_id: "entitlement.sync".to_string(),
        })
        .expect("expected entitlement action result");

    match dispatch {
        StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                command_id,
                detail,
                target_window_id,
                ..
            },
        ) => {
            assert_eq!(command_id, "entitlement.sync");
            assert_eq!(detail, "Open a studio window before syncing entitlement");
            assert_eq!(target_window_id, None);
        }
        other => panic!("expected ignored entitlement action outcome, got {other:?}"),
    }
}

#[test]
fn gui_host_preserves_timer_retirement_summary_on_close() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
    let opened = gui_host.open_window().expect("expected window open");
    let _ = gui_host
        .dispatch_window_trigger(
            opened.registration.window_id,
            StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        )
        .expect("expected timer trigger");

    let closed = gui_host
        .close_window(opened.registration.window_id)
        .expect("expected close result");
    let close = closed.close.expect("expected close summary");

    assert!(matches!(
        close.retirement,
        StudioWindowHostRetirement::Parked {
            parked_entitlement_timer: Some(_)
        }
    ));
    assert!(matches!(
        closed.native_timers.operations.as_slice(),
        [crate::StudioGuiNativeTimerOperation::Park { from_window_id, .. }]
        if *from_window_id == opened.registration.window_id
    ));
    assert_eq!(closed.projection.state.windows.len(), 0);
    assert!(
        !closed
            .ui_commands
            .command("run_panel.run_manual")
            .expect("expected run command")
            .enabled
    );
}

#[test]
fn gui_host_routes_window_foregrounded_lifecycle_event_to_target_window() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
    let first = gui_host.open_window().expect("expected first window");
    let second = gui_host.open_window().expect("expected second window");

    let dispatch = gui_host
        .dispatch_lifecycle_event(StudioGuiHostLifecycleEvent::WindowForegrounded {
            window_id: second.registration.window_id,
        })
        .expect("expected lifecycle dispatch");

    let routed = dispatch.dispatch.expect("expected foreground dispatch");
    assert_eq!(routed.target_window_id, second.registration.window_id);
    assert_ne!(routed.target_window_id, first.registration.window_id);
    assert_eq!(
        dispatch.projection.state.foreground_window_id,
        Some(second.registration.window_id)
    );
    assert_eq!(
        dispatch
            .ui_commands
            .command("run_panel.run_manual")
            .and_then(|command| command.target_window_id),
        Some(second.registration.window_id)
    );
    match &routed.effects.runtime_report.dispatch {
        crate::StudioRuntimeDispatch::EntitlementSessionEvent(outcome) => {
            assert_eq!(outcome.event, crate::EntitlementSessionEvent::TimerElapsed);
        }
        other => panic!("expected entitlement session event dispatch, got {other:?}"),
    }
}

#[test]
fn gui_host_routes_timer_elapsed_lifecycle_event_to_timer_owner_window() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
    let first = gui_host.open_window().expect("expected first window");
    let second = gui_host.open_window().expect("expected second window");
    let _ = gui_host
        .dispatch_window_trigger(
            first.registration.window_id,
            StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        )
        .expect("expected first timer dispatch");
    let _ = gui_host
        .dispatch_lifecycle_event(StudioGuiHostLifecycleEvent::WindowForegrounded {
            window_id: second.registration.window_id,
        })
        .expect("expected foreground update");

    let dispatch = gui_host
        .dispatch_lifecycle_event(StudioGuiHostLifecycleEvent::TimerElapsed)
        .expect("expected timer elapsed lifecycle dispatch");

    let routed = dispatch.dispatch.expect("expected routed timer dispatch");
    assert_eq!(routed.target_window_id, first.registration.window_id);
    assert_ne!(routed.target_window_id, second.registration.window_id);
    assert!(matches!(
        routed.native_timers.operations.as_slice(),
        [crate::StudioGuiNativeTimerOperation::Keep { window_id, .. }]
        if *window_id == first.registration.window_id
    ));
    assert_eq!(
        dispatch.projection.state.foreground_window_id,
        Some(second.registration.window_id)
    );
}

#[test]
fn gui_host_execute_command_routes_open_ui_command_and_close() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");

    let opened = gui_host
        .execute_command(StudioGuiHostCommand::OpenWindow)
        .expect("expected open command");
    let window_id = match opened {
        StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
        other => panic!("expected window opened outcome, got {other:?}"),
    };

    let command_dispatch = gui_host
        .execute_command(StudioGuiHostCommand::DispatchUiCommand {
            command_id: "run_panel.set_active".to_string(),
        })
        .expect("expected ui command dispatch");
    match command_dispatch {
        StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        ) => {
            assert_eq!(dispatch.target_window_id, window_id);
        }
        other => panic!("expected executed ui command outcome, got {other:?}"),
    }

    let closed = gui_host
        .execute_command(StudioGuiHostCommand::CloseWindow { window_id })
        .expect("expected close command");
    match closed {
        StudioGuiHostCommandOutcome::WindowClosed(closed) => {
            assert!(closed.close.is_some());
            assert!(closed.projection.state.windows.is_empty());
        }
        other => panic!("expected window closed outcome, got {other:?}"),
    }
}
