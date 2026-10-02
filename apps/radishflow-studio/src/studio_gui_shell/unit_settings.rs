use super::*;
use rf_types::units::{ALL_UNITS, DisplayUnitSet, QuantityKind};
use rf_ui::ProjectPresentationCommand;

pub(super) struct UnitSettingsState {
    pub draft: Option<(String, DisplayUnitSet)>,
    pub view_draft: Option<super::view_units::ViewUnitSettingsDraft>,
    pub closed_inspectors: std::collections::BTreeSet<rf_ui::DisplayUnitViewId>,
    pub default_units: Result<DisplayUnitSet, String>,
    pub pending_new: Option<Option<AuthoringCaseKind>>,
    default_path: PathBuf,
    notice: Option<ProjectOpenNotice>,
    recovery_required: bool,
    pending_recovery: Option<DisplayUnitSet>,
}

impl UnitSettingsState {
    pub fn load(preferences: &std::path::Path) -> Self {
        let path = preferences.with_file_name(rf_store::UNIT_DEFAULTS_FILE_NAME);
        let units = rf_store::read_unit_defaults(&path)
            .map(|units| units.unwrap_or_default())
            .map_err(|error| error.to_string());
        Self {
            draft: None,
            view_draft: None,
            closed_inspectors: Default::default(),
            recovery_required: units.is_err(),
            default_units: units,
            pending_new: None,
            default_path: path,
            notice: None,
            pending_recovery: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.draft.is_some()
            || self.view_draft.is_some()
            || self.pending_new.is_some()
            || self.pending_recovery.is_some()
    }
}

impl ReadyAppState {
    pub(super) fn initialize_new_project_units(&mut self, units: DisplayUnitSet) {
        if let Some(window_id) = self.current_window_id() {
            self.dispatch_event(StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: StudioRuntimeTrigger::NewProjectDisplayUnits(units),
            });
        }
    }

    pub(super) fn open_unit_settings(&mut self) {
        self.command_palette.close();
        self.unit_settings.view_draft = None;
        let document = self.platform_host.snapshot().runtime.workspace_document;
        self.unit_settings.draft = Some((
            document.document_id,
            document.presentation.display_units().clone(),
        ));
    }

