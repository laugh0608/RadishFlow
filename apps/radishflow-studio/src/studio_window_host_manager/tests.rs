use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    StudioAppWindowHostCanvasInteractionResult, StudioAppWindowHostCommand,
    StudioAppWindowHostCommandOutcome, StudioAppWindowHostGlobalEvent, StudioAppWindowHostManager,
    StudioAppWindowHostUiAction, StudioAppWindowHostUiActionAvailability,
    StudioAppWindowHostUiActionDisabledReason, StudioAppWindowHostUiActionState,
    StudioCanvasInteractionAction, StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
    StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger, StudioWindowHostRole,
    StudioWindowTimerDriverTransition,
};
use rf_ui::{EntitlementActionId, RunPanelActionId};

fn lease_expiring_config() -> crate::StudioRuntimeConfig {
    crate::StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
        ..crate::StudioRuntimeConfig::default()
    }
}

fn solver_failure_config() -> (crate::StudioRuntimeConfig, PathBuf) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected time after epoch")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-window-host-recovery-{unique}.rfproj.json"
    ));
    let project_json = include_str!("../../../../examples/flowsheets/feed-valve-flash.rfproj.json")
        .replacen(
            "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 90000.0,",
            "\"name\": \"Valve Outlet\",\n          \"temperature_k\": 300.0,\n          \"pressure_pa\": 130000.0,",
            1,
        );
    fs::write(&project_path, project_json).expect("expected temporary failure project");

    (
        crate::StudioRuntimeConfig {
            project_path: project_path.clone(),
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            trigger: crate::StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        },
        project_path,
    )
}

fn synced_workspace_config() -> crate::StudioRuntimeConfig {
    crate::StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
        ..crate::StudioRuntimeConfig::default()
    }
}

fn flash_drum_local_rules_synced_config() -> (crate::StudioRuntimeConfig, PathBuf) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected time after epoch")
        .as_nanos();
    let project_path = std::env::temp_dir().join(format!(
        "radishflow-window-host-local-rules-{unique}.rfproj.json"
    ));
    let project_json = include_str!("../../../../examples/flowsheets/feed-heater-flash.rfproj.json")
        .replacen(
            ",\n        \"stream-vapor\": {\n          \"id\": \"stream-vapor\",\n          \"name\": \"Vapor Outlet\",\n          \"temperature_k\": 345.0,\n          \"pressure_pa\": 95000.0,\n          \"total_molar_flow_mol_s\": 0.0,\n          \"overall_mole_fractions\": {\n            \"component-a\": 0.5,\n            \"component-b\": 0.5\n          },\n          \"phases\": []\n        }",
            "",
            1,
        )
        .replacen(
            "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
            "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
            1,
        );
    fs::write(&project_path, project_json).expect("expected local rules project");

    (
        crate::StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..synced_workspace_config()
        },
        project_path,
    )
}

#[test]
fn app_window_host_manager_tracks_foreground_window_across_open_and_close() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();

    assert_eq!(first.role, StudioWindowHostRole::EntitlementTimerOwner);
    assert_eq!(manager.foreground_window_id(), Some(first.window_id));
    assert_eq!(
        manager.registered_windows(),
        vec![first.window_id, second.window_id]
    );

    let close = manager
        .close_window(first.window_id)
        .expect("expected first window close");

    assert_eq!(close.next_foreground_window_id, Some(second.window_id));
    assert_eq!(manager.foreground_window_id(), Some(second.window_id));
}

#[test]
fn app_window_host_manager_focuses_window_through_single_entry() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();

    let dispatch = manager
        .focus_window(second.window_id)
        .expect("expected focus dispatch");

    assert_eq!(dispatch.target_window_id, second.window_id);
    assert_eq!(manager.foreground_window_id(), Some(second.window_id));
    assert_eq!(
        dispatch.dispatch.host_output.runtime_output.trigger,
        StudioRuntimeTrigger::EntitlementSessionEvent(
            StudioRuntimeEntitlementSessionEvent::WindowForegrounded
        )
    );
    assert_eq!(
        manager.session().host_port().entitlement_timer_owner(),
        Some(first.window_id)
    );
}

