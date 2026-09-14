use rf_types::{RfError, RfResult};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(any(windows, test))]
mod staged_replace;

/// Whether a completed staged write may replace a destination file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOverwritePolicy {
    Forbid,
    Allow,
}

/// Writes UTF-8 text through a synced sibling file, preserving the destination on failure.
/// The parent directory must already exist. Forbid also rejects a destination created
/// concurrently with this operation. Windows recovery failures retain a backup path in the error.
pub fn write_text_file(
    path: impl AsRef<Path>,
    contents: &str,
    overwrite: FileOverwritePolicy,
) -> RfResult<()> {
    write_staged_file(
        path.as_ref(),
        contents.as_bytes(),
        "write text file",
        overwrite,
    )
}

pub(crate) fn write_staged_file(
    path: &Path,
    contents: &[u8],
    action: &str,
    overwrite: FileOverwritePolicy,
) -> RfResult<()> {
    write_staged_file_with(path, contents, action, overwrite, write_all_and_sync)
}

fn write_staged_file_with(
    path: &Path,
    contents: &[u8],
    action: &str,
    overwrite: FileOverwritePolicy,
    write: impl FnOnce(File, &Path, &[u8], &str) -> RfResult<()>,
) -> RfResult<()> {
    if path.exists() && !path.is_file() {
        return Err(RfError::invalid_input(format!(
            "{action} `{}`: target path exists and is not a file",
            path.display()
        )));
    }

    let (temp_path, file) = create_unique_temp_sibling(path)?;
    let write_result = write(file, &temp_path, contents, action).and_then(|_| match overwrite {
        FileOverwritePolicy::Allow => replace_with_staged_file(&temp_path, path, action),
        FileOverwritePolicy::Forbid => {
            fs::hard_link(&temp_path, path).map_err(|error| map_io_error(action, path, &error))?;
            let _ = fs::remove_file(&temp_path);
            Ok(())
        }
    });

    if write_result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    write_result
}

fn write_all_and_sync(mut file: File, path: &Path, contents: &[u8], action: &str) -> RfResult<()> {
    file.write_all(contents)
        .map_err(|error| map_io_error(action, path, &error))?;
    file.sync_all()
        .map_err(|error| map_io_error(action, path, &error))
}

#[cfg(not(windows))]
fn replace_with_staged_file(temp_path: &Path, path: &Path, action: &str) -> RfResult<()> {
    fs::rename(temp_path, path).map_err(|error| map_io_error(action, path, &error))
}

#[cfg(windows)]
fn replace_with_staged_file(temp_path: &Path, path: &Path, action: &str) -> RfResult<()> {
    if !path.exists() {
        return fs::rename(temp_path, path).map_err(|error| map_io_error(action, path, &error));
    }

    let backup_path = create_unique_backup_sibling(path)?;
    staged_replace::replace_existing_file(temp_path, path, &backup_path, action, |from, to| {
        fs::rename(from, to)
    })
}

fn create_unique_temp_sibling(path: &Path) -> RfResult<(PathBuf, File)> {
    create_unique_sibling(path, "tmp")
}

#[cfg(windows)]
fn create_unique_backup_sibling(path: &Path) -> RfResult<PathBuf> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("radishflow");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = process::id();

    for attempt in 0..32 {
        let candidate = directory.join(format!(".{file_name}.{pid}.{timestamp}.{attempt}.bak"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(RfError::invalid_input(format!(
        "create staged backup file `{}`: could not allocate a unique sibling path",
        path.display()
    )))
}

fn create_unique_sibling(path: &Path, suffix: &str) -> RfResult<(PathBuf, File)> {
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("radishflow");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = process::id();

    for attempt in 0..32 {
        let candidate =
            directory.join(format!(".{file_name}.{pid}.{timestamp}.{attempt}.{suffix}"));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(map_io_error("create staged file", &candidate, &error));
            }
        }
    }

    Err(RfError::invalid_input(format!(
        "create staged file `{}`: could not allocate a unique sibling path",
        path.display()
    )))
}

pub(crate) fn map_io_error(action: &str, path: &Path, error: &std::io::Error) -> RfError {
    RfError::invalid_input(format!("{action} `{}`: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_file_creation_rejects_existing_target_and_leaves_no_staging_file() {
        let directory = std::env::temp_dir().join(format!(
            "rf-text-file-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("结果.txt");
        write_text_file(&path, "original", FileOverwritePolicy::Forbid).unwrap();
        assert!(write_text_file(&path, "replacement", FileOverwritePolicy::Forbid).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        write_text_file(&path, "替换文本", FileOverwritePolicy::Allow).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "替换文本");
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn text_file_failed_staging_preserves_target() {
        let directory = std::env::temp_dir().join(format!(
            "rf-text-failure-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("result.txt");
        fs::write(&path, "original").unwrap();
        let result = write_staged_file_with(
            &path,
            b"replacement",
            "test write",
            FileOverwritePolicy::Allow,
            |mut file, staged, _, action| {
                file.write_all(b"partial replacement").unwrap();
                Err(map_io_error(
                    action,
                    staged,
                    &std::io::Error::other("injected write failure"),
                ))
            },
        );
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "original");
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }
}
