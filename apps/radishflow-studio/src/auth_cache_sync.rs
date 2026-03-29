use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rf_store::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
    StoredPropertyPackageManifest, StoredPropertyPackagePayload, StoredPropertyPackageRecord,
    StoredPropertyPackageSource, auth_cache_index_to_pretty_json,
    property_package_manifest_to_pretty_json, property_package_payload_to_pretty_json,
};
use rf_types::{RfError, RfResult};
use rf_ui::{
    AuthSessionState, EntitlementSnapshot, EntitlementState, OfflineLeaseRefreshRequest,
    OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant, PropertyPackageManifest,
    PropertyPackageSource,
};

pub fn build_auth_cache_index(
    auth_session: &AuthSessionState,
    entitlement: &EntitlementState,
) -> RfResult<StoredAuthCacheIndex> {
    let (authority_url, subject_id, credential) = resolve_auth_identity(auth_session)?;
    ensure_entitlement_subject_matches(entitlement, &subject_id)?;

    let mut index = StoredAuthCacheIndex::new(authority_url, subject_id, credential);
    sync_auth_cache_index(&mut index, auth_session, entitlement)?;
    Ok(index)
}

pub fn sync_auth_cache_index(
    index: &mut StoredAuthCacheIndex,
    auth_session: &AuthSessionState,
    entitlement: &EntitlementState,
) -> RfResult<()> {
    let (authority_url, subject_id, credential) = resolve_auth_identity(auth_session)?;
    ensure_entitlement_subject_matches(entitlement, &subject_id)?;

    index.authority_url = authority_url;
    index.subject_id = subject_id;
    index.credential = credential;
    index.entitlement = stored_entitlement_cache_from_state(entitlement)?;
    index.last_synced_at = entitlement.last_synced_at;

    if let Some(stored_entitlement) = &index.entitlement {
        let record_expires_at = record_expires_at_from_entitlement(stored_entitlement);
        index.property_packages.retain(|record| {
            stored_entitlement
                .allowed_package_ids
                .contains(&record.package_id)
        });
        for record in &mut index.property_packages {
            record.expires_at = Some(record_expires_at);
        }
    }

    index.validate()
}

pub fn build_offline_refresh_request(
    index: &StoredAuthCacheIndex,
) -> RfResult<OfflineLeaseRefreshRequest> {
    index.validate()?;

    Ok(OfflineLeaseRefreshRequest {
        package_ids: index
            .property_packages
            .iter()
            .map(|record| record.package_id.clone())
            .collect::<BTreeSet<_>>(),
        current_offline_lease_expires_at: index
            .entitlement
            .as_ref()
            .and_then(|entitlement| entitlement.offline_lease_expires_at),
        installation_id: None,
    })
}

pub fn record_downloaded_package(
    index: &mut StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    lease_grant: &PropertyPackageLeaseGrant,
    downloaded_at: SystemTime,
) -> RfResult<()> {
    index.validate()?;
    let record = build_downloaded_package_record(index, manifest, lease_grant, downloaded_at)?;
    upsert_downloaded_package_record(index, record)
}

pub fn persist_downloaded_package_to_cache(
    cache_root: impl AsRef<Path>,
    index: &mut StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    lease_grant: &PropertyPackageLeaseGrant,
    payload: &StoredPropertyPackagePayload,
    downloaded_at: SystemTime,
) -> RfResult<()> {
    persist_downloaded_package_to_cache_with_store(
        cache_root.as_ref(),
        index,
        manifest,
        lease_grant,
        payload,
        downloaded_at,
        &StdCacheFileStore,
    )
}

