use super::super::*;

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn render_canvas_area(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
    ) {
        let widget = &window.canvas.widget;
        self.render_canvas_toolbar(ui, widget);
        ui.add_space(4.0);
        self.render_canvas_stage_summary(ui, widget);
        self.render_canvas_object_action_strip(ui, widget);
        self.render_canvas_legend(ui, widget);
        ui.separator();
        self.render_canvas_drop_surface(ui, widget);
        ui.add_space(8.0);
        self.render_canvas_suggestions(ui, widget, window, area_id);
    }

    fn render_canvas_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.small(
                egui::RichText::new(self.locale.runtime_label("Canvas tools").as_ref()).strong(),
            );
            ui.separator();
            ui.small(
                egui::RichText::new(self.locale.text(ShellText::ViewOptions))
                    .color(egui::Color32::from_rgb(92, 104, 117)),
            );
            if ui
                .small_button(self.locale.text(ShellText::FitToContent))
                .on_hover_text(self.locale.text(ShellText::FitToContentDetail))
                .clicked()
            {
                self.request_canvas_viewport_fit_to_content();
            }
            if !widget.view().suggestions.is_empty() {
                ui.separator();
                self.render_canvas_toolbar_group(ui, widget, "Suggestion", |action| {
                    matches!(
                        action.id,
                        radishflow_studio::StudioGuiCanvasActionId::AcceptFocused
                            | radishflow_studio::StudioGuiCanvasActionId::RejectFocused
                            | radishflow_studio::StudioGuiCanvasActionId::FocusNext
                            | radishflow_studio::StudioGuiCanvasActionId::FocusPrevious
                    )
                });
            }
        });
    }

    fn render_canvas_toolbar_group(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
        title: &str,
        filter: impl Fn(&radishflow_studio::StudioGuiCanvasRenderableAction) -> bool,
    ) {
        ui.small(
            egui::RichText::new(self.locale.runtime_label(title).as_ref())
                .color(egui::Color32::from_rgb(92, 104, 117)),
        );
        for action in widget.actions.iter().filter(|action| filter(action)) {
            self.render_canvas_toolbar_action(ui, widget, action);
        }
    }

    fn render_canvas_toolbar_action(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
        action: &radishflow_studio::StudioGuiCanvasRenderableAction,
    ) {
        let label = match action.shortcut.as_ref() {
            Some(shortcut) => format!(
                "{} ({})",
                self.locale.runtime_label(&action.label),
                format_shortcut(shortcut)
            ),
            None => self.locale.runtime_label(&action.label).into_owned(),
        };
        if ui
            .add_enabled(action.enabled, egui::Button::new(label))
            .on_hover_text(self.locale.runtime_label(&action.detail).as_ref())
            .clicked()
        {
            match widget.activate(action.id) {
                radishflow_studio::StudioGuiCanvasWidgetEvent::Requested { event, .. } => {
                    self.dispatch_event(event)
                }
                radishflow_studio::StudioGuiCanvasWidgetEvent::Disabled { .. }
                | radishflow_studio::StudioGuiCanvasWidgetEvent::Missing { .. }
                | radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionRequested { .. }
                | radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionDisabled { .. }
                | radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionMissing { .. } => {}
            }
        }
    }

    fn render_canvas_stage_summary(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
    ) {
        let view = widget.view();
        let object_list = &view.object_list;
        let viewport = &view.viewport;
        ui.horizontal_wrapped(|ui| {
            ui.small(
                egui::RichText::new(self.locale.runtime_label("Canvas status").as_ref()).strong(),
            );
            render_status_chip(
                ui,
                &self
                    .locale
                    .count_label(object_list.unit_count, "unit", "units"),
                egui::Color32::from_rgb(86, 118, 168),
            );
            render_status_chip(
                ui,
                &compact_canvas_material_line_count(object_list.stream_count, self.locale),
                egui::Color32::from_rgb(42, 142, 122),
            );
            render_status_chip(
                ui,
                &self
                    .locale
                    .count_label(view.suggestions.len(), "suggestion", "suggestions"),
                egui::Color32::from_rgb(86, 96, 108),
            );
            if object_list.attention_count > 0 {
                render_status_chip(
                    ui,
                    &self
                        .locale
                        .count_label(object_list.attention_count, "attention", "attention"),
                    notice_color(rf_ui::RunPanelNoticeLevel::Warning),
                );
            }
            if let Some(status) = view.run_status.as_ref() {
                render_status_chip(
                    ui,
                    self.locale.runtime_label(status.status_label).as_ref(),
                    run_status_color(status.status_label),
                );
            }
            render_status_chip(
                ui,
                self.locale.runtime_label(viewport.mode_label).as_ref(),
                egui::Color32::from_rgb(86, 118, 168),
            );
            render_status_chip(
                ui,
                self.locale.runtime_label(viewport.layout_label).as_ref(),
                egui::Color32::from_rgb(86, 96, 108),
            );
        });
        if let Some(result) = self.canvas_command_result.as_ref() {
            ui.colored_label(notice_color(result.level), &result.title);
            render_wrapped_small(ui, &result.detail);
        }
    }

    fn render_canvas_suggestions(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
    ) {
        if widget.view().suggestions.is_empty() {
            return;
        }

        egui::CollapsingHeader::new(self.locale.text(ShellText::Suggestions))
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(format!(
                        "scroll:{}:{}:suggestions",
                        window.layout_state.scope.layout_key,
                        area_label(area_id)
                    ))
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for suggestion in &widget.view().suggestions {
                            let frame = egui::Frame::group(ui.style());
                            frame.show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal_wrapped(|ui| {
                                    let focus = if suggestion.is_focused {
                                        "Focused"
                                    } else {
                                        "Suggestion"
                                    };
                                    ui.label(
                                        egui::RichText::new(
                                            self.locale.runtime_label(focus).as_ref(),
                                        )
                                        .strong(),
                                    );
                                    render_status_chip(
                                        ui,
                                        &format!("{:.0}%", suggestion.confidence * 100.0),
                                        egui::Color32::from_rgb(86, 118, 168),
                                    );
                                    render_status_chip(
                                        ui,
                                        self.locale.runtime_label(suggestion.status_label).as_ref(),
                                        egui::Color32::from_rgb(86, 96, 108),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add_enabled(
                                                    suggestion.explicit_accept_enabled,
                                                    egui::Button::new(
                                                        self.locale
                                                            .runtime_label(suggestion.action_label)
                                                            .as_ref(),
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                self.dispatch_canvas_suggestion(
                                                    widget,
                                                    &suggestion.id,
                                                );
                                            }
                                        },
                                    );
                                });
                                render_wrapped_small(ui, &suggestion.reason);
                            });
                            ui.add_space(6.0);
                        }
                    });
            });
    }

    fn dispatch_canvas_suggestion(
        &mut self,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
        suggestion_id: &str,
    ) {
        match widget.activate_suggestion(suggestion_id) {
            radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionRequested {
                event, ..
            } => self.dispatch_event(event),
            radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionDisabled { .. }
            | radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionMissing { .. }
            | radishflow_studio::StudioGuiCanvasWidgetEvent::Requested { .. }
            | radishflow_studio::StudioGuiCanvasWidgetEvent::Disabled { .. }
            | radishflow_studio::StudioGuiCanvasWidgetEvent::Missing { .. } => {}
        }
    }

    fn render_canvas_object_action_strip(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
    ) {
        let selection = widget.view().current_selection.as_ref();
        let suggestion = selection.is_none().then(|| {
            widget
                .view()
                .suggestions
                .iter()
                .find(|item| item.is_focused)
        });
        let suggestion = suggestion.flatten();
        if selection.is_none() && suggestion.is_none() {
            return;
        }

        ui.horizontal_wrapped(|ui| {
            ui.small(
                egui::RichText::new(self.locale.runtime_label("Canvas actions").as_ref()).strong(),
            );
            if let Some(selection) = selection {
                render_status_chip(
                    ui,
                    self.locale.runtime_label(selection.kind_label).as_ref(),
                    egui::Color32::from_rgb(48, 112, 188),
                );
                ui.small(&selection.target_id);
                if ui
                    .small_button(self.locale.text(ShellText::Focus))
                    .on_hover_text(self.locale.runtime_label("Focus selected object").as_ref())
                    .clicked()
                {
                    self.dispatch_ui_command(&selection.command_id);
                }
                if selection.kind_label == "Unit" {
                    for direction in
                        radishflow_studio::StudioGuiCanvasUnitLayoutNudgeDirection::all()
                    {
                        if let Some(action) = widget.action(
                            radishflow_studio::StudioGuiCanvasActionId::MoveSelectedUnit(
                                *direction,
                            ),
                        ) {
                            if ui
                                .add_enabled(
                                    action.enabled,
                                    egui::Button::new(
                                        self.locale.runtime_label(&action.label).as_ref(),
                                    ),
                                )
                                .on_hover_text(self.locale.runtime_label(&action.detail).as_ref())
                                .clicked()
                            {
                                self.dispatch_ui_command(&action.command_id);
                            }
                        }
                    }
                } else if selection.kind_label == "Stream" {
                    for action_id in [
                        radishflow_studio::StudioGuiCanvasActionId::DisconnectSelectedStream,
                        radishflow_studio::StudioGuiCanvasActionId::DisconnectSelectedStreamSource,
                        radishflow_studio::StudioGuiCanvasActionId::DisconnectSelectedStreamSink,
                        radishflow_studio::StudioGuiCanvasActionId::ReconnectSelectedStream,
                        radishflow_studio::StudioGuiCanvasActionId::DeleteSelectedStream,
                    ] {
                        if let Some(action) = widget.action(action_id) {
                            if ui
                                .add_enabled(
                                    action.enabled,
                                    egui::Button::new(
                                        self.locale.runtime_label(&action.label).as_ref(),
                                    ),
                                )
                                .on_hover_text(self.locale.runtime_label(&action.detail).as_ref())
                                .clicked()
                            {
                                self.dispatch_ui_command(&action.command_id);
                            }
                        }
                    }
                }
            } else if let Some(suggestion) = suggestion {
                render_status_chip(
                    ui,
                    self.locale.runtime_label("Suggestion").as_ref(),
                    egui::Color32::from_rgb(86, 118, 168),
                );
                if ui
                    .add_enabled(
                        suggestion.explicit_accept_enabled,
                        egui::Button::new(
                            self.locale.runtime_label(suggestion.action_label).as_ref(),
                        ),
                    )
                    .on_hover_text(
                        self.locale
                            .runtime_label("Apply the focused canvas suggestion")
                            .as_ref(),
                    )
                    .clicked()
                {
                    self.dispatch_canvas_suggestion(widget, &suggestion.id);
                }
            }
        });
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
            ui.small(
                egui::RichText::new(self.locale.runtime_label(legend.title).as_ref()).strong(),
            );
            for item in &legend.items {
                let color = canvas_legend_swatch_color(item.swatch_label);
                let label = compact_canvas_legend_item_label(item, self.locale);
                render_canvas_chip_with_hover(ui, &label, color, &item.detail);
            }
        });
    }

    fn render_canvas_drop_surface(
        &mut self,
        ui: &mut egui::Ui,
        widget: &radishflow_studio::StudioGuiCanvasWidgetModel,
    ) {
        let view = widget.view();
        let pending_edit = view.pending_edit.as_ref();
        let focus_callout = view.focus_callout.as_ref();
        let unit_blocks = &view.unit_blocks;
        let stream_lines = &view.stream_lines;
        self.reconcile_canvas_viewport_navigation(view.viewport.focus.as_ref());
        let available_width = ui.available_width().max(320.0);
        let desired_size = egui::vec2(available_width, 280.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());
        let viewport_transform = if self.canvas_viewport_fit_to_content_requested {
            CanvasViewportTransform {
                offset: self.fit_canvas_viewport_to_content(rect, unit_blocks, stream_lines),
            }
        } else {
            canvas_initial_viewport_transform(
                &mut self.canvas_initial_viewport_fit,
                rect,
                unit_blocks,
                stream_lines,
            )
        };
        let current_viewport_offset = viewport_transform.offset;
        let painter = ui.painter_at(rect);
        paint_canvas_drop_surface(&painter, rect, pending_edit.is_some());

        if pending_edit.is_some() || (unit_blocks.is_empty() && stream_lines.is_empty()) {
            let title = pending_edit
                .map(|pending| pending.summary.as_str())
                .unwrap_or(self.locale.text(ShellText::CanvasToolPrompt));
            let subtitle = if pending_edit.is_some() {
                self.locale.text(ShellText::CanvasPlacePrompt)
            } else {
                self.locale.text(ShellText::CanvasEditPrompt)
            };
            paint_canvas_surface_labels(&painter, rect, title, subtitle);
        }

        let mut clicked_stream_command = None;
        let mut hovered_stream = false;
        for stream in stream_lines {
            let geometry = canvas_stream_line_geometry(rect, &viewport_transform, stream);
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
            paint_canvas_stream_label(&painter, rect, geometry, stream);
            paint_canvas_stream_status_badges(&painter, geometry, &stream.status_badges);
            let stream_response = ui
                .interact(
                    canvas_stream_line_hit_rect(geometry),
                    ui.make_persistent_id(format!("canvas-stream:{}", stream.line_id)),
                    egui::Sense::click(),
                )
                .on_hover_text(&stream.hover_text)
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            hovered_stream |= stream_response.hovered();
            if stream_response.clicked() {
                clicked_stream_command = Some(stream.command_id.clone());
            }
        }

        let mut clicked_unit = false;
        let mut hovered_unit = false;
        let mut clicked_port_command = None;
        let mut hovered_port_callout = None;
        let mut completed_unit_drag = None;
        for unit in unit_blocks {
            let drag_preview_position = self
                .canvas_unit_drag
                .as_ref()
                .filter(|drag| drag.unit_id == unit.unit_id)
                .map(|drag| drag.current_position);
            let unit_rect = canvas_unit_block_rect(
                rect,
                &viewport_transform,
                unit.layout_slot,
                drag_preview_position.or(unit.layout_position),
            );
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
                        if port.stream_command_id.is_some() {
                            egui::Sense::click()
                        } else {
                            egui::Sense::hover()
                        },
                    )
                    .on_hover_text(&port.hover_text)
                    .on_hover_cursor(if port.stream_command_id.is_some() {
                        egui::CursorIcon::PointingHand
                    } else {
                        egui::CursorIcon::Default
                    });
                if port_response.hovered() {
                    hovered_port_callout = Some((port_anchor, port));
                }
                if port_response.clicked() {
                    clicked_port_command = port.stream_command_id.clone();
                }
            }
            let unit_response = ui
                .interact(
                    unit_rect,
                    ui.make_persistent_id(format!("canvas-unit:{}", unit.unit_id)),
                    egui::Sense::click_and_drag(),
                )
                .on_hover_text(&unit.hover_text)
                .on_hover_cursor(egui::CursorIcon::Grab);
            hovered_unit |= unit_response.hovered();
            if unit_response.drag_started()
                && clicked_port_command.is_none()
                && hovered_port_callout.is_none()
            {
                self.canvas_viewport_drag = None;
                let start_rect = canvas_unit_block_world_rect(
                    rect.width(),
                    unit.layout_slot,
                    unit.layout_position,
                );
                let start_position = start_rect.min;
                let pointer_offset = unit_response
                    .interact_pointer_pos()
                    .map(|pointer_pos| {
                        viewport_transform.screen_to_world(rect, pointer_pos) - start_position
                    })
                    .unwrap_or(egui::Vec2::ZERO);
                self.canvas_unit_drag = Some(CanvasUnitDragState {
                    unit_id: unit.unit_id.clone(),
                    start_position: rf_ui::CanvasPoint::new(
                        start_position.x as f64,
                        start_position.y as f64,
                    ),
                    pointer_offset,
                    current_position: rf_ui::CanvasPoint::new(
                        start_position.x as f64,
                        start_position.y as f64,
                    ),
                });
            }
            if unit_response.dragged() {
                if let Some(drag) = self
                    .canvas_unit_drag
                    .as_mut()
                    .filter(|drag| drag.unit_id == unit.unit_id)
                {
                    drag.current_position = canvas_unit_drag_position(
                        rect,
                        &viewport_transform,
                        drag,
                        unit_response.interact_pointer_pos(),
                        unit_response.drag_delta(),
                    );
                }
            }
            if self
                .canvas_unit_drag
                .as_ref()
                .is_some_and(|drag| drag.unit_id == unit.unit_id)
                && !ui.input(|input| input.pointer.primary_down())
            {
                if let Some(drag) = self
                    .canvas_unit_drag
                    .as_mut()
                    .filter(|drag| drag.unit_id == unit.unit_id)
                {
                    drag.current_position = canvas_unit_drag_position(
                        rect,
                        &viewport_transform,
                        drag,
                        unit_response.interact_pointer_pos(),
                        unit_response.drag_delta(),
                    );
                }
                completed_unit_drag = self.canvas_unit_drag.take();
            }
            if unit_response.clicked() && clicked_port_command.is_none() {
                clicked_unit = true;
                self.dispatch_ui_command(&unit.command_id);
            }
        }

        if let Some(callout) = focus_callout {
            if let Some(anchor) = canvas_focus_callout_anchor(
                rect,
                &viewport_transform,
                callout,
                unit_blocks,
                stream_lines,
            ) {
                paint_canvas_focus_callout(&painter, rect, anchor, callout);
            }
        }
        if let Some((anchor, port)) = hovered_port_callout {
            paint_canvas_port_hover_callout(&painter, rect, anchor, port);
        }

        let clicked_port = clicked_port_command.is_some();
        if let Some(command_id) = clicked_port_command {
            self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
            self.dispatch_ui_command(command_id);
        }

        let clicked_stream = clicked_stream_command.is_some();
        if !clicked_unit && !clicked_port {
            if let Some(command_id) = clicked_stream_command {
                self.dispatch_ui_command(command_id);
            }
        }

        let hovered_port = hovered_port_callout.is_some();
        let can_pan_viewport = canvas_can_pan_viewport(
            pending_edit.is_some(),
            hovered_stream,
            hovered_unit,
            hovered_port,
            self.canvas_unit_drag.is_some(),
        );
        if can_pan_viewport && response.drag_started() {
            self.canvas_viewport_drag = Some(CanvasViewportDragState {
                start_offset: current_viewport_offset,
            });
        }
        if can_pan_viewport && response.dragged() {
            let start_offset = self
                .canvas_viewport_drag
                .map(|drag| drag.start_offset)
                .unwrap_or(current_viewport_offset);
            self.update_canvas_viewport_offset(start_offset + response.drag_delta());
        }
        if self.canvas_viewport_drag.is_some() && !ui.input(|input| input.pointer.primary_down()) {
            self.canvas_viewport_drag = None;
        }
        if let Some(drag) = completed_unit_drag {
            self.dispatch_canvas_unit_layout_move(
                rf_types::UnitId::new(drag.unit_id),
                drag.current_position,
            );
        }

        let response = if pending_edit.is_some() {
            response.on_hover_cursor(egui::CursorIcon::Crosshair)
        } else if can_pan_viewport {
            response.on_hover_cursor(egui::CursorIcon::Grab)
        } else {
            response
        };
        if pending_edit.is_some() && response.clicked() && !clicked_unit && !clicked_stream {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let local = viewport_transform.screen_to_world(rect, pointer_pos);
                self.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(
                    local.x.max(0.0) as f64,
                    local.y.max(0.0) as f64,
                ));
            }
        } else if response.clicked() && !clicked_unit && !clicked_stream && !clicked_port {
            self.clear_canvas_selection();
        }
    }
}

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn fit_canvas_viewport_to_content(
        &mut self,
        rect: egui::Rect,
        unit_blocks: &[radishflow_studio::StudioGuiCanvasUnitBlockViewModel],
        stream_lines: &[radishflow_studio::StudioGuiCanvasStreamLineViewModel],
    ) -> egui::Vec2 {
        let transform = canvas_viewport_transform(rect, unit_blocks, stream_lines);
        self.canvas_viewport_fit_to_content_requested = false;
        self.update_canvas_viewport_offset(transform.offset);
        let result =
            radishflow_studio::StudioGuiCanvasCommandResultViewModel::viewport_fit_to_content(
                transform.offset.x,
                transform.offset.y,
            );
        self.platform_host
            .record_activity_line(result.activity_line.clone());
        self.canvas_command_result = Some(result);
        transform.offset
    }
}

