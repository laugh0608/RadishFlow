use super::*;

#[test]
fn platform_host_dispatches_native_timer_callback_and_executes_platform_timer() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let mut start_executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9001,
        },
    ]);
    let _ = host
        .dispatch_event_and_execute_platform_timer(
            StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                    crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            },
            &mut start_executor,
        )
        .expect("expected initial platform dispatch execution");

    let mut callback_executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9002,
        },
    ]);
    let callback = host
        .dispatch_native_timer_elapsed_by_native_id_and_execute_platform_timer(
            9001,
            &mut callback_executor,
        )
        .expect("expected callback execution");

    match callback {
        StudioGuiPlatformExecutedNativeTimerCallbackOutcome::Dispatched(executed) => {
            assert!(matches!(
                executed.dispatch.native_timer_request,
                Some(StudioGuiPlatformTimerRequest::Rearm { .. })
            ));
            assert!(matches!(
                executed.timer_execution,
                StudioGuiPlatformTimerExecutionOutcome::Executed {
                    host_outcome: StudioGuiPlatformTimerHostOutcome::Started(
                        StudioGuiPlatformTimerStartedOutcome::Applied(_)
                    ),
                    ..
                }
            ));
        }
        other => panic!("expected dispatched callback execution, got {other:?}"),
    }
}

#[test]
fn platform_host_reports_ignored_native_timer_callback_during_combined_execution() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let mut executor = TestPlatformTimerExecutor::default();

    let callback = host
        .dispatch_native_timer_elapsed_by_native_id_and_execute_platform_timer(9001, &mut executor)
        .expect("expected ignored callback outcome");

    assert_eq!(
        callback,
        StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
            native_timer_id: 9001,
        }
    );
    assert!(executor.commands.is_empty());
    assert!(executor.follow_up_commands.is_empty());
}

#[test]
fn platform_host_batches_native_timer_callbacks_without_sync_execution() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let first = host
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger dispatch");
    let schedule = match first.native_timer_request.as_ref() {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => schedule.clone(),
        other => panic!("expected arm timer request, got {other:?}"),
    };
    let _ = host.apply_platform_timer_request(first.native_timer_request.as_ref());
    let _ = host.acknowledge_platform_timer_started(&schedule, 9001);

    let batch = host
        .dispatch_native_timer_elapsed_by_native_ids(&[9001, 9999])
        .expect("expected callback batch");

    assert_eq!(batch.len(), 2);
    assert!(!batch.is_empty());
    match &batch.callbacks[0] {
        StudioGuiPlatformNativeTimerCallbackOutcome::Dispatched(dispatch) => {
            assert!(matches!(
                dispatch.native_timer_request,
                Some(StudioGuiPlatformTimerRequest::Rearm { .. })
            ));
        }
        other => panic!("expected dispatched callback outcome, got {other:?}"),
    }
    assert_eq!(
        batch.callbacks[1],
        StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
            native_timer_id: 9999,
        }
    );
    assert_eq!(batch.native_timer_requests().len(), 1);
    assert!(matches!(
        batch.native_timer_requests().first(),
        Some(StudioGuiPlatformTimerRequest::Rearm { .. })
    ));
    assert_eq!(batch.snapshot, host.snapshot());
    let window = batch.window_model_for_window(Some(window_id));
    assert_eq!(window.layout_state.scope.window_id, Some(window_id));
}

