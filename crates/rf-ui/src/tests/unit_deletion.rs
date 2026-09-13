use super::*;

#[test]
fn deleting_each_builtin_unit_preserves_streams_neighbors_and_is_one_transaction() {
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::FlashDrum,
    ] {
        let spec = builtin_unit_spec(kind);
        let id = UnitId::new("target");
        let mut document = sample_document();
        let mut ports = Vec::new();
        for port in spec.ports {
            let stream_id = StreamId::new(format!("stream-{}", port.name));
            document
                .flowsheet
                .insert_stream(MaterialStreamState::from_tpzf(
                    stream_id.clone(),
                    "Preserved",
                    350.0,
                    200_000.0,
                    2.0,
                    [(ComponentId::new("a"), 0.4), (ComponentId::new("b"), 0.6)]
                        .into_iter()
                        .collect(),
                ))
                .unwrap();
            ports.push(UnitPort::new(
                port.name,
                port.direction,
                port.kind,
                Some(stream_id.clone()),
            ));
            document
                .flowsheet
                .insert_unit(UnitNode::new(
                    format!("neighbor-{}", port.name),
                    "Neighbor",
                    "feed",
                    vec![UnitPort::new(
                        "port",
                        PortDirection::Outlet,
                        PortKind::Material,
                        Some(stream_id),
                    )],
                ))
                .unwrap();
        }
        let mut unit = UnitNode::new(id.clone(), "Target", spec.kind.as_str(), ports);
        unit.parameters.outlet_temperature_k = Some(350.0);
        document.flowsheet.insert_unit(unit).unwrap();
        let original = document.flowsheet.clone();
        let mut app = AppState::new(document);
        app.focus_inspector_target(InspectorTarget::Unit(id.clone()));
        app.begin_canvas_place_unit("feed");
        let revision = app.delete_unit(&id, timestamp(20)).unwrap();
        assert_eq!(revision, 1);
        assert_eq!(app.workspace.command_history.len(), 1);
        assert_eq!(app.workspace.document.flowsheet.streams, original.streams);
        for (other_id, neighbor) in &original.units {
            if other_id != &id {
                assert_eq!(
                    app.workspace.document.flowsheet.units.get(other_id),
                    Some(neighbor)
                );
            }
        }
        assert!(app.workspace.drafts.active_target.is_none());
        assert!(app.workspace.selection.selected_units.is_empty());
        assert!(app.workspace.canvas_interaction.pending_edit.is_none());
        let deleted = app.workspace.document.flowsheet.clone();
        app.undo_document_command(timestamp(21)).unwrap().unwrap();
        assert_eq!(app.workspace.document.flowsheet, original);
        app.redo_document_command(timestamp(22)).unwrap().unwrap();
        assert_eq!(app.workspace.document.flowsheet, deleted);
        assert_eq!(app.workspace.command_history.len(), 1);
        let before_failure = app.clone();
        assert!(app.delete_unit(&id, timestamp(23)).is_err());
        assert_eq!(app, before_failure, "stale ID must not mutate state");
    }
}

#[test]
fn deleted_and_undone_unit_ids_are_not_reused_after_history_branching() {
    let mut app = AppState::new(sample_document());
    let create = |app: &mut AppState| {
        app.begin_canvas_place_unit("feed");
        app.commit_canvas_pending_edit_at(CanvasPoint::new(50.0, 60.0), timestamp(20))
            .unwrap()
            .unwrap()
            .unit_id
    };
    let first = create(&mut app);
    app.delete_unit(&first, timestamp(21)).unwrap();
    let second = create(&mut app);
    assert_ne!(first, second);
    app.undo_document_command(timestamp(22)).unwrap().unwrap();
    let third = create(&mut app);
    assert_ne!(third, second);
    assert_ne!(third, first);
    assert!(!app.workspace.command_history.can_redo());
    app.undo_document_command(timestamp(23)).unwrap().unwrap();
    app.undo_document_command(timestamp(24)).unwrap().unwrap();
    assert!(app.workspace.document.flowsheet.units.contains_key(&first));
}
