use std::path::PathBuf;

pub(super) trait ProjectFilePicker {
    fn supports_project_dialogs(&self) -> bool {
        true
    }
    fn pick_project_file(&mut self) -> Option<PathBuf>;
    fn pick_save_project_file(&mut self) -> Option<PathBuf>;
    fn pick_result_export_file(&mut self) -> Option<PathBuf>;
}

#[derive(Debug, Default)]
pub(super) struct NativeProjectFilePicker;

impl ProjectFilePicker for NativeProjectFilePicker {
    fn supports_project_dialogs(&self) -> bool {
        cfg!(any(target_os = "windows", target_os = "macos"))
    }
    fn pick_project_file(&mut self) -> Option<PathBuf> {
        pick_native_project_file()
    }

    fn pick_save_project_file(&mut self) -> Option<PathBuf> {
        pick_native_save_project_file()
    }

    fn pick_result_export_file(&mut self) -> Option<PathBuf> {
        pick_native_result_export_file()
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn pick_native_project_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open RadishFlow project")
        .add_filter("RadishFlow project", &["rfproj.json"])
        .pick_file()
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn pick_native_save_project_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Save RadishFlow project")
        .add_filter("RadishFlow project", &["rfproj.json"])
        .save_file()
        .map(ensure_project_file_extension)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn pick_native_project_file() -> Option<PathBuf> {
    None
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn pick_native_save_project_file() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "windows")]
fn pick_native_result_export_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export RadishFlow solve snapshot")
        .add_filter("Text", &["txt"])
        .save_file()
        .map(ensure_text_file_extension)
}

#[cfg(not(target_os = "windows"))]
fn pick_native_result_export_file() -> Option<PathBuf> {
    None
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn ensure_project_file_extension(path: PathBuf) -> PathBuf {
    let value = path.to_string_lossy();
    if value.ends_with(rf_store::STORED_PROJECT_FILE_EXTENSION) {
        return path;
    }
    PathBuf::from(format!(
        "{}{}",
        value,
        rf_store::STORED_PROJECT_FILE_EXTENSION
    ))
}

#[cfg(target_os = "windows")]
fn ensure_text_file_extension(path: PathBuf) -> PathBuf {
    let value = path.to_string_lossy();
    if value.ends_with(".txt") {
        return path;
    }
    PathBuf::from(format!("{value}.txt"))
}

impl super::ReadyAppState {
    pub(super) fn project_dialogs_available(&mut self) -> bool {
        if self.project_file_picker.supports_project_dialogs() {
            return true;
        }
        self.project_open.notice = Some(super::ProjectOpenNotice {
            level: super::ProjectOpenNoticeLevel::Warning,
            title: "文件选择器不可用".to_string(),
            detail: "当前平台尚未接入原生项目文件选择器，工作区保持不变。".to_string(),
        });
        false
    }
}
