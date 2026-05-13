use super::*;

impl ReadyAppState {
    pub(super) fn render_top_bar(
        &mut self,
        ctx: &egui::Context,
        windows: &[StudioAppHostWindowState],
        window: &StudioGuiWindowModel,
        _hovered_drop_target: &mut bool,
    ) {
        let current_window_id = window.layout_state.scope.window_id;
        egui::TopBottomPanel::top("studio.titlebar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading(window.header.title);
                if window.runtime.workspace_document.has_unsaved_changes {
                    render_status_chip(
                        ui,
                        self.locale.text(ShellText::Unsaved),
                        egui::Color32::from_rgb(160, 120, 40),
                    );
                }
                if current_window_id.is_none() {
                    ui.small(self.locale.text(ShellText::NoActiveLogicalWindow));
                }
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(self.locale.text(ShellText::QuickActions)).strong());
                ui.menu_button(self.locale.text(ShellText::OpenExample), |ui| {
                    if window.runtime.example_projects.is_empty() {
                        ui.small(self.locale.text(ShellText::NoRecentProjects));
                        return;
                    }
                    let mut requested_project = None;
                    for example in &window.runtime.example_projects {
                        if ui
                            .add_enabled(!example.is_current, egui::Button::new(example.title))
                            .on_hover_text(example.detail)
                            .clicked()
                        {
                            requested_project = Some(example.project_path.clone());
                            ui.close_menu();
                        }
                    }
                    if let Some(project_path) = requested_project {
                        self.open_example_project(project_path);
                    }
                });
                if ui
                    .button(self.locale.text(ShellText::OpenProjectFromDisk))
                    .clicked()
                {
                    self.open_project_from_picker();
                }
                if ui
                    .button(self.locale.text(ShellText::RunCurrentWorkspace))
                    .clicked()
                {
                    self.dispatch_ui_command("run_panel.run_manual");
                }
                if ui
                    .button(self.locale.text(ShellText::SaveProject))
                    .clicked()
                {
                    self.save_project();
                }
                let commands_visible = window
                    .layout_state
                    .panel(StudioGuiWindowAreaId::Commands)
                    .map(|panel| panel.visible)
                    .unwrap_or(false);
                let commands_label = if commands_visible {
                    self.locale.text(ShellText::HideCommands)
                } else {
                    self.locale.text(ShellText::ShowCommands)
                };
                if ui.button(commands_label).clicked() {
                    self.dispatch_layout_mutation(
                        current_window_id,
                        StudioGuiWindowLayoutMutation::SetPanelVisibility {
                            area_id: StudioGuiWindowAreaId::Commands,
                            visible: !commands_visible,
                        },
                    );
                }
                let palette_label = if self.command_palette.open {
                    self.locale.text(ShellText::HideCommandPalette)
                } else {
                    self.locale.text(ShellText::CommandPalette)
                };
                if ui.button(palette_label).clicked() {
                    self.command_palette.toggle();
                }
            });
            ui.horizontal_wrapped(|ui| {
                render_wrapped_small(ui, &window.runtime.workspace_document.title);
                if let Some(path) = window.runtime.workspace_document.project_path.as_ref() {
                    render_wrapped_small(ui, path);
                }
            });
            ui.horizontal_wrapped(|ui| {
                let english = self.locale.text(ShellText::English);
                let chinese = self.locale.text(ShellText::Chinese);
                ui.selectable_value(&mut self.locale, StudioShellLocale::ZhCn, chinese);
                ui.selectable_value(&mut self.locale, StudioShellLocale::En, english);
                if windows.len() > 1 {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(self.locale.text(ShellText::LogicalWindows)).strong(),
                    );
                    self.render_logical_window_chips(ui, windows);
                }
                if ui
                    .button(self.locale.text(ShellText::NewLogicalWindow))
                    .clicked()
                {
                    self.dispatch_event(StudioGuiEvent::OpenWindowRequested);
                }
            });
            if !window.commands.menu_tree.is_empty()
                && window
                    .layout_state
                    .panel(StudioGuiWindowAreaId::Commands)
                    .map(|panel| panel.visible)
                    .unwrap_or(false)
            {
                ui.separator();
                self.render_command_menu_bar(ui, &window.commands.menu_tree);
                ui.horizontal_wrapped(|ui| {
                    self.render_command_toolbar(ui, &window.commands.toolbar_sections);
                });
            }
            if let Some(drag_session) = self.drag_session {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(self.locale.text(ShellText::DropPreview)).strong(),
                    );
                    ui.label(format!("dragging {}", area_label(drag_session.area_id)));
                    if drag_session.window_id == current_window_id {
                        ui.small(
                            "drag across region lane / stack lane / panel header, release to drop",
                        );
                    } else {
                        ui.small("return to source window to drop");
                    }
                    if let Some(preview) = window.drop_preview.as_ref() {
                        ui.small(
                            egui::RichText::new(format_compact_drop_preview_status(preview))
                                .color(egui::Color32::from_rgb(92, 104, 117)),
                        );
                    }
                    if ui.button(self.locale.text(ShellText::Cancel)).clicked() {
                        self.cancel_drag_session(current_window_id);
                    }
                });
            }
            if self.drag_session.is_none() {
                if let Some(preview) = window.drop_preview.as_ref() {
                    ui.separator();
                    ui.small(
                        egui::RichText::new(format_compact_drop_preview_status(preview))
                            .color(egui::Color32::from_rgb(92, 104, 117)),
                    );
                }
            }
            if let Some(error) = self.platform_host.latest_gui_error_line() {
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(180, 40, 40), error);
            }
        });
    }

    pub(super) fn render_logical_window_chips(
        &mut self,
        ui: &mut egui::Ui,
        windows: &[StudioAppHostWindowState],
    ) {
        for window_state in windows {
            ui.horizontal(|ui| {
                let label = format_window_chip(window_state);
                let chip = ui.selectable_label(window_state.is_foreground, label);
                if chip.clicked() {
                    self.dispatch_event(StudioGuiEvent::WindowForegrounded {
                        window_id: window_state.window_id,
                    });
                }

                let close_button = egui::Button::new(
                    egui::RichText::new("x")
                        .small()
                        .color(egui::Color32::from_rgb(120, 120, 120)),
                )
                .frame(false);
                if ui
                    .add(close_button)
                    .on_hover_text("Close logical window")
                    .clicked()
                {
                    self.dispatch_event(StudioGuiEvent::CloseWindowRequested {
                        window_id: window_state.window_id,
                    });
                }
            });
        }
    }

    pub(super) fn render_left_sidebar(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
        hovered_drop_target: &mut bool,
    ) {
        let left_width = region_panel_width(
            &window.layout_state,
            ctx,
            StudioGuiWindowDockRegion::LeftSidebar,
        );
        let visible = window
            .layout_state
            .panels_in_dock_region(StudioGuiWindowDockRegion::LeftSidebar)
            .into_iter()
            .any(|panel| panel.visible);
        if !visible {
            return;
        }

        egui::SidePanel::left("studio.left_sidebar")
            .default_width(left_width)
            .min_width(left_width)
            .max_width(left_width)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_region(
                    ui,
                    window,
                    StudioGuiWindowDockRegion::LeftSidebar,
                    hovered_drop_target,
                );
            });
    }

    pub(super) fn render_right_sidebar(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
        hovered_drop_target: &mut bool,
    ) {
        let right_width = region_panel_width(
            &window.layout_state,
            ctx,
            StudioGuiWindowDockRegion::RightSidebar,
        );
        let visible = window
            .layout_state
            .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
            .into_iter()
            .any(|panel| panel.visible);
        if !visible {
            return;
        }

        egui::SidePanel::right("studio.right_sidebar")
            .default_width(right_width)
            .min_width(right_width)
            .max_width(right_width)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_region(
                    ui,
                    window,
                    StudioGuiWindowDockRegion::RightSidebar,
                    hovered_drop_target,
                );
            });
    }

    pub(super) fn render_center_stage(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
        hovered_drop_target: &mut bool,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_region(
                ui,
                window,
                StudioGuiWindowDockRegion::CenterStage,
                hovered_drop_target,
            );
        });
    }

    pub(super) fn render_region(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        region: StudioGuiWindowDockRegion,
        hovered_drop_target: &mut bool,
    ) {
        let layout = window.layout();
        let groups = layout.stack_groups_in_dock_region(region);
        let window_id = window.layout_state.scope.window_id;
        let drag_session = self.active_drag_session_for_window(window_id);
        let region_preview = drop_preview_for_region(window.drop_preview.as_ref(), region);
        if let Some(preview) = region_preview {
            ui.colored_label(
                egui::Color32::from_rgb(56, 126, 214),
                format!(
                    "Preview target: {} stack {}",
                    dock_region_label(preview.overlay.target_dock_region),
                    preview.overlay.target_stack_group
                ),
            );
            ui.small(format!(
                "highlighted panels: {}",
                format_area_id_list(&preview.overlay.highlighted_area_ids)
            ));
            ui.add_space(6.0);
        }
        if let Some(drag_session) = drag_session {
            self.render_drop_target_lane(
                ui,
                window_id,
                StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: drag_session.area_id,
                    dock_region: region,
                    placement: StudioGuiWindowDockPlacement::End,
                },
                &format!(
                    "Drop {} into {} region",
                    area_label(drag_session.area_id),
                    dock_region_label(region)
                ),
                hovered_drop_target,
            );
            ui.add_space(6.0);
        }
        if groups.is_empty() {
            ui.label(self.locale.text(ShellText::NoPanelsInRegion));
            return;
        }

        let new_stack_insert_group_index = new_stack_preview_group_index(region_preview);
        for (group_index, group) in groups.iter().enumerate() {
            if new_stack_insert_group_index == Some(group_index) {
                if let Some(preview) = region_preview {
                    let rect = render_new_stack_insert_overlay(ui, preview);
                    self.record_drop_preview_overlay_anchor(
                        rect,
                        drop_preview_anchor_priority_new_stack(),
                    );
                    ui.add_space(8.0);
                }
            }

            let visible_tabs = group
                .tabs
                .iter()
                .filter(|tab| {
                    layout
                        .panel(tab.area_id)
                        .map(|panel| panel.visible)
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
            if visible_tabs.is_empty() {
                continue;
            }

            let is_target_stack =
                drop_preview_targets_stack(window.drop_preview.as_ref(), region, group.stack_group);
            egui::Frame::group(ui.style())
                .fill(stack_preview_fill(is_target_stack))
                .stroke(stack_preview_stroke(is_target_stack))
                .show(ui, |ui| {
                    if group.tabbed {
                        if let Some(drag_session) = drag_session {
                            if let Some(query) =
                                stack_group_drop_target_query(&layout, drag_session, group)
                            {
                                self.render_drop_target_lane(
                                    ui,
                                    window_id,
                                    query,
                                    &format!(
                                        "Append {} to current stack",
                                        area_label(drag_session.area_id)
                                    ),
                                    hovered_drop_target,
                                );
                                ui.add_space(4.0);
                            }
                        }
                        let mut tab_rects = Vec::new();
                        let tab_strip = ui.horizontal_wrapped(|ui| {
                            for tab in &visible_tabs {
                                let tab_label = if preview_anchor_matches_area(
                                    window.drop_preview.as_ref(),
                                    tab.area_id,
                                ) {
                                    format!("{} <- anchor", tab.title)
                                } else {
                                    tab.title.to_string()
                                };
                                let tab_text = if preview_anchor_matches_area(
                                    window.drop_preview.as_ref(),
                                    tab.area_id,
                                ) {
                                    egui::RichText::new(tab_label)
                                        .color(egui::Color32::from_rgb(56, 126, 214))
                                } else {
                                    egui::RichText::new(tab_label)
                                };
                                let response = ui.selectable_label(tab.active, tab_text);
                                tab_rects.push((tab.area_id, response.rect));
                                if drag_session.is_none() && response.drag_started() {
                                    self.begin_drag_session(window_id, tab.area_id);
                                }
                                if response.clicked() {
                                    self.dispatch_layout_mutation(
                                        window.layout_state.scope.window_id,
                                        StudioGuiWindowLayoutMutation::SetActivePanelInStack {
                                            area_id: tab.area_id,
                                        },
                                    );
                                }
                            }
                        });
                        paint_stack_tab_insert_marker(
                            ui,
                            tab_strip.response.rect,
                            &tab_rects,
                            window.drop_preview.as_ref(),
                            region,
                            group.stack_group,
                        );
                        if stack_accepts_overlay_anchor(
                            window.drop_preview.as_ref(),
                            region,
                            group.stack_group,
                        ) {
                            self.record_drop_preview_overlay_anchor(
                                tab_strip.response.rect,
                                drop_preview_anchor_priority_stack_tabs(),
                            );
                        }
                        ui.separator();
                    }

                    let active_area_id = group.active_area_id;
                    self.render_area(ui, window, active_area_id, hovered_drop_target);
                });
            ui.add_space(8.0);
        }

        if new_stack_insert_group_index == Some(groups.len()) {
            if let Some(preview) = region_preview {
                let rect = render_new_stack_insert_overlay(ui, preview);
                self.record_drop_preview_overlay_anchor(
                    rect,
                    drop_preview_anchor_priority_new_stack(),
                );
                ui.add_space(8.0);
            }
        }
    }

    pub(super) fn render_area(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
        hovered_drop_target: &mut bool,
    ) {
        let layout = window.layout();
        let Some(panel) = layout.panel(area_id).cloned() else {
            return;
        };
        if !panel.visible {
            return;
        }
        let window_id = window.layout_state.scope.window_id;
        let drag_session = self.active_drag_session_for_window(window_id);
        let preview_badges = preview_area_badges(window, area_id);
        let preview_transition = preview_area_transition(window, area_id);
        let header_drop_query = drag_session
            .and_then(|drag_session| area_drop_target_query(&layout, drag_session, area_id));
        let header_rect;
        let header_drag_id = ui.make_persistent_id(format!(
            "panel-drag-header:{}:{area_id:?}",
            window.layout_state.scope.layout_key
        ));

        if let Some(query) = header_drop_query {
            let is_active_preview =
                self.active_drop_preview == Some(ActiveDropPreview { window_id, query });
            let header = egui::Frame::group(ui.style())
                .fill(drop_lane_fill(is_active_preview))
                .stroke(drop_lane_stroke(is_active_preview))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(panel.title).strong());
                        if let Some(badge) = panel.badge.as_ref() {
                            ui.label(format!("[{badge}]"));
                        }
                        for badge in &preview_badges {
                            ui.label(
                                egui::RichText::new(format!("[{badge}]"))
                                    .small()
                                    .color(egui::Color32::from_rgb(56, 126, 214)),
                            );
                        }
                        ui.label(&panel.summary);
                        ui.small("hover to preview, click to drop before this panel");
                    });
                    if let Some(preview_transition) = preview_transition.as_ref() {
                        ui.small(
                            egui::RichText::new(preview_transition)
                                .color(egui::Color32::from_rgb(56, 126, 214)),
                        );
                    }
                });
            header_rect = header.response.rect;
            let response = ui.interact(
                header_rect,
                ui.make_persistent_id(format!("panel-drop-header:{window_id:?}:{query:?}")),
                egui::Sense::click(),
            );
            self.process_drop_target_response(response, window_id, query, hovered_drop_target);
        } else {
            let header = ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(panel.title).strong());
                if let Some(badge) = panel.badge.as_ref() {
                    ui.label(format!("[{badge}]"));
                }
                for badge in &preview_badges {
                    ui.label(
                        egui::RichText::new(format!("[{badge}]"))
                            .small()
                            .color(egui::Color32::from_rgb(56, 126, 214)),
                    );
                }
                ui.label(&panel.summary);
            });
            if let Some(preview_transition) = preview_transition.as_ref() {
                ui.small(
                    egui::RichText::new(preview_transition)
                        .color(egui::Color32::from_rgb(56, 126, 214)),
                );
            }
            header_rect = header.response.rect;
            let header_drag_response =
                ui.interact(header_rect, header_drag_id, egui::Sense::click_and_drag());
            if drag_session.is_none() && header_drag_response.drag_started() {
                self.begin_drag_session(window_id, area_id);
            }
        }
        if area_accepts_overlay_anchor(window, area_id) {
            self.record_drop_preview_overlay_anchor(
                header_rect,
                drop_preview_anchor_priority_area(window, area_id),
            );
        }
        paint_area_preview_overlay(ui, header_rect, window, area_id);
        let body_rect = ui
            .push_id(
                format!(
                    "panel:{}:{}",
                    window.layout_state.scope.layout_key,
                    area_label(area_id)
                ),
                |ui| {
                    ui.vertical(|ui| {
                        if panel.collapsed {
                            ui.label(self.locale.text(ShellText::PanelIsCollapsed));
                            return;
                        }

                        match area_id {
                            StudioGuiWindowAreaId::Commands => {
                                self.render_commands_area(ui, window, area_id)
                            }
                            StudioGuiWindowAreaId::Canvas => {
                                self.render_canvas_area(ui, window, area_id)
                            }
                            StudioGuiWindowAreaId::Runtime => {
                                self.render_runtime_area(ui, window, area_id)
                            }
                        }
                    })
                    .response
                    .rect
                },
            )
            .inner;
        self.update_area_focus_from_rect(ui.ctx(), area_id, header_rect);
        self.update_area_focus_from_rect(ui.ctx(), area_id, body_rect);
    }
}
