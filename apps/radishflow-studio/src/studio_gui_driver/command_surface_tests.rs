use std::fs;

use rf_store::read_project_file;

use super::test_support::{
    find_menu_command, find_menu_command_by_label, flash_drum_local_rules_config,
    flash_drum_local_rules_synced_config, lease_expiring_config, synced_workspace_config,
    unbound_outlet_failure_synced_config,
};
use super::*;

#[test]
fn gui_driver_opens_window_and_refreshes_command_state() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
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
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "run_panel.set_active".to_string(),
        })
        .expect("expected ui command dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => {
            assert_eq!(executed.target_window_id, window_id);
        }
        other => panic!("expected executed ui command outcome, got {other:?}"),
    }
}

#[test]
fn gui_driver_saves_current_project_through_command_surface() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let focus = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_stream:stream-feed".to_string(),
        })
        .expect("expected stream focus dispatch");
    let field = focus
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .cloned()
        .expect("expected temperature field");
    let update = driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftUpdateRequested {
            command_id: field.draft_update_command_id,
            raw_value: "333.5".to_string(),
        })
        .expect("expected draft update");
    let commit_command_id = update
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .and_then(|field| field.commit_command_id.clone())
        .expect("expected commit command");
    let dirty = driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftCommitRequested {
            command_id: commit_command_id,
        })
        .expect("expected draft commit");
    assert!(dirty.window.runtime.workspace_document.has_unsaved_changes);

    let save = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: crate::FILE_SAVE_COMMAND_ID.to_string(),
        })
        .expect("expected save dispatch");

    match &save.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
            crate::StudioRuntimeDispatch::DocumentLifecycle(outcome) => {
                assert_eq!(outcome.action, crate::StudioDocumentLifecycleAction::Save);
                assert_eq!(outcome.path, project_path);
                assert!(!outcome.has_unsaved_changes);
            }
            other => panic!("expected document lifecycle dispatch, got {other:?}"),
        },
        other => panic!("expected save command dispatch, got {other:?}"),
    }
    assert!(!save.window.runtime.workspace_document.has_unsaved_changes);
    let saved = read_project_file(&project_path).expect("expected saved project");
    assert_eq!(
        saved.document.revision,
        save.window.runtime.workspace_document.revision
    );
}

#[test]
fn gui_driver_routes_inspector_draft_update_through_driver_boundary() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let focus = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_stream:stream-feed".to_string(),
        })
        .expect("expected inspector focus dispatch");
    let field = focus
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .cloned()
        .expect("expected stream temperature field");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftUpdateRequested {
            command_id: field.draft_update_command_id,
            raw_value: "333.5".to_string(),
        })
        .expect("expected draft update dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftUpdated(updated),
        ) => {
            assert_eq!(updated.target_window_id, 1);
            match &updated.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::InspectorDraftUpdate(outcome) => {
                    assert!(outcome.applied);
                    assert_eq!(outcome.document_revision, 0);
                    assert_eq!(outcome.command_history_len, 0);
                }
                other => panic!("expected inspector draft update dispatch, got {other:?}"),
            }
        }
        other => panic!("expected inspector draft update outcome, got {other:?}"),
    }

    let updated_field = dispatch
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .expect("expected updated temperature field");
    assert_eq!(updated_field.current_value, "333.5");
    assert_eq!(updated_field.status_label, "Draft");
    assert!(updated_field.is_dirty);
    assert_eq!(
        updated_field.commit_command_id.as_deref(),
        Some("inspector.commit_stream_draft:stream:stream-feed:temperature_k")
    );
}

