use std::time::{Duration, SystemTime};

use crate::entitlement_panel_view::EntitlementPanelViewModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPanelTextView {
    pub title: &'static str,
    pub lines: Vec<String>,
}

impl EntitlementPanelTextView {
    pub fn from_view_model(view: &EntitlementPanelViewModel) -> Self {
        let mut lines = vec![
            format!("Auth: {}", view.auth_label),
            format!("Entitlement: {}", view.entitlement_label),
            format!("Allowed packages: {}", view.allowed_package_count),
            format!("Cached manifests: {}", view.package_manifest_count),
        ];

        if let Some(user) = view.current_user_label.as_deref() {
            lines.push(format!("User: {user}"));
        }
        if let Some(authority_url) = view.authority_url.as_deref() {
            lines.push(format!("Authority: {authority_url}"));
        }
        if let Some(last_synced_at) = view.last_synced_at {
            lines.push(format!(
                "Last synced: {}",
                format_system_time(last_synced_at)
            ));
        }
        if let Some(offline_lease_expires_at) = view.offline_lease_expires_at {
            lines.push(format!(
                "Offline lease expires: {}",
                format_system_time(offline_lease_expires_at)
            ));
        }
        if let Some(notice) = view.notice.as_ref() {
            lines.push(format!(
                "Notice: {} [{}]",
                notice.title,
                notice_level_label(notice.level)
            ));
            lines.push(format!("Notice detail: {}", notice.message));
        }
        if let Some(last_error) = view.last_error.as_deref() {
            lines.push(format!("Last error: {last_error}"));
        }

        lines.push(format!(
            "Primary action: {} [{}]",
            view.primary_action.label,
            enabled_label(view.primary_action.enabled)
        ));
        lines.push(format!("Primary detail: {}", view.primary_action.detail));
        if !view.secondary_actions.is_empty() {
            lines.push("Secondary actions:".to_string());
            lines.extend(view.secondary_actions.iter().map(|action| {
                format!(
                    "  - {} [{}] | {}",
                    action.label,
                    enabled_label(action.enabled),
                    action.detail
                )
            }));
        }

        Self {
            title: "Entitlement",
            lines,
        }
    }
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled { "enabled" } else { "disabled" }
}

fn notice_level_label(level: crate::EntitlementNoticeLevel) -> &'static str {
    match level {
        crate::EntitlementNoticeLevel::Info => "info",
        crate::EntitlementNoticeLevel::Warning => "warning",
        crate::EntitlementNoticeLevel::Error => "error",
    }
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
