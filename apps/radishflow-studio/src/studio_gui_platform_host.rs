use std::time::SystemTime;

use rf_types::RfResult;
use rf_ui::{AppLogEntry, AppLogLevel, RunPanelNotice, RunPanelNoticeLevel};

use crate::{
    StudioAppHostState, StudioAppHostUiCommandModel, StudioGuiCanvasState,
    StudioGuiCommandRegistry, StudioGuiDriver, StudioGuiDriverDispatch, StudioGuiDriverOutcome,
    StudioGuiEvent, StudioGuiNativeTimerSchedule, StudioGuiPlatformNativeTimerId,
    StudioGuiPlatformTimerBinding, StudioGuiPlatformTimerCallbackResolution,
    StudioGuiPlatformTimerCommand, StudioGuiPlatformTimerDriverState,
    StudioGuiPlatformTimerStartAckResult, StudioGuiPlatformTimerStartFailureResult,
    StudioGuiSnapshot, StudioGuiWindowModel, StudioWindowHostId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerRequest {
    Arm {
        schedule: StudioGuiNativeTimerSchedule,
    },
    Rearm {
        previous: StudioGuiNativeTimerSchedule,
        schedule: StudioGuiNativeTimerSchedule,
    },
    Clear {
        previous: StudioGuiNativeTimerSchedule,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformDispatch {
    pub event: StudioGuiEvent,
    pub outcome: StudioGuiDriverOutcome,
    pub snapshot: StudioGuiSnapshot,
    pub window: StudioGuiWindowModel,
    pub state: StudioAppHostState,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub command_registry: StudioGuiCommandRegistry,
    pub canvas: StudioGuiCanvasState,
    pub native_timer_request: Option<StudioGuiPlatformTimerRequest>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StudioGuiPlatformNativeTimerCallbackOutcome {
    Dispatched(StudioGuiPlatformDispatch),
    IgnoredUnknownNativeTimer {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
    IgnoredStaleNativeTimer {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerFollowUpCommand {
    ClearNativeTimer {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerExecutorResponse {
    Started {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
    StartFailed {
        detail: String,
    },
    Cleared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerStartedOutcome {
    Applied(StudioGuiPlatformTimerStartAckResult),
    IgnoredMissingPendingSchedule {
        ack: StudioGuiPlatformTimerStartAckResult,
        clear_native_timer_id: StudioGuiPlatformNativeTimerId,
    },
    IgnoredStalePendingSchedule {
        ack: StudioGuiPlatformTimerStartAckResult,
        clear_native_timer_id: StudioGuiPlatformNativeTimerId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerStartFailedOutcome {
    Applied(StudioGuiPlatformTimerStartFailureResult),
    IgnoredMissingPendingSchedule {
        failure: StudioGuiPlatformTimerStartFailureResult,
    },
    IgnoredStalePendingSchedule {
        failure: StudioGuiPlatformTimerStartFailureResult,
    },
}

impl StudioGuiPlatformTimerStartedOutcome {
    pub fn follow_up_command(&self) -> Option<StudioGuiPlatformTimerFollowUpCommand> {
        match self {
            Self::Applied(_) => None,
            Self::IgnoredMissingPendingSchedule {
                clear_native_timer_id,
                ..
            }
            | Self::IgnoredStalePendingSchedule {
                clear_native_timer_id,
                ..
            } => Some(StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer {
                native_timer_id: *clear_native_timer_id,
            }),
        }
    }
}

impl StudioGuiPlatformTimerStartFailedOutcome {
    pub fn follow_up_command(&self) -> Option<StudioGuiPlatformTimerFollowUpCommand> {
        match self {
            Self::Applied(_)
            | Self::IgnoredMissingPendingSchedule { .. }
            | Self::IgnoredStalePendingSchedule { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerExecutionOutcome {
    NoCommand,
    Executed {
        command: StudioGuiPlatformTimerCommand,
        executor_response: StudioGuiPlatformTimerExecutorResponse,
        host_outcome: StudioGuiPlatformTimerHostOutcome,
        follow_up_command: Option<StudioGuiPlatformTimerFollowUpCommand>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformTimerHostOutcome {
    Started(StudioGuiPlatformTimerStartedOutcome),
    StartFailed(StudioGuiPlatformTimerStartFailedOutcome),
    Cleared,
}

pub trait StudioGuiPlatformTimerExecutor {
    fn execute_platform_timer_command(
        &mut self,
        command: &StudioGuiPlatformTimerCommand,
    ) -> RfResult<StudioGuiPlatformTimerExecutorResponse>;

    fn execute_platform_timer_follow_up_command(
        &mut self,
        command: &StudioGuiPlatformTimerFollowUpCommand,
    ) -> RfResult<()>;
}

pub struct StudioGuiPlatformHost {
    driver: StudioGuiDriver,
    platform_timer_driver: StudioGuiPlatformTimerDriverState,
    platform_notice: Option<RunPanelNotice>,
    platform_log_entries: Vec<AppLogEntry>,
    current_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformHost {
    pub fn new(config: &crate::StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            driver: StudioGuiDriver::new(config)?,
            platform_timer_driver: StudioGuiPlatformTimerDriverState::default(),
            platform_notice: None,
            platform_log_entries: Vec::new(),
            current_schedule: None,
        })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.driver.state()
    }

    pub fn snapshot(&self) -> StudioGuiSnapshot {
        self.enrich_snapshot(self.driver.snapshot())
    }

    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.current_schedule
            .as_ref()
            .map(|schedule| schedule.slot.timer.due_at)
    }

    pub fn next_native_timer_schedule(&self) -> Option<&StudioGuiNativeTimerSchedule> {
        self.current_schedule.as_ref()
    }

    pub fn current_platform_timer_binding(&self) -> Option<&StudioGuiPlatformTimerBinding> {
        self.platform_timer_driver.current_binding()
    }

    pub fn platform_log_entries(&self) -> &[AppLogEntry] {
        &self.platform_log_entries
    }

    pub fn platform_notice(&self) -> Option<&RunPanelNotice> {
        self.platform_notice.as_ref()
    }

    pub fn apply_platform_timer_request(
        &mut self,
        request: Option<&StudioGuiPlatformTimerRequest>,
    ) -> Option<StudioGuiPlatformTimerCommand> {
        self.platform_timer_driver.apply_request(request)
    }

    pub fn execute_platform_timer_request(
        &mut self,
        request: Option<&StudioGuiPlatformTimerRequest>,
        executor: &mut impl StudioGuiPlatformTimerExecutor,
    ) -> RfResult<StudioGuiPlatformTimerExecutionOutcome> {
        let Some(command) = self.apply_platform_timer_request(request) else {
            return Ok(StudioGuiPlatformTimerExecutionOutcome::NoCommand);
        };

        let executor_response = executor.execute_platform_timer_command(&command)?;
        let host_outcome = match (&command, &executor_response) {
            (
                StudioGuiPlatformTimerCommand::Arm { schedule }
                | StudioGuiPlatformTimerCommand::Rearm { schedule, .. },
                StudioGuiPlatformTimerExecutorResponse::Started { native_timer_id },
            ) => StudioGuiPlatformTimerHostOutcome::Started(
                self.acknowledge_platform_timer_started(schedule, *native_timer_id),
            ),
            (
                StudioGuiPlatformTimerCommand::Arm { schedule }
                | StudioGuiPlatformTimerCommand::Rearm { schedule, .. },
                StudioGuiPlatformTimerExecutorResponse::StartFailed { detail },
            ) => StudioGuiPlatformTimerHostOutcome::StartFailed(
                self.acknowledge_platform_timer_start_failed(schedule, detail),
            ),
            (
                StudioGuiPlatformTimerCommand::Clear { .. },
                StudioGuiPlatformTimerExecutorResponse::Cleared,
            ) => StudioGuiPlatformTimerHostOutcome::Cleared,
            (
                StudioGuiPlatformTimerCommand::Arm { .. }
                | StudioGuiPlatformTimerCommand::Rearm { .. },
                StudioGuiPlatformTimerExecutorResponse::Cleared,
            ) => {
                return Err(rf_types::RfError::invalid_input(
                    "platform timer executor returned `Cleared` for an arm/rearm command"
                        .to_string(),
                ));
            }
            (
                StudioGuiPlatformTimerCommand::Clear { .. },
                StudioGuiPlatformTimerExecutorResponse::Started { .. }
                | StudioGuiPlatformTimerExecutorResponse::StartFailed { .. },
            ) => {
                return Err(rf_types::RfError::invalid_input(
                    "platform timer executor returned a start response for a clear command"
                        .to_string(),
                ));
            }
        };

        let follow_up_command = match &host_outcome {
            StudioGuiPlatformTimerHostOutcome::Started(outcome) => outcome.follow_up_command(),
            StudioGuiPlatformTimerHostOutcome::StartFailed(outcome) => outcome.follow_up_command(),
            StudioGuiPlatformTimerHostOutcome::Cleared => None,
        };
        if let Some(command) = follow_up_command.as_ref() {
            executor.execute_platform_timer_follow_up_command(command)?;
        }

        Ok(StudioGuiPlatformTimerExecutionOutcome::Executed {
            command,
            executor_response,
            host_outcome,
            follow_up_command,
        })
    }

    pub fn acknowledge_platform_timer_started(
        &mut self,
        schedule: &StudioGuiNativeTimerSchedule,
        native_timer_id: StudioGuiPlatformNativeTimerId,
    ) -> StudioGuiPlatformTimerStartedOutcome {
        let result = self
            .platform_timer_driver
            .acknowledge_timer_started(schedule, native_timer_id);
        match result.status {
            crate::StudioGuiPlatformTimerStartAckStatus::Applied => {
                self.platform_notice = None;
                StudioGuiPlatformTimerStartedOutcome::Applied(result)
            }
            crate::StudioGuiPlatformTimerStartAckStatus::MissingPendingSchedule => {
                StudioGuiPlatformTimerStartedOutcome::IgnoredMissingPendingSchedule {
                    clear_native_timer_id: native_timer_id,
                    ack: result,
                }
            }
            crate::StudioGuiPlatformTimerStartAckStatus::StalePendingSchedule => {
                StudioGuiPlatformTimerStartedOutcome::IgnoredStalePendingSchedule {
                    clear_native_timer_id: native_timer_id,
                    ack: result,
                }
            }
        }
    }

    pub fn acknowledge_platform_timer_start_failed(
        &mut self,
        schedule: &StudioGuiNativeTimerSchedule,
        detail: impl AsRef<str>,
    ) -> StudioGuiPlatformTimerStartFailedOutcome {
        let result = self
            .platform_timer_driver
            .acknowledge_timer_start_failed(schedule);
        match result.status {
            crate::StudioGuiPlatformTimerStartFailureStatus::Applied => {
                self.platform_notice = Some(RunPanelNotice::new(
                    RunPanelNoticeLevel::Error,
                    "Platform timer unavailable",
                    format!(
                        "Failed to start native timer for window={:?}, handle={}, due_at={:?}. {}",
                        schedule.window_id,
                        schedule.handle_id,
                        schedule.slot.timer.due_at,
                        detail.as_ref()
                    ),
                ));
                self.platform_log_entries.push(AppLogEntry {
                    level: AppLogLevel::Error,
                    message: format!(
                        "Platform native timer start failed for window={:?} handle={} due_at={:?}: {}",
                        schedule.window_id,
                        schedule.handle_id,
                        schedule.slot.timer.due_at,
                        detail.as_ref()
                    ),
                });
                StudioGuiPlatformTimerStartFailedOutcome::Applied(result)
            }
            crate::StudioGuiPlatformTimerStartFailureStatus::MissingPendingSchedule => {
                StudioGuiPlatformTimerStartFailedOutcome::IgnoredMissingPendingSchedule {
                    failure: result,
                }
            }
            crate::StudioGuiPlatformTimerStartFailureStatus::StalePendingSchedule => {
                StudioGuiPlatformTimerStartFailedOutcome::IgnoredStalePendingSchedule {
                    failure: result,
                }
            }
        }
    }

    pub fn callback_schedule_for_native_timer(
        &self,
        native_timer_id: StudioGuiPlatformNativeTimerId,
    ) -> Option<&StudioGuiNativeTimerSchedule> {
        self.platform_timer_driver
            .callback_schedule(native_timer_id)
    }

    pub fn dispatch_native_timer_elapsed_by_native_id(
        &mut self,
        native_timer_id: StudioGuiPlatformNativeTimerId,
    ) -> RfResult<StudioGuiPlatformNativeTimerCallbackOutcome> {
        match self.platform_timer_driver.resolve_callback(native_timer_id) {
            StudioGuiPlatformTimerCallbackResolution::Dispatch { schedule } => self
                .dispatch_native_timer_elapsed(schedule.window_id, schedule.handle_id)
                .map(StudioGuiPlatformNativeTimerCallbackOutcome::Dispatched),
            StudioGuiPlatformTimerCallbackResolution::IgnoredUnknownNativeTimer {
                native_timer_id,
            } => Ok(
                StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
                    native_timer_id,
                },
            ),
            StudioGuiPlatformTimerCallbackResolution::IgnoredStaleNativeTimer {
                native_timer_id,
            } => Ok(
                StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
                    native_timer_id,
                },
            ),
        }
    }

    pub fn dispatch_event(&mut self, event: StudioGuiEvent) -> RfResult<StudioGuiPlatformDispatch> {
        let previous_schedule = self.current_schedule.clone();
        let dispatch = self.driver.dispatch_event(event)?;
        let next_schedule = self.driver.native_timer_runtime().next_schedule();
        self.current_schedule = next_schedule.clone();
        Ok(platform_dispatch_from_driver(
            self,
            dispatch,
            plan_platform_timer_request(previous_schedule, next_schedule),
        ))
    }

    pub fn dispatch_native_timer_elapsed(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        handle_id: crate::StudioWindowNativeTimerHandleId,
    ) -> RfResult<StudioGuiPlatformDispatch> {
        self.dispatch_event(StudioGuiEvent::NativeTimerElapsed {
            window_id,
            handle_id,
        })
    }

    pub fn dispatch_due_native_timer_events(
        &mut self,
        now: SystemTime,
    ) -> RfResult<Vec<StudioGuiPlatformDispatch>> {
        let due_dispatches = self.driver.drain_due_native_timer_events(now)?;
        let mut platform_dispatches = Vec::with_capacity(due_dispatches.len());
        for dispatch in due_dispatches {
            let previous_schedule = self.current_schedule.clone();
            let next_schedule = self.driver.native_timer_runtime().next_schedule();
            self.current_schedule = next_schedule.clone();
            platform_dispatches.push(platform_dispatch_from_driver(
                self,
                dispatch,
                plan_platform_timer_request(previous_schedule, next_schedule),
            ));
        }
        Ok(platform_dispatches)
    }
}

fn platform_dispatch_from_driver(
    host: &StudioGuiPlatformHost,
    dispatch: StudioGuiDriverDispatch,
    native_timer_request: Option<StudioGuiPlatformTimerRequest>,
) -> StudioGuiPlatformDispatch {
    let snapshot = host.enrich_snapshot(dispatch.snapshot);
    let window = host.enrich_window(dispatch.window);
    StudioGuiPlatformDispatch {
        event: dispatch.event,
        outcome: dispatch.outcome,
        snapshot,
        window,
        state: dispatch.state,
        ui_commands: dispatch.ui_commands,
        command_registry: dispatch.command_registry,
        canvas: dispatch.canvas,
        native_timer_request,
    }
}

impl StudioGuiPlatformHost {
    fn enrich_snapshot(&self, mut snapshot: StudioGuiSnapshot) -> StudioGuiSnapshot {
        snapshot.runtime.platform_notice = self.platform_notice.clone();
        snapshot
            .runtime
            .log_entries
            .extend(self.platform_log_entries.iter().cloned());
        snapshot
    }

    fn enrich_window(&self, mut window: StudioGuiWindowModel) -> StudioGuiWindowModel {
        window.runtime.platform_notice = self.platform_notice.clone();
        window
            .runtime
            .log_entries
            .extend(self.platform_log_entries.iter().cloned());
        window.runtime.latest_log_entry = window.runtime.log_entries.last().cloned();
        window
    }
}

fn plan_platform_timer_request(
    previous_schedule: Option<StudioGuiNativeTimerSchedule>,
    next_schedule: Option<StudioGuiNativeTimerSchedule>,
) -> Option<StudioGuiPlatformTimerRequest> {
    match (previous_schedule, next_schedule) {
        (None, Some(schedule)) => Some(StudioGuiPlatformTimerRequest::Arm { schedule }),
        (Some(previous), Some(schedule)) if previous != schedule => {
            Some(StudioGuiPlatformTimerRequest::Rearm { previous, schedule })
        }
        (Some(previous), None) => Some(StudioGuiPlatformTimerRequest::Clear { previous }),
        (Some(_), Some(_)) | (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use rf_types::RfResult;

    use crate::{
        StudioGuiEvent, StudioGuiPlatformHost, StudioGuiPlatformNativeTimerCallbackOutcome,
        StudioGuiPlatformTimerExecutionOutcome, StudioGuiPlatformTimerExecutor,
        StudioGuiPlatformTimerExecutorResponse, StudioGuiPlatformTimerFollowUpCommand,
        StudioGuiPlatformTimerHostOutcome, StudioGuiPlatformTimerRequest,
        StudioGuiPlatformTimerStartFailedOutcome, StudioGuiPlatformTimerStartedOutcome,
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
        let layout = window.layout();
        let runtime_panel = layout
            .panel(crate::StudioGuiWindowAreaId::Runtime)
            .expect("expected runtime panel");
        assert_eq!(runtime_panel.badge.as_deref(), Some("!"));
        assert!(runtime_panel.summary.contains("platform=Error"));
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

        let failure =
            host.acknowledge_platform_timer_start_failed(&schedule, "duplicate failure ack");

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
                assert!(matches!(command, crate::StudioGuiPlatformTimerCommand::Arm { .. }));
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
            .execute_platform_timer_request(dispatched.native_timer_request.as_ref(), &mut start_executor)
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
}
