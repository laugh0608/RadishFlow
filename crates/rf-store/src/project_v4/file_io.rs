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
    target_fingerprint: Option<String>,
    requires_upgrade: bool,
}
impl PreparedProjectV4Write {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn target_fingerprint(&self) -> Option<&str> {
        self.target_fingerprint.as_deref()
    }
    pub fn requires_upgrade(&self) -> bool {
        self.requires_upgrade
    }
}
pub fn read(path: impl AsRef<Path>) -> RfResult<ProjectV4Read> {
    let contents = fs::read_to_string(path)
        .map_err(|e| RfError::invalid_input(format!("read v4 project: {e}")))?;
    parse(&contents)
}
pub fn prepare_write(
    path: impl AsRef<Path>,
    project: &ProjectV4Read,
    overwrite: FileOverwritePolicy,
) -> RfResult<PreparedProjectV4Write> {
    let path = path.as_ref();
    let contents = to_pretty_json(project)?;
    let existing = target_bytes(path)?;
    if existing.is_some() && overwrite == FileOverwritePolicy::Forbid {
        return Err(RfError::invalid_input("project target already exists"));
    }
    let requires_upgrade = if let Some(bytes) = &existing {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| RfError::invalid_input(format!("project target UTF-8: {e}")))?;
        match crate::project_v3::probe_version(text)? {
            1 | 2 => {
                crate::parse_project_file_json(text)?;
                true
            }
            3 => match crate::project_v3::parse(text)? {
                crate::project_v3::ProjectV3Read::Editable(_) => true,
                _ => return Err(readonly_target()),
            },
            4 => match parse(text)? {
                ProjectV4Read::ReadOnly(_) => return Err(readonly_target()),
                ProjectV4Read::Editable(existing) => {
                    let ProjectV4Read::Editable(new) = project else {
                        unreachable!("validated editable write");
                    };
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
        target_fingerprint: existing.as_ref().map(|b| sha256_hex(b)),
        requires_upgrade,
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
