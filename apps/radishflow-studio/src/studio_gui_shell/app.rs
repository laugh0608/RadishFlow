use crate::studio_gui_preferences_store::save_recent_project_paths;

use super::*;

impl ReadyAppState {
    pub(super) fn open_example_project(&mut self, project_path: PathBuf) {
        self.request_open_project(project_path, "example project");
    }

    pub(super) fn open_recent_project(&mut self, project_path: PathBuf) {
        self.request_open_project(project_path, "recent project");
    }

    pub(super) fn open_project_from_input(&mut self) {
        let Some(project_path) = self.project_open.current_path() else {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Error,
                title: "Project path is empty".to_string(),
                detail: "Enter a .rfproj.json path before opening a project.".to_string(),
            });
            return;
        };
        self.request_open_project(project_path, "project");
    }

    pub(super) fn open_project_from_picker(&mut self) {
        let Some(project_path) = self.project_file_picker.pick_project_file() else {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Info,
                title: "Project picker canceled".to_string(),
                detail: "Current workspace remains open.".to_string(),
            });
            return;
        };

        self.project_open.path_input = project_path.display().to_string();
        self.request_open_project(project_path, "project picker");
    }

    pub(super) fn create_blank_project(&mut self) {
        self.request_blank_project(None);
    }

    pub(super) fn start_mixer_flash_authoring_case(&mut self) {
        self.request_blank_project(Some(AuthoringCaseKind::MixerFlash));
    }

    pub(super) fn start_heater_flash_authoring_case(&mut self) {
        self.request_blank_project(Some(AuthoringCaseKind::HeaterFlash));
    }

    fn request_blank_project(&mut self, authoring_case: Option<AuthoringCaseKind>) {
        if self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .has_unsaved_changes
        {
            self.project_open.pending_confirmation = None;
            self.project_open.pending_blank_project_confirmation = true;
            self.project_open.pending_authoring_blank_project = authoring_case;
            self.project_open.pending_save_as_overwrite = None;
            self.project_open.pending_close_window_confirmation = None;
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: unsaved_changes_notice_title(self.locale).to_string(),
                detail: create_blank_project_discard_notice_detail(self.locale),
            });
            return;
        }

        self.create_blank_project_without_confirmation(authoring_case);
    }

    pub(super) fn confirm_pending_blank_project(&mut self) {
        if !self.project_open.pending_blank_project_confirmation {
            return;
        }
        let authoring_case = self.project_open.pending_authoring_blank_project;
        self.project_open.pending_blank_project_confirmation = false;
        self.project_open.pending_authoring_blank_project = None;
        self.create_blank_project_without_confirmation(authoring_case);
    }

    pub(super) fn cancel_pending_blank_project(&mut self) {
        self.project_open.pending_blank_project_confirmation = false;
        self.project_open.pending_authoring_blank_project = None;
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Info,
            title: blank_project_canceled_notice_title(self.locale).to_string(),
            detail: current_workspace_remains_open_notice_detail(self.locale).to_string(),
        });
    }

    fn create_blank_project_without_confirmation(
        &mut self,
        authoring_case: Option<AuthoringCaseKind>,
    ) {
        let config = studio_shell_blank_runtime_config();

        match StudioGuiPlatformHost::new(&config) {
            Ok(platform_host) => {
                self.platform_host = platform_host;
                self.platform_timer_executor = EguiPlatformTimerExecutor::default();
                self.command_palette.close();
                self.last_area_focus = None;
                self.drag_session = None;
                self.active_drop_preview = None;
                self.drop_preview_overlay_anchor = None;
                self.last_viewport_focused = None;
                self.canvas_viewport_navigation = CanvasViewportNavigationState::default();
                self.canvas_initial_viewport_fit.reset();
                self.canvas_viewport_fit_to_content_requested = false;
                self.canvas_viewport_drag = None;
                self.canvas_unit_drag = None;
                self.canvas_command_result = None;
                self.result_inspector.reset();
                self.project_open.path_input.clear();
                self.project_open.pending_confirmation = None;
                self.project_open.pending_blank_project_confirmation = false;
                self.project_open.pending_authoring_blank_project = None;
                self.project_open.pending_save_as_overwrite = None;
                self.project_open.pending_close_window_confirmation = None;
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Info,
                    title: blank_project_created_notice_title(self.locale, authoring_case)
                        .to_string(),
                    detail: blank_project_created_notice_detail(self.locale, authoring_case)
                        .to_string(),
                });
                self.screen = StudioShellScreen::Workbench;
                self.active_authoring_case = authoring_case;
                if authoring_case.is_some() {
                    self.left_sidebar_tab = StudioShellLeftSidebarTab::Palette;
                    self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                    self.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
                }
                self.platform_host.record_activity_line(
                    blank_project_created_activity_line(authoring_case).to_string(),
                );
                self.dispatch_event(StudioGuiEvent::OpenWindowRequested);
                if let Err(error) = self.apply_default_hidden_commands_panel_for_current_window() {
                    self.platform_host.record_activity_line(format!(
                        "apply default commands panel visibility failed [{}]: {}",
                        error.code().as_str(),
                        error.message()
                    ));
                }
            }
            Err(error) => {
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: "Blank project creation failed".to_string(),
                    detail: format!(
                        "[{}] {}. Current workspace remains open.",
                        error.code().as_str(),
                        error.message()
                    ),
                });
                self.platform_host.record_activity_line(format!(
                    "create blank project failed [{}]: {}",
                    error.code().as_str(),
                    error.message()
                ));
            }
        }
    }

    pub(super) fn save_project(&mut self) {
        if self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .project_path
            .is_none()
        {
            self.save_project_as_from_picker();
            return;
        }

        match self.dispatch_event_result(StudioGuiEvent::UiCommandRequested {
            command_id: radishflow_studio::FILE_SAVE_COMMAND_ID.to_string(),
        }) {
            Ok(dispatch) => {
                let document = &dispatch.dispatch.window.runtime.workspace_document;
                let Some(path) = document.project_path.as_ref() else {
                    self.project_open.notice = Some(ProjectOpenNotice {
                        level: ProjectOpenNoticeLevel::Warning,
                        title: "Project save skipped".to_string(),
                        detail: "Current document has no project path; use Save As.".to_string(),
                    });
                    return;
                };
                let level = if document.has_unsaved_changes {
                    ProjectOpenNoticeLevel::Warning
                } else {
                    ProjectOpenNoticeLevel::Info
                };
                self.project_open.notice = Some(ProjectOpenNotice {
                    level,
                    title: if document.has_unsaved_changes {
                        "Project save incomplete".to_string()
                    } else {
                        "Project saved".to_string()
                    },
                    detail: format!("Saved revision {} to {path}", document.revision),
                });
                self.project_open.pending_save_as_overwrite = None;
            }
            Err(error) => {
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: "Project save failed".to_string(),
                    detail: format!(
                        "[{}] {}. Current workspace remains open.",
                        error.code().as_str(),
                        error.message()
                    ),
                });
                self.platform_host.record_activity_line(format!(
                    "save project failed [{}]: {}",
                    error.code().as_str(),
                    error.message()
                ));
            }
        }
    }

    pub(super) fn save_project_as_from_picker(&mut self) {
        let Some(project_path) = self.project_file_picker.pick_save_project_file() else {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Info,
                title: "Save As canceled".to_string(),
                detail: "Current workspace remains open.".to_string(),
            });
            return;
        };

        self.request_save_project_as(project_path, true);
    }

    pub(super) fn request_save_project_as(
        &mut self,
        project_path: PathBuf,
        require_overwrite_confirmation: bool,
    ) {
        let Some(window_id) = self.current_window_id() else {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Error,
                title: "Save As unavailable".to_string(),
                detail: "Open a Studio window before saving the project.".to_string(),
            });
            return;
        };

        if require_overwrite_confirmation
            && self.save_as_requires_overwrite_confirmation(&project_path)
        {
            self.project_open.pending_save_as_overwrite = Some(project_path.clone());
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: "Confirm overwrite".to_string(),
                detail: format!(
                    "Save As target already exists and will be replaced: {}",
                    project_path.display()
                ),
            });
            return;
        }

        let trigger = StudioRuntimeTrigger::DocumentLifecycle(
            radishflow_studio::StudioDocumentLifecycleCommand::SaveAs {
                path: project_path.clone(),
            },
        );
        match self
            .dispatch_event_result(StudioGuiEvent::WindowTriggerRequested { window_id, trigger })
        {
            Ok(dispatch) => {
                let document = &dispatch.dispatch.window.runtime.workspace_document;
                self.project_open.path_input = project_path.display().to_string();
                self.project_open.pending_save_as_overwrite = None;
                let recent_projects_notice =
                    self.record_and_persist_recent_project(project_path.clone());
                self.project_open.notice =
                    Some(recent_projects_notice.unwrap_or(ProjectOpenNotice {
                        level: ProjectOpenNoticeLevel::Info,
                        title: "Project saved as".to_string(),
                        detail: format!(
                            "Saved revision {} to {}",
                            document.revision,
                            project_path.display()
                        ),
                    }));
            }
            Err(error) => {
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: "Save As failed".to_string(),
                    detail: format!(
                        "[{}] {} ({}). Current workspace remains open; choose another target or retry.",
                        error.code().as_str(),
                        error.message(),
                        project_path.display()
                    ),
                });
                self.platform_host.record_activity_line(format!(
                    "save as failed [{}]: {} ({})",
                    error.code().as_str(),
                    error.message(),
                    project_path.display()
                ));
            }
        }
    }

    pub(super) fn confirm_pending_save_as_overwrite(&mut self) {
        let Some(project_path) = self.project_open.pending_save_as_overwrite.clone() else {
            return;
        };
        self.request_save_project_as(project_path, false);
    }

    pub(super) fn cancel_pending_save_as_overwrite(&mut self) {
        self.project_open.pending_save_as_overwrite = None;
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Info,
            title: "Save As canceled".to_string(),
            detail: "Existing project file was not overwritten.".to_string(),
        });
    }

    pub(super) fn copy_solve_snapshot_to_clipboard(
        &mut self,
        ctx: &egui::Context,
        snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    ) {
        ctx.copy_text(snapshot.light_text_export());
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
        let Some(path) = self.project_file_picker.pick_result_export_file() else {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Info,
                title: solve_snapshot_export_canceled_notice_title(self.locale).to_string(),
                detail: current_workspace_remains_open_notice_detail(self.locale).to_string(),
            });
            return;
        };

        self.export_solve_snapshot_to_path(snapshot, path);
    }

    pub(super) fn export_solve_snapshot_to_path(
        &mut self,
        snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
        path: PathBuf,
    ) {
        let snapshot_text = snapshot.light_text_export();
        match std::fs::write(&path, snapshot_text) {
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

    fn save_as_requires_overwrite_confirmation(&self, project_path: &std::path::Path) -> bool {
        if !project_path.exists() {
            return false;
        }

        self.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .project_path
            .as_deref()
            .map(std::path::Path::new)
            .map(|current_path| !paths_match(current_path, project_path))
            .unwrap_or(true)
    }

    pub(super) fn request_open_project(&mut self, project_path: PathBuf, source_label: &str) {
        if self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .has_unsaved_changes
        {
            self.project_open.pending_confirmation = Some(ProjectOpenRequest {
                project_path: project_path.clone(),
                source_label: source_label.to_string(),
            });
            self.project_open.pending_blank_project_confirmation = false;
            self.project_open.pending_authoring_blank_project = None;
            self.project_open.pending_save_as_overwrite = None;
            self.project_open.pending_close_window_confirmation = None;
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: unsaved_changes_notice_title(self.locale).to_string(),
                detail: open_project_discard_notice_detail(
                    self.locale,
                    source_label,
                    &project_path,
                ),
            });
            return;
        }

        self.open_project(project_path, source_label);
    }

    pub(super) fn confirm_pending_project_open(&mut self) {
        let Some(request) = self.project_open.pending_confirmation.take() else {
            return;
        };
        self.open_project(request.project_path, &request.source_label);
    }

    pub(super) fn cancel_pending_project_open(&mut self) {
        self.project_open.pending_confirmation = None;
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Info,
            title: "Project open canceled".to_string(),
            detail: "Current workspace remains open.".to_string(),
        });
    }

    pub(super) fn open_project(&mut self, project_path: PathBuf, source_label: &str) {
        let config = studio_shell_runtime_config(Some(project_path.clone()));

        match StudioGuiPlatformHost::new(&config) {
            Ok(platform_host) => {
                self.platform_host = platform_host;
                self.platform_timer_executor = EguiPlatformTimerExecutor::default();
                self.command_palette.close();
                self.last_area_focus = None;
                self.drag_session = None;
                self.active_drop_preview = None;
                self.drop_preview_overlay_anchor = None;
                self.last_viewport_focused = None;
                self.canvas_viewport_navigation = CanvasViewportNavigationState::default();
                self.canvas_initial_viewport_fit = canvas_initial_viewport_fit_from_config(&config);
                self.canvas_viewport_fit_to_content_requested = false;
                self.canvas_viewport_drag = None;
                self.canvas_unit_drag = None;
                self.canvas_command_result = None;
                self.result_inspector.reset();
                self.project_open.path_input = project_path.display().to_string();
                let recent_projects_notice =
                    self.record_and_persist_recent_project(project_path.clone());
                self.project_open.pending_confirmation = None;
                self.project_open.pending_blank_project_confirmation = false;
                self.project_open.pending_authoring_blank_project = None;
                self.project_open.pending_save_as_overwrite = None;
                self.project_open.pending_close_window_confirmation = None;
                self.project_open.notice =
                    Some(recent_projects_notice.unwrap_or(ProjectOpenNotice {
                        level: ProjectOpenNoticeLevel::Info,
                        title: project_opened_notice_title(self.locale).to_string(),
                        detail: project_opened_notice_detail(
                            self.locale,
                            source_label,
                            &project_path,
                        ),
                    }));
                self.screen = StudioShellScreen::Workbench;
                self.active_authoring_case = None;
                self.platform_host.record_activity_line(format!(
                    "opened {source_label}: {}",
                    project_path.display()
                ));
                self.dispatch_event(StudioGuiEvent::OpenWindowRequested);
                if let Err(error) = self.apply_default_hidden_commands_panel_for_current_window() {
                    self.platform_host.record_activity_line(format!(
                        "apply default commands panel visibility failed [{}]: {}",
                        error.code().as_str(),
                        error.message()
                    ));
                }
            }
            Err(error) => {
                self.project_open.notice = Some(ProjectOpenNotice {
                    level: ProjectOpenNoticeLevel::Error,
                    title: "Project open failed".to_string(),
                    detail: format!(
                        "[{}] {} ({})",
                        error.code().as_str(),
                        error.message(),
                        project_path.display()
                    ),
                });
                self.platform_host.record_activity_line(format!(
                    "open {source_label} failed [{}]: {} ({})",
                    error.code().as_str(),
                    error.message(),
                    project_path.display()
                ));
            }
        }
    }

    pub(super) fn record_and_persist_recent_project(
        &mut self,
        project_path: PathBuf,
    ) -> Option<ProjectOpenNotice> {
        self.home_selected_recent_project = Some(project_path.clone());
        self.project_open.record_recent_project(project_path);
        if let Err(error) =
            save_recent_project_paths(&self.preferences_path, &self.project_open.recent_projects)
        {
            self.platform_host.record_activity_line(format!(
                "save recent projects failed [{}]: {} ({})",
                error.code().as_str(),
                error.message(),
                self.preferences_path.display()
            ));
            return Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: "Recent projects not saved".to_string(),
                detail: format!(
                    "[{}] {} ({})",
                    error.code().as_str(),
                    error.message(),
                    self.preferences_path.display()
                ),
            });
        }
        None
    }

    pub(super) fn update(&mut self, ctx: &egui::Context) {
        let close_frame_snapshot = ctx
            .input(|input| input.viewport().close_requested())
            .then(|| self.platform_host.snapshot());
        if self.sync_viewport_close(ctx) {
            if let Some(snapshot) = close_frame_snapshot.as_ref() {
                self.render_viewport_close_frame(ctx, snapshot);
            }
            return;
        }
        self.sync_viewport_lifecycle(ctx);
        let toggle_shortcut_consumed = self.handle_command_palette_toggle_shortcut(ctx);
        self.drain_due_timers(ctx);
        self.drop_preview_overlay_anchor = None;

        let snapshot = self.platform_host.snapshot();
        let window = snapshot.window_model();
        let palette_keyboard_consumed = self.handle_command_palette_keyboard(ctx, &window.commands);
        if !toggle_shortcut_consumed && !palette_keyboard_consumed {
            self.dispatch_shortcuts(ctx);
        }
        let mut hovered_drop_target = false;
        if self.screen == StudioShellScreen::Home {
            self.render_home_dashboard(ctx, &window);
            self.render_command_palette(ctx, &window.commands);
            self.render_pending_close_window_dialog(ctx);
            return;
        }
        self.render_top_bar(
            ctx,
            &snapshot.app_host_state.windows,
            &window,
            &mut hovered_drop_target,
        );
        self.render_left_sidebar(ctx, &window, &mut hovered_drop_target);
        self.render_right_sidebar(ctx, &window, &mut hovered_drop_target);
        self.render_bottom_status_bar(ctx, &window);
        self.render_bottom_drawer(ctx, &window);
        self.render_center_stage(ctx, &window, &mut hovered_drop_target);
        self.render_command_palette(ctx, &window.commands);
        self.render_floating_drop_preview_overlay(ctx, &window);
        self.render_pending_close_window_dialog(ctx);
        self.finish_drop_preview_cycle(
            ctx,
            window.layout_state.scope.window_id,
            hovered_drop_target,
        );
    }

    fn render_viewport_close_frame(
        &mut self,
        ctx: &egui::Context,
        snapshot: &radishflow_studio::StudioGuiSnapshot,
    ) {
        let window = snapshot.window_model();
        let mut hovered_drop_target = false;
        if self.screen == StudioShellScreen::Home {
            self.render_home_dashboard(ctx, &window);
            self.render_command_palette(ctx, &window.commands);
            self.render_pending_close_window_dialog(ctx);
            return;
        }

        self.render_top_bar(
            ctx,
            &snapshot.app_host_state.windows,
            &window,
            &mut hovered_drop_target,
        );
        self.render_left_sidebar(ctx, &window, &mut hovered_drop_target);
        self.render_right_sidebar(ctx, &window, &mut hovered_drop_target);
        self.render_bottom_status_bar(ctx, &window);
        self.render_bottom_drawer(ctx, &window);
        self.render_center_stage(ctx, &window, &mut hovered_drop_target);
        self.render_command_palette(ctx, &window.commands);
        self.render_floating_drop_preview_overlay(ctx, &window);
        self.render_pending_close_window_dialog(ctx);
    }

    pub(super) fn dispatch_run_panel_widget(&mut self, event: RunPanelWidgetEvent) {
        match event {
            RunPanelWidgetEvent::Dispatched { intent, .. } => match intent {
                RunPanelIntent::RunManual(_) => self.dispatch_ui_command("run_panel.run_manual"),
                RunPanelIntent::Resume(_) => self.dispatch_ui_command("run_panel.resume_workspace"),
                RunPanelIntent::SetMode(SimulationMode::Hold) => {
                    self.dispatch_ui_command("run_panel.set_hold")
                }
                RunPanelIntent::SetMode(SimulationMode::Active) => {
                    self.dispatch_ui_command("run_panel.set_active")
                }
            },
            RunPanelWidgetEvent::Disabled { .. } | RunPanelWidgetEvent::Missing { .. } => {}
        }
    }

    pub(super) fn dispatch_menu_command(&mut self, command: &StudioGuiCommandMenuCommandModel) {
        self.dispatch_ui_command(&command.command_id);
    }

    pub(super) fn dispatch_ui_command(&mut self, command_id: impl Into<String>) {
        let command_id = command_id.into();
        if self.intercept_authoring_run_if_needed(&command_id) {
            return;
        }

        let canvas_navigation = self.canvas_object_navigation_request(&command_id);
        match self.dispatch_event_result(StudioGuiEvent::UiCommandRequested {
            command_id: command_id.clone(),
        }) {
            Ok(dispatch) => {
                let viewport_requested = self.record_canvas_viewport_navigation_for_command(
                    &command_id,
                    canvas_navigation.as_ref(),
                );
                self.update_workbench_tabs_after_command(&command_id, &dispatch.dispatch.window);
                self.record_canvas_object_navigation_feedback(
                    canvas_navigation.as_ref(),
                    viewport_requested,
                    None,
                );
                self.record_canvas_unit_layout_move_feedback(&dispatch);
                self.record_ui_command_ignored_feedback(&dispatch.dispatch.outcome);
            }
            Err(error) => {
                let message = format!("[{}] {}", error.code().as_str(), error.message());
                self.platform_host
                    .record_activity_line(format!("event failed: {message}"));
                self.record_canvas_object_navigation_feedback(
                    canvas_navigation.as_ref(),
                    false,
                    Some(message.as_str()),
                );
            }
        }
    }

    fn update_workbench_tabs_after_command(
        &mut self,
        command_id: &str,
        window: &StudioGuiWindowModel,
    ) {
        if !matches!(
            command_id,
            "run_panel.run_manual" | "run_panel.resume_workspace" | "run_panel.recover_failure"
        ) {
            return;
        }

        if window.runtime.latest_failure.is_some() {
            self.right_sidebar_tab = StudioShellRightSidebarTab::Run;
            self.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
        } else if window.runtime.latest_solve_snapshot.is_some() {
            self.right_sidebar_tab = StudioShellRightSidebarTab::Results;
            self.bottom_drawer_tab = StudioShellBottomDrawerTab::ResultsTable;
        } else {
            self.right_sidebar_tab = StudioShellRightSidebarTab::Run;
            self.bottom_drawer_tab = StudioShellBottomDrawerTab::RunLog;
        }
    }

    pub(super) fn dispatch_inspector_field_draft_update(
        &mut self,
        command_id: impl Into<String>,
        raw_value: impl Into<String>,
    ) {
        self.dispatch_event(StudioGuiEvent::InspectorFieldDraftUpdateRequested {
            command_id: command_id.into(),
            raw_value: raw_value.into(),
        });
    }

    pub(super) fn dispatch_inspector_field_draft_commit(&mut self, command_id: impl Into<String>) {
        self.dispatch_event(StudioGuiEvent::InspectorFieldDraftCommitRequested {
            command_id: command_id.into(),
        });
    }

    pub(super) fn dispatch_inspector_field_draft_discard(&mut self, command_id: impl Into<String>) {
        self.dispatch_event(StudioGuiEvent::InspectorFieldDraftDiscardRequested {
            command_id: command_id.into(),
        });
    }

    pub(super) fn dispatch_inspector_field_draft_batch_commit(
        &mut self,
        command_id: impl Into<String>,
    ) {
        self.dispatch_event(StudioGuiEvent::InspectorFieldDraftBatchCommitRequested {
            command_id: command_id.into(),
        });
    }

    pub(super) fn dispatch_inspector_field_draft_batch_discard(
        &mut self,
        command_id: impl Into<String>,
    ) {
        self.dispatch_event(StudioGuiEvent::InspectorFieldDraftBatchDiscardRequested {
            command_id: command_id.into(),
        });
    }

    pub(super) fn dispatch_inspector_composition_normalize(
        &mut self,
        command_id: impl Into<String>,
    ) {
        self.dispatch_event(StudioGuiEvent::InspectorCompositionNormalizeRequested {
            command_id: command_id.into(),
        });
    }

    pub(super) fn dispatch_inspector_composition_component_add(
        &mut self,
        command_id: impl Into<String>,
    ) {
        self.dispatch_event(StudioGuiEvent::InspectorCompositionComponentAddRequested {
            command_id: command_id.into(),
        });
    }

    pub(super) fn dispatch_inspector_composition_component_remove(
        &mut self,
        command_id: impl Into<String>,
    ) {
        self.dispatch_event(
            StudioGuiEvent::InspectorCompositionComponentRemoveRequested {
                command_id: command_id.into(),
            },
        );
    }

    pub(super) fn dispatch_canvas_pending_edit_commit(&mut self, position: rf_ui::CanvasPoint) {
        match self
            .dispatch_event_result(StudioGuiEvent::CanvasPendingEditCommitRequested { position })
        {
            Ok(dispatch) => self.record_canvas_pending_edit_commit_feedback(&dispatch, position),
            Err(error) => {
                let message = format!("[{}] {}", error.code().as_str(), error.message());
                self.platform_host
                    .record_activity_line(format!("event failed: {message}"));
                self.record_canvas_pending_edit_commit_error(position, &message);
            }
        }
    }

    pub(super) fn dispatch_canvas_unit_layout_move(
        &mut self,
        unit_id: rf_types::UnitId,
        position: rf_ui::CanvasPoint,
    ) {
        match self.dispatch_event_result(StudioGuiEvent::CanvasUnitLayoutMoveRequested {
            unit_id,
            position,
        }) {
            Ok(dispatch) => self.record_canvas_unit_layout_move_feedback(&dispatch),
            Err(error) => {
                let message = format!("[{}] {}", error.code().as_str(), error.message());
                self.platform_host
                    .record_activity_line(format!("event failed: {message}"));
            }
        }
    }

    pub(super) fn clear_canvas_selection(&mut self) {
        let Some(window_id) = self.current_window_id() else {
            return;
        };
        self.dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::ClearInspectorTarget,
        });
        self.canvas_command_result = None;
        self.canvas_viewport_navigation.active_anchor = None;
    }

    pub(super) fn update_canvas_viewport_offset(&mut self, offset: egui::Vec2) {
        self.canvas_initial_viewport_fit.set_offset(offset);
        let Some(project_path) = self.project_open.current_path() else {
            return;
        };

        if let Err(error) = save_persisted_canvas_viewport(
            &project_path,
            rf_ui::CanvasPoint::new(offset.x as f64, offset.y as f64),
        ) {
            self.platform_host.record_activity_line(format!(
                "save canvas viewport failed [{}]: {} ({})",
                error.code().as_str(),
                error.message(),
                project_path.display()
            ));
        }
    }

    pub(super) fn request_canvas_viewport_fit_to_content(&mut self) {
        self.canvas_viewport_fit_to_content_requested = true;
    }

    pub(super) fn dispatch_layout_mutation(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        mutation: StudioGuiWindowLayoutMutation,
    ) {
        self.dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id,
            mutation,
        });
    }

    pub(super) fn apply_default_hidden_commands_panel_for_current_window(
        &mut self,
    ) -> RfResult<()> {
        self.platform_host.apply_window_layout_preference(
            self.current_window_id(),
            StudioGuiWindowLayoutMutation::SetPanelVisibility {
                area_id: StudioGuiWindowAreaId::Commands,
                visible: false,
            },
        )?;
        Ok(())
    }

    pub(super) fn begin_drag_session(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        area_id: StudioGuiWindowAreaId,
    ) {
        self.clear_drop_preview(window_id);
        self.drag_session = Some(PanelDragSession { area_id, window_id });
    }

    pub(super) fn active_drag_session_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> Option<PanelDragSession> {
        self.drag_session
            .filter(|drag_session| drag_session.window_id == window_id)
    }

    pub(super) fn process_drop_target_response(
        &mut self,
        response: egui::Response,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
        hovered_drop_target: &mut bool,
    ) {
        if response.hovered() {
            *hovered_drop_target = true;
            self.ensure_drop_preview(window_id, query);
        }
        if response.clicked() {
            self.apply_drop_target(window_id, query);
        }
    }

    pub(super) fn ensure_drop_preview(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    ) {
        let preview = ActiveDropPreview { window_id, query };
        if self.active_drop_preview == Some(preview) {
            return;
        }
        self.dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested { window_id, query });
        self.active_drop_preview = Some(preview);
    }

    pub(super) fn clear_drop_preview(&mut self, window_id: Option<StudioWindowHostId>) {
        let Some(active_preview) = self.active_drop_preview else {
            return;
        };
        self.dispatch_event(StudioGuiEvent::WindowDropTargetPreviewCleared {
            window_id: active_preview.window_id.or(window_id),
        });
        self.active_drop_preview = None;
    }

    pub(super) fn apply_drop_target(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    ) {
        self.dispatch_event(StudioGuiEvent::WindowDropTargetApplyRequested { window_id, query });
        self.active_drop_preview = None;
        self.drag_session = None;
    }

    pub(super) fn cancel_drag_session(&mut self, window_id: Option<StudioWindowHostId>) {
        self.drag_session = None;
        self.clear_drop_preview(window_id);
    }

    pub(super) fn finish_drop_preview_cycle(
        &mut self,
        ctx: &egui::Context,
        window_id: Option<StudioWindowHostId>,
        hovered_drop_target: bool,
    ) {
        if self.drag_session.is_none() {
            self.clear_drop_preview(window_id);
            return;
        }
        if ctx.input(|input| input.pointer.any_released()) {
            if let Some(active_preview) = self.active_drop_preview {
                self.apply_drop_target(active_preview.window_id, active_preview.query);
                return;
            }
            self.clear_drop_preview(window_id);
            self.drag_session = None;
            return;
        }
        if !hovered_drop_target {
            self.clear_drop_preview(window_id);
        }
    }

    pub(super) fn record_drop_preview_overlay_anchor(&mut self, rect: egui::Rect, priority: u8) {
        let candidate = DropPreviewOverlayAnchor { rect, priority };
        let replace = self
            .drop_preview_overlay_anchor
            .map(|current| priority >= current.priority)
            .unwrap_or(true);
        if replace {
            self.drop_preview_overlay_anchor = Some(candidate);
        }
    }

    pub(super) fn dispatch_event(&mut self, event: StudioGuiEvent) {
        match self.dispatch_event_result(event.clone()) {
            Ok(_) => {}
            Err(error) => {
                let message = format!("[{}] {}", error.code().as_str(), error.message());
                self.platform_host
                    .record_activity_line(format!("event failed: {message}"));
            }
        }
    }

    pub(super) fn dispatch_event_result(
        &mut self,
        event: StudioGuiEvent,
    ) -> RfResult<StudioGuiPlatformExecutedDispatch> {
        self.platform_host
            .dispatch_event_and_execute_platform_timer(event, &mut self.platform_timer_executor)
    }

    pub(super) fn record_canvas_viewport_navigation_for_command(
        &mut self,
        command_id: &str,
        canvas_navigation: Option<&radishflow_studio::StudioGuiCanvasCommandTargetViewModel>,
    ) -> bool {
        let snapshot = self.platform_host.snapshot();
        let window = snapshot.window_model();
        let focus = window.canvas.widget.view().viewport.focus.as_ref();
        if let Some(anchor_label) = self
            .canvas_viewport_navigation
            .request_for_command(command_id, focus)
        {
            self.last_area_focus = Some(StudioGuiWindowAreaId::Canvas);
            if let Some(target) = canvas_navigation {
                let result = radishflow_studio::StudioGuiCanvasCommandResultViewModel::located(
                    target.clone(),
                    anchor_label,
                );
                self.platform_host
                    .record_activity_line(result.activity_line.clone());
                self.canvas_command_result = Some(result);
            }
            return true;
        }
        false
    }

    pub(super) fn reconcile_canvas_viewport_navigation(
        &mut self,
        focus: Option<&radishflow_studio::StudioGuiCanvasViewportFocusViewModel>,
    ) {
        let Some(expired_anchor) = self.canvas_viewport_navigation.reconcile(focus) else {
            return;
        };
        let Some(target) = self
            .canvas_command_result
            .as_ref()
            .filter(|result| result.anchor_label.as_deref() == Some(expired_anchor.as_str()))
            .map(|result| result.target.clone())
        else {
            return;
        };
        self.canvas_command_result = Some(
            radishflow_studio::StudioGuiCanvasCommandResultViewModel::anchor_expired(
                target,
                expired_anchor,
            ),
        );
    }

    pub(super) fn canvas_object_navigation_request(
        &self,
        command_id: &str,
    ) -> Option<radishflow_studio::StudioGuiCanvasCommandTargetViewModel> {
        let snapshot = self.platform_host.snapshot();
        let window = snapshot.window_model();
        if let Some(item) = window
            .canvas
            .widget
            .view()
            .object_list
            .items
            .iter()
            .find(|item| item.command_id == command_id)
        {
            return Some(item.command_target());
        }

        radishflow_studio::inspector_target_from_command_id(command_id).map(|target| {
            let (kind_label, target_id) = match target {
                rf_ui::InspectorTarget::Unit(unit_id) => ("Unit", unit_id.as_str().to_string()),
                rf_ui::InspectorTarget::Stream(stream_id) => {
                    ("Stream", stream_id.as_str().to_string())
                }
            };
            radishflow_studio::StudioGuiCanvasCommandTargetViewModel {
                kind_label,
                label: target_id.clone(),
                target_id,
                viewport_anchor_label: None,
                command_id: command_id.to_string(),
            }
        })
    }

    pub(super) fn record_canvas_object_navigation_feedback(
        &mut self,
        request: Option<&radishflow_studio::StudioGuiCanvasCommandTargetViewModel>,
        viewport_requested: bool,
        error_message: Option<&str>,
    ) {
        let Some(request) = request else {
            return;
        };
        if viewport_requested {
            return;
        }

        let result = match error_message {
            Some(error_message) => {
                radishflow_studio::StudioGuiCanvasCommandResultViewModel::dispatch_failed(
                    request.clone(),
                    error_message,
                )
            }
            None => radishflow_studio::StudioGuiCanvasCommandResultViewModel::anchor_unavailable(
                request.clone(),
            ),
        };
        self.platform_host
            .record_activity_line(result.activity_line.clone());
        self.canvas_command_result = Some(result);
    }

    pub(super) fn record_canvas_pending_edit_commit_feedback(
        &mut self,
        dispatch: &StudioGuiPlatformExecutedDispatch,
        position: rf_ui::CanvasPoint,
    ) {
        let committed = match &dispatch.dispatch.outcome {
            StudioGuiDriverOutcome::CanvasInteraction(result) => result.committed_edit.as_ref(),
            StudioGuiDriverOutcome::HostCommand(
                radishflow_studio::StudioGuiHostCommandOutcome::UiCommandDispatched(
                    radishflow_studio::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                        result,
                        ..
                    },
                ),
            ) => result.committed_edit.as_ref(),
            _ => None,
        };
        let Some(committed) = committed else {
            let result =
                radishflow_studio::StudioGuiCanvasCommandResultViewModel::pending_edit_unavailable(
                    position,
                );
            self.platform_host
                .record_activity_line(result.activity_line.clone());
            self.canvas_command_result = Some(result);
            return;
        };

        let window = dispatch.dispatch.window.clone();
        let view = window.canvas.widget.view();
        let target = view
            .object_list
            .items
            .iter()
            .find(|item| item.kind_label == "Unit" && item.target_id == committed.unit_id.as_str())
            .map(|item| item.command_target())
            .unwrap_or_else(|| {
                let command_id = radishflow_studio::inspector_target_command_id(
                    &rf_ui::InspectorTarget::Unit(committed.unit_id.clone()),
                );
                radishflow_studio::StudioGuiCanvasCommandTargetViewModel {
                    kind_label: "Unit",
                    target_id: committed.unit_id.as_str().to_string(),
                    label: committed.unit_id.as_str().to_string(),
                    viewport_anchor_label: None,
                    command_id,
                }
            });
        let anchor_label = view
            .viewport
            .focus
            .as_ref()
            .and_then(|focus| {
                (focus.kind_label == target.kind_label
                    && focus.target_id == target.target_id
                    && focus.command_id == target.command_id)
                    .then(|| focus.anchor_label.clone())
            })
            .or_else(|| target.viewport_anchor_label.clone());

        let result = if let Some(anchor_label) = anchor_label {
            let anchor_label = self.canvas_viewport_navigation.request_anchor(anchor_label);
            self.last_area_focus = Some(StudioGuiWindowAreaId::Canvas);
            radishflow_studio::StudioGuiCanvasCommandResultViewModel::created_unit(
                target,
                anchor_label,
                committed,
            )
        } else {
            radishflow_studio::StudioGuiCanvasCommandResultViewModel::anchor_unavailable(target)
        };
        self.platform_host
            .record_activity_line(result.activity_line.clone());
        self.canvas_command_result = Some(result);
    }

    pub(super) fn record_canvas_pending_edit_commit_error(
        &mut self,
        position: rf_ui::CanvasPoint,
        error_message: &str,
    ) {
        let result = radishflow_studio::StudioGuiCanvasCommandResultViewModel::pending_edit_failed(
            position,
            error_message,
        );
        self.platform_host
            .record_activity_line(result.activity_line.clone());
        self.canvas_command_result = Some(result);
    }

    pub(super) fn record_canvas_unit_layout_move_feedback(
        &mut self,
        dispatch: &StudioGuiPlatformExecutedDispatch,
    ) {
        let moved = match &dispatch.dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                radishflow_studio::StudioGuiHostCommandOutcome::CanvasUnitLayoutMoved(result),
            ) => Some(result),
            StudioGuiDriverOutcome::HostCommand(
                radishflow_studio::StudioGuiHostCommandOutcome::UiCommandDispatched(
                    radishflow_studio::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasUnitLayoutMove {
                        result,
                        ..
                    },
                ),
            ) => Some(result),
            _ => None,
        };
        let Some(moved) = moved else {
            return;
        };

        let window = dispatch.dispatch.window.clone();
        let view = window.canvas.widget.view();
        let target = view
            .object_list
            .items
            .iter()
            .find(|item| item.kind_label == "Unit" && item.target_id == moved.unit_id.as_str())
            .map(|item| item.command_target())
            .unwrap_or_else(
                || radishflow_studio::StudioGuiCanvasCommandTargetViewModel {
                    kind_label: "Unit",
                    target_id: moved.unit_id.as_str().to_string(),
                    label: moved.unit_id.as_str().to_string(),
                    viewport_anchor_label: None,
                    command_id: radishflow_studio::inspector_target_command_id(
                        &rf_ui::InspectorTarget::Unit(moved.unit_id.clone()),
                    ),
                },
            );
        let anchor_label = target
            .viewport_anchor_label
            .clone()
            .unwrap_or_else(|| moved.unit_id.as_str().to_string());
        let anchor_label = self.canvas_viewport_navigation.request_anchor(anchor_label);
        self.last_area_focus = Some(StudioGuiWindowAreaId::Canvas);
        let result = radishflow_studio::StudioGuiCanvasCommandResultViewModel::moved_unit(
            target,
            anchor_label,
            moved.previous_position,
            moved.position,
        );
        self.platform_host
            .record_activity_line(result.activity_line.clone());
        self.canvas_command_result = Some(result);
    }

    pub(super) fn canvas_command_result_command_surface(
        &self,
    ) -> Option<radishflow_studio::StudioGuiCanvasCommandResultCommandSurfaceViewModel> {
        self.canvas_command_result
            .as_ref()
            .map(|result| result.command_surface())
    }

    pub(super) fn record_ui_command_ignored_feedback(&mut self, outcome: &StudioGuiDriverOutcome) {
        if let StudioGuiDriverOutcome::HostCommand(
            radishflow_studio::StudioGuiHostCommandOutcome::UiCommandDispatched(result),
        ) = outcome
        {
            match result {
                radishflow_studio::StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                    command_id,
                    detail,
                    ..
                } => {
                    self.platform_host
                        .record_activity_line(format!("ui command disabled: {command_id}: {detail}"));
                }
                radishflow_studio::StudioGuiHostUiCommandDispatchResult::IgnoredMissing {
                    command_id,
                    ..
                } => {
                    self.platform_host
                        .record_activity_line(format!("ui command missing: {command_id}"));
                }
                radishflow_studio::StudioGuiHostUiCommandDispatchResult::Executed(_)
                | radishflow_studio::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasInteraction {
                    ..
                }
                | radishflow_studio::StudioGuiHostUiCommandDispatchResult::ExecutedCanvasUnitLayoutMove {
                    ..
                } => {}
            }
        }
    }

    pub(super) fn drain_due_timers(&mut self, ctx: &egui::Context) {
        let now = SystemTime::now();
        match drain_due_platform_timer_callbacks(
            &mut self.platform_host,
            &mut self.platform_timer_executor,
            now,
        ) {
            Ok(callback_batch) => {
                for callback in callback_batch.callbacks {
                    match callback {
                        StudioGuiPlatformExecutedNativeTimerCallbackOutcome::Dispatched(_) => {}
                        StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer { .. } => {}
                        StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredStaleNativeTimer { .. } => {}
                    }
                }
            }
            Err(error) => {
                self.platform_host.record_activity_line(format!(
                    "timer dispatch failed [{}]: {}",
                    error.code().as_str(),
                    error.message()
                ));
            }
        }

        if let Some(next_due_at) = self.platform_host.next_native_timer_due_at() {
            let delay = next_due_at.duration_since(now).unwrap_or(Duration::ZERO);
            ctx.request_repaint_after(delay);
        }
    }

    pub(super) fn sync_viewport_close(&mut self, ctx: &egui::Context) -> bool {
        if !ctx.input(|input| input.viewport().close_requested()) {
            return false;
        }

        let should_stop_rendering = self.close_current_window_for_viewport_request();
        if !should_stop_rendering {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        should_stop_rendering
    }

    pub(super) fn close_current_window_for_viewport_request(&mut self) -> bool {
        let Some(window_id) = self.current_window_id() else {
            return true;
        };

        if self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .has_unsaved_changes
        {
            self.request_close_window_confirmation(window_id);
            return false;
        }

        self.close_window_without_confirmation(window_id)
    }

    fn request_close_window_confirmation(&mut self, window_id: StudioWindowHostId) {
        self.project_open.pending_confirmation = None;
        self.project_open.pending_blank_project_confirmation = false;
        self.project_open.pending_authoring_blank_project = None;
        self.project_open.pending_save_as_overwrite = None;
        self.project_open.pending_close_window_confirmation = Some(window_id);
        self.project_open.notice = None;
        self.platform_host
            .record_activity_line("close blocked by unsaved workspace changes".to_string());
    }

    fn render_pending_close_window_dialog(&mut self, ctx: &egui::Context) {
        if self
            .project_open
            .pending_close_window_confirmation
            .is_none()
        {
            return;
        }

        egui::Window::new(unsaved_changes_notice_title(self.locale))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(360.0);
                render_wrapped_label(ui, close_workspace_discard_notice_detail(self.locale));
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(self.locale.text(ShellText::SaveAndCloseProject))
                        .clicked()
                        && self.save_pending_close_window()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui
                        .button(self.locale.text(ShellText::DiscardAndCloseProject))
                        .clicked()
                        && self.confirm_pending_close_window()
                    {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui
                        .button(self.locale.text(ShellText::CancelCloseProject))
                        .clicked()
                    {
                        self.cancel_pending_close_window();
                    }
                });
            });
    }

    pub(super) fn save_pending_close_window(&mut self) -> bool {
        if self
            .project_open
            .pending_close_window_confirmation
            .is_none()
        {
            return false;
        }

        self.save_project();
        if self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .has_unsaved_changes
        {
            return false;
        }

        self.confirm_pending_close_window()
    }

    pub(super) fn confirm_pending_close_window(&mut self) -> bool {
        let Some(window_id) = self.project_open.pending_close_window_confirmation.take() else {
            return false;
        };
        self.close_window_without_confirmation(window_id)
    }

    pub(super) fn cancel_pending_close_window(&mut self) {
        self.project_open.pending_close_window_confirmation = None;
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Info,
            title: close_workspace_canceled_notice_title(self.locale).to_string(),
            detail: current_workspace_remains_open_notice_detail(self.locale).to_string(),
        });
    }

    fn close_window_without_confirmation(&mut self, window_id: StudioWindowHostId) -> bool {
        self.cancel_drag_session(Some(window_id));
        self.dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id });
        self.logical_window_count() == 0
    }

    pub(super) fn sync_viewport_lifecycle(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|input| input.viewport().focused.unwrap_or(input.focused));
        self.last_viewport_focused = Some(focused);
    }

    pub(super) fn handle_command_palette_toggle_shortcut(&mut self, ctx: &egui::Context) -> bool {
        let toggle_requested =
            ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::K));
        if toggle_requested {
            self.command_palette.toggle();
        }
        toggle_requested
    }

    pub(super) fn handle_command_palette_keyboard(
        &mut self,
        ctx: &egui::Context,
        commands: &radishflow_studio::StudioGuiWindowCommandAreaModel,
    ) -> bool {
        if !self.command_palette.open {
            return false;
        }

        let palette_items = commands.palette_items(&self.command_palette.query);
        self.command_palette.sync_selection(&palette_items);

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.command_palette.close();
            return true;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::ArrowDown)) {
            self.command_palette.move_selection(1, &palette_items);
            return true;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::ArrowUp)) {
            self.command_palette.move_selection(-1, &palette_items);
            return true;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::Enter)) {
            let selected_command_id = selected_palette_item_command_id(
                &palette_items,
                self.command_palette.selected_index,
            );
            if let Some(command_id) = selected_command_id {
                self.dispatch_ui_command(command_id);
                self.command_palette.close();
            }
            return true;
        }

        false
    }

    pub(super) fn dispatch_shortcuts(&mut self, ctx: &egui::Context) {
        let focus_context = self.focus_context(ctx);
        if matches!(focus_context, StudioGuiFocusContext::CommandPalette) {
            return;
        }

        if self.drag_session.is_some() && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.cancel_drag_session(self.current_window_id());
            return;
        }

        let shortcuts = ctx.input(collect_shortcuts);
        for shortcut in shortcuts {
            self.dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut,
                focus_context,
            });
        }
    }

    pub(super) fn focus_context(&self, ctx: &egui::Context) -> StudioGuiFocusContext {
        if self.command_palette.open {
            StudioGuiFocusContext::CommandPalette
        } else if ctx.wants_keyboard_input() {
            StudioGuiFocusContext::TextInput
        } else if self
            .platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .is_some()
        {
            StudioGuiFocusContext::CanvasSuggestionFocused
        } else if self.last_area_focus == Some(StudioGuiWindowAreaId::Canvas) {
            StudioGuiFocusContext::Canvas
        } else {
            StudioGuiFocusContext::Global
        }
    }

    pub(super) fn current_window_id(&self) -> Option<StudioWindowHostId> {
        self.platform_host
            .snapshot()
            .window_model()
            .layout_state
            .scope
            .window_id
    }

    pub(super) fn logical_window_count(&self) -> usize {
        self.platform_host.snapshot().app_host_state.windows.len()
    }

    pub(super) fn update_area_focus_from_rect(
        &mut self,
        ctx: &egui::Context,
        area_id: StudioGuiWindowAreaId,
        rect: egui::Rect,
    ) {
        let pointer_pos = ctx.pointer_latest_pos();
        let pressed = ctx.input(|input| input.pointer.any_pressed());
        let released = ctx.input(|input| input.pointer.any_released());
        if pointer_pos.is_some_and(|pos| rect.contains(pos)) && (pressed || released) {
            self.last_area_focus = Some(area_id);
        }
    }
}

