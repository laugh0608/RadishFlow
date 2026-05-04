use super::*;

#[test]
fn app_host_returns_snapshot_with_window_open_and_focus_updates() {
    let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");

    let first = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected first window open");
    let first_window = match &first.outcome {
        StudioAppHostCommandOutcome::WindowOpened(opened) => {
            super::registration_from_opened_window(opened)
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    assert_eq!(
        first_window.role,
        StudioWindowHostRole::EntitlementTimerOwner
    );
    assert_eq!(
        first.snapshot.registered_windows,
        vec![first_window.window_id]
    );
    assert_eq!(
        first.snapshot.foreground_window_id,
        Some(first_window.window_id)
    );
    assert_eq!(
        first.snapshot.entitlement_timer_owner_window_id,
        Some(first_window.window_id)
    );
    assert_eq!(
        first.snapshot.windows,
        vec![crate::StudioAppHostWindowSnapshot {
            window_id: first_window.window_id,
            role: StudioWindowHostRole::EntitlementTimerOwner,
            layout_slot: 1,
            is_foreground: true,
            entitlement_timer: None,
        }]
    );
    assert_eq!(
        first.snapshot.ui_actions,
        vec![
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::SaveDocument,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: first_window.window_id,
                },
            },
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::UndoDocumentCommand,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::UndoUnavailable,
                    target_window_id: Some(first_window.window_id),
                },
            },
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::RedoDocumentCommand,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::RedoUnavailable,
                    target_window_id: Some(first_window.window_id),
                },
            },
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::RunManualWorkspace,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: first_window.window_id,
                },
            },
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::ResumeWorkspace,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: first_window.window_id,
                },
            },
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::HoldWorkspace,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::HoldUnavailable,
                    target_window_id: Some(first_window.window_id),
                },
            },
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::ActivateWorkspace,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: first_window.window_id,
                },
            },
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::RecoverRunPanelFailure,
                availability: StudioAppHostUiActionAvailability::Disabled {
                    reason: StudioAppHostUiActionDisabledReason::NoRunPanelRecovery,
                    target_window_id: Some(first_window.window_id),
                },
            },
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::SyncEntitlement,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: first_window.window_id,
                },
            },
            StudioAppHostUiActionState {
                action: StudioAppHostUiAction::RefreshOfflineLease,
                availability: StudioAppHostUiActionAvailability::Enabled {
                    target_window_id: first_window.window_id,
                },
            },
        ]
    );
    assert_eq!(
        first.changes.window_changes,
        vec![StudioAppHostWindowChange::Added {
            current: crate::StudioAppHostWindowSnapshot {
                window_id: first_window.window_id,
                role: StudioWindowHostRole::EntitlementTimerOwner,
                layout_slot: 1,
                is_foreground: true,
                entitlement_timer: None,
            },
        }]
    );
    assert_eq!(
        first.changes.foreground_window_change,
        Some(StudioAppHostWindowSelectionChange {
            previous: None,
            current: Some(first_window.window_id),
        })
    );
    assert_eq!(
        first.changes.entitlement_timer_owner_change,
        Some(StudioAppHostWindowSelectionChange {
            previous: None,
            current: Some(first_window.window_id),
        })
    );
    assert_eq!(first.changes.parked_entitlement_timer_change, None);

    let second = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected second window open");
    let second_window = match &second.outcome {
        StudioAppHostCommandOutcome::WindowOpened(opened) => {
            super::registration_from_opened_window(opened)
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    assert_eq!(second_window.role, StudioWindowHostRole::Observer);
    assert_eq!(
        second.snapshot.registered_windows,
        vec![first_window.window_id, second_window.window_id]
    );
    assert_eq!(
        second.snapshot.foreground_window_id,
        Some(first_window.window_id)
    );
    assert_eq!(second.snapshot.windows.len(), 2);
    assert_eq!(
        second.snapshot.windows[0].role,
        StudioWindowHostRole::EntitlementTimerOwner
    );
    assert!(second.snapshot.windows[0].is_foreground);
    assert_eq!(
        second.snapshot.windows[1].role,
        StudioWindowHostRole::Observer
    );
    assert!(!second.snapshot.windows[1].is_foreground);
    assert_eq!(
        second.changes.window_changes,
        vec![StudioAppHostWindowChange::Added {
            current: crate::StudioAppHostWindowSnapshot {
                window_id: second_window.window_id,
                role: StudioWindowHostRole::Observer,
                layout_slot: 1,
                is_foreground: false,
                entitlement_timer: None,
            },
        }]
    );
    assert_eq!(second.changes.foreground_window_change, None);
    assert_eq!(second.changes.entitlement_timer_owner_change, None);

    let focused = app_host
        .execute_command(StudioAppHostCommand::FocusWindow {
            window_id: second_window.window_id,
        })
        .expect("expected focus command");
    assert_eq!(
        focused.snapshot.foreground_window_id,
        Some(second_window.window_id)
    );
    assert_eq!(
        focused.snapshot.entitlement_timer_owner_window_id,
        Some(first_window.window_id)
    );
    assert_eq!(
        focused.snapshot.windows[1],
        crate::StudioAppHostWindowSnapshot {
            window_id: second_window.window_id,
            role: StudioWindowHostRole::Observer,
            layout_slot: 1,
            is_foreground: true,
            entitlement_timer: None,
        }
    );
    let focused_owner_timer = focused.snapshot.windows[0].entitlement_timer.clone();
    assert_eq!(
        focused.changes.window_changes,
        vec![
            StudioAppHostWindowChange::Updated {
                previous: crate::StudioAppHostWindowSnapshot {
                    window_id: first_window.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    layout_slot: 1,
                    is_foreground: true,
                    entitlement_timer: None,
                },
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: first_window.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    layout_slot: 1,
                    is_foreground: false,
                    entitlement_timer: focused_owner_timer,
                },
            },
            StudioAppHostWindowChange::Updated {
                previous: crate::StudioAppHostWindowSnapshot {
                    window_id: second_window.window_id,
                    role: StudioWindowHostRole::Observer,
                    layout_slot: 1,
                    is_foreground: false,
                    entitlement_timer: None,
                },
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: second_window.window_id,
                    role: StudioWindowHostRole::Observer,
                    layout_slot: 1,
                    is_foreground: true,
                    entitlement_timer: None,
                },
            },
        ]
    );
    assert_eq!(
        focused.changes.foreground_window_change,
        Some(StudioAppHostWindowSelectionChange {
            previous: Some(first_window.window_id),
            current: Some(second_window.window_id),
        })
    );
    assert_eq!(focused.changes.entitlement_timer_owner_change, None);
}

