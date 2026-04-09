use std::time::{Duration, SystemTime};

use eframe::egui;
use radishflow_studio::{
    StudioAppHostWindowState, StudioGuiCommandEntry, StudioGuiDriver, StudioGuiDriverDispatch,
    StudioGuiDriverOutcome, StudioGuiEvent, StudioGuiFocusContext, StudioGuiShortcut,
    StudioGuiShortcutKey, StudioGuiShortcutModifier, StudioGuiWindowAreaId,
    StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion, StudioGuiWindowDropTargetQuery,
    StudioGuiWindowLayoutModel, StudioGuiWindowLayoutMutation, StudioGuiWindowModel,
    StudioGuiWindowPanelDisplayMode, StudioGuiWindowStackGroupLayout, StudioRuntimeConfig,
    StudioWindowHostId, StudioWindowHostRole,
};
use rf_ui::{
    RunPanelIntent, RunPanelNoticeLevel, RunPanelRecoveryWidgetEvent, RunPanelWidgetEvent,
    SimulationMode,
};

pub fn run() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "RadishFlow Studio",
        native_options,
        Box::new(|_cc| Ok(Box::new(RadishFlowStudioApp::new()))),
    )
}

struct RadishFlowStudioApp {
    state: AppState,
}

enum AppState {
    Ready(ReadyAppState),
    Failed(String),
}

struct ReadyAppState {
    driver: StudioGuiDriver,
    activity_log: Vec<String>,
    last_error: Option<String>,
    drag_session: Option<PanelDragSession>,
    active_drop_preview: Option<ActiveDropPreview>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PanelDragSession {
    area_id: StudioGuiWindowAreaId,
    window_id: Option<StudioWindowHostId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActiveDropPreview {
    window_id: Option<StudioWindowHostId>,
    query: StudioGuiWindowDropTargetQuery,
}

impl RadishFlowStudioApp {
    fn new() -> Self {
        let config = StudioRuntimeConfig::default();
        let state = match StudioGuiDriver::new(&config) {
            Ok(driver) => {
                let mut ready = ReadyAppState {
                    driver,
                    activity_log: Vec::new(),
                    last_error: None,
                    drag_session: None,
                    active_drop_preview: None,
                };
                ready.dispatch_event(StudioGuiEvent::OpenWindowRequested);
                AppState::Ready(ready)
            }
            Err(error) => AppState::Failed(format!(
                "Studio 初始化失败 [{}]: {}",
                error.code().as_str(),
                error.message()
            )),
        };

        Self { state }
    }
}

impl eframe::App for RadishFlowStudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match &mut self.state {
            AppState::Failed(message) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("RadishFlow Studio");
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(180, 40, 40), message);
                });
            }
            AppState::Ready(app) => app.update(ctx),
        }
    }
}

impl ReadyAppState {
    fn update(&mut self, ctx: &egui::Context) {
        self.dispatch_shortcuts(ctx);
        self.drain_due_timers(ctx);

        let snapshot = self.driver.snapshot();
        let window = snapshot.window_model();
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
        self.finish_drop_preview_cycle(window.layout_state.scope.window_id, hovered_drop_target);
    }

