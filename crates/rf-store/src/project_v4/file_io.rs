use super::*;
use crate::{FileOverwritePolicy, sha256_hex, write_text_file};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Captures exactly the serialized input and target bytes being reviewed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedProjectV4Write {
    path: PathBuf,
    contents: String,
    support: FlowPlanSupport,
    equipment_support: EquipmentSupport,
    target_fingerprint: Option<String>,
    requires_upgrade: bool,
    target_has_object_identities: bool,
    replaces_target_object_identities: bool,
    replaces_target_variable_watch_list: bool,
    target_has_variable_watch_list: bool,
}
impl PreparedProjectV4Write {
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Check the same canonical project bytes that this preparation would write.
    pub fn captures_project(&self, project: &ProjectV4Read) -> RfResult<bool> {
        Ok(
            to_pretty_json_with_equipment_support(project, self.support, self.equipment_support)?
                == self.contents,
        )
    }
    pub fn target_fingerprint(&self) -> Option<&str> {
        self.target_fingerprint.as_deref()
    }
    pub fn requires_upgrade(&self) -> bool {
        self.requires_upgrade
    }
    pub fn target_has_object_identities(&self) -> bool {
        self.target_has_object_identities
    }
    pub fn replaces_target_object_identities(&self) -> bool {
        self.replaces_target_object_identities
    }
    pub fn replaces_target_variable_watch_list(&self) -> bool {
        self.replaces_target_variable_watch_list
    }
    pub fn target_has_variable_watch_list(&self) -> bool {
        self.target_has_variable_watch_list
    }
}
pub fn read(path: impl AsRef<Path>) -> RfResult<ProjectV4Read> {
    read_with_support(path, FlowPlanSupport::Mixed)
}

pub fn read_with_support(
    path: impl AsRef<Path>,
    support: FlowPlanSupport,
) -> RfResult<ProjectV4Read> {
    read_with_equipment_support(path, support, EquipmentSupport::SeparatorOnly)
}

pub fn read_with_equipment_support(
    path: impl AsRef<Path>,
    support: FlowPlanSupport,
    equipment_support: EquipmentSupport,
) -> RfResult<ProjectV4Read> {
    let contents = fs::read_to_string(path)
        .map_err(|e| RfError::invalid_input(format!("read v4 project: {e}")))?;
    parse_with_equipment_support(&contents, support, equipment_support)
}
pub fn prepare_write(
    path: impl AsRef<Path>,
    project: &ProjectV4Read,
    overwrite: FileOverwritePolicy,
) -> RfResult<PreparedProjectV4Write> {
    prepare_write_with_support(path, project, overwrite, FlowPlanSupport::Mixed)
}

pub fn prepare_write_with_support(
    path: impl AsRef<Path>,
    project: &ProjectV4Read,
    overwrite: FileOverwritePolicy,
    support: FlowPlanSupport,
) -> RfResult<PreparedProjectV4Write> {
    prepare_write_with_equipment_support(
        path,
        project,
        overwrite,
        support,
        EquipmentSupport::SeparatorOnly,
    )
}

