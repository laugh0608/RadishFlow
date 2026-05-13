use super::super::*;

impl ReadyAppState {
    pub(in crate::studio_gui_shell) fn render_commands_area(
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
                if let Some(result) = self.canvas_command_result_command_surface() {
                    self.render_canvas_command_result_command_surface(ui, &result);
                    ui.add_space(6.0);
                }
                for section in &window.commands.command_list_sections {
                    ui.label(egui::RichText::new(section.title).strong());
                    for command in &section.items {
                        self.render_command_list_entry(ui, command);
                    }
                    ui.add_space(6.0);
                }
            });
    }

    pub(in crate::studio_gui_shell) fn render_command_list_entry(
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

    pub(in crate::studio_gui_shell) fn render_canvas_command_result_command_surface(
        &self,
        ui: &mut egui::Ui,
        result: &radishflow_studio::StudioGuiCanvasCommandResultCommandSurfaceViewModel,
    ) {
        ui.label(egui::RichText::new("Canvas result").strong());
        ui.horizontal_wrapped(|ui| {
            render_status_chip(ui, result.status_label, notice_color(result.level));
            ui.label(egui::RichText::new(&result.title).strong());
        });
        render_wrapped_small(ui, &result.detail);
        ui.small(format!(
            "{} | target command={}",
            result.menu_path_text, result.target_command_id
        ));
    }

    pub(in crate::studio_gui_shell) fn render_command_menu_bar(
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

    pub(in crate::studio_gui_shell) fn render_command_menu_node(
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

    pub(in crate::studio_gui_shell) fn render_command_toolbar(
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

    pub(in crate::studio_gui_shell) fn render_command_palette(
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
                let canvas_result = self
                    .canvas_command_result_command_surface()
                    .filter(|result| result.matches_query(&self.command_palette.query));
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
                        if let Some(result) = canvas_result.as_ref() {
                            self.render_canvas_command_result_palette_surface(ui, result);
                            ui.separator();
                        }
                        if palette_items.is_empty() && canvas_result.is_none() {
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

    fn render_canvas_command_result_palette_surface(
        &self,
        ui: &mut egui::Ui,
        result: &radishflow_studio::StudioGuiCanvasCommandResultCommandSurfaceViewModel,
    ) {
        ui.small(egui::RichText::new("Canvas result").strong());
        ui.horizontal_wrapped(|ui| {
            render_status_chip(ui, result.status_label, notice_color(result.level));
            ui.label(&result.title);
        });
        render_wrapped_small(ui, &result.detail);
        ui.small(&result.menu_path_text);
        ui.add_space(6.0);
    }

    pub(in crate::studio_gui_shell) fn render_drop_target_lane(
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

    pub(in crate::studio_gui_shell) fn render_floating_drop_preview_overlay(
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

pub(super) fn entitlement_command_id(action_id: rf_ui::EntitlementActionId) -> &'static str {
    match action_id {
        rf_ui::EntitlementActionId::SyncEntitlement => "entitlement.sync",
        rf_ui::EntitlementActionId::RefreshOfflineLease => "entitlement.refresh_offline_lease",
    }
}