#[test]
fn platform_host_batches_native_timer_callbacks_and_exposes_final_snapshot() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let mut start_executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9001,
        },
    ]);
    let _ = host
        .dispatch_event_and_execute_platform_timer(
            StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                    crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            },
            &mut start_executor,
        )
        .expect("expected initial platform dispatch execution");

    let mut callback_executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9002,
        },
    ]);
    let batch = host
        .dispatch_native_timer_elapsed_by_native_ids_and_execute_platform_timers(
            &[9001, 9999],
            &mut callback_executor,
        )
        .expect("expected callback batch execution");

    assert_eq!(batch.len(), 2);
    assert!(!batch.is_empty());
    match &batch.callbacks[0] {
        StudioGuiPlatformExecutedNativeTimerCallbackOutcome::Dispatched(executed) => {
            assert!(matches!(
                executed.dispatch.native_timer_request,
                Some(StudioGuiPlatformTimerRequest::Rearm { .. })
            ));
        }
        other => panic!("expected dispatched callback outcome, got {other:?}"),
    }
    assert_eq!(
        batch.callbacks[1],
        StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
            native_timer_id: 9999,
        }
    );
    assert_eq!(batch.snapshot, host.snapshot());
    assert!(matches!(
        batch.next_native_timer_schedule.as_ref(),
        Some(schedule) if schedule.window_id == Some(window_id)
    ));
    let window = batch.window_model_for_window(Some(window_id));
    assert_eq!(window.layout_state.scope.window_id, Some(window_id));
    assert!(matches!(
        host.current_platform_timer_binding(),
        Some(binding) if binding.native_timer_id == 9002
    ));
}

#[test]
fn platform_host_batches_due_timer_dispatches_without_sync_execution() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let first = host
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected first timer trigger");
    let schedule = match first.native_timer_request.as_ref() {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => schedule.clone(),
        other => panic!("expected arm timer request, got {other:?}"),
    };
    let _ = host.apply_platform_timer_request(first.native_timer_request.as_ref());
    let _ = host.acknowledge_platform_timer_started(&schedule, 9001);
    let due_at = host
        .next_native_timer_due_at()
        .expect("expected native timer due time");

    let batch = host
        .dispatch_due_native_timer_events_batch(due_at)
        .expect("expected due timer batch");

    assert!(!batch.is_empty());
    assert_eq!(batch.now, due_at);
    assert_eq!(batch.snapshot, host.snapshot());
    assert_eq!(
        batch.next_native_timer_due_at(),
        host.next_native_timer_due_at()
    );
    assert!(batch.dispatches.iter().all(|dispatch| matches!(
        dispatch.native_timer_request,
        Some(StudioGuiPlatformTimerRequest::Rearm { .. }) | None
    )));
    assert!(
        batch
            .native_timer_requests()
            .iter()
            .all(|request| matches!(request, StudioGuiPlatformTimerRequest::Rearm { .. }))
    );
    let window = batch.window_model_for_window(Some(window_id));
    assert_eq!(window.layout_state.scope.window_id, Some(window_id));
}

#[test]
fn platform_host_processes_async_round_and_aggregates_requests() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let dispatched = host
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger dispatch");
    let schedule = match dispatched.native_timer_request.as_ref() {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => schedule.clone(),
        other => panic!("expected arm timer request, got {other:?}"),
    };
    let _ = host.apply_platform_timer_request(dispatched.native_timer_request.as_ref());

    let round = host
        .process_async_platform_round(StudioGuiPlatformAsyncRoundInput {
            native_timer_ids: vec![9001],
            started_feedbacks: vec![StudioGuiPlatformTimerStartedFeedback {
                schedule: schedule.clone(),
                native_timer_id: 9001,
            }],
            ..StudioGuiPlatformAsyncRoundInput::default()
        })
        .expect("expected async round");

    assert!(matches!(
        &round.started_feedback_batch.entries[0].outcome,
        StudioGuiPlatformTimerStartedOutcome::Applied(_)
    ));
    assert_eq!(round.start_failed_feedback_batch.len(), 0);
    assert_eq!(round.follow_up_commands(), Vec::new());
    assert_eq!(round.native_timer_requests().len(), 1);
    assert!(matches!(
        round.native_timer_requests().first(),
        Some(StudioGuiPlatformTimerRequest::Rearm { .. })
    ));
    assert_eq!(round.snapshot, host.snapshot());
    assert_eq!(
        round.next_native_timer_due_at(),
        host.next_native_timer_due_at()
    );
    let window = round.window_model_for_window(Some(window_id));
    assert_eq!(window.layout_state.scope.window_id, Some(window_id));
}

