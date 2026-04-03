use std::time::SystemTime;

use crate::auth::{
    AuthSessionState, AuthSessionStatus, EntitlementNotice, EntitlementState, EntitlementStatus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementPanelState {
    pub auth_status: AuthSessionStatus,
    pub entitlement_status: EntitlementStatus,
    pub authority_url: Option<String>,
    pub current_user_label: Option<String>,
    pub last_synced_at: Option<SystemTime>,
    pub offline_lease_expires_at: Option<SystemTime>,
    pub allowed_package_count: usize,
    pub package_manifest_count: usize,
    pub last_error: Option<String>,
    pub notice: Option<EntitlementNotice>,
    pub can_sync: bool,
    pub can_refresh_offline_lease: bool,
    pub commands: EntitlementCommandModel,
}

impl EntitlementPanelState {
    pub fn from_runtime(auth_session: &AuthSessionState, entitlement: &EntitlementState) -> Self {
        let can_sync = has_authenticated_session(auth_session)
            && !matches!(entitlement.status, EntitlementStatus::Syncing);
        let can_refresh_offline_lease = can_sync
            && entitlement
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.offline_lease_expires_at)
                .is_some();

        let mut state = Self {
            auth_status: auth_session.status,
            entitlement_status: entitlement.status,
            authority_url: auth_session.authority_url.clone(),
            current_user_label: current_user_label(auth_session),
            last_synced_at: entitlement.last_synced_at,
            offline_lease_expires_at: entitlement
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.offline_lease_expires_at),
            allowed_package_count: entitlement
                .snapshot
                .as_ref()
                .map(|snapshot| snapshot.allowed_package_ids.len())
                .unwrap_or(0),
            package_manifest_count: entitlement.package_manifests.len(),
            last_error: entitlement.last_error.clone(),
            notice: entitlement.notice.clone(),
            can_sync,
            can_refresh_offline_lease,
            commands: EntitlementCommandModel::default(),
        };
        state.commands = EntitlementCommandModel::from_state(&state);
        state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntitlementActionId {
    SyncEntitlement,
    RefreshOfflineLease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementActionModel {
    pub id: EntitlementActionId,
    pub label: &'static str,
    pub intent: EntitlementIntent,
    pub enabled: bool,
    pub visible: bool,
}

impl EntitlementActionModel {
    fn new(
        id: EntitlementActionId,
        label: &'static str,
        intent: EntitlementIntent,
        enabled: bool,
        visible: bool,
    ) -> Self {
        Self {
            id,
            label,
            intent,
            enabled,
            visible,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementCommandModel {
    pub primary_action: EntitlementActionId,
    pub actions: Vec<EntitlementActionModel>,
}

impl Default for EntitlementCommandModel {
    fn default() -> Self {
        Self {
            primary_action: EntitlementActionId::SyncEntitlement,
            actions: Vec::new(),
        }
    }
}

impl EntitlementCommandModel {
    pub fn from_state(state: &EntitlementPanelState) -> Self {
        Self {
            primary_action: primary_action_id(state),
            actions: vec![
                EntitlementActionModel::new(
                    EntitlementActionId::SyncEntitlement,
                    "Sync entitlement",
                    EntitlementIntent::sync_entitlement(),
                    state.can_sync,
                    true,
                ),
                EntitlementActionModel::new(
                    EntitlementActionId::RefreshOfflineLease,
                    "Refresh offline lease",
                    EntitlementIntent::refresh_offline_lease(),
                    state.can_refresh_offline_lease,
                    true,
                ),
            ],
        }
    }

    pub fn action(&self, id: EntitlementActionId) -> Option<&EntitlementActionModel> {
        self.actions.iter().find(|action| action.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntitlementIntent {
    SyncEntitlement,
    RefreshOfflineLease,
}

impl EntitlementIntent {
    pub fn sync_entitlement() -> Self {
        Self::SyncEntitlement
    }

    pub fn refresh_offline_lease() -> Self {
        Self::RefreshOfflineLease
    }
}

fn has_authenticated_session(auth_session: &AuthSessionState) -> bool {
    matches!(auth_session.status, AuthSessionStatus::Authenticated)
        && auth_session.authority_url.is_some()
        && auth_session.current_user.is_some()
        && auth_session.token_lease.is_some()
}

fn current_user_label(auth_session: &AuthSessionState) -> Option<String> {
    auth_session.current_user.as_ref().map(|user| {
        user.display_name
            .clone()
            .unwrap_or_else(|| user.preferred_username.clone())
    })
}

fn primary_action_id(state: &EntitlementPanelState) -> EntitlementActionId {
    if state.can_refresh_offline_lease
        && matches!(
            state.entitlement_status,
            EntitlementStatus::Active | EntitlementStatus::LeaseExpired
        )
    {
        EntitlementActionId::RefreshOfflineLease
    } else {
        EntitlementActionId::SyncEntitlement
    }
}
