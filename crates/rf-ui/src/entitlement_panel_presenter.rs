use crate::entitlement_panel::{EntitlementActionId, EntitlementIntent, EntitlementPanelState};
use crate::entitlement_panel_text::EntitlementPanelTextView;
use crate::entitlement_panel_view::EntitlementPanelViewModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPanelPresentation {
    pub view: EntitlementPanelViewModel,
    pub text: EntitlementPanelTextView,
}

impl EntitlementPanelPresentation {
    pub fn from_state(state: &EntitlementPanelState) -> Self {
        let view = EntitlementPanelViewModel::from_state(state);
        let text = EntitlementPanelTextView::from_view_model(&view);
        Self { view, text }
    }

    pub fn dispatchable_primary_intent(&self) -> Option<EntitlementIntent> {
        self.view.dispatchable_primary_intent()
    }

    pub fn dispatchable_intent(&self, id: EntitlementActionId) -> Option<EntitlementIntent> {
        self.view.dispatchable_intent(id)
    }
}