#[test]
fn platform_host_async_round_surfaces_cleanup_follow_up_commands() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let dispatched = host
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger dispatch");
    let schedule = match dispatched.native_timer_request.as_ref() {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => schedule.clone(),
        other => panic!("expected arm timer request, got {other:?}"),
    };
    let _ = host.apply_platform_timer_request(dispatched.native_timer_request.as_ref());
    let _ = host.acknowledge_platform_timer_start_failed(
        &schedule,
        "clear pending before stale started ack",
    );

    let round = host
        .process_async_platform_round(StudioGuiPlatformAsyncRoundInput {
            started_feedbacks: vec![StudioGuiPlatformTimerStartedFeedback {
                schedule,
                native_timer_id: 9002,
            }],
            ..StudioGuiPlatformAsyncRoundInput::default()
        })
        .expect("expected async round");

    assert_eq!(
        round.follow_up_commands(),
        vec![StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
            native_timer_id: 9002,
        }]
    );
    assert!(matches!(
        &round.started_feedback_batch.entries[0].outcome,
        StudioGuiPlatformTimerStartedOutcome::IgnoredMissingPendingSchedule { .. }
    ));
    assert_eq!(round.native_timer_requests(), Vec::new());
    assert_eq!(round.snapshot, host.snapshot());
}

#[test]
fn platform_host_async_round_actions_order_follow_up_before_timer_requests() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let dispatched = host
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger dispatch");
    let schedule = match dispatched.native_timer_request.as_ref() {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => schedule.clone(),
        other => panic!("expected arm timer request, got {other:?}"),
    };
    let _ = host.apply_platform_timer_request(dispatched.native_timer_request.as_ref());
    let _ = host.acknowledge_platform_timer_started(&schedule, 9001);

    let round = host
        .process_async_platform_round(StudioGuiPlatformAsyncRoundInput {
            started_feedbacks: vec![StudioGuiPlatformTimerStartedFeedback {
                schedule: schedule.clone(),
                native_timer_id: 9002,
            }],
            native_timer_ids: vec![9001],
            ..StudioGuiPlatformAsyncRoundInput::default()
        })
        .expect("expected async round");

    let actions = round.actions();
    assert_eq!(actions.len(), 2);
    assert_eq!(
        actions.first(),
        Some(&StudioGuiPlatformAsyncRoundAction::FollowUpCommand(
            StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
                native_timer_id: 9002,
            }
        ))
    );
    assert!(matches!(
        actions.get(1),
        Some(StudioGuiPlatformAsyncRoundAction::TimerRequest(
            StudioGuiPlatformTimerRequest::Rearm { .. }
        ))
    ));
}

#[test]
fn platform_host_executes_async_round_actions_and_exposes_final_snapshot() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let dispatched = host
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer trigger dispatch");
    let schedule = match dispatched.native_timer_request.as_ref() {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => schedule.clone(),
        other => panic!("expected arm timer request, got {other:?}"),
    };
    let _ = host.apply_platform_timer_request(dispatched.native_timer_request.as_ref());
    let _ = host.acknowledge_platform_timer_started(&schedule, 9001);

    let mut executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9003,
        },
    ]);
    let executed = host
        .process_async_platform_round_and_execute_actions(
            StudioGuiPlatformAsyncRoundInput {
                started_feedbacks: vec![StudioGuiPlatformTimerStartedFeedback {
                    schedule: schedule.clone(),
                    native_timer_id: 9002,
                }],
                native_timer_ids: vec![9001],
                ..StudioGuiPlatformAsyncRoundInput::default()
            },
            &mut executor,
        )
        .expect("expected executed async round");

    assert_eq!(executed.actions.len(), 2);
    assert_eq!(
        executed.actions.first(),
        Some(&StudioGuiPlatformExecutedAsyncRoundAction::FollowUpCommand(
            StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
                native_timer_id: 9002,
            }
        ))
    );
    assert!(matches!(
        executed.actions.get(1),
        Some(StudioGuiPlatformExecutedAsyncRoundAction::TimerRequest {
            request: StudioGuiPlatformTimerRequest::Rearm { .. },
            execution: StudioGuiPlatformTimerExecutionOutcome::Executed {
                host_outcome: StudioGuiPlatformTimerHostOutcome::Started(
                    StudioGuiPlatformTimerStartedOutcome::Applied(_)
                ),
                follow_up_command: None,
                ..
            }
        })
    ));
    assert_eq!(
        executor.follow_up_commands,
        vec![StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
            native_timer_id: 9002,
        }]
    );
    assert_eq!(executor.commands.len(), 1);
    assert!(matches!(
        executor.commands.first(),
        Some(crate::StudioGuiPlatformTimerCommand::Rearm { .. })
    ));
    assert_eq!(executed.snapshot, host.snapshot());
    assert_eq!(
        executed.next_native_timer_due_at(),
        host.next_native_timer_due_at()
    );
    let window = executed.window_model_for_window(Some(window_id));
    assert_eq!(window.layout_state.scope.window_id, Some(window_id));
    assert_eq!(
        host.current_platform_timer_binding()
            .map(|binding| binding.native_timer_id),
        Some(9003)
    );
}