    pub(super) fn apply_presentation_command(
        &mut self,
        command: ProjectPresentationCommand,
    ) -> RfResult<()> {
        let window_id = self
            .current_window_id()
            .ok_or_else(|| rf_types::RfError::invalid_input("no active window"))?;
        self.dispatch_event_result(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::ProjectPresentation(command),
        })?;
        Ok(())
    }

    pub(super) fn save_unit_default(&mut self, recover: bool) {
        let units = if recover {
            let Some(units) = self.unit_settings.pending_recovery.take() else {
                return;
            };
            units
        } else {
            self.platform_host
                .snapshot()
                .runtime
                .workspace_document
                .presentation
                .display_units()
                .clone()
        };
        let result = if recover {
            rf_store::recover_unit_defaults(&self.unit_settings.default_path, &units)
                .map(|backup| format!("新工程默认已恢复；原文件备份：{}", backup.display()))
        } else {
            rf_store::write_unit_defaults(&self.unit_settings.default_path, &units)
                .map(|()| "新工程默认已保存；现有工程保持原设置。".into())
        };
        match result {
            Ok(message) => {
                self.unit_settings.default_units = Ok(units);
                self.unit_settings.recovery_required = false;
                self.unit_settings.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Info,
                    title: "个人默认已保存".into(),
                    detail: message,
                });
            }
            Err(error) => {
                if recover {
                    self.unit_settings.pending_recovery = Some(units);
                }
                self.unit_settings.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: "个人默认保存失败".into(),
                    detail: format!("{error}。内存默认与工程设置保持不变，可重试。"),
                });
                self.unit_settings.recovery_required =
                    rf_store::read_unit_defaults(&self.unit_settings.default_path).is_err();
            }
        }
    }

    pub(super) fn request_unit_default_recovery(&mut self) {
        self.unit_settings.pending_recovery = Some(
            self.platform_host
                .snapshot()
                .runtime
                .workspace_document
                .presentation
                .display_units()
                .clone(),
        );
    }

    pub(super) fn render_unit_settings(&mut self, ctx: &egui::Context) -> bool {
        if self.render_view_unit_settings(ctx) {
            return true;
        }
        if let Some(authoring) = self.unit_settings.pending_new {
            let response = egui::Modal::new(egui::Id::new("invalid-unit-default-new-project")).show(ctx, |ui| {
                ui.heading("个人单位默认不可用");
                if let Err(error) = &self.unit_settings.default_units { ui.label(error); }
                ui.label("原默认文件保持不变。可明确选择完整 SI 创建此工程，或取消并在显示单位设置中恢复默认。");
                if ui.button("本次使用 SI 新建").clicked() {
                    self.unit_settings.pending_new = None;
                    self.create_blank_project_with_units(authoring, DisplayUnitSet::si());
                }
                if ui.button("取消").clicked() { self.unit_settings.pending_new = None; }
            });
            if response.should_close() {
                self.unit_settings.pending_new = None;
            }
            return true;
        }
        if self.unit_settings.pending_recovery.is_some() {
            let response = egui::Modal::new(egui::Id::new("recover-unit-defaults")).show(ctx, |ui| {
                ui.heading("恢复个人单位默认");
                ui.label("将先备份原默认文件，再用当前已应用的项目单位集替换。备份或替换失败时保留可重试状态。");
                ui.label(self.unit_settings.default_path.display().to_string());
                if let Some(notice) = &self.unit_settings.notice { render_project_notice(ui, notice); }
                if ui.button("备份并替换默认").clicked() { self.save_unit_default(true); }
                if ui.button("取消").clicked() { self.unit_settings.pending_recovery = None; }
            });
            if response.should_close() {
                self.unit_settings.pending_recovery = None;
            }
            return true;
        }
        let Some((document_id, mut units)) = self.unit_settings.draft.clone() else {
            return false;
        };
        let document = self.platform_host.snapshot().runtime.workspace_document;
        if document_id != document.document_id {
            self.unit_settings.draft = None;
            return false;
        }
        let mut close = false;
        let response = egui::Modal::new(egui::Id::new("project-display-units")).show(ctx, |ui| {
            ui.set_min_width(430.0);
            ui.heading("项目显示单位");
            ui.label("作用范围：当前工程，随工程文件保存。检查器可单独覆盖；当前输入保持本次输入单位，结果仍以自身标签为准。");
            ui.horizontal(|ui| {
                if ui.button("完整 SI").clicked() {
                    units = DisplayUnitSet::si();
                }
                if ui.button("工程集").clicked() {
                    units = DisplayUnitSet::engineering();
                }
            });
            let choices: Vec<_> = units.entries().collect();
            egui::Grid::new("project-display-unit-choices").show(ui, |ui| {
                for (quantity, selected) in choices {
                    let label = match quantity {
                        QuantityKind::AbsoluteTemperature => "绝对温度",
                        QuantityKind::TemperatureDifference => "温差",
                        QuantityKind::AbsolutePressure => "绝对压力",
                        QuantityKind::MolarFlow => "摩尔流量",
                        QuantityKind::MoleFraction => "摩尔分数",
                        QuantityKind::MolarPhaseFraction => "相摩尔分率",
                        QuantityKind::MolarEnthalpy => "摩尔焓",
                    };
                    ui.label(label);
                    let choices: Vec<_> = ALL_UNITS
                        .iter()
                        .filter(|unit| units.clone().set_unit(quantity, **unit).is_ok())
                        .map(|unit| (*unit, unit.definition().symbol))
                        .collect();
                    if let Some(unit) = super::unit_selector::unit_selector(
                        ui,
                        quantity.definition().id,
                        &format!("项目显示单位：{label}"),
                        selected,
                        &choices,
                    ).inner {
                        units.set_unit(quantity, unit).expect("menu contains valid units");
                    }
                    ui.end_row();
                }
            });
            render_project_save_state(ui, &document.save_state, self.locale);
            if units != *document.presentation.display_units() {
                super::state_presentation::light_state_surface(ui, |ui| {
                    ui.colored_label(super::state_presentation::StudioStateTokens::SECONDARY, "当前选择尚未应用；应用后才进入显示设置保存状态。");
                });
            }
            ui.horizontal(|ui| {
                if ui.button("应用显示设置").clicked() {
                    match self.apply_presentation_command(ProjectPresentationCommand::Apply(
                        units.clone(),
                    )) {
                        Ok(()) => close = true,
                        Err(error) => self.unit_settings.notice = Some(ProjectOpenNotice { level: ProjectOpenNoticeLevel::Error, title: "显示设置未应用".into(), detail: error.to_string() }),
                    }
                }
                if ui.button("取消").clicked() {
                    close = true;
                }
            });
            ui.label("撤销 / 重做按项目与检查器共用的呈现历史执行；恢复已保存设置只影响项目。");
            ui.horizontal(|ui| {
                for (label, command, enabled) in [
                    (
                        "撤销显示设置",
                        ProjectPresentationCommand::Undo,
                        document.presentation.can_undo(),
                    ),
                    (
                        "重做显示设置",
                        ProjectPresentationCommand::Redo,
                        document.presentation.can_redo(),
                    ),
                    (
                        "恢复已保存设置",
                        ProjectPresentationCommand::RestoreSaved,
                        document.save_state.presentation_dirty,
                    ),
                ] {
                    if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                        match self.apply_presentation_command(command) {
                            Ok(()) => {
                                units = self
                                    .platform_host
                                    .snapshot()
                                    .runtime
                                    .workspace_document
                                    .presentation
                                    .display_units()
                                    .clone()
                            }
                            Err(error) => self.unit_settings.notice = Some(ProjectOpenNotice { level: ProjectOpenNoticeLevel::Error, title: "显示设置未修改".into(), detail: error.to_string() }),
                        }
                    }
                }
            });
            ui.separator();
            ui.label("个人默认只采用已应用的项目选择；未应用的设置草稿不写入默认。");
            if let Err(error) = &self.unit_settings.default_units {
                ui.label(format!("个人默认未加载：{error}"));
            }
            if ui
                .add_enabled(
                    !self.unit_settings.recovery_required,
                    egui::Button::new("设为新工程默认"),
                )
                .clicked()
            {
                self.save_unit_default(false);
            }
            if self.unit_settings.recovery_required && ui.button("恢复个人默认…").clicked()
            {
                self.request_unit_default_recovery();
            }
            if let Some(notice) = &self.unit_settings.notice {
                render_project_notice(ui, notice);
            }
        });
        self.unit_settings.draft = if close || response.should_close() {
            None
        } else {
            Some((document_id, units))
        };
        true
    }
}
