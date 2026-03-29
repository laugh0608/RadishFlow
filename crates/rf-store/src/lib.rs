mod auth_cache;
mod project;

pub use auth_cache::{
    StoredAuthCacheIndex, StoredCredentialReference, StoredEntitlementCache,
    StoredPropertyPackageRecord, StoredPropertyPackageSource,
};
pub use project::{DateTimeUtc, StoredDocumentMetadata, StoredProjectDocument, StoredProjectFile};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, UNIX_EPOCH};

    use rf_model::Flowsheet;

    use crate::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredDocumentMetadata, StoredProjectFile,
        StoredPropertyPackageRecord, StoredPropertyPackageSource,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    #[test]
    fn project_file_keeps_document_payload_separate_from_auth_cache() {
        let flowsheet = Flowsheet::new("demo");
        let metadata = StoredDocumentMetadata::new("Demo Project", timestamp(10));
        let project = StoredProjectFile::new(flowsheet, metadata);
        let auth_cache = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );

        assert_eq!(project.schema_version, 1);
        assert_eq!(project.document.revision, 0);
        assert_eq!(auth_cache.schema_version, 1);
        assert!(auth_cache.entitlement.is_none());
    }

    #[test]
    fn property_package_record_reports_expiration_from_cached_metadata() {
        let record = StoredPropertyPackageRecord {
            package_id: "binary-hydrocarbon-lite-v1".to_string(),
            version: "2026.03.1".to_string(),
            source: StoredPropertyPackageSource::RemoteDerivedPackage,
            local_path: PathBuf::from("packages/binary-hydrocarbon-lite-v1.rfpkg"),
            hash: "sha256:test".to_string(),
            downloaded_at: timestamp(100),
            expires_at: Some(timestamp(200)),
        };

        assert!(!record.is_expired_at(timestamp(150)));
        assert!(record.is_expired_at(timestamp(250)));
    }
}
