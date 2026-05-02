use std::path::PathBuf;

pub(super) trait ProjectFilePicker {
    fn pick_project_file(&mut self) -> Option<PathBuf>;
    fn pick_save_project_file(&mut self) -> Option<PathBuf>;
}

#[derive(Debug, Default)]
pub(super) struct NativeProjectFilePicker;

impl ProjectFilePicker for NativeProjectFilePicker {
    fn pick_project_file(&mut self) -> Option<PathBuf> {
        pick_native_project_file()
    }

    fn pick_save_project_file(&mut self) -> Option<PathBuf> {
        pick_native_save_project_file()
    }
}

#[cfg(target_os = "windows")]
fn pick_native_project_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open RadishFlow project")
        .add_filter("RadishFlow project", &["rfproj.json"])
        .pick_file()
}

#[cfg(target_os = "windows")]
fn pick_native_save_project_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Save RadishFlow project")
        .add_filter("RadishFlow project", &["rfproj.json"])
        .save_file()
        .map(ensure_project_file_extension)
}

#[cfg(not(target_os = "windows"))]
fn pick_native_project_file() -> Option<PathBuf> {
    None
}

#[cfg(not(target_os = "windows"))]
fn pick_native_save_project_file() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "windows")]
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
