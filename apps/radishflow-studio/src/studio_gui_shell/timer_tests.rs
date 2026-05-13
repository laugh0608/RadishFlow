use super::*;
use radishflow_studio::{
    StudioGuiDriverOutcome, StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
    StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger,
};

fn lease_expiring_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Auto,
        entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
        ..StudioRuntimeConfig::default()
    }
}

fn synced_skip_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
        ..StudioRuntimeConfig::default()
    }
}

#[test]
fn shell_runtime_config_skips_startup_entitlement_preflight() {
    let config = studio_shell_runtime_config(None);

    assert_eq!(
        config.entitlement_preflight,
        StudioRuntimeEntitlementPreflight::Skip
    );
    assert_eq!(
        config.project_path,
        StudioRuntimeConfig::default().project_path
    );
}

#[test]
fn startup_uses_default_hidden_commands_panel_without_layout_dispatch() {
    let preferences_path = std::env::temp_dir()
        .join("radishflow-studio-shell-startup-hidden-commands.preferences.json");
    let app = ReadyAppState::from_config(&synced_skip_config(), preferences_path)
        .expect("expected ready app");

    let snapshot = app.platform_host.snapshot();
    let window = snapshot.window_model();
    assert_eq!(
        window
            .layout_state
            .panel(StudioGuiWindowAreaId::Commands)
            .map(|panel| panel.visible),
        Some(false)
    );
    assert!(
        app.platform_host
            .gui_activity_lines()
            .iter()
            .all(|line| !line.contains("layout SetPanelVisibility")),
        "startup should not dispatch layout mutations for default commands visibility"
    );
}

#[test]
fn viewport_focus_tracking_does_not_dispatch_foreground_entitlement_tick() {
    let preferences_path = std::env::temp_dir()
        .join("radishflow-studio-shell-viewport-focus-no-foreground-dispatch.preferences.json");
    let mut app = ReadyAppState::from_config(&lease_expiring_config(), preferences_path)
        .expect("expected ready app");
    let previous_activity_count = app.platform_host.gui_activity_lines().len();
    app.last_viewport_focused = Some(false);

    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput {
        focused: true,
        ..Default::default()
    });
    app.sync_viewport_lifecycle(&ctx);
    let _ = ctx.end_pass();

    assert_eq!(app.last_viewport_focused, Some(true));
    assert_eq!(
        app.platform_host.gui_activity_lines().len(),
        previous_activity_count
    );
}

#[test]
fn egui_platform_timer_executor_allocates_and_clears_native_ids() {
    let mut executor = EguiPlatformTimerExecutor::default();
    let arm_schedule = radishflow_studio::StudioGuiNativeTimerSchedule {
        window_id: Some(7),
        handle_id: 41,
        slot: radishflow_studio::StudioRuntimeTimerHandleSlot {
            effect_id: 1001,
            timer: radishflow_studio::EntitlementSessionTimerArm {
                event: radishflow_studio::EntitlementSessionLifecycleEvent::TimerElapsed,
                due_at: SystemTime::UNIX_EPOCH + Duration::from_secs(60),
                delay: Duration::from_secs(60),
                reason: radishflow_studio::EntitlementSessionTimerReason::ScheduledCheck,
            },
        },
    };

    let started = executor
        .execute_platform_timer_command(&StudioGuiPlatformTimerCommand::Arm {
            schedule: arm_schedule.clone(),
        })
        .expect("expected arm response");
    assert_eq!(
        started,
        StudioGuiPlatformTimerExecutorResponse::Started { native_timer_id: 1 }
    );
    assert_eq!(
        executor.next_due_at(),
        Some(SystemTime::UNIX_EPOCH + Duration::from_secs(60))
    );
    assert!(executor.active_native_timers.contains_key(&1));

    let cleared = executor
        .execute_platform_timer_command(&StudioGuiPlatformTimerCommand::Clear {
            previous: Some(radishflow_studio::StudioGuiPlatformTimerBinding {
                schedule: arm_schedule,
                native_timer_id: 1,
            }),
        })
        .expect("expected clear response");
    assert_eq!(cleared, StudioGuiPlatformTimerExecutorResponse::Cleared);
    assert!(!executor.active_native_timers.contains_key(&1));
    assert_eq!(executor.next_due_at(), None);
}

