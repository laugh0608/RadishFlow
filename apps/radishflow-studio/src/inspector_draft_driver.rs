use rf_types::{RfError, RfResult};
use rf_ui::{AppState, InspectorTarget};

use crate::StudioInspectorDraftUpdateCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorDraftUpdateOutcome {
    pub command: StudioInspectorDraftUpdateCommand,
    pub applied: bool,
    pub active_target: Option<InspectorTarget>,
    pub document_revision: u64,
    pub command_history_len: usize,
}

pub fn update_inspector_draft(
    app_state: &mut AppState,
    command: StudioInspectorDraftUpdateCommand,
) -> RfResult<InspectorDraftUpdateOutcome> {
    let (stream_id, field) = rf_ui::stream_inspector_draft_key_parts(&command.draft_key)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "inspector draft key `{}` is not a supported stream field",
                command.draft_key
            ))
        })?;
    let applied = app_state
        .update_stream_inspector_draft(&stream_id, field, command.raw_value.clone())
        .is_some();

    Ok(InspectorDraftUpdateOutcome {
        command,
        applied,
        active_target: app_state.workspace.drafts.active_target.clone(),
        document_revision: app_state.workspace.document.revision,
        command_history_len: app_state.workspace.command_history.len(),
    })
}

#[cfg(test)]
mod tests {
    use rf_model::{Flowsheet, MaterialStreamState};
    use rf_types::StreamId;
    use rf_ui::{AppState, DocumentMetadata, FlowsheetDocument, InspectorTarget};

    use crate::{StudioInspectorDraftUpdateCommand, update_inspector_draft};

    #[test]
    fn inspector_draft_driver_updates_active_stream_draft_without_document_mutation() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_stream(MaterialStreamState::new("stream-feed", "Feed stream"))
            .expect("expected stream insert");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Demo", std::time::UNIX_EPOCH),
        ));
        app_state.focus_inspector_target(InspectorTarget::Stream(StreamId::new("stream-feed")));

        let outcome = update_inspector_draft(
            &mut app_state,
            StudioInspectorDraftUpdateCommand::new("stream:stream-feed:temperature_k", "333.5"),
        )
        .expect("expected draft update");

        assert!(outcome.applied);
        assert_eq!(outcome.document_revision, 0);
        assert_eq!(outcome.command_history_len, 0);
        assert_eq!(
            app_state
                .workspace
                .drafts
                .fields
                .get(&outcome.command.draft_key),
            Some(&rf_ui::DraftValue::Number(rf_ui::FieldDraft {
                original: "298.15".to_string(),
                current: "333.5".to_string(),
                is_dirty: true,
                validation: rf_ui::DraftValidationState::Valid,
            }))
        );
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-feed")]
                .temperature_k,
            298.15
        );
    }

    #[test]
    fn inspector_draft_driver_reports_stale_inactive_stream_without_mutation() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_stream(MaterialStreamState::new("stream-feed", "Feed stream"))
            .expect("expected stream insert");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Demo", std::time::UNIX_EPOCH),
        ));

        let outcome = update_inspector_draft(
            &mut app_state,
            StudioInspectorDraftUpdateCommand::new("stream:stream-feed:name", "Edited"),
        )
        .expect("expected stale update outcome");

        assert!(!outcome.applied);
        assert_eq!(outcome.document_revision, 0);
        assert_eq!(outcome.command_history_len, 0);
        assert!(app_state.workspace.drafts.fields.is_empty());
    }
}
