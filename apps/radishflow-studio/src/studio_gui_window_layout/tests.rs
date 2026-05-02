use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use super::*;
use crate::{
    StudioGuiDriver, StudioGuiEvent, StudioRuntimeConfig, StudioRuntimeEntitlementPreflight,
    StudioRuntimeEntitlementSeed, StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger,
};

fn lease_expiring_config() -> StudioRuntimeConfig {
    StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
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
        "radishflow-studio-window-layout-{timestamp}.rfproj.json"
    ));
    let project =
        include_str!("../../../../examples/flowsheets/feed-heater-flash.rfproj.json")
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

#[test]
fn studio_gui_window_layout_maps_panels_into_dock_regions() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let layout = dispatch.window.layout();

    assert_eq!(layout.titlebar.title, "RadishFlow Studio");
    assert!(layout.titlebar.close_enabled);
    assert_eq!(layout.titlebar.registered_window_count, 1);
    assert_eq!(layout.titlebar.foreground_window_id, Some(1));
    assert_eq!(layout.center_area, StudioGuiWindowAreaId::Canvas);
    assert_eq!(layout.default_focus_area, StudioGuiWindowAreaId::Canvas);
    assert_eq!(
        layout.state.scope.kind,
        StudioGuiWindowLayoutScopeKind::Window
    );
    assert_eq!(layout.state.scope.window_id, Some(1));
    assert_eq!(layout.state.scope.layout_slot, Some(1));
    assert_eq!(layout.state.scope.layout_key, "studio.window.owner.slot-1");
    assert_eq!(
        layout
            .state
            .region_weight(StudioGuiWindowDockRegion::CenterStage)
            .map(|region| region.weight),
        Some(52)
    );

    let commands = layout
        .panel(StudioGuiWindowAreaId::Commands)
        .expect("expected commands panel");
    assert_eq!(commands.dock_region, StudioGuiWindowDockRegion::LeftSidebar);
    assert_eq!(
        commands.display_mode,
        StudioGuiWindowPanelDisplayMode::Standalone
    );
    assert!(commands.active_in_stack);
    assert!(commands.visible);
    assert!(!commands.collapsed);
    assert_eq!(commands.badge.as_deref(), Some("21"));

    let canvas = layout
        .panel(StudioGuiWindowAreaId::Canvas)
        .expect("expected canvas panel");
    assert_eq!(canvas.dock_region, StudioGuiWindowDockRegion::CenterStage);
    assert!(canvas.active_in_stack);
    assert_eq!(canvas.badge.as_deref(), Some("3"));
    assert!(canvas.summary.contains("3 suggestions"));

    let runtime = layout
        .panel(StudioGuiWindowAreaId::Runtime)
        .expect("expected runtime panel");
    assert_eq!(runtime.dock_region, StudioGuiWindowDockRegion::RightSidebar);
    assert_eq!(
        runtime.display_mode,
        StudioGuiWindowPanelDisplayMode::Standalone
    );
    assert!(runtime.active_in_stack);
    assert!(runtime.summary.contains("status=Idle"));
    assert!(runtime.summary.contains("entitlement=attached"));
    assert_eq!(
        layout
            .stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
            .map(|group| group.active_area_id),
        Some(StudioGuiWindowAreaId::Runtime)
    );

    let _ = fs::remove_file(project_path);
}

#[test]
fn studio_gui_window_layout_uses_distinct_scope_keys_for_different_windows() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

    let first = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected first open dispatch");
    let first_layout_key = first.window.layout_state.scope.layout_key.clone();

    let second = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected second open dispatch");
    let second_layout_key = second.window.layout_state.scope.layout_key.clone();

    assert_eq!(first.window.layout_state.scope.layout_slot, Some(1));
    assert_eq!(second.window.layout_state.scope.layout_slot, Some(1));
    assert_eq!(first_layout_key, "studio.window.owner.slot-1");
    assert_eq!(second_layout_key, "studio.window.observer.slot-1");
    assert_ne!(first_layout_key, second_layout_key);
    assert_eq!(
        second.snapshot.layout_state.scope.layout_key,
        "studio.window.owner.slot-1"
    );
}

