use rf_types::RfResult;

use crate::{
    StudioGuiEvent, StudioGuiPlatformAsyncRoundAction, StudioGuiPlatformAsyncRoundInput,
    StudioGuiPlatformExecutedAsyncRoundAction, StudioGuiPlatformExecutedNativeTimerCallbackOutcome,
    StudioGuiPlatformHost, StudioGuiPlatformNativeTimerCallbackOutcome,
    StudioGuiPlatformTimerExecutionOutcome, StudioGuiPlatformTimerExecutor,
    StudioGuiPlatformTimerExecutorResponse, StudioGuiPlatformTimerFollowUpCommand,
    StudioGuiPlatformTimerHostOutcome, StudioGuiPlatformTimerRequest,
    StudioGuiPlatformTimerStartFailedFeedback, StudioGuiPlatformTimerStartFailedOutcome,
    StudioGuiPlatformTimerStartedFeedback, StudioGuiPlatformTimerStartedOutcome,
    StudioRuntimeConfig, StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
};

#[derive(Default)]
struct TestPlatformTimerExecutor {
    responses: Vec<StudioGuiPlatformTimerExecutorResponse>,
    commands: Vec<crate::StudioGuiPlatformTimerCommand>,
    follow_up_commands: Vec<StudioGuiPlatformTimerFollowUpCommand>,
}

impl TestPlatformTimerExecutor {
    fn with_responses(responses: Vec<StudioGuiPlatformTimerExecutorResponse>) -> Self {
        Self {
            responses,
            commands: Vec::new(),
            follow_up_commands: Vec::new(),
        }
    }
}

impl StudioGuiPlatformTimerExecutor for TestPlatformTimerExecutor {
    fn execute_platform_timer_command(
        &mut self,
        command: &crate::StudioGuiPlatformTimerCommand,
    ) -> RfResult<StudioGuiPlatformTimerExecutorResponse> {
        self.commands.push(command.clone());
        Ok(self.responses.remove(0))
    }

    fn execute_platform_timer_follow_up_command(
        &mut self,
        command: &StudioGuiPlatformTimerFollowUpCommand,
    ) -> RfResult<()> {
        self.follow_up_commands.push(command.clone());
        Ok(())
    }
}

fn lease_expiring_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
        ..StudioRuntimeConfig::default()
    }
}

fn synced_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
        ..StudioRuntimeConfig::default()
    }
}

#[test]
fn platform_host_records_run_command_and_latest_app_log_activity() {
    let mut host = StudioGuiPlatformHost::new(&synced_config()).expect("expected platform host");
    host.dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");

    host.dispatch_event(StudioGuiEvent::UiCommandRequested {
        command_id: "run_panel.run_manual".to_string(),
    })
    .expect("expected run command dispatch");

    assert!(
        host.gui_activity_lines()
            .iter()
            .any(|line| line.starts_with("command dispatch #")),
        "expected command dispatch audit line"
    );
    assert!(
        host.gui_activity_lines()
            .iter()
            .any(|line| line.starts_with("app log info: Solved document revision")),
        "expected latest app log audit line"
    );
}

#[test]
fn platform_host_reports_arm_request_when_native_timer_first_appears() {
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

    assert!(matches!(
        dispatched.native_timer_request.as_ref(),
        Some(StudioGuiPlatformTimerRequest::Arm { schedule })
            if schedule.window_id == Some(window_id)
    ));
    assert!(matches!(
        host.next_native_timer_schedule(),
        Some(schedule) if schedule.window_id == Some(window_id)
    ));
}

#[test]
fn platform_host_reports_rearm_after_due_timer_dispatch() {
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
    assert!(matches!(
        first.native_timer_request,
        Some(StudioGuiPlatformTimerRequest::Arm { .. })
    ));
    let due_at = host
        .next_native_timer_due_at()
        .expect("expected scheduled timer due at");

    let due_dispatches = host
        .dispatch_due_native_timer_events(due_at)
        .expect("expected due timer dispatches");

    assert!(!due_dispatches.is_empty());
    assert!(due_dispatches.iter().all(|dispatch| {
        match dispatch.native_timer_request.as_ref() {
            Some(StudioGuiPlatformTimerRequest::Rearm { previous, schedule }) => {
                previous.window_id == Some(window_id) && schedule.window_id == Some(window_id)
            }
            None => true,
            Some(StudioGuiPlatformTimerRequest::Arm { .. })
            | Some(StudioGuiPlatformTimerRequest::Clear { .. }) => false,
        }
    }));
}

