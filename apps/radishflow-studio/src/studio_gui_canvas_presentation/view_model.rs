use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::StudioGuiCanvasState;

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
        let place_unit_palette = StudioGuiCanvasPlaceUnitPaletteViewModel::from_pending_edit(
            state.pending_edit.as_ref(),
        );
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
                let ports = canvas_unit_material_ports(unit, &stream_names, &state.diagnostics);
                let status_badges =
                    canvas_unit_status_badges(unit.unit_id.as_str(), &state.diagnostics);
                let attention_summary =
                    canvas_unit_attention_summary(unit.unit_id.as_str(), &state.diagnostics);
                let hover_text = append_canvas_hover_attention(
                    format!(
                        "Focus unit inspector for `{}` ({})",
                        unit.unit_id.as_str(),
                        unit.kind
                    ),
                    attention_summary.as_deref(),
                );
                StudioGuiCanvasUnitBlockViewModel {
                    unit_id: unit.unit_id.as_str().to_string(),
                    name: unit.name.clone(),
                    kind: unit.kind.clone(),
                    ports,
                    status_badges,
                    port_count: unit.port_count,
                    connected_port_count: unit.connected_port_count,
                    action_label: format!("Unit {}", unit.unit_id.as_str()),
                    hover_text,
                    attention_summary,
                    command_id,
                    layout_slot,
                    layout_position: unit.layout_position,
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
                        (
                            unit.layout_slot,
                            unit.layout_position,
                            port.side_index,
                            port.side_count,
                        ),
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
                        .map(
                            |(layout_slot, layout_position, port_side_index, port_side_count)| {
                                StudioGuiCanvasStreamLineEndpointViewModel {
                                    unit_id: endpoint.unit_id.as_str().to_string(),
                                    port_name: endpoint.port_name.clone(),
                                    layout_slot,
                                    layout_position,
                                    port_side_index,
                                    port_side_count,
                                }
                            },
                        )
                });
                let sink = stream.sink.as_ref().and_then(|endpoint| {
                    unit_port_layouts
                        .get(&(
                            endpoint.unit_id.as_str().to_string(),
                            endpoint.port_name.clone(),
                        ))
                        .copied()
                        .map(
                            |(layout_slot, layout_position, port_side_index, port_side_count)| {
                                StudioGuiCanvasStreamLineEndpointViewModel {
                                    unit_id: endpoint.unit_id.as_str().to_string(),
                                    port_name: endpoint.port_name.clone(),
                                    layout_slot,
                                    layout_position,
                                    port_side_index,
                                    port_side_count,
                                }
                            },
                        )
                });
                if source.is_none() && sink.is_none() {
                    return None;
                }

                let target = rf_ui::InspectorTarget::Stream(stream.stream_id.clone());
                let command_id = crate::inspector_target_command_id(&target);
                let status_badges =
                    canvas_stream_status_badges(stream.stream_id.as_str(), &state.diagnostics);
                let attention_summary =
                    canvas_stream_attention_summary(stream.stream_id.as_str(), &state.diagnostics);
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
                    hover_text: append_canvas_hover_attention(
                        format!(
                            "Focus stream inspector for `{}` ({} -> {})",
                            stream.stream_id.as_str(),
                            source_label,
                            sink_label
                        ),
                        attention_summary.as_deref(),
                    ),
                    attention_summary,
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
                layout_source_label: Some(canvas_unit_layout_source_label(unit)),
                layout_detail: Some(canvas_unit_layout_detail(unit)),
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
                            layout_source_label: None,
                            layout_detail: None,
                        }
                    })
            });
        let focus_callout = canvas_focus_callout(&unit_blocks, &stream_lines);
        let object_list = canvas_object_list(&unit_blocks, &stream_lines);
        let viewport = canvas_viewport(
            state.view_mode,
            &unit_blocks,
            &stream_lines,
            current_selection.as_ref(),
            state.focused_suggestion_id.as_ref().map(|id| id.as_str()),
        );
        let legend = canvas_legend(
            run_status.as_ref(),
            pending_edit.as_ref(),
            &object_list,
            &unit_blocks,
            &stream_lines,
        );
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
                explicit_accept_enabled: suggestion.can_accept_explicitly(),
                explicit_accept_command_id: format!(
                    "canvas.accept_suggestion.{}",
                    suggestion.id.as_str()
                ),
            })
            .collect::<Vec<_>>();

        Self {
            run_status,
            pending_edit,
            place_unit_palette,
            focused_suggestion_id,
            current_selection,
            focus_callout,
            viewport,
            object_list,
            legend,
            unit_count: unit_blocks.len(),
            stream_line_count: stream_lines.len(),
            suggestion_count: suggestions.len(),
            unit_blocks,
            stream_lines,
            suggestions,
        }
    }
}

