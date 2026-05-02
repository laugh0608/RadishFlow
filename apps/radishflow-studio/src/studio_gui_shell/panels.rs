use super::*;

impl ReadyAppState {
    pub(super) fn render_commands_area(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
    ) {
        egui::ScrollArea::vertical()
            .id_salt(format!(
                "scroll:{}:{}",
                window.layout_state.scope.layout_key,
                area_label(area_id)
            ))
            .show(ui, |ui| {
                for section in &window.commands.command_list_sections {
                    ui.label(egui::RichText::new(section.title).strong());
                    for command in &section.items {
                        self.render_command_list_entry(ui, command);
                    }
                    ui.add_space(6.0);
                }
            });
    }

    pub(super) fn render_command_list_entry(
        &mut self,
        ui: &mut egui::Ui,
        command: &radishflow_studio::StudioGuiWindowCommandListItemModel,
    ) {
        if ui
            .add_enabled(command.enabled, egui::Button::new(&command.label))
            .clicked()
        {
            self.dispatch_ui_command(&command.command_id);
        }
        ui.label(&command.detail);
        ui.small(&command.menu_path_text);
        ui.add_space(4.0);
    }

    pub(super) fn render_canvas_area(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
    ) {
        let widget = &window.canvas.widget;
        ui.horizontal_wrapped(|ui| {
            for action in &widget.actions {
                let label = match action.shortcut.as_ref() {
                    Some(shortcut) => format!("{} ({})", action.label, format_shortcut(shortcut)),
                    None => action.label.to_string(),
                };
                if ui
                    .add_enabled(action.enabled, egui::Button::new(label))
                    .clicked()
                {
                    match widget.activate(action.id) {
                        radishflow_studio::StudioGuiCanvasWidgetEvent::Requested {
                            event, ..
                        } => self.dispatch_event(event),
                        radishflow_studio::StudioGuiCanvasWidgetEvent::Disabled { .. }
                        | radishflow_studio::StudioGuiCanvasWidgetEvent::Missing { .. } => {}
                    }
                }
            }
        });
        self.render_canvas_selection_summary(ui, widget);
        self.render_canvas_viewport_summary(ui, widget);
        self.render_canvas_legend(ui, widget);
        ui.separator();
        let hovered_stream_id = self.render_canvas_drop_surface(ui, widget);
        ui.add_space(8.0);
        self.render_canvas_object_list(ui, widget, hovered_stream_id.as_deref());
        ui.add_space(8.0);
        egui::ScrollArea::vertical()
            .id_salt(format!(
                "scroll:{}:{}:suggestions",
                window.layout_state.scope.layout_key,
                area_label(area_id)
            ))
            .show(ui, |ui| {
                for suggestion in &widget.view().suggestions {
                    let frame = egui::Frame::group(ui.style());
                    frame.show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let focus = if suggestion.is_focused {
                                "Focused"
                            } else {
                                "Suggestion"
                            };
                            ui.label(egui::RichText::new(focus).strong());
                            ui.label(format!("{:.0}%", suggestion.confidence * 100.0));
                            ui.label(format!("source={}", suggestion.source_label));
                            ui.label(format!("status={}", suggestion.status_label));
                        });
                        ui.label(format!("target={}", suggestion.target_unit_id));
                        ui.label(&suggestion.reason);
                        ui.small(format!("id={}", suggestion.id));
                    });
                    ui.add_space(6.0);
                }
            });
    }

    fn render_canvas_object_list(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
        hovered_stream_id: Option<&str>,
    ) {
        let object_list = &widget.view().object_list;
        let selected_filter_enabled = object_list
            .filter_options
            .iter()
            .find(|option| option.filter_id == self.canvas_object_filter.filter_id())
            .map(|option| option.enabled)
            .unwrap_or(false);
        if !selected_filter_enabled {
            self.canvas_object_filter = CanvasObjectListFilter::All;
        }
        ui.horizontal_wrapped(|ui| {
            ui.small(egui::RichText::new("Objects").strong());
            render_status_chip(
                ui,
                &format!("{} units", object_list.unit_count),
                egui::Color32::from_rgb(86, 118, 168),
            );
            render_status_chip(
                ui,
                &format!("{} streams", object_list.stream_count),
                egui::Color32::from_rgb(42, 142, 122),
            );
            if object_list.attention_count > 0 {
                render_status_chip(
                    ui,
                    &format!("{} attention", object_list.attention_count),
                    notice_color(rf_ui::RunPanelNoticeLevel::Warning),
                );
            }
        });
        ui.horizontal_wrapped(|ui| {
            for option in &object_list.filter_options {
                let selected = self.canvas_object_filter.filter_id() == option.filter_id;
                let label = format!("{} {}", option.label, option.count);
                if ui
                    .add_enabled(option.enabled, egui::Button::new(label).selected(selected))
                    .on_hover_text(option.detail)
                    .clicked()
                {
                    if let Some(filter) = CanvasObjectListFilter::from_filter_id(option.filter_id) {
                        self.canvas_object_filter = filter;
                    }
                }
            }
        });
        if object_list.items.is_empty() {
            ui.small("none");
            return;
        }
        let visible_items = object_list
            .items
            .iter()
            .filter(|item| self.canvas_object_filter.matches(item))
            .collect::<Vec<_>>();
        if visible_items.is_empty() {
            ui.small("no objects in this filter");
            return;
        }

        egui::Grid::new("canvas-object-list")
            .num_columns(3)
            .striped(true)
            .show(ui, |ui| {
                for item in visible_items {
                    let is_hover_related = hovered_stream_id
                        .map(|stream_id| {
                            item.related_stream_ids
                                .iter()
                                .any(|related_stream_id| related_stream_id == stream_id)
                        })
                        .unwrap_or(false);
                    render_status_chip(
                        ui,
                        item.kind_label,
                        if is_hover_related {
                            egui::Color32::from_rgb(180, 124, 42)
                        } else if item.kind_label == "Unit" {
                            egui::Color32::from_rgb(48, 112, 188)
                        } else {
                            egui::Color32::from_rgb(42, 142, 122)
                        },
                    );
                    let response = ui
                        .add(
                            egui::Button::new(&item.label)
                                .selected(item.is_active || is_hover_related),
                        )
                        .on_hover_text(&item.detail);
                    if response.clicked() {
                        self.dispatch_ui_command(&item.command_id);
                    }
                    ui.horizontal_wrapped(|ui| {
                        for badge in &item.status_badges {
                            render_status_chip(
                                ui,
                                &badge.short_label,
                                canvas_status_badge_color(badge.severity_label),
                            );
                        }
                        ui.small(format!("{} · {}", item.target_id, item.detail));
                    });
                    ui.end_row();
                }
            });
    }

    fn render_canvas_selection_summary(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            if let Some(status) = widget.view().run_status.as_ref() {
                render_status_chip(
                    ui,
                    status.status_label,
                    run_status_color(status.status_label),
                );
                if status.attention_count > 0 {
                    render_status_chip(
                        ui,
                        &format!("{} attention", status.attention_count),
                        notice_color(rf_ui::RunPanelNoticeLevel::Warning),
                    );
                }
                if let Some(summary) = status.summary.as_ref() {
                    ui.small(truncate_canvas_label(summary, 42));
                } else if let Some(reason) = status.pending_reason_label {
                    ui.small(format!("pending={reason}"));
                }
                ui.separator();
            }
            ui.small(egui::RichText::new("Selection").strong());
            if let Some(selection) = widget.view().current_selection.as_ref() {
                render_status_chip(
                    ui,
                    selection.kind_label,
                    egui::Color32::from_rgb(48, 112, 188),
                );
                ui.small(format!("{} · {}", selection.target_id, selection.summary));
                if ui
                    .small_button("Focus")
                    .on_hover_text("Focus the selected Inspector target")
                    .clicked()
                {
                    self.dispatch_ui_command(&selection.command_id);
                }
            } else {
                ui.small("none");
            }
        });
    }

    fn render_canvas_viewport_summary(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
    ) {
        let viewport = &widget.view().viewport;
        ui.horizontal_wrapped(|ui| {
            ui.small(egui::RichText::new("Viewport").strong());
            render_status_chip(
                ui,
                viewport.mode_label,
                egui::Color32::from_rgb(86, 118, 168),
            );
            render_status_chip(
                ui,
                viewport.layout_label,
                egui::Color32::from_rgb(86, 96, 108),
            );
            ui.small(&viewport.summary);
            if let Some(focus) = viewport.focus.as_ref() {
                render_status_chip(
                    ui,
                    &format!("{} {}", focus.kind_label, focus.target_id),
                    egui::Color32::from_rgb(48, 112, 188),
                );
                ui.small(&focus.anchor_label);
                if ui
                    .small_button("Focus")
                    .on_hover_text(&focus.detail)
                    .clicked()
                {
                    self.dispatch_ui_command(&focus.command_id);
                }
            }
        });
        if let Some(result) = self.canvas_command_result.as_ref() {
            ui.colored_label(notice_color(result.level), &result.title);
            render_wrapped_small(ui, &result.detail);
        }
    }

    fn render_canvas_legend(
        &self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
    ) {
        let legend = &widget.view().legend;
        if legend.items.is_empty() {
            return;
        }

        ui.horizontal_wrapped(|ui| {
            ui.small(egui::RichText::new(legend.title).strong());
            for item in &legend.items {
                let color = canvas_legend_swatch_color(item.swatch_label);
                let label = format!("{}: {}", item.kind_label, item.label);
                render_status_chip(ui, &label, color);
                ui.add(
                    egui::Label::new(egui::RichText::new(&item.detail).small())
                        .wrap()
                        .sense(egui::Sense::hover()),
                )
                .on_hover_text(&item.detail);
            }
        });
    }

    fn render_canvas_drop_surface(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
    ) -> Option<String> {
        let view = widget.view();
        let pending_edit = view.pending_edit.as_ref();
        let focus_callout = view.focus_callout.as_ref();
        let unit_blocks = &view.unit_blocks;
        let stream_lines = &view.stream_lines;
        self.reconcile_canvas_viewport_navigation(view.viewport.focus.as_ref());
        let available_width = ui.available_width().max(320.0);
        let desired_size = egui::vec2(available_width, 280.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let painter = ui.painter_at(rect);
        paint_canvas_drop_surface(&painter, rect, pending_edit.is_some());

        let title = pending_edit
            .map(|pending| pending.summary.as_str())
            .unwrap_or("Select a canvas tool");
        let subtitle = if pending_edit.is_some() {
            "Click to place the pending unit"
        } else {
            "Use Place Flash Drum to start a canvas edit"
        };
        paint_canvas_surface_labels(&painter, rect, title, subtitle);

        let mut clicked_stream_command = None;
        for stream in stream_lines {
            let geometry = canvas_stream_line_geometry(rect, stream);
            let is_viewport_focus = self
                .canvas_viewport_navigation
                .is_active_anchor(&stream.line_id);
            if self
                .canvas_viewport_navigation
                .take_pending_scroll_for_anchor(&stream.line_id)
            {
                ui.scroll_to_rect(
                    canvas_stream_line_hit_rect(geometry).expand(42.0),
                    Some(egui::Align::Center),
                );
            }
            if is_viewport_focus {
                paint_canvas_viewport_stream_focus(&painter, geometry);
            }
            paint_canvas_stream_line(&painter, geometry, stream);
            paint_canvas_stream_status_badges(&painter, geometry, &stream.status_badges);
            let stream_response = ui
                .interact(
                    canvas_stream_line_hit_rect(geometry),
                    ui.make_persistent_id(format!("canvas-stream:{}", stream.line_id)),
                    egui::Sense::click(),
                )
                .on_hover_text(&stream.hover_text)
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if stream_response.clicked() {
                clicked_stream_command = Some(stream.command_id.clone());
            }
        }

        let mut clicked_unit = false;
        let mut hovered_port_stream_id = None;
        let mut hovered_port_callout = None;
        for unit in unit_blocks {
            let unit_rect = canvas_unit_block_rect(rect, unit.layout_slot);
            let anchor_label = canvas_unit_viewport_anchor_label(unit.layout_slot);
            let is_viewport_focus = self
                .canvas_viewport_navigation
                .is_active_anchor(&anchor_label);
            if self
                .canvas_viewport_navigation
                .take_pending_scroll_for_anchor(&anchor_label)
            {
                ui.scroll_to_rect(unit_rect.expand(42.0), Some(egui::Align::Center));
            }
            if is_viewport_focus {
                paint_canvas_viewport_unit_focus(&painter, unit_rect);
            }
            paint_canvas_unit_block(&painter, unit_rect, unit);
            paint_canvas_unit_status_badges(&painter, unit_rect, &unit.status_badges);
            for port in &unit.ports {
                let port_anchor = canvas_unit_port_anchor_in_rect(
                    unit_rect,
                    port.direction_label == "outlet",
                    port.side_index,
                    port.side_count,
                );
                let port_response = ui
                    .interact(
                        egui::Rect::from_center_size(port_anchor, egui::vec2(18.0, 18.0)),
                        ui.make_persistent_id(format!(
                            "canvas-port:{}:{}",
                            unit.unit_id, port.name
                        )),
                        egui::Sense::hover(),
                    )
                    .on_hover_text(&port.hover_text);
                if port_response.hovered() {
                    hovered_port_stream_id = port.stream_id.clone();
                    hovered_port_callout = Some((port_anchor, port));
                }
            }
            let unit_response = ui
                .interact(
                    unit_rect,
                    ui.make_persistent_id(format!("canvas-unit:{}", unit.unit_id)),
                    egui::Sense::click(),
                )
                .on_hover_text(&unit.hover_text)
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if unit_response.clicked() {
                clicked_unit = true;
                self.dispatch_ui_command(&unit.command_id);
            }
        }

        if let Some(callout) = focus_callout {
            if let Some(anchor) =
                canvas_focus_callout_anchor(rect, callout, unit_blocks, stream_lines)
            {
                paint_canvas_focus_callout(&painter, rect, anchor, callout);
            }
        }
        if let Some((anchor, port)) = hovered_port_callout {
            paint_canvas_port_hover_callout(&painter, rect, anchor, port);
        }

        let clicked_stream = clicked_stream_command.is_some();
        if !clicked_unit {
            if let Some(command_id) = clicked_stream_command {
                self.dispatch_ui_command(command_id);
            }
        }

        let response = if pending_edit.is_some() {
            response.on_hover_cursor(egui::CursorIcon::Crosshair)
        } else {
            response
        };
        if pending_edit.is_some() && response.clicked() && !clicked_unit && !clicked_stream {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let local = pointer_pos - rect.min;
                self.dispatch_event(StudioGuiEvent::CanvasPendingEditCommitRequested {
                    position: rf_ui::CanvasPoint::new(local.x as f64, local.y as f64),
                });
            }
        }

        hovered_port_stream_id
    }

    pub(super) fn render_runtime_area(
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

    pub(super) fn render_runtime_area_contents(
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
            ui.vertical(|ui| {
                let primary = run_panel.primary_action();
                let response = ui.add_enabled(
                    primary.enabled,
                    egui::Button::new(self.locale.runtime_label(primary.label).as_ref()),
                );
                let response = response.on_hover_text(primary.detail);
                if response.clicked() {
                    self.dispatch_run_panel_widget(run_panel.activate_primary());
                }
                render_wrapped_small(ui, primary.detail);

                for action in &run_panel_view.secondary_actions {
                    ui.add_space(4.0);
                    let response = ui.add_enabled(
                        action.enabled,
                        egui::Button::new(self.locale.runtime_label(action.label).as_ref()),
                    );
                    let response = response.on_hover_text(action.detail);
                    if response.clicked() {
                        self.dispatch_run_panel_widget(run_panel.activate(action.id));
                    }
                    render_wrapped_small(ui, action.detail);
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
                self.render_active_inspector_detail(ui, detail);
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
            ui.label(egui::RichText::new(self.locale.text(ShellText::ProjectPath)).strong());
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
                    let inspector = snapshot.result_inspector_with_comparison(
                        selected_stream_id.as_deref(),
                        self.result_inspector.comparison_stream_id.as_deref(),
                    );
                    self.render_result_inspector(ui, &inspector);
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
                if let Some(target) = failure.recovery_target.as_ref() {
                    render_wrapped_small(
                        ui,
                        format!(
                            "{}: {}",
                            self.locale.text(ShellText::RecoveryTarget),
                            target.summary
                        ),
                    );
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
                        render_wrapped_small(
                            ui,
                            format!(
                                "#{} {} -> {}",
                                step.index,
                                step.unit_id,
                                step.produced_streams.join(", ")
                            ),
                        );
                        render_wrapped_label(ui, &step.summary);
                        ui.add_space(4.0);
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
                            for diagnostic in &snapshot.diagnostics {
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
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new(self.locale.text(ShellText::Scheduler)).strong());
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
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(self.locale.text(ShellText::RuntimeLog)).strong());
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
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(self.locale.text(ShellText::GuiActivity)).strong());
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

    pub(super) fn render_stream_result_inspector(
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
                .num_columns(4)
                .striped(true)
                .show(ui, |ui| {
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Phase)).strong());
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Fraction)).strong());
                    ui.small(
                        egui::RichText::new(self.locale.text(ShellText::OverallComposition))
                            .strong(),
                    );
                    ui.small(egui::RichText::new(self.locale.text(ShellText::Enthalpy)).strong());
                    ui.end_row();
                    for row in &stream.phase_rows {
                        ui.small(&row.label);
                        ui.small(&row.phase_fraction_text);
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
                            if submit_on_enter
                                || ui
                                    .small_button(self.locale.text(ShellText::InspectorFieldApply))
                                    .clicked()
                            {
                                self.dispatch_inspector_field_draft_commit(command_id.clone());
                            }
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
                .num_columns(4)
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
                    ui.end_row();
                    for port in &detail.unit_ports {
                        render_wrapped_small(ui, &port.name);
                        render_wrapped_small(ui, &port.direction);
                        render_wrapped_small(ui, &port.kind);
                        match (&port.stream_id, &port.stream_action) {
                            (_, Some(action)) => self.render_small_command_action(ui, action),
                            (Some(stream_id), None) => render_wrapped_small(ui, stream_id),
                            (None, None) => {
                                ui.small("-");
                            }
                        };
                        ui.end_row();
                    }
                });
        }

        if let Some(stream) = detail.latest_stream_result.as_ref() {
            ui.add_space(4.0);
            ui.small(
                egui::RichText::new(self.locale.text(ShellText::InspectorLatestResult)).strong(),
            );
            self.render_stream_result_inspector(ui, stream);
        }

        if !detail.related_steps.is_empty() {
            ui.add_space(4.0);
            ui.collapsing(self.locale.text(ShellText::RelatedSolveSteps), |ui| {
                for step in &detail.related_steps {
                    render_wrapped_small(
                        ui,
                        format!(
                            "#{} {} -> {}",
                            step.index,
                            step.unit_id,
                            step.produced_streams.join(", ")
                        ),
                    );
                    render_wrapped_label(ui, &step.summary);
                    ui.add_space(4.0);
                }
            });
        }

        if !detail.related_diagnostics.is_empty() {
            ui.add_space(4.0);
            ui.collapsing(self.locale.text(ShellText::RelatedDiagnostics), |ui| {
                for diagnostic in &detail.related_diagnostics {
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
                    ui.add_space(4.0);
                }
            });
        }
    }

    pub(super) fn render_result_inspector(
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
            }
        });
        if inspector.has_stale_selection {
            render_wrapped_small(ui, self.locale.text(ShellText::StaleStreamSelection));
        }
        ui.separator();

        if let Some(stream) = inspector.selected_stream.as_ref() {
            self.render_stream_result_inspector(ui, stream);
        } else {
            ui.small(self.locale.text(ShellText::NoStreamResults));
            return;
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

        ui.collapsing(self.locale.text(ShellText::RelatedSolveSteps), |ui| {
            if inspector.related_steps.is_empty() {
                ui.small(self.locale.text(ShellText::NoRelatedSteps));
                return;
            }
            for step in &inspector.related_steps {
                render_wrapped_small(
                    ui,
                    format!(
                        "#{} {} -> {}",
                        step.index,
                        step.unit_id,
                        step.produced_streams.join(", ")
                    ),
                );
                render_wrapped_label(ui, &step.summary);
                ui.add_space(4.0);
            }
        });

        ui.collapsing(self.locale.text(ShellText::RelatedDiagnostics), |ui| {
            if inspector.related_diagnostics.is_empty() {
                ui.small(self.locale.text(ShellText::NoRelatedDiagnostics));
                return;
            }
            for diagnostic in &inspector.related_diagnostics {
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
                ui.add_space(4.0);
            }
        });
    }

    fn render_result_inspector_comparison(
        &self,
        ui: &mut egui::Ui,
        comparison: &radishflow_studio::StudioGuiWindowResultInspectorComparisonModel,
    ) {
        ui.add_space(4.0);
        render_wrapped_small(
            ui,
            format!(
                "{}: {}  {}: {}",
                self.locale.text(ShellText::BaseStream),
                comparison.base_stream_id,
                self.locale.text(ShellText::ComparedStream),
                comparison.compared_stream_id
            ),
        );
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
                self.render_small_command_action(ui, &target.action);
            }
        });
    }

    fn render_small_command_action(
        &mut self,
        ui: &mut egui::Ui,
        action: &radishflow_studio::StudioGuiWindowCommandActionModel,
    ) {
        if ui
            .small_button(&action.label)
            .on_hover_text(&action.hover_text)
            .clicked()
        {
            self.dispatch_ui_command(&action.command_id);
        }
    }

    pub(super) fn render_command_menu_bar(
        &mut self,
        ui: &mut egui::Ui,
        menu_tree: &[StudioGuiCommandMenuNode],
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(self.locale.text(ShellText::Menu)).strong());
            for node in menu_tree {
                self.render_command_menu_node(ui, node);
            }
        });
    }

    pub(super) fn render_command_menu_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &StudioGuiCommandMenuNode,
    ) {
        if let Some(command) = node.command.as_ref() {
            if ui
                .add_enabled(command.enabled, egui::Button::new(&command.label))
                .on_hover_text(&command.hover_text)
                .clicked()
            {
                self.dispatch_menu_command(command);
                ui.close_menu();
            }
            return;
        }

        ui.menu_button(&node.label, |ui| {
            for child in &node.children {
                self.render_command_menu_node(ui, child);
            }
        });
    }

    pub(super) fn render_command_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        sections: &[StudioGuiWindowToolbarSectionModel],
    ) {
        ui.label(egui::RichText::new(self.locale.text(ShellText::Toolbar)).strong());
        let mut first_section = true;
        for section in sections {
            if section.items.is_empty() {
                continue;
            }
            if !first_section {
                ui.separator();
            }
            first_section = false;
            ui.label(
                egui::RichText::new(section.title)
                    .small()
                    .color(egui::Color32::from_rgb(92, 104, 117)),
            );
            for command in &section.items {
                let response = ui
                    .add_enabled(command.enabled, egui::Button::new(&command.label))
                    .on_hover_text(&command.hover_text);
                if response.clicked() {
                    self.dispatch_ui_command(&command.command_id);
                }
            }
        }
    }

    pub(super) fn render_command_palette(
        &mut self,
        ctx: &egui::Context,
        commands: &radishflow_studio::StudioGuiWindowCommandAreaModel,
    ) {
        if !self.command_palette.open {
            return;
        }

        let mut open = self.command_palette.open;
        egui::Window::new(self.locale.text(ShellText::CommandPaletteTitle))
            .id(egui::Id::new("studio.command_palette"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 72.0))
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_min_width(560.0);
                let palette_items = commands.palette_items(&self.command_palette.query);
                ui.small(format!(
                    "{} / {} commands",
                    commands.total_command_count.min(palette_items.len()),
                    commands.total_command_count
                ));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.command_palette.query)
                        .hint_text("按 label / menu path / search terms 过滤"),
                );
                if self.command_palette.focus_query_input {
                    response.request_focus();
                    self.command_palette.focus_query_input = false;
                }
                if response.changed() {
                    self.command_palette.selected_index = 0;
                }

                ui.add_space(8.0);
                self.command_palette.sync_selection(&palette_items);

                egui::ScrollArea::vertical()
                    .max_height(320.0)
                    .show(ui, |ui| {
                        if palette_items.is_empty() {
                            ui.small(self.locale.text(ShellText::NoMatchingCommands));
                            return;
                        }

                        for (index, item) in palette_items.iter().enumerate() {
                            let selected = index == self.command_palette.selected_index;
                            let response = ui
                                .add_enabled(
                                    item.enabled,
                                    egui::Button::new(&item.label).selected(selected),
                                )
                                .on_hover_text(&item.hover_text);
                            if selected {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                            if response.hovered() && item.enabled {
                                self.command_palette.selected_index = index;
                            }
                            ui.small(&item.menu_path_text);
                            ui.small(
                                egui::RichText::new(&item.detail)
                                    .color(egui::Color32::from_rgb(92, 104, 117)),
                            );
                            ui.add_space(6.0);

                            if response.clicked() {
                                let command_id = item.command_id.clone();
                                self.dispatch_ui_command(command_id);
                                self.command_palette.close();
                            }
                        }
                    });
            });

        if !open {
            self.command_palette.close();
        }
    }

    pub(super) fn render_panel_toggle(
        &mut self,
        ui: &mut egui::Ui,
        window_id: Option<StudioWindowHostId>,
        layout_state: &radishflow_studio::StudioGuiWindowLayoutState,
        area_id: StudioGuiWindowAreaId,
        label: &str,
    ) {
        let visible = layout_state
            .panel(area_id)
            .map(|panel| panel.visible)
            .unwrap_or(false);
        let mut desired = visible;
        if ui.checkbox(&mut desired, label).changed() {
            self.dispatch_layout_mutation(
                window_id,
                StudioGuiWindowLayoutMutation::SetPanelVisibility {
                    area_id,
                    visible: desired,
                },
            );
        }
    }

    pub(super) fn render_region_weight_slider(
        &mut self,
        ui: &mut egui::Ui,
        window_id: Option<StudioWindowHostId>,
        layout_state: &radishflow_studio::StudioGuiWindowLayoutState,
        dock_region: StudioGuiWindowDockRegion,
        label: &str,
    ) {
        let Some(region_weight) = layout_state.region_weight(dock_region) else {
            return;
        };
        let mut weight = region_weight.weight;
        if ui
            .add(egui::Slider::new(&mut weight, 10..=80).text(label))
            .changed()
        {
            self.dispatch_layout_mutation(
                window_id,
                StudioGuiWindowLayoutMutation::SetRegionWeight {
                    dock_region,
                    weight,
                },
            );
        }
    }

    pub(super) fn render_move_menu(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
        current_region: StudioGuiWindowDockRegion,
    ) {
        ui.menu_button("Move", |ui| {
            for region in [
                StudioGuiWindowDockRegion::LeftSidebar,
                StudioGuiWindowDockRegion::CenterStage,
                StudioGuiWindowDockRegion::RightSidebar,
            ] {
                let label = dock_region_label(region);
                if ui
                    .add_enabled(region != current_region, egui::Button::new(label))
                    .clicked()
                {
                    self.dispatch_layout_mutation(
                        window.layout_state.scope.window_id,
                        StudioGuiWindowLayoutMutation::SetPanelDockRegion {
                            area_id,
                            dock_region: region,
                            order: None,
                        },
                    );
                    ui.close_menu();
                }
            }
        });
    }

    pub(super) fn render_stack_menu(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
        display_mode: StudioGuiWindowPanelDisplayMode,
    ) {
        ui.menu_button("Stack", |ui| {
            for target_area_id in [
                StudioGuiWindowAreaId::Commands,
                StudioGuiWindowAreaId::Canvas,
                StudioGuiWindowAreaId::Runtime,
            ] {
                if target_area_id == area_id {
                    continue;
                }
                let Some(target_panel) = window.layout().panel(target_area_id).cloned() else {
                    continue;
                };
                if !target_panel.visible {
                    continue;
                }

                if ui.button(format!("With {}", target_panel.title)).clicked() {
                    self.dispatch_layout_mutation(
                        window.layout_state.scope.window_id,
                        StudioGuiWindowLayoutMutation::StackPanelWith {
                            area_id,
                            anchor_area_id: target_area_id,
                            placement: radishflow_studio::StudioGuiWindowDockPlacement::Before {
                                anchor_area_id: target_area_id,
                            },
                        },
                    );
                    ui.close_menu();
                }
            }

            if !matches!(display_mode, StudioGuiWindowPanelDisplayMode::Standalone)
                && ui.button("Unstack").clicked()
            {
                self.dispatch_layout_mutation(
                    window.layout_state.scope.window_id,
                    StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
                        area_id,
                        placement: radishflow_studio::StudioGuiWindowDockPlacement::End,
                    },
                );
                ui.close_menu();
            }
        });
    }

    pub(super) fn render_drop_target_lane(
        &mut self,
        ui: &mut egui::Ui,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
        label: &str,
        hovered_drop_target: &mut bool,
    ) {
        let is_active_preview =
            self.active_drop_preview == Some(ActiveDropPreview { window_id, query });
        let lane = egui::Frame::group(ui.style())
            .fill(drop_lane_fill(is_active_preview))
            .stroke(drop_lane_stroke(is_active_preview))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(label).small());
            });
        let response = ui.interact(
            lane.response.rect,
            ui.make_persistent_id(format!("drop-lane:{window_id:?}:{query:?}")),
            egui::Sense::click(),
        );
        self.process_drop_target_response(response, window_id, query, hovered_drop_target);
    }

    pub(super) fn render_floating_drop_preview_overlay(
        &self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
    ) {
        let Some(preview) = window.drop_preview.as_ref() else {
            return;
        };
        let anchor_pos = self
            .drop_preview_overlay_anchor
            .map(|anchor| preferred_overlay_pos(anchor.rect))
            .or_else(|| {
                ctx.pointer_latest_pos()
                    .map(|pointer| pointer + egui::vec2(18.0, 18.0))
            });
        let Some(anchor_pos) = anchor_pos else {
            return;
        };
        let overlay_pos = clamp_overlay_pos(ctx, anchor_pos, egui::vec2(280.0, 110.0));

        egui::Area::new(egui::Id::new("studio.drop_preview.floating_overlay"))
            .order(egui::Order::Foreground)
            .interactable(false)
            .fixed_pos(overlay_pos)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgb(246, 250, 255))
                    .stroke(egui::Stroke::new(
                        1.5,
                        egui::Color32::from_rgb(56, 126, 214),
                    ))
                    .show(ui, |ui| {
                        ui.set_max_width(260.0);
                        ui.label(
                            egui::RichText::new("Drop preview")
                                .strong()
                                .color(egui::Color32::from_rgb(56, 126, 214)),
                        );
                        ui.small(format!(
                            "{} region / stack {}",
                            dock_region_label(preview.overlay.target_dock_region),
                            preview.overlay.target_stack_group
                        ));
                        ui.label(egui::RichText::new(preview_insert_hint(preview)).strong());
                        ui.small(format!(
                            "changed: {}",
                            format_area_id_list(&preview.changed_area_ids)
                        ));
                    });
            });
    }
}

