use super::*;

pub(super) fn apply_canvas_edit_intent(
    flowsheet: &Flowsheet,
    intent: &CanvasEditIntent,
) -> RfResult<(DocumentCommand, Flowsheet, UnitId)> {
    match intent {
        CanvasEditIntent::PlaceUnit { unit_kind } => {
            apply_place_unit_canvas_edit(flowsheet, unit_kind)
        }
    }
}

pub(super) fn apply_place_unit_canvas_edit(
    flowsheet: &Flowsheet,
    unit_kind: &str,
) -> RfResult<(DocumentCommand, Flowsheet, UnitId)> {
    let builtin_kind = parse_canvas_unit_kind(unit_kind).ok_or_else(|| {
        RfError::invalid_input(format!(
            "canvas place unit intent uses unsupported unit kind `{unit_kind}`"
        ))
    })?;
    let spec = builtin_unit_spec(builtin_kind);
    let unit_id = next_canvas_unit_id(flowsheet, builtin_kind);
    let unit = UnitNode::new(
        unit_id.clone(),
        next_canvas_unit_name(flowsheet, builtin_kind),
        spec.kind.as_str(),
        spec.ports
            .iter()
            .map(|port| UnitPort::new(port.name, port.direction, port.kind, None))
            .collect(),
    );

    let mut next_flowsheet = flowsheet.clone();
    next_flowsheet.insert_unit(unit)?;

    Ok((
        DocumentCommand::CreateUnit {
            unit_id: unit_id.clone(),
            kind: spec.kind.as_str().to_string(),
        },
        next_flowsheet,
        unit_id,
    ))
}

pub(super) fn parse_canvas_unit_kind(unit_kind: &str) -> Option<BuiltinUnitKind> {
    let normalized = unit_kind
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "_");
    match normalized.as_str() {
        "feed" => Some(BuiltinUnitKind::Feed),
        "mixer" => Some(BuiltinUnitKind::Mixer),
        "heater" => Some(BuiltinUnitKind::Heater),
        "cooler" => Some(BuiltinUnitKind::Cooler),
        "valve" => Some(BuiltinUnitKind::Valve),
        "flash" | "flash_drum" => Some(BuiltinUnitKind::FlashDrum),
        _ => None,
    }
}

pub(super) fn next_canvas_unit_id(flowsheet: &Flowsheet, kind: BuiltinUnitKind) -> UnitId {
    let prefix = canvas_unit_id_prefix(kind);
    for index in 1.. {
        let candidate = UnitId::new(format!("{prefix}-{index}"));
        if !flowsheet.units.contains_key(&candidate) {
            return candidate;
        }
    }
    unreachable!("unbounded unit id sequence should eventually find an unused id")
}

pub(super) fn next_canvas_unit_name(flowsheet: &Flowsheet, kind: BuiltinUnitKind) -> String {
    let label = canvas_unit_label(kind);
    let used_count = flowsheet
        .units
        .values()
        .filter(|unit| parse_canvas_unit_kind(&unit.kind) == Some(kind))
        .count();
    if used_count == 0 {
        label.to_string()
    } else {
        format!("{label} {}", used_count + 1)
    }
}

pub(super) fn canvas_unit_id_prefix(kind: BuiltinUnitKind) -> &'static str {
    match kind {
        BuiltinUnitKind::Feed => "feed",
        BuiltinUnitKind::Mixer => "mixer",
        BuiltinUnitKind::Heater => "heater",
        BuiltinUnitKind::Cooler => "cooler",
        BuiltinUnitKind::Valve => "valve",
        BuiltinUnitKind::FlashDrum => "flash",
    }
}

pub(super) fn canvas_unit_label(kind: BuiltinUnitKind) -> &'static str {
    match kind {
        BuiltinUnitKind::Feed => "Feed",
        BuiltinUnitKind::Mixer => "Mixer",
        BuiltinUnitKind::Heater => "Heater",
        BuiltinUnitKind::Cooler => "Cooler",
        BuiltinUnitKind::Valve => "Valve",
        BuiltinUnitKind::FlashDrum => "Flash Drum",
    }
}

pub(super) fn format_canvas_edit_commit_message(
    intent: &CanvasEditIntent,
    unit_id: &UnitId,
    position: CanvasPoint,
) -> String {
    match intent {
        CanvasEditIntent::PlaceUnit { unit_kind } => format!(
            "Created canvas unit `{}` of kind `{}` at ({:.1}, {:.1})",
            unit_id.as_str(),
            unit_kind,
            position.x,
            position.y
        ),
    }
}

