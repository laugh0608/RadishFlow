use std::fs;

use super::test_support::{flash_drum_local_rules_config, lease_expiring_config};
use super::*;

#[test]
fn gui_driver_updates_window_layout_and_preserves_per_window_overrides() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    let first = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected first open dispatch");
    let first_window_id = match first.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected first window opened outcome, got {other:?}"),
    };
    let second = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected second open dispatch");
    let second_window_id = match second.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected second window opened outcome, got {other:?}"),
    };

    let hidden_runtime = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::SetPanelVisibility {
                area_id: crate::StudioGuiWindowAreaId::Runtime,
                visible: false,
            },
        })
        .expect("expected layout visibility update");
    match hidden_runtime.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(result.target_window_id, Some(second_window_id));
            assert_eq!(
                result
                    .layout_state
                    .panel(crate::StudioGuiWindowAreaId::Runtime)
                    .map(|panel| panel.visible),
                Some(false)
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let collapsed_commands = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                area_id: crate::StudioGuiWindowAreaId::Commands,
                collapsed: true,
            },
        })
        .expect("expected layout collapsed update");
    match collapsed_commands.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(result.target_window_id, Some(second_window_id));
            assert_eq!(
                result
                    .layout_state
                    .panel(crate::StudioGuiWindowAreaId::Commands)
                    .map(|panel| panel.collapsed),
                Some(true)
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let weighted = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::SetRegionWeight {
                dock_region: crate::StudioGuiWindowDockRegion::RightSidebar,
                weight: 31,
            },
        })
        .expect("expected layout weight update");
    match weighted.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(result.target_window_id, Some(second_window_id));
            assert_eq!(
                result
                    .layout_state
                    .region_weight(crate::StudioGuiWindowDockRegion::RightSidebar)
                    .map(|region| region.weight),
                Some(31)
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let centered = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::SetCenterArea {
                area_id: crate::StudioGuiWindowAreaId::Runtime,
            },
        })
        .expect("expected layout center update");
    match centered.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(result.target_window_id, Some(second_window_id));
            assert_eq!(result.layout_state.center_area, crate::StudioGuiWindowAreaId::Runtime);
            assert_eq!(
                result
                    .layout_state
                    .panel(crate::StudioGuiWindowAreaId::Runtime)
                    .map(|panel| panel.visible),
                Some(true)
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let reordered = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::SetPanelOrder {
                area_id: crate::StudioGuiWindowAreaId::Runtime,
                order: 5,
            },
        })
        .expect("expected layout order update");
    match reordered.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(result.target_window_id, Some(second_window_id));
            assert_eq!(
                result
                    .layout_state
                    .panels
                    .iter()
                    .map(|panel| (panel.area_id, panel.order))
                    .collect::<Vec<_>>(),
                vec![
                    (crate::StudioGuiWindowAreaId::Commands, 10),
                    (crate::StudioGuiWindowAreaId::Canvas, 20),
                    (crate::StudioGuiWindowAreaId::Runtime, 5),
                ]
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let moved = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                area_id: crate::StudioGuiWindowAreaId::Commands,
                dock_region: crate::StudioGuiWindowDockRegion::RightSidebar,
                placement: crate::StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: crate::StudioGuiWindowAreaId::Runtime,
                },
            },
        })
        .expect("expected layout dock region update");
    match moved.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(result.target_window_id, Some(second_window_id));
            assert_eq!(
                result
                    .layout_state
                    .panels_in_dock_region(crate::StudioGuiWindowDockRegion::RightSidebar)
                    .into_iter()
                    .map(|panel| (panel.area_id, panel.order))
                    .collect::<Vec<_>>(),
                vec![
                    (crate::StudioGuiWindowAreaId::Commands, 10),
                    (crate::StudioGuiWindowAreaId::Runtime, 10),
                ]
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let stacked = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::StackPanelWith {
                area_id: crate::StudioGuiWindowAreaId::Commands,
                anchor_area_id: crate::StudioGuiWindowAreaId::Runtime,
                placement: crate::StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: crate::StudioGuiWindowAreaId::Runtime,
                },
            },
        })
        .expect("expected layout stack update");
    let stacked_window_layout = stacked.window.layout();
    match &stacked.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(result.target_window_id, Some(second_window_id));
            assert_eq!(
                result
                    .layout_state
                    .panels_in_stack_group(crate::StudioGuiWindowDockRegion::RightSidebar, 10)
                    .into_iter()
                    .map(|panel| (panel.area_id, panel.order))
                    .collect::<Vec<_>>(),
                vec![
                    (crate::StudioGuiWindowAreaId::Commands, 10),
                    (crate::StudioGuiWindowAreaId::Runtime, 20),
                ]
            );
            assert_eq!(
                result
                    .layout_state
                    .active_panel_in_stack(crate::StudioGuiWindowDockRegion::RightSidebar, 10),
                Some(crate::StudioGuiWindowAreaId::Commands)
            );
            assert_eq!(
                stacked_window_layout
                    .panel(crate::StudioGuiWindowAreaId::Commands)
                    .map(|panel| panel.display_mode),
                Some(crate::StudioGuiWindowPanelDisplayMode::ActiveTab)
            );
            assert_eq!(
                stacked_window_layout
                    .panel(crate::StudioGuiWindowAreaId::Runtime)
                    .map(|panel| panel.display_mode),
                Some(crate::StudioGuiWindowPanelDisplayMode::InactiveTab)
            );
            assert_eq!(
                stacked_window_layout
                    .stack_group(crate::StudioGuiWindowDockRegion::RightSidebar, 10)
                    .map(|group| group.tabbed),
                Some(true)
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let activated_next = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::ActivateNextPanelInStack {
                area_id: crate::StudioGuiWindowAreaId::Commands,
            },
        })
        .expect("expected activate-next update");
    match &activated_next.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(
                result
                    .layout_state
                    .active_panel_in_stack(crate::StudioGuiWindowDockRegion::RightSidebar, 10),
                Some(crate::StudioGuiWindowAreaId::Runtime)
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let reordered_stack = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::MovePanelWithinStack {
                area_id: crate::StudioGuiWindowAreaId::Runtime,
                placement: crate::StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: crate::StudioGuiWindowAreaId::Commands,
                },
            },
        })
        .expect("expected stack reorder update");
    match &reordered_stack.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(
                result
                    .layout_state
                    .panels_in_stack_group(crate::StudioGuiWindowDockRegion::RightSidebar, 10)
                    .into_iter()
                    .map(|panel| (panel.area_id, panel.order))
                    .collect::<Vec<_>>(),
                vec![
                    (crate::StudioGuiWindowAreaId::Runtime, 10),
                    (crate::StudioGuiWindowAreaId::Commands, 20),
                ]
            );
            assert_eq!(
                result
                    .layout_state
                    .active_panel_in_stack(crate::StudioGuiWindowDockRegion::RightSidebar, 10),
                Some(crate::StudioGuiWindowAreaId::Runtime)
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let unstacked = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(second_window_id),
            mutation: StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
                area_id: crate::StudioGuiWindowAreaId::Commands,
                placement: crate::StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: crate::StudioGuiWindowAreaId::Runtime,
                },
            },
        })
        .expect("expected unstack update");
    match &unstacked.outcome {
        StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
            assert_eq!(
                result
                    .layout_state
                    .panels_in_dock_region(crate::StudioGuiWindowDockRegion::RightSidebar)
                    .into_iter()
                    .map(|panel| (panel.area_id, panel.stack_group, panel.order))
                    .collect::<Vec<_>>(),
                vec![
                    (crate::StudioGuiWindowAreaId::Commands, 10, 10),
                    (crate::StudioGuiWindowAreaId::Runtime, 20, 10),
                ]
            );
        }
        other => panic!("expected window layout update outcome, got {other:?}"),
    }

    let first_window = driver.window_model_for_window(Some(first_window_id));
    let second_window = driver.window_model_for_window(Some(second_window_id));

    assert_eq!(
        first_window
            .layout_state
            .panel(crate::StudioGuiWindowAreaId::Runtime)
            .map(|panel| panel.visible),
        Some(true)
    );
    assert_eq!(
        second_window
            .layout_state
            .panel(crate::StudioGuiWindowAreaId::Runtime)
            .map(|panel| panel.visible),
        Some(true)
    );
    assert_eq!(
        second_window
            .layout_state
            .panel(crate::StudioGuiWindowAreaId::Commands)
            .map(|panel| {
                (
                    panel.collapsed,
                    panel.dock_region,
                    panel.stack_group,
                    panel.order,
                )
            }),
        Some((true, crate::StudioGuiWindowDockRegion::RightSidebar, 10, 10))
    );
    assert_eq!(
        first_window
            .layout_state
            .region_weight(crate::StudioGuiWindowDockRegion::RightSidebar)
            .map(|region| region.weight),
        Some(24)
    );
    assert_eq!(
        second_window
            .layout_state
            .region_weight(crate::StudioGuiWindowDockRegion::RightSidebar)
            .map(|region| region.weight),
        Some(31)
    );
    assert_eq!(
        first_window.layout_state.center_area,
        crate::StudioGuiWindowAreaId::Canvas
    );
    assert_eq!(
        second_window.layout_state.center_area,
        crate::StudioGuiWindowAreaId::Runtime
    );
    assert_eq!(
        second_window
            .layout_state
            .panels_in_dock_region(crate::StudioGuiWindowDockRegion::RightSidebar)
            .into_iter()
            .map(|panel| (panel.area_id, panel.dock_region, panel.stack_group, panel.order))
            .collect::<Vec<_>>(),
        vec![
            (
                crate::StudioGuiWindowAreaId::Commands,
                crate::StudioGuiWindowDockRegion::RightSidebar,
                10,
                10,
            ),
            (
                crate::StudioGuiWindowAreaId::Runtime,
                crate::StudioGuiWindowDockRegion::RightSidebar,
                20,
                10,
            ),
        ]
    );
    assert_eq!(
        second_window
            .layout_state
            .active_panel_in_stack(crate::StudioGuiWindowDockRegion::RightSidebar, 10),
        Some(crate::StudioGuiWindowAreaId::Commands)
    );
    assert_eq!(
        second_window
            .layout_state
            .active_panel_in_stack(crate::StudioGuiWindowDockRegion::RightSidebar, 20),
        Some(crate::StudioGuiWindowAreaId::Runtime)
    );

    let layout_path = rf_store::studio_layout_path_for_project(&project_path);
    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_driver_routes_drop_target_queries_without_mutating_layout_state() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected opened window outcome, got {other:?}"),
    };

    let queried = driver
        .dispatch_event(StudioGuiEvent::WindowDropTargetQueryRequested {
            window_id: Some(window_id),
            query: crate::StudioGuiWindowDropTargetQuery::Stack {
                area_id: crate::StudioGuiWindowAreaId::Commands,
                anchor_area_id: crate::StudioGuiWindowAreaId::Runtime,
                placement: crate::StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: crate::StudioGuiWindowAreaId::Runtime,
                },
            },
        })
        .expect("expected drop target query dispatch");

    match queried.outcome {
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetQueried(result),
        ) => {
            let target = result.drop_target.expect("expected stack preview target");
            assert_eq!(result.target_window_id, Some(window_id));
            assert_eq!(target.target_stack_group, 10);
            assert_eq!(target.target_tab_index, 0);
            assert_eq!(
                result.preview_layout_state.as_ref().and_then(|layout| {
                    layout
                        .panel(crate::StudioGuiWindowAreaId::Commands)
                        .map(|panel| (panel.dock_region, panel.stack_group, panel.order))
                }),
                Some((crate::StudioGuiWindowDockRegion::RightSidebar, 10, 10))
            );
            assert_eq!(
                result.preview_window.as_ref().and_then(|window| {
                    window
                        .layout_state
                        .panel(crate::StudioGuiWindowAreaId::Runtime)
                        .map(|panel| (panel.dock_region, panel.stack_group, panel.order))
                }),
                Some((crate::StudioGuiWindowDockRegion::RightSidebar, 10, 20))
            );
            assert_eq!(
                result
                    .layout_state
                    .panel(crate::StudioGuiWindowAreaId::Commands)
                    .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
                Some((crate::StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
            );
        }
        other => panic!("expected drop target query outcome, got {other:?}"),
    }

    assert_eq!(
        queried
            .window
            .layout_state
            .panel(crate::StudioGuiWindowAreaId::Commands)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((crate::StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
    );
    assert_eq!(queried.window.drop_preview, None);
    let window = driver.window_model_for_window(Some(window_id));
    assert_eq!(window.drop_preview, None);
    assert_eq!(
        window
            .layout_state
            .panel(crate::StudioGuiWindowAreaId::Commands)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((crate::StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
    );
    assert_eq!(
        window
            .layout_state
            .panel(crate::StudioGuiWindowAreaId::Runtime)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((crate::StudioGuiWindowDockRegion::RightSidebar, 10, 30))
    );

    let layout_path = rf_store::studio_layout_path_for_project(&project_path);
    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_driver_routes_drop_preview_updates_through_single_event_entry() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected opened window outcome, got {other:?}"),
    };

    let previewed = driver
        .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested {
            window_id: Some(window_id),
            query: crate::StudioGuiWindowDropTargetQuery::Stack {
                area_id: crate::StudioGuiWindowAreaId::Commands,
                anchor_area_id: crate::StudioGuiWindowAreaId::Runtime,
                placement: crate::StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: crate::StudioGuiWindowAreaId::Runtime,
                },
            },
        })
        .expect("expected drop preview dispatch");

    match previewed.outcome {
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(result),
        ) => {
            assert_eq!(result.target_window_id, Some(window_id));
            assert!(result.drop_target.is_some());
            assert!(result.preview_layout_state.is_some());
        }
        other => panic!("expected drop preview outcome, got {other:?}"),
    }

    let preview = previewed
        .window
        .drop_preview
        .as_ref()
        .expect("expected drop preview in dispatch window");
    assert_eq!(preview.overlay.drag_area_id, crate::StudioGuiWindowAreaId::Commands);
    assert_eq!(
        preview.overlay.target_dock_region,
        crate::StudioGuiWindowDockRegion::RightSidebar
    );
    assert_eq!(preview.overlay.target_stack_group, 10);
    assert_eq!(preview.overlay.target_tab_index, 0);
    assert_eq!(
        preview.overlay.target_stack_area_ids,
        vec![
            crate::StudioGuiWindowAreaId::Commands,
            crate::StudioGuiWindowAreaId::Runtime,
        ]
    );
    assert_eq!(
        preview.overlay.target_stack_active_area_id,
        crate::StudioGuiWindowAreaId::Commands
    );
    assert_eq!(preview.drop_target.target_stack_group, 10);
    assert_eq!(
        preview
            .preview_layout
            .panel(crate::StudioGuiWindowAreaId::Commands)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((crate::StudioGuiWindowDockRegion::RightSidebar, 10, 10))
    );
    assert_eq!(
        preview.changed_area_ids,
        vec![
            crate::StudioGuiWindowAreaId::Commands,
            crate::StudioGuiWindowAreaId::Runtime,
        ]
    );
    assert_eq!(
        preview
            .preview_layout_state
            .panel(crate::StudioGuiWindowAreaId::Commands)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((crate::StudioGuiWindowDockRegion::RightSidebar, 10, 10))
    );
    assert_eq!(
        previewed
            .window
            .layout_state
            .panel(crate::StudioGuiWindowAreaId::Commands)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((crate::StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
    );
    assert!(driver.window_model_for_window(Some(window_id)).drop_preview.is_some());

    let layout_path = rf_store::studio_layout_path_for_project(&project_path);
    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_driver_clears_drop_preview_through_single_event_entry() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let _ = driver
        .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested {
            window_id: Some(window_id),
            query: crate::StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: crate::StudioGuiWindowAreaId::Runtime,
                dock_region: crate::StudioGuiWindowDockRegion::LeftSidebar,
                placement: crate::StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect("expected drop preview dispatch");

    let cleared = driver
        .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewCleared {
            window_id: Some(window_id),
        })
        .expect("expected drop preview clear dispatch");

    match cleared.outcome {
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(result),
        ) => {
            assert_eq!(result.target_window_id, Some(window_id));
            assert!(result.had_preview);
        }
        other => panic!("expected drop preview clear outcome, got {other:?}"),
    }
    assert_eq!(cleared.window.drop_preview, None);
    assert_eq!(driver.window_model_for_window(Some(window_id)).drop_preview, None);

    let layout_path = rf_store::studio_layout_path_for_project(&project_path);
    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_driver_applies_drop_target_queries_through_single_event_entry() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let _ = driver
        .dispatch_event(StudioGuiEvent::WindowDropTargetPreviewRequested {
            window_id: Some(window_id),
            query: crate::StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: crate::StudioGuiWindowAreaId::Runtime,
                dock_region: crate::StudioGuiWindowDockRegion::LeftSidebar,
                placement: crate::StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect("expected drop preview dispatch");

    let applied = driver
        .dispatch_event(StudioGuiEvent::WindowDropTargetApplyRequested {
            window_id: Some(window_id),
            query: crate::StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: crate::StudioGuiWindowAreaId::Runtime,
                dock_region: crate::StudioGuiWindowDockRegion::LeftSidebar,
                placement: crate::StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect("expected drop target apply dispatch");

    match applied.outcome {
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDropTargetApplied(result),
        ) => {
            assert_eq!(result.target_window_id, Some(window_id));
            assert_eq!(
                result.mutation,
                crate::StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: crate::StudioGuiWindowAreaId::Runtime,
                    dock_region: crate::StudioGuiWindowDockRegion::LeftSidebar,
                    placement: crate::StudioGuiWindowDockPlacement::Start,
                }
                .layout_mutation()
            );
            assert_eq!(
                result
                    .layout_state
                    .panel(crate::StudioGuiWindowAreaId::Runtime)
                    .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
                Some((crate::StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
            );
        }
        other => panic!("expected drop target apply outcome, got {other:?}"),
    }

    assert_eq!(
        applied
            .window
            .layout_state
            .panel(crate::StudioGuiWindowAreaId::Runtime)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((crate::StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
    );
    assert_eq!(applied.window.drop_preview, None);

    let layout_path = rf_store::studio_layout_path_for_project(&project_path);
    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_driver_rejects_layout_update_for_unknown_window() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

    let error = driver
        .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
            window_id: Some(99),
            mutation: StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                area_id: crate::StudioGuiWindowAreaId::Commands,
                collapsed: true,
            },
        })
        .expect_err("expected invalid layout target");

    assert_eq!(error.code().as_str(), "invalid_input");
}

#[test]
fn gui_driver_rejects_drop_target_query_for_unknown_window() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

    let error = driver
        .dispatch_event(StudioGuiEvent::WindowDropTargetQueryRequested {
            window_id: Some(99),
            query: crate::StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: crate::StudioGuiWindowAreaId::Runtime,
                dock_region: crate::StudioGuiWindowDockRegion::LeftSidebar,
                placement: crate::StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect_err("expected invalid drop target query target");

    assert_eq!(error.code().as_str(), "invalid_input");
}

#[test]
fn gui_driver_rejects_inapplicable_drop_target_apply() {
    let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
    let opened = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let window_id = match opened.outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            opened.registration.window_id
        }
        other => panic!("expected opened window outcome, got {other:?}"),
    };

    let error = driver
        .dispatch_event(StudioGuiEvent::WindowDropTargetApplyRequested {
            window_id: Some(window_id),
            query: crate::StudioGuiWindowDropTargetQuery::Unstack {
                area_id: crate::StudioGuiWindowAreaId::Commands,
                placement: crate::StudioGuiWindowDockPlacement::End,
            },
        })
        .expect_err("expected invalid drop target apply");

    assert_eq!(error.code().as_str(), "invalid_input");
}
