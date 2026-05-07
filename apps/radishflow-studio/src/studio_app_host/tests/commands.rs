use super::*;

#[test]
fn app_host_controller_routes_global_recovery_event_to_foreground_window() {
    let (config, project_path) = solver_failure_config();
    let mut controller = StudioAppHostController::new(&config).expect("expected controller");
    let first = controller
        .open_window()
        .expect("expected first window open");
    let second = controller
        .open_window()
        .expect("expected second window open");
    let _ = controller
        .focus_window(second.registration.window_id)
        .expect("expected second window focus");
    let _ = controller
        .dispatch_window_trigger(
            second.registration.window_id,
            StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");

    let dispatch = controller
        .dispatch_global_event(StudioAppWindowHostGlobalEvent::RunPanelRecoveryRequested)
        .expect("expected global recovery event");
    let recovery = dispatch.dispatch.expect("expected recovery dispatch");

    assert_eq!(recovery.target_window_id, second.registration.window_id);
    assert_ne!(recovery.target_window_id, first.registration.window_id);
    match &recovery.effects.runtime_report.dispatch {
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
    assert_eq!(
        controller.state().foreground_window_id,
        Some(second.registration.window_id)
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_controller_ignores_recovery_ui_action_without_windows() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let recovery = controller
        .dispatch_ui_action(StudioAppHostUiAction::RecoverRunPanelFailure)
        .expect("expected optional recovery ui action");

    assert!(recovery.is_none());
}

#[test]
fn app_host_controller_ignores_ui_actions_without_windows() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    assert_eq!(
        controller
            .state()
            .ui_action_state(StudioAppHostUiAction::RunManualWorkspace),
        Some(&StudioAppHostUiActionState {
            action: StudioAppHostUiAction::RunManualWorkspace,
            availability: StudioAppHostUiActionAvailability::Disabled {
                reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        })
    );
    assert_eq!(
        controller
            .state()
            .ui_action_state(StudioAppHostUiAction::ResumeWorkspace),
        Some(&StudioAppHostUiActionState {
            action: StudioAppHostUiAction::ResumeWorkspace,
            availability: StudioAppHostUiActionAvailability::Disabled {
                reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        })
    );
    assert_eq!(
        controller
            .state()
            .ui_action_state(StudioAppHostUiAction::HoldWorkspace),
        Some(&StudioAppHostUiActionState {
            action: StudioAppHostUiAction::HoldWorkspace,
            availability: StudioAppHostUiActionAvailability::Disabled {
                reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        })
    );
    assert_eq!(
        controller
            .state()
            .ui_action_state(StudioAppHostUiAction::ActivateWorkspace),
        Some(&StudioAppHostUiActionState {
            action: StudioAppHostUiAction::ActivateWorkspace,
            availability: StudioAppHostUiActionAvailability::Disabled {
                reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        })
    );
    assert_eq!(
        controller
            .state()
            .ui_action_state(StudioAppHostUiAction::RecoverRunPanelFailure),
        Some(&StudioAppHostUiActionState {
            action: StudioAppHostUiAction::RecoverRunPanelFailure,
            availability: StudioAppHostUiActionAvailability::Disabled {
                reason: StudioAppHostUiActionDisabledReason::NoRegisteredWindow,
                target_window_id: None,
            },
        })
    );

    let run_manual = controller
        .dispatch_ui_action(StudioAppHostUiAction::RunManualWorkspace)
        .expect("expected optional ui action result");
    assert!(run_manual.is_none());

    let resume = controller
        .dispatch_ui_action(StudioAppHostUiAction::ResumeWorkspace)
        .expect("expected optional ui action result");
    assert!(resume.is_none());

    let hold = controller
        .dispatch_ui_action(StudioAppHostUiAction::HoldWorkspace)
        .expect("expected optional ui action result");
    assert!(hold.is_none());

    let activate = controller
        .dispatch_ui_action(StudioAppHostUiAction::ActivateWorkspace)
        .expect("expected optional ui action result");
    assert!(activate.is_none());

    let recovery = controller
        .dispatch_ui_action(StudioAppHostUiAction::RecoverRunPanelFailure)
        .expect("expected optional ui action result");

    assert!(recovery.is_none());

    let entitlement_primary = controller
        .dispatch_ui_action(StudioAppHostUiAction::RefreshOfflineLease)
        .expect("expected optional entitlement primary result");
    assert!(entitlement_primary.is_none());

    let entitlement_action = controller
        .dispatch_ui_action(StudioAppHostUiAction::SyncEntitlement)
        .expect("expected optional entitlement action result");
    assert!(entitlement_action.is_none());
}

#[test]
fn app_host_snapshot_derives_enabled_ui_command_model_after_failed_run() {
    let (config, project_path) = solver_failure_config();
    let mut controller = StudioAppHostController::new(&config).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");
    let _ = controller
        .dispatch_window_trigger(
            opened.registration.window_id,
            StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");

    let model = controller.state().ui_command_model();
    assert_eq!(
        model.action(StudioAppHostUiAction::UndoDocumentCommand),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::UndoDocumentCommand),
            command_id: "edit.undo",
            group: StudioAppHostUiCommandGroup::Edit,
            sort_order: 10,
            label: "Undo",
            enabled: false,
            detail: "There is no document command to undo in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        model.action(StudioAppHostUiAction::RedoDocumentCommand),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::RedoDocumentCommand),
            command_id: "edit.redo",
            group: StudioAppHostUiCommandGroup::Edit,
            sort_order: 20,
            label: "Redo",
            enabled: false,
            detail: "There is no document command to redo in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        model.action(StudioAppHostUiAction::RunManualWorkspace),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::RunManualWorkspace),
            command_id: "run_panel.run_manual",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 100,
            label: "Run workspace",
            enabled: true,
            detail: "Dispatch the current manual run action in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        model.action(StudioAppHostUiAction::ResumeWorkspace),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::ResumeWorkspace),
            command_id: "run_panel.resume_workspace",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 110,
            label: "Resume workspace",
            enabled: false,
            detail: "Resume is currently unavailable in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        model.action(StudioAppHostUiAction::HoldWorkspace),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::HoldWorkspace),
            command_id: "run_panel.set_hold",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 120,
            label: "Hold workspace",
            enabled: false,
            detail: "Hold is currently unavailable in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        model.action(StudioAppHostUiAction::ActivateWorkspace),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::ActivateWorkspace),
            command_id: "run_panel.set_active",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 130,
            label: "Activate workspace",
            enabled: true,
            detail: "Dispatch the current activate action in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        model.action(StudioAppHostUiAction::RecoverRunPanelFailure),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::RecoverRunPanelFailure),
            command_id: "run_panel.recover_failure",
            group: StudioAppHostUiCommandGroup::Recovery,
            sort_order: 200,
            label: "Recover run panel failure",
            enabled: true,
            detail: "Apply the current run panel recovery action in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        model.action(StudioAppHostUiAction::SyncEntitlement),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::SyncEntitlement),
            command_id: "entitlement.sync",
            group: StudioAppHostUiCommandGroup::Entitlement,
            sort_order: 300,
            label: "Sync entitlement",
            enabled: true,
            detail: "Dispatch the current entitlement sync action in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        model.action(StudioAppHostUiAction::RefreshOfflineLease),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::RefreshOfflineLease),
            command_id: "entitlement.refresh_offline_lease",
            group: StudioAppHostUiCommandGroup::Entitlement,
            sort_order: 310,
            label: "Refresh offline lease",
            enabled: true,
            detail: "Dispatch the current offline lease refresh action in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(model.actions.len(), 10);

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_executes_canvas_interaction_through_command_surface() {
    let (config, project_path) = flash_drum_local_rules_synced_config();
    let mut app_host = StudioAppHost::new(&config).expect("expected app host");
    app_host.refresh_local_canvas_suggestions();
    let opened = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected window open");
    let window_id = match opened.outcome {
        StudioAppHostCommandOutcome::WindowOpened(opened) => opened.window_id,
        other => panic!("expected window opened outcome, got {other:?}"),
    };

    let _ = app_host
        .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
            window_id,
            trigger: StudioRuntimeTrigger::WidgetAction(RunPanelActionId::SetActive),
        })
        .expect("expected activate command");

    let interaction = app_host
        .execute_command(StudioAppHostCommand::DispatchCanvasInteraction {
            action: StudioCanvasInteractionAction::AcceptFocusedByTab,
        })
        .expect("expected canvas interaction command");
    match interaction.outcome {
        StudioAppHostCommandOutcome::CanvasInteracted(result) => {
            assert_eq!(
                result.action,
                StudioCanvasInteractionAction::AcceptFocusedByTab
            );
            assert_eq!(
                result
                    .accepted
                    .as_ref()
                    .map(|suggestion| suggestion.id.as_str()),
                Some("local.flash_drum.create_outlet.flash-1.vapor")
            );
        }
        other => panic!("expected canvas interaction outcome, got {other:?}"),
    }
    assert_eq!(
        app_host.workspace_control_state().run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(app_host.workspace_control_state().pending_reason, None);

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_controller_reports_disabled_run_manual_command_without_windows() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let dispatch = controller
        .dispatch_ui_command("run_panel.run_manual")
        .expect("expected ui command result");

    assert_eq!(
        dispatch,
        StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
            command_id: "run_panel.run_manual".to_string(),
            detail: "Open a studio window before running the workspace".to_string(),
            target_window_id: None,
        }
    );
}

#[test]
fn app_host_controller_reports_disabled_resume_command_without_windows() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let dispatch = controller
        .dispatch_ui_command("run_panel.resume_workspace")
        .expect("expected ui command result");

    assert_eq!(
        dispatch,
        StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
            command_id: "run_panel.resume_workspace".to_string(),
            detail: "Open a studio window before resuming the workspace".to_string(),
            target_window_id: None,
        }
    );
}

#[test]
fn app_host_controller_reports_disabled_hold_command_without_windows() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let dispatch = controller
        .dispatch_ui_command("run_panel.set_hold")
        .expect("expected ui command result");

    assert_eq!(
        dispatch,
        StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
            command_id: "run_panel.set_hold".to_string(),
            detail: "Open a studio window before holding the workspace".to_string(),
            target_window_id: None,
        }
    );
}

#[test]
fn app_host_controller_reports_disabled_activate_command_without_windows() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let dispatch = controller
        .dispatch_ui_command("run_panel.set_active")
        .expect("expected ui command result");

    assert_eq!(
        dispatch,
        StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
            command_id: "run_panel.set_active".to_string(),
            detail: "Open a studio window before activating the workspace".to_string(),
            target_window_id: None,
        }
    );
}

