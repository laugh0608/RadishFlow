use rf_model::MaterialStreamState;
use rf_types::{StreamId, UnitId};

use crate::ids::CanvasSuggestionId;

const DEFAULT_TAB_ACCEPT_CONFIDENCE: f32 = 0.8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanvasViewMode {
    #[default]
    Planar,
    Perspective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamVisualKind {
    Material,
    Energy,
    Signal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamVisualState {
    Suggested,
    Incomplete,
    PendingSolve,
    Converged,
    Unconverged,
    Warning,
    Error,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamAnimationMode {
    Static,
    Directional,
    Pulsing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuggestionSource {
    LocalRules,
    RadishMind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GhostElementKind {
    Port,
    Connection,
    StreamName,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasSuggestedStreamBinding {
    Existing { stream_id: StreamId },
    Create { stream: MaterialStreamState },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasSuggestedMaterialConnection {
    pub stream: CanvasSuggestedStreamBinding,
    pub source_unit_id: UnitId,
    pub source_port: String,
    pub sink_unit_id: Option<UnitId>,
    pub sink_port: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasSuggestionAcceptance {
    MaterialConnection(CanvasSuggestedMaterialConnection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuggestionStatus {
    Proposed,
    Focused,
    Accepted,
    Rejected,
    Invalidated,
}

impl SuggestionStatus {
    fn is_terminal(self) -> bool {
        matches!(
            self,
            SuggestionStatus::Accepted | SuggestionStatus::Rejected | SuggestionStatus::Invalidated
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhostElement {
    pub kind: GhostElementKind,
    pub target_unit_id: UnitId,
    pub visual_kind: StreamVisualKind,
    pub visual_state: StreamVisualState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasSuggestion {
    pub id: CanvasSuggestionId,
    pub source: SuggestionSource,
    pub status: SuggestionStatus,
    pub confidence: f32,
    pub ghost: GhostElement,
    pub acceptance: Option<CanvasSuggestionAcceptance>,
    pub reason: String,
}

impl CanvasSuggestion {
    pub fn new(
        id: impl Into<CanvasSuggestionId>,
        source: SuggestionSource,
        confidence: f32,
        ghost: GhostElement,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source,
            status: SuggestionStatus::Proposed,
            confidence,
            ghost,
            acceptance: None,
            reason: reason.into(),
        }
    }

    pub fn with_acceptance(mut self, acceptance: CanvasSuggestionAcceptance) -> Self {
        self.acceptance = Some(acceptance);
        self
    }

    pub fn can_accept_with_tab(&self) -> bool {
        matches!(
            self.status,
            SuggestionStatus::Proposed | SuggestionStatus::Focused
        ) && self.confidence >= DEFAULT_TAB_ACCEPT_CONFIDENCE
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanvasInteractionState {
    pub view_mode: CanvasViewMode,
    pub suggestions: Vec<CanvasSuggestion>,
    pub focused_suggestion_id: Option<CanvasSuggestionId>,
}

impl CanvasInteractionState {
    pub fn set_view_mode(&mut self, view_mode: CanvasViewMode) {
        self.view_mode = view_mode;
    }

    pub fn replace_suggestions(&mut self, mut suggestions: Vec<CanvasSuggestion>) {
        suggestions.sort_by(|left, right| {
            right
                .confidence
                .total_cmp(&left.confidence)
                .then_with(|| left.id.as_str().cmp(right.id.as_str()))
        });

        self.suggestions = suggestions;
        self.focus_next_available();
    }

    pub fn focused_suggestion(&self) -> Option<&CanvasSuggestion> {
        let focused_id = self.focused_suggestion_id.as_ref()?;
        self.suggestions.iter().find(|item| &item.id == focused_id)
    }

    pub fn accept_focused_by_tab(&mut self) -> Option<CanvasSuggestion> {
        let suggestion = self.focused_suggestion()?.clone();
        if !suggestion.can_accept_with_tab() {
            return None;
        }

        self.accept_suggestion(&suggestion.id)
    }

    pub fn accept_suggestion(
        &mut self,
        suggestion_id: &CanvasSuggestionId,
    ) -> Option<CanvasSuggestion> {
        let accepted = self.update_suggestion_status(suggestion_id, SuggestionStatus::Accepted)?;
        self.focus_next_available();
        Some(accepted)
    }

    pub fn reject_focused(&mut self) -> Option<CanvasSuggestion> {
        let suggestion_id = self.focused_suggestion_id.clone()?;
        let rejected = self.update_suggestion_status(&suggestion_id, SuggestionStatus::Rejected)?;
        self.focus_next_available();
        Some(rejected)
    }

    pub fn invalidate_all(&mut self) {
        for suggestion in &mut self.suggestions {
            if !suggestion.status.is_terminal() {
                suggestion.status = SuggestionStatus::Invalidated;
            }
        }
        self.focused_suggestion_id = None;
    }

    fn focus_next_available(&mut self) {
        self.focused_suggestion_id = None;

        for suggestion in &mut self.suggestions {
            if suggestion.status.is_terminal() {
                continue;
            }

            suggestion.status = SuggestionStatus::Proposed;
        }

        if let Some(next) = self
            .suggestions
            .iter_mut()
            .find(|item| !item.status.is_terminal())
        {
            next.status = SuggestionStatus::Focused;
            self.focused_suggestion_id = Some(next.id.clone());
        }
    }

    fn update_suggestion_status(
        &mut self,
        suggestion_id: &CanvasSuggestionId,
        next_status: SuggestionStatus,
    ) -> Option<CanvasSuggestion> {
        let suggestion = self
            .suggestions
            .iter_mut()
            .find(|item| &item.id == suggestion_id)?;
        suggestion.status = next_status;
        Some(suggestion.clone())
    }
}
