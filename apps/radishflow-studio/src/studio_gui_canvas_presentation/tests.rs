use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    StudioGuiDriver, StudioGuiEvent, StudioRuntimeConfig, StudioRuntimeEntitlementPreflight,
    StudioRuntimeEntitlementSeed,
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
        "radishflow-studio-canvas-presentation-{timestamp}.rfproj.json"
    ));
    let project = crate::test_support::build_flash_drum_local_rules_project_json();
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
fn canvas_presentation_reports_empty_canvas_state() {
    let presentation = crate::StudioGuiCanvasState::default().presentation();

    assert_eq!(presentation.view.run_status, None);
    assert_eq!(presentation.view.focused_suggestion_id, None);
    assert_eq!(presentation.view.pending_edit, None);
    assert_eq!(presentation.view.current_selection, None);
    assert_eq!(presentation.view.focus_callout, None);
    assert_eq!(
        presentation.view.object_list,
        crate::StudioGuiCanvasObjectListViewModel {
            unit_count: 0,
            stream_count: 0,
            attention_count: 0,
            filter_options: vec![
                crate::StudioGuiCanvasObjectListFilterOptionViewModel {
                    filter_id: "all",
                    label: "All",
                    detail: "Every canvas object surfaced by the current document",
                    count: 0,
                    enabled: false,
                },
                crate::StudioGuiCanvasObjectListFilterOptionViewModel {
                    filter_id: "attention",
                    label: "Attention",
                    detail: "Objects with warning or error badges",
                    count: 0,
                    enabled: false,
                },
                crate::StudioGuiCanvasObjectListFilterOptionViewModel {
                    filter_id: "units",
                    label: "Units",
                    detail: "Unit blocks",
                    count: 0,
                    enabled: false,
                },
                crate::StudioGuiCanvasObjectListFilterOptionViewModel {
                    filter_id: "streams",
                    label: "Streams",
                    detail: "Material stream lines",
                    count: 0,
                    enabled: false,
                },
            ],
            items: Vec::new(),
        }
    );
    assert_eq!(presentation.view.unit_count, 0);
    assert!(presentation.view.unit_blocks.is_empty());
    assert_eq!(presentation.view.stream_line_count, 0);
    assert!(presentation.view.stream_lines.is_empty());
    assert_eq!(presentation.view.suggestion_count, 0);
    assert!(presentation.view.suggestions.is_empty());
    assert_eq!(
        presentation.text.lines,
        vec![
            "run status: none".to_string(),
            "pending edit: none".to_string(),
            "focused suggestion: none".to_string(),
            "current selection: none".to_string(),
            "focus callout: none".to_string(),
            "viewport: mode=Planar layout=transient_grid units=0 streams=0 focus=none".to_string(),
            "unit count: 0".to_string(),
            "stream line count: 0".to_string(),
            "object list count: units=0 streams=0 attention=0 items=0".to_string(),
            "legend: Canvas legend items=1".to_string(),
            "suggestion count: 0".to_string(),
        ]
    );
    assert_eq!(
        presentation.view.viewport,
        crate::StudioGuiCanvasViewportViewModel {
            mode_label: "Planar",
            layout_label: "transient_grid",
            summary: "Planar transient_grid: 0 unit(s), 0 material line(s), no active focus target"
                .to_string(),
            unit_count: 0,
            stream_line_count: 0,
            focus: None,
        }
    );
    assert_eq!(
        presentation.view.legend,
        crate::StudioGuiCanvasLegendViewModel {
            title: "Canvas legend",
            items: vec![crate::StudioGuiCanvasLegendItemViewModel {
                kind_label: "Attention",
                label: "No warning/error badges".to_string(),
                detail: "info diagnostics stay out of the canvas badge layer".to_string(),
                swatch_label: "neutral",
            }],
        }
    );
}