fn entitlement_command_id(action_id: rf_ui::EntitlementActionId) -> &'static str {
    match action_id {
        rf_ui::EntitlementActionId::SyncEntitlement => "entitlement.sync",
        rf_ui::EntitlementActionId::RefreshOfflineLease => "entitlement.refresh_offline_lease",
    }
}

fn paint_canvas_drop_surface(painter: &egui::Painter, rect: egui::Rect, active: bool) {
    let fill = if active {
        egui::Color32::from_rgb(236, 247, 242)
    } else {
        egui::Color32::from_rgb(246, 248, 250)
    };
    let stroke_color = if active {
        egui::Color32::from_rgb(52, 128, 89)
    } else {
        egui::Color32::from_rgb(170, 178, 188)
    };
    painter.rect_filled(rect, 6.0, fill);
    paint_canvas_rect_border(painter, rect, egui::Stroke::new(1.5, stroke_color));
    paint_canvas_grid(
        painter,
        rect.shrink(1.0),
        egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(120, 135, 150, 34),
        ),
    );
}

fn paint_canvas_rect_border(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    painter.line_segment([rect.left_top(), rect.right_top()], stroke);
    painter.line_segment([rect.right_top(), rect.right_bottom()], stroke);
    painter.line_segment([rect.right_bottom(), rect.left_bottom()], stroke);
    painter.line_segment([rect.left_bottom(), rect.left_top()], stroke);
}

