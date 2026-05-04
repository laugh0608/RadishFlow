use super::*;

#[test]
fn app_host_change_set_captures_owner_transfer_on_close() {
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
    let second = match app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected second window open")
        .outcome
    {
        StudioAppHostCommandOutcome::WindowOpened(opened) => {
            super::registration_from_opened_window(opened)
        }
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    let _ = app_host
        .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
            window_id: first.window_id,
            trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger");

    let closed = app_host
        .execute_command(StudioAppHostCommand::CloseWindow {
            window_id: first.window_id,
        })
        .expect("expected owner close");
    let transferred_timer = closed.snapshot.windows[0]
        .entitlement_timer
        .clone()
        .expect("expected transferred timer");

    assert_eq!(
        closed.changes.window_changes,
        vec![
            StudioAppHostWindowChange::Removed {
                previous: crate::StudioAppHostWindowSnapshot {
                    window_id: first.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    layout_slot: 1,
                    is_foreground: true,
                    entitlement_timer: Some(transferred_timer.clone()),
                },
            },
            StudioAppHostWindowChange::Updated {
                previous: crate::StudioAppHostWindowSnapshot {
                    window_id: second.window_id,
                    role: StudioWindowHostRole::Observer,
                    layout_slot: 1,
                    is_foreground: false,
                    entitlement_timer: None,
                },
                current: crate::StudioAppHostWindowSnapshot {
                    window_id: second.window_id,
                    role: StudioWindowHostRole::EntitlementTimerOwner,
                    layout_slot: 1,
                    is_foreground: true,
                    entitlement_timer: Some(transferred_timer),
                },
            },
        ]
    );
    assert_eq!(
        closed.changes.foreground_window_change,
        Some(StudioAppHostWindowSelectionChange {
            previous: Some(first.window_id),
            current: Some(second.window_id),
        })
    );
    assert_eq!(
        closed.changes.entitlement_timer_owner_change,
        Some(StudioAppHostWindowSelectionChange {
            previous: Some(first.window_id),
            current: Some(second.window_id),
        })
    );
    assert_eq!(closed.changes.parked_entitlement_timer_change, None);
}

#[test]
fn app_host_surfaces_ignored_global_events_with_stable_snapshot() {
    let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");

    let output = app_host
        .execute_command(StudioAppHostCommand::DispatchGlobalEvent {
            event: StudioAppWindowHostGlobalEvent::NetworkRestored,
        })
        .expect("expected ignored global event");

    assert_eq!(
        output.outcome,
        StudioAppHostCommandOutcome::IgnoredGlobalEvent {
            event: StudioAppWindowHostGlobalEvent::NetworkRestored,
        }
    );
    assert!(output.snapshot.registered_windows.is_empty());
    assert!(output.snapshot.windows.is_empty());
    assert!(output.snapshot.foreground_window_id.is_none());
    assert!(output.changes.window_changes.is_empty());
    assert_eq!(output.changes.foreground_window_change, None);
    assert_eq!(output.changes.entitlement_timer_owner_change, None);
    assert_eq!(output.changes.parked_entitlement_timer_change, None);
}

#[test]
fn app_host_store_projects_output_into_single_state_boundary() {
    let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
    let mut store = StudioAppHostStore::from_snapshot(&app_host.snapshot());

    let first_open = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected first window open");
    let first_projection = store.apply_output(&first_open);
    let first_window = first_projection
        .added_windows
        .first()
        .expect("expected first added window");

    assert_eq!(
        first_projection.state.registered_windows,
        vec![first_window.window_id]
    );
    assert_eq!(
        first_projection.state.foreground_window_id,
        Some(first_window.window_id)
    );
    assert_eq!(
        first_projection.state.entitlement_timer,
        StudioAppHostEntitlementTimerState::Owned {
            owner_window_id: first_window.window_id,
            slot: None,
        }
    );
    assert_eq!(first_projection.removed_window_ids, Vec::<u64>::new());
    assert!(first_projection.updated_windows.is_empty());

    let second_open = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected second window open");
    let second_projection = store.apply_output(&second_open);
    let second_window = second_projection
        .added_windows
        .first()
        .expect("expected second added window");

    assert_eq!(
        second_projection.state.registered_windows,
        vec![first_window.window_id, second_window.window_id]
    );
    assert_eq!(
        second_projection.state.entitlement_timer,
        StudioAppHostEntitlementTimerState::Owned {
            owner_window_id: first_window.window_id,
            slot: None,
        }
    );

    let focused = app_host
        .execute_command(StudioAppHostCommand::FocusWindow {
            window_id: second_window.window_id,
        })
        .expect("expected focus command");
    let focused_projection = store.apply_output(&focused);

    assert_eq!(
        focused_projection.state.foreground_window_id,
        Some(second_window.window_id)
    );
    assert_eq!(focused_projection.added_windows, Vec::new());
    assert_eq!(focused_projection.removed_window_ids, Vec::<u64>::new());
    assert_eq!(focused_projection.updated_windows.len(), 2);
    assert_eq!(
        focused_projection.state.entitlement_timer,
        StudioAppHostEntitlementTimerState::Owned {
            owner_window_id: first_window.window_id,
            slot: focused_projection
                .state
                .window(first_window.window_id)
                .and_then(|window| window.entitlement_timer.clone()),
        }
    );
    assert!(focused_projection.entitlement_timer_change.is_some());
}

#[test]
fn app_host_store_collapses_owner_and_parked_timer_semantics() {
    let mut app_host = StudioAppHost::new(&lease_expiring_config()).expect("expected app host");
    let mut store = StudioAppHostStore::from_snapshot(&app_host.snapshot());

    let opened = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected window open");
    let opened_projection = store.apply_output(&opened);
    let window_id = opened_projection
        .added_windows
        .first()
        .expect("expected opened window")
        .window_id;

    let triggered = app_host
        .execute_command(StudioAppHostCommand::DispatchWindowTrigger {
            window_id,
            trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger");
    let triggered_projection = store.apply_output(&triggered);
    let owned_slot = triggered_projection
        .state
        .window(window_id)
        .and_then(|window| window.entitlement_timer.clone())
        .expect("expected owned timer slot");

    assert_eq!(
        triggered_projection.state.entitlement_timer,
        StudioAppHostEntitlementTimerState::Owned {
            owner_window_id: window_id,
            slot: Some(owned_slot.clone()),
        }
    );

    let closed = app_host
        .execute_command(StudioAppHostCommand::CloseWindow { window_id })
        .expect("expected close command");
    let closed_projection = store.apply_output(&closed);

    assert!(closed_projection.state.windows.is_empty());
    assert_eq!(
        closed_projection.state.entitlement_timer,
        StudioAppHostEntitlementTimerState::Parked {
            slot: owned_slot.clone(),
        }
    );
    assert_eq!(closed_projection.removed_window_ids, vec![window_id]);

    let reopened = app_host
        .execute_command(StudioAppHostCommand::OpenWindow)
        .expect("expected reopen");
    let reopened_projection = store.apply_output(&reopened);
    let reopened_window_id = reopened_projection
        .added_windows
        .first()
        .expect("expected reopened window")
        .window_id;

    assert_eq!(
        reopened_projection.state.entitlement_timer,
        StudioAppHostEntitlementTimerState::Owned {
            owner_window_id: reopened_window_id,
            slot: reopened_projection
                .state
                .window(reopened_window_id)
                .and_then(|window| window.entitlement_timer.clone()),
        }
    );
    assert!(
        reopened_projection
            .state
            .parked_entitlement_timer()
            .is_none()
    );
}
