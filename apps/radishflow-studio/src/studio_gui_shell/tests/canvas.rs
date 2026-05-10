use super::*;

#[test]
fn canvas_viewport_navigation_records_inspector_focus_commands() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("inspector.focus_unit:flash-1");

    let unit_focus = app
        .canvas_viewport_navigation
        .active_anchor
        .as_ref()
        .expect("expected unit viewport focus");
    assert_eq!(unit_focus.anchor_label, "unit-slot-1");
    assert!(unit_focus.pending_scroll);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str()
        )),
        Some((
            RunPanelNoticeLevel::Info,
            "located",
            "Canvas object located"
        ))
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line == "canvas object located: Unit flash-1 -> unit-slot-1")
    );
    assert_eq!(app.last_area_focus, Some(StudioGuiWindowAreaId::Canvas));
    assert!(
        app.canvas_viewport_navigation
            .take_pending_scroll_for_anchor("unit-slot-1")
    );
    assert!(
        !app.canvas_viewport_navigation
            .take_pending_scroll_for_anchor("unit-slot-1")
    );

    app.dispatch_ui_command("inspector.focus_stream:stream-feed");

    let stream_focus = app
        .canvas_viewport_navigation
        .active_anchor
        .as_ref()
        .expect("expected stream viewport focus");
    assert_eq!(stream_focus.anchor_label, "stream-feed:0");
    assert!(stream_focus.pending_scroll);

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn canvas_pending_edit_commit_records_created_unit_focus_feedback() {
    let (config, project_path) = flash_drum_local_rules_config();
    let layout_path = studio_layout_path_for_project(&project_path);
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(144.0, 88.0));

    let window = app.platform_host.snapshot().window_model();
    let focus = window
        .canvas
        .widget
        .view()
        .viewport
        .focus
        .as_ref()
        .expect("expected created unit focus");
    assert_eq!(focus.kind_label, "Unit");
    assert_eq!(focus.target_id, "flash-2");
    assert_eq!(focus.command_id, "inspector.focus_unit:flash-2");
    assert_eq!(
        window
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| {
                (
                    target.kind_label,
                    target.target_id.as_str(),
                    target.command_id.as_str(),
                )
            }),
        Some(("Unit", "flash-2", "inspector.focus_unit:flash-2"))
    );

    let active_anchor = app
        .canvas_viewport_navigation
        .active_anchor
        .as_ref()
        .expect("expected created unit canvas anchor");
    assert_eq!(active_anchor.anchor_label, focus.anchor_label);
    assert!(active_anchor.pending_scroll);
    assert_eq!(app.last_area_focus, Some(StudioGuiWindowAreaId::Canvas));
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str(),
            result.target.command_id.as_str(),
            result.anchor_label.as_deref()
        )),
        Some((
            RunPanelNoticeLevel::Info,
            "created",
            "Canvas unit created",
            "inspector.focus_unit:flash-2",
            Some(focus.anchor_label.as_str())
        ))
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line
                == &format!(
                    "canvas unit created: Unit flash-2 -> {}",
                    focus.anchor_label
                ))
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    let surface = app
        .canvas_command_result_command_surface()
        .expect("expected canvas command result command surface");
    assert_eq!(surface.status_label, "created");
    assert_eq!(surface.target_command_id, "inspector.focus_unit:flash-2");
    assert!(surface.matches_query("canvas result created flash-2"));
    assert!(!surface.matches_query("stream-feed"));

    let _ = std::fs::remove_file(layout_path);
    let _ = std::fs::remove_file(project_path);
}

