use super::locale::tr;
use super::{ProjectOpenNotice, ProjectOpenNoticeLevel, ReadyAppState};
use rf_store::FileOverwritePolicy;
use std::path::{Path, PathBuf};

pub(super) struct ExportTarget {
    pub(super) path: PathBuf,
    sha256: Option<String>,
}

impl ExportTarget {
    pub(super) fn requires_confirmation(&self) -> bool {
        self.sha256.is_some()
    }
}

impl ReadyAppState {
    pub(super) fn select_export_target(&mut self, path: PathBuf) -> Option<ExportTarget> {
        match save_target_fingerprint(&path) {
            Ok(sha256) => Some(ExportTarget { path, sha256 }),
            Err(error) => {
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: tr(
                        self.locale,
                        "无法检查导出目标",
                        "Cannot inspect export target",
                    )
                    .into(),
                    detail: error.to_string(),
                });
                None
            }
        }
    }

    pub(super) fn export_write_policy(
        &mut self,
        target: &ExportTarget,
    ) -> Option<FileOverwritePolicy> {
        let Some(expected) = target.sha256.as_deref() else {
            return Some(FileOverwritePolicy::Forbid);
        };
        let detail = match save_target_fingerprint(&target.path) {
            Ok(current) if current.as_deref() == Some(expected) => {
                return Some(FileOverwritePolicy::Allow);
            }
            Ok(_) => tr(
                self.locale,
                "目标文件已被改写或删除。",
                "The target file was changed or deleted.",
            )
            .into(),
            Err(error) => error.to_string(),
        };
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Warning,
            title: tr(
                self.locale,
                "导出覆盖确认已失效",
                "Export overwrite confirmation expired",
            )
            .into(),
            detail: format!(
                "{detail} {} ({})",
                tr(
                    self.locale,
                    "请重新选择导出目标。",
                    "Select the export target again."
                ),
                target.path.display()
            ),
        });
        None
    }
}

pub(super) fn save_target_fingerprint(path: &Path) -> rf_types::RfResult<Option<String>> {
    let inspect_error = |error: std::io::Error| {
        rf_types::RfError::invalid_input(format!("inspect save target {}: {error}", path.display()))
    };
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => std::fs::read(path)
            .map(|bytes| Some(rf_store::sha256_hex(&bytes)))
            .map_err(inspect_error),
        Ok(_) => Err(rf_types::RfError::invalid_input(format!(
            "save target {} is not a regular file",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(inspect_error(error)),
    }
}
