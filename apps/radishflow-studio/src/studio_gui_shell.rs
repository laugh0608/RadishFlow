use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

use eframe::egui;
use radishflow_studio::{
    StudioAppHostWindowState, StudioGuiCommandEntry, StudioGuiCommandMenuCommandModel,
    StudioGuiCommandMenuNode, StudioGuiEvent, StudioGuiFocusContext,
    StudioGuiPlatformExecutedNativeTimerCallbackBatch,
    StudioGuiPlatformExecutedNativeTimerCallbackOutcome, StudioGuiPlatformHost,
    StudioGuiPlatformNativeTimerId, StudioGuiPlatformTimerCommand, StudioGuiPlatformTimerExecutor,
    StudioGuiPlatformTimerExecutorResponse, StudioGuiPlatformTimerFollowUpCommand,
    StudioGuiShortcut, StudioGuiShortcutKey, StudioGuiShortcutModifier, StudioGuiWindowAreaId,
    StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion, StudioGuiWindowDropTargetQuery,
    StudioGuiWindowLayoutModel, StudioGuiWindowLayoutMutation, StudioGuiWindowModel,
    StudioGuiWindowPanelDisplayMode, StudioGuiWindowStackGroupLayout,
    StudioGuiWindowToolbarSectionModel, StudioRuntimeConfig, StudioWindowHostId,
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
    command_palette: CommandPaletteState,
    last_area_focus: Option<StudioGuiWindowAreaId>,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CommandPaletteState {
    open: bool,
    query: String,
    selected_index: usize,
    focus_query_input: bool,
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
                    command_palette: CommandPaletteState::default(),
                    last_area_focus: None,
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

impl CommandPaletteState {
    fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected_index = 0;
        self.focus_query_input = true;
    }

    fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.selected_index = 0;
        self.focus_query_input = false;
    }

    fn sync_selection<T: PaletteSelectable>(&mut self, commands: &[T]) {
        self.selected_index = normalized_palette_selection(commands, self.selected_index);
    }

    fn move_selection<T: PaletteSelectable>(&mut self, delta: isize, commands: &[T]) {
        self.selected_index = moved_palette_selection(commands, self.selected_index, delta);
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
        self.sync_viewport_close(ctx);
        self.sync_viewport_lifecycle(ctx);
        let toggle_shortcut_consumed = self.handle_command_palette_toggle_shortcut(ctx);
        self.drain_due_timers(ctx);
        self.drop_preview_overlay_anchor = None;

        let snapshot = self.platform_host.snapshot();
        let window = snapshot.window_model();
        let palette_keyboard_consumed = self.handle_command_palette_keyboard(ctx, &window.commands);
        if !toggle_shortcut_consumed && !palette_keyboard_consumed {
            self.dispatch_shortcuts(ctx);
        }
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
        self.render_command_palette(ctx, &window.commands);
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
                if ui.button("New logical window").clicked() {
                    self.dispatch_event(StudioGuiEvent::OpenWindowRequested);
                }
                if current_window_id.is_none() {
                    ui.small("no active logical window");
                }
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Logical windows").strong());
                if windows.is_empty() {
                    ui.small("none");
                } else {
                    self.render_logical_window_chips(ui, windows);
                }
            });
            if !window.commands.menu_tree.is_empty() {
                ui.separator();
                self.render_command_menu_bar(ui, &window.commands.menu_tree);
                ui.horizontal_wrapped(|ui| {
                    self.render_command_toolbar(ui, &window.commands.toolbar_sections);
                    let palette_label = if self.command_palette.open {
                        "Hide command palette"
                    } else {
                        "Command palette (Ctrl+K)"
                    };
                    if ui.button(palette_label).clicked() {
                        self.command_palette.toggle();
                    }
                });
            }
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

    fn render_logical_window_chips(
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
        let body_rect = ui
            .push_id(
                format!(
                    "panel:{}:{}",
                    window.layout_state.scope.layout_key,
                    area_label(area_id)
                ),
                |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            let is_drag_source = drag_session
                                .map(|drag_session| drag_session.area_id == area_id)
                                .unwrap_or(false);
                            if is_drag_source {
                                ui.small(
                                    egui::RichText::new("Dragging from header/tab")
                                        .color(egui::Color32::from_rgb(56, 126, 214)),
                                );
                            } else {
                                ui.small("Drag header or tab to move");
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
                                        StudioGuiWindowLayoutMutation::ActivateNextPanelInStack {
                                            area_id,
                                        },
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
                for section in &window.commands.command_list_sections {
                    ui.label(egui::RichText::new(section.title).strong());
                    for command in &section.items {
                        self.render_command_list_entry(ui, command);
                    }
                    ui.add_space(6.0);
                }
            });
    }

    fn render_command_list_entry(
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

    fn render_command_menu_bar(
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

    fn render_command_menu_node(&mut self, ui: &mut egui::Ui, node: &StudioGuiCommandMenuNode) {
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

    fn render_command_toolbar(
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

    fn render_command_palette(
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
                RunPanelIntent::RunManual(_) => self.dispatch_ui_command("run_panel.run_manual"),
                RunPanelIntent::Resume(_) => self.dispatch_ui_command("run_panel.resume_workspace"),
                RunPanelIntent::SetMode(SimulationMode::Hold) => {
                    self.dispatch_ui_command("run_panel.set_hold")
                }
                RunPanelIntent::SetMode(SimulationMode::Active) => {
                    self.dispatch_ui_command("run_panel.set_active")
                }
            },
            RunPanelWidgetEvent::Disabled { .. } | RunPanelWidgetEvent::Missing { .. } => {}
        }
    }

    fn dispatch_menu_command(&mut self, command: &StudioGuiCommandMenuCommandModel) {
        self.dispatch_ui_command(&command.command_id);
    }

    fn dispatch_ui_command(&mut self, command_id: impl Into<String>) {
        self.dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: command_id.into(),
        });
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
            Ok(callback_batch) => {
                for callback in callback_batch.callbacks {
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

        if let Some(next_due_at) = self.platform_host.next_native_timer_due_at() {
            let delay = next_due_at.duration_since(now).unwrap_or(Duration::ZERO);
            ctx.request_repaint_after(delay);
        }
    }

    fn sync_viewport_close(&mut self, ctx: &egui::Context) {
        if !ctx.input(|input| input.viewport().close_requested()) {
            return;
        }

        let Some(window_id) = self.current_window_id() else {
            return;
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        self.cancel_drag_session(Some(window_id));
        self.dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id });

        if self.logical_window_count() == 0 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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

        let window_id = self.current_window_id();
        if let Some(window_id) = window_id {
            self.dispatch_event(StudioGuiEvent::WindowForegrounded { window_id });
        }
    }

    fn handle_command_palette_toggle_shortcut(&mut self, ctx: &egui::Context) -> bool {
        let toggle_requested =
            ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::K));
        if toggle_requested {
            self.command_palette.toggle();
        }
        toggle_requested
    }

    fn handle_command_palette_keyboard(
        &mut self,
        ctx: &egui::Context,
        commands: &radishflow_studio::StudioGuiWindowCommandAreaModel,
    ) -> bool {
        if !self.command_palette.open {
            return false;
        }

        let palette_items = commands.palette_items(&self.command_palette.query);
        self.command_palette.sync_selection(&palette_items);

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.command_palette.close();
            return true;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::ArrowDown)) {
            self.command_palette.move_selection(1, &palette_items);
            return true;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::ArrowUp)) {
            self.command_palette.move_selection(-1, &palette_items);
            return true;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::Enter)) {
            let selected_command_id = selected_palette_item_command_id(
                &palette_items,
                self.command_palette.selected_index,
            );
            if let Some(command_id) = selected_command_id {
                self.dispatch_ui_command(command_id);
                self.command_palette.close();
            }
            return true;
        }

        false
    }

    fn dispatch_shortcuts(&mut self, ctx: &egui::Context) {
        let focus_context = self.focus_context(ctx);
        if matches!(focus_context, StudioGuiFocusContext::CommandPalette) {
            return;
        }

        if self.drag_session.is_some() && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.cancel_drag_session(self.current_window_id());
            return;
        }

        let shortcuts = ctx.input(|input| collect_shortcuts(input));
        for shortcut in shortcuts {
            self.dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut,
                focus_context,
            });
        }
    }

    fn focus_context(&self, ctx: &egui::Context) -> StudioGuiFocusContext {
        if self.command_palette.open {
            StudioGuiFocusContext::CommandPalette
        } else if ctx.wants_keyboard_input() {
            StudioGuiFocusContext::TextInput
        } else if self
            .platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .is_some()
        {
            StudioGuiFocusContext::CanvasSuggestionFocused
        } else if self.last_area_focus == Some(StudioGuiWindowAreaId::Canvas) {
            StudioGuiFocusContext::Canvas
        } else {
            StudioGuiFocusContext::Global
        }
    }

    fn current_window_id(&self) -> Option<StudioWindowHostId> {
        self.platform_host
            .snapshot()
            .window_model()
            .layout_state
            .scope
            .window_id
    }

    fn logical_window_count(&self) -> usize {
        self.platform_host.snapshot().app_host_state.windows.len()
    }

    fn update_area_focus_from_rect(
        &mut self,
        ctx: &egui::Context,
        area_id: StudioGuiWindowAreaId,
        rect: egui::Rect,
    ) {
        let pointer_pos = ctx.pointer_latest_pos();
        let pressed = ctx.input(|input| input.pointer.any_pressed());
        let released = ctx.input(|input| input.pointer.any_released());
        if pointer_pos.is_some_and(|pos| rect.contains(pos)) && (pressed || released) {
            self.last_area_focus = Some(area_id);
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
                Ok(StudioGuiPlatformTimerExecutorResponse::Started { native_timer_id })
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
                Ok(StudioGuiPlatformTimerExecutorResponse::Started { native_timer_id })
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

    #[cfg(test)]
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
) -> RfResult<StudioGuiPlatformExecutedNativeTimerCallbackBatch> {
    let due_native_timer_ids = executor.drain_due_native_timer_ids(now);
    platform_host.dispatch_native_timer_elapsed_by_native_ids_and_execute_platform_timers(
        &due_native_timer_ids,
        executor,
    )
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

trait PaletteSelectable {
    fn enabled(&self) -> bool;
}

impl PaletteSelectable for &StudioGuiCommandEntry {
    fn enabled(&self) -> bool {
        self.enabled
    }
}

impl PaletteSelectable for radishflow_studio::StudioGuiWindowCommandPaletteItemModel {
    fn enabled(&self) -> bool {
        self.enabled
    }
}

fn normalized_palette_selection<T: PaletteSelectable>(
    commands: &[T],
    current_index: usize,
) -> usize {
    if commands.is_empty() {
        return 0;
    }

    let clamped = current_index.min(commands.len() - 1);
    if commands[clamped].enabled() || !commands.iter().any(PaletteSelectable::enabled) {
        return clamped;
    }

    (clamped..commands.len())
        .find(|index| commands[*index].enabled())
        .or_else(|| (0..clamped).rev().find(|index| commands[*index].enabled()))
        .unwrap_or(0)
}

fn moved_palette_selection<T: PaletteSelectable>(
    commands: &[T],
    current_index: usize,
    delta: isize,
) -> usize {
    if commands.is_empty() {
        return 0;
    }

    let last = (commands.len() - 1) as isize;
    if !commands.iter().any(PaletteSelectable::enabled) {
        let current = current_index.min(commands.len() - 1) as isize;
        return (current + delta).clamp(0, last) as usize;
    }

    let mut selected = normalized_palette_selection(commands, current_index);
    for _ in 0..delta.unsigned_abs() {
        let step = delta.signum();
        if step == 0 {
            break;
        }

        let mut candidate = selected as isize;
        let mut advanced = false;
        loop {
            candidate += step;
            if candidate < 0 || candidate > last {
                break;
            }
            if commands[candidate as usize].enabled() {
                selected = candidate as usize;
                advanced = true;
                break;
            }
        }
        if !advanced {
            break;
        }
    }

    selected
}

#[cfg(test)]
fn selected_palette_command_id(
    commands: &[&StudioGuiCommandEntry],
    selected_index: usize,
) -> Option<String> {
    commands
        .get(normalized_palette_selection(commands, selected_index))
        .filter(|command| command.enabled)
        .map(|command| command.command_id.clone())
}

fn selected_palette_item_command_id(
    items: &[radishflow_studio::StudioGuiWindowCommandPaletteItemModel],
    selected_index: usize,
) -> Option<String> {
    items
        .get(normalized_palette_selection(items, selected_index))
        .filter(|item| item.enabled)
        .map(|item| item.command_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use radishflow_studio::{
        StudioGuiDriverOutcome, StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger,
    };
    use std::{fs, path::PathBuf, time::UNIX_EPOCH};

    fn lease_expiring_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Auto,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        }
    }

    fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-studio-shell-local-rules-{timestamp}.rfproj.json"
        ));
        let project =
            include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json")
                .replacen(
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-heated\"",
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-liquid\"",
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                );
        fs::write(&project_path, project).expect("expected local rules project");

        (
            StudioRuntimeConfig {
                project_path: project_path.clone(),
                ..lease_expiring_config()
            },
            project_path,
        )
    }

    fn unbound_outlet_failure_synced_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            project_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("examples")
                .join("flowsheets")
                .join("failures")
                .join("unbound-outlet-port.rfproj.json"),
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::Synced,
            trigger: StudioRuntimeTrigger::WidgetAction(rf_ui::RunPanelActionId::RunManual),
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
    fn command_palette_state_open_close_and_selection_reset_cleanly() {
        let mut state = CommandPaletteState {
            open: false,
            query: "recover".to_string(),
            selected_index: 3,
            focus_query_input: false,
        };

        state.open();

        assert!(state.open);
        assert!(state.query.is_empty());
        assert_eq!(state.selected_index, 0);
        assert!(state.focus_query_input);

        state.close();

        assert!(!state.open);
        assert!(state.query.is_empty());
        assert_eq!(state.selected_index, 0);
        assert!(!state.focus_query_input);
    }

    #[test]
    fn command_palette_state_moves_selection_within_bounds() {
        let commands = palette_commands_for_test(&[
            ("run_panel.run_manual", true),
            ("run_panel.recover_failure", false),
            ("run_panel.resume_workspace", true),
        ]);
        let mut state = CommandPaletteState {
            open: true,
            query: String::new(),
            selected_index: 0,
            focus_query_input: false,
        };

        state.move_selection(1, &commands);
        assert_eq!(state.selected_index, 2);

        state.move_selection(1, &commands);
        assert_eq!(state.selected_index, 2);

        state.move_selection(-5, &commands);
        assert_eq!(state.selected_index, 0);

        let empty_commands: [&StudioGuiCommandEntry; 0] = [];
        state.move_selection(1, &empty_commands);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn command_palette_state_syncs_disabled_selection_to_nearest_enabled_command() {
        let commands = palette_commands_for_test(&[
            ("run_panel.run_manual", false),
            ("run_panel.resume_workspace", false),
            ("run_panel.set_active", true),
            ("run_panel.recover_failure", false),
        ]);
        let mut state = CommandPaletteState {
            open: true,
            query: String::new(),
            selected_index: 1,
            focus_query_input: false,
        };

        state.sync_selection(&commands);

        assert_eq!(state.selected_index, 2);
    }

    #[test]
    fn selected_palette_command_id_ignores_disabled_entries() {
        let commands = palette_commands_for_test(&[
            ("run_panel.run_manual", false),
            ("run_panel.resume_workspace", true),
        ]);

        assert_eq!(
            selected_palette_command_id(&commands, 0),
            Some("run_panel.resume_workspace".to_string())
        );
    }

    #[test]
    fn focus_context_prioritizes_command_palette_over_canvas_focus() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut app = ready_app_state(&config);
        assert!(
            app.platform_host
                .snapshot()
                .window_model()
                .canvas
                .focused_suggestion_id
                .is_some()
        );

        app.command_palette.open();

        run_with_key_press(egui::Key::F5, egui::Modifiers::NONE, |ctx| {
            assert_eq!(
                app.focus_context(ctx),
                StudioGuiFocusContext::CommandPalette
            );
        });

        let _ = std::fs::remove_file(project_path);
    }

    #[test]
    fn focus_context_prioritizes_text_input_over_canvas_focus() {
        let (config, project_path) = flash_drum_local_rules_config();
        let app = ready_app_state(&config);
        assert!(
            app.platform_host
                .snapshot()
                .window_model()
                .canvas
                .focused_suggestion_id
                .is_some()
        );

        run_with_key_press_and_focus(
            egui::Key::F5,
            egui::Modifiers::NONE,
            egui::Id::new("studio.test_input"),
            |ctx| {
                assert_eq!(app.focus_context(ctx), StudioGuiFocusContext::TextInput);
            },
        );

        let _ = std::fs::remove_file(project_path);
    }

    #[test]
    fn focus_context_reports_canvas_when_canvas_was_last_interacted_area() {
        let mut app = ready_app_state(&lease_expiring_config());
        app.last_area_focus = Some(StudioGuiWindowAreaId::Canvas);

        run_with_key_press(egui::Key::F5, egui::Modifiers::NONE, |ctx| {
            assert_eq!(app.focus_context(ctx), StudioGuiFocusContext::Canvas);
        });
    }

    #[test]
    fn focus_context_keeps_global_when_non_canvas_area_was_last_interacted() {
        let mut app = ready_app_state(&lease_expiring_config());
        app.last_area_focus = Some(StudioGuiWindowAreaId::Runtime);

        run_with_key_press(egui::Key::F5, egui::Modifiers::NONE, |ctx| {
            assert_eq!(app.focus_context(ctx), StudioGuiFocusContext::Global);
        });
    }

    #[test]
    fn command_palette_toggle_shortcut_opens_and_closes_palette() {
        let mut app = ready_app_state(&lease_expiring_config());
        assert!(!app.command_palette.open);

        run_with_key_press(
            egui::Key::K,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..egui::Modifiers::NONE
            },
            |ctx| {
                assert!(app.handle_command_palette_toggle_shortcut(ctx));
            },
        );
        assert!(app.command_palette.open);
        assert!(app.command_palette.focus_query_input);

        run_with_key_press(
            egui::Key::K,
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..egui::Modifiers::NONE
            },
            |ctx| {
                assert!(app.handle_command_palette_toggle_shortcut(ctx));
            },
        );
        assert!(!app.command_palette.open);
    }

    #[test]
    fn command_palette_enter_executes_selected_command_and_closes_palette() {
        let mut app = ready_app_state(&lease_expiring_config());
        app.command_palette.open();
        app.command_palette.query = "activate".to_string();

        let commands = app.platform_host.snapshot().window_model().commands;
        run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
            assert!(app.handle_command_palette_keyboard(ctx, &commands));
        });

        assert!(!app.command_palette.open);
        assert_eq!(
            app.platform_host
                .snapshot()
                .window_model()
                .runtime
                .control_state
                .simulation_mode,
            SimulationMode::Active
        );
    }

    #[test]
    fn command_surface_interactions_converge_to_same_window_state_for_activate_workspace() {
        let mut menu_app = ready_app_state(&lease_expiring_config());
        let mut toolbar_app = ready_app_state(&lease_expiring_config());
        let mut palette_app = ready_app_state(&lease_expiring_config());

        let initial_window = menu_app.platform_host.snapshot().window_model();
        let menu_command =
            find_menu_command(&initial_window.commands.menu_tree, "run_panel.set_active")
                .cloned()
                .expect("expected activate menu command");
        let toolbar_command_id = find_toolbar_command_id(
            &initial_window.commands.toolbar_sections,
            "run_panel.set_active",
        )
        .expect("expected activate toolbar command");

        menu_app.dispatch_menu_command(&menu_command);
        toolbar_app.dispatch_ui_command(toolbar_command_id);

        palette_app.command_palette.open();
        palette_app.command_palette.query = "activate".to_string();
        let commands = palette_app.platform_host.snapshot().window_model().commands;
        assert_eq!(
            selected_palette_item_command_id(&commands.palette_items("activate"), 0),
            Some("run_panel.set_active".to_string())
        );
        run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
            assert!(palette_app.handle_command_palette_keyboard(ctx, &commands));
        });

        let menu_window = menu_app.platform_host.snapshot().window_model();
        let toolbar_window = toolbar_app.platform_host.snapshot().window_model();
        let palette_window = palette_app.platform_host.snapshot().window_model();

        assert!(!palette_app.command_palette.open);
        assert_eq!(
            menu_window.runtime.control_state.simulation_mode,
            SimulationMode::Active
        );
        assert_eq!(menu_window, toolbar_window);
        assert_eq!(menu_window, palette_window);
    }

    #[test]
    fn command_surface_interactions_converge_to_same_window_state_for_run_panel_recovery() {
        let mut menu_app = ready_failed_app_state();
        let mut toolbar_app = ready_failed_app_state();
        let mut palette_app = ready_failed_app_state();

        let failed_window = menu_app.platform_host.snapshot().window_model();
        let menu_command = find_menu_command(
            &failed_window.commands.menu_tree,
            "run_panel.recover_failure",
        )
        .cloned()
        .expect("expected recovery menu command");
        let toolbar_command_id = find_toolbar_command_id(
            &failed_window.commands.toolbar_sections,
            "run_panel.recover_failure",
        )
        .expect("expected recovery toolbar command");

        menu_app.dispatch_menu_command(&menu_command);
        toolbar_app.dispatch_ui_command(toolbar_command_id);

        palette_app.command_palette.open();
        palette_app.command_palette.query = "diagnostic".to_string();
        let commands = palette_app.platform_host.snapshot().window_model().commands;
        assert_eq!(
            selected_palette_item_command_id(&commands.palette_items("diagnostic"), 0),
            Some("run_panel.recover_failure".to_string())
        );
        run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
            assert!(palette_app.handle_command_palette_keyboard(ctx, &commands));
        });

        let menu_window = menu_app.platform_host.snapshot().window_model();
        let toolbar_window = toolbar_app.platform_host.snapshot().window_model();
        let palette_window = palette_app.platform_host.snapshot().window_model();

        assert!(!palette_app.command_palette.open);
        assert_eq!(menu_window.runtime.control_state.run_status, rf_ui::RunStatus::Dirty);
        assert_eq!(
            menu_window.runtime.control_state.pending_reason,
            Some(rf_ui::SolvePendingReason::DocumentRevisionAdvanced)
        );
        assert_eq!(
            menu_window.runtime.run_panel.view().primary_action.label,
            "Resume"
        );
        assert_eq!(menu_window, toolbar_window);
        assert_eq!(menu_window, palette_window);
    }

    #[test]
    fn disabled_command_surface_interactions_do_not_change_window_state_for_run_panel_recovery() {
        let mut menu_app = ready_app_state(&lease_expiring_config());
        let mut toolbar_app = ready_app_state(&lease_expiring_config());
        let mut palette_app = ready_app_state(&lease_expiring_config());

        let initial_window = menu_app.platform_host.snapshot().window_model();
        assert_eq!(toolbar_app.platform_host.snapshot().window_model(), initial_window);
        assert_eq!(palette_app.platform_host.snapshot().window_model(), initial_window);

        let menu_command = find_menu_command(
            &initial_window.commands.menu_tree,
            "run_panel.recover_failure",
        )
        .cloned()
        .expect("expected recovery menu command");
        assert!(
            !menu_command.enabled,
            "expected recovery menu command to stay disabled before failure"
        );

        let toolbar_command = initial_window
            .commands
            .toolbar_sections
            .iter()
            .flat_map(|section| section.items.iter())
            .find(|command| command.command_id == "run_panel.recover_failure")
            .cloned()
            .expect("expected recovery toolbar command");
        assert!(
            !toolbar_command.enabled,
            "expected recovery toolbar command to stay disabled before failure"
        );

        menu_app.dispatch_menu_command(&menu_command);
        toolbar_app.dispatch_ui_command(&toolbar_command.command_id);

        palette_app.command_palette.open();
        palette_app.command_palette.query = "diagnostic".to_string();
        let commands = palette_app.platform_host.snapshot().window_model().commands;
        let palette_items = commands.palette_items("diagnostic");
        assert_eq!(palette_items.len(), 1);
        assert_eq!(
            palette_items[0].command_id,
            "run_panel.recover_failure".to_string()
        );
        assert!(
            !palette_items[0].enabled,
            "expected recovery palette item to stay disabled before failure"
        );
        assert_eq!(selected_palette_item_command_id(&palette_items, 0), None);
        run_with_key_press(egui::Key::Enter, egui::Modifiers::NONE, |ctx| {
            assert!(palette_app.handle_command_palette_keyboard(ctx, &commands));
        });

        let menu_window = menu_app.platform_host.snapshot().window_model();
        let toolbar_window = toolbar_app.platform_host.snapshot().window_model();
        let palette_window = palette_app.platform_host.snapshot().window_model();

        assert_eq!(menu_window, initial_window);
        assert_eq!(menu_window, toolbar_window);
        assert_eq!(menu_window, palette_window);
        assert!(palette_app.command_palette.open);
        assert_eq!(palette_app.command_palette.query, "diagnostic");
    }

    #[test]
    fn dispatch_shortcuts_does_not_leak_host_shortcuts_while_palette_is_open() {
        let mut app = ready_app_state(&lease_expiring_config());
        app.command_palette.open();

        run_with_key_press(
            egui::Key::F6,
            egui::Modifiers {
                shift: true,
                ..egui::Modifiers::NONE
            },
            |ctx| app.dispatch_shortcuts(ctx),
        );

        assert!(app.command_palette.open);
        assert_eq!(
            app.platform_host
                .snapshot()
                .window_model()
                .runtime
                .control_state
                .simulation_mode,
            SimulationMode::Hold
        );
    }

    #[test]
    fn dispatch_shortcuts_allows_function_keys_from_text_input_context() {
        let mut app = ready_app_state(&lease_expiring_config());

        run_with_key_press_and_focus(
            egui::Key::F6,
            egui::Modifiers {
                shift: true,
                ..egui::Modifiers::NONE
            },
            egui::Id::new("studio.test_input"),
            |ctx| app.dispatch_shortcuts(ctx),
        );

        assert_eq!(
            app.platform_host
                .snapshot()
                .window_model()
                .runtime
                .control_state
                .simulation_mode,
            SimulationMode::Active
        );
    }

    #[test]
    fn command_palette_items_surface_window_model_results() {
        let app = ready_app_state(&lease_expiring_config());

        let filtered = app
            .platform_host
            .snapshot()
            .window_model()
            .commands
            .palette_items("activate");

        assert_eq!(
            filtered
                .into_iter()
                .map(|item| item.command_id)
                .collect::<Vec<_>>(),
            vec!["run_panel.set_active".to_string()]
        );
    }

    fn palette_commands_for_test(commands: &[(&str, bool)]) -> Vec<&'static StudioGuiCommandEntry> {
        commands
            .iter()
            .map(|(command_id, enabled)| {
                let entry = Box::leak(Box::new(StudioGuiCommandEntry {
                    command_id: (*command_id).to_string(),
                    label: (*command_id).to_string(),
                    detail: "test".to_string(),
                    enabled: *enabled,
                    sort_order: 100,
                    target_window_id: None,
                    menu_path: vec!["Commands".to_string()],
                    search_terms: Vec::new(),
                    shortcut: None,
                }));
                &*entry
            })
            .collect()
    }

    fn find_menu_command<'a>(
        nodes: &'a [StudioGuiCommandMenuNode],
        command_id: &str,
    ) -> Option<&'a StudioGuiCommandMenuCommandModel> {
        for node in nodes {
            if let Some(command) = node.command.as_ref() {
                if command.command_id == command_id {
                    return Some(command);
                }
            }
            if let Some(command) = find_menu_command(&node.children, command_id) {
                return Some(command);
            }
        }
        None
    }

    fn find_toolbar_command_id<'a>(
        sections: &'a [StudioGuiWindowToolbarSectionModel],
        command_id: &str,
    ) -> Option<&'a str> {
        sections
            .iter()
            .flat_map(|section| section.items.iter())
            .find(|command| command.command_id == command_id)
            .map(|command| command.command_id.as_str())
    }

    fn ready_failed_app_state() -> ReadyAppState {
        let mut app = ready_app_state(&unbound_outlet_failure_synced_config());
        app.dispatch_ui_command("run_panel.run_manual");

        let window = app.platform_host.snapshot().window_model();
        assert_eq!(window.runtime.control_state.run_status, rf_ui::RunStatus::Error);
        assert!(
            find_menu_command(&window.commands.menu_tree, "run_panel.recover_failure")
                .map(|command| command.enabled)
                .unwrap_or(false),
            "expected recovery command to be enabled after failed run"
        );

        app
    }

    fn ready_app_state(config: &StudioRuntimeConfig) -> ReadyAppState {
        let mut app = ReadyAppState {
            platform_host: StudioGuiPlatformHost::new(config).expect("expected platform host"),
            platform_timer_executor: EguiPlatformTimerExecutor::default(),
            last_error: None,
            command_palette: CommandPaletteState::default(),
            last_area_focus: None,
            drag_session: None,
            active_drop_preview: None,
            drop_preview_overlay_anchor: None,
            last_viewport_focused: None,
        };
        app.dispatch_event(StudioGuiEvent::OpenWindowRequested);
        app
    }

    fn run_with_key_press<R>(
        key: egui::Key,
        modifiers: egui::Modifiers,
        run: impl FnOnce(&egui::Context) -> R,
    ) -> R {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 720.0),
            )),
            focused: true,
            modifiers,
            events: vec![egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed: true,
                repeat: false,
                modifiers,
            }],
            ..Default::default()
        });
        let output = run(&ctx);
        let _ = ctx.end_pass();
        output
    }

    fn run_with_key_press_and_focus<R>(
        key: egui::Key,
        modifiers: egui::Modifiers,
        focused_id: egui::Id,
        run: impl FnOnce(&egui::Context) -> R,
    ) -> R {
        let ctx = egui::Context::default();
        ctx.begin_pass(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 720.0),
            )),
            focused: true,
            modifiers,
            events: vec![egui::Event::Key {
                key,
                physical_key: Some(key),
                pressed: true,
                repeat: false,
                modifiers,
            }],
            ..Default::default()
        });
        ctx.memory_mut(|mem| mem.request_focus(focused_id));
        let output = run(&ctx);
        let _ = ctx.end_pass();
        output
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

        assert_eq!(callbacks.callbacks.len(), 1);
        assert_eq!(
            callbacks.next_native_timer_due_at(),
            platform_host.next_native_timer_due_at()
        );
        match &callbacks.callbacks[0] {
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
