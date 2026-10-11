use super::*;
use rf_ui::CanvasSuggestedStreamBinding;

#[test]
fn six_builtin_outlet_previews_match_atomic_creation_and_do_not_fill_parameters() {
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::FlashDrum,
    ] {
        let mut app = missing_outlet(kind);
        let mut session = ModelingRecommendationSession::new(&app.workspace.document);
        let key = key(&mut session, &app, RecommendationRule::CreateOutletStream);
        let before = app.clone();
        let preview = session.prepare(&app, &key).unwrap();
        assert_eq!(app, before);
        let RecommendedAction::MaterialConnection(connection) = &preview.recommendation().action
        else {
            panic!()
        };
        let CanvasSuggestedStreamBinding::Create { stream } = &connection.stream else {
            panic!()
        };
        if kind == BuiltinUnitKind::Feed {
            assert_eq!(
                (
                    stream.temperature_k,
                    stream.pressure_pa,
                    stream.total_molar_flow_mol_s
                ),
                (298.15, 101325., 1.)
            );
            assert_eq!(
                stream.overall_mole_fractions,
                [("a".into(), 0.5), ("b".into(), 0.5)].into()
            );
        }
        session.accept(&mut app, &preview, UNIX_EPOCH).unwrap();
        assert_eq!(&flow(&app).streams[&stream.id], stream);
        assert_eq!(app.workspace.document.revision, 1);
        assert_eq!(app.workspace.command_history.len(), 1);
        assert!(
            flow(&app)
                .units
                .values()
                .all(|u| u.parameters.outlet_temperature_k.is_none()
                    && u.parameters.outlet_pressure_pa.is_none())
        );
        assert_eq!(flow(&app).streams.len(), flow(&before).streams.len() + 1);
        assert!(app.workspace.solve_session.latest_snapshot.is_none());
        app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
        assert_eq!(flow(&app), flow(&before));
        app.redo_document_command(UNIX_EPOCH).unwrap().unwrap();
        assert_eq!(&flow(&app).streams[&stream.id], stream);
    }
}

#[test]
fn feed_creation_without_components_preserves_empty_composition_and_remaining_problem() {
    let mut app = missing_outlet(BuiltinUnitKind::Feed);
    app.workspace.document.flowsheet.components.clear();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    let k = key(&mut session, &app, RecommendationRule::CreateOutletStream);
    let p = session.prepare(&app, &k).unwrap();
    session.accept(&mut app, &p, UNIX_EPOCH).unwrap();
    assert!(flow(&app).components.is_empty());
    assert!(
        flow(&app)
            .streams
            .values()
            .all(|s| s.overall_mole_fractions.is_empty())
    );
    assert!(session.checks().current_count().unwrap() > 0);
}

fn two_sinks() -> AppState {
    let mut app = feed_app();
    for id in ["heater-a", "heater-b"] {
        app.workspace
            .document
            .flowsheet
            .insert_unit(UnitNode::new(
                id,
                id,
                "heater",
                vec![
                    port("inlet", PortDirection::Inlet, None),
                    port("outlet", PortDirection::Outlet, None),
                ],
            ))
            .unwrap();
    }
    app
}

#[test]
fn competing_connections_require_one_exact_choice_then_invalidate_the_other() {
    let mut app = two_sinks();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    session.refresh(&app);
    let keys: Vec<_> = session
        .entries()
        .iter()
        .filter(|e| e.recommendation.key.rule == RecommendationRule::ConnectMaterialStream)
        .map(|e| e.recommendation.key.clone())
        .collect();
    assert_eq!(keys.len(), 2);
    let a = session.prepare(&app, &keys[0]).unwrap();
    let b = session.prepare(&app, &keys[1]).unwrap();
    let before = app.clone();
    session.accept(&mut app, &a, UNIX_EPOCH).unwrap();
    let committed = app.clone();
    assert!(matches!(
        session.accept(&mut app, &b, UNIX_EPOCH),
        Err(RecommendationError::StaleRevision { .. })
    ));
    assert_eq!(app, committed);
    assert_eq!(flow(&app).streams.len(), flow(&before).streams.len());
    assert_eq!(app.workspace.command_history.len(), 1);
    assert_eq!(
        flow(&app)
            .units
            .values()
            .flat_map(|u| &u.ports)
            .filter(|p| p.direction == PortDirection::Inlet && p.stream_id == Some("source".into()))
            .count(),
        1
    );
}

