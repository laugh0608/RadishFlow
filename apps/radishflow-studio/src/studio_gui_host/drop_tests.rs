use std::fs;

use super::test_support::*;
use super::*;

#[test]
fn gui_host_queries_drop_target_through_explicit_command_surface() {
    let (config, project_path, layout_path) = layout_persistence_config();
    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
    let window_id = match gui_host
        .execute_command(StudioGuiHostCommand::OpenWindow)
        .expect("expected window open")
    {
        StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };

    let queried = gui_host
        .execute_command(StudioGuiHostCommand::QueryWindowDropTarget {
            window_id: Some(window_id),
            query: StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: StudioGuiWindowAreaId::Runtime,
                dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                placement: StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect("expected drop target query");

    match queried {
        StudioGuiHostCommandOutcome::WindowDropTargetQueried(result) => {
            assert_eq!(result.target_window_id, Some(window_id));
            assert_eq!(
                result.query,
                StudioGuiWindowDropTargetQuery::DockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                    placement: StudioGuiWindowDockPlacement::Start,
                }
            );
            assert_eq!(
                result.drop_target.as_ref().map(|target| target.dock_region),
                Some(StudioGuiWindowDockRegion::LeftSidebar)
            );
            assert_eq!(
                result.preview_layout_state.as_ref().map(|layout| {
                    layout
                        .panel(StudioGuiWindowAreaId::Runtime)
                        .map(|panel| (panel.dock_region, panel.stack_group, panel.order))
                }),
                Some(Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10)))
            );
            assert_eq!(
                result.preview_window.as_ref().and_then(|window| {
                    window
                        .layout_state
                        .panel(StudioGuiWindowAreaId::Runtime)
                        .map(|panel| (panel.dock_region, panel.stack_group, panel.order))
                }),
                Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
            );
            assert_eq!(
                result
                    .layout_state
                    .panel(StudioGuiWindowAreaId::Runtime)
                    .map(|panel| (panel.dock_region, panel.stack_group)),
                Some((StudioGuiWindowDockRegion::RightSidebar, 10))
            );
        }
        other => panic!("expected drop target query outcome, got {other:?}"),
    }
    assert_eq!(gui_host.window_model_for_window(Some(window_id)).drop_preview, None);

    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_host_sets_drop_preview_and_surfaces_it_through_window_model() {
    let (config, project_path, layout_path) = layout_persistence_config();
    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
    let window_id = match gui_host
        .execute_command(StudioGuiHostCommand::OpenWindow)
        .expect("expected window open")
    {
        StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };

    let previewed = gui_host
        .execute_command(StudioGuiHostCommand::SetWindowDropTargetPreview {
            window_id: Some(window_id),
            query: StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: StudioGuiWindowAreaId::Runtime,
                dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                placement: StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect("expected drop preview update");

    match previewed {
        StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated(result) => {
            assert_eq!(result.target_window_id, Some(window_id));
            assert!(result.drop_target.is_some());
            assert!(result.preview_layout_state.is_some());
        }
        other => panic!("expected drop preview update outcome, got {other:?}"),
    }

    let window = gui_host.window_model_for_window(Some(window_id));
    let preview = window
        .drop_preview
        .expect("expected drop preview in window model");
    assert_eq!(
        preview.drop_target.dock_region,
        StudioGuiWindowDockRegion::LeftSidebar
    );
    assert_eq!(
        preview
            .preview_layout_state
            .panel(StudioGuiWindowAreaId::Runtime)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
    );
    assert_eq!(
        preview
            .preview_layout
            .panel(StudioGuiWindowAreaId::Runtime)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
    );
    assert_eq!(preview.overlay.drag_area_id, StudioGuiWindowAreaId::Runtime);
    assert_eq!(
        preview.overlay.target_dock_region,
        StudioGuiWindowDockRegion::LeftSidebar
    );
    assert_eq!(preview.overlay.target_stack_group, 10);
    assert_eq!(preview.overlay.target_tab_index, 0);
    assert_eq!(
        preview.overlay.target_stack_area_ids,
        vec![StudioGuiWindowAreaId::Runtime]
    );
    assert_eq!(
        preview.overlay.target_stack_active_area_id,
        StudioGuiWindowAreaId::Runtime
    );
    assert_eq!(
        preview.changed_area_ids,
        vec![
            StudioGuiWindowAreaId::Commands,
            StudioGuiWindowAreaId::Runtime,
        ]
    );
    assert_eq!(
        window
            .layout_state
            .panel(StudioGuiWindowAreaId::Runtime)
            .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
        Some((StudioGuiWindowDockRegion::RightSidebar, 10, 30))
    );
    assert!(
        gui_host
            .snapshot()
            .window_model_for_window(Some(window_id))
            .drop_preview
            .is_some()
    );

    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_host_clears_drop_preview_through_explicit_command_surface() {
    let (config, project_path, layout_path) = layout_persistence_config();
    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
    let window_id = match gui_host
        .execute_command(StudioGuiHostCommand::OpenWindow)
        .expect("expected window open")
    {
        StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let _ = gui_host
        .execute_command(StudioGuiHostCommand::SetWindowDropTargetPreview {
            window_id: Some(window_id),
            query: StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: StudioGuiWindowAreaId::Runtime,
                dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                placement: StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect("expected drop preview update");

    let cleared = gui_host
        .execute_command(StudioGuiHostCommand::ClearWindowDropTargetPreview {
            window_id: Some(window_id),
        })
        .expect("expected drop preview clear");

    match cleared {
        StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared(result) => {
            assert_eq!(result.target_window_id, Some(window_id));
            assert!(result.had_preview);
        }
        other => panic!("expected drop preview clear outcome, got {other:?}"),
    }
    assert_eq!(gui_host.window_model_for_window(Some(window_id)).drop_preview, None);

    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_host_applies_drop_target_through_explicit_command_surface() {
    let (config, project_path, layout_path) = layout_persistence_config();
    let mut gui_host = StudioGuiHost::new(&config).expect("expected gui host");
    let window_id = match gui_host
        .execute_command(StudioGuiHostCommand::OpenWindow)
        .expect("expected window open")
    {
        StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };
    let _ = gui_host
        .execute_command(StudioGuiHostCommand::SetWindowDropTargetPreview {
            window_id: Some(window_id),
            query: StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: StudioGuiWindowAreaId::Runtime,
                dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                placement: StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect("expected drop preview update");

    let applied = gui_host
        .execute_command(StudioGuiHostCommand::ApplyWindowDropTarget {
            window_id: Some(window_id),
            query: StudioGuiWindowDropTargetQuery::DockRegion {
                area_id: StudioGuiWindowAreaId::Runtime,
                dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                placement: StudioGuiWindowDockPlacement::Start,
            },
        })
        .expect("expected drop target apply");

    match applied {
        StudioGuiHostCommandOutcome::WindowDropTargetApplied(result) => {
            assert_eq!(result.target_window_id, Some(window_id));
            assert_eq!(
                result.drop_target.dock_region,
                StudioGuiWindowDockRegion::LeftSidebar
            );
            assert_eq!(
                result
                    .layout_state
                    .panel(StudioGuiWindowAreaId::Runtime)
                    .map(|panel| (panel.dock_region, panel.stack_group, panel.order)),
                Some((StudioGuiWindowDockRegion::LeftSidebar, 10, 10))
            );
        }
        other => panic!("expected drop target apply outcome, got {other:?}"),
    }
    assert_eq!(gui_host.window_model_for_window(Some(window_id)).drop_preview, None);

    let _ = fs::remove_file(layout_path);
    let _ = fs::remove_file(project_path);
}

#[test]
fn gui_host_returns_no_preview_window_for_inapplicable_drop_query() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
    let window_id = match gui_host
        .execute_command(StudioGuiHostCommand::OpenWindow)
        .expect("expected window open")
    {
        StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };

    let queried = gui_host
        .execute_command(StudioGuiHostCommand::QueryWindowDropTarget {
            window_id: Some(window_id),
            query: StudioGuiWindowDropTargetQuery::Unstack {
                area_id: StudioGuiWindowAreaId::Commands,
                placement: StudioGuiWindowDockPlacement::End,
            },
        })
        .expect("expected drop target query");

    match queried {
        StudioGuiHostCommandOutcome::WindowDropTargetQueried(result) => {
            assert_eq!(result.drop_target, None);
            assert_eq!(result.preview_layout_state, None);
            assert_eq!(result.preview_window, None);
        }
        other => panic!("expected drop target query outcome, got {other:?}"),
    }
}

#[test]
fn gui_host_rejects_inapplicable_drop_target_apply() {
    let mut gui_host = StudioGuiHost::new(&lease_expiring_config()).expect("expected gui host");
    let window_id = match gui_host
        .execute_command(StudioGuiHostCommand::OpenWindow)
        .expect("expected window open")
    {
        StudioGuiHostCommandOutcome::WindowOpened(opened) => opened.registration.window_id,
        other => panic!("expected opened window outcome, got {other:?}"),
    };

    let error = gui_host
        .execute_command(StudioGuiHostCommand::ApplyWindowDropTarget {
            window_id: Some(window_id),
            query: StudioGuiWindowDropTargetQuery::Unstack {
                area_id: StudioGuiWindowAreaId::Commands,
                placement: StudioGuiWindowDockPlacement::End,
            },
        })
        .expect_err("expected invalid drop apply");

    assert_eq!(error.code().as_str(), "invalid_input");
}
