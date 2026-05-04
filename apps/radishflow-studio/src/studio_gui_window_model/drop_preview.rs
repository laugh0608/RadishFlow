use super::*;

pub(super) fn build_drop_preview_overlay(
    preview_layout: &StudioGuiWindowLayoutModel,
    drop_target: &StudioGuiWindowDropTarget,
) -> StudioGuiWindowDropPreviewOverlayModel {
    let target_stack =
        preview_layout.stack_group(drop_target.dock_region, drop_target.target_stack_group);
    let target_stack_area_ids = target_stack
        .map(|group| group.tabs.iter().map(|tab| tab.area_id).collect::<Vec<_>>())
        .unwrap_or_else(|| drop_target.preview_area_ids.clone());
    let target_stack_active_area_id = target_stack
        .map(|group| group.active_area_id)
        .unwrap_or(drop_target.preview_active_area_id);

    StudioGuiWindowDropPreviewOverlayModel {
        drag_area_id: drop_target.area_id,
        kind: drop_target.kind,
        target_dock_region: drop_target.dock_region,
        target_stack_group: drop_target.target_stack_group,
        target_group_index: drop_target.target_group_index,
        target_tab_index: drop_target.target_tab_index,
        target_stack_area_ids: target_stack_area_ids.clone(),
        target_stack_active_area_id,
        highlighted_area_ids: target_stack_area_ids,
        anchor_area_id: drop_target.anchor_area_id,
        creates_new_stack: drop_target.creates_new_stack,
        merges_into_existing_stack: drop_target.merges_into_existing_stack,
    }
}

pub(super) fn changed_area_ids_for_preview(
    current_layout_state: &StudioGuiWindowLayoutState,
    preview_layout_state: &StudioGuiWindowLayoutState,
) -> Vec<StudioGuiWindowAreaId> {
    let mut changed = [
        StudioGuiWindowAreaId::Commands,
        StudioGuiWindowAreaId::Canvas,
        StudioGuiWindowAreaId::Runtime,
    ]
    .into_iter()
    .filter(|area_id| {
        current_layout_state.panel(*area_id) != preview_layout_state.panel(*area_id)
            || area_is_active_in_stack(current_layout_state, *area_id)
                != area_is_active_in_stack(preview_layout_state, *area_id)
    })
    .collect::<Vec<_>>();
    changed.sort_by_key(|area_id| match area_id {
        StudioGuiWindowAreaId::Commands => 0,
        StudioGuiWindowAreaId::Canvas => 1,
        StudioGuiWindowAreaId::Runtime => 2,
    });
    changed
}

fn area_is_active_in_stack(
    layout_state: &StudioGuiWindowLayoutState,
    area_id: StudioGuiWindowAreaId,
) -> bool {
    layout_state
        .panel(area_id)
        .map(|panel| {
            layout_state.active_panel_in_stack(panel.dock_region, panel.stack_group)
                == Some(area_id)
        })
        .unwrap_or(false)
}