#[test]
fn canvas_placement_palette_commit_matrix_records_created_unit_feedback() {
    let cases = [
        ("canvas.begin_place_unit.feed", "feed", "feed-"),
        ("canvas.begin_place_unit.mixer", "mixer", "mixer-"),
        ("canvas.begin_place_unit.heater", "heater", "heater-"),
        ("canvas.begin_place_unit.cooler", "cooler", "cooler-"),
        ("canvas.begin_place_unit.valve", "valve", "valve-"),
        ("canvas.begin_place_unit.flash_drum", "flash_drum", "flash-"),
    ];

    for (command_id, expected_kind, expected_prefix) in cases {
        let (config, project_path) = blank_workspace_config();
        let mut app = ready_app_state(&config);

        app.dispatch_ui_command(command_id);
        app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));

        let window = app.platform_host.snapshot().window_model();
        let focus = window
            .canvas
            .widget
            .view()
            .viewport
            .focus
            .as_ref()
            .unwrap_or_else(|| panic!("{command_id} should focus the created unit"));
        assert_eq!(focus.kind_label, "Unit", "{command_id}");
        assert!(
            focus.target_id.starts_with(expected_prefix),
            "{command_id} should allocate unit id with prefix `{expected_prefix}`"
        );
        assert_eq!(
            focus.command_id,
            format!("inspector.focus_unit:{}", focus.target_id),
            "{command_id}"
        );

        let created_unit = window
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == focus.target_id)
            .unwrap_or_else(|| panic!("{command_id} should expose created unit block"));
        assert_eq!(created_unit.kind, expected_kind, "{command_id}");
        assert!(
            created_unit.port_count > 0,
            "{command_id} should expose canonical ports"
        );

        assert_eq!(
            window
                .runtime
                .active_inspector_target
                .as_ref()
                .map(|target| target.command_id.as_str()),
            Some(focus.command_id.as_str()),
            "{command_id}"
        );
        assert_eq!(
            app.canvas_viewport_navigation
                .active_anchor
                .as_ref()
                .map(|anchor| (anchor.anchor_label.as_str(), anchor.pending_scroll)),
            Some((focus.anchor_label.as_str(), true)),
            "{command_id}"
        );
        assert_eq!(
            app.canvas_command_result.as_ref().map(|result| (
                result.level,
                result.status_label,
                result.title.as_str(),
                result.target.target_id.as_str(),
                result.target.command_id.as_str(),
                result.anchor_label.as_deref(),
            )),
            Some((
                RunPanelNoticeLevel::Info,
                "created",
                "Canvas unit created",
                focus.target_id.as_str(),
                focus.command_id.as_str(),
                Some(focus.anchor_label.as_str()),
            )),
            "{command_id}"
        );
        assert!(
            app.canvas_command_result_command_surface()
                .expect("expected canvas command result command surface")
                .matches_query(&format!("created {}", focus.target_id)),
            "{command_id}"
        );
        assert!(
            app.platform_host
                .snapshot()
                .runtime
                .workspace_document
                .has_unsaved_changes,
            "{command_id}"
        );

        let _ = fs::remove_file(studio_layout_path_for_project(&project_path));
        let _ = fs::remove_file(project_path);
    }
}

#[test]
fn canvas_feed_to_flash_minimal_path_surfaces_local_connection_suggestions() {
    let mut app = ready_app_state(&lease_expiring_config());

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));

    let after_feed = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_feed.canvas.focused_suggestion_id.as_deref(),
        Some("local.feed.create_outlet.feed-2")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_feed_outlet = app.platform_host.snapshot().window_model();
    assert_eq!(after_feed_outlet.canvas.suggestion_count, 0);

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(220.0, 40.0));

    let after_flash = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_flash.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-2.stream-feed-2-outlet")
    );
    assert_eq!(after_flash.canvas.suggestion_count, 3);

    app.dispatch_ui_command("canvas.accept_focused");
    let after_inlet = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_inlet.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.create_outlet.flash-2.liquid")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_liquid = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_liquid.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.create_outlet.flash-2.vapor")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let completed = app.platform_host.snapshot().window_model();
    assert_eq!(completed.canvas.suggestion_count, 0);
    assert!(completed.runtime.workspace_document.has_unsaved_changes);
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-feed-2-outlet")
    );
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-flash-2-liquid")
    );
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-flash-2-vapor")
    );
}

#[test]
fn canvas_feed_to_flash_explicit_suggestion_selection_can_run() {
    let mut app = ready_app_state(&lease_expiring_config());

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-2");

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(220.0, 40.0));
    let after_flash = app.platform_host.snapshot().window_model();
    assert!(
        after_flash
            .canvas
            .widget
            .view()
            .suggestions
            .iter()
            .any(
                |suggestion| suggestion.id == "local.flash_drum.create_outlet.flash-2.vapor"
                    && suggestion.explicit_accept_enabled
            )
    );

    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-2.vapor");
    accept_canvas_suggestion_by_id(
        &mut app,
        "local.flash_drum.connect_inlet.flash-2.stream-feed-2-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-2.liquid");

    app.dispatch_ui_command("run_panel.run_manual");
    let completed = app.platform_host.snapshot().window_model();
    assert_eq!(
        completed.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(completed.runtime.control_state.pending_reason, None);
    assert_eq!(completed.canvas.suggestion_count, 0);
}

