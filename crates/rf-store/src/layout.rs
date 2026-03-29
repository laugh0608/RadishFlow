use std::path::PathBuf;

pub const STORED_AUTH_CACHE_INDEX_FILE_NAME: &str = "index.json";
pub const STORED_AUTH_ROOT_DIR: &str = "auth";
pub const STORED_PACKAGE_CACHE_ROOT_DIR: &str = "packages";
pub const STORED_PROPERTY_PACKAGE_MANIFEST_FILE_NAME: &str = "manifest.json";
pub const STORED_PROPERTY_PACKAGE_PAYLOAD_FILE_NAME: &str = "payload.rfpkg";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StoredAuthCacheLayout;

impl StoredAuthCacheLayout {
    pub fn index_relative_path(authority_url: &str, subject_id: &str) -> PathBuf {
        PathBuf::from(STORED_AUTH_ROOT_DIR)
            .join(Self::sanitize_segment(authority_url))
            .join(Self::sanitize_segment(subject_id))
            .join(STORED_AUTH_CACHE_INDEX_FILE_NAME)
    }

    pub fn package_directory_relative_path(package_id: &str, version: &str) -> PathBuf {
        PathBuf::from(STORED_PACKAGE_CACHE_ROOT_DIR)
            .join(Self::sanitize_segment(package_id))
            .join(Self::sanitize_segment(version))
    }

    pub fn package_manifest_relative_path(package_id: &str, version: &str) -> PathBuf {
        Self::package_directory_relative_path(package_id, version)
            .join(STORED_PROPERTY_PACKAGE_MANIFEST_FILE_NAME)
    }

    pub fn package_payload_relative_path(package_id: &str, version: &str) -> PathBuf {
        Self::package_directory_relative_path(package_id, version)
            .join(STORED_PROPERTY_PACKAGE_PAYLOAD_FILE_NAME)
    }

    fn sanitize_segment(value: &str) -> String {
        let mut result = String::with_capacity(value.len());
        let mut last_was_separator = false;

        for ch in value.chars() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                result.push(ch);
                last_was_separator = false;
            } else if !last_was_separator {
                result.push('_');
                last_was_separator = true;
            }
        }

        let trimmed = result.trim_matches('_');
        if trimmed.is_empty() {
            "default".to_string()
        } else {
            trimmed.to_string()
        }
    }
}