fn paint_canvas_grid(painter: &egui::Painter, rect: egui::Rect, stroke: egui::Stroke) {
    let step = 32.0;
    let mut x = rect.left() + step;
    while x < rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            stroke,
        );
        x += step;
    }
    let mut y = rect.top() + step;
    while y < rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            stroke,
        );
        y += step;
    }
}

fn paint_canvas_surface_labels(
    painter: &egui::Painter,
    rect: egui::Rect,
    title: &str,
    subtitle: &str,
) {
    let title_color = egui::Color32::from_rgb(35, 49, 63);
    let subtitle_color = egui::Color32::from_rgb(86, 96, 108);
    let title_galley = painter.layout_no_wrap(
        title.to_owned(),
        egui::FontId::proportional(16.0),
        title_color,
    );
    let subtitle_galley = painter.layout_no_wrap(
        subtitle.to_owned(),
        egui::FontId::proportional(12.0),
        subtitle_color,
    );
    let title_pos = egui::pos2(rect.left() + 16.0, rect.top() + 16.0);
    painter.galley(title_pos, title_galley, title_color);
    painter.galley(
        title_pos + egui::vec2(0.0, 24.0),
        subtitle_galley,
        subtitle_color,
    );
}

#[derive(Debug, Clone, Copy)]
struct CanvasStreamLineGeometry {
    start: egui::Pos2,
    end: egui::Pos2,
}

