#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StudioStreamReconnectAvailability {
    Available { detail: String },
    Unavailable { reason: String },
}

impl StudioStreamReconnectAvailability {
    pub(crate) fn is_available(&self) -> bool {
        matches!(self, Self::Available { .. })
    }

    pub(crate) fn action_detail(&self, stream_id: &str) -> String {
        match self {
            Self::Available { detail } => detail.clone(),
            Self::Unavailable { reason } => {
                format!("Cannot reconnect selected stream `{stream_id}`: {reason}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StudioStreamReconnectMissingSide {
    Source,
    Sink,
}

impl StudioStreamReconnectMissingSide {
    fn endpoint_kind(self) -> &'static str {
        match self {
            Self::Source => "material outlet",
            Self::Sink => "material inlet",
        }
    }

    fn endpoint_kind_plural(self) -> &'static str {
        match self {
            Self::Source => "material outlets",
            Self::Sink => "material inlets",
        }
    }

    fn stream_shape(self) -> &'static str {
        match self {
            Self::Source => "sink-only",
            Self::Sink => "source-only",
        }
    }
}

pub(crate) fn stream_reconnect_availability(
    stream_id: &str,
    status: &rf_ui::StreamReconnect,
) -> StudioStreamReconnectAvailability {
    use rf_ui::StreamReconnect;
    let reason = match status {
        StreamReconnect::Missing => {
            "the selected stream is no longer present in the current document".into()
        }
        StreamReconnect::Unbound => "the stream has no material endpoint to reconnect".into(),
        StreamReconnect::InvalidBindings => {
            "the stream has duplicate material endpoint bindings".into()
        }
        StreamReconnect::Connected { sink } => format!(
            "it already has both material endpoints; disconnect source or sink before reconnecting (current sink `{}:{}`)",
            sink.unit_id, sink.port
        ),
        StreamReconnect::Open(options) => {
            let side = match options.missing {
                rf_types::PortDirection::Outlet => StudioStreamReconnectMissingSide::Source,
                rf_types::PortDirection::Inlet => StudioStreamReconnectMissingSide::Sink,
            };
            match options.candidates.as_slice() {
                [port] => {
                    return StudioStreamReconnectAvailability::Available {
                        detail: format!(
                            "Reconnect selected {} stream `{}` to the only available {} `{}:{}`.",
                            side.stream_shape(),
                            stream_id,
                            side.endpoint_kind(),
                            port.unit_id,
                            port.port
                        ),
                    };
                }
                [] => {
                    let mut reason = if options.cycle_blocked > 0 {
                        cycle_blocked_reason(side, options.cycle_blocked)
                    } else {
                        format!("there is no available {}", side.endpoint_kind())
                    };
                    if options.occupied > 0 {
                        reason.push_str(&format!(" ({} occupied)", options.occupied));
                    }
                    reason
                }
                ports => format!(
                    "there are {} available {}; choose a port in the stream inspector",
                    ports.len(),
                    side.endpoint_kind_plural()
                ),
            }
        }
    };
    StudioStreamReconnectAvailability::Unavailable { reason }
}

fn cycle_blocked_reason(
    missing_side: StudioStreamReconnectMissingSide,
    cycle_blocked_count: usize,
) -> String {
    if cycle_blocked_count <= 1 {
        return format!(
            "the only available {} would create a unit dependency cycle",
            missing_side.endpoint_kind()
        );
    }
    format!(
        "all {cycle_blocked_count} available {} would create unit dependency cycles",
        missing_side.endpoint_kind_plural()
    )
}