    fn render_top_bar(
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
                ui.label(&window.header.status_line);
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                if ui.button("Open window").clicked() {
                    self.dispatch_event(StudioGuiEvent::OpenWindowRequested);
                }
                if let Some(window_id) = current_window_id {
                    if ui.button("Close window").clicked() {
                        self.dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id });
                    }
                    if ui.button("Foreground current").clicked() {
                        self.dispatch_event(StudioGuiEvent::WindowForegrounded { window_id });
                    }
                }
                if ui.button("Login completed").clicked() {
                    self.dispatch_event(StudioGuiEvent::LoginCompleted);
                }
                if ui.button("Network restored").clicked() {
                    self.dispatch_event(StudioGuiEvent::NetworkRestored);
                }
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Logical windows").strong());
                if windows.is_empty() {
                    ui.small("none");
                } else {
                    for window_state in windows {
                        let label = format_window_chip(window_state);
                        if ui
                            .selectable_label(window_state.is_foreground, label)
                            .clicked()
                        {
                            self.dispatch_event(StudioGuiEvent::WindowForegrounded {
                                window_id: window_state.window_id,
                            });
                        }
                    }
                }
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                self.render_panel_toggle(
                    ui,
                    current_window_id,
                    &window.layout_state,
                    StudioGuiWindowAreaId::Commands,
                    "Commands",
                );
                self.render_panel_toggle(
                    ui,
                    current_window_id,
                    &window.layout_state,
                    StudioGuiWindowAreaId::Canvas,
                    "Canvas",
                );
                self.render_panel_toggle(
                    ui,
                    current_window_id,
                    &window.layout_state,
                    StudioGuiWindowAreaId::Runtime,
                    "Runtime",
                );
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Region weights").strong());
                self.render_region_weight_slider(
                    ui,
                    current_window_id,
                    &window.layout_state,
                    StudioGuiWindowDockRegion::LeftSidebar,
                    "Left",
                );
                self.render_region_weight_slider(
                    ui,
                    current_window_id,
                    &window.layout_state,
                    StudioGuiWindowDockRegion::CenterStage,
                    "Center",
                );
                self.render_region_weight_slider(
                    ui,
                    current_window_id,
                    &window.layout_state,
                    StudioGuiWindowDockRegion::RightSidebar,
                    "Right",
                );
            });
            if let Some(drag_session) = self.drag_session {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("Drop preview").strong());
                    ui.label(format!("dragging {}", area_label(drag_session.area_id)));
                    if drag_session.window_id == current_window_id {
                        ui.small("hover region lane / stack lane / panel header, click to drop");
                    } else {
                        ui.small("return to source window to drop");
                    }
                    if ui.button("Cancel").clicked() {
                        self.cancel_drag_session(current_window_id);
                    }
                });
            }
            if let Some(preview) = window.drop_preview.as_ref() {
                ui.separator();
                ui.small(format!(
                    "preview: {:?} -> {:?} stack={} tabs={:?}",
                    preview.query,
                    preview.overlay.target_dock_region,
                    preview.overlay.target_stack_group,
                    preview.overlay.target_stack_area_ids
                ));
            }
            if let Some(error) = self.last_error.as_ref() {
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(180, 40, 40), error);
            }
        });
    }

    fn render_left_sidebar(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
        hovered_drop_target: &mut bool,
    ) {
        let left_width =
            region_panel_width(&window.layout_state, ctx, StudioGuiWindowDockRegion::LeftSidebar);
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

    fn render_right_sidebar(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
        hovered_drop_target: &mut bool,
    ) {
        let right_width =
            region_panel_width(&window.layout_state, ctx, StudioGuiWindowDockRegion::RightSidebar);
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

    fn render_center_stage(
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

    fn render_region(
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
            ui.label("No panels in this region.");
            return;
        }

        for group in groups {
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

            ui.group(|ui| {
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
                    ui.horizontal_wrapped(|ui| {
                        for tab in &visible_tabs {
                            if ui.selectable_label(tab.active, tab.title).clicked() {
                                self.dispatch_layout_mutation(
                                    window.layout_state.scope.window_id,
                                    StudioGuiWindowLayoutMutation::SetActivePanelInStack {
                                        area_id: tab.area_id,
                                    },
                                );
                            }
                        }
                    });
                    ui.separator();
                }

                let active_area_id = group.active_area_id;
                self.render_area(ui, window, active_area_id, hovered_drop_target);
            });
            ui.add_space(8.0);
        }
    }

    fn render_area(
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
        let header_drop_query = drag_session.and_then(|drag_session| {
            area_drop_target_query(&layout, drag_session, area_id)
        });

        if let Some(query) = header_drop_query {
            let is_active_preview = self.active_drop_preview
                == Some(ActiveDropPreview { window_id, query });
            let header = egui::Frame::group(ui.style())
                .fill(drop_lane_fill(is_active_preview))
                .stroke(drop_lane_stroke(is_active_preview))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(panel.title).strong());
                        if let Some(badge) = panel.badge.as_ref() {
                            ui.label(format!("[{badge}]"));
                        }
                        ui.label(&panel.summary);
                        ui.small("hover to preview, click to drop before this panel");
                    });
                });
            let response = ui.interact(
                header.response.rect,
                ui.make_persistent_id(format!("panel-drop-header:{window_id:?}:{query:?}")),
                egui::Sense::click(),
            );
            self.process_drop_target_response(response, window_id, query, hovered_drop_target);
        } else {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(panel.title).strong());
                if let Some(badge) = panel.badge.as_ref() {
                    ui.label(format!("[{badge}]"));
                }
                ui.label(&panel.summary);
            });
        }
        ui.horizontal_wrapped(|ui| {
            let is_drag_source = drag_session
                .map(|drag_session| drag_session.area_id == area_id)
                .unwrap_or(false);
            if ui
                .add_enabled(
                    !is_drag_source,
                    egui::Button::new(if is_drag_source { "Dragging" } else { "Pick up" }),
                )
                .clicked()
            {
                self.begin_drag_session(window_id, area_id);
            }

            if ui.button("Center").clicked() {
                self.dispatch_layout_mutation(
                    window_id,
                    StudioGuiWindowLayoutMutation::SetCenterArea { area_id },
                );
            }

            let collapse_label = if panel.collapsed {
                "Expand"
            } else {
                "Collapse"
            };
            if ui.button(collapse_label).clicked() {
                self.dispatch_layout_mutation(
                    window_id,
                    StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                        area_id,
                        collapsed: !panel.collapsed,
                    },
                );
            }

            if ui.button("Hide").clicked() {
                self.dispatch_layout_mutation(
                    window_id,
                    StudioGuiWindowLayoutMutation::SetPanelVisibility {
                        area_id,
                        visible: false,
                    },
                );
            }

            self.render_move_menu(ui, window, area_id, panel.dock_region);
            self.render_stack_menu(ui, window, area_id, panel.display_mode);

            if !matches!(panel.display_mode, StudioGuiWindowPanelDisplayMode::Standalone) {
                if ui.button("Prev tab").clicked() {
                    self.dispatch_layout_mutation(
                        window_id,
                        StudioGuiWindowLayoutMutation::ActivatePreviousPanelInStack { area_id },
                    );
                }
                if ui.button("Next tab").clicked() {
                    self.dispatch_layout_mutation(
                        window_id,
                        StudioGuiWindowLayoutMutation::ActivateNextPanelInStack { area_id },
                    );
                }
            }
        });
        ui.separator();

        if panel.collapsed {
            ui.label("Panel is collapsed.");
            return;
        }

        match area_id {
            StudioGuiWindowAreaId::Commands => self.render_commands_area(ui, window),
            StudioGuiWindowAreaId::Canvas => self.render_canvas_area(ui, window),
            StudioGuiWindowAreaId::Runtime => self.render_runtime_area(ui, window),
        }
    }

    fn render_commands_area(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for section in &window.commands.sections {
                ui.label(egui::RichText::new(section.title).strong());
                for command in &section.commands {
                    self.render_command_entry(ui, command);
                }
                ui.add_space(6.0);
            }
        });
    }

    fn render_command_entry(&mut self, ui: &mut egui::Ui, command: &StudioGuiCommandEntry) {
        let label = match command.shortcut.as_ref() {
            Some(shortcut) => format!("{} ({})", command.label, format_shortcut(shortcut)),
            None => command.label.clone(),
        };
        if ui
            .add_enabled(command.enabled, egui::Button::new(label))
            .clicked()
        {
            self.dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: command.command_id.clone(),
            });
        }
        ui.label(&command.detail);
        ui.small(
            command
                .menu_path
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" > "),
        );
        ui.add_space(4.0);
    }

    fn render_canvas_area(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
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
        egui::ScrollArea::vertical().show(ui, |ui| {
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

    fn render_runtime_area(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        let run_panel = &window.runtime.run_panel;
        let run_panel_view = run_panel.view();

        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    run_panel.primary_action().enabled,
                    egui::Button::new(run_panel.primary_action().label),
                )
                .clicked()
            {
                self.dispatch_run_panel_widget(run_panel.activate_primary());
            }

            for action in &run_panel_view.secondary_actions {
                if ui
                    .add_enabled(action.enabled, egui::Button::new(action.label))
                    .clicked()
                {
                    self.dispatch_run_panel_widget(run_panel.activate(action.id));
                }
            }
        });

        ui.label(format!(
            "mode={} status={}",
            run_panel_view.mode_label, run_panel_view.status_label
        ));
        if let Some(pending) = run_panel_view.pending_label {
            ui.label(format!("pending={pending}"));
        }
        if let Some(snapshot_id) = run_panel_view.latest_snapshot_id.as_ref() {
            ui.label(format!("snapshot={snapshot_id}"));
        }
        if let Some(summary) = run_panel_view.latest_snapshot_summary.as_ref() {
            ui.label(summary);
        }
        if let Some(message) = run_panel_view.latest_log_message.as_ref() {
            ui.small(format!("latest log: {message}"));
        }

        if let Some(notice) = run_panel_view.notice.as_ref() {
            ui.separator();
            ui.colored_label(notice_color(notice.level), &notice.title);
            ui.label(&notice.message);
            if notice.recovery_action.is_some() {
                let recovery_label = notice
                    .recovery_action
                    .as_ref()
                    .map(|action| action.title)
                    .unwrap_or("Recover");
                if ui.button(recovery_label).clicked() {
                    match run_panel.activate_recovery_action() {
                        RunPanelRecoveryWidgetEvent::Requested { .. } => {
                            self.dispatch_event(StudioGuiEvent::RunPanelRecoveryRequested);
                        }
                        RunPanelRecoveryWidgetEvent::Missing => {}
                    }
                }
            }
        }

        if let Some(platform_notice) = window.runtime.platform_notice.as_ref() {
            ui.separator();
            ui.colored_label(notice_color(platform_notice.level), &platform_notice.title);
            ui.label(&platform_notice.message);
        }

        if let Some(entitlement_host) = window.runtime.entitlement_host.as_ref() {
            ui.separator();
            ui.label(egui::RichText::new("Entitlement").strong());
            for line in &entitlement_host.presentation.panel.text.lines {
                ui.small(line);
            }
            for line in &entitlement_host.presentation.text.lines {
                ui.small(line);
            }
        }

        ui.separator();
        ui.label(egui::RichText::new("Runtime log").strong());
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                for entry in window.runtime.log_entries.iter().rev().take(20) {
                    ui.label(format!("{:?}: {}", entry.level, entry.message));
                }
            });

        ui.separator();
        ui.label(egui::RichText::new("GUI activity").strong());
        egui::ScrollArea::vertical()
            .max_height(160.0)
            .show(ui, |ui| {
                for line in self.activity_log.iter().rev().take(16) {
                    ui.small(line);
                }
            });
    }

    fn render_panel_toggle(
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

    fn render_region_weight_slider(
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

    fn render_move_menu(
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

    fn render_stack_menu(
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

                if ui
                    .button(format!("With {}", target_panel.title))
                    .clicked()
                {
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

    fn dispatch_run_panel_widget(&mut self, event: RunPanelWidgetEvent) {
        match event {
            RunPanelWidgetEvent::Dispatched { intent, .. } => match intent {
                RunPanelIntent::RunManual(_) => {
                    self.dispatch_event(StudioGuiEvent::UiCommandRequested {
                        command_id: "run_panel.run_manual".to_string(),
                    });
                }
                RunPanelIntent::Resume(_) => {
                    self.dispatch_event(StudioGuiEvent::UiCommandRequested {
                        command_id: "run_panel.resume_workspace".to_string(),
                    });
                }
                RunPanelIntent::SetMode(SimulationMode::Hold) => {
                    self.dispatch_event(StudioGuiEvent::UiCommandRequested {
                        command_id: "run_panel.set_hold".to_string(),
                    });
                }
                RunPanelIntent::SetMode(SimulationMode::Active) => {
                    self.dispatch_event(StudioGuiEvent::UiCommandRequested {
                        command_id: "run_panel.set_active".to_string(),
                    });
                }
            },
            RunPanelWidgetEvent::Disabled { .. } | RunPanelWidgetEvent::Missing { .. } => {}
        }
    }

    fn dispatch_layout_mutation(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        mutation: StudioGuiWindowLayoutMutation,
    ) {
        self.dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id,
            mutation,
        });
    }

    fn begin_drag_session(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        area_id: StudioGuiWindowAreaId,
    ) {
        self.clear_drop_preview(window_id);
        self.drag_session = Some(PanelDragSession { area_id, window_id });
    }

    fn active_drag_session_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> Option<PanelDragSession> {
        self.drag_session
            .filter(|drag_session| drag_session.window_id == window_id)
    }

    fn render_drop_target_lane(
        &mut self,
        ui: &mut egui::Ui,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
        label: &str,
        hovered_drop_target: &mut bool,
    ) {
        let is_active_preview = self.active_drop_preview == Some(ActiveDropPreview { window_id, query });
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

    fn process_drop_target_response(
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

    fn ensure_drop_preview(
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

    fn clear_drop_preview(&mut self, window_id: Option<StudioWindowHostId>) {
        let Some(active_preview) = self.active_drop_preview else {
            return;
        };
        self.dispatch_event(StudioGuiEvent::WindowDropTargetPreviewCleared {
            window_id: active_preview.window_id.or(window_id),
        });
        self.active_drop_preview = None;
    }

    fn apply_drop_target(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        query: StudioGuiWindowDropTargetQuery,
    ) {
        self.dispatch_event(StudioGuiEvent::WindowDropTargetApplyRequested { window_id, query });
        self.active_drop_preview = None;
        self.drag_session = None;
    }

    fn cancel_drag_session(&mut self, window_id: Option<StudioWindowHostId>) {
        self.drag_session = None;
        self.clear_drop_preview(window_id);
    }

    fn finish_drop_preview_cycle(
        &mut self,
        window_id: Option<StudioWindowHostId>,
        hovered_drop_target: bool,
    ) {
        if self.drag_session.is_none() {
            self.clear_drop_preview(window_id);
            return;
        }
        if !hovered_drop_target {
            self.clear_drop_preview(window_id);
        }
    }

    fn dispatch_event(&mut self, event: StudioGuiEvent) {
        match self.driver.dispatch_event(event.clone()) {
            Ok(dispatch) => {
                self.record_dispatch(&dispatch);
                self.last_error = None;
            }
            Err(error) => {
                let message = format!("[{}] {}", error.code().as_str(), error.message());
                self.activity_log.push(format!("event failed: {message}"));
                self.trim_activity_log();
                self.last_error = Some(message);
            }
        }
    }

    fn drain_due_timers(&mut self, ctx: &egui::Context) {
        let now = SystemTime::now();
        match self.driver.drain_due_native_timer_events(now) {
            Ok(dispatches) => {
                for dispatch in &dispatches {
                    self.record_dispatch(dispatch);
                }
            }
            Err(error) => {
                self.last_error = Some(format!(
                    "timer dispatch failed [{}]: {}",
                    error.code().as_str(),
                    error.message()
                ));
            }
        }

        if let Some(next_due_at) = self.driver.next_due_native_timer_at() {
            let delay = next_due_at.duration_since(now).unwrap_or(Duration::ZERO);
            ctx.request_repaint_after(delay);
        }
    }

    fn dispatch_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }

        let shortcuts = ctx.input(|input| collect_shortcuts(input));
        let focus_context = self.focus_context();
        for shortcut in shortcuts {
            self.dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut,
                focus_context,
            });
        }
    }

    fn focus_context(&self) -> StudioGuiFocusContext {
        let window = self.driver.window_model();
        if window.canvas.focused_suggestion_id.is_some() {
            StudioGuiFocusContext::CanvasSuggestionFocused
        } else {
            StudioGuiFocusContext::Global
        }
    }

    fn record_dispatch(&mut self, dispatch: &StudioGuiDriverDispatch) {
        let summary = match &dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                radishflow_studio::StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(_),
            )
            | StudioGuiDriverOutcome::HostCommand(
                radishflow_studio::StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(_),
            ) => return,
            StudioGuiDriverOutcome::HostCommand(outcome) => format!("host::{outcome:?}"),
            StudioGuiDriverOutcome::CanvasInteraction(result) => {
                format!("canvas::{:?}", result.action)
            }
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                format!("layout::{:?}", result.mutation)
            }
            StudioGuiDriverOutcome::IgnoredNativeTimerElapsed { handle_id, .. } => {
                format!("timer::ignored handle={handle_id}")
            }
            StudioGuiDriverOutcome::IgnoredShortcut { shortcut, reason } => format!(
                "shortcut::ignored {} {:?}",
                format_shortcut(shortcut),
                reason
            ),
        };
        self.activity_log.push(summary);
        self.trim_activity_log();
    }

    fn trim_activity_log(&mut self) {
        const MAX_ENTRIES: usize = 64;
        if self.activity_log.len() > MAX_ENTRIES {
            let drain_count = self.activity_log.len() - MAX_ENTRIES;
            self.activity_log.drain(0..drain_count);
        }
    }
}

