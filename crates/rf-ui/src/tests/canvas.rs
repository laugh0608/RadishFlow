use super::*;

#[test]
fn workspace_initializes_canvas_interaction_in_planar_mode() {
    let app_state = AppState::new(sample_document());

    assert_eq!(
        app_state.workspace.canvas_interaction.view_mode,
        CanvasViewMode::Planar
    );
    assert!(
        app_state
            .workspace
            .canvas_interaction
            .suggestions
            .is_empty()
    );
    assert_eq!(
        app_state.workspace.canvas_interaction.focused_suggestion_id,
        None
    );
    assert_eq!(app_state.workspace.canvas_interaction.pending_edit, None);
}

#[test]
fn canvas_place_unit_intent_is_transient_and_not_document_history() {
    let mut app_state = AppState::new(sample_document());

    let intent = app_state.begin_canvas_place_unit("Flash Drum");

    assert_eq!(
        intent,
        CanvasEditIntent::PlaceUnit {
            unit_kind: "Flash Drum".to_string()
        }
    );
    assert_eq!(
        app_state.workspace.canvas_interaction.pending_edit,
        Some(CanvasEditIntent::PlaceUnit {
            unit_kind: "Flash Drum".to_string()
        })
    );
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(!app_state.workspace.command_history.can_undo());
}

#[test]
fn cancelling_canvas_place_unit_intent_clears_pending_edit() {
    let mut app_state = AppState::new(sample_document());
    app_state.begin_canvas_place_unit("Flash Drum");

    let cancelled = app_state.cancel_canvas_pending_edit();

    assert_eq!(
        cancelled,
        Some(CanvasEditIntent::PlaceUnit {
            unit_kind: "Flash Drum".to_string()
        })
    );
    assert_eq!(app_state.workspace.canvas_interaction.pending_edit, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(!app_state.workspace.command_history.can_undo());
}

#[test]
fn document_change_invalidates_canvas_pending_edit() {
    let mut app_state = AppState::new(sample_document());
    app_state.begin_canvas_place_unit("Flash Drum");
    let mut next_flowsheet = app_state.workspace.document.flowsheet.clone();
    next_flowsheet
        .insert_unit(UnitNode::new(
            "flash-1",
            "Flash Drum",
            "flash_drum",
            Vec::new(),
        ))
        .expect("expected unit insert");

    app_state.commit_document_change(
        DocumentCommand::CreateUnit {
            unit_id: UnitId::new("flash-1"),
            kind: "flash_drum".to_string(),
        },
        next_flowsheet,
        timestamp(20),
    );

    assert_eq!(app_state.workspace.canvas_interaction.pending_edit, None);
}

#[test]
fn committing_canvas_place_unit_intent_creates_canonical_unit_command() {
    let mut app_state = AppState::new(sample_document());
    app_state.begin_canvas_place_unit("Flash Drum");

    let result = app_state
        .commit_canvas_pending_edit_at(CanvasPoint::new(160.0, 96.0), timestamp(30))
        .expect("expected canvas edit commit")
        .expect("expected pending canvas edit");

    assert_eq!(result.unit_id, UnitId::new("flash-1"));
    assert_eq!(result.position, CanvasPoint::new(160.0, 96.0));
    assert_eq!(
        result.command,
        DocumentCommand::CreateUnit {
            unit_id: UnitId::new("flash-1"),
            kind: "flash_drum".to_string(),
        }
    );
    assert_eq!(result.revision, 1);
    assert_eq!(app_state.workspace.canvas_interaction.pending_edit, None);
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(InspectorTarget::Unit(UnitId::new("flash-1")))
    );
    assert!(app_state.workspace.panels.inspector_open);
    assert!(app_state.workspace.command_history.can_undo());

    let unit = app_state
        .workspace
        .document
        .flowsheet
        .unit(&UnitId::new("flash-1"))
        .expect("expected committed flash unit");
    assert_eq!(unit.name, "Flash Drum");
    assert_eq!(unit.kind, "flash_drum");
    assert_eq!(unit.ports.len(), 3);
    assert!(unit.ports.iter().any(|port| {
        port.name == "inlet"
            && port.direction == PortDirection::Inlet
            && port.kind == PortKind::Material
            && port.stream_id.is_none()
    }));
    assert!(unit.ports.iter().any(|port| {
        port.name == "liquid"
            && port.direction == PortDirection::Outlet
            && port.kind == PortKind::Material
            && port.stream_id.is_none()
    }));
    assert_eq!(
        app_state
            .log_feed
            .entries
            .back()
            .map(|entry| entry.message.clone()),
        Some("Created canvas unit `flash-1` of kind `Flash Drum` at (160.0, 96.0)".to_string())
    );
}

