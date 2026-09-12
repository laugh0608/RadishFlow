use std::{fs, io, path::Path};

use rf_types::{FileRecoveryContext, RfError, RfResult};

use super::map_io_error;

// A rename operation is injected so the Windows recovery sequence can be tested
// deterministically on every platform, without changing global filesystem state.
pub(super) fn replace_existing_file(
    staged_path: &Path,
    target_path: &Path,
    backup_path: &Path,
    action: &str,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> RfResult<()> {
    rename(target_path, backup_path).map_err(|error| map_io_error(action, target_path, &error))?;

    match rename(staged_path, target_path) {
        Ok(()) => {
            let _ = fs::remove_file(backup_path);
            Ok(())
        }
        Err(replacement_error) => match rename(backup_path, target_path) {
            Ok(()) => Err(map_io_error(action, target_path, &replacement_error)),
            Err(recovery_error) => {
                let recovery = FileRecoveryContext {
                    target_path: target_path.to_path_buf(),
                    backup_path: backup_path.to_path_buf(),
                    replacement_error: replacement_error.to_string(),
                    recovery_error: recovery_error.to_string(),
                };
                Err(RfError::invalid_input(format!(
                    "{action} `{}`: replacement failed: {}; restoring backup `{}` failed: {}",
                    target_path.display(),
                    recovery.replacement_error,
                    backup_path.display(),
                    recovery.recovery_error,
                ))
                .with_diagnostic_code("store.file_replace.recovery_failed")
                .with_file_recovery(recovery))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Files {
        directory: PathBuf,
        target: PathBuf,
        staged: PathBuf,
        backup: PathBuf,
    }

    impl Files {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let directory = std::env::temp_dir().join(format!(
                "rf-store-recovery-{}-{nonce}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&directory).unwrap();
            let files = Self {
                target: directory.join("project.json"),
                staged: directory.join("project.tmp"),
                backup: directory.join("project.bak"),
                directory,
            };
            fs::write(&files.target, br#"{"revision":1}"#).unwrap();
            fs::write(&files.staged, br#"{"revision":2}"#).unwrap();
            files
        }

        fn replace(&self, failed_calls: &[usize]) -> RfResult<()> {
            let mut calls = 0;
            replace_existing_file(
                &self.staged,
                &self.target,
                &self.backup,
                "save project",
                |from, to| {
                    calls += 1;
                    if failed_calls.contains(&calls) {
                        Err(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            format!("injected rename {calls}"),
                        ))
                    } else {
                        fs::rename(from, to)
                    }
                },
            )
        }
    }

    impl Drop for Files {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.directory).unwrap();
        }
    }

    fn revision(path: &Path) -> u64 {
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        value["revision"].as_u64().unwrap()
    }

    #[test]
    fn successful_replacement_reopens_new_content_and_removes_backup() {
        let files = Files::new();
        files.replace(&[]).unwrap();
        assert_eq!(revision(&files.target), 2);
        assert!(!files.backup.exists());
        assert!(!files.staged.exists());
    }

    #[test]
    fn failed_backup_move_leaves_original_untouched() {
        let files = Files::new();
        let error = files.replace(&[1]).unwrap_err();
        assert!(error.message().contains("injected rename 1"));
        assert!(error.context().file_recovery().is_none());
        assert_eq!(revision(&files.target), 1);
        assert_eq!(revision(&files.staged), 2);
        assert!(!files.backup.exists());
    }

    #[test]
    fn failed_replacement_restores_original_for_reopen() {
        let files = Files::new();
        let error = files.replace(&[2]).unwrap_err();
        assert!(error.message().contains("injected rename 2"));
        assert!(error.context().file_recovery().is_none());
        assert_eq!(revision(&files.target), 1);
        assert!(!files.backup.exists());
    }

    #[test]
    fn failed_recovery_preserves_both_errors_and_recoverable_backup() {
        let files = Files::new();
        let error = files.replace(&[2, 3]).unwrap_err();
        assert_eq!(
            error.context().diagnostic_code(),
            Some("store.file_replace.recovery_failed")
        );
        let recovery = error.context().file_recovery().unwrap();
        assert_eq!(recovery.target_path, files.target);
        assert_eq!(recovery.backup_path, files.backup);
        assert!(recovery.replacement_error.contains("injected rename 2"));
        assert!(recovery.recovery_error.contains("injected rename 3"));
        assert!(
            error
                .message()
                .contains(&files.backup.display().to_string())
        );
        assert!(error.message().contains(&recovery.replacement_error));
        assert!(error.message().contains(&recovery.recovery_error));
        assert!(!files.target.exists());
        assert_eq!(revision(&recovery.backup_path), 1);
        fs::rename(&recovery.backup_path, &recovery.target_path).unwrap();
        assert_eq!(revision(&files.target), 1);
    }
}
