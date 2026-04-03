use crate::run_panel_view::RunPanelViewModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelTextView {
    pub title: &'static str,
    pub lines: Vec<String>,
}

impl RunPanelTextView {
    pub fn from_view_model(view: &RunPanelViewModel) -> Self {
        let mut lines = vec![
            format!("Mode: {}", view.mode_label),
            format!("Status: {}", view.status_label),
        ];

        if let Some(notice) = view.notice.as_ref() {
            lines.push(format!(
                "Notice: {} [{}]",
                notice.title,
                notice_level_label(notice.level)
            ));
            lines.push(format!("Notice detail: {}", notice.message));
        }
        if let Some(pending) = view.pending_label {
            lines.push(format!("Pending: {pending}"));
        }
        if let Some(snapshot_id) = view.latest_snapshot_id.as_deref() {
            lines.push(format!("Latest snapshot: {snapshot_id}"));
        }
        if let Some(summary) = view.latest_snapshot_summary.as_deref() {
            lines.push(format!("Summary: {summary}"));
        }
        if let Some(message) = view.latest_log_message.as_deref() {
            lines.push(format!("Latest log: {message}"));
        }

        lines.push(format!(
            "Primary action: {} [{}]",
            view.primary_action.label,
            enabled_label(view.primary_action.enabled)
        ));
        if !view.secondary_actions.is_empty() {
            lines.push("Secondary actions:".to_string());
            lines.extend(
                view.secondary_actions.iter().map(|action| {
                    format!("  - {} [{}]", action.label, enabled_label(action.enabled))
                }),
            );
        }

        Self {
            title: "Run panel",
            lines,
        }
    }
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled { "enabled" } else { "disabled" }
}

fn notice_level_label(level: crate::RunPanelNoticeLevel) -> &'static str {
    match level {
        crate::RunPanelNoticeLevel::Info => "info",
        crate::RunPanelNoticeLevel::Warning => "warning",
        crate::RunPanelNoticeLevel::Error => "error",
    }
}