#[test]
fn app_host_controller_reports_disabled_entitlement_sync_command_without_windows() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let dispatch = controller
        .dispatch_ui_command("entitlement.sync")
        .expect("expected ui command result");

    assert_eq!(
        dispatch,
        StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
            command_id: "entitlement.sync".to_string(),
            detail: "Open a studio window before syncing entitlement".to_string(),
            target_window_id: None,
        }
    );
}

#[test]
fn app_host_controller_dispatches_ui_command_by_command_id() {
    let (config, project_path) = solver_failure_config();
    let mut controller = StudioAppHostController::new(&config).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");

    let dispatch = controller
        .dispatch_ui_command("run_panel.run_manual")
        .expect("expected ui command dispatch");

    match dispatch {
        StudioAppHostUiCommandDispatchResult::Executed(run) => {
            assert_eq!(run.target_window_id, opened.registration.window_id);
            match &run.effects.runtime_report.dispatch {
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
        }
        other => panic!("expected executed ui command dispatch, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_controller_dispatches_entitlement_refresh_ui_command_by_command_id() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");

    let dispatch = controller
        .dispatch_ui_command("entitlement.refresh_offline_lease")
        .expect("expected ui command dispatch");

    match dispatch {
        StudioAppHostUiCommandDispatchResult::Executed(refresh) => {
            assert_eq!(refresh.target_window_id, opened.registration.window_id);
            match &refresh.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::Entitlement(dispatch) => {
                        assert_eq!(
                            dispatch.action,
                            crate::StudioEntitlementAction::RefreshOfflineLease
                        );
                    }
                    other => panic!("expected entitlement dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            }
        }
        other => panic!("expected executed ui command dispatch, got {other:?}"),
    }
}

#[test]
fn app_host_controller_dispatches_activate_ui_command_by_command_id() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");

    let dispatch = controller
        .dispatch_ui_command("run_panel.set_active")
        .expect("expected ui command dispatch");

    match dispatch {
        StudioAppHostUiCommandDispatchResult::Executed(activate) => {
            assert_eq!(activate.target_window_id, opened.registration.window_id);
            match &activate.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                        assert_eq!(dispatch.simulation_mode, rf_ui::SimulationMode::Active);
                    }
                    other => panic!("expected workspace mode dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            }
        }
        other => panic!("expected executed ui command dispatch, got {other:?}"),
    }
}

#[test]
fn app_host_controller_dispatches_resume_ui_command_by_command_id() {
    let (config, project_path) = solver_failure_config();
    let mut controller = StudioAppHostController::new(&config).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");

    let dispatch = controller
        .dispatch_ui_command("run_panel.resume_workspace")
        .expect("expected ui command dispatch");

    match dispatch {
        StudioAppHostUiCommandDispatchResult::Executed(resume) => {
            assert_eq!(resume.target_window_id, opened.registration.window_id);
            match &resume.effects.runtime_report.dispatch {
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
        }
        other => panic!("expected executed ui command dispatch, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_controller_dispatches_hold_ui_command_by_command_id_after_activation() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");
    let _ = controller
        .dispatch_ui_command("run_panel.set_active")
        .expect("expected activate command dispatch");

    let dispatch = controller
        .dispatch_ui_command("run_panel.set_hold")
        .expect("expected ui command dispatch");

    match dispatch {
        StudioAppHostUiCommandDispatchResult::Executed(hold) => {
            assert_eq!(hold.target_window_id, opened.registration.window_id);
            match &hold.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                        assert_eq!(dispatch.simulation_mode, rf_ui::SimulationMode::Hold);
                    }
                    other => panic!("expected workspace mode dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            }
        }
        other => panic!("expected executed ui command dispatch, got {other:?}"),
    }
}

#[test]
fn app_host_controller_reports_disabled_ui_command_by_command_id() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");

    let dispatch = controller
        .dispatch_ui_command("run_panel.recover_failure")
        .expect("expected ui command result");

    assert_eq!(
        dispatch,
        StudioAppHostUiCommandDispatchResult::IgnoredDisabled {
            command_id: "run_panel.recover_failure".to_string(),
            detail: "No run panel recovery action is currently available in the target window"
                .to_string(),
            target_window_id: Some(opened.registration.window_id),
        }
    );
}

#[test]
fn app_host_controller_dispatches_recovery_ui_command_by_command_id() {
    let (config, project_path) = solver_failure_config();
    let mut controller = StudioAppHostController::new(&config).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");
    let _ = controller
        .dispatch_window_trigger(
            opened.registration.window_id,
            StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");

    let dispatch = controller
        .dispatch_ui_command("run_panel.recover_failure")
        .expect("expected ui command dispatch");

    match dispatch {
        StudioAppHostUiCommandDispatchResult::Executed(recovery) => {
            assert_eq!(recovery.target_window_id, opened.registration.window_id);
            match &recovery.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::RunPanelRecovery(outcome) => {
                    assert_eq!(outcome.action.title, "Inspect unit inputs");
                }
                other => panic!("expected run panel recovery dispatch, got {other:?}"),
            }
        }
        other => panic!("expected executed ui command dispatch, got {other:?}"),
    }

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_controller_reports_missing_ui_command_by_command_id() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let dispatch = controller
        .dispatch_ui_command("run_panel.unknown")
        .expect("expected ui command result");

    assert_eq!(
        dispatch,
        StudioAppHostUiCommandDispatchResult::IgnoredMissing {
            command_id: "run_panel.unknown".to_string(),
        }
    );
}

#[test]
fn app_host_controller_returns_optional_results_for_ignored_cases() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let ignored_global = controller
        .dispatch_global_event(StudioAppWindowHostGlobalEvent::NetworkRestored)
        .expect("expected ignored global event");
    assert!(ignored_global.dispatch.is_none());
    assert!(ignored_global.projection.state.windows.is_empty());

    let opened = controller.open_window().expect("expected window open");
    let closed = controller
        .close_window(opened.registration.window_id)
        .expect("expected close");
    assert!(closed.close.is_some());

    let ignored_close = controller
        .close_window(opened.registration.window_id)
        .expect("expected ignored close");
    assert!(ignored_close.close.is_none());
    assert!(ignored_close.projection.state.windows.is_empty());
}

#[test]
fn app_host_controller_maps_close_side_effects_into_gui_facing_summary() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");
    let _ = controller
        .dispatch_window_trigger(
            opened.registration.window_id,
            StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        )
        .expect("expected timer trigger");

    let closed = controller
        .close_window(opened.registration.window_id)
        .expect("expected close");
    let close = closed.close.expect("expected close effects");

    assert_eq!(close.window_id, opened.registration.window_id);
    assert!(matches!(
        close.retirement,
        StudioWindowHostRetirement::Parked {
            parked_entitlement_timer: Some(_)
        }
    ));
    assert!(matches!(
        close.native_timer_transitions.as_slice(),
        [crate::StudioWindowTimerDriverTransition::ParkNativeTimer { from_window_id, .. }]
        if *from_window_id == opened.registration.window_id
    ));
    assert!(close.native_timer_acks.is_empty());
    assert!(matches!(
        closed.projection.state.entitlement_timer,
        StudioAppHostEntitlementTimerState::Parked { .. }
    ));
}