#[test]
fn app_window_host_manager_routes_global_timer_elapsed_to_current_owner() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();
    let _ = manager
        .dispatch_trigger(
            first.window_id,
            &StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        )
        .expect("expected first timer dispatch");
    let _ = manager
        .focus_window(second.window_id)
        .expect("expected second window focus");

    let dispatch = manager
        .dispatch_global_event(StudioAppWindowHostGlobalEvent::TimerElapsed)
        .expect("expected global timer dispatch")
        .expect("expected routed timer dispatch");

    assert_eq!(dispatch.target_window_id, first.window_id);
    assert!(matches!(
        dispatch.dispatch.timer_driver_transitions.as_slice(),
        [StudioWindowTimerDriverTransition::KeepNativeTimer { window_id, .. }]
        if *window_id == first.window_id
    ));
}

#[test]
fn app_window_host_manager_routes_global_network_restored_to_foreground_window() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();
    let _ = manager
        .focus_window(second.window_id)
        .expect("expected second window focus");

    let dispatch = manager
        .dispatch_global_event(StudioAppWindowHostGlobalEvent::NetworkRestored)
        .expect("expected global network dispatch")
        .expect("expected routed network dispatch");

    assert_eq!(dispatch.target_window_id, second.window_id);
    assert_eq!(
        dispatch.dispatch.host_output.runtime_output.trigger,
        StudioRuntimeTrigger::EntitlementSessionEvent(
            StudioRuntimeEntitlementSessionEvent::NetworkRestored
        )
    );
    assert_eq!(manager.foreground_window_id(), Some(second.window_id));
    assert_eq!(
        manager.registered_windows(),
        vec![first.window_id, second.window_id]
    );
}

#[test]
fn app_window_host_manager_ignores_global_events_without_windows() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");

    let dispatch = manager
        .dispatch_global_event(StudioAppWindowHostGlobalEvent::NetworkRestored)
        .expect("expected global network dispatch");

    assert!(dispatch.is_none());
}

#[test]
fn app_window_host_manager_executes_commands_through_single_entry() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
    let first = match manager
        .execute_command(StudioAppWindowHostCommand::OpenWindow)
        .expect("expected first window open")
    {
        StudioAppWindowHostCommandOutcome::WindowOpened(registration) => registration,
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    let second = match manager
        .execute_command(StudioAppWindowHostCommand::OpenWindow)
        .expect("expected second window open")
    {
        StudioAppWindowHostCommandOutcome::WindowOpened(registration) => registration,
        other => panic!("expected window opened outcome, got {other:?}"),
    };

    assert_eq!(first.role, StudioWindowHostRole::EntitlementTimerOwner);
    assert_eq!(second.role, StudioWindowHostRole::Observer);

    let focus = manager
        .execute_command(StudioAppWindowHostCommand::FocusWindow {
            window_id: second.window_id,
        })
        .expect("expected focus command");
    match focus {
        StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
            assert_eq!(dispatch.target_window_id, second.window_id);
        }
        other => panic!("expected focus dispatch outcome, got {other:?}"),
    }
    assert_eq!(manager.foreground_window_id(), Some(second.window_id));

    let trigger = manager
        .execute_command(StudioAppWindowHostCommand::DispatchTrigger {
            window_id: first.window_id,
            trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected trigger command");
    match trigger {
        StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
            assert_eq!(dispatch.target_window_id, first.window_id);
        }
        other => panic!("expected trigger dispatch outcome, got {other:?}"),
    }

    let global = manager
        .execute_command(StudioAppWindowHostCommand::DispatchGlobalEvent {
            event: StudioAppWindowHostGlobalEvent::TimerElapsed,
        })
        .expect("expected global event command");
    match global {
        StudioAppWindowHostCommandOutcome::WindowDispatched(dispatch) => {
            assert_eq!(dispatch.target_window_id, first.window_id);
        }
        other => panic!("expected global dispatch outcome, got {other:?}"),
    }

    let close = manager
        .execute_command(StudioAppWindowHostCommand::CloseWindow {
            window_id: first.window_id,
        })
        .expect("expected close command");
    match close {
        StudioAppWindowHostCommandOutcome::WindowClosed(close) => {
            assert_eq!(close.window_id, first.window_id);
            assert_eq!(close.next_foreground_window_id, Some(second.window_id));
        }
        other => panic!("expected close outcome, got {other:?}"),
    }
}

