use super::super::*;
use super::commands::entitlement_command_id;

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

    pub(in crate::studio_gui_shell) fn render_runtime_results_tab(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
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
            render_wrapped_label(ui, &snapshot.summary);
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

            ui.separator();
            ui.collapsing(self.locale.text(ShellText::SolveSteps), |ui| {
                if snapshot.steps.is_empty() {
                    ui.small(self.locale.text(ShellText::NoSteps));
                } else {
                    for step in &snapshot.steps {
                        self.render_solve_step_inspector(ui, step);
                    }
                }
            });
            ui.collapsing(self.locale.text(ShellText::Diagnostics), |ui| {
                if snapshot.diagnostics.is_empty() {
                    ui.small(self.locale.text(ShellText::NoDiagnostics));
                } else {
                    for (index, diagnostic) in snapshot.diagnostics.iter().enumerate() {
                        self.render_diagnostic_summary(
                            ui,
                            diagnostic,
                            format!("runtime-results-tab:diagnostic:{index}"),
                        );
                        ui.add_space(6.0);
                    }
                }
            });
        } else if let Some(failure) = window.runtime.latest_failure.as_ref() {
            self.render_latest_failure_summary(ui, failure);
        } else {
            ui.small(self.locale.text(ShellText::NoVisibleSolveResults));
        }
    }

    pub(in crate::studio_gui_shell) fn render_runtime_run_tab(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        let run_panel = &window.runtime.run_panel;
        let run_panel_view = run_panel.view();

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
            if response.on_hover_text(primary.detail).clicked() {
                self.dispatch_run_panel_widget(run_panel.activate_primary());
            }

            for action in &run_panel_view.secondary_actions {
                let response = ui.add_enabled(
                    action.enabled,
                    egui::Button::new(self.locale.runtime_label(action.label).as_ref()),
                );
                if response.on_hover_text(action.detail).clicked() {
                    self.dispatch_run_panel_widget(run_panel.activate(action.id));
                }
            }
        });
        ui.add_space(6.0);
        if let Some(summary) = run_panel_view.latest_snapshot_summary.as_ref() {
            render_wrapped_label(ui, summary);
        } else {
            ui.small(self.locale.text(ShellText::NoSolveSnapshot));
        }
        if let Some(message) = run_panel_view.latest_log_message.as_ref() {
            render_wrapped_small(
                ui,
                format!("{}: {message}", self.locale.text(ShellText::LatestLog)),
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

        ui.separator();
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
        if let Some(notice) = self.project_open.notice.as_ref() {
            let color = match notice.level {
                ProjectOpenNoticeLevel::Info => egui::Color32::from_rgb(66, 118, 92),
                ProjectOpenNoticeLevel::Warning => egui::Color32::from_rgb(160, 120, 40),
                ProjectOpenNoticeLevel::Error => egui::Color32::from_rgb(180, 40, 40),
            };
            ui.colored_label(color, &notice.title);
            render_wrapped_small(ui, &notice.detail);
        }
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
            });
        });
    }

    pub(in crate::studio_gui_shell) fn render_runtime_package_tab(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        if let Some(platform_notice) = window.runtime.platform_notice.as_ref() {
            ui.label(egui::RichText::new(self.locale.text(ShellText::PlatformNotice)).strong());
            ui.colored_label(notice_color(platform_notice.level), &platform_notice.title);
            render_wrapped_label(ui, &platform_notice.message);
            ui.separator();
        }

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(self.locale.text(ShellText::PropertyPackage)).strong());
            render_status_chip(
                ui,
                self.locale.runtime_label("Ready").as_ref(),
                egui::Color32::from_rgb(66, 118, 92),
            );
        });
        ui.add_space(4.0);

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("binary-hydrocarbon-lite-v1").strong());
                render_status_chip(
                    ui,
                    self.locale.runtime_label("Ready").as_ref(),
                    egui::Color32::from_rgb(66, 118, 92),
                );
            });
            render_wrapped_small(
                ui,
                match self.locale {
                    StudioShellLocale::En => {
                        "Local binary hydrocarbon package used by the bundled examples."
                    }
                    StudioShellLocale::ZhCn => "本地二元烃物性包，用于内置示例和当前工作台。",
                },
            );
            ui.add_space(4.0);
            egui::Grid::new("runtime-package-summary")
                .num_columns(2)
                .striped(false)
                .show(ui, |ui| {
                    ui.small(match self.locale {
                        StudioShellLocale::En => "Components",
                        StudioShellLocale::ZhCn => "组分",
                    });
                    ui.small("Methane, Ethane");
                    ui.end_row();

                    ui.small(match self.locale {
                        StudioShellLocale::En => "Cache",
                        StudioShellLocale::ZhCn => "缓存",
                    });
                    ui.small(match self.locale {
                        StudioShellLocale::En => "Ready",
                        StudioShellLocale::ZhCn => "就绪",
                    });
                    ui.end_row();

                    ui.small(match self.locale {
                        StudioShellLocale::En => "Example source",
                        StudioShellLocale::ZhCn => "示例来源",
                    });
                    ui.small("examples/flowsheets");
                    ui.end_row();

                    ui.small(match self.locale {
                        StudioShellLocale::En => "Bundled examples",
                        StudioShellLocale::ZhCn => "内置示例",
                    });
                    ui.small(window.runtime.example_projects.len().to_string());
                    ui.end_row();
                });
        });

        if let Some(entitlement_host) = window.runtime.entitlement_host.as_ref() {
            let entitlement = &entitlement_host.presentation.panel.view;
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(egui::RichText::new(self.locale.text(ShellText::Platform)).strong());
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
                if let Some(notice) = entitlement.notice.as_ref() {
                    ui.add_space(4.0);
                    ui.colored_label(notice_color_from_entitlement(notice.level), &notice.title);
                    render_wrapped_label(ui, &notice.message);
                }
            });
        }
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
            if let Some(summary) = run_panel_view.latest_snapshot_summary.as_ref() {
                render_wrapped_label(ui, summary);
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
                render_wrapped_label(ui, &snapshot.summary);
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

    pub(in crate::studio_gui_shell) fn render_stream_result_inspector(
        &self,
        ui: &mut egui::Ui,
        stream: &radishflow_studio::StudioGuiWindowStreamResultModel,
    ) {
        ui.label(egui::RichText::new(&stream.stream_id).strong());
        render_wrapped_small(ui, &stream.label);

        ui.small(egui::RichText::new(self.locale.text(ShellText::StreamSummary)).strong());
        egui::Grid::new(format!("stream-summary:{}", stream.stream_id))
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for row in &stream.summary_rows {
                    ui.small(format!(
                        "{} · {}",
                        row.label,
                        self.locale.runtime_label(row.detail_label)
                    ));
                    ui.small(&row.value);
                    ui.end_row();
                }
            });

        if let Some(window) = stream.bubble_dew_window.as_ref() {
            ui.collapsing(self.locale.text(ShellText::BubbleDewWindow), |ui| {
                egui::Grid::new(format!("stream-bubble-dew-window:{}", stream.stream_id))
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.small(
                            egui::RichText::new(self.locale.runtime_label("Phase region").as_ref())
                                .strong(),
                        );
                        ui.small(self.locale.runtime_label(&window.phase_region).as_ref());
                        ui.end_row();

                        ui.small(self.locale.runtime_label("Bubble pressure").as_ref());
                        ui.small(&window.bubble_pressure_text);
                        ui.end_row();

                        ui.small(self.locale.runtime_label("Dew pressure").as_ref());
                        ui.small(&window.dew_pressure_text);
                        ui.end_row();

                        ui.small(self.locale.runtime_label("Bubble temperature").as_ref());
                        ui.small(&window.bubble_temperature_text);
                        ui.end_row();

                        ui.small(self.locale.runtime_label("Dew temperature").as_ref());
                        ui.small(&window.dew_temperature_text);
                        ui.end_row();
                    });
            });
        }

        ui.collapsing(self.locale.text(ShellText::OverallComposition), |ui| {
            if stream.composition_rows.is_empty() {
                ui.small(self.locale.text(ShellText::NoComposition));
                return;
            }
            egui::Grid::new(format!("stream-composition:{}", stream.stream_id))
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Component)).strong());
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::MoleFraction)).strong(),
                    );
                    ui.end_row();
                    for row in &stream.composition_rows {
                        ui.small(&row.component_id);
                        ui.small(&row.fraction_text);
                        ui.end_row();
                    }
                });
        });

        ui.collapsing(self.locale.text(ShellText::PhaseResults), |ui| {
            if stream.phase_rows.is_empty() {
                ui.small(self.locale.text(ShellText::NoPhases));
                return;
            }
            egui::Grid::new(format!("stream-phases:{}", stream.stream_id))
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Phase)).strong());
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Fraction)).strong());
                    ui.small(egui::RichText::new(self.locale.runtime_label("Molar flow")).strong());
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::OverallComposition))
                            .strong(),
                    );
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Enthalpy)).strong());
                    ui.end_row();
                    for row in &stream.phase_rows {
                        ui.small(&row.label);
                        ui.small(&row.phase_fraction_text);
                        ui.small(&row.molar_flow_text);
                        render_wrapped_small(ui, &row.composition_text);
                        ui.small(row.molar_enthalpy_text.as_deref().unwrap_or("-"));
                        ui.end_row();
                    }
                });
        });

        render_wrapped_small(ui, &stream.composition_text);
        render_wrapped_small(ui, &stream.phase_text);
        ui.add_space(8.0);
    }

    fn render_unit_execution_result_inspector(
        &mut self,
        ui: &mut egui::Ui,
        unit: &radishflow_studio::StudioGuiWindowUnitExecutionResultModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(&unit.unit_id).strong());
            render_status_chip(
                ui,
                self.locale.runtime_label(unit.status_label).as_ref(),
                run_status_color(unit.status_label),
            );
            ui.small(format!("#{}", unit.step_index));
        });
        render_wrapped_label(ui, &unit.summary);
        if !unit.consumed_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!("unit-consumed-streams:{}:{}", unit.unit_id, unit.step_index),
                self.locale.text(ShellText::InspectorConsumedStreams),
                &unit.consumed_stream_results,
            );
        }
        if !unit.produced_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!("unit-produced-streams:{}:{}", unit.unit_id, unit.step_index),
                self.locale.text(ShellText::InspectorProducedStreams),
                &unit.produced_stream_results,
            );
        }
        ui.add_space(8.0);
    }

    fn render_solve_step_inspector(
        &mut self,
        ui: &mut egui::Ui,
        step: &radishflow_studio::StudioGuiWindowSolveStepModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.small(format!("#{}", step.index));
            render_status_chip(
                ui,
                self.locale
                    .runtime_label(step.execution_status_label)
                    .as_ref(),
                run_status_color(step.execution_status_label),
            );
            if !step.consumed_stream_actions.is_empty() {
                for action in &step.consumed_stream_actions {
                    let _ = self.render_small_command_action(ui, action);
                }
                ui.small("->");
            }
            let _ = self.render_small_command_action(ui, &step.unit_action);
            if !step.produced_stream_actions.is_empty() {
                ui.small("->");
                for action in &step.produced_stream_actions {
                    let _ = self.render_small_command_action(ui, action);
                }
            }
        });
        render_wrapped_label(ui, &step.summary);
        if !step.consumed_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!(
                    "solve-step-consumed-streams:{}:{}",
                    step.unit_id, step.index
                ),
                self.locale.text(ShellText::InspectorConsumedStreams),
                &step.consumed_stream_results,
            );
        }
        if !step.produced_stream_results.is_empty() {
            self.render_stream_result_reference_grid(
                ui,
                format!(
                    "solve-step-produced-streams:{}:{}",
                    step.unit_id, step.index
                ),
                self.locale.text(ShellText::InspectorProducedStreams),
                &step.produced_stream_results,
            );
        }
        ui.add_space(4.0);
    }

    fn render_stream_result_reference_grid(
        &mut self,
        ui: &mut egui::Ui,
        grid_id: String,
        title: &str,
        streams: &[radishflow_studio::StudioGuiWindowStreamResultReferenceModel],
    ) {
        ui.add_space(4.0);
        ui.small(egui::RichText::new(title).strong());
        egui::Grid::new(grid_id)
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for stream in streams {
                    let _ = self.render_small_command_action(ui, &stream.focus_action);
                    render_wrapped_small(ui, &stream.summary);
                    ui.end_row();
                }
            });
    }

    fn render_active_inspector_detail(
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
            egui::Grid::new(format!("inspector-fields:{}", detail.target.command_id))
                .num_columns(5)
                .spacing([8.0, 3.0])
                .show(ui, |ui| {
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::InspectorFieldName))
                            .strong(),
                    );
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::InspectorFieldKind))
                            .strong(),
                    );
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::InspectorFieldValue))
                            .strong(),
                    );
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::InspectorFieldStatus))
                            .strong(),
                    );
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::InspectorFieldAction))
                            .strong(),
                    );
                    ui.end_row();
                    for field in &detail.property_fields {
                        render_wrapped_small(ui, &field.label);
                        render_wrapped_small(
                            ui,
                            self.locale.runtime_label(field.value_kind_label).as_ref(),
                        );
                        let mut draft_value = field.current_value.clone();
                        let response = ui
                            .add_sized([150.0, 22.0], egui::TextEdit::singleline(&mut draft_value));
                        if response.changed() {
                            self.dispatch_inspector_field_draft_update(
                                field.draft_update_command_id.clone(),
                                draft_value,
                            );
                        }
                        let submit_on_enter = response.lost_focus()
                            && ui.input(|input| {
                                input.key_pressed(egui::Key::Enter)
                                    && input.modifiers == egui::Modifiers::NONE
                            });
                        render_status_chip(
                            ui,
                            self.locale.runtime_label(field.status_label).as_ref(),
                            inspector_field_status_color(field.status_label),
                        );
                        if let Some(command_id) = field.commit_command_id.as_ref() {
                            ui.horizontal_wrapped(|ui| {
                                if submit_on_enter
                                    || ui
                                        .small_button(
                                            self.locale.text(ShellText::InspectorFieldApply),
                                        )
                                        .clicked()
                                {
                                    self.dispatch_inspector_field_draft_commit(command_id.clone());
                                }
                                if let Some(discard_command_id) = field.discard_command_id.as_ref()
                                {
                                    if ui
                                        .small_button(
                                            self.locale.text(ShellText::InspectorFieldDiscard),
                                        )
                                        .clicked()
                                    {
                                        self.dispatch_inspector_field_draft_discard(
                                            discard_command_id.clone(),
                                        );
                                    }
                                }
                                if let Some(remove_command_id) = field.remove_command_id.as_ref() {
                                    if ui
                                        .small_button(
                                            self.locale.text(
                                                ShellText::InspectorRemoveCompositionComponent,
                                            ),
                                        )
                                        .clicked()
                                    {
                                        self.dispatch_inspector_composition_component_remove(
                                            remove_command_id.clone(),
                                        );
                                    }
                                }
                            });
                        } else if field.discard_command_id.is_some()
                            || field.remove_command_id.is_some()
                        {
                            ui.horizontal_wrapped(|ui| {
                                if let Some(command_id) = field.discard_command_id.as_ref() {
                                    if ui
                                        .small_button(
                                            self.locale.text(ShellText::InspectorFieldDiscard),
                                        )
                                        .clicked()
                                    {
                                        self.dispatch_inspector_field_draft_discard(
                                            command_id.clone(),
                                        );
                                    }
                                }
                                if let Some(remove_command_id) = field.remove_command_id.as_ref() {
                                    if ui
                                        .small_button(
                                            self.locale.text(
                                                ShellText::InspectorRemoveCompositionComponent,
                                            ),
                                        )
                                        .clicked()
                                    {
                                        self.dispatch_inspector_composition_component_remove(
                                            remove_command_id.clone(),
                                        );
                                    }
                                }
                            });
                        } else {
                            ui.small("-");
                        }
                        ui.end_row();
                    }
                });
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
                            (_, Some(action)) => {
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

    pub(in crate::studio_gui_shell) fn render_result_inspector(
        &mut self,
        ui: &mut egui::Ui,
        inspector: &radishflow_studio::StudioGuiWindowResultInspectorModel,
    ) {
        ui.label(egui::RichText::new(self.locale.text(ShellText::ResultInspector)).strong());
        ui.small(self.locale.text(ShellText::SelectStream));
        ui.horizontal_wrapped(|ui| {
            for option in &inspector.stream_options {
                let label = if option.label.is_empty() {
                    option.stream_id.as_str()
                } else {
                    option.label.as_str()
                };
                let response = ui
                    .add(egui::Button::new(label).selected(option.is_selected))
                    .on_hover_text(&option.summary);
                if response.clicked() {
                    self.result_inspector
                        .select_stream(&inspector.snapshot_id, option.stream_id.clone());
                }
                let _ = self.render_small_command_action(ui, &option.focus_action);
            }
        });
        if inspector.has_stale_selection {
            render_wrapped_small(ui, self.locale.text(ShellText::StaleStreamSelection));
        }
        ui.separator();

        if let Some(stream) = inspector.selected_stream.as_ref() {
            ui.push_id(
                format!(
                    "result-inspector:selected-stream:{}:{}",
                    inspector.snapshot_id, stream.stream_id
                ),
                |ui| self.render_stream_result_inspector(ui, stream),
            );
        } else {
            ui.small(self.locale.text(ShellText::NoStreamResults));
            return;
        }

        if !inspector.unit_options.is_empty() {
            ui.collapsing(self.locale.text(ShellText::ResultUnitView), |ui| {
                ui.small(self.locale.text(ShellText::SelectUnit));
                ui.horizontal_wrapped(|ui| {
                    for option in &inspector.unit_options {
                        let response = ui
                            .add(egui::Button::new(&option.unit_id).selected(option.is_selected))
                            .on_hover_text(&option.summary);
                        if response.clicked() {
                            self.result_inspector
                                .select_unit(&inspector.snapshot_id, option.unit_id.clone());
                        }
                        let _ = self.render_small_command_action(ui, &option.focus_action);
                    }
                });
                if inspector.has_stale_unit_selection {
                    render_wrapped_small(ui, self.locale.text(ShellText::StaleUnitSelection));
                }
                if let Some(unit) = inspector.selected_unit.as_ref() {
                    ui.add_space(4.0);
                    self.render_unit_execution_result_inspector(ui, unit);
                } else {
                    ui.small(self.locale.text(ShellText::NoUnitResults));
                }

                if !inspector.unit_diagnostic_actions.is_empty() {
                    ui.collapsing(self.locale.text(ShellText::DiagnosticTargets), |ui| {
                        self.render_diagnostic_target_actions(
                            ui,
                            &inspector.unit_diagnostic_actions,
                        );
                    });
                }

                ui.collapsing(self.locale.text(ShellText::RelatedSolveSteps), |ui| {
                    if inspector.unit_related_steps.is_empty() {
                        ui.small(self.locale.text(ShellText::NoRelatedSteps));
                        return;
                    }
                    for step in &inspector.unit_related_steps {
                        self.render_solve_step_inspector(ui, step);
                    }
                });

                ui.collapsing(self.locale.text(ShellText::RelatedDiagnostics), |ui| {
                    if inspector.unit_related_diagnostics.is_empty() {
                        ui.small(self.locale.text(ShellText::NoRelatedDiagnostics));
                        return;
                    }
                    for (index, diagnostic) in inspector.unit_related_diagnostics.iter().enumerate()
                    {
                        self.render_diagnostic_summary(
                            ui,
                            diagnostic,
                            format!(
                                "result-unit:{}:{}:diagnostic:{index}",
                                inspector.snapshot_id,
                                inspector.selected_unit_id.as_deref().unwrap_or("none")
                            ),
                        );
                        ui.add_space(4.0);
                    }
                });
            });
        }

        if !inspector.comparison_options.is_empty() {
            ui.collapsing(self.locale.text(ShellText::StreamComparison), |ui| {
                ui.small(self.locale.text(ShellText::CompareWith));
                ui.horizontal_wrapped(|ui| {
                    for option in &inspector.comparison_options {
                        let label = if option.label.is_empty() {
                            option.stream_id.as_str()
                        } else {
                            option.label.as_str()
                        };
                        let response = ui
                            .add(egui::Button::new(label).selected(option.is_selected))
                            .on_hover_text(&option.summary);
                        if response.clicked() {
                            self.result_inspector.select_comparison_stream(
                                &inspector.snapshot_id,
                                option.stream_id.clone(),
                            );
                        }
                        let _ = self.render_small_command_action(ui, &option.focus_action);
                    }
                });
                if inspector.has_stale_comparison {
                    render_wrapped_small(ui, self.locale.text(ShellText::StaleStreamSelection));
                }
                if let Some(comparison) = inspector.comparison.as_ref() {
                    self.render_result_inspector_comparison(ui, comparison);
                } else {
                    ui.small(self.locale.text(ShellText::NoComparison));
                }
            });
        }

        if !inspector.diagnostic_actions.is_empty() {
            ui.collapsing(self.locale.text(ShellText::DiagnosticTargets), |ui| {
                self.render_diagnostic_target_actions(ui, &inspector.diagnostic_actions);
            });
        }

        ui.collapsing(self.locale.text(ShellText::RelatedSolveSteps), |ui| {
            if inspector.related_steps.is_empty() {
                ui.small(self.locale.text(ShellText::NoRelatedSteps));
                return;
            }
            for step in &inspector.related_steps {
                self.render_solve_step_inspector(ui, step);
            }
        });

        ui.collapsing(self.locale.text(ShellText::RelatedDiagnostics), |ui| {
            if inspector.related_diagnostics.is_empty() {
                ui.small(self.locale.text(ShellText::NoRelatedDiagnostics));
                return;
            }
            for (index, diagnostic) in inspector.related_diagnostics.iter().enumerate() {
                self.render_diagnostic_summary(
                    ui,
                    diagnostic,
                    format!(
                        "result-stream:{}:{}:diagnostic:{index}",
                        inspector.snapshot_id,
                        inspector.selected_stream_id.as_deref().unwrap_or("none")
                    ),
                );
                ui.add_space(4.0);
            }
        });
    }

    pub(in crate::studio_gui_shell) fn render_result_inspector_comparison(
        &mut self,
        ui: &mut egui::Ui,
        comparison: &radishflow_studio::StudioGuiWindowResultInspectorComparisonModel,
    ) {
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.small(format!(
                "{}: {}",
                self.locale.text(ShellText::BaseStream),
                comparison.base_stream_id
            ));
            let _ = self.render_small_command_action(ui, &comparison.base_stream_focus_action);
            ui.small(format!(
                "{}: {}",
                self.locale.text(ShellText::ComparedStream),
                comparison.compared_stream_id
            ));
            let _ = self.render_small_command_action(ui, &comparison.compared_stream_focus_action);
        });
        egui::Grid::new(format!(
            "result-comparison-summary:{}:{}",
            comparison.base_stream_id, comparison.compared_stream_id
        ))
        .num_columns(4)
        .striped(true)
        .show(ui, |ui| {
            ui.small(self.locale.text(ShellText::StreamSummary));
            ui.small(self.locale.text(ShellText::BaseStream));
            ui.small(self.locale.text(ShellText::ComparedStream));
            ui.small(self.locale.text(ShellText::Delta));
            ui.end_row();
            for row in &comparison.summary_rows {
                ui.small(format!(
                    "{} · {}",
                    row.label,
                    self.locale.runtime_label(row.detail_label)
                ));
                ui.small(&row.base_value);
                ui.small(&row.compared_value);
                ui.small(&row.delta_text);
                ui.end_row();
            }
        });

        if !comparison.composition_rows.is_empty() {
            ui.add_space(4.0);
            ui.small(egui::RichText::new(self.locale.text(ShellText::OverallComposition)).strong());
            egui::Grid::new(format!(
                "result-comparison-composition:{}:{}",
                comparison.base_stream_id, comparison.compared_stream_id
            ))
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                ui.small(self.locale.text(ShellText::Component));
                ui.small(self.locale.text(ShellText::BaseStream));
                ui.small(self.locale.text(ShellText::ComparedStream));
                ui.small(self.locale.text(ShellText::Delta));
                ui.end_row();
                for row in &comparison.composition_rows {
                    ui.small(&row.component_id);
                    ui.small(&row.base_fraction_text);
                    ui.small(&row.compared_fraction_text);
                    ui.small(&row.delta_text);
                    ui.end_row();
                }
            });
        }

        if !comparison.phase_rows.is_empty() {
            ui.add_space(4.0);
            ui.small(egui::RichText::new(self.locale.text(ShellText::PhaseResults)).strong());
            egui::Grid::new(format!(
                "result-comparison-phase-flows:{}:{}",
                comparison.base_stream_id, comparison.compared_stream_id
            ))
            .num_columns(7)
            .striped(true)
            .show(ui, |ui| {
                ui.small(self.locale.text(ShellText::Phase));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::BaseStream),
                    self.locale.text(ShellText::Fraction)
                ));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::ComparedStream),
                    self.locale.text(ShellText::Fraction)
                ));
                ui.small(self.locale.text(ShellText::Delta));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::BaseStream),
                    self.locale.runtime_label("Molar flow")
                ));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::ComparedStream),
                    self.locale.runtime_label("Molar flow")
                ));
                ui.small(self.locale.text(ShellText::Delta));
                ui.end_row();
                for row in &comparison.phase_rows {
                    ui.small(&row.phase_label);
                    ui.small(&row.base_fraction_text);
                    ui.small(&row.compared_fraction_text);
                    ui.small(&row.fraction_delta_text);
                    ui.small(&row.base_molar_flow_text);
                    ui.small(&row.compared_molar_flow_text);
                    ui.small(&row.molar_flow_delta_text);
                    ui.end_row();
                }
            });

            ui.add_space(4.0);
            egui::Grid::new(format!(
                "result-comparison-phase-enthalpy:{}:{}",
                comparison.base_stream_id, comparison.compared_stream_id
            ))
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                ui.small(self.locale.text(ShellText::Phase));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::BaseStream),
                    self.locale.text(ShellText::Enthalpy)
                ));
                ui.small(format!(
                    "{} {}",
                    self.locale.text(ShellText::ComparedStream),
                    self.locale.text(ShellText::Enthalpy)
                ));
                ui.small(self.locale.text(ShellText::Delta));
                ui.end_row();
                for row in &comparison.phase_rows {
                    ui.small(&row.phase_label);
                    ui.small(&row.base_molar_enthalpy_text);
                    ui.small(&row.compared_molar_enthalpy_text);
                    ui.small(&row.molar_enthalpy_delta_text);
                    ui.end_row();
                }
            });
        }
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
                ui.small(format!(
                    "{} | {} | {}",
                    action.source_label, action.target_label, action.summary
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
        let response = ui
            .small_button(&action.label)
            .on_hover_text(&action.hover_text);
        if response.clicked() {
            self.dispatch_ui_command(&action.command_id);
        }
        response
    }
}
