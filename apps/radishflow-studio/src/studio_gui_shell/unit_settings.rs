use super::locale::UnitSettingsText;
use super::*;
use rf_types::units::{ALL_UNITS, DisplayUnitSet};
use rf_ui::ProjectPresentationCommand;

enum UnitSettingsNotice {
    DefaultsSaved,
    DefaultsRecovered(PathBuf),
    DefaultsSaveFailed(String),
    ApplyFailed(String),
    HistoryFailed(String),
}

pub(super) struct UnitSettingsState {
    pub draft: Option<(String, DisplayUnitSet)>,
    pub view_draft: Option<super::view_units::ViewUnitSettingsDraft>,
    pub closed_inspectors: std::collections::BTreeSet<rf_ui::DisplayUnitViewId>,
    pub default_units: Result<DisplayUnitSet, String>,
    pub pending_new: Option<Option<AuthoringCaseKind>>,
    default_path: PathBuf,
    notice: Option<UnitSettingsNotice>,
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
                .map(UnitSettingsNotice::DefaultsRecovered)
        } else {
            rf_store::write_unit_defaults(&self.unit_settings.default_path, &units)
                .map(|()| UnitSettingsNotice::DefaultsSaved)
        };
        match result {
            Ok(notice) => {
                self.unit_settings.default_units = Ok(units);
                self.unit_settings.recovery_required = false;
                self.unit_settings.notice = Some(notice);
            }
            Err(error) => {
                if recover {
                    self.unit_settings.pending_recovery = Some(units);
                }
                self.unit_settings.notice =
                    Some(UnitSettingsNotice::DefaultsSaveFailed(error.to_string()));
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

    fn render_unit_settings_notice(&self, ui: &mut egui::Ui) {
        let Some(notice) = &self.unit_settings.notice else {
            return;
        };
        let (level, title, detail) = match notice {
            UnitSettingsNotice::DefaultsSaved => (
                ProjectOpenNoticeLevel::Info,
                UnitSettingsText::DefaultsSaved,
                self.locale
                    .unit_settings_text(UnitSettingsText::DefaultsSavedHelp)
                    .to_owned(),
            ),
            UnitSettingsNotice::DefaultsRecovered(backup) => (
                ProjectOpenNoticeLevel::Info,
                UnitSettingsText::DefaultsSaved,
                self.locale.unit_defaults_recovered(backup),
            ),
            UnitSettingsNotice::DefaultsSaveFailed(error) => (
                ProjectOpenNoticeLevel::Error,
                UnitSettingsText::DefaultsSaveFailed,
                format!(
                    "{}\n{}: {error}",
                    self.locale
                        .unit_settings_text(UnitSettingsText::DefaultsSaveFailedHelp),
                    self.locale
                        .unit_settings_text(UnitSettingsText::DiagnosticDetails)
                ),
            ),
            UnitSettingsNotice::ApplyFailed(error) => (
                ProjectOpenNoticeLevel::Error,
                UnitSettingsText::ApplyFailed,
                error.clone(),
            ),
            UnitSettingsNotice::HistoryFailed(error) => (
                ProjectOpenNoticeLevel::Error,
                UnitSettingsText::HistoryFailed,
                error.clone(),
            ),
        };
        render_project_notice(
            ui,
            &ProjectOpenNotice {
                level,
                title: self.locale.unit_settings_text(title).to_owned(),
                detail,
            },
        );
    }

    pub(super) fn render_unit_settings(&mut self, ctx: &egui::Context) -> bool {
        if self.render_view_unit_settings(ctx) {
            return true;
        }
        let locale = self.locale;
        if let Some(authoring) = self.unit_settings.pending_new {
            let response = egui::Modal::new(egui::Id::new("invalid-unit-default-new-project"))
                .show(ctx, |ui| {
                    ui.set_width(520.0);
                    ui.heading(locale.unit_settings_text(UnitSettingsText::DefaultUnavailable));
                    if let Err(error) = &self.unit_settings.default_units {
                        ui.collapsing(
                            locale.unit_settings_text(UnitSettingsText::DiagnosticDetails),
                            |ui| {
                                ui.label(error);
                            },
                        );
                    }
                    ui.label(locale.unit_settings_text(UnitSettingsText::DefaultUnavailableHelp));
                    if ui
                        .button(locale.unit_settings_text(UnitSettingsText::UseSiOnce))
                        .clicked()
                    {
                        self.unit_settings.pending_new = None;
                        self.create_blank_project_with_units(authoring, DisplayUnitSet::si());
                    }
                    if ui.button(locale.text(ShellText::Cancel)).clicked() {
                        self.unit_settings.pending_new = None;
                    }
                });
            if response.should_close() {
                self.unit_settings.pending_new = None;
            }
            return true;
        }
        if self.unit_settings.pending_recovery.is_some() {
            let response =
                egui::Modal::new(egui::Id::new("recover-unit-defaults")).show(ctx, |ui| {
                    ui.set_width(520.0);
                    ui.heading(locale.unit_settings_text(UnitSettingsText::RecoveryTitle));
                    ui.label(locale.unit_settings_text(UnitSettingsText::RecoveryHelp));
                    ui.label(self.unit_settings.default_path.display().to_string());
                    self.render_unit_settings_notice(ui);
                    if ui
                        .button(locale.unit_settings_text(UnitSettingsText::BackupAndReplace))
                        .clicked()
                    {
                        self.save_unit_default(true);
                    }
                    if ui.button(locale.text(ShellText::Cancel)).clicked() {
                        self.unit_settings.pending_recovery = None;
                    }
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
            ui.set_width(520.0);
            ui.heading(locale.unit_settings_text(UnitSettingsText::Title));
            ui.label(locale.unit_settings_text(UnitSettingsText::Scope));
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(locale.unit_settings_text(UnitSettingsText::FullSi))
                    .clicked()
                {
                    units = DisplayUnitSet::si();
                }
                if ui
                    .button(locale.unit_settings_text(UnitSettingsText::Engineering))
                    .clicked()
                {
                    units = DisplayUnitSet::engineering();
                }
            });
            let choices: Vec<_> = units.entries().collect();
            egui::Grid::new("project-display-unit-choices").show(ui, |ui| {
                for (quantity, selected) in choices {
                    let label = locale.quantity_name(quantity);
                    ui.label(label);
                    let choices: Vec<_> = ALL_UNITS
                        .iter()
                        .filter(|unit| units.clone().set_unit(quantity, **unit).is_ok())
                        .map(|unit| (*unit, unit.definition().symbol))
                        .collect();
                    if let Some(unit) = super::unit_selector::unit_selector(
                        ui,
                        quantity.definition().id,
                        &locale.project_unit_selector_name(quantity),
                        selected,
                        &choices,
                    )
                    .inner
                    {
                        units
                            .set_unit(quantity, unit)
                            .expect("menu contains valid units");
                    }
                    ui.end_row();
                }
            });
            render_project_save_state(ui, &document.save_state, self.locale);
            if units != *document.presentation.display_units() {
                super::state_presentation::light_state_surface(ui, |ui| {
                    ui.colored_label(
                        super::state_presentation::StudioStateTokens::SECONDARY,
                        locale.unit_settings_text(UnitSettingsText::Unapplied),
                    );
                });
            }
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(locale.unit_settings_text(UnitSettingsText::Apply))
                    .clicked()
                {
                    match self.apply_presentation_command(ProjectPresentationCommand::Apply(
                        units.clone(),
                    )) {
                        Ok(()) => close = true,
                        Err(error) => {
                            self.unit_settings.notice =
                                Some(UnitSettingsNotice::ApplyFailed(error.to_string()))
                        }
                    }
                }
                if ui.button(locale.text(ShellText::Cancel)).clicked() {
                    close = true;
                }
            });
            ui.label(locale.unit_settings_text(UnitSettingsText::HistoryScope));
            ui.horizontal_wrapped(|ui| {
                for (label, command, enabled) in [
                    (
                        locale.unit_settings_text(UnitSettingsText::Undo),
                        ProjectPresentationCommand::Undo,
                        document.presentation.can_undo(),
                    ),
                    (
                        locale.unit_settings_text(UnitSettingsText::Redo),
                        ProjectPresentationCommand::Redo,
                        document.presentation.can_redo(),
                    ),
                    (
                        locale.unit_settings_text(UnitSettingsText::RestoreSaved),
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
                            Err(error) => {
                                self.unit_settings.notice =
                                    Some(UnitSettingsNotice::HistoryFailed(error.to_string()))
                            }
                        }
                    }
                }
            });
            ui.separator();
            ui.label(locale.unit_settings_text(UnitSettingsText::PersonalScope));
            if let Err(error) = &self.unit_settings.default_units {
                ui.label(locale.unit_settings_text(UnitSettingsText::DefaultLoadFailed));
                ui.collapsing(
                    locale.unit_settings_text(UnitSettingsText::DiagnosticDetails),
                    |ui| {
                        ui.label(error);
                    },
                );
            }
            if ui
                .add_enabled(
                    !self.unit_settings.recovery_required,
                    egui::Button::new(locale.unit_settings_text(UnitSettingsText::SaveDefaults)),
                )
                .clicked()
            {
                self.save_unit_default(false);
            }
            if self.unit_settings.recovery_required
                && ui
                    .button(locale.unit_settings_text(UnitSettingsText::RecoverDefaults))
                    .clicked()
            {
                self.request_unit_default_recovery();
            }
            self.render_unit_settings_notice(ui);
        });
        self.unit_settings.draft = if close || response.should_close() {
            None
        } else {
            Some((document_id, units))
        };
        true
    }
}
