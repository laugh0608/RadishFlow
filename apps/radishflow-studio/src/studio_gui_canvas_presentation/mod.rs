mod view_model;
pub use view_model::{StudioGuiCanvasPresentation, StudioGuiCanvasTextView};
use view_model::{command_result_activity_line, normalize_canvas_unit_kind};

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasUnitBlockViewModel {
    pub unit_id: String,
    pub name: String,
    pub kind: String,
    pub ports: Vec<StudioGuiCanvasUnitPortViewModel>,
    pub status_badges: Vec<StudioGuiCanvasStatusBadgeViewModel>,
    pub port_count: usize,
    pub connected_port_count: usize,
    pub command_id: String,
    pub action_label: String,
    pub hover_text: String,
    pub attention_summary: Option<String>,
    pub layout_slot: usize,
    pub layout_position: Option<rf_ui::CanvasPoint>,
    pub is_active_inspector_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasUnitPortViewModel {
    pub name: String,
    pub direction_label: &'static str,
    pub kind_label: &'static str,
    pub stream_id: Option<String>,
    pub stream_label: Option<String>,
    pub stream_command_id: Option<String>,
    pub binding_label: String,
    pub hover_text: String,
    pub is_connected: bool,
    pub side_index: usize,
    pub side_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasStreamLineEndpointViewModel {
    pub unit_id: String,
    pub port_name: String,
    pub layout_slot: usize,
    pub layout_position: Option<rf_ui::CanvasPoint>,
    pub port_side_index: usize,
    pub port_side_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasStreamLineViewModel {
    pub line_id: String,
    pub stream_id: String,
    pub name: String,
    pub source: Option<StudioGuiCanvasStreamLineEndpointViewModel>,
    pub sink: Option<StudioGuiCanvasStreamLineEndpointViewModel>,
    pub status_badges: Vec<StudioGuiCanvasStatusBadgeViewModel>,
    pub command_id: String,
    pub action_label: String,
    pub hover_text: String,
    pub attention_summary: Option<String>,
    pub is_active_inspector_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasSelectionViewModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub summary: String,
    pub command_id: String,
    pub layout_source_label: Option<&'static str>,
    pub layout_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasFocusCalloutViewModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub title: String,
    pub detail: String,
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasViewportFocusViewModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub source_label: &'static str,
    pub anchor_label: String,
    pub detail: String,
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasViewportViewModel {
    pub mode_label: &'static str,
    pub layout_label: &'static str,
    pub summary: String,
    pub unit_count: usize,
    pub stream_line_count: usize,
    pub focus: Option<StudioGuiCanvasViewportFocusViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasCommandTargetViewModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub label: String,
    pub viewport_anchor_label: Option<String>,
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasCommandResultViewModel {
    pub level: rf_ui::RunPanelNoticeLevel,
    pub status_label: &'static str,
    pub title: String,
    pub detail: String,
    pub activity_line: String,
    pub target: StudioGuiCanvasCommandTargetViewModel,
    pub anchor_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasCommandResultCommandSurfaceViewModel {
    pub level: rf_ui::RunPanelNoticeLevel,
    pub status_label: &'static str,
    pub title: String,
    pub detail: String,
    pub target_command_id: String,
    pub target_label: String,
    pub menu_path_text: String,
    pub search_text: String,
}

impl StudioGuiCanvasCommandResultCommandSurfaceViewModel {
    pub fn matches_query(&self, query: &str) -> bool {
        let query = query.trim().to_ascii_lowercase();
        query.is_empty()
            || query
                .split_whitespace()
                .all(|token| self.search_text.contains(token))
    }
}

impl StudioGuiCanvasCommandResultViewModel {
    pub fn command_surface(&self) -> StudioGuiCanvasCommandResultCommandSurfaceViewModel {
        let target_label = format!(
            "{} {} ({})",
            self.target.kind_label, self.target.target_id, self.target.label
        );
        let menu_path_text = format!(
            "Canvas > Last result > {} > {}",
            self.status_label, self.target.command_id
        );
        let search_text = [
            "canvas",
            "result",
            self.status_label,
            self.title.as_str(),
            self.detail.as_str(),
            self.target.kind_label,
            self.target.target_id.as_str(),
            self.target.label.as_str(),
            self.target.command_id.as_str(),
            self.anchor_label.as_deref().unwrap_or(""),
        ]
        .join(" ")
        .to_ascii_lowercase();

        StudioGuiCanvasCommandResultCommandSurfaceViewModel {
            level: self.level,
            status_label: self.status_label,
            title: self.title.clone(),
            detail: self.detail.clone(),
            target_command_id: self.target.command_id.clone(),
            target_label,
            menu_path_text,
            search_text,
        }
    }

    pub fn pending_edit_unavailable(position: rf_ui::CanvasPoint) -> Self {
        let target = pending_edit_command_target();
        let title = "Canvas pending edit unavailable".to_string();
        let activity_line = command_result_activity_line(&title, &target);
        Self {
            level: rf_ui::RunPanelNoticeLevel::Warning,
            status_label: "pending_edit_unavailable",
            detail: format!(
                "Canvas pending edit commit at ({:.1}, {:.1}) did not create a unit because no pending edit was active.",
                position.x, position.y
            ),
            title,
            activity_line,
            anchor_label: None,
            target,
        }
    }

    pub fn pending_edit_failed(position: rf_ui::CanvasPoint, error_message: &str) -> Self {
        let target = pending_edit_command_target();
        let title = "Canvas pending edit failed".to_string();
        let activity_line = command_result_activity_line(&title, &target);
        Self {
            level: rf_ui::RunPanelNoticeLevel::Error,
            status_label: "pending_edit_failed",
            detail: format!(
                "Canvas pending edit commit at ({:.1}, {:.1}) failed through `{}`: {}",
                position.x, position.y, target.command_id, error_message
            ),
            title,
            activity_line,
            anchor_label: None,
            target,
        }
    }

    pub fn created_unit(
        target: StudioGuiCanvasCommandTargetViewModel,
        anchor_label: impl Into<String>,
        committed: &rf_ui::CanvasEditCommitResult,
    ) -> Self {
        let anchor_label = anchor_label.into();
        let title = "Canvas unit created".to_string();
        Self {
            level: rf_ui::RunPanelNoticeLevel::Info,
            status_label: "created",
            detail: format!(
                "{} `{}` was created at ({:.1}, {:.1}), revision {}, and anchored at `{}`.",
                target.kind_label,
                target.target_id,
                committed.position.x,
                committed.position.y,
                committed.revision,
                anchor_label
            ),
            activity_line: format!(
                "canvas unit created: {} {} -> {}",
                target.kind_label, target.target_id, anchor_label
            ),
            title,
            target,
            anchor_label: Some(anchor_label),
        }
    }

    pub fn moved_unit(
        target: StudioGuiCanvasCommandTargetViewModel,
        anchor_label: impl Into<String>,
        previous_position: Option<rf_ui::CanvasPoint>,
        position: rf_ui::CanvasPoint,
    ) -> Self {
        let anchor_label = anchor_label.into();
        let previous_label = previous_position
            .map(|previous| format!("sidecar ({:.1}, {:.1})", previous.x, previous.y))
            .unwrap_or_else(|| "transient grid slot".to_string());
        let title = if previous_position.is_some() {
            "Canvas unit moved".to_string()
        } else {
            "Canvas unit pinned and moved".to_string()
        };
        let detail = if previous_position.is_some() {
            format!(
                "{} `{}` moved from {} to sidecar ({:.1}, {:.1}) and remains anchored at `{}`.",
                target.kind_label,
                target.target_id,
                previous_label,
                position.x,
                position.y,
                anchor_label
            )
        } else {
            format!(
                "{} `{}` had no sidecar position, so it was pinned from its {} and moved to sidecar ({:.1}, {:.1}); it remains anchored at `{}`.",
                target.kind_label,
                target.target_id,
                previous_label,
                position.x,
                position.y,
                anchor_label
            )
        };
        Self {
            level: rf_ui::RunPanelNoticeLevel::Info,
            status_label: "moved",
            detail,
            activity_line: format!(
                "canvas unit moved: {} {} -> {}",
                target.kind_label, target.target_id, anchor_label
            ),
            title,
            target,
            anchor_label: Some(anchor_label),
        }
    }

    pub fn located(
        target: StudioGuiCanvasCommandTargetViewModel,
        anchor_label: impl Into<String>,
    ) -> Self {
        let anchor_label = anchor_label.into();
        let title = "Canvas object located".to_string();
        Self {
            level: rf_ui::RunPanelNoticeLevel::Info,
            status_label: "located",
            detail: format!(
                "{} `{}` is anchored at `{}`.",
                target.kind_label, target.target_id, anchor_label
            ),
            activity_line: format!(
                "canvas object located: {} {} -> {}",
                target.kind_label, target.target_id, anchor_label
            ),
            title,
            target,
            anchor_label: Some(anchor_label),
        }
    }

    pub fn anchor_unavailable(target: StudioGuiCanvasCommandTargetViewModel) -> Self {
        let title = "Canvas viewport anchor unavailable".to_string();
        let detail = match target.viewport_anchor_label.as_ref() {
            Some(anchor) => format!(
                "{} `{}` was requested at `{anchor}`, but the current Canvas presentation did not confirm that focus anchor.",
                target.kind_label, target.target_id
            ),
            None => format!(
                "{} `{}` was requested, but it is not exposed as a current Canvas object.",
                target.kind_label, target.target_id
            ),
        };
        let activity_line = command_result_activity_line(&title, &target);
        Self {
            level: rf_ui::RunPanelNoticeLevel::Warning,
            status_label: "anchor_unavailable",
            title,
            detail,
            activity_line,
            anchor_label: target.viewport_anchor_label.clone(),
            target,
        }
    }

    pub fn dispatch_failed(
        target: StudioGuiCanvasCommandTargetViewModel,
        error_message: &str,
    ) -> Self {
        let title = "Canvas object navigation failed".to_string();
        let activity_line = command_result_activity_line(&title, &target);
        Self {
            level: rf_ui::RunPanelNoticeLevel::Error,
            status_label: "dispatch_failed",
            detail: format!(
                "{} `{}` could not be focused through `{}`: {}",
                target.kind_label, target.target_id, target.command_id, error_message
            ),
            title,
            activity_line,
            anchor_label: target.viewport_anchor_label.clone(),
            target,
        }
    }

    pub fn anchor_expired(
        target: StudioGuiCanvasCommandTargetViewModel,
        anchor_label: impl Into<String>,
    ) -> Self {
        let anchor_label = anchor_label.into();
        let title = "Canvas navigation anchor expired".to_string();
        let activity_line = command_result_activity_line(&title, &target);
        Self {
            level: rf_ui::RunPanelNoticeLevel::Warning,
            status_label: "anchor_expired",
            detail: format!(
                "{} `{}` is no longer exposed by the current Canvas viewport presentation.",
                target.kind_label, target.target_id
            ),
            title,
            activity_line,
            target,
            anchor_label: Some(anchor_label),
        }
    }
}

fn pending_edit_command_target() -> StudioGuiCanvasCommandTargetViewModel {
    StudioGuiCanvasCommandTargetViewModel {
        kind_label: "Edit",
        target_id: "pending_edit".to_string(),
        label: "Pending canvas edit".to_string(),
        viewport_anchor_label: None,
        command_id: "canvas.commit_pending_edit_at".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasObjectListItemViewModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub label: String,
    pub detail: String,
    pub attention_summary: Option<String>,
    pub viewport_anchor_label: String,
    pub command_id: String,
    pub related_stream_ids: Vec<String>,
    pub status_badges: Vec<StudioGuiCanvasStatusBadgeViewModel>,
    pub is_active: bool,
}

impl StudioGuiCanvasObjectListItemViewModel {
    pub fn command_target(&self) -> StudioGuiCanvasCommandTargetViewModel {
        StudioGuiCanvasCommandTargetViewModel {
            kind_label: self.kind_label,
            target_id: self.target_id.clone(),
            label: self.label.clone(),
            viewport_anchor_label: Some(self.viewport_anchor_label.clone()),
            command_id: self.command_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasObjectListViewModel {
    pub unit_count: usize,
    pub stream_count: usize,
    pub attention_count: usize,
    pub filter_options: Vec<StudioGuiCanvasObjectListFilterOptionViewModel>,
    pub items: Vec<StudioGuiCanvasObjectListItemViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasObjectListFilterOptionViewModel {
    pub filter_id: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
    pub count: usize,
    pub enabled: bool,
}

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
    pub explicit_accept_enabled: bool,
    pub explicit_accept_command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasPendingEditViewModel {
    pub intent_label: &'static str,
    pub summary: String,
    pub cancel_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasRunStatusViewModel {
    pub status_label: &'static str,
    pub pending_reason_label: Option<&'static str>,
    pub latest_snapshot_id: Option<String>,
    pub summary: Option<String>,
    pub diagnostic_count: usize,
    pub attention_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasStatusBadgeViewModel {
    pub severity_label: &'static str,
    pub short_label: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasLegendItemViewModel {
    pub kind_label: &'static str,
    pub label: String,
    pub detail: String,
    pub swatch_label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasLegendViewModel {
    pub title: &'static str,
    pub items: Vec<StudioGuiCanvasLegendItemViewModel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasViewModel {
    pub run_status: Option<StudioGuiCanvasRunStatusViewModel>,
    pub pending_edit: Option<StudioGuiCanvasPendingEditViewModel>,
    pub place_unit_palette: StudioGuiCanvasPlaceUnitPaletteViewModel,
    pub focused_suggestion_id: Option<String>,
    pub current_selection: Option<StudioGuiCanvasSelectionViewModel>,
    pub focus_callout: Option<StudioGuiCanvasFocusCalloutViewModel>,
    pub viewport: StudioGuiCanvasViewportViewModel,
    pub object_list: StudioGuiCanvasObjectListViewModel,
    pub legend: StudioGuiCanvasLegendViewModel,
    pub unit_count: usize,
    pub stream_line_count: usize,
    pub suggestion_count: usize,
    pub unit_blocks: Vec<StudioGuiCanvasUnitBlockViewModel>,
    pub stream_lines: Vec<StudioGuiCanvasStreamLineViewModel>,
    pub suggestions: Vec<StudioGuiCanvasSuggestionViewModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StudioGuiCanvasPlaceUnitKind {
    Feed,
    Mixer,
    Heater,
    Cooler,
    Valve,
    FlashDrum,
}

impl StudioGuiCanvasPlaceUnitKind {
    pub const fn all() -> &'static [Self] {
        &[
            Self::Feed,
            Self::Mixer,
            Self::Heater,
            Self::Cooler,
            Self::Valve,
            Self::FlashDrum,
        ]
    }

    pub const fn command_id(self) -> &'static str {
        match self {
            Self::Feed => "canvas.begin_place_unit.feed",
            Self::Mixer => "canvas.begin_place_unit.mixer",
            Self::Heater => "canvas.begin_place_unit.heater",
            Self::Cooler => "canvas.begin_place_unit.cooler",
            Self::Valve => "canvas.begin_place_unit.valve",
            Self::FlashDrum => "canvas.begin_place_unit.flash_drum",
        }
    }

    pub const fn unit_kind(self) -> &'static str {
        match self {
            Self::Feed => "Feed",
            Self::Mixer => "Mixer",
            Self::Heater => "Heater",
            Self::Cooler => "Cooler",
            Self::Valve => "Valve",
            Self::FlashDrum => "Flash Drum",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Feed => "Place Feed",
            Self::Mixer => "Place Mixer",
            Self::Heater => "Place Heater",
            Self::Cooler => "Place Cooler",
            Self::Valve => "Place Valve",
            Self::FlashDrum => "Place Flash Drum",
        }
    }

    pub const fn detail(self) -> &'static str {
        match self {
            Self::Feed => "Start placing a Feed on the canvas",
            Self::Mixer => "Start placing a Mixer on the canvas",
            Self::Heater => "Start placing a Heater on the canvas",
            Self::Cooler => "Start placing a Cooler on the canvas",
            Self::Valve => "Start placing a Valve on the canvas",
            Self::FlashDrum => "Start placing a Flash Drum on the canvas",
        }
    }

    pub const fn menu_label(self) -> &'static str {
        match self {
            Self::Feed => "Place Feed",
            Self::Mixer => "Place Mixer",
            Self::Heater => "Place Heater",
            Self::Cooler => "Place Cooler",
            Self::Valve => "Place Valve",
            Self::FlashDrum => "Place Flash Drum",
        }
    }

    pub const fn sort_index(self) -> u16 {
        match self {
            Self::Feed => 0,
            Self::Mixer => 1,
            Self::Heater => 2,
            Self::Cooler => 3,
            Self::Valve => 4,
            Self::FlashDrum => 5,
        }
    }

    pub fn from_command_id(command_id: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|kind| kind.command_id() == command_id)
    }

    fn matches_unit_kind(self, unit_kind: &str) -> bool {
        normalize_canvas_unit_kind(self.unit_kind()) == normalize_canvas_unit_kind(unit_kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasPlaceUnitPaletteViewModel {
    pub title: &'static str,
    pub enabled: bool,
    pub active_unit_kind: Option<String>,
    pub options: Vec<StudioGuiCanvasPlaceUnitOptionViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasPlaceUnitOptionViewModel {
    pub kind: StudioGuiCanvasPlaceUnitKind,
    pub command_id: String,
    pub unit_kind: String,
    pub label: String,
    pub detail: String,
    pub enabled: bool,
    pub active: bool,
    pub search_terms: Vec<String>,
}

impl StudioGuiCanvasPlaceUnitPaletteViewModel {
    fn from_pending_edit(pending_edit: Option<&rf_ui::CanvasEditIntent>) -> Self {
        let active_unit_kind = pending_edit.map(|intent| match intent {
            rf_ui::CanvasEditIntent::PlaceUnit { unit_kind } => unit_kind.clone(),
        });
        let enabled = pending_edit.is_none();
        let options = StudioGuiCanvasPlaceUnitKind::all()
            .iter()
            .copied()
            .map(|kind| {
                let active = active_unit_kind
                    .as_deref()
                    .map(|unit_kind| kind.matches_unit_kind(unit_kind))
                    .unwrap_or(false);
                StudioGuiCanvasPlaceUnitOptionViewModel {
                    kind,
                    command_id: kind.command_id().to_string(),
                    unit_kind: kind.unit_kind().to_string(),
                    label: kind.label().to_string(),
                    detail: kind.detail().to_string(),
                    enabled,
                    active,
                    search_terms: vec![
                        "canvas".to_string(),
                        "place".to_string(),
                        "unit".to_string(),
                        kind.unit_kind().to_string(),
                    ],
                }
            })
            .collect();

        Self {
            title: "Place unit",
            enabled,
            active_unit_kind,
            options,
        }
    }
}

#[cfg(test)]
mod tests;