fn project_opened_notice_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Project opened",
        StudioShellLocale::ZhCn => "项目已打开",
    }
}

fn project_opened_notice_detail(
    locale: StudioShellLocale,
    source_label: &str,
    project_path: &std::path::Path,
) -> String {
    match locale {
        StudioShellLocale::En => format!("Opened {source_label}: {}", project_path.display()),
        StudioShellLocale::ZhCn => format!(
            "已打开{}: {}",
            localized_project_source_label(source_label),
            project_path.display()
        ),
    }
}

fn unsaved_changes_notice_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Unsaved changes",
        StudioShellLocale::ZhCn => "未保存更改",
    }
}

fn open_project_discard_notice_detail(
    locale: StudioShellLocale,
    source_label: &str,
    project_path: &std::path::Path,
) -> String {
    match locale {
        StudioShellLocale::En => format!(
            "Opening {source_label} will discard changes after the last saved revision: {}",
            project_path.display()
        ),
        StudioShellLocale::ZhCn => format!(
            "打开{}会放弃上次保存修订之后的更改: {}",
            localized_project_source_label(source_label),
            project_path.display()
        ),
    }
}

fn create_blank_project_discard_notice_detail(locale: StudioShellLocale) -> String {
    match locale {
        StudioShellLocale::En => {
            "Creating a blank project will discard changes after the last saved revision."
                .to_string()
        }
        StudioShellLocale::ZhCn => "新建空白项目会放弃上次保存修订之后的更改。".to_string(),
    }
}

