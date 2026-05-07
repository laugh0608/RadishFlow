use super::*;

#[test]
fn app_host_controller_advances_state_through_typed_command_methods() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let opened = controller.open_window().expect("expected window open");
    assert_eq!(
        opened.projection.state.registered_windows,
        vec![opened.registration.window_id]
    );
    assert_eq!(
        controller.state().foreground_window_id,
        Some(opened.registration.window_id)
    );

    let dispatched = controller
        .dispatch_window_trigger(
            opened.registration.window_id,
            StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        )
        .expect("expected trigger dispatch");
    let owner_slot = dispatched
        .projection
        .state
        .window(opened.registration.window_id)
        .and_then(|window| window.entitlement_timer.clone())
        .expect("expected timer slot");

    assert_eq!(dispatched.target_window_id, opened.registration.window_id);
    assert!(matches!(
        dispatched.effects.entitlement_timer_effect,
        Some(StudioAppHostEntitlementTimerEffect::Rearm {
            owner_window_id,
            effect_id: 1,
            ..
        }) if owner_window_id == opened.registration.window_id
    ));
    assert!(matches!(
        dispatched.effects.native_timer_transitions.as_slice(),
        [crate::StudioWindowTimerDriverTransition::RearmNativeTimer { window_id, .. }]
        if *window_id == opened.registration.window_id
    ));
    assert_eq!(dispatched.effects.native_timer_acks.len(), 1);
    assert_eq!(
        controller.state().entitlement_timer,
        StudioAppHostEntitlementTimerState::Owned {
            owner_window_id: opened.registration.window_id,
            slot: Some(owner_slot),
        }
    );
}

