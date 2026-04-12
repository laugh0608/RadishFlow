use std::time::{Duration, SystemTime};

use rf_types::RfResult;
use rf_ui::{EntitlementActionId, EntitlementNotice, EntitlementNoticeLevel};

use crate::{
    EntitlementPanelDriverState, EntitlementSessionDriverState, EntitlementSessionEvent,
    EntitlementSessionEventDriverOutcome, EntitlementSessionPanelDriverOutcome,
    EntitlementSessionRuntime, RadishFlowControlPlaneClient,
    dispatch_entitlement_session_event_with_control_plane,
    dispatch_entitlement_session_panel_primary_action_with_control_plane,
    dispatch_entitlement_session_panel_widget_action_with_control_plane,
    snapshot_entitlement_panel_driver_state, snapshot_entitlement_session_driver_state,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitlementSessionLifecycleEvent {
    SessionStarted,
    LoginCompleted,
    TimerElapsed,
    NetworkRestored,
    WindowForegrounded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementSessionHostTrigger {
    LifecycleEvent(EntitlementSessionLifecycleEvent),
    EntitlementCommandCompleted(crate::StudioEntitlementActionOutcome),
    PanelPrimaryAction,
    PanelAction(EntitlementActionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementSessionHostDispatch {
    Event(EntitlementSessionEventDriverOutcome),
    Panel(EntitlementSessionPanelDriverOutcome),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitlementSessionTimerReason {
    ImmediateCheck,
    ScheduledCheck,
    BackoffRetry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionTimerArm {
    pub event: EntitlementSessionLifecycleEvent,
    pub due_at: SystemTime,
    pub delay: Duration,
    pub reason: EntitlementSessionTimerReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementSessionTimerCommand {
    Keep {
        timer: EntitlementSessionTimerArm,
    },
    Schedule {
        timer: EntitlementSessionTimerArm,
    },
    Reschedule {
        previous: EntitlementSessionTimerArm,
        next: EntitlementSessionTimerArm,
    },
    Clear {
        previous: EntitlementSessionTimerArm,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostState {
    pub driver: EntitlementSessionDriverState,
    pub next_timer: Option<EntitlementSessionTimerArm>,
    pub host_notice: Option<EntitlementNotice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostSnapshot {
    pub state: EntitlementSessionHostState,
    pub timer_command: Option<EntitlementSessionTimerCommand>,
    pub panel: EntitlementPanelDriverState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntitlementSessionHostContext {
    current_timer: Option<EntitlementSessionTimerArm>,
    last_snapshot: Option<EntitlementSessionHostSnapshot>,
}

impl EntitlementSessionHostContext {
    pub fn current_timer(&self) -> Option<&EntitlementSessionTimerArm> {
        self.current_timer.as_ref()
    }

    pub fn last_snapshot(&self) -> Option<&EntitlementSessionHostSnapshot> {
        self.last_snapshot.as_ref()
    }

    pub fn record_snapshot(&mut self, snapshot: EntitlementSessionHostSnapshot) {
        self.current_timer = snapshot.state.next_timer.clone();
        self.last_snapshot = Some(snapshot);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostOutcome {
    pub trigger: EntitlementSessionHostTrigger,
    pub dispatch: EntitlementSessionHostDispatch,
    pub snapshot: EntitlementSessionHostSnapshot,
}

pub fn snapshot_entitlement_session_host_state(
    app_state: &rf_ui::AppState,
    now: SystemTime,
    policy: &crate::EntitlementSessionPolicy,
    session_state: &crate::EntitlementSessionState,
) -> EntitlementSessionHostState {
    let driver = snapshot_entitlement_session_driver_state(app_state, now, policy, session_state);
    let next_timer = timer_arm_from_schedule(&driver.schedule, now);
    let host_notice = host_notice_from_schedule(&driver.schedule, next_timer.as_ref());

    EntitlementSessionHostState {
        driver,
        next_timer,
        host_notice,
    }
}

pub fn plan_entitlement_session_timer_command(
    current: Option<&EntitlementSessionTimerArm>,
    next: Option<&EntitlementSessionTimerArm>,
) -> Option<EntitlementSessionTimerCommand> {
    match (current, next) {
        (Some(current), Some(next)) if current == next => {
            Some(EntitlementSessionTimerCommand::Keep {
                timer: current.clone(),
            })
        }
        (None, Some(next)) => Some(EntitlementSessionTimerCommand::Schedule {
            timer: next.clone(),
        }),
        (Some(current), Some(next)) => Some(EntitlementSessionTimerCommand::Reschedule {
            previous: current.clone(),
            next: next.clone(),
        }),
        (Some(current), None) => Some(EntitlementSessionTimerCommand::Clear {
            previous: current.clone(),
        }),
        (None, None) => None,
    }
}

pub fn snapshot_entitlement_session_panel_driver_state_with_host_notice(
    app_state: &rf_ui::AppState,
    now: SystemTime,
    policy: &crate::EntitlementSessionPolicy,
    session_state: &crate::EntitlementSessionState,
) -> EntitlementPanelDriverState {
    snapshot_entitlement_session_host(app_state, now, policy, session_state, None).panel
}

pub fn snapshot_entitlement_session_host(
    app_state: &rf_ui::AppState,
    now: SystemTime,
    policy: &crate::EntitlementSessionPolicy,
    session_state: &crate::EntitlementSessionState,
    current_timer: Option<&EntitlementSessionTimerArm>,
) -> EntitlementSessionHostSnapshot {
    let state = snapshot_entitlement_session_host_state(app_state, now, policy, session_state);
    let timer_command =
        plan_entitlement_session_timer_command(current_timer, state.next_timer.as_ref());
    let mut panel = snapshot_entitlement_panel_driver_state(app_state);
    if panel.panel_state.notice.is_none() {
        panel.panel_state.notice = state.host_notice.clone();
        panel.widget = rf_ui::EntitlementPanelWidgetModel::from_state(&panel.panel_state);
    }

    EntitlementSessionHostSnapshot {
        state,
        timer_command,
        panel,
    }
}

pub fn snapshot_entitlement_session_host_with_context(
    app_state: &rf_ui::AppState,
    now: SystemTime,
    policy: &crate::EntitlementSessionPolicy,
    session_state: &crate::EntitlementSessionState,
    context: &mut EntitlementSessionHostContext,
) -> EntitlementSessionHostSnapshot {
    let snapshot = snapshot_entitlement_session_host(
        app_state,
        now,
        policy,
        session_state,
        context.current_timer(),
    );
    context.record_snapshot(snapshot.clone());
    snapshot
}

pub fn dispatch_entitlement_session_host_trigger_with_control_plane<Client>(
    trigger: EntitlementSessionHostTrigger,
    current_timer: Option<&EntitlementSessionTimerArm>,
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionHostOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let dispatch = match &trigger {
        EntitlementSessionHostTrigger::LifecycleEvent(event) => {
            EntitlementSessionHostDispatch::Event(
                dispatch_entitlement_session_event_with_control_plane(
                    map_lifecycle_event_to_session_event(*event),
                    runtime,
                )?,
            )
        }
        EntitlementSessionHostTrigger::EntitlementCommandCompleted(outcome) => {
            EntitlementSessionHostDispatch::Event(
                dispatch_entitlement_session_event_with_control_plane(
                    EntitlementSessionEvent::EntitlementCommandCompleted(outcome.clone()),
                    runtime,
                )?,
            )
        }
        EntitlementSessionHostTrigger::PanelPrimaryAction => EntitlementSessionHostDispatch::Panel(
            dispatch_entitlement_session_panel_primary_action_with_control_plane(runtime)?,
        ),
        EntitlementSessionHostTrigger::PanelAction(action_id) => {
            EntitlementSessionHostDispatch::Panel(
                dispatch_entitlement_session_panel_widget_action_with_control_plane(
                    *action_id, runtime,
                )?,
            )
        }
    };
    let snapshot = snapshot_entitlement_session_host(
        runtime.app_state,
        runtime.now,
        runtime.policy,
        runtime.session_state,
        current_timer,
    );

    Ok(EntitlementSessionHostOutcome {
        trigger,
        dispatch,
        snapshot,
    })
}

pub fn dispatch_entitlement_session_lifecycle_event_with_control_plane<Client>(
    event: EntitlementSessionLifecycleEvent,
    current_timer: Option<&EntitlementSessionTimerArm>,
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionHostOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    dispatch_entitlement_session_host_trigger_with_control_plane(
        EntitlementSessionHostTrigger::LifecycleEvent(event),
        current_timer,
        runtime,
    )
}

pub fn dispatch_entitlement_session_host_trigger_with_context_and_control_plane<Client>(
    trigger: EntitlementSessionHostTrigger,
    context: &mut EntitlementSessionHostContext,
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionHostOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    let current_timer = context.current_timer.clone();
    let outcome = dispatch_entitlement_session_host_trigger_with_control_plane(
        trigger,
        current_timer.as_ref(),
        runtime,
    )?;
    context.record_snapshot(outcome.snapshot.clone());
    Ok(outcome)
}

pub fn dispatch_entitlement_session_lifecycle_event_with_context_and_control_plane<Client>(
    event: EntitlementSessionLifecycleEvent,
    context: &mut EntitlementSessionHostContext,
    runtime: &mut EntitlementSessionRuntime<'_, '_, Client>,
) -> RfResult<EntitlementSessionHostOutcome>
where
    Client: RadishFlowControlPlaneClient,
{
    dispatch_entitlement_session_host_trigger_with_context_and_control_plane(
        EntitlementSessionHostTrigger::LifecycleEvent(event),
        context,
        runtime,
    )
}

fn map_lifecycle_event_to_session_event(
    event: EntitlementSessionLifecycleEvent,
) -> EntitlementSessionEvent {
    match event {
        EntitlementSessionLifecycleEvent::SessionStarted => EntitlementSessionEvent::SessionStarted,
        EntitlementSessionLifecycleEvent::LoginCompleted => EntitlementSessionEvent::LoginCompleted,
        EntitlementSessionLifecycleEvent::TimerElapsed
        | EntitlementSessionLifecycleEvent::NetworkRestored
        | EntitlementSessionLifecycleEvent::WindowForegrounded => {
            EntitlementSessionEvent::TimerElapsed
        }
    }
}

fn timer_arm_from_schedule(
    schedule: &crate::EntitlementSessionSchedule,
    now: SystemTime,
) -> Option<EntitlementSessionTimerArm> {
    let due_at = schedule.next_check_at?;
    let delay = due_at.duration_since(now).unwrap_or(Duration::ZERO);
    let reason = if schedule.blocked_by_backoff {
        EntitlementSessionTimerReason::BackoffRetry
    } else if schedule.recommended_action.is_some() && delay == Duration::ZERO {
        EntitlementSessionTimerReason::ImmediateCheck
    } else {
        EntitlementSessionTimerReason::ScheduledCheck
    };

    Some(EntitlementSessionTimerArm {
        event: EntitlementSessionLifecycleEvent::TimerElapsed,
        due_at,
        delay,
        reason,
    })
}

fn host_notice_from_schedule(
    schedule: &crate::EntitlementSessionSchedule,
    timer: Option<&EntitlementSessionTimerArm>,
) -> Option<EntitlementNotice> {
    if schedule.blocked_by_backoff {
        let backoff = schedule.backoff.as_ref()?;
        return Some(EntitlementNotice::new(
            EntitlementNoticeLevel::Warning,
            "Automatic retry scheduled",
            format!(
                "entitlement session will retry {:?} after backoff at {:?}",
                backoff.action, backoff.retry_not_before
            ),
        ));
    }

    let timer = timer?;
    if timer.reason == EntitlementSessionTimerReason::ScheduledCheck {
        return Some(EntitlementNotice::new(
            EntitlementNoticeLevel::Info,
            "Automatic check scheduled",
            format!(
                "entitlement session will trigger {:?} at {:?}",
                timer.event, timer.due_at
            ),
        ));
    }

    None
}

#[cfg(test)]
mod tests;