#[test]
fn platform_host_batches_due_timer_drain_and_exposes_final_snapshot() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let mut start_executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9001,
        },
    ]);
    let _ = host
        .dispatch_event_and_execute_platform_timer(
            StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                    crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            },
            &mut start_executor,
        )
        .expect("expected initial platform dispatch execution");
    let due_at = host
        .next_native_timer_due_at()
        .expect("expected scheduled native timer");
    let mut due_executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9002,
        },
    ]);

    let drained = host
        .drain_due_native_timer_events_and_execute_platform_timers(due_at, &mut due_executor)
        .expect("expected due timer drain execution");

    assert!(!drained.is_empty());
    assert_eq!(drained.now, due_at);
    assert_eq!(drained.snapshot, host.snapshot());
    assert_eq!(
        drained.next_native_timer_due_at(),
        host.next_native_timer_due_at()
    );
    assert!(drained.dispatches.iter().all(|executed| matches!(
        executed.timer_execution,
        StudioGuiPlatformTimerExecutionOutcome::Executed {
            host_outcome: StudioGuiPlatformTimerHostOutcome::Started(
                StudioGuiPlatformTimerStartedOutcome::Applied(_)
            ),
            ..
        }
    )));
    let window = drained.window_model_for_window(Some(window_id));
    assert_eq!(window.layout_state.scope.window_id, Some(window_id));
    assert!(matches!(
        host.current_platform_timer_binding(),
        Some(binding) if binding.native_timer_id == 9002
    ));
}

#[test]
fn platform_host_combined_execution_refreshes_dispatch_snapshot_after_start_failure() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let opened = host
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let mut executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::StartFailed {
            detail: "simulated combined execution failure".to_string(),
        },
    ]);

    let executed = host
        .dispatch_event_and_execute_platform_timer(
            StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                    crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            },
            &mut executor,
        )
        .expect("expected platform dispatch execution");

    match executed.timer_execution {
        StudioGuiPlatformTimerExecutionOutcome::Executed {
            host_outcome:
                StudioGuiPlatformTimerHostOutcome::StartFailed(
                    StudioGuiPlatformTimerStartFailedOutcome::Applied(_),
                ),
            ..
        } => {}
        other => panic!("expected start failed platform timer outcome, got {other:?}"),
    }
    let platform_notice = executed
        .dispatch
        .snapshot
        .runtime
        .platform_notice
        .as_ref()
        .expect("expected platform notice in refreshed dispatch snapshot");
    assert_eq!(platform_notice.title, "Platform timer unavailable");
    assert!(
        platform_notice
            .message
            .contains("simulated combined execution failure")
    );
    let latest_log = executed
        .dispatch
        .window
        .runtime
        .latest_log_entry
        .as_ref()
        .expect("expected platform log entry in refreshed dispatch window");
    assert!(
        latest_log
            .message
            .contains("simulated combined execution failure")
    );
}

#[test]
fn platform_host_reports_latest_gui_error_line_from_activity_log() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");

    host.record_activity_line("regular gui activity");
    host.record_activity_line("event failed: [invalid_input] simulated dispatch failure");
    host.record_activity_line("timer dispatch failed [invalid_input]: simulated timer failure");

    assert_eq!(
        host.latest_gui_error_line(),
        Some("timer dispatch failed [invalid_input]: simulated timer failure")
    );
}
