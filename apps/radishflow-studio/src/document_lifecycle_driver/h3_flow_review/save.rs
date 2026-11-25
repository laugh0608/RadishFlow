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
use rf_ui::engineering::EquipmentCaseKey;
use std::{fs, path::Path, time::SystemTime};

/// An editor candidate qualified against one accepted flow result and the old binding.
#[derive(Debug)]
pub struct H3AssociationPreview {
    flow: H3FlowPreview,
    previous: Option<ConcentricTubeFlowAssociation>,
}

impl H3AssociationPreview {
    pub fn flow(&self) -> &H3FlowPreview {
        &self.flow
    }
}

pub enum H3AssociationSaveTarget<'a> {
    Source,
    NewFile(&'a Path),
}

/// Captures one confirmed stream selection and the exact bytes of its save target.
#[derive(Debug)]
pub struct PreparedH3FlowAssociationSave {
    preview: H3FlowPreview,
    previous: Option<ConcentricTubeFlowAssociation>,
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
    pub fn preview_association_edit(
        &self,
        member: EquipmentCaseKey,
        pair: ConcentricTubeFlowAssociation,
    ) -> RfResult<H3AssociationPreview> {
        let task = self.saved.case(&member).ok_or_else(super::inputs_changed)?;
        let previous = task.case.flow_association.clone();
        if previous.as_ref() == Some(&pair) {
            return Err(rejected(
                "association_unchanged",
                "the saved association is unchanged",
            ));
        }
        let flow = self.preview_pair(task, member, pair)?;
        Ok(H3AssociationPreview { flow, previous })
    }

    pub fn prepare_association_edit_save(
        &self,
        preview: &H3AssociationPreview,
        destination: H3AssociationSaveTarget<'_>,
    ) -> RfResult<PreparedH3FlowAssociationSave> {
        let (path, overwrite) = match destination {
            H3AssociationSaveTarget::Source => (self.source_path()?, FileOverwritePolicy::Allow),
            H3AssociationSaveTarget::NewFile(path) => {
                if path.as_os_str().is_empty()
                    || path.try_exists().map_err(|error| {
                        RfError::invalid_input(format!("inspect association save target: {error}"))
                    })?
                {
                    return Err(rejected("copy_target_exists", "choose a new project file"));
                }
                (path, FileOverwritePolicy::Forbid)
            }
        };
        self.prepare_save(
            preview.flow.clone(),
            preview.previous.clone(),
            path,
            overwrite,
        )
    }

    pub fn prepare_association_save(
        &self,
        preview: H3FlowPreview,
        target: &Path,
        overwrite: FileOverwritePolicy,
    ) -> RfResult<PreparedH3FlowAssociationSave> {
        if self.current_case(&preview)?.case.flow_association.is_some() {
            return Err(rejected(
                "association_exists",
                "the H3 case already has a saved association",
            ));
        }
        self.prepare_save(preview, None, target, overwrite)
    }

    fn prepare_save(
        &self,
        preview: H3FlowPreview,
        previous: Option<ConcentricTubeFlowAssociation>,
        target: &Path,
        overwrite: FileOverwritePolicy,
    ) -> RfResult<PreparedH3FlowAssociationSave> {
        let mut task = self.check_save_source(&preview, previous.as_ref())?.clone();
        let mut project = parse_project(&self.saved.original_json)?;
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
        self.check_save_source(&preview, previous.as_ref())?;
        Ok(PreparedH3FlowAssociationSave {
            preview,
            previous,
            write,
        })
    }

    /// Confirm the prepared format upgrade, if required, and write the reviewed bytes.
    /// The saved binding takes effect when the destination is opened as a new session.
    pub fn save_association(
        &self,
        prepared: &PreparedH3FlowAssociationSave,
        upgrade_confirmed: bool,
    ) -> RfResult<()> {
        self.check_save_source(&prepared.preview, prepared.previous.as_ref())?;
        project_v4::write_prepared(&prepared.write, upgrade_confirmed)
    }

    fn check_save_source(
        &self,
        preview: &H3FlowPreview,
        previous: Option<&ConcentricTubeFlowAssociation>,
    ) -> RfResult<&ConcentricTubeRatingTask> {
        let task = self.current_case(preview)?;
        if task.case.flow_association.as_ref() != previous {
            return Err(super::inputs_changed());
        }
        if !self
            .saved_flow_input
            .same_input(&self.workspace().flow_execution_qualification())
        {
            return Err(rejected(
                "source_changed",
                "association save requires a flow result from the saved source inputs",
            ));
        }
        self.check_source_bytes()?;
        Ok(task)
    }

    pub(super) fn source_path(&self) -> RfResult<&Path> {
        self.workspace()
            .document_path
            .as_deref()
            .ok_or_else(|| rejected("source_changed", "the saved source path is unavailable"))
    }

    pub(super) fn check_source_bytes(&self) -> RfResult<()> {
        let source = self.source_path()?;
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
        Ok(())
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
