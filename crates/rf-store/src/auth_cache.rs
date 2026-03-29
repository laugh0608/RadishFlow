use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::SystemTime;

use rf_types::{RfError, RfResult};
use serde::{Deserialize, Serialize};

use crate::layout::StoredAuthCacheLayout;

pub const STORED_AUTH_CACHE_INDEX_KIND: &str = "radishflow.auth-cache-index";
pub const STORED_AUTH_CACHE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StoredPropertyPackageSource {
    LocalBundled,
    RemoteDerivedPackage,
    RemoteEvaluationService,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredEntitlementCache {
    pub subject_id: String,
    pub tenant_id: Option<String>,
    #[serde(with = "crate::json::time_format")]
    pub synced_at: SystemTime,
    #[serde(with = "crate::json::time_format")]
    pub issued_at: SystemTime,
    #[serde(with = "crate::json::time_format")]
    pub expires_at: SystemTime,
    #[serde(with = "crate::json::option_time_format")]
    pub offline_lease_expires_at: Option<SystemTime>,
    pub feature_keys: BTreeSet<String>,
    pub allowed_package_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredPropertyPackageRecord {
    pub package_id: String,
    pub version: String,
    pub source: StoredPropertyPackageSource,
    #[serde(with = "crate::json::relative_path_format")]
    pub manifest_relative_path: PathBuf,
    #[serde(with = "crate::json::option_relative_path_format")]
    pub payload_relative_path: Option<PathBuf>,
    pub hash: String,
    pub size_bytes: u64,
    #[serde(with = "crate::json::time_format")]
    pub downloaded_at: SystemTime,
    #[serde(with = "crate::json::option_time_format")]
    pub expires_at: Option<SystemTime>,
}

impl StoredPropertyPackageRecord {
    pub fn new(
        package_id: impl Into<String>,
        version: impl Into<String>,
        source: StoredPropertyPackageSource,
        hash: impl Into<String>,
        size_bytes: u64,
        downloaded_at: SystemTime,
    ) -> Self {
        let package_id = package_id.into();
        let version = version.into();
        let manifest_relative_path =
            StoredAuthCacheLayout::package_manifest_relative_path(&package_id, &version);
        let payload_relative_path = match source {
            StoredPropertyPackageSource::LocalBundled
            | StoredPropertyPackageSource::RemoteDerivedPackage => Some(
                StoredAuthCacheLayout::package_payload_relative_path(&package_id, &version),
            ),
            StoredPropertyPackageSource::RemoteEvaluationService => None,
        };

        Self {
            package_id,
            version,
            source,
            manifest_relative_path,
            payload_relative_path,
            hash: hash.into(),
            size_bytes,
            downloaded_at,
            expires_at: None,
        }
    }

    pub fn is_expired_at(&self, now: SystemTime) -> bool {
        self.expires_at
            .map(|expires_at| now >= expires_at)
            .unwrap_or(false)
    }

    pub fn validate(&self) -> RfResult<()> {
        if self.package_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored property package record must contain a non-empty package_id",
            ));
        }

        if self.version.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored property package record must contain a non-empty version",
            ));
        }

        if self.manifest_relative_path.is_absolute() {
            return Err(RfError::invalid_input(
                "stored property package manifest path must be relative",
            ));
        }

        if self
            .payload_relative_path
            .as_ref()
            .is_some_and(|path| path.is_absolute())
        {
            return Err(RfError::invalid_input(
                "stored property package payload path must be relative",
            ));
        }

        if matches!(
            self.source,
            StoredPropertyPackageSource::RemoteEvaluationService
        ) && self.payload_relative_path.is_some()
        {
            return Err(RfError::invalid_input(
                "remote evaluation packages must not declare a local payload path",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredAuthCacheIndex {
    pub kind: String,
    pub schema_version: u32,
    pub authority_url: String,
    pub subject_id: String,
    pub credential: StoredCredentialReference,
    pub entitlement: Option<StoredEntitlementCache>,
    pub property_packages: Vec<StoredPropertyPackageRecord>,
    #[serde(with = "crate::json::option_time_format")]
    pub last_synced_at: Option<SystemTime>,
}

impl StoredAuthCacheIndex {
    pub fn new(
        authority_url: impl Into<String>,
        subject_id: impl Into<String>,
        credential: StoredCredentialReference,
    ) -> Self {
        Self {
            kind: STORED_AUTH_CACHE_INDEX_KIND.to_string(),
            schema_version: STORED_AUTH_CACHE_SCHEMA_VERSION,
            authority_url: authority_url.into(),
            subject_id: subject_id.into(),
            credential,
            entitlement: None,
            property_packages: Vec::new(),
            last_synced_at: None,
        }
    }

    pub fn index_relative_path(&self) -> PathBuf {
        StoredAuthCacheLayout::index_relative_path(&self.authority_url, &self.subject_id)
    }

    pub fn validate(&self) -> RfResult<()> {
        if self.kind != STORED_AUTH_CACHE_INDEX_KIND {
            return Err(RfError::invalid_input(format!(
                "unsupported stored auth cache index kind `{}`",
                self.kind
            )));
        }

        if self.schema_version != STORED_AUTH_CACHE_SCHEMA_VERSION {
            return Err(RfError::invalid_input(format!(
                "unsupported stored auth cache index schema version `{}`",
                self.schema_version
            )));
        }

        if self.authority_url.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored auth cache index must contain a non-empty authority_url",
            ));
        }

        if self.subject_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored auth cache index must contain a non-empty subject_id",
            ));
        }

        for package in &self.property_packages {
            package.validate()?;
        }

        Ok(())
    }
}