#[test]
fn studio_gui_window_layout_applies_center_area_and_panel_order_mutations() {
    let state = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::SetPanelVisibility {
            area_id: StudioGuiWindowAreaId::Runtime,
            visible: false,
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::SetCenterArea {
            area_id: StudioGuiWindowAreaId::Runtime,
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::SetPanelOrder {
            area_id: StudioGuiWindowAreaId::Runtime,
            order: 5,
        });

    assert_eq!(state.center_area, StudioGuiWindowAreaId::Runtime);
    assert_eq!(
        state
            .panel(StudioGuiWindowAreaId::Runtime)
            .map(|panel| (panel.visible, panel.order)),
        Some((true, 5))
    );
    assert_eq!(
        state
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
}

#[test]
fn studio_gui_window_layout_moves_panels_across_dock_regions() {
    let state = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::SetPanelDockRegion {
            area_id: StudioGuiWindowAreaId::Runtime,
            dock_region: StudioGuiWindowDockRegion::CenterStage,
            order: Some(5),
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::SetPanelDockRegion {
            area_id: StudioGuiWindowAreaId::Canvas,
            dock_region: StudioGuiWindowDockRegion::LeftSidebar,
            order: Some(25),
        });

    assert_eq!(state.center_area, StudioGuiWindowAreaId::Runtime);
    assert_eq!(
        state.panel(StudioGuiWindowAreaId::Runtime).map(|panel| (
            panel.dock_region,
            panel.order,
            panel.visible
        )),
        Some((StudioGuiWindowDockRegion::CenterStage, 10, true))
    );
    assert_eq!(
        state
            .panel(StudioGuiWindowAreaId::Canvas)
            .map(|panel| (panel.dock_region, panel.order)),
        Some((StudioGuiWindowDockRegion::LeftSidebar, 25))
    );
}

#[test]
fn studio_gui_window_layout_places_panels_within_region_by_anchor() {
    let state = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Runtime,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::After {
                anchor_area_id: StudioGuiWindowAreaId::Commands,
            },
        });

    assert_eq!(
        state
            .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
            .into_iter()
            .map(|panel| (panel.area_id, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (StudioGuiWindowAreaId::Commands, 10),
            (StudioGuiWindowAreaId::Runtime, 10),
        ]
    );
    assert_eq!(state.center_area, StudioGuiWindowAreaId::Canvas);
}

#[test]
fn studio_gui_window_layout_previews_region_drop_targets() {
    let target = StudioGuiWindowLayoutState::default()
        .drop_target_for_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Runtime,
            dock_region: StudioGuiWindowDockRegion::LeftSidebar,
            placement: StudioGuiWindowDockPlacement::Start,
        })
        .expect("expected region drop target");

    assert_eq!(target.kind, StudioGuiWindowDropTargetKind::DockRegionGroup);
    assert_eq!(target.dock_region, StudioGuiWindowDockRegion::LeftSidebar);
    assert_eq!(target.anchor_area_id, None);
    assert!(target.creates_new_stack);
    assert!(!target.merges_into_existing_stack);
    assert_eq!(target.target_group_index, 0);
    assert_eq!(target.target_tab_index, 0);
    assert_eq!(
        target.preview_area_ids,
        vec![StudioGuiWindowAreaId::Runtime]
    );
}

#[test]
fn studio_gui_window_layout_supports_gui_facing_drop_target_queries() {
    let state = StudioGuiWindowLayoutState::default().applying_mutation(
        &StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        },
    );
    let query = StudioGuiWindowDropTargetQuery::Stack {
        area_id: StudioGuiWindowAreaId::Commands,
        anchor_area_id: StudioGuiWindowAreaId::Runtime,
        placement: StudioGuiWindowDockPlacement::After {
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
        },
    };

    let target = state
        .drop_target_for_query(&query)
        .expect("expected stack drop target from query");

    assert_eq!(
        target,
        state
            .drop_target_for_mutation(&query.layout_mutation())
            .unwrap()
    );
    assert_eq!(target.kind, StudioGuiWindowDropTargetKind::StackTab);
    assert_eq!(target.anchor_area_id, Some(StudioGuiWindowAreaId::Runtime));
    assert_eq!(target.target_tab_index, 1);
    assert_eq!(
        state.preview_layout_state_for_query(&query),
        Some(state.applying_mutation(&query.layout_mutation()))
    );
}

#[test]
fn studio_gui_window_layout_stacks_panel_with_anchor_group() {
    let state = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::StackPanelWith {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        });

    assert_eq!(
        state
            .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
            .into_iter()
            .map(|panel| (panel.area_id, panel.stack_group, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (StudioGuiWindowAreaId::Commands, 10, 10),
            (StudioGuiWindowAreaId::Runtime, 10, 20),
        ]
    );
    assert_eq!(
        state
            .panels_in_stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
            .into_iter()
            .map(|panel| panel.area_id)
            .collect::<Vec<_>>(),
        vec![
            StudioGuiWindowAreaId::Commands,
            StudioGuiWindowAreaId::Runtime
        ]
    );
    assert_eq!(
        state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
        Some(StudioGuiWindowAreaId::Commands)
    );
}

