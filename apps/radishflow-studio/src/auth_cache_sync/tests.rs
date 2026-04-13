use std::cell::Cell;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rf_store::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
    StoredPropertyPackagePayload, StoredPropertyPackageRecord, StoredPropertyPackageSource,
    StoredThermoComponent, property_package_payload_integrity, read_auth_cache_index,
    read_property_package_manifest, read_property_package_payload, write_auth_cache_index,
    write_property_package_manifest, write_property_package_payload,
};
use rf_types::{RfError, RfResult};
use rf_ui::{
    AuthSessionState, AuthenticatedUser, EntitlementSnapshot, EntitlementState,
    OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant, PropertyPackageManifest,
    PropertyPackageManifestList, PropertyPackageSource, SecureCredentialHandle, TokenLease,
};

use super::{CacheFileStore, StdCacheFileStore, persist_downloaded_package_to_cache_with_store};
use crate::{
    apply_offline_refresh_to_auth_cache, build_auth_cache_index, build_offline_refresh_request,
    persist_downloaded_package_to_cache, record_downloaded_package, sync_auth_cache_index,
};

fn timestamp(seconds: u64) -> std::time::SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn sample_auth_session(subject_id: &str) -> AuthSessionState {
    let mut auth_session = AuthSessionState::default();
    auth_session.complete_login(
        "https://id.radish.local",
        AuthenticatedUser::new(subject_id, "user@radish.local"),
        TokenLease::new(
            timestamp(400),
            SecureCredentialHandle::new("radishflow-studio", "user-credential"),
        ),
        timestamp(120),
    );
    auth_session
}

fn sample_entitlement_state(
    subject_id: &str,
    allowed_package_ids: impl IntoIterator<Item = &'static str>,
    synced_at: u64,
    offline_lease_expires_at: Option<u64>,
) -> EntitlementState {
    let snapshot = EntitlementSnapshot {
        schema_version: 1,
        subject_id: subject_id.to_string(),
        tenant_id: Some("tenant-1".to_string()),
        issued_at: timestamp(100),
        expires_at: timestamp(500),
        offline_lease_expires_at: offline_lease_expires_at.map(timestamp),
        features: BTreeSet::from([
            "desktop-login".to_string(),
            "local-thermo-packages".to_string(),
        ]),
        allowed_package_ids: allowed_package_ids
            .into_iter()
            .map(str::to_string)
            .collect(),
    };

    let mut state = EntitlementState::default();
    state.update(snapshot, Vec::new(), timestamp(synced_at));
    state
}

#[test]
fn build_auth_cache_index_requires_consistent_authenticated_state() {
    let auth_session = sample_auth_session("user-123");
    let mut entitlement = sample_entitlement_state("user-123", ["pkg-1"], 140, Some(700));
    entitlement.package_manifests.insert(
        "pkg-1".to_string(),
        PropertyPackageManifest::new(
            "pkg-1",
            "2026.03.1",
            PropertyPackageSource::RemoteDerivedPackage,
        ),
    );

    let index =
        build_auth_cache_index(&auth_session, &entitlement).expect("expected auth cache index");

    assert_eq!(index.authority_url, "https://id.radish.local");
    assert_eq!(index.subject_id, "user-123");
    assert_eq!(index.credential.service, "radishflow-studio");
    assert_eq!(index.last_synced_at, Some(timestamp(140)));
    assert!(index.property_packages.is_empty());
}

#[test]
fn sync_auth_cache_index_prunes_disallowed_packages_and_refreshes_expiration() {
    let auth_session = sample_auth_session("user-123");
    let entitlement = sample_entitlement_state("user-123", ["pkg-1"], 150, Some(900));
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    let mut kept = StoredPropertyPackageRecord::new(
        "pkg-1",
        "2026.03.1",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        "sha256:pkg-1",
        1024,
        timestamp(120),
    );
    kept.expires_at = Some(timestamp(300));
    index.property_packages.push(kept);
    index
        .property_packages
        .push(StoredPropertyPackageRecord::new(
            "pkg-2",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:pkg-2",
            2048,
            timestamp(121),
        ));

    sync_auth_cache_index(&mut index, &auth_session, &entitlement).expect("expected sync");

    assert_eq!(index.property_packages.len(), 1);
    assert_eq!(index.property_packages[0].package_id, "pkg-1");
    assert_eq!(index.property_packages[0].expires_at, Some(timestamp(900)));
}