#[test]
fn app_window_host_manager_dispatches_run_panel_recovery_through_typed_entry() {
    let (config, project_path) = solver_failure_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    let window = manager.open_window();

    let run = manager
        .dispatch_trigger(
            window.window_id,
            &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");
    match &run.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
            crate::StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert!(matches!(
                    dispatch.outcome,
                    crate::StudioWorkspaceRunOutcome::Failed(_)
                ));
            }
            other => panic!("expected workspace run dispatch, got {other:?}"),
        },
        other => panic!("expected app command dispatch, got {other:?}"),
    }

    let recovery = manager
        .dispatch_run_panel_recovery_action(window.window_id)
        .expect("expected run panel recovery dispatch");

    assert_eq!(recovery.target_window_id, window.window_id);
    match &recovery.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
            assert_eq!(outcome.action.title, "Inspect unit inputs");
            assert_eq!(
                outcome.applied_target,
                Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                    "valve-1"
                )))
            );
        }
        other => panic!("expected run panel recovery dispatch, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_accept_canvas_suggestion_rejoins_automatic_mainline() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    manager.refresh_local_canvas_suggestions();
    let window = manager.open_window();

    let activate = manager
        .dispatch_run_panel_action(window.window_id, RunPanelActionId::SetActive)
        .expect("expected activate dispatch");
    match &activate.dispatch.host_output.runtime_output.report.dispatch {
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
    }

    let accepted = manager
        .accept_focused_canvas_suggestion_by_tab()
        .expect("expected canvas suggestion acceptance")
        .expect("expected focused suggestion");
    assert_eq!(
        accepted.id.as_str(),
        "local.flash_drum.create_outlet.flash-1.vapor"
    );

    let app_state = manager.session().host_port().runtime().app_state();
    assert_eq!(
        app_state.workspace.run_panel.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(app_state.workspace.run_panel.pending_reason, None);
    assert_eq!(
        app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
        Some("example-feed-heater-flash-rev-1-seq-1")
    );
    assert!(
        app_state
            .workspace
            .canvas_interaction
            .suggestions
            .iter()
            .all(|suggestion| {
                suggestion.id.as_str() != "local.flash_drum.create_outlet.flash-1.vapor"
            }),
        "accepted suggestion should be removed after local rules refresh"
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_executes_canvas_interaction_through_command_surface() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    manager.refresh_local_canvas_suggestions();
    let window = manager.open_window();

    let _ = manager
        .execute_command(StudioAppWindowHostCommand::DispatchTrigger {
            window_id: window.window_id,
            trigger: StudioRuntimeTrigger::WidgetAction(RunPanelActionId::SetActive),
        })
        .expect("expected activate command dispatch");

    let interaction = manager
        .execute_command(StudioAppWindowHostCommand::DispatchCanvasInteraction {
            action: StudioCanvasInteractionAction::AcceptFocusedByTab,
        })
        .expect("expected canvas interaction command");
    match interaction {
        StudioAppWindowHostCommandOutcome::CanvasInteracted(
            StudioAppWindowHostCanvasInteractionResult {
                action: StudioCanvasInteractionAction::AcceptFocusedByTab,
                accepted: Some(accepted),
                rejected: None,
                focused: None,
            },
        ) => {
            assert_eq!(
                accepted.id.as_str(),
                "local.flash_drum.create_outlet.flash-1.vapor"
            );
        }
        other => panic!("expected canvas interaction outcome, got {other:?}"),
    }

    let app_state = manager.session().host_port().runtime().app_state();
    assert_eq!(
        app_state.workspace.run_panel.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(app_state.workspace.run_panel.pending_reason, None);

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_dispatches_foreground_run_panel_recovery() {
    let (config, project_path) = solver_failure_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();
    let _ = manager
        .focus_window(second.window_id)
        .expect("expected second window focus");

    let _ = manager
        .dispatch_trigger(
            second.window_id,
            &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");

    let recovery = manager
        .dispatch_foreground_run_panel_recovery_action()
        .expect("expected recovery dispatch")
        .expect("expected foreground recovery dispatch");

    assert_eq!(recovery.target_window_id, second.window_id);
    assert_ne!(recovery.target_window_id, first.window_id);
    match &recovery.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
            assert_eq!(
                outcome.applied_target,
                Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                    "valve-1"
                )))
            );
        }
        other => panic!("expected run panel recovery dispatch, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_dispatches_foreground_entitlement_primary_action() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();
    let _ = manager
        .focus_window(second.window_id)
        .expect("expected second window focus");

    let dispatch = manager
        .dispatch_foreground_entitlement_primary_action()
        .expect("expected foreground entitlement primary result")
        .expect("expected foreground entitlement primary dispatch");

    assert_eq!(dispatch.target_window_id, second.window_id);
    assert_ne!(dispatch.target_window_id, first.window_id);
    assert_eq!(manager.foreground_window_id(), Some(second.window_id));
    match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioBootstrapDispatch::AppCommand(outcome) => {
            assert!(matches!(
                outcome.dispatch,
                crate::StudioAppResultDispatch::Entitlement(_)
            ));
        }
        other => panic!("expected entitlement app command dispatch, got {other:?}"),
    }
}

#[test]
fn app_window_host_manager_dispatches_foreground_entitlement_action() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();
    let _ = manager
        .focus_window(second.window_id)
        .expect("expected second window focus");

    let dispatch = manager
        .dispatch_foreground_entitlement_action(EntitlementActionId::SyncEntitlement)
        .expect("expected foreground entitlement action result")
        .expect("expected foreground entitlement action dispatch");

    assert_eq!(dispatch.target_window_id, second.window_id);
    assert_ne!(dispatch.target_window_id, first.window_id);
    assert_eq!(manager.foreground_window_id(), Some(second.window_id));
    match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioBootstrapDispatch::AppCommand(outcome) => match &outcome.dispatch {
            crate::StudioAppResultDispatch::Entitlement(entitlement) => {
                assert_eq!(
                    entitlement.action,
                    crate::StudioEntitlementAction::SyncEntitlement
                );
            }
            other => panic!("expected entitlement dispatch, got {other:?}"),
        },
        other => panic!("expected entitlement app command dispatch, got {other:?}"),
    }
}

