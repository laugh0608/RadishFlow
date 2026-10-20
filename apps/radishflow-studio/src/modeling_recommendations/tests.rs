use super::*;
use crate::modeling_specifications::ModelingCheckFailure;
use rf_model::{Component, Flowsheet, MaterialStreamState, UnitNode, UnitPort};
use rf_types::{ComponentId, PortDirection, PortKind, UnitId};
use rf_ui::{
    AppState, BuiltinUnitKind, DocumentMetadata, FlowsheetDocument, InspectorTarget,
    StreamInspectorDraftField,
};
use std::time::UNIX_EPOCH;

mod composition;
mod connections;
mod integration;
mod lifecycle;

fn feed_app() -> AppState {
    let mut flow = Flowsheet::new("Recommendations");
    for id in ["a", "b"] {
        flow.insert_component(Component::new(id, id)).unwrap();
    }
    flow.insert_stream(MaterialStreamState::from_tpzf(
        "source",
        "Feed outlet",
        300.,
        100_000.,
        1.,
        [("a".into(), 0.2), ("b".into(), 0.3)].into(),
    ))
    .unwrap();
    flow.insert_unit(UnitNode::new(
        "feed",
        "Feed",
        "feed",
        vec![port("outlet", PortDirection::Outlet, Some("source"))],
    ))
    .unwrap();
    AppState::new(FlowsheetDocument::new(
        flow,
        DocumentMetadata::new("recommendations", "Recommendations", UNIX_EPOCH),
    ))
}

fn port(name: &str, direction: PortDirection, stream: Option<&str>) -> UnitPort {
    UnitPort::new(name, direction, PortKind::Material, stream.map(Into::into))
}

fn key(
    session: &mut ModelingRecommendationSession,
    app: &AppState,
    rule: RecommendationRule,
) -> RecommendationKey {
    session.refresh(app);
    session
        .entries()
        .iter()
        .find(|e| e.recommendation.key.rule == rule)
        .unwrap()
        .recommendation
        .key
        .clone()
}

fn normalization(
    session: &mut ModelingRecommendationSession,
    app: &AppState,
) -> RecommendationPreview {
    let key = key(session, app, RecommendationRule::NormalizeComposition);
    session.prepare(app, &key).unwrap()
}

fn flow(app: &AppState) -> &Flowsheet {
    &app.workspace.document.flowsheet
}
fn source(app: &AppState) -> &MaterialStreamState {
    &flow(app).streams[&StreamId::new("source")]
}

fn missing_outlet(kind: BuiltinUnitKind) -> AppState {
    let mut app = feed_app();
    if kind == BuiltinUnitKind::Feed {
        app.workspace.document.flowsheet.streams.clear();
        app.workspace
            .document
            .flowsheet
            .units
            .get_mut(&UnitId::new("feed"))
            .unwrap()
            .ports[0]
            .stream_id = None;
    } else {
        let ports = match kind {
            BuiltinUnitKind::Mixer => {
                let mut second = source(&app).clone();
                second.id = "source-b".into();
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
                        vec![port("outlet", PortDirection::Outlet, Some("source-b"))],
                    ))
                    .unwrap();
                vec![
                    port("inlet_a", PortDirection::Inlet, Some("source")),
                    port("inlet_b", PortDirection::Inlet, Some("source-b")),
                    port("outlet", PortDirection::Outlet, None),
                ]
            }
            BuiltinUnitKind::FlashDrum => vec![
                port("inlet", PortDirection::Inlet, Some("source")),
                port("liquid", PortDirection::Outlet, None),
                port("vapor", PortDirection::Outlet, None),
            ],
            _ => vec![
                port("inlet", PortDirection::Inlet, Some("source")),
                port("outlet", PortDirection::Outlet, None),
            ],
        };
        app.workspace
            .document
            .flowsheet
            .insert_unit(UnitNode::new("unit", "Unit", kind.as_str(), ports))
            .unwrap();
    }
    app
}
