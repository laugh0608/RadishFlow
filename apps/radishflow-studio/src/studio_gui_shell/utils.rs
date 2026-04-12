use super::*;
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
    pub(super) fn allocate_native_timer_id(&mut self) -> StudioGuiPlatformNativeTimerId {
        self.next_native_timer_id = self.next_native_timer_id.saturating_add(1).max(1);
        self.next_native_timer_id
    }

    #[cfg(test)]
    pub(super) fn next_due_at(&self) -> Option<SystemTime> {
        self.active_native_timers
            .values()
            .map(|registration| registration.schedule.slot.timer.due_at)
            .min()
    }

    pub(super) fn drain_due_native_timer_ids(
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

pub(super) fn drain_due_platform_timer_callbacks(
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

pub(super) fn collect_shortcuts(input: &egui::InputState) -> Vec<StudioGuiShortcut> {
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

pub(super) fn modifiers_from_egui(modifiers: egui::Modifiers) -> Vec<StudioGuiShortcutModifier> {
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

pub(super) fn region_panel_width(
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

pub(super) fn dock_region_label(dock_region: StudioGuiWindowDockRegion) -> &'static str {
    match dock_region {
        StudioGuiWindowDockRegion::LeftSidebar => "Left",
        StudioGuiWindowDockRegion::CenterStage => "Center",
        StudioGuiWindowDockRegion::RightSidebar => "Right",
    }
}

pub(super) fn drop_preview_for_region<'a>(
    preview: Option<&'a radishflow_studio::StudioGuiWindowDropPreviewModel>,
    dock_region: StudioGuiWindowDockRegion,
) -> Option<&'a radishflow_studio::StudioGuiWindowDropPreviewModel> {
    preview.filter(|preview| preview.overlay.target_dock_region == dock_region)
}

pub(super) fn drop_preview_targets_stack(
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

pub(super) fn new_stack_preview_group_index(
    preview: Option<&radishflow_studio::StudioGuiWindowDropPreviewModel>,
) -> Option<usize> {
    preview
        .filter(|preview| preview.overlay.creates_new_stack)
        .map(|preview| preview.overlay.target_group_index)
}

pub(super) fn preview_anchor_matches_area(
    preview: Option<&radishflow_studio::StudioGuiWindowDropPreviewModel>,
    area_id: StudioGuiWindowAreaId,
) -> bool {
    preview
        .and_then(|preview| preview.overlay.anchor_area_id)
        .map(|anchor_area_id| anchor_area_id == area_id)
        .unwrap_or(false)
}

pub(super) fn render_new_stack_insert_overlay(
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

pub(super) fn area_drop_target_query(
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

pub(super) fn stack_group_drop_target_query(
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

pub(super) fn drop_lane_fill(is_active_preview: bool) -> egui::Color32 {
    if is_active_preview {
        egui::Color32::from_rgb(214, 235, 255)
    } else {
        egui::Color32::from_rgb(242, 246, 250)
    }
}

pub(super) fn drop_lane_stroke(is_active_preview: bool) -> egui::Stroke {
    if is_active_preview {
        egui::Stroke::new(1.5, egui::Color32::from_rgb(56, 126, 214))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(171, 181, 190))
    }
}

pub(super) fn stack_preview_fill(is_target_stack: bool) -> egui::Color32 {
    if is_target_stack {
        egui::Color32::from_rgb(235, 244, 255)
    } else {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0)
    }
}

pub(super) fn stack_preview_stroke(is_target_stack: bool) -> egui::Stroke {
    if is_target_stack {
        egui::Stroke::new(1.5, egui::Color32::from_rgb(56, 126, 214))
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60))
    }
}

pub(super) fn stack_accepts_overlay_anchor(
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

pub(super) fn paint_stack_tab_insert_marker(
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

pub(super) fn stack_insert_marker_x(
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

pub(super) fn preview_area_badges(
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

pub(super) fn preview_area_transition(
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

pub(super) fn area_accepts_overlay_anchor(
    window: &StudioGuiWindowModel,
    area_id: StudioGuiWindowAreaId,
) -> bool {
    let Some(preview) = window.drop_preview.as_ref() else {
        return false;
    };
    preview_anchor_matches_area(Some(preview), area_id)
        || preview.overlay.highlighted_area_ids.contains(&area_id)
}

pub(super) fn drop_preview_anchor_priority_area(
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

pub(super) fn drop_preview_anchor_priority_stack_tabs() -> u8 {
    1
}

pub(super) fn drop_preview_anchor_priority_new_stack() -> u8 {
    1
}

pub(super) fn paint_area_preview_overlay(
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

pub(super) fn preferred_overlay_pos(anchor_rect: egui::Rect) -> egui::Pos2 {
    egui::pos2(anchor_rect.right() + 12.0, anchor_rect.top() - 4.0)
}

pub(super) fn clamp_overlay_pos(ctx: &egui::Context, pos: egui::Pos2, size: egui::Vec2) -> egui::Pos2 {
    clamp_overlay_pos_to_rect(ctx.screen_rect(), pos, size)
}

pub(super) fn clamp_overlay_pos_to_rect(screen: egui::Rect, pos: egui::Pos2, size: egui::Vec2) -> egui::Pos2 {
    let max_x = (screen.right() - size.x - 8.0).max(screen.left() + 8.0);
    let max_y = (screen.bottom() - size.y - 8.0).max(screen.top() + 8.0);
    egui::pos2(
        pos.x.clamp(screen.left() + 8.0, max_x),
        pos.y.clamp(screen.top() + 8.0, max_y),
    )
}

pub(super) fn paint_preview_hint_pill_centered(painter: &egui::Painter, center: egui::Pos2, text: &str) {
    let font_id = egui::FontId::proportional(11.0);
    let text_color = egui::Color32::from_rgb(33, 82, 153);
    let galley = painter.layout_no_wrap(text.to_owned(), font_id.clone(), text_color);
    let size = galley.size() + egui::vec2(14.0, 8.0);
    let rect = egui::Rect::from_center_size(center, size);
    painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(232, 242, 255));
    painter.galley(rect.center() - galley.size() * 0.5, galley, text_color);
}

pub(super) fn paint_preview_hint_pill_top_right(painter: &egui::Painter, right_top: egui::Pos2, text: &str) {
    let font_id = egui::FontId::proportional(11.0);
    let text_color = egui::Color32::from_rgb(33, 82, 153);
    let galley = painter.layout_no_wrap(text.to_owned(), font_id.clone(), text_color);
    let size = galley.size() + egui::vec2(14.0, 8.0);
    let rect = egui::Rect::from_min_size(egui::pos2(right_top.x - size.x, right_top.y), size);
    painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(232, 242, 255));
    painter.galley(rect.center() - galley.size() * 0.5, galley, text_color);
}

pub(super) fn format_compact_drop_preview_status(
    preview: &radishflow_studio::StudioGuiWindowDropPreviewModel,
) -> String {
    format!(
        "target {} stack {} | {}",
        dock_region_label(preview.overlay.target_dock_region),
        preview.overlay.target_stack_group,
        preview_insert_hint(preview)
    )
}

pub(super) fn preview_insert_hint(preview: &radishflow_studio::StudioGuiWindowDropPreviewModel) -> String {
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

pub(super) fn preview_insert_neighbors(
    preview: &radishflow_studio::StudioGuiWindowDropPreviewModel,
) -> (Option<StudioGuiWindowAreaId>, Option<StudioGuiWindowAreaId>) {
    insert_neighbors_from_area_ids(
        &preview.overlay.target_stack_area_ids,
        preview.overlay.target_tab_index,
    )
}

pub(super) fn insert_neighbors_from_area_ids(
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

pub(super) fn format_area_id_list(area_ids: &[StudioGuiWindowAreaId]) -> String {
    area_ids
        .iter()
        .map(|area_id| area_label(*area_id))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn area_label(area_id: StudioGuiWindowAreaId) -> &'static str {
    match area_id {
        StudioGuiWindowAreaId::Commands => "Commands",
        StudioGuiWindowAreaId::Canvas => "Canvas",
        StudioGuiWindowAreaId::Runtime => "Runtime",
    }
}

pub(super) fn format_window_chip(window: &StudioAppHostWindowState) -> String {
    let role = match window.role {
        StudioWindowHostRole::EntitlementTimerOwner => "owner",
        StudioWindowHostRole::Observer => "observer",
    };
    format!("#{}/{}-{}", window.window_id, role, window.layout_slot)
}

pub(super) fn notice_color(level: RunPanelNoticeLevel) -> egui::Color32 {
    match level {
        RunPanelNoticeLevel::Info => egui::Color32::from_rgb(40, 90, 160),
        RunPanelNoticeLevel::Warning => egui::Color32::from_rgb(180, 120, 20),
        RunPanelNoticeLevel::Error => egui::Color32::from_rgb(180, 40, 40),
    }
}

pub(super) fn notice_color_from_entitlement(level: rf_ui::EntitlementNoticeLevel) -> egui::Color32 {
    match level {
        rf_ui::EntitlementNoticeLevel::Info => egui::Color32::from_rgb(40, 90, 160),
        rf_ui::EntitlementNoticeLevel::Warning => egui::Color32::from_rgb(180, 120, 20),
        rf_ui::EntitlementNoticeLevel::Error => egui::Color32::from_rgb(180, 40, 40),
    }
}

pub(super) fn render_status_chip(ui: &mut egui::Ui, label: &str, color: egui::Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.12))
        .stroke(egui::Stroke::new(1.0, color.gamma_multiply(0.8)))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(label).color(color).small());
        });
}

pub(super) fn run_status_color(status_label: &str) -> egui::Color32 {
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

pub(super) fn entitlement_status_color(status_label: &str) -> egui::Color32 {
    match status_label {
        "Active" => egui::Color32::from_rgb(54, 128, 84),
        "Syncing" => egui::Color32::from_rgb(40, 90, 160),
        "Lease expired" => egui::Color32::from_rgb(180, 120, 20),
        "Error" => egui::Color32::from_rgb(180, 40, 40),
        _ => egui::Color32::from_rgb(110, 110, 110),
    }
}

pub(super) fn log_level_label(level: rf_ui::AppLogLevel) -> &'static str {
    match level {
        rf_ui::AppLogLevel::Info => "info",
        rf_ui::AppLogLevel::Warning => "warn",
        rf_ui::AppLogLevel::Error => "error",
    }
}

pub(super) fn format_system_time(value: SystemTime) -> String {
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

pub(super) fn format_duration(duration: Duration) -> String {
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

pub(super) fn format_shortcut(shortcut: &StudioGuiShortcut) -> String {
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

pub(super) trait PaletteSelectable {
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

pub(super) fn normalized_palette_selection<T: PaletteSelectable>(
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

pub(super) fn moved_palette_selection<T: PaletteSelectable>(
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
pub(super) fn selected_palette_command_id(
    commands: &[&StudioGuiCommandEntry],
    selected_index: usize,
) -> Option<String> {
    commands
        .get(normalized_palette_selection(commands, selected_index))
        .filter(|command| command.enabled)
        .map(|command| command.command_id.clone())
}

pub(super) fn selected_palette_item_command_id(
    items: &[radishflow_studio::StudioGuiWindowCommandPaletteItemModel],
    selected_index: usize,
) -> Option<String> {
    items
        .get(normalized_palette_selection(items, selected_index))
        .filter(|item| item.enabled)
        .map(|item| item.command_id.clone())
}