#[test]
fn app_window_host_manager_dispatches_run_panel_recovery_via_ui_action() {
    let (config, project_path) = solver_failure_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();
    let _ = manager
        .focus_window(second.window_id)
        .expect("expected second window focus");
    let _ = manager
        .dispatch_trigger(
            second.window_id,
            &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");

    let recovery = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::RecoverRunPanelFailure)
        .expect("expected ui action dispatch")
        .expect("expected routed recovery dispatch");

    assert_eq!(recovery.target_window_id, second.window_id);
    assert_ne!(recovery.target_window_id, first.window_id);
    match &recovery.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
            assert_eq!(
                outcome.applied_target,
                Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                    "valve-1"
                )))
            );
        }
        other => panic!("expected run panel recovery dispatch, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_dispatches_run_manual_via_ui_action() {
    let (config, project_path) = solver_failure_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    let window = manager.open_window();

    let dispatch = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::RunManualWorkspace)
        .expect("expected ui action dispatch")
        .expect("expected routed run dispatch");

    assert_eq!(dispatch.target_window_id, window.window_id);
    assert_eq!(
        dispatch.dispatch.host_output.runtime_output.trigger,
        StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual)
    );
    match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
            crate::StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert!(matches!(
                    dispatch.outcome,
                    crate::StudioWorkspaceRunOutcome::Failed(_)
                ));
            }
            other => panic!("expected workspace run dispatch, got {other:?}"),
        },
        other => panic!("expected app command dispatch, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_dispatches_resume_via_ui_action() {
    let (config, project_path) = solver_failure_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    let window = manager.open_window();

    let dispatch = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::ResumeWorkspace)
        .expect("expected ui action dispatch")
        .expect("expected routed resume dispatch");

    assert_eq!(dispatch.target_window_id, window.window_id);
    assert_eq!(
        dispatch.dispatch.host_output.runtime_output.trigger,
        StudioRuntimeTrigger::WidgetAction(RunPanelActionId::Resume)
    );
    match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
            crate::StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert!(matches!(
                    dispatch.outcome,
                    crate::StudioWorkspaceRunOutcome::Failed(_)
                ));
            }
            other => panic!("expected workspace run dispatch, got {other:?}"),
        },
        other => panic!("expected app command dispatch, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_dispatches_activate_via_ui_action() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
    let window = manager.open_window();

    let dispatch = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::ActivateWorkspace)
        .expect("expected ui action dispatch")
        .expect("expected routed activate dispatch");

    assert_eq!(dispatch.target_window_id, window.window_id);
    assert_eq!(
        dispatch.dispatch.host_output.runtime_output.trigger,
        StudioRuntimeTrigger::WidgetAction(RunPanelActionId::SetActive)
    );
    match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
            crate::StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                assert_eq!(dispatch.simulation_mode, rf_ui::SimulationMode::Active);
            }
            other => panic!("expected workspace mode dispatch, got {other:?}"),
        },
        other => panic!("expected app command dispatch, got {other:?}"),
    }
}