#[test]
fn committing_canvas_place_unit_intent_covers_builtin_unit_matrix() {
    let cases = [
        (BuiltinUnitKind::Feed, "Feed", "feed-1"),
        (BuiltinUnitKind::Mixer, "Mixer", "mixer-1"),
        (BuiltinUnitKind::Heater, "Heater", "heater-1"),
        (BuiltinUnitKind::Cooler, "Cooler", "cooler-1"),
        (BuiltinUnitKind::Valve, "Valve", "valve-1"),
        (BuiltinUnitKind::FlashDrum, "Flash Drum", "flash-1"),
    ];

    for (builtin_kind, unit_kind, expected_unit_id) in cases {
        let mut app_state = AppState::new(sample_document());
        let expected_spec = builtin_unit_spec(builtin_kind);

        app_state.begin_canvas_place_unit(unit_kind);
        let result = app_state
            .commit_canvas_pending_edit_at(CanvasPoint::new(32.0, 48.0), timestamp(30))
            .expect("expected canvas edit commit")
            .expect("expected pending canvas edit");

        assert_eq!(result.unit_id, UnitId::new(expected_unit_id), "{unit_kind}");
        assert_eq!(result.position, CanvasPoint::new(32.0, 48.0));
        assert_eq!(
            result.command,
            DocumentCommand::CreateUnit {
                unit_id: UnitId::new(expected_unit_id),
                kind: expected_spec.kind.as_str().to_string(),
            },
            "{unit_kind}"
        );
        assert_eq!(result.revision, 1, "{unit_kind}");
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(InspectorTarget::Unit(UnitId::new(expected_unit_id))),
            "{unit_kind}"
        );
        assert!(app_state.workspace.panels.inspector_open, "{unit_kind}");
        assert!(
            app_state.workspace.command_history.can_undo(),
            "{unit_kind}"
        );

        let unit = app_state
            .workspace
            .document
            .flowsheet
            .unit(&UnitId::new(expected_unit_id))
            .unwrap_or_else(|_| panic!("expected committed {unit_kind} unit"));
        assert_eq!(unit.name, unit_kind, "{unit_kind}");
        assert_eq!(unit.kind, expected_spec.kind.as_str(), "{unit_kind}");
        assert_eq!(unit.ports.len(), expected_spec.ports.len(), "{unit_kind}");
        for expected_port in expected_spec.ports {
            assert!(
                unit.ports.iter().any(|port| {
                    port.name == expected_port.name
                        && port.direction == expected_port.direction
                        && port.kind == expected_port.kind
                        && port.stream_id.is_none()
                }),
                "{unit_kind} should include canonical material port `{}`",
                expected_port.name
            );
        }

        assert_eq!(
            app_state
                .log_feed
                .entries
                .back()
                .map(|entry| (entry.level, entry.message.clone())),
            Some((
                AppLogLevel::Info,
                format!(
                    "Created canvas unit `{expected_unit_id}` of kind `{unit_kind}` at (32.0, 48.0)"
                ),
            )),
            "{unit_kind}"
        );
    }
}

#[test]
fn committing_canvas_place_unit_intent_allocates_next_available_unit_id() {
    let mut app_state = AppState::new(sample_document());
    let mut flowsheet = app_state.workspace.document.flowsheet.clone();
    flowsheet
        .insert_unit(UnitNode::new(
            "flash-1",
            "Flash Drum",
            "flash_drum",
            Vec::new(),
        ))
        .expect("expected existing unit insert");
    app_state.commit_document_change(
        DocumentCommand::CreateUnit {
            unit_id: UnitId::new("flash-1"),
            kind: "flash_drum".to_string(),
        },
        flowsheet,
        timestamp(25),
    );
    app_state.begin_canvas_place_unit("flash_drum");

    let result = app_state
        .commit_canvas_pending_edit_at(CanvasPoint::new(10.0, 20.0), timestamp(30))
        .expect("expected canvas edit commit")
        .expect("expected pending canvas edit");

    assert_eq!(result.unit_id, UnitId::new("flash-2"));
    assert_eq!(
        app_state
            .workspace
            .document
            .flowsheet
            .unit(&UnitId::new("flash-2"))
            .expect("expected second flash unit")
            .name,
        "Flash Drum 2"
    );
}

