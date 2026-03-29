mod auth_cache;
mod layout;
mod project;

pub use auth_cache::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
    StoredPropertyPackageRecord, StoredPropertyPackageSource,
};
pub use layout::{
    STORED_AUTH_CACHE_INDEX_FILE_NAME, STORED_AUTH_ROOT_DIR, STORED_PACKAGE_CACHE_ROOT_DIR,
    STORED_PROPERTY_PACKAGE_MANIFEST_FILE_NAME, STORED_PROPERTY_PACKAGE_PAYLOAD_FILE_NAME,
    StoredAuthCacheLayout,
};
pub use project::{
    DateTimeUtc, STORED_PROJECT_FILE_EXTENSION, StoredDocumentMetadata, StoredProjectDocument,
    StoredProjectFile,
};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    use rf_model::Flowsheet;

    use crate::{
        STORED_PROJECT_FILE_EXTENSION, StoredAuthCacheIndex, StoredAuthCacheLayout,
        StoredCredentialReference, StoredDocumentMetadata, StoredProjectFile,
        StoredPropertyPackageRecord, StoredPropertyPackageSource,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    #[test]
    fn project_file_keeps_document_payload_separate_from_auth_cache() {
        let flowsheet = Flowsheet::new("demo");
        let metadata = StoredDocumentMetadata::new("doc-1", "Demo Project", timestamp(10));
        let project = StoredProjectFile::new(flowsheet, metadata);
        let auth_cache = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );

        assert_eq!(project.kind, "radishflow.project-file");
        assert_eq!(project.schema_version, 1);
        assert_eq!(project.document.metadata.document_id, "doc-1");
        assert_eq!(STORED_PROJECT_FILE_EXTENSION, ".rfproj.json");
        assert_eq!(auth_cache.kind, "radishflow.auth-cache-index");
        assert_eq!(project.document.revision, 0);
        assert_eq!(auth_cache.schema_version, 1);
        assert!(auth_cache.entitlement.is_none());
        assert_eq!(
            auth_cache.index_relative_path(),
            PathBuf::from("auth")
                .join("https_id.radish.local")
                .join("user-123")
                .join("index.json")
        );
    }

    #[test]
    fn property_package_record_reports_expiration_from_cached_metadata() {
        let mut record = StoredPropertyPackageRecord::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:test",
            1024,
            timestamp(100),
        );
        record.expires_at = Some(timestamp(200));

        assert!(!record.is_expired_at(timestamp(150)));
        assert!(record.is_expired_at(timestamp(250)));
        assert_eq!(
            record.manifest_relative_path,
            StoredAuthCacheLayout::package_manifest_relative_path(
                "binary-hydrocarbon-lite-v1",
                "2026.03.1"
            )
        );
        assert_eq!(
            record.payload_relative_path,
            Some(StoredAuthCacheLayout::package_payload_relative_path(
                "binary-hydrocarbon-lite-v1",
                "2026.03.1"
            ))
        );
    }

    #[test]
    fn remote_evaluation_packages_do_not_claim_local_payload_paths() {
        let record = StoredPropertyPackageRecord::new(
            "premium-eos-v1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteEvaluationService,
            "sha256:test",
            0,
            timestamp(300),
        );

        assert!(record.payload_relative_path.is_none());
        assert_eq!(
            record.manifest_relative_path,
            PathBuf::from("packages")
                .join("premium-eos-v1")
                .join("2026.03.1")
                .join("manifest.json")
        );
    }
}