#[test]
fn blank_project_initializes_components_saves_reopens_and_runs_feed_flash_path() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);

    let opened_blank = app.platform_host.snapshot().window_model();
    assert_eq!(opened_blank.runtime.workspace_document.revision, 1);
    assert!(opened_blank.runtime.workspace_document.has_unsaved_changes);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .as_deref(),
        Some("local.feed.create_outlet.feed-1")
    );
    app.dispatch_ui_command("canvas.accept_focused");

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(220.0, 40.0));
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .as_deref(),
        Some("local.flash_drum.connect_inlet.flash-1.stream-feed-1-outlet")
    );
    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");

    app.dispatch_ui_command("run_panel.run_manual");
    let solved = app.platform_host.snapshot().window_model();
    assert_eq!(
        solved.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(solved.runtime.control_state.pending_reason, None);

    app.save_project();
    let saved = read_project_file(&project_path).expect("expected saved blank project");
    assert_eq!(saved.document.flowsheet.components.len(), 2);
    let feed_stream = saved
        .document
        .flowsheet
        .streams
        .get(&rf_types::StreamId::new("stream-feed-1-outlet"))
        .expect("expected feed source stream");
    assert_eq!(feed_stream.total_molar_flow_mol_s, 1.0);
    assert_eq!(feed_stream.overall_mole_fractions.len(), 2);
    assert_eq!(
        feed_stream
            .overall_mole_fractions
            .get(&rf_types::ComponentId::new("methane"))
            .copied(),
        Some(0.5)
    );
    assert_eq!(
        feed_stream
            .overall_mole_fractions
            .get(&rf_types::ComponentId::new("ethane"))
            .copied(),
        Some(0.5)
    );

    app.open_project(project_path.clone(), "project");
    let reopened = app.platform_host.snapshot().window_model();
    assert_eq!(
        reopened.runtime.workspace_document.revision,
        saved.document.revision
    );
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);

    app.dispatch_ui_command("run_panel.run_manual");
    let rerun = app.platform_host.snapshot().window_model();
    assert_eq!(
        rerun.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(rerun.runtime.control_state.pending_reason, None);

    let _ = fs::remove_file(project_path);
}