#[test]
fn committing_canvas_edit_without_pending_intent_is_noop() {
    let mut app_state = AppState::new(sample_document());

    let result = app_state
        .commit_canvas_pending_edit_at(CanvasPoint::new(1.0, 2.0), timestamp(30))
        .expect("expected no-op commit");

    assert_eq!(result, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(!app_state.workspace.command_history.can_undo());
}

#[test]
fn replacing_canvas_suggestions_orders_by_confidence_and_focuses_first() {
    let mut app_state = AppState::new(sample_document());
    app_state.replace_canvas_suggestions(vec![
        sample_canvas_suggestion("sug-low", 0.40, SuggestionSource::LocalRules),
        sample_canvas_suggestion("sug-high", 0.95, SuggestionSource::RadishMind),
        sample_canvas_suggestion("sug-mid", 0.70, SuggestionSource::LocalRules),
    ]);

    let suggestions = &app_state.workspace.canvas_interaction.suggestions;
    assert_eq!(suggestions[0].id.as_str(), "sug-high");
    assert_eq!(suggestions[1].id.as_str(), "sug-mid");
    assert_eq!(suggestions[2].id.as_str(), "sug-low");
    assert_eq!(
        app_state
            .workspace
            .canvas_interaction
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str()),
        Some("sug-high")
    );
    assert_eq!(suggestions[0].status, SuggestionStatus::Focused);
    assert_eq!(suggestions[1].status, SuggestionStatus::Proposed);
    assert_eq!(suggestions[2].status, SuggestionStatus::Proposed);
}

#[test]
fn focus_next_canvas_suggestion_rotates_between_available_entries() {
    let mut app_state = AppState::new(sample_document());
    app_state.replace_canvas_suggestions(vec![
        sample_canvas_suggestion("sug-low", 0.40, SuggestionSource::LocalRules),
        sample_canvas_suggestion("sug-high", 0.95, SuggestionSource::RadishMind),
        sample_canvas_suggestion("sug-mid", 0.70, SuggestionSource::LocalRules),
    ]);

    let next = app_state
        .focus_next_canvas_suggestion()
        .expect("expected next focused suggestion");
    assert_eq!(next.id.as_str(), "sug-mid");
    assert_eq!(
        app_state
            .workspace
            .canvas_interaction
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str()),
        Some("sug-mid")
    );

    let wrapped = app_state
        .focus_next_canvas_suggestion()
        .expect("expected wrapped focus");
    assert_eq!(wrapped.id.as_str(), "sug-low");
    assert_eq!(
        app_state
            .workspace
            .canvas_interaction
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str()),
        Some("sug-low")
    );
}

#[test]
fn focus_previous_canvas_suggestion_wraps_to_last_available_entry() {
    let mut app_state = AppState::new(sample_document());
    app_state.replace_canvas_suggestions(vec![
        sample_canvas_suggestion("sug-low", 0.40, SuggestionSource::LocalRules),
        sample_canvas_suggestion("sug-high", 0.95, SuggestionSource::RadishMind),
        sample_canvas_suggestion("sug-mid", 0.70, SuggestionSource::LocalRules),
    ]);

    let previous = app_state
        .focus_previous_canvas_suggestion()
        .expect("expected previous focused suggestion");
    assert_eq!(previous.id.as_str(), "sug-low");
    assert_eq!(
        app_state
            .workspace
            .canvas_interaction
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str()),
        Some("sug-low")
    );
}

#[test]
fn rejecting_focused_canvas_suggestion_advances_focus_to_next_available_entry() {
    let mut app_state = AppState::new(sample_document());
    app_state.replace_canvas_suggestions(vec![
        sample_canvas_suggestion("sug-low", 0.40, SuggestionSource::LocalRules),
        sample_canvas_suggestion("sug-high", 0.95, SuggestionSource::RadishMind),
        sample_canvas_suggestion("sug-mid", 0.70, SuggestionSource::LocalRules),
    ]);

    let rejected = app_state
        .reject_focused_canvas_suggestion()
        .expect("expected rejected suggestion");
    assert_eq!(rejected.id.as_str(), "sug-high");
    assert_eq!(rejected.status, SuggestionStatus::Rejected);
    assert_eq!(
        app_state
            .workspace
            .canvas_interaction
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str()),
        Some("sug-mid")
    );
    assert_eq!(
        app_state.workspace.canvas_interaction.suggestions[0].status,
        SuggestionStatus::Rejected
    );
    assert_eq!(
        app_state.workspace.canvas_interaction.suggestions[1].status,
        SuggestionStatus::Focused
    );
}

