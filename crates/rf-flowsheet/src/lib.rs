use std::collections::BTreeMap;

use rf_model::Flowsheet;
use rf_types::{DiagnosticPortTarget, PortDirection, PortKind, RfError, RfResult, StreamId, UnitId};
use rf_unitops::{builtin_unit_spec_by_name, validate_unit_node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialPortRef {
    pub unit_id: UnitId,
    pub unit_name: String,
    pub port_name: String,
}

impl MaterialPortRef {
    fn new(unit_id: UnitId, unit_name: String, port_name: String) -> Self {
        Self {
            unit_id,
            unit_name,
            port_name,
        }
    }

    fn as_diagnostic_port_target(&self) -> DiagnosticPortTarget {
        DiagnosticPortTarget::new(self.unit_id.clone(), self.port_name.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialConnection {
    pub stream_id: StreamId,
    pub source: MaterialPortRef,
    pub sink: Option<MaterialPortRef>,
}

#[derive(Debug, Clone, Default)]
struct MaterialEndpoints {
    source: Option<MaterialPortRef>,
    sink: Option<MaterialPortRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionValidationDiagnosticCode {
    UnsupportedUnitKind,
    InvalidPortSignature,
    UnboundInletPort,
    UnboundOutletPort,
    MissingStreamReference,
    DuplicateUpstreamSource,
    DuplicateDownstreamSink,
    OrphanStream,
    MissingUpstreamSource,
}

impl ConnectionValidationDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedUnitKind => "flowsheet.connection_validation.unsupported_unit_kind",
            Self::InvalidPortSignature => "flowsheet.connection_validation.invalid_port_signature",
            Self::UnboundInletPort => "flowsheet.connection_validation.unbound_inlet_port",
            Self::UnboundOutletPort => "flowsheet.connection_validation.unbound_outlet_port",
            Self::MissingStreamReference => {
                "flowsheet.connection_validation.missing_stream_reference"
            }
            Self::DuplicateUpstreamSource => {
                "flowsheet.connection_validation.duplicate_upstream_source"
            }
            Self::DuplicateDownstreamSink => {
                "flowsheet.connection_validation.duplicate_downstream_sink"
            }
            Self::OrphanStream => "flowsheet.connection_validation.orphan_stream",
            Self::MissingUpstreamSource => {
                "flowsheet.connection_validation.missing_upstream_source"
            }
        }
    }
}

fn invalid_connection_error(
    code: ConnectionValidationDiagnosticCode,
    message: impl Into<String>,
) -> RfError {
    RfError::invalid_connection(message).with_diagnostic_code(code.as_str())
}

pub fn validate_connections(flowsheet: &Flowsheet) -> RfResult<Vec<MaterialConnection>> {
    let mut endpoints_by_stream = BTreeMap::<StreamId, MaterialEndpoints>::new();

    for unit in flowsheet.units.values() {
        let validation_code = if builtin_unit_spec_by_name(&unit.kind).is_none() {
            ConnectionValidationDiagnosticCode::UnsupportedUnitKind
        } else {
            ConnectionValidationDiagnosticCode::InvalidPortSignature
        };
        validate_unit_node(unit).map_err(|error| {
            invalid_connection_error(
                validation_code,
                format!(
                "unit `{}` does not match its canonical built-in port signature: {}",
                unit.id,
                error.message()
                ),
            )
            .with_related_unit_id(unit.id.clone())
        })?;

        for port in &unit.ports {
            if port.kind != PortKind::Material {
                continue;
            }

            let stream_id = port.stream_id.clone().ok_or_else(|| {
                let diagnostic_code = match port.direction {
                    PortDirection::Inlet => ConnectionValidationDiagnosticCode::UnboundInletPort,
                    PortDirection::Outlet => ConnectionValidationDiagnosticCode::UnboundOutletPort,
                };
                invalid_connection_error(
                    diagnostic_code,
                    format!(
                    "unit `{}` material port `{}` is not connected to any stream",
                    unit.id, port.name
                    ),
                )
                .with_related_unit_id(unit.id.clone())
                .with_related_port_target(DiagnosticPortTarget::new(
                    unit.id.clone(),
                    port.name.clone(),
                ))
            })?;
            if !flowsheet.streams.contains_key(&stream_id) {
                return Err(
                invalid_connection_error(
                    ConnectionValidationDiagnosticCode::MissingStreamReference,
                    format!(
                        "unit `{}` material port `{}` references missing stream `{}`",
                        unit.id, port.name, stream_id
                    ),
                    )
                    .with_related_unit_id(unit.id.clone())
                    .with_related_port_target(DiagnosticPortTarget::new(
                        unit.id.clone(),
                        port.name.clone(),
                    ))
                    .with_related_stream_id(stream_id),
                );
            }

            let endpoints = endpoints_by_stream.entry(stream_id.clone()).or_default();
            let port_ref =
                MaterialPortRef::new(unit.id.clone(), unit.name.clone(), port.name.clone());

            match port.direction {
                PortDirection::Outlet => {
                    if let Some(existing) = &endpoints.source {
                        return Err(
                            invalid_connection_error(
                                ConnectionValidationDiagnosticCode::DuplicateUpstreamSource,
                                format!(
                                "stream `{}` is produced by both `{}.{}` and `{}.{}`",
                                stream_id,
                                existing.unit_id,
                                existing.port_name,
                                port_ref.unit_id,
                                port_ref.port_name
                                ),
                            )
                            .with_related_unit_ids(vec![
                                existing.unit_id.clone(),
                                port_ref.unit_id.clone(),
                            ])
                            .with_related_port_targets(vec![
                                existing.as_diagnostic_port_target(),
                                port_ref.as_diagnostic_port_target(),
                            ])
                            .with_related_stream_id(stream_id.clone()),
                        );
                    }

                    endpoints.source = Some(port_ref);
                }
                PortDirection::Inlet => {
                    if let Some(existing) = &endpoints.sink {
                        return Err(
                            invalid_connection_error(
                                ConnectionValidationDiagnosticCode::DuplicateDownstreamSink,
                                format!(
                                "stream `{}` is consumed by both `{}.{}` and `{}.{}`",
                                stream_id,
                                existing.unit_id,
                                existing.port_name,
                                port_ref.unit_id,
                                port_ref.port_name
                                ),
                            )
                            .with_related_unit_ids(vec![
                                existing.unit_id.clone(),
                                port_ref.unit_id.clone(),
                            ])
                            .with_related_port_targets(vec![
                                existing.as_diagnostic_port_target(),
                                port_ref.as_diagnostic_port_target(),
                            ])
                            .with_related_stream_id(stream_id.clone()),
                        );
                    }

                    endpoints.sink = Some(port_ref);
                }
            }
        }
    }

    for stream_id in flowsheet.streams.keys() {
        if !endpoints_by_stream.contains_key(stream_id) {
            return Err(invalid_connection_error(
                ConnectionValidationDiagnosticCode::OrphanStream,
                format!("stream `{}` is not connected to any material port", stream_id),
            )
            .with_related_stream_id(stream_id.clone()));
        }
    }

    endpoints_by_stream
        .into_iter()
        .map(|(stream_id, endpoints)| {
            let source = endpoints.source.ok_or_else(|| match &endpoints.sink {
                Some(sink) => invalid_connection_error(
                    ConnectionValidationDiagnosticCode::MissingUpstreamSource,
                    format!("stream `{}` is missing an upstream outlet connection", stream_id),
                )
                .with_related_unit_id(sink.unit_id.clone())
                .with_related_port_target(sink.as_diagnostic_port_target())
                .with_related_stream_id(stream_id.clone()),
                None => invalid_connection_error(
                    ConnectionValidationDiagnosticCode::MissingUpstreamSource,
                    format!("stream `{}` is missing an upstream outlet connection", stream_id),
                )
                .with_related_stream_id(stream_id.clone()),
            })?;

            Ok(MaterialConnection {
                stream_id,
                source,
                sink: endpoints.sink,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::validate_connections;
    use rf_model::{Composition, Flowsheet, MaterialStreamState, UnitNode, UnitPort};
    use rf_types::{ComponentId, DiagnosticPortTarget, PortDirection, PortKind, StreamId, UnitId};
    use rf_unitops::{FEED_KIND, build_feed_node, build_flash_drum_node, build_mixer_node};

    fn binary_composition(first: f64, second: f64) -> Composition {
        [
            (ComponentId::new("component-a"), first),
            (ComponentId::new("component-b"), second),
        ]
        .into_iter()
        .collect()
    }

    fn build_stream(id: &str) -> MaterialStreamState {
        MaterialStreamState::from_tpzf(id, id, 300.0, 101_325.0, 1.0, binary_composition(0.5, 0.5))
    }

    #[test]
    fn validates_feed_mixer_flash_drum_connection_chain() {
        let mut flowsheet = Flowsheet::new("demo");
        for stream_id in [
            "stream-feed-a",
            "stream-feed-b",
            "stream-mix-out",
            "stream-liquid",
            "stream-vapor",
        ] {
            flowsheet
                .insert_stream(build_stream(stream_id))
                .expect("expected stream insert");
        }
        for unit in [
            build_feed_node("feed-a", "Feed A", "stream-feed-a"),
            build_feed_node("feed-b", "Feed B", "stream-feed-b"),
            build_mixer_node(
                "mixer-1",
                "Mixer",
                "stream-feed-a",
                "stream-feed-b",
                "stream-mix-out",
            ),
            build_flash_drum_node(
                "flash-1",
                "Flash Drum",
                "stream-mix-out",
                "stream-liquid",
                "stream-vapor",
            ),
        ] {
            flowsheet.insert_unit(unit).expect("expected unit insert");
        }

        let connections = validate_connections(&flowsheet).expect("expected valid connections");

        assert_eq!(connections.len(), 5);
        let liquid = connections
            .iter()
            .find(|connection| connection.stream_id.as_str() == "stream-liquid")
            .expect("expected liquid connection");
        assert!(liquid.sink.is_none());
    }

    #[test]
    fn rejects_duplicate_stream_consumers() {
        let mut flowsheet = Flowsheet::new("demo");
        for stream_id in [
            "shared-stream",
            "stream-b",
            "stream-out",
            "stream-liquid",
            "stream-vapor",
        ] {
            flowsheet
                .insert_stream(build_stream(stream_id))
                .expect("expected stream insert");
        }
        flowsheet
            .insert_unit(build_feed_node("feed-1", "Feed", "shared-stream"))
            .expect("expected feed insert");
        flowsheet
            .insert_unit(build_mixer_node(
                "mixer-1",
                "Mixer",
                "shared-stream",
                "stream-b",
                "stream-out",
            ))
            .expect("expected mixer insert");
        flowsheet
            .insert_unit(build_flash_drum_node(
                "flash-1",
                "Flash Drum",
                "shared-stream",
                "stream-liquid",
                "stream-vapor",
            ))
            .expect("expected flash insert");

        let error = validate_connections(&flowsheet).expect_err("expected duplicate sink error");

        assert_eq!(error.code().as_str(), "invalid_connection");
        assert_eq!(
            error.context().diagnostic_code(),
            Some("flowsheet.connection_validation.duplicate_downstream_sink")
        );
        assert!(error.message().contains("consumed by both"));
        assert_eq!(
            error.context().related_unit_ids(),
            &[UnitId::new("flash-1"), UnitId::new("mixer-1")]
        );
        assert_eq!(error.context().related_stream_ids(), &[StreamId::new("shared-stream")]);
        assert_eq!(
            error.context().related_port_targets(),
            &[
                DiagnosticPortTarget::new("flash-1", "inlet"),
                DiagnosticPortTarget::new("mixer-1", "inlet_a"),
            ]
        );
    }

    #[test]
    fn rejects_material_port_without_upstream_source() {
        let mut flowsheet = Flowsheet::new("demo");
        for stream_id in ["stream-feed-a", "stream-feed-b", "stream-out"] {
            flowsheet
                .insert_stream(build_stream(stream_id))
                .expect("expected stream insert");
        }
        flowsheet
            .insert_unit(build_mixer_node(
                "mixer-1",
                "Mixer",
                "stream-feed-a",
                "stream-feed-b",
                "stream-out",
            ))
            .expect("expected mixer insert");

        let error = validate_connections(&flowsheet).expect_err("expected missing source error");

        assert_eq!(error.code().as_str(), "invalid_connection");
        assert_eq!(
            error.context().diagnostic_code(),
            Some("flowsheet.connection_validation.missing_upstream_source")
        );
        assert!(
            error
                .message()
                .contains("missing an upstream outlet connection")
        );
        assert_eq!(error.context().related_unit_ids(), &[UnitId::new("mixer-1")]);
        assert_eq!(error.context().related_stream_ids(), &[StreamId::new("stream-feed-a")]);
        assert_eq!(
            error.context().related_port_targets(),
            &[DiagnosticPortTarget::new("mixer-1", "inlet_a")]
        );
    }

    #[test]
    fn rejects_unknown_unit_port_signature() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_stream(build_stream("stream-feed"))
            .expect("expected stream insert");
        flowsheet
            .insert_unit(UnitNode::new(
                "feed-1",
                "Feed",
                FEED_KIND,
                vec![UnitPort::new(
                    "unexpected",
                    PortDirection::Outlet,
                    PortKind::Material,
                    Some("stream-feed".into()),
                )],
            ))
            .expect("expected unit insert");

        let error = validate_connections(&flowsheet).expect_err("expected invalid unit error");

        assert_eq!(error.code().as_str(), "invalid_connection");
        assert_eq!(
            error.context().diagnostic_code(),
            Some("flowsheet.connection_validation.invalid_port_signature")
        );
        assert!(
            error
                .message()
                .contains("canonical built-in port signature")
        );
        assert_eq!(error.context().related_unit_ids(), &[UnitId::new("feed-1")]);
        assert!(error.context().related_stream_ids().is_empty());
    }
}