#[test]
fn canvas_unit_positions_persist_through_project_save_and_reopen() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    accept_canvas_suggestion_by_id(&mut app, "local.feed.create_outlet.feed-1");

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(220.0, 40.0));

    let before_save = app.platform_host.snapshot().window_model();
    let before_feed = before_save
        .canvas
        .widget
        .view()
        .unit_blocks
        .iter()
        .find(|unit| unit.unit_id == "feed-1")
        .expect("expected feed block before save");
    let before_flash = before_save
        .canvas
        .widget
        .view()
        .unit_blocks
        .iter()
        .find(|unit| unit.unit_id == "flash-1")
        .expect("expected flash block before save");
    assert_eq!(
        before_feed.layout_position,
        Some(rf_ui::CanvasPoint::new(64.0, 40.0))
    );
    assert_eq!(
        before_flash.layout_position,
        Some(rf_ui::CanvasPoint::new(220.0, 40.0))
    );
    assert_eq!(
        before_save.canvas.widget.view().viewport.layout_label,
        "persisted_positions"
    );

    let layout_path = studio_layout_path_for_project(&project_path);
    let stored_layout = read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert!(stored_layout.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "feed-1" && position.x == 64.0 && position.y == 40.0
    }));
    assert!(stored_layout.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "flash-1" && position.x == 220.0 && position.y == 40.0
    }));

    app.save_project();
    app.open_project(project_path.clone(), "project");

    let reopened = app.platform_host.snapshot().window_model();
    assert!(!reopened.runtime.workspace_document.has_unsaved_changes);
    assert_eq!(
        reopened
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(64.0, 40.0))
    );
    assert_eq!(
        reopened
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "flash-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(220.0, 40.0))
    );
    assert_eq!(
        reopened.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-1.stream-feed-1-outlet")
    );

    accept_canvas_suggestion_by_id(
        &mut app,
        "local.flash_drum.connect_inlet.flash-1.stream-feed-1-outlet",
    );
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.liquid");
    accept_canvas_suggestion_by_id(&mut app, "local.flash_drum.create_outlet.flash-1.vapor");
    app.dispatch_ui_command("run_panel.run_manual");
    let solved = app.platform_host.snapshot().window_model();
    assert_eq!(
        solved.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_unit_layout_nudge_commands_move_selected_unit_from_command_surface() {
    let (config, project_path) = blank_workspace_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    app.save_project();
    app.dispatch_ui_command("inspector.focus_unit:feed-1");

    let before = app.platform_host.snapshot().window_model();
    let move_right = before.commands.palette_items("move right");
    assert_eq!(move_right.len(), 1);
    assert_eq!(
        move_right[0].command_id,
        "canvas.move_selected_unit.right".to_string()
    );
    assert!(move_right[0].enabled);
    assert_eq!(
        find_command_list_command_id(
            &before.commands.command_list_sections,
            "canvas.move_selected_unit.right"
        ),
        Some("canvas.move_selected_unit.right")
    );

    app.dispatch_ui_command("canvas.move_selected_unit.right");
    let moved = app.platform_host.snapshot().window_model();
    assert_eq!(
        moved
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(104.0, 40.0))
    );
    assert!(
        !moved.runtime.workspace_document.has_unsaved_changes,
        "layout nudge should only update the Studio layout sidecar"
    );
    let result = app
        .canvas_command_result_command_surface()
        .expect("expected canvas move command result");
    assert_eq!(result.status_label, "moved");
    assert_eq!(result.title, "Canvas unit moved");
    assert!(result.detail.contains("moved from sidecar (64.0, 40.0)"));
    assert_eq!(result.target_command_id, "inspector.focus_unit:feed-1");

    let layout_path = studio_layout_path_for_project(&project_path);
    let stored_layout = read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert!(stored_layout.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "feed-1" && position.x == 104.0 && position.y == 40.0
    }));

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_unit_layout_nudge_pins_transient_grid_without_dirtying_project() {
    let (config, project_path) = flash_drum_local_rules_config();
    let layout_path = studio_layout_path_for_project(&project_path);
    let _ = fs::remove_file(&layout_path);
    let project_before = fs::read_to_string(&project_path).expect("expected project file");
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("inspector.focus_unit:feed-1");
    let before = app.platform_host.snapshot().window_model();
    let revision_before = before.runtime.workspace_document.revision;
    let saved_revision_before = before.runtime.workspace_document.last_saved_revision;
    assert!(
        !before.runtime.workspace_document.has_unsaved_changes,
        "fixture should start from a saved project"
    );
    assert_eq!(
        before
            .canvas
            .widget
            .view()
            .current_selection
            .as_ref()
            .and_then(|selection| selection.layout_source_label),
        Some("transient grid")
    );
    assert_eq!(
        before
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        None,
        "fixture should not have a sidecar position before the first nudge"
    );

    app.dispatch_ui_command("canvas.move_selected_unit.right");

    let moved = app.platform_host.snapshot().window_model();
    assert_eq!(moved.runtime.workspace_document.revision, revision_before);
    assert_eq!(
        moved.runtime.workspace_document.last_saved_revision,
        saved_revision_before
    );
    assert!(
        !moved.runtime.workspace_document.has_unsaved_changes,
        "layout nudge must not dirty the project document"
    );
    assert_eq!(
        fs::read_to_string(&project_path).expect("expected project file after nudge"),
        project_before,
        "layout nudge must not rewrite the project JSON"
    );
    assert_eq!(
        moved
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(58.0, 72.0))
    );
    assert_eq!(
        moved
            .canvas
            .widget
            .view()
            .current_selection
            .as_ref()
            .and_then(|selection| selection.layout_source_label),
        Some("sidecar position")
    );
    let result = app
        .canvas_command_result_command_surface()
        .expect("expected canvas move command result");
    assert_eq!(result.status_label, "moved");
    assert!(result.detail.contains("had no sidecar position"));
    assert!(
        result
            .detail
            .contains("pinned from its transient grid slot")
    );
    assert_eq!(result.target_command_id, "inspector.focus_unit:feed-1");

    let stored_layout = read_studio_layout_file(&layout_path).expect("expected layout sidecar");
    assert!(stored_layout.canvas_unit_positions.iter().any(|position| {
        position.unit_id == "feed-1" && position.x == 58.0 && position.y == 72.0
    }));

    let reopened = ready_app_state(&config)
        .platform_host
        .snapshot()
        .window_model();
    assert_eq!(
        reopened
            .canvas
            .widget
            .view()
            .unit_blocks
            .iter()
            .find(|unit| unit.unit_id == "feed-1")
            .and_then(|unit| unit.layout_position),
        Some(rf_ui::CanvasPoint::new(58.0, 72.0)),
        "reopened project should restore the pinned sidecar position"
    );

    let _ = fs::remove_file(project_path);
    let _ = fs::remove_file(layout_path);
}

