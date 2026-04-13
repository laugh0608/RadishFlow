use std::collections::BTreeMap;

use rf_model::{Flowsheet, MaterialStreamState, UnitPort};
use rf_types::{PortDirection, PortKind, StreamId, UnitId};
use rf_ui::{
    AppState, CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding, CanvasSuggestion,
    CanvasSuggestionAcceptance, CanvasSuggestionId, GhostElement, GhostElementKind,
    StreamVisualKind, StreamVisualState, SuggestionSource,
};

const FLASH_DRUM_KIND: &str = "flash_drum";
const FLASH_DRUM_INLET_PORT: &str = "inlet";
const FLASH_DRUM_LIQUID_PORT: &str = "liquid";
const FLASH_DRUM_VAPOR_PORT: &str = "vapor";

#[derive(Debug, Clone, Default)]
struct MaterialStreamEndpoints {
    source: Option<(UnitId, String)>,
    sinks: Vec<(UnitId, String)>,
}

pub fn generate_local_canvas_suggestions(app_state: &AppState) -> Vec<CanvasSuggestion> {
    generate_local_canvas_suggestions_for_flowsheet(&app_state.workspace.document.flowsheet)
}

fn generate_local_canvas_suggestions_for_flowsheet(flowsheet: &Flowsheet) -> Vec<CanvasSuggestion> {
    let endpoints = material_stream_endpoints(flowsheet);
    let source_only_streams: Vec<_> = endpoints
        .iter()
        .filter_map(|(stream_id, endpoint)| {
            if endpoint.source.is_some() && endpoint.sinks.is_empty() {
                Some(stream_id.clone())
            } else {
                None
            }
        })
        .collect();
    let mut suggestions = Vec::new();

    for unit in flowsheet.units.values() {
        if unit.kind != FLASH_DRUM_KIND {
            continue;
        }

        let inlet = unit
            .ports
            .iter()
            .find(|port| port.name == FLASH_DRUM_INLET_PORT);
        if let Some(inlet) = inlet.filter(|port| port.stream_id.is_none()) {
            if source_only_streams.len() == 1 {
                let stream_id = source_only_streams[0].clone();
                if let Some((source_unit_id, source_port)) = endpoints
                    .get(&stream_id)
                    .and_then(|endpoint| endpoint.source.clone())
                {
                    suggestions.push(
                        CanvasSuggestion::new(
                            CanvasSuggestionId::new(format!(
                                "local.flash_drum.connect_inlet.{}.{}",
                                unit.id, stream_id
                            )),
                            SuggestionSource::LocalRules,
                            0.97,
                            GhostElement {
                                kind: GhostElementKind::Connection,
                                target_unit_id: unit.id.clone(),
                                visual_kind: StreamVisualKind::Material,
                                visual_state: StreamVisualState::Suggested,
                            },
                            format!(
                                "Connect stream `{}` to flash drum inlet `{}`",
                                stream_id, inlet.name
                            ),
                        )
                        .with_acceptance(
                            CanvasSuggestionAcceptance::MaterialConnection(
                                CanvasSuggestedMaterialConnection {
                                    stream: CanvasSuggestedStreamBinding::Existing { stream_id },
                                    source_unit_id,
                                    source_port,
                                    sink_unit_id: Some(unit.id.clone()),
                                    sink_port: Some(inlet.name.clone()),
                                },
                            ),
                        ),
                    );
                }
            }
        }

        suggestions.extend(build_missing_flash_drum_outlet_suggestions(
            flowsheet,
            unit.id.clone(),
            &unit.name,
            &unit.ports,
        ));
    }

    suggestions
}