#[test]
fn gui_driver_routes_inspector_draft_commit_through_driver_boundary() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let focus = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_stream:stream-feed".to_string(),
        })
        .expect("expected inspector focus dispatch");
    let field = focus
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .cloned()
        .expect("expected stream temperature field");
    let update = driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftUpdateRequested {
            command_id: field.draft_update_command_id,
            raw_value: "333.5".to_string(),
        })
        .expect("expected draft update dispatch");
    let commit_command_id = update
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .and_then(|field| field.commit_command_id.clone())
        .expect("expected commit command id");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftCommitRequested {
            command_id: commit_command_id,
        })
        .expect("expected draft commit dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftCommitted(committed),
        ) => {
            assert_eq!(committed.target_window_id, 1);
            match &committed.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::InspectorDraftCommit(outcome) => {
                    assert!(outcome.applied);
                    assert_eq!(outcome.document_revision, 1);
                    assert_eq!(outcome.command_history_len, 1);
                }
                other => panic!("expected inspector draft commit dispatch, got {other:?}"),
            }
        }
        other => panic!("expected inspector draft commit outcome, got {other:?}"),
    }

    assert_eq!(dispatch.window.runtime.workspace_document.revision, 1);
    assert_eq!(
        dispatch
            .window
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Stream", "stream-feed"))
    );
    let committed_field = dispatch
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .expect("expected committed temperature field");
    assert_eq!(committed_field.current_value, "333.5");
    assert_eq!(committed_field.status_label, "Synced");
    assert!(!committed_field.is_dirty);
    assert!(committed_field.commit_command_id.is_none());
    assert_eq!(
        dispatch.window.runtime.control_state.pending_reason,
        Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn gui_driver_routes_inspector_draft_batch_commit_through_driver_boundary() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_stream:stream-feed".to_string(),
        })
        .expect("expected inspector focus dispatch");
    let update_temperature = driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftUpdateRequested {
            command_id: "inspector.update_stream_draft:stream:stream-feed:temperature_k"
                .to_string(),
            raw_value: "333.5".to_string(),
        })
        .expect("expected temperature draft update");
    assert!(
        update_temperature
            .window
            .runtime
            .active_inspector_detail
            .as_ref()
            .and_then(|detail| detail.property_batch_commit_command_id.as_ref())
            .is_none()
    );
    let update_pressure = driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftUpdateRequested {
            command_id: "inspector.update_stream_draft:stream:stream-feed:pressure_pa".to_string(),
            raw_value: "202650".to_string(),
        })
        .expect("expected pressure draft update");
    let batch_command_id = update_pressure
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| detail.property_batch_commit_command_id.clone())
        .expect("expected batch commit command id");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftBatchCommitRequested {
            command_id: batch_command_id,
        })
        .expect("expected batch draft commit dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::InspectorDraftBatchCommitted(committed),
        ) => {
            assert_eq!(committed.target_window_id, 1);
            match &committed.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::InspectorDraftBatchCommit(outcome) => {
                    assert!(outcome.applied);
                    assert_eq!(outcome.document_revision, 1);
                    assert_eq!(outcome.command_history_len, 1);
                    assert_eq!(
                        outcome.committed_keys,
                        vec![
                            "stream:stream-feed:temperature_k".to_string(),
                            "stream:stream-feed:pressure_pa".to_string()
                        ]
                    );
                }
                other => panic!("expected inspector draft batch commit dispatch, got {other:?}"),
            }
        }
        other => panic!("expected inspector draft batch commit outcome, got {other:?}"),
    }

    let detail = dispatch
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .expect("expected inspector detail");
    assert!(detail.property_batch_commit_command_id.is_none());
    let temperature = detail
        .property_fields
        .iter()
        .find(|field| field.key == "stream:stream-feed:temperature_k")
        .expect("expected temperature field");
    let pressure = detail
        .property_fields
        .iter()
        .find(|field| field.key == "stream:stream-feed:pressure_pa")
        .expect("expected pressure field");
    assert_eq!(temperature.current_value, "333.5");
    assert_eq!(pressure.current_value, "202650");
    assert_eq!(temperature.status_label, "Synced");
    assert_eq!(pressure.status_label, "Synced");
    assert_eq!(
        dispatch.window.runtime.control_state.pending_reason,
        Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn gui_driver_routes_document_history_commands_through_command_surface() {
    let mut driver = StudioGuiDriver::new(&synced_workspace_config()).expect("expected driver");
    driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let focus = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_stream:stream-feed".to_string(),
        })
        .expect("expected inspector focus dispatch");
    let field = focus
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .cloned()
        .expect("expected stream temperature field");
    let update = driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftUpdateRequested {
            command_id: field.draft_update_command_id,
            raw_value: "333.5".to_string(),
        })
        .expect("expected draft update dispatch");
    let commit_command_id = update
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .and_then(|field| field.commit_command_id.clone())
        .expect("expected commit command id");
    driver
        .dispatch_event(StudioGuiEvent::InspectorFieldDraftCommitRequested {
            command_id: commit_command_id,
        })
        .expect("expected draft commit dispatch");

    let undo = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "edit.undo".to_string(),
        })
        .expect("expected undo dispatch");

    match &undo.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
            crate::StudioRuntimeDispatch::DocumentHistory(outcome) => {
                assert_eq!(outcome.command, crate::StudioDocumentHistoryCommand::Undo);
                assert!(outcome.applied);
                assert_eq!(outcome.document_revision, 2);
                assert_eq!(outcome.command_history_cursor, 0);
            }
            other => panic!("expected document history dispatch, got {other:?}"),
        },
        other => panic!("expected executed undo outcome, got {other:?}"),
    }
    assert_eq!(undo.window.runtime.workspace_document.revision, 2);
    let undone_field = undo
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .expect("expected undone temperature field");
    assert_eq!(undone_field.current_value, "300");
    assert!(!undone_field.is_dirty);

    let redo = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "edit.redo".to_string(),
        })
        .expect("expected redo dispatch");

    match &redo.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
            crate::StudioRuntimeDispatch::DocumentHistory(outcome) => {
                assert_eq!(outcome.command, crate::StudioDocumentHistoryCommand::Redo);
                assert!(outcome.applied);
                assert_eq!(outcome.document_revision, 3);
                assert_eq!(outcome.command_history_cursor, 1);
            }
            other => panic!("expected document history dispatch, got {other:?}"),
        },
        other => panic!("expected executed redo outcome, got {other:?}"),
    }
    let redone_field = redo
        .window
        .runtime
        .active_inspector_detail
        .as_ref()
        .and_then(|detail| {
            detail
                .property_fields
                .iter()
                .find(|field| field.key == "stream:stream-feed:temperature_k")
        })
        .expect("expected redone temperature field");
    assert_eq!(redone_field.current_value, "333.5");
    assert!(!redone_field.is_dirty);
}