#[test]
fn tab_accepts_only_high_confidence_suggestions_without_recording_history() {
    let mut flowsheet = Flowsheet::new("demo");
    flowsheet
        .insert_component(rf_model::Component::new("component-a", "Component A"))
        .expect("expected component-a");
    flowsheet
        .insert_component(rf_model::Component::new("component-b", "Component B"))
        .expect("expected component-b");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "feed-1",
            "Feed",
            "feed",
            vec![rf_model::UnitPort::new(
                "outlet",
                rf_types::PortDirection::Outlet,
                rf_types::PortKind::Material,
                Some("stream-feed".into()),
            )],
        ))
        .expect("expected feed insert");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "flash-1",
            "Flash Drum",
            "flash_drum",
            vec![
                rf_model::UnitPort::new(
                    "inlet",
                    rf_types::PortDirection::Inlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "liquid",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    Some("stream-liquid".into()),
                ),
                rf_model::UnitPort::new(
                    "vapor",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    Some("stream-vapor".into()),
                ),
            ],
        ))
        .expect("expected flash insert");
    for stream_id in ["stream-feed", "stream-liquid", "stream-vapor"] {
        flowsheet
            .insert_stream(MaterialStreamState::new(stream_id, stream_id))
            .expect("expected stream insert");
    }
    let mut app_state = AppState::new(FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("doc-accept", "Accept", timestamp(10)),
    ));
    app_state.replace_canvas_suggestions(vec![
        sample_canvas_suggestion("sug-high", 0.90, SuggestionSource::LocalRules)
            .with_acceptance(sample_existing_connection_acceptance()),
    ]);

    let accepted = app_state
        .accept_focused_canvas_suggestion_by_tab()
        .expect("expected suggestion acceptance");

    assert_eq!(
        accepted.as_ref().map(|item| item.id.as_str()),
        Some("sug-high")
    );
    assert_eq!(app_state.workspace.command_history.len(), 1);
    assert!(matches!(
        app_state.workspace.command_history.current_entry(),
        Some(crate::CommandHistoryEntry {
            command: crate::DocumentCommand::ConnectPorts {
                stream_id,
                from_unit_id,
                from_port,
                to_unit_id: Some(to_unit_id),
                to_port: Some(to_port),
            },
            ..
        }) if stream_id.as_str() == "stream-feed"
            && from_unit_id.as_str() == "feed-1"
            && from_port == "outlet"
            && to_unit_id.as_str() == "flash-1"
            && to_port == "inlet"
    ));
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state
            .workspace
            .document
            .flowsheet
            .units
            .get(&UnitId::new("flash-1"))
            .and_then(|unit| unit.ports.iter().find(|port| port.name == "inlet"))
            .and_then(|port| port.stream_id.as_ref())
            .map(|stream_id| stream_id.as_str()),
        Some("stream-feed")
    );
    assert_eq!(
        app_state.workspace.canvas_interaction.focused_suggestion_id,
        None
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("flash-1")))
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&UnitId::new("flash-1"))
    );
    assert!(app_state.workspace.panels.inspector_open);
    assert_eq!(
        app_state.log_feed.entries.back(),
        Some(&crate::AppLogEntry {
            level: AppLogLevel::Info,
            message: "Accepted canvas suggestion `sug-high` from local rules for unit flash-1"
                .to_string(),
        })
    );
    assert_eq!(
        app_state.workspace.run_panel.latest_log_message.as_deref(),
        Some("Accepted canvas suggestion `sug-high` from local rules for unit flash-1")
    );
}

