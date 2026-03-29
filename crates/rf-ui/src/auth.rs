use std::collections::{BTreeMap, BTreeSet};
use std::time::SystemTime;

use rf_types::ComponentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthSessionStatus {
    SignedOut,
    PendingBrowserLogin,
    ExchangingCode,
    Authenticated,
    Refreshing,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureCredentialHandle {
    pub service: String,
    pub account: String,
}

impl SecureCredentialHandle {
    pub fn new(service: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            account: account.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub subject_id: String,
    pub tenant_id: Option<String>,
    pub preferred_username: String,
    pub display_name: Option<String>,
}

impl AuthenticatedUser {
    pub fn new(subject_id: impl Into<String>, preferred_username: impl Into<String>) -> Self {
        Self {
            subject_id: subject_id.into(),
            tenant_id: None,
            preferred_username: preferred_username.into(),
            display_name: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenLease {
    pub scopes: BTreeSet<String>,
    pub access_token_expires_at: SystemTime,
    pub refresh_token_expires_at: Option<SystemTime>,
    pub credential_handle: SecureCredentialHandle,
}

impl TokenLease {
    pub fn new(
        access_token_expires_at: SystemTime,
        credential_handle: SecureCredentialHandle,
    ) -> Self {
        Self {
            scopes: BTreeSet::new(),
            access_token_expires_at,
            refresh_token_expires_at: None,
            credential_handle,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSessionState {
    pub status: AuthSessionStatus,
    pub authority_url: Option<String>,
    pub current_user: Option<AuthenticatedUser>,
    pub token_lease: Option<TokenLease>,
    pub last_authenticated_at: Option<SystemTime>,
    pub last_error: Option<String>,
}

impl Default for AuthSessionState {
    fn default() -> Self {
        Self {
            status: AuthSessionStatus::SignedOut,
            authority_url: None,
            current_user: None,
            token_lease: None,
            last_authenticated_at: None,
            last_error: None,
        }
    }
}

impl AuthSessionState {
    pub fn begin_browser_login(&mut self, authority_url: impl Into<String>) {
        self.status = AuthSessionStatus::PendingBrowserLogin;
        self.authority_url = Some(authority_url.into());
        self.last_error = None;
    }

    pub fn begin_code_exchange(&mut self) {
        self.status = AuthSessionStatus::ExchangingCode;
        self.last_error = None;
    }

    pub fn complete_login(
        &mut self,
        authority_url: impl Into<String>,
        user: AuthenticatedUser,
        token_lease: TokenLease,
        authenticated_at: SystemTime,
    ) {
        self.status = AuthSessionStatus::Authenticated;
        self.authority_url = Some(authority_url.into());
        self.current_user = Some(user);
        self.token_lease = Some(token_lease);
        self.last_authenticated_at = Some(authenticated_at);
        self.last_error = None;
    }

    pub fn begin_refresh(&mut self) {
        self.status = AuthSessionStatus::Refreshing;
        self.last_error = None;
    }

    pub fn record_error(&mut self, message: impl Into<String>) {
        self.status = AuthSessionStatus::Error;
        self.last_error = Some(message.into());
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntitlementStatus {
    Unknown,
    Syncing,
    Active,
    LeaseExpired,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyPackageSource {
    LocalBundled,
    RemoteDerivedPackage,
    RemoteEvaluationService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyPackageClassification {
    Derived,
    RemoteOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageManifest {
    pub package_id: String,
    pub version: String,
    pub classification: PropertyPackageClassification,
    pub source: PropertyPackageSource,
    pub hash: String,
    pub size_bytes: u64,
    pub component_ids: Vec<ComponentId>,
    pub expires_at: Option<SystemTime>,
}

impl PropertyPackageManifest {
    pub fn new(
        package_id: impl Into<String>,
        version: impl Into<String>,
        source: PropertyPackageSource,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            version: version.into(),
            classification: PropertyPackageClassification::Derived,
            source,
            hash: String::new(),
            size_bytes: 0,
            component_ids: Vec::new(),
            expires_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSnapshot {
    pub subject_id: String,
    pub tenant_id: Option<String>,
    pub issued_at: SystemTime,
    pub expires_at: SystemTime,
    pub offline_lease_expires_at: Option<SystemTime>,
    pub features: BTreeSet<String>,
    pub allowed_package_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementState {
    pub status: EntitlementStatus,
    pub snapshot: Option<EntitlementSnapshot>,
    pub package_manifests: BTreeMap<String, PropertyPackageManifest>,
    pub last_synced_at: Option<SystemTime>,
    pub last_error: Option<String>,
}

impl Default for EntitlementState {
    fn default() -> Self {
        Self {
            status: EntitlementStatus::Unknown,
            snapshot: None,
            package_manifests: BTreeMap::new(),
            last_synced_at: None,
            last_error: None,
        }
    }
}

impl EntitlementState {
    pub fn begin_sync(&mut self) {
        self.status = EntitlementStatus::Syncing;
        self.last_error = None;
    }

    pub fn update(
        &mut self,
        snapshot: EntitlementSnapshot,
        manifests: Vec<PropertyPackageManifest>,
        synced_at: SystemTime,
    ) {
        self.package_manifests = manifests
            .into_iter()
            .map(|manifest| (manifest.package_id.clone(), manifest))
            .collect();
        self.snapshot = Some(snapshot);
        self.status = EntitlementStatus::Active;
        self.last_synced_at = Some(synced_at);
        self.last_error = None;
    }

    pub fn mark_lease_expired(&mut self, message: impl Into<String>) {
        self.status = EntitlementStatus::LeaseExpired;
        self.last_error = Some(message.into());
    }

    pub fn record_error(&mut self, message: impl Into<String>) {
        self.status = EntitlementStatus::Error;
        self.last_error = Some(message.into());
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_package_allowed(&self, package_id: &str) -> bool {
        self.snapshot
            .as_ref()
            .map(|snapshot| snapshot.allowed_package_ids.contains(package_id))
            .unwrap_or(false)
    }
}