fn canvas_stream_line_geometry(
    rect: egui::Rect,
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) -> CanvasStreamLineGeometry {
    let source = stream.source.as_ref().map(|endpoint| {
        canvas_unit_port_anchor(
            rect,
            endpoint.layout_slot,
            true,
            endpoint.port_side_index,
            endpoint.port_side_count,
        )
    });
    let sink = stream.sink.as_ref().map(|endpoint| {
        canvas_unit_port_anchor(
            rect,
            endpoint.layout_slot,
            false,
            endpoint.port_side_index,
            endpoint.port_side_count,
        )
    });

    match (source, sink) {
        (Some(start), Some(end)) => CanvasStreamLineGeometry { start, end },
        (Some(start), None) => CanvasStreamLineGeometry {
            start,
            end: egui::pos2((start.x + 88.0).min(rect.right() - 18.0), start.y),
        },
        (None, Some(end)) => CanvasStreamLineGeometry {
            start: egui::pos2((end.x - 88.0).max(rect.left() + 18.0), end.y),
            end,
        },
        (None, None) => {
            let center = rect.center();
            CanvasStreamLineGeometry {
                start: center,
                end: center,
            }
        }
    }
}

fn canvas_stream_line_hit_rect(geometry: CanvasStreamLineGeometry) -> egui::Rect {
    egui::Rect::from_two_pos(geometry.start, geometry.end).expand(8.0)
}

