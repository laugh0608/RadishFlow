use super::*;

#[test]
fn applying_run_panel_recovery_action_selects_unit_and_opens_inspector() {
    let mut document = sample_document();
    document
        .flowsheet
        .insert_unit(UnitNode::new(
            "heater-1",
            "Heater",
            "heater",
            vec![
                UnitPort::new("inlet", PortDirection::Inlet, PortKind::Material, None),
                UnitPort::new("outlet", PortDirection::Outlet, PortKind::Material, None),
            ],
        ))
        .expect("expected heater insert");
    let mut app_state = AppState::new(document);
    let summary = DiagnosticSummary::new(
        0,
        DiagnosticSeverity::Error,
        "solver.step.spec: solver step 1 unit spec validation failed",
    )
    .with_primary_code("solver.step.spec")
    .with_related_unit_ids(vec![UnitId::new("heater-1")]);

    app_state.record_failure(0, RunStatus::Error, summary);
    let action = app_state
        .workspace
        .run_panel
        .notice
        .as_ref()
        .and_then(|notice| notice.recovery_action.as_ref())
        .cloned()
        .expect("expected recovery action");

    let applied_target = app_state.apply_run_panel_recovery_action(&action);

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
    );
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&UnitId::new("heater-1"))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
    );
    assert!(app_state.workspace.panels.inspector_open);
}

#[test]
fn focusing_inspector_target_selects_unit_without_document_mutation() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);

    let applied_target =
        app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&UnitId::new("feed-1"))
    );
    assert!(app_state.workspace.selection.selected_streams.is_empty());
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
    assert!(app_state.workspace.panels.inspector_open);
}

#[test]
fn focusing_inspector_target_selects_stream_and_clears_previous_unit() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

    let applied_target = app_state
        .focus_inspector_target(crate::InspectorTarget::Stream(StreamId::new("stream-feed")));

    assert_eq!(
        applied_target,
        Some(crate::InspectorTarget::Stream(StreamId::new("stream-feed")))
    );
    assert!(app_state.workspace.selection.selected_units.is_empty());
    assert!(
        app_state
            .workspace
            .selection
            .selected_streams
            .contains(&StreamId::new("stream-feed"))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Stream(StreamId::new("stream-feed")))
    );
}

#[test]
fn focusing_missing_inspector_target_keeps_current_focus() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

    let applied_target =
        app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("missing-unit")));

    assert_eq!(applied_target, None);
    assert!(
        app_state
            .workspace
            .selection
            .selected_units
            .contains(&UnitId::new("feed-1"))
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
    );
}

#[test]
fn updating_stream_inspector_draft_keeps_document_unchanged() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(StreamId::new("stream-feed")));

    let outcome = app_state
        .update_stream_inspector_draft(
            &StreamId::new("stream-feed"),
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected draft update");

    assert_eq!(
        outcome.key,
        crate::stream_inspector_draft_key(
            &StreamId::new("stream-feed"),
            &crate::StreamInspectorDraftField::TemperatureK,
        )
    );
    assert!(outcome.is_dirty);
    assert_eq!(outcome.validation, crate::DraftValidationState::Valid);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-feed")].temperature_k,
        298.15
    );
    assert_eq!(
        app_state.workspace.drafts.fields.get(&outcome.key),
        Some(&crate::DraftValue::Number(crate::FieldDraft {
            original: "298.15".to_string(),
            current: "333.5".to_string(),
            is_dirty: true,
            validation: crate::DraftValidationState::Valid,
        }))
    );
}

#[test]
fn updating_stream_inspector_draft_preserves_invalid_raw_number() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(StreamId::new("stream-feed")));

    let outcome = app_state
        .update_stream_inspector_draft(
            &StreamId::new("stream-feed"),
            crate::StreamInspectorDraftField::PressurePa,
            "not-a-pressure",
        )
        .expect("expected draft update");

    assert_eq!(outcome.validation, crate::DraftValidationState::Invalid);
    assert_eq!(
        app_state.workspace.drafts.fields.get(&outcome.key),
        Some(&crate::DraftValue::Number(crate::FieldDraft {
            original: "101325".to_string(),
            current: "not-a-pressure".to_string(),
            is_dirty: true,
            validation: crate::DraftValidationState::Invalid,
        }))
    );
    assert_eq!(app_state.workspace.document.revision, 0);
    assert!(app_state.workspace.command_history.is_empty());
}

#[test]
fn updating_stream_inspector_draft_requires_active_stream_target() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

    let outcome = app_state.update_stream_inspector_draft(
        &StreamId::new("stream-feed"),
        crate::StreamInspectorDraftField::Name,
        "Edited stream",
    );

    assert_eq!(outcome, None);
    assert!(app_state.workspace.drafts.fields.is_empty());
    assert_eq!(app_state.workspace.document.revision, 0);
}