#[test]
fn app_window_host_manager_dispatches_hold_via_ui_action_after_activation() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");
    let window = manager.open_window();
    let _ = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::ActivateWorkspace)
        .expect("expected ui action dispatch")
        .expect("expected routed activate dispatch");

    let dispatch = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::HoldWorkspace)
        .expect("expected ui action dispatch")
        .expect("expected routed hold dispatch");

    assert_eq!(dispatch.target_window_id, window.window_id);
    assert_eq!(
        dispatch.dispatch.host_output.runtime_output.trigger,
        StudioRuntimeTrigger::WidgetAction(RunPanelActionId::SetHold)
    );
    match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
            crate::StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                assert_eq!(dispatch.simulation_mode, rf_ui::SimulationMode::Hold);
            }
            other => panic!("expected workspace mode dispatch, got {other:?}"),
        },
        other => panic!("expected app command dispatch, got {other:?}"),
    }
}

#[test]
fn app_window_host_manager_reports_ui_action_states_for_run_panel_commands() {
    let (config, project_path) = solver_failure_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();
    let _ = manager
        .focus_window(second.window_id)
        .expect("expected second window focus");

    let run_manual = manager.ui_action_state(StudioAppWindowHostUiAction::RunManualWorkspace);
    assert_eq!(
        run_manual,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::RunManualWorkspace,
            availability: StudioAppWindowHostUiActionAvailability::Enabled {
                target_window_id: second.window_id,
            },
        }
    );
    assert!(run_manual.enabled());

    let resume = manager.ui_action_state(StudioAppWindowHostUiAction::ResumeWorkspace);
    assert_eq!(
        resume,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::ResumeWorkspace,
            availability: StudioAppWindowHostUiActionAvailability::Enabled {
                target_window_id: second.window_id,
            },
        }
    );
    assert!(resume.enabled());

    let hold = manager.ui_action_state(StudioAppWindowHostUiAction::HoldWorkspace);
    assert_eq!(
        hold,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::HoldWorkspace,
            availability: StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::HoldUnavailable,
                target_window_id: Some(second.window_id),
            },
        }
    );
    assert!(!hold.enabled());

    let activate = manager.ui_action_state(StudioAppWindowHostUiAction::ActivateWorkspace);
    assert_eq!(
        activate,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::ActivateWorkspace,
            availability: StudioAppWindowHostUiActionAvailability::Enabled {
                target_window_id: second.window_id,
            },
        }
    );
    assert!(activate.enabled());

    let initial = manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
    assert_eq!(
        initial,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
            availability: StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::NoRunPanelRecovery,
                target_window_id: Some(second.window_id),
            },
        }
    );
    assert!(!initial.enabled());
    assert_eq!(initial.target_window_id(), Some(second.window_id));

    let failed_run = manager
        .dispatch_trigger(
            second.window_id,
            &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");
    assert_eq!(failed_run.target_window_id, second.window_id);

    let recovery = manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
    assert_eq!(
        recovery,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
            availability: StudioAppWindowHostUiActionAvailability::Enabled {
                target_window_id: second.window_id,
            },
        }
    );
    assert!(recovery.enabled());

    let resume_disabled = manager.ui_action_state(StudioAppWindowHostUiAction::ResumeWorkspace);
    assert_eq!(
        resume_disabled,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::ResumeWorkspace,
            availability: StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::ResumeUnavailable,
                target_window_id: Some(second.window_id),
            },
        }
    );
    assert!(!resume_disabled.enabled());

    let sync_entitlement = manager.ui_action_state(StudioAppWindowHostUiAction::SyncEntitlement);
    assert_eq!(
        sync_entitlement,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::SyncEntitlement,
            availability: StudioAppWindowHostUiActionAvailability::Enabled {
                target_window_id: second.window_id,
            },
        }
    );
    assert!(sync_entitlement.enabled());

    let refresh_offline_lease =
        manager.ui_action_state(StudioAppWindowHostUiAction::RefreshOfflineLease);
    assert_eq!(
        refresh_offline_lease,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::RefreshOfflineLease,
            availability: StudioAppWindowHostUiActionAvailability::Enabled {
                target_window_id: second.window_id,
            },
        }
    );
    assert!(refresh_offline_lease.enabled());

    let states = manager.ui_action_states();
    assert_eq!(
        states,
        vec![
            run_manual.clone(),
            resume_disabled.clone(),
            hold.clone(),
            activate.clone(),
            recovery.clone(),
            sync_entitlement.clone(),
            refresh_offline_lease.clone(),
        ]
    );
    assert_ne!(run_manual.target_window_id(), Some(first.window_id));
    assert_ne!(resume_disabled.target_window_id(), Some(first.window_id));
    assert_ne!(hold.target_window_id(), Some(first.window_id));
    assert_ne!(activate.target_window_id(), Some(first.window_id));
    assert_ne!(recovery.target_window_id(), Some(first.window_id));
    assert_ne!(sync_entitlement.target_window_id(), Some(first.window_id));
    assert_ne!(
        refresh_offline_lease.target_window_id(),
        Some(first.window_id)
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_reports_ui_action_state_for_foreground_recovery() {
    let (config, project_path) = solver_failure_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();
    let _ = manager
        .focus_window(second.window_id)
        .expect("expected second window focus");

    let initial = manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
    assert_eq!(
        initial,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
            availability: StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::NoRunPanelRecovery,
                target_window_id: Some(second.window_id),
            },
        }
    );
    assert!(!initial.enabled());
    assert_eq!(initial.target_window_id(), Some(second.window_id));

    let _ = manager
        .dispatch_trigger(
            second.window_id,
            &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");

    let available = manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
    assert_eq!(
        available,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
            availability: StudioAppWindowHostUiActionAvailability::Enabled {
                target_window_id: second.window_id,
            },
        }
    );
    assert!(available.enabled());

    let states = manager.ui_action_states();
    assert_eq!(
        states,
        vec![
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::RunManualWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Enabled {
                    target_window_id: second.window_id,
                },
            },
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::ResumeWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::ResumeUnavailable,
                    target_window_id: Some(second.window_id),
                },
            },
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::HoldWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Disabled {
                    reason: StudioAppWindowHostUiActionDisabledReason::HoldUnavailable,
                    target_window_id: Some(second.window_id),
                },
            },
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::ActivateWorkspace,
                availability: StudioAppWindowHostUiActionAvailability::Enabled {
                    target_window_id: second.window_id,
                },
            },
            available.clone(),
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::SyncEntitlement,
                availability: StudioAppWindowHostUiActionAvailability::Enabled {
                    target_window_id: second.window_id,
                },
            },
            StudioAppWindowHostUiActionState {
                action: StudioAppWindowHostUiAction::RefreshOfflineLease,
                availability: StudioAppWindowHostUiActionAvailability::Enabled {
                    target_window_id: second.window_id,
                },
            },
        ]
    );
    assert_ne!(available.target_window_id(), Some(first.window_id));

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_routes_global_recovery_request_to_foreground_window() {
    let (config, project_path) = solver_failure_config();
    let mut manager = StudioAppWindowHostManager::new(&config).expect("expected manager");
    let first = manager.open_window();
    let second = manager.open_window();
    let _ = manager
        .focus_window(second.window_id)
        .expect("expected second window focus");
    let _ = manager
        .dispatch_trigger(
            second.window_id,
            &StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");

    let dispatch = manager
        .dispatch_global_event(StudioAppWindowHostGlobalEvent::RunPanelRecoveryRequested)
        .expect("expected global recovery dispatch")
        .expect("expected routed recovery dispatch");

    assert_eq!(dispatch.target_window_id, second.window_id);
    assert_ne!(dispatch.target_window_id, first.window_id);
    match &dispatch.dispatch.host_output.runtime_output.report.dispatch {
        crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
            assert_eq!(
                outcome.applied_target,
                Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                    "valve-1"
                )))
            );
        }
        other => panic!("expected run panel recovery dispatch, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_window_host_manager_ignores_ui_actions_without_windows() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");

    let run_manual = manager.ui_action_state(StudioAppWindowHostUiAction::RunManualWorkspace);
    assert_eq!(
        run_manual,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::RunManualWorkspace,
            availability: StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        }
    );
    assert!(!run_manual.enabled());
    assert_eq!(run_manual.target_window_id(), None);

    let resume = manager.ui_action_state(StudioAppWindowHostUiAction::ResumeWorkspace);
    assert_eq!(
        resume,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::ResumeWorkspace,
            availability: StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        }
    );
    assert!(!resume.enabled());
    assert_eq!(resume.target_window_id(), None);

    let hold = manager.ui_action_state(StudioAppWindowHostUiAction::HoldWorkspace);
    assert_eq!(
        hold,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::HoldWorkspace,
            availability: StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        }
    );
    assert!(!hold.enabled());
    assert_eq!(hold.target_window_id(), None);

    let activate = manager.ui_action_state(StudioAppWindowHostUiAction::ActivateWorkspace);
    assert_eq!(
        activate,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::ActivateWorkspace,
            availability: StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        }
    );
    assert!(!activate.enabled());
    assert_eq!(activate.target_window_id(), None);

    let recovery = manager.ui_action_state(StudioAppWindowHostUiAction::RecoverRunPanelFailure);
    assert_eq!(
        recovery,
        StudioAppWindowHostUiActionState {
            action: StudioAppWindowHostUiAction::RecoverRunPanelFailure,
            availability: StudioAppWindowHostUiActionAvailability::Disabled {
                reason: StudioAppWindowHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        }
    );
    assert!(!recovery.enabled());
    assert_eq!(recovery.target_window_id(), None);

    let run_dispatch = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::RunManualWorkspace)
        .expect("expected ui action result");
    assert!(run_dispatch.is_none());

    let resume_dispatch = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::ResumeWorkspace)
        .expect("expected ui action result");
    assert!(resume_dispatch.is_none());

    let hold_dispatch = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::HoldWorkspace)
        .expect("expected ui action result");
    assert!(hold_dispatch.is_none());

    let activate_dispatch = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::ActivateWorkspace)
        .expect("expected ui action result");
    assert!(activate_dispatch.is_none());

    let recovery_dispatch = manager
        .dispatch_ui_action(StudioAppWindowHostUiAction::RecoverRunPanelFailure)
        .expect("expected ui action result");

    assert!(recovery_dispatch.is_none());
}

