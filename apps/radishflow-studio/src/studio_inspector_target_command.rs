use rf_types::{StreamId, UnitId};
use rf_ui::InspectorTarget;

const FOCUS_UNIT_PREFIX: &str = "inspector.focus_unit:";
const FOCUS_STREAM_PREFIX: &str = "inspector.focus_stream:";

pub fn inspector_target_command_id(target: &InspectorTarget) -> String {
    match target {
        InspectorTarget::Unit(unit_id) => format!("{FOCUS_UNIT_PREFIX}{}", unit_id.as_str()),
        InspectorTarget::Stream(stream_id) => {
            format!("{FOCUS_STREAM_PREFIX}{}", stream_id.as_str())
        }
    }
}

pub fn inspector_target_from_command_id(command_id: &str) -> Option<InspectorTarget> {
    command_id
        .strip_prefix(FOCUS_UNIT_PREFIX)
        .filter(|target_id| !target_id.is_empty())
        .map(|target_id| InspectorTarget::Unit(UnitId::new(target_id)))
        .or_else(|| {
            command_id
                .strip_prefix(FOCUS_STREAM_PREFIX)
                .filter(|target_id| !target_id.is_empty())
                .map(|target_id| InspectorTarget::Stream(StreamId::new(target_id)))
        })
}

#[cfg(test)]
mod tests {
    use rf_types::{StreamId, UnitId};
    use rf_ui::InspectorTarget;

    use crate::{inspector_target_command_id, inspector_target_from_command_id};

    #[test]
    fn inspector_target_command_round_trips_unit() {
        let target = InspectorTarget::Unit(UnitId::new("heater-1"));

        let command_id = inspector_target_command_id(&target);

        assert_eq!(command_id, "inspector.focus_unit:heater-1");
        assert_eq!(inspector_target_from_command_id(&command_id), Some(target));
    }

    #[test]
    fn inspector_target_command_round_trips_stream() {
        let target = InspectorTarget::Stream(StreamId::new("stream-heated"));

        let command_id = inspector_target_command_id(&target);

        assert_eq!(command_id, "inspector.focus_stream:stream-heated");
        assert_eq!(inspector_target_from_command_id(&command_id), Some(target));
    }

    #[test]
    fn inspector_target_command_rejects_unknown_or_empty_command_id() {
        assert_eq!(
            inspector_target_from_command_id("inspector.focus_unit:"),
            None
        );
        assert_eq!(
            inspector_target_from_command_id("run_panel.run_manual"),
            None
        );
    }
}
