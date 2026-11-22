use super::super::{project_file::read_project_file, v3::protocol::*};
use rf_store::{
    StoredProjectDocument,
    project_v3::{self, EquipmentSupport, ProjectV3Read, StoredEngineeringInputs},
    project_v4::{self, FlowPlanSupport, ProjectV4Read},
};
use std::path::Path;

const EQUIPMENT: EquipmentSupport = EquipmentSupport::ConcentricTubeRating;

pub(super) fn load_project(
    request: &Path,
    project: &Path,
) -> Result<(ProjectSnapshot, Option<StoredEngineeringInputs>), Failure> {
    if project.as_os_str().is_empty() {
        return Err(Failure::new(
            Stage::Request,
            Code::InvalidRequest,
            "project path must not be empty",
        ));
    }
    let path = request.parent().unwrap_or(Path::new(".")).join(project);
    let file = read_project_file(&path)?;
    let invalid = |error: rf_types::RfError| {
        Failure::new(Stage::Project, Code::InvalidProject, error.to_string())
    };
    let version = project_v3::probe_version(&file.contents).map_err(invalid)?;
    let snapshot = |document_id, flow_revision, count, read_only| ProjectSnapshot {
        document_id,
        project_schema_version: version,
        flow_revision,
        file_sha256: file.sha256.clone(),
        read_only,
        read_only_reason: read_only.then_some(Code::UnsupportedCapability),
        equipment_task_count: count,
        in_memory_required_project_schema_version: None,
        in_memory_required_capabilities: None,
    };
    let editable = |document: StoredProjectDocument, inputs: Option<StoredEngineeringInputs>| {
        let inputs = inputs.unwrap_or_default();
        let project = snapshot(
            document.metadata.document_id.to_string(),
            document.revision,
            Some(inputs.tasks.len()),
            false,
        );
        (project, Some(inputs))
    };
    match version {
        project_v3::VERSION => {
            match project_v3::parse_with_equipment_support(&file.contents, EQUIPMENT)
                .map_err(invalid)?
            {
                ProjectV3Read::Editable(p) => Ok(editable(p.document, p.engineering)),
                ProjectV3Read::ReadOnly(p) => Ok((
                    snapshot(
                        p.document().metadata.document_id.to_string(),
                        p.document().revision,
                        Some(p.task_count()),
                        true,
                    ),
                    None,
                )),
            }
        }
        project_v4::VERSION => {
            match project_v4::parse_with_equipment_support(
                &file.contents,
                FlowPlanSupport::SinglePhaseRecycle,
                EQUIPMENT,
            )
            .map_err(invalid)?
            {
                ProjectV4Read::Editable(p) => Ok(editable(p.document, p.engineering)),
                ProjectV4Read::ReadOnly(p) => Ok((
                    snapshot(
                        p.metadata().document_id.to_string(),
                        p.revision(),
                        None,
                        true,
                    ),
                    None,
                )),
            }
        }
        _ => Err(Failure::new(
            Stage::Project,
            Code::UnsupportedCapability,
            "CLI v13 requires project schema version 3 or 4",
        )),
    }
}
