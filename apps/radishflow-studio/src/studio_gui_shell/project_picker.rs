use std::path::PathBuf;

pub(super) trait ProjectFilePicker {
    fn pick_project_file(&mut self) -> Option<PathBuf>;
}

#[derive(Debug, Default)]
pub(super) struct NativeProjectFilePicker;

impl ProjectFilePicker for NativeProjectFilePicker {
    fn pick_project_file(&mut self) -> Option<PathBuf> {
        pick_native_project_file()
    }
}

#[cfg(target_os = "windows")]
fn pick_native_project_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Open RadishFlow project")
        .add_filter("RadishFlow project", &["rfproj.json"])
        .pick_file()
}

#[cfg(not(target_os = "windows"))]
fn pick_native_project_file() -> Option<PathBuf> {
    None
}
