use std::time::SystemTime;

use crate::auth::{AuthSessionStatus, EntitlementNotice, EntitlementStatus};
use crate::entitlement_panel::{EntitlementActionId, EntitlementIntent, EntitlementPanelState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitlementActionProminence {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementRenderableAction {
    pub id: EntitlementActionId,
    pub label: &'static str,
    pub detail: &'static str,
    pub intent: EntitlementIntent,
    pub enabled: bool,
    pub prominence: EntitlementActionProminence,
}

impl EntitlementRenderableAction {
    pub fn dispatchable_intent(&self) -> Option<EntitlementIntent> {
        self.enabled.then(|| self.intent.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPanelViewModel {
    pub auth_label: &'static str,
    pub entitlement_label: &'static str,
    pub authority_url: Option<String>,
    pub current_user_label: Option<String>,
    pub last_synced_at: Option<SystemTime>,
    pub offline_lease_expires_at: Option<SystemTime>,
    pub allowed_package_count: usize,
    pub package_manifest_count: usize,
    pub last_error: Option<String>,
    pub notice: Option<EntitlementNotice>,
    pub primary_action: EntitlementRenderableAction,
    pub secondary_actions: Vec<EntitlementRenderableAction>,
}

impl EntitlementPanelViewModel {
    pub fn from_state(state: &EntitlementPanelState) -> Self {
        let visible_actions = state
            .commands
            .actions
            .iter()
            .filter(|action| action.visible)
            .collect::<Vec<_>>();
        let primary_action = visible_actions
            .iter()
            .find(|action| action.id == state.commands.primary_action)
            .copied()
            .or_else(|| visible_actions.first().copied())
            .expect("entitlement command model must expose at least one visible action");

        Self {
            auth_label: auth_status_label(state.auth_status),
            entitlement_label: entitlement_status_label(state.entitlement_status),
            authority_url: state.authority_url.clone(),
            current_user_label: state.current_user_label.clone(),
            last_synced_at: state.last_synced_at,
            offline_lease_expires_at: state.offline_lease_expires_at,
            allowed_package_count: state.allowed_package_count,
            package_manifest_count: state.package_manifest_count,
            last_error: state.last_error.clone(),
            notice: state.notice.clone(),
            primary_action: EntitlementRenderableAction {
                id: primary_action.id,
                label: primary_action.label,
                detail: primary_action.detail,
                intent: primary_action.intent.clone(),
                enabled: primary_action.enabled,
                prominence: EntitlementActionProminence::Primary,
            },
            secondary_actions: visible_actions
                .into_iter()
                .filter(|action| action.id != primary_action.id)
                .map(|action| EntitlementRenderableAction {
                    id: action.id,
                    label: action.label,
                    detail: action.detail,
                    intent: action.intent.clone(),
                    enabled: action.enabled,
                    prominence: EntitlementActionProminence::Secondary,
                })
                .collect(),
        }
    }

    pub fn action(&self, id: EntitlementActionId) -> Option<&EntitlementRenderableAction> {
        if self.primary_action.id == id {
            return Some(&self.primary_action);
        }

        self.secondary_actions.iter().find(|action| action.id == id)
    }

    pub fn dispatchable_intent(&self, id: EntitlementActionId) -> Option<EntitlementIntent> {
        self.action(id)
            .and_then(EntitlementRenderableAction::dispatchable_intent)
    }

    pub fn dispatchable_primary_intent(&self) -> Option<EntitlementIntent> {
        self.primary_action.dispatchable_intent()
    }
}

fn auth_status_label(status: AuthSessionStatus) -> &'static str {
    match status {
        AuthSessionStatus::SignedOut => "Signed out",
        AuthSessionStatus::PendingBrowserLogin => "Browser login pending",
        AuthSessionStatus::ExchangingCode => "Exchanging code",
        AuthSessionStatus::Authenticated => "Authenticated",
        AuthSessionStatus::Refreshing => "Refreshing",
        AuthSessionStatus::Error => "Error",
    }
}

fn entitlement_status_label(status: EntitlementStatus) -> &'static str {
    match status {
        EntitlementStatus::Unknown => "Unknown",
        EntitlementStatus::Syncing => "Syncing",
        EntitlementStatus::Active => "Active",
        EntitlementStatus::LeaseExpired => "Lease expired",
        EntitlementStatus::Error => "Error",
    }
}
