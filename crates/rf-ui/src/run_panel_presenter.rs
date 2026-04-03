use crate::run_panel::{RunPanelActionId, RunPanelIntent, RunPanelState};
use crate::run_panel_text::RunPanelTextView;
use crate::run_panel_view::RunPanelViewModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelPresentation {
    pub view: RunPanelViewModel,
    pub text: RunPanelTextView,
}

impl RunPanelPresentation {
    pub fn from_state(state: &RunPanelState) -> Self {
        let view = RunPanelViewModel::from_state(state);
        let text = RunPanelTextView::from_view_model(&view);
        Self { view, text }
    }

    pub fn dispatchable_primary_intent(&self) -> Option<RunPanelIntent> {
        self.view.dispatchable_primary_intent()
    }

    pub fn dispatchable_intent(&self, id: RunPanelActionId) -> Option<RunPanelIntent> {
        self.view.dispatchable_intent(id)
    }
}
