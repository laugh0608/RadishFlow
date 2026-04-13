use crate::run_panel::{RunPanelActionId, RunPanelIntent, RunPanelRecoveryAction, RunPanelState};
use crate::run_panel_presenter::RunPanelPresentation;
use crate::run_panel_text::RunPanelTextView;
use crate::run_panel_view::{RunPanelRenderableAction, RunPanelViewModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunPanelWidgetEvent {
    Dispatched {
        action_id: RunPanelActionId,
        intent: RunPanelIntent,
    },
    Disabled {
        action_id: RunPanelActionId,
        detail: &'static str,
    },
    Missing {
        action_id: RunPanelActionId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunPanelRecoveryWidgetEvent {
    Requested { action: RunPanelRecoveryAction },
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelWidgetModel {
    pub presentation: RunPanelPresentation,
}

impl RunPanelWidgetModel {
    pub fn from_state(state: &RunPanelState) -> Self {
        Self {
            presentation: RunPanelPresentation::from_state(state),
        }
    }

    pub fn view(&self) -> &RunPanelViewModel {
        &self.presentation.view
    }

    pub fn text(&self) -> &RunPanelTextView {
        &self.presentation.text
    }

    pub fn primary_action(&self) -> &RunPanelRenderableAction {
        &self.presentation.view.primary_action
    }

    pub fn action(&self, id: RunPanelActionId) -> Option<&RunPanelRenderableAction> {
        self.presentation.view.action(id)
    }

    pub fn recovery_action(&self) -> Option<&RunPanelRecoveryAction> {
        self.presentation
            .view
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
    }

    pub fn activate_primary(&self) -> RunPanelWidgetEvent {
        self.activate(self.primary_action().id)
    }

    pub fn activate(&self, id: RunPanelActionId) -> RunPanelWidgetEvent {
        match self.action(id) {
            Some(action) => match action.dispatchable_intent() {
                Some(intent) => RunPanelWidgetEvent::Dispatched {
                    action_id: id,
                    intent,
                },
                None => RunPanelWidgetEvent::Disabled {
                    action_id: id,
                    detail: action.detail,
                },
            },
            None => RunPanelWidgetEvent::Missing { action_id: id },
        }
    }

    pub fn activate_recovery_action(&self) -> RunPanelRecoveryWidgetEvent {
        match self.recovery_action() {
            Some(action) => RunPanelRecoveryWidgetEvent::Requested {
                action: action.clone(),
            },
            None => RunPanelRecoveryWidgetEvent::Missing,
        }
    }
}