#[test]
fn app_host_snapshot_tracks_parked_timer_across_last_close_and_reopen() {
    let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
    let first = match app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected first window open")
        .outcome
    {
        StudioAppHostCommandOutcome::WindowOpened(opened) => {
            super::registration_from_opened_window(opened)
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };

    let triggered = app_host
        .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
            window_id: first.window_id,
            trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger");
    assert_eq!(
        triggered.snapshot.entitlement_timer_owner_window_id,
        Some(first.window_id)
    );
    assert_eq!(triggered.snapshot.windows.len(), 1);
    assert_eq!(
        triggered.snapshot.windows[0]
            .entitlement_timer
            .as_ref()
            .map(|slot| slot.effect_id),
        Some(1)
    );
    let triggered_timer = triggered.snapshot.windows[0]
        .entitlement_timer
        .clone()
        .expect("expected timer slot");
    assert_eq!(
        triggered.changes.window_changes,
        vec![StudioAppHostWindowChange::Updated {
            previous: crate::StudioAppHostWindowSnapshot {
                window_id: first.window_id,
                role: StudioWindowHostRole::EntitlementTimerOwner,
                layout_slot: 1,
                is_foreground: true,
                entitlement_timer: None,
            },
            current: crate::StudioAppHostWindowSnapshot {
                window_id: first.window_id,
                role: StudioWindowHostRole::EntitlementTimerOwner,
                layout_slot: 1,
                is_foreground: true,
                entitlement_timer: Some(triggered_timer.clone()),
            },
        }]
    );
    assert_eq!(triggered.changes.foreground_window_change, None);
    assert_eq!(triggered.changes.entitlement_timer_owner_change, None);

    let closed = app_host
        .execute_command(StudioAppHostCommand::CloseWindow {
            window_id: first.window_id,
        })
        .expect("expected close command");
    match &closed.outcome {
        StudioAppHostCommandOutcome::WindowClosed(close) => {
            assert_eq!(
                close.shutdown.host_shutdown.retirement,
                StudioWindowHostRetirement::Parked {
                    parked_entitlement_timer: close
                        .shutdown
                        .host_shutdown
                        .cleared_entitlement_timer
                        .clone(),
                }
            );
        }
        other => panic!("expected window closed outcome, got {other:?}"),
    }
    assert!(closed.snapshot.registered_windows.is_empty());
    assert!(closed.snapshot.windows.is_empty());
    assert!(closed.snapshot.foreground_window_id.is_none());
    assert!(closed.snapshot.entitlement_timer_owner_window_id.is_none());
    assert_eq!(
        closed
            .snapshot
            .parked_entitlement_timer
            .as_ref()
            .map(|slot| slot.effect_id),
        Some(1)
    );
    let parked_timer = closed
        .snapshot
        .parked_entitlement_timer
        .clone()
        .expect("expected parked timer");
    assert_eq!(
        closed.changes.window_changes,
        vec![StudioAppHostWindowChange::Removed {
            previous: crate::StudioAppHostWindowSnapshot {
                window_id: first.window_id,
                role: StudioWindowHostRole::EntitlementTimerOwner,
                layout_slot: 1,
                is_foreground: true,
                entitlement_timer: Some(triggered_timer.clone()),
            },
        }]
    );
    assert_eq!(
        closed.changes.foreground_window_change,
        Some(StudioAppHostWindowSelectionChange {
            previous: Some(first.window_id),
            current: None,
        })
    );
    assert_eq!(
        closed.changes.entitlement_timer_owner_change,
        Some(StudioAppHostWindowSelectionChange {
            previous: Some(first.window_id),
            current: None,
        })
    );
    assert_eq!(
        closed.changes.parked_entitlement_timer_change,
        Some(StudioAppHostTimerSlotChange {
            previous: None,
            current: Some(parked_timer.clone()),
        })
    );

    let reopened = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected reopen");
    assert_eq!(reopened.snapshot.parked_entitlement_timer, None);
    assert_eq!(reopened.snapshot.registered_windows.len(), 1);
    assert_eq!(
        reopened.snapshot.windows[0]
            .entitlement_timer
            .as_ref()
            .map(|slot| slot.effect_id),
        Some(1)
    );
    let restored_timer = reopened.snapshot.windows[0]
        .entitlement_timer
        .clone()
        .expect("expected restored timer");
    assert!(matches!(
        reopened.outcome,
        StudioAppHostCommandOutcome::WindowOpened(_)
    ));
    assert_eq!(
        reopened.changes.window_changes,
        vec![StudioAppHostWindowChange::Added {
            current: crate::StudioAppHostWindowSnapshot {
                window_id: reopened.snapshot.windows[0].window_id,
                role: StudioWindowHostRole::EntitlementTimerOwner,
                layout_slot: 1,
                is_foreground: true,
                entitlement_timer: Some(restored_timer.clone()),
            },
        }]
    );
    assert_eq!(
        reopened.changes.foreground_window_change,
        Some(StudioAppHostWindowSelectionChange {
            previous: None,
            current: Some(reopened.snapshot.windows[0].window_id),
        })
    );
    assert_eq!(
        reopened.changes.entitlement_timer_owner_change,
        Some(StudioAppHostWindowSelectionChange {
            previous: None,
            current: Some(reopened.snapshot.windows[0].window_id),
        })
    );
    assert_eq!(
        reopened.changes.parked_entitlement_timer_change,
        Some(StudioAppHostTimerSlotChange {
            previous: Some(parked_timer),
            current: None,
        })
    );
}