pub fn apply_offline_refresh_to_auth_cache(
    index: &mut StoredAuthCacheIndex,
    response: &OfflineLeaseRefreshResponse,
) -> RfResult<()> {
    index.validate()?;

    if index.subject_id != response.snapshot.subject_id {
        return Err(RfError::invalid_input(format!(
            "offline refresh subject_id `{}` does not match auth cache subject_id `{}`",
            response.snapshot.subject_id, index.subject_id
        )));
    }

    let stored_entitlement =
        stored_entitlement_cache_from_snapshot(&response.snapshot, response.refreshed_at);
    let record_expires_at = record_expires_at_from_entitlement(&stored_entitlement);

    index.property_packages.retain(|record| {
        response.manifest_list.packages.iter().any(|manifest| {
            manifest.package_id == record.package_id
                && manifest.version == record.version
                && response
                    .snapshot
                    .allowed_package_ids
                    .contains(&record.package_id)
                && manifest_matches_cached_record(manifest, record)
        })
    });

    for record in &mut index.property_packages {
        if let Some(manifest) = response.manifest_list.packages.iter().find(|manifest| {
            manifest.package_id == record.package_id && manifest.version == record.version
        }) {
            record.source = stored_source_from_manifest_source(manifest.source)?;
            record.expires_at = Some(record_expires_at);
        }
    }

    index.entitlement = Some(stored_entitlement);
    index.last_synced_at = Some(response.refreshed_at);
    index.validate()
}

fn persist_downloaded_package_to_cache_with_store<Store>(
    cache_root: &Path,
    index: &mut StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    lease_grant: &PropertyPackageLeaseGrant,
    payload: &StoredPropertyPackagePayload,
    downloaded_at: SystemTime,
    store: &Store,
) -> RfResult<()>
where
    Store: CacheFileStore,
{
    index.validate()?;
    let record = build_downloaded_package_record(index, manifest, lease_grant, downloaded_at)?;
    let stored_manifest = build_stored_downloaded_manifest(index, manifest, lease_grant)?;
    stored_manifest.validate_against_record(&record)?;
    payload.validate_against_manifest(&stored_manifest)?;

    let mut updated_index = index.clone();
    upsert_downloaded_package_record(&mut updated_index, record.clone())?;

    let writes = vec![
        PendingCacheWrite::new(
            record.payload_path_under(cache_root).ok_or_else(|| {
                RfError::invalid_input(format!(
                    "downloaded package `{}` is missing a local payload path",
                    record.package_id
                ))
            })?,
            property_package_payload_to_pretty_json(payload)?.into_bytes(),
        ),
        PendingCacheWrite::new(
            record.manifest_path_under(cache_root),
            property_package_manifest_to_pretty_json(&stored_manifest)?.into_bytes(),
        ),
        PendingCacheWrite::new(
            updated_index.index_path_under(cache_root),
            auth_cache_index_to_pretty_json(&updated_index)?.into_bytes(),
        ),
    ];

    write_cache_files_with_rollback(store, &writes).map_err(|error| {
        RfError::invalid_input(format!(
            "persist downloaded package `{}` to cache: {}",
            record.package_id,
            error.message()
        ))
    })?;

    *index = updated_index;
    Ok(())
}

trait CacheFileStore {
    fn read_existing(&self, path: &Path) -> RfResult<Option<Vec<u8>>>;

    fn write_all(&self, path: &Path, contents: &[u8]) -> RfResult<()>;

    fn remove_file(&self, path: &Path) -> RfResult<()>;
}

#[derive(Debug, Clone, Copy)]
struct StdCacheFileStore;

impl CacheFileStore for StdCacheFileStore {
    fn read_existing(&self, path: &Path) -> RfResult<Option<Vec<u8>>> {
        match fs::read(path) {
            Ok(contents) => Ok(Some(contents)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(map_cache_io_error("read existing cache file", path, &error)),
        }
    }

    fn write_all(&self, path: &Path, contents: &[u8]) -> RfResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                map_cache_io_error("create cache parent directories", parent, &error)
            })?;
        }

        fs::write(path, contents)
            .map_err(|error| map_cache_io_error("write cache file", path, &error))
    }

    fn remove_file(&self, path: &Path) -> RfResult<()> {
        match fs::remove_file(path) {
            Ok(()) => {
                prune_empty_parent_directories(path);
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(map_cache_io_error("remove cache file", path, &error)),
        }
    }
}

