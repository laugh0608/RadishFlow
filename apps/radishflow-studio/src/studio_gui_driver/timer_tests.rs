use super::test_support::lease_expiring_config;
use super::*;

#[test]
fn gui_driver_tracks_parked_timer_restore_when_reopening_owner_window() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let first_window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected opened window outcome, got {other:?}"),
    };

    let _ = driver
        .dispatch_event(StudioGuiEvent::EntitlementTimerElapsed)
        .expect("expected timer elapsed dispatch");
    let closed = driver
        .dispatch_event(StudioGuiEvent::CloseWindowRequested {
            window_id: first_window_id,
        })
        .expect("expected close dispatch");
    match closed.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(closed)) => {
            assert!(
                closed
                    .native_timers
                    .operations
                    .iter()
                    .any(|operation| matches!(
                        operation,
                        crate::StudioGuiNativeTimerOperation::Park { .. }
                    ))
            );
        }
        other => panic!("expected window closed outcome, got {other:?}"),
    }
    assert!(driver.native_timer_runtime().parked_binding().is_some());

    let reopened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected reopen dispatch");
    let second_window_id = match reopened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            assert!(
                opened
                    .native_timers
                    .operations
                    .iter()
                    .any(|operation| matches!(
                        operation,
                        crate::StudioGuiNativeTimerOperation::RestoreParked { .. }
                    ))
            );
            opened.registration.window_id
        }
        other => panic!("expected reopened window outcome, got {other:?}"),
    };

    assert!(driver.native_timer_runtime().parked_binding().is_none());
    assert!(
        driver
            .native_timer_runtime()
            .window_binding(second_window_id)
            .is_some()
    );
}

#[test]
fn gui_driver_drains_due_native_timer_events_through_lifecycle_entry() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let _ = driver
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger");

    let due_at = driver
        .next_due_native_timer_at()
        .expect("expected scheduled native timer");
    let due_dispatches = driver
        .drain_due_native_timer_events(due_at)
        .expect("expected due timer dispatch");

    assert_eq!(due_dispatches.len(), 1);
    assert!(matches!(
        due_dispatches[0].outcome,
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(_))
    ));
    assert!(driver.next_due_native_timer_at().is_some());
}

#[test]
fn gui_driver_routes_native_timer_callback_for_current_binding() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let _ = driver
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger");
    let binding = driver
        .native_timer_runtime()
        .window_binding(window_id)
        .cloned()
        .expect("expected native timer binding");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::NativeTimerElapsed {
            window_id: Some(window_id),
            handle_id: binding.handle_id,
        })
        .expect("expected native timer callback dispatch");

    assert!(matches!(
        dispatch.outcome,
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(_))
    ));
}

#[test]
fn gui_driver_ignores_unknown_native_timer_callback_handle() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let _ = driver
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected first timer trigger");
    let current_handle_id = driver
        .native_timer_runtime()
        .window_binding(window_id)
        .map(|binding| binding.handle_id)
        .expect("expected initial binding");

    let ignored = driver
        .dispatch_event(StudioGuiEvent::NativeTimerElapsed {
            window_id: Some(window_id),
            handle_id: current_handle_id + 999,
        })
        .expect("expected unknown native timer callback");

    assert!(matches!(
        ignored.outcome,
        StudioGuiDriverOutcome::IgnoredNativeTimerElapsed {
            window_id: Some(_),
            ..
        }
    ));
}