#[test]
fn tab_accepts_connection_suggestion_while_other_ports_remain_unbound() {
    let mut flowsheet = Flowsheet::new("demo");
    flowsheet
        .insert_component(rf_model::Component::new("component-a", "Component A"))
        .expect("expected component-a");
    flowsheet
        .insert_component(rf_model::Component::new("component-b", "Component B"))
        .expect("expected component-b");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "feed-1",
            "Feed",
            "feed",
            vec![rf_model::UnitPort::new(
                "outlet",
                rf_types::PortDirection::Outlet,
                rf_types::PortKind::Material,
                Some("stream-feed".into()),
            )],
        ))
        .expect("expected feed insert");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "flash-1",
            "Flash Drum",
            "flash_drum",
            vec![
                rf_model::UnitPort::new(
                    "inlet",
                    rf_types::PortDirection::Inlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "liquid",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "vapor",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    None,
                ),
            ],
        ))
        .expect("expected flash insert");
    flowsheet
        .insert_stream(MaterialStreamState::new("stream-feed", "Feed"))
        .expect("expected feed stream");
    let mut app_state = AppState::new(FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("doc-incremental", "Incremental", timestamp(10)),
    ));
    app_state.replace_canvas_suggestions(vec![
        sample_canvas_suggestion("sug-connect-inlet", 0.97, SuggestionSource::LocalRules)
            .with_acceptance(sample_existing_connection_acceptance()),
    ]);

    let accepted = app_state
        .accept_focused_canvas_suggestion_by_tab()
        .expect("expected partial connection acceptance");

    assert_eq!(
        accepted.as_ref().map(|item| item.id.as_str()),
        Some("sug-connect-inlet")
    );
    assert!(matches!(
        app_state.workspace.command_history.current_entry(),
        Some(crate::CommandHistoryEntry {
            command: crate::DocumentCommand::ConnectPorts {
                stream_id,
                from_unit_id,
                from_port,
                to_unit_id: Some(to_unit_id),
                to_port: Some(to_port),
            },
            ..
        }) if stream_id.as_str() == "stream-feed"
            && from_unit_id.as_str() == "feed-1"
            && from_port == "outlet"
            && to_unit_id.as_str() == "flash-1"
            && to_port == "inlet"
    ));
    let flash = app_state
        .workspace
        .document
        .flowsheet
        .units
        .get(&UnitId::new("flash-1"))
        .expect("expected flash unit");
    assert_eq!(
        flash
            .ports
            .iter()
            .find(|port| port.name == "inlet")
            .and_then(|port| port.stream_id.as_ref())
            .map(|stream_id| stream_id.as_str()),
        Some("stream-feed")
    );
    assert!(
        flash
            .ports
            .iter()
            .filter(|port| port.name == "liquid" || port.name == "vapor")
            .all(|port| port.stream_id.is_none()),
        "incremental canvas connection should not require all other ports to be complete"
    );
}

#[test]
fn explicit_accept_can_apply_non_focused_suggestion_by_id() {
    let mut app_state = AppState::new(sample_feed_flash_document());
    app_state.replace_canvas_suggestions(vec![
        sample_canvas_suggestion("sug-focused", 0.97, SuggestionSource::LocalRules)
            .with_acceptance(sample_existing_connection_acceptance()),
        sample_canvas_suggestion("sug-explicit", 0.60, SuggestionSource::LocalRules)
            .with_acceptance(sample_existing_connection_acceptance()),
    ]);

    assert_eq!(
        app_state
            .workspace
            .canvas_interaction
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str()),
        Some("sug-focused")
    );

    let accepted = app_state
        .accept_canvas_suggestion(&CanvasSuggestionId::new("sug-explicit"))
        .expect("expected explicit suggestion acceptance");

    assert_eq!(
        accepted.as_ref().map(|item| item.id.as_str()),
        Some("sug-explicit")
    );
    assert!(matches!(
        app_state.workspace.command_history.current_entry(),
        Some(crate::CommandHistoryEntry {
            command: crate::DocumentCommand::ConnectPorts {
                stream_id,
                from_unit_id,
                from_port,
                to_unit_id: Some(to_unit_id),
                to_port: Some(to_port),
            },
            ..
        }) if stream_id.as_str() == "stream-feed"
            && from_unit_id.as_str() == "feed-1"
            && from_port == "outlet"
            && to_unit_id.as_str() == "flash-1"
            && to_port == "inlet"
    ));
}