#[test]
fn build_offline_refresh_request_uses_cached_package_ids() {
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    index.entitlement = Some(StoredEntitlementCache {
        subject_id: "user-123".to_string(),
        tenant_id: Some("tenant-1".to_string()),
        synced_at: timestamp(100),
        issued_at: timestamp(90),
        expires_at: timestamp(500),
        offline_lease_expires_at: Some(timestamp(700)),
        feature_keys: BTreeSet::from(["desktop-login".to_string()]),
        allowed_package_ids: BTreeSet::from(["pkg-1".to_string(), "pkg-2".to_string()]),
    });
    index
        .property_packages
        .push(StoredPropertyPackageRecord::new(
            "pkg-1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:pkg-1",
            1024,
            timestamp(110),
        ));
    index
        .property_packages
        .push(StoredPropertyPackageRecord::new(
            "pkg-2",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:pkg-2",
            2048,
            timestamp(111),
        ));

    let request = build_offline_refresh_request(&index).expect("expected offline refresh request");

    assert_eq!(
        request.package_ids,
        BTreeSet::from(["pkg-1".to_string(), "pkg-2".to_string()])
    );
    assert_eq!(
        request.current_offline_lease_expires_at,
        Some(timestamp(700))
    );
}

#[test]
fn record_downloaded_package_uses_entitlement_expiration_not_download_url_expiration() {
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    index.entitlement = Some(StoredEntitlementCache {
        subject_id: "user-123".to_string(),
        tenant_id: Some("tenant-1".to_string()),
        synced_at: timestamp(100),
        issued_at: timestamp(90),
        expires_at: timestamp(500),
        offline_lease_expires_at: Some(timestamp(900)),
        feature_keys: BTreeSet::from(["desktop-login".to_string()]),
        allowed_package_ids: BTreeSet::from(["pkg-1".to_string()]),
    });
    index
        .property_packages
        .push(StoredPropertyPackageRecord::new(
            "pkg-1",
            "2026.03.0",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:old",
            512,
            timestamp(80),
        ));

    let mut manifest = PropertyPackageManifest::new(
        "pkg-1",
        "2026.03.1",
        PropertyPackageSource::RemoteDerivedPackage,
    );
    manifest.hash = "sha256:new".to_string();
    manifest.size_bytes = 1024;
    let lease_grant = PropertyPackageLeaseGrant {
        package_id: "pkg-1".to_string(),
        version: "2026.03.1".to_string(),
        lease_id: "lease-1".to_string(),
        download_url: "https://assets.radish.local/lease-1".to_string(),
        hash: "sha256:new".to_string(),
        size_bytes: 1024,
        expires_at: timestamp(210),
    };

    record_downloaded_package(&mut index, &manifest, &lease_grant, timestamp(200))
        .expect("expected downloaded package record");

    assert_eq!(index.property_packages.len(), 1);
    assert_eq!(index.property_packages[0].version, "2026.03.1");
    assert_eq!(index.property_packages[0].expires_at, Some(timestamp(900)));
}

#[test]
fn apply_offline_refresh_prunes_stale_cached_packages() {
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    index.entitlement = Some(StoredEntitlementCache {
        subject_id: "user-123".to_string(),
        tenant_id: Some("tenant-1".to_string()),
        synced_at: timestamp(100),
        issued_at: timestamp(90),
        expires_at: timestamp(500),
        offline_lease_expires_at: Some(timestamp(700)),
        feature_keys: BTreeSet::from(["desktop-login".to_string()]),
        allowed_package_ids: BTreeSet::from(["pkg-1".to_string(), "pkg-2".to_string()]),
    });
    index
        .property_packages
        .push(StoredPropertyPackageRecord::new(
            "pkg-1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:pkg-1",
            1024,
            timestamp(110),
        ));
    index
        .property_packages
        .push(StoredPropertyPackageRecord::new(
            "pkg-2",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:stale",
            2048,
            timestamp(111),
        ));

    let snapshot = EntitlementSnapshot {
        schema_version: 1,
        subject_id: "user-123".to_string(),
        tenant_id: Some("tenant-1".to_string()),
        issued_at: timestamp(200),
        expires_at: timestamp(600),
        offline_lease_expires_at: Some(timestamp(950)),
        features: BTreeSet::from([
            "desktop-login".to_string(),
            "local-thermo-packages".to_string(),
        ]),
        allowed_package_ids: BTreeSet::from(["pkg-1".to_string()]),
    };
    let mut manifest = PropertyPackageManifest::new(
        "pkg-1",
        "2026.03.1",
        PropertyPackageSource::RemoteDerivedPackage,
    );
    manifest.hash = "sha256:pkg-1".to_string();
    manifest.size_bytes = 1024;
    let response = OfflineLeaseRefreshResponse {
        refreshed_at: timestamp(210),
        snapshot,
        manifest_list: PropertyPackageManifestList::new(timestamp(205), vec![manifest]),
    };

    apply_offline_refresh_to_auth_cache(&mut index, &response).expect("expected offline refresh");

    assert_eq!(index.property_packages.len(), 1);
    assert_eq!(index.property_packages[0].package_id, "pkg-1");
    assert_eq!(index.property_packages[0].expires_at, Some(timestamp(950)));
    assert_eq!(index.last_synced_at, Some(timestamp(210)));
}

