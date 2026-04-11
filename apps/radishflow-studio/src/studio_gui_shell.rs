use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use eframe::egui;
use radishflow_studio::{
    StudioAppHostWindowState, StudioGuiCommandEntry, StudioGuiEvent, StudioGuiFocusContext,
    StudioGuiPlatformExecutedNativeTimerCallbackOutcome, StudioGuiPlatformHost,
    StudioGuiPlatformNativeTimerId, StudioGuiPlatformTimerCommand, StudioGuiPlatformTimerExecutor,
    StudioGuiPlatformTimerExecutorResponse, StudioGuiPlatformTimerFollowUpCommand,
    StudioGuiRuntimeHostActionId, StudioGuiShortcut, StudioGuiShortcutKey,
    StudioGuiShortcutModifier, StudioGuiWindowAreaId, StudioGuiWindowDockPlacement,
    StudioGuiWindowDockRegion, StudioGuiWindowDropTargetQuery, StudioGuiWindowLayoutModel,
    StudioGuiWindowLayoutMutation, StudioGuiWindowModel, StudioGuiWindowPanelDisplayMode,
    StudioGuiWindowStackGroupLayout, StudioRuntimeConfig, StudioWindowHostId,
    StudioWindowHostRole,
};
use rf_types::RfResult;
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
    platform_host: StudioGuiPlatformHost,
    platform_timer_executor: EguiPlatformTimerExecutor,
    last_error: Option<String>,
    drag_session: Option<PanelDragSession>,
    active_drop_preview: Option<ActiveDropPreview>,
    drop_preview_overlay_anchor: Option<DropPreviewOverlayAnchor>,
    last_viewport_focused: Option<bool>,
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

#[derive(Debug, Clone, Copy)]
struct DropPreviewOverlayAnchor {
    rect: egui::Rect,
    priority: u8,
}