#[test]
fn existing_ambiguity_policy_is_not_expanded_and_missing_association_is_not_invented() {
    let mut app = two_sinks();
    let mut second = source(&app).clone();
    second.id = "second".into();
    app.workspace
        .document
        .flowsheet
        .insert_stream(second)
        .unwrap();
    app.workspace
        .document
        .flowsheet
        .insert_unit(UnitNode::new(
            "feed-b",
            "Feed B",
            "feed",
            vec![port("outlet", PortDirection::Outlet, Some("second"))],
        ))
        .unwrap();
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    session.refresh(&app);
    assert!(
        !session
            .entries()
            .iter()
            .any(|e| e.recommendation.key.rule == RecommendationRule::ConnectMaterialStream)
    );
    let app = missing_outlet(BuiltinUnitKind::FlashDrum);
    let mut session = ModelingRecommendationSession::new(&app.workspace.document);
    session.refresh(&app);
    let outlets: Vec<_> = session
        .entries()
        .iter()
        .filter(|e| e.recommendation.key.rule == RecommendationRule::CreateOutletStream)
        .collect();
    assert_eq!(outlets.len(), 2);
    assert_eq!(
        outlets
            .iter()
            .filter(|e| e.recommendation.related_problems.is_empty())
            .count(),
        1,
        "A1 reports only the first unbound outlet"
    );
}

#[test]
fn invalid_ports_and_dependency_cycles_never_become_executable_recommendations() {
    let mut bad = missing_outlet(BuiltinUnitKind::Feed);
    bad.workspace
        .document
        .flowsheet
        .units
        .get_mut(&"feed".into())
        .unwrap()
        .ports[0]
        .direction = PortDirection::Inlet;
    let before = bad.clone();
    let mut session = ModelingRecommendationSession::new(&bad.workspace.document);
    session.refresh(&bad);
    assert!(
        !session
            .entries()
            .iter()
            .any(|e| e.recommendation.key.rule == RecommendationRule::CreateOutletStream)
    );
    assert_eq!(bad, before);
    // A downstream free outlet would close A -> B -> A. Local enumeration alone
    // offers it; the shared graph check must reject both this and the self-edge.
    for two_units in [false, true] {
        let mut app = feed_app();
        app.workspace.document.flowsheet.units.clear();
        app.workspace
            .document
            .flowsheet
            .insert_unit(UnitNode::new(
                "a",
                "A",
                "heater",
                vec![
                    port("inlet", PortDirection::Inlet, None),
                    port("outlet", PortDirection::Outlet, Some("source")),
                ],
            ))
            .unwrap();
        if two_units {
            let mut second = source(&app).clone();
            second.id = "second".into();
            app.workspace
                .document
                .flowsheet
                .insert_stream(second)
                .unwrap();
            app.workspace
                .document
                .flowsheet
                .insert_unit(UnitNode::new(
                    "b",
                    "B",
                    "heater",
                    vec![
                        port("inlet", PortDirection::Inlet, Some("source")),
                        port("outlet", PortDirection::Outlet, Some("second")),
                    ],
                ))
                .unwrap();
        }
        let mut session = ModelingRecommendationSession::new(&app.workspace.document);
        session.refresh(&app);
        assert!(
            !session
                .entries()
                .iter()
                .any(|e| e.recommendation.key.rule == RecommendationRule::ConnectMaterialStream)
        );
    }
}

#[test]
fn ambiguous_sources_unknown_kinds_and_duplicate_signatures_are_not_recommended() {
    for case in 0..3 {
        let mut app = two_sinks();
        match case {
            0 => {
                app.workspace
                    .document
                    .flowsheet
                    .insert_unit(UnitNode::new(
                        "other-feed",
                        "Duplicate source",
                        "feed",
                        vec![port("outlet", PortDirection::Outlet, Some("source"))],
                    ))
                    .unwrap();
            }
            1 => {
                app.workspace
                    .document
                    .flowsheet
                    .units
                    .get_mut(&"feed".into())
                    .unwrap()
                    .kind = "unknown".into();
            }
            _ => {
                app.workspace
                    .document
                    .flowsheet
                    .units
                    .get_mut(&"feed".into())
                    .unwrap()
                    .ports
                    .push(port("outlet", PortDirection::Outlet, Some("source")));
            }
        }
        let before = app.clone();
        let mut session = ModelingRecommendationSession::new(&app.workspace.document);
        session.refresh(&app);
        assert!(
            !session
                .entries()
                .iter()
                .any(|e| e.recommendation.key.rule == RecommendationRule::ConnectMaterialStream)
        );
        assert_eq!(app, before);
    }
}