fn canvas_unit_layout_source_label(unit: &StudioGuiCanvasUnitBlockViewModel) -> &'static str {
    if unit.layout_position.is_some() {
        "sidecar position"
    } else {
        "transient grid"
    }
}

fn canvas_unit_layout_detail(unit: &StudioGuiCanvasUnitBlockViewModel) -> String {
    match unit.layout_position {
        Some(position) => format!(
            "layout sidecar position ({:.1}, {:.1})",
            position.x, position.y
        ),
        None => format!(
            "no sidecar position; nudge will pin from unit-slot-{}",
            unit.layout_slot
        ),
    }
}

pub(super) fn normalize_canvas_unit_kind(unit_kind: &str) -> String {
    unit_kind
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_")
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
            format!(
                "viewport: mode={} layout={} units={} streams={} focus={}",
                view.viewport.mode_label,
                view.viewport.layout_label,
                view.viewport.unit_count,
                view.viewport.stream_line_count,
                view.viewport
                    .focus
                    .as_ref()
                    .map(|focus| format!(
                        "{} {} source={} anchor={}",
                        focus.kind_label, focus.target_id, focus.source_label, focus.anchor_label
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
            format!(
                "legend: {} items={}",
                view.legend.title,
                view.legend.items.len()
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
                "{focus_marker} {} [{}] source={} confidence={:.2} target={} tab_accept={} explicit_accept={} reason={}",
                suggestion.id,
                suggestion.status_label,
                suggestion.source_label,
                suggestion.confidence,
                suggestion.target_unit_id,
                enabled_label(suggestion.tab_accept_enabled),
                enabled_label(suggestion.explicit_accept_enabled),
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
    diagnostics: &[crate::StudioGuiCanvasDiagnosticState],
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
            let attention_summary =
                canvas_port_attention_summary(unit.unit_id.as_str(), &port.name, diagnostics);
            let hover_text = append_canvas_hover_attention(
                canvas_port_hover_text(unit, port, stream_label.as_deref()),
                attention_summary.as_deref(),
            );
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

pub(super) fn command_result_activity_line(
    title: &str,
    target: &StudioGuiCanvasCommandTargetViewModel,
) -> String {
    format!(
        "{}: {} {} ({})",
        title, target.kind_label, target.target_id, target.label
    )
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

fn canvas_viewport(
    view_mode: rf_ui::CanvasViewMode,
    units: &[StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[StudioGuiCanvasStreamLineViewModel],
    current_selection: Option<&StudioGuiCanvasSelectionViewModel>,
    focused_suggestion_id: Option<&str>,
) -> StudioGuiCanvasViewportViewModel {
    let layout_label = if units.iter().any(|unit| unit.layout_position.is_some()) {
        "persisted_positions"
    } else {
        "transient_grid"
    };
    let mode_label = canvas_view_mode_label(view_mode);
    let unit_count = units.len();
    let stream_line_count = stream_lines.len();
    let focus = current_selection
        .and_then(|selection| canvas_viewport_focus_for_selection(selection, units, stream_lines));
    let focus_summary = focus
        .as_ref()
        .map(|focus| format!("focus {} {}", focus.kind_label, focus.target_id))
        .or_else(|| focused_suggestion_id.map(|id| format!("suggestion {id} focused")))
        .unwrap_or_else(|| "no active focus target".to_string());

    StudioGuiCanvasViewportViewModel {
        mode_label,
        layout_label,
        summary: format!(
            "{mode_label} {layout_label}: {unit_count} unit(s), {stream_line_count} material line(s), {focus_summary}"
        ),
        unit_count,
        stream_line_count,
        focus,
    }
}

fn canvas_viewport_focus_for_selection(
    selection: &StudioGuiCanvasSelectionViewModel,
    units: &[StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[StudioGuiCanvasStreamLineViewModel],
) -> Option<StudioGuiCanvasViewportFocusViewModel> {
    match selection.kind_label {
        "Unit" => units
            .iter()
            .find(|unit| unit.unit_id == selection.target_id)
            .map(|unit| StudioGuiCanvasViewportFocusViewModel {
                kind_label: "Unit",
                target_id: unit.unit_id.clone(),
                source_label: "active_inspector_target",
                anchor_label: format!("unit-slot-{}", unit.layout_slot),
                detail: format!(
                    "{} | ports {}/{}",
                    unit.kind, unit.connected_port_count, unit.port_count
                ),
                command_id: unit.command_id.clone(),
            }),
        "Stream" => stream_lines
            .iter()
            .find(|stream| stream.stream_id == selection.target_id)
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
                StudioGuiCanvasViewportFocusViewModel {
                    kind_label: "Stream",
                    target_id: stream.stream_id.clone(),
                    source_label: "active_inspector_target",
                    anchor_label: stream.line_id.clone(),
                    detail: format!("{source} -> {sink}"),
                    command_id: stream.command_id.clone(),
                }
            }),
        _ => None,
    }
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
            attention_summary: unit.attention_summary.clone(),
            viewport_anchor_label: format!("unit-slot-{}", unit.layout_slot),
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
                    attention_summary: stream.attention_summary.clone(),
                    viewport_anchor_label: stream.line_id.clone(),
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
                detail: "Every canvas object surfaced by the current document",
                count: items.len(),
                enabled: !items.is_empty(),
            },
            StudioGuiCanvasObjectListFilterOptionViewModel {
                filter_id: "attention",
                label: "Attention",
                detail: "Objects with warning or error badges",
                count: attention_count,
                enabled: attention_count > 0,
            },
            StudioGuiCanvasObjectListFilterOptionViewModel {
                filter_id: "units",
                label: "Units",
                detail: "Unit blocks",
                count: units.len(),
                enabled: !units.is_empty(),
            },
            StudioGuiCanvasObjectListFilterOptionViewModel {
                filter_id: "streams",
                label: "Streams",
                detail: "Material stream lines",
                count: stream_count,
                enabled: stream_count > 0,
            },
        ],
        items,
    }
}

fn canvas_legend(
    run_status: Option<&StudioGuiCanvasRunStatusViewModel>,
    pending_edit: Option<&StudioGuiCanvasPendingEditViewModel>,
    object_list: &StudioGuiCanvasObjectListViewModel,
    units: &[StudioGuiCanvasUnitBlockViewModel],
    stream_lines: &[StudioGuiCanvasStreamLineViewModel],
) -> StudioGuiCanvasLegendViewModel {
    let mut items = Vec::new();

    if let Some(status) = run_status {
        let run_detail = status
            .summary
            .clone()
            .or_else(|| {
                status
                    .pending_reason_label
                    .map(|reason| format!("pending reason {reason}"))
            })
            .unwrap_or_else(|| "no current solve summary".to_string());
        items.push(StudioGuiCanvasLegendItemViewModel {
            kind_label: "Run",
            label: status.status_label.to_string(),
            detail: format!(
                "{run_detail}; diagnostics={} attention={}",
                status.diagnostic_count, status.attention_count
            ),
            swatch_label: "run_status",
        });
    }

    if object_list.attention_count > 0 {
        items.push(StudioGuiCanvasLegendItemViewModel {
            kind_label: "Attention",
            label: format!("{} object(s)", object_list.attention_count),
            detail: "warning/error badges are aggregated onto related units and streams"
                .to_string(),
            swatch_label: "attention",
        });
    } else {
        items.push(StudioGuiCanvasLegendItemViewModel {
            kind_label: "Attention",
            label: "No warning/error badges".to_string(),
            detail: "info diagnostics stay out of the canvas badge layer".to_string(),
            swatch_label: "neutral",
        });
    }

    let connected_port_count = units
        .iter()
        .flat_map(|unit| unit.ports.iter())
        .filter(|port| port.is_connected)
        .count();
    let total_port_count = units.iter().map(|unit| unit.ports.len()).sum::<usize>();
    if total_port_count > 0 {
        items.push(StudioGuiCanvasLegendItemViewModel {
            kind_label: "Ports",
            label: format!("{connected_port_count}/{total_port_count} bound"),
            detail: "green markers have stream bindings; gray markers are unbound".to_string(),
            swatch_label: "port",
        });
    }

    if !stream_lines.is_empty() {
        items.push(StudioGuiCanvasLegendItemViewModel {
            kind_label: "Streams",
            label: format!("{} material line(s)", stream_lines.len()),
            detail:
                "arrows indicate source port to sink port; terminal lines end at product outlets"
                    .to_string(),
            swatch_label: "stream",
        });
    }

    if pending_edit.is_some() {
        items.push(StudioGuiCanvasLegendItemViewModel {
            kind_label: "Edit",
            label: "Pending placement".to_string(),
            detail: "unit placement intent is active".to_string(),
            swatch_label: "pending_edit",
        });
    }

    StudioGuiCanvasLegendViewModel {
        title: "Canvas legend",
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

fn canvas_unit_attention_summary(
    unit_id: &str,
    diagnostics: &[crate::StudioGuiCanvasDiagnosticState],
) -> Option<String> {
    canvas_attention_summary(diagnostics.iter().filter(|diagnostic| {
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

fn canvas_stream_attention_summary(
    stream_id: &str,
    diagnostics: &[crate::StudioGuiCanvasDiagnosticState],
) -> Option<String> {
    canvas_attention_summary(diagnostics.iter().filter(|diagnostic| {
        diagnostic
            .related_stream_ids
            .iter()
            .any(|related_stream_id| related_stream_id.as_str() == stream_id)
    }))
}

fn canvas_port_attention_summary(
    unit_id: &str,
    port_name: &str,
    diagnostics: &[crate::StudioGuiCanvasDiagnosticState],
) -> Option<String> {
    canvas_attention_summary(diagnostics.iter().filter(|diagnostic| {
        diagnostic.related_port_targets.iter().any(|target| {
            target.unit_id.as_str() == unit_id && target.port_name.as_str() == port_name
        })
    }))
}

fn canvas_attention_summary<'a>(
    diagnostics: impl Iterator<Item = &'a crate::StudioGuiCanvasDiagnosticState>,
) -> Option<String> {
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut codes = BTreeSet::new();
    let mut ports = BTreeSet::new();

    for diagnostic in diagnostics {
        if !canvas_diagnostic_requires_attention(diagnostic.severity) {
            continue;
        }
        match diagnostic.severity {
            rf_ui::DiagnosticSeverity::Error => error_count += 1,
            rf_ui::DiagnosticSeverity::Warning => warning_count += 1,
            rf_ui::DiagnosticSeverity::Info => {}
        }
        codes.insert(diagnostic.code.clone());
        for target in &diagnostic.related_port_targets {
            ports.insert(format!("{}:{}", target.unit_id.as_str(), target.port_name));
        }
    }

    if error_count == 0 && warning_count == 0 {
        return None;
    }

    let mut parts = Vec::new();
    if error_count > 0 {
        parts.push(format!("{error_count} error(s)"));
    }
    if warning_count > 0 {
        parts.push(format!("{warning_count} warning(s)"));
    }
    if !ports.is_empty() {
        parts.push(format!(
            "ports {}",
            ports.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    if !codes.is_empty() {
        parts.push(format!(
            "codes {}",
            codes.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }

    Some(format!("attention: {}", parts.join("; ")))
}

fn append_canvas_hover_attention(base: String, attention_summary: Option<&str>) -> String {
    match attention_summary {
        Some(summary) => format!("{base}\n{summary}\nRead-only attention summary"),
        None => base,
    }
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
    let port_targets = diagnostic
        .related_port_targets
        .iter()
        .map(|target| format!("{}:{}", target.unit_id.as_str(), target.port_name))
        .collect::<Vec<_>>();
    if port_targets.is_empty() {
        format!("{}: {}", diagnostic.code, diagnostic.message)
    } else {
        format!(
            "{}: {} (ports {})",
            diagnostic.code,
            diagnostic.message,
            port_targets.join(", ")
        )
    }
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

fn canvas_view_mode_label(view_mode: rf_ui::CanvasViewMode) -> &'static str {
    match view_mode {
        rf_ui::CanvasViewMode::Planar => "Planar",
        rf_ui::CanvasViewMode::Perspective => "Perspective",
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
