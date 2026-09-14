use super::*;
use crate::{DraftValidationState, DraftValue, StreamInspectorDraftField, UnitInspectorDraftField};

fn pressure_draft_app(kind: BuiltinUnitKind) -> AppState {
    let mut document = inspector_focus_document();
    document
        .flowsheet
        .insert_unit(UnitNode::new(
            "unit-1",
            "Pressure unit",
            kind.as_str(),
            vec![UnitPort::new(
                "inlet",
                PortDirection::Inlet,
                PortKind::Material,
                Some("stream-feed".into()),
            )],
        ))
        .unwrap();
    document
        .flowsheet
        .units
        .get_mut(&UnitId::new("unit-1"))
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(90_000.0);
    AppState::new(document)
}

fn update_inlet_pressure(app: &mut AppState, pressure: &str, through_feed: bool) {
    if through_feed {
        let id = UnitId::new("feed-1");
        app.focus_inspector_target(InspectorTarget::Unit(id.clone()));
        app.update_unit_inspector_draft(&id, UnitInspectorDraftField::OutletPressurePa, pressure)
            .unwrap();
        app.commit_unit_inspector_draft(
            &id,
            UnitInspectorDraftField::OutletPressurePa,
            timestamp(20),
        )
        .unwrap()
        .unwrap();
    } else {
        let id = StreamId::new("stream-feed");
        app.focus_inspector_target(InspectorTarget::Stream(id.clone()));
        app.update_stream_inspector_draft(&id, StreamInspectorDraftField::PressurePa, pressure)
            .unwrap();
        app.commit_stream_inspector_draft(
            &id,
            StreamInspectorDraftField::PressurePa,
            timestamp(20),
        )
        .unwrap()
        .unwrap();
    }
}

#[test]
fn retained_pressure_drafts_follow_inlet_changes_without_extra_document_commits() {
    for kind in [
        BuiltinUnitKind::Valve,
        BuiltinUnitKind::Heater,
        BuiltinUnitKind::Cooler,
        BuiltinUnitKind::Mixer,
    ] {
        for through_feed in [true, false] {
            let mut app = pressure_draft_app(kind);
            let id = UnitId::new("unit-1");
            app.focus_inspector_target(InspectorTarget::Unit(id.clone()));
            app.update_unit_inspector_draft(
                &id,
                UnitInspectorDraftField::OutletPressurePa,
                " 95000 ",
            )
            .unwrap();
            for (pressure, validation) in [
                ("85000", DraftValidationState::Invalid),
                ("120000", DraftValidationState::Valid),
            ] {
                let revision = app.workspace.document.revision;
                let history = app.workspace.command_history.len();
                update_inlet_pressure(&mut app, pressure, through_feed);
                let DraftValue::Number(draft) =
                    &app.workspace.drafts.fields["unit:unit-1:outlet_pressure_pa"]
                else {
                    panic!("expected numeric draft")
                };
                assert_eq!(
                    draft.validation, validation,
                    "{kind:?}, through_feed={through_feed}"
                );
                assert_eq!(draft.current, " 95000 ");
                assert!(draft.is_dirty);
                assert_eq!(app.workspace.document.revision, revision + 1);
                assert_eq!(app.workspace.command_history.len(), history + 1);
                assert_eq!(
                    app.workspace.document.flowsheet.units[&id]
                        .parameters
                        .outlet_pressure_pa,
                    Some(90_000.0)
                );
                app.focus_inspector_target(InspectorTarget::Unit(id.clone()));
                if validation == DraftValidationState::Invalid {
                    assert!(
                        app.commit_unit_inspector_draft(
                            &id,
                            UnitInspectorDraftField::OutletPressurePa,
                            timestamp(21)
                        )
                        .unwrap()
                        .is_none()
                    );
                }
            }
            app.commit_unit_inspector_draft(
                &id,
                UnitInspectorDraftField::OutletPressurePa,
                timestamp(22),
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                app.workspace.document.flowsheet.units[&id]
                    .parameters
                    .outlet_pressure_pa,
                Some(95_000.0)
            );
            app.undo_document_command(timestamp(23)).unwrap().unwrap();
            assert_eq!(
                app.workspace.document.flowsheet.units[&id]
                    .parameters
                    .outlet_pressure_pa,
                Some(90_000.0)
            );
            app.redo_document_command(timestamp(24)).unwrap().unwrap();
            assert_eq!(
                app.workspace.document.flowsheet.units[&id]
                    .parameters
                    .outlet_pressure_pa,
                Some(95_000.0)
            );
        }
    }
}

