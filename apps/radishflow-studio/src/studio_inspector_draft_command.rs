const UPDATE_STREAM_DRAFT_PREFIX: &str = "inspector.update_stream_draft:";
const COMMIT_STREAM_DRAFT_PREFIX: &str = "inspector.commit_stream_draft:";
const COMMIT_STREAM_DRAFTS_PREFIX: &str = "inspector.commit_stream_drafts:";
const DISCARD_STREAM_DRAFT_PREFIX: &str = "inspector.discard_stream_draft:";
const DISCARD_STREAM_DRAFTS_PREFIX: &str = "inspector.discard_stream_drafts:";
const NORMALIZE_STREAM_COMPOSITION_PREFIX: &str = "inspector.normalize_stream_composition:";
const ADD_STREAM_COMPOSITION_COMPONENT_PREFIX: &str = "inspector.add_stream_composition_component:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioInspectorDraftUpdateCommand {
    pub draft_key: String,
    pub raw_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioInspectorDraftCommitCommand {
    pub draft_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioInspectorDraftDiscardCommand {
    pub draft_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioInspectorDraftBatchCommitCommand {
    pub stream_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioInspectorDraftBatchDiscardCommand {
    pub stream_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioInspectorCompositionNormalizeCommand {
    pub stream_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioInspectorCompositionComponentAddCommand {
    pub stream_id: String,
    pub component_id: String,
}

impl StudioInspectorDraftUpdateCommand {
    pub fn new(draft_key: impl Into<String>, raw_value: impl Into<String>) -> Self {
        Self {
            draft_key: draft_key.into(),
            raw_value: raw_value.into(),
        }
    }
}

impl StudioInspectorDraftCommitCommand {
    pub fn new(draft_key: impl Into<String>) -> Self {
        Self {
            draft_key: draft_key.into(),
        }
    }
}

impl StudioInspectorDraftDiscardCommand {
    pub fn new(draft_key: impl Into<String>) -> Self {
        Self {
            draft_key: draft_key.into(),
        }
    }
}

impl StudioInspectorDraftBatchCommitCommand {
    pub fn new(stream_id: impl Into<String>) -> Self {
        Self {
            stream_id: stream_id.into(),
        }
    }
}

impl StudioInspectorDraftBatchDiscardCommand {
    pub fn new(stream_id: impl Into<String>) -> Self {
        Self {
            stream_id: stream_id.into(),
        }
    }
}

impl StudioInspectorCompositionNormalizeCommand {
    pub fn new(stream_id: impl Into<String>) -> Self {
        Self {
            stream_id: stream_id.into(),
        }
    }
}

impl StudioInspectorCompositionComponentAddCommand {
    pub fn new(stream_id: impl Into<String>, component_id: impl Into<String>) -> Self {
        Self {
            stream_id: stream_id.into(),
            component_id: component_id.into(),
        }
    }
}

pub fn inspector_draft_update_command_id(draft_key: &str) -> String {
    format!("{UPDATE_STREAM_DRAFT_PREFIX}{draft_key}")
}

pub fn inspector_draft_commit_command_id(draft_key: &str) -> String {
    format!("{COMMIT_STREAM_DRAFT_PREFIX}{draft_key}")
}

pub fn inspector_draft_batch_commit_command_id(stream_id: &str) -> String {
    format!("{COMMIT_STREAM_DRAFTS_PREFIX}stream:{stream_id}")
}

pub fn inspector_draft_discard_command_id(draft_key: &str) -> String {
    format!("{DISCARD_STREAM_DRAFT_PREFIX}{draft_key}")
}

pub fn inspector_draft_batch_discard_command_id(stream_id: &str) -> String {
    format!("{DISCARD_STREAM_DRAFTS_PREFIX}stream:{stream_id}")
}

pub fn inspector_composition_normalize_command_id(stream_id: &str) -> String {
    format!("{NORMALIZE_STREAM_COMPOSITION_PREFIX}stream:{stream_id}")
}

pub fn inspector_composition_component_add_command_id(
    stream_id: &str,
    component_id: &str,
) -> String {
    format!("{ADD_STREAM_COMPOSITION_COMPONENT_PREFIX}stream:{stream_id}:component:{component_id}")
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

pub fn inspector_draft_commit_command_from_id(
    command_id: &str,
) -> Option<StudioInspectorDraftCommitCommand> {
    command_id
        .strip_prefix(COMMIT_STREAM_DRAFT_PREFIX)
        .filter(|draft_key| !draft_key.is_empty())
        .map(StudioInspectorDraftCommitCommand::new)
}

pub fn inspector_draft_batch_commit_command_from_id(
    command_id: &str,
) -> Option<StudioInspectorDraftBatchCommitCommand> {
    command_id
        .strip_prefix(COMMIT_STREAM_DRAFTS_PREFIX)
        .and_then(|target| target.strip_prefix("stream:"))
        .filter(|stream_id| !stream_id.is_empty())
        .map(StudioInspectorDraftBatchCommitCommand::new)
}

pub fn inspector_draft_discard_command_from_id(
    command_id: &str,
) -> Option<StudioInspectorDraftDiscardCommand> {
    command_id
        .strip_prefix(DISCARD_STREAM_DRAFT_PREFIX)
        .filter(|draft_key| !draft_key.is_empty())
        .map(StudioInspectorDraftDiscardCommand::new)
}

pub fn inspector_draft_batch_discard_command_from_id(
    command_id: &str,
) -> Option<StudioInspectorDraftBatchDiscardCommand> {
    command_id
        .strip_prefix(DISCARD_STREAM_DRAFTS_PREFIX)
        .and_then(|target| target.strip_prefix("stream:"))
        .filter(|stream_id| !stream_id.is_empty())
        .map(StudioInspectorDraftBatchDiscardCommand::new)
}

pub fn inspector_composition_normalize_command_from_id(
    command_id: &str,
) -> Option<StudioInspectorCompositionNormalizeCommand> {
    command_id
        .strip_prefix(NORMALIZE_STREAM_COMPOSITION_PREFIX)
        .and_then(|target| target.strip_prefix("stream:"))
        .filter(|stream_id| !stream_id.is_empty())
        .map(StudioInspectorCompositionNormalizeCommand::new)
}

pub fn inspector_composition_component_add_command_from_id(
    command_id: &str,
) -> Option<StudioInspectorCompositionComponentAddCommand> {
    let target = command_id.strip_prefix(ADD_STREAM_COMPOSITION_COMPONENT_PREFIX)?;
    let (stream_id, component_id) = target.split_once(":component:")?;
    let stream_id = stream_id.strip_prefix("stream:")?;
    (!stream_id.is_empty() && !component_id.is_empty())
        .then(|| StudioInspectorCompositionComponentAddCommand::new(stream_id, component_id))
}

#[cfg(test)]
mod tests {
    use crate::{
        inspector_composition_component_add_command_from_id,
        inspector_composition_component_add_command_id,
        inspector_composition_normalize_command_from_id,
        inspector_composition_normalize_command_id, inspector_draft_batch_commit_command_from_id,
        inspector_draft_batch_commit_command_id, inspector_draft_batch_discard_command_from_id,
        inspector_draft_batch_discard_command_id, inspector_draft_commit_command_from_id,
        inspector_draft_commit_command_id, inspector_draft_discard_command_from_id,
        inspector_draft_discard_command_id, inspector_draft_update_command_from_id,
        inspector_draft_update_command_id,
    };

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

    #[test]
    fn inspector_draft_commit_command_round_trips_key() {
        let command_id = inspector_draft_commit_command_id("stream:stream-feed:temperature_k");

        let command =
            inspector_draft_commit_command_from_id(&command_id).expect("expected commit command");

        assert_eq!(
            command_id,
            "inspector.commit_stream_draft:stream:stream-feed:temperature_k"
        );
        assert_eq!(command.draft_key, "stream:stream-feed:temperature_k");
    }

    #[test]
    fn inspector_draft_commit_command_rejects_unknown_or_empty_command_id() {
        assert_eq!(
            inspector_draft_commit_command_from_id("inspector.commit_stream_draft:"),
            None
        );
        assert_eq!(
            inspector_draft_commit_command_from_id(
                "inspector.update_stream_draft:stream:stream-feed:temperature_k"
            ),
            None
        );
    }

    #[test]
    fn inspector_draft_discard_command_round_trips_key() {
        let command_id = inspector_draft_discard_command_id("stream:stream-feed:temperature_k");

        let command =
            inspector_draft_discard_command_from_id(&command_id).expect("expected discard command");

        assert_eq!(
            command_id,
            "inspector.discard_stream_draft:stream:stream-feed:temperature_k"
        );
        assert_eq!(command.draft_key, "stream:stream-feed:temperature_k");
    }

    #[test]
    fn inspector_draft_discard_command_rejects_unknown_or_empty_command_id() {
        assert_eq!(
            inspector_draft_discard_command_from_id("inspector.discard_stream_draft:"),
            None
        );
        assert_eq!(
            inspector_draft_discard_command_from_id(
                "inspector.commit_stream_draft:stream:stream-feed:temperature_k"
            ),
            None
        );
    }

    #[test]
    fn inspector_draft_batch_commit_command_round_trips_stream() {
        let command_id = inspector_draft_batch_commit_command_id("stream-feed");

        let command = inspector_draft_batch_commit_command_from_id(&command_id)
            .expect("expected batch commit command");

        assert_eq!(
            command_id,
            "inspector.commit_stream_drafts:stream:stream-feed"
        );
        assert_eq!(command.stream_id, "stream-feed");
    }

    #[test]
    fn inspector_draft_batch_commit_command_rejects_unknown_or_empty_command_id() {
        assert_eq!(
            inspector_draft_batch_commit_command_from_id("inspector.commit_stream_drafts:stream:"),
            None
        );
        assert_eq!(
            inspector_draft_batch_commit_command_from_id(
                "inspector.commit_stream_draft:stream:stream-feed:temperature_k"
            ),
            None
        );
    }

    #[test]
    fn inspector_draft_batch_discard_command_round_trips_stream() {
        let command_id = inspector_draft_batch_discard_command_id("stream-feed");

        let command = inspector_draft_batch_discard_command_from_id(&command_id)
            .expect("expected batch discard command");

        assert_eq!(
            command_id,
            "inspector.discard_stream_drafts:stream:stream-feed"
        );
        assert_eq!(command.stream_id, "stream-feed");
    }

    #[test]
    fn inspector_draft_batch_discard_command_rejects_unknown_or_empty_command_id() {
        assert_eq!(
            inspector_draft_batch_discard_command_from_id(
                "inspector.discard_stream_drafts:stream:"
            ),
            None
        );
        assert_eq!(
            inspector_draft_batch_discard_command_from_id(
                "inspector.commit_stream_drafts:stream:stream-feed"
            ),
            None
        );
    }

    #[test]
    fn inspector_composition_normalize_command_round_trips_stream() {
        let command_id = inspector_composition_normalize_command_id("stream-feed");

        let command = inspector_composition_normalize_command_from_id(&command_id)
            .expect("expected normalize command");

        assert_eq!(
            command_id,
            "inspector.normalize_stream_composition:stream:stream-feed"
        );
        assert_eq!(command.stream_id, "stream-feed");
    }

    #[test]
    fn inspector_composition_normalize_command_rejects_unknown_or_empty_command_id() {
        assert_eq!(
            inspector_composition_normalize_command_from_id(
                "inspector.normalize_stream_composition:stream:"
            ),
            None
        );
        assert_eq!(
            inspector_composition_normalize_command_from_id(
                "inspector.commit_stream_drafts:stream:stream-feed"
            ),
            None
        );
    }

    #[test]
    fn inspector_composition_component_add_command_round_trips_stream_and_component() {
        let command_id =
            inspector_composition_component_add_command_id("stream-feed", "component-c");

        let command = inspector_composition_component_add_command_from_id(&command_id)
            .expect("expected component add command");

        assert_eq!(
            command_id,
            "inspector.add_stream_composition_component:stream:stream-feed:component:component-c"
        );
        assert_eq!(command.stream_id, "stream-feed");
        assert_eq!(command.component_id, "component-c");
    }

    #[test]
    fn inspector_composition_component_add_command_rejects_unknown_or_empty_command_id() {
        assert_eq!(
            inspector_composition_component_add_command_from_id(
                "inspector.add_stream_composition_component:stream:stream-feed:component:"
            ),
            None
        );
        assert_eq!(
            inspector_composition_component_add_command_from_id(
                "inspector.add_stream_composition_component:stream::component:component-c"
            ),
            None
        );
        assert_eq!(
            inspector_composition_component_add_command_from_id(
                "inspector.normalize_stream_composition:stream:stream-feed"
            ),
            None
        );
    }
}