fn paint_canvas_stream_line(
    painter: &egui::Painter,
    geometry: CanvasStreamLineGeometry,
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) {
    let color = if stream.is_active_inspector_target {
        egui::Color32::from_rgb(32, 102, 176)
    } else {
        egui::Color32::from_rgb(42, 142, 122)
    };
    let stroke = egui::Stroke::new(
        if stream.is_active_inspector_target {
            2.4
        } else {
            1.6
        },
        color,
    );
    if stream.is_active_inspector_target {
        painter.line_segment(
            [geometry.start, geometry.end],
            egui::Stroke::new(6.0, egui::Color32::from_rgba_unmultiplied(48, 112, 188, 42)),
        );
    }
    painter.line_segment([geometry.start, geometry.end], stroke);
    painter.circle_filled(geometry.start, 3.5, color);
    paint_canvas_stream_arrow(painter, geometry, color);
}

fn paint_canvas_viewport_stream_focus(painter: &egui::Painter, geometry: CanvasStreamLineGeometry) {
    painter.line_segment(
        [geometry.start, geometry.end],
        egui::Stroke::new(8.0, egui::Color32::from_rgba_unmultiplied(210, 128, 38, 54)),
    );
    painter.circle_stroke(
        geometry.start.lerp(geometry.end, 0.5),
        13.0,
        egui::Stroke::new(2.0, egui::Color32::from_rgb(210, 128, 38)),
    );
}

