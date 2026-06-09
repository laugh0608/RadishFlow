use super::*;

impl ReadyAppState {
    pub(super) fn render_module_settings_panel(
        &mut self,
        ui: &mut egui::Ui,
        settings: &radishflow_studio::StudioGuiWindowModuleSettingsModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(self.locale.runtime_label(settings.title).as_ref()).strong(),
            );
            render_status_chip(
                ui,
                self.locale.runtime_label(settings.state_label).as_ref(),
                module_settings_state_color(settings.state),
            );
            if let Some(unit) = settings.selected_unit.as_ref() {
                ui.small(self.locale.runtime_label(unit.kind_label).as_ref());
                let _ = self.render_small_command_action(ui, &unit.action);
            }
        });
        render_wrapped_label(ui, &settings.detail);

        if !settings.summary_rows.is_empty() {
            egui::Grid::new(format!(
                "module-settings-summary:{}",
                settings
                    .selected_unit
                    .as_ref()
                    .map(|unit| unit.command_id.as_str())
                    .unwrap_or("none")
            ))
            .num_columns(2)
            .spacing([8.0, 3.0])
            .show(ui, |ui| {
                for row in &settings.summary_rows {
                    ui.small(egui::RichText::new(&row.label).strong());
                    render_wrapped_small(ui, &row.value);
                    ui.end_row();
                }
            });
        }

        if !settings.connection_actions.is_empty() {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.small(egui::RichText::new("Connections").strong());
                for action in &settings.connection_actions {
                    if ui
                        .add_enabled(
                            action.enabled,
                            egui::Button::new(
                                egui::RichText::new(
                                    self.locale.runtime_label(&action.label).as_ref(),
                                )
                                .small(),
                            ),
                        )
                        .on_hover_text(self.locale.runtime_label(&action.hover_text).as_ref())
                        .clicked()
                    {
                        self.dispatch_ui_command(&action.command_id);
                    }
                }
            });
        }

        if let Some(summary) = settings.parameter_summary.as_ref() {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.small(
                    egui::RichText::new(self.locale.runtime_label(summary.title).as_ref()).strong(),
                );
                render_status_chip(
                    ui,
                    self.locale.runtime_label(summary.status_label).as_ref(),
                    inspector_field_status_color(summary.status_label),
                );
                render_status_chip(
                    ui,
                    &format!(
                        "{} {}",
                        summary.total_field_count,
                        self.locale.runtime_label("Fields")
                    ),
                    egui::Color32::from_rgb(86, 118, 168),
                );
                render_status_chip(
                    ui,
                    &format!(
                        "{} {}",
                        summary.dirty_field_count,
                        self.locale.runtime_label("Drafts")
                    ),
                    inspector_field_status_color(if summary.dirty_field_count > 0 {
                        "Draft"
                    } else {
                        "Synced"
                    }),
                );
                render_status_chip(
                    ui,
                    &format!(
                        "{} {}",
                        summary.issue_count,
                        self.locale.runtime_label("Issues")
                    ),
                    inspector_field_status_color(if summary.issue_count > 0 {
                        "Invalid"
                    } else {
                        "Synced"
                    }),
                );
                render_status_chip(
                    ui,
                    &format!(
                        "{} {}",
                        summary.notice_count,
                        self.locale.runtime_label("Notices")
                    ),
                    egui::Color32::from_rgb(96, 106, 118),
                );
                if summary.batch_commit_available {
                    render_status_chip(
                        ui,
                        self.locale.runtime_label("Commit ready").as_ref(),
                        egui::Color32::from_rgb(54, 128, 84),
                    );
                }
            });
            render_wrapped_small(ui, &summary.detail);
        }

        if !settings.parameter_fields.is_empty() {
            ui.add_space(4.0);
            ui.small(
                egui::RichText::new(self.locale.text(ShellText::InspectorProperties)).strong(),
            );
            if let Some(command_id) = settings.parameter_batch_commit_command_id.as_ref() {
                if ui
                    .small_button(self.locale.text(ShellText::InspectorFieldApplyAll))
                    .clicked()
                {
                    self.dispatch_inspector_field_draft_batch_commit(command_id.clone());
                }
            }
            if let Some(command_id) = settings.parameter_batch_discard_command_id.as_ref() {
                if ui
                    .small_button(self.locale.text(ShellText::InspectorFieldDiscardAll))
                    .clicked()
                {
                    self.dispatch_inspector_field_draft_batch_discard(command_id.clone());
                }
            }
            for notice in &settings.parameter_notices {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(notice.status_label).as_ref(),
                        inspector_field_status_color(notice.status_label),
                    );
                    render_wrapped_small(ui, &notice.message);
                });
            }
            for field in &settings.parameter_fields {
                self.render_inspector_property_field(ui, field);
            }
        }

        if !settings.ports.is_empty() {
            ui.add_space(4.0);
            ui.small(egui::RichText::new(self.locale.text(ShellText::InspectorPorts)).strong());
            egui::Grid::new(format!(
                "module-settings-ports:{}",
                settings
                    .selected_unit
                    .as_ref()
                    .map(|unit| unit.command_id.as_str())
                    .unwrap_or("none")
            ))
            .num_columns(5)
            .spacing([8.0, 3.0])
            .show(ui, |ui| {
                ui.small(
                    egui::RichText::new(self.locale.text(ShellText::InspectorPortName)).strong(),
                );
                ui.small(
                    egui::RichText::new(self.locale.text(ShellText::InspectorPortDirection))
                        .strong(),
                );
                ui.small(
                    egui::RichText::new(self.locale.text(ShellText::InspectorPortKind)).strong(),
                );
                ui.small(
                    egui::RichText::new(self.locale.text(ShellText::InspectorPortStream)).strong(),
                );
                ui.small(egui::RichText::new("Attention").strong());
                ui.end_row();
                for port in &settings.ports {
                    render_wrapped_small(ui, &port.name);
                    render_wrapped_small(ui, &port.direction);
                    render_wrapped_small(ui, &port.kind);
                    match (&port.stream_id, &port.stream_action) {
                        (Some(stream_id), Some(action)) => {
                            self.render_port_stream_action(ui, stream_id, action);
                        }
                        (None, Some(action)) => {
                            let _ = self.render_small_command_action(ui, action);
                        }
                        (Some(stream_id), None) => render_wrapped_small(ui, stream_id),
                        (None, None) => {
                            ui.small("-");
                        }
                    };
                    if let Some(summary) = port.attention_summary.as_ref() {
                        ui.vertical(|ui| {
                            render_status_chip(
                                ui,
                                "attention",
                                notice_color(rf_ui::RunPanelNoticeLevel::Warning),
                            );
                            render_wrapped_small(ui, summary);
                        });
                    } else {
                        ui.small("-");
                    }
                    ui.end_row();
                }
            });
        }

        ui.add_space(4.0);
        ui.small(egui::RichText::new(self.locale.runtime_label("Help").as_ref()).strong());
        if settings.help_actions.is_empty() {
            render_wrapped_small(ui, &settings.help_detail);
        } else {
            ui.horizontal_wrapped(|ui| {
                for action in &settings.help_actions {
                    let _ = self.render_small_command_action(ui, action);
                }
            });
        }

        if !settings.diagnostic_actions.is_empty() {
            ui.add_space(4.0);
            ui.collapsing(self.locale.text(ShellText::DiagnosticTargets), |ui| {
                self.render_diagnostic_target_actions(ui, &settings.diagnostic_actions);
            });
        }

        if !settings.related_diagnostics.is_empty() {
            ui.add_space(4.0);
            ui.collapsing(self.locale.text(ShellText::RelatedDiagnostics), |ui| {
                for (index, diagnostic) in settings.related_diagnostics.iter().enumerate() {
                    self.render_diagnostic_summary(
                        ui,
                        diagnostic,
                        format!("module-settings:diagnostic:{index}"),
                    );
                    ui.add_space(4.0);
                }
            });
        }
    }

    pub(super) fn render_active_inspector_detail(
        &mut self,
        ui: &mut egui::Ui,
        detail: &radishflow_studio::StudioGuiWindowInspectorTargetDetailModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(self.locale.text(ShellText::ActiveInspectorTarget)).strong(),
            );
            render_status_chip(
                ui,
                self.locale.runtime_label(detail.target.kind_label).as_ref(),
                egui::Color32::from_rgb(86, 118, 168),
            );
            ui.small(&detail.target.target_id);
        });
        render_wrapped_label(ui, &detail.title);

        if !detail.summary_rows.is_empty() {
            egui::Grid::new(format!("inspector-summary:{}", detail.target.command_id))
                .num_columns(2)
                .spacing([8.0, 3.0])
                .show(ui, |ui| {
                    for row in &detail.summary_rows {
                        ui.small(egui::RichText::new(&row.label).strong());
                        render_wrapped_small(ui, &row.value);
                        ui.end_row();
                    }
                });
        }

        if !detail.connection_actions.is_empty() {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.small(egui::RichText::new("Connections").strong());
                for action in &detail.connection_actions {
                    if ui
                        .add_enabled(
                            action.enabled,
                            egui::Button::new(
                                egui::RichText::new(
                                    self.locale.runtime_label(&action.label).as_ref(),
                                )
                                .small(),
                            ),
                        )
                        .on_hover_text(self.locale.runtime_label(&action.hover_text).as_ref())
                        .clicked()
                    {
                        self.dispatch_ui_command(&action.command_id);
                    }
                }
            });
        }

        if !detail.property_fields.is_empty() {
            ui.add_space(4.0);
            ui.small(
                egui::RichText::new(self.locale.text(ShellText::InspectorProperties)).strong(),
            );
            if let Some(command_id) = detail.property_batch_commit_command_id.as_ref() {
                if ui
                    .small_button(self.locale.text(ShellText::InspectorFieldApplyAll))
                    .clicked()
                {
                    self.dispatch_inspector_field_draft_batch_commit(command_id.clone());
                }
            }
            if let Some(command_id) = detail.property_batch_discard_command_id.as_ref() {
                if ui
                    .small_button(self.locale.text(ShellText::InspectorFieldDiscardAll))
                    .clicked()
                {
                    self.dispatch_inspector_field_draft_batch_discard(command_id.clone());
                }
            }
            if let Some(command_id) = detail.property_composition_normalize_command_id.as_ref() {
                if ui
                    .small_button(self.locale.text(ShellText::InspectorNormalizeComposition))
                    .clicked()
                {
                    self.dispatch_inspector_composition_normalize(command_id.clone());
                }
            }
            for notice in &detail.property_notices {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(notice.status_label).as_ref(),
                        inspector_field_status_color(notice.status_label),
                    );
                    render_wrapped_small(ui, &notice.message);
                });
            }
            if let Some(summary) = detail.property_composition_summary.as_ref() {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.small(
                        egui::RichText::new(
                            self.locale.text(ShellText::InspectorCompositionSummary),
                        )
                        .strong(),
                    );
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(summary.status_label).as_ref(),
                        inspector_field_status_color(summary.status_label),
                    );
                    ui.small(format!(
                        "{} {}",
                        self.locale.text(ShellText::InspectorCompositionSum),
                        summary.current_sum_text
                    ));
                });
                render_wrapped_small(
                    ui,
                    format!(
                        "{}: {}",
                        self.locale.text(ShellText::InspectorNormalizedPreview),
                        summary.normalized_preview_text
                    ),
                );
            }
            if !detail.property_composition_component_actions.is_empty() {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.small(egui::RichText::new("Add component").strong());
                    for component_action in &detail.property_composition_component_actions {
                        if ui
                            .small_button(&component_action.action.label)
                            .on_hover_text(&component_action.action.hover_text)
                            .clicked()
                        {
                            self.dispatch_inspector_composition_component_add(
                                component_action.action.command_id.clone(),
                            );
                        }
                    }
                });
            }
            for field in &detail.property_fields {
                self.render_inspector_property_field(ui, field);
            }
        }

        if !detail.unit_ports.is_empty() {
            ui.add_space(4.0);
            ui.small(egui::RichText::new(self.locale.text(ShellText::InspectorPorts)).strong());
            egui::Grid::new(format!("inspector-ports:{}", detail.target.command_id))
                .num_columns(5)
                .spacing([8.0, 3.0])
                .show(ui, |ui| {
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::InspectorPortName))
                            .strong(),
                    );
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::InspectorPortDirection))
                            .strong(),
                    );
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::InspectorPortKind))
                            .strong(),
                    );
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::InspectorPortStream))
                            .strong(),
                    );
                    ui.small(egui::RichText::new("Attention").strong());
                    ui.end_row();
                    for port in &detail.unit_ports {
                        render_wrapped_small(ui, &port.name);
                        render_wrapped_small(ui, &port.direction);
                        render_wrapped_small(ui, &port.kind);
                        match (&port.stream_id, &port.stream_action) {
                            (Some(stream_id), Some(action)) => {
                                self.render_port_stream_action(ui, stream_id, action);
                            }
                            (None, Some(action)) => {
                                let _ = self.render_small_command_action(ui, action);
                            }
                            (Some(stream_id), None) => render_wrapped_small(ui, stream_id),
                            (None, None) => {
                                ui.small("-");
                            }
                        };
                        if let Some(summary) = port.attention_summary.as_ref() {
                            ui.vertical(|ui| {
                                render_status_chip(
                                    ui,
                                    "attention",
                                    notice_color(rf_ui::RunPanelNoticeLevel::Warning),
                                );
                                render_wrapped_small(ui, summary);
                            });
                        } else {
                            ui.small("-");
                        }
                        ui.end_row();
                    }
                });
        }

        if let Some(unit) = detail.latest_unit_result.as_ref() {
            ui.add_space(4.0);
            ui.small(
                egui::RichText::new(self.locale.text(ShellText::InspectorLatestResult)).strong(),
            );
            self.render_unit_execution_result_inspector(ui, unit);
        }

        if let Some(stream) = detail.latest_stream_result.as_ref() {
            ui.add_space(4.0);
            ui.small(
                egui::RichText::new(self.locale.text(ShellText::InspectorLatestResult)).strong(),
            );
            ui.push_id(
                format!(
                    "active-inspector-latest-stream:{}",
                    detail.target.command_id
                ),
                |ui| self.render_stream_result_inspector(ui, stream),
            );
        }

        if !detail.related_steps.is_empty() {
            ui.add_space(4.0);
            ui.collapsing(self.locale.text(ShellText::RelatedSolveSteps), |ui| {
                for step in &detail.related_steps {
                    self.render_solve_step_inspector(ui, step);
                }
            });
        }

        if !detail.diagnostic_actions.is_empty() {
            ui.add_space(4.0);
            ui.collapsing(self.locale.text(ShellText::DiagnosticTargets), |ui| {
                self.render_diagnostic_target_actions(ui, &detail.diagnostic_actions);
            });
        }

        if !detail.related_diagnostics.is_empty() {
            ui.add_space(4.0);
            ui.collapsing(self.locale.text(ShellText::RelatedDiagnostics), |ui| {
                for (index, diagnostic) in detail.related_diagnostics.iter().enumerate() {
                    self.render_diagnostic_summary(
                        ui,
                        diagnostic,
                        format!(
                            "active-inspector:{}:diagnostic:{index}",
                            detail.target.command_id
                        ),
                    );
                    ui.add_space(4.0);
                }
            });
        }
    }

    fn render_inspector_property_field(
        &mut self,
        ui: &mut egui::Ui,
        field: &radishflow_studio::StudioGuiWindowInspectorTargetFieldModel,
    ) {
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(inspector_field_label(self.locale, field)).strong());
            render_status_chip(
                ui,
                self.locale.runtime_label(field.status_label).as_ref(),
                inspector_field_status_color(field.status_label),
            );
            ui.small(self.locale.runtime_label(field.value_kind_label).as_ref());
        });

        ui.horizontal_wrapped(|ui| {
            let input_width = (ui.available_width() * 0.58).clamp(120.0, 240.0);
            let mut draft_value = field.current_value.clone();
            let response = ui.add_sized(
                [input_width, 24.0],
                egui::TextEdit::singleline(&mut draft_value)
                    .id_salt(format!("inspector-property-field:{}", field.key)),
            );
            if response.changed() {
                self.dispatch_inspector_field_draft_update(
                    field.draft_update_command_id.clone(),
                    draft_value,
                );
            }
            let submit_on_enter = response.lost_focus()
                && ui.input(|input| {
                    input.key_pressed(egui::Key::Enter) && input.modifiers == egui::Modifiers::NONE
                });
            if let Some(unit_label) = inspector_field_unit_label(field) {
                ui.small(egui::RichText::new(unit_label).strong());
            }

            if let Some(command_id) = field.commit_command_id.as_ref() {
                if submit_on_enter
                    || ui
                        .small_button(self.locale.text(ShellText::InspectorFieldApply))
                        .clicked()
                {
                    self.dispatch_inspector_field_draft_commit(command_id.clone());
                }
            }
            if let Some(command_id) = field.discard_command_id.as_ref() {
                if ui
                    .small_button(self.locale.text(ShellText::InspectorFieldDiscard))
                    .clicked()
                {
                    self.dispatch_inspector_field_draft_discard(command_id.clone());
                }
            }
            if let Some(remove_command_id) = field.remove_command_id.as_ref() {
                if ui
                    .small_button(
                        self.locale
                            .text(ShellText::InspectorRemoveCompositionComponent),
                    )
                    .clicked()
                {
                    self.dispatch_inspector_composition_component_remove(remove_command_id.clone());
                }
            }
        });

        if let Some(constraint_text) = field.constraint_text.as_ref() {
            render_wrapped_small(
                ui,
                localized_inspector_constraint(self.locale, constraint_text).as_ref(),
            );
        }
    }

    fn render_port_stream_action(
        &mut self,
        ui: &mut egui::Ui,
        stream_id: &str,
        action: &radishflow_studio::StudioGuiWindowCommandActionModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            render_wrapped_small(ui, stream_id);
            let label = match self.locale {
                StudioShellLocale::En => "Open stream",
                StudioShellLocale::ZhCn => "打开流股",
            };
            let response = ui.small_button(label).on_hover_text(&action.hover_text);
            if response.clicked() {
                self.dispatch_ui_command(&action.command_id);
            }
        });
    }
}

