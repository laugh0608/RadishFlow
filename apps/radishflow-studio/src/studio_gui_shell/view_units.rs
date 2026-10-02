use super::*;
use rf_types::units::{ALL_UNITS, QuantityKind};
use rf_ui::{DisplayUnitViewId, ProjectPresentationCommand, ViewDisplayUnits};

#[derive(Clone)]
pub(super) struct ViewUnitSettingsDraft {
    document: String,
    view: DisplayUnitViewId,
    units: ViewDisplayUnits,
    notice: Option<String>,
}

impl ReadyAppState {
    pub(super) fn open_view_unit_settings(&mut self, view: DisplayUnitViewId) {
        let document = self.platform_host.snapshot().runtime.workspace_document;
        self.unit_settings.draft = None;
        self.command_palette.close();
        self.unit_settings.view_draft = Some(ViewUnitSettingsDraft {
            document: document.document_id,
            view,
            units: document.presentation.view_units(view),
            notice: None,
        });
    }

    pub(super) fn close_inspector_view(&mut self, view: DisplayUnitViewId) -> RfResult<()> {
        self.apply_presentation_command(ProjectPresentationCommand::CloseView(view))?;
        self.unit_settings.closed_inspectors.insert(view);
        if self
            .unit_settings
            .view_draft
            .as_ref()
            .is_some_and(|draft| draft.view == view)
        {
            self.unit_settings.view_draft = None;
        }
        Ok(())
    }

