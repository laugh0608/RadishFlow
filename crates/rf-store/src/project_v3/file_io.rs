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

/// Foundation storage only. The caller must explicitly allow replacement; this
/// does not enable product upgrades. Existing legacy, invalid or unsupported-task
/// files are protected even if the caller supplies a new task-free document.
/// Replacement uses the repository's existing staged-write concurrency semantics.
pub fn write(
    path: impl AsRef<Path>,
    project: &ProjectV3Read,
    overwrite: FileOverwritePolicy,
) -> RfResult<()> {
    let contents = to_pretty_json(project)?;
    let path = path.as_ref();
    let policy = if overwrite == FileOverwritePolicy::Allow {
        match fs::read_to_string(path) {
            Ok(existing) => match parse(&existing)? {
                ProjectV3Read::Foundation(_) => FileOverwritePolicy::Allow,
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