pub(super) fn apply_canvas_suggestion_acceptance(
    app_state: &mut AppState,
    suggestion: &CanvasSuggestion,
) -> RfResult<()> {
    let acceptance = suggestion.acceptance.as_ref().ok_or_else(|| {
        RfError::invalid_input(format!(
            "canvas suggestion `{}` is missing an acceptance payload",
            suggestion.id
        ))
    })?;

    let (command, next_flowsheet) = match acceptance {
        CanvasSuggestionAcceptance::MaterialConnection(connection) => {
            apply_material_connection_acceptance(
                &app_state.workspace.document.flowsheet,
                connection,
            )?
        }
    };

    app_state.commit_document_change(command, next_flowsheet, SystemTime::now());
    Ok(())
}

pub(super) fn apply_canvas_suggestion_target(
    app_state: &mut AppState,
    suggestion: &CanvasSuggestion,
) {
    let unit_id = suggestion.ghost.target_unit_id.clone();
    app_state.workspace.selection.selected_units.clear();
    app_state.workspace.selection.selected_streams.clear();
    app_state
        .workspace
        .selection
        .selected_units
        .insert(unit_id.clone());
    app_state.workspace.drafts.active_target = Some(InspectorTarget::Unit(unit_id));
    app_state.workspace.panels.inspector_open = true;
}

pub(super) fn apply_material_connection_acceptance(
    flowsheet: &Flowsheet,
    connection: &CanvasSuggestedMaterialConnection,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    let stream_id = match &connection.stream {
        CanvasSuggestedStreamBinding::Existing { stream_id } => {
            next_flowsheet.stream(stream_id)?;
            stream_id.clone()
        }
        CanvasSuggestedStreamBinding::Create { stream } => {
            next_flowsheet.insert_stream(stream.clone())?;
            stream.id.clone()
        }
    };

    bind_material_stream_port(
        &mut next_flowsheet,
        &connection.source_unit_id,
        &connection.source_port,
        &stream_id,
    )?;
    if let (Some(sink_unit_id), Some(sink_port)) = (&connection.sink_unit_id, &connection.sink_port)
    {
        bind_material_stream_port(&mut next_flowsheet, sink_unit_id, sink_port, &stream_id)?;
    } else if connection.sink_unit_id.is_some() || connection.sink_port.is_some() {
        return Err(RfError::invalid_input(
            "canvas suggestion material connection must provide both sink unit and sink port or neither",
        ));
    }

    validate_material_connection_acceptance(flowsheet, connection, &stream_id)?;

    let command = DocumentCommand::ConnectPorts {
        stream_id,
        from_unit_id: connection.source_unit_id.clone(),
        from_port: connection.source_port.clone(),
        to_unit_id: connection.sink_unit_id.clone(),
        to_port: connection.sink_port.clone(),
    };

    Ok((command, next_flowsheet))
}

pub(super) fn validate_material_connection_acceptance(
    flowsheet: &Flowsheet,
    connection: &CanvasSuggestedMaterialConnection,
    stream_id: &StreamId,
) -> RfResult<()> {
    validate_connection_endpoint(
        flowsheet,
        &connection.source_unit_id,
        &connection.source_port,
        PortDirection::Outlet,
        stream_id,
    )?;

    if let (Some(sink_unit_id), Some(sink_port)) = (&connection.sink_unit_id, &connection.sink_port)
    {
        validate_connection_endpoint(
            flowsheet,
            sink_unit_id,
            sink_port,
            PortDirection::Inlet,
            stream_id,
        )?;
    }

    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != PortKind::Material || port.stream_id.as_ref() != Some(stream_id) {
                continue;
            }

            match port.direction {
                PortDirection::Outlet
                    if unit.id != connection.source_unit_id
                        || port.name != connection.source_port =>
                {
                    return Err(RfError::invalid_connection(format!(
                        "stream `{}` already has upstream source `{}.{}`",
                        stream_id, unit.id, port.name
                    )));
                }
                PortDirection::Inlet => {
                    let expected_sink = connection
                        .sink_unit_id
                        .as_ref()
                        .zip(connection.sink_port.as_ref());
                    if expected_sink.is_none_or(|(sink_unit_id, sink_port)| {
                        sink_unit_id != &unit.id || sink_port != &port.name
                    }) {
                        return Err(RfError::invalid_connection(format!(
                            "stream `{}` already has downstream sink `{}.{}`",
                            stream_id, unit.id, port.name
                        )));
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

pub(super) fn validate_connection_endpoint(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
    expected_direction: PortDirection,
    stream_id: &StreamId,
) -> RfResult<()> {
    let unit = flowsheet
        .units
        .get(unit_id)
        .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
    let port = unit
        .ports
        .iter()
        .find(|port| port.name == port_name)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "unit `{}` does not expose material port `{}`",
                unit_id, port_name
            ))
        })?;
    if port.kind != PortKind::Material {
        return Err(RfError::invalid_input(format!(
            "unit `{}` port `{}` is not a material port",
            unit_id, port_name
        )));
    }
    if port.direction != expected_direction {
        return Err(RfError::invalid_connection(format!(
            "unit `{}` port `{}` has direction `{:?}` but canvas connection expected `{:?}`",
            unit_id, port_name, port.direction, expected_direction
        )));
    }
    if port
        .stream_id
        .as_ref()
        .is_some_and(|existing| existing != stream_id)
    {
        return Err(RfError::invalid_connection(format!(
            "unit `{}` port `{}` is already connected to stream `{}`",
            unit_id,
            port_name,
            port.stream_id
                .as_ref()
                .expect("checked existing stream id above")
        )));
    }

    Ok(())
}