#[test]
fn canvas_presentation_reports_pending_canvas_edit() {
    let state = crate::StudioGuiCanvasState {
        pending_edit: Some(rf_ui::CanvasEditIntent::PlaceUnit {
            unit_kind: "Flash Drum".to_string(),
        }),
        ..crate::StudioGuiCanvasState::default()
    };

    let presentation = state.presentation();

    assert_eq!(
        presentation.view.pending_edit,
        Some(crate::StudioGuiCanvasPendingEditViewModel {
            intent_label: "place_unit",
            summary: "place unit kind=Flash Drum".to_string(),
            cancel_enabled: true,
        })
    );
    assert_eq!(presentation.view.place_unit_palette.title, "Place unit");
    assert!(!presentation.view.place_unit_palette.enabled);
    assert_eq!(
        presentation.view.place_unit_palette.active_unit_kind,
        Some("Flash Drum".to_string())
    );
    assert_eq!(
        presentation
            .view
            .place_unit_palette
            .options
            .iter()
            .map(|option| (
                option.command_id.as_str(),
                option.unit_kind.as_str(),
                option.enabled,
                option.active
            ))
            .collect::<Vec<_>>(),
        vec![
            ("canvas.begin_place_unit.feed", "Feed", false, false),
            ("canvas.begin_place_unit.mixer", "Mixer", false, false),
            ("canvas.begin_place_unit.heater", "Heater", false, false),
            ("canvas.begin_place_unit.cooler", "Cooler", false, false),
            ("canvas.begin_place_unit.valve", "Valve", false, false),
            (
                "canvas.begin_place_unit.flash_drum",
                "Flash Drum",
                false,
                true
            ),
        ]
    );
    assert_eq!(
        presentation.text.lines,
        vec![
            "run status: none".to_string(),
            "pending edit: place_unit summary=place unit kind=Flash Drum cancel=yes".to_string(),
            "focused suggestion: none".to_string(),
            "current selection: none".to_string(),
            "focus callout: none".to_string(),
            "viewport: mode=Planar layout=transient_grid units=0 streams=0 focus=none".to_string(),
            "unit count: 0".to_string(),
            "stream line count: 0".to_string(),
            "object list count: units=0 streams=0 attention=0 items=0".to_string(),
            "legend: Canvas legend items=2".to_string(),
            "suggestion count: 0".to_string(),
        ]
    );
    assert!(presentation.view.legend.items.iter().any(|item| {
        item.kind_label == "Edit"
            && item.label == "Pending placement"
            && item.swatch_label == "pending_edit"
    }));
}

