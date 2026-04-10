use crate::entitlement_panel::{EntitlementActionId, EntitlementIntent, EntitlementPanelState};
use crate::entitlement_panel_presenter::EntitlementPanelPresentation;
use crate::entitlement_panel_text::EntitlementPanelTextView;
use crate::entitlement_panel_view::{EntitlementPanelViewModel, EntitlementRenderableAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementPanelWidgetEvent {
    Dispatched {
        action_id: EntitlementActionId,
        intent: EntitlementIntent,
    },
    Disabled {
        action_id: EntitlementActionId,
        detail: &'static str,
    },
    Missing {
        action_id: EntitlementActionId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPanelWidgetModel {
    pub presentation: EntitlementPanelPresentation,
}

impl EntitlementPanelWidgetModel {
    pub fn from_state(state: &EntitlementPanelState) -> Self {
        Self {
            presentation: EntitlementPanelPresentation::from_state(state),
        }
    }

    pub fn view(&self) -> &EntitlementPanelViewModel {
        &self.presentation.view
    }

    pub fn text(&self) -> &EntitlementPanelTextView {
        &self.presentation.text
    }

    pub fn primary_action(&self) -> &EntitlementRenderableAction {
        &self.presentation.view.primary_action
    }

    pub fn action(&self, id: EntitlementActionId) -> Option<&EntitlementRenderableAction> {
        self.presentation.view.action(id)
    }

    pub fn activate_primary(&self) -> EntitlementPanelWidgetEvent {
        self.activate(self.primary_action().id)
    }

    pub fn activate(&self, id: EntitlementActionId) -> EntitlementPanelWidgetEvent {
        match self.action(id) {
            Some(action) => match action.dispatchable_intent() {
                Some(intent) => EntitlementPanelWidgetEvent::Dispatched {
                    action_id: id,
                    intent,
                },
                None => EntitlementPanelWidgetEvent::Disabled {
                    action_id: id,
                    detail: action.detail,
                },
            },
            None => EntitlementPanelWidgetEvent::Missing { action_id: id },
        }
    }
}