fn close_workspace_discard_notice_detail(locale: StudioShellLocale) -> String {
    match locale {
        StudioShellLocale::En => {
            "Closing RadishFlow Studio will discard changes after the last saved revision."
                .to_string()
        }
        StudioShellLocale::ZhCn => {
            "关闭 RadishFlow Studio 会放弃上次保存修订之后的更改。".to_string()
        }
    }
}

fn blank_project_canceled_notice_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Blank project canceled",
        StudioShellLocale::ZhCn => "已取消新建项目",
    }
}

fn blank_project_created_notice_title(
    locale: StudioShellLocale,
    authoring_case: Option<AuthoringCaseKind>,
) -> &'static str {
    match (locale, authoring_case) {
        (StudioShellLocale::En, Some(AuthoringCaseKind::MixerFlash)) => "Mixer-Flash case started",
        (StudioShellLocale::En, Some(AuthoringCaseKind::HeaterFlash)) => {
            "Heater-Flash case started"
        }
        (StudioShellLocale::En, None) => "Blank project created",
        (StudioShellLocale::ZhCn, Some(AuthoringCaseKind::MixerFlash)) => {
            "已开始 Mixer-Flash 小案例"
        }
        (StudioShellLocale::ZhCn, Some(AuthoringCaseKind::HeaterFlash)) => {
            "已开始 Heater-Flash 小案例"
        }
        (StudioShellLocale::ZhCn, None) => "Blank project created",
    }
}