pub fn prepare_write_with_equipment_support(
    path: impl AsRef<Path>,
    project: &ProjectV4Read,
    overwrite: FileOverwritePolicy,
    support: FlowPlanSupport,
    equipment_support: EquipmentSupport,
) -> RfResult<PreparedProjectV4Write> {
    let path = path.as_ref();
    let contents = to_pretty_json_with_equipment_support(project, support, equipment_support)?;
    let existing = target_bytes(path)?;
    if existing.is_some() && overwrite == FileOverwritePolicy::Forbid {
        return Err(RfError::invalid_input("project target already exists"));
    }
    let mut target_has_object_identities = false;
    let mut replaces_target_object_identities = false;
    let mut replaces_target_variable_watch_list = false;
    let mut target_has_variable_watch_list = false;
    let requires_upgrade = if let Some(bytes) = &existing {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| RfError::invalid_input(format!("project target UTF-8: {e}")))?;
        match crate::project_v3::probe_version(text)? {
            1 | 2 => {
                crate::parse_project_file_json(text)?;
                true
            }
            3 => match crate::project_v3::parse_with_equipment_support(text, equipment_support)? {
                crate::project_v3::ProjectV3Read::Editable(_) => true,
                _ => return Err(readonly_target()),
            },
            4 => match parse_with_equipment_support(text, support, equipment_support)? {
                ProjectV4Read::ReadOnly(_) => return Err(readonly_target()),
                ProjectV4Read::Editable(existing) => {
                    let ProjectV4Read::Editable(new) = project else {
                        unreachable!("validated editable write");
                    };
                    h3_flow_association::check_overwrite(&existing, new)?;
                    if existing.steady_state_plan.requires_product_composition()
                        && !new.steady_state_plan.requires_product_composition()
                    {
                        return Err(RfError::invalid_input(
                            "save the lower-capability plan to a different file; the target contains a product composition specification",
                        ));
                    }
                    if matches!(existing.steady_state_plan, StoredFlowPlan::SinglePhase(_))
                        && !matches!(new.steady_state_plan, StoredFlowPlan::SinglePhase(_))
                    {
                        return Err(RfError::invalid_input(
                            "save the lower-capability plan to a different file; the target contains a single-vapor sensible enthalpy plan",
                        ));
                    }
                    target_has_object_identities = existing.object_identities.is_some();
                    if target_has_object_identities && new.object_identities.is_none() {
                        return Err(RfError::invalid_input(
                            "existing project has object identities; overwriting without identities is prohibited",
                        ));
                    }
                    if let Some(old_watch_list) = &existing.variable_watch_list {
                        target_has_variable_watch_list = true;
                        let Some(new_watch_list) = &new.variable_watch_list else {
                            return Err(RfError::invalid_input(
                                "existing project has a variable watch list; overwriting without it is prohibited",
                            ));
                        };
                        replaces_target_variable_watch_list = old_watch_list != new_watch_list
                            || existing.document.metadata.document_id
                                != new.document.metadata.document_id;
                    }
                    if let (Some(old), Some(next)) =
                        (&existing.object_identities, &new.object_identities)
                    {
                        replaces_target_object_identities = existing.document.metadata.document_id
                            != new.document.metadata.document_id
                            || old
                                .units
                                .iter()
                                .any(|(id, creation)| next.units.get(id) != Some(creation))
                            || old
                                .streams
                                .iter()
                                .any(|(id, creation)| next.streams.get(id) != Some(creation));
                    }
                    let old: BTreeSet<_> = required_capabilities(&existing).into_iter().collect();
                    required_capabilities(new).iter().any(|c| !old.contains(c))
                        || new
                            .engineering
                            .as_ref()
                            .map_or(0, StoredEngineeringInputs::rating_method_version)
                            > existing
                                .engineering
                                .as_ref()
                                .map_or(0, StoredEngineeringInputs::rating_method_version)
                }
            },
            _ => {
                return Err(RfError::invalid_input(
                    "unsupported target project version; overwriting prohibited",
                ));
            }
        }
    } else {
        false
    };
    Ok(PreparedProjectV4Write {
        path: path.into(),
        contents,
        support,
        equipment_support,
        target_fingerprint: existing.as_ref().map(|b| sha256_hex(b)),
        requires_upgrade,
        target_has_object_identities,
        replaces_target_object_identities,
        replaces_target_variable_watch_list,
        target_has_variable_watch_list,
    })
}
pub fn write_prepared(prepared: &PreparedProjectV4Write, upgrade_confirmed: bool) -> RfResult<()> {
    if prepared.requires_upgrade && !upgrade_confirmed {
        return Err(RfError::invalid_input(
            "v4 format or capability upgrade requires confirmation",
        ));
    }
    let current = target_bytes(&prepared.path)?;
    if current.as_ref().map(|b| sha256_hex(b)) != prepared.target_fingerprint {
        return Err(
            RfError::invalid_input("project save target changed after preparation")
                .with_diagnostic_code("store.project.save_target_changed"),
        );
    }
    write_text_file(
        &prepared.path,
        &prepared.contents,
        if current.is_some() {
            FileOverwritePolicy::Allow
        } else {
            FileOverwritePolicy::Forbid
        },
    )
}
pub fn write(
    path: impl AsRef<Path>,
    project: &ProjectV4Read,
    overwrite: FileOverwritePolicy,
) -> RfResult<()> {
    write_prepared(&prepare_write(path, project, overwrite)?, false)
}
fn readonly_target() -> RfError {
    RfError::invalid_input("existing project has unsupported capabilities; overwriting prohibited")
}
fn target_bytes(path: &Path) -> RfResult<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(RfError::invalid_input(format!(
            "inspect project save target: {e}"
        ))),
    }
}