#[test]
fn canvas_command_result_presentation_reports_navigation_outcomes() {
    let target = crate::StudioGuiCanvasCommandTargetViewModel {
        kind_label: "Unit",
        target_id: "flash-1".to_string(),
        label: "Flash Drum".to_string(),
        viewport_anchor_label: Some("unit-slot-1".to_string()),
        command_id: "inspector.focus_unit:flash-1".to_string(),
    };

    let located =
        crate::StudioGuiCanvasCommandResultViewModel::located(target.clone(), "unit-slot-1");
    assert_eq!(located.level, rf_ui::RunPanelNoticeLevel::Info);
    assert_eq!(located.status_label, "located");
    assert_eq!(
        located.activity_line,
        "canvas object located: Unit flash-1 -> unit-slot-1"
    );

    let committed = rf_ui::CanvasEditCommitResult {
        intent: rf_ui::CanvasEditIntent::PlaceUnit {
            unit_kind: "Flash Drum".to_string(),
        },
        command: rf_ui::DocumentCommand::CreateUnit {
            unit_id: rf_types::UnitId::new("flash-1"),
            kind: "flash_drum".to_string(),
        },
        revision: 3,
        unit_id: rf_types::UnitId::new("flash-1"),
        position: rf_ui::CanvasPoint::new(12.0, 24.0),
    };
    let created = crate::StudioGuiCanvasCommandResultViewModel::created_unit(
        target.clone(),
        "unit-slot-1",
        &committed,
    );
    assert_eq!(created.level, rf_ui::RunPanelNoticeLevel::Info);
    assert_eq!(created.status_label, "created");
    assert_eq!(created.title, "Canvas unit created");
    assert!(created.detail.contains("revision 3"));
    assert_eq!(
        created.activity_line,
        "canvas unit created: Unit flash-1 -> unit-slot-1"
    );
    let surface = created.command_surface();
    assert_eq!(surface.status_label, "created");
    assert_eq!(surface.title, "Canvas unit created");
    assert_eq!(surface.target_command_id, "inspector.focus_unit:flash-1");
    assert_eq!(
        surface.menu_path_text,
        "Canvas > Last result > created > inspector.focus_unit:flash-1"
    );
    assert!(surface.matches_query("canvas created"));
    assert!(surface.matches_query("flash-1"));
    assert!(!surface.matches_query("stream-feed"));

    let pinned = crate::StudioGuiCanvasCommandResultViewModel::moved_unit(
        target.clone(),
        "unit-slot-1",
        None,
        rf_ui::CanvasPoint::new(52.0, 72.0),
    );
    assert_eq!(pinned.status_label, "moved");
    assert_eq!(pinned.title, "Canvas unit pinned and moved");
    assert!(pinned.detail.contains("had no sidecar position"));
    assert!(
        pinned
            .detail
            .contains("pinned from its transient grid slot")
    );
    assert!(pinned.detail.contains("sidecar (52.0, 72.0)"));
    let pinned_surface = pinned.command_surface();
    assert_eq!(pinned_surface.title, "Canvas unit pinned and moved");
    assert!(pinned_surface.matches_query("pinned sidecar"));

    let moved = crate::StudioGuiCanvasCommandResultViewModel::moved_unit(
        target.clone(),
        "unit-slot-1",
        Some(rf_ui::CanvasPoint::new(52.0, 72.0)),
        rf_ui::CanvasPoint::new(92.0, 72.0),
    );
    assert_eq!(moved.title, "Canvas unit moved");
    assert!(moved.detail.contains("moved from sidecar (52.0, 72.0)"));

    let unavailable_edit = crate::StudioGuiCanvasCommandResultViewModel::pending_edit_unavailable(
        rf_ui::CanvasPoint::new(4.0, 8.0),
    );
    assert_eq!(unavailable_edit.level, rf_ui::RunPanelNoticeLevel::Warning);
    assert_eq!(unavailable_edit.status_label, "pending_edit_unavailable");
    assert_eq!(
        unavailable_edit.activity_line,
        "Canvas pending edit unavailable: Edit pending_edit (Pending canvas edit)"
    );
    assert!(
        unavailable_edit
            .detail
            .contains("no pending edit was active")
    );

    let failed_edit = crate::StudioGuiCanvasCommandResultViewModel::pending_edit_failed(
        rf_ui::CanvasPoint::new(5.0, 9.0),
        "[invalid_input] unsupported unit kind",
    );
    assert_eq!(failed_edit.level, rf_ui::RunPanelNoticeLevel::Error);
    assert_eq!(failed_edit.status_label, "pending_edit_failed");
    assert_eq!(
        failed_edit.target.command_id,
        "canvas.commit_pending_edit_at"
    );
    assert!(failed_edit.detail.contains("unsupported unit kind"));

    let unavailable =
        crate::StudioGuiCanvasCommandResultViewModel::anchor_unavailable(target.clone());
    assert_eq!(unavailable.level, rf_ui::RunPanelNoticeLevel::Warning);
    assert_eq!(unavailable.status_label, "anchor_unavailable");
    assert!(unavailable.detail.contains("unit-slot-1"));

    let failed = crate::StudioGuiCanvasCommandResultViewModel::dispatch_failed(
        target.clone(),
        "[invalid_input] missing target",
    );
    assert_eq!(failed.level, rf_ui::RunPanelNoticeLevel::Error);
    assert_eq!(failed.status_label, "dispatch_failed");
    assert!(failed.detail.contains("inspector.focus_unit:flash-1"));

    let expired =
        crate::StudioGuiCanvasCommandResultViewModel::anchor_expired(target, "unit-slot-1");
    assert_eq!(expired.level, rf_ui::RunPanelNoticeLevel::Warning);
    assert_eq!(expired.status_label, "anchor_expired");
    assert_eq!(expired.anchor_label.as_deref(), Some("unit-slot-1"));
}