pub(super) fn apply_run_panel_recovery_mutation(
    flowsheet: &Flowsheet,
    mutation: &RunPanelRecoveryMutation,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    match mutation {
        RunPanelRecoveryMutation::DeleteStream { stream_id } => {
            apply_delete_stream_mutation(flowsheet, stream_id)
        }
        RunPanelRecoveryMutation::CreateAndBindOutletStream { unit_id, port_name } => {
            apply_create_and_bind_outlet_stream_mutation(flowsheet, unit_id, port_name)
        }
        RunPanelRecoveryMutation::DisconnectPortAndDeleteStream {
            unit_id,
            port_name,
            stream_id,
        } => apply_disconnect_port_and_delete_stream_mutation(
            flowsheet, unit_id, port_name, stream_id,
        ),
        RunPanelRecoveryMutation::RestoreCanonicalPortSignature { unit_id } => {
            apply_restore_canonical_port_signature_mutation(flowsheet, unit_id)
        }
        RunPanelRecoveryMutation::DisconnectPort { unit_id, port_name } => {
            apply_disconnect_port_mutation(flowsheet, unit_id, port_name)
        }
    }
}

pub(super) fn apply_disconnect_port_mutation(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    disconnect_material_stream_port(&mut next_flowsheet, unit_id, port_name)?;

    Ok((
        DocumentCommand::DisconnectPorts {
            unit_id: unit_id.clone(),
            port: port_name.to_string(),
        },
        next_flowsheet,
    ))
}

pub(super) fn apply_delete_stream_mutation(
    flowsheet: &Flowsheet,
    stream_id: &StreamId,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    next_flowsheet.remove_stream(stream_id)?;

    Ok((
        DocumentCommand::DeleteStream {
            stream_id: stream_id.clone(),
        },
        next_flowsheet,
    ))
}

pub(super) fn apply_create_and_bind_outlet_stream_mutation(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    let stream_id = next_available_placeholder_stream_id(&next_flowsheet, unit_id, port_name);
    let stream_name = format!("{} {} Stream", unit_id.as_str(), port_name);
    next_flowsheet.insert_stream(MaterialStreamState::new(stream_id.clone(), stream_name))?;
    bind_material_stream_port(&mut next_flowsheet, unit_id, port_name, &stream_id)?;

    Ok((
        DocumentCommand::ConnectPorts {
            stream_id,
            from_unit_id: unit_id.clone(),
            from_port: port_name.to_string(),
            to_unit_id: None,
            to_port: None,
        },
        next_flowsheet,
    ))
}

pub(super) fn apply_disconnect_port_and_delete_stream_mutation(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
    stream_id: &StreamId,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    disconnect_material_stream_port(&mut next_flowsheet, unit_id, port_name)?;
    next_flowsheet.remove_stream(stream_id)?;

    Ok((
        DocumentCommand::DisconnectPortAndDeleteStream {
            unit_id: unit_id.clone(),
            port: port_name.to_string(),
            stream_id: stream_id.clone(),
        },
        next_flowsheet,
    ))
}