#[test]
fn app_window_host_manager_command_entry_surfaces_ignored_cases() {
    let mut manager =
        StudioAppWindowHostManager::new(&lease_expiring_config()).expect("expected manager");

    let ignored_global = manager
        .execute_command(StudioAppWindowHostCommand::DispatchGlobalEvent {
            event: StudioAppWindowHostGlobalEvent::NetworkRestored,
        })
        .expect("expected ignored global event");
    assert_eq!(
        ignored_global,
        StudioAppWindowHostCommandOutcome::IgnoredGlobalEvent {
            event: StudioAppWindowHostGlobalEvent::NetworkRestored,
        }
    );

    let ignored_action = manager
        .execute_command(StudioAppWindowHostCommand::DispatchUiAction {
            action: StudioAppWindowHostUiAction::RunManualWorkspace,
        })
        .expect("expected ignored ui action");
    assert_eq!(
        ignored_action,
        StudioAppWindowHostCommandOutcome::IgnoredUiAction
    );

    let window = match manager
        .execute_command(StudioAppWindowHostCommand::OpenWindow)
        .expect("expected window open")
    {
        StudioAppWindowHostCommandOutcome::WindowOpened(registration) => registration,
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    let _ = manager
        .execute_command(StudioAppWindowHostCommand::CloseWindow {
            window_id: window.window_id,
        })
        .expect("expected first close");
    let ignored_close = manager
        .execute_command(StudioAppWindowHostCommand::CloseWindow {
            window_id: window.window_id,
        })
        .expect("expected ignored close");
    assert_eq!(
        ignored_close,
        StudioAppWindowHostCommandOutcome::IgnoredClose {
            window_id: window.window_id,
        }
    );
}
