use super::{
    EQUIPMENT_SUPPORT, FLOW_SUPPORT, H3FlowPreview, H3FlowReview, parse_project, rejected,
};
use rf_model::equipment::{ConcentricTubeFlowAssociation, ConcentricTubeRatingTask};
use rf_store::{
    FileOverwritePolicy,
    project_v4::{self, PreparedProjectV4Write, ProjectV4Read},
    sha256_hex,
};
use rf_types::{RfError, RfResult};
use std::{fs, path::Path, time::SystemTime};

/// Captures one confirmed stream selection and the exact bytes of its save target.
#[derive(Debug)]
pub struct PreparedH3FlowAssociationSave {
    preview: H3FlowPreview,
    write: PreparedProjectV4Write,
}

impl PreparedH3FlowAssociationSave {
    pub fn path(&self) -> &Path {
        self.write.path()
    }

    pub fn requires_upgrade(&self) -> bool {
        self.write.requires_upgrade()
    }
}

impl H3FlowReview {
    pub fn prepare_association_save(
        &self,
        preview: H3FlowPreview,
        target: &Path,
        overwrite: FileOverwritePolicy,
    ) -> RfResult<PreparedH3FlowAssociationSave> {
        let mut task = self.check_save_source(&preview)?.clone();
        let mut project = parse_project(&self.saved.original_json)?;
        // Keep the saved flowsheet and all unrelated records exactly as modeled.
        // Only the chosen case changes its inlet source.
        if task.case.flow_association.is_some() {
            return Err(rejected(
                "association_exists",
                "the H3 case already has a saved association",
            ));
        }
        task.revision = next_revision(task.revision)?;
        task.case.revision = next_revision(task.case.revision)?;
        task.case.hot.inlet = Default::default();
        task.case.cold.inlet = Default::default();
        task.case.flow_association = Some(ConcentricTubeFlowAssociation {
            hot: preview.hot.clone(),
            cold: preview.cold.clone(),
        });
        let entry = project
            .engineering
            .iter_mut()
            .flat_map(|inputs| &mut inputs.tasks)
            .find(|entry| entry.id() == &task.id)
            .ok_or_else(|| rejected("case_missing", "saved H3 task / case does not exist"))?;
        *entry = task.into();
        project.document.revision = next_revision(project.document.revision)?;
        project.document.metadata.updated_at =
            SystemTime::now().max(project.document.metadata.updated_at);
        let target = std::path::absolute(target).map_err(|error| {
            RfError::invalid_input(format!("resolve association save target: {error}"))
        })?;
        let write = project_v4::prepare_write_with_equipment_support(
            target,
            &ProjectV4Read::Editable(project),
            overwrite,
            FLOW_SUPPORT,
            EQUIPMENT_SUPPORT,
        )?;
        self.check_save_source(&preview)?;
        Ok(PreparedH3FlowAssociationSave { preview, write })
    }

    /// Confirm the prepared format upgrade, if required, and write the reviewed bytes.
    /// The saved binding takes effect when the destination is opened as a new session.
    pub fn save_association(
        &self,
        prepared: &PreparedH3FlowAssociationSave,
        upgrade_confirmed: bool,
    ) -> RfResult<()> {
        self.check_save_source(&prepared.preview)?;
        project_v4::write_prepared(&prepared.write, upgrade_confirmed)
    }

    fn check_save_source(&self, preview: &H3FlowPreview) -> RfResult<&ConcentricTubeRatingTask> {
        let task = self.current_case(preview)?;
        if !self
            .saved_flow_input
            .same_input(&self.workspace().flow_execution_qualification())
        {
            return Err(rejected(
                "source_changed",
                "association save requires a flow result from the saved source inputs",
            ));
        }
        let source =
            self.workspace().document_path.as_deref().ok_or_else(|| {
                rejected("source_changed", "the saved source path is unavailable")
            })?;
        let bytes = fs::read(source).map_err(|error| {
            rejected(
                "source_changed",
                format!("cannot verify saved source {}: {error}", source.display()),
            )
        })?;
        if sha256_hex(&bytes) != self.saved.file_sha256() {
            return Err(rejected(
                "source_changed",
                "the saved source file changed after opening",
            ));
        }
        Ok(task)
    }
}

fn next_revision(revision: u64) -> RfResult<u64> {
    revision.checked_add(1).ok_or_else(|| {
        rejected(
            "revision_exhausted",
            "project, task or case revision exhausted",
        )
    })
}
