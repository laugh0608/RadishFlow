use super::*;

/// Accepted light roles shared by numeric controls, settings and file-operation notices.
pub(super) struct StudioStateTokens;
impl StudioStateTokens {
    pub const SURFACE: egui::Color32 = egui::Color32::WHITE;
    pub const EDIT: egui::Color32 = egui::Color32::from_rgb(239, 246, 255);
    pub const TEXT: egui::Color32 = egui::Color32::from_rgb(31, 41, 55);
    pub const SPECIFIED: egui::Color32 = egui::Color32::from_rgb(29, 78, 216);
    pub const SECONDARY: egui::Color32 = egui::Color32::from_rgb(71, 85, 105);
    pub const ERROR: egui::Color32 = egui::Color32::from_rgb(185, 28, 28);
    pub const WARNING: egui::Color32 = egui::Color32::from_rgb(146, 64, 14);
    pub const BORDER: egui::Color32 = egui::Color32::from_rgb(100, 116, 139);
    pub const FOCUS: egui::Color32 = egui::Color32::from_rgb(37, 99, 235);
}

pub(super) fn light_state_surface<R>(
    ui: &mut egui::Ui,
    contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.scope(|ui| {
        ui.visuals_mut().clone_from(&egui::Visuals::light());
        egui::Frame::new()
            .fill(StudioStateTokens::SURFACE)
            .inner_margin(4.0)
            .show(ui, contents)
            .inner
    })
    .inner
}

pub(super) fn render_project_save_state(
    ui: &mut egui::Ui,
    state: &rf_ui::ProjectSaveState,
    locale: StudioShellLocale,
) {
    let zh = locale == StudioShellLocale::ZhCn;
    light_state_surface(ui, |ui| {
        // The three facts remain independent. Drafts and unsaved choices are not diagnostics.
        ui.colored_label(
            StudioStateTokens::SECONDARY,
            match (zh, state.document_dirty) {
                (true, true) => "工程内容：待保存",
                (true, false) => "工程内容：无待保存修改",
                (false, true) => "Project content: unsaved",
                (false, false) => "Project content: no unsaved changes",
            },
        );
        ui.colored_label(
            StudioStateTokens::SECONDARY,
            match (zh, state.presentation_dirty) {
                (true, true) => "显示设置：待保存",
                (true, false) => "显示设置：无待保存修改",
                (false, true) => "Display settings: unsaved",
                (false, false) => "Display settings: no unsaved changes",
            },
        );
        ui.colored_label(
            StudioStateTokens::SECONDARY,
            if zh {
                format!(
                    "未提交输入：{} 项（不写入工程文件）",
                    state.pending_input_count
                )
            } else {
                format!(
                    "Unsubmitted inputs: {} (not included in the saved file)",
                    state.pending_input_count
                )
            },
        );
    });
}

pub(super) fn render_project_notice(ui: &mut egui::Ui, notice: &ProjectOpenNotice) {
    light_state_surface(ui, |ui| {
        let (prefix, color) = match notice.level {
            ProjectOpenNoticeLevel::Info => ("ℹ", StudioStateTokens::SECONDARY),
            ProjectOpenNoticeLevel::Warning => ("⚠", StudioStateTokens::WARNING),
            ProjectOpenNoticeLevel::Error => ("!", StudioStateTokens::ERROR),
        };
        ui.colored_label(color, format!("{prefix} {}", notice.title));
        ui.add(
            egui::Label::new(
                egui::RichText::new(&notice.detail)
                    .small()
                    .color(StudioStateTokens::TEXT),
            )
            .wrap(),
        );
    });
}