#[derive(Debug, Clone)]
struct PendingCacheWrite {
    path: PathBuf,
    contents: Vec<u8>,
}

impl PendingCacheWrite {
    fn new(path: PathBuf, contents: Vec<u8>) -> Self {
        Self { path, contents }
    }
}

#[derive(Debug, Clone)]
struct CacheRollbackEntry {
    path: PathBuf,
    original_contents: Option<Vec<u8>>,
}

fn write_cache_files_with_rollback<Store>(
    store: &Store,
    writes: &[PendingCacheWrite],
) -> RfResult<()>
where
    Store: CacheFileStore,
{
    let mut rollback_entries = Vec::with_capacity(writes.len());

    for write in writes {
        rollback_entries.push(CacheRollbackEntry {
            path: write.path.clone(),
            original_contents: store.read_existing(&write.path)?,
        });

        if let Err(error) = store.write_all(&write.path, &write.contents) {
            let rollback_message = rollback_cache_files(store, &rollback_entries);
            return match rollback_message {
                Ok(()) => Err(error),
                Err(rollback_error) => Err(RfError::invalid_input(format!(
                    "{}; rollback also failed: {}",
                    error.message(),
                    rollback_error.message()
                ))),
            };
        }
    }

    Ok(())
}

fn rollback_cache_files<Store>(
    store: &Store,
    rollback_entries: &[CacheRollbackEntry],
) -> RfResult<()>
where
    Store: CacheFileStore,
{
    let mut rollback_errors = Vec::new();

    for entry in rollback_entries.iter().rev() {
        let result = match &entry.original_contents {
            Some(original_contents) => store.write_all(&entry.path, original_contents),
            None => store.remove_file(&entry.path),
        };

        if let Err(error) = result {
            rollback_errors.push(format!("`{}`: {}", entry.path.display(), error.message()));
        }
    }

    if rollback_errors.is_empty() {
        Ok(())
    } else {
        Err(RfError::invalid_input(format!(
            "restore cache files after failure: {}",
            rollback_errors.join("; ")
        )))
    }
}

fn map_cache_io_error(action: &str, path: &Path, error: &std::io::Error) -> RfError {
    RfError::invalid_input(format!("{action} `{}`: {error}", path.display()))
}

fn prune_empty_parent_directories(path: &Path) {
    for parent in path.ancestors().skip(1) {
        if parent.as_os_str().is_empty() {
            break;
        }

        match fs::remove_dir(parent) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::DirectoryNotEmpty
                        | std::io::ErrorKind::PermissionDenied
                        | std::io::ErrorKind::InvalidInput
                ) =>
            {
                break;
            }
            Err(_) => break,
        }
    }
}

fn resolve_auth_identity(
    auth_session: &AuthSessionState,
) -> RfResult<(String, String, StoredCredentialReference)> {
    let authority_url = auth_session
        .authority_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            RfError::invalid_input("auth session must contain a non-empty authority_url")
        })?;
    let user = auth_session.current_user.as_ref().ok_or_else(|| {
        RfError::invalid_input("auth session must contain current_user before syncing auth cache")
    })?;
    let token_lease = auth_session.token_lease.as_ref().ok_or_else(|| {
        RfError::invalid_input("auth session must contain token_lease before syncing auth cache")
    })?;

    if user.subject_id.trim().is_empty() {
        return Err(RfError::invalid_input(
            "auth session current_user must contain a non-empty subject_id",
        ));
    }

    if token_lease.credential_handle.service.trim().is_empty()
        || token_lease.credential_handle.account.trim().is_empty()
    {
        return Err(RfError::invalid_input(
            "token lease credential handle must contain non-empty service and account",
        ));
    }

    Ok((
        authority_url.to_string(),
        user.subject_id.clone(),
        StoredCredentialReference::new(
            token_lease.credential_handle.service.clone(),
            token_lease.credential_handle.account.clone(),
        ),
    ))
}