#[test]
fn egui_platform_timer_executor_drains_due_native_timer_ids_from_native_schedule() {
    let mut executor = EguiPlatformTimerExecutor::default();
    let arm_schedule = radishflow_studio::StudioGuiNativeTimerSchedule {
        window_id: Some(3),
        handle_id: 9,
        slot: radishflow_studio::StudioRuntimeTimerHandleSlot {
            effect_id: 2002,
            timer: radishflow_studio::EntitlementSessionTimerArm {
                event: radishflow_studio::EntitlementSessionLifecycleEvent::TimerElapsed,
                due_at: SystemTime::UNIX_EPOCH + Duration::from_secs(30),
                delay: Duration::from_secs(30),
                reason: radishflow_studio::EntitlementSessionTimerReason::ScheduledCheck,
            },
        },
    };
    executor
        .execute_platform_timer_command(&StudioGuiPlatformTimerCommand::Arm {
            schedule: arm_schedule,
        })
        .expect("expected arm response");

    let due_native_timer_ids =
        executor.drain_due_native_timer_ids(SystemTime::UNIX_EPOCH + Duration::from_secs(31));

    assert_eq!(due_native_timer_ids, vec![1]);
    assert_eq!(executor.next_due_at(), None);
}

#[test]
fn drain_due_platform_timer_callbacks_dispatches_due_binding_and_rearms() {
    let mut platform_host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let mut executor = EguiPlatformTimerExecutor::default();
    let opened = platform_host
        .dispatch_event_and_execute_platform_timer(
            StudioGuiEvent::OpenWindowRequested,
            &mut executor,
        )
        .expect("expected opened window");
    let window_id = match opened.dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(
            radishflow_studio::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let triggered = platform_host
        .dispatch_event_and_execute_platform_timer(
            StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            },
            &mut executor,
        )
        .expect("expected timer trigger dispatch");
    assert!(triggered.dispatch.native_timer_request.is_some());

    let due_at = platform_host
        .next_native_timer_due_at()
        .expect("expected current native timer due time");
    let callbacks = drain_due_platform_timer_callbacks(&mut platform_host, &mut executor, due_at)
        .expect("expected due timer callbacks");

    assert_eq!(callbacks.callbacks.len(), 1);
    assert_eq!(
        callbacks.next_native_timer_due_at(),
        platform_host.next_native_timer_due_at()
    );
    match &callbacks.callbacks[0] {
        StudioGuiPlatformExecutedNativeTimerCallbackOutcome::Dispatched(executed) => {
            assert!(matches!(
                executed.dispatch.outcome,
                StudioGuiDriverOutcome::HostCommand(
                    radishflow_studio::StudioGuiHostCommandOutcome::LifecycleDispatched(_)
                )
            ));
            assert!(executed.dispatch.native_timer_request.is_some());
        }
        other => panic!("expected dispatched callback outcome, got {other:?}"),
    }
    assert!(platform_host.current_platform_timer_binding().is_some());
    assert!(platform_host.next_native_timer_due_at().is_some());
}

#[test]
fn drain_due_platform_timer_callbacks_ignores_unknown_native_timer_ids() {
    let mut platform_host =
        StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
    let mut executor = EguiPlatformTimerExecutor::default();
    let opened = platform_host
        .dispatch_event_and_execute_platform_timer(
            StudioGuiEvent::OpenWindowRequested,
            &mut executor,
        )
        .expect("expected opened window");
    let window_id = match opened.dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(
            radishflow_studio::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let _ = platform_host
        .dispatch_event_and_execute_platform_timer(
            StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            },
            &mut executor,
        )
        .expect("expected timer trigger dispatch");

    let ignored = platform_host
        .dispatch_native_timer_elapsed_by_native_id_and_execute_platform_timer(9999, &mut executor)
        .expect("expected ignored callback");

    assert!(matches!(
        ignored,
        StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
            native_timer_id: 9999
        } | StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
            native_timer_id: 9999
        }
    ));
}
