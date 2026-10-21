use super::{ProjectV3Read, parse, to_pretty_json};
use crate::{FileOverwritePolicy, write_text_file};
use rf_types::{RfError, RfResult};
use std::{fs, path::Path};

pub fn read(path: impl AsRef<Path>) -> RfResult<ProjectV3Read> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|e| RfError::invalid_input(format!("read project v3 {}: {e}", path.display())))?;
    parse(&contents)
}

/// Replacement requires explicit consent. Legacy upgrades use write_upgraded;
/// invalid and unsupported-task targets are protected by both entry points.
/// Replacement uses the repository's existing staged-write concurrency semantics.
pub fn write(
    path: impl AsRef<Path>,
    project: &ProjectV3Read,
    overwrite: FileOverwritePolicy,
) -> RfResult<()> {
    write_with_upgrade(path.as_ref(), project, overwrite, false)
}

/// Explicit lifecycle consent to upgrade a validated v1/v2 target or add rating
/// capability to a known v3 target. Unknown capabilities can never be overwritten.
pub fn write_upgraded(
    path: impl AsRef<Path>,
    project: &ProjectV3Read,
    overwrite: FileOverwritePolicy,
) -> RfResult<()> {
    write_with_upgrade(path.as_ref(), project, overwrite, true)
}
fn write_with_upgrade(
    path: &Path,
    project: &ProjectV3Read,
    overwrite: FileOverwritePolicy,
    upgrade: bool,
) -> RfResult<()> {
    let contents = to_pretty_json(project)?;
    let policy = if overwrite == FileOverwritePolicy::Allow {
        match fs::read_to_string(path) {
            Ok(existing) if super::probe_version(&existing)? != super::VERSION && upgrade => {
                crate::parse_project_file_json(&existing)?;
                FileOverwritePolicy::Allow
            }
            Ok(existing) => match parse(&existing)? {
                ProjectV3Read::Editable(existing) => {
                    if !upgrade
                        && matches!(project, ProjectV3Read::Editable(p) if p.rating_method_version() > existing.rating_method_version())
                    {
                        return Err(RfError::invalid_input(
                            "engineering.tasks: raising rating capability requires confirmation; older v3 readers will open the file read-only",
                        ));
                    }
                    FileOverwritePolicy::Allow
                }
                ProjectV3Read::ReadOnly(_) => {
                    return Err(RfError::invalid_input(
                        "engineering.tasks: existing project is read-only; overwriting or stripping tasks is prohibited",
                    ));
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => FileOverwritePolicy::Forbid,
            Err(e) => {
                return Err(RfError::invalid_input(format!(
                    "inspect project v3 target {}: {e}",
                    path.display()
                )));
            }
        }
    } else {
        FileOverwritePolicy::Forbid
    };
    write_text_file(path, &contents, policy)
}
