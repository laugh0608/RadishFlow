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
                self.render_file_top_menu(ui, window);
                ui.separator();
                self.render_top_screen_navigation(ui, window);
                ui.separator();
                self.render_tools_top_menu(ui, windows, current_window_id, window);
                self.render_settings_top_menu(ui);
            });
            let context_toolbar = match self.screen {
                StudioShellScreen::Property => Some(&window.property_context_toolbar),
                StudioShellScreen::Workbench => Some(&window.flowsheet_context_toolbar),
                StudioShellScreen::Run => Some(&window.run_context_toolbar),
                StudioShellScreen::Results => Some(&window.result_context_toolbar),
                StudioShellScreen::Home => None,
            };
            if let Some(context_toolbar) = context_toolbar {
                ui.separator();
                self.render_context_toolbar(ui, context_toolbar);
            }
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
            if self.drag_session.is_none()
                && let Some(preview) = window.drop_preview.as_ref()
            {
                ui.separator();
                ui.small(
                    egui::RichText::new(format_compact_drop_preview_status(preview))
                        .color(egui::Color32::from_rgb(92, 104, 117)),
                );
            }
            if let Some(error) = self.platform_host.latest_gui_error_line() {
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(180, 40, 40), error);
            }
        });
    }

    fn render_context_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        toolbar: &radishflow_studio::StudioGuiWindowContextToolbarModel,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.small(
                egui::RichText::new(self.locale.runtime_label(toolbar.title).as_ref()).strong(),
            );
            for section in &toolbar.sections {
                if section.items.is_empty() {
                    continue;
                }
                if context_toolbar_section_is_top_bar_summary_only(toolbar.title, section.title) {
                    continue;
                }
                ui.separator();
                ui.small(
                    egui::RichText::new(self.locale.runtime_label(section.title).as_ref())
                        .color(egui::Color32::from_rgb(92, 104, 117)),
                );
                for item in &section.items {
                    self.render_context_toolbar_item(ui, toolbar.title, section.title, item);
                }
            }
        });
        if !toolbar.status_items.is_empty() {
            ui.horizontal_wrapped(|ui| {
                for status in &toolbar.status_items {
                    ui.small(format!(
                        "{}: {}",
                        self.locale.runtime_label(status.label),
                        self.locale.runtime_label(&status.value)
                    ));
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(&status.status_label).as_ref(),
                        context_toolbar_status_color(&status.status_label),
                    );
                }
            });
        }
    }

    fn render_context_toolbar_item(
        &mut self,
        ui: &mut egui::Ui,
        toolbar_title: &str,
        section_title: &str,
        item: &radishflow_studio::StudioGuiWindowContextToolbarItemModel,
    ) {
        let label = self.locale.runtime_label(&item.label);
        let detail = self.locale.runtime_label(&item.detail);
        let response = ui
            .add_enabled(item.enabled, egui::Button::new(label.as_ref()))
            .on_hover_text(detail.as_ref());
        if response.clicked() {
            match item.target {
                StudioGuiWindowContextToolbarItemTarget::Command => {
                    if let Some(command_id) = item.command_id.as_deref() {
                        self.dispatch_ui_command(command_id);
                    }
                }
                StudioGuiWindowContextToolbarItemTarget::FlowsheetModeling => {
                    self.enter_flowsheet_modeling_from_property();
                }
                StudioGuiWindowContextToolbarItemTarget::RunLog => {
                    self.focus_run_bottom_drawer_tab(StudioShellBottomDrawerTab::RunLog);
                }
                StudioGuiWindowContextToolbarItemTarget::Convergence => {
                    self.focus_run_bottom_drawer_tab(StudioShellBottomDrawerTab::Convergence);
                }
                StudioGuiWindowContextToolbarItemTarget::Suggestions => {
                    self.focus_run_bottom_drawer_tab(StudioShellBottomDrawerTab::Suggestions);
                }
                StudioGuiWindowContextToolbarItemTarget::Diagnostics => {
                    self.focus_run_bottom_drawer_tab(StudioShellBottomDrawerTab::Diagnostics);
                }
                StudioGuiWindowContextToolbarItemTarget::ModuleResults => {
                    if self.screen != StudioShellScreen::Results {
                        self.screen = StudioShellScreen::Workbench;
                    }
                    self.right_sidebar_tab = StudioShellRightSidebarTab::ModuleResults;
                }
                StudioGuiWindowContextToolbarItemTarget::ResultsTable => {
                    if self.screen == StudioShellScreen::Results {
                        self.bottom_drawer_tab = StudioShellBottomDrawerTab::ResultsTable;
                    } else {
                        self.focus_bottom_drawer_tab(StudioShellBottomDrawerTab::ResultsTable);
                    }
                }
            }
        }
        if let Some(status_label) = item.status_label.as_deref()
            && !context_toolbar_item_status_is_top_bar_summary_only(toolbar_title, section_title)
        {
            render_status_chip(
                ui,
                self.locale.runtime_label(status_label).as_ref(),
                context_toolbar_status_color(status_label),
            );
        }
    }

    fn render_file_top_menu(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.menu_button(self.locale.text(ShellText::File), |ui| {
            if ui
                .button(self.locale.text(ShellText::NewBlankProject))
                .clicked()
            {
                self.create_blank_project();
                ui.close_menu();
            }
            if ui
                .button(self.locale.text(ShellText::OpenProjectFromDisk))
                .clicked()
            {
                self.open_project_from_picker();
                ui.close_menu();
            }
            ui.menu_button(self.locale.text(ShellText::OpenExample), |ui| {
                self.render_example_project_top_menu(ui, window);
            });
            ui.separator();
            if ui
                .button(self.locale.text(ShellText::SaveProject))
                .clicked()
            {
                self.save_project();
                ui.close_menu();
            }
            if ui
                .button(self.locale.text(ShellText::SaveProjectAs))
                .clicked()
            {
                self.save_project_as_from_picker();
                ui.close_menu();
            }
        });
    }

    fn render_example_project_top_menu(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
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
    }

    fn render_top_screen_navigation(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.selectable_value(
            &mut self.screen,
            StudioShellScreen::Home,
            self.locale.text(ShellText::Home),
        );
        ui.selectable_value(
            &mut self.screen,
            StudioShellScreen::Property,
            self.locale.text(ShellText::Property),
        );
        let flowsheet_response = ui
            .add_enabled(
                window.property_page.flowsheet_modeling_enabled,
                egui::SelectableLabel::new(
                    self.screen == StudioShellScreen::Workbench,
                    self.locale.text(ShellText::Flowsheet),
                ),
            )
            .on_hover_text(
                self.locale
                    .runtime_label(&window.property_page.flowsheet_modeling_detail)
                    .as_ref(),
            );
        if flowsheet_response.clicked() {
            self.enter_flowsheet_modeling_from_property();
        }
        ui.selectable_value(
            &mut self.screen,
            StudioShellScreen::Run,
            self.locale.text(ShellText::Run),
        );
        ui.selectable_value(
            &mut self.screen,
            StudioShellScreen::Results,
            self.locale.text(ShellText::Results),
        );
    }

    fn render_tools_top_menu(
        &mut self,
        ui: &mut egui::Ui,
        windows: &[StudioAppHostWindowState],
        current_window_id: Option<StudioWindowHostId>,
        window: &StudioGuiWindowModel,
    ) {
        ui.menu_button(self.locale.text(ShellText::Tools), |ui| {
            self.render_tools_top_menu_content(ui, windows, current_window_id, window);
        });
    }

    fn render_settings_top_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button(self.locale.text(ShellText::Settings), |ui| {
            self.render_settings_top_menu_content(ui);
        });
    }

    pub(super) fn render_tools_top_menu_content(
        &mut self,
        ui: &mut egui::Ui,
        windows: &[StudioAppHostWindowState],
        current_window_id: Option<StudioWindowHostId>,
        window: &StudioGuiWindowModel,
    ) {
        ui.label(egui::RichText::new(self.locale.text(ShellText::Commands)).strong());
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
        ui.label(egui::RichText::new(self.locale.text(ShellText::LogicalWindows)).strong());
        if ui
            .button(self.locale.text(ShellText::NewLogicalWindow))
            .clicked()
        {
            self.dispatch_event(StudioGuiEvent::OpenWindowRequested);
            ui.close_menu();
        }
        if windows.len() > 1 {
            self.render_logical_window_chips(ui, windows);
        }
    }

    pub(super) fn render_settings_top_menu_content(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new(self.locale.text(ShellText::Language)).strong());
        let english = self.locale.text(ShellText::English);
        let chinese = self.locale.text(ShellText::Chinese);
        ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_value(&mut self.locale, StudioShellLocale::ZhCn, chinese)
                .clicked()
            {
                ui.close_menu();
            }
            if ui
                .selectable_value(&mut self.locale, StudioShellLocale::En, english)
                .clicked()
            {
                ui.close_menu();
            }
        });
    }

    fn focus_bottom_drawer_tab(&mut self, tab: StudioShellBottomDrawerTab) {
        self.screen = StudioShellScreen::Workbench;
        self.bottom_drawer_tab = tab;
    }

    fn focus_run_bottom_drawer_tab(&mut self, tab: StudioShellBottomDrawerTab) {
        self.screen = StudioShellScreen::Run;
        self.bottom_drawer_tab = tab;
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
        let height = workbench_bottom_drawer_height(self.bottom_drawer_tab, window);
        egui::TopBottomPanel::bottom("studio.bottom_drawer")
            .exact_height(height)
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
                    ui.small(
                        egui::RichText::new(self.locale.runtime_label("Run").as_ref()).strong(),
                    );
                    if let Some(metric) = window
                        .status_summary
                        .metrics
                        .iter()
                        .find(|metric| metric.label == "Run")
                    {
                        render_status_chip(
                            ui,
                            self.locale.runtime_label(&metric.status_label).as_ref(),
                            run_status_color(&metric.status_label),
                        );
                    }
                    ui.separator();
                    ui.small(
                        egui::RichText::new(self.locale.runtime_label("Snapshot").as_ref())
                            .strong(),
                    );
                    render_status_chip(
                        ui,
                        self.locale
                            .runtime_label(window.status_summary.snapshot_consistency_label)
                            .as_ref(),
                        context_toolbar_status_color(
                            window.status_summary.snapshot_consistency_label,
                        ),
                    );
                    ui.separator();
                    ui.small(self.locale.text(ShellText::UnitsSi));
                    ui.separator();
                    ui.small(self.locale.text(ShellText::SolverSequentialModular));
                    ui.separator();
                    ui.small(self.locale.text(ShellText::FlowsheetMode));
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
                StudioShellLeftSidebarTab::Palette,
                self.locale.text(ShellText::Palette),
            );
            ui.selectable_value(
                &mut self.left_sidebar_tab,
                StudioShellLeftSidebarTab::Project,
                self.locale.text(ShellText::Project),
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
                StudioShellLeftSidebarTab::Palette => self.render_canvas_palette(ui, window),
            });
    }

    fn render_project_navigator(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        let document = &window.runtime.workspace_document;
        ui.label(egui::RichText::new(&document.flowsheet_name).strong());
        render_wrapped_small(ui, &document.title);
        ui.add_space(8.0);

        self.render_project_section_header(
            ui,
            match self.locale {
                StudioShellLocale::En => "Project inputs",
                StudioShellLocale::ZhCn => "项目输入",
            },
            match self.locale {
                StudioShellLocale::En => "Readiness inputs come from the current project document.",
                StudioShellLocale::ZhCn => "建模输入摘要来自当前项目文档。",
            },
        );
        self.render_project_inputs_section(ui, document);
        ui.add_space(8.0);

        self.render_project_section_header(
            ui,
            match self.locale {
                StudioShellLocale::En => "Example entry",
                StudioShellLocale::ZhCn => "示例入口",
            },
            match self.locale {
                StudioShellLocale::En => {
                    "Bundled examples stay discoverable without becoming project state."
                }
                StudioShellLocale::ZhCn => "内置示例保持可发现，但不成为项目状态。",
            },
        );
        self.render_project_examples_section(ui, window);
        ui.add_space(8.0);

        self.render_project_section_header(
            ui,
            match self.locale {
                StudioShellLocale::En => "Object tree",
                StudioShellLocale::ZhCn => "对象树",
            },
            match self.locale {
                StudioShellLocale::En => {
                    "Select a stream or unit to focus the canvas and inspector."
                }
                StudioShellLocale::ZhCn => "选择流股或单元会聚焦画布和右侧检查器。",
            },
        );
        self.render_project_objects_section(ui, window, document);
        ui.add_space(8.0);

        self.render_project_section_header(
            ui,
            match self.locale {
                StudioShellLocale::En => "Review status",
                StudioShellLocale::ZhCn => "审阅状态",
            },
            match self.locale {
                StudioShellLocale::En => "Results and diagnostics read the current run state.",
                StudioShellLocale::ZhCn => "结果和诊断只读消费当前运行状态。",
            },
        );
        self.render_project_review_section(ui, window);
    }

    fn render_project_inputs_section(
        &mut self,
        ui: &mut egui::Ui,
        document: &radishflow_studio::StudioGuiWorkspaceDocumentSnapshot,
    ) {
        let property_package_summary = document
            .property_package_choices
            .iter()
            .find(|choice| choice.selected)
            .map(|choice| choice.label.as_str())
            .or(document.property_package_id.as_deref())
            .unwrap_or("Unselected");
        let localized_property_package_summary =
            self.locale.runtime_label(property_package_summary);
        self.render_project_tree_row(
            ui,
            self.locale.text(ShellText::PropertyPackage),
            localized_property_package_summary.as_ref(),
            None,
        );
        ui.add_space(6.0);
        self.render_project_components(ui, document);
    }

    fn render_project_objects_section(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
        document: &radishflow_studio::StudioGuiWorkspaceDocumentSnapshot,
    ) {
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
    }

    fn render_project_review_section(&self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
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

    fn render_project_section_header(&self, ui: &mut egui::Ui, title: &str, detail: &str) {
        ui.separator();
        ui.label(egui::RichText::new(title).strong());
        render_wrapped_small(ui, detail);
        ui.add_space(4.0);
    }

    fn render_project_examples_section(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        egui::CollapsingHeader::new(self.locale.text(ShellText::ExampleProjects))
            .default_open(false)
            .show(ui, |ui| {
                render_wrapped_small(
                    ui,
                    match self.locale {
                        StudioShellLocale::En => {
                            "Bundled examples open into the current workbench."
                        }
                        StudioShellLocale::ZhCn => "内置示例会打开到当前工作台。",
                    },
                );
                ui.add_space(4.0);

                if window.runtime.example_projects.is_empty() {
                    ui.small(self.locale.text(ShellText::NoRecentProjects));
                    return;
                }

                for example in &window.runtime.example_projects {
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
                    ui.add_space(6.0);
                }
            });
    }

    fn render_project_components(
        &mut self,
        ui: &mut egui::Ui,
        document: &radishflow_studio::StudioGuiWorkspaceDocumentSnapshot,
    ) {
        self.render_project_tree_row(
            ui,
            match self.locale {
                StudioShellLocale::En => "Project components",
                StudioShellLocale::ZhCn => "项目组分",
            },
            "",
            Some(
                document
                    .project_component_choices
                    .iter()
                    .filter(|choice| choice.selected)
                    .count(),
            ),
        );

        for component in &document.project_component_choices {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(&component.name).strong());
                if let Some(formula) = component.formula.as_ref() {
                    ui.small(formula);
                }
                let status = if component.selected {
                    match self.locale {
                        StudioShellLocale::En => "Selected",
                        StudioShellLocale::ZhCn => "已选择",
                    }
                } else {
                    match self.locale {
                        StudioShellLocale::En => "Available",
                        StudioShellLocale::ZhCn => "可选",
                    }
                };
                ui.small(status);
            });

            if component.selected {
                let remove_label = match self.locale {
                    StudioShellLocale::En => "Remove",
                    StudioShellLocale::ZhCn => "移除",
                };
                if ui
                    .add_enabled(component.remove_enabled, egui::Button::new(remove_label))
                    .on_hover_text(&component.remove_detail)
                    .clicked()
                {
                    self.dispatch_ui_command(&component.remove_command_id);
                }
            } else {
                let select_label = match self.locale {
                    StudioShellLocale::En => "Select",
                    StudioShellLocale::ZhCn => "选择",
                };
                if ui
                    .button(select_label)
                    .on_hover_text(&component.component_id)
                    .clicked()
                {
                    self.dispatch_ui_command(&component.select_command_id);
                }
            }
        }
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
        if let Some(summary) = item.attention_summary.as_ref() {
            render_wrapped_small(ui, summary);
        }
    }

    fn render_canvas_palette(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        let palette = &window.canvas.widget.view().place_unit_palette;
        ui.label(egui::RichText::new(self.locale.runtime_label(palette.title).as_ref()).strong());
        render_wrapped_small(
            ui,
            match self.locale {
                StudioShellLocale::En => "Use the supported module set to build small flowsheets.",
                StudioShellLocale::ZhCn => "使用当前受控模块集搭建小流程。",
            },
        );
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

        ui.add(
            egui::TextEdit::singleline(&mut self.module_palette_filter)
                .hint_text(match self.locale {
                    StudioShellLocale::En => "Filter modules",
                    StudioShellLocale::ZhCn => "筛选模块",
                })
                .desired_width(ui.available_width()),
        );
        ui.add_space(6.0);

        self.render_authoring_checklists(ui, window);
        ui.add_space(8.0);

        let normalized_filter = normalized_module_palette_filter(&self.module_palette_filter);
        let mut visible_options = 0usize;
        for category in module_palette_categories(self.locale) {
            let category_options = palette
                .options
                .iter()
                .filter(|option| category.matches(option.kind))
                .filter(|option| module_palette_option_matches(option, &normalized_filter))
                .collect::<Vec<_>>();

            if category_options.is_empty() {
                continue;
            }

            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(category.title).strong())
                    .on_hover_text(category.detail);
                render_status_chip(
                    ui,
                    &category_options.len().to_string(),
                    egui::Color32::from_rgb(86, 96, 108),
                );
            });

            for option in category_options {
                visible_options += 1;
                self.render_module_palette_option(ui, option);
            }
        }

        if visible_options == 0 {
            ui.separator();
            render_wrapped_small(
                ui,
                match self.locale {
                    StudioShellLocale::En => "No supported modules match the current filter.",
                    StudioShellLocale::ZhCn => "当前筛选没有匹配的受控模块。",
                },
            );
        }

        let suggestions = &window.canvas.widget.view().suggestions;
        if !suggestions.is_empty() {
            ui.separator();
            ui.label(
                egui::RichText::new(match self.locale {
                    StudioShellLocale::En => "Canvas suggestions",
                    StudioShellLocale::ZhCn => "画布建议",
                })
                .strong(),
            );
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
                        egui::Button::new(
                            self.locale.runtime_label(suggestion.action_label).as_ref(),
                        ),
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

    fn render_module_palette_option(
        &mut self,
        ui: &mut egui::Ui,
        option: &radishflow_studio::StudioGuiCanvasPlaceUnitOptionViewModel,
    ) {
        let option_label = self.locale.runtime_label(&option.label);
        let option_detail = self.locale.runtime_label(&option.detail);
        ui.horizontal_wrapped(|ui| {
            let response = ui
                .add_enabled(
                    option.enabled,
                    egui::Button::new(option_label.as_ref())
                        .selected(option.active)
                        .min_size(egui::vec2(ui.available_width().min(170.0), 24.0)),
                )
                .on_hover_text(option_detail.as_ref());
            if response.clicked() {
                self.dispatch_ui_command(&option.command_id);
            }
            if option.active {
                render_status_chip(
                    ui,
                    match self.locale {
                        StudioShellLocale::En => "Active",
                        StudioShellLocale::ZhCn => "活动",
                    },
                    egui::Color32::from_rgb(52, 128, 89),
                );
            }
        });
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
                StudioShellRightSidebarTab::ModuleSettings,
                self.locale.runtime_label("Module Settings").as_ref(),
            );
            ui.selectable_value(
                &mut self.right_sidebar_tab,
                StudioShellRightSidebarTab::ModuleResults,
                self.locale.runtime_label("Module Results").as_ref(),
            );
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt(format!(
                "scroll:{}:right-workbench",
                window.layout_state.scope.layout_key
            ))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.render_right_sidebar_selection_context(ui, window);
                match self.right_sidebar_tab {
                    StudioShellRightSidebarTab::Inspector => {
                        self.render_runtime_inspector_tab(ui, window)
                    }
                    StudioShellRightSidebarTab::ModuleSettings => {
                        self.render_runtime_module_settings_tab(ui, window)
                    }
                    StudioShellRightSidebarTab::ModuleResults => {
                        self.render_runtime_module_results_tab(ui, window)
                    }
                }
            });
    }

    fn render_project_operation_strip(&mut self, ui: &mut egui::Ui) {
        if self.project_open.pending_confirmation.is_none()
            && !self.project_open.pending_blank_project_confirmation
            && self.project_open.pending_save_as_overwrite.is_none()
            && self
                .project_open
                .pending_close_window_confirmation
                .is_none()
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
            if self.project_open.pending_blank_project_confirmation {
                if ui
                    .button(self.locale.text(ShellText::ContinueNewBlankProject))
                    .clicked()
                {
                    self.confirm_pending_blank_project();
                }
                if ui
                    .button(self.locale.text(ShellText::CancelNewBlankProject))
                    .clicked()
                {
                    self.cancel_pending_blank_project();
                }
            }
            if self
                .project_open
                .pending_close_window_confirmation
                .is_some()
            {
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
        let available_width = ui.available_width();
        let status_summary_width = if available_width >= 960.0 {
            320.0
        } else {
            (available_width * 0.34).clamp(240.0, 320.0)
        };
        let run_information_width = (available_width - status_summary_width - 18.0).max(420.0);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(run_information_width);
                self.render_bottom_run_information(ui, window);
            });
            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.set_width(status_summary_width);
                self.render_bottom_status_summary_card(ui, window);
            });
        });
    }

    fn render_bottom_run_information(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
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
                StudioShellBottomDrawerTab::Convergence,
                self.locale.runtime_label("Convergence").as_ref(),
            );
            ui.selectable_value(
                &mut self.bottom_drawer_tab,
                StudioShellBottomDrawerTab::Suggestions,
                self.locale.text(ShellText::Suggestions),
            );
            ui.selectable_value(
                &mut self.bottom_drawer_tab,
                StudioShellBottomDrawerTab::Diagnostics,
                self.locale.text(ShellText::Diagnostics),
            );
            ui.selectable_value(
                &mut self.bottom_drawer_tab,
                StudioShellBottomDrawerTab::ResultsTable,
                self.locale.text(ShellText::ResultsTable),
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
                StudioShellBottomDrawerTab::Convergence => {
                    self.render_bottom_convergence(ui, window)
                }
                StudioShellBottomDrawerTab::Suggestions => {
                    self.render_bottom_suggestions(ui, window)
                }
                StudioShellBottomDrawerTab::Diagnostics => {
                    self.render_bottom_diagnostics(ui, window)
                }
                StudioShellBottomDrawerTab::ResultsTable => {
                    self.render_bottom_results_table(ui, window)
                }
            });
    }

    fn render_bottom_status_summary_card(&self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(
                            self.locale
                                .runtime_label(window.status_summary.title)
                                .as_ref(),
                        )
                        .strong(),
                    );
                    ui.separator();
                    ui.small(
                        egui::RichText::new(self.locale.runtime_label("Snapshot").as_ref())
                            .strong(),
                    );
                    render_status_chip(
                        ui,
                        self.locale
                            .runtime_label(window.status_summary.snapshot_consistency_label)
                            .as_ref(),
                        context_toolbar_status_color(
                            window.status_summary.snapshot_consistency_label,
                        ),
                    );
                });
                ui.add_space(4.0);
                egui::Grid::new("bottom-status-summary-metrics")
                    .num_columns(3)
                    .spacing([8.0, 3.0])
                    .show(ui, |ui| {
                        for metric in &window.status_summary.metrics {
                            ui.small(
                                egui::RichText::new(
                                    self.locale.runtime_label(metric.label).as_ref(),
                                )
                                .strong(),
                            );
                            render_status_chip(
                                ui,
                                self.locale.runtime_label(&metric.status_label).as_ref(),
                                run_status_color(&metric.status_label),
                            );
                            if metric.value != metric.status_label {
                                render_wrapped_small(
                                    ui,
                                    self.locale.runtime_label(&metric.value).as_ref(),
                                );
                            } else {
                                ui.small("");
                            }
                            ui.end_row();
                        }
                    });
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
                render_wrapped_label(
                    ui,
                    self.locale.solve_snapshot_primary_summary(
                        window.runtime.workspace_document.unit_count,
                        snapshot.diagnostic_count,
                        snapshot.stream_count,
                    ),
                );
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

    fn render_bottom_convergence(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(self.locale.runtime_label("Convergence").as_ref()).strong(),
            );
            render_status_chip(
                ui,
                self.locale
                    .runtime_label(window.status_summary.snapshot_consistency_label)
                    .as_ref(),
                run_status_color(window.status_summary.snapshot_consistency_label),
            );
        });
        ui.add_space(4.0);
        let current_snapshot = window.runtime.latest_solve_snapshot.as_ref();
        if let Some(snapshot) = current_snapshot {
            render_wrapped_label(
                ui,
                self.locale.solve_snapshot_primary_summary(
                    window.runtime.workspace_document.unit_count,
                    snapshot.diagnostic_count,
                    snapshot.stream_count,
                ),
            );
            render_wrapped_small(
                ui,
                self.locale
                    .snapshot_identity(&snapshot.snapshot_id, snapshot.sequence),
            );
            ui.small(self.locale.solve_snapshot_counts(
                snapshot.stream_count,
                snapshot.step_count,
                snapshot.diagnostic_count,
            ));
            ui.add_space(6.0);
        }
        egui::Grid::new("bottom-convergence-summary")
            .num_columns(3)
            .spacing([8.0, 3.0])
            .striped(true)
            .show(ui, |ui| {
                for metric in &window.status_summary.metrics {
                    if !matches!(
                        metric.label,
                        "Run" | "Convergence" | "Steps" | "Diagnostics"
                    ) {
                        continue;
                    }
                    ui.small(
                        egui::RichText::new(self.locale.runtime_label(metric.label).as_ref())
                            .strong(),
                    );
                    render_status_chip(
                        ui,
                        self.locale.runtime_label(&metric.status_label).as_ref(),
                        run_status_color(&metric.status_label),
                    );
                    render_wrapped_small(ui, self.locale.runtime_label(&metric.value).as_ref());
                    ui.end_row();
                }
            });

        if current_snapshot.is_some() {
            return;
        }
        ui.add_space(6.0);
        if let Some(failure) = window.runtime.latest_failure.as_ref() {
            self.render_latest_failure_summary(ui, failure);
            return;
        }
        if let Some(stale_snapshot) = window.runtime.stale_solve_snapshot.as_ref() {
            self.render_stale_solve_snapshot_notice(ui, stale_snapshot);
            return;
        }
        if let Some(summary) = window
            .runtime
            .run_panel
            .view()
            .latest_snapshot_summary
            .as_ref()
        {
            render_wrapped_label(ui, self.locale.runtime_label(summary).as_ref());
        } else {
            ui.small(self.locale.text(ShellText::NoSolveSnapshot));
        }
    }

    fn render_bottom_suggestions(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        let canvas_view = window.canvas.widget.view();
        let run_panel_view = window.runtime.run_panel.view();
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(self.locale.text(ShellText::Suggestions)).strong());
            render_status_chip(
                ui,
                &canvas_view.suggestion_count.to_string(),
                egui::Color32::from_rgb(86, 118, 168),
            );
        });

        let mut rendered_any = false;
        if let Some(notice) = run_panel_view.notice.as_ref() {
            rendered_any = true;
            ui.add_space(4.0);
            ui.colored_label(notice_color(notice.level), &notice.title);
            render_wrapped_label(ui, &notice.message);
            if let Some(recovery_action) = notice.recovery_action.as_ref() {
                render_wrapped_small(ui, recovery_action.detail);
                if ui.button(recovery_action.title).clicked() {
                    match window.runtime.run_panel.activate_recovery_action() {
                        RunPanelRecoveryWidgetEvent::Requested { .. } => {
                            self.dispatch_ui_command("run_panel.recover_failure");
                        }
                        RunPanelRecoveryWidgetEvent::Missing => {}
                    }
                }
            }
        }

        for suggestion in canvas_view.suggestions.iter().take(6) {
            rendered_any = true;
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                render_status_chip(
                    ui,
                    self.locale.runtime_label(suggestion.status_label).as_ref(),
                    egui::Color32::from_rgb(86, 118, 168),
                );
                ui.small(format!("{:.0}%", suggestion.confidence * 100.0));
                if suggestion.is_focused {
                    render_status_chip(
                        ui,
                        self.locale.runtime_label("Focused").as_ref(),
                        egui::Color32::from_rgb(66, 118, 92),
                    );
                }
            });
            render_wrapped_small(ui, &suggestion.reason);
            if ui
                .add_enabled(
                    suggestion.explicit_accept_enabled,
                    egui::Button::new(self.locale.runtime_label(suggestion.action_label).as_ref()),
                )
                .on_hover_text(&suggestion.id)
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
                    | radishflow_studio::StudioGuiCanvasWidgetEvent::SuggestionMissing { .. }
                    | radishflow_studio::StudioGuiCanvasWidgetEvent::Requested { .. }
                    | radishflow_studio::StudioGuiCanvasWidgetEvent::Disabled { .. }
                    | radishflow_studio::StudioGuiCanvasWidgetEvent::Missing { .. } => {}
                }
            }
        }

        if !rendered_any {
            ui.small(match self.locale {
                StudioShellLocale::En => "No current suggestions.",
                StudioShellLocale::ZhCn => "暂无建议。",
            });
        }
    }

    pub(in crate::studio_gui_shell) fn render_bottom_results_table(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        let Some(snapshot) = window.runtime.latest_solve_snapshot.as_ref() else {
            if let Some(stale_snapshot) = window.runtime.stale_solve_snapshot.as_ref() {
                self.render_stale_solve_snapshot_notice(ui, stale_snapshot);
                return;
            }
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
            .min_col_width(78.0)
            .show(ui, |ui| {
                ui.strong(result_table_header(self.locale, ResultTableHeader::Stream));
                ui.strong("T (K)");
                ui.strong("P (Pa)");
                ui.strong("F (mol/s)");
                ui.strong("H (J/mol)");
                ui.strong(result_table_header(self.locale, ResultTableHeader::Phase));
                ui.end_row();
                for stream in &snapshot.streams {
                    let response = ui
                        .add(egui::Button::new(&stream.label).frame(false))
                        .on_hover_text(&stream.stream_id);
                    if response.clicked() {
                        self.focus_result_table_stream(
                            &snapshot.snapshot_id,
                            stream.stream_id.clone(),
                        );
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
                    ui.label(result_table_phase_summary(self.locale, stream))
                        .on_hover_text(&stream.phase_text);
                    ui.end_row();
                }
            });

        if !snapshot.review_summary.unit_results.is_empty() {
            ui.add_space(8.0);
            ui.strong(self.locale.text(ShellText::Units));
            egui::Grid::new(format!(
                "bottom-results-units-table:{}",
                snapshot.snapshot_id
            ))
            .num_columns(5)
            .striped(true)
            .min_col_width(78.0)
            .show(ui, |ui| {
                ui.strong(result_table_header(self.locale, ResultTableHeader::Unit));
                ui.strong(result_table_header(self.locale, ResultTableHeader::Status));
                ui.strong(result_table_header(self.locale, ResultTableHeader::Step));
                ui.strong(self.locale.text(ShellText::InspectorConsumedStreams));
                ui.strong(self.locale.text(ShellText::InspectorProducedStreams));
                ui.end_row();

                for unit in &snapshot.review_summary.unit_results {
                    let unit_response = ui
                        .add(egui::Button::new(&unit.unit_id).frame(false))
                        .on_hover_text(&unit.summary);
                    if unit_response.clicked() {
                        self.focus_result_table_unit(&snapshot.snapshot_id, unit.unit_id.clone());
                    }
                    ui.label(self.locale.runtime_label(unit.status_label).as_ref());
                    ui.label(format!("#{}", unit.step_index));
                    render_wrapped_small(ui, result_table_stream_ids(&unit.consumed_stream_ids));
                    render_wrapped_small(ui, result_table_stream_ids(&unit.produced_stream_ids));
                    ui.end_row();
                }
            });
        }
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

    pub(super) fn render_property_page(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_property_workspace_page(ui, window);
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
            if new_stack_insert_group_index == Some(group_index)
                && let Some(preview) = region_preview
            {
                let rect = render_new_stack_insert_overlay(ui, preview);
                self.record_drop_preview_overlay_anchor(
                    rect,
                    drop_preview_anchor_priority_new_stack(),
                );
                ui.add_space(8.0);
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
                        if let Some(drag_session) = drag_session
                            && let Some(query) =
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

        if new_stack_insert_group_index == Some(groups.len())
            && let Some(preview) = region_preview
        {
            let rect = render_new_stack_insert_overlay(ui, preview);
            self.record_drop_preview_overlay_anchor(rect, drop_preview_anchor_priority_new_stack());
            ui.add_space(8.0);
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
                        ui.label(
                            egui::RichText::new(self.locale.runtime_label(panel.title).as_ref())
                                .strong(),
                        );
                        if let Some(badge) = visible_panel_badge(area_id, panel.badge.as_deref()) {
                            ui.label(format!("[{badge}]"));
                        }
                        for badge in &preview_badges {
                            ui.label(
                                egui::RichText::new(format!("[{badge}]"))
                                    .small()
                                    .color(egui::Color32::from_rgb(56, 126, 214)),
                            );
                        }
                        if let Some(summary) =
                            visible_panel_summary(area_id, &panel.summary, self.locale)
                        {
                            ui.label(summary.as_ref());
                        }
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
                ui.label(
                    egui::RichText::new(self.locale.runtime_label(panel.title).as_ref()).strong(),
                );
                if let Some(badge) = visible_panel_badge(area_id, panel.badge.as_deref()) {
                    ui.label(format!("[{badge}]"));
                }
                for badge in &preview_badges {
                    ui.label(
                        egui::RichText::new(format!("[{badge}]"))
                            .small()
                            .color(egui::Color32::from_rgb(56, 126, 214)),
                    );
                }
                if let Some(summary) = visible_panel_summary(area_id, &panel.summary, self.locale) {
                    ui.label(summary.as_ref());
                }
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

fn workbench_bottom_drawer_height(
    tab: StudioShellBottomDrawerTab,
    _window: &StudioGuiWindowModel,
) -> f32 {
    match tab {
        StudioShellBottomDrawerTab::Messages => 156.0,
        StudioShellBottomDrawerTab::RunLog => 170.0,
        StudioShellBottomDrawerTab::Convergence => 230.0,
        StudioShellBottomDrawerTab::ResultsTable => 250.0,
        StudioShellBottomDrawerTab::Suggestions | StudioShellBottomDrawerTab::Diagnostics => 190.0,
    }
}

#[derive(Debug, Clone, Copy)]
struct ModulePaletteCategory {
    title: &'static str,
    detail: &'static str,
    kinds: &'static [radishflow_studio::StudioGuiCanvasPlaceUnitKind],
}

impl ModulePaletteCategory {
    fn matches(self, kind: radishflow_studio::StudioGuiCanvasPlaceUnitKind) -> bool {
        self.kinds.contains(&kind)
    }
}

const MODULE_PALETTE_STREAM_SOURCES: &[radishflow_studio::StudioGuiCanvasPlaceUnitKind] =
    &[radishflow_studio::StudioGuiCanvasPlaceUnitKind::Feed];
const MODULE_PALETTE_CONDITIONING_UNITS: &[radishflow_studio::StudioGuiCanvasPlaceUnitKind] = &[
    radishflow_studio::StudioGuiCanvasPlaceUnitKind::Heater,
    radishflow_studio::StudioGuiCanvasPlaceUnitKind::Cooler,
    radishflow_studio::StudioGuiCanvasPlaceUnitKind::Valve,
];
const MODULE_PALETTE_MIXING_AND_SEPARATION: &[radishflow_studio::StudioGuiCanvasPlaceUnitKind] = &[
    radishflow_studio::StudioGuiCanvasPlaceUnitKind::Mixer,
    radishflow_studio::StudioGuiCanvasPlaceUnitKind::FlashDrum,
];

fn module_palette_categories(locale: StudioShellLocale) -> [ModulePaletteCategory; 3] {
    match locale {
        StudioShellLocale::En => [
            ModulePaletteCategory {
                title: "Stream sources",
                detail: "Create material feeds that define composition and source conditions.",
                kinds: MODULE_PALETTE_STREAM_SOURCES,
            },
            ModulePaletteCategory {
                title: "Conditioning units",
                detail: "Adjust temperature, pressure, or pressure loss before separation.",
                kinds: MODULE_PALETTE_CONDITIONING_UNITS,
            },
            ModulePaletteCategory {
                title: "Mixing and separation",
                detail: "Combine feeds and split phases with the supported flowsheet units.",
                kinds: MODULE_PALETTE_MIXING_AND_SEPARATION,
            },
        ],
        StudioShellLocale::ZhCn => [
            ModulePaletteCategory {
                title: "流股源",
                detail: "创建定义组成和源条件的物料进料。",
                kinds: MODULE_PALETTE_STREAM_SOURCES,
            },
            ModulePaletteCategory {
                title: "调节单元",
                detail: "在分离前调整温度、压力或压降。",
                kinds: MODULE_PALETTE_CONDITIONING_UNITS,
            },
            ModulePaletteCategory {
                title: "汇合与分离",
                detail: "用当前支持的单元完成进料汇合和相态分离。",
                kinds: MODULE_PALETTE_MIXING_AND_SEPARATION,
            },
        ],
    }
}

fn normalized_module_palette_filter(filter: &str) -> String {
    filter.trim().to_ascii_lowercase()
}

fn module_palette_option_matches(
    option: &radishflow_studio::StudioGuiCanvasPlaceUnitOptionViewModel,
    normalized_filter: &str,
) -> bool {
    if normalized_filter.is_empty() {
        return true;
    }

    option
        .label
        .to_ascii_lowercase()
        .contains(normalized_filter)
        || option
            .detail
            .to_ascii_lowercase()
            .contains(normalized_filter)
        || option
            .unit_kind
            .to_ascii_lowercase()
            .contains(normalized_filter)
        || option
            .search_terms
            .iter()
            .any(|term| term.to_ascii_lowercase().contains(normalized_filter))
}

fn context_toolbar_status_color(status_label: &str) -> egui::Color32 {
    match status_label {
        "Selected" | "Available" | "Current" | "Ready" => egui::Color32::from_rgb(54, 128, 84),
        "Unselected" | "SnapshotMissing" | "Stale" | "Incomplete" => {
            egui::Color32::from_rgb(180, 120, 20)
        }
        _ => run_status_color(status_label),
    }
}

fn context_toolbar_section_is_top_bar_summary_only(
    toolbar_title: &str,
    section_title: &str,
) -> bool {
    toolbar_title == "Result Context" && section_title == "Focus"
}

fn context_toolbar_item_status_is_top_bar_summary_only(
    toolbar_title: &str,
    section_title: &str,
) -> bool {
    toolbar_title == "Run Context" && section_title == "Monitor"
}

#[derive(Debug, Clone, Copy)]
enum ResultTableHeader {
    Stream,
    Unit,
    Status,
    Step,
    Phase,
}

fn result_table_header(locale: StudioShellLocale, header: ResultTableHeader) -> &'static str {
    match locale {
        StudioShellLocale::En => match header {
            ResultTableHeader::Stream => "Stream",
            ResultTableHeader::Unit => "Unit",
            ResultTableHeader::Status => "Status",
            ResultTableHeader::Step => "Step",
            ResultTableHeader::Phase => "Phase",
        },
        StudioShellLocale::ZhCn => match header {
            ResultTableHeader::Stream => "流股",
            ResultTableHeader::Unit => "单元",
            ResultTableHeader::Status => "状态",
            ResultTableHeader::Step => "步骤",
            ResultTableHeader::Phase => "相态",
        },
    }
}

fn result_table_stream_ids(stream_ids: &[String]) -> String {
    if stream_ids.is_empty() {
        return "-".to_string();
    }

    stream_ids.join(", ")
}

fn result_table_phase_summary(
    locale: StudioShellLocale,
    stream: &radishflow_studio::StudioGuiWindowStreamResultModel,
) -> String {
    if stream.phase_rows.is_empty() {
        return locale.text(ShellText::NoneValue).to_string();
    }

    let visible_rows = stream
        .phase_rows
        .iter()
        .filter(|row| row.label != "overall")
        .collect::<Vec<_>>();
    let visible_rows = if visible_rows.is_empty() {
        stream.phase_rows.iter().collect::<Vec<_>>()
    } else {
        visible_rows
    };
    let hidden_count = visible_rows.len().saturating_sub(2);
    let mut parts = visible_rows
        .iter()
        .take(2)
        .map(|row| {
            format!(
                "{} {:.3}",
                locale.runtime_label(&row.label),
                row.phase_fraction
            )
        })
        .collect::<Vec<_>>();
    if hidden_count > 0 {
        parts.push(format!("+{hidden_count}"));
    }
    parts.join(" / ")
}

fn visible_panel_badge(area_id: StudioGuiWindowAreaId, badge: Option<&str>) -> Option<&str> {
    let badge = badge?;
    match area_id {
        StudioGuiWindowAreaId::Canvas | StudioGuiWindowAreaId::Runtime if badge == "0" => None,
        _ => Some(badge),
    }
}

fn visible_panel_summary<'a>(
    area_id: StudioGuiWindowAreaId,
    summary: &'a str,
    locale: StudioShellLocale,
) -> Option<std::borrow::Cow<'a, str>> {
    match area_id {
        StudioGuiWindowAreaId::Canvas
            if summary.contains("suggestions") && summary.contains("actions enabled") =>
        {
            None
        }
        StudioGuiWindowAreaId::Runtime if summary.contains("status=") => None,
        StudioGuiWindowAreaId::Commands if summary.contains("commands") => None,
        _ if summary.is_empty() => None,
        _ => Some(locale.runtime_label(summary)),
    }
}
