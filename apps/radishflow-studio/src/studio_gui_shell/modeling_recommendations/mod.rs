use super::*;
use radishflow_studio::modeling_recommendations::*;
use radishflow_studio::modeling_specifications::{
    ModelingCheckPhase, SpecificationProblemKey, SpecificationTarget,
};
use rf_ui::{CanvasSuggestionAcceptance, CanvasSuggestionId};

mod panel;
mod presentation;
use presentation::*;

#[derive(Default)]
pub(super) struct RecommendationUiState {
    pub review: Option<RecommendationReview>,
    pub notice: Option<RecommendationError>,
    pub show_ignored: bool,
    pub return_tab: Option<StudioShellBottomDrawerTab>,
}

pub(super) struct RecommendationReview {
    pub preview: RecommendationPreview,
    pub outcome: Option<Result<RecommendationReceipt, RecommendationError>>,
    pub first_frame: Option<u64>,
    pub requires_regeneration: bool,
}

impl ReadyAppState {
    pub(super) fn recommendation_command_label<'a>(&self, id: &str, fallback: &'a str) -> &'a str {
        match id {
            "canvas.accept_focused" => tr(
                self.locale,
                "预览推荐 (Tab)",
                "Preview recommendation (Tab)",
            ),
            "canvas.reject_focused" => tr(
                self.locale,
                "本修订忽略 (Escape)",
                "Ignore this revision (Escape)",
            ),
            _ => fallback,
        }
    }

    pub(super) fn preview_modeling_recommendation(&mut self, key: &RecommendationKey) {
        let result = self
            .modeling_problems
            .recommendations
            .prepare(self.platform_host.modeling_inputs(), key);
        match result {
            Ok(preview) => {
                if self.modeling_problems.recommendation_ui.review.is_none() {
                    self.modeling_problems.recommendation_ui.return_tab =
                        Some(self.bottom_drawer_tab);
                }
                self.modeling_problems.recommendation_ui.review = Some(RecommendationReview {
                    preview,
                    outcome: None,
                    first_frame: None,
                    requires_regeneration: false,
                });
                self.modeling_problems.recommendation_ui.notice = None;
                self.open_modeling_problems();
            }
            Err(error) => self.modeling_problems.recommendation_ui.notice = Some(error),
        }
    }

    pub(super) fn accept_modeling_recommendation(&mut self) {
        let Some(review) = &self.modeling_problems.recommendation_ui.review else {
            return;
        };
        if matches!(review.outcome, Some(Ok(_))) {
            return;
        }
        let preview = review.preview.clone();
        let outcome = self
            .platform_host
            .accept_modeling_recommendation(&mut self.modeling_problems.recommendations, &preview);
        if let Ok(receipt) = &outcome {
            self.platform_host.record_activity_line(format!(
                "modeling recommendation {:?} committed at r{}; checks {:?}",
                receipt.key.rule, receipt.revision, receipt.check_phase,
            ));
        }
        self.modeling_problems
            .recommendation_ui
            .review
            .as_mut()
            .expect("review retained during dispatch")
            .outcome = Some(outcome);
        self.refresh_active_modeling_readiness_notice();
    }

    pub(super) fn ignore_modeling_recommendation(
        &mut self,
        key: &RecommendationKey,
        ignored: bool,
    ) {
        let result = self.modeling_problems.recommendations.set_ignored(
            self.platform_host.modeling_inputs(),
            key,
            ignored,
        );
        self.modeling_problems.recommendation_ui.notice = result.err();
        if self
            .modeling_problems
            .recommendation_ui
            .review
            .as_ref()
            .is_some_and(|r| &r.preview.recommendation().key == key)
        {
            self.modeling_problems.recommendation_ui.review = None;
        }
    }

    pub(super) fn close_recommendation_review(&mut self) {
        if let Some(tab) = self.modeling_problems.recommendation_ui.return_tab.take() {
            self.bottom_drawer_tab = tab;
        }
        self.modeling_problems.recommendation_ui.review = None;
        self.modeling_problems.recommendation_ui.notice = None;
    }

    pub(super) fn canvas_recommendation_key(&self, id: &str) -> Option<RecommendationKey> {
        let suggestion = self
            .platform_host
            .modeling_inputs()
            .workspace
            .canvas_interaction
            .suggestion(&CanvasSuggestionId::new(id))?;
        let CanvasSuggestionAcceptance::MaterialConnection(connection) =
            suggestion.acceptance.as_ref()?;
        self.modeling_problems.recommendations.entries().iter().find(|entry| {
            matches!(&entry.recommendation.action, RecommendedAction::MaterialConnection(current) if current == connection)
        }).map(|entry| entry.recommendation.key.clone())
    }

    pub(super) fn canvas_suggestion_visible(&self, id: &str) -> bool {
        let inputs = self.platform_host.modeling_inputs();
        let Some(suggestion) = inputs
            .workspace
            .canvas_interaction
            .suggestion(&CanvasSuggestionId::new(id))
        else {
            return false;
        };
        if suggestion.acceptance.is_none() {
            return true;
        }
        self.canvas_recommendation_key(id).is_some_and(|key| {
            self.modeling_problems
                .recommendations
                .entries()
                .iter()
                .any(|e| {
                    e.recommendation.key == key
                        && e.visibility == RecommendationVisibility::Available
                })
        })
    }

    /// The legacy command ID remains a UI route. Material writes always require A2 review.
    pub(super) fn intercept_recommendation_command(&mut self, command: &str) -> bool {
        match command {
            "canvas.accept_focused" => self.intercept_canvas_recommendation(None, false),
            "canvas.reject_focused" => self.intercept_canvas_recommendation(None, true),
            _ => false,
        }
    }

    pub(super) fn intercept_recommendation_event(&mut self, event: &StudioGuiEvent) -> bool {
        match event {
            StudioGuiEvent::CanvasSuggestionAcceptRequested => {
                self.intercept_canvas_recommendation(None, false)
            }
            StudioGuiEvent::CanvasSuggestionAcceptByIdRequested { suggestion_id } => {
                self.intercept_canvas_recommendation(Some(suggestion_id), false)
            }
            StudioGuiEvent::CanvasSuggestionRejectRequested => {
                self.intercept_canvas_recommendation(None, true)
            }
            StudioGuiEvent::UiCommandRequested { command_id } => {
                self.intercept_recommendation_command(command_id)
            }
            _ => false,
        }
    }

    fn intercept_canvas_recommendation(
        &mut self,
        id: Option<&CanvasSuggestionId>,
        ignore: bool,
    ) -> bool {
        self.refresh_modeling_problems();
        let canvas = &self
            .platform_host
            .modeling_inputs()
            .workspace
            .canvas_interaction;
        let suggestion = match id {
            Some(id) => canvas.suggestion(id),
            None => canvas.focused_suggestion(),
        }
        .cloned();
        let Some(suggestion) = suggestion else {
            return false;
        };
        // Non-writing legacy navigation suggestions retain their existing behavior.
        if suggestion.acceptance.is_none() {
            return false;
        }
        let Some(key) = self.canvas_recommendation_key(suggestion.id.as_str()) else {
            self.modeling_problems.recommendation_ui.notice =
                Some(RecommendationError::Unavailable);
            self.open_modeling_problems();
            return true;
        };
        if ignore {
            self.ignore_modeling_recommendation(&key, true);
        } else {
            self.select_recommendation_problem(&key);
            self.preview_modeling_recommendation(&key);
        }
        true
    }

    fn select_recommendation_problem(&mut self, key: &RecommendationKey) {
        let problem = self
            .modeling_problems
            .recommendations
            .entries()
            .iter()
            .find(|e| &e.recommendation.key == key)
            .and_then(|e| e.recommendation.related_problems.first())
            .cloned();
        self.modeling_problems.select_problem(problem);
    }

    pub(super) fn recommendation_preview_shortcuts(&mut self, ctx: &egui::Context) -> bool {
        if self.modeling_problems.recommendation_ui.review.is_some()
            && !self.modeling_problems.is_field_page()
            && self.bottom_drawer_tab == StudioShellBottomDrawerTab::Diagnostics
            && !numeric_input::owns_keyboard(ctx)
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            self.close_recommendation_review();
            return true;
        }
        false
    }

    pub(super) fn project_canvas_recommendations(&self, window: &mut StudioGuiWindowModel) {
        let view = &mut window.canvas.widget.presentation.view;
        view.suggestions
            .retain(|s| self.canvas_suggestion_visible(&s.id));
        for suggestion in &mut view.suggestions {
            suggestion.action_label = "Preview recommendation";
        }
        view.suggestion_count = view.suggestions.len();
        if !view
            .suggestions
            .iter()
            .any(|s| Some(&s.id) == view.focused_suggestion_id.as_ref())
        {
            view.focused_suggestion_id = None;
        }
        view.legend
            .items
            .retain(|item| item.kind_label != "Suggestion");
        if let Some(focused) = view.suggestions.iter().find(|s| s.is_focused) {
            view.legend
                .items
                .push(radishflow_studio::StudioGuiCanvasLegendItemViewModel {
                    kind_label: "Suggestion",
                    label: "Preview recommendation".into(),
                    detail: focused.reason.clone(),
                    swatch_label: "suggestion",
                });
        }
        for section in &mut window.flowsheet_context_toolbar.sections {
            for item in &mut section.items {
                if matches!(
                    item.command_id.as_deref(),
                    Some("canvas.accept_focused" | "canvas.reject_focused")
                ) {
                    item.enabled &= view.focused_suggestion_id.is_some();
                }
            }
        }
        window.canvas.suggestion_count = view.suggestion_count;
        window.canvas.focused_suggestion_id = view.focused_suggestion_id.clone();
        for action in &mut window.canvas.widget.actions {
            match action.id {
                radishflow_studio::StudioGuiCanvasActionId::AcceptFocused => {
                    action.label = "Preview recommendation".into()
                }
                radishflow_studio::StudioGuiCanvasActionId::RejectFocused => {
                    action.label = "Ignore this revision".into()
                }
                _ => {}
            }
            if matches!(
                action.id,
                radishflow_studio::StudioGuiCanvasActionId::AcceptFocused
                    | radishflow_studio::StudioGuiCanvasActionId::RejectFocused
            ) && view.focused_suggestion_id.is_none()
            {
                action.enabled = false;
            }
        }
    }
}

fn consume_recommendation_activation(ui: &mut egui::Ui) {
    ui.input_mut(|i| {
        i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
        i.consume_key(egui::Modifiers::NONE, egui::Key::Space);
        i.consume_key(egui::Modifiers::NONE, egui::Key::Tab);
    });
}
