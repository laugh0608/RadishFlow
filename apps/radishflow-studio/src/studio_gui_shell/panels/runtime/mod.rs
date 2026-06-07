use super::super::*;
use super::commands::entitlement_command_id;

mod inspector;
mod property;
mod results;

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn render_runtime_area(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
    ) {
        egui::ScrollArea::vertical()
            .id_salt(format!(
                "scroll:{}:{}:runtime-area",
                window.layout_state.scope.layout_key,
                area_label(area_id)
            ))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.render_runtime_area_contents(ui, window, area_id);
            });
    }

    pub(in crate::studio_gui_shell) fn render_runtime_inspector_tab(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        if let Some(detail) = window.runtime.active_inspector_detail.as_ref() {
            ui.push_id(
                format!("runtime:active-inspector:{}", detail.target.command_id),
                |ui| self.render_active_inspector_detail(ui, detail),
            );
            return;
        }

        ui.label(egui::RichText::new(self.locale.text(ShellText::InspectorProperties)).strong());
        if let Some(target) = window.runtime.active_inspector_target.as_ref() {
            render_wrapped_label(ui, &target.summary);
            let _ = self.render_small_command_action(ui, &target.action);
        } else {
            ui.small(self.locale.text(ShellText::NoActiveInspectorTarget));
        }
    }

    pub(in crate::studio_gui_shell) fn render_runtime_module_settings_tab(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        ui.push_id(
            format!(
                "runtime:module-settings:{}",
                window
                    .module_settings
                    .selected_unit
                    .as_ref()
                    .map(|unit| unit.command_id.as_str())
                    .unwrap_or("none")
            ),
            |ui| self.render_module_settings_panel(ui, &window.module_settings),
        );
    }

    pub(in crate::studio_gui_shell) fn render_runtime_module_results_tab(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        self.render_module_results_summary(ui, &window.module_results);
    }

    pub(in crate::studio_gui_shell) fn render_module_results_summary(
        &mut self,
        ui: &mut egui::Ui,
        module: &radishflow_studio::StudioGuiWindowModuleResultsModel,
    ) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(self.locale.runtime_label(module.title).as_ref()).strong(),
                );
                render_status_chip(
                    ui,
                    self.locale.runtime_label(module.state_label).as_ref(),
                    module_results_state_color(module.state),
                );
                if let Some(unit) = module.selected_unit.as_ref() {
                    ui.small(self.locale.runtime_label(unit.kind_label).as_ref());
                    let _ = self.render_small_command_action(ui, &unit.action);
                }
            });

            if let Some(stale) = module.stale_snapshot.as_ref() {
                render_wrapped_small(
                    ui,
                    self.locale.stale_solve_snapshot_detail(
                        &stale.snapshot_id,
                        stale.snapshot_document_revision,
                        stale.current_document_revision,
                    ),
                );
                return;
            }

            render_wrapped_small(ui, &module.detail);
            if let Some(unit) = module.selected_unit_result.as_ref() {
                self.render_unit_execution_result_inspector(ui, unit);
            }

            if !module.related_steps.is_empty() {
                ui.collapsing(self.locale.text(ShellText::RelatedSolveSteps), |ui| {
                    for step in &module.related_steps {
                        self.render_solve_step_inspector(ui, step);
                    }
                });
            }

            if !module.diagnostic_actions.is_empty() {
                ui.collapsing(self.locale.text(ShellText::DiagnosticTargets), |ui| {
                    self.render_diagnostic_target_actions(ui, &module.diagnostic_actions);
                });
            }

            if !module.related_diagnostics.is_empty() {
                ui.collapsing(self.locale.text(ShellText::RelatedDiagnostics), |ui| {
                    for (index, diagnostic) in module.related_diagnostics.iter().enumerate() {
                        self.render_diagnostic_summary(
                            ui,
                            diagnostic,
                            format!("module-results:diagnostic:{index}"),
                        );
                        ui.add_space(4.0);
                    }
                });
            }
        });
    }

    fn render_solve_snapshot_transfer_actions(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            if ui
                .small_button(self.locale.text(ShellText::CopySnapshot))
                .clicked()
            {
                self.copy_solve_snapshot_to_clipboard(ui.ctx(), snapshot);
            }
            if ui
                .small_button(self.locale.text(ShellText::ExportSnapshot))
                .clicked()
            {
                self.export_solve_snapshot_from_picker(snapshot);
            }
        });
    }

    pub(in crate::studio_gui_shell) fn render_latest_failure_summary(
        &mut self,
        ui: &mut egui::Ui,
        failure: &radishflow_studio::StudioGuiWindowFailureResultModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            render_status_chip(
                ui,
                self.locale.runtime_label(failure.status_label).as_ref(),
                run_status_color(failure.status_label),
            );
            ui.label(egui::RichText::new(
                self.locale.text(ShellText::LastRunFailed),
            ));
        });
        ui.colored_label(
            notice_color(rf_ui::RunPanelNoticeLevel::Error),
            &failure.title,
        );
        render_wrapped_label(ui, &failure.message);
        if let Some(detail) = failure.diagnostic_detail.as_ref() {
            self.render_failure_diagnostic_detail(ui, detail);
        }
        if let Some(message) = failure.latest_log_message.as_ref() {
            render_wrapped_small(
                ui,
                format!("{}: {message}", self.locale.text(ShellText::LatestLog)),
            );
        }
        if let Some(recovery_detail) = failure.recovery_detail {
            ui.add_space(4.0);
            let title = failure
                .recovery_title
                .unwrap_or(self.locale.text(ShellText::SuggestedRecovery));
            ui.small(egui::RichText::new(title).strong());
            render_wrapped_small(ui, recovery_detail);
        }
        if !failure.diagnostic_actions.is_empty() {
            self.render_diagnostic_target_actions(ui, &failure.diagnostic_actions);
        }
    }

    pub(in crate::studio_gui_shell) fn render_stale_solve_snapshot_notice(
        &self,
        ui: &mut egui::Ui,
        stale_snapshot: &radishflow_studio::StudioGuiWindowStaleSolveSnapshotModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            render_status_chip(
                ui,
                self.locale.runtime_label("dirty").as_ref(),
                run_status_color("dirty"),
            );
            ui.label(egui::RichText::new(self.locale.stale_solve_snapshot_title()).strong());
        });
        ui.colored_label(
            notice_color(rf_ui::RunPanelNoticeLevel::Warning),
            self.locale.stale_solve_snapshot_detail(
                &stale_snapshot.snapshot_id,
                stale_snapshot.snapshot_document_revision,
                stale_snapshot.current_document_revision,
            ),
        );
    }

    pub(in crate::studio_gui_shell) fn render_runtime_area_contents(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
    ) {
        let run_panel = &window.runtime.run_panel;
        let run_panel_view = run_panel.view();

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(self.locale.text(ShellText::Run)).strong());
                render_status_chip(
                    ui,
                    self.locale
                        .runtime_label(run_panel_view.mode_label)
                        .as_ref(),
                    egui::Color32::from_rgb(86, 118, 168),
                );
                render_status_chip(
                    ui,
                    self.locale
                        .runtime_label(run_panel_view.status_label)
                        .as_ref(),
                    run_status_color(run_panel_view.status_label),
                );
                if let Some(pending) = run_panel_view.pending_label {
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(pending).as_ref(),
                        egui::Color32::from_rgb(160, 120, 40),
                    );
                }
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let primary = run_panel.primary_action();
                let response = ui.add_enabled(
                    primary.enabled,
                    egui::Button::new(self.locale.runtime_label(primary.label).as_ref())
                        .fill(egui::Color32::from_rgb(230, 239, 252)),
                );
                let response = response.on_hover_text(primary.detail);
                if response.clicked() {
                    self.dispatch_run_panel_widget(run_panel.activate_primary());
                }

                for action in &run_panel_view.secondary_actions {
                    let response = ui.add_enabled(
                        action.enabled,
                        egui::Button::new(self.locale.runtime_label(action.label).as_ref()),
                    );
                    let response = response.on_hover_text(action.detail);
                    if response.clicked() {
                        self.dispatch_run_panel_widget(run_panel.activate(action.id));
                    }
                }
            });
            ui.add_space(6.0);
            if let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() {
                render_wrapped_label(
                    ui,
                    self.locale.solve_snapshot_primary_summary(
                        window.runtime.workspace_document.unit_count,
                        snapshot.diagnostic_count,
                        snapshot.stream_count,
                    ),
                );
            } else if let Some(summary) = run_panel_view.latest_snapshot_summary.as_ref() {
                render_wrapped_label(ui, self.locale.runtime_label(summary).as_ref());
            } else {
                ui.small(self.locale.text(ShellText::NoSolveSnapshot));
            }
            if let Some(snapshot_id) = run_panel_view.latest_snapshot_id.as_ref() {
                render_wrapped_small(
                    ui,
                    format!("{}: {snapshot_id}", self.locale.text(ShellText::Snapshot)),
                );
            }
            if let Some(message) = run_panel_view.latest_log_message.as_ref() {
                render_wrapped_small(
                    ui,
                    format!("{}: {message}", self.locale.text(ShellText::LatestLog)),
                );
            }
            if let Some(target) = window.runtime.active_inspector_target.as_ref() {
                render_wrapped_small(
                    ui,
                    format!(
                        "{}: {}",
                        self.locale.text(ShellText::ActiveInspectorTarget),
                        target.summary
                    ),
                );
            }
            if let Some(notice) = run_panel_view.notice.as_ref() {
                ui.add_space(6.0);
                ui.colored_label(notice_color(notice.level), &notice.title);
                render_wrapped_label(ui, &notice.message);
                if let Some(recovery_action) = notice.recovery_action.as_ref() {
                    render_wrapped_small(ui, recovery_action.detail);
                    if ui.button(recovery_action.title).clicked() {
                        match run_panel.activate_recovery_action() {
                            RunPanelRecoveryWidgetEvent::Requested { .. } => {
                                self.dispatch_ui_command("run_panel.recover_failure");
                            }
                            RunPanelRecoveryWidgetEvent::Missing => {}
                        }
                    }
                }
            }
        });

        if let Some(detail) = window.runtime.active_inspector_detail.as_ref() {
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.push_id(
                    format!("runtime:active-inspector:{}", detail.target.command_id),
                    |ui| self.render_active_inspector_detail(ui, detail),
                );
            });
        }

        if let Some(platform_notice) = window.runtime.platform_notice.as_ref() {
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new(self.locale.text(ShellText::PlatformNotice)).strong());
                ui.colored_label(notice_color(platform_notice.level), &platform_notice.title);
                render_wrapped_label(ui, &platform_notice.message);
                for line in &window.runtime.platform_timer_lines {
                    render_wrapped_small(ui, line);
                }
            });
        } else if !window.runtime.platform_timer_lines.is_empty() {
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new(self.locale.text(ShellText::Platform)).strong());
                for line in &window.runtime.platform_timer_lines {
                    render_wrapped_small(ui, line);
                }
            });
        }

        ui.add_space(8.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            let document = &window.runtime.workspace_document;
            ui.label(egui::RichText::new(self.locale.text(ShellText::Workspace)).strong());
            ui.horizontal_wrapped(|ui| {
                render_wrapped_label(ui, &document.title);
                render_status_chip(
                    ui,
                    &format!("rev {}", document.revision),
                    egui::Color32::from_rgb(86, 118, 168),
                );
                if document.has_unsaved_changes {
                    render_status_chip(
                        ui,
                        self.locale.text(ShellText::Unsaved),
                        egui::Color32::from_rgb(160, 120, 40),
                    );
                }
            });
            ui.small(self.locale.workspace_counts(
                &document.flowsheet_name,
                document.unit_count,
                document.stream_count,
                document.snapshot_history_count,
            ));
            if let Some(path) = document.project_path.as_ref() {
                render_wrapped_small(ui, path);
            }
            ui.separator();
            ui.collapsing(self.locale.text(ShellText::ProjectPath), |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.project_open.path_input)
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(self.locale.text(ShellText::SaveProject))
                        .clicked()
                    {
                        self.save_project();
                    }
                    if ui
                        .button(self.locale.text(ShellText::SaveProjectAs))
                        .clicked()
                    {
                        self.save_project_as_from_picker();
                    }
                    if ui
                        .button(self.locale.text(ShellText::OpenProject))
                        .clicked()
                    {
                        self.open_project_from_input();
                    }
                    if ui
                        .button(self.locale.text(ShellText::BrowseProject))
                        .clicked()
                    {
                        self.open_project_from_picker();
                    }
                    if let Some(path) = document.project_path.as_ref() {
                        if ui
                            .button(self.locale.text(ShellText::UseCurrentPath))
                            .clicked()
                        {
                            self.project_open.path_input = path.clone();
                            self.project_open.notice = None;
                        }
                    }
                });
            });
            if let Some(notice) = self.project_open.notice.as_ref() {
                let color = match notice.level {
                    ProjectOpenNoticeLevel::Info => egui::Color32::from_rgb(66, 118, 92),
                    ProjectOpenNoticeLevel::Warning => egui::Color32::from_rgb(160, 120, 40),
                    ProjectOpenNoticeLevel::Error => egui::Color32::from_rgb(180, 40, 40),
                };
                ui.colored_label(color, &notice.title);
                render_wrapped_small(ui, &notice.detail);
            }
            if self.project_open.pending_confirmation.is_some() {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(self.locale.text(ShellText::ContinueOpenProject))
                        .clicked()
                    {
                        self.confirm_pending_project_open();
                    }
                    if ui
                        .button(self.locale.text(ShellText::CancelOpenProject))
                        .clicked()
                    {
                        self.cancel_pending_project_open();
                    }
                });
            }
            if self.project_open.pending_blank_project_confirmation {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(self.locale.text(ShellText::ContinueNewBlankProject))
                        .clicked()
                    {
                        self.confirm_pending_blank_project();
                    }
                    if ui
                        .button(self.locale.text(ShellText::CancelNewBlankProject))
                        .clicked()
                    {
                        self.cancel_pending_blank_project();
                    }
                });
            }
            if self.project_open.pending_save_as_overwrite.is_some() {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(self.locale.text(ShellText::ConfirmSaveAsOverwrite))
                        .clicked()
                    {
                        self.confirm_pending_save_as_overwrite();
                    }
                    if ui
                        .button(self.locale.text(ShellText::CancelSaveAsOverwrite))
                        .clicked()
                    {
                        self.cancel_pending_save_as_overwrite();
                    }
                });
            }
            ui.separator();
            ui.label(egui::RichText::new(self.locale.text(ShellText::RecentProjects)).strong());
            if self.project_open.recent_projects.is_empty() {
                ui.small(self.locale.text(ShellText::NoRecentProjects));
            } else {
                let current_project_path =
                    document.project_path.as_deref().map(std::path::Path::new);
                let mut requested_recent_project = None;
                for recent_project in self.project_open.recent_projects.clone() {
                    let is_current = current_project_path
                        .map(|current| paths_match(&recent_project, current))
                        .unwrap_or(false);
                    ui.vertical(|ui| {
                        let label = recent_project
                            .file_name()
                            .and_then(|file_name| file_name.to_str())
                            .unwrap_or("project");
                        let button = egui::Button::new(label).selected(is_current);
                        if ui
                            .add_enabled(!is_current, button)
                            .on_hover_text(recent_project.display().to_string())
                            .clicked()
                        {
                            requested_recent_project = Some(recent_project.clone());
                        }
                        render_wrapped_small(ui, recent_project.display().to_string());
                    });
                    ui.add_space(4.0);
                }
                if let Some(project_path) = requested_recent_project {
                    self.open_recent_project(project_path);
                }
            }
            if !window.runtime.example_projects.is_empty() {
                ui.separator();
                ui.label(
                    egui::RichText::new(self.locale.text(ShellText::ExampleProjects)).strong(),
                );
                let mut requested_project = None;
                for example in &window.runtime.example_projects {
                    ui.vertical(|ui| {
                        let button = egui::Button::new(example.title).selected(example.is_current);
                        if ui
                            .add_enabled(!example.is_current, button)
                            .on_hover_text(example.detail)
                            .clicked()
                        {
                            requested_project = Some(example.project_path.clone());
                        }
                        render_wrapped_small(ui, example.detail);
                    });
                    ui.add_space(4.0);
                }
                if let Some(project_path) = requested_project {
                    self.open_example_project(project_path);
                }
            }
        });

        ui.add_space(8.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(self.locale.text(ShellText::Results)).strong());
            if let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() {
                ui.horizontal_wrapped(|ui| {
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(snapshot.status_label).as_ref(),
                        run_status_color(snapshot.status_label),
                    );
                    ui.small(self.locale.solve_snapshot_counts(
                        snapshot.stream_count,
                        snapshot.step_count,
                        snapshot.diagnostic_count,
                    ));
                });
                ui.small(
                    self.locale
                        .snapshot_identity(&snapshot.snapshot_id, snapshot.sequence),
                );
                render_wrapped_label(
                    ui,
                    self.locale.solve_snapshot_primary_summary(
                        window.runtime.workspace_document.unit_count,
                        snapshot.diagnostic_count,
                        snapshot.stream_count,
                    ),
                );
                self.render_solve_snapshot_transfer_actions(ui, snapshot);
                ui.separator();
                if snapshot.streams.is_empty() {
                    ui.small(self.locale.text(ShellText::NoStreamResults));
                } else {
                    let selected_stream_id = self
                        .result_inspector
                        .selected_stream_id_for_snapshot(snapshot);
                    let selected_unit_id = self
                        .result_inspector
                        .selected_unit_id_for_snapshot(snapshot);
                    let inspector = snapshot.result_inspector_with_unit(
                        selected_stream_id.as_deref(),
                        self.result_inspector.comparison_stream_id.as_deref(),
                        selected_unit_id.as_deref(),
                    );
                    ui.push_id(
                        format!("runtime:result-inspector:{}", snapshot.snapshot_id),
                        |ui| self.render_result_inspector(ui, &inspector),
                    );
                }
            } else if let Some(failure) = window.runtime.latest_failure.as_ref() {
                ui.horizontal_wrapped(|ui| {
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(failure.status_label).as_ref(),
                        run_status_color(failure.status_label),
                    );
                    ui.label(egui::RichText::new(
                        self.locale.text(ShellText::LastRunFailed),
                    ));
                });
                ui.colored_label(
                    notice_color(rf_ui::RunPanelNoticeLevel::Error),
                    &failure.title,
                );
                render_wrapped_label(ui, &failure.message);
                if let Some(detail) = failure.diagnostic_detail.as_ref() {
                    self.render_failure_diagnostic_detail(ui, detail);
                }
                if let Some(message) = failure.latest_log_message.as_ref() {
                    render_wrapped_small(
                        ui,
                        format!("{}: {message}", self.locale.text(ShellText::LatestLog)),
                    );
                }
                if let Some(recovery_detail) = failure.recovery_detail {
                    ui.add_space(4.0);
                    let title = failure
                        .recovery_title
                        .unwrap_or(self.locale.text(ShellText::SuggestedRecovery));
                    ui.small(egui::RichText::new(title).strong());
                    render_wrapped_small(ui, recovery_detail);
                }
                if !failure.diagnostic_actions.is_empty() {
                    self.render_diagnostic_target_actions(ui, &failure.diagnostic_actions);
                }
            } else if let Some(stale_snapshot) = window.runtime.stale_solve_snapshot.as_ref() {
                self.render_stale_solve_snapshot_notice(ui, stale_snapshot);
            } else {
                ui.small(self.locale.text(ShellText::NoVisibleSolveResults));
            }
        });

        if let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() {
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new(self.locale.text(ShellText::SolveSteps)).strong());
                if snapshot.steps.is_empty() {
                    ui.small(self.locale.text(ShellText::NoSteps));
                } else {
                    for step in &snapshot.steps {
                        self.render_solve_step_inspector(ui, step);
                    }
                }
            });

            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new(self.locale.text(ShellText::Diagnostics)).strong());
                if snapshot.diagnostics.is_empty() {
                    ui.small(self.locale.text(ShellText::NoDiagnostics));
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt(format!(
                            "scroll:{}:{}:diagnostics",
                            window.layout_state.scope.layout_key,
                            area_label(area_id)
                        ))
                        .max_height(180.0)
                        .show(ui, |ui| {
                            for (index, diagnostic) in snapshot.diagnostics.iter().enumerate() {
                                self.render_diagnostic_summary(
                                    ui,
                                    diagnostic,
                                    format!(
                                        "runtime:{}:{}:diagnostic:{index}",
                                        window.layout_state.scope.layout_key,
                                        area_label(area_id)
                                    ),
                                );
                                ui.add_space(6.0);
                            }
                        });
                }
            });
        }

        if let Some(entitlement_host) = window.runtime.entitlement_host.as_ref() {
            let entitlement = &entitlement_host.presentation.panel.view;
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(self.locale.text(ShellText::Entitlement)).strong(),
                    );
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(entitlement.auth_label).as_ref(),
                        egui::Color32::from_rgb(66, 118, 92),
                    );
                    render_status_chip(
                        ui,
                        self.locale
                            .runtime_label(entitlement.entitlement_label)
                            .as_ref(),
                        entitlement_status_color(entitlement.entitlement_label),
                    );
                });
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    render_wrapped_small(
                        ui,
                        format!(
                            "{}: {}",
                            self.locale.text(ShellText::AllowedPackages),
                            entitlement.allowed_package_count
                        ),
                    );
                    render_wrapped_small(
                        ui,
                        format!(
                            "{}: {}",
                            self.locale.text(ShellText::CachedManifests),
                            entitlement.package_manifest_count
                        ),
                    );
                    if let Some(user) = entitlement.current_user_label.as_deref() {
                        render_wrapped_small(
                            ui,
                            format!("{}: {user}", self.locale.text(ShellText::User)),
                        );
                    }
                });
                if let Some(authority_url) = entitlement.authority_url.as_deref() {
                    render_wrapped_small(
                        ui,
                        format!(
                            "{}: {authority_url}",
                            self.locale.text(ShellText::Authority)
                        ),
                    );
                }
                if let Some(last_synced_at) = entitlement.last_synced_at {
                    render_wrapped_small(
                        ui,
                        format!(
                            "{}: {}",
                            self.locale.text(ShellText::LastSynced),
                            format_system_time(last_synced_at)
                        ),
                    );
                }
                if let Some(offline_lease_expires_at) = entitlement.offline_lease_expires_at {
                    render_wrapped_small(
                        ui,
                        format!(
                            "{}: {}",
                            self.locale.text(ShellText::OfflineLeaseExpires),
                            format_system_time(offline_lease_expires_at)
                        ),
                    );
                }
                if let Some(notice) = entitlement.notice.as_ref() {
                    ui.add_space(4.0);
                    ui.colored_label(notice_color_from_entitlement(notice.level), &notice.title);
                    render_wrapped_label(ui, &notice.message);
                }
                if let Some(last_error) = entitlement.last_error.as_ref() {
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::from_rgb(180, 40, 40), last_error);
                }
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    let primary = &entitlement.primary_action;
                    let response = ui.add_enabled(
                        primary.enabled,
                        egui::Button::new(primary.label)
                            .fill(egui::Color32::from_rgb(230, 239, 252)),
                    );
                    let response = response.on_hover_text(primary.detail);
                    if response.clicked() {
                        self.dispatch_ui_command(entitlement_command_id(primary.id));
                    }
                    render_wrapped_small(ui, primary.detail);
                    for action in &entitlement.secondary_actions {
                        ui.add_space(4.0);
                        let response =
                            ui.add_enabled(action.enabled, egui::Button::new(action.label));
                        let response = response.on_hover_text(action.detail);
                        if response.clicked() {
                            self.dispatch_ui_command(entitlement_command_id(action.id));
                        }
                        render_wrapped_small(ui, action.detail);
                    }
                });
            });

            ui.add_space(8.0);
            ui.collapsing(self.locale.text(ShellText::Scheduler), |ui| {
                render_wrapped_small(
                    ui,
                    "Host lifecycle actions are routed through dedicated UI surfaces or native events.",
                );
                ui.add_space(4.0);
                for line in &entitlement_host.presentation.text.lines {
                    render_wrapped_small(ui, line);
                }
            });
        }

        ui.add_space(8.0);
        ui.collapsing(self.locale.text(ShellText::RuntimeLog), |ui| {
            egui::ScrollArea::vertical()
                .id_salt(format!(
                    "scroll:{}:{}:runtime-log",
                    window.layout_state.scope.layout_key,
                    area_label(area_id)
                ))
                .max_height(220.0)
                .show(ui, |ui| {
                    if window.runtime.log_entries.is_empty() {
                        ui.small(self.locale.text(ShellText::NoRuntimeLog));
                    } else {
                        for entry in window.runtime.log_entries.iter().rev().take(20) {
                            render_wrapped_small(
                                ui,
                                format!("[{}] {}", log_level_label(entry.level), entry.message),
                            );
                        }
                    }
                });
        });

        ui.add_space(8.0);
        ui.collapsing(self.locale.text(ShellText::GuiActivity), |ui| {
            egui::ScrollArea::vertical()
                .id_salt(format!(
                    "scroll:{}:{}:gui-activity",
                    window.layout_state.scope.layout_key,
                    area_label(area_id)
                ))
                .max_height(160.0)
                .show(ui, |ui| {
                    if window.runtime.gui_activity_lines.is_empty() {
                        ui.small(self.locale.text(ShellText::NoGuiActivity));
                    } else {
                        for line in window.runtime.gui_activity_lines.iter().rev().take(16) {
                            render_wrapped_small(ui, line);
                        }
                    }
                });
        });
    }

    fn render_diagnostic_targets(
        &mut self,
        ui: &mut egui::Ui,
        diagnostic: &radishflow_studio::StudioGuiWindowDiagnosticModel,
    ) {
        if diagnostic.target_candidates.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.small(self.locale.text(ShellText::DiagnosticTargets));
            for target in &diagnostic.target_candidates {
                let _ = self.render_small_command_action(ui, &target.action);
            }
        });
    }

    fn render_diagnostic_summary(
        &mut self,
        ui: &mut egui::Ui,
        diagnostic: &radishflow_studio::StudioGuiWindowDiagnosticModel,
        grid_id_prefix: String,
    ) {
        ui.horizontal_wrapped(|ui| {
            render_status_chip(
                ui,
                self.locale
                    .runtime_label(diagnostic.severity_label)
                    .as_ref(),
                diagnostic_color(diagnostic.severity_label),
            );
            ui.small(&diagnostic.code);
        });
        render_wrapped_label(ui, &diagnostic.message);
        self.render_diagnostic_targets(ui, diagnostic);
        if let Some(units) = diagnostic.related_units_text.as_ref() {
            render_wrapped_small(ui, format!("units: {units}"));
        }
        if let Some(streams) = diagnostic.related_streams_text.as_ref() {
            render_wrapped_small(ui, format!("streams: {streams}"));
        }
        if !diagnostic.related_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!("{grid_id_prefix}:streams"),
                "stream context",
                &diagnostic.related_stream_results,
            );
        }
    }

    pub(crate) fn render_diagnostic_target_actions(
        &mut self,
        ui: &mut egui::Ui,
        actions: &[radishflow_studio::StudioGuiWindowDiagnosticTargetActionModel],
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.small(self.locale.text(ShellText::DiagnosticTargets));
            for action in actions {
                let source_label = self.localized_diagnostic_action_label(action.source_label);
                let target_label = self.localized_diagnostic_action_label(action.target_label);
                let summary = self.localized_diagnostic_action_summary(&action.summary);
                ui.small(format!(
                    "{} | {} | {}",
                    source_label.as_ref(),
                    target_label.as_ref(),
                    summary.as_ref()
                ));
                let _ = self.render_small_command_action(ui, &action.action);
            }
        });
    }

    fn render_failure_diagnostic_detail(
        &mut self,
        ui: &mut egui::Ui,
        detail: &radishflow_studio::StudioGuiWindowFailureDiagnosticDetailModel,
    ) {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.small(egui::RichText::new("Failure diagnostic").strong());
            render_status_chip(
                ui,
                self.locale.runtime_label(detail.severity_label).as_ref(),
                diagnostic_color(detail.severity_label),
            );
            ui.small(format!("revision {}", detail.document_revision));
            ui.small(format!("count {}", detail.diagnostic_count));
        });
        if let Some(code) = detail.primary_code.as_ref() {
            render_wrapped_small(ui, format!("code: {code}"));
        }
        if !detail.related_units.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.small("units");
                for target in &detail.related_units {
                    let _ = self.render_small_command_action(ui, &target.action);
                }
            });
        }
        if !detail.related_streams.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.small("streams");
                for target in &detail.related_streams {
                    let _ = self.render_small_command_action(ui, &target.action);
                }
            });
        }
        if !detail.related_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!(
                    "failure-diagnostic:{}:{}:streams",
                    detail.document_revision,
                    detail.primary_code.as_deref().unwrap_or("none")
                ),
                "stream context",
                &detail.related_stream_results,
            );
        }
        if !detail.related_ports.is_empty() {
            ui.add_space(4.0);
            ui.small(egui::RichText::new("port context").strong());
            for target in &detail.related_ports {
                ui.horizontal_wrapped(|ui| {
                    ui.small(format!("{}:{}", target.unit_id, target.port_name));
                    let _ = self.render_small_command_action(ui, &target.unit_action);
                    if let Some(stream) = target.stream_result.as_ref() {
                        let _ = self.render_small_command_action(ui, &stream.focus_action);
                    }
                });
                if let Some(stream) = target.stream_result.as_ref() {
                    render_wrapped_small(ui, &stream.summary);
                }
            }
        }
    }

    pub(in crate::studio_gui_shell) fn render_small_command_action(
        &mut self,
        ui: &mut egui::Ui,
        action: &radishflow_studio::StudioGuiWindowCommandActionModel,
    ) -> egui::Response {
        let label = self.localized_command_action_label(action);
        let response = ui
            .small_button(label.as_ref())
            .on_hover_text(&action.hover_text);
        if response.clicked() {
            self.dispatch_ui_command(&action.command_id);
        }
        response
    }

    fn localized_command_action_label<'a>(
        &'a self,
        action: &'a radishflow_studio::StudioGuiWindowCommandActionModel,
    ) -> std::borrow::Cow<'a, str> {
        match self.locale {
            StudioShellLocale::ZhCn if action.label == "Inspect" => {
                std::borrow::Cow::Borrowed(self.locale.text(ShellText::InspectObject))
            }
            _ => std::borrow::Cow::Borrowed(action.label.as_str()),
        }
    }

    fn localized_diagnostic_action_label<'a>(&self, label: &'a str) -> std::borrow::Cow<'a, str> {
        match self.locale {
            StudioShellLocale::ZhCn if label == "Recovery mutation" => {
                std::borrow::Cow::Borrowed("修复会修改文档")
            }
            StudioShellLocale::ZhCn if label == "Recovery focus" => {
                std::borrow::Cow::Borrowed("修复会打开检查器")
            }
            StudioShellLocale::ZhCn if label == "Document" => std::borrow::Cow::Borrowed("文档"),
            StudioShellLocale::ZhCn if label == "Inspector" => std::borrow::Cow::Borrowed("检查器"),
            _ => std::borrow::Cow::Borrowed(label),
        }
    }

    fn localized_diagnostic_action_summary<'a>(
        &self,
        summary: &'a str,
    ) -> std::borrow::Cow<'a, str> {
        if matches!(self.locale, StudioShellLocale::ZhCn) {
            if let Some(rest) = summary.strip_prefix("Document mutation: ") {
                return std::borrow::Cow::Owned(format!("修改文档: {rest}"));
            }
            if let Some(rest) = summary.strip_prefix("Inspector focus: ") {
                return std::borrow::Cow::Owned(format!("打开检查器: {rest}"));
            }
        }
        std::borrow::Cow::Borrowed(summary)
    }
}

fn module_results_state_color(
    state: radishflow_studio::StudioGuiWindowModuleResultsState,
) -> egui::Color32 {
    match state {
        radishflow_studio::StudioGuiWindowModuleResultsState::Current => {
            egui::Color32::from_rgb(66, 118, 92)
        }
        radishflow_studio::StudioGuiWindowModuleResultsState::Stale
        | radishflow_studio::StudioGuiWindowModuleResultsState::NoCurrentResult
        | radishflow_studio::StudioGuiWindowModuleResultsState::NoUnitResult => {
            egui::Color32::from_rgb(160, 120, 40)
        }
        radishflow_studio::StudioGuiWindowModuleResultsState::NoUnitSelected => {
            egui::Color32::from_rgb(86, 96, 108)
        }
    }
}
