const UPDATE_STREAM_DRAFT_PREFIX: &str = "inspector.update_stream_draft:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioInspectorDraftUpdateCommand {
    pub draft_key: String,
    pub raw_value: String,
}

impl StudioInspectorDraftUpdateCommand {
    pub fn new(draft_key: impl Into<String>, raw_value: impl Into<String>) -> Self {
        Self {
            draft_key: draft_key.into(),
            raw_value: raw_value.into(),
        }
    }
}

pub fn inspector_draft_update_command_id(draft_key: &str) -> String {
    format!("{UPDATE_STREAM_DRAFT_PREFIX}{draft_key}")
}

pub fn inspector_draft_update_command_from_id(
    command_id: &str,
    raw_value: impl Into<String>,
) -> Option<StudioInspectorDraftUpdateCommand> {
    command_id
        .strip_prefix(UPDATE_STREAM_DRAFT_PREFIX)
        .filter(|draft_key| !draft_key.is_empty())
        .map(|draft_key| StudioInspectorDraftUpdateCommand::new(draft_key, raw_value))
}

#[cfg(test)]
mod tests {
    use crate::{inspector_draft_update_command_from_id, inspector_draft_update_command_id};

    #[test]
    fn inspector_draft_update_command_round_trips_key_and_value() {
        let command_id = inspector_draft_update_command_id("stream:stream-feed:temperature_k");

        let command = inspector_draft_update_command_from_id(&command_id, "333.5")
            .expect("expected draft update command");

        assert_eq!(
            command_id,
            "inspector.update_stream_draft:stream:stream-feed:temperature_k"
        );
        assert_eq!(command.draft_key, "stream:stream-feed:temperature_k");
        assert_eq!(command.raw_value, "333.5");
    }

    #[test]
    fn inspector_draft_update_command_rejects_unknown_or_empty_command_id() {
        assert_eq!(
            inspector_draft_update_command_from_id("inspector.update_stream_draft:", "333.5"),
            None
        );
        assert_eq!(
            inspector_draft_update_command_from_id("inspector.focus_stream:stream-feed", "333.5"),
            None
        );
    }
}
