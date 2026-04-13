use std::fmt::Debug;
use std::time::{Duration, SystemTime};

use rf_ui::{EntitlementNoticeLevel, EntitlementPanelPresentation};

use crate::{
    EntitlementSessionHostSnapshot, EntitlementSessionHostTimerEffect, EntitlementSessionTimerArm,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostTextView {
    pub title: &'static str,
    pub lines: Vec<String>,
}

impl EntitlementSessionHostTextView {
    pub fn from_snapshot(snapshot: &EntitlementSessionHostSnapshot) -> Self {
        let schedule = &snapshot.state.driver.schedule;
        let mut lines = vec![
            format!(
                "Next check: {}",
                format_optional_system_time(schedule.next_check_at)
            ),
            format!(
                "Next sync window: {}",
                format_optional_system_time(schedule.next_sync_at)
            ),
            format!(
                "Next offline refresh window: {}",
                format_optional_system_time(schedule.next_offline_refresh_at)
            ),
            format!(
                "Recommended action: {}",
                format_optional_debug(schedule.recommended_action.as_ref())
            ),
            format!(
                "Recommended reason: {}",
                schedule.recommended_reason.as_ref().unwrap_or(&"None")
            ),
            format!(
                "Scheduler backoff active: {}",
                enabled_label(schedule.blocked_by_backoff)
            ),
            format!(
                "Next timer: {}",
                snapshot
                    .state
                    .next_timer
                    .as_ref()
                    .map(format_timer_arm)
                    .unwrap_or_else(|| "None".to_string())
            ),
            format!(
                "Timer effect: {}",
                snapshot
                    .timer_effect()
                    .as_ref()
                    .map(format_timer_effect)
                    .unwrap_or_else(|| "None".to_string())
            ),
        ];

        match snapshot.state.host_notice.as_ref() {
            Some(notice) => {
                lines.push(format!(
                    "Host notice: {} [{}]",
                    notice.title,
                    notice_level_label(notice.level)
                ));
                lines.push(format!("Host notice detail: {}", notice.message));
            }
            None => lines.push("Host notice: None".to_string()),
        }

        Self {
            title: "Entitlement host",
            lines,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSessionHostPresentation {
    pub panel: EntitlementPanelPresentation,
    pub text: EntitlementSessionHostTextView,
}

impl EntitlementSessionHostPresentation {
    pub fn from_snapshot(snapshot: &EntitlementSessionHostSnapshot) -> Self {
        Self {
            panel: snapshot.panel.widget.presentation.clone(),
            text: EntitlementSessionHostTextView::from_snapshot(snapshot),
        }
    }
}

impl EntitlementSessionHostSnapshot {
    pub fn text(&self) -> EntitlementSessionHostTextView {
        EntitlementSessionHostTextView::from_snapshot(self)
    }

    pub fn presentation(&self) -> EntitlementSessionHostPresentation {
        EntitlementSessionHostPresentation::from_snapshot(self)
    }
}

fn format_optional_debug<T>(value: Option<&T>) -> String
where
    T: Debug,
{
    value
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|| "None".to_string())
}

fn format_timer_arm(timer: &EntitlementSessionTimerArm) -> String {
    format!(
        "{:?} at {} after {} ({:?})",
        timer.event,
        format_system_time(timer.due_at),
        format_duration(timer.delay),
        timer.reason
    )
}

fn format_timer_effect(effect: &EntitlementSessionHostTimerEffect) -> String {
    match effect {
        EntitlementSessionHostTimerEffect::KeepTimer { timer } => {
            format!("Keep timer {}", format_timer_arm(timer))
        }
        EntitlementSessionHostTimerEffect::ArmTimer { timer } => {
            format!("Arm timer {}", format_timer_arm(timer))
        }
        EntitlementSessionHostTimerEffect::RearmTimer { previous, next } => {
            format!(
                "Rearm timer {} -> {}",
                format_timer_arm(previous),
                format_timer_arm(next)
            )
        }
        EntitlementSessionHostTimerEffect::ClearTimer { previous } => {
            format!("Clear timer {}", format_timer_arm(previous))
        }
    }
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled { "yes" } else { "no" }
}

fn notice_level_label(level: EntitlementNoticeLevel) -> &'static str {
    match level {
        EntitlementNoticeLevel::Info => "info",
        EntitlementNoticeLevel::Warning => "warning",
        EntitlementNoticeLevel::Error => "error",
    }
}

fn format_optional_system_time(value: Option<SystemTime>) -> String {
    value
        .map(format_system_time)
        .unwrap_or_else(|| "None".to_string())
}

fn format_system_time(value: SystemTime) -> String {
    let unix = value
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "before-epoch".to_string());
    let relative = match value.duration_since(SystemTime::now()) {
        Ok(duration) => format!("in {}", format_duration(duration)),
        Err(error) => format!("{} ago", format_duration(error.duration())),
    };
    format!("{relative} (unix={unix}s)")
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3_600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86_400 {
        format!("{}h{}m", seconds / 3_600, (seconds % 3_600) / 60)
    } else {
        format!("{}d{}h", seconds / 86_400, (seconds % 86_400) / 3_600)
    }
}