#[test]
fn committing_stream_inspector_draft_writes_document_command_and_preserves_focus() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected draft update");

    let outcome = app_state
        .commit_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            timestamp(42),
        )
        .expect("expected draft commit")
        .expect("expected applied draft commit");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: "temperature_k".to_string(),
            value: CommandValue::Number(333.5),
        }
    );
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        333.5
    );
    assert_eq!(app_state.workspace.command_history.len(), 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&outcome.command)
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Stream(stream_id.clone()))
    );
    assert!(!app_state.workspace.drafts.fields.contains_key(&outcome.key));
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
    assert_eq!(app_state.workspace.solve_session.status, RunStatus::Dirty);
}

#[test]
fn committing_stream_inspector_composition_draft_updates_overall_mole_fraction() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    let component_id = ComponentId::new("component-a");
    let field = crate::StreamInspectorDraftField::OverallMoleFraction(component_id.clone());
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

    let update = app_state
        .update_stream_inspector_draft(&stream_id, field.clone(), "0.25")
        .expect("expected composition draft update");

    assert_eq!(
        update.key,
        "stream:stream-feed:overall_mole_fraction:component-a"
    );
    assert!(update.is_dirty);
    assert_eq!(update.validation, crate::DraftValidationState::Valid);
    assert_eq!(
        app_state.workspace.drafts.fields.get(&update.key),
        Some(&crate::DraftValue::Number(crate::FieldDraft {
            original: "0.4".to_string(),
            current: "0.25".to_string(),
            is_dirty: true,
            validation: crate::DraftValidationState::Valid,
        }))
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].overall_mole_fractions
            [&component_id],
        0.4
    );

    let outcome = app_state
        .commit_stream_inspector_draft(&stream_id, field, timestamp(42))
        .expect("expected draft commit")
        .expect("expected applied composition draft commit");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.command,
        DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: "overall_mole_fraction:component-a".to_string(),
            value: CommandValue::Number(0.25),
        }
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].overall_mole_fractions
            [&component_id],
        0.25
    );
    assert!(!app_state.workspace.drafts.fields.contains_key(&update.key));
}

#[test]
fn normalizing_stream_inspector_composition_drafts_commits_all_mole_fractions() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::OverallMoleFraction(ComponentId::new("component-a")),
            "0.25",
        )
        .expect("expected component-a draft update");

    let outcome = app_state
        .normalize_stream_inspector_composition_drafts(&stream_id, timestamp(42))
        .expect("expected composition normalize")
        .expect("expected applied composition normalize");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.keys,
        vec![
            "stream:stream-feed:overall_mole_fraction:component-a".to_string(),
            "stream:stream-feed:overall_mole_fraction:component-b".to_string(),
        ]
    );
    assert_eq!(
        outcome.command,
        DocumentCommand::SetStreamSpecifications {
            stream_id: stream_id.clone(),
            values: vec![
                crate::StreamSpecificationValue {
                    field: "overall_mole_fraction:component-a".to_string(),
                    value: CommandValue::Number(0.25 / 0.85),
                },
                crate::StreamSpecificationValue {
                    field: "overall_mole_fraction:component-b".to_string(),
                    value: CommandValue::Number(0.6 / 0.85),
                },
            ],
        }
    );
    let stream = &app_state.workspace.document.flowsheet.streams[&stream_id];
    let component_a = stream.overall_mole_fractions[&ComponentId::new("component-a")];
    let component_b = stream.overall_mole_fractions[&ComponentId::new("component-b")];
    assert_eq!(component_a, 0.25 / 0.85);
    assert_eq!(component_b, 0.6 / 0.85);
    assert!((component_a + component_b - 1.0).abs() <= 1e-12);
    assert!(app_state.workspace.drafts.fields.is_empty());
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn normalizing_stream_inspector_composition_drafts_preserves_invalid_drafts() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    let update = app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::OverallMoleFraction(ComponentId::new("component-a")),
            "not-a-fraction",
        )
        .expect("expected invalid component draft update");

    let outcome = app_state
        .normalize_stream_inspector_composition_drafts(&stream_id, timestamp(42))
        .expect("expected ignored composition normalize");

    assert_eq!(outcome, None);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(app_state.workspace.command_history.len(), 0);
    assert!(app_state.workspace.drafts.fields.contains_key(&update.key));
}

#[test]
fn updating_stream_inspector_composition_draft_rejects_unknown_component() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

    let outcome = app_state.update_stream_inspector_draft(
        &stream_id,
        crate::StreamInspectorDraftField::OverallMoleFraction(ComponentId::new(
            "missing-component",
        )),
        "0.25",
    );

    assert_eq!(outcome, None);
    assert!(app_state.workspace.drafts.fields.is_empty());
    assert_eq!(app_state.workspace.document.revision, 0);
}

