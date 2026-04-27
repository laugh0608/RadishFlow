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
        ui.separator();
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

    pub(super) fn render_runtime_area(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        area_id: StudioGuiWindowAreaId,
    ) {
        let run_panel = &window.runtime.run_panel;
        let run_panel_view = run_panel.view();

        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Run").strong());
                render_status_chip(
                    ui,
                    run_panel_view.mode_label,
                    egui::Color32::from_rgb(86, 118, 168),
                );
                render_status_chip(
                    ui,
                    run_panel_view.status_label,
                    run_status_color(run_panel_view.status_label),
                );
                if let Some(pending) = run_panel_view.pending_label {
                    render_status_chip(ui, pending, egui::Color32::from_rgb(160, 120, 40));
                }
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let primary = run_panel.primary_action();
                ui.vertical(|ui| {
                    let response =
                        ui.add_enabled(primary.enabled, egui::Button::new(primary.label));
                    let response = response.on_hover_text(primary.detail);
                    if response.clicked() {
                        self.dispatch_run_panel_widget(run_panel.activate_primary());
                    }
                    ui.small(
                        egui::RichText::new(primary.detail)
                            .color(egui::Color32::from_rgb(92, 104, 117)),
                    );
                });

                for action in &run_panel_view.secondary_actions {
                    ui.vertical(|ui| {
                        let response =
                            ui.add_enabled(action.enabled, egui::Button::new(action.label));
                        let response = response.on_hover_text(action.detail);
                        if response.clicked() {
                            self.dispatch_run_panel_widget(run_panel.activate(action.id));
                        }
                        ui.small(
                            egui::RichText::new(action.detail)
                                .color(egui::Color32::from_rgb(92, 104, 117)),
                        );
                    });
                }
            });
            ui.add_space(6.0);
            if let Some(summary) = run_panel_view.latest_snapshot_summary.as_ref() {
                ui.label(summary);
            } else {
                ui.small("还没有求解快照。");
            }
            if let Some(snapshot_id) = run_panel_view.latest_snapshot_id.as_ref() {
                ui.small(format!("Snapshot: {snapshot_id}"));
            }
            if let Some(message) = run_panel_view.latest_log_message.as_ref() {
                ui.small(format!("Latest log: {message}"));
            }
            if let Some(notice) = run_panel_view.notice.as_ref() {
                ui.add_space(6.0);
                ui.colored_label(notice_color(notice.level), &notice.title);
                ui.label(&notice.message);
                if let Some(recovery_action) = notice.recovery_action.as_ref() {
                    ui.small(recovery_action.detail);
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

        if let Some(platform_notice) = window.runtime.platform_notice.as_ref() {
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new("Platform notice").strong());
                ui.colored_label(notice_color(platform_notice.level), &platform_notice.title);
                ui.label(&platform_notice.message);
                for line in &window.runtime.platform_timer_lines {
                    ui.small(line);
                }
            });
        } else if !window.runtime.platform_timer_lines.is_empty() {
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new("Platform").strong());
                for line in &window.runtime.platform_timer_lines {
                    ui.small(line);
                }
            });
        }

        ui.add_space(8.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            let document = &window.runtime.workspace_document;
            ui.label(egui::RichText::new("Workspace").strong());
            ui.horizontal_wrapped(|ui| {
                ui.label(&document.title);
                render_status_chip(
                    ui,
                    &format!("rev {}", document.revision),
                    egui::Color32::from_rgb(86, 118, 168),
                );
            });
            ui.small(format!(
                "{} | {} unit(s) | {} stream(s) | {} snapshot(s)",
                document.flowsheet_name,
                document.unit_count,
                document.stream_count,
                document.snapshot_history_count
            ));
            if let Some(path) = document.project_path.as_ref() {
                ui.small(path);
            }
        });

        ui.add_space(8.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new("Results").strong());
            if let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() {
                ui.horizontal_wrapped(|ui| {
                    render_status_chip(
                        ui,
                        snapshot.status_label,
                        run_status_color(snapshot.status_label),
                    );
                    ui.small(format!(
                        "{} stream(s), {} step(s), {} diagnostic(s)",
                        snapshot.stream_count, snapshot.step_count, snapshot.diagnostic_count
                    ));
                });
                ui.small(format!(
                    "Snapshot {} seq {}",
                    snapshot.snapshot_id, snapshot.sequence
                ));
                ui.label(&snapshot.summary);
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt(format!(
                        "scroll:{}:{}:results",
                        window.layout_state.scope.layout_key,
                        area_label(area_id)
                    ))
                    .max_height(260.0)
                    .show(ui, |ui| {
                        if snapshot.streams.is_empty() {
                            ui.small("当前快照没有流股结果。");
                        } else {
                            for stream in &snapshot.streams {
                                ui.label(egui::RichText::new(&stream.stream_id).strong());
                                ui.small(&stream.label);
                                ui.horizontal_wrapped(|ui| {
                                    ui.small(format!("T {}", stream.temperature_text));
                                    ui.small(format!("P {}", stream.pressure_text));
                                    ui.small(format!("F {}", stream.molar_flow_text));
                                });
                                ui.small(&stream.composition_text);
                                ui.small(&stream.phase_text);
                                ui.add_space(6.0);
                            }
                        }
                    });
            } else {
                ui.small("还没有可显示的求解结果。");
            }
        });

        if let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() {
            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new("Solve steps").strong());
                if snapshot.steps.is_empty() {
                    ui.small("当前快照没有逐步执行记录。");
                } else {
                    for step in &snapshot.steps {
                        ui.small(format!(
                            "#{} {} -> {}",
                            step.index,
                            step.unit_id,
                            step.produced_streams.join(", ")
                        ));
                        ui.label(&step.summary);
                        ui.add_space(4.0);
                    }
                }
            });

            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new("Diagnostics").strong());
                if snapshot.diagnostics.is_empty() {
                    ui.small("当前快照没有诊断记录。");
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
                                        diagnostic.severity_label,
                                        diagnostic_color(diagnostic.severity_label),
                                    );
                                    ui.small(&diagnostic.code);
                                });
                                ui.label(&diagnostic.message);
                                if let Some(units) = diagnostic.related_units_text.as_ref() {
                                    ui.small(format!("units: {units}"));
                                }
                                if let Some(streams) = diagnostic.related_streams_text.as_ref() {
                                    ui.small(format!("streams: {streams}"));
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
                    ui.label(egui::RichText::new("Entitlement").strong());
                    render_status_chip(
                        ui,
                        entitlement.auth_label,
                        egui::Color32::from_rgb(66, 118, 92),
                    );
                    render_status_chip(
                        ui,
                        entitlement.entitlement_label,
                        entitlement_status_color(entitlement.entitlement_label),
                    );
                });
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.small(format!(
                        "Allowed packages: {}",
                        entitlement.allowed_package_count
                    ));
                    ui.small(format!(
                        "Cached manifests: {}",
                        entitlement.package_manifest_count
                    ));
                    if let Some(user) = entitlement.current_user_label.as_deref() {
                        ui.small(format!("User: {user}"));
                    }
                });
                if let Some(authority_url) = entitlement.authority_url.as_deref() {
                    ui.small(format!("Authority: {authority_url}"));
                }
                if let Some(last_synced_at) = entitlement.last_synced_at {
                    ui.small(format!(
                        "Last synced: {}",
                        format_system_time(last_synced_at)
                    ));
                }
                if let Some(offline_lease_expires_at) = entitlement.offline_lease_expires_at {
                    ui.small(format!(
                        "Offline lease expires: {}",
                        format_system_time(offline_lease_expires_at)
                    ));
                }
                if let Some(notice) = entitlement.notice.as_ref() {
                    ui.add_space(4.0);
                    ui.colored_label(notice_color_from_entitlement(notice.level), &notice.title);
                    ui.label(&notice.message);
                }
                if let Some(last_error) = entitlement.last_error.as_ref() {
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::from_rgb(180, 40, 40), last_error);
                }
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    let primary = &entitlement.primary_action;
                    ui.vertical(|ui| {
                        let response = ui.add_enabled(
                            primary.enabled,
                            egui::Button::new(primary.label)
                                .fill(egui::Color32::from_rgb(230, 239, 252)),
                        );
                        let response = response.on_hover_text(primary.detail);
                        if response.clicked() {
                            self.dispatch_ui_command(entitlement_command_id(primary.id));
                        }
                        ui.small(
                            egui::RichText::new(primary.detail)
                                .color(egui::Color32::from_rgb(92, 104, 117)),
                        );
                    });
                    for action in &entitlement.secondary_actions {
                        ui.vertical(|ui| {
                            let response =
                                ui.add_enabled(action.enabled, egui::Button::new(action.label));
                            let response = response.on_hover_text(action.detail);
                            if response.clicked() {
                                self.dispatch_ui_command(entitlement_command_id(action.id));
                            }
                            ui.small(
                                egui::RichText::new(action.detail)
                                    .color(egui::Color32::from_rgb(92, 104, 117)),
                            );
                        });
                    }
                });
            });

            ui.add_space(8.0);
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new("Scheduler").strong());
                ui.small(
                    egui::RichText::new(
                        "Host lifecycle actions are routed through dedicated UI surfaces or native events.",
                    )
                    .color(egui::Color32::from_rgb(92, 104, 117)),
                );
                ui.add_space(4.0);
                for line in &entitlement_host.presentation.text.lines {
                    ui.small(line);
                }
            });
        }

        ui.add_space(8.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new("Runtime log").strong());
            egui::ScrollArea::vertical()
                .id_salt(format!(
                    "scroll:{}:{}:runtime-log",
                    window.layout_state.scope.layout_key,
                    area_label(area_id)
                ))
                .max_height(220.0)
                .show(ui, |ui| {
                    if window.runtime.log_entries.is_empty() {
                        ui.small("暂无运行日志。");
                    } else {
                        for entry in window.runtime.log_entries.iter().rev().take(20) {
                            ui.small(format!(
                                "[{}] {}",
                                log_level_label(entry.level),
                                entry.message
                            ));
                        }
                    }
                });
        });

        ui.add_space(8.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new("GUI activity").strong());
            egui::ScrollArea::vertical()
                .id_salt(format!(
                    "scroll:{}:{}:gui-activity",
                    window.layout_state.scope.layout_key,
                    area_label(area_id)
                ))
                .max_height(160.0)
                .show(ui, |ui| {
                    if window.runtime.gui_activity_lines.is_empty() {
                        ui.small("暂无 GUI 宿主事件。");
                    } else {
                        for line in window.runtime.gui_activity_lines.iter().rev().take(16) {
                            ui.small(line);
                        }
                    }
                });
        });
    }

    pub(super) fn render_command_menu_bar(
        &mut self,
        ui: &mut egui::Ui,
        menu_tree: &[StudioGuiCommandMenuNode],
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Menu").strong());
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
        ui.label(egui::RichText::new("Toolbar").strong());
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
        egui::Window::new("Command Palette")
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
                            ui.small("没有匹配的命令。");
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
