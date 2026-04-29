use std::time::SystemTime;

use rf_types::RfResult;
use rf_ui::{AppState, DocumentHistoryDirection, InspectorTarget};

use crate::StudioDocumentHistoryCommand;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentHistoryOutcome {
    pub command: StudioDocumentHistoryCommand,
    pub direction: DocumentHistoryDirection,
    pub applied: bool,
    pub active_target: Option<InspectorTarget>,
    pub document_revision: u64,
    pub command_history_cursor: usize,
    pub command_history_len: usize,
}

pub fn dispatch_document_history(
    app_state: &mut AppState,
    command: StudioDocumentHistoryCommand,
) -> RfResult<DocumentHistoryOutcome> {
    dispatch_document_history_at(app_state, command, SystemTime::now())
}

pub fn dispatch_document_history_at(
    app_state: &mut AppState,
    command: StudioDocumentHistoryCommand,
    changed_at: rf_ui::DateTimeUtc,
) -> RfResult<DocumentHistoryOutcome> {
    let direction = match command {
        StudioDocumentHistoryCommand::Undo => DocumentHistoryDirection::Undo,
        StudioDocumentHistoryCommand::Redo => DocumentHistoryDirection::Redo,
    };
    let applied = match command {
        StudioDocumentHistoryCommand::Undo => app_state.undo_document_command(changed_at)?,
        StudioDocumentHistoryCommand::Redo => app_state.redo_document_command(changed_at)?,
    }
    .is_some();

    Ok(DocumentHistoryOutcome {
        command,
        direction,
        applied,
        active_target: app_state.workspace.drafts.active_target.clone(),
        document_revision: app_state.workspace.document.revision,
        command_history_cursor: app_state.workspace.command_history.cursor,
        command_history_len: app_state.workspace.command_history.len(),
    })
}

#[cfg(test)]
mod tests {
    use rf_model::{Flowsheet, MaterialStreamState};
    use rf_types::StreamId;
    use rf_ui::{AppState, DocumentMetadata, FlowsheetDocument, InspectorTarget};

    use crate::{
        StudioDocumentHistoryCommand, StudioInspectorDraftCommitCommand,
        StudioInspectorDraftUpdateCommand, commit_inspector_draft_at, dispatch_document_history_at,
        update_inspector_draft,
    };

    #[test]
    fn document_history_driver_undoes_and_redoes_committed_stream_draft() {
        let stream_id = StreamId::new("stream-feed");
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_stream(MaterialStreamState::new(stream_id.clone(), "Feed stream"))
            .expect("expected stream insert");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Demo", std::time::UNIX_EPOCH),
        ));
        app_state.focus_inspector_target(InspectorTarget::Stream(stream_id.clone()));
        update_inspector_draft(
            &mut app_state,
            StudioInspectorDraftUpdateCommand::new("stream:stream-feed:temperature_k", "333.5"),
        )
        .expect("expected draft update");
        commit_inspector_draft_at(
            &mut app_state,
            StudioInspectorDraftCommitCommand::new("stream:stream-feed:temperature_k"),
            std::time::UNIX_EPOCH,
        )
        .expect("expected draft commit");

        let undo = dispatch_document_history_at(
            &mut app_state,
            StudioDocumentHistoryCommand::Undo,
            std::time::UNIX_EPOCH,
        )
        .expect("expected undo");

        assert!(undo.applied);
        assert_eq!(undo.document_revision, 2);
        assert_eq!(undo.command_history_cursor, 0);
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
            298.15
        );

        let redo = dispatch_document_history_at(
            &mut app_state,
            StudioDocumentHistoryCommand::Redo,
            std::time::UNIX_EPOCH,
        )
        .expect("expected redo");

        assert!(redo.applied);
        assert_eq!(redo.document_revision, 3);
        assert_eq!(redo.command_history_cursor, 1);
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
            333.5
        );
    }
}
