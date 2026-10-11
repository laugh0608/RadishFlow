//! Pressure evidence from committed inputs, without interpreting stream templates
//! as solved values or running upstream operations.

use rf_model::{Flowsheet, UnitNode};
use rf_types::{PortDirection, PortKind, StreamId, UnitId};
use rf_unitops::specifications::{
    FeedInputField, FeedInputSource, InletPressure, InletPressureLimit, OutletPressureRule,
    effective_feed_input, inlet_pressure_limit, input_specification, is_positive_finite,
};
use rf_unitops::{BuiltinUnitKind, validate_unit_node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentPressureSource {
    FeedStream {
        unit_id: UnitId,
        stream_id: StreamId,
    },
    UnitParameter {
        unit_id: UnitId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcedInletPressure {
    pub input: InletPressure,
    pub source: DocumentPressureSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PressureContextGapReason {
    UnboundInlet,
    InvalidInletSignature,
    MissingStream(StreamId),
    MissingSource(StreamId),
    AmbiguousSource(StreamId),
    UnsupportedSource(UnitId),
    InvalidSourcePorts(UnitId),
    MissingExplicitPressure(UnitId),
    InvalidPressure(DocumentPressureSource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PressureContextGap {
    pub inlet_port: &'static str,
    pub reason: PressureContextGapReason,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentPressureContext {
    pub rule: OutletPressureRule,
    pub inputs: Vec<SourcedInletPressure>,
    pub gaps: Vec<PressureContextGap>,
}

impl DocumentPressureContext {
    pub fn limit(&self) -> Option<InletPressureLimit> {
        if !self.gaps.is_empty() {
            return None;
        }
        let inputs = self
            .inputs
            .iter()
            .map(|input| input.input.clone())
            .collect::<Vec<_>>();
        inlet_pressure_limit(self.rule, &inputs)
    }
}

/// Partial editing topologies may lack an inlet. Keep the gap instead of taking
/// the minimum of whichever streams happen to exist (especially for Mixer).
pub fn document_pressure_context(
    flowsheet: &Flowsheet,
    unit: &UnitNode,
    kind: BuiltinUnitKind,
) -> DocumentPressureContext {
    let rule = input_specification(kind).outlet_pressure_rule;
    let mut context = DocumentPressureContext {
        rule,
        inputs: Vec::new(),
        gaps: Vec::new(),
    };
    for &port in rule.inlet_ports() {
        match resolve_inlet_pressure(flowsheet, unit, port) {
            Ok(input) => context.inputs.push(input),
            Err(reason) => context.gaps.push(PressureContextGap {
                inlet_port: port,
                reason,
            }),
        }
    }
    context
}

fn resolve_inlet_pressure(
    flowsheet: &Flowsheet,
    unit: &UnitNode,
    inlet_port: &'static str,
) -> Result<SourcedInletPressure, PressureContextGapReason> {
    use PressureContextGapReason::*;
    let mut ports = unit.ports.iter().filter(|port| port.name == inlet_port);
    let port = ports.next().ok_or(UnboundInlet)?;
    if ports.next().is_some()
        || port.direction != PortDirection::Inlet
        || port.kind != PortKind::Material
    {
        return Err(InvalidInletSignature);
    }
    let stream_id = port.stream_id.as_ref().ok_or(UnboundInlet)?;
    let stream = flowsheet
        .streams
        .get(stream_id)
        .ok_or_else(|| MissingStream(stream_id.clone()))?;
    let mut sources = flowsheet.units.values().flat_map(|source| {
        source.ports.iter().filter_map(move |port| {
            (port.direction == PortDirection::Outlet
                && port.kind == PortKind::Material
                && port.stream_id.as_ref() == Some(stream_id))
            .then_some(source)
        })
    });
    let source = sources
        .next()
        .ok_or_else(|| MissingSource(stream_id.clone()))?;
    if sources.next().is_some() {
        return Err(AmbiguousSource(stream_id.clone()));
    }
    let kind =
        BuiltinUnitKind::parse(&source.kind).ok_or_else(|| UnsupportedSource(source.id.clone()))?;
    validate_unit_node(source).map_err(|_| InvalidSourcePorts(source.id.clone()))?;
    let (pressure_pa, origin) = if kind == BuiltinUnitKind::Feed {
        let value = effective_feed_input(&source.parameters, stream, FeedInputField::Pressure);
        let origin = match value.source {
            FeedInputSource::StreamInput => DocumentPressureSource::FeedStream {
                unit_id: source.id.clone(),
                stream_id: stream_id.clone(),
            },
            FeedInputSource::UnitParameter(_) => DocumentPressureSource::UnitParameter {
                unit_id: source.id.clone(),
            },
        };
        (value.value, origin)
    } else {
        // Every other current built-in produces its specified pressure. A missing
        // parameter's execution fallback is not an explicit authoring specification.
        let pressure = source
            .parameters
            .outlet_pressure_pa
            .ok_or_else(|| MissingExplicitPressure(source.id.clone()))?;
        (
            pressure,
            DocumentPressureSource::UnitParameter {
                unit_id: source.id.clone(),
            },
        )
    };
    if !is_positive_finite(pressure_pa) {
        return Err(InvalidPressure(origin));
    }
    Ok(SourcedInletPressure {
        input: InletPressure {
            port: inlet_port,
            stream_id: stream_id.clone(),
            pressure_pa,
        },
        source: origin,
    })
}

#[cfg(test)]
mod tests;
