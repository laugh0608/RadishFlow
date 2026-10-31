use super::*;

struct CapturedResultOutput {
    document_id: String,
    snapshot: radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    text: String,
}

pub(super) struct PendingResultExport {
    output: CapturedResultOutput,
    path: PathBuf,
}

impl ReadyAppState {
    pub(super) fn copy_solve_snapshot_to_clipboard(
        &mut self,
        ctx: &egui::Context,
        snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    ) {
        let Some(output) = self.capture_result_output(snapshot) else {
            return;
        };
        ctx.copy_text(output.text);
        let snapshot = &output.snapshot;
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Info,
            title: solve_snapshot_copied_notice_title(self.locale).to_string(),
            detail: solve_snapshot_copied_notice_detail(self.locale, &snapshot.snapshot_id),
        });
        self.platform_host.record_activity_line(format!(
            "copied solve snapshot {} results to clipboard",
            snapshot.snapshot_id
        ));
    }

    pub(super) fn export_solve_snapshot_from_picker(
        &mut self,
        snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    ) {
        let Some(output) = self.capture_result_output(snapshot) else {
            return;
        };
        if !self.project_file_picker.supports_result_export_dialogs() {
            self.result_output_notice(
                ProjectOpenNoticeLevel::Warning,
                "Result export unavailable",
                "当前平台暂不支持文件导出",
                "Use Copy current results to copy the text instead.",
                "可使用“复制当前结果”获取文本。",
            );
            return;
        }
        let Some(path) = self.project_file_picker.pick_result_export_file() else {
            self.cancel_result_export();
            return;
        };
        self.export_captured_result_to_path(output, path);
    }

    #[cfg(test)]
    pub(super) fn export_solve_snapshot_to_path(
        &mut self,
        snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
        path: PathBuf,
    ) {
        let Some(output) = self.capture_result_output(snapshot) else {
            return;
        };
        self.export_captured_result_to_path(output, path);
    }

    fn export_captured_result_to_path(&mut self, output: CapturedResultOutput, path: PathBuf) {
        let path = project_picker::ensure_text_file_extension(path);
        if path.is_file() {
            self.pending_result_export = Some(PendingResultExport { output, path });
            return;
        }
        self.write_result_export(output, path, rf_store::FileOverwritePolicy::Forbid);
    }

    fn write_result_export(
        &mut self,
        output: CapturedResultOutput,
        path: PathBuf,
        overwrite: rf_store::FileOverwritePolicy,
    ) {
        if self.platform_host.document().metadata.document_id.as_str() != output.document_id
            || self.current_result_for_output(&output.snapshot).is_none()
        {
            self.result_output_notice(
                ProjectOpenNoticeLevel::Warning,
                "Current results unavailable",
                "当前结果不可输出",
                "The project or results changed. Start a new export.",
                "工程或结果已变化，请重新发起导出。",
            );
            return;
        }
        let snapshot = &output.snapshot;
        match rf_store::write_text_file(&path, &output.text, overwrite) {
            Ok(()) => {
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Info,
                    title: solve_snapshot_exported_notice_title(self.locale).to_string(),
                    detail: solve_snapshot_exported_notice_detail(
                        self.locale,
                        &snapshot.snapshot_id,
                        &path,
                    ),
                });
                self.platform_host.record_activity_line(format!(
                    "exported solve snapshot {} results to {}",
                    snapshot.snapshot_id,
                    path.display()
                ));
            }
            Err(error) => {
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: solve_snapshot_export_failed_notice_title(self.locale).to_string(),
                    detail: solve_snapshot_export_failed_notice_detail(self.locale, &path, &error),
                });
                self.platform_host.record_activity_line(format!(
                    "export solve snapshot {} failed: {} ({})",
                    snapshot.snapshot_id,
                    error,
                    path.display()
                ));
            }
        }
    }

    fn capture_result_output(
        &mut self,
        requested: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    ) -> Option<CapturedResultOutput> {
        // Re-read SI and project units at invocation, before opening the picker.
        let snapshot = self.current_result_for_output(requested)?;
        let text = match snapshot.light_text_export() {
            Ok(text) => text,
            Err(error) => {
                let detail = error.to_string();
                self.result_output_notice(
                    ProjectOpenNoticeLevel::Error,
                    "Result conversion failed",
                    "结果单位转换失败",
                    &detail,
                    &detail,
                );
                return None;
            }
        };
        Some(CapturedResultOutput {
            document_id: self
                .platform_host
                .document()
                .metadata
                .document_id
                .as_str()
                .into(),
            snapshot,
            text,
        })
    }

    fn current_result_for_output(
        &mut self,
        requested: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    ) -> Option<radishflow_studio::StudioGuiWindowSolveSnapshotModel> {
        if self.modular_plan_draft_pending() {
            self.result_output_notice(
                ProjectOpenNoticeLevel::Warning,
                "Current results unavailable",
                "当前结果不可输出",
                "Apply or discard the modular plan draft before output.",
                "请先应用或放弃联立计划草稿再输出。",
            );
            return None;
        }
        if !requested
            .result_guard
            .as_ref()
            .is_some_and(|guard| guard.is_current(&self.platform_host.modeling_inputs().workspace))
        {
            self.result_output_notice(
                ProjectOpenNoticeLevel::Warning,
                "Current results unavailable",
                "当前结果不可输出",
                "The result belongs to a changed execution or session. Start a new export.",
                "结果所属运行或会话已变化，请重新发起导出。",
            );
            return None;
        }
        let window = self.platform_host.snapshot().window_model();
        if let Some(current) = window.runtime.latest_solve_snapshot
            && current.snapshot_id == requested.snapshot_id
            && current.sequence == requested.sequence
            && current.document_revision == requested.document_revision
            && current.document_revision == window.runtime.workspace_document.revision
        {
            return Some(current);
        }
        self.result_output_notice(
            ProjectOpenNoticeLevel::Warning,
            "Current results unavailable",
            "当前结果不可输出",
            "Results changed or are missing. Run again and retry from the results page.",
            "结果已变化或尚未生成，请重新运行后从结果页操作。",
        );
        None
    }

    pub(super) fn render_result_output_actions(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        let snapshot = window.runtime.latest_solve_snapshot.as_ref();
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(
                snapshot.is_some() && !self.modular_plan_draft_pending(),
                |ui| {
                    if ui
                        .small_button(self.locale.text(ShellText::CopySnapshot))
                        .clicked()
                        && let Some(snapshot) = snapshot
                    {
                        self.copy_solve_snapshot_to_clipboard(ui.ctx(), snapshot);
                    }
                    if ui
                        .small_button(self.locale.text(ShellText::ExportSnapshot))
                        .clicked()
                        && let Some(snapshot) = snapshot
                    {
                        self.export_solve_snapshot_from_picker(snapshot);
                    }
                },
            );
            if snapshot.is_none() {
                ui.small(result_output_text(
                    self.locale,
                    "Run to generate current results before exporting.",
                    "请先运行以生成当前结果，再复制或导出。",
                ));
            }
        });
        ui.small(result_output_text(
            self.locale,
            "Copy / export uses project units captured when requested.",
            "复制 / 导出使用发起时的项目单位。",
        ));
        if let Some(notice) = &self.project_open.notice {
            ui.label(format!("{}：{}", notice.title, notice.detail));
        }
    }

    pub(super) fn confirm_result_export(&mut self) {
        let Some(pending) = self.pending_result_export.take() else {
            return;
        };
        self.write_result_export(
            pending.output,
            pending.path,
            rf_store::FileOverwritePolicy::Allow,
        );
    }

    pub(super) fn cancel_result_export(&mut self) {
        self.pending_result_export = None;
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Info,
            title: solve_snapshot_export_canceled_notice_title(self.locale).to_string(),
            detail: result_output_text(
                self.locale,
                "No result file was written.",
                "未写入结果文件。",
            )
            .to_string(),
        });
    }

    pub(super) fn render_export_dialogs(&mut self, ctx: &egui::Context) {
        self.render_equipment_export_dialog(ctx);
        let Some(pending) = self.pending_result_export.as_ref() else {
            return;
        };
        let path = pending.path.clone();
        let response = egui::Modal::new(egui::Id::new("result-export-overwrite")).show(ctx, |ui| {
            ui.set_max_width(480.0);
            ui.heading(result_output_text(
                self.locale,
                "Replace result file?",
                "覆盖结果文件？",
            ));
            ui.label(path.display().to_string());
            ui.label(result_output_text(
                self.locale,
                "This file already exists. Replace it with the current results?",
                "此文件已存在，是否用当前结果替换？",
            ));
            ui.horizontal(|ui| {
                if ui
                    .button(result_output_text(self.locale, "Cancel export", "取消导出"))
                    .clicked()
                {
                    self.cancel_result_export();
                }
                if ui
                    .button(result_output_text(self.locale, "Replace file", "确认覆盖"))
                    .clicked()
                {
                    self.confirm_result_export();
                }
            });
        });
        if response.should_close() && self.pending_result_export.is_some() {
            self.cancel_result_export();
        }
    }

    fn result_output_notice(
        &mut self,
        level: ProjectOpenNoticeLevel,
        title_en: &str,
        title_zh: &str,
        detail_en: &str,
        detail_zh: &str,
    ) {
        self.project_open.notice = Some(ProjectOpenNotice {
            level,
            title: result_output_text(self.locale, title_en, title_zh).to_string(),
            detail: result_output_text(self.locale, detail_en, detail_zh).to_string(),
        });
    }
}