#[test]
fn gui_driver_routes_entitlement_primary_action_through_single_event_entry() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let open = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match open.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "entitlement.refresh_offline_lease".to_string(),
        })
        .expect("expected entitlement primary action dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        )) => {
            assert_eq!(dispatch.target_window_id, window_id);
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
fn gui_driver_routes_entitlement_action_through_single_event_entry() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let open = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match open.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "entitlement.sync".to_string(),
        })
        .expect("expected entitlement action dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
        )) => {
            assert_eq!(dispatch.target_window_id, window_id);
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
fn gui_driver_stably_ignores_entitlement_action_without_registered_window() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "entitlement.sync".to_string(),
        })
        .expect("expected entitlement action result");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                command_id,
                detail,
                target_window_id,
                ..
            },
        )) => {
            assert_eq!(command_id, "entitlement.sync");
            assert_eq!(detail, "Open a studio window before syncing entitlement");
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
        !dispatch.snapshot.command_registry.sections.is_empty(),
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
        Some("File")
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
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
            lifecycle,
        )) => {
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
                modifiers: vec![crate::StudioGuiShortcutModifier::Shift],
                key: crate::StudioGuiShortcutKey::F6,
            },
            focus_context: StudioGuiFocusContext::Global,
        })
        .expect("expected shortcut dispatch");

    match dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
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
                modifiers: vec![crate::StudioGuiShortcutModifier::Shift],
                key: crate::StudioGuiShortcutKey::F6,
            },
            focus_context: StudioGuiFocusContext::Global,
        })
        .expect("expected activate shortcut dispatch");

    match &activate.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
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
                key: crate::StudioGuiShortcutKey::Tab,
            },
            focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
        })
        .expect("expected canvas acceptance dispatch");

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
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
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
                key: crate::StudioGuiShortcutKey::F8,
            },
            focus_context: StudioGuiFocusContext::Global,
        })
        .expect("expected recovery shortcut dispatch");
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
                modifiers: vec![crate::StudioGuiShortcutModifier::Shift],
                key: crate::StudioGuiShortcutKey::F5,
            },
            focus_context: StudioGuiFocusContext::Global,
        })
        .expect("expected resume shortcut dispatch");
    match resumed.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
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
fn gui_driver_keeps_recovery_command_presentation_aligned_across_surfaces_after_failure() {
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

    let recovery_palette_item = failed
        .window
        .commands
        .palette_items("diagnostic")
        .into_iter()
        .find(|item| item.command_id == "run_panel.recover_failure")
        .expect("expected recovery palette item");
    let recovery_toolbar_item = failed
        .window
        .commands
        .toolbar_sections
        .iter()
        .find(|section| section.title == "Recovery")
        .and_then(|section| {
            section
                .items
                .iter()
                .find(|item| item.command_id == "run_panel.recover_failure")
        })
        .expect("expected recovery toolbar item");
    let recovery_list_item = failed
        .window
        .commands
        .command_list_sections
        .iter()
        .find(|section| section.title == "Recovery")
        .and_then(|section| {
            section
                .items
                .iter()
                .find(|item| item.command_id == "run_panel.recover_failure")
        })
        .expect("expected recovery command list item");
    let recovery_menu_item = find_menu_command(
        &failed.window.commands.menu_tree,
        "run_panel.recover_failure",
    )
    .expect("expected recovery menu item");

    assert!(recovery_palette_item.enabled);
    assert!(recovery_toolbar_item.enabled);
    assert!(recovery_list_item.enabled);
    assert!(recovery_menu_item.enabled);
    assert_eq!(
        recovery_palette_item.label,
        "Recover run panel failure (F8)"
    );
    assert_eq!(recovery_list_item.label, "Recover run panel failure (F8)");
    assert_eq!(recovery_toolbar_item.label, "Recover run panel failure");
    assert_eq!(recovery_menu_item.label, "Recover run panel failure (F8)");
    assert_eq!(
        recovery_palette_item.menu_path_text,
        "Run > Recovery > Recover Run Panel Failure"
    );
    assert_eq!(
        recovery_list_item.menu_path_text,
        recovery_palette_item.menu_path_text
    );
    assert_eq!(
        recovery_palette_item.hover_text,
        "Apply the current run panel recovery action in the target window\nMenu: Run > Recovery > Recover Run Panel Failure"
    );
    assert_eq!(
        recovery_toolbar_item.hover_text,
        recovery_palette_item.hover_text
    );
    assert_eq!(
        recovery_menu_item.hover_text,
        recovery_palette_item.hover_text
    );
}