#[test]
fn canvas_object_navigation_contract_aligns_object_focus_and_result_target() {
    let state = crate::StudioGuiCanvasState {
        units: vec![crate::StudioGuiCanvasUnitState {
            unit_id: rf_types::UnitId::new("flash-1"),
            name: "Flash Drum".to_string(),
            kind: "flash_drum".to_string(),
            layout_position: None,
            ports: Vec::new(),
            port_count: 0,
            connected_port_count: 0,
            is_active_inspector_target: true,
        }],
        ..crate::StudioGuiCanvasState::default()
    };

    let presentation = state.presentation();
    let focus = presentation
        .view
        .viewport
        .focus
        .as_ref()
        .expect("expected viewport focus");
    let item = presentation
        .view
        .object_list
        .items
        .iter()
        .find(|item| item.is_active)
        .expect("expected active object list item");
    let target = item.command_target();
    let result = crate::StudioGuiCanvasCommandResultViewModel::located(
        target.clone(),
        focus.anchor_label.clone(),
    );

    assert_eq!(target.command_id, focus.command_id);
    assert_eq!(target.kind_label, focus.kind_label);
    assert_eq!(target.target_id, focus.target_id);
    assert_eq!(
        target.viewport_anchor_label.as_deref(),
        Some(focus.anchor_label.as_str())
    );
    assert_eq!(result.target, target);
    assert_eq!(result.anchor_label.as_deref(), Some("unit-slot-0"));
}