fn solve_snapshot_copied_notice_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Snapshot copied",
        StudioShellLocale::ZhCn => "快照已复制",
    }
}

fn solve_snapshot_copied_notice_detail(locale: StudioShellLocale, snapshot_id: &str) -> String {
    match locale {
        StudioShellLocale::En => {
            format!("Copied the current solve snapshot to the clipboard: {snapshot_id}.")
        }
        StudioShellLocale::ZhCn => {
            format!("已将当前求解快照复制到剪贴板：{snapshot_id}。")
        }
    }
}

fn solve_snapshot_export_canceled_notice_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Snapshot export canceled",
        StudioShellLocale::ZhCn => "已取消快照导出",
    }
}

fn solve_snapshot_exported_notice_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Snapshot exported",
        StudioShellLocale::ZhCn => "快照已导出",
    }
}

fn solve_snapshot_exported_notice_detail(
    locale: StudioShellLocale,
    snapshot_id: &str,
    path: &std::path::Path,
) -> String {
    match locale {
        StudioShellLocale::En => {
            format!(
                "Exported current solve snapshot {snapshot_id} to {}.",
                path.display()
            )
        }
        StudioShellLocale::ZhCn => {
            format!("已将当前求解快照 {snapshot_id} 导出到 {}。", path.display())
        }
    }
}

fn solve_snapshot_export_failed_notice_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Snapshot export failed",
        StudioShellLocale::ZhCn => "快照导出失败",
    }
}

fn solve_snapshot_export_failed_notice_detail(
    locale: StudioShellLocale,
    path: &std::path::Path,
    error: &rf_types::RfError,
) -> String {
    match locale {
        StudioShellLocale::En => format!(
            "Could not write the solve snapshot to {}: {error}.",
            path.display()
        ),
        StudioShellLocale::ZhCn => {
            format!("无法将求解快照写入 {}：{error}。", path.display())
        }
    }
}

fn result_output_text<'a>(locale: StudioShellLocale, en: &'a str, zh: &'a str) -> &'a str {
    match locale {
        StudioShellLocale::En => en,
        StudioShellLocale::ZhCn => zh,
    }
}