#[test]
fn studio_gui_window_layout_previews_stack_drop_targets() {
    let state = StudioGuiWindowLayoutState::default().applying_mutation(
        &StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        },
    );
    let target = state
        .drop_target_for_mutation(&StudioGuiWindowLayoutMutation::StackPanelWith {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::After {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .expect("expected stack drop target");

    assert_eq!(target.kind, StudioGuiWindowDropTargetKind::StackTab);
    assert_eq!(target.dock_region, StudioGuiWindowDockRegion::RightSidebar);
    assert_eq!(target.anchor_area_id, Some(StudioGuiWindowAreaId::Runtime));
    assert!(!target.creates_new_stack);
    assert!(target.merges_into_existing_stack);
    assert_eq!(target.target_group_index, 0);
    assert_eq!(target.target_tab_index, 1);
    assert_eq!(
        target.preview_active_area_id,
        StudioGuiWindowAreaId::Commands
    );
    assert_eq!(
        target.preview_area_ids,
        vec![
            StudioGuiWindowAreaId::Runtime,
            StudioGuiWindowAreaId::Commands
        ]
    );
}

#[test]
fn studio_gui_window_layout_switches_active_panel_within_stack_group() {
    let state = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::StackPanelWith {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::SetActivePanelInStack {
            area_id: StudioGuiWindowAreaId::Runtime,
        });

    assert_eq!(
        state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
        Some(StudioGuiWindowAreaId::Runtime)
    );
}

#[test]
fn studio_gui_window_layout_cycles_active_panel_within_stack_group() {
    let state = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::StackPanelWith {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::ActivateNextPanelInStack {
            area_id: StudioGuiWindowAreaId::Commands,
        })
        .applying_mutation(
            &StudioGuiWindowLayoutMutation::ActivatePreviousPanelInStack {
                area_id: StudioGuiWindowAreaId::Runtime,
            },
        );

    assert_eq!(
        state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
        Some(StudioGuiWindowAreaId::Commands)
    );
}

#[test]
fn studio_gui_window_layout_unstacks_panel_into_separate_group() {
    let state = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::StackPanelWith {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
            area_id: StudioGuiWindowAreaId::Commands,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        });

    assert_eq!(
        state
            .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
            .into_iter()
            .map(|panel| (panel.area_id, panel.stack_group, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (StudioGuiWindowAreaId::Commands, 10, 10),
            (StudioGuiWindowAreaId::Runtime, 20, 10),
        ]
    );
    assert_eq!(
        state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
        Some(StudioGuiWindowAreaId::Commands)
    );
    assert_eq!(
        state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 20),
        Some(StudioGuiWindowAreaId::Runtime)
    );
}

#[test]
fn studio_gui_window_layout_previews_unstack_target_only_for_shared_stack() {
    let standalone = StudioGuiWindowLayoutState::default();
    assert!(
        standalone
            .drop_target_for_mutation(&StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
                area_id: StudioGuiWindowAreaId::Commands,
                placement: StudioGuiWindowDockPlacement::End,
            })
            .is_none()
    );

    let stacked = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::StackPanelWith {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        });
    let target = stacked
        .drop_target_for_mutation(&StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
            area_id: StudioGuiWindowAreaId::Commands,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .expect("expected unstack drop target");

    assert_eq!(target.kind, StudioGuiWindowDropTargetKind::DockRegionGroup);
    assert_eq!(target.dock_region, StudioGuiWindowDockRegion::RightSidebar);
    assert_eq!(target.target_group_index, 0);
    assert_eq!(target.target_stack_group, 10);
    assert_eq!(
        target.preview_area_ids,
        vec![StudioGuiWindowAreaId::Commands]
    );
    assert_eq!(
        target.preview_active_area_id,
        StudioGuiWindowAreaId::Commands
    );
}

#[test]
fn studio_gui_window_layout_reorders_panels_within_same_stack_group() {
    let state = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::StackPanelWith {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::MovePanelWithinStack {
            area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Commands,
            },
        });

    assert_eq!(
        state
            .panels_in_stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
            .into_iter()
            .map(|panel| (panel.area_id, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (StudioGuiWindowAreaId::Runtime, 10),
            (StudioGuiWindowAreaId::Commands, 20),
        ]
    );
    assert_eq!(
        state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
        Some(StudioGuiWindowAreaId::Commands)
    );
}

