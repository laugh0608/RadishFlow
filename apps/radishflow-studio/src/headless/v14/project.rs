use super::super::{
    project_file::{ProjectFile, read_project_file},
    v3::protocol::*,
};
use crate::document_lifecycle_driver::h3_flow_review::{EQUIPMENT_SUPPORT, FLOW_SUPPORT};
use rf_store::project_v4::{self, ProjectV4Read, StoredFlowPlan, StoredProjectV4};
use rf_types::RfError;
use std::path::{Path, PathBuf};

pub(super) struct ReviewProject {
    pub path: PathBuf,
    pub contents: String,
    pub data: Box<StoredProjectV4>,
}

pub(super) fn load_project(
    request: &Path,
    project: &Path,
) -> Result<(ProjectSnapshot, Option<ReviewProject>), Failure> {
    if project.as_os_str().is_empty() {
        return Err(Failure::new(
            Stage::Request,
            Code::InvalidRequest,
            "project path must not be empty",
        ));
    }
    let path = request.parent().unwrap_or(Path::new(".")).join(project);
    let ProjectFile { contents, sha256 } = read_project_file(&path)?;
    let invalid =
        |error: RfError| Failure::new(Stage::Project, Code::InvalidProject, error.to_string());
    let version = rf_store::project_v3::probe_version(&contents).map_err(invalid)?;
    if version != project_v4::VERSION {
        return Err(Failure::new(
            Stage::Project,
            Code::UnsupportedCapability,
            "CLI v14 requires project schema version 4",
        ));
    }
    let read = project_v4::parse_with_equipment_support(&contents, FLOW_SUPPORT, EQUIPMENT_SUPPORT)
        .map_err(invalid)?;
    let snapshot = |document_id: String, flow_revision, read_only, count| ProjectSnapshot {
        document_id,
        project_schema_version: version,
        flow_revision,
        file_sha256: sha256,
        read_only,
        read_only_reason: read_only.then_some(Code::UnsupportedCapability),
        equipment_task_count: count,
        in_memory_required_project_schema_version: None,
        in_memory_required_capabilities: None,
    };
    match read {
        ProjectV4Read::ReadOnly(p) => Ok((
            snapshot(
                p.metadata().document_id.to_string(),
                p.revision(),
                true,
                None,
            ),
            None,
        )),
        ProjectV4Read::Editable(data) => {
            let document_id = data.document.metadata.document_id.to_string();
            let revision = data.document.revision;
            if !matches!(data.steady_state_plan, StoredFlowPlan::Acyclic) {
                return Ok((snapshot(document_id, revision, true, None), None));
            }
            let count = data
                .engineering
                .as_ref()
                .map_or(0, |engineering| engineering.tasks.len());
            Ok((
                snapshot(document_id, revision, false, Some(count)),
                Some(ReviewProject {
                    path,
                    contents,
                    data,
                }),
            ))
        }
    }
}
