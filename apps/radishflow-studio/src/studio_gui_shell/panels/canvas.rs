use super::super::*;

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn render_canvas_area(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
    ) {
        let widget = &window.canvas.widget;
        ui.horizontal_wrapped(|ui| {
            for action in &widget.actions {
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
                    .clicked()
                {
                    match widget.activate(action.id) {
                        radishflow_studio::StudioGuiCanvasWidgetEvent::Requested {
                            event, ..
                        } => self.dispatch_event(event),
                        radishflow_studio::StudioGuiCanvasWidgetEvent::Disabled { .. }
                        | radishflow_studio::StudioGuiCanvasWidgetEvent::Missing { .. }
                        | radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionRequested {
                            ..
                        }
                        | radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionDisabled {
                            ..
                        }
                        | radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionMissing {
                            ..
                        } => {}
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
                            ui.label(
                                egui::RichText::new(self.locale.runtime_label(focus).as_ref())
                                    .strong(),
                            );
                            ui.label(format!("{:.0}%", suggestion.confidence * 100.0));
                            ui.label(format!("source={}", suggestion.source_label));
                            ui.label(format!(
                                "status={}",
                                self.locale.runtime_label(suggestion.status_label)
                            ));
                        });
                        ui.label(format!("target={}", suggestion.target_unit_id));
                        ui.label(&suggestion.reason);
                        ui.small(format!("id={}", suggestion.id));
                        if ui
                            .add_enabled(
                                suggestion.explicit_accept_enabled,
                                egui::Button::new(self.locale.text(ShellText::ConnectSuggestion)),
                            )
                            .clicked()
                        {
                            match widget.activate_suggestion(&suggestion.id) {
                                radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionRequested {
                                    event,
                                    ..
                                } => self.dispatch_event(event),
                                radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionDisabled {
                                    ..
                                }
                                | radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionMissing {
                                    ..
                                }
                                | radishflow_studio::StudioGuiCanvasWidgetEvent::Requested {
                                    ..
                                }
                                | radishflow_studio::StudioGuiCanvasWidgetEvent::Disabled {
                                    ..
                                }
                                | radishflow_studio::StudioGuiCanvasWidgetEvent::Missing {
                                    ..
                                } => {}
                            }
                        }
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
            ui.small(egui::RichText::new(self.locale.text(ShellText::Objects)).strong());
            render_status_chip(
                ui,
                &self
                    .locale
                    .count_label(object_list.unit_count, "unit", "units"),
                egui::Color32::from_rgb(86, 118, 168),
            );
            render_status_chip(
                ui,
                &self
                    .locale
                    .count_label(object_list.stream_count, "stream", "streams"),
                egui::Color32::from_rgb(42, 142, 122),
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
        });
        ui.horizontal_wrapped(|ui| {
            for option in &object_list.filter_options {
                let selected = self.canvas_object_filter.filter_id() == option.filter_id;
                let label = format!(
                    "{} {}",
                    self.locale.runtime_label(option.label),
                    option.count
                );
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
            ui.small(self.locale.text(ShellText::NoneValue));
            return;
        }
        let visible_items = object_list
            .items
            .iter()
            .filter(|item| self.canvas_object_filter.matches(item))
            .collect::<Vec<_>>();
        if visible_items.is_empty() {
            ui.small(self.locale.text(ShellText::NoObjectsInFilter));
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
                        self.locale.runtime_label(item.kind_label).as_ref(),
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
                        self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
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
                        if let Some(summary) = item.attention_summary.as_ref() {
                            render_status_chip(
                                ui,
                                self.locale.text(ShellText::Attention),
                                notice_color(rf_ui::RunPanelNoticeLevel::Warning),
                            );
                            ui.small(summary);
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
                    self.locale.runtime_label(status.status_label).as_ref(),
                    run_status_color(status.status_label),
                );
                if status.attention_count > 0 {
                    render_status_chip(
                        ui,
                        &self
                            .locale
                            .count_label(status.attention_count, "attention", "attention"),
                        notice_color(rf_ui::RunPanelNoticeLevel::Warning),
                    );
                }
                if let Some(summary) = status.summary.as_ref() {
                    ui.small(truncate_canvas_label(summary, 42));
                } else if let Some(reason) = status.pending_reason_label {
                    match self.locale {
                        StudioShellLocale::En => ui.small(format!("pending={reason}")),
                        StudioShellLocale::ZhCn => {
                            ui.small(format!("待处理={}", self.locale.runtime_label(reason)))
                        }
                    };
                }
                ui.separator();
            }
            ui.small(egui::RichText::new(self.locale.text(ShellText::Selection)).strong());
            if let Some(selection) = widget.view().current_selection.as_ref() {
                render_status_chip(
                    ui,
                    self.locale.runtime_label(selection.kind_label).as_ref(),
                    egui::Color32::from_rgb(48, 112, 188),
                );
                ui.small(format!("{} · {}", selection.target_id, selection.summary));
                if let Some(layout_source) = selection.layout_source_label {
                    render_status_chip(ui, layout_source, egui::Color32::from_rgb(86, 96, 108));
                }
                if let Some(layout_detail) = selection.layout_detail.as_ref() {
                    ui.small(layout_detail);
                }
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
                }
            } else {
                ui.small(self.locale.text(ShellText::NoneValue));
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
            ui.small(egui::RichText::new(self.locale.text(ShellText::Viewport)).strong());
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
            ui.small(&viewport.summary);
            if let Some(focus) = viewport.focus.as_ref() {
                render_status_chip(
                    ui,
                    &format!(
                        "{} {}",
                        self.locale.runtime_label(focus.kind_label),
                        focus.target_id
                    ),
                    egui::Color32::from_rgb(48, 112, 188),
                );
                ui.small(&focus.anchor_label);
                if ui
                    .small_button(self.locale.text(ShellText::Focus))
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
            ui.small(
                egui::RichText::new(self.locale.runtime_label(legend.title).as_ref()).strong(),
            );
            for item in &legend.items {
                let color = canvas_legend_swatch_color(item.swatch_label);
                let label = format!(
                    "{}: {}",
                    self.locale.runtime_label(item.kind_label),
                    self.locale.runtime_label(&item.label)
                );
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
            .unwrap_or(self.locale.text(ShellText::CanvasToolPrompt));
        let subtitle = if pending_edit.is_some() {
            self.locale.text(ShellText::CanvasPlacePrompt)
        } else {
            self.locale.text(ShellText::CanvasEditPrompt)
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
            let unit_rect = canvas_unit_block_rect(rect, unit.layout_slot, unit.layout_position);
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
                self.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(
                    local.x as f64,
                    local.y as f64,
                ));
            }
        }

        hovered_port_stream_id
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
            endpoint.layout_position,
            true,
            endpoint.port_side_index,
            endpoint.port_side_count,
        )
    });
    let sink = stream.sink.as_ref().map(|endpoint| {
        canvas_unit_port_anchor(
            rect,
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

fn canvas_unit_block_rect(
    rect: egui::Rect,
    layout_slot: usize,
    layout_position: Option<rf_ui::CanvasPoint>,
) -> egui::Rect {
    let block_size = egui::vec2(156.0, 72.0);
    if let Some(position) = layout_position {
        let min = egui::pos2(
            rect.left() + (position.x as f32).clamp(0.0, (rect.width() - block_size.x).max(0.0)),
            rect.top() + (position.y as f32).clamp(0.0, (rect.height() - block_size.y).max(0.0)),
        );
        return egui::Rect::from_min_size(min, block_size);
    }

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
    layout_position: Option<rf_ui::CanvasPoint>,
    is_outlet: bool,
    side_index: usize,
    side_count: usize,
) -> egui::Pos2 {
    let unit_rect = canvas_unit_block_rect(rect, layout_slot, layout_position);
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
            .map(|unit| {
                canvas_unit_block_rect(rect, unit.layout_slot, unit.layout_position).right_top()
            });
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