#[test]
fn persist_downloaded_package_to_cache_writes_assets_and_index_under_cache_root() {
    let root = unique_temp_path("downloaded-package-cache");
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    index.entitlement = Some(StoredEntitlementCache {
        subject_id: "user-123".to_string(),
        tenant_id: Some("tenant-1".to_string()),
        synced_at: timestamp(100),
        issued_at: timestamp(90),
        expires_at: timestamp(500),
        offline_lease_expires_at: Some(timestamp(900)),
        feature_keys: BTreeSet::from(["desktop-login".to_string()]),
        allowed_package_ids: BTreeSet::from(["pkg-1".to_string()]),
    });
    let mut manifest = PropertyPackageManifest::new(
        "pkg-1",
        "2026.03.1",
        PropertyPackageSource::RemoteDerivedPackage,
    );
    let payload = StoredPropertyPackagePayload::new(
        "pkg-1",
        "2026.03.1",
        vec![StoredThermoComponent::new(
            rf_types::ComponentId::new("methane"),
            "Methane",
        )],
    );
    let integrity =
        property_package_payload_integrity(&payload).expect("expected payload integrity");
    manifest.hash = integrity.hash.clone();
    manifest.size_bytes = integrity.size_bytes;
    manifest.component_ids = vec![rf_types::ComponentId::new("methane")];
    let lease_grant = PropertyPackageLeaseGrant {
        package_id: "pkg-1".to_string(),
        version: "2026.03.1".to_string(),
        lease_id: "lease-1".to_string(),
        download_url: "https://assets.radish.local/lease-1".to_string(),
        hash: integrity.hash.clone(),
        size_bytes: integrity.size_bytes,
        expires_at: timestamp(210),
    };

    persist_downloaded_package_to_cache(
        &root,
        &mut index,
        &manifest,
        &lease_grant,
        &payload,
        timestamp(200),
    )
    .expect("expected downloaded package persistence");

    let cached_record = &index.property_packages[0];
    let stored_manifest = read_property_package_manifest(cached_record.manifest_path_under(&root))
        .expect("expected stored manifest read");
    let stored_payload = read_property_package_payload(
        cached_record
            .payload_path_under(&root)
            .expect("expected payload path"),
    )
    .expect("expected stored payload read");
    let stored_index = read_auth_cache_index(index.index_path_under(&root))
        .expect("expected stored auth cache index");

    assert_eq!(stored_manifest.package_id, "pkg-1");
    assert_eq!(stored_manifest.hash, integrity.hash);
    assert_eq!(stored_manifest.expires_at, Some(timestamp(900)));
    assert_eq!(stored_payload.package_id, "pkg-1");
    assert_eq!(stored_index.property_packages.len(), 1);
    assert_eq!(
        stored_index.property_packages[0].downloaded_at,
        timestamp(200)
    );

    fs::remove_dir_all(&root).expect("expected temp dir cleanup");
}