#[test]
fn canvas_presentation_maps_attention_diagnostics_to_canvas_objects() {
    let state = crate::StudioGuiCanvasState {
        units: vec![crate::StudioGuiCanvasUnitState {
            unit_id: rf_types::UnitId::new("flash-1"),
            name: "Flash Drum".to_string(),
            kind: "flash_drum".to_string(),
            layout_position: None,
            ports: vec![crate::StudioGuiCanvasUnitPortState {
                name: "inlet".to_string(),
                direction: rf_types::PortDirection::Inlet,
                kind: rf_types::PortKind::Material,
                stream_id: Some(rf_types::StreamId::new("stream-feed")),
            }],
            port_count: 1,
            connected_port_count: 1,
            is_active_inspector_target: false,
        }],
        streams: vec![crate::StudioGuiCanvasStreamState {
            stream_id: rf_types::StreamId::new("stream-feed"),
            name: "Feed".to_string(),
            source: None,
            sink: Some(crate::StudioGuiCanvasStreamEndpointState {
                unit_id: rf_types::UnitId::new("flash-1"),
                port_name: "inlet".to_string(),
            }),
            is_active_inspector_target: false,
        }],
        run_status: Some(rf_ui::RunStatus::Error),
        latest_snapshot_summary: Some("Unit execution failed".to_string()),
        diagnostics: vec![crate::StudioGuiCanvasDiagnosticState {
            severity: rf_ui::DiagnosticSeverity::Error,
            code: "solver.step.execution".to_string(),
            message: "unit failed".to_string(),
            related_unit_ids: vec![rf_types::UnitId::new("flash-1")],
            related_stream_ids: vec![rf_types::StreamId::new("stream-feed")],
            related_port_targets: vec![rf_types::DiagnosticPortTarget::new("flash-1", "inlet")],
        }],
        ..crate::StudioGuiCanvasState::default()
    };

    let presentation = state.presentation();

    assert_eq!(
        presentation
            .view
            .run_status
            .as_ref()
            .map(|status| (status.status_label, status.attention_count)),
        Some(("Error", 1))
    );
    assert!(presentation.view.legend.items.iter().any(|item| {
        item.kind_label == "Run"
            && item.label == "Error"
            && item.detail.contains("diagnostics=1 attention=1")
    }));
    assert!(presentation.view.legend.items.iter().any(|item| {
        item.kind_label == "Attention"
            && item.label == "2 object(s)"
            && item.swatch_label == "attention"
    }));
    assert!(presentation.view.legend.items.iter().any(|item| {
        item.kind_label == "Ports"
            && item.label == "1/1 bound"
            && item.detail.contains("green markers")
    }));
    assert!(presentation.view.legend.items.iter().any(|item| {
        item.kind_label == "Streams"
            && item.label == "1 material line(s)"
            && item.detail.contains("terminal lines")
    }));
    let unit = presentation
        .view
        .unit_blocks
        .iter()
        .find(|unit| unit.unit_id == "flash-1")
        .expect("expected unit block");
    assert_eq!(
        unit.status_badges,
        vec![crate::StudioGuiCanvasStatusBadgeViewModel {
            severity_label: "Error",
            short_label: "E1".to_string(),
            detail: "solver.step.execution: unit failed (ports flash-1:inlet)".to_string(),
        }]
    );
    assert_eq!(
        unit.attention_summary.as_deref(),
        Some("attention: 1 error(s); ports flash-1:inlet; codes solver.step.execution")
    );
    assert!(
        unit.hover_text
            .contains("attention: 1 error(s); ports flash-1:inlet")
    );
    let port = unit
        .ports
        .iter()
        .find(|port| port.name == "inlet")
        .expect("expected inlet port");
    assert!(
        port.hover_text
            .contains("attention: 1 error(s); ports flash-1:inlet")
    );
    let stream = presentation
        .view
        .stream_lines
        .iter()
        .find(|stream| stream.stream_id == "stream-feed")
        .expect("expected stream line");
    assert_eq!(stream.status_badges, unit.status_badges);
    assert_eq!(
        stream.attention_summary.as_deref(),
        Some("attention: 1 error(s); ports flash-1:inlet; codes solver.step.execution")
    );
    assert!(presentation.view.object_list.items.iter().any(|item| {
        item.target_id == "flash-1"
            && item.status_badges == unit.status_badges
            && item.attention_summary == unit.attention_summary
    }));
    assert_eq!(presentation.view.object_list.attention_count, 2);
    assert_eq!(
        presentation
            .view
            .object_list
            .filter_options
            .iter()
            .map(|option| (option.filter_id, option.count, option.enabled))
            .collect::<Vec<_>>(),
        vec![
            ("all", 2, true),
            ("attention", 2, true),
            ("units", 1, true),
            ("streams", 1, true),
        ]
    );
    assert!(presentation.text.lines.iter().any(|line| {
        line == "- unit flash-1 kind=flash_drum ports=1/1 badges=E1 command=inspector.focus_unit:flash-1"
    }));
}