#[test]
fn canvas_feed_heater_flash_minimal_path_can_run_after_accepting_suggestions() {
    let mut app = ready_app_state(&synced_workspace_config());

    app.dispatch_ui_command("canvas.begin_place_unit.feed");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(64.0, 40.0));
    assert_eq!(
        app.platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .as_deref(),
        Some("local.feed.create_outlet.feed-2")
    );
    app.dispatch_ui_command("canvas.accept_focused");

    app.dispatch_ui_command("canvas.begin_place_unit.heater");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(180.0, 40.0));
    let after_heater = app.platform_host.snapshot().window_model();
    assert_eq!(after_heater.canvas.suggestion_count, 2);
    assert_eq!(
        after_heater.canvas.focused_suggestion_id.as_deref(),
        Some("local.heater.connect_inlet.heater-2.stream-feed-2-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_heater_inlet = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_heater_inlet.canvas.focused_suggestion_id.as_deref(),
        Some("local.heater.create_outlet.heater-2")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_heater_outlet = app.platform_host.snapshot().window_model();
    assert_eq!(after_heater_outlet.canvas.suggestion_count, 0);

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(320.0, 40.0));
    let after_flash = app.platform_host.snapshot().window_model();
    assert_eq!(after_flash.canvas.suggestion_count, 3);
    assert_eq!(
        after_flash.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-2.stream-heater-2-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");

    app.dispatch_ui_command("run_panel.run_manual");
    let completed = app.platform_host.snapshot().window_model();
    assert_eq!(
        completed.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(completed.runtime.control_state.pending_reason, None);
    assert!(
        completed
            .runtime
            .control_state
            .latest_snapshot_id
            .as_deref()
            .is_some_and(|snapshot_id| snapshot_id.contains("rev-9-seq-1"))
    );
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-heater-2-outlet")
    );
}

#[test]
fn canvas_feed_mixer_flash_minimal_path_can_run_after_accepting_suggestions() {
    let mut app = ready_app_state(&synced_workspace_config());

    for point in [
        rf_ui::CanvasPoint::new(64.0, 40.0),
        rf_ui::CanvasPoint::new(64.0, 140.0),
    ] {
        app.dispatch_ui_command("canvas.begin_place_unit.feed");
        app.dispatch_canvas_pending_edit_commit(point);
        app.dispatch_ui_command("canvas.accept_focused");
    }

    app.dispatch_ui_command("canvas.begin_place_unit.mixer");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(210.0, 90.0));
    let after_mixer = app.platform_host.snapshot().window_model();
    assert_eq!(after_mixer.canvas.suggestion_count, 3);
    assert_eq!(
        after_mixer.canvas.focused_suggestion_id.as_deref(),
        Some("local.mixer.connect_inlet_a.mixer-1.stream-feed-2-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_mixer_inlet_a = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_mixer_inlet_a.canvas.focused_suggestion_id.as_deref(),
        Some("local.mixer.connect_inlet_b.mixer-1.stream-feed-3-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_mixer_inlet_b = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_mixer_inlet_b.canvas.focused_suggestion_id.as_deref(),
        Some("local.mixer.create_outlet.mixer-1")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    let after_mixer_outlet = app.platform_host.snapshot().window_model();
    assert_eq!(after_mixer_outlet.canvas.suggestion_count, 0);

    app.dispatch_ui_command("canvas.begin_place_unit.flash_drum");
    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(360.0, 90.0));
    let after_flash = app.platform_host.snapshot().window_model();
    assert_eq!(
        after_flash.canvas.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-2.stream-mixer-1-outlet")
    );

    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");
    app.dispatch_ui_command("canvas.accept_focused");

    app.dispatch_ui_command("run_panel.run_manual");
    let completed = app.platform_host.snapshot().window_model();
    assert_eq!(
        completed.runtime.control_state.run_status,
        rf_ui::RunStatus::Converged
    );
    assert_eq!(completed.runtime.control_state.pending_reason, None);
    assert!(
        completed
            .runtime
            .control_state
            .latest_snapshot_id
            .as_deref()
            .is_some_and(|snapshot_id| snapshot_id.contains("rev-12-seq-1"))
    );
    assert!(
        completed
            .canvas
            .widget
            .view()
            .stream_lines
            .iter()
            .any(|stream| stream.stream_id == "stream-mixer-1-outlet")
    );
}

#[test]
fn canvas_pending_edit_commit_reports_missing_pending_edit_through_command_result() {
    let mut app = ready_app_state(&lease_expiring_config());

    app.dispatch_canvas_pending_edit_commit(rf_ui::CanvasPoint::new(12.0, 24.0));

    assert_eq!(app.canvas_viewport_navigation.active_anchor, None);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str(),
            result.target.kind_label,
            result.target.target_id.as_str(),
            result.target.command_id.as_str(),
        )),
        Some((
            RunPanelNoticeLevel::Warning,
            "pending_edit_unavailable",
            "Canvas pending edit unavailable",
            "Edit",
            "pending_edit",
            "canvas.commit_pending_edit_at",
        ))
    );
    assert!(
        app.canvas_command_result
            .as_ref()
            .map(|result| result.detail.contains("no pending edit was active"))
            .unwrap_or(false)
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line
                == "Canvas pending edit unavailable: Edit pending_edit (Pending canvas edit)")
    );
    assert!(
        !app.platform_host
            .snapshot()
            .runtime
            .workspace_document
            .has_unsaved_changes
    );
    let surface = app
        .canvas_command_result_command_surface()
        .expect("expected canvas command result command surface");
    assert_eq!(surface.status_label, "pending_edit_unavailable");
    assert_eq!(surface.target_command_id, "canvas.commit_pending_edit_at");
    assert!(surface.matches_query("pending edit unavailable"));
}