#[test]
fn platform_host_surfaces_timer_start_failure_in_snapshot_and_window_model() {
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
    let failure = host.acknowledge_platform_timer_start_failed(
        &schedule,
        "simulated native timer creation failure",
    );
    match failure {
        StudioGuiPlatformTimerStartFailedOutcome::Applied(failure) => {
            assert_eq!(
                failure.status,
                crate::StudioGuiPlatformTimerStartFailureStatus::Applied
            );
        }
        other => panic!("expected applied platform timer failure, got {other:?}"),
    }

    let snapshot = host.snapshot();
    let platform_notice = snapshot
        .runtime
        .platform_notice
        .as_ref()
        .expect("expected platform notice in snapshot");
    assert_eq!(platform_notice.level, rf_ui::RunPanelNoticeLevel::Error);
    assert_eq!(platform_notice.title, "Platform timer unavailable");
    assert!(
        platform_notice
            .message
            .contains("simulated native timer creation failure")
    );
    assert_eq!(snapshot.runtime.platform_timer_lines.len(), 2);
    assert!(snapshot.runtime.platform_timer_lines[0].contains("Current schedule: window=Some"));
    assert!(
        snapshot
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("window opened"))
    );
    assert!(
        snapshot
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("request arm"))
    );
    let latest = snapshot
        .runtime
        .log_entries
        .last()
        .expect("expected platform failure log entry");
    assert_eq!(latest.level, rf_ui::AppLogLevel::Error);
    assert!(latest.message.contains("native timer start failed"));

    let window = snapshot.window_model_for_window(Some(window_id));
    let window_notice = window
        .runtime
        .platform_notice
        .as_ref()
        .expect("expected platform notice in window model");
    assert_eq!(window_notice.title, "Platform timer unavailable");
    assert_eq!(
        window.runtime.platform_timer_lines,
        snapshot.runtime.platform_timer_lines
    );
    assert_eq!(
        window.runtime.gui_activity_lines,
        snapshot.runtime.gui_activity_lines
    );
    let layout = window.layout();
    let runtime_panel = layout
        .panel(crate::StudioGuiWindowAreaId::Runtime)
        .expect("expected runtime panel");
    assert_eq!(runtime_panel.badge.as_deref(), Some("!"));
    assert!(runtime_panel.summary.contains("platform=Error"));
    assert!(runtime_panel.summary.contains("activity="));
    assert!(runtime_panel.summary.contains("Platform timer unavailable"));
    let latest_window_log = window
        .runtime
        .latest_log_entry
        .expect("expected latest window log entry");
    assert!(
        latest_window_log
            .message
            .contains("simulated native timer creation failure")
    );
}

#[test]
fn platform_host_dispatches_native_timer_callback_by_native_id() {
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
    let command = host.apply_platform_timer_request(dispatched.native_timer_request.as_ref());
    assert!(matches!(
        command,
        Some(crate::StudioGuiPlatformTimerCommand::Arm { .. })
    ));
    let _ = host.acknowledge_platform_timer_started(&schedule, 9001);

    let callback = host
        .dispatch_native_timer_elapsed_by_native_id(9001)
        .expect("expected native timer callback dispatch");

    match callback {
        StudioGuiPlatformNativeTimerCallbackOutcome::Dispatched(callback) => {
            assert!(matches!(
                callback.outcome,
                crate::StudioGuiDriverOutcome::HostCommand(
                    crate::StudioGuiHostCommandOutcome::LifecycleDispatched(_)
                )
            ));
        }
        other => panic!("expected dispatched native timer callback, got {other:?}"),
    }
}

#[test]
fn platform_host_ignores_unknown_native_timer_callback_by_native_id() {
    let mut host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");

    let callback = host
        .dispatch_native_timer_elapsed_by_native_id(9001)
        .expect("expected ignored callback outcome");

    assert_eq!(
        callback,
        StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
            native_timer_id: 9001,
        }
    );
}

