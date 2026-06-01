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

pub(crate) fn stream_reconnect_from_candidates(
    stream_id: &str,
    missing_side: StudioStreamReconnectMissingSide,
    candidate_count: usize,
    cycle_blocked_count: usize,
    available_targets: Vec<String>,
) -> StudioStreamReconnectAvailability {
    match available_targets.len() {
        1 => StudioStreamReconnectAvailability::Available {
            detail: format!(
                "Reconnect selected {} stream `{}` to the only available {} `{}`.",
                missing_side.stream_shape(),
                stream_id,
                missing_side.endpoint_kind(),
                available_targets[0]
            ),
        },
        0 if candidate_count == 0 => StudioStreamReconnectAvailability::Unavailable {
            reason: format!("there is no available {}", missing_side.endpoint_kind()),
        },
        0 => StudioStreamReconnectAvailability::Unavailable {
            reason: cycle_blocked_reason(missing_side, cycle_blocked_count),
        },
        count => StudioStreamReconnectAvailability::Unavailable {
            reason: format!(
                "there are {count} available {}; use suggestions or resolve the ambiguous target first",
                missing_side.endpoint_kind_plural()
            ),
        },
    }
}

pub(crate) fn stream_reconnect_already_connected(
    sink_endpoint_label: impl AsRef<str>,
) -> StudioStreamReconnectAvailability {
    StudioStreamReconnectAvailability::Unavailable {
        reason: format!(
            "it already has both material endpoints; disconnect source or sink before reconnecting (current sink `{}`)",
            sink_endpoint_label.as_ref()
        ),
    }
}

pub(crate) fn stream_reconnect_missing_endpoint() -> StudioStreamReconnectAvailability {
    StudioStreamReconnectAvailability::Unavailable {
        reason: "the stream has no material endpoint to reconnect".to_string(),
    }
}

pub(crate) fn stream_reconnect_missing_stream() -> StudioStreamReconnectAvailability {
    StudioStreamReconnectAvailability::Unavailable {
        reason: "the selected stream is no longer present in the current document".to_string(),
    }
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