fn canvas_can_pan_viewport(
    pending_edit: bool,
    hovered_stream: bool,
    hovered_unit: bool,
    hovered_port: bool,
    unit_drag_active: bool,
) -> bool {
    !pending_edit && !hovered_stream && !hovered_unit && !hovered_port && !unit_drag_active
}

fn canvas_unit_drag_position(
    rect: egui::Rect,
    viewport_transform: &CanvasViewportTransform,
    drag: &CanvasUnitDragState,
    pointer_pos: Option<egui::Pos2>,
    drag_delta: egui::Vec2,
) -> rf_ui::CanvasPoint {
    if let Some(pointer_pos) = pointer_pos {
        let world = viewport_transform.screen_to_world(rect, pointer_pos) - drag.pointer_offset;
        return rf_ui::CanvasPoint::new(world.x.max(0.0) as f64, world.y.max(0.0) as f64);
    }

    rf_ui::CanvasPoint::new(
        (drag.start_position.x + drag_delta.x as f64).max(0.0),
        (drag.start_position.y + drag_delta.y as f64).max(0.0),
    )
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct CanvasViewportTransform {
    offset: egui::Vec2,
}

impl CanvasViewportTransform {
    const ZERO: Self = Self {
        offset: egui::Vec2::ZERO,
    };

    fn world_rect_to_screen(self, rect: egui::Rect, world_rect: egui::Rect) -> egui::Rect {
        egui::Rect::from_min_size(
            rect.min + self.offset + world_rect.min.to_vec2(),
            world_rect.size(),
        )
    }

    fn world_pos_to_screen(self, rect: egui::Rect, world_pos: egui::Pos2) -> egui::Pos2 {
        rect.min + self.offset + world_pos.to_vec2()
    }

    fn screen_to_world(self, rect: egui::Rect, screen_pos: egui::Pos2) -> egui::Pos2 {
        let world = screen_pos - rect.min - self.offset;
        egui::pos2(world.x, world.y)
    }
}

fn canvas_viewport_transform(
    rect: egui::Rect,
    unit_blocks: &[radishflow_studio::StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[radishflow_studio::StudioGuiCanvasStreamLineViewModel],
) -> CanvasViewportTransform {
    let Some(bounds) = canvas_content_world_bounds(rect.width(), unit_blocks, stream_lines) else {
        return CanvasViewportTransform::ZERO;
    };

    let margin = egui::vec2(26.0, 24.0);
    let available = rect.size();
    let content = bounds.size();
    let mut offset = egui::vec2(-bounds.left(), -bounds.top());

    offset.x += if content.x + margin.x * 2.0 <= available.x {
        (available.x - content.x) * 0.5
    } else {
        margin.x
    };
    offset.y += if content.y + margin.y * 2.0 <= available.y {
        (available.y - content.y) * 0.5
    } else {
        margin.y
    };

    CanvasViewportTransform { offset }
}

fn canvas_initial_viewport_transform(
    state: &mut CanvasInitialViewportFitState,
    rect: egui::Rect,
    unit_blocks: &[radishflow_studio::StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[radishflow_studio::StudioGuiCanvasStreamLineViewModel],
) -> CanvasViewportTransform {
    if state.pending {
        let transform = canvas_viewport_transform(rect, unit_blocks, stream_lines);
        state.offset = transform.offset;
        state.pending = false;
        return transform;
    }

    CanvasViewportTransform {
        offset: state.offset,
    }
}

fn canvas_content_world_bounds(
    viewport_width: f32,
    unit_blocks: &[radishflow_studio::StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[radishflow_studio::StudioGuiCanvasStreamLineViewModel],
) -> Option<egui::Rect> {
    let mut bounds = None;
    for unit in unit_blocks {
        canvas_union_rect(
            &mut bounds,
            canvas_unit_block_world_rect(viewport_width, unit.layout_slot, unit.layout_position),
        );
    }
    for stream in stream_lines {
        let geometry = canvas_stream_line_world_geometry(viewport_width, stream);
        canvas_union_rect(
            &mut bounds,
            egui::Rect::from_two_pos(geometry.start, geometry.end).expand(16.0),
        );
        if let Some(label_rect) = canvas_stream_label_rect(geometry, stream) {
            canvas_union_rect(&mut bounds, label_rect);
        }
    }
    bounds
}

fn canvas_union_rect(bounds: &mut Option<egui::Rect>, rect: egui::Rect) {
    *bounds = Some(match bounds.take() {
        Some(bounds) => bounds.union(rect),
        None => rect,
    });
}

fn canvas_stream_line_geometry(
    rect: egui::Rect,
    viewport_transform: &CanvasViewportTransform,
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) -> CanvasStreamLineGeometry {
    let geometry = canvas_stream_line_world_geometry(rect.width(), stream);
    CanvasStreamLineGeometry {
        start: viewport_transform.world_pos_to_screen(rect, geometry.start),
        end: viewport_transform.world_pos_to_screen(rect, geometry.end),
    }
}

fn canvas_stream_line_world_geometry(
    viewport_width: f32,
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) -> CanvasStreamLineGeometry {
    let source = stream.source.as_ref().map(|endpoint| {
        canvas_unit_port_world_anchor(
            viewport_width,
            endpoint.layout_slot,
            endpoint.layout_position,
            true,
            endpoint.port_side_index,
            endpoint.port_side_count,
        )
    });
    let sink = stream.sink.as_ref().map(|endpoint| {
        canvas_unit_port_world_anchor(
            viewport_width,
            endpoint.layout_slot,
            endpoint.layout_position,
            false,
            endpoint.port_side_index,
            endpoint.port_side_count,
        )
    });

    match (source, sink) {
        (Some(start), Some(end)) => CanvasStreamLineGeometry { start, end },
        (Some(start), None) => CanvasStreamLineGeometry {
            start,
            end: egui::pos2(start.x + 88.0, start.y),
        },
        (None, Some(end)) => CanvasStreamLineGeometry {
            start: egui::pos2(end.x - 88.0, end.y),
            end,
        },
        (None, None) => {
            let center = egui::pos2(viewport_width * 0.5, 140.0);
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
    painter.line_segment(
        [geometry.start, geometry.end],
        egui::Stroke::new(
            4.4,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 210),
        ),
    );
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

fn paint_canvas_stream_label(
    painter: &egui::Painter,
    canvas_rect: egui::Rect,
    geometry: CanvasStreamLineGeometry,
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) {
    let label = canvas_stream_label_text(stream);
    let size = canvas_stream_label_size(&label);
    let Some(label_rect) = canvas_stream_label_rect(geometry, stream) else {
        return;
    };
    let mut min = label_rect.min;
    min.x = min
        .x
        .clamp(canvas_rect.left() + 8.0, canvas_rect.right() - size.x - 8.0);
    min.y = min
        .y
        .clamp(canvas_rect.top() + 8.0, canvas_rect.bottom() - size.y - 8.0);
    let rect = egui::Rect::from_min_size(min, size);
    let color = if stream.is_active_inspector_target {
        egui::Color32::from_rgb(32, 102, 176)
    } else {
        egui::Color32::from_rgb(42, 142, 122)
    };

    painter.rect_filled(
        rect.translate(egui::vec2(0.0, 1.5)),
        4.0,
        egui::Color32::from_rgba_unmultiplied(30, 42, 54, 24),
    );
    painter.rect_filled(
        rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 238),
    );
    paint_canvas_rect_border(
        painter,
        rect,
        egui::Stroke::new(1.0, color.gamma_multiply(0.7)),
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(10.5),
        color,
    );
}

fn canvas_stream_label_rect(
    geometry: CanvasStreamLineGeometry,
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) -> Option<egui::Rect> {
    let label = canvas_stream_label_text(stream);
    let size = canvas_stream_label_size(&label);
    if stream.sink.is_none() {
        let vertical_offset = canvas_terminal_stream_label_vertical_offset(stream);
        let min = egui::pos2(
            geometry.start.x + 14.0,
            geometry.start.y - size.y * 0.5 + vertical_offset,
        );
        return Some(egui::Rect::from_min_size(min, size));
    }

    if !canvas_connected_stream_has_label_room(geometry, size) {
        return None;
    }

    let center = canvas_stream_label_center(geometry);
    Some(egui::Rect::from_center_size(center, size))
}

fn canvas_connected_stream_has_label_room(
    geometry: CanvasStreamLineGeometry,
    label_size: egui::Vec2,
) -> bool {
    let line_length = (geometry.end - geometry.start).length();
    line_length >= (label_size.x + 24.0).max(86.0)
}

fn canvas_stream_label_center(geometry: CanvasStreamLineGeometry) -> egui::Pos2 {
    let delta = geometry.end - geometry.start;
    let normal = if delta.length() > 1.0 {
        let direction = delta.normalized();
        egui::vec2(-direction.y, direction.x)
    } else {
        egui::vec2(0.0, -1.0)
    };
    geometry.start.lerp(geometry.end, 0.5) + normal * 14.0
}

fn canvas_terminal_stream_label_vertical_offset(
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) -> f32 {
    stream
        .source
        .as_ref()
        .filter(|source| source.port_side_count > 1)
        .map(|source| {
            let middle = (source.port_side_count.saturating_sub(1)) as f32 * 0.5;
            (source.port_side_index as f32 - middle) * 28.0
        })
        .unwrap_or(0.0)
}

fn canvas_stream_label_text(
    stream: &radishflow_studio::StudioGuiCanvasStreamLineViewModel,
) -> String {
    if stream.sink.is_none() {
        return truncate_canvas_label(&stream.name, 18);
    }

    let label = if stream.name == stream.stream_id {
        stream.name.clone()
    } else {
        format!("{} ({})", stream.name, stream.stream_id)
    };
    truncate_canvas_label(&label, 26)
}

fn canvas_stream_label_size(label: &str) -> egui::Vec2 {
    egui::vec2(
        (26.0 + label.chars().count() as f32 * 6.2).clamp(54.0, 188.0),
        20.0,
    )
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

fn canvas_unit_block_rect(
    rect: egui::Rect,
    viewport_transform: &CanvasViewportTransform,
    layout_slot: usize,
    layout_position: Option<rf_ui::CanvasPoint>,
) -> egui::Rect {
    viewport_transform.world_rect_to_screen(
        rect,
        canvas_unit_block_world_rect(rect.width(), layout_slot, layout_position),
    )
}

fn canvas_unit_block_world_rect(
    viewport_width: f32,
    layout_slot: usize,
    layout_position: Option<rf_ui::CanvasPoint>,
) -> egui::Rect {
    let block_size = egui::vec2(168.0, 82.0);
    if let Some(position) = layout_position {
        let min = egui::pos2(position.x as f32, position.y as f32);
        return egui::Rect::from_min_size(min, block_size);
    }

    let gap = egui::vec2(22.0, 20.0);
    let left_padding = 18.0;
    let top_padding = 72.0;
    let available_width = (viewport_width - left_padding * 2.0).max(block_size.x);
    let columns = ((available_width + gap.x) / (block_size.x + gap.x))
        .floor()
        .max(1.0) as usize;
    let column = layout_slot % columns;
    let row = layout_slot / columns;
    let min = egui::pos2(
        left_padding + column as f32 * (block_size.x + gap.x),
        top_padding + row as f32 * (block_size.y + gap.y),
    );
    egui::Rect::from_min_size(min, block_size)
}

fn canvas_unit_viewport_anchor_label(layout_slot: usize) -> String {
    format!("unit-slot-{layout_slot}")
}

fn canvas_unit_port_world_anchor(
    viewport_width: f32,
    layout_slot: usize,
    layout_position: Option<rf_ui::CanvasPoint>,
    is_outlet: bool,
    side_index: usize,
    side_count: usize,
) -> egui::Pos2 {
    let unit_rect = canvas_unit_block_world_rect(viewport_width, layout_slot, layout_position);
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
            "{} · {}/{}",
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
    viewport_transform: &CanvasViewportTransform,
    callout: &radishflow_studio::StudioGuiCanvasFocusCalloutViewModel,
    unit_blocks: &[radishflow_studio::StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[radishflow_studio::StudioGuiCanvasStreamLineViewModel],
) -> Option<egui::Pos2> {
    if callout.kind_label == "Unit" {
        return unit_blocks
            .iter()
            .find(|unit| unit.unit_id == callout.target_id)
            .map(|unit| {
                canvas_unit_block_rect(
                    rect,
                    viewport_transform,
                    unit.layout_slot,
                    unit.layout_position,
                )
                .right_top()
            });
    }

    stream_lines
        .iter()
        .find(|stream| stream.stream_id == callout.target_id)
        .map(|stream| {
            let geometry = canvas_stream_line_geometry(rect, viewport_transform, stream);
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

fn compact_canvas_material_line_count(count: usize, locale: StudioShellLocale) -> String {
    match locale {
        StudioShellLocale::ZhCn => format!("{count} 条物料线"),
        StudioShellLocale::En => {
            let label = if count == 1 {
                "material line"
            } else {
                "material lines"
            };
            format!("{count} {label}")
        }
    }
}

fn compact_canvas_legend_item_label(
    item: &radishflow_studio::StudioGuiCanvasLegendItemViewModel,
    locale: StudioShellLocale,
) -> String {
    match item.kind_label {
        "Run" => match locale {
            StudioShellLocale::ZhCn => format!("运行: {}", locale.runtime_label(&item.label)),
            StudioShellLocale::En => format!("Run: {}", item.label),
        },
        "Attention" => locale.runtime_label(&item.label).into_owned(),
        "Ports" => {
            let port_count = item.label.split_whitespace().next().unwrap_or(&item.label);
            match locale {
                StudioShellLocale::ZhCn => format!("端口: {port_count}"),
                StudioShellLocale::En => format!("Ports: {port_count}"),
            }
        }
        "Streams" => {
            let stream_count = item.label.split_whitespace().next().unwrap_or(&item.label);
            match locale {
                StudioShellLocale::ZhCn => format!("流股: {stream_count}"),
                StudioShellLocale::En => format!("Streams: {stream_count}"),
            }
        }
        "Edit" => locale.runtime_label(&item.label).into_owned(),
        "Suggestion" => match locale {
            StudioShellLocale::ZhCn => format!("建议: {}", locale.runtime_label(&item.label)),
            StudioShellLocale::En => format!("Suggestion: {}", item.label),
        },
        _ => format!("{}: {}", item.kind_label, item.label),
    }
}

fn render_canvas_chip_with_hover(
    ui: &mut egui::Ui,
    label: &str,
    color: egui::Color32,
    hover_text: &str,
) {
    let response = egui::Frame::new()
        .fill(color.gamma_multiply(0.12))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.8)))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).color(color).small());
        })
        .response;
    response.on_hover_text(hover_text);
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
        "suggestion" => egui::Color32::from_rgb(86, 118, 168),
        _ => egui::Color32::from_rgb(86, 96, 108),
    }
}

#[cfg(test)]
mod viewport_geometry_tests {
    use super::*;

    fn unit_block(
        unit_id: &str,
        layout_slot: usize,
        layout_position: Option<rf_ui::CanvasPoint>,
    ) -> radishflow_studio::StudioGuiCanvasUnitBlockViewModel {
        radishflow_studio::StudioGuiCanvasUnitBlockViewModel {
            unit_id: unit_id.to_string(),
            name: unit_id.to_string(),
            kind: "feed".to_string(),
            ports: Vec::new(),
            status_badges: Vec::new(),
            port_count: 0,
            connected_port_count: 0,
            command_id: format!("inspector.focus_unit:{unit_id}"),
            action_label: format!("Unit {unit_id}"),
            hover_text: String::new(),
            attention_summary: None,
            layout_slot,
            layout_position,
            is_active_inspector_target: false,
        }
    }

    fn stream_endpoint(
        unit_id: &str,
        layout_position: rf_ui::CanvasPoint,
        is_source: bool,
    ) -> radishflow_studio::StudioGuiCanvasStreamLineEndpointViewModel {
        radishflow_studio::StudioGuiCanvasStreamLineEndpointViewModel {
            unit_id: unit_id.to_string(),
            port_name: if is_source { "outlet" } else { "inlet" }.to_string(),
            layout_slot: 0,
            layout_position: Some(layout_position),
            port_side_index: 0,
            port_side_count: 1,
        }
    }

    fn stream_line(
        stream_id: &str,
        source: Option<radishflow_studio::StudioGuiCanvasStreamLineEndpointViewModel>,
        sink: Option<radishflow_studio::StudioGuiCanvasStreamLineEndpointViewModel>,
    ) -> radishflow_studio::StudioGuiCanvasStreamLineViewModel {
        radishflow_studio::StudioGuiCanvasStreamLineViewModel {
            line_id: format!("{stream_id}:0"),
            stream_id: stream_id.to_string(),
            name: stream_id.to_string(),
            source,
            sink,
            status_badges: Vec::new(),
            command_id: format!("inspector.focus_stream:{stream_id}"),
            action_label: format!("Stream {stream_id}"),
            hover_text: String::new(),
            attention_summary: None,
            is_active_inspector_target: false,
        }
    }

    #[test]
    fn viewport_transform_centers_persisted_small_flowsheet_without_rewriting_coordinates() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(640.0, 280.0));
        let units = vec![
            unit_block("feed-1", 0, Some(rf_ui::CanvasPoint::new(64.0, 40.0))),
            unit_block("flash-1", 1, Some(rf_ui::CanvasPoint::new(220.0, 40.0))),
        ];

        let transform = canvas_viewport_transform(rect, &units, &[]);
        let feed_rect = canvas_unit_block_rect(
            rect,
            &transform,
            units[0].layout_slot,
            units[0].layout_position,
        );
        let flash_rect = canvas_unit_block_rect(
            rect,
            &transform,
            units[1].layout_slot,
            units[1].layout_position,
        );
        let visible_bounds = feed_rect.union(flash_rect);

        assert!((visible_bounds.center().x - rect.center().x).abs() < 0.1);
        assert!((visible_bounds.center().y - rect.center().y).abs() < 0.1);

        let feed_world = transform.screen_to_world(rect, feed_rect.min);
        assert!((feed_world.x - 64.0).abs() < 0.1);
        assert!((feed_world.y - 40.0).abs() < 0.1);
    }

    #[test]
    fn initial_viewport_fit_keeps_offset_after_layout_moves() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(640.0, 280.0));
        let mut state = CanvasInitialViewportFitState::default();
        let units = vec![unit_block(
            "flash-1",
            0,
            Some(rf_ui::CanvasPoint::new(64.0, 40.0)),
        )];
        let moved_units = vec![unit_block(
            "flash-1",
            0,
            Some(rf_ui::CanvasPoint::new(160.0, 40.0)),
        )];

        let initial_transform = canvas_initial_viewport_transform(&mut state, rect, &units, &[]);
        let moved_transform =
            canvas_initial_viewport_transform(&mut state, rect, &moved_units, &[]);

        assert!(!state.pending);
        assert_eq!(moved_transform, initial_transform);
        assert_ne!(
            canvas_viewport_transform(rect, &moved_units, &[]),
            initial_transform
        );
    }

    #[test]
    fn initial_viewport_fit_does_not_wait_for_content_after_blank_render() {
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(640.0, 280.0));
        let mut state = CanvasInitialViewportFitState::default();

        let blank_transform = canvas_initial_viewport_transform(&mut state, rect, &[], &[]);
        let later_units = vec![unit_block(
            "feed-1",
            0,
            Some(rf_ui::CanvasPoint::new(64.0, 40.0)),
        )];
        let later_transform =
            canvas_initial_viewport_transform(&mut state, rect, &later_units, &[]);

        assert_eq!(blank_transform, CanvasViewportTransform::ZERO);
        assert_eq!(later_transform, CanvasViewportTransform::ZERO);
    }

    #[test]
    fn connected_stream_label_is_hidden_when_adjacent_units_leave_no_room() {
        let stream = stream_line(
            "stream-throttled",
            Some(stream_endpoint(
                "valve-1",
                rf_ui::CanvasPoint::new(64.0, 40.0),
                true,
            )),
            Some(stream_endpoint(
                "flash-1",
                rf_ui::CanvasPoint::new(254.0, 40.0),
                false,
            )),
        );
        let geometry = canvas_stream_line_world_geometry(640.0, &stream);

        assert!(canvas_stream_label_rect(geometry, &stream).is_none());
    }

    #[test]
    fn connected_stream_label_is_kept_when_line_has_clear_room() {
        let stream = stream_line(
            "s1",
            Some(stream_endpoint(
                "feed-1",
                rf_ui::CanvasPoint::new(64.0, 40.0),
                true,
            )),
            Some(stream_endpoint(
                "heater-1",
                rf_ui::CanvasPoint::new(340.0, 40.0),
                false,
            )),
        );
        let geometry = canvas_stream_line_world_geometry(640.0, &stream);

        assert!(canvas_stream_label_rect(geometry, &stream).is_some());
    }

    #[test]
    fn terminal_stream_label_is_kept_next_to_source_port() {
        let stream = stream_line(
            "stream-vapor",
            Some(stream_endpoint(
                "flash-1",
                rf_ui::CanvasPoint::new(64.0, 40.0),
                true,
            )),
            None,
        );
        let geometry = canvas_stream_line_world_geometry(640.0, &stream);

        assert!(canvas_stream_label_rect(geometry, &stream).is_some());
    }

    #[test]
    fn unit_drag_position_offsets_from_world_start_and_clamps_to_canvas_origin() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 280.0));
        let transform = CanvasViewportTransform::ZERO;
        let drag = CanvasUnitDragState {
            unit_id: "feed-1".to_string(),
            start_position: rf_ui::CanvasPoint::new(64.0, 40.0),
            pointer_offset: egui::Vec2::ZERO,
            current_position: rf_ui::CanvasPoint::new(64.0, 40.0),
        };

        let moved =
            canvas_unit_drag_position(rect, &transform, &drag, None, egui::vec2(32.0, 18.0));
        assert_eq!(moved, rf_ui::CanvasPoint::new(96.0, 58.0));

        let pointer_moved = canvas_unit_drag_position(
            rect,
            &transform,
            &CanvasUnitDragState {
                pointer_offset: egui::vec2(12.0, 8.0),
                ..drag.clone()
            },
            Some(egui::pos2(120.0, 80.0)),
            egui::Vec2::ZERO,
        );
        assert_eq!(pointer_moved, rf_ui::CanvasPoint::new(108.0, 72.0));

        let clamped = canvas_unit_drag_position(
            rect,
            &transform,
            &CanvasUnitDragState {
                start_position: rf_ui::CanvasPoint::new(20.0, 16.0),
                current_position: rf_ui::CanvasPoint::new(20.0, 16.0),
                ..drag
            },
            None,
            egui::vec2(-48.0, -24.0),
        );
        assert_eq!(clamped, rf_ui::CanvasPoint::new(0.0, 0.0));
    }

    #[test]
    fn viewport_pan_is_disabled_while_unit_drag_is_active() {
        assert!(canvas_can_pan_viewport(false, false, false, false, false));
        assert!(!canvas_can_pan_viewport(false, false, false, false, true));
    }
}