#[test]
fn platform_host_ignores_stale_native_timer_callback_while_rearm_is_pending() {
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
    let first_schedule = match first.native_timer_request.as_ref() {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => schedule.clone(),
        other => panic!("expected arm timer request, got {other:?}"),
    };
    let _ = host.apply_platform_timer_request(first.native_timer_request.as_ref());
    let _ = host.acknowledge_platform_timer_started(&first_schedule, 9001);

    let callback = host
        .dispatch_native_timer_elapsed_by_native_id(9001)
        .expect("expected callback dispatch");
    let callback_dispatch = match callback {
        StudioGuiPlatformNativeTimerCallbackOutcome::Dispatched(dispatch) => dispatch,
        other => panic!("expected dispatched callback, got {other:?}"),
    };
    assert!(matches!(
        callback_dispatch.native_timer_request,
        Some(StudioGuiPlatformTimerRequest::Rearm { .. })
    ));

    let _ = host.apply_platform_timer_request(callback_dispatch.native_timer_request.as_ref());

    let stale = host
        .dispatch_native_timer_elapsed_by_native_id(9001)
        .expect("expected stale callback outcome");

    assert_eq!(
        stale,
        StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
            native_timer_id: 9001,
        }
    );
}

#[test]
fn platform_host_reports_cleanup_when_started_ack_arrives_without_pending_schedule() {
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
        "simulated native timer creation failure",
    );

    let started = host.acknowledge_platform_timer_started(&schedule, 9001);

    assert_eq!(
        started,
        StudioGuiPlatformTimerStartedOutcome::IgnoredMissingPendingSchedule {
            ack: crate::StudioGuiPlatformTimerStartAckResult {
                schedule,
                native_timer_id: 9001,
                status: crate::StudioGuiPlatformTimerStartAckStatus::MissingPendingSchedule,
            },
            clear_native_timer_id: 9001,
        }
    );
    assert_eq!(
        started.follow_up_command(),
        Some(StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
            native_timer_id: 9001,
        })
    );
}

#[test]
fn platform_host_reports_cleanup_when_started_ack_arrives_for_stale_schedule() {
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
    let first_schedule = match first.native_timer_request.as_ref() {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => schedule.clone(),
        other => panic!("expected arm timer request, got {other:?}"),
    };
    let _ = host.apply_platform_timer_request(first.native_timer_request.as_ref());
    let _ = host.acknowledge_platform_timer_start_failed(
        &first_schedule,
        "simulated native timer creation failure",
    );

    let second = host
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected second timer trigger");
    let _ = host.apply_platform_timer_request(second.native_timer_request.as_ref());

    let started = host.acknowledge_platform_timer_started(&first_schedule, 9001);

    assert_eq!(
        started,
        StudioGuiPlatformTimerStartedOutcome::IgnoredStalePendingSchedule {
            ack: crate::StudioGuiPlatformTimerStartAckResult {
                schedule: first_schedule,
                native_timer_id: 9001,
                status: crate::StudioGuiPlatformTimerStartAckStatus::StalePendingSchedule,
            },
            clear_native_timer_id: 9001,
        }
    );
    assert_eq!(
        started.follow_up_command(),
        Some(StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
            native_timer_id: 9001,
        })
    );
}

#[test]
fn platform_host_ignores_missing_start_failure_ack_after_pending_schedule_is_cleared() {
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
        "simulated native timer creation failure",
    );

    let failure = host.acknowledge_platform_timer_start_failed(&schedule, "duplicate failure ack");

    assert_eq!(
        failure,
        StudioGuiPlatformTimerStartFailedOutcome::IgnoredMissingPendingSchedule {
            failure: crate::StudioGuiPlatformTimerStartFailureResult {
                schedule,
                status: crate::StudioGuiPlatformTimerStartFailureStatus::MissingPendingSchedule,
            },
        }
    );
    assert_eq!(failure.follow_up_command(), None);
}

#[test]
fn platform_host_batches_started_feedbacks_and_executes_follow_up_commands() {
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
    let feedbacks = vec![
        StudioGuiPlatformTimerStartedFeedback {
            schedule: schedule.clone(),
            native_timer_id: 9001,
        },
        StudioGuiPlatformTimerStartedFeedback {
            schedule: schedule.clone(),
            native_timer_id: 9002,
        },
    ];
    let mut executor = TestPlatformTimerExecutor::default();

    let batch = host
        .acknowledge_platform_timer_started_feedbacks_and_execute_follow_up_commands(
            &feedbacks,
            &mut executor,
        )
        .expect("expected started feedback batch");

    assert_eq!(batch.len(), 2);
    assert!(!batch.is_empty());
    assert!(matches!(
        &batch.entries[0].outcome,
        StudioGuiPlatformTimerStartedOutcome::Applied(_)
    ));
    assert_eq!(batch.entries[0].follow_up_command.as_ref(), None);
    assert!(matches!(
        &batch.entries[1].outcome,
        StudioGuiPlatformTimerStartedOutcome::IgnoredMissingPendingSchedule { .. }
    ));
    assert_eq!(
        batch.entries[1].follow_up_command.as_ref(),
        Some(&StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
            native_timer_id: 9002,
        })
    );
    assert_eq!(
        batch.follow_up_commands(),
        vec![StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
            native_timer_id: 9002,
        }]
    );
    assert_eq!(batch.snapshot, host.snapshot());
    assert_eq!(
        batch.next_native_timer_due_at(),
        host.next_native_timer_due_at()
    );
    let window = batch.window_model_for_window(Some(window_id));
    assert_eq!(window.layout_state.scope.window_id, Some(window_id));
    assert_eq!(
        executor.follow_up_commands,
        vec![StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
            native_timer_id: 9002,
        }]
    );
    assert!(matches!(
        host.current_platform_timer_binding(),
        Some(binding) if binding.native_timer_id == 9001
    ));
}

