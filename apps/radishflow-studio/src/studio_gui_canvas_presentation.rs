use crate::StudioGuiCanvasState;

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasSuggestionViewModel {
    pub id: String,
    pub source_label: &'static str,
    pub status_label: &'static str,
    pub confidence: f32,
    pub target_unit_id: String,
    pub reason: String,
    pub is_focused: bool,
    pub tab_accept_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasViewModel {
    pub focused_suggestion_id: Option<String>,
    pub suggestion_count: usize,
    pub suggestions: Vec<StudioGuiCanvasSuggestionViewModel>,
}

impl StudioGuiCanvasViewModel {
    pub fn from_state(state: &StudioGuiCanvasState) -> Self {
        let focused_suggestion_id = state
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str().to_string());
        let suggestions = state
            .suggestions
            .iter()
            .map(|suggestion| StudioGuiCanvasSuggestionViewModel {
                id: suggestion.id.as_str().to_string(),
                source_label: suggestion_source_label(suggestion.source),
                status_label: suggestion_status_label(suggestion.status),
                confidence: suggestion.confidence,
                target_unit_id: suggestion.ghost.target_unit_id.as_str().to_string(),
                reason: suggestion.reason.clone(),
                is_focused: state.focused_suggestion_id.as_ref() == Some(&suggestion.id),
                tab_accept_enabled: suggestion.can_accept_with_tab(),
            })
            .collect::<Vec<_>>();

        Self {
            focused_suggestion_id,
            suggestion_count: suggestions.len(),
            suggestions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasTextView {
    pub title: &'static str,
    pub lines: Vec<String>,
}

impl StudioGuiCanvasTextView {
    pub fn from_view_model(view: &StudioGuiCanvasViewModel) -> Self {
        let mut lines = vec![
            format!(
                "focused suggestion: {}",
                view.focused_suggestion_id.as_deref().unwrap_or("none")
            ),
            format!("suggestion count: {}", view.suggestion_count),
        ];

        lines.extend(view.suggestions.iter().map(|suggestion| {
            let focus_marker = if suggestion.is_focused { "*" } else { "-" };
            format!(
                "{focus_marker} {} [{}] source={} confidence={:.2} target={} tab_accept={} reason={}",
                suggestion.id,
                suggestion.status_label,
                suggestion.source_label,
                suggestion.confidence,
                suggestion.target_unit_id,
                enabled_label(suggestion.tab_accept_enabled),
                suggestion.reason
            )
        }));

        Self {
            title: "Canvas suggestions",
            lines,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasPresentation {
    pub view: StudioGuiCanvasViewModel,
    pub text: StudioGuiCanvasTextView,
}

impl StudioGuiCanvasPresentation {
    pub fn from_state(state: &StudioGuiCanvasState) -> Self {
        let view = StudioGuiCanvasViewModel::from_state(state);
        let text = StudioGuiCanvasTextView::from_view_model(&view);
        Self { view, text }
    }
}

impl StudioGuiCanvasState {
    pub fn view(&self) -> StudioGuiCanvasViewModel {
        StudioGuiCanvasViewModel::from_state(self)
    }

    pub fn text(&self) -> StudioGuiCanvasTextView {
        StudioGuiCanvasTextView::from_view_model(&self.view())
    }

    pub fn presentation(&self) -> StudioGuiCanvasPresentation {
        StudioGuiCanvasPresentation::from_state(self)
    }
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled { "yes" } else { "no" }
}

fn suggestion_source_label(source: rf_ui::SuggestionSource) -> &'static str {
    match source {
        rf_ui::SuggestionSource::LocalRules => "local_rules",
        rf_ui::SuggestionSource::RadishMind => "radish_mind",
    }
}

fn suggestion_status_label(status: rf_ui::SuggestionStatus) -> &'static str {
    match status {
        rf_ui::SuggestionStatus::Proposed => "proposed",
        rf_ui::SuggestionStatus::Focused => "focused",
        rf_ui::SuggestionStatus::Accepted => "accepted",
        rf_ui::SuggestionStatus::Rejected => "rejected",
        rf_ui::SuggestionStatus::Invalidated => "invalidated",
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        StudioGuiDriver, StudioGuiEvent, StudioRuntimeConfig, StudioRuntimeEntitlementPreflight,
        StudioRuntimeEntitlementSeed,
    };

    fn lease_expiring_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        }
    }

    fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-studio-canvas-presentation-{timestamp}.rfproj.json"
        ));
        let project =
            include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json")
                .replacen(
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-heated\"",
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-liquid\"",
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                );
        fs::write(&project_path, project).expect("expected local rules project");

        (
            StudioRuntimeConfig {
                project_path: project_path.clone(),
                ..lease_expiring_config()
            },
            project_path,
        )
    }

    #[test]
    fn canvas_presentation_reports_empty_canvas_state() {
        let presentation = crate::StudioGuiCanvasState::default().presentation();

        assert_eq!(presentation.view.focused_suggestion_id, None);
        assert_eq!(presentation.view.suggestion_count, 0);
        assert!(presentation.view.suggestions.is_empty());
        assert_eq!(
            presentation.text.lines,
            vec![
                "focused suggestion: none".to_string(),
                "suggestion count: 0".to_string(),
            ]
        );
    }

    #[test]
    fn canvas_presentation_consumes_driver_dispatch_canvas_state() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let presentation = dispatch.canvas.presentation();

        assert_eq!(
            presentation.view.focused_suggestion_id.as_deref(),
            Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
        );
        assert_eq!(presentation.view.suggestion_count, 3);
        assert_eq!(presentation.view.suggestions[0].status_label, "focused");
        assert_eq!(presentation.view.suggestions[0].source_label, "local_rules");
        assert!(presentation.view.suggestions[0].is_focused);
        assert!(presentation.view.suggestions[0].tab_accept_enabled);
        assert_eq!(
            presentation.text.lines,
            vec![
                "focused suggestion: local.flash_drum.connect_inlet.flash-1.stream-heated"
                    .to_string(),
                "suggestion count: 3".to_string(),
                "* local.flash_drum.connect_inlet.flash-1.stream-heated [focused] source=local_rules confidence=0.97 target=flash-1 tab_accept=yes reason=Connect stream `stream-heated` to flash drum inlet `inlet`".to_string(),
                "- local.flash_drum.create_outlet.flash-1.liquid [proposed] source=local_rules confidence=0.93 target=flash-1 tab_accept=yes reason=Create terminal stream `Flash Drum Liquid Outlet` for flash drum outlet `liquid`".to_string(),
                "- local.flash_drum.create_outlet.flash-1.vapor [proposed] source=local_rules confidence=0.92 target=flash-1 tab_accept=yes reason=Create terminal stream `Flash Drum Vapor Outlet` for flash drum outlet `vapor`".to_string(),
            ]
        );

        let _ = fs::remove_file(project_path);
    }
}
