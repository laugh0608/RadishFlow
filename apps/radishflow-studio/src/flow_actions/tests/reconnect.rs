use super::*;
use rf_ui::{StreamReconnect, StreamReconnectOptions, stream_reconnect_options};

fn mixer() -> AppState {
    crate::load_project_app_state(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"),
    )
    .unwrap()
}

fn disconnect(app: &mut AppState, stream: &str, missing: PortDirection) {
    execute(
        app,
        FlowAction::DisconnectStream {
            stream: stream.into(),
            direction: Some(missing),
        },
    );
}

fn open(app: &AppState, stream: &str) -> StreamReconnectOptions {
    let StreamReconnect::Open(options) =
        stream_reconnect_options(&app.workspace.document.flowsheet, &stream.into())
    else {
        panic!("expected a single bound end")
    };
    options
}

fn pick(app: &AppState, stream: &str, index: usize) -> FlowAction {
    let options = open(app, stream);
    FlowAction::Reconnect {
        stream: stream.into(),
        fixed: options.fixed,
        missing: options.missing,
        chosen: options.candidates[index].clone(),
    }
}

fn port(unit: &str, port: &str) -> MaterialPortTarget {
    MaterialPortTarget {
        unit_id: unit.into(),
        port: port.into(),
    }
}

fn set_binding(
    flow: &mut rf_model::Flowsheet,
    endpoint: &MaterialPortTarget,
    stream: Option<StreamId>,
) {
    flow.units
        .get_mut(&endpoint.unit_id)
        .unwrap()
        .ports
        .iter_mut()
        .find(|p| p.name == endpoint.port)
        .unwrap()
        .stream_id = stream;
}

#[test]
fn second_reconnect_candidate_changes_only_the_selected_binding_and_survives_reopen() {
    for (missing, expected) in [
        (
            PortDirection::Outlet,
            [port("feed-a", "outlet"), port("feed-b", "outlet")],
        ),
        (
            PortDirection::Inlet,
            [port("mixer-1", "inlet_a"), port("mixer-1", "inlet_b")],
        ),
    ] {
        let mut app = mixer();
        // Labels and the insertion order of the Mixer's ports cannot identify the user's choice.
        app.workspace
            .document
            .flowsheet
            .units
            .get_mut(&"feed-a".into())
            .unwrap()
            .name = "Feed B".into();
        app.workspace
            .document
            .flowsheet
            .units
            .get_mut(&"mixer-1".into())
            .unwrap()
            .ports
            .reverse();
        disconnect(&mut app, "stream-feed-a", missing);
        assert_eq!(open(&app, "stream-feed-a").candidates, expected[..1]);
        disconnect(&mut app, "stream-feed-b", missing);
        assert_eq!(open(&app, "stream-feed-a").candidates, expected);
        let before = app.workspace.document.flowsheet.clone();
        let mut after = before.clone();
        set_binding(&mut after, &expected[1], Some("stream-feed-a".into()));
        let action = pick(&app, "stream-feed-a", 1);
        let receipt = execute(&mut app, action);
        assert_eq!(
            (receipt.before.revision(), receipt.after.revision()),
            (2, 3)
        );
        assert_eq!(app.workspace.command_history.len(), 3);
        assert_eq!(app.workspace.document.flowsheet, after);
        assert_eq!(open(&app, "stream-feed-b").candidates, expected[..1]);
        app.undo_document_command(UNIX_EPOCH).unwrap().unwrap();
        assert_eq!(app.workspace.document.flowsheet, before);
        app.redo_document_command(UNIX_EPOCH).unwrap().unwrap();
        assert_eq!(app.workspace.document.flowsheet, after);
        let file = std::env::temp_dir().join(format!(
            "rf-reconnect-{}-{}.rfproj.json",
            std::process::id(),
            missing.as_str()
        ));
        crate::dispatch_document_lifecycle(
            &mut app,
            crate::StudioDocumentLifecycleCommand::SaveUpgraded { path: file.clone() },
        )
        .unwrap();
        let reopened = crate::load_project_app_state(&file).unwrap();
        std::fs::remove_file(file).unwrap();
        assert_eq!(reopened.workspace.document.revision, 5);
        assert_eq!(reopened.workspace.document.flowsheet, after);
    }
}

