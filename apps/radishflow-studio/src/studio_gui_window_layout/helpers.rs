use super::*;

pub(super) fn build_stack_group_layouts(
    state: &StudioGuiWindowLayoutState,
    panels: &[StudioGuiWindowPanelLayout],
) -> Vec<StudioGuiWindowStackGroupLayout> {
    let mut groups = state
        .stack_groups
        .iter()
        .map(|group| {
            let mut tabs = panels
                .iter()
                .filter(|panel| {
                    panel.dock_region == group.dock_region && panel.stack_group == group.stack_group
                })
                .map(|panel| StudioGuiWindowStackTabLayout {
                    area_id: panel.area_id,
                    title: panel.title,
                    active: panel.area_id == group.active_area_id,
                    visible: panel.visible,
                    collapsed: panel.collapsed,
                })
                .collect::<Vec<_>>();
            tabs.sort_by_key(|tab| {
                panels
                    .iter()
                    .find(|panel| panel.area_id == tab.area_id)
                    .map(|panel| panel.order)
                    .unwrap_or(u8::MAX)
            });
            StudioGuiWindowStackGroupLayout {
                dock_region: group.dock_region,
                stack_group: group.stack_group,
                tabbed: tabs.len() > 1,
                active_area_id: group.active_area_id,
                tabs,
            }
        })
        .collect::<Vec<_>>();
    groups.sort_by_key(|group| (dock_region_rank(group.dock_region), group.stack_group));
    groups
}

pub(super) fn layout_scope_from_state(
    state: &StudioAppHostState,
    requested_window_id: Option<StudioWindowHostId>,
) -> StudioGuiWindowLayoutScope {
    let window_id = requested_window_id
        .filter(|window_id| state.window(*window_id).is_some())
        .or(state.foreground_window_id)
        .or_else(|| state.registered_windows.first().copied());

    match window_id {
        Some(window_id) => {
            let window = state.window(window_id);
            let window_role = window.map(|window| window.role);
            let layout_slot = window.map(|window| window.layout_slot);
            StudioGuiWindowLayoutScope {
                kind: StudioGuiWindowLayoutScopeKind::Window,
                window_id: Some(window_id),
                window_role,
                layout_slot,
                layout_key: layout_key_for_window(window_role, layout_slot),
            }
        }
        None => StudioGuiWindowLayoutScope {
            kind: StudioGuiWindowLayoutScopeKind::EmptyWorkspace,
            window_id: None,
            window_role: None,
            layout_slot: None,
            layout_key: "studio.window.empty".to_string(),
        },
    }
}

pub(super) fn layout_key_for_window(
    window_role: Option<StudioWindowHostRole>,
    layout_slot: Option<u16>,
) -> String {
    match (window_role, layout_slot) {
        (Some(StudioWindowHostRole::EntitlementTimerOwner), Some(layout_slot)) => {
            format!("studio.window.owner.slot-{layout_slot}")
        }
        (Some(StudioWindowHostRole::Observer), Some(layout_slot)) => {
            format!("studio.window.observer.slot-{layout_slot}")
        }
        (Some(StudioWindowHostRole::EntitlementTimerOwner), None) => {
            "studio.window.owner.slot-1".to_string()
        }
        (Some(StudioWindowHostRole::Observer), None) => "studio.window.observer.slot-1".to_string(),
        (None, Some(layout_slot)) => format!("studio.window.window.slot-{layout_slot}"),
        (None, None) => "studio.window.window.slot-1".to_string(),
    }
}

pub(super) fn legacy_layout_key_for_window(
    window_id: StudioWindowHostId,
    window_role: StudioWindowHostRole,
) -> String {
    match window_role {
        StudioWindowHostRole::EntitlementTimerOwner => format!("studio.window.owner.{window_id}"),
        StudioWindowHostRole::Observer => format!("studio.window.observer.{window_id}"),
    }
}

pub(super) fn default_panel_states() -> Vec<StudioGuiWindowPanelLayoutState> {
    vec![
        default_panel_state(StudioGuiWindowAreaId::Commands),
        default_panel_state(StudioGuiWindowAreaId::Canvas),
        default_panel_state(StudioGuiWindowAreaId::Runtime),
    ]
}

pub(super) fn default_stack_group_states(
    panels: &[StudioGuiWindowPanelLayoutState],
) -> Vec<StudioGuiWindowStackGroupState> {
    derive_stack_group_states(panels, &[])
}

