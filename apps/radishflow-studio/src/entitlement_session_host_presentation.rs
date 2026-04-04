use std::fmt::Debug;

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
                format_optional_debug(schedule.next_check_at.as_ref())
            ),
            format!(
                "Next sync window: {}",
                format_optional_debug(schedule.next_sync_at.as_ref())
            ),
            format!(
                "Next offline refresh window: {}",
                format_optional_debug(schedule.next_offline_refresh_at.as_ref())
            ),
            format!(
                "Recommended action: {}",
                format_optional_debug(schedule.recommended_action.as_ref())
            ),
            format!(
                "Recommended reason: {}",
                schedule.recommended_reason.as_deref().unwrap_or("None")
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
        "{:?} at {:?} after {:?} ({:?})",
        timer.event, timer.due_at, timer.delay, timer.reason
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