#[derive(Debug, Default)]
struct EguiPlatformTimerExecutor {
    next_native_timer_id: StudioGuiPlatformNativeTimerId,
    active_native_timers: BTreeMap<StudioGuiPlatformNativeTimerId, EguiNativeTimerRegistration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EguiNativeTimerRegistration {
    schedule: radishflow_studio::StudioGuiNativeTimerSchedule,
}

impl RadishFlowStudioApp {
    fn new() -> Self {
        let config = StudioRuntimeConfig::default();
        let state = match StudioGuiPlatformHost::new(&config) {
            Ok(platform_host) => {
                let mut ready = ReadyAppState {
                    platform_host,
                    platform_timer_executor: EguiPlatformTimerExecutor::default(),
                    last_error: None,
                    drag_session: None,
                    active_drop_preview: None,
                    drop_preview_overlay_anchor: None,
                    last_viewport_focused: None,
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
        self.sync_viewport_lifecycle(ctx);
        self.dispatch_shortcuts(ctx);
        self.drain_due_timers(ctx);
        self.drop_preview_overlay_anchor = None;

        let snapshot = self.platform_host.snapshot();
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
        self.render_floating_drop_preview_overlay(ctx, &window);
        self.finish_drop_preview_cycle(
            ctx,
            window.layout_state.scope.window_id,
            hovered_drop_target,
        );
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
                    if let Some(preview) = window.drop_preview.as_ref() {
                        ui.small(
                            egui::RichText::new(format_compact_drop_preview_status(preview))
                                .color(egui::Color32::from_rgb(92, 104, 117)),
                        );
                    }
                    if ui.button("Cancel").clicked() {
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

    fn render_right_sidebar(
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
            ui.label("No panels in this region.");
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
        ui.push_id(
            format!(
                "panel:{}:{}",
                window.layout_state.scope.layout_key,
                area_label(area_id)
            ),
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    let is_drag_source = drag_session
                        .map(|drag_session| drag_session.area_id == area_id)
                        .unwrap_or(false);
                    if ui
                        .add_enabled(
                            !is_drag_source,
                            egui::Button::new(if is_drag_source {
                                "Dragging"
                            } else {
                                "Pick up"
                            }),
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

                    if !matches!(
                        panel.display_mode,
                        StudioGuiWindowPanelDisplayMode::Standalone
                    ) {
                        if ui.button("Prev tab").clicked() {
                            self.dispatch_layout_mutation(
                                window_id,
                                StudioGuiWindowLayoutMutation::ActivatePreviousPanelInStack {
                                    area_id,
                                },
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
                    StudioGuiWindowAreaId::Commands => {
                        self.render_commands_area(ui, window, area_id)
                    }
                    StudioGuiWindowAreaId::Canvas => self.render_canvas_area(ui, window, area_id),
                    StudioGuiWindowAreaId::Runtime => self.render_runtime_area(ui, window, area_id),
                }
            },
        );
    }

    fn render_commands_area(
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

    fn render_canvas_area(
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

    fn render_runtime_area(
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
                                self.dispatch_event(StudioGuiEvent::RunPanelRecoveryRequested);
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
                            self.dispatch_event(StudioGuiEvent::EntitlementPrimaryActionRequested);
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
                                self.dispatch_event(StudioGuiEvent::EntitlementActionRequested {
                                    action_id: action.id,
                                });
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
                ui.horizontal_wrapped(|ui| {
                    for action in &window.runtime.host_actions {
                        ui.vertical(|ui| {
                            let response =
                                ui.add_enabled(action.enabled, egui::Button::new(action.label));
                            let response = response.on_hover_text(&action.detail);
                            if response.clicked() {
                                self.dispatch_runtime_host_action(
                                    action.id,
                                    window.layout_state.scope.window_id,
                                );
                            }
                            ui.small(
                                egui::RichText::new(&action.detail)
                                    .color(egui::Color32::from_rgb(92, 104, 117)),
                            );
                        });
                    }
                });
                ui.add_space(6.0);
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

    fn render_floating_drop_preview_overlay(
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

    fn record_drop_preview_overlay_anchor(&mut self, rect: egui::Rect, priority: u8) {
        let candidate = DropPreviewOverlayAnchor { rect, priority };
        let replace = self
            .drop_preview_overlay_anchor
            .map(|current| priority >= current.priority)
            .unwrap_or(true);
        if replace {
            self.drop_preview_overlay_anchor = Some(candidate);
        }
    }

    fn dispatch_event(&mut self, event: StudioGuiEvent) {
        match self
            .platform_host
            .dispatch_event_and_execute_platform_timer(
                event.clone(),
                &mut self.platform_timer_executor,
            ) {
            Ok(_) => self.last_error = None,
            Err(error) => {
                let message = format!("[{}] {}", error.code().as_str(), error.message());
                self.platform_host
                    .record_activity_line(format!("event failed: {message}"));
                self.last_error = Some(message);
            }
        }
    }

    fn drain_due_timers(&mut self, ctx: &egui::Context) {
        let now = SystemTime::now();
        match drain_due_platform_timer_callbacks(
            &mut self.platform_host,
            &mut self.platform_timer_executor,
            now,
        ) {
            Ok(callbacks) => {
                for callback in callbacks {
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
                self.last_error = Some(format!(
                    "timer dispatch failed [{}]: {}",
                    error.code().as_str(),
                    error.message()
                ));
            }
        }

        if let Some(next_due_at) = self.platform_timer_executor.next_due_at() {
            let delay = next_due_at.duration_since(now).unwrap_or(Duration::ZERO);
            ctx.request_repaint_after(delay);
        }
    }

    fn sync_viewport_lifecycle(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|input| input.viewport().focused.unwrap_or(input.focused));
        let became_focused = self
            .last_viewport_focused
            .map(|previous| !previous && focused)
            .unwrap_or(false);
        self.last_viewport_focused = Some(focused);

        if !became_focused {
            return;
        }

        let window_id = self
            .platform_host
            .snapshot()
            .window_model()
            .layout_state
            .scope
            .window_id;
        if let Some(window_id) = window_id {
            self.dispatch_event(StudioGuiEvent::WindowForegrounded { window_id });
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
        let window = self.platform_host.snapshot().window_model();
        if window.canvas.focused_suggestion_id.is_some() {
            StudioGuiFocusContext::CanvasSuggestionFocused
        } else {
            StudioGuiFocusContext::Global
        }
    }

    fn dispatch_runtime_host_action(
        &mut self,
        action_id: StudioGuiRuntimeHostActionId,
        current_window_id: Option<StudioWindowHostId>,
    ) {
        match action_id {
            StudioGuiRuntimeHostActionId::ForegroundCurrentWindow => {
                if let Some(window_id) = current_window_id {
                    self.dispatch_event(StudioGuiEvent::WindowForegrounded { window_id });
                }
            }
            StudioGuiRuntimeHostActionId::LoginCompleted => {
                self.dispatch_event(StudioGuiEvent::LoginCompleted);
            }
            StudioGuiRuntimeHostActionId::NetworkRestored => {
                self.dispatch_event(StudioGuiEvent::NetworkRestored);
            }
            StudioGuiRuntimeHostActionId::TimerElapsed => {
                self.dispatch_event(StudioGuiEvent::EntitlementTimerElapsed);
            }
        }
    }
}

impl StudioGuiPlatformTimerExecutor for EguiPlatformTimerExecutor {
    fn execute_platform_timer_command(
        &mut self,
        command: &StudioGuiPlatformTimerCommand,
    ) -> RfResult<StudioGuiPlatformTimerExecutorResponse> {
        match command {
            StudioGuiPlatformTimerCommand::Arm { schedule } => {
                let native_timer_id = self.allocate_native_timer_id();
                self.active_native_timers.insert(
                    native_timer_id,
                    EguiNativeTimerRegistration {
                        schedule: schedule.clone(),
                    },
                );
                Ok(StudioGuiPlatformTimerExecutorResponse::Started {
                    native_timer_id,
                })
            }
            StudioGuiPlatformTimerCommand::Rearm { previous, schedule } => {
                if let Some(previous) = previous.as_ref() {
                    self.active_native_timers.remove(&previous.native_timer_id);
                }
                let native_timer_id = self.allocate_native_timer_id();
                self.active_native_timers.insert(
                    native_timer_id,
                    EguiNativeTimerRegistration {
                        schedule: schedule.clone(),
                    },
                );
                Ok(StudioGuiPlatformTimerExecutorResponse::Started {
                    native_timer_id,
                })
            }
            StudioGuiPlatformTimerCommand::Clear { previous } => {
                if let Some(previous) = previous.as_ref() {
                    self.active_native_timers.remove(&previous.native_timer_id);
                }
                Ok(StudioGuiPlatformTimerExecutorResponse::Cleared)
            }
        }
    }

    fn execute_platform_timer_follow_up_command(
        &mut self,
        command: &StudioGuiPlatformTimerFollowUpCommand,
    ) -> RfResult<()> {
        match command {
            StudioGuiPlatformTimerFollowUpCommand::ClearNativeTimer { native_timer_id } => {
                self.active_native_timers.remove(native_timer_id);
            }
        }
        Ok(())
    }
}

impl EguiPlatformTimerExecutor {
    fn allocate_native_timer_id(&mut self) -> StudioGuiPlatformNativeTimerId {
        self.next_native_timer_id = self.next_native_timer_id.saturating_add(1).max(1);
        self.next_native_timer_id
    }

    fn next_due_at(&self) -> Option<SystemTime> {
        self.active_native_timers
            .values()
            .map(|registration| registration.schedule.slot.timer.due_at)
            .min()
    }

    fn drain_due_native_timer_ids(
        &mut self,
        now: SystemTime,
    ) -> Vec<StudioGuiPlatformNativeTimerId> {
        let due_native_timer_ids = self
            .active_native_timers
            .iter()
            .filter_map(|(native_timer_id, registration)| {
                (registration.schedule.slot.timer.due_at <= now).then_some(*native_timer_id)
            })
            .collect::<Vec<_>>();
        for native_timer_id in &due_native_timer_ids {
            self.active_native_timers.remove(native_timer_id);
        }
        due_native_timer_ids
    }
}

fn drain_due_platform_timer_callbacks(
    platform_host: &mut StudioGuiPlatformHost,
    executor: &mut EguiPlatformTimerExecutor,
    now: SystemTime,
) -> RfResult<Vec<StudioGuiPlatformExecutedNativeTimerCallbackOutcome>> {
    executor
        .drain_due_native_timer_ids(now)
        .into_iter()
        .map(|native_timer_id| {
            platform_host.dispatch_native_timer_elapsed_by_native_id_and_execute_platform_timer(
                native_timer_id,
                executor,
            )
        })
        .collect()
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

fn drop_preview_for_region<'a>(
    preview: Option<&'a radishflow_studio::StudioGuiWindowDropPreviewModel>,
    dock_region: StudioGuiWindowDockRegion,
) -> Option<&'a radishflow_studio::StudioGuiWindowDropPreviewModel> {
    preview.filter(|preview| preview.overlay.target_dock_region == dock_region)
}

fn drop_preview_targets_stack(
    preview: Option<&radishflow_studio::StudioGuiWindowDropPreviewModel>,
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
) -> bool {
    preview
        .map(|preview| {
            preview.overlay.target_dock_region == dock_region
                && preview.overlay.target_stack_group == stack_group
        })
        .unwrap_or(false)
}

fn new_stack_preview_group_index(
    preview: Option<&radishflow_studio::StudioGuiWindowDropPreviewModel>,
) -> Option<usize> {
    preview
        .filter(|preview| preview.overlay.creates_new_stack)
        .map(|preview| preview.overlay.target_group_index)
}

fn preview_anchor_matches_area(
    preview: Option<&radishflow_studio::StudioGuiWindowDropPreviewModel>,
    area_id: StudioGuiWindowAreaId,
) -> bool {
    preview
        .and_then(|preview| preview.overlay.anchor_area_id)
        .map(|anchor_area_id| anchor_area_id == area_id)
        .unwrap_or(false)
}

fn render_new_stack_insert_overlay(
    ui: &mut egui::Ui,
    preview: &radishflow_studio::StudioGuiWindowDropPreviewModel,
) -> egui::Rect {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(235, 244, 255))
        .stroke(egui::Stroke::new(
            1.5,
            egui::Color32::from_rgb(56, 126, 214),
        ))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.small(
                    egui::RichText::new(preview_insert_hint(preview))
                        .strong()
                        .color(egui::Color32::from_rgb(56, 126, 214)),
                );
            });
        })
        .response
        .rect
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
    let placement = StudioGuiWindowDockPlacement::Before {
        anchor_area_id: area_id,
    };

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

fn stack_preview_fill(is_target_stack: bool) -> egui::Color32 {
    if is_target_stack {
        egui::Color32::from_rgb(235, 244, 255)
    } else {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0)
    }
}

fn stack_preview_stroke(is_target_stack: bool) -> egui::Stroke {
    if is_target_stack {
        egui::Stroke::new(1.5, egui::Color32::from_rgb(56, 126, 214))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60))
    }
}

fn stack_accepts_overlay_anchor(
    preview: Option<&radishflow_studio::StudioGuiWindowDropPreviewModel>,
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
) -> bool {
    preview
        .map(|preview| {
            preview.overlay.target_dock_region == dock_region
                && preview.overlay.target_stack_group == stack_group
                && preview.overlay.merges_into_existing_stack
        })
        .unwrap_or(false)
}

fn paint_stack_tab_insert_marker(
    ui: &mut egui::Ui,
    tab_strip_rect: egui::Rect,
    tab_rects: &[(StudioGuiWindowAreaId, egui::Rect)],
    preview: Option<&radishflow_studio::StudioGuiWindowDropPreviewModel>,
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
) {
    let Some(preview) = preview.filter(|preview| {
        preview.overlay.target_dock_region == dock_region
            && preview.overlay.target_stack_group == stack_group
            && preview.overlay.merges_into_existing_stack
    }) else {
        return;
    };

    let Some(x) = stack_insert_marker_x(tab_strip_rect, tab_rects, preview) else {
        return;
    };
    let stroke = egui::Stroke::new(2.5, egui::Color32::from_rgb(56, 126, 214));
    let top = tab_strip_rect.top() + 2.0;
    let bottom = tab_strip_rect.bottom() - 2.0;
    let painter = ui.painter();
    painter.line_segment([egui::pos2(x, top), egui::pos2(x, bottom)], stroke);
    painter.line_segment([egui::pos2(x - 5.0, top), egui::pos2(x + 5.0, top)], stroke);
    painter.line_segment(
        [egui::pos2(x - 5.0, bottom), egui::pos2(x + 5.0, bottom)],
        stroke,
    );
    paint_preview_hint_pill_centered(
        painter,
        egui::pos2(x, top - 10.0),
        &preview_insert_hint(preview),
    );
}

fn stack_insert_marker_x(
    tab_strip_rect: egui::Rect,
    tab_rects: &[(StudioGuiWindowAreaId, egui::Rect)],
    preview: &radishflow_studio::StudioGuiWindowDropPreviewModel,
) -> Option<f32> {
    let (previous, next) = preview_insert_neighbors(preview);

    let x = if let Some(next_area_id) = next {
        tab_rects
            .iter()
            .find(|(area_id, _)| *area_id == next_area_id)
            .map(|(_, rect)| rect.left() - 6.0)
    } else if let Some(previous_area_id) = previous {
        tab_rects
            .iter()
            .find(|(area_id, _)| *area_id == previous_area_id)
            .map(|(_, rect)| rect.right() + 6.0)
    } else {
        tab_rects
            .first()
            .map(|(_, rect)| rect.left() - 6.0)
            .or(Some(tab_strip_rect.center().x))
    }?;

    Some(x.clamp(tab_strip_rect.left() + 6.0, tab_strip_rect.right() - 6.0))
}

fn preview_area_badges(
    window: &StudioGuiWindowModel,
    area_id: StudioGuiWindowAreaId,
) -> Vec<&'static str> {
    let Some(preview) = window.drop_preview.as_ref() else {
        return Vec::new();
    };

    let mut badges = Vec::new();
    if preview.overlay.drag_area_id == area_id {
        badges.push("Drag Source");
    }
    if preview_anchor_matches_area(window.drop_preview.as_ref(), area_id) {
        badges.push("Insertion Anchor");
    }
    if preview.overlay.highlighted_area_ids.contains(&area_id) {
        badges.push("Preview Target");
    }
    if preview.changed_area_ids.contains(&area_id) {
        badges.push("Layout Change");
    }
    badges
}

fn preview_area_transition(
    window: &StudioGuiWindowModel,
    area_id: StudioGuiWindowAreaId,
) -> Option<String> {
    let preview = window.drop_preview.as_ref()?;
    if !preview.changed_area_ids.contains(&area_id) {
        return None;
    }

    let layout = window.layout();
    let current_panel = layout.panel(area_id)?;
    let preview_panel = preview.preview_layout.panel(area_id)?;
    let mut parts = Vec::new();

    if current_panel.dock_region != preview_panel.dock_region {
        parts.push(format!(
            "region {} -> {}",
            dock_region_label(current_panel.dock_region),
            dock_region_label(preview_panel.dock_region)
        ));
    }
    if current_panel.stack_group != preview_panel.stack_group {
        parts.push(format!(
            "stack {} -> {}",
            current_panel.stack_group, preview_panel.stack_group
        ));
    }
    if current_panel.order != preview_panel.order {
        parts.push(format!(
            "order {} -> {}",
            current_panel.order, preview_panel.order
        ));
    }
    if parts.is_empty() {
        parts.push("active stack focus will change".to_string());
    }

    Some(parts.join(" | "))
}

fn area_accepts_overlay_anchor(
    window: &StudioGuiWindowModel,
    area_id: StudioGuiWindowAreaId,
) -> bool {
    let Some(preview) = window.drop_preview.as_ref() else {
        return false;
    };
    preview_anchor_matches_area(Some(preview), area_id)
        || preview.overlay.highlighted_area_ids.contains(&area_id)
}

fn drop_preview_anchor_priority_area(
    window: &StudioGuiWindowModel,
    area_id: StudioGuiWindowAreaId,
) -> u8 {
    let Some(preview) = window.drop_preview.as_ref() else {
        return 0;
    };
    if preview_anchor_matches_area(Some(preview), area_id) {
        3
    } else if preview.overlay.highlighted_area_ids.contains(&area_id) {
        2
    } else {
        0
    }
}

fn drop_preview_anchor_priority_stack_tabs() -> u8 {
    1
}

fn drop_preview_anchor_priority_new_stack() -> u8 {
    1
}

fn paint_area_preview_overlay(
    ui: &mut egui::Ui,
    header_rect: egui::Rect,
    window: &StudioGuiWindowModel,
    area_id: StudioGuiWindowAreaId,
) {
    let Some(preview) = window.drop_preview.as_ref() else {
        return;
    };

    let painter = ui.painter();
    if preview.changed_area_ids.contains(&area_id) {
        let accent_x = header_rect.left() + 3.0;
        painter.line_segment(
            [
                egui::pos2(accent_x, header_rect.top() + 3.0),
                egui::pos2(accent_x, header_rect.bottom() - 3.0),
            ],
            egui::Stroke::new(3.0, egui::Color32::from_rgb(150, 196, 255)),
        );
    }

    if preview.overlay.highlighted_area_ids.contains(&area_id) {
        let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(56, 126, 214));
        let left = header_rect.left() + 6.0;
        let right = header_rect.right() - 6.0;
        let top = header_rect.top() + 3.0;
        let bottom = header_rect.bottom() - 3.0;
        painter.line_segment([egui::pos2(left, top), egui::pos2(right, top)], stroke);
        painter.line_segment(
            [egui::pos2(left, bottom), egui::pos2(right, bottom)],
            stroke,
        );
    }

    if preview_anchor_matches_area(Some(preview), area_id) {
        let stroke = egui::Stroke::new(2.5, egui::Color32::from_rgb(56, 126, 214));
        let y = header_rect.top() + 2.0;
        let left = header_rect.left() + 10.0;
        let right = header_rect.right() - 10.0;
        painter.line_segment([egui::pos2(left, y), egui::pos2(right, y)], stroke);
        painter.line_segment(
            [egui::pos2(left, y - 4.0), egui::pos2(left, y + 4.0)],
            stroke,
        );
        painter.line_segment(
            [egui::pos2(right, y - 4.0), egui::pos2(right, y + 4.0)],
            stroke,
        );
        paint_preview_hint_pill_top_right(
            painter,
            egui::pos2(right, header_rect.top() + 14.0),
            &preview_insert_hint(preview),
        );
    }
}

fn preferred_overlay_pos(anchor_rect: egui::Rect) -> egui::Pos2 {
    egui::pos2(anchor_rect.right() + 12.0, anchor_rect.top() - 4.0)
}

fn clamp_overlay_pos(ctx: &egui::Context, pos: egui::Pos2, size: egui::Vec2) -> egui::Pos2 {
    clamp_overlay_pos_to_rect(ctx.screen_rect(), pos, size)
}

fn clamp_overlay_pos_to_rect(screen: egui::Rect, pos: egui::Pos2, size: egui::Vec2) -> egui::Pos2 {
    let max_x = (screen.right() - size.x - 8.0).max(screen.left() + 8.0);
    let max_y = (screen.bottom() - size.y - 8.0).max(screen.top() + 8.0);
    egui::pos2(
        pos.x.clamp(screen.left() + 8.0, max_x),
        pos.y.clamp(screen.top() + 8.0, max_y),
    )
}

fn paint_preview_hint_pill_centered(painter: &egui::Painter, center: egui::Pos2, text: &str) {
    let font_id = egui::FontId::proportional(11.0);
    let text_color = egui::Color32::from_rgb(33, 82, 153);
    let galley = painter.layout_no_wrap(text.to_owned(), font_id.clone(), text_color);
    let size = galley.size() + egui::vec2(14.0, 8.0);
    let rect = egui::Rect::from_center_size(center, size);
    painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(232, 242, 255));
    painter.galley(rect.center() - galley.size() * 0.5, galley, text_color);
}

fn paint_preview_hint_pill_top_right(painter: &egui::Painter, right_top: egui::Pos2, text: &str) {
    let font_id = egui::FontId::proportional(11.0);
    let text_color = egui::Color32::from_rgb(33, 82, 153);
    let galley = painter.layout_no_wrap(text.to_owned(), font_id.clone(), text_color);
    let size = galley.size() + egui::vec2(14.0, 8.0);
    let rect = egui::Rect::from_min_size(egui::pos2(right_top.x - size.x, right_top.y), size);
    painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(232, 242, 255));
    painter.galley(rect.center() - galley.size() * 0.5, galley, text_color);
}

fn format_compact_drop_preview_status(
    preview: &radishflow_studio::StudioGuiWindowDropPreviewModel,
) -> String {
    format!(
        "target {} stack {} | {}",
        dock_region_label(preview.overlay.target_dock_region),
        preview.overlay.target_stack_group,
        preview_insert_hint(preview)
    )
}

fn preview_insert_hint(preview: &radishflow_studio::StudioGuiWindowDropPreviewModel) -> String {
    let (previous, next) = preview_insert_neighbors(preview);

    match (previous, next) {
        (_, Some(next_area_id)) => format!("insert before {}", area_label(next_area_id)),
        (Some(previous_area_id), None) => format!("insert after {}", area_label(previous_area_id)),
        (None, None) if preview.overlay.creates_new_stack => format!(
            "insert as new stack {} in {}",
            preview.overlay.target_group_index + 1,
            dock_region_label(preview.overlay.target_dock_region)
        ),
        (None, None) => format!("insert at tab {}", preview.overlay.target_tab_index + 1),
    }
}

fn preview_insert_neighbors(
    preview: &radishflow_studio::StudioGuiWindowDropPreviewModel,
) -> (Option<StudioGuiWindowAreaId>, Option<StudioGuiWindowAreaId>) {
    insert_neighbors_from_area_ids(
        &preview.overlay.target_stack_area_ids,
        preview.overlay.target_tab_index,
    )
}

fn insert_neighbors_from_area_ids(
    area_ids: &[StudioGuiWindowAreaId],
    target_tab_index: usize,
) -> (Option<StudioGuiWindowAreaId>, Option<StudioGuiWindowAreaId>) {
    let drag_index = target_tab_index.min(area_ids.len().saturating_sub(1));
    let previous = drag_index
        .checked_sub(1)
        .and_then(|index| area_ids.get(index))
        .copied();
    let next = area_ids.get(drag_index + 1).copied();
    (previous, next)
}

fn format_area_id_list(area_ids: &[StudioGuiWindowAreaId]) -> String {
    area_ids
        .iter()
        .map(|area_id| area_label(*area_id))
        .collect::<Vec<_>>()
        .join(", ")
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

fn notice_color_from_entitlement(level: rf_ui::EntitlementNoticeLevel) -> egui::Color32 {
    match level {
        rf_ui::EntitlementNoticeLevel::Info => egui::Color32::from_rgb(40, 90, 160),
        rf_ui::EntitlementNoticeLevel::Warning => egui::Color32::from_rgb(180, 120, 20),
        rf_ui::EntitlementNoticeLevel::Error => egui::Color32::from_rgb(180, 40, 40),
    }
}

fn render_status_chip(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.12))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.8)))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).color(color).small());
        });
}