pub(super) fn default_panel_state(
    area_id: StudioGuiWindowAreaId,
) -> StudioGuiWindowPanelLayoutState {
    match area_id {
        StudioGuiWindowAreaId::Commands => StudioGuiWindowPanelLayoutState {
            area_id,
            dock_region: StudioGuiWindowDockRegion::LeftSidebar,
            stack_group: 10,
            order: 10,
            visible: true,
            collapsed: false,
        },
        StudioGuiWindowAreaId::Canvas => StudioGuiWindowPanelLayoutState {
            area_id,
            dock_region: StudioGuiWindowDockRegion::CenterStage,
            stack_group: 10,
            order: 20,
            visible: true,
            collapsed: false,
        },
        StudioGuiWindowAreaId::Runtime => StudioGuiWindowPanelLayoutState {
            area_id,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            stack_group: 10,
            order: 30,
            visible: true,
            collapsed: false,
        },
    }
}

pub(super) fn default_region_weights() -> Vec<StudioGuiWindowRegionWeight> {
    vec![
        StudioGuiWindowRegionWeight {
            dock_region: StudioGuiWindowDockRegion::LeftSidebar,
            weight: 24,
        },
        StudioGuiWindowRegionWeight {
            dock_region: StudioGuiWindowDockRegion::CenterStage,
            weight: 52,
        },
        StudioGuiWindowRegionWeight {
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            weight: 24,
        },
    ]
}

pub(super) fn upsert_panel_state(
    panels: &mut Vec<StudioGuiWindowPanelLayoutState>,
    panel: StudioGuiWindowPanelLayoutState,
) {
    if let Some(index) = panels
        .iter()
        .position(|candidate| candidate.area_id == panel.area_id)
    {
        panels[index] = panel;
    } else {
        panels.push(panel);
    }
    sort_panels(panels);
}

pub(super) fn upsert_stack_group_state(
    stack_groups: &mut Vec<StudioGuiWindowStackGroupState>,
    stack_group: StudioGuiWindowStackGroupState,
) {
    if let Some(index) = stack_groups.iter().position(|candidate| {
        candidate.dock_region == stack_group.dock_region
            && candidate.stack_group == stack_group.stack_group
    }) {
        stack_groups[index] = stack_group;
    } else {
        stack_groups.push(stack_group);
    }
    sort_stack_groups(stack_groups);
}

pub(super) fn set_active_panel_in_stack(
    stack_groups: &mut Vec<StudioGuiWindowStackGroupState>,
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
    area_id: StudioGuiWindowAreaId,
) {
    upsert_stack_group_state(
        stack_groups,
        StudioGuiWindowStackGroupState {
            dock_region,
            stack_group,
            active_area_id: area_id,
        },
    );
}

pub(super) fn set_active_panel_for_area(
    layout: &mut StudioGuiWindowLayoutState,
    area_id: StudioGuiWindowAreaId,
) {
    let mut panel = layout
        .panel(area_id)
        .cloned()
        .unwrap_or_else(|| default_panel_state(area_id));
    panel.visible = true;
    let dock_region = panel.dock_region;
    let stack_group = panel.stack_group;
    upsert_panel_state(&mut layout.panels, panel);
    set_active_panel_in_stack(&mut layout.stack_groups, dock_region, stack_group, area_id);
    reconcile_center_area_with_active_panel(layout, area_id);
}

pub(super) fn reconcile_stack_group_states(layout: &mut StudioGuiWindowLayoutState) {
    layout.stack_groups = derive_stack_group_states(&layout.panels, &layout.stack_groups);
}

pub(super) fn reconcile_center_area_with_active_panel(
    layout: &mut StudioGuiWindowLayoutState,
    area_id: StudioGuiWindowAreaId,
) {
    if layout
        .panel(area_id)
        .map(|panel| panel.dock_region == StudioGuiWindowDockRegion::CenterStage)
        .unwrap_or(false)
    {
        layout.center_area = area_id;
    }
}

pub(super) fn derive_stack_group_states(
    panels: &[StudioGuiWindowPanelLayoutState],
    preferred: &[StudioGuiWindowStackGroupState],
) -> Vec<StudioGuiWindowStackGroupState> {
    let mut groups = grouped_area_ids_by_region(panels)
        .into_iter()
        .flat_map(|(dock_region, grouped_area_ids)| {
            grouped_area_ids
                .into_iter()
                .enumerate()
                .map(move |(group_index, area_ids)| {
                    let stack_group = dock_group_order(group_index);
                    let preferred_active = preferred
                        .iter()
                        .find(|candidate| {
                            candidate.dock_region == dock_region
                                && candidate.stack_group == stack_group
                        })
                        .map(|candidate| candidate.active_area_id);
                    let active_area_id =
                        choose_active_area_id_for_stack(panels, &area_ids, preferred_active);
                    StudioGuiWindowStackGroupState {
                        dock_region,
                        stack_group,
                        active_area_id,
                    }
                })
        })
        .collect::<Vec<_>>();
    sort_stack_groups(&mut groups);
    groups
}