fn inspector_field_label<'a>(
    locale: StudioShellLocale,
    field: &'a radishflow_studio::StudioGuiWindowInspectorTargetFieldModel,
) -> std::borrow::Cow<'a, str> {
    if matches!(locale, StudioShellLocale::ZhCn) {
        match field.label.as_str() {
            "Source temperature (K)" => return std::borrow::Cow::Borrowed("源温度"),
            "Source pressure (Pa)" => return std::borrow::Cow::Borrowed("源压力"),
            "Outlet temperature (K)" => return std::borrow::Cow::Borrowed("出口温度"),
            "Outlet pressure (Pa)" => return std::borrow::Cow::Borrowed("出口压力"),
            "Flash temperature (K)" => return std::borrow::Cow::Borrowed("闪蒸温度"),
            "Flash pressure (Pa)" => return std::borrow::Cow::Borrowed("闪蒸压力"),
            "Temperature (K)" => return std::borrow::Cow::Borrowed("温度"),
            "Pressure (Pa)" => return std::borrow::Cow::Borrowed("压力"),
            "Total molar flow (mol/s)" => return std::borrow::Cow::Borrowed("总摩尔流量"),
            "Name" => return std::borrow::Cow::Borrowed("名称"),
            _ => {}
        }
    }

    if let Some((label, _unit)) = field.label.rsplit_once(" (") {
        return std::borrow::Cow::Owned(label.to_string());
    }
    std::borrow::Cow::Borrowed(field.label.as_str())
}