fn run_status_color(status_label: &str) -> egui::Color32 {
    match status_label {
        "Converged" | "Runnable" => egui::Color32::from_rgb(54, 128, 84),
        "Solving" | "Checking" => egui::Color32::from_rgb(40, 90, 160),
        "Under-specified" | "Over-specified" | "Unconverged" => {
            egui::Color32::from_rgb(180, 120, 20)
        }
        "Error" => egui::Color32::from_rgb(180, 40, 40),
        _ => egui::Color32::from_rgb(110, 110, 110),
    }
}

fn entitlement_status_color(status_label: &str) -> egui::Color32 {
    match status_label {
        "Active" => egui::Color32::from_rgb(54, 128, 84),
        "Syncing" => egui::Color32::from_rgb(40, 90, 160),
        "Lease expired" => egui::Color32::from_rgb(180, 120, 20),
        "Error" => egui::Color32::from_rgb(180, 40, 40),
        _ => egui::Color32::from_rgb(110, 110, 110),
    }
}

fn log_level_label(level: rf_ui::AppLogLevel) -> &'static str {
    match level {
        rf_ui::AppLogLevel::Info => "info",
        rf_ui::AppLogLevel::Warning => "warn",
        rf_ui::AppLogLevel::Error => "error",
    }
}

fn format_system_time(value: SystemTime) -> String {
    let unix = value
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "before-epoch".to_string());
    let relative = match value.duration_since(SystemTime::now()) {
        Ok(duration) => format!("in {}", format_duration(duration)),
        Err(error) => format!("{} ago", format_duration(error.duration())),
    };
    format!("{relative} (unix={unix}s)")
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3_600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86_400 {
        format!("{}h{}m", seconds / 3_600, (seconds % 3_600) / 60)
    } else {
        format!("{}d{}h", seconds / 86_400, (seconds % 86_400) / 3_600)
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

#[cfg(test)]
mod tests {
    use super::*;
    use radishflow_studio::{
        StudioGuiDriverOutcome, StudioRuntimeEntitlementPreflight,
        StudioRuntimeEntitlementSeed, StudioRuntimeEntitlementSessionEvent,
        StudioRuntimeTrigger,
    };

    fn lease_expiring_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Auto,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        }
    }

    #[test]
    fn insert_neighbors_from_area_ids_returns_previous_and_next_for_middle_target() {
        let area_ids = [
            StudioGuiWindowAreaId::Commands,
            StudioGuiWindowAreaId::Canvas,
            StudioGuiWindowAreaId::Runtime,
        ];

        let (previous, next) = insert_neighbors_from_area_ids(&area_ids, 1);

        assert_eq!(previous, Some(StudioGuiWindowAreaId::Commands));
        assert_eq!(next, Some(StudioGuiWindowAreaId::Runtime));
    }

    #[test]
    fn insert_neighbors_from_area_ids_clamps_to_stack_end() {
        let area_ids = [
            StudioGuiWindowAreaId::Commands,
            StudioGuiWindowAreaId::Canvas,
        ];

        let (previous, next) = insert_neighbors_from_area_ids(&area_ids, 8);

        assert_eq!(previous, Some(StudioGuiWindowAreaId::Commands));
        assert_eq!(next, None);
    }

    #[test]
    fn clamp_overlay_pos_to_rect_keeps_overlay_inside_screen_padding() {
        let screen = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(200.0, 120.0));
        let size = egui::vec2(80.0, 40.0);

        let clamped = clamp_overlay_pos_to_rect(screen, egui::pos2(180.0, 110.0), size);

        assert_eq!(clamped, egui::pos2(112.0, 72.0));
    }

    #[test]
    fn egui_platform_timer_executor_allocates_and_clears_native_ids() {
        let mut executor = EguiPlatformTimerExecutor::default();
        let arm_schedule = radishflow_studio::StudioGuiNativeTimerSchedule {
            window_id: Some(7),
            handle_id: 41,
            slot: radishflow_studio::StudioRuntimeTimerHandleSlot {
                effect_id: 1001,
                timer: radishflow_studio::EntitlementSessionTimerArm {
                    event: radishflow_studio::EntitlementSessionLifecycleEvent::TimerElapsed,
                    due_at: SystemTime::UNIX_EPOCH + Duration::from_secs(60),
                    delay: Duration::from_secs(60),
                    reason: radishflow_studio::EntitlementSessionTimerReason::ScheduledCheck,
                },
            },
        };

        let started = executor
            .execute_platform_timer_command(&StudioGuiPlatformTimerCommand::Arm {
                schedule: arm_schedule.clone(),
            })
            .expect("expected arm response");
        assert_eq!(
            started,
            StudioGuiPlatformTimerExecutorResponse::Started { native_timer_id: 1 }
        );
        assert_eq!(
            executor.next_due_at(),
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(60))
        );
        assert!(executor.active_native_timers.contains_key(&1));

        let cleared = executor
            .execute_platform_timer_command(&StudioGuiPlatformTimerCommand::Clear {
                previous: Some(radishflow_studio::StudioGuiPlatformTimerBinding {
                    schedule: arm_schedule,
                    native_timer_id: 1,
                }),
            })
            .expect("expected clear response");
        assert_eq!(cleared, StudioGuiPlatformTimerExecutorResponse::Cleared);
        assert!(!executor.active_native_timers.contains_key(&1));
        assert_eq!(executor.next_due_at(), None);
    }

    #[test]
    fn egui_platform_timer_executor_drains_due_native_timer_ids_from_native_schedule() {
        let mut executor = EguiPlatformTimerExecutor::default();
        let arm_schedule = radishflow_studio::StudioGuiNativeTimerSchedule {
            window_id: Some(3),
            handle_id: 9,
            slot: radishflow_studio::StudioRuntimeTimerHandleSlot {
                effect_id: 2002,
                timer: radishflow_studio::EntitlementSessionTimerArm {
                    event: radishflow_studio::EntitlementSessionLifecycleEvent::TimerElapsed,
                    due_at: SystemTime::UNIX_EPOCH + Duration::from_secs(30),
                    delay: Duration::from_secs(30),
                    reason: radishflow_studio::EntitlementSessionTimerReason::ScheduledCheck,
                },
            },
        };
        executor
            .execute_platform_timer_command(&StudioGuiPlatformTimerCommand::Arm {
                schedule: arm_schedule,
            })
            .expect("expected arm response");

        let due_native_timer_ids =
            executor.drain_due_native_timer_ids(SystemTime::UNIX_EPOCH + Duration::from_secs(31));

        assert_eq!(due_native_timer_ids, vec![1]);
        assert_eq!(executor.next_due_at(), None);
    }

    #[test]
    fn drain_due_platform_timer_callbacks_dispatches_due_binding_and_rearms() {
        let mut platform_host =
            StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
        let mut executor = EguiPlatformTimerExecutor::default();
        let opened = platform_host
            .dispatch_event_and_execute_platform_timer(
                StudioGuiEvent::OpenWindowRequested,
                &mut executor,
            )
            .expect("expected opened window");
        let window_id = match opened.dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                radishflow_studio::StudioGuiHostCommandOutcome::WindowOpened(opened),
            ) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let triggered = platform_host
            .dispatch_event_and_execute_platform_timer(
                StudioGuiEvent::WindowTriggerRequested {
                    window_id,
                    trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                        StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                    ),
                },
                &mut executor,
            )
            .expect("expected timer trigger dispatch");
        assert!(triggered.dispatch.native_timer_request.is_some());

        let due_at = platform_host
            .next_native_timer_due_at()
            .expect("expected current native timer due time");
        let callbacks =
            drain_due_platform_timer_callbacks(&mut platform_host, &mut executor, due_at)
                .expect("expected due timer callbacks");

        assert_eq!(callbacks.len(), 1);
        match &callbacks[0] {
            StudioGuiPlatformExecutedNativeTimerCallbackOutcome::Dispatched(executed) => {
                assert!(matches!(
                    executed.dispatch.outcome,
                    StudioGuiDriverOutcome::HostCommand(
                        radishflow_studio::StudioGuiHostCommandOutcome::LifecycleDispatched(_)
                    )
                ));
                assert!(executed.dispatch.native_timer_request.is_some());
            }
            other => panic!("expected dispatched callback outcome, got {other:?}"),
        }
        assert!(platform_host.current_platform_timer_binding().is_some());
        assert!(platform_host.next_native_timer_due_at().is_some());
    }

    #[test]
    fn drain_due_platform_timer_callbacks_ignores_unknown_native_timer_ids() {
        let mut platform_host =
            StudioGuiPlatformHost::new(&lease_expiring_config()).expect("expected platform host");
        let mut executor = EguiPlatformTimerExecutor::default();
        let opened = platform_host
            .dispatch_event_and_execute_platform_timer(
                StudioGuiEvent::OpenWindowRequested,
                &mut executor,
            )
            .expect("expected opened window");
        let window_id = match opened.dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                radishflow_studio::StudioGuiHostCommandOutcome::WindowOpened(opened),
            ) => opened.registration.window_id,
            other => panic!("expected opened window outcome, got {other:?}"),
        };
        let _ = platform_host
            .dispatch_event_and_execute_platform_timer(
                StudioGuiEvent::WindowTriggerRequested {
                    window_id,
                    trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                        StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                    ),
                },
                &mut executor,
            )
            .expect("expected timer trigger dispatch");

        let ignored = platform_host
            .dispatch_native_timer_elapsed_by_native_id_and_execute_platform_timer(
                9999,
                &mut executor,
            )
            .expect("expected ignored callback");

        assert!(matches!(
            ignored,
            StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredStaleNativeTimer {
                native_timer_id: 9999
            } | StudioGuiPlatformExecutedNativeTimerCallbackOutcome::IgnoredUnknownNativeTimer {
                native_timer_id: 9999
            }
        ));
    }
}