fn blank_project_created_notice_detail(
    locale: StudioShellLocale,
    authoring_case: Option<AuthoringCaseKind>,
) -> &'static str {
    match (locale, authoring_case) {
        (StudioShellLocale::En, Some(_)) => {
            "Created an untitled blank project and opened the placement checklist."
        }
        (StudioShellLocale::En, None) => {
            "Created an untitled blank project. Use Save to choose a .rfproj.json path."
        }
        (StudioShellLocale::ZhCn, Some(_)) => {
            "已新建未命名空白项目，并打开放置面板中的小案例任务清单。"
        }
        (StudioShellLocale::ZhCn, None) => {
            "Created an untitled blank project. Use Save to choose a .rfproj.json path."
        }
    }
}

fn blank_project_created_activity_line(authoring_case: Option<AuthoringCaseKind>) -> &'static str {
    match authoring_case {
        Some(AuthoringCaseKind::MixerFlash) => {
            "started mixer-flash authoring case from blank project"
        }
        Some(AuthoringCaseKind::HeaterFlash) => {
            "started heater-flash authoring case from blank project"
        }
        None => "created untitled blank project",
    }
}

fn close_workspace_canceled_notice_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Close canceled",
        StudioShellLocale::ZhCn => "已取消关闭",
    }
}

fn current_workspace_remains_open_notice_detail(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Current workspace remains open.",
        StudioShellLocale::ZhCn => "当前工作区保持打开。",
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
    error: &std::io::Error,
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

fn localized_project_source_label(source_label: &str) -> &'static str {
    match source_label {
        "example project" => "示例",
        "recent project" => "最近项目",
        "project picker" => "文件选择器项目",
        "project" => "项目",
        _ => "项目",
    }
}
