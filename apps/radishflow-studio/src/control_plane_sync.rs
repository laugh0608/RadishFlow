use std::collections::BTreeSet;
use std::path::Path;
use std::time::SystemTime;

use rf_store::StoredAuthCacheIndex;
use rf_types::{RfError, RfResult};
use rf_ui::{
    EntitlementSnapshot, EntitlementState, OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant,
    PropertyPackageLeaseRequest, PropertyPackageManifest, PropertyPackageManifestList,
    PropertyPackageSource,
};

use crate::{
    PropertyPackageDownloadFetcher, RadishFlowControlPlaneClient,
    RadishFlowControlPlaneClientError, apply_offline_refresh_to_auth_cache,
    build_offline_refresh_request, download_property_package_to_cache,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitlementSyncResult {
    pub snapshot: EntitlementSnapshot,
    pub manifest_list: PropertyPackageManifestList,
    pub synced_at: SystemTime,
}

#[derive(Debug, Clone)]
pub struct RadishFlowControlPlaneSyncService<Client, Fetcher> {
    control_plane_client: Client,
    download_fetcher: Fetcher,
}

impl<Client, Fetcher> RadishFlowControlPlaneSyncService<Client, Fetcher> {
    pub fn new(control_plane_client: Client, download_fetcher: Fetcher) -> Self {
        Self {
            control_plane_client,
            download_fetcher,
        }
    }

    pub fn control_plane_client(&self) -> &Client {
        &self.control_plane_client
    }

    pub fn download_fetcher(&self) -> &Fetcher {
        &self.download_fetcher
    }
}

impl<Client, Fetcher> RadishFlowControlPlaneSyncService<Client, Fetcher>
where
    Client: RadishFlowControlPlaneClient,
    Fetcher: PropertyPackageDownloadFetcher,
{
    pub fn fetch_entitlement_sync_result(
        &self,
        access_token: &str,
    ) -> RfResult<EntitlementSyncResult> {
        let snapshot = self
            .control_plane_client
            .fetch_entitlement_snapshot(access_token)
            .map_err(|error| map_control_plane_error("fetch entitlement snapshot", error))?;
        let manifest_list = self
            .control_plane_client
            .fetch_property_package_manifest_list(access_token)
            .map_err(|error| {
                map_control_plane_error("fetch property package manifest list", error)
            })?;

        validate_manifest_sync_consistency(&snapshot.value, &manifest_list.value.packages)?;

        Ok(EntitlementSyncResult {
            snapshot: snapshot.value,
            manifest_list: manifest_list.value,
            synced_at: latest_received_at(snapshot.received_at, manifest_list.received_at),
        })
    }

    pub fn sync_entitlement_state(
        &self,
        access_token: &str,
        entitlement: &mut EntitlementState,
    ) -> RfResult<EntitlementSyncResult> {
        let result = self.fetch_entitlement_sync_result(access_token)?;
        entitlement.update_from_manifest_list(
            result.snapshot.clone(),
            result.manifest_list.clone(),
            result.synced_at,
        );
        Ok(result)
    }

    pub fn download_entitled_package_to_cache(
        &self,
        cache_root: impl AsRef<Path>,
        auth_cache_index: &mut StoredAuthCacheIndex,
        entitlement: &EntitlementState,
        package_id: &str,
        access_token: &str,
        installation_id: Option<String>,
    ) -> RfResult<PropertyPackageLeaseGrant> {
        auth_cache_index.validate()?;
        ensure_auth_cache_allows_package(auth_cache_index, package_id)?;

        let manifest = entitlement
            .package_manifests
            .get(package_id)
            .ok_or_else(|| {
                RfError::invalid_input(format!(
                    "entitlement state does not contain manifest for package `{package_id}`"
                ))
            })?;
        ensure_entitlement_allows_manifest(entitlement, manifest)?;
        ensure_manifest_supports_download(manifest)?;

        let lease_request = build_lease_request(auth_cache_index, manifest, installation_id)?;
        let lease_grant = self
            .control_plane_client
            .request_property_package_lease(access_token, package_id, &lease_request)
            .map_err(|error| map_control_plane_error("request property package lease", error))?
            .value;

        download_property_package_to_cache(
            cache_root,
            auth_cache_index,
            manifest,
            &lease_grant,
            &self.download_fetcher,
        )?;

        Ok(lease_grant)
    }

    pub fn refresh_offline_auth_cache(
        &self,
        access_token: &str,
        auth_cache_index: &mut StoredAuthCacheIndex,
    ) -> RfResult<OfflineLeaseRefreshResponse> {
        let request = build_offline_refresh_request(auth_cache_index)?;
        let response = self
            .control_plane_client
            .refresh_offline_leases(access_token, &request)
            .map_err(|error| map_control_plane_error("refresh offline lease", error))?
            .value;
        apply_offline_refresh_to_auth_cache(auth_cache_index, &response)?;
        Ok(response)
    }
}

fn ensure_entitlement_allows_manifest(
    entitlement: &EntitlementState,
    manifest: &PropertyPackageManifest,
) -> RfResult<()> {
    if !entitlement.is_package_allowed(&manifest.package_id) {
        return Err(RfError::invalid_input(format!(
            "entitlement snapshot does not allow package `{}`",
            manifest.package_id
        )));
    }

    Ok(())
}

fn ensure_manifest_supports_download(manifest: &PropertyPackageManifest) -> RfResult<()> {
    if manifest.source != PropertyPackageSource::RemoteDerivedPackage {
        return Err(RfError::invalid_input(format!(
            "package `{}` cannot be downloaded because source is `{:?}`",
            manifest.package_id, manifest.source
        )));
    }

    if !manifest.lease_required {
        return Err(RfError::invalid_input(format!(
            "package `{}` must require a lease before download",
            manifest.package_id
        )));
    }

    Ok(())
}

fn ensure_auth_cache_allows_package(
    auth_cache_index: &StoredAuthCacheIndex,
    package_id: &str,
) -> RfResult<()> {
    let entitlement = auth_cache_index.entitlement.as_ref().ok_or_else(|| {
        RfError::invalid_input(
            "auth cache index must contain entitlement before downloading property packages",
        )
    })?;

    if !entitlement.allowed_package_ids.contains(package_id) {
        return Err(RfError::invalid_input(format!(
            "auth cache index is not synced for package `{package_id}`; sync entitlement before downloading"
        )));
    }

    Ok(())
}

fn build_lease_request(
    auth_cache_index: &StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    installation_id: Option<String>,
) -> RfResult<PropertyPackageLeaseRequest> {
    let mut request = PropertyPackageLeaseRequest::new(manifest.version.clone());
    request.current_hash = auth_cache_index
        .property_packages
        .iter()
        .find(|record| {
            record.package_id == manifest.package_id && record.version == manifest.version
        })
        .map(|record| record.hash.clone());
    request.installation_id = installation_id;
    if request.version.trim().is_empty() {
        return Err(RfError::invalid_input(format!(
            "manifest for package `{}` must contain a non-empty version before requesting a lease",
            manifest.package_id
        )));
    }
    Ok(request)
}

fn validate_manifest_sync_consistency(
    snapshot: &EntitlementSnapshot,
    manifests: &[PropertyPackageManifest],
) -> RfResult<()> {
    let mut seen = BTreeSet::new();
    for manifest in manifests {
        if !seen.insert(manifest.package_id.clone()) {
            return Err(RfError::invalid_input(format!(
                "control plane manifest list contains duplicate package `{}`",
                manifest.package_id
            )));
        }
        if !snapshot.allowed_package_ids.contains(&manifest.package_id) {
            return Err(RfError::invalid_input(format!(
                "control plane manifest list returned package `{}` outside allowedPackageIds",
                manifest.package_id
            )));
        }
    }

    Ok(())
}

fn latest_received_at(left: SystemTime, right: SystemTime) -> SystemTime {
    if left >= right { left } else { right }
}

fn map_control_plane_error(operation: &str, error: RadishFlowControlPlaneClientError) -> RfError {
    error.into_rf_error(operation)
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
        StoredPropertyPackageRecord, StoredPropertyPackageSource,
        property_package_payload_integrity, read_property_package_payload,
    };
    use rf_types::ComponentId;
    use rf_ui::{
        EntitlementSnapshot, EntitlementState, OfflineLeaseRefreshRequest,
        OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant, PropertyPackageLeaseRequest,
        PropertyPackageManifest, PropertyPackageManifestList, PropertyPackageSource,
    };

    use crate::{
        PropertyPackageDownloadFetchError, PropertyPackageDownloadFetcher,
        PropertyPackageDownloadResponse, RadishFlowControlPlaneClient,
        RadishFlowControlPlaneClientError, RadishFlowControlPlaneResponse,
        RadishFlowControlPlaneSyncService, parse_property_package_download_json,
    };

    const SAMPLE_PACKAGE_ID: &str = "binary-hydrocarbon-lite-v1";
    const STALE_PACKAGE_ID: &str = "stale-package-v1";

    #[test]
    fn sync_entitlement_state_updates_runtime_state() {
        let service = sample_service();
        let mut entitlement = EntitlementState::default();

        let result = service
            .sync_entitlement_state("access-token", &mut entitlement)
            .expect("expected entitlement sync");

        assert_eq!(result.synced_at, timestamp(210));
        assert_eq!(entitlement.last_synced_at, Some(timestamp(210)));
        assert!(entitlement.is_package_allowed(SAMPLE_PACKAGE_ID));
    }

    #[test]
    fn sync_entitlement_state_rejects_manifest_outside_allowed_packages() {
        let client = ScriptedControlPlaneClient::new(
            Ok(RadishFlowControlPlaneResponse::new(
                sample_snapshot([SAMPLE_PACKAGE_ID]),
                timestamp(200),
            )),
            Ok(RadishFlowControlPlaneResponse::new(
                PropertyPackageManifestList::new(
                    timestamp(205),
                    vec![sample_manifest(STALE_PACKAGE_ID)],
                ),
                timestamp(210),
            )),
            Ok(RadishFlowControlPlaneResponse::new(
                sample_lease_grant(SAMPLE_PACKAGE_ID),
                timestamp(220),
            )),
            Ok(RadishFlowControlPlaneResponse::new(
                sample_offline_refresh_response(),
                timestamp(230),
            )),
        );
        let service = RadishFlowControlPlaneSyncService::new(
            client,
            StaticDownloadFetcher {
                response: PropertyPackageDownloadResponse::new(
                    sample_download_json(),
                    timestamp(240),
                ),
            },
        );
        let mut entitlement = EntitlementState::default();

        let error = service
            .sync_entitlement_state("access-token", &mut entitlement)
            .expect_err("expected manifest inconsistency");

        assert!(error.message().contains("outside allowedPackageIds"));
    }

    #[test]
    fn download_entitled_package_to_cache_requests_lease_and_persists_payload() {
        let root = unique_temp_path("control-plane-sync-download");
        let service = sample_service();
        let mut auth_cache_index = sample_auth_cache_index([SAMPLE_PACKAGE_ID]);
        auth_cache_index
            .property_packages
            .push(sample_cached_record(
                SAMPLE_PACKAGE_ID,
                "2026.03.1",
                "sha256:cached",
                512,
            ));
        let mut entitlement = EntitlementState::default();
        entitlement.update(
            sample_snapshot([SAMPLE_PACKAGE_ID]),
            vec![sample_manifest(SAMPLE_PACKAGE_ID)],
            timestamp(150),
        );

        let lease_grant = service
            .download_entitled_package_to_cache(
                &root,
                &mut auth_cache_index,
                &entitlement,
                SAMPLE_PACKAGE_ID,
                "access-token",
                Some("studio-installation-001".to_string()),
            )
            .expect("expected package download");

        assert_eq!(lease_grant.lease_id, "lease-1");
        assert_eq!(service.control_plane_client().lease_request_count(), 1);
        assert_eq!(
            service
                .control_plane_client()
                .last_lease_request()
                .current_hash
                .as_deref(),
            Some("sha256:cached")
        );
        let payload = read_property_package_payload(
            auth_cache_index.property_packages[0]
                .payload_path_under(&root)
                .expect("expected payload path"),
        )
        .expect("expected payload read");
        assert_eq!(payload.package_id, SAMPLE_PACKAGE_ID);

        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn refresh_offline_auth_cache_applies_control_plane_response() {
        let service = sample_service();
        let mut auth_cache_index = sample_auth_cache_index([SAMPLE_PACKAGE_ID, STALE_PACKAGE_ID]);
        let (hash, size_bytes) = sample_download_integrity();
        auth_cache_index
            .property_packages
            .push(sample_cached_record(
                SAMPLE_PACKAGE_ID,
                "2026.03.1",
                &hash,
                size_bytes,
            ));
        auth_cache_index
            .property_packages
            .push(sample_cached_record(
                STALE_PACKAGE_ID,
                "2026.03.1",
                "sha256:stale",
                1024,
            ));

        let response = service
            .refresh_offline_auth_cache("access-token", &mut auth_cache_index)
            .expect("expected offline refresh");

        assert_eq!(
            response.snapshot.allowed_package_ids,
            BTreeSet::from([SAMPLE_PACKAGE_ID.to_string()])
        );
        assert_eq!(auth_cache_index.property_packages.len(), 1);
        assert_eq!(
            auth_cache_index.property_packages[0].package_id,
            SAMPLE_PACKAGE_ID
        );
        assert_eq!(
            service
                .control_plane_client()
                .last_offline_refresh_request()
                .package_ids,
            BTreeSet::from([SAMPLE_PACKAGE_ID.to_string(), STALE_PACKAGE_ID.to_string(),])
        );
    }

    fn sample_service()
    -> RadishFlowControlPlaneSyncService<ScriptedControlPlaneClient, StaticDownloadFetcher> {
        let client = ScriptedControlPlaneClient::new(
            Ok(RadishFlowControlPlaneResponse::new(
                sample_snapshot([SAMPLE_PACKAGE_ID]),
                timestamp(200),
            )),
            Ok(RadishFlowControlPlaneResponse::new(
                PropertyPackageManifestList::new(
                    timestamp(205),
                    vec![sample_manifest(SAMPLE_PACKAGE_ID)],
                ),
                timestamp(210),
            )),
            Ok(RadishFlowControlPlaneResponse::new(
                sample_lease_grant(SAMPLE_PACKAGE_ID),
                timestamp(220),
            )),
            Ok(RadishFlowControlPlaneResponse::new(
                sample_offline_refresh_response(),
                timestamp(230),
            )),
        );
        RadishFlowControlPlaneSyncService::new(
            client,
            StaticDownloadFetcher {
                response: PropertyPackageDownloadResponse::new(
                    sample_download_json(),
                    timestamp(240),
                ),
            },
        )
    }

    fn sample_snapshot<const N: usize>(package_ids: [&str; N]) -> EntitlementSnapshot {
        EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            issued_at: timestamp(100),
            expires_at: timestamp(500),
            offline_lease_expires_at: Some(timestamp(900)),
            features: BTreeSet::from(["desktop-login".to_string()]),
            allowed_package_ids: package_ids
                .into_iter()
                .map(|package_id| package_id.to_string())
                .collect(),
        }
    }

    fn sample_manifest(package_id: &str) -> PropertyPackageManifest {
        let mut manifest = PropertyPackageManifest::new(
            package_id,
            "2026.03.1",
            PropertyPackageSource::RemoteDerivedPackage,
        );
        manifest.component_ids = vec![ComponentId::new("methane"), ComponentId::new("ethane")];
        let (hash, size_bytes) = sample_download_integrity();
        manifest.hash = hash;
        manifest.size_bytes = size_bytes;
        manifest.expires_at = Some(timestamp(900));
        manifest
    }

    fn sample_lease_grant(package_id: &str) -> PropertyPackageLeaseGrant {
        let (hash, size_bytes) = sample_download_integrity();
        PropertyPackageLeaseGrant {
            package_id: package_id.to_string(),
            version: "2026.03.1".to_string(),
            lease_id: "lease-1".to_string(),
            download_url: "https://assets.radish.local/leases/lease-1/download".to_string(),
            hash,
            size_bytes,
            expires_at: timestamp(210),
        }
    }

    fn sample_offline_refresh_response() -> OfflineLeaseRefreshResponse {
        OfflineLeaseRefreshResponse {
            refreshed_at: timestamp(210),
            snapshot: sample_snapshot([SAMPLE_PACKAGE_ID]),
            manifest_list: PropertyPackageManifestList::new(
                timestamp(205),
                vec![sample_manifest(SAMPLE_PACKAGE_ID)],
            ),
        }
    }

    fn sample_auth_cache_index<const N: usize>(package_ids: [&str; N]) -> StoredAuthCacheIndex {
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
            allowed_package_ids: package_ids
                .into_iter()
                .map(|package_id| package_id.to_string())
                .collect(),
        });
        index
    }

    fn sample_cached_record(
        package_id: &str,
        version: &str,
        hash: &str,
        size_bytes: u64,
    ) -> StoredPropertyPackageRecord {
        let mut record = StoredPropertyPackageRecord::new(
            package_id,
            version,
            StoredPropertyPackageSource::RemoteDerivedPackage,
            hash,
            size_bytes,
            timestamp(120),
        );
        record.expires_at = Some(timestamp(900));
        record
    }

    fn sample_download_integrity() -> (String, u64) {
        let download = parse_property_package_download_json(&sample_download_json())
            .expect("expected sample download");
        let payload = download
            .to_stored_payload()
            .expect("expected sample payload");
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        (integrity.hash, integrity.size_bytes)
    }

    fn sample_download_json() -> String {
        fs::read_to_string(sample_download_path()).expect("expected sample download json")
    }

    fn sample_download_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1/download.json",
        )
    }

    fn timestamp(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
    }

    #[derive(Debug, Clone)]
    struct StaticDownloadFetcher {
        response: PropertyPackageDownloadResponse,
    }

    impl PropertyPackageDownloadFetcher for StaticDownloadFetcher {
        fn fetch_download(
            &self,
            _lease_grant: &PropertyPackageLeaseGrant,
        ) -> Result<PropertyPackageDownloadResponse, PropertyPackageDownloadFetchError> {
            Ok(self.response.clone())
        }
    }

    #[derive(Debug, Clone)]
    struct ScriptedControlPlaneClient {
        entitlement_response: Result<
            RadishFlowControlPlaneResponse<EntitlementSnapshot>,
            RadishFlowControlPlaneClientError,
        >,
        manifest_response: Result<
            RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
            RadishFlowControlPlaneClientError,
        >,
        lease_response: Result<
            RadishFlowControlPlaneResponse<PropertyPackageLeaseGrant>,
            RadishFlowControlPlaneClientError,
        >,
        offline_refresh_response: Result<
            RadishFlowControlPlaneResponse<OfflineLeaseRefreshResponse>,
            RadishFlowControlPlaneClientError,
        >,
        lease_request_count: Cell<u32>,
        last_lease_request: RefCell<Option<PropertyPackageLeaseRequest>>,
        last_offline_refresh_request: RefCell<Option<OfflineLeaseRefreshRequest>>,
    }

    impl ScriptedControlPlaneClient {
        fn new(
            entitlement_response: Result<
                RadishFlowControlPlaneResponse<EntitlementSnapshot>,
                RadishFlowControlPlaneClientError,
            >,
            manifest_response: Result<
                RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
                RadishFlowControlPlaneClientError,
            >,
            lease_response: Result<
                RadishFlowControlPlaneResponse<PropertyPackageLeaseGrant>,
                RadishFlowControlPlaneClientError,
            >,
            offline_refresh_response: Result<
                RadishFlowControlPlaneResponse<OfflineLeaseRefreshResponse>,
                RadishFlowControlPlaneClientError,
            >,
        ) -> Self {
            Self {
                entitlement_response,
                manifest_response,
                lease_response,
                offline_refresh_response,
                lease_request_count: Cell::new(0),
                last_lease_request: RefCell::new(None),
                last_offline_refresh_request: RefCell::new(None),
            }
        }

        fn lease_request_count(&self) -> u32 {
            self.lease_request_count.get()
        }

        fn last_lease_request(&self) -> PropertyPackageLeaseRequest {
            self.last_lease_request
                .borrow()
                .clone()
                .expect("expected lease request")
        }

        fn last_offline_refresh_request(&self) -> OfflineLeaseRefreshRequest {
            self.last_offline_refresh_request
                .borrow()
                .clone()
                .expect("expected offline refresh request")
        }
    }

    impl RadishFlowControlPlaneClient for ScriptedControlPlaneClient {
        fn fetch_entitlement_snapshot(
            &self,
            _access_token: &str,
        ) -> Result<
            RadishFlowControlPlaneResponse<EntitlementSnapshot>,
            RadishFlowControlPlaneClientError,
        > {
            self.entitlement_response.clone()
        }

        fn fetch_property_package_manifest_list(
            &self,
            _access_token: &str,
        ) -> Result<
            RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
            RadishFlowControlPlaneClientError,
        > {
            self.manifest_response.clone()
        }

        fn request_property_package_lease(
            &self,
            _access_token: &str,
            _package_id: &str,
            request: &PropertyPackageLeaseRequest,
        ) -> Result<
            RadishFlowControlPlaneResponse<PropertyPackageLeaseGrant>,
            RadishFlowControlPlaneClientError,
        > {
            self.lease_request_count
                .set(self.lease_request_count.get() + 1);
            *self.last_lease_request.borrow_mut() = Some(request.clone());
            self.lease_response.clone()
        }

        fn refresh_offline_leases(
            &self,
            _access_token: &str,
            request: &OfflineLeaseRefreshRequest,
        ) -> Result<
            RadishFlowControlPlaneResponse<OfflineLeaseRefreshResponse>,
            RadishFlowControlPlaneClientError,
        > {
            *self.last_offline_refresh_request.borrow_mut() = Some(request.clone());
            self.offline_refresh_response.clone()
        }
    }
}