pub(super) fn choose_active_area_id_for_stack(
    panels: &[StudioGuiWindowPanelLayoutState],
    area_ids: &[StudioGuiWindowAreaId],
    preferred_active: Option<StudioGuiWindowAreaId>,
) -> StudioGuiWindowAreaId {
    if preferred_active
        .filter(|area_id| {
            area_ids.contains(area_id)
                && panels
                    .iter()
                    .find(|panel| panel.area_id == *area_id)
                    .map(|panel| panel.visible)
                    .unwrap_or(false)
        })
        .is_some()
    {
        return preferred_active.expect("preferred active just checked");
    }

    area_ids
        .iter()
        .copied()
        .find(|area_id| {
            panels
                .iter()
                .find(|panel| panel.area_id == *area_id)
                .map(|panel| panel.visible)
                .unwrap_or(false)
        })
        .or_else(|| area_ids.first().copied())
        .unwrap_or(StudioGuiWindowAreaId::Canvas)
}

pub(super) fn place_panel_in_dock_region(
    panels: &mut [StudioGuiWindowPanelLayoutState],
    area_id: StudioGuiWindowAreaId,
    dock_region: StudioGuiWindowDockRegion,
    placement: StudioGuiWindowDockPlacement,
) {
    let mut grouped_area_ids =
        grouped_area_ids_for_region_excluding_area(panels, dock_region, Some(area_id));
    let insert_index = match placement {
        StudioGuiWindowDockPlacement::Start => 0,
        StudioGuiWindowDockPlacement::End => grouped_area_ids.len(),
        StudioGuiWindowDockPlacement::Before { anchor_area_id } => grouped_area_ids
            .iter()
            .position(|group| group.contains(&anchor_area_id))
            .unwrap_or(grouped_area_ids.len()),
        StudioGuiWindowDockPlacement::After { anchor_area_id } => grouped_area_ids
            .iter()
            .position(|group| group.contains(&anchor_area_id))
            .map(|index| index + 1)
            .unwrap_or(grouped_area_ids.len()),
    };
    grouped_area_ids.insert(insert_index.min(grouped_area_ids.len()), vec![area_id]);
    assign_region_stack_groups(panels, dock_region, &grouped_area_ids);
}

pub(super) fn place_panel_in_stack_group(
    panels: &mut [StudioGuiWindowPanelLayoutState],
    area_id: StudioGuiWindowAreaId,
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
    placement: StudioGuiWindowDockPlacement,
) {
    let mut ordered_area_ids = panels
        .iter()
        .filter(|panel| {
            panel.area_id != area_id
                && panel.dock_region == dock_region
                && panel.stack_group == stack_group
        })
        .map(|panel| panel.area_id)
        .collect::<Vec<_>>();
    let insert_index = match placement {
        StudioGuiWindowDockPlacement::Start => 0,
        StudioGuiWindowDockPlacement::End => ordered_area_ids.len(),
        StudioGuiWindowDockPlacement::Before { anchor_area_id } => ordered_area_ids
            .iter()
            .position(|candidate| *candidate == anchor_area_id)
            .unwrap_or(ordered_area_ids.len()),
        StudioGuiWindowDockPlacement::After { anchor_area_id } => ordered_area_ids
            .iter()
            .position(|candidate| *candidate == anchor_area_id)
            .map(|index| index + 1)
            .unwrap_or(ordered_area_ids.len()),
    };
    ordered_area_ids.insert(insert_index.min(ordered_area_ids.len()), area_id);
    assign_stack_group_orders(panels, dock_region, stack_group, &ordered_area_ids);
}