#[test]
fn committing_stream_inspector_drafts_records_one_batch_history_entry() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected temperature draft update");
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::PressurePa,
            "202650",
        )
        .expect("expected pressure draft update");

    let outcome = app_state
        .commit_stream_inspector_drafts(&stream_id, timestamp(42))
        .expect("expected batch commit")
        .expect("expected applied batch commit");

    assert_eq!(outcome.revision, 1);
    assert_eq!(
        outcome.keys,
        vec![
            "stream:stream-feed:temperature_k".to_string(),
            "stream:stream-feed:pressure_pa".to_string()
        ]
    );
    assert_eq!(
        outcome.command,
        DocumentCommand::SetStreamSpecifications {
            stream_id: stream_id.clone(),
            values: vec![
                crate::StreamSpecificationValue {
                    field: "temperature_k".to_string(),
                    value: CommandValue::Number(333.5),
                },
                crate::StreamSpecificationValue {
                    field: "pressure_pa".to_string(),
                    value: CommandValue::Number(202650.0),
                },
            ],
        }
    );
    let stream = &app_state.workspace.document.flowsheet.streams[&stream_id];
    assert_eq!(stream.temperature_k, 333.5);
    assert_eq!(stream.pressure_pa, 202650.0);
    assert_eq!(app_state.workspace.command_history.len(), 1);
    assert_eq!(
        app_state
            .workspace
            .command_history
            .current_entry()
            .map(|entry| &entry.command),
        Some(&outcome.command)
    );
    assert!(app_state.workspace.drafts.fields.is_empty());
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );
}

#[test]
fn batch_commit_preserves_invalid_stream_inspector_drafts() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected temperature draft update");
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::PressurePa,
            "not-a-pressure",
        )
        .expect("expected invalid pressure draft update");

    let outcome = app_state
        .commit_stream_inspector_drafts(&stream_id, timestamp(42))
        .expect("expected batch commit")
        .expect("expected applied batch commit");

    assert_eq!(outcome.keys, vec!["stream:stream-feed:temperature_k"]);
    assert_eq!(app_state.workspace.document.revision, 1);
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        333.5
    );
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].pressure_pa,
        101_325.0
    );
    assert!(
        app_state
            .workspace
            .drafts
            .fields
            .contains_key("stream:stream-feed:pressure_pa")
    );
}

#[test]
fn discarding_stream_inspector_draft_removes_field_without_document_mutation() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    let update = app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected temperature draft update");

    let outcome = app_state
        .discard_stream_inspector_draft(&stream_id, crate::StreamInspectorDraftField::TemperatureK)
        .expect("expected discarded draft");

    assert_eq!(outcome.key, update.key);
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(app_state.workspace.command_history.len(), 0);
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        298.15
    );
    assert!(app_state.workspace.drafts.fields.is_empty());
}

#[test]
fn discarding_stream_inspector_drafts_removes_valid_and_invalid_fields() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected temperature draft update");
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::PressurePa,
            "not-a-pressure",
        )
        .expect("expected invalid pressure draft update");

    let outcome = app_state
        .discard_stream_inspector_drafts(&stream_id)
        .expect("expected discarded drafts");

    assert_eq!(
        outcome.keys,
        vec![
            "stream:stream-feed:temperature_k".to_string(),
            "stream:stream-feed:pressure_pa".to_string()
        ]
    );
    assert_eq!(app_state.workspace.document.revision, 0);
    assert_eq!(app_state.workspace.command_history.len(), 0);
    assert!(app_state.workspace.drafts.fields.is_empty());
}

#[test]
fn undo_redo_replays_stream_inspector_document_snapshots() {
    let document = inspector_focus_document();
    let mut app_state = AppState::new(document);
    let stream_id = StreamId::new("stream-feed");
    app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
    app_state
        .update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            "333.5",
        )
        .expect("expected draft update");
    app_state
        .commit_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::TemperatureK,
            timestamp(42),
        )
        .expect("expected draft commit")
        .expect("expected applied draft commit");

    let undo = app_state
        .undo_document_command(timestamp(43))
        .expect("expected undo")
        .expect("expected undo result");

    assert_eq!(undo.direction, crate::DocumentHistoryDirection::Undo);
    assert_eq!(undo.revision, 2);
    assert_eq!(app_state.workspace.command_history.cursor, 0);
    assert!(app_state.workspace.command_history.can_redo());
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        298.15
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Stream(stream_id.clone()))
    );
    assert_eq!(
        app_state.workspace.solve_session.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
    );

    let redo = app_state
        .redo_document_command(timestamp(44))
        .expect("expected redo")
        .expect("expected redo result");

    assert_eq!(redo.direction, crate::DocumentHistoryDirection::Redo);
    assert_eq!(redo.revision, 3);
    assert_eq!(app_state.workspace.command_history.cursor, 1);
    assert!(!app_state.workspace.command_history.can_redo());
    assert_eq!(
        app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
        333.5
    );
    assert_eq!(
        app_state.workspace.drafts.active_target,
        Some(crate::InspectorTarget::Stream(stream_id))
    );
    assert!(app_state.workspace.drafts.fields.is_empty());
}
