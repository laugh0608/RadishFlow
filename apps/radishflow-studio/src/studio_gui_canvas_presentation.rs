use std::collections::BTreeMap;

use crate::StudioGuiCanvasState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasUnitBlockViewModel {
    pub unit_id: String,
    pub name: String,
    pub kind: String,
    pub ports: Vec<StudioGuiCanvasUnitPortViewModel>,
    pub status_badges: Vec<StudioGuiCanvasStatusBadgeViewModel>,
    pub port_count: usize,
    pub connected_port_count: usize,
    pub command_id: String,
    pub action_label: String,
    pub hover_text: String,
    pub layout_slot: usize,
    pub is_active_inspector_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasUnitPortViewModel {
    pub name: String,
    pub direction_label: &'static str,
    pub kind_label: &'static str,
    pub stream_id: Option<String>,
    pub stream_label: Option<String>,
    pub stream_command_id: Option<String>,
    pub binding_label: String,
    pub hover_text: String,
    pub is_connected: bool,
    pub side_index: usize,
    pub side_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasStreamLineEndpointViewModel {
    pub unit_id: String,
    pub port_name: String,
    pub layout_slot: usize,
    pub port_side_index: usize,
    pub port_side_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasStreamLineViewModel {
    pub line_id: String,
    pub stream_id: String,
    pub name: String,
    pub source: Option<StudioGuiCanvasStreamLineEndpointViewModel>,
    pub sink: Option<StudioGuiCanvasStreamLineEndpointViewModel>,
    pub status_badges: Vec<StudioGuiCanvasStatusBadgeViewModel>,
    pub command_id: String,
    pub action_label: String,
    pub hover_text: String,
    pub is_active_inspector_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasSelectionViewModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub summary: String,
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasFocusCalloutViewModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub title: String,
    pub detail: String,
    pub command_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasObjectListItemViewModel {
    pub kind_label: &'static str,
    pub target_id: String,
    pub label: String,
    pub detail: String,
    pub command_id: String,
    pub related_stream_ids: Vec<String>,
    pub status_badges: Vec<StudioGuiCanvasStatusBadgeViewModel>,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasObjectListViewModel {
    pub unit_count: usize,
    pub stream_count: usize,
    pub attention_count: usize,
    pub filter_options: Vec<StudioGuiCanvasObjectListFilterOptionViewModel>,
    pub items: Vec<StudioGuiCanvasObjectListItemViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasObjectListFilterOptionViewModel {
    pub filter_id: &'static str,
    pub label: &'static str,
    pub count: usize,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasSuggestionViewModel {
    pub id: String,
    pub source_label: &'static str,
    pub status_label: &'static str,
    pub confidence: f32,
    pub target_unit_id: String,
    pub reason: String,
    pub is_focused: bool,
    pub tab_accept_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasPendingEditViewModel {
    pub intent_label: &'static str,
    pub summary: String,
    pub cancel_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasRunStatusViewModel {
    pub status_label: &'static str,
    pub pending_reason_label: Option<&'static str>,
    pub latest_snapshot_id: Option<String>,
    pub summary: Option<String>,
    pub diagnostic_count: usize,
    pub attention_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasStatusBadgeViewModel {
    pub severity_label: &'static str,
    pub short_label: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasViewModel {
    pub run_status: Option<StudioGuiCanvasRunStatusViewModel>,
    pub pending_edit: Option<StudioGuiCanvasPendingEditViewModel>,
    pub focused_suggestion_id: Option<String>,
    pub current_selection: Option<StudioGuiCanvasSelectionViewModel>,
    pub focus_callout: Option<StudioGuiCanvasFocusCalloutViewModel>,
    pub object_list: StudioGuiCanvasObjectListViewModel,
    pub unit_count: usize,
    pub stream_line_count: usize,
    pub suggestion_count: usize,
    pub unit_blocks: Vec<StudioGuiCanvasUnitBlockViewModel>,
    pub stream_lines: Vec<StudioGuiCanvasStreamLineViewModel>,
    pub suggestions: Vec<StudioGuiCanvasSuggestionViewModel>,
}

impl StudioGuiCanvasViewModel {
    pub fn from_state(state: &StudioGuiCanvasState) -> Self {
        let focused_suggestion_id = state
            .focused_suggestion_id
            .as_ref()
            .map(|id| id.as_str().to_string());
        let run_status = state.run_status.map(|status| {
            let attention_count = state
                .diagnostics
                .iter()
                .filter(|diagnostic| canvas_diagnostic_requires_attention(diagnostic.severity))
                .count();
            StudioGuiCanvasRunStatusViewModel {
                status_label: run_status_label(status),
                pending_reason_label: state.pending_reason.map(solve_pending_reason_label),
                latest_snapshot_id: state.latest_snapshot_id.clone(),
                summary: state.latest_snapshot_summary.clone(),
                diagnostic_count: state.diagnostics.len(),
                attention_count,
            }
        });
        let pending_edit = state.pending_edit.as_ref().map(|intent| match intent {
            rf_ui::CanvasEditIntent::PlaceUnit { unit_kind } => {
                StudioGuiCanvasPendingEditViewModel {
                    intent_label: "place_unit",
                    summary: format!("place unit kind={unit_kind}"),
                    cancel_enabled: true,
                }
            }
        });
        let stream_names = state
            .streams
            .iter()
            .map(|stream| (stream.stream_id.as_str().to_string(), stream.name.clone()))
            .collect::<BTreeMap<_, _>>();
        let unit_blocks = state
            .units
            .iter()
            .enumerate()
            .map(|(layout_slot, unit)| {
                let target = rf_ui::InspectorTarget::Unit(unit.unit_id.clone());
                let command_id = crate::inspector_target_command_id(&target);
                let ports = canvas_unit_material_ports(unit, &stream_names);
                let status_badges =
                    canvas_unit_status_badges(unit.unit_id.as_str(), &state.diagnostics);
                StudioGuiCanvasUnitBlockViewModel {
                    unit_id: unit.unit_id.as_str().to_string(),
                    name: unit.name.clone(),
                    kind: unit.kind.clone(),
                    ports,
                    status_badges,
                    port_count: unit.port_count,
                    connected_port_count: unit.connected_port_count,
                    action_label: format!("Unit {}", unit.unit_id.as_str()),
                    hover_text: format!(
                        "Focus unit inspector for `{}` ({})",
                        unit.unit_id.as_str(),
                        unit.kind
                    ),
                    command_id,
                    layout_slot,
                    is_active_inspector_target: unit.is_active_inspector_target,
                }
            })
            .collect::<Vec<_>>();
        let unit_port_layouts = unit_blocks
            .iter()
            .flat_map(|unit| {
                unit.ports.iter().map(|port| {
                    (
                        (unit.unit_id.clone(), port.name.clone()),
                        (unit.layout_slot, port.side_index, port.side_count),
                    )
                })
            })
            .collect::<BTreeMap<_, _>>();
        let stream_lines = state
            .streams
            .iter()
            .enumerate()
            .filter_map(|(line_index, stream)| {
                let source = stream.source.as_ref().and_then(|endpoint| {
                    unit_port_layouts
                        .get(&(
                            endpoint.unit_id.as_str().to_string(),
                            endpoint.port_name.clone(),
                        ))
                        .copied()
                        .map(|(layout_slot, port_side_index, port_side_count)| {
                            StudioGuiCanvasStreamLineEndpointViewModel {
                                unit_id: endpoint.unit_id.as_str().to_string(),
                                port_name: endpoint.port_name.clone(),
                                layout_slot,
                                port_side_index,
                                port_side_count,
                            }
                        })
                });
                let sink = stream.sink.as_ref().and_then(|endpoint| {
                    unit_port_layouts
                        .get(&(
                            endpoint.unit_id.as_str().to_string(),
                            endpoint.port_name.clone(),
                        ))
                        .copied()
                        .map(|(layout_slot, port_side_index, port_side_count)| {
                            StudioGuiCanvasStreamLineEndpointViewModel {
                                unit_id: endpoint.unit_id.as_str().to_string(),
                                port_name: endpoint.port_name.clone(),
                                layout_slot,
                                port_side_index,
                                port_side_count,
                            }
                        })
                });
                if source.is_none() && sink.is_none() {
                    return None;
                }

                let target = rf_ui::InspectorTarget::Stream(stream.stream_id.clone());
                let command_id = crate::inspector_target_command_id(&target);
                let status_badges =
                    canvas_stream_status_badges(stream.stream_id.as_str(), &state.diagnostics);
                let source_label = source
                    .as_ref()
                    .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                    .unwrap_or_else(|| "unbound-source".to_string());
                let sink_label = sink
                    .as_ref()
                    .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                    .unwrap_or_else(|| "terminal".to_string());
                Some(StudioGuiCanvasStreamLineViewModel {
                    line_id: format!("{}:{line_index}", stream.stream_id.as_str()),
                    stream_id: stream.stream_id.as_str().to_string(),
                    name: stream.name.clone(),
                    source,
                    sink,
                    status_badges,
                    action_label: format!("Stream {}", stream.stream_id.as_str()),
                    hover_text: format!(
                        "Focus stream inspector for `{}` ({} -> {})",
                        stream.stream_id.as_str(),
                        source_label,
                        sink_label
                    ),
                    command_id,
                    is_active_inspector_target: stream.is_active_inspector_target,
                })
            })
            .collect::<Vec<_>>();
        let current_selection = unit_blocks
            .iter()
            .find(|unit| unit.is_active_inspector_target)
            .map(|unit| StudioGuiCanvasSelectionViewModel {
                kind_label: "Unit",
                target_id: unit.unit_id.clone(),
                summary: format!(
                    "{} ({}) ports {}/{}",
                    unit.name, unit.kind, unit.connected_port_count, unit.port_count
                ),
                command_id: unit.command_id.clone(),
            })
            .or_else(|| {
                stream_lines
                    .iter()
                    .find(|stream| stream.is_active_inspector_target)
                    .map(|stream| {
                        let source = stream
                            .source
                            .as_ref()
                            .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                            .unwrap_or_else(|| "unbound-source".to_string());
                        let sink = stream
                            .sink
                            .as_ref()
                            .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                            .unwrap_or_else(|| "terminal".to_string());
                        StudioGuiCanvasSelectionViewModel {
                            kind_label: "Stream",
                            target_id: stream.stream_id.clone(),
                            summary: format!("{} {} -> {}", stream.name, source, sink),
                            command_id: stream.command_id.clone(),
                        }
                    })
            });
        let focus_callout = canvas_focus_callout(&unit_blocks, &stream_lines);
        let object_list = canvas_object_list(&unit_blocks, &stream_lines);
        let suggestions = state
            .suggestions
            .iter()
            .map(|suggestion| StudioGuiCanvasSuggestionViewModel {
                id: suggestion.id.as_str().to_string(),
                source_label: suggestion_source_label(suggestion.source),
                status_label: suggestion_status_label(suggestion.status),
                confidence: suggestion.confidence,
                target_unit_id: suggestion.ghost.target_unit_id.as_str().to_string(),
                reason: suggestion.reason.clone(),
                is_focused: state.focused_suggestion_id.as_ref() == Some(&suggestion.id),
                tab_accept_enabled: suggestion.can_accept_with_tab(),
            })
            .collect::<Vec<_>>();

        Self {
            run_status,
            pending_edit,
            focused_suggestion_id,
            current_selection,
            focus_callout,
            object_list,
            unit_count: unit_blocks.len(),
            stream_line_count: stream_lines.len(),
            suggestion_count: suggestions.len(),
            unit_blocks,
            stream_lines,
            suggestions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCanvasTextView {
    pub title: &'static str,
    pub lines: Vec<String>,
}

impl StudioGuiCanvasTextView {
    pub fn from_view_model(view: &StudioGuiCanvasViewModel) -> Self {
        let mut lines = vec![
            format!(
                "run status: {}",
                view.run_status
                    .as_ref()
                    .map(|status| format!(
                        "{} pending={} snapshot={} diagnostics={} attention={} summary={}",
                        status.status_label,
                        status.pending_reason_label.unwrap_or("none"),
                        status.latest_snapshot_id.as_deref().unwrap_or("none"),
                        status.diagnostic_count,
                        status.attention_count,
                        status.summary.as_deref().unwrap_or("none")
                    ))
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!(
                "pending edit: {}",
                view.pending_edit
                    .as_ref()
                    .map(|pending| format!(
                        "{} summary={} cancel={}",
                        pending.intent_label,
                        pending.summary,
                        enabled_label(pending.cancel_enabled)
                    ))
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!(
                "focused suggestion: {}",
                view.focused_suggestion_id.as_deref().unwrap_or("none")
            ),
            format!(
                "current selection: {}",
                view.current_selection
                    .as_ref()
                    .map(|selection| format!(
                        "{} {} summary={} command={}",
                        selection.kind_label,
                        selection.target_id,
                        selection.summary,
                        selection.command_id
                    ))
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!(
                "focus callout: {}",
                view.focus_callout
                    .as_ref()
                    .map(|callout| format!(
                        "{} {} title={} detail={} command={}",
                        callout.kind_label,
                        callout.target_id,
                        callout.title,
                        callout.detail,
                        callout.command_id
                    ))
                    .unwrap_or_else(|| "none".to_string())
            ),
            format!("unit count: {}", view.unit_count),
            format!("stream line count: {}", view.stream_line_count),
            format!(
                "object list count: units={} streams={} attention={} items={}",
                view.object_list.unit_count,
                view.object_list.stream_count,
                view.object_list.attention_count,
                view.object_list.items.len()
            ),
            format!("suggestion count: {}", view.suggestion_count),
        ];

        lines.extend(view.unit_blocks.iter().map(|unit| {
            let focus_marker = if unit.is_active_inspector_target {
                "*"
            } else {
                "-"
            };
            format!(
                "{focus_marker} unit {} kind={} ports={}/{} badges={} command={}",
                unit.unit_id,
                unit.kind,
                unit.connected_port_count,
                unit.port_count,
                canvas_badges_text(&unit.status_badges),
                unit.command_id
            )
        }));

        lines.extend(view.unit_blocks.iter().flat_map(|unit| {
            unit.ports.iter().map(move |port| {
                let stream = port.stream_id.as_deref().unwrap_or("unbound");
                format!(
                    "  port {}:{} direction={} kind={} stream={} binding={} slot={}/{}",
                    unit.unit_id,
                    port.name,
                    port.direction_label,
                    port.kind_label,
                    stream,
                    port.binding_label,
                    port.side_index + 1,
                    port.side_count
                )
            })
        }));

        lines.extend(view.stream_lines.iter().map(|stream| {
            let focus_marker = if stream.is_active_inspector_target {
                "*"
            } else {
                "-"
            };
            let source = stream
                .source
                .as_ref()
                .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                .unwrap_or_else(|| "unbound-source".to_string());
            let sink = stream
                .sink
                .as_ref()
                .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                .unwrap_or_else(|| "terminal".to_string());
            format!(
                "{focus_marker} stream {} {} -> {} badges={} command={}",
                stream.stream_id,
                source,
                sink,
                canvas_badges_text(&stream.status_badges),
                stream.command_id
            )
        }));

        lines.extend(view.suggestions.iter().map(|suggestion| {
            let focus_marker = if suggestion.is_focused { "*" } else { "-" };
            format!(
                "{focus_marker} {} [{}] source={} confidence={:.2} target={} tab_accept={} reason={}",
                suggestion.id,
                suggestion.status_label,
                suggestion.source_label,
                suggestion.confidence,
                suggestion.target_unit_id,
                enabled_label(suggestion.tab_accept_enabled),
                suggestion.reason
            )
        }));

        Self {
            title: "Canvas suggestions",
            lines,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiCanvasPresentation {
    pub view: StudioGuiCanvasViewModel,
    pub text: StudioGuiCanvasTextView,
}

impl StudioGuiCanvasPresentation {
    pub fn from_state(state: &StudioGuiCanvasState) -> Self {
        let view = StudioGuiCanvasViewModel::from_state(state);
        let text = StudioGuiCanvasTextView::from_view_model(&view);
        Self { view, text }
    }
}

fn canvas_unit_material_ports(
    unit: &crate::StudioGuiCanvasUnitState,
    stream_names: &BTreeMap<String, String>,
) -> Vec<StudioGuiCanvasUnitPortViewModel> {
    let inlet_count = unit
        .ports
        .iter()
        .filter(|port| {
            port.kind == rf_types::PortKind::Material
                && port.direction == rf_types::PortDirection::Inlet
        })
        .count();
    let outlet_count = unit
        .ports
        .iter()
        .filter(|port| {
            port.kind == rf_types::PortKind::Material
                && port.direction == rf_types::PortDirection::Outlet
        })
        .count();
    let mut inlet_index = 0;
    let mut outlet_index = 0;

    unit.ports
        .iter()
        .filter(|port| port.kind == rf_types::PortKind::Material)
        .map(|port| {
            let (side_index, side_count) = match port.direction {
                rf_types::PortDirection::Inlet => {
                    let index = inlet_index;
                    inlet_index += 1;
                    (index, inlet_count)
                }
                rf_types::PortDirection::Outlet => {
                    let index = outlet_index;
                    outlet_index += 1;
                    (index, outlet_count)
                }
            };
            let stream_id = port
                .stream_id
                .as_ref()
                .map(|stream_id| stream_id.as_str().to_string());
            let stream_label = stream_id
                .as_ref()
                .map(|stream_id| canvas_port_stream_label(stream_id, stream_names));
            let stream_command_id = port.stream_id.as_ref().map(|stream_id| {
                crate::inspector_target_command_id(&rf_ui::InspectorTarget::Stream(
                    stream_id.clone(),
                ))
            });
            let binding_label = stream_label
                .clone()
                .unwrap_or_else(|| "unbound".to_string());
            let hover_text = canvas_port_hover_text(unit, port, stream_label.as_deref());
            StudioGuiCanvasUnitPortViewModel {
                name: port.name.clone(),
                direction_label: port.direction.as_str(),
                kind_label: port.kind.as_str(),
                stream_id,
                stream_label,
                stream_command_id,
                binding_label,
                hover_text,
                is_connected: port.stream_id.is_some(),
                side_index,
                side_count,
            }
        })
        .collect()
}

fn canvas_port_stream_label(stream_id: &str, stream_names: &BTreeMap<String, String>) -> String {
    match stream_names.get(stream_id) {
        Some(name) if name != stream_id => format!("{name} ({stream_id})"),
        _ => stream_id.to_string(),
    }
}

fn canvas_port_hover_text(
    unit: &crate::StudioGuiCanvasUnitState,
    port: &crate::StudioGuiCanvasUnitPortState,
    stream_label: Option<&str>,
) -> String {
    let port_label = format!(
        "{}:{} {} {}",
        unit.unit_id.as_str(),
        port.name,
        port.direction.as_str(),
        port.kind.as_str()
    );
    match stream_label {
        Some(stream_label) => {
            format!("{port_label}\nbound stream: {stream_label}\nRead-only marker")
        }
        None => format!("{port_label}\nbound stream: unbound\nRead-only marker"),
    }
}

fn canvas_focus_callout(
    units: &[StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[StudioGuiCanvasStreamLineViewModel],
) -> Option<StudioGuiCanvasFocusCalloutViewModel> {
    units
        .iter()
        .find(|unit| unit.is_active_inspector_target)
        .map(|unit| StudioGuiCanvasFocusCalloutViewModel {
            kind_label: "Unit",
            target_id: unit.unit_id.clone(),
            title: unit.name.clone(),
            detail: format!(
                "{} | ports {}/{}",
                unit.kind, unit.connected_port_count, unit.port_count
            ),
            command_id: unit.command_id.clone(),
        })
        .or_else(|| {
            stream_lines
                .iter()
                .find(|stream| stream.is_active_inspector_target)
                .map(|stream| {
                    let source = stream
                        .source
                        .as_ref()
                        .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                        .unwrap_or_else(|| "unbound-source".to_string());
                    let sink = stream
                        .sink
                        .as_ref()
                        .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                        .unwrap_or_else(|| "terminal".to_string());
                    StudioGuiCanvasFocusCalloutViewModel {
                        kind_label: "Stream",
                        target_id: stream.stream_id.clone(),
                        title: stream.name.clone(),
                        detail: format!("{source} -> {sink}"),
                        command_id: stream.command_id.clone(),
                    }
                })
        })
}

fn canvas_object_list(
    units: &[StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[StudioGuiCanvasStreamLineViewModel],
) -> StudioGuiCanvasObjectListViewModel {
    let mut items = units
        .iter()
        .map(|unit| StudioGuiCanvasObjectListItemViewModel {
            kind_label: "Unit",
            target_id: unit.unit_id.clone(),
            label: unit.name.clone(),
            detail: format!(
                "{} | ports {}/{}",
                unit.kind, unit.connected_port_count, unit.port_count
            ),
            command_id: unit.command_id.clone(),
            related_stream_ids: unit
                .ports
                .iter()
                .filter_map(|port| port.stream_id.clone())
                .collect(),
            status_badges: unit.status_badges.clone(),
            is_active: unit.is_active_inspector_target,
        })
        .collect::<Vec<_>>();

    let mut stream_items = BTreeMap::<String, StudioGuiCanvasObjectListItemViewModel>::new();
    for stream in stream_lines {
        stream_items
            .entry(stream.stream_id.clone())
            .or_insert_with(|| {
                let source = stream
                    .source
                    .as_ref()
                    .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                    .unwrap_or_else(|| "unbound-source".to_string());
                let sink = stream
                    .sink
                    .as_ref()
                    .map(|endpoint| format!("{}:{}", endpoint.unit_id, endpoint.port_name))
                    .unwrap_or_else(|| "terminal".to_string());
                StudioGuiCanvasObjectListItemViewModel {
                    kind_label: "Stream",
                    target_id: stream.stream_id.clone(),
                    label: stream.name.clone(),
                    detail: format!("{source} -> {sink}"),
                    command_id: stream.command_id.clone(),
                    related_stream_ids: vec![stream.stream_id.clone()],
                    status_badges: stream.status_badges.clone(),
                    is_active: stream.is_active_inspector_target,
                }
            });
    }
    let stream_count = stream_items.len();
    items.extend(stream_items.into_values());
    items.sort_by_key(|item| {
        (
            item.status_badges.is_empty(),
            object_list_kind_order(item.kind_label),
            item.label.clone(),
            item.target_id.clone(),
        )
    });
    let attention_count = items
        .iter()
        .filter(|item| !item.status_badges.is_empty())
        .count();

    StudioGuiCanvasObjectListViewModel {
        unit_count: units.len(),
        stream_count,
        attention_count,
        filter_options: vec![
            StudioGuiCanvasObjectListFilterOptionViewModel {
                filter_id: "all",
                label: "All",
                count: items.len(),
                enabled: !items.is_empty(),
            },
            StudioGuiCanvasObjectListFilterOptionViewModel {
                filter_id: "attention",
                label: "Attention",
                count: attention_count,
                enabled: attention_count > 0,
            },
            StudioGuiCanvasObjectListFilterOptionViewModel {
                filter_id: "units",
                label: "Units",
                count: units.len(),
                enabled: !units.is_empty(),
            },
            StudioGuiCanvasObjectListFilterOptionViewModel {
                filter_id: "streams",
                label: "Streams",
                count: stream_count,
                enabled: stream_count > 0,
            },
        ],
        items,
    }
}

fn object_list_kind_order(kind_label: &str) -> u8 {
    match kind_label {
        "Unit" => 0,
        "Stream" => 1,
        _ => 2,
    }
}

fn canvas_unit_status_badges(
    unit_id: &str,
    diagnostics: &[crate::StudioGuiCanvasDiagnosticState],
) -> Vec<StudioGuiCanvasStatusBadgeViewModel> {
    canvas_status_badges(diagnostics.iter().filter(|diagnostic| {
        diagnostic
            .related_unit_ids
            .iter()
            .any(|related_unit_id| related_unit_id.as_str() == unit_id)
            || diagnostic
                .related_port_targets
                .iter()
                .any(|target| target.unit_id.as_str() == unit_id)
    }))
}

fn canvas_stream_status_badges(
    stream_id: &str,
    diagnostics: &[crate::StudioGuiCanvasDiagnosticState],
) -> Vec<StudioGuiCanvasStatusBadgeViewModel> {
    canvas_status_badges(diagnostics.iter().filter(|diagnostic| {
        diagnostic
            .related_stream_ids
            .iter()
            .any(|related_stream_id| related_stream_id.as_str() == stream_id)
    }))
}

fn canvas_status_badges<'a>(
    diagnostics: impl Iterator<Item = &'a crate::StudioGuiCanvasDiagnosticState>,
) -> Vec<StudioGuiCanvasStatusBadgeViewModel> {
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut error_detail = None;
    let mut warning_detail = None;

    for diagnostic in diagnostics {
        match diagnostic.severity {
            rf_ui::DiagnosticSeverity::Error => {
                error_count += 1;
                error_detail.get_or_insert_with(|| canvas_diagnostic_detail(diagnostic));
            }
            rf_ui::DiagnosticSeverity::Warning => {
                warning_count += 1;
                warning_detail.get_or_insert_with(|| canvas_diagnostic_detail(diagnostic));
            }
            rf_ui::DiagnosticSeverity::Info => {}
        }
    }

    let mut badges = Vec::new();
    if error_count > 0 {
        badges.push(StudioGuiCanvasStatusBadgeViewModel {
            severity_label: "Error",
            short_label: format!("E{error_count}"),
            detail: error_detail.unwrap_or_else(|| "Error".to_string()),
        });
    }
    if warning_count > 0 {
        badges.push(StudioGuiCanvasStatusBadgeViewModel {
            severity_label: "Warning",
            short_label: format!("W{warning_count}"),
            detail: warning_detail.unwrap_or_else(|| "Warning".to_string()),
        });
    }
    badges
}

fn canvas_diagnostic_detail(diagnostic: &crate::StudioGuiCanvasDiagnosticState) -> String {
    format!("{}: {}", diagnostic.code, diagnostic.message)
}

fn canvas_diagnostic_requires_attention(severity: rf_ui::DiagnosticSeverity) -> bool {
    matches!(
        severity,
        rf_ui::DiagnosticSeverity::Warning | rf_ui::DiagnosticSeverity::Error
    )
}

fn canvas_badges_text(badges: &[StudioGuiCanvasStatusBadgeViewModel]) -> String {
    if badges.is_empty() {
        return "none".to_string();
    }

    badges
        .iter()
        .map(|badge| badge.short_label.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

impl StudioGuiCanvasState {
    pub fn view(&self) -> StudioGuiCanvasViewModel {
        StudioGuiCanvasViewModel::from_state(self)
    }

    pub fn text(&self) -> StudioGuiCanvasTextView {
        StudioGuiCanvasTextView::from_view_model(&self.view())
    }

    pub fn presentation(&self) -> StudioGuiCanvasPresentation {
        StudioGuiCanvasPresentation::from_state(self)
    }
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled { "yes" } else { "no" }
}

fn run_status_label(status: rf_ui::RunStatus) -> &'static str {
    match status {
        rf_ui::RunStatus::Idle => "Idle",
        rf_ui::RunStatus::Dirty => "Dirty",
        rf_ui::RunStatus::Checking => "Checking",
        rf_ui::RunStatus::Runnable => "Runnable",
        rf_ui::RunStatus::Solving => "Solving",
        rf_ui::RunStatus::Converged => "Converged",
        rf_ui::RunStatus::UnderSpecified => "Under-specified",
        rf_ui::RunStatus::OverSpecified => "Over-specified",
        rf_ui::RunStatus::Unconverged => "Unconverged",
        rf_ui::RunStatus::Error => "Error",
    }
}

fn solve_pending_reason_label(reason: rf_ui::SolvePendingReason) -> &'static str {
    match reason {
        rf_ui::SolvePendingReason::DocumentRevisionAdvanced => "DocumentRevisionAdvanced",
        rf_ui::SolvePendingReason::ModeActivated => "ModeActivated",
        rf_ui::SolvePendingReason::ManualRunRequested => "ManualRunRequested",
        rf_ui::SolvePendingReason::SnapshotMissing => "SnapshotMissing",
    }
}

fn suggestion_source_label(source: rf_ui::SuggestionSource) -> &'static str {
    match source {
        rf_ui::SuggestionSource::LocalRules => "local_rules",
        rf_ui::SuggestionSource::RadishMind => "radish_mind",
    }
}

fn suggestion_status_label(status: rf_ui::SuggestionStatus) -> &'static str {
    match status {
        rf_ui::SuggestionStatus::Proposed => "proposed",
        rf_ui::SuggestionStatus::Focused => "focused",
        rf_ui::SuggestionStatus::Accepted => "accepted",
        rf_ui::SuggestionStatus::Rejected => "rejected",
        rf_ui::SuggestionStatus::Invalidated => "invalidated",
    }
}

#[cfg(test)]
mod tests {
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
        let project =
            include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json")
                .replacen(
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-heated\"",
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-liquid\"",
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                );
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
                        count: 0,
                        enabled: false,
                    },
                    crate::StudioGuiCanvasObjectListFilterOptionViewModel {
                        filter_id: "attention",
                        label: "Attention",
                        count: 0,
                        enabled: false,
                    },
                    crate::StudioGuiCanvasObjectListFilterOptionViewModel {
                        filter_id: "units",
                        label: "Units",
                        count: 0,
                        enabled: false,
                    },
                    crate::StudioGuiCanvasObjectListFilterOptionViewModel {
                        filter_id: "streams",
                        label: "Streams",
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
                "unit count: 0".to_string(),
                "stream line count: 0".to_string(),
                "object list count: units=0 streams=0 attention=0 items=0".to_string(),
                "suggestion count: 0".to_string(),
            ]
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
        assert_eq!(
            presentation.text.lines,
            vec![
                "run status: none".to_string(),
                "pending edit: place_unit summary=place unit kind=Flash Drum cancel=yes"
                    .to_string(),
                "focused suggestion: none".to_string(),
                "current selection: none".to_string(),
                "focus callout: none".to_string(),
                "unit count: 0".to_string(),
                "stream line count: 0".to_string(),
                "object list count: units=0 streams=0 attention=0 items=0".to_string(),
                "suggestion count: 0".to_string(),
            ]
        );
    }

    #[test]
    fn canvas_presentation_maps_attention_diagnostics_to_canvas_objects() {
        let state = crate::StudioGuiCanvasState {
            units: vec![crate::StudioGuiCanvasUnitState {
                unit_id: rf_types::UnitId::new("flash-1"),
                name: "Flash Drum".to_string(),
                kind: "flash_drum".to_string(),
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
                detail: "solver.step.execution: unit failed".to_string(),
            }]
        );
        let stream = presentation
            .view
            .stream_lines
            .iter()
            .find(|stream| stream.stream_id == "stream-feed")
            .expect("expected stream line");
        assert_eq!(stream.status_badges, unit.status_badges);
        assert!(presentation.view.object_list.items.iter().any(|item| {
            item.target_id == "flash-1" && item.status_badges == unit.status_badges
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
                        port.name == "inlet"
                            && port.stream_id.is_none()
                            && port.stream_label.is_none()
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
        assert_eq!(
            presentation.text.lines,
            vec![
                "run status: Idle pending=SnapshotMissing snapshot=none diagnostics=0 attention=0 summary=none".to_string(),
                "pending edit: none".to_string(),
                "focused suggestion: local.flash_drum.connect_inlet.flash-1.stream-heated"
                    .to_string(),
                "current selection: none".to_string(),
                "focus callout: none".to_string(),
                "unit count: 3".to_string(),
                "stream line count: 2".to_string(),
                "object list count: units=3 streams=2 attention=0 items=5".to_string(),
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
                "* local.flash_drum.connect_inlet.flash-1.stream-heated [focused] source=local_rules confidence=0.97 target=flash-1 tab_accept=yes reason=Connect stream `stream-heated` to flash drum inlet `inlet`".to_string(),
                "- local.flash_drum.create_outlet.flash-1.liquid [proposed] source=local_rules confidence=0.93 target=flash-1 tab_accept=yes reason=Create terminal stream `Flash Drum Liquid Outlet` for flash drum outlet `liquid`".to_string(),
                "- local.flash_drum.create_outlet.flash-1.vapor [proposed] source=local_rules confidence=0.92 target=flash-1 tab_accept=yes reason=Create terminal stream `Flash Drum Vapor Outlet` for flash drum outlet `vapor`".to_string(),
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
        assert!(
            focused_unit
                .window
                .canvas
                .widget
                .view()
                .object_list
                .items
                .iter()
                .any(|item| item.kind_label == "Unit"
                    && item.target_id == "flash-1"
                    && item.is_active)
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
}