pub(super) fn normalize_stack_group(
    panels: &mut [StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
) {
    let ordered_area_ids = panels
        .iter()
        .filter(|panel| panel.dock_region == dock_region && panel.stack_group == stack_group)
        .map(|panel| panel.area_id)
        .collect::<Vec<_>>();
    assign_stack_group_orders(panels, dock_region, stack_group, &ordered_area_ids);
}

pub(super) fn normalize_region_stack_groups(
    panels: &mut [StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
) {
    let grouped_area_ids = grouped_area_ids_for_region_excluding_area(panels, dock_region, None);
    assign_region_stack_groups(panels, dock_region, &grouped_area_ids);
}

pub(super) fn assign_region_stack_groups(
    panels: &mut [StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
    grouped_area_ids: &[Vec<StudioGuiWindowAreaId>],
) {
    for (group_index, group) in grouped_area_ids.iter().enumerate() {
        let stack_group = dock_group_order(group_index);
        for area_id in group {
            if let Some(panel) = panels
                .iter_mut()
                .find(|panel| panel.area_id == *area_id && panel.dock_region == dock_region)
            {
                panel.stack_group = stack_group;
            }
        }
        assign_stack_group_orders(panels, dock_region, stack_group, group);
    }
    sort_panels(panels);
}

pub(super) fn assign_stack_group_orders(
    panels: &mut [StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
    ordered_area_ids: &[StudioGuiWindowAreaId],
) {
    for (index, area_id) in ordered_area_ids.iter().enumerate() {
        if let Some(panel) = panels.iter_mut().find(|panel| {
            panel.area_id == *area_id
                && panel.dock_region == dock_region
                && panel.stack_group == stack_group
        }) {
            panel.order = dock_order(index);
        }
    }
    sort_panels(panels);
}

pub(super) fn grouped_area_ids_for_region_excluding_area(
    panels: &[StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
    excluded_area_id: Option<StudioGuiWindowAreaId>,
) -> Vec<Vec<StudioGuiWindowAreaId>> {
    let mut ordered_panels = panels
        .iter()
        .filter(|panel| panel.dock_region == dock_region && Some(panel.area_id) != excluded_area_id)
        .collect::<Vec<_>>();
    ordered_panels.sort_by_key(|panel| {
        (
            panel.stack_group,
            panel.order,
            area_sort_rank(panel.area_id),
        )
    });

    let mut groups = Vec::<Vec<StudioGuiWindowAreaId>>::new();
    let mut current_group = None;
    for panel in ordered_panels {
        if current_group != Some(panel.stack_group) {
            groups.push(Vec::new());
            current_group = Some(panel.stack_group);
        }
        if let Some(group) = groups.last_mut() {
            group.push(panel.area_id);
        }
    }
    groups
}

pub(super) fn grouped_area_ids_by_region(
    panels: &[StudioGuiWindowPanelLayoutState],
) -> Vec<(StudioGuiWindowDockRegion, Vec<Vec<StudioGuiWindowAreaId>>)> {
    [
        StudioGuiWindowDockRegion::LeftSidebar,
        StudioGuiWindowDockRegion::CenterStage,
        StudioGuiWindowDockRegion::RightSidebar,
    ]
    .into_iter()
    .map(|dock_region| {
        (
            dock_region,
            grouped_area_ids_for_region_excluding_area(panels, dock_region, None),
        )
    })
    .filter(|(_, groups)| !groups.is_empty())
    .collect()
}

pub(super) fn panel_stack_group(
    panels: &[StudioGuiWindowPanelLayoutState],
    area_id: StudioGuiWindowAreaId,
) -> Option<u8> {
    panels
        .iter()
        .find(|panel| panel.area_id == area_id)
        .map(|panel| panel.stack_group)
}

pub(super) fn panel_stack_location(
    panels: &[StudioGuiWindowPanelLayoutState],
    area_id: StudioGuiWindowAreaId,
) -> Option<(StudioGuiWindowDockRegion, u8)> {
    panels
        .iter()
        .find(|panel| panel.area_id == area_id)
        .map(|panel| (panel.dock_region, panel.stack_group))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StackCycleDirection {
    Next,
    Previous,
}

pub(super) fn adjacent_panel_in_stack(
    panels: &[StudioGuiWindowPanelLayoutState],
    area_id: StudioGuiWindowAreaId,
    direction: StackCycleDirection,
) -> Option<StudioGuiWindowAreaId> {
    let (dock_region, stack_group) = panel_stack_location(panels, area_id)?;
    let ordered_area_ids = panels
        .iter()
        .filter(|panel| panel.dock_region == dock_region && panel.stack_group == stack_group)
        .map(|panel| panel.area_id)
        .collect::<Vec<_>>();
    if ordered_area_ids.len() <= 1 {
        return Some(area_id);
    }
    let current_index = ordered_area_ids
        .iter()
        .position(|candidate| *candidate == area_id)?;
    let next_index = match direction {
        StackCycleDirection::Next => (current_index + 1) % ordered_area_ids.len(),
        StackCycleDirection::Previous => {
            (current_index + ordered_area_ids.len() - 1) % ordered_area_ids.len()
        }
    };
    ordered_area_ids.get(next_index).copied()
}

pub(super) fn next_available_stack_group(
    panels: &[StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
) -> u8 {
    let max_group = panels
        .iter()
        .filter(|panel| panel.dock_region == dock_region)
        .map(|panel| panel.stack_group)
        .max()
        .unwrap_or(0);
    max_group.saturating_add(10).max(10)
}

pub(super) fn anchor_area_id_from_placement(
    placement: StudioGuiWindowDockPlacement,
) -> Option<StudioGuiWindowAreaId> {
    match placement {
        StudioGuiWindowDockPlacement::Start | StudioGuiWindowDockPlacement::End => None,
        StudioGuiWindowDockPlacement::Before { anchor_area_id }
        | StudioGuiWindowDockPlacement::After { anchor_area_id } => Some(anchor_area_id),
    }
}

pub(super) fn dock_order(index: usize) -> u8 {
    ((index + 1) * 10).min(u8::MAX as usize) as u8
}

pub(super) fn dock_group_order(index: usize) -> u8 {
    ((index + 1) * 10).min(u8::MAX as usize) as u8
}

pub(super) fn reconcile_center_stage_after_panel_move(
    layout: &mut StudioGuiWindowLayoutState,
    moved_area_id: StudioGuiWindowAreaId,
    moved_region: StudioGuiWindowDockRegion,
) {
    if moved_region == StudioGuiWindowDockRegion::CenterStage {
        layout.center_area = moved_area_id;
        return;
    }

    if let Some(replacement_area_id) =
        first_panel_in_region(&layout.panels, StudioGuiWindowDockRegion::CenterStage)
    {
        if layout.center_area == moved_area_id {
            layout.center_area = replacement_area_id;
        }
        return;
    }

    let fallback_area_id = layout
        .panels
        .iter()
        .find(|panel| panel.visible)
        .map(|panel| panel.area_id)
        .or_else(|| layout.panels.first().map(|panel| panel.area_id))
        .unwrap_or(StudioGuiWindowAreaId::Canvas);
    let mut fallback_panel = layout
        .panel(fallback_area_id)
        .cloned()
        .unwrap_or_else(|| default_panel_state(fallback_area_id));
    fallback_panel.dock_region = StudioGuiWindowDockRegion::CenterStage;
    fallback_panel.visible = true;
    upsert_panel_state(&mut layout.panels, fallback_panel);
    layout.center_area = fallback_area_id;
}

pub(super) fn first_panel_in_region(
    panels: &[StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
) -> Option<StudioGuiWindowAreaId> {
    panels
        .iter()
        .find(|panel| panel.dock_region == dock_region)
        .map(|panel| panel.area_id)
}

pub(super) fn upsert_region_weight(
    regions: &mut Vec<StudioGuiWindowRegionWeight>,
    region: StudioGuiWindowRegionWeight,
) {
    if let Some(index) = regions
        .iter()
        .position(|candidate| candidate.dock_region == region.dock_region)
    {
        regions[index] = region;
    } else {
        regions.push(region);
    }
}

pub(super) fn sort_stack_groups(stack_groups: &mut [StudioGuiWindowStackGroupState]) {
    stack_groups.sort_by_key(|candidate| {
        (
            dock_region_rank(candidate.dock_region),
            candidate.stack_group,
            area_sort_rank(candidate.active_area_id),
        )
    });
}

pub(super) fn sort_panels(panels: &mut [StudioGuiWindowPanelLayoutState]) {
    panels.sort_by_key(|candidate| {
        (
            dock_region_rank(candidate.dock_region),
            candidate.stack_group,
            candidate.order,
            area_sort_rank(candidate.area_id),
        )
    });
}

pub(super) fn area_sort_rank(area_id: StudioGuiWindowAreaId) -> u8 {
    match area_id {
        StudioGuiWindowAreaId::Commands => 1,
        StudioGuiWindowAreaId::Canvas => 2,
        StudioGuiWindowAreaId::Runtime => 3,
    }
}

pub(super) fn dock_region_rank(dock_region: StudioGuiWindowDockRegion) -> u8 {
    match dock_region {
        StudioGuiWindowDockRegion::LeftSidebar => 1,
        StudioGuiWindowDockRegion::CenterStage => 2,
        StudioGuiWindowDockRegion::RightSidebar => 3,
    }
}
