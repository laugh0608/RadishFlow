use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rf_types::{RfError, RfResult};

#[derive(Debug)]
pub(super) struct TemporaryCacheRoot {
    path: PathBuf,
}

static TEMPORARY_CACHE_ROOT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

impl TemporaryCacheRoot {
    pub(super) fn new(prefix: &str) -> RfResult<Self> {
        let temp_dir = std::env::temp_dir();
        for _ in 0..32 {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| {
                    RfError::invalid_input(format!(
                        "create temporary cache root timestamp failed: {error}"
                    ))
                })?
                .as_nanos();
            let sequence = TEMPORARY_CACHE_ROOT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = temp_dir.join(format!(
                "radishflow-{prefix}-{}-{timestamp}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(RfError::invalid_input(format!(
                        "create temporary cache root `{}`: {error}",
                        path.display()
                    )));
                }
            }
        }

        Err(RfError::invalid_input(format!(
            "create temporary cache root for prefix `{prefix}` exhausted retry attempts"
        )))
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryCacheRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