fn inspector_field_unit_label(
    field: &radishflow_studio::StudioGuiWindowInspectorTargetFieldModel,
) -> Option<&str> {
    field
        .label
        .rsplit_once(" (")
        .and_then(|(_, unit)| unit.strip_suffix(')'))
}

fn localized_inspector_constraint<'a>(
    locale: StudioShellLocale,
    text: &'a str,
) -> std::borrow::Cow<'a, str> {
    if !matches!(locale, StudioShellLocale::ZhCn) {
        return std::borrow::Cow::Borrowed(text);
    }

    match text {
        "Unit K; positive finite source outlet temperature; commit syncs the Feed outlet template." => {
            std::borrow::Cow::Borrowed("单位 K；输入正数，提交后同步 Feed outlet。")
        }
        "Unit K; positive finite flash temperature; commit syncs liquid/vapor outlet templates." => {
            std::borrow::Cow::Borrowed("单位 K；输入正数，提交后同步液相 / 气相出口。")
        }
        "Unit K; positive finite outlet temperature; commit syncs the outlet stream template." => {
            std::borrow::Cow::Borrowed("单位 K；输入正数，提交后同步出口流股。")
        }
        "Unit Pa; positive finite source outlet pressure; commit syncs the Feed outlet template." => {
            std::borrow::Cow::Borrowed("单位 Pa；输入正数，提交后同步 Feed outlet。")
        }
        "Unit Pa; positive finite flash pressure; commit syncs liquid/vapor outlet templates." => {
            std::borrow::Cow::Borrowed("单位 Pa；输入正数，提交后同步液相 / 气相出口。")
        }
        _ => {
            if let Some(limit) = text.strip_prefix(
                "Unit Pa; positive finite outlet pressure; cannot exceed connected inlet pressure. Inlet limit: ",
            ) {
                return std::borrow::Cow::Owned(format!(
                    "单位 Pa；输入正数，不能高于已连接入口压力。入口上限: {limit}"
                ));
            }
            if text
                == "Unit Pa; positive finite outlet pressure; cannot exceed connected inlet pressure."
            {
                return std::borrow::Cow::Borrowed("单位 Pa；输入正数，不能高于已连接入口压力。");
            }
            std::borrow::Cow::Borrowed(text)
        }
    }
}

fn module_settings_state_color(
    state: radishflow_studio::StudioGuiWindowModuleSettingsState,
) -> egui::Color32 {
    match state {
        radishflow_studio::StudioGuiWindowModuleSettingsState::Ready => {
            egui::Color32::from_rgb(74, 132, 92)
        }
        radishflow_studio::StudioGuiWindowModuleSettingsState::UnitDetailUnavailable
        | radishflow_studio::StudioGuiWindowModuleSettingsState::NoUnitSelected => {
            egui::Color32::from_rgb(160, 120, 40)
        }
    }
}
