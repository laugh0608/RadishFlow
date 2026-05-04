use std::collections::BTreeMap;

use rf_model::{Flowsheet, MaterialStreamState, UnitPort};
use rf_types::{PortDirection, PortKind, StreamId, UnitId};
use rf_ui::{
    AppState, CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding, CanvasSuggestion,
    CanvasSuggestionAcceptance, CanvasSuggestionId, GhostElement, GhostElementKind,
    StreamVisualKind, StreamVisualState, SuggestionSource,
};

const FEED_KIND: &str = "feed";
const FEED_OUTLET_PORT: &str = "outlet";
const HEATER_KIND: &str = "heater";
const COOLER_KIND: &str = "cooler";
const VALVE_KIND: &str = "valve";
const SINGLE_INLET_PORT: &str = "inlet";
const SINGLE_OUTLET_PORT: &str = "outlet";
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
    let connectable_source_only_streams = connectable_source_only_streams(flowsheet, &endpoints);
    let mut suggestions = Vec::new();

    for unit in flowsheet.units.values() {
        if unit.kind == FEED_KIND {
            suggestions.extend(build_missing_feed_outlet_suggestions(
                flowsheet,
                unit.id.clone(),
                &unit.name,
                &unit.ports,
            ));
        }

        if is_single_inlet_outlet_unit_kind(&unit.kind) {
            suggestions.extend(build_single_inlet_outlet_unit_suggestions(
                flowsheet,
                &endpoints,
                &connectable_source_only_streams,
                unit.id.clone(),
                &unit.kind,
                &unit.name,
                &unit.ports,
            ));
        }

        if unit.kind != FLASH_DRUM_KIND {
            continue;
        }

        let inlet = unit
            .ports
            .iter()
            .find(|port| port.name == FLASH_DRUM_INLET_PORT);
        if let Some(inlet) = inlet.filter(|port| port.stream_id.is_none()) {
            if connectable_source_only_streams.len() == 1 {
                let stream_id = connectable_source_only_streams[0].clone();
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

fn build_missing_feed_outlet_suggestions(
    flowsheet: &Flowsheet,
    unit_id: UnitId,
    unit_name: &str,
    ports: &[UnitPort],
) -> Vec<CanvasSuggestion> {
    let Some(port) = ports
        .iter()
        .find(|candidate| candidate.name == FEED_OUTLET_PORT)
    else {
        return Vec::new();
    };
    if port.stream_id.is_some() {
        return Vec::new();
    }

    let stream_id = unique_stream_id(flowsheet, &unit_id, FEED_OUTLET_PORT);
    let stream_name = format!("{unit_name} Outlet");
    vec![
        CanvasSuggestion::new(
            CanvasSuggestionId::new(format!("local.feed.create_outlet.{}", unit_id)),
            SuggestionSource::LocalRules,
            0.96,
            GhostElement {
                kind: GhostElementKind::Connection,
                target_unit_id: unit_id.clone(),
                visual_kind: StreamVisualKind::Material,
                visual_state: StreamVisualState::Suggested,
            },
            format!(
                "Create source stream `{}` for feed outlet `{}`",
                stream_name, port.name
            ),
        )
        .with_acceptance(CanvasSuggestionAcceptance::MaterialConnection(
            CanvasSuggestedMaterialConnection {
                stream: CanvasSuggestedStreamBinding::Create {
                    stream: default_source_stream(flowsheet, stream_id, stream_name),
                },
                source_unit_id: unit_id,
                source_port: port.name.clone(),
                sink_unit_id: None,
                sink_port: None,
            },
        )),
    ]
}

fn build_single_inlet_outlet_unit_suggestions(
    flowsheet: &Flowsheet,
    endpoints: &BTreeMap<StreamId, MaterialStreamEndpoints>,
    connectable_source_only_streams: &[StreamId],
    unit_id: UnitId,
    unit_kind: &str,
    unit_name: &str,
    ports: &[UnitPort],
) -> Vec<CanvasSuggestion> {
    let mut suggestions = Vec::new();

    let inlet = ports
        .iter()
        .find(|candidate| candidate.name == SINGLE_INLET_PORT);
    if let Some(inlet) = inlet.filter(|port| port.stream_id.is_none()) {
        if connectable_source_only_streams.len() == 1 {
            let stream_id = connectable_source_only_streams[0].clone();
            if let Some((source_unit_id, source_port)) = endpoints
                .get(&stream_id)
                .and_then(|endpoint| endpoint.source.clone())
            {
                suggestions.push(
                    CanvasSuggestion::new(
                        CanvasSuggestionId::new(format!(
                            "local.{}.connect_inlet.{}.{}",
                            unit_kind, unit_id, stream_id
                        )),
                        SuggestionSource::LocalRules,
                        0.965,
                        GhostElement {
                            kind: GhostElementKind::Connection,
                            target_unit_id: unit_id.clone(),
                            visual_kind: StreamVisualKind::Material,
                            visual_state: StreamVisualState::Suggested,
                        },
                        format!(
                            "Connect stream `{}` to {} inlet `{}`",
                            stream_id,
                            unit_display_name(unit_kind),
                            inlet.name
                        ),
                    )
                    .with_acceptance(
                        CanvasSuggestionAcceptance::MaterialConnection(
                            CanvasSuggestedMaterialConnection {
                                stream: CanvasSuggestedStreamBinding::Existing { stream_id },
                                source_unit_id,
                                source_port,
                                sink_unit_id: Some(unit_id.clone()),
                                sink_port: Some(inlet.name.clone()),
                            },
                        ),
                    ),
                );
            }
        }
    }

    let outlet = ports
        .iter()
        .find(|candidate| candidate.name == SINGLE_OUTLET_PORT);
    if let Some(outlet) = outlet.filter(|port| port.stream_id.is_none()) {
        let stream_id = unique_stream_id(flowsheet, &unit_id, SINGLE_OUTLET_PORT);
        let stream_name = format!("{} Outlet", unit_name);
        suggestions.push(
            CanvasSuggestion::new(
                CanvasSuggestionId::new(format!("local.{}.create_outlet.{}", unit_kind, unit_id)),
                SuggestionSource::LocalRules,
                0.94,
                GhostElement {
                    kind: GhostElementKind::Connection,
                    target_unit_id: unit_id.clone(),
                    visual_kind: StreamVisualKind::Material,
                    visual_state: StreamVisualState::Suggested,
                },
                format!(
                    "Create source stream `{}` for {} outlet `{}`",
                    stream_name,
                    unit_display_name(unit_kind),
                    outlet.name
                ),
            )
            .with_acceptance(CanvasSuggestionAcceptance::MaterialConnection(
                CanvasSuggestedMaterialConnection {
                    stream: CanvasSuggestedStreamBinding::Create {
                        stream: default_single_inlet_outlet_stream(
                            unit_kind,
                            stream_id,
                            stream_name,
                        ),
                    },
                    source_unit_id: unit_id,
                    source_port: outlet.name.clone(),
                    sink_unit_id: None,
                    sink_port: None,
                },
            )),
        );
    }

    suggestions
}

fn is_single_inlet_outlet_unit_kind(unit_kind: &str) -> bool {
    matches!(unit_kind, HEATER_KIND | COOLER_KIND | VALVE_KIND)
}

fn connectable_source_only_streams(
    flowsheet: &Flowsheet,
    endpoints: &BTreeMap<StreamId, MaterialStreamEndpoints>,
) -> Vec<StreamId> {
    endpoints
        .iter()
        .filter_map(|(stream_id, endpoint)| {
            let (source_unit_id, source_port) = endpoint.source.as_ref()?;
            if !endpoint.sinks.is_empty()
                || is_terminal_flash_drum_outlet(flowsheet, source_unit_id, source_port)
            {
                return None;
            }

            Some(stream_id.clone())
        })
        .collect()
}

fn is_terminal_flash_drum_outlet(
    flowsheet: &Flowsheet,
    source_unit_id: &UnitId,
    source_port: &str,
) -> bool {
    flowsheet.units.get(source_unit_id).is_some_and(|unit| {
        unit.kind == FLASH_DRUM_KIND
            && matches!(source_port, FLASH_DRUM_LIQUID_PORT | FLASH_DRUM_VAPOR_PORT)
    })
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

fn default_source_stream(
    flowsheet: &Flowsheet,
    stream_id: StreamId,
    stream_name: String,
) -> MaterialStreamState {
    let composition = if flowsheet.components.is_empty() {
        Default::default()
    } else {
        let fraction = 1.0 / flowsheet.components.len() as f64;
        flowsheet
            .components
            .keys()
            .cloned()
            .map(|component_id| (component_id, fraction))
            .collect()
    };
    MaterialStreamState::from_tpzf(stream_id, stream_name, 298.15, 101_325.0, 1.0, composition)
}

fn default_single_inlet_outlet_stream(
    unit_kind: &str,
    stream_id: StreamId,
    stream_name: String,
) -> MaterialStreamState {
    let (temperature_k, pressure_pa) = match unit_kind {
        HEATER_KIND => (345.0, 101_325.0),
        COOLER_KIND => (285.0, 101_325.0),
        VALVE_KIND => (298.15, 90_000.0),
        _ => (298.15, 101_325.0),
    };
    MaterialStreamState::from_tpzf(
        stream_id,
        stream_name,
        temperature_k,
        pressure_pa,
        0.0,
        Default::default(),
    )
}

fn unit_display_name(unit_kind: &str) -> &'static str {
    match unit_kind {
        HEATER_KIND => "heater",
        COOLER_KIND => "cooler",
        VALVE_KIND => "valve",
        FLASH_DRUM_KIND => "flash drum",
        FEED_KIND => "feed",
        _ => "unit",
    }
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

    #[test]
    fn local_rules_generate_feed_outlet_suggestion_for_unbound_feed() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_component(Component::new("component-a", "Component A"))
            .expect("expected component-a");
        flowsheet
            .insert_component(Component::new("component-b", "Component B"))
            .expect("expected component-b");
        flowsheet
            .insert_unit(UnitNode::new(
                "feed-1",
                "Feed",
                "feed",
                vec![UnitPort::new(
                    "outlet",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    None,
                )],
            ))
            .expect("expected feed unit");

        let suggestions = generate_local_canvas_suggestions_for_flowsheet(&flowsheet);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].id.as_str(),
            "local.feed.create_outlet.feed-1"
        );
        let Some(rf_ui::CanvasSuggestionAcceptance::MaterialConnection(connection)) =
            suggestions[0].acceptance.as_ref()
        else {
            panic!("expected material connection acceptance");
        };
        assert_eq!(connection.source_unit_id.as_str(), "feed-1");
        assert_eq!(connection.source_port, "outlet");
        assert!(connection.sink_unit_id.is_none());
        let rf_ui::CanvasSuggestedStreamBinding::Create { stream } = &connection.stream else {
            panic!("expected stream creation");
        };
        assert_eq!(stream.id.as_str(), "stream-feed-1-outlet");
        assert_eq!(stream.total_molar_flow_mol_s, 1.0);
        assert_eq!(stream.overall_mole_fractions.len(), 2);
    }

    #[test]
    fn local_rules_generate_single_inlet_outlet_unit_connection_suggestions() {
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
                "heater-1",
                "Heater",
                "heater",
                vec![
                    UnitPort::new(
                        "inlet",
                        rf_types::PortDirection::Inlet,
                        rf_types::PortKind::Material,
                        None,
                    ),
                    UnitPort::new(
                        "outlet",
                        rf_types::PortDirection::Outlet,
                        rf_types::PortKind::Material,
                        None,
                    ),
                ],
            ))
            .expect("expected heater unit");

        let suggestions = generate_local_canvas_suggestions_for_flowsheet(&flowsheet);

        assert_eq!(suggestions.len(), 2);
        assert_eq!(
            suggestions[0].id.as_str(),
            "local.heater.connect_inlet.heater-1.stream-feed"
        );
        assert_eq!(
            suggestions[1].id.as_str(),
            "local.heater.create_outlet.heater-1"
        );
        let Some(rf_ui::CanvasSuggestionAcceptance::MaterialConnection(connection)) =
            suggestions[1].acceptance.as_ref()
        else {
            panic!("expected outlet material connection acceptance");
        };
        let rf_ui::CanvasSuggestedStreamBinding::Create { stream } = &connection.stream else {
            panic!("expected heater outlet stream creation");
        };
        assert_eq!(stream.id.as_str(), "stream-heater-1-outlet");
        assert_eq!(stream.temperature_k, 345.0);
        assert_eq!(stream.pressure_pa, 101_325.0);
    }
}