#[test]
fn persist_downloaded_package_to_cache_restores_previous_files_when_manifest_write_fails() {
    let root = unique_temp_path("downloaded-package-rollback");
    let mut index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "user-123",
        StoredCredentialReference::new("radishflow-studio", "user-credential"),
    );
    index.entitlement = Some(StoredEntitlementCache {
        subject_id: "user-123".to_string(),
        tenant_id: Some("tenant-1".to_string()),
        synced_at: timestamp(100),
        issued_at: timestamp(90),
        expires_at: timestamp(500),
        offline_lease_expires_at: Some(timestamp(900)),
        feature_keys: BTreeSet::from(["desktop-login".to_string()]),
        allowed_package_ids: BTreeSet::from(["pkg-1".to_string()]),
    });

    let existing_payload = StoredPropertyPackagePayload::new(
        "pkg-1",
        "2026.03.1",
        vec![StoredThermoComponent::new(
            rf_types::ComponentId::new("methane"),
            "Methane Legacy",
        )],
    );
    let existing_integrity = property_package_payload_integrity(&existing_payload)
        .expect("expected existing payload integrity");
    let existing_record = sample_cached_record(
        "pkg-1",
        "2026.03.1",
        &existing_integrity.hash,
        existing_integrity.size_bytes,
        timestamp(120),
        timestamp(900),
    );
    let existing_manifest = sample_stored_manifest(
        "pkg-1",
        "2026.03.1",
        &existing_integrity.hash,
        existing_integrity.size_bytes,
        timestamp(900),
    );
    index.property_packages.push(existing_record.clone());
    write_property_package_payload(
        existing_record
            .payload_path_under(&root)
            .expect("expected existing payload path"),
        &existing_payload,
    )
    .expect("expected existing payload write");
    write_property_package_manifest(
        existing_record.manifest_path_under(&root),
        &existing_manifest,
    )
    .expect("expected existing manifest write");
    write_auth_cache_index(index.index_path_under(&root), &index)
        .expect("expected existing auth cache write");

    let updated_payload = StoredPropertyPackagePayload::new(
        "pkg-1",
        "2026.03.1",
        vec![StoredThermoComponent::new(
            rf_types::ComponentId::new("methane"),
            "Methane Updated",
        )],
    );
    let updated_integrity = property_package_payload_integrity(&updated_payload)
        .expect("expected updated payload integrity");
    let mut manifest = PropertyPackageManifest::new(
        "pkg-1",
        "2026.03.1",
        PropertyPackageSource::RemoteDerivedPackage,
    );
    manifest.hash = updated_integrity.hash.clone();
    manifest.size_bytes = updated_integrity.size_bytes;
    manifest.component_ids = vec![rf_types::ComponentId::new("methane")];
    let lease_grant = PropertyPackageLeaseGrant {
        package_id: "pkg-1".to_string(),
        version: "2026.03.1".to_string(),
        lease_id: "lease-1".to_string(),
        download_url: "https://assets.radish.local/lease-1".to_string(),
        hash: updated_integrity.hash,
        size_bytes: updated_integrity.size_bytes,
        expires_at: timestamp(210),
    };
    let store = FailingCacheFileStore::new(2);

    let error = persist_downloaded_package_to_cache_with_store(
        &root,
        &mut index,
        &manifest,
        &lease_grant,
        &updated_payload,
        timestamp(200),
        &store,
    )
    .expect_err("expected manifest write failure");

    assert_eq!(error.code().as_str(), "invalid_input");
    assert!(error.message().contains("simulated write failure"));

    let restored_payload = read_property_package_payload(
        existing_record
            .payload_path_under(&root)
            .expect("expected restored payload path"),
    )
    .expect("expected restored payload read");
    let restored_manifest =
        read_property_package_manifest(existing_record.manifest_path_under(&root))
            .expect("expected restored manifest read");
    let restored_index =
        read_auth_cache_index(index.index_path_under(&root)).expect("expected restored index read");

    assert_eq!(restored_payload, existing_payload);
    assert_eq!(restored_manifest, existing_manifest);
    assert_eq!(
        restored_index.property_packages[0].hash,
        existing_integrity.hash
    );
    assert_eq!(index.property_packages[0].hash, existing_integrity.hash);

    fs::remove_dir_all(&root).expect("expected temp dir cleanup");
}

fn unique_temp_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("expected time after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
}

fn sample_cached_record(
    package_id: &str,
    version: &str,
    hash: &str,
    size_bytes: u64,
    downloaded_at: SystemTime,
    expires_at: SystemTime,
) -> StoredPropertyPackageRecord {
    let mut record = StoredPropertyPackageRecord::new(
        package_id,
        version,
        StoredPropertyPackageSource::RemoteDerivedPackage,
        hash,
        size_bytes,
        downloaded_at,
    );
    record.expires_at = Some(expires_at);
    record
}

fn sample_stored_manifest(
    package_id: &str,
    version: &str,
    hash: &str,
    size_bytes: u64,
    expires_at: SystemTime,
) -> rf_store::StoredPropertyPackageManifest {
    let mut manifest = rf_store::StoredPropertyPackageManifest::new(
        package_id,
        version,
        StoredPropertyPackageSource::RemoteDerivedPackage,
        vec![rf_types::ComponentId::new("methane")],
    );
    manifest.hash = hash.to_string();
    manifest.size_bytes = size_bytes;
    manifest.expires_at = Some(expires_at);
    manifest
}

struct FailingCacheFileStore {
    writes: Cell<usize>,
    fail_on_write: usize,
}

impl FailingCacheFileStore {
    fn new(fail_on_write: usize) -> Self {
        Self {
            writes: Cell::new(0),
            fail_on_write,
        }
    }
}

impl CacheFileStore for FailingCacheFileStore {
    fn read_existing(&self, path: &Path) -> RfResult<Option<Vec<u8>>> {
        StdCacheFileStore.read_existing(path)
    }

    fn write_all(&self, path: &Path, contents: &[u8]) -> RfResult<()> {
        StdCacheFileStore.write_all(path, contents)?;

        let write_number = self.writes.get() + 1;
        self.writes.set(write_number);
        if write_number == self.fail_on_write {
            return Err(RfError::invalid_input("simulated write failure"));
        }

        Ok(())
    }

    fn remove_file(&self, path: &Path) -> RfResult<()> {
        StdCacheFileStore.remove_file(path)
    }
}
