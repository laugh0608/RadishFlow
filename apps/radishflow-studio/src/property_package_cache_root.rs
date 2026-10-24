use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rf_types::{RfError, RfResult};

/// A cache path lease. Temporary directories are deleted only after the last
/// loading task and owning session release their lease.
#[derive(Debug, Clone)]
pub struct PropertyPackageCacheRoot(Arc<CacheRootStorage>);

#[derive(Debug)]
struct CacheRootStorage {
    path: PathBuf,
    temporary: bool,
}

static TEMPORARY_CACHE_ROOT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

impl PropertyPackageCacheRoot {
    pub fn temporary(prefix: &str) -> RfResult<Self> {
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
                Ok(()) => {
                    return Ok(Self(Arc::new(CacheRootStorage {
                        path,
                        temporary: true,
                    })));
                }
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

    /// Persistent cache lifetime is controlled by its external owner.
    pub fn persistent(path: impl Into<PathBuf>) -> Self {
        Self(Arc::new(CacheRootStorage {
            path: path.into(),
            temporary: false,
        }))
    }

    pub fn path(&self) -> &Path {
        &self.0.path
    }
}

impl Drop for CacheRootStorage {
    fn drop(&mut self) {
        if self.temporary {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