#[test]
fn tab_accepts_suggestion_that_creates_terminal_outlet_stream() {
    let mut flowsheet = Flowsheet::new("demo");
    flowsheet
        .insert_component(rf_model::Component::new("component-a", "Component A"))
        .expect("expected component-a");
    flowsheet
        .insert_component(rf_model::Component::new("component-b", "Component B"))
        .expect("expected component-b");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "feed-1",
            "Feed",
            "feed",
            vec![rf_model::UnitPort::new(
                "outlet",
                rf_types::PortDirection::Outlet,
                rf_types::PortKind::Material,
                Some("stream-feed".into()),
            )],
        ))
        .expect("expected feed insert");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "flash-1",
            "Flash Drum",
            "flash_drum",
            vec![
                rf_model::UnitPort::new(
                    "inlet",
                    rf_types::PortDirection::Inlet,
                    rf_types::PortKind::Material,
                    Some("stream-feed".into()),
                ),
                rf_model::UnitPort::new(
                    "liquid",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "vapor",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    Some("stream-vapor".into()),
                ),
            ],
        ))
        .expect("expected flash insert");
    for stream_id in ["stream-feed", "stream-vapor"] {
        flowsheet
            .insert_stream(MaterialStreamState::new(stream_id, stream_id))
            .expect("expected stream insert");
    }
    let mut app_state = AppState::new(FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("doc-create", "Create", timestamp(10)),
    ));
    app_state.replace_canvas_suggestions(vec![
        sample_canvas_suggestion("sug-liquid", 0.92, SuggestionSource::LocalRules).with_acceptance(
            CanvasSuggestionAcceptance::MaterialConnection(CanvasSuggestedMaterialConnection {
                stream: CanvasSuggestedStreamBinding::Create {
                    stream: MaterialStreamState::new("stream-liquid", "Liquid Outlet"),
                },
                source_unit_id: UnitId::new("flash-1"),
                source_port: "liquid".to_string(),
                sink_unit_id: None,
                sink_port: None,
            }),
        ),
    ]);

    let accepted = app_state
        .accept_focused_canvas_suggestion_by_tab()
        .expect("expected terminal outlet suggestion acceptance");

    assert_eq!(
        accepted.as_ref().map(|item| item.id.as_str()),
        Some("sug-liquid")
    );
    assert!(matches!(
        app_state.workspace.command_history.current_entry(),
        Some(crate::CommandHistoryEntry {
            command: crate::DocumentCommand::ConnectPorts {
                stream_id,
                from_unit_id,
                from_port,
                to_unit_id: None,
                to_port: None,
            },
            ..
        }) if stream_id.as_str() == "stream-liquid"
            && from_unit_id.as_str() == "flash-1"
            && from_port == "liquid"
    ));
    assert_eq!(
        app_state
            .workspace
            .document
            .flowsheet
            .streams
            .get(&rf_types::StreamId::new("stream-liquid"))
            .map(|stream| stream.name.as_str()),
        Some("Liquid Outlet")
    );
    assert_eq!(
        app_state
            .workspace
            .document
            .flowsheet
            .units
            .get(&UnitId::new("flash-1"))
            .and_then(|unit| unit.ports.iter().find(|port| port.name == "liquid"))
            .and_then(|port| port.stream_id.as_ref())
            .map(|stream_id| stream_id.as_str()),
        Some("stream-liquid")
    );
}

#[test]
fn tab_does_not_accept_low_confidence_suggestion() {
    let mut app_state = AppState::new(sample_document());
    app_state.replace_canvas_suggestions(vec![sample_canvas_suggestion(
        "sug-low",
        0.60,
        SuggestionSource::RadishMind,
    )]);

    let accepted = app_state
        .accept_focused_canvas_suggestion_by_tab()
        .expect("expected low-confidence acceptance check");

    assert!(accepted.is_none());
    assert_eq!(app_state.workspace.command_history.len(), 0);
    assert_eq!(
        app_state.workspace.canvas_interaction.suggestions[0].status,
        SuggestionStatus::Focused
    );
    assert_eq!(
        app_state
            .workspace
            .canvas_interaction
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str()),
        Some("sug-low")
    );
    assert!(app_state.workspace.selection.selected_units.is_empty());
    assert_eq!(app_state.workspace.drafts.active_target, None);
    assert!(app_state.log_feed.entries.is_empty());
    assert_eq!(app_state.workspace.run_panel.latest_log_message, None);
}

#[test]
fn document_change_invalidates_canvas_suggestions_but_only_records_document_command() {
    let mut app_state = AppState::new(sample_document());
    app_state.replace_canvas_suggestions(vec![sample_canvas_suggestion(
        "sug-high",
        0.95,
        SuggestionSource::LocalRules,
    )]);

    let next_flowsheet = Flowsheet::new("demo-updated");
    app_state.commit_document_change(
        DocumentCommand::MoveUnit {
            unit_id: UnitId::new("flash-1"),
            position: CanvasPoint::new(40.0, 20.0),
        },
        next_flowsheet,
        timestamp(20),
    );

    assert_eq!(app_state.workspace.command_history.len(), 1);
    assert_eq!(
        app_state.workspace.canvas_interaction.suggestions[0].status,
        SuggestionStatus::Invalidated
    );
    assert_eq!(
        app_state.workspace.canvas_interaction.focused_suggestion_id,
        None
    );
}
