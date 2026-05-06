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
pub struct StudioGuiPlatformExecutedDispatch {
    pub dispatch: StudioGuiPlatformDispatch,
    pub timer_execution: StudioGuiPlatformTimerExecutionOutcome,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformExecutedDueTimerDrain {
    pub now: SystemTime,
    pub dispatches: Vec<StudioGuiPlatformExecutedDispatch>,
    pub snapshot: StudioGuiSnapshot,
    pub next_native_timer_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformExecutedDueTimerDrain {
    pub fn is_empty(&self) -> bool {
        self.dispatches.is_empty()
    }

    pub fn len(&self) -> usize {
        self.dispatches.len()
    }

    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.next_native_timer_schedule
            .as_ref()
            .map(|schedule| schedule.slot.timer.due_at)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.snapshot.window_model_for_window(window_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformNativeTimerCallbackBatch {
    pub callbacks: Vec<StudioGuiPlatformNativeTimerCallbackOutcome>,
    pub snapshot: StudioGuiSnapshot,
    pub next_native_timer_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformNativeTimerCallbackBatch {
    pub fn is_empty(&self) -> bool {
        self.callbacks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.callbacks.len()
    }

    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.next_native_timer_schedule
            .as_ref()
            .map(|schedule| schedule.slot.timer.due_at)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.snapshot.window_model_for_window(window_id)
    }

    pub fn native_timer_requests(&self) -> Vec<StudioGuiPlatformTimerRequest> {
        self.callbacks
            .iter()
            .filter_map(|callback| match callback {
                StudioGuiPlatformNativeTimerCallbackOutcome::Dispatched(dispatch) => {
                    dispatch.native_timer_request.clone()
                }
                StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
                    ..
                }
                | StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredStaleNativeTimer { .. } => {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StudioGuiPlatformNativeTimerCallbackOutcome {
    Dispatched(StudioGuiPlatformDispatch),
    IgnoredUnknownNativeTimer {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
    IgnoredStaleNativeTimer {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StudioGuiPlatformExecutedNativeTimerCallbackOutcome {
    Dispatched(StudioGuiPlatformExecutedDispatch),
    IgnoredUnknownNativeTimer {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
    IgnoredStaleNativeTimer {
        native_timer_id: StudioGuiPlatformNativeTimerId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformDueTimerDrain {
    pub now: SystemTime,
    pub dispatches: Vec<StudioGuiPlatformDispatch>,
    pub snapshot: StudioGuiSnapshot,
    pub next_native_timer_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformDueTimerDrain {
    pub fn is_empty(&self) -> bool {
        self.dispatches.is_empty()
    }

    pub fn len(&self) -> usize {
        self.dispatches.len()
    }

    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.next_native_timer_schedule
            .as_ref()
            .map(|schedule| schedule.slot.timer.due_at)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.snapshot.window_model_for_window(window_id)
    }

    pub fn native_timer_requests(&self) -> Vec<StudioGuiPlatformTimerRequest> {
        self.dispatches
            .iter()
            .filter_map(|dispatch| dispatch.native_timer_request.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformExecutedNativeTimerCallbackBatch {
    pub callbacks: Vec<StudioGuiPlatformExecutedNativeTimerCallbackOutcome>,
    pub snapshot: StudioGuiSnapshot,
    pub next_native_timer_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformExecutedNativeTimerCallbackBatch {
    pub fn is_empty(&self) -> bool {
        self.callbacks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.callbacks.len()
    }

    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.next_native_timer_schedule
            .as_ref()
            .map(|schedule| schedule.slot.timer.due_at)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.snapshot.window_model_for_window(window_id)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StudioGuiPlatformAsyncRoundInput {
    pub due_at: Option<SystemTime>,
    pub native_timer_ids: Vec<StudioGuiPlatformNativeTimerId>,
    pub started_feedbacks: Vec<StudioGuiPlatformTimerStartedFeedback>,
    pub start_failed_feedbacks: Vec<StudioGuiPlatformTimerStartFailedFeedback>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiPlatformAsyncRoundAction {
    FollowUpCommand(StudioGuiPlatformTimerFollowUpCommand),
    TimerRequest(StudioGuiPlatformTimerRequest),
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum StudioGuiPlatformExecutedAsyncRoundAction {
    FollowUpCommand(StudioGuiPlatformTimerFollowUpCommand),
    TimerRequest {
        request: StudioGuiPlatformTimerRequest,
        execution: StudioGuiPlatformTimerExecutionOutcome,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformAsyncRound {
    pub input: StudioGuiPlatformAsyncRoundInput,
    pub started_feedback_batch: StudioGuiPlatformTimerStartedFeedbackBatch,
    pub start_failed_feedback_batch: StudioGuiPlatformTimerStartFailedFeedbackBatch,
    pub native_timer_callback_batch: StudioGuiPlatformNativeTimerCallbackBatch,
    pub due_timer_drain: Option<StudioGuiPlatformDueTimerDrain>,
    pub snapshot: StudioGuiSnapshot,
    pub next_native_timer_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformAsyncRound {
    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.next_native_timer_schedule
            .as_ref()
            .map(|schedule| schedule.slot.timer.due_at)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.snapshot.window_model_for_window(window_id)
    }

    pub fn native_timer_requests(&self) -> Vec<StudioGuiPlatformTimerRequest> {
        let mut requests = self.native_timer_callback_batch.native_timer_requests();
        if let Some(due_timer_drain) = self.due_timer_drain.as_ref() {
            requests.extend(due_timer_drain.native_timer_requests());
        }
        requests
    }

    pub fn follow_up_commands(&self) -> Vec<StudioGuiPlatformTimerFollowUpCommand> {
        self.started_feedback_batch.follow_up_commands()
    }

    pub fn actions(&self) -> Vec<StudioGuiPlatformAsyncRoundAction> {
        let mut actions = self
            .follow_up_commands()
            .into_iter()
            .map(StudioGuiPlatformAsyncRoundAction::FollowUpCommand)
            .collect::<Vec<_>>();
        actions.extend(
            self.native_timer_requests()
                .into_iter()
                .map(StudioGuiPlatformAsyncRoundAction::TimerRequest),
        );
        actions
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformExecutedAsyncRound {
    pub round: StudioGuiPlatformAsyncRound,
    pub actions: Vec<StudioGuiPlatformExecutedAsyncRoundAction>,
    pub snapshot: StudioGuiSnapshot,
    pub next_native_timer_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformExecutedAsyncRound {
    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.next_native_timer_schedule
            .as_ref()
            .map(|schedule| schedule.slot.timer.due_at)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.snapshot.window_model_for_window(window_id)
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiPlatformTimerStartedFeedback {
    pub schedule: StudioGuiNativeTimerSchedule,
    pub native_timer_id: StudioGuiPlatformNativeTimerId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiPlatformTimerStartedFeedbackEntry {
    pub feedback: StudioGuiPlatformTimerStartedFeedback,
    pub outcome: StudioGuiPlatformTimerStartedOutcome,
    pub follow_up_command: Option<StudioGuiPlatformTimerFollowUpCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformTimerStartedFeedbackBatch {
    pub entries: Vec<StudioGuiPlatformTimerStartedFeedbackEntry>,
    pub snapshot: StudioGuiSnapshot,
    pub next_native_timer_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformTimerStartedFeedbackBatch {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.next_native_timer_schedule
            .as_ref()
            .map(|schedule| schedule.slot.timer.due_at)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.snapshot.window_model_for_window(window_id)
    }

    pub fn follow_up_commands(&self) -> Vec<StudioGuiPlatformTimerFollowUpCommand> {
        self.entries
            .iter()
            .filter_map(|entry| entry.follow_up_command.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiPlatformTimerStartFailedFeedback {
    pub schedule: StudioGuiNativeTimerSchedule,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiPlatformTimerStartFailedFeedbackEntry {
    pub feedback: StudioGuiPlatformTimerStartFailedFeedback,
    pub outcome: StudioGuiPlatformTimerStartFailedOutcome,
    pub follow_up_command: Option<StudioGuiPlatformTimerFollowUpCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiPlatformTimerStartFailedFeedbackBatch {
    pub entries: Vec<StudioGuiPlatformTimerStartFailedFeedbackEntry>,
    pub snapshot: StudioGuiSnapshot,
    pub next_native_timer_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformTimerStartFailedFeedbackBatch {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn next_native_timer_due_at(&self) -> Option<SystemTime> {
        self.next_native_timer_schedule
            .as_ref()
            .map(|schedule| schedule.slot.timer.due_at)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.snapshot.window_model_for_window(window_id)
    }
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
#[allow(clippy::large_enum_variant)]
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
    gui_activity_lines: Vec<String>,
    current_schedule: Option<StudioGuiNativeTimerSchedule>,
}

impl StudioGuiPlatformHost {
    pub fn new(config: &crate::StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            driver: StudioGuiDriver::new(config)?,
            platform_timer_driver: StudioGuiPlatformTimerDriverState::default(),
            platform_notice: None,
            platform_log_entries: Vec::new(),
            gui_activity_lines: Vec::new(),
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

    pub fn gui_activity_lines(&self) -> &[String] {
        &self.gui_activity_lines
    }

    pub fn latest_gui_error_line(&self) -> Option<&str> {
        self.gui_activity_lines
            .iter()
            .rev()
            .find(|line| {
                line.starts_with("event failed:") || line.starts_with("timer dispatch failed")
            })
            .map(String::as_str)
    }

    pub fn platform_notice(&self) -> Option<&RunPanelNotice> {
        self.platform_notice.as_ref()
    }

    pub fn record_activity_line(&mut self, line: impl Into<String>) {
        self.push_activity_line(line.into());
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

    pub fn execute_platform_dispatch(
        &mut self,
        mut dispatch: StudioGuiPlatformDispatch,
        executor: &mut impl StudioGuiPlatformTimerExecutor,
    ) -> RfResult<StudioGuiPlatformExecutedDispatch> {
        let timer_execution =
            self.execute_platform_timer_request(dispatch.native_timer_request.as_ref(), executor)?;
        self.record_timer_execution_activity(&timer_execution);
        let snapshot = self.snapshot();
        let window_id = dispatch.window.layout_state.scope.window_id;
        dispatch.snapshot = snapshot.clone();
        dispatch.window = snapshot.window_model_for_window(window_id);
        Ok(StudioGuiPlatformExecutedDispatch {
            dispatch,
            timer_execution,
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

    pub fn acknowledge_platform_timer_started_feedbacks(
        &mut self,
        feedbacks: &[StudioGuiPlatformTimerStartedFeedback],
    ) -> StudioGuiPlatformTimerStartedFeedbackBatch {
        let mut entries = Vec::with_capacity(feedbacks.len());
        for feedback in feedbacks {
            let outcome = self
                .acknowledge_platform_timer_started(&feedback.schedule, feedback.native_timer_id);
            let follow_up_command = outcome.follow_up_command();
            entries.push(StudioGuiPlatformTimerStartedFeedbackEntry {
                feedback: feedback.clone(),
                outcome,
                follow_up_command,
            });
        }
        StudioGuiPlatformTimerStartedFeedbackBatch {
            entries,
            snapshot: self.snapshot(),
            next_native_timer_schedule: self.current_schedule.clone(),
        }
    }

    pub fn acknowledge_platform_timer_started_feedbacks_and_execute_follow_up_commands(
        &mut self,
        feedbacks: &[StudioGuiPlatformTimerStartedFeedback],
        executor: &mut impl StudioGuiPlatformTimerExecutor,
    ) -> RfResult<StudioGuiPlatformTimerStartedFeedbackBatch> {
        let batch = self.acknowledge_platform_timer_started_feedbacks(feedbacks);
        for command in batch.follow_up_commands() {
            executor.execute_platform_timer_follow_up_command(&command)?;
        }
        Ok(batch)
    }

    pub fn acknowledge_platform_timer_start_failed_feedbacks(
        &mut self,
        feedbacks: &[StudioGuiPlatformTimerStartFailedFeedback],
    ) -> StudioGuiPlatformTimerStartFailedFeedbackBatch {
        let mut entries = Vec::with_capacity(feedbacks.len());
        for feedback in feedbacks {
            let outcome =
                self.acknowledge_platform_timer_start_failed(&feedback.schedule, &feedback.detail);
            let follow_up_command = outcome.follow_up_command();
            entries.push(StudioGuiPlatformTimerStartFailedFeedbackEntry {
                feedback: feedback.clone(),
                outcome,
                follow_up_command,
            });
        }
        StudioGuiPlatformTimerStartFailedFeedbackBatch {
            entries,
            snapshot: self.snapshot(),
            next_native_timer_schedule: self.current_schedule.clone(),
        }
    }

    pub fn process_async_platform_round(
        &mut self,
        input: StudioGuiPlatformAsyncRoundInput,
    ) -> RfResult<StudioGuiPlatformAsyncRound> {
        let started_feedback_batch =
            self.acknowledge_platform_timer_started_feedbacks(&input.started_feedbacks);
        let start_failed_feedback_batch =
            self.acknowledge_platform_timer_start_failed_feedbacks(&input.start_failed_feedbacks);
        let native_timer_callback_batch =
            self.dispatch_native_timer_elapsed_by_native_ids(&input.native_timer_ids)?;
        let due_timer_drain = match input.due_at {
            Some(now) => Some(self.dispatch_due_native_timer_events_batch(now)?),
            None => None,
        };

        Ok(StudioGuiPlatformAsyncRound {
            input,
            started_feedback_batch,
            start_failed_feedback_batch,
            native_timer_callback_batch,
            due_timer_drain,
            snapshot: self.snapshot(),
            next_native_timer_schedule: self.current_schedule.clone(),
        })
    }

    pub fn process_async_platform_round_and_execute_actions(
        &mut self,
        input: StudioGuiPlatformAsyncRoundInput,
        executor: &mut impl StudioGuiPlatformTimerExecutor,
    ) -> RfResult<StudioGuiPlatformExecutedAsyncRound> {
        let round = self.process_async_platform_round(input)?;
        let mut actions = Vec::new();
        for action in round.actions() {
            match action {
                StudioGuiPlatformAsyncRoundAction::FollowUpCommand(command) => {
                    executor.execute_platform_timer_follow_up_command(&command)?;
                    actions.push(StudioGuiPlatformExecutedAsyncRoundAction::FollowUpCommand(
                        command,
                    ));
                }
                StudioGuiPlatformAsyncRoundAction::TimerRequest(request) => {
                    let execution =
                        self.execute_platform_timer_request(Some(&request), executor)?;
                    actions.push(StudioGuiPlatformExecutedAsyncRoundAction::TimerRequest {
                        request,
                        execution,
                    });
                }
            }
        }

        Ok(StudioGuiPlatformExecutedAsyncRound {
            round,
            actions,
            snapshot: self.snapshot(),
            next_native_timer_schedule: self.current_schedule.clone(),
        })
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
        let outcome = match self.platform_timer_driver.resolve_callback(native_timer_id) {
            StudioGuiPlatformTimerCallbackResolution::Dispatch { schedule } => self
                .dispatch_native_timer_elapsed(schedule.window_id, schedule.handle_id)
                .map(StudioGuiPlatformNativeTimerCallbackOutcome::Dispatched)?,
            StudioGuiPlatformTimerCallbackResolution::IgnoredUnknownNativeTimer {
                native_timer_id,
            } => StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
                native_timer_id,
            },
            StudioGuiPlatformTimerCallbackResolution::IgnoredStaleNativeTimer {
                native_timer_id,
            } => StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
                native_timer_id,
            },
        };
        match &outcome {
            StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
                native_timer_id,
            } => self.push_activity_line(format!(
                "timer::ignored unknown native_timer_id={native_timer_id}"
            )),
            StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
                native_timer_id,
            } => self.push_activity_line(format!(
                "timer::ignored stale native_timer_id={native_timer_id}"
            )),
            StudioGuiPlatformNativeTimerCallbackOutcome::Dispatched(_) => {}
        }
        Ok(outcome)
    }

    pub fn dispatch_native_timer_elapsed_by_native_id_and_execute_platform_timer(
        &mut self,
        native_timer_id: StudioGuiPlatformNativeTimerId,
        executor: &mut impl StudioGuiPlatformTimerExecutor,
    ) -> RfResult<StudioGuiPlatformExecutedNativeTimerCallbackOutcome> {
        match self.dispatch_native_timer_elapsed_by_native_id(native_timer_id)? {
            StudioGuiPlatformNativeTimerCallbackOutcome::Dispatched(dispatch) => self
                .execute_platform_dispatch(dispatch, executor)
                .map(StudioGuiPlatformExecutedNativeTimerCallbackOutcome::Dispatched),
            StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
                native_timer_id,
            } => Ok(
                StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
                    native_timer_id,
                },
            ),
            StudioGuiPlatformNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
                native_timer_id,
            } => Ok(
                StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
                    native_timer_id,
                },
            ),
        }
    }

    pub fn dispatch_native_timer_elapsed_by_native_ids(
        &mut self,
        native_timer_ids: &[StudioGuiPlatformNativeTimerId],
    ) -> RfResult<StudioGuiPlatformNativeTimerCallbackBatch> {
        let mut callbacks = Vec::with_capacity(native_timer_ids.len());
        for native_timer_id in native_timer_ids {
            callbacks.push(self.dispatch_native_timer_elapsed_by_native_id(*native_timer_id)?);
        }
        Ok(StudioGuiPlatformNativeTimerCallbackBatch {
            callbacks,
            snapshot: self.snapshot(),
            next_native_timer_schedule: self.current_schedule.clone(),
        })
    }

    pub fn dispatch_native_timer_elapsed_by_native_ids_and_execute_platform_timers(
        &mut self,
        native_timer_ids: &[StudioGuiPlatformNativeTimerId],
        executor: &mut impl StudioGuiPlatformTimerExecutor,
    ) -> RfResult<StudioGuiPlatformExecutedNativeTimerCallbackBatch> {
        let mut callbacks = Vec::with_capacity(native_timer_ids.len());
        for native_timer_id in native_timer_ids {
            callbacks.push(
                self.dispatch_native_timer_elapsed_by_native_id_and_execute_platform_timer(
                    *native_timer_id,
                    executor,
                )?,
            );
        }
        Ok(StudioGuiPlatformExecutedNativeTimerCallbackBatch {
            callbacks,
            snapshot: self.snapshot(),
            next_native_timer_schedule: self.current_schedule.clone(),
        })
    }

    pub fn dispatch_event(&mut self, event: StudioGuiEvent) -> RfResult<StudioGuiPlatformDispatch> {
        let previous_schedule = self.current_schedule.clone();
        let dispatch = self.driver.dispatch_event(event)?;
        let next_schedule = self.driver.native_timer_runtime().next_schedule();
        self.current_schedule = next_schedule.clone();
        let dispatch = platform_dispatch_from_driver(
            self,
            dispatch,
            plan_platform_timer_request(previous_schedule, next_schedule),
        );
        if should_record_dispatch_activity(&dispatch) {
            self.record_dispatch_activity(&dispatch);
        }
        Ok(dispatch)
    }

    pub fn dispatch_event_and_execute_platform_timer(
        &mut self,
        event: StudioGuiEvent,
        executor: &mut impl StudioGuiPlatformTimerExecutor,
    ) -> RfResult<StudioGuiPlatformExecutedDispatch> {
        let dispatch = self.dispatch_event(event)?;
        self.execute_platform_dispatch(dispatch, executor)
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
        Ok(self.dispatch_due_native_timer_events_batch(now)?.dispatches)
    }

    pub fn dispatch_due_native_timer_events_batch(
        &mut self,
        now: SystemTime,
    ) -> RfResult<StudioGuiPlatformDueTimerDrain> {
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
        Ok(StudioGuiPlatformDueTimerDrain {
            now,
            dispatches: platform_dispatches,
            snapshot: self.snapshot(),
            next_native_timer_schedule: self.current_schedule.clone(),
        })
    }

    pub fn dispatch_due_native_timer_events_and_execute_platform_timers(
        &mut self,
        now: SystemTime,
        executor: &mut impl StudioGuiPlatformTimerExecutor,
    ) -> RfResult<Vec<StudioGuiPlatformExecutedDispatch>> {
        Ok(self
            .drain_due_native_timer_events_and_execute_platform_timers(now, executor)?
            .dispatches)
    }

    pub fn drain_due_native_timer_events_and_execute_platform_timers(
        &mut self,
        now: SystemTime,
        executor: &mut impl StudioGuiPlatformTimerExecutor,
    ) -> RfResult<StudioGuiPlatformExecutedDueTimerDrain> {
        let due_dispatches = self.dispatch_due_native_timer_events(now)?;
        let mut executed_dispatches = Vec::with_capacity(due_dispatches.len());
        for dispatch in due_dispatches {
            executed_dispatches.push(self.execute_platform_dispatch(dispatch, executor)?);
        }
        Ok(StudioGuiPlatformExecutedDueTimerDrain {
            now,
            dispatches: executed_dispatches,
            snapshot: self.snapshot(),
            next_native_timer_schedule: self.current_schedule.clone(),
        })
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
        snapshot.runtime.platform_timer_lines = self.platform_timer_lines();
        snapshot.runtime.gui_activity_lines = self.gui_activity_lines.clone();
        snapshot
            .runtime
            .log_entries
            .extend(self.platform_log_entries.iter().cloned());
        snapshot
    }

    fn enrich_window(&self, mut window: StudioGuiWindowModel) -> StudioGuiWindowModel {
        window.runtime.platform_notice = self.platform_notice.clone();
        window.runtime.platform_timer_lines = self.platform_timer_lines();
        window.runtime.gui_activity_lines = self.gui_activity_lines.clone();
        window
            .runtime
            .log_entries
            .extend(self.platform_log_entries.iter().cloned());
        window.runtime.latest_log_entry = window.runtime.log_entries.last().cloned();
        window
    }

    fn platform_timer_lines(&self) -> Vec<String> {
        vec![
            format!(
                "Current schedule: {}",
                self.current_schedule
                    .as_ref()
                    .map(format_native_timer_schedule)
                    .unwrap_or_else(|| "None".to_string())
            ),
            format!(
                "Native binding: {}",
                self.current_platform_timer_binding()
                    .map(format_platform_timer_binding)
                    .unwrap_or_else(|| "None".to_string())
            ),
        ]
    }

    fn push_activity_line(&mut self, line: String) {
        const MAX_ENTRIES: usize = 64;
        self.gui_activity_lines.push(line);
        if self.gui_activity_lines.len() > MAX_ENTRIES {
            let drain_count = self.gui_activity_lines.len() - MAX_ENTRIES;
            self.gui_activity_lines.drain(0..drain_count);
        }
    }

    fn record_dispatch_activity(&mut self, dispatch: &StudioGuiPlatformDispatch) {
        self.push_activity_line(format_platform_dispatch_activity(dispatch));
    }

    fn record_timer_execution_activity(
        &mut self,
        execution: &StudioGuiPlatformTimerExecutionOutcome,
    ) {
        match execution {
            StudioGuiPlatformTimerExecutionOutcome::NoCommand => {}
            StudioGuiPlatformTimerExecutionOutcome::Executed {
                command,
                executor_response,
                host_outcome,
                follow_up_command,
            } => {
                let mut line = format!(
                    "platform timer {} -> {} -> {}",
                    format_platform_timer_command(command),
                    format_platform_timer_executor_response(executor_response),
                    format_platform_timer_host_outcome(host_outcome)
                );
                if let Some(follow_up_command) = follow_up_command.as_ref() {
                    line.push_str(&format!(
                        " | follow-up {}",
                        format_platform_timer_follow_up_command(follow_up_command)
                    ));
                }
                self.push_activity_line(line);
            }
        }
    }
}

fn format_platform_dispatch_activity(dispatch: &StudioGuiPlatformDispatch) -> String {
    let summary = match &dispatch.outcome {
        StudioGuiDriverOutcome::HostCommand(crate::StudioGuiHostCommandOutcome::WindowOpened(
            opened,
        )) => format!(
            "window opened #{}/{}-{}",
            opened.registration.window_id,
            match opened.registration.role {
                crate::StudioWindowHostRole::EntitlementTimerOwner => "owner",
                crate::StudioWindowHostRole::Observer => "observer",
            },
            opened.registration.layout_slot
        ),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowDispatched(dispatch),
        ) => format!("window dispatch #{}", dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::LifecycleDispatched(lifecycle),
        ) => format!("lifecycle {:?}", lifecycle.event),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::UiCommandDispatched(result),
        ) => match result {
            crate::StudioGuiHostUiCommandDispatchResult::Executed(dispatch) => {
                format!("command dispatch #{}", dispatch.target_window_id)
            }
            crate::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                command_id,
                target_window_id,
                ..
            }
            | crate::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasUnitLayoutMove {
                command_id,
                target_window_id,
                ..
            } => match target_window_id {
                Some(window_id) => format!("command {command_id} -> #{window_id}"),
                None => format!("command {command_id}"),
            },
            crate::StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                command_id,
                detail,
                ..
            } => format!("command disabled {command_id}: {detail}"),
            crate::StudioGuiHostUiCommandDispatchResult::IgnoredMissing { command_id, .. } => {
                format!("command missing {command_id}")
            }
        },
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::InspectorDraftUpdated(dispatch),
        ) => format!("inspector draft update #{}", dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::InspectorDraftCommitted(dispatch),
        ) => format!("inspector draft commit #{}", dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::InspectorDraftDiscarded(dispatch),
        ) => format!("inspector draft discard #{}", dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::InspectorDraftBatchCommitted(dispatch),
        ) => format!(
            "inspector draft batch commit #{}",
            dispatch.target_window_id
        ),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::InspectorDraftBatchDiscarded(dispatch),
        ) => format!(
            "inspector draft batch discard #{}",
            dispatch.target_window_id
        ),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::InspectorCompositionNormalized(dispatch),
        ) => format!(
            "inspector composition normalize #{}",
            dispatch.target_window_id
        ),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::InspectorCompositionComponentAdded(dispatch),
        ) => format!(
            "inspector composition component add #{}",
            dispatch.target_window_id
        ),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowDropTargetQueried(result),
        ) => format!("drop query {:?}", result.query),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowDropTargetApplied(result),
        ) => format!("drop apply {:?}", result.query),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::CanvasInteracted(result),
        ) => format!("canvas {}", format!("{:?}", result.action).to_lowercase()),
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::CanvasUnitLayoutMoved(result),
        ) => format!(
            "canvas layout move {} -> ({:.1}, {:.1})",
            result.unit_id.as_str(),
            result.position.x,
            result.position.y
        ),
        StudioGuiDriverOutcome::HostCommand(crate::StudioGuiHostCommandOutcome::WindowClosed(
            result,
        )) => match result.close.as_ref() {
            Some(close) => format!("window closed #{}", close.window_id),
            None => "window close ignored".to_string(),
        },
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(_),
        )
        | StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(_),
        ) => "drop preview".to_string(),
        StudioGuiDriverOutcome::CanvasInteraction(result) => {
            format!("canvas {}", format!("{:?}", result.action).to_lowercase())
        }
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            format!("layout {:?}", result.mutation)
        }
        StudioGuiDriverOutcome::IgnoredNativeTimerElapsed { handle_id, .. } => {
            format!("timer ignored handle={handle_id}")
        }
        StudioGuiDriverOutcome::IgnoredShortcut { shortcut, reason } => {
            format!("shortcut ignored {:?} {:?}", shortcut, reason)
        }
    };

    match dispatch.native_timer_request.as_ref() {
        Some(request) => format!(
            "{summary} | request {}",
            format_platform_timer_request(request)
        ),
        None => summary,
    }
}

fn should_record_dispatch_activity(dispatch: &StudioGuiPlatformDispatch) -> bool {
    !matches!(
        &dispatch.outcome,
        StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::UiCommandDispatched(
                crate::StudioGuiHostUiCommandDispatchResult::IgnoredDisabled { .. }
            )
        )
    )
}

fn format_native_timer_schedule(schedule: &StudioGuiNativeTimerSchedule) -> String {
    format!(
        "window={:?} handle={} effect={} due_at={:?} reason={:?}",
        schedule.window_id,
        schedule.handle_id,
        schedule.slot.effect_id,
        schedule.slot.timer.due_at,
        schedule.slot.timer.reason
    )
}

fn format_platform_timer_binding(binding: &StudioGuiPlatformTimerBinding) -> String {
    format!(
        "native_timer_id={} {}",
        binding.native_timer_id,
        format_native_timer_schedule(&binding.schedule)
    )
}

fn format_platform_timer_request(request: &StudioGuiPlatformTimerRequest) -> String {
    match request {
        StudioGuiPlatformTimerRequest::Arm { schedule } => {
            format!("arm {}", format_native_timer_schedule(schedule))
        }
        StudioGuiPlatformTimerRequest::Rearm { previous, schedule } => format!(
            "rearm {} -> {}",
            format_native_timer_schedule(previous),
            format_native_timer_schedule(schedule)
        ),
        StudioGuiPlatformTimerRequest::Clear { previous } => {
            format!("clear {}", format_native_timer_schedule(previous))
        }
    }
}

fn format_platform_timer_command(command: &StudioGuiPlatformTimerCommand) -> String {
    match command {
        StudioGuiPlatformTimerCommand::Arm { schedule } => {
            format!("arm {}", format_native_timer_schedule(schedule))
        }
        StudioGuiPlatformTimerCommand::Rearm { previous, schedule } => format!(
            "rearm {} -> {}",
            previous
                .as_ref()
                .map(format_platform_timer_binding)
                .unwrap_or_else(|| "None".to_string()),
            format_native_timer_schedule(schedule)
        ),
        StudioGuiPlatformTimerCommand::Clear { previous } => match previous {
            Some(previous) => format!("clear {}", format_platform_timer_binding(previous)),
            None => "clear none".to_string(),
        },
    }
}

fn format_platform_timer_executor_response(
    response: &StudioGuiPlatformTimerExecutorResponse,
) -> String {
    match response {
        StudioGuiPlatformTimerExecutorResponse::Started { native_timer_id } => {
            format!("started native_timer_id={native_timer_id}")
        }
        StudioGuiPlatformTimerExecutorResponse::StartFailed { detail } => {
            format!("start_failed {detail}")
        }
        StudioGuiPlatformTimerExecutorResponse::Cleared => "cleared".to_string(),
    }
}

fn format_platform_timer_host_outcome(outcome: &StudioGuiPlatformTimerHostOutcome) -> String {
    match outcome {
        StudioGuiPlatformTimerHostOutcome::Started(outcome) => {
            format!(
                "host_started {}",
                format_platform_timer_started_outcome(outcome)
            )
        }
        StudioGuiPlatformTimerHostOutcome::StartFailed(outcome) => {
            format!(
                "host_start_failed {}",
                format_platform_timer_start_failed_outcome(outcome)
            )
        }
        StudioGuiPlatformTimerHostOutcome::Cleared => "host_cleared".to_string(),
    }
}

fn format_platform_timer_started_outcome(outcome: &StudioGuiPlatformTimerStartedOutcome) -> String {
    match outcome {
        StudioGuiPlatformTimerStartedOutcome::Applied(result) => {
            format!("applied {:?}", result.status)
        }
        StudioGuiPlatformTimerStartedOutcome::IgnoredMissingPendingSchedule { ack, .. } => {
            format!("ignored_missing {:?}", ack.status)
        }
        StudioGuiPlatformTimerStartedOutcome::IgnoredStalePendingSchedule { ack, .. } => {
            format!("ignored_stale {:?}", ack.status)
        }
    }
}

fn format_platform_timer_start_failed_outcome(
    outcome: &StudioGuiPlatformTimerStartFailedOutcome,
) -> String {
    match outcome {
        StudioGuiPlatformTimerStartFailedOutcome::Applied(result) => {
            format!("applied {:?}", result.status)
        }
        StudioGuiPlatformTimerStartFailedOutcome::IgnoredMissingPendingSchedule { failure } => {
            format!("ignored_missing {:?}", failure.status)
        }
        StudioGuiPlatformTimerStartFailedOutcome::IgnoredStalePendingSchedule { failure } => {
            format!("ignored_stale {:?}", failure.status)
        }
    }
}

fn format_platform_timer_follow_up_command(
    command: &StudioGuiPlatformTimerFollowUpCommand,
) -> String {
    match command {
        StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer { native_timer_id } => {
            format!("clear native_timer_id={native_timer_id}")
        }
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
mod tests;