fn ensure_entitlement_subject_matches(
    entitlement: &EntitlementState,
    subject_id: &str,
) -> RfResult<()> {
    if let Some(snapshot) = &entitlement.snapshot
        && snapshot.subject_id != subject_id
    {
        return Err(RfError::invalid_input(format!(
            "entitlement subject_id `{}` does not match authenticated user `{subject_id}`",
            snapshot.subject_id
        )));
    }

    Ok(())
}

fn stored_entitlement_cache_from_state(
    entitlement: &EntitlementState,
) -> RfResult<Option<StoredEntitlementCache>> {
    let Some(snapshot) = &entitlement.snapshot else {
        return Ok(None);
    };
    let synced_at = entitlement.last_synced_at.ok_or_else(|| {
        RfError::invalid_input("entitlement state with snapshot must contain last_synced_at")
    })?;

    Ok(Some(stored_entitlement_cache_from_snapshot(
        snapshot, synced_at,
    )))
}

fn stored_entitlement_cache_from_snapshot(
    snapshot: &EntitlementSnapshot,
    synced_at: SystemTime,
) -> StoredEntitlementCache {
    StoredEntitlementCache {
        subject_id: snapshot.subject_id.clone(),
        tenant_id: snapshot.tenant_id.clone(),
        synced_at,
        issued_at: snapshot.issued_at,
        expires_at: snapshot.expires_at,
        offline_lease_expires_at: snapshot.offline_lease_expires_at,
        feature_keys: snapshot.features.clone(),
        allowed_package_ids: snapshot.allowed_package_ids.clone(),
    }
}

fn stored_source_from_manifest_source(
    source: PropertyPackageSource,
) -> RfResult<StoredPropertyPackageSource> {
    match source {
        PropertyPackageSource::LocalBundled => Ok(StoredPropertyPackageSource::LocalBundled),
        PropertyPackageSource::RemoteDerivedPackage => {
            Ok(StoredPropertyPackageSource::RemoteDerivedPackage)
        }
        PropertyPackageSource::RemoteEvaluationService => Err(RfError::invalid_input(
            "remote evaluation packages must not be recorded as downloaded local packages",
        )),
    }
}

fn build_downloaded_package_record(
    index: &StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    lease_grant: &PropertyPackageLeaseGrant,
    downloaded_at: SystemTime,
) -> RfResult<StoredPropertyPackageRecord> {
    ensure_package_download_allowed(index, manifest)?;

    if manifest.source != PropertyPackageSource::RemoteDerivedPackage {
        return Err(RfError::invalid_input(format!(
            "downloaded package persistence only supports remote derived packages, received `{:?}`",
            manifest.source
        )));
    }

    if manifest.package_id != lease_grant.package_id {
        return Err(RfError::invalid_input(format!(
            "lease grant package_id `{}` does not match manifest package_id `{}`",
            lease_grant.package_id, manifest.package_id
        )));
    }

    if manifest.version != lease_grant.version {
        return Err(RfError::invalid_input(format!(
            "lease grant version `{}` does not match manifest version `{}`",
            lease_grant.version, manifest.version
        )));
    }

    if lease_grant.hash.trim().is_empty() {
        return Err(RfError::invalid_input(format!(
            "lease grant for package `{}` must contain a non-empty hash",
            lease_grant.package_id
        )));
    }

    if lease_grant.size_bytes == 0 {
        return Err(RfError::invalid_input(format!(
            "lease grant for package `{}` must contain a non-zero size_bytes",
            lease_grant.package_id
        )));
    }

    if !manifest.hash.is_empty() && manifest.hash != lease_grant.hash {
        return Err(RfError::invalid_input(format!(
            "lease grant hash `{}` does not match manifest hash `{}`",
            lease_grant.hash, manifest.hash
        )));
    }

    if manifest.size_bytes != 0 && manifest.size_bytes != lease_grant.size_bytes {
        return Err(RfError::invalid_input(format!(
            "lease grant size `{}` does not match manifest size `{}`",
            lease_grant.size_bytes, manifest.size_bytes
        )));
    }

    let mut record = StoredPropertyPackageRecord::new(
        manifest.package_id.clone(),
        manifest.version.clone(),
        stored_source_from_manifest_source(manifest.source)?,
        lease_grant.hash.clone(),
        lease_grant.size_bytes,
        downloaded_at,
    );
    record.expires_at = Some(active_package_expiration(index)?);
    Ok(record)
}

