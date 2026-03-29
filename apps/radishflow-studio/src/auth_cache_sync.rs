use std::collections::BTreeSet;
use std::time::SystemTime;

use rf_store::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
    StoredPropertyPackageRecord, StoredPropertyPackageSource,
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
    ensure_package_download_allowed(index, manifest)?;

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

    index
        .property_packages
        .retain(|existing| existing.package_id != manifest.package_id);
    index.property_packages.push(record);
    index
        .property_packages
        .sort_by(|left, right| left.package_id.cmp(&right.package_id));

    index.validate()
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
    use std::collections::BTreeSet;
    use std::time::{Duration, UNIX_EPOCH};

    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
        StoredPropertyPackageRecord, StoredPropertyPackageSource,
    };
    use rf_ui::{
        AuthSessionState, AuthenticatedUser, EntitlementSnapshot, EntitlementState,
        OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant, PropertyPackageManifest,
        PropertyPackageManifestList, PropertyPackageSource, SecureCredentialHandle, TokenLease,
    };

    use crate::{
        apply_offline_refresh_to_auth_cache, build_auth_cache_index, build_offline_refresh_request,
        record_downloaded_package, sync_auth_cache_index,
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
}