fn collect_shortcuts(input: &egui::InputState) -> Vec<StudioGuiShortcut> {
    let mut shortcuts = Vec::new();

    if input.key_pressed(egui::Key::F5) {
        shortcuts.push(StudioGuiShortcut {
            modifiers: modifiers_from_egui(input.modifiers),
            key: StudioGuiShortcutKey::F5,
        });
    }
    if input.key_pressed(egui::Key::F6) {
        shortcuts.push(StudioGuiShortcut {
            modifiers: modifiers_from_egui(input.modifiers),
            key: StudioGuiShortcutKey::F6,
        });
    }
    if input.key_pressed(egui::Key::F8) {
        shortcuts.push(StudioGuiShortcut {
            modifiers: modifiers_from_egui(input.modifiers),
            key: StudioGuiShortcutKey::F8,
        });
    }
    if input.key_pressed(egui::Key::Tab) {
        shortcuts.push(StudioGuiShortcut {
            modifiers: modifiers_from_egui(input.modifiers),
            key: StudioGuiShortcutKey::Tab,
        });
    }
    if input.key_pressed(egui::Key::Escape) {
        shortcuts.push(StudioGuiShortcut {
            modifiers: modifiers_from_egui(input.modifiers),
            key: StudioGuiShortcutKey::Escape,
        });
    }

    shortcuts
}