#[test]
fn reconnect_rejects_obsolete_or_ineligible_choices_without_touching_history() {
    for missing in [PortDirection::Inlet, PortDirection::Outlet] {
        let mut app = mixer();
        disconnect(&mut app, "stream-feed-a", missing);
        disconnect(&mut app, "stream-feed-b", missing);
        let chosen = pick(&app, "stream-feed-a", 1);
        let req = request(&app, chosen.clone());
        let before = app.clone();
        assert_eq!(
            execute_flow_action(&mut app, FlowEditAccess::ReadOnly, &req, UNIX_EPOCH),
            Err(FlowActionError::ReadOnly)
        );
        assert_eq!(app, before);
        let mut reopened = before.clone();
        reopened.workspace.reset_flow_run_session();
        rejects(
            &mut reopened,
            &req,
            FlowActionError::Context(FlowEditConflict::DifferentSession),
        );
        let mut pending = before.clone();
        pending.begin_canvas_place_unit("heater");
        rejects(&mut pending, &req, FlowActionError::PendingCanvasEdit);
        let mut draft = before.clone();
        let id = UnitId::new("feed-b");
        draft.focus_inspector_target(rf_ui::InspectorTarget::Unit(id.clone()));
        draft.update_unit_inspector_draft(&id, rf_ui::UnitInspectorDraftField::Name, "pending");
        rejects(&mut draft, &req, FlowActionError::PendingDrafts);
        let other = pick(&app, "stream-feed-b", 1);
        execute(&mut app, other);
        rejects(
            &mut app,
            &req,
            FlowActionError::Context(FlowEditConflict::Revision {
                expected: 2,
                actual: 3,
            }),
        );
        let current = request(&app, chosen);
        let before = app.clone();
        assert!(matches!(
            execute_flow_action(&mut app, FlowEditAccess::Editable, &current, UNIX_EPOCH),
            Err(FlowActionError::Rejected(_))
        ));
        assert_eq!(app, before);
        let mut wrong_fixed = pick(&app, "stream-feed-a", 0);
        let FlowAction::Reconnect { fixed, .. } = &mut wrong_fixed else {
            unreachable!()
        };
        fixed.port = "moved".into();
        let wrong = request(&app, wrong_fixed);
        assert!(
            execute_flow_action(&mut app, FlowEditAccess::Editable, &wrong, UNIX_EPOCH).is_err()
        );
        assert_eq!(app, before);
    }
}

#[test]
fn reconnect_distinguishes_occupied_unbound_and_duplicate_ends_in_both_directions() {
    let stream: StreamId = "stream-feed-a".into();
    for missing in [PortDirection::Inlet, PortDirection::Outlet] {
        let mut app = mixer();
        assert!(matches!(
            stream_reconnect_options(&app.workspace.document.flowsheet, &stream),
            StreamReconnect::Connected { .. }
        ));
        disconnect(&mut app, stream.as_str(), missing);
        let vacant = open(&app, stream.as_str()).candidates.remove(0);
        let mut flow = app.workspace.document.flowsheet.clone();
        set_binding(&mut flow, &vacant, Some("stream-blocker".into()));
        let StreamReconnect::Open(options) = stream_reconnect_options(&flow, &stream) else {
            panic!()
        };
        assert!(options.candidates.is_empty() && options.occupied > 0);
        set_binding(&mut flow, &options.fixed, None);
        assert_eq!(
            stream_reconnect_options(&flow, &stream),
            StreamReconnect::Unbound
        );
        let duplicate = if missing == PortDirection::Inlet {
            port("feed-b", "outlet")
        } else {
            port("mixer-1", "inlet_b")
        };
        set_binding(&mut flow, &options.fixed, Some(stream.clone()));
        set_binding(&mut flow, &duplicate, Some(stream.clone()));
        assert_eq!(
            stream_reconnect_options(&flow, &stream),
            StreamReconnect::InvalidBindings
        );
    }
}

#[test]
fn reconnect_with_multiple_legal_ports_still_refuses_cycle_self_and_wrong_direction() {
    for missing in [PortDirection::Inlet, PortDirection::Outlet] {
        let mut app = app();
        let valves: Vec<_> = (0..4)
            .map(|_| create(&mut app, BuiltinUnitKind::Valve))
            .collect();
        let inlet = |i: usize| port(valves[i].as_str(), "inlet");
        let outlet = |i: usize| port(valves[i].as_str(), "outlet");
        let connect = |id: &str, source, sink| {
            FlowAction::ConnectExact(MaterialConnectionTarget {
                stream_id: id.into(),
                source,
                sink: Some(sink),
            })
        };
        execute(&mut app, connect("v1-to-v2", outlet(0), inlet(1)));
        let (source, sink, cycle_endpoint, self_endpoint, wrong_direction) = match missing {
            PortDirection::Inlet => (outlet(1), inlet(2), inlet(0), inlet(1), outlet(3)),
            PortDirection::Outlet => (outlet(2), inlet(0), outlet(1), outlet(0), inlet(3)),
        };
        execute(&mut app, connect("partial", source, sink));
        disconnect(&mut app, "partial", missing);
        let options = open(&app, "partial");
        assert_eq!(options.candidates.len(), 2);
        assert!(options.cycle_blocked >= 1);
        for forbidden in [cycle_endpoint, self_endpoint, wrong_direction] {
            assert!(!options.candidates.contains(&forbidden));
            let request = request(
                &app,
                FlowAction::Reconnect {
                    stream: "partial".into(),
                    fixed: options.fixed.clone(),
                    missing,
                    chosen: forbidden,
                },
            );
            let before = app.clone();
            assert!(matches!(
                execute_flow_action(&mut app, FlowEditAccess::Editable, &request, UNIX_EPOCH),
                Err(FlowActionError::Rejected(_))
            ));
            assert_eq!(app, before);
        }
    }
}