fn build_missing_flash_drum_outlet_suggestions(
    flowsheet: &Flowsheet,
    unit_id: UnitId,
    unit_name: &str,
    ports: &[UnitPort],
) -> Vec<CanvasSuggestion> {
    let mut suggestions = Vec::new();
    for (port_name, confidence, display_name) in [
        (FLASH_DRUM_LIQUID_PORT, 0.93, "Liquid Outlet"),
        (FLASH_DRUM_VAPOR_PORT, 0.92, "Vapor Outlet"),
    ] {
        let Some(port) = ports.iter().find(|candidate| candidate.name == port_name) else {
            continue;
        };
        if port.stream_id.is_some() {
            continue;
        }

        let stream_id = unique_stream_id(flowsheet, &unit_id, port_name);
        let stream_name = format!("{unit_name} {display_name}");
        suggestions.push(
            CanvasSuggestion::new(
                CanvasSuggestionId::new(format!(
                    "local.flash_drum.create_outlet.{}.{}",
                    unit_id, port_name
                )),
                SuggestionSource::LocalRules,
                confidence,
                GhostElement {
                    kind: GhostElementKind::Connection,
                    target_unit_id: unit_id.clone(),
                    visual_kind: StreamVisualKind::Material,
                    visual_state: StreamVisualState::Suggested,
                },
                format!(
                    "Create terminal stream `{}` for flash drum outlet `{}`",
                    stream_name, port_name
                ),
            )
            .with_acceptance(CanvasSuggestionAcceptance::MaterialConnection(
                CanvasSuggestedMaterialConnection {
                    stream: CanvasSuggestedStreamBinding::Create {
                        stream: MaterialStreamState::new(stream_id, stream_name),
                    },
                    source_unit_id: unit_id.clone(),
                    source_port: port_name.to_string(),
                    sink_unit_id: None,
                    sink_port: None,
                },
            )),
        );
    }

    suggestions
}

fn material_stream_endpoints(flowsheet: &Flowsheet) -> BTreeMap<StreamId, MaterialStreamEndpoints> {
    let mut endpoints = BTreeMap::<StreamId, MaterialStreamEndpoints>::new();
    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != PortKind::Material {
                continue;
            }
            let Some(stream_id) = port.stream_id.clone() else {
                continue;
            };
            let entry = endpoints.entry(stream_id).or_default();
            match port.direction {
                PortDirection::Outlet => {
                    if entry.source.is_none() {
                        entry.source = Some((unit.id.clone(), port.name.clone()));
                    }
                }
                PortDirection::Inlet => {
                    entry.sinks.push((unit.id.clone(), port.name.clone()));
                }
            }
        }
    }
    endpoints
}

fn unique_stream_id(flowsheet: &Flowsheet, unit_id: &UnitId, port_name: &str) -> StreamId {
    let base = format!("stream-{}-{}", unit_id.as_str(), port_name);
    if !flowsheet.streams.contains_key(&StreamId::new(base.clone())) {
        return StreamId::new(base);
    }

    let mut next_suffix = 2_u32;
    loop {
        let candidate = format!("{}-{}", base, next_suffix);
        let stream_id = StreamId::new(candidate);
        if !flowsheet.streams.contains_key(&stream_id) {
            return stream_id;
        }
        next_suffix += 1;
    }
}

#[cfg(test)]
mod tests {
    use rf_model::{Component, Flowsheet, UnitNode, UnitPort};

    use super::generate_local_canvas_suggestions_for_flowsheet;

    fn sample_flowsheet() -> Flowsheet {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_component(Component::new("component-a", "Component A"))
            .expect("expected component-a");
        flowsheet
            .insert_component(Component::new("component-b", "Component B"))
            .expect("expected component-b");
        flowsheet
            .insert_stream(rf_model::MaterialStreamState::new("stream-feed", "Feed"))
            .expect("expected feed stream");
        flowsheet
            .insert_unit(UnitNode::new(
                "feed-1",
                "Feed",
                "feed",
                vec![UnitPort::new(
                    "outlet",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    Some("stream-feed".into()),
                )],
            ))
            .expect("expected feed unit");
        flowsheet
            .insert_unit(UnitNode::new(
                "flash-1",
                "Flash Drum",
                "flash_drum",
                vec![
                    UnitPort::new(
                        "inlet",
                        rf_types::PortDirection::Inlet,
                        rf_types::PortKind::Material,
                        None,
                    ),
                    UnitPort::new(
                        "liquid",
                        rf_types::PortDirection::Outlet,
                        rf_types::PortKind::Material,
                        None,
                    ),
                    UnitPort::new(
                        "vapor",
                        rf_types::PortDirection::Outlet,
                        rf_types::PortKind::Material,
                        None,
                    ),
                ],
            ))
            .expect("expected flash unit");

        flowsheet
    }

    #[test]
    fn local_rules_generate_flash_drum_connection_and_outlet_suggestions() {
        let suggestions = generate_local_canvas_suggestions_for_flowsheet(&sample_flowsheet());

        assert_eq!(suggestions.len(), 3);
        assert_eq!(
            suggestions[0].id.as_str(),
            "local.flash_drum.connect_inlet.flash-1.stream-feed"
        );
        assert_eq!(
            suggestions[1].id.as_str(),
            "local.flash_drum.create_outlet.flash-1.liquid"
        );
        assert_eq!(
            suggestions[2].id.as_str(),
            "local.flash_drum.create_outlet.flash-1.vapor"
        );
    }
}