fn modifiers_from_egui(modifiers: egui::Modifiers) -> Vec<StudioGuiShortcutModifier> {
    let mut items = Vec::new();
    if modifiers.ctrl {
        items.push(StudioGuiShortcutModifier::Ctrl);
    }
    if modifiers.shift {
        items.push(StudioGuiShortcutModifier::Shift);
    }
    if modifiers.alt {
        items.push(StudioGuiShortcutModifier::Alt);
    }
    items
}

fn region_panel_width(
    layout_state: &radishflow_studio::StudioGuiWindowLayoutState,
    ctx: &egui::Context,
    dock_region: StudioGuiWindowDockRegion,
) -> f32 {
    let total_weight = layout_state
        .region_weights
        .iter()
        .map(|item| item.weight)
        .sum::<u16>()
        .max(1) as f32;
    let region_weight = layout_state
        .region_weight(dock_region)
        .map(|item| item.weight)
        .unwrap_or(24) as f32;
    let available_width = ctx.available_rect().width().max(960.0);
    (available_width * (region_weight / total_weight)).clamp(180.0, 480.0)
}

fn dock_region_label(dock_region: StudioGuiWindowDockRegion) -> &'static str {
    match dock_region {
        StudioGuiWindowDockRegion::LeftSidebar => "Left",
        StudioGuiWindowDockRegion::CenterStage => "Center",
        StudioGuiWindowDockRegion::RightSidebar => "Right",
    }
}