#[test]
fn revalidated_default_pressure_still_requires_explicit_commit() {
    let mut app = pressure_draft_app(BuiltinUnitKind::Valve);
    let id = UnitId::new("unit-1");
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&id)
        .unwrap()
        .parameters
        .outlet_pressure_pa = None;
    update_inlet_pressure(&mut app, "85000", true);
    app.focus_inspector_target(InspectorTarget::Unit(id.clone()));
    app.update_unit_inspector_draft(&id, UnitInspectorDraftField::OutletPressurePa, "90000")
        .unwrap();
    update_inlet_pressure(&mut app, "120000", true);
    let DraftValue::Number(draft) = &app.workspace.drafts.fields["unit:unit-1:outlet_pressure_pa"]
    else {
        panic!("expected numeric draft")
    };
    assert_eq!(draft.validation, DraftValidationState::Valid);
    assert!(draft.is_dirty);
    assert_eq!(
        app.workspace.document.flowsheet.units[&id]
            .parameters
            .outlet_pressure_pa,
        None
    );
    app.focus_inspector_target(InspectorTarget::Unit(id.clone()));
    app.commit_unit_inspector_draft(
        &id,
        UnitInspectorDraftField::OutletPressurePa,
        timestamp(22),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        app.workspace.document.flowsheet.units[&id]
            .parameters
            .outlet_pressure_pa,
        Some(90_000.0)
    );
}

#[test]
fn inlet_changes_preserve_unparseable_drafts_as_invalid() {
    for raw in ["", "abc", "NaN", "-1", "inf"] {
        let mut app = pressure_draft_app(BuiltinUnitKind::Valve);
        let id = UnitId::new("unit-1");
        app.focus_inspector_target(InspectorTarget::Unit(id.clone()));
        app.update_unit_inspector_draft(&id, UnitInspectorDraftField::OutletPressurePa, raw)
            .unwrap();
        update_inlet_pressure(&mut app, "120000", true);
        let DraftValue::Number(draft) =
            &app.workspace.drafts.fields["unit:unit-1:outlet_pressure_pa"]
        else {
            panic!("expected numeric draft")
        };
        assert_eq!(draft.validation, DraftValidationState::Invalid);
        assert_eq!(draft.current, raw);
    }
}

#[test]
fn mixer_draft_tracks_both_inlets_when_the_limiting_source_changes() {
    let mut app = pressure_draft_app(BuiltinUnitKind::Mixer);
    let id = UnitId::new("unit-1");
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&id)
        .unwrap()
        .ports[0]
        .name = "inlet_a".to_string();
    app.workspace
        .document
        .flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-second",
            "Second feed",
            330.0,
            100_000.0,
            1.0,
            Default::default(),
        ))
        .unwrap();
    app.workspace
        .document
        .flowsheet
        .units
        .get_mut(&id)
        .unwrap()
        .ports
        .push(UnitPort::new(
            "inlet_b",
            PortDirection::Inlet,
            PortKind::Material,
            Some("stream-second".into()),
        ));
    app.focus_inspector_target(InspectorTarget::Unit(id.clone()));
    app.update_unit_inspector_draft(&id, UnitInspectorDraftField::OutletPressurePa, "110000")
        .unwrap();
    for (stream, pressure, validation) in [
        ("stream-feed", "130000", DraftValidationState::Invalid),
        ("stream-second", "120000", DraftValidationState::Valid),
        ("stream-feed", "105000", DraftValidationState::Invalid),
        ("stream-second", "160000", DraftValidationState::Invalid),
        ("stream-feed", "110000", DraftValidationState::Valid),
    ] {
        let stream_id = StreamId::new(stream);
        let revision = app.workspace.document.revision;
        app.focus_inspector_target(InspectorTarget::Stream(stream_id.clone()));
        app.update_stream_inspector_draft(
            &stream_id,
            StreamInspectorDraftField::PressurePa,
            pressure,
        )
        .unwrap();
        app.commit_stream_inspector_draft(
            &stream_id,
            StreamInspectorDraftField::PressurePa,
            timestamp(20 + revision),
        )
        .unwrap()
        .unwrap();
        let DraftValue::Number(draft) =
            &app.workspace.drafts.fields["unit:unit-1:outlet_pressure_pa"]
        else {
            panic!("expected pressure draft")
        };
        assert_eq!(draft.validation, validation, "after {stream} = {pressure}");
        assert_eq!(draft.current, "110000");
        assert_eq!(app.workspace.document.revision, revision + 1);
        assert_eq!(
            app.workspace.document.flowsheet.units[&id]
                .parameters
                .outlet_pressure_pa,
            Some(90_000.0)
        );
    }
    app.focus_inspector_target(InspectorTarget::Unit(id.clone()));
    app.commit_unit_inspector_draft(
        &id,
        UnitInspectorDraftField::OutletPressurePa,
        timestamp(30),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        app.workspace.document.flowsheet.units[&id]
            .parameters
            .outlet_pressure_pa,
        Some(110_000.0)
    );
}
