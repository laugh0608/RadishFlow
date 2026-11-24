//! Reconnecting an existing stream preserves its bound end and only offers acyclic edges.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamReconnect {
    Missing,
    Unbound,
    Connected { sink: StreamPortBinding },
    InvalidBindings,
    Open(StreamReconnectOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamReconnectOptions {
    pub fixed: StreamPortBinding,
    pub missing: PortDirection,
    pub candidates: Vec<StreamPortBinding>,
    pub occupied: usize,
    pub cycle_blocked: usize,
}

impl StreamReconnectOptions {
    fn endpoints(&self, chosen: &StreamPortBinding) -> (StreamPortBinding, StreamPortBinding) {
        match self.missing {
            PortDirection::Outlet => (chosen.clone(), self.fixed.clone()),
            PortDirection::Inlet => (self.fixed.clone(), chosen.clone()),
        }
    }
}

pub fn stream_reconnect_options(flow: &Flowsheet, stream: &StreamId) -> StreamReconnect {
    if !flow.streams.contains_key(stream) {
        return StreamReconnect::Missing;
    }
    let mut source = None;
    let mut sink = None;
    for unit in flow.units.values() {
        for port in &unit.ports {
            if port.kind != PortKind::Material || port.stream_id.as_ref() != Some(stream) {
                continue;
            }
            let slot = match port.direction {
                PortDirection::Outlet => &mut source,
                PortDirection::Inlet => &mut sink,
            };
            if slot.is_some() {
                return StreamReconnect::InvalidBindings;
            }
            *slot = Some(StreamPortBinding {
                unit_id: unit.id.clone(),
                port: port.name.clone(),
            });
        }
    }
    let (fixed, missing) = match (source, sink) {
        (Some(_), Some(sink)) => return StreamReconnect::Connected { sink },
        (None, None) => return StreamReconnect::Unbound,
        (Some(source), None) => (source, PortDirection::Inlet),
        (None, Some(sink)) => (sink, PortDirection::Outlet),
    };
    let mut candidates = Vec::new();
    let mut occupied = 0;
    let mut cycle_blocked = 0;
    let blocked_units = reachable_material_units(flow, &fixed.unit_id, missing);
    for unit in flow.units.values() {
        for port in unit
            .ports
            .iter()
            .filter(|p| p.kind == PortKind::Material && p.direction == missing)
        {
            if port.stream_id.is_some() {
                occupied += 1;
                continue;
            }
            if blocked_units.contains(&unit.id) {
                cycle_blocked += 1;
            } else {
                candidates.push(StreamPortBinding {
                    unit_id: unit.id.clone(),
                    port: port.name.clone(),
                });
            }
        }
    }
    candidates.sort_by(|a, b| (&a.unit_id, &a.port).cmp(&(&b.unit_id, &b.port)));
    StreamReconnect::Open(StreamReconnectOptions {
        fixed,
        missing,
        candidates,
        occupied,
        cycle_blocked,
    })
}

impl AppState {
    pub fn reconnect_stream_to_unique_available_endpoint(
        &mut self,
        stream_id: &StreamId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamReconnectEditResult>> {
        let StreamReconnect::Open(options) =
            stream_reconnect_options(&self.workspace.document.flowsheet, stream_id)
        else {
            return Ok(None);
        };
        let [chosen] = options.candidates.as_slice() else {
            return Ok(None);
        };
        self.commit_stream_reconnect(stream_id, &options, chosen, changed_at)
            .map(Some)
    }

    pub fn reconnect_stream_to_selected_endpoint(
        &mut self,
        stream_id: &StreamId,
        fixed: &StreamPortBinding,
        missing: PortDirection,
        chosen: &StreamPortBinding,
        changed_at: DateTimeUtc,
    ) -> RfResult<StreamReconnectEditResult> {
        let StreamReconnect::Open(options) =
            stream_reconnect_options(&self.workspace.document.flowsheet, stream_id)
        else {
            return Err(RfError::invalid_connection(
                "stream no longer has exactly one bound material endpoint",
            ));
        };
        if &options.fixed != fixed || options.missing != missing {
            return Err(RfError::invalid_connection(
                "the fixed material endpoint changed",
            ));
        }
        if !options.candidates.contains(chosen) {
            return Err(RfError::invalid_connection(format!(
                "{}:{} is no longer an available acyclic material endpoint",
                chosen.unit_id, chosen.port
            )));
        }
        self.commit_stream_reconnect(stream_id, &options, chosen, changed_at)
    }

    fn commit_stream_reconnect(
        &mut self,
        stream_id: &StreamId,
        options: &StreamReconnectOptions,
        chosen: &StreamPortBinding,
        changed_at: DateTimeUtc,
    ) -> RfResult<StreamReconnectEditResult> {
        let (source_port, sink_port) = options.endpoints(chosen);
        let connection = CanvasSuggestedMaterialConnection {
            stream: CanvasSuggestedStreamBinding::Existing {
                stream_id: stream_id.clone(),
            },
            source_unit_id: source_port.unit_id.clone(),
            source_port: source_port.port.clone(),
            sink_unit_id: Some(sink_port.unit_id.clone()),
            sink_port: Some(sink_port.port.clone()),
        };
        let (command, revision) = self.commit_material_connection(&connection, changed_at)?;
        self.focus_inspector_target(InspectorTarget::Stream(stream_id.clone()));
        Ok(StreamReconnectEditResult {
            stream_id: stream_id.clone(),
            source_port,
            sink_port,
            command,
            revision,
        })
    }
}