pub(super) fn apply_restore_canonical_port_signature_mutation(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    let unit = next_flowsheet
        .units
        .get_mut(unit_id)
        .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
    let spec = builtin_unit_spec_by_name(&unit.kind).ok_or_else(|| {
        RfError::invalid_input(format!(
            "unit `{}` kind `{}` does not expose a canonical built-in spec",
            unit_id, unit.kind
        ))
    })?;
    let restored_ports = rebuild_ports_from_canonical_spec(&unit.ports, spec);
    unit.ports = restored_ports;

    Ok((
        DocumentCommand::RestoreCanonicalUnitPorts {
            unit_id: unit_id.clone(),
        },
        next_flowsheet,
    ))
}

pub(super) fn bind_material_stream_port(
    flowsheet: &mut Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
    stream_id: &StreamId,
) -> RfResult<()> {
    let unit = flowsheet
        .units
        .get_mut(unit_id)
        .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
    let port = unit
        .ports
        .iter_mut()
        .find(|port| port.name == port_name)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "unit `{}` does not expose material port `{}`",
                unit_id, port_name
            ))
        })?;
    if port.kind != rf_types::PortKind::Material {
        return Err(RfError::invalid_input(format!(
            "unit `{}` port `{}` is not a material port",
            unit_id, port_name
        )));
    }
    port.stream_id = Some(stream_id.clone());
    Ok(())
}

pub(super) fn disconnect_material_stream_port(
    flowsheet: &mut Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
) -> RfResult<()> {
    let unit = flowsheet
        .units
        .get_mut(unit_id)
        .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
    let port = unit
        .ports
        .iter_mut()
        .find(|port| port.name == port_name)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "unit `{}` does not expose material port `{}`",
                unit_id, port_name
            ))
        })?;
    if port.kind != rf_types::PortKind::Material {
        return Err(RfError::invalid_input(format!(
            "unit `{}` port `{}` is not a material port",
            unit_id, port_name
        )));
    }
    port.stream_id = None;
    Ok(())
}

pub(super) fn next_available_placeholder_stream_id(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
) -> StreamId {
    let base = format!("stream-{}-{}", unit_id.as_str(), port_name);
    let mut candidate = base.clone();
    let mut suffix = 1usize;
    while flowsheet
        .streams
        .contains_key(&StreamId::new(candidate.as_str()))
    {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
    StreamId::new(candidate)
}

pub(super) fn rebuild_ports_from_canonical_spec(
    existing_ports: &[UnitPort],
    spec: &UnitOperationSpec,
) -> Vec<UnitPort> {
    let mut remaining_ports = existing_ports.to_vec();
    let mut rebuilt = Vec::with_capacity(spec.ports.len());

    for expected in spec.ports {
        let stream_id =
            take_matching_stream_id(&mut remaining_ports, |port| port.name == expected.name)
                .or_else(|| {
                    take_unique_matching_stream_id(&mut remaining_ports, |port| {
                        port.direction == expected.direction && port.kind == expected.kind
                    })
                });
        rebuilt.push(UnitPort::new(
            expected.name,
            expected.direction,
            expected.kind,
            stream_id,
        ));
    }

    rebuilt
}

pub(super) fn take_matching_stream_id<F>(
    remaining_ports: &mut Vec<UnitPort>,
    predicate: F,
) -> Option<StreamId>
where
    F: Fn(&UnitPort) -> bool,
{
    let index = remaining_ports.iter().position(predicate)?;
    Some(remaining_ports.remove(index).stream_id).flatten()
}

pub(super) fn take_unique_matching_stream_id<F>(
    remaining_ports: &mut Vec<UnitPort>,
    predicate: F,
) -> Option<StreamId>
where
    F: Fn(&UnitPort) -> bool,
{
    let mut matches = remaining_ports
        .iter()
        .enumerate()
        .filter_map(|(index, port)| predicate(port).then_some(index));
    let index = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(remaining_ports.remove(index).stream_id).flatten()
}

pub(super) fn format_canvas_suggestion_accept_message(suggestion: &CanvasSuggestion) -> String {
    format!(
        "Accepted canvas suggestion `{}` from {} for unit {}",
        suggestion.id.as_str(),
        canvas_suggestion_source_label(suggestion.source),
        suggestion.ghost.target_unit_id.as_str()
    )
}

pub(super) fn canvas_suggestion_source_label(source: SuggestionSource) -> &'static str {
    match source {
        SuggestionSource::LocalRules => "local rules",
        SuggestionSource::RadishMind => "RadishMind",
    }
}
