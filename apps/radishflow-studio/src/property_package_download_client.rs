use std::path::Path;
use std::time::SystemTime;

use rf_store::StoredAuthCacheIndex;
use rf_types::{RfError, RfResult};
use rf_ui::{PropertyPackageLeaseGrant, PropertyPackageManifest};

use crate::persist_downloaded_package_response_to_cache;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyPackageDownloadResponse {
    pub contents: String,
    pub downloaded_at: SystemTime,
}

impl PropertyPackageDownloadResponse {
    pub fn new(contents: impl Into<String>, downloaded_at: SystemTime) -> Self {
        Self {
            contents: contents.into(),
            downloaded_at,
        }
    }
}

pub trait PropertyPackageDownloadFetcher {
    fn fetch_download(
        &self,
        lease_grant: &PropertyPackageLeaseGrant,
    ) -> RfResult<PropertyPackageDownloadResponse>;
}

pub fn download_property_package_to_cache<Fetcher>(
    cache_root: impl AsRef<Path>,
    index: &mut StoredAuthCacheIndex,
    manifest: &PropertyPackageManifest,
    lease_grant: &PropertyPackageLeaseGrant,
    fetcher: &Fetcher,
) -> RfResult<()>
where
    Fetcher: PropertyPackageDownloadFetcher,
{
    let response = fetcher.fetch_download(lease_grant)?;
    if response.contents.trim().is_empty() {
        return Err(RfError::invalid_input(format!(
            "download response for package `{}` must not be empty",
            lease_grant.package_id
        )));
    }

    persist_downloaded_package_response_to_cache(
        cache_root,
        index,
        manifest,
        lease_grant,
        &response.contents,
        response.downloaded_at,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
        property_package_payload_integrity, read_property_package_payload,
    };
    use rf_types::ComponentId;
    use rf_ui::{PropertyPackageLeaseGrant, PropertyPackageManifest, PropertyPackageSource};

    use crate::{
        PropertyPackageDownloadFetcher, PropertyPackageDownloadResponse,
        download_property_package_to_cache, parse_property_package_download_json,
    };

    struct StaticDownloadFetcher {
        response: PropertyPackageDownloadResponse,
    }

    impl PropertyPackageDownloadFetcher for StaticDownloadFetcher {
        fn fetch_download(
            &self,
            _lease_grant: &PropertyPackageLeaseGrant,
        ) -> rf_types::RfResult<PropertyPackageDownloadResponse> {
            Ok(self.response.clone())
        }
    }

    #[test]
    fn download_property_package_to_cache_fetches_response_and_persists_assets() {
        let root = unique_temp_path("download-fetcher-success");
        let mut index = sample_auth_cache_index();
        let download = parse_property_package_download_json(&sample_download_json())
            .expect("expected sample download");
        let payload = download
            .to_stored_payload()
            .expect("expected sample payload");
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
        let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
        let fetcher = StaticDownloadFetcher {
            response: PropertyPackageDownloadResponse::new(sample_download_json(), timestamp(200)),
        };

        download_property_package_to_cache(&root, &mut index, &manifest, &lease_grant, &fetcher)
            .expect("expected cached download");

        let cached_record = &index.property_packages[0];
        let payload = read_property_package_payload(
            cached_record
                .payload_path_under(&root)
                .expect("expected payload path"),
        )
        .expect("expected payload read");

        assert_eq!(payload.package_id, "binary-hydrocarbon-lite-v1");
        assert_eq!(payload.components.len(), 2);

        fs::remove_dir_all(&root).expect("expected temp dir cleanup");
    }

    #[test]
    fn download_property_package_to_cache_rejects_hash_mismatch_before_updating_index() {
        let root = unique_temp_path("download-fetcher-mismatch");
        let mut index = sample_auth_cache_index();
        let download = parse_property_package_download_json(&sample_download_json())
            .expect("expected sample download");
        let payload = download
            .to_stored_payload()
            .expect("expected sample payload");
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let manifest = sample_manifest(&integrity.hash, integrity.size_bytes);
        let lease_grant = sample_lease_grant(&integrity.hash, integrity.size_bytes);
        let fetcher = StaticDownloadFetcher {
            response: PropertyPackageDownloadResponse::new(
                sample_download_json().replace("\"Methane\"", "\"Methane Modified\""),
                timestamp(200),
            ),
        };

        let error = download_property_package_to_cache(
            &root,
            &mut index,
            &manifest,
            &lease_grant,
            &fetcher,
        )
        .expect_err("expected hash mismatch");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert!(error.message().contains("payload hash"));
        assert!(index.property_packages.is_empty());
        fs::remove_dir_all(&root).ok();
    }

    fn sample_auth_cache_index() -> StoredAuthCacheIndex {
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
            allowed_package_ids: BTreeSet::from(["binary-hydrocarbon-lite-v1".to_string()]),
        });
        index
    }

    fn sample_manifest(hash: &str, size_bytes: u64) -> PropertyPackageManifest {
        let mut manifest = PropertyPackageManifest::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            PropertyPackageSource::RemoteDerivedPackage,
        );
        manifest.hash = hash.to_string();
        manifest.size_bytes = size_bytes;
        manifest.component_ids = vec![ComponentId::new("methane"), ComponentId::new("ethane")];
        manifest
    }

    fn sample_lease_grant(hash: &str, size_bytes: u64) -> PropertyPackageLeaseGrant {
        PropertyPackageLeaseGrant {
            package_id: "binary-hydrocarbon-lite-v1".to_string(),
            version: "2026.03.1".to_string(),
            lease_id: "lease-1".to_string(),
            download_url: "https://assets.radish.local/lease-1".to_string(),
            hash: hash.to_string(),
            size_bytes,
            expires_at: timestamp(210),
        }
    }

    fn sample_download_json() -> String {
        fs::read_to_string(sample_download_path()).expect("expected sample download json")
    }

    fn sample_download_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/sample-components/property-packages/binary-hydrocarbon-lite-v1/download.json")
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
}