#[test]
fn canvas_pending_edit_commit_reports_dispatch_error_through_command_result() {
    let mut app = ready_app_state(&lease_expiring_config());

    app.record_canvas_pending_edit_commit_error(
        rf_ui::CanvasPoint::new(12.0, 24.0),
        "[invalid_input] canvas place unit intent uses unsupported unit kind `Pump`",
    );

    assert_eq!(app.canvas_viewport_navigation.active_anchor, None);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str(),
            result.target.kind_label,
            result.target.target_id.as_str(),
            result.target.command_id.as_str(),
        )),
        Some((
            RunPanelNoticeLevel::Error,
            "pending_edit_failed",
            "Canvas pending edit failed",
            "Edit",
            "pending_edit",
            "canvas.commit_pending_edit_at",
        ))
    );
    assert!(
        app.canvas_command_result
            .as_ref()
            .map(|result| result.detail.contains("unsupported unit kind `Pump`"))
            .unwrap_or(false)
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line
                == "Canvas pending edit failed: Edit pending_edit (Pending canvas edit)")
    );
}

#[test]
fn canvas_viewport_navigation_reports_missing_inspector_target() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);

    app.dispatch_ui_command("inspector.focus_unit:missing-unit");

    assert_eq!(app.canvas_viewport_navigation.active_anchor, None);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str()
        )),
        Some((
            RunPanelNoticeLevel::Error,
            "dispatch_failed",
            "Canvas object navigation failed"
        ))
    );
    assert!(
        app.canvas_command_result
            .as_ref()
            .map(|result| result.detail.contains("missing-unit"))
            .unwrap_or(false)
    );
    assert!(
        app.platform_host
            .snapshot()
            .runtime
            .gui_activity_lines
            .iter()
            .any(|line| line.contains("Canvas object navigation failed: Unit missing-unit"))
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn canvas_viewport_navigation_reconciles_against_current_presentation_focus() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut app = ready_app_state(&config);
    app.dispatch_ui_command("inspector.focus_unit:flash-1");
    assert!(app.canvas_viewport_navigation.active_anchor.is_some());

    app.dispatch_ui_command("inspector.focus_stream:stream-feed");
    let window = app.platform_host.snapshot().window_model();
    app.reconcile_canvas_viewport_navigation(window.canvas.widget.view().viewport.focus.as_ref());

    assert_eq!(
        app.canvas_viewport_navigation
            .active_anchor
            .as_ref()
            .map(|focus| focus.anchor_label.as_str()),
        Some("stream-feed:0")
    );

    app.reconcile_canvas_viewport_navigation(None);

    assert_eq!(app.canvas_viewport_navigation.active_anchor, None);
    assert_eq!(
        app.canvas_command_result.as_ref().map(|result| (
            result.level,
            result.status_label,
            result.title.as_str()
        )),
        Some((
            RunPanelNoticeLevel::Warning,
            "anchor_expired",
            "Canvas navigation anchor expired"
        ))
    );

    let _ = std::fs::remove_file(project_path);
}

#[test]
fn right_sidebar_width_keeps_runtime_panel_readable() {
    let width = region_panel_width_from_values(
        StudioGuiWindowDockRegion::RightSidebar,
        1_280.0,
        100.0,
        24.0,
    );

    assert_eq!(width, 360.0);
}
