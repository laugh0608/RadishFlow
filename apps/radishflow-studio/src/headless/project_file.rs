//! Read and fingerprint the same bytes for every saved-project protocol.

use super::v3::{
    actions::ExpectedProject,
    protocol::{Code, Failure, ProjectSnapshot, Stage, Status},
};
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

pub(super) fn check_expected(
    project: &ProjectSnapshot,
    expected: &ExpectedProject,
) -> Result<(), Failure> {
    if expected.document_id != project.document_id {
        return Err(Failure::new(
            Stage::Project,
            Code::DifferentDocument,
            "document id differs",
        ));
    }
    if expected.file_sha256 != project.file_sha256 {
        return Err(Failure::new(
            Stage::Project,
            Code::ProjectConflict,
            "project file snapshot differs",
        ));
    }
    Ok(())
}

pub(super) fn read_only(stage: Stage) -> Failure {
    let mut error = Failure::new(
        stage,
        Code::ReadOnlyProject,
        "unsupported capability protects the whole project",
    );
    error.status = Status::Blocked;
    error
}
