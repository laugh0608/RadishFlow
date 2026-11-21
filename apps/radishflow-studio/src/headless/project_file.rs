//! Read and fingerprint the same bytes for every saved-project protocol.

use super::v3::protocol::{Code, Failure, Stage};
use std::path::Path;

pub(super) struct ProjectFile {
    pub contents: String,
    pub sha256: String,
}

pub(super) fn read_project_file(path: &Path) -> Result<ProjectFile, Failure> {
    let bytes = std::fs::read(path)
        .map_err(|e| Failure::new(Stage::Project, Code::ProjectIo, e.to_string()))?;
    let sha256 = rf_store::sha256_hex(&bytes);
    let contents = String::from_utf8(bytes).map_err(|e| {
        Failure::new(
            Stage::Project,
            Code::InvalidProject,
            e.utf8_error().to_string(),
        )
    })?;
    Ok(ProjectFile { contents, sha256 })
}
