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
                ui.separator();
                render_wrapped_small(ui, &window.runtime.workspace_document.title);
                render_status_chip(
                    ui,
                    self.locale
                        .runtime_label(window.runtime.run_panel.view().mode_label)
                        .as_ref(),
                    egui::Color32::from_rgb(86, 118, 168),
                );
                render_status_chip(
                    ui,
                    self.locale
                        .runtime_label(window.runtime.run_panel.view().status_label)
                        .as_ref(),
                    run_status_color(window.runtime.run_panel.view().status_label),
                );
                if let Some(pending) = window.runtime.run_panel.view().pending_label {
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(pending).as_ref(),
                        egui::Color32::from_rgb(160, 120, 40),
                    );
                }
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
                if ui.button("Home").clicked() {
                    self.screen = StudioShellScreen::Home;
                }
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
                    .button(self.locale.text(ShellText::NewBlankProject))
                    .clicked()
                {
                    self.create_blank_project();
                }
                if ui
                    .button(self.locale.text(ShellText::OpenProjectFromDisk))
                    .clicked()
                {
                    self.open_project_from_picker();
                }
                let run_command =
                    window_command_toolbar_item(&window.commands, "run_panel.run_manual");
                let run_enabled = run_command.map(|command| command.enabled).unwrap_or(false);
                let run_hover = run_command
                    .map(|command| command.hover_text.as_str())
                    .unwrap_or("Run command is not available in the current workspace.");
                if ui
                    .add_enabled(
                        run_enabled,
                        egui::Button::new(self.locale.text(ShellText::RunCurrentWorkspace))
                            .fill(egui::Color32::from_rgb(230, 239, 252)),
                    )
                    .on_hover_text(run_hover)
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
                if ui
                    .button(self.locale.text(ShellText::SaveProjectAs))
                    .clicked()
                {
                    self.save_project_as_from_picker();
                }
                ui.menu_button(self.locale.text(ShellText::ViewOptions), |ui| {
                    let palette_label = if self.command_palette.open {
                        self.locale.text(ShellText::HideCommandPalette)
                    } else {
                        self.locale.text(ShellText::CommandPalette)
                    };
                    if ui.button(palette_label).clicked() {
                        self.command_palette.toggle();
                        ui.close_menu();
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
                        ui.close_menu();
                    }
                    ui.separator();
                    let english = self.locale.text(ShellText::English);
                    let chinese = self.locale.text(ShellText::Chinese);
                    ui.horizontal_wrapped(|ui| {
                        ui.selectable_value(&mut self.locale, StudioShellLocale::ZhCn, chinese);
                        ui.selectable_value(&mut self.locale, StudioShellLocale::En, english);
                    });
                    ui.separator();
                    if ui
                        .button(self.locale.text(ShellText::NewLogicalWindow))
                        .clicked()
                    {
                        self.dispatch_event(StudioGuiEvent::OpenWindowRequested);
                        ui.close_menu();
                    }
                    if windows.len() > 1 {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(self.locale.text(ShellText::LogicalWindows))
                                .strong(),
                        );
                        self.render_logical_window_chips(ui, windows);
                    }
                });
            });
            self.render_project_operation_strip(ui);
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
        _hovered_drop_target: &mut bool,
    ) {
        let left_width = region_panel_width(
            &window.layout_state,
            ctx,
            StudioGuiWindowDockRegion::LeftSidebar,
        )
        .clamp(240.0, 280.0);
        egui::SidePanel::left("studio.left_sidebar")
            .default_width(left_width)
            .min_width(240.0)
            .max_width(280.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_left_workbench(ui, window);
            });
    }

    pub(super) fn render_right_sidebar(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
        _hovered_drop_target: &mut bool,
    ) {
        let right_width = region_panel_width(
            &window.layout_state,
            ctx,
            StudioGuiWindowDockRegion::RightSidebar,
        )
        .clamp(340.0, 420.0);
        egui::SidePanel::right("studio.right_sidebar")
            .default_width(right_width)
            .min_width(340.0)
            .max_width(420.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_right_workbench(ui, window);
            });
    }

    pub(super) fn render_bottom_drawer(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
    ) {
        egui::TopBottomPanel::bottom("studio.bottom_drawer")
            .exact_height(190.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.render_bottom_workbench(ui, window);
            });
    }

    pub(super) fn render_bottom_status_bar(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
    ) {
        egui::TopBottomPanel::bottom("studio.status_bar")
            .exact_height(30.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let run_panel_view = window.runtime.run_panel.view();
                    render_status_chip(
                        ui,
                        self.locale
                            .runtime_label(run_panel_view.status_label)
                            .as_ref(),
                        run_status_color(run_panel_view.status_label),
                    );
                    ui.separator();
                    ui.small(self.locale.text(ShellText::UnitsSi));
                    ui.separator();
                    ui.small(self.locale.text(ShellText::SolverSequentialModular));
                    ui.separator();
                    ui.small(self.locale.text(ShellText::FlowsheetMode));
                    ui.separator();
                    ui.small(self.locale.unit_stream_counts(
                        window.runtime.workspace_document.unit_count,
                        window.runtime.workspace_document.stream_count,
                    ));
                    if let Some(selection) = window.canvas.widget.view().current_selection.as_ref()
                    {
                        ui.separator();
                        ui.small(
                            self.locale
                                .selected_target(selection.kind_label, &selection.target_id),
                        );
                    }
                });
            });
    }

    fn render_left_workbench(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.left_sidebar_tab,
                StudioShellLeftSidebarTab::Project,
                self.locale.text(ShellText::Project),
            );
            ui.selectable_value(
                &mut self.left_sidebar_tab,
                StudioShellLeftSidebarTab::Examples,
                self.locale.text(ShellText::ExampleProjects),
            );
            ui.selectable_value(
                &mut self.left_sidebar_tab,
                StudioShellLeftSidebarTab::Palette,
                self.locale.text(ShellText::Palette),
            );
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt(format!(
                "scroll:{}:left-workbench",
                window.layout_state.scope.layout_key
            ))
            .auto_shrink([false, false])
            .show(ui, |ui| match self.left_sidebar_tab {
                StudioShellLeftSidebarTab::Project => self.render_project_navigator(ui, window),
                StudioShellLeftSidebarTab::Examples => self.render_examples_navigator(ui, window),
                StudioShellLeftSidebarTab::Palette => self.render_canvas_palette(ui, window),
            });
    }

    fn render_examples_navigator(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.label(egui::RichText::new(self.locale.text(ShellText::ExampleProjects)).strong());
        render_wrapped_small(
            ui,
            match self.locale {
                StudioShellLocale::En => "Bundled examples open into the current workbench.",
                StudioShellLocale::ZhCn => "内置示例会打开到当前工作台。",
            },
        );
        ui.add_space(8.0);

        if window.runtime.example_projects.is_empty() {
            ui.colored_label(
                egui::Color32::from_rgb(160, 120, 40),
                self.locale.text(ShellText::NoRecentProjects),
            );
            return;
        }

        for example in &window.runtime.example_projects {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(example.title).strong());
                    if example.is_current {
                        render_status_chip(
                            ui,
                            self.locale.runtime_label("Current").as_ref(),
                            egui::Color32::from_rgb(52, 128, 89),
                        );
                    }
                });
                render_wrapped_small(ui, example.detail);
                if ui
                    .add_enabled(
                        !example.is_current,
                        egui::Button::new(self.locale.text(ShellText::OpenExample)),
                    )
                    .on_hover_text(example.project_path.display().to_string())
                    .clicked()
                {
                    self.open_example_project(example.project_path.clone());
                }
            });
            ui.add_space(6.0);
        }
    }

    fn render_project_navigator(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        let document = &window.runtime.workspace_document;
        ui.label(egui::RichText::new(&document.flowsheet_name).strong());
        render_wrapped_small(ui, &document.title);
        ui.add_space(8.0);

        self.render_project_tree_row(
            ui,
            self.locale.text(ShellText::PropertyPackage),
            "binary-hydrocarbon-lite-v1",
            None,
        );
        self.render_project_tree_row(
            ui,
            self.locale.text(ShellText::Streams),
            "",
            Some(document.stream_count),
        );
        for item in window
            .canvas
            .widget
            .view()
            .object_list
            .items
            .iter()
            .filter(|item| item.kind_label == "Stream")
        {
            self.render_project_object_button(ui, item);
        }
        ui.add_space(6.0);

        self.render_project_tree_row(
            ui,
            self.locale.text(ShellText::Units),
            "",
            Some(document.unit_count),
        );
        for item in window
            .canvas
            .widget
            .view()
            .object_list
            .items
            .iter()
            .filter(|item| item.kind_label == "Unit")
        {
            self.render_project_object_button(ui, item);
        }
        ui.add_space(6.0);

        self.render_project_tree_row(
            ui,
            self.locale.text(ShellText::Results),
            "",
            Some(
                window
                    .runtime
                    .latest_solve_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.stream_count)
                    .unwrap_or(0),
            ),
        );
        self.render_project_tree_row(
            ui,
            self.locale.text(ShellText::Diagnostics),
            "",
            Some(
                window
                    .runtime
                    .latest_solve_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.diagnostic_count)
                    .or_else(|| {
                        window.runtime.latest_failure.as_ref().and_then(|failure| {
                            failure
                                .diagnostic_detail
                                .as_ref()
                                .map(|detail| detail.diagnostic_count)
                        })
                    })
                    .unwrap_or(0),
            ),
        );
    }

    fn render_project_tree_row(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        detail: &str,
        count: Option<usize>,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(title).strong());
            if let Some(count) = count {
                render_status_chip(ui, &count.to_string(), egui::Color32::from_rgb(86, 96, 108));
            }
        });
        if !detail.is_empty() {
            render_wrapped_small(ui, detail);
        }
    }

    fn render_project_object_button(
        &mut self,
        ui: &mut egui::Ui,
        item: &radishflow_studio::StudioGuiCanvasObjectListItemViewModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            let response = ui
                .add(
                    egui::Button::new(format!("  {}", item.label))
                        .selected(item.is_active)
                        .frame(false),
                )
                .on_hover_text(&item.detail);
            if response.clicked() {
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                self.dispatch_ui_command(&item.command_id);
            }
            if ui
                .small_button(self.locale.text(ShellText::InspectObject))
                .on_hover_text(&item.detail)
                .clicked()
            {
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                self.dispatch_ui_command(&item.command_id);
            }
        });
        if let Some(summary) = item.attention_summary.as_ref() {
            render_wrapped_small(ui, summary);
        }
    }

    fn render_canvas_palette(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        let palette = &window.canvas.widget.view().place_unit_palette;
        ui.label(egui::RichText::new(self.locale.runtime_label(palette.title).as_ref()).strong());
        if let Some(active) = palette.active_unit_kind.as_ref() {
            render_status_chip(
                ui,
                &match self.locale {
                    StudioShellLocale::En => format!("placing {active}"),
                    StudioShellLocale::ZhCn => format!("正在放置 {active}"),
                },
                egui::Color32::from_rgb(52, 128, 89),
            );
            ui.add_space(6.0);
        }

        for option in &palette.options {
            let option_label = self.locale.runtime_label(&option.label);
            let option_detail = self.locale.runtime_label(&option.detail);
            let response = ui
                .add_enabled(
                    option.enabled,
                    egui::Button::new(option_label.as_ref())
                        .selected(option.active)
                        .min_size(egui::vec2(ui.available_width(), 30.0)),
                )
                .on_hover_text(option_detail.as_ref());
            if response.clicked() {
                self.dispatch_ui_command(&option.command_id);
            }
            ui.add_space(4.0);
        }

        let suggestions = &window.canvas.widget.view().suggestions;
        if !suggestions.is_empty() {
            ui.separator();
            ui.label(egui::RichText::new(self.locale.text(ShellText::Suggestions)).strong());
            for suggestion in suggestions.iter().take(4) {
                ui.horizontal_wrapped(|ui| {
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(suggestion.status_label).as_ref(),
                        egui::Color32::from_rgb(86, 118, 168),
                    );
                    ui.small(format!("{:.0}%", suggestion.confidence * 100.0));
                });
                render_wrapped_small(ui, &suggestion.reason);
                if ui
                    .add_enabled(
                        suggestion.explicit_accept_enabled,
                        egui::Button::new(self.locale.text(ShellText::ConnectSuggestion)),
                    )
                    .clicked()
                {
                    match window.canvas.widget.activate_suggestion(&suggestion.id) {
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
                        | radishflow_studio::StudioGuiCanvasWidgetEvent::Requested { .. }
                        | radishflow_studio::StudioGuiCanvasWidgetEvent::Disabled { .. }
                        | radishflow_studio::StudioGuiCanvasWidgetEvent::Missing { .. } => {}
                    }
                }
                ui.add_space(6.0);
            }
        }
    }

    fn render_right_workbench(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(
                &mut self.right_sidebar_tab,
                StudioShellRightSidebarTab::Inspector,
                self.locale.text(ShellText::Inspector),
            );
            ui.selectable_value(
                &mut self.right_sidebar_tab,
                StudioShellRightSidebarTab::Results,
                self.locale.text(ShellText::Results),
            );
            ui.selectable_value(
                &mut self.right_sidebar_tab,
                StudioShellRightSidebarTab::Run,
                self.locale.text(ShellText::Run),
            );
            ui.selectable_value(
                &mut self.right_sidebar_tab,
                StudioShellRightSidebarTab::Package,
                self.locale.text(ShellText::PropertyPackage),
            );
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt(format!(
                "scroll:{}:right-workbench",
                window.layout_state.scope.layout_key
            ))
            .auto_shrink([false, false])
            .show(ui, |ui| match self.right_sidebar_tab {
                StudioShellRightSidebarTab::Inspector => {
                    self.render_runtime_inspector_tab(ui, window)
                }
                StudioShellRightSidebarTab::Results => self.render_runtime_results_tab(ui, window),
                StudioShellRightSidebarTab::Run => self.render_runtime_run_tab(ui, window),
                StudioShellRightSidebarTab::Package => self.render_runtime_package_tab(ui, window),
            });
    }

    fn render_project_operation_strip(&mut self, ui: &mut egui::Ui) {
        if self.project_open.pending_confirmation.is_none()
            && self.project_open.pending_save_as_overwrite.is_none()
        {
            return;
        }

        ui.separator();
        ui.horizontal_wrapped(|ui| {
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
            }
            if self.project_open.pending_save_as_overwrite.is_some() {
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
            }
        });
    }

    fn render_bottom_workbench(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(
                &mut self.bottom_drawer_tab,
                StudioShellBottomDrawerTab::Messages,
                self.locale.text(ShellText::Messages),
            );
            ui.selectable_value(
                &mut self.bottom_drawer_tab,
                StudioShellBottomDrawerTab::RunLog,
                self.locale.text(ShellText::RuntimeLog),
            );
            ui.selectable_value(
                &mut self.bottom_drawer_tab,
                StudioShellBottomDrawerTab::ResultsTable,
                self.locale.text(ShellText::ResultsTable),
            );
            ui.selectable_value(
                &mut self.bottom_drawer_tab,
                StudioShellBottomDrawerTab::Diagnostics,
                self.locale.text(ShellText::Diagnostics),
            );
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt(format!(
                "scroll:{}:bottom-workbench",
                window.layout_state.scope.layout_key
            ))
            .auto_shrink([false, false])
            .show(ui, |ui| match self.bottom_drawer_tab {
                StudioShellBottomDrawerTab::Messages => self.render_bottom_messages(ui, window),
                StudioShellBottomDrawerTab::RunLog => self.render_bottom_run_log(ui, window),
                StudioShellBottomDrawerTab::ResultsTable => {
                    self.render_bottom_results_table(ui, window)
                }
                StudioShellBottomDrawerTab::Diagnostics => {
                    self.render_bottom_diagnostics(ui, window)
                }
            });
    }

    fn render_bottom_messages(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        if let Some(notice) = self.project_open.notice.as_ref() {
            ui.colored_label(
                match notice.level {
                    ProjectOpenNoticeLevel::Info => egui::Color32::from_rgb(66, 118, 92),
                    ProjectOpenNoticeLevel::Warning => egui::Color32::from_rgb(160, 120, 40),
                    ProjectOpenNoticeLevel::Error => egui::Color32::from_rgb(180, 40, 40),
                },
                &notice.title,
            );
            render_wrapped_small(ui, &notice.detail);
            ui.add_space(4.0);
        }
        if let Some(failure) = window.runtime.latest_failure.as_ref() {
            self.render_latest_failure_summary(ui, failure);
            return;
        }
        if let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() {
            ui.horizontal_wrapped(|ui| {
                render_status_chip(
                    ui,
                    self.locale.runtime_label(snapshot.status_label).as_ref(),
                    run_status_color(snapshot.status_label),
                );
                render_wrapped_label(ui, &snapshot.summary);
            });
            if snapshot.diagnostics.is_empty() {
                ui.small(self.locale.text(ShellText::NoDiagnostics));
            } else {
                ui.small(self.locale.solve_snapshot_counts(
                    snapshot.stream_count,
                    snapshot.step_count,
                    snapshot.diagnostic_count,
                ));
            }
            return;
        }
        if let Some(message) = window.runtime.run_panel.view().latest_log_message.as_ref() {
            render_wrapped_label(ui, message);
        } else {
            ui.small(self.locale.text(ShellText::NoSolveSnapshot));
        }
    }

    fn render_bottom_run_log(&self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        if window.runtime.log_entries.is_empty() {
            ui.small(self.locale.text(ShellText::NoRuntimeLog));
            return;
        }
        for entry in window.runtime.log_entries.iter().rev().take(12) {
            render_wrapped_small(
                ui,
                format!("[{}] {}", log_level_label(entry.level), entry.message),
            );
        }
    }

    fn render_bottom_results_table(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() else {
            ui.small(self.locale.text(ShellText::NoVisibleSolveResults));
            return;
        };
        if snapshot.streams.is_empty() {
            ui.small(self.locale.text(ShellText::NoStreamResults));
            return;
        }
        egui::Grid::new(format!("bottom-results-table:{}", snapshot.snapshot_id))
            .num_columns(6)
            .striped(true)
            .min_col_width(92.0)
            .show(ui, |ui| {
                ui.strong("Stream");
                ui.strong("T (K)");
                ui.strong("P (Pa)");
                ui.strong("F (mol/s)");
                ui.strong("H (J/mol)");
                ui.strong("Phase");
                ui.end_row();
                for stream in &snapshot.streams {
                    let response = ui
                        .add(egui::Button::new(&stream.label).frame(false))
                        .on_hover_text(&stream.stream_id);
                    if response.clicked() {
                        self.result_inspector
                            .select_stream(&snapshot.snapshot_id, stream.stream_id.clone());
                        self.right_sidebar_tab = StudioShellRightSidebarTab::Results;
                    }
                    ui.label(format!("{:.2}", stream.temperature_k));
                    ui.label(format!("{:.0}", stream.pressure_pa));
                    ui.label(format!("{:.6}", stream.total_molar_flow_mol_s));
                    ui.label(
                        stream
                            .molar_enthalpy_j_per_mol
                            .map(|value| format!("{value:.3}"))
                            .unwrap_or_else(|| "-".to_string()),
                    );
                    ui.label(&stream.phase_text);
                    ui.end_row();
                }
            });
    }

    fn render_bottom_diagnostics(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        if let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() {
            if snapshot.diagnostics.is_empty() {
                ui.small(self.locale.text(ShellText::NoDiagnostics));
                return;
            }
            for (index, diagnostic) in snapshot.diagnostics.iter().enumerate() {
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
                if !diagnostic.diagnostic_actions.is_empty() {
                    self.render_diagnostic_target_actions(ui, &diagnostic.diagnostic_actions);
                }
                if index + 1 < snapshot.diagnostics.len() {
                    ui.separator();
                }
            }
            return;
        }
        if let Some(failure) = window.runtime.latest_failure.as_ref() {
            self.render_latest_failure_summary(ui, failure);
        } else {
            ui.small(self.locale.text(ShellText::NoDiagnostics));
        }
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

fn window_command_toolbar_item<'a>(
    commands: &'a radishflow_studio::StudioGuiWindowCommandAreaModel,
    command_id: &str,
) -> Option<&'a radishflow_studio::StudioGuiWindowToolbarItemModel> {
    commands
        .toolbar_sections
        .iter()
        .flat_map(|section| section.items.iter())
        .find(|item| item.command_id == command_id)
}