#[test]
fn platform_host_batches_start_failed_feedbacks_and_refreshes_snapshot() {
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
    let feedbacks = vec![
        StudioGuiPlatformTimerStartFailedFeedback {
            schedule: schedule.clone(),
            detail: "simulated batch start failure".to_string(),
        },
        StudioGuiPlatformTimerStartFailedFeedback {
            schedule: schedule.clone(),
            detail: "duplicate batch start failure".to_string(),
        },
    ];

    let batch = host.acknowledge_platform_timer_start_failed_feedbacks(&feedbacks);

    assert_eq!(batch.len(), 2);
    assert!(!batch.is_empty());
    assert!(matches!(
        &batch.entries[0].outcome,
        StudioGuiPlatformTimerStartFailedOutcome::Applied(_)
    ));
    assert!(matches!(
        &batch.entries[1].outcome,
        StudioGuiPlatformTimerStartFailedOutcome::IgnoredMissingPendingSchedule { .. }
    ));
    assert_eq!(batch.entries[0].follow_up_command.as_ref(), None);
    assert_eq!(batch.entries[1].follow_up_command.as_ref(), None);
    assert_eq!(batch.snapshot, host.snapshot());
    assert_eq!(
        batch.next_native_timer_due_at(),
        host.next_native_timer_due_at()
    );
    let platform_notice = batch
        .snapshot
        .runtime
        .platform_notice
        .as_ref()
        .expect("expected platform notice in batch snapshot");
    assert!(
        platform_notice
            .message
            .contains("simulated batch start failure")
    );
    let window = batch.window_model_for_window(Some(window_id));
    let latest_log = window
        .runtime
        .latest_log_entry
        .as_ref()
        .expect("expected latest log entry in batch window");
    assert!(latest_log.message.contains("simulated batch start failure"));
}

#[test]
fn platform_host_clears_platform_notice_after_successful_timer_start() {
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
        "simulated native timer creation failure",
    );
    assert!(host.platform_notice().is_some());

    let next_dispatch = host
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: crate::StudioRuntimeTrigger::EntitlementSessionEvent(
                crate::StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer retrigger dispatch");
    let next_schedule = match next_dispatch.native_timer_request.as_ref() {
        Some(StudioGuiPlatformTimerRequest::Arm { schedule }) => schedule.clone(),
        Some(StudioGuiPlatformTimerRequest::Rearm { schedule, .. }) => schedule.clone(),
        other => panic!("expected arm or rearm timer request, got {other:?}"),
    };

    let _ = host.apply_platform_timer_request(next_dispatch.native_timer_request.as_ref());
    let started = host.acknowledge_platform_timer_started(&next_schedule, 9001);
    match started {
        StudioGuiPlatformTimerStartedOutcome::Applied(ref started) => {
            assert_eq!(
                started.status,
                crate::StudioGuiPlatformTimerStartAckStatus::Applied
            );
        }
        other => panic!("expected applied platform timer started outcome, got {other:?}"),
    }
    assert_eq!(started.follow_up_command(), None);
    assert!(host.platform_notice().is_none());
    assert!(host.snapshot().runtime.platform_notice.is_none());
}