fn paint_canvas_stream_status_badges(
    painter: &egui::Painter,
    geometry: CanvasStreamLineGeometry,
    badges: &[radishflow_studio::StudioGuiCanvasStatusBadgeViewModel],
) {
    if badges.is_empty() {
        return;
    }

    let anchor = geometry.start.lerp(geometry.end, 0.5) + egui::vec2(0.0, -16.0);
    paint_canvas_status_badges(painter, anchor, badges);
}

fn paint_canvas_stream_arrow(
    painter: &egui::Painter,
    geometry: CanvasStreamLineGeometry,
    color: egui::Color32,
) {
    let delta = geometry.end - geometry.start;
    let length = delta.length();
    if length <= 1.0 {
        return;
    }

    let direction = delta / length;
    let normal = egui::vec2(-direction.y, direction.x);
    let back = geometry.end - direction * 10.0;
    painter.line_segment(
        [geometry.end, back + normal * 4.5],
        egui::Stroke::new(1.6, color),
    );
    painter.line_segment(
        [geometry.end, back - normal * 4.5],
        egui::Stroke::new(1.6, color),
    );
}

fn canvas_unit_block_rect(rect: egui::Rect, layout_slot: usize) -> egui::Rect {
    let block_size = egui::vec2(156.0, 72.0);
    let gap = egui::vec2(22.0, 20.0);
    let left_padding = 18.0;
    let top_padding = 72.0;
    let available_width = (rect.width() - left_padding * 2.0).max(block_size.x);
    let columns = ((available_width + gap.x) / (block_size.x + gap.x))
        .floor()
        .max(1.0) as usize;
    let column = layout_slot % columns;
    let row = layout_slot / columns;
    let min = egui::pos2(
        rect.left() + left_padding + column as f32 * (block_size.x + gap.x),
        rect.top() + top_padding + row as f32 * (block_size.y + gap.y),
    );
    egui::Rect::from_min_size(min, block_size)
}

fn canvas_unit_viewport_anchor_label(layout_slot: usize) -> String {
    format!("unit-slot-{layout_slot}")
}