    /// Inspector is one logical view per window; switching tabs/objects does not retire it.
    pub(super) fn render_inspector_view_controls(&mut self, ui: &mut egui::Ui) -> bool {
        let Some(view) = self.current_window_id().map(DisplayUnitViewId) else {
            return false;
        };
        let zh = matches!(self.locale, StudioShellLocale::ZhCn);
        if self.unit_settings.closed_inspectors.contains(&view) {
            if ui
                .button(if zh {
                    "重新打开检查器"
                } else {
                    "Reopen Inspector"
                })
                .clicked()
            {
                self.unit_settings.closed_inspectors.remove(&view);
            }
            return false;
        }
        let mut closed = false;
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(if zh {
                    "本视图显示单位…"
                } else {
                    "View display units…"
                })
                .clicked()
            {
                self.open_view_unit_settings(view);
            }
            if ui
                .small_button(if zh {
                    "关闭检查器"
                } else {
                    "Close Inspector"
                })
                .clicked()
            {
                match self.close_inspector_view(view) {
                    Ok(()) => closed = true,
                    Err(error) => self.platform_host.record_activity_line(error.to_string()),
                }
            }
        });
        let presentation = self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .presentation;
        ui.small(if presentation.view_units(view).is_empty() {
            if zh {
                "显示跟随项目；临时输入单位另行保留。"
            } else {
                "Display follows project; input units stay with each edit."
            }
        } else if zh {
            "仅本检查器覆盖显示；不随工程保存。"
        } else {
            "Display overridden only in this Inspector; not saved with the project."
        });
        !closed
    }

    pub(super) fn render_view_unit_settings(&mut self, ctx: &egui::Context) -> bool {
        let Some(mut draft) = self.unit_settings.view_draft.clone() else {
            return false;
        };
        let snapshot = self.platform_host.snapshot();
        if draft.document != snapshot.runtime.workspace_document.document_id
            || self.current_window_id() != Some(draft.view.0)
            || snapshot.app_host_state.window(draft.view.0).is_none()
            || self.unit_settings.closed_inspectors.contains(&draft.view)
        {
            self.unit_settings.view_draft = None;
            return false;
        }
        let zh = matches!(self.locale, StudioShellLocale::ZhCn);
        let mut close = false;
        let response = egui::Modal::new(egui::Id::new("inspector-display-units")).show(ctx, |ui| {
            ui.heading(if zh { "仅本视图：检查器显示单位" } else { "This view: Inspector display units" });
            ui.label(if zh { "跨对象和标签页保持；关闭检查器或窗口后清除。项目设置、模块设置和结果页保持各自单位。当前输入不改写。" } else { "Kept across objects and tabs; cleared when this Inspector or window closes. Project settings, module settings and results keep their units. Active inputs are preserved." });
            for (quantity, label) in [
                (QuantityKind::AbsoluteTemperature, if zh { "温度" } else { "Temperature" }),
                (QuantityKind::AbsolutePressure, if zh { "压力" } else { "Pressure" }),
                (QuantityKind::MolarFlow, if zh { "摩尔流量" } else { "Molar flow" }),
            ] {
                ui.horizontal(|ui| {
                    ui.label(label);
                    let selected = draft.units.unit_for(quantity);
                    egui::ComboBox::from_id_salt(quantity.definition().id)
                        .selected_text(selected.map_or(if zh { "跟随项目" } else { "Follow project" }, |unit| unit.definition().symbol))
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(selected.is_none(), if zh { "跟随项目" } else { "Follow project" }).clicked() { draft.units.set_unit(quantity, None).expect("clear is valid"); }
                            for unit in ALL_UNITS {
                                let mut candidate = draft.units.clone();
                                if candidate.set_unit(quantity, Some(*unit)).is_ok() && ui.selectable_label(selected == Some(*unit), unit.definition().symbol).clicked() { draft.units = candidate; }
                            }
                        });
                    ui.small(format!("{} {}", if zh { "项目" } else { "Project" }, snapshot.runtime.workspace_document.presentation.display_units().unit_for(quantity).definition().symbol));
                });
            }
            if ui.button(if zh { "全部跟随项目" } else { "Follow project for all" }).clicked() { draft.units = ViewDisplayUnits::default(); }
            ui.horizontal(|ui| {
                if ui.button(if zh { "应用本视图" } else { "Apply to this view" }).clicked() {
                    match self.apply_presentation_command(ProjectPresentationCommand::ApplyView { view: draft.view, units: draft.units.clone() }) {
                        Ok(()) => close = true,
                        Err(error) => draft.notice = Some(error.to_string()),
                    }
                }
                if ui.button(if zh { "取消" } else { "Cancel" }).clicked() { close = true; }
            });
            ui.separator();
            ui.label(if zh { "呈现历史包含项目与检查器显示变更；工程输入历史独立。" } else { "Presentation history includes project and Inspector display changes; input history is separate." });
            for (label, command, enabled) in [
                (if zh { "撤销呈现变更" } else { "Undo presentation" }, ProjectPresentationCommand::Undo, snapshot.runtime.workspace_document.presentation.can_undo()),
                (if zh { "重做呈现变更" } else { "Redo presentation" }, ProjectPresentationCommand::Redo, snapshot.runtime.workspace_document.presentation.can_redo()),
            ] {
                if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                    match self.apply_presentation_command(command) {
                        Ok(()) => draft.units = self.platform_host.snapshot().runtime.workspace_document.presentation.view_units(draft.view),
                        Err(error) => draft.notice = Some(error.to_string()),
                    }
                }
            }
            if draft.units != snapshot.runtime.workspace_document.presentation.view_units(draft.view) {
                super::state_presentation::light_state_surface(ui, |ui| {
                    ui.colored_label(super::state_presentation::StudioStateTokens::SECONDARY, if zh { "本视图选择尚未应用；不写入工程文件。" } else { "View choices not yet applied; not saved in the project file." });
                });
            }
            render_project_save_state(ui, &snapshot.runtime.workspace_document.save_state, self.locale);
            if let Some(detail) = &draft.notice {
                render_project_notice(ui, &ProjectOpenNotice { level: ProjectOpenNoticeLevel::Error, title: if zh { "视图设置未修改" } else { "View settings unchanged" }.into(), detail: detail.clone() });
            }
        });
        self.unit_settings.view_draft = if close || response.should_close() {
            None
        } else {
            Some(draft)
        };
        true
    }
}