#[test]
fn platform_host_executes_platform_timer_request_through_sync_executor() {
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
    let mut executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9001,
        },
    ]);

    let execution = host
        .execute_platform_timer_request(dispatched.native_timer_request.as_ref(), &mut executor)
        .expect("expected platform timer execution");

    match execution {
        StudioGuiPlatformTimerExecutionOutcome::Executed {
            command,
            executor_response,
            host_outcome,
            follow_up_command,
        } => {
            assert!(matches!(
                command,
                crate::StudioGuiPlatformTimerCommand::Arm { .. }
            ));
            assert_eq!(
                executor_response,
                StudioGuiPlatformTimerExecutorResponse::Started {
                    native_timer_id: 9001,
                }
            );
            match host_outcome {
                StudioGuiPlatformTimerHostOutcome::Started(
                    StudioGuiPlatformTimerStartedOutcome::Applied(ack),
                ) => {
                    assert_eq!(ack.native_timer_id, 9001);
                    assert_eq!(
                        ack.status,
                        crate::StudioGuiPlatformTimerStartAckStatus::Applied
                    );
                }
                other => panic!("expected started outcome, got {other:?}"),
            }
            assert_eq!(follow_up_command, None);
        }
        other => panic!("expected executed platform timer request, got {other:?}"),
    }
    assert_eq!(executor.follow_up_commands, Vec::new());
    assert!(matches!(
        host.current_platform_timer_binding(),
        Some(binding) if binding.native_timer_id == 9001
    ));
}

#[test]
fn platform_host_executes_platform_timer_request_failure_through_sync_executor() {
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
    let mut executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::StartFailed {
            detail: "simulated platform failure".to_string(),
        },
    ]);

    let execution = host
        .execute_platform_timer_request(dispatched.native_timer_request.as_ref(), &mut executor)
        .expect("expected platform timer execution");

    match execution {
        StudioGuiPlatformTimerExecutionOutcome::Executed {
            host_outcome,
            follow_up_command,
            ..
        } => {
            match host_outcome {
                StudioGuiPlatformTimerHostOutcome::StartFailed(
                    StudioGuiPlatformTimerStartFailedOutcome::Applied(failure),
                ) => {
                    assert_eq!(
                        failure.status,
                        crate::StudioGuiPlatformTimerStartFailureStatus::Applied
                    );
                }
                other => panic!("expected start failed outcome, got {other:?}"),
            }
            assert_eq!(follow_up_command, None);
        }
        other => panic!("expected executed platform timer request, got {other:?}"),
    }
    assert!(host.platform_notice().is_some());
    assert_eq!(executor.follow_up_commands, Vec::new());
}

#[test]
fn platform_host_executes_clear_request_through_sync_executor() {
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
    let mut start_executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9001,
        },
    ]);
    let _ = host
        .execute_platform_timer_request(
            dispatched.native_timer_request.as_ref(),
            &mut start_executor,
        )
        .expect("expected start execution");

    let current_schedule = host
        .next_native_timer_schedule()
        .cloned()
        .expect("expected current native timer schedule");
    let clear_request = StudioGuiPlatformTimerRequest::Clear {
        previous: current_schedule,
    };
    let mut clear_executor = TestPlatformTimerExecutor::with_responses(vec![
        StudioGuiPlatformTimerExecutorResponse::Cleared,
    ]);

    let execution = host
        .execute_platform_timer_request(Some(&clear_request), &mut clear_executor)
        .expect("expected clear execution");

    match execution {
        StudioGuiPlatformTimerExecutionOutcome::Executed {
            command,
            host_outcome,
            ..
        } => {
            assert!(matches!(
                command,
                crate::StudioGuiPlatformTimerCommand::Clear { previous: Some(_) }
            ));
            assert_eq!(host_outcome, StudioGuiPlatformTimerHostOutcome::Cleared);
        }
        other => panic!("expected executed clear request, got {other:?}"),
    }
    assert_eq!(clear_executor.follow_up_commands, Vec::new());
}

#[test]
fn platform_host_dispatch_event_and_executes_platform_timer_through_sync_executor() {
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
        StudioGuiPlatformTimerExecutorResponse::Started {
            native_timer_id: 9001,
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

    assert!(matches!(
        executed.dispatch.native_timer_request,
        Some(StudioGuiPlatformTimerRequest::Arm { .. })
    ));
    match executed.timer_execution {
        StudioGuiPlatformTimerExecutionOutcome::Executed {
            host_outcome,
            follow_up_command,
            ..
        } => {
            assert!(matches!(
                host_outcome,
                StudioGuiPlatformTimerHostOutcome::Started(
                    StudioGuiPlatformTimerStartedOutcome::Applied(_)
                )
            ));
            assert_eq!(follow_up_command, None);
        }
        other => panic!("expected executed platform timer outcome, got {other:?}"),
    }
    assert!(matches!(
        host.current_platform_timer_binding(),
        Some(binding) if binding.native_timer_id == 9001
    ));
}

mod combined_execution;
