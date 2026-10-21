//! File-dialog orchestration for an immutable, qualified equipment table.
use super::*;
use rf_types::EquipmentTaskId;
use rf_ui::engineering::{CalculationId, EquipmentCaseKey};

enum EquipmentOutputKey {
    Sizing(EquipmentTaskId),
    Rating {
        member: EquipmentCaseKey,
        units: DisplayUnitSet,
    },
}

pub(in crate::studio_gui_shell) struct CapturedEquipmentOutput {
    key: EquipmentOutputKey,
    calculation_id: CalculationId,
    format: CalculationTableFormat,
    text: String,
}
pub(in crate::studio_gui_shell) struct PendingEquipmentExport {
    output: CapturedEquipmentOutput,
    path: PathBuf,
}

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn capture_equipment_output(
        &mut self,
        id: &EquipmentTaskId,
        format: CalculationTableFormat,
    ) -> Option<CapturedEquipmentOutput> {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let session = &workspace.engineering;
        let Some(task) = session.inputs().equipment_task(id) else {
            self.equipment_export_unavailable();
            return None;
        };
        let units = workspace.project_presentation.display_units().clone();
        let document_id = workspace.document.metadata.document_id.as_str();
        let (key, calculation_id, encoded) = match task {
            rf_model::equipment::EquipmentTask::SeparatorSizing(task) => {
                let Some(candidate) = session
                    .current_candidate(id)
                    .filter(|_| !workspace.equipment_editor.pending_for_task(id, session))
                else {
                    self.equipment_export_unavailable();
                    return None;
                };
                (
                    EquipmentOutputKey::Sizing(id.clone()),
                    session
                        .calculation_id(id)
                        .expect("current calculation identity")
                        .clone(),
                    calculation_table::encode(
                        candidate,
                        task,
                        document_id,
                        session.generation(id).expect("current generation"),
                        units,
                        format,
                    ),
                )
            }
            rf_model::equipment::EquipmentTask::SeparatorRating(task) => {
                let [case] = task.cases.as_slice() else {
                    self.equipment_export_unavailable();
                    return None;
                };
                let member = EquipmentCaseKey {
                    task_id: id.clone(),
                    case_id: case.id.clone(),
                };
                let Some(result) = session.current_rating(&member).filter(|_| {
                    !workspace
                        .equipment_editor
                        .pending_for_case(&member, session)
                }) else {
                    self.equipment_export_unavailable();
                    return None;
                };
                let calculation_id = session
                    .case_calculation_id(&member)
                    .expect("current rating identity")
                    .clone();
                let encoded = calculation_table::encode_rating(
                    result,
                    task,
                    document_id,
                    session
                        .case_generation(&member)
                        .expect("current rating generation"),
                    units.clone(),
                    format,
                );
                (
                    EquipmentOutputKey::Rating { member, units },
                    calculation_id,
                    encoded,
                )
            }
        };
        match encoded {
            Ok(text) => Some(CapturedEquipmentOutput {
                key,
                calculation_id,
                format,
                text,
            }),
            Err(error) => {
                self.equipment_export_notice(
                    ProjectOpenNoticeLevel::Error,
                    tr(self.locale, "计算表生成失败", "Calculation table failed"),
                    error.to_string(),
                );
                None
            }
        }
    }
    pub(in crate::studio_gui_shell) fn export_equipment_from_picker(
        &mut self,
        id: &EquipmentTaskId,
        format: CalculationTableFormat,
    ) {
        let Some(output) = self.capture_equipment_output(id, format) else {
            return;
        };
        if !self.project_file_picker.supports_equipment_export_dialogs() {
            self.equipment_export_notice(
                ProjectOpenNoticeLevel::Warning,
                tr(self.locale, "文件导出不可用", "File export unavailable"),
                tr(
                    self.locale,
                    "当前平台尚未接入设备计算表保存选择器。",
                    "The equipment table save dialog is not available on this platform.",
                ),
            );
            return;
        }
        let Some(path) = self.project_file_picker.pick_equipment_export_file(format) else {
            self.cancel_equipment_export();
            return;
        };
        self.export_equipment_to_path(output, path);
    }
    pub(in crate::studio_gui_shell) fn export_equipment_to_path(
        &mut self,
        output: CapturedEquipmentOutput,
        path: PathBuf,
    ) {
        let path = equipment_export_path(path, output.format);
        if path.is_file() {
            self.equipment.pending_export = Some(PendingEquipmentExport { output, path });
        } else {
            self.write_equipment_export(output, path, rf_store::FileOverwritePolicy::Forbid);
        }
    }
    fn write_equipment_export(
        &mut self,
        output: CapturedEquipmentOutput,
        path: PathBuf,
        policy: rf_store::FileOverwritePolicy,
    ) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let session = &workspace.engineering;
        // CalculationId is session-local and non-reusable, including after an
        // identical recalculation or reopening the same document/task IDs.
        let qualified = match &output.key {
            EquipmentOutputKey::Sizing(id) => {
                session.current_candidate(id).is_some()
                    && session.calculation_id(id) == Some(&output.calculation_id)
                    && !workspace.equipment_editor.pending_for_task(id, session)
            }
            EquipmentOutputKey::Rating { member, units } => {
                session.current_rating(member).is_some()
                    && session.case_calculation_id(member) == Some(&output.calculation_id)
                    && !workspace.equipment_editor.pending_for_case(member, session)
                    && workspace.project_presentation.display_units() == units
            }
        };
        if !qualified {
            self.equipment_export_unavailable();
            return;
        }
        match rf_store::write_text_file(&path, &output.text, policy) {
            Ok(()) => self.equipment_export_notice(
                ProjectOpenNoticeLevel::Info,
                tr(self.locale, "计算表已导出", "Calculation table exported"),
                path.display().to_string(),
            ),
            Err(error) => self.equipment_export_notice(
                ProjectOpenNoticeLevel::Error,
                tr(
                    self.locale,
                    "计算表写入失败",
                    "Calculation table write failed",
                ),
                format!("{}: {error}", path.display()),
            ),
        }
    }
    pub(in crate::studio_gui_shell) fn confirm_equipment_export(&mut self) {
        if let Some(pending) = self.equipment.pending_export.take() {
            self.write_equipment_export(
                pending.output,
                pending.path,
                rf_store::FileOverwritePolicy::Allow,
            );
        }
    }
    pub(in crate::studio_gui_shell) fn cancel_equipment_export(&mut self) {
        self.equipment.pending_export = None;
        self.equipment_export_notice(
            ProjectOpenNoticeLevel::Info,
            tr(
                self.locale,
                "已取消计算表导出",
                "Calculation table export canceled",
            ),
            tr(
                self.locale,
                "文件及工程保持不变。",
                "Files and project are unchanged.",
            ),
        );
    }
    fn equipment_export_unavailable(&mut self) {
        self.equipment_export_notice(ProjectOpenNoticeLevel::Warning,
            tr(self.locale, "当前计算表不可导出", "Current calculation table unavailable"),
            tr(self.locale, "请处理本任务及所引用依据的草稿，确认当前计算与项目单位后重新导出。", "Resolve drafts for this task and its referenced evidence, then export a current calculation with the current project units."));
    }
    fn equipment_export_notice(
        &mut self,
        level: ProjectOpenNoticeLevel,
        title: &str,
        detail: impl Into<String>,
    ) {
        self.project_open.notice = Some(ProjectOpenNotice {
            level,
            title: title.into(),
            detail: detail.into(),
        });
    }
    pub(in crate::studio_gui_shell) fn render_equipment_export_dialog(
        &mut self,
        ctx: &egui::Context,
    ) {
        let Some(pending) = self.equipment.pending_export.as_ref() else {
            return;
        };
        let path = pending.path.display().to_string();
        let locale = self.locale;
        let response =
            egui::Modal::new(egui::Id::new("equipment-export-overwrite")).show(ctx, |ui| {
                ui.set_width((ctx.screen_rect().width() - 40.0).clamp(220.0, 520.0));
                ui.heading(tr(
                    locale,
                    "覆盖设备计算表？",
                    "Replace equipment calculation table?",
                ));
                ui.label(path);
                ui.label(tr(
                    locale,
                    "确认前将再次检查计算是否有效。",
                    "Calculation eligibility is checked again before writing.",
                ));
                ui.horizontal_wrapped(|ui| {
                    let cancel = ui.button(tr(locale, "取消导出", "Cancel export"));
                    let confirm = ui.button(tr(locale, "确认覆盖", "Replace file"));
                    if !cancel.has_focus() && !confirm.has_focus() && !confirm.lost_focus() {
                        cancel.request_focus();
                    }
                    if cancel.clicked() {
                        self.cancel_equipment_export();
                    }
                    if confirm.clicked() {
                        self.confirm_equipment_export();
                    }
                });
            });
        if response.should_close() && self.equipment.pending_export.is_some() {
            self.cancel_equipment_export();
        }
    }
}
fn equipment_export_path(path: PathBuf, format: CalculationTableFormat) -> PathBuf {
    let extension = format.extension();
    if path
        .to_string_lossy()
        .to_ascii_lowercase()
        .ends_with(rf_store::STORED_PROJECT_FILE_EXTENSION)
    {
        let mut name = path.into_os_string();
        name.push(format!(".calculation.{extension}"));
        return name.into();
    }

    if path
        .extension()
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
    {
        return path;
    }
    // Preserve the chosen basename and any unrelated suffix when adding the
    // format extension. Project-file suffixes receive separate protection above.
    let mut name = path.into_os_string();
    name.push(format!(".{extension}"));
    name.into()
}