#[test]
fn app_host_controller_dispatches_run_panel_recovery_through_typed_method() {
    let (config, project_path) = solver_failure_config();
    let mut controller = StudioAppHostController::new(&config).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");

    let run = controller
        .dispatch_window_trigger(
            opened.registration.window_id,
            StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        )
        .expect("expected failed run dispatch");
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

    let recovery = controller
        .dispatch_window_trigger(
            opened.registration.window_id,
            StudioRuntimeTrigger::WidgetRecoveryAction,
        )
        .expect("expected recovery dispatch");

    assert_eq!(recovery.target_window_id, opened.registration.window_id);
    match &recovery.effects.runtime_report.dispatch {
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
    assert_eq!(
        controller.state().foreground_window_id,
        Some(opened.registration.window_id)
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_controller_dispatches_run_panel_recovery_via_ui_action_to_foreground_window() {
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

    let recovery = controller
        .dispatch_ui_action(StudioAppHostUiAction::RecoverRunPanelFailure)
        .expect("expected recovery ui action call")
        .expect("expected recovery ui action dispatch");

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
fn app_host_controller_dispatches_refresh_offline_lease_via_ui_action() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
    let first = controller
        .open_window()
        .expect("expected first window open");
    let second = controller
        .open_window()
        .expect("expected second window open");
    let _ = controller
        .focus_window(second.registration.window_id)
        .expect("expected second window focus");

    let dispatch = controller
        .dispatch_ui_action(StudioAppHostUiAction::RefreshOfflineLease)
        .expect("expected refresh offline lease action call")
        .expect("expected refresh offline lease dispatch");

    assert_eq!(dispatch.target_window_id, second.registration.window_id);
    assert_ne!(dispatch.target_window_id, first.registration.window_id);
    assert_eq!(
        controller.state().foreground_window_id,
        Some(second.registration.window_id)
    );
    match &dispatch.effects.runtime_report.dispatch {
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
fn app_host_controller_dispatches_sync_entitlement_via_ui_action() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
    let first = controller
        .open_window()
        .expect("expected first window open");
    let second = controller
        .open_window()
        .expect("expected second window open");
    let _ = controller
        .focus_window(second.registration.window_id)
        .expect("expected second window focus");

    let dispatch = controller
        .dispatch_ui_action(StudioAppHostUiAction::SyncEntitlement)
        .expect("expected sync entitlement action call")
        .expect("expected sync entitlement dispatch");

    assert_eq!(dispatch.target_window_id, second.registration.window_id);
    assert_ne!(dispatch.target_window_id, first.registration.window_id);
    assert_eq!(
        controller.state().foreground_window_id,
        Some(second.registration.window_id)
    );
    match &dispatch.effects.runtime_report.dispatch {
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
fn app_host_controller_dispatches_run_panel_recovery_via_ui_action() {
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

    let recovery = controller
        .dispatch_ui_action(StudioAppHostUiAction::RecoverRunPanelFailure)
        .expect("expected ui action call")
        .expect("expected ui action dispatch");

    assert_eq!(recovery.target_window_id, second.registration.window_id);
    assert_ne!(recovery.target_window_id, first.registration.window_id);
    match &recovery.effects.runtime_report.dispatch {
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
    assert_eq!(
        controller.state().foreground_window_id,
        Some(second.registration.window_id)
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_controller_dispatches_run_manual_via_ui_action() {
    let (config, project_path) = solver_failure_config();
    let mut controller = StudioAppHostController::new(&config).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");

    let run = controller
        .dispatch_ui_action(StudioAppHostUiAction::RunManualWorkspace)
        .expect("expected ui action call")
        .expect("expected ui action dispatch");

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

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_controller_dispatches_resume_via_ui_action() {
    let (config, project_path) = solver_failure_config();
    let mut controller = StudioAppHostController::new(&config).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");

    let resume = controller
        .dispatch_ui_action(StudioAppHostUiAction::ResumeWorkspace)
        .expect("expected ui action call")
        .expect("expected ui action dispatch");

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

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_controller_dispatches_activate_via_ui_action() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");

    let activate = controller
        .dispatch_ui_action(StudioAppHostUiAction::ActivateWorkspace)
        .expect("expected ui action call")
        .expect("expected ui action dispatch");

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

#[test]
fn app_host_controller_dispatches_hold_via_ui_action_after_activation() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");
    let opened = controller.open_window().expect("expected window open");
    let _ = controller
        .dispatch_ui_action(StudioAppHostUiAction::ActivateWorkspace)
        .expect("expected ui action call")
        .expect("expected ui action dispatch");

    let hold = controller
        .dispatch_ui_action(StudioAppHostUiAction::HoldWorkspace)
        .expect("expected ui action call")
        .expect("expected ui action dispatch");

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

#[test]
fn app_host_snapshot_tracks_ui_action_availability_for_recovery() {
    let (config, project_path) = solver_failure_config();
    let mut app_host = StudioAppHost::new(&config).expect("expected app host");
    let first = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected first window open");
    let first_window = match &first.outcome {
        StudioAppHostCommandOutcome::WindowOpened(opened) => {
            super::registration_from_opened_window(opened)
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    let second = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected second window open");
    let second_window = match &second.outcome {
        StudioAppHostCommandOutcome::WindowOpened(opened) => {
            super::registration_from_opened_window(opened)
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    let focused = app_host
        .execute_command(StudioAppHostCommand::FocusWindow {
            window_id: second_window.window_id,
        })
        .expect("expected focus command");
    assert_eq!(
        focused
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::RunManualWorkspace)
            .expect("expected run manual ui action state"),
        &StudioAppHostUiActionState {
            action: StudioAppHostUiAction::RunManualWorkspace,
            availability: StudioAppHostUiActionAvailability::Enabled {
                target_window_id: second_window.window_id,
            },
        }
    );
    assert_eq!(
        focused
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::ResumeWorkspace)
            .expect("expected resume ui action state"),
        &StudioAppHostUiActionState {
            action: StudioAppHostUiAction::ResumeWorkspace,
            availability: StudioAppHostUiActionAvailability::Enabled {
                target_window_id: second_window.window_id,
            },
        }
    );
    assert_eq!(
        focused
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::HoldWorkspace)
            .expect("expected hold ui action state"),
        &StudioAppHostUiActionState {
            action: StudioAppHostUiAction::HoldWorkspace,
            availability: StudioAppHostUiActionAvailability::Disabled {
                reason: StudioAppHostUiActionDisabledReason::HoldUnavailable,
                target_window_id: Some(second_window.window_id),
            },
        }
    );
    assert_eq!(
        focused
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::ActivateWorkspace)
            .expect("expected activate ui action state"),
        &StudioAppHostUiActionState {
            action: StudioAppHostUiAction::ActivateWorkspace,
            availability: StudioAppHostUiActionAvailability::Enabled {
                target_window_id: second_window.window_id,
            },
        }
    );
    assert_eq!(
        focused
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::RecoverRunPanelFailure)
            .expect("expected recovery ui action state"),
        &StudioAppHostUiActionState {
            action: StudioAppHostUiAction::RecoverRunPanelFailure,
            availability: StudioAppHostUiActionAvailability::Disabled {
                reason: StudioAppHostUiActionDisabledReason::NoRunPanelRecovery,
                target_window_id: Some(second_window.window_id),
            },
        }
    );

    let failed_run = app_host
        .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
            window_id: second_window.window_id,
            trigger: StudioRuntimeTrigger::WidgetAction(RunPanelActionId::RunManual),
        })
        .expect("expected failed run dispatch");
    assert_eq!(
        failed_run
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::RecoverRunPanelFailure)
            .expect("expected recovery ui action state"),
        &StudioAppHostUiActionState {
            action: StudioAppHostUiAction::RecoverRunPanelFailure,
            availability: StudioAppHostUiActionAvailability::Enabled {
                target_window_id: second_window.window_id,
            },
        }
    );
    assert_eq!(
        failed_run
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::ResumeWorkspace)
            .expect("expected resume ui action state"),
        &StudioAppHostUiActionState {
            action: StudioAppHostUiAction::ResumeWorkspace,
            availability: StudioAppHostUiActionAvailability::Disabled {
                reason: StudioAppHostUiActionDisabledReason::ResumeUnavailable,
                target_window_id: Some(second_window.window_id),
            },
        }
    );
    assert_eq!(
        failed_run
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::HoldWorkspace)
            .expect("expected hold ui action state"),
        &StudioAppHostUiActionState {
            action: StudioAppHostUiAction::HoldWorkspace,
            availability: StudioAppHostUiActionAvailability::Disabled {
                reason: StudioAppHostUiActionDisabledReason::HoldUnavailable,
                target_window_id: Some(second_window.window_id),
            },
        }
    );
    assert_eq!(
        failed_run
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::ActivateWorkspace)
            .expect("expected activate ui action state"),
        &StudioAppHostUiActionState {
            action: StudioAppHostUiAction::ActivateWorkspace,
            availability: StudioAppHostUiActionAvailability::Enabled {
                target_window_id: second_window.window_id,
            },
        }
    );
    assert_ne!(
        failed_run
            .snapshot
            .ui_actions
            .iter()
            .find(|state| state.action == StudioAppHostUiAction::RecoverRunPanelFailure)
            .and_then(|state| state.target_window_id()),
        Some(first_window.window_id)
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn app_host_state_derives_ui_command_model_from_availability() {
    let mut controller =
        StudioAppHostController::new(&lease_expiring_config()).expect("expected controller");

    let disabled = controller.state().ui_command_model();
    assert_eq!(
        disabled.action(StudioAppHostUiAction::RunManualWorkspace),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::RunManualWorkspace),
            command_id: "run_panel.run_manual",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 100,
            label: "Run workspace",
            enabled: false,
            detail: "Open a studio window before running the workspace",
            target_window_id: None,
        })
    );
    assert_eq!(
        disabled.action(StudioAppHostUiAction::ResumeWorkspace),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::ResumeWorkspace),
            command_id: "run_panel.resume_workspace",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 110,
            label: "Resume workspace",
            enabled: false,
            detail: "Open a studio window before resuming the workspace",
            target_window_id: None,
        })
    );
    assert_eq!(
        disabled.action(StudioAppHostUiAction::HoldWorkspace),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::HoldWorkspace),
            command_id: "run_panel.set_hold",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 120,
            label: "Hold workspace",
            enabled: false,
            detail: "Open a studio window before holding the workspace",
            target_window_id: None,
        })
    );
    assert_eq!(
        disabled.action(StudioAppHostUiAction::ActivateWorkspace),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::ActivateWorkspace),
            command_id: "run_panel.set_active",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 130,
            label: "Activate workspace",
            enabled: false,
            detail: "Open a studio window before activating the workspace",
            target_window_id: None,
        })
    );
    assert_eq!(
        disabled.action(StudioAppHostUiAction::RecoverRunPanelFailure),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::RecoverRunPanelFailure),
            command_id: "run_panel.recover_failure",
            group: StudioAppHostUiCommandGroup::Recovery,
            sort_order: 200,
            label: "Recover run panel failure",
            enabled: false,
            detail: "Open a studio window before requesting run panel recovery",
            target_window_id: None,
        })
    );
    assert_eq!(
        disabled.command("run_panel.recover_failure"),
        disabled.action(StudioAppHostUiAction::RecoverRunPanelFailure)
    );

    let opened = controller.open_window().expect("expected window open");
    let no_recovery = opened.projection.state.ui_command_model();
    assert_eq!(
        no_recovery.action(StudioAppHostUiAction::SaveDocument),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::SaveDocument),
            command_id: "file.save",
            group: StudioAppHostUiCommandGroup::File,
            sort_order: 10,
            label: "Save",
            enabled: true,
            detail: "Save the current document to its project path",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        no_recovery.action(StudioAppHostUiAction::UndoDocumentCommand),
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
        no_recovery.action(StudioAppHostUiAction::RedoDocumentCommand),
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
        no_recovery.actions[3],
        StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::RunManualWorkspace),
            command_id: "run_panel.run_manual",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 100,
            label: "Run workspace",
            enabled: true,
            detail: "Dispatch the current manual run action in the target window",
            target_window_id: Some(opened.registration.window_id),
        }
    );
    assert_eq!(
        no_recovery.actions[4],
        StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::ResumeWorkspace),
            command_id: "run_panel.resume_workspace",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 110,
            label: "Resume workspace",
            enabled: true,
            detail: "Dispatch the current resume action in the target window",
            target_window_id: Some(opened.registration.window_id),
        }
    );
    assert_eq!(
        no_recovery.actions[5],
        StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::HoldWorkspace),
            command_id: "run_panel.set_hold",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 120,
            label: "Hold workspace",
            enabled: false,
            detail: "Hold is currently unavailable in the target window",
            target_window_id: Some(opened.registration.window_id),
        }
    );
    assert_eq!(
        no_recovery.actions[6],
        StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::ActivateWorkspace),
            command_id: "run_panel.set_active",
            group: StudioAppHostUiCommandGroup::RunPanel,
            sort_order: 130,
            label: "Activate workspace",
            enabled: true,
            detail: "Dispatch the current activate action in the target window",
            target_window_id: Some(opened.registration.window_id),
        }
    );
    assert_eq!(
        no_recovery.action(StudioAppHostUiAction::RecoverRunPanelFailure),
        Some(&StudioAppHostUiActionModel {
            action: Some(StudioAppHostUiAction::RecoverRunPanelFailure),
            command_id: "run_panel.recover_failure",
            group: StudioAppHostUiCommandGroup::Recovery,
            sort_order: 200,
            label: "Recover run panel failure",
            enabled: false,
            detail: "No run panel recovery action is currently available in the target window",
            target_window_id: Some(opened.registration.window_id),
        })
    );
    assert_eq!(
        no_recovery.command("file.save"),
        no_recovery.action(StudioAppHostUiAction::SaveDocument)
    );
    assert_eq!(
        no_recovery.command("run_panel.run_manual"),
        no_recovery.action(StudioAppHostUiAction::RunManualWorkspace)
    );
    assert_eq!(
        no_recovery.command("run_panel.resume_workspace"),
        no_recovery.action(StudioAppHostUiAction::ResumeWorkspace)
    );
    assert_eq!(
        no_recovery.command("run_panel.set_hold"),
        no_recovery.action(StudioAppHostUiAction::HoldWorkspace)
    );
    assert_eq!(
        no_recovery.command("run_panel.set_active"),
        no_recovery.action(StudioAppHostUiAction::ActivateWorkspace)
    );
}