fn build_stored_downloaded_manifest(
    index: &StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    lease_grant: &PropertyPackageLeaseGrant,
) -> RfResult<StoredPropertyPackageManifest> {
    let mut stored_manifest = StoredPropertyPackageManifest::new(
        manifest.package_id.clone(),
        manifest.version.clone(),
        stored_source_from_manifest_source(manifest.source)?,
        manifest.component_ids.clone(),
    );
    stored_manifest.hash = if manifest.hash.is_empty() {
        lease_grant.hash.clone()
    } else {
        manifest.hash.clone()
    };
    stored_manifest.size_bytes = if manifest.size_bytes == 0 {
        lease_grant.size_bytes
    } else {
        manifest.size_bytes
    };
    stored_manifest.expires_at = Some(active_package_expiration(index)?);
    stored_manifest.validate()?;
    Ok(stored_manifest)
}

fn upsert_downloaded_package_record(
    index: &mut StoredAuthCacheIndex,
    record: StoredPropertyPackageRecord,
) -> RfResult<()> {
    index
        .property_packages
        .retain(|existing| existing.package_id != record.package_id);
    index.property_packages.push(record);
    index
        .property_packages
        .sort_by(|left, right| left.package_id.cmp(&right.package_id));

    index.validate()
}

fn ensure_package_download_allowed(
    index: &StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
) -> RfResult<()> {
    if manifest.package_id.trim().is_empty() {
        return Err(RfError::invalid_input(
            "property package manifest must contain a non-empty package_id",
        ));
    }

    if manifest.version.trim().is_empty() {
        return Err(RfError::invalid_input(
            "property package manifest must contain a non-empty version",
        ));
    }

    let entitlement = index.entitlement.as_ref().ok_or_else(|| {
        RfError::invalid_input("auth cache must contain entitlement before recording downloads")
    })?;

    if !entitlement
        .allowed_package_ids
        .contains(&manifest.package_id)
    {
        return Err(RfError::invalid_input(format!(
            "package `{}` is not present in current entitlement snapshot",
            manifest.package_id
        )));
    }

    Ok(())
}

fn active_package_expiration(index: &StoredAuthCacheIndex) -> RfResult<SystemTime> {
    let entitlement = index.entitlement.as_ref().ok_or_else(|| {
        RfError::invalid_input("auth cache must contain entitlement before recording downloads")
    })?;

    Ok(record_expires_at_from_entitlement(entitlement))
}

fn record_expires_at_from_entitlement(entitlement: &StoredEntitlementCache) -> SystemTime {
    entitlement
        .offline_lease_expires_at
        .unwrap_or(entitlement.expires_at)
}

fn manifest_matches_cached_record(
    manifest: &PropertyPackageManifest,
    record: &StoredPropertyPackageRecord,
) -> bool {
    manifest.source != PropertyPackageSource::RemoteEvaluationService
        && (manifest.hash.is_empty() || manifest.hash == record.hash)
        && (manifest.size_bytes == 0 || manifest.size_bytes == record.size_bytes)
}

#[cfg(test)]
mod tests {
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

    use super::{
        CacheFileStore, StdCacheFileStore, persist_downloaded_package_to_cache_with_store,
    };
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

        let request =
            build_offline_refresh_request(&index).expect("expected offline refresh request");

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

        apply_offline_refresh_to_auth_cache(&mut index, &response)
            .expect("expected offline refresh");

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
        let stored_manifest =
            read_property_package_manifest(cached_record.manifest_path_under(&root))
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
        let restored_index = read_auth_cache_index(index.index_path_under(&root))
            .expect("expected restored index read");

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
}
