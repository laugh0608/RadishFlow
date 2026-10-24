use std::path::PathBuf;

pub(super) trait ProjectFilePicker {
    fn supports_equipment_export_dialogs(&self) -> bool {
        false
    }
    fn pick_equipment_export_file(
        &mut self,
        _format: super::equipment::CalculationTableFormat,
    ) -> Option<PathBuf> {
        None
    }
    fn supports_project_dialogs(&self) -> bool {
        true
    }
    fn supports_result_export_dialogs(&self) -> bool {
        true
    }
    fn pick_project_file(&mut self) -> Option<PathBuf>;
    fn pick_save_project_file(&mut self) -> Option<PathBuf>;
    fn pick_result_export_file(&mut self) -> Option<PathBuf>;
}

#[derive(Debug, Default)]
pub(super) struct NativeProjectFilePicker {
    dialog_sequence: u64,
}

impl NativeProjectFilePicker {
    fn trace_selection(
        &mut self,
        operation: &str,
        select: impl FnOnce() -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        if !cfg!(any(target_os = "windows", target_os = "macos")) {
            return select();
        }
        use std::io::Write;

        self.dialog_sequence += 1;
        let sequence = self.dialog_sequence;
        let process = std::process::id();
        let thread = std::thread::current().id();
        let started = std::time::Instant::now();
        // Diagnostic sequence only; never used as a document or request identity.
        // Release stderr before entering the modal loop. Logging failures must not
        // change picker behavior, and selected paths must not enter this trace.
        let _ = writeln!(
            std::io::stderr().lock(),
            "[radishflow-studio] native dialog started process={process} sequence={sequence} operation={operation} thread={thread:?}"
        );
        let selection = select();
        // rfd returns None for no selection; it does not expose a structured
        // failure reason, so do not claim that every None was a user cancellation.
        let outcome = if selection.is_some() {
            "selected"
        } else {
            "no_selection"
        };
        let _ = writeln!(
            std::io::stderr().lock(),
            "[radishflow-studio] native dialog returned process={process} sequence={sequence} operation={operation} outcome={outcome} elapsed_ms={}",
            started.elapsed().as_millis()
        );
        selection
    }
}

impl ProjectFilePicker for NativeProjectFilePicker {
    fn supports_equipment_export_dialogs(&self) -> bool {
        cfg!(any(target_os = "windows", target_os = "macos"))
    }
    fn pick_equipment_export_file(
        &mut self,
        format: super::equipment::CalculationTableFormat,
    ) -> Option<PathBuf> {
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        {
            let operation = match format {
                super::equipment::CalculationTableFormat::Csv => "equipment_export_csv",
                super::equipment::CalculationTableFormat::Json => "equipment_export_json",
            };
            self.trace_selection(operation, || {
                rfd::FileDialog::new()
                    .set_title("Export RadishFlow equipment calculation")
                    .add_filter("Calculation table", &[format.extension()])
                    .set_file_name(format!("equipment-calculation.{}", format.extension()))
                    .save_file()
            })
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let _ = format;
            None
        }
    }
    fn supports_project_dialogs(&self) -> bool {
        cfg!(any(target_os = "windows", target_os = "macos"))
    }
    fn pick_project_file(&mut self) -> Option<PathBuf> {
        self.trace_selection("project_open", pick_native_project_file)
    }

    fn pick_save_project_file(&mut self) -> Option<PathBuf> {
        self.trace_selection("project_save", pick_native_save_project_file)
    }

    fn supports_result_export_dialogs(&self) -> bool {
        cfg!(any(target_os = "windows", target_os = "macos"))
    }

    fn pick_result_export_file(&mut self) -> Option<PathBuf> {
        self.trace_selection("result_export", pick_native_result_export_file)
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

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn pick_native_result_export_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export RadishFlow solve snapshot")
        .add_filter("Text", &["txt"])
        .save_file()
        .map(ensure_text_file_extension)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
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

pub(super) fn ensure_text_file_extension(path: PathBuf) -> PathBuf {
    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"))
    {
        return path;
    }
    let mut value = path.into_os_string();
    value.push(".txt");
    PathBuf::from(value)
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