fn canvas_unit_port_anchor(
    rect: egui::Rect,
    layout_slot: usize,
    is_outlet: bool,
    side_index: usize,
    side_count: usize,
) -> egui::Pos2 {
    let unit_rect = canvas_unit_block_rect(rect, layout_slot);
    canvas_unit_port_anchor_in_rect(unit_rect, is_outlet, side_index, side_count)
}

fn canvas_unit_port_anchor_in_rect(
    unit_rect: egui::Rect,
    is_outlet: bool,
    side_index: usize,
    side_count: usize,
) -> egui::Pos2 {
    let count = side_count.max(1) as f32;
    let y_min = unit_rect.top() + 17.0;
    let y_max = unit_rect.bottom() - 17.0;
    let ratio = (side_index as f32 + 1.0) / (count + 1.0);
    let y = egui::lerp(y_min..=y_max, ratio);
    if is_outlet {
        egui::pos2(unit_rect.right(), y)
    } else {
        egui::pos2(unit_rect.left(), y)
    }
}

fn paint_canvas_unit_block(
    painter: &egui::Painter,
    rect: egui::Rect,
    unit: &radishflow_studio::StudioGuiCanvasUnitBlockViewModel,
) {
    let fill = if unit.is_active_inspector_target {
        egui::Color32::from_rgb(230, 243, 255)
    } else {
        egui::Color32::from_rgb(255, 255, 255)
    };
    let stroke = if unit.is_active_inspector_target {
        egui::Stroke::new(2.0, egui::Color32::from_rgb(48, 112, 188))
    } else {
        egui::Stroke::new(1.2, egui::Color32::from_rgb(98, 113, 126))
    };
    if unit.is_active_inspector_target {
        paint_canvas_rect_border(
            painter,
            rect.expand(4.0),
            egui::Stroke::new(3.0, egui::Color32::from_rgba_unmultiplied(48, 112, 188, 50)),
        );
    }
    painter.rect_filled(rect, 6.0, fill);
    paint_canvas_rect_border(painter, rect, stroke);

    let accent = match unit.kind.as_str() {
        "feed" | "Feed" => egui::Color32::from_rgb(48, 132, 98),
        "heater" | "Heater" => egui::Color32::from_rgb(190, 112, 42),
        "valve" | "Valve" => egui::Color32::from_rgb(96, 96, 150),
        "flash_drum" | "Flash Drum" => egui::Color32::from_rgb(48, 112, 188),
        "mixer" | "Mixer" => egui::Color32::from_rgb(132, 86, 150),
        _ => egui::Color32::from_rgb(86, 96, 108),
    };
    painter.rect_filled(
        egui::Rect::from_min_size(rect.min, egui::vec2(6.0, rect.height())),
        0.0,
        accent,
    );
    for port in &unit.ports {
        paint_canvas_unit_port_marker(painter, rect, port);
    }

    let text_left = rect.left() + 22.0;
    let text_top = rect.top() + 10.0;
    painter.text(
        egui::pos2(text_left, text_top),
        egui::Align2::LEFT_TOP,
        truncate_canvas_label(&unit.name, 18),
        egui::FontId::proportional(14.0),
        egui::Color32::from_rgb(35, 49, 63),
    );
    painter.text(
        egui::pos2(text_left, text_top + 21.0),
        egui::Align2::LEFT_TOP,
        truncate_canvas_label(&unit.kind, 20),
        egui::FontId::proportional(12.0),
        egui::Color32::from_rgb(86, 96, 108),
    );
    painter.text(
        egui::pos2(text_left, text_top + 43.0),
        egui::Align2::LEFT_TOP,
        format!(
            "{} | ports {}/{}",
            unit.unit_id, unit.connected_port_count, unit.port_count
        ),
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(86, 96, 108),
    );
}

fn paint_canvas_viewport_unit_focus(painter: &egui::Painter, rect: egui::Rect) {
    let focus_rect = rect.expand(8.0);
    painter.rect_filled(
        focus_rect,
        7.0,
        egui::Color32::from_rgba_unmultiplied(210, 128, 38, 28),
    );
    paint_canvas_rect_border(
        painter,
        focus_rect,
        egui::Stroke::new(2.4, egui::Color32::from_rgb(210, 128, 38)),
    );
}

fn paint_canvas_unit_status_badges(
    painter: &egui::Painter,
    rect: egui::Rect,
    badges: &[radishflow_studio::StudioGuiCanvasStatusBadgeViewModel],
) {
    if badges.is_empty() {
        return;
    }

    paint_canvas_status_badges(painter, rect.right_top() + egui::vec2(-8.0, 8.0), badges);
}

fn paint_canvas_status_badges(
    painter: &egui::Painter,
    anchor: egui::Pos2,
    badges: &[radishflow_studio::StudioGuiCanvasStatusBadgeViewModel],
) {
    let mut right = anchor.x;
    for badge in badges.iter().rev() {
        let width = 16.0 + badge.short_label.chars().count() as f32 * 6.0;
        let rect =
            egui::Rect::from_min_size(egui::pos2(right - width, anchor.y), egui::vec2(width, 18.0));
        painter.rect_filled(rect, 4.0, canvas_status_badge_color(badge.severity_label));
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &badge.short_label,
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
        right = rect.left() - 4.0;
    }
}

fn paint_canvas_unit_port_marker(
    painter: &egui::Painter,
    rect: egui::Rect,
    port: &radishflow_studio::StudioGuiCanvasUnitPortViewModel,
) {
    let is_outlet = port.direction_label == "outlet";
    let anchor = canvas_unit_port_anchor_in_rect(rect, is_outlet, port.side_index, port.side_count);
    let fill = if port.is_connected {
        egui::Color32::from_rgb(42, 142, 122)
    } else {
        egui::Color32::from_rgb(174, 184, 194)
    };
    let stroke = if port.is_connected {
        egui::Stroke::new(1.1, egui::Color32::from_rgb(34, 92, 82))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(112, 124, 136))
    };
    painter.circle_filled(anchor, 4.2, fill);
    painter.circle_stroke(anchor, 4.2, stroke);

    let label_pos = if is_outlet {
        anchor + egui::vec2(-8.0, -5.5)
    } else {
        anchor + egui::vec2(8.0, -5.5)
    };
    let align = if is_outlet {
        egui::Align2::RIGHT_TOP
    } else {
        egui::Align2::LEFT_TOP
    };
    painter.text(
        label_pos,
        align,
        truncate_canvas_label(&port.name, 9),
        egui::FontId::proportional(9.5),
        if port.is_connected {
            egui::Color32::from_rgb(49, 71, 84)
        } else {
            egui::Color32::from_rgb(106, 118, 130)
        },
    );
}