#[test]
fn gui_driver_command_surface_dispatch_matches_shortcut_route_for_activate_workspace() {
    let mut surface_driver =
        StudioGuiDriver::new(&lease_expiring_config()).expect("expected surface driver");
    let opened = surface_driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");

    let palette_command_id = opened
        .window
        .commands
        .palette_items("activate")
        .into_iter()
        .map(|item| item.command_id)
        .collect::<Vec<_>>();
    let toolbar_command_id = opened
        .window
        .commands
        .toolbar_sections
        .iter()
        .flat_map(|section| section.items.iter())
        .find(|item| item.label == "Activate workspace")
        .map(|item| item.command_id.clone())
        .expect("expected activate toolbar command");
    let command_list_command_id = opened
        .window
        .commands
        .command_list_sections
        .iter()
        .flat_map(|section| section.items.iter())
        .find(|item| item.label == "Activate workspace (Shift+F6)")
        .map(|item| item.command_id.clone())
        .expect("expected activate command list item");
    let menu_command_id = find_menu_command_by_label(
        &opened.window.commands.menu_tree,
        "Activate workspace (Shift+F6)",
    )
    .map(|item| item.command_id.clone())
    .expect("expected activate menu item");

    assert_eq!(palette_command_id, vec!["run_panel.set_active".to_string()]);
    assert_eq!(toolbar_command_id, "run_panel.set_active");
    assert_eq!(command_list_command_id, "run_panel.set_active");
    assert_eq!(menu_command_id, "run_panel.set_active");

    let surface_dispatch = surface_driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: palette_command_id[0].clone(),
        })
        .expect("expected surface ui command dispatch");

    let mut shortcut_driver =
        StudioGuiDriver::new(&lease_expiring_config()).expect("expected shortcut driver");
    let _ = shortcut_driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let shortcut_dispatch = shortcut_driver
        .dispatch_event(StudioGuiEvent::ShortcutPressed {
            shortcut: StudioGuiShortcut {
                modifiers: vec![crate::StudioGuiShortcutModifier::Shift],
                key: crate::StudioGuiShortcutKey::F6,
            },
            focus_context: StudioGuiFocusContext::Global,
        })
        .expect("expected shortcut dispatch");

    match &surface_dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
            crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                crate::StudioAppResultDispatch::WorkspaceMode(mode) => {
                    assert_eq!(mode.simulation_mode, rf_ui::SimulationMode::Active);
                }
                other => panic!("expected workspace mode dispatch, got {other:?}"),
            },
            other => panic!("expected app command dispatch, got {other:?}"),
        },
        other => panic!("expected executed surface command outcome, got {other:?}"),
    }
    match &shortcut_dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
            StudioGuiHostUiCommandDispatchResult::Executed(executed),
        )) => match &executed.effects.runtime_report.dispatch {
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

    assert_eq!(
        surface_dispatch.window.commands,
        shortcut_dispatch.window.commands
    );
    assert_eq!(
        surface_dispatch.window.runtime.control_state,
        shortcut_dispatch.window.runtime.control_state
    );
    assert_eq!(
        surface_dispatch.window.runtime.run_panel.view(),
        shortcut_dispatch.window.runtime.run_panel.view()
    );
}
