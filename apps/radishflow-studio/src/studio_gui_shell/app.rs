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

    pub(super) fn save_project(&mut self) {
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
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: "Unsaved changes".to_string(),
                detail: format!(
                    "Opening {source_label} will discard changes after the last saved revision: {}",
                    project_path.display()
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
        let config = StudioRuntimeConfig {
            project_path: project_path.clone(),
            ..StudioRuntimeConfig::default()
        };

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
                self.canvas_command_result = None;
                self.result_inspector.reset();
                self.project_open.path_input = project_path.display().to_string();
                let recent_projects_notice =
                    self.record_and_persist_recent_project(project_path.clone());
                self.project_open.pending_confirmation = None;
                self.project_open.pending_save_as_overwrite = None;
                self.project_open.notice =
                    Some(recent_projects_notice.unwrap_or(ProjectOpenNotice {
                        level: ProjectOpenNoticeLevel::Info,
                        title: "Project opened".to_string(),
                        detail: format!("Opened {source_label}: {}", project_path.display()),
                    }));
                self.platform_host.record_activity_line(format!(
                    "opened {source_label}: {}",
                    project_path.display()
                ));
                self.dispatch_event(StudioGuiEvent::OpenWindowRequested);
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
        self.sync_viewport_close(ctx);
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
        self.render_top_bar(
            ctx,
            &snapshot.app_host_state.windows,
            &window,
            &mut hovered_drop_target,
        );
        self.render_left_sidebar(ctx, &window, &mut hovered_drop_target);
        self.render_right_sidebar(ctx, &window, &mut hovered_drop_target);
        self.render_center_stage(ctx, &window, &mut hovered_drop_target);
        self.render_command_palette(ctx, &window.commands);
        self.render_floating_drop_preview_overlay(ctx, &window);
        self.finish_drop_preview_cycle(
            ctx,
            window.layout_state.scope.window_id,
            hovered_drop_target,
        );
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
        let canvas_navigation = self.canvas_object_navigation_request(&command_id);
        match self.dispatch_event_result(StudioGuiEvent::UiCommandRequested {
            command_id: command_id.clone(),
        }) {
            Ok(dispatch) => {
                let viewport_requested = self.record_canvas_viewport_navigation_for_command(
                    &command_id,
                    canvas_navigation.as_ref(),
                );
                self.record_canvas_object_navigation_feedback(
                    canvas_navigation.as_ref(),
                    viewport_requested,
                    None,
                );
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

    pub(super) fn dispatch_inspector_field_draft_batch_commit(
        &mut self,
        command_id: impl Into<String>,
    ) {
        self.dispatch_event(StudioGuiEvent::InspectorFieldDraftBatchCommitRequested {
            command_id: command_id.into(),
        });
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

    pub(super) fn sync_viewport_close(&mut self, ctx: &egui::Context) {
        if !ctx.input(|input| input.viewport().close_requested()) {
            return;
        }

        let Some(window_id) = self.current_window_id() else {
            return;
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        self.cancel_drag_session(Some(window_id));
        self.dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id });

        if self.logical_window_count() == 0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    pub(super) fn sync_viewport_lifecycle(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|input| input.viewport().focused.unwrap_or(input.focused));
        let became_focused = self
            .last_viewport_focused
            .map(|previous| !previous && focused)
            .unwrap_or(false);
        self.last_viewport_focused = Some(focused);

        if !became_focused {
            return;
        }

        let window_id = self.current_window_id();
        if let Some(window_id) = window_id {
            self.dispatch_event(StudioGuiEvent::WindowForegrounded { window_id });
        }
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