fn paint_canvas_port_hover_callout(
    painter: &egui::Painter,
    canvas_rect: egui::Rect,
    anchor: egui::Pos2,
    port: &radishflow_studio::StudioGuiCanvasUnitPortViewModel,
) {
    let color = if port.is_connected {
        egui::Color32::from_rgb(42, 142, 122)
    } else {
        egui::Color32::from_rgb(112, 124, 136)
    };
    let size = egui::vec2(188.0, 46.0);
    let mut min = if port.direction_label == "outlet" {
        anchor + egui::vec2(-size.x - 12.0, -22.0)
    } else {
        anchor + egui::vec2(12.0, -22.0)
    };
    min.x = min
        .x
        .clamp(canvas_rect.left() + 8.0, canvas_rect.right() - size.x - 8.0);
    min.y = min
        .y
        .clamp(canvas_rect.top() + 8.0, canvas_rect.bottom() - size.y - 8.0);
    let callout_rect = egui::Rect::from_min_size(min, size);
    let connector_end = if port.direction_label == "outlet" {
        egui::pos2(callout_rect.right(), callout_rect.center().y)
    } else {
        egui::pos2(callout_rect.left(), callout_rect.center().y)
    };

    painter.line_segment(
        [anchor, connector_end],
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(38, 50, 62, 120)),
    );
    painter.rect_filled(
        callout_rect.translate(egui::vec2(0.0, 2.0)),
        5.0,
        egui::Color32::from_rgba_unmultiplied(30, 42, 54, 26),
    );
    painter.rect_filled(callout_rect, 5.0, egui::Color32::from_rgb(255, 255, 255));
    paint_canvas_rect_border(painter, callout_rect, egui::Stroke::new(1.2, color));
    painter.text(
        callout_rect.left_top() + egui::vec2(9.0, 7.0),
        egui::Align2::LEFT_TOP,
        truncate_canvas_label(&format!("{} · {}", port.direction_label, port.name), 24),
        egui::FontId::proportional(11.5),
        egui::Color32::from_rgb(35, 49, 63),
    );
    painter.text(
        callout_rect.left_top() + egui::vec2(9.0, 25.0),
        egui::Align2::LEFT_TOP,
        truncate_canvas_label(&port.binding_label, 28),
        egui::FontId::proportional(10.5),
        egui::Color32::from_rgb(86, 96, 108),
    );
}

fn canvas_focus_callout_anchor(
    rect: egui::Rect,
    callout: &radishflow_studio::StudioGuiCanvasFocusCalloutViewModel,
    unit_blocks: &[radishflow_studio::StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[radishflow_studio::StudioGuiCanvasStreamLineViewModel],
) -> Option<egui::Pos2> {
    if callout.kind_label == "Unit" {
        return unit_blocks
            .iter()
            .find(|unit| unit.unit_id == callout.target_id)
            .map(|unit| canvas_unit_block_rect(rect, unit.layout_slot).right_top());
    }

    stream_lines
        .iter()
        .find(|stream| stream.stream_id == callout.target_id)
        .map(|stream| {
            let geometry = canvas_stream_line_geometry(rect, stream);
            geometry.start.lerp(geometry.end, 0.58)
        })
}

fn paint_canvas_focus_callout(
    painter: &egui::Painter,
    canvas_rect: egui::Rect,
    anchor: egui::Pos2,
    callout: &radishflow_studio::StudioGuiCanvasFocusCalloutViewModel,
) {
    let color = if callout.kind_label == "Stream" {
        egui::Color32::from_rgb(42, 142, 122)
    } else {
        egui::Color32::from_rgb(48, 112, 188)
    };
    let size = egui::vec2(196.0, 54.0);
    let mut min = anchor + egui::vec2(14.0, -62.0);
    min.x = min.x.clamp(
        canvas_rect.left() + 10.0,
        canvas_rect.right() - size.x - 10.0,
    );
    min.y = min.y.clamp(
        canvas_rect.top() + 10.0,
        canvas_rect.bottom() - size.y - 10.0,
    );
    let callout_rect = egui::Rect::from_min_size(min, size);
    let connector_end = egui::pos2(
        callout_rect.left() + 18.0,
        callout_rect.top() + callout_rect.height() * 0.5,
    );

    painter.line_segment(
        [anchor, connector_end],
        egui::Stroke::new(1.2, egui::Color32::from_rgba_unmultiplied(38, 50, 62, 130)),
    );
    painter.rect_filled(
        callout_rect.translate(egui::vec2(0.0, 2.0)),
        6.0,
        egui::Color32::from_rgba_unmultiplied(30, 42, 54, 32),
    );
    painter.rect_filled(callout_rect, 6.0, egui::Color32::from_rgb(255, 255, 255));
    paint_canvas_rect_border(painter, callout_rect, egui::Stroke::new(1.4, color));
    painter.rect_filled(
        egui::Rect::from_min_size(callout_rect.min, egui::vec2(5.0, callout_rect.height())),
        0.0,
        color,
    );
    painter.text(
        callout_rect.left_top() + egui::vec2(13.0, 8.0),
        egui::Align2::LEFT_TOP,
        truncate_canvas_label(&format!("{} · {}", callout.kind_label, callout.title), 24),
        egui::FontId::proportional(13.0),
        egui::Color32::from_rgb(35, 49, 63),
    );
    painter.text(
        callout_rect.left_top() + egui::vec2(13.0, 30.0),
        egui::Align2::LEFT_TOP,
        truncate_canvas_label(&callout.detail, 30),
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(86, 96, 108),
    );
}

fn truncate_canvas_label(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let mut label = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        label.push_str("...");
    }
    label
}

fn canvas_status_badge_color(severity_label: &str) -> egui::Color32 {
    match severity_label {
        "Error" => egui::Color32::from_rgb(180, 40, 40),
        "Warning" => egui::Color32::from_rgb(180, 120, 20),
        _ => egui::Color32::from_rgb(86, 96, 108),
    }
}

fn canvas_legend_swatch_color(swatch_label: &str) -> egui::Color32 {
    match swatch_label {
        "run_status" => egui::Color32::from_rgb(86, 118, 168),
        "attention" => notice_color(rf_ui::RunPanelNoticeLevel::Warning),
        "port" => egui::Color32::from_rgb(42, 142, 122),
        "stream" => egui::Color32::from_rgb(42, 142, 122),
        "pending_edit" => egui::Color32::from_rgb(52, 128, 89),
        _ => egui::Color32::from_rgb(86, 96, 108),
    }
}