fn area_drop_target_query(
    layout: &StudioGuiWindowLayoutModel,
    drag_session: PanelDragSession,
    area_id: StudioGuiWindowAreaId,
) -> Option<StudioGuiWindowDropTargetQuery> {
    if drag_session.area_id == area_id {
        return None;
    }

    let drag_panel = layout.panel(drag_session.area_id)?;
    let target_panel = layout.panel(area_id)?;
    let placement = StudioGuiWindowDockPlacement::Before { anchor_area_id: area_id };

    if drag_panel.dock_region == target_panel.dock_region
        && drag_panel.stack_group == target_panel.stack_group
    {
        Some(StudioGuiWindowDropTargetQuery::CurrentStack {
            area_id: drag_session.area_id,
            placement,
        })
    } else {
        Some(StudioGuiWindowDropTargetQuery::Stack {
            area_id: drag_session.area_id,
            anchor_area_id: area_id,
            placement,
        })
    }
}

fn stack_group_drop_target_query(
    layout: &StudioGuiWindowLayoutModel,
    drag_session: PanelDragSession,
    group: &StudioGuiWindowStackGroupLayout,
) -> Option<StudioGuiWindowDropTargetQuery> {
    let drag_panel = layout.panel(drag_session.area_id)?;
    let target_panel = layout.panel(group.active_area_id)?;

    if drag_panel.dock_region == target_panel.dock_region
        && drag_panel.stack_group == target_panel.stack_group
    {
        Some(StudioGuiWindowDropTargetQuery::CurrentStack {
            area_id: drag_session.area_id,
            placement: StudioGuiWindowDockPlacement::End,
        })
    } else {
        Some(StudioGuiWindowDropTargetQuery::Stack {
            area_id: drag_session.area_id,
            anchor_area_id: group.active_area_id,
            placement: StudioGuiWindowDockPlacement::End,
        })
    }
}

