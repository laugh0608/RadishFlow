use std::fs;

use rf_store::{read_studio_layout_file, write_studio_layout_file};

use super::test_support::*;
use super::*;

#[test]
fn gui_host_persists_window_layout_overrides_into_project_sidecar() {
    let (config, project_path, layout_path) = layout_persistence_config();
    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
    let opened = gui_host.open_window().expect("expected window open");

    let updated = gui_host
        .update_window_layout(
            Some(opened.registration.window_id),
            StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                area_id: StudioGuiWindowAreaId::Commands,
                collapsed: true,
            },
        )
        .expect("expected layout update");
    assert_eq!(
        updated
            .layout_state
            .panel(StudioGuiWindowAreaId::Commands)
            .map(|panel| panel.collapsed),
        Some(true)
    );

    let second_update = gui_host
        .update_window_layout(
            Some(opened.registration.window_id),
            StudioGuiWindowLayoutMutation::SetRegionWeight {
                dock_region: StudioGuiWindowDockRegion::RightSidebar,
                weight: 33,
            },
        )
        .expect("expected region weight update");
    assert_eq!(
        second_update
            .layout_state
            .region_weight(StudioGuiWindowDockRegion::RightSidebar)
            .map(|region| region.weight),
        Some(33)
    );

    let third_update = gui_host
        .update_window_layout(
            Some(opened.registration.window_id),
            StudioGuiWindowLayoutMutation::SetCenterArea {
                area_id: StudioGuiWindowAreaId::Runtime,
            },
        )
        .expect("expected center area update");
    assert_eq!(third_update.layout_state.center_area, StudioGuiWindowAreaId::Runtime);

    let fourth_update = gui_host
        .update_window_layout(
            Some(opened.registration.window_id),
            StudioGuiWindowLayoutMutation::SetPanelOrder {
                area_id: StudioGuiWindowAreaId::Runtime,
                order: 5,
            },
        )
        .expect("expected panel order update");
    assert_eq!(
        fourth_update
            .layout_state
            .panels
            .iter()
            .map(|panel| (panel.area_id, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (StudioGuiWindowAreaId::Commands, 10),
            (StudioGuiWindowAreaId::Canvas, 20),
            (StudioGuiWindowAreaId::Runtime, 5),
        ]
    );

    let fifth_update = gui_host
        .update_window_layout(
            Some(opened.registration.window_id),
            StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                area_id: StudioGuiWindowAreaId::Commands,
                dock_region: StudioGuiWindowDockRegion::RightSidebar,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            },
        )
        .expect("expected panel dock region update");
    assert_eq!(
        fifth_update
            .layout_state
            .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
            .into_iter()
            .map(|panel| (panel.area_id, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (StudioGuiWindowAreaId::Commands, 10),
            (StudioGuiWindowAreaId::Runtime, 10),
        ]
    );

    let sixth_update = gui_host
        .update_window_layout(
            Some(opened.registration.window_id),
            StudioGuiWindowLayoutMutation::StackPanelWith {
                area_id: StudioGuiWindowAreaId::Commands,
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            },
        )
        .expect("expected panel stack update");
    assert_eq!(
        sixth_update
            .layout_state
            .panels_in_stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
            .into_iter()
            .map(|panel| (panel.area_id, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (StudioGuiWindowAreaId::Commands, 10),
            (StudioGuiWindowAreaId::Runtime, 20),
        ]
    );

    let seventh_update = gui_host
        .update_window_layout(
            Some(opened.registration.window_id),
            StudioGuiWindowLayoutMutation::ActivateNextPanelInStack {
                area_id: StudioGuiWindowAreaId::Commands,
            },
        )
        .expect("expected stack cycle update");
    assert_eq!(
        seventh_update
            .layout_state
            .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
        Some(StudioGuiWindowAreaId::Runtime)
    );

    let eighth_update = gui_host
        .update_window_layout(
            Some(opened.registration.window_id),
            StudioGuiWindowLayoutMutation::MovePanelWithinStack {
                area_id: StudioGuiWindowAreaId::Runtime,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Commands,
                },
            },
        )
        .expect("expected stack reorder update");
    assert_eq!(
        eighth_update
            .layout_state
            .panels_in_stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
            .into_iter()
            .map(|panel| (panel.area_id, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (StudioGuiWindowAreaId::Runtime, 10),
            (StudioGuiWindowAreaId::Commands, 20),
        ]
    );

    let ninth_update = gui_host
        .update_window_layout(
            Some(opened.registration.window_id),
            StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
                area_id: StudioGuiWindowAreaId::Commands,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            },
        )
        .expect("expected panel unstack update");
    assert_eq!(
        ninth_update
            .layout_state
            .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
            .into_iter()
            .map(|panel| (panel.area_id, panel.stack_group, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (StudioGuiWindowAreaId::Commands, 10, 10),
            (StudioGuiWindowAreaId::Runtime, 20, 10),
        ]
    );

    let stored = read_studio_layout_file(&layout_path).expect("expected stored layout sidecar");
    assert_eq!(stored.entries.len(), 1);
    assert_eq!(stored.entries[0].layout_key, "studio.window.owner.slot-1");
    assert_eq!(stored.entries[0].center_area, "runtime");
    let mut stored_panels = stored.entries[0]
        .panels
        .iter()
        .map(|panel| {
            (
                panel.area_id.as_str(),
                panel.dock_region.as_str(),
                panel.stack_group,
                panel.order,
            )
        })
        .collect::<Vec<_>>();
    stored_panels.sort_unstable();
    assert_eq!(
        stored_panels,
        vec![
            ("canvas", "center-stage", 10, 20),
            ("commands", "right-sidebar", 10, 10),
            ("runtime", "right-sidebar", 20, 10),
        ]
    );
    assert_eq!(stored.entries[0].stack_groups.len(), 3);
    assert_eq!(
        stored.entries[0]
            .stack_groups
            .iter()
            .find(|group| group.dock_region == "right-sidebar" && group.stack_group == 10)
            .map(|group| group.active_area_id.as_str()),
        Some("commands")
    );
    assert_eq!(
        stored.entries[0]
            .stack_groups
            .iter()
            .find(|group| group.dock_region == "right-sidebar" && group.stack_group == 20)
            .map(|group| group.active_area_id.as_str()),
        Some("runtime")
    );

    drop(gui_host);

    let mut reloaded = StudioGuiHost::new(&config).expect("expected reloaded gui host");
    let reopened = reloaded.open_window().expect("expected reopened window");
    let window = reloaded.window_model_for_window(Some(reopened.registration.window_id));

    assert_eq!(
        window
            .layout_state
            .panel(StudioGuiWindowAreaId::Commands)
            .map(|panel| panel.collapsed),
        Some(true)
    );
    assert_eq!(
        window
            .layout_state
            .region_weight(StudioGuiWindowDockRegion::RightSidebar)
            .map(|region| region.weight),
        Some(33)
    );
    assert_eq!(window.layout_state.center_area, StudioGuiWindowAreaId::Runtime);
    assert_eq!(
        window
            .layout_state
            .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
            .into_iter()
            .map(|panel| (panel.area_id, panel.dock_region, panel.stack_group, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (
                StudioGuiWindowAreaId::Commands,
                StudioGuiWindowDockRegion::RightSidebar,
                10,
                10,
            ),
            (
                StudioGuiWindowAreaId::Runtime,
                StudioGuiWindowDockRegion::RightSidebar,
                20,
                10,
            ),
        ]
    );
    assert_eq!(
        window
            .layout_state
            .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
        Some(StudioGuiWindowAreaId::Commands)
    );
    assert_eq!(
        window
            .layout_state
            .active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 20),
        Some(StudioGuiWindowAreaId::Runtime)
    );

    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_host_loads_legacy_window_layout_key_for_current_owner_scope() {
    let (config, project_path, layout_path) = layout_persistence_config();
    write_studio_layout_file(
        &layout_path,
        &rf_store::StoredStudioLayoutFile::new(vec![rf_store::StoredStudioWindowLayoutEntry {
            layout_key: "studio.window.owner.1".to_string(),
            center_area: "canvas".to_string(),
            panels: vec![rf_store::StoredStudioLayoutPanelState {
                area_id: "commands".to_string(),
                dock_region: "left-sidebar".to_string(),
                stack_group: 10,
                order: 10,
                visible: true,
                collapsed: true,
            }],
            stack_groups: Vec::new(),
            region_weights: vec![rf_store::StoredStudioLayoutRegionWeight {
                dock_region: "right-sidebar".to_string(),
                weight: 35,
            }],
        }]),
    )
    .expect("expected legacy layout sidecar");

    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
    let opened = gui_host.open_window().expect("expected window open");
    let window = gui_host.window_model_for_window(Some(opened.registration.window_id));

    assert_eq!(window.layout_state.scope.layout_slot, Some(1));
    assert_eq!(window.layout_state.scope.layout_key, "studio.window.owner.slot-1");
    assert_eq!(
        window
            .layout_state
            .panel(StudioGuiWindowAreaId::Commands)
            .map(|panel| panel.collapsed),
        Some(true)
    );
    assert_eq!(
        window
            .layout_state
            .region_weight(StudioGuiWindowDockRegion::RightSidebar)
            .map(|region| region.weight),
        Some(35)
    );

    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}