#[test]
fn studio_gui_window_layout_keeps_drop_target_prediction_stable_after_unstack() {
    let stacked = StudioGuiWindowLayoutState::default()
        .applying_mutation(&StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
            area_id: StudioGuiWindowAreaId::Commands,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .applying_mutation(&StudioGuiWindowLayoutMutation::StackPanelWith {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        });
    let unstacked =
        stacked.applying_mutation(&StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
            area_id: StudioGuiWindowAreaId::Commands,
            placement: StudioGuiWindowDockPlacement::Before {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        });
    let target = unstacked
        .drop_target_for_mutation(&StudioGuiWindowLayoutMutation::StackPanelWith {
            area_id: StudioGuiWindowAreaId::Commands,
            anchor_area_id: StudioGuiWindowAreaId::Runtime,
            placement: StudioGuiWindowDockPlacement::After {
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
            },
        })
        .expect("expected stack drop target after unstack");

    assert_eq!(target.kind, StudioGuiWindowDropTargetKind::StackTab);
    assert_eq!(target.target_group_index, 0);
    assert_eq!(target.target_tab_index, 1);
    assert_eq!(target.target_stack_group, 10);
    assert_eq!(
        target.preview_area_ids,
        vec![
            StudioGuiWindowAreaId::Runtime,
            StudioGuiWindowAreaId::Commands
        ]
    );
}

#[test]
fn studio_gui_window_layout_promotes_visible_panel_when_center_stage_becomes_empty() {
    let state = StudioGuiWindowLayoutState::default().applying_mutation(
        &StudioGuiWindowLayoutMutation::SetPanelDockRegion {
            area_id: StudioGuiWindowAreaId::Canvas,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            order: None,
        },
    );

    assert_eq!(state.center_area, StudioGuiWindowAreaId::Commands);
    assert_eq!(
        state
            .panel(StudioGuiWindowAreaId::Canvas)
            .map(|panel| panel.dock_region),
        Some(StudioGuiWindowDockRegion::RightSidebar)
    );
    assert_eq!(
        state
            .panel(StudioGuiWindowAreaId::Commands)
            .map(|panel| (panel.dock_region, panel.visible)),
        Some((StudioGuiWindowDockRegion::CenterStage, true))
    );
}

#[test]
fn studio_gui_window_layout_keeps_observer_slots_stable_when_peer_closes() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let first = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected first open dispatch");
    let first_window_id = match first.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected first window opened outcome, got {other:?}"),
    };
    let second = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected second open dispatch");
    let second_window_id = match second.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected second window opened outcome, got {other:?}"),
    };
    let third = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected third open dispatch");
    let third_window_id = match third.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected third window opened outcome, got {other:?}"),
    };

    assert_eq!(
        driver
            .window_model_for_window(Some(third_window_id))
            .layout_state
            .scope
            .layout_key,
        "studio.window.observer.slot-2"
    );

    let _ = driver
        .dispatch_event(StudioGuiEvent::CloseWindowRequested {
            window_id: second_window_id,
        })
        .expect("expected second window close");

    let remaining_observer = driver.window_model_for_window(Some(third_window_id));
    assert_eq!(remaining_observer.layout_state.scope.layout_slot, Some(2));
    assert_eq!(
        remaining_observer.layout_state.scope.layout_key,
        "studio.window.observer.slot-2"
    );

    let _ = driver
        .dispatch_event(StudioGuiEvent::CloseWindowRequested {
            window_id: first_window_id,
        })
        .expect("expected first window close");

    let promoted_owner = driver.window_model_for_window(Some(third_window_id));
    assert_eq!(promoted_owner.layout_state.scope.layout_slot, Some(1));
    assert_eq!(
        promoted_owner.layout_state.scope.layout_key,
        "studio.window.owner.slot-1"
    );
}

#[test]
fn studio_gui_window_layout_disables_close_when_all_windows_are_closed() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        crate::StudioGuiDriverOutcome::HostCommand(
            crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
        ) => opened.registration.window_id,
        other => panic!("expected window opened outcome, got {other:?}"),
    };
    let _ = driver
        .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
            window_id,
            trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                StudioRuntimeEntitlementSessionEvent::TimerElapsed,
            ),
        })
        .expect("expected timer dispatch");
    let _ = driver
        .dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id })
        .expect("expected close dispatch");

    let layout = driver.snapshot().window_model().layout();

    assert_eq!(layout.titlebar.registered_window_count, 0);
    assert!(!layout.titlebar.close_enabled);
    assert_eq!(layout.default_focus_area, StudioGuiWindowAreaId::Runtime);
    assert_eq!(
        layout.state.scope.kind,
        StudioGuiWindowLayoutScopeKind::EmptyWorkspace
    );
    assert_eq!(layout.state.scope.layout_key, "studio.window.empty");
    assert_eq!(
        layout
            .panel(StudioGuiWindowAreaId::Commands)
            .and_then(|panel| panel.badge.as_deref()),
        Some("10")
    );
}