fn drop_lane_fill(is_active_preview: bool) -> egui::Color32 {
    if is_active_preview {
        egui::Color32::from_rgb(214, 235, 255)
    } else {
        egui::Color32::from_rgb(242, 246, 250)
    }
}

fn drop_lane_stroke(is_active_preview: bool) -> egui::Stroke {
    if is_active_preview {
        egui::Stroke::new(1.5, egui::Color32::from_rgb(56, 126, 214))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(171, 181, 190))
    }
}

fn area_label(area_id: StudioGuiWindowAreaId) -> &'static str {
    match area_id {
        StudioGuiWindowAreaId::Commands => "Commands",
        StudioGuiWindowAreaId::Canvas => "Canvas",
        StudioGuiWindowAreaId::Runtime => "Runtime",
    }
}

fn format_window_chip(window: &StudioAppHostWindowState) -> String {
    let role = match window.role {
        StudioWindowHostRole::EntitlementTimerOwner => "owner",
        StudioWindowHostRole::Observer => "observer",
    };
    format!("#{}/{}-{}", window.window_id, role, window.layout_slot)
}

fn notice_color(level: RunPanelNoticeLevel) -> egui::Color32 {
    match level {
        RunPanelNoticeLevel::Info => egui::Color32::from_rgb(40, 90, 160),
        RunPanelNoticeLevel::Warning => egui::Color32::from_rgb(180, 120, 20),
        RunPanelNoticeLevel::Error => egui::Color32::from_rgb(180, 40, 40),
    }
}

fn format_shortcut(shortcut: &StudioGuiShortcut) -> String {
    let mut parts = shortcut
        .modifiers
        .iter()
        .map(|modifier| match modifier {
            StudioGuiShortcutModifier::Ctrl => "Ctrl",
            StudioGuiShortcutModifier::Shift => "Shift",
            StudioGuiShortcutModifier::Alt => "Alt",
        })
        .collect::<Vec<_>>();
    let key = match shortcut.key {
        StudioGuiShortcutKey::F5 => "F5",
        StudioGuiShortcutKey::F6 => "F6",
        StudioGuiShortcutKey::F8 => "F8",
        StudioGuiShortcutKey::Tab => "Tab",
        StudioGuiShortcutKey::Escape => "Escape",
    };
    parts.push(key);
    parts.join("+")
}