#[test]
fn canvas_presentation_consumes_driver_dispatch_canvas_state() {
    let (config, project_path) = flash_drum_local_rules_config();
    let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

    let dispatch = driver
        .dispatch_event(StudioGuiEvent::OpenWindowRequested)
        .expect("expected open dispatch");
    let presentation = dispatch.canvas.presentation();

    assert_eq!(
        presentation.view.focused_suggestion_id.as_deref(),
        Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
    );
    assert_eq!(
        presentation.view.run_status,
        Some(crate::StudioGuiCanvasRunStatusViewModel {
            status_label: "Idle",
            pending_reason_label: Some("SnapshotMissing"),
            latest_snapshot_id: None,
            summary: None,
            diagnostic_count: 0,
            attention_count: 0,
        })
    );
    assert!(
        presentation.view.unit_blocks.iter().any(|unit| {
            unit.unit_id == "flash-1"
                && unit.kind == "flash_drum"
                && unit.command_id == "inspector.focus_unit:flash-1"
                && unit.port_count == 3
                && unit.ports.len() == 3
                && unit.ports.iter().any(|port| {
                    port.name == "liquid"
                        && port.direction_label == "outlet"
                        && !port.is_connected
                        && port.binding_label == "unbound"
                        && port.stream_command_id.is_none()
                        && port.hover_text.contains("bound stream: unbound")
                        && port.side_index == 0
                        && port.side_count == 2
                })
                && unit.ports.iter().any(|port| {
                    port.name == "inlet" && port.stream_id.is_none() && port.stream_label.is_none()
                })
                && unit.layout_slot > 0
        }),
        "expected canvas presentation to surface existing UnitNode blocks"
    );
    let feed_block = presentation
        .view
        .unit_blocks
        .iter()
        .find(|unit| unit.unit_id == "feed-1")
        .expect("expected feed unit block");
    let feed_outlet = feed_block
        .ports
        .iter()
        .find(|port| port.name == "outlet")
        .expect("expected feed outlet port");
    assert_eq!(feed_outlet.stream_id.as_deref(), Some("stream-feed"));
    assert_eq!(
        feed_outlet.stream_label.as_deref(),
        Some("Feed (stream-feed)")
    );
    assert_eq!(
        feed_outlet.stream_command_id.as_deref(),
        Some("inspector.focus_stream:stream-feed")
    );
    assert!(
        feed_outlet
            .hover_text
            .contains("bound stream: Feed (stream-feed)")
    );
    assert!(
        presentation.view.stream_lines.iter().any(|stream| {
            stream.stream_id == "stream-feed"
                && stream.command_id == "inspector.focus_stream:stream-feed"
                && stream.source.as_ref().is_some_and(|source| {
                    source.unit_id == "feed-1"
                        && source.port_name == "outlet"
                        && source.port_side_index == 0
                        && source.port_side_count == 1
                })
                && stream.sink.as_ref().is_some_and(|sink| {
                    sink.unit_id == "heater-1"
                        && sink.port_name == "inlet"
                        && sink.port_side_index == 0
                        && sink.port_side_count == 1
                })
        }),
        "expected canvas presentation to surface existing stream connection lines"
    );
    assert_eq!(presentation.view.suggestion_count, 3);
    assert_eq!(presentation.view.object_list.unit_count, 3);
    assert_eq!(presentation.view.object_list.stream_count, 2);
    assert_eq!(presentation.view.object_list.attention_count, 0);
    assert_eq!(presentation.view.object_list.items.len(), 5);
    assert_eq!(
        presentation
            .view
            .object_list
            .filter_options
            .iter()
            .map(|option| (option.filter_id, option.count, option.enabled))
            .collect::<Vec<_>>(),
        vec![
            ("all", 5, true),
            ("attention", 0, false),
            ("units", 3, true),
            ("streams", 2, true),
        ]
    );
    assert!(
        presentation.view.object_list.items.iter().any(|item| {
            item.kind_label == "Unit"
                && item.target_id == "flash-1"
                && item.command_id == "inspector.focus_unit:flash-1"
                && item.detail == "flash_drum | ports 0/3"
                && item.related_stream_ids.is_empty()
        }),
        "expected object list to expose unit navigation entries"
    );
    assert!(
        presentation.view.object_list.items.iter().any(|item| {
            item.kind_label == "Stream"
                && item.target_id == "stream-feed"
                && item.command_id == "inspector.focus_stream:stream-feed"
                && item.detail == "feed-1:outlet -> heater-1:inlet"
                && item.related_stream_ids == vec!["stream-feed".to_string()]
        }),
        "expected object list to expose stream navigation entries"
    );
    assert_eq!(presentation.view.suggestions[0].status_label, "focused");
    assert_eq!(presentation.view.suggestions[0].source_label, "local_rules");
    assert!(presentation.view.suggestions[0].is_focused);
    assert!(presentation.view.suggestions[0].tab_accept_enabled);
    assert!(presentation.view.suggestions[0].explicit_accept_enabled);
    assert_eq!(
        presentation.text.lines,
        vec![
            "run status: Idle pending=SnapshotMissing snapshot=none diagnostics=0 attention=0 summary=none".to_string(),
            "pending edit: none".to_string(),
            "focused suggestion: local.flash_drum.connect_inlet.flash-1.stream-heated"
                .to_string(),
            "current selection: none".to_string(),
            "focus callout: none".to_string(),
            "viewport: mode=Planar layout=transient_grid units=3 streams=2 focus=none"
                .to_string(),
            "unit count: 3".to_string(),
            "stream line count: 2".to_string(),
            "object list count: units=3 streams=2 attention=0 items=5".to_string(),
            "legend: Canvas legend items=4".to_string(),
            "suggestion count: 3".to_string(),
            "- unit feed-1 kind=feed ports=1/1 badges=none command=inspector.focus_unit:feed-1".to_string(),
            "- unit flash-1 kind=flash_drum ports=0/3 badges=none command=inspector.focus_unit:flash-1".to_string(),
            "- unit heater-1 kind=heater ports=2/2 badges=none command=inspector.focus_unit:heater-1".to_string(),
            "  port feed-1:outlet direction=outlet kind=material stream=stream-feed binding=Feed (stream-feed) slot=1/1".to_string(),
            "  port flash-1:inlet direction=inlet kind=material stream=unbound binding=unbound slot=1/1".to_string(),
            "  port flash-1:liquid direction=outlet kind=material stream=unbound binding=unbound slot=1/2".to_string(),
            "  port flash-1:vapor direction=outlet kind=material stream=unbound binding=unbound slot=2/2".to_string(),
            "  port heater-1:inlet direction=inlet kind=material stream=stream-feed binding=Feed (stream-feed) slot=1/1".to_string(),
            "  port heater-1:outlet direction=outlet kind=material stream=stream-heated binding=Heated Outlet (stream-heated) slot=1/1".to_string(),
            "- stream stream-feed feed-1:outlet -> heater-1:inlet badges=none command=inspector.focus_stream:stream-feed".to_string(),
            "- stream stream-heated heater-1:outlet -> terminal badges=none command=inspector.focus_stream:stream-heated".to_string(),
            "* local.flash_drum.connect_inlet.flash-1.stream-heated [focused] source=local_rules confidence=0.97 target=flash-1 tab_accept=yes explicit_accept=yes reason=Connect stream `stream-heated` to flash drum inlet `inlet`".to_string(),
            "- local.flash_drum.create_outlet.flash-1.liquid [proposed] source=local_rules confidence=0.93 target=flash-1 tab_accept=yes explicit_accept=yes reason=Create terminal stream `Flash Drum Liquid Outlet` for flash drum outlet `liquid`".to_string(),
            "- local.flash_drum.create_outlet.flash-1.vapor [proposed] source=local_rules confidence=0.92 target=flash-1 tab_accept=yes explicit_accept=yes reason=Create terminal stream `Flash Drum Vapor Outlet` for flash drum outlet `vapor`".to_string(),
        ]
    );

    let focused_unit = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_unit:flash-1".to_string(),
        })
        .expect("expected unit focus dispatch");
    let focused_unit_block = focused_unit
        .window
        .canvas
        .widget
        .view()
        .unit_blocks
        .iter()
        .find(|unit| unit.unit_id == "flash-1")
        .expect("expected focused flash unit block");
    assert!(focused_unit_block.is_active_inspector_target);
    assert_eq!(
        focused_unit.window.canvas.widget.view().current_selection,
        Some(crate::StudioGuiCanvasSelectionViewModel {
            kind_label: "Unit",
            target_id: "flash-1".to_string(),
            summary: "Flash Drum (flash_drum) ports 0/3".to_string(),
            command_id: "inspector.focus_unit:flash-1".to_string(),
            layout_source_label: Some("transient grid"),
            layout_detail: Some("no sidecar position; nudge will pin from unit-slot-1".to_string()),
        })
    );
    assert_eq!(
        focused_unit.window.canvas.widget.view().focus_callout,
        Some(crate::StudioGuiCanvasFocusCalloutViewModel {
            kind_label: "Unit",
            target_id: "flash-1".to_string(),
            title: "Flash Drum".to_string(),
            detail: "flash_drum | ports 0/3".to_string(),
            command_id: "inspector.focus_unit:flash-1".to_string(),
        })
    );
    assert_eq!(
        focused_unit.window.canvas.widget.view().viewport.focus,
        Some(crate::StudioGuiCanvasViewportFocusViewModel {
            kind_label: "Unit",
            target_id: "flash-1".to_string(),
            source_label: "active_inspector_target",
            anchor_label: "unit-slot-1".to_string(),
            detail: "flash_drum | ports 0/3".to_string(),
            command_id: "inspector.focus_unit:flash-1".to_string(),
        })
    );
    assert!(
        focused_unit
            .window
            .canvas
            .widget
            .view()
            .object_list
            .items
            .iter()
            .any(|item| item.kind_label == "Unit" && item.target_id == "flash-1" && item.is_active)
    );
    assert_eq!(
        focused_unit
            .window
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Unit", "flash-1"))
    );

    let focused_stream = driver
        .dispatch_event(StudioGuiEvent::UiCommandRequested {
            command_id: "inspector.focus_stream:stream-feed".to_string(),
        })
        .expect("expected stream focus dispatch");
    let focused_stream_line = focused_stream
        .window
        .canvas
        .widget
        .view()
        .stream_lines
        .iter()
        .find(|stream| stream.stream_id == "stream-feed")
        .expect("expected focused feed stream line");
    assert!(focused_stream_line.is_active_inspector_target);
    assert_eq!(
        focused_stream.window.canvas.widget.view().current_selection,
        Some(crate::StudioGuiCanvasSelectionViewModel {
            kind_label: "Stream",
            target_id: "stream-feed".to_string(),
            summary: "Feed feed-1:outlet -> heater-1:inlet".to_string(),
            command_id: "inspector.focus_stream:stream-feed".to_string(),
            layout_source_label: None,
            layout_detail: None,
        })
    );
    assert_eq!(
        focused_stream.window.canvas.widget.view().focus_callout,
        Some(crate::StudioGuiCanvasFocusCalloutViewModel {
            kind_label: "Stream",
            target_id: "stream-feed".to_string(),
            title: "Feed".to_string(),
            detail: "feed-1:outlet -> heater-1:inlet".to_string(),
            command_id: "inspector.focus_stream:stream-feed".to_string(),
        })
    );
    assert_eq!(
        focused_stream.window.canvas.widget.view().viewport.focus,
        Some(crate::StudioGuiCanvasViewportFocusViewModel {
            kind_label: "Stream",
            target_id: "stream-feed".to_string(),
            source_label: "active_inspector_target",
            anchor_label: "stream-feed:0".to_string(),
            detail: "feed-1:outlet -> heater-1:inlet".to_string(),
            command_id: "inspector.focus_stream:stream-feed".to_string(),
        })
    );
    assert!(
        focused_stream
            .window
            .canvas
            .widget
            .view()
            .object_list
            .items
            .iter()
            .any(|item| item.kind_label == "Stream"
                && item.target_id == "stream-feed"
                && item.is_active)
    );
    assert_eq!(
        focused_stream
            .window
            .runtime
            .active_inspector_target
            .as_ref()
            .map(|target| (target.kind_label, target.target_id.as_str())),
        Some(("Stream", "stream-feed"))
    );

    let _ = fs::remove_file(project_path);
}
