use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCredentialReference {
    pub service: String,
    pub account: String,
}

impl StoredCredentialReference {
    pub fn new(service: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            account: account.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoredPropertyPackageSource {
    LocalBundled,
    RemoteDerivedPackage,
    RemoteEvaluationService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEntitlementCache {
    pub subject_id: String,
    pub tenant_id: Option<String>,
    pub synced_at: SystemTime,
    pub entitlement_expires_at: SystemTime,
    pub offline_lease_expires_at: Option<SystemTime>,
    pub feature_keys: BTreeSet<String>,
    pub allowed_package_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPropertyPackageRecord {
    pub package_id: String,
    pub version: String,
    pub source: StoredPropertyPackageSource,
    pub local_path: PathBuf,
    pub hash: String,
    pub downloaded_at: SystemTime,
    pub expires_at: Option<SystemTime>,
}

impl StoredPropertyPackageRecord {
    pub fn is_expired_at(&self, now: SystemTime) -> bool {
        self.expires_at
            .map(|expires_at| now >= expires_at)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredAuthCacheIndex {
    pub schema_version: u32,
    pub authority_url: String,
    pub subject_id: String,
    pub credential: StoredCredentialReference,
    pub entitlement: Option<StoredEntitlementCache>,
    pub property_packages: Vec<StoredPropertyPackageRecord>,
    pub last_synced_at: Option<SystemTime>,
}

impl StoredAuthCacheIndex {
    pub fn new(
        authority_url: impl Into<String>,
        subject_id: impl Into<String>,
        credential: StoredCredentialReference,
    ) -> Self {
        Self {
            schema_version: 1,
            authority_url: authority_url.into(),
            subject_id: subject_id.into(),
            credential,
            entitlement: None,
            property_packages: Vec::new(),
            last_synced_at: None,
        }
    }
}
