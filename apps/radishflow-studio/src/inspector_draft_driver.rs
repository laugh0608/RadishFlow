use std::time::SystemTime;

use rf_types::{RfError, RfResult};
use rf_ui::{AppState, InspectorTarget};

use crate::{
    StudioInspectorDraftBatchCommitCommand, StudioInspectorDraftCommitCommand,
    StudioInspectorDraftUpdateCommand,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorDraftUpdateOutcome {
    pub command: StudioInspectorDraftUpdateCommand,
    pub applied: bool,
    pub active_target: Option<InspectorTarget>,
    pub document_revision: u64,
    pub command_history_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorDraftCommitOutcome {
    pub command: StudioInspectorDraftCommitCommand,
    pub applied: bool,
    pub active_target: Option<InspectorTarget>,
    pub document_revision: u64,
    pub command_history_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorDraftBatchCommitOutcome {
    pub command: StudioInspectorDraftBatchCommitCommand,
    pub applied: bool,
    pub committed_keys: Vec<String>,
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

pub fn commit_inspector_draft(
    app_state: &mut AppState,
    command: StudioInspectorDraftCommitCommand,
) -> RfResult<InspectorDraftCommitOutcome> {
    commit_inspector_draft_at(app_state, command, SystemTime::now())
}

pub fn commit_inspector_draft_at(
    app_state: &mut AppState,
    command: StudioInspectorDraftCommitCommand,
    changed_at: rf_ui::DateTimeUtc,
) -> RfResult<InspectorDraftCommitOutcome> {
    let (stream_id, field) = rf_ui::stream_inspector_draft_key_parts(&command.draft_key)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "inspector draft key `{}` is not a supported stream field",
                command.draft_key
            ))
        })?;
    let applied = app_state
        .commit_stream_inspector_draft(&stream_id, field, changed_at)?
        .is_some();

    Ok(InspectorDraftCommitOutcome {
        command,
        applied,
        active_target: app_state.workspace.drafts.active_target.clone(),
        document_revision: app_state.workspace.document.revision,
        command_history_len: app_state.workspace.command_history.len(),
    })
}

pub fn commit_inspector_drafts(
    app_state: &mut AppState,
    command: StudioInspectorDraftBatchCommitCommand,
) -> RfResult<InspectorDraftBatchCommitOutcome> {
    commit_inspector_drafts_at(app_state, command, SystemTime::now())
}

pub fn commit_inspector_drafts_at(
    app_state: &mut AppState,
    command: StudioInspectorDraftBatchCommitCommand,
    changed_at: rf_ui::DateTimeUtc,
) -> RfResult<InspectorDraftBatchCommitOutcome> {
    let stream_id = rf_types::StreamId::new(command.stream_id.clone());
    let result = app_state.commit_stream_inspector_drafts(&stream_id, changed_at)?;
    let committed_keys = result
        .as_ref()
        .map(|result| result.keys.clone())
        .unwrap_or_default();
    let applied = result.is_some();

    Ok(InspectorDraftBatchCommitOutcome {
        command,
        applied,
        committed_keys,
        active_target: app_state.workspace.drafts.active_target.clone(),
        document_revision: app_state.workspace.document.revision,
        command_history_len: app_state.workspace.command_history.len(),
    })
}

#[cfg(test)]
mod tests {
    use rf_model::{Flowsheet, MaterialStreamState};
    use rf_types::{ComponentId, StreamId};
    use rf_ui::{
        AppState, CommandValue, DocumentCommand, DocumentMetadata, FlowsheetDocument,
        InspectorTarget, SolvePendingReason,
    };

    use crate::{
        StudioInspectorDraftBatchCommitCommand, StudioInspectorDraftCommitCommand,
        StudioInspectorDraftUpdateCommand, commit_inspector_draft_at, commit_inspector_drafts_at,
        update_inspector_draft,
    };

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

    #[test]
    fn inspector_draft_driver_commits_active_stream_draft_into_document_command() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_stream(MaterialStreamState::new("stream-feed", "Feed stream"))
            .expect("expected stream insert");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Demo", std::time::UNIX_EPOCH),
        ));
        app_state.focus_inspector_target(InspectorTarget::Stream(StreamId::new("stream-feed")));
        update_inspector_draft(
            &mut app_state,
            StudioInspectorDraftUpdateCommand::new("stream:stream-feed:temperature_k", "333.5"),
        )
        .expect("expected draft update");

        let outcome = commit_inspector_draft_at(
            &mut app_state,
            StudioInspectorDraftCommitCommand::new("stream:stream-feed:temperature_k"),
            std::time::UNIX_EPOCH,
        )
        .expect("expected draft commit");

        assert!(outcome.applied);
        assert_eq!(outcome.document_revision, 1);
        assert_eq!(outcome.command_history_len, 1);
        assert_eq!(
            outcome.active_target,
            Some(InspectorTarget::Stream(StreamId::new("stream-feed")))
        );
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::SetStreamSpecification {
                stream_id: StreamId::new("stream-feed"),
                field: "temperature_k".to_string(),
                value: CommandValue::Number(333.5),
            })
        );
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-feed")]
                .temperature_k,
            333.5
        );
        assert_eq!(
            app_state.workspace.solve_session.pending_reason,
            Some(SolvePendingReason::DocumentRevisionAdvanced)
        );
        assert!(app_state.workspace.drafts.fields.is_empty());
    }

    #[test]
    fn inspector_draft_driver_ignores_invalid_commit_without_document_mutation() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_stream(MaterialStreamState::new("stream-feed", "Feed stream"))
            .expect("expected stream insert");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Demo", std::time::UNIX_EPOCH),
        ));
        app_state.focus_inspector_target(InspectorTarget::Stream(StreamId::new("stream-feed")));
        update_inspector_draft(
            &mut app_state,
            StudioInspectorDraftUpdateCommand::new("stream:stream-feed:pressure_pa", "bad"),
        )
        .expect("expected invalid draft update");

        let outcome = commit_inspector_draft_at(
            &mut app_state,
            StudioInspectorDraftCommitCommand::new("stream:stream-feed:pressure_pa"),
            std::time::UNIX_EPOCH,
        )
        .expect("expected ignored commit");

        assert!(!outcome.applied);
        assert_eq!(outcome.document_revision, 0);
        assert_eq!(outcome.command_history_len, 0);
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-feed")]
                .pressure_pa,
            101_325.0
        );
        assert!(
            app_state
                .workspace
                .drafts
                .fields
                .contains_key("stream:stream-feed:pressure_pa")
        );
    }

    #[test]
    fn inspector_draft_driver_commits_stream_drafts_as_one_history_entry() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_stream(MaterialStreamState::new("stream-feed", "Feed stream"))
            .expect("expected stream insert");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Demo", std::time::UNIX_EPOCH),
        ));
        app_state.focus_inspector_target(InspectorTarget::Stream(StreamId::new("stream-feed")));
        update_inspector_draft(
            &mut app_state,
            StudioInspectorDraftUpdateCommand::new("stream:stream-feed:temperature_k", "333.5"),
        )
        .expect("expected temperature draft update");
        update_inspector_draft(
            &mut app_state,
            StudioInspectorDraftUpdateCommand::new("stream:stream-feed:pressure_pa", "202650"),
        )
        .expect("expected pressure draft update");

        let outcome = commit_inspector_drafts_at(
            &mut app_state,
            StudioInspectorDraftBatchCommitCommand::new("stream-feed"),
            std::time::UNIX_EPOCH,
        )
        .expect("expected batch commit");

        assert!(outcome.applied);
        assert_eq!(outcome.document_revision, 1);
        assert_eq!(outcome.command_history_len, 1);
        assert_eq!(
            outcome.committed_keys,
            vec![
                "stream:stream-feed:temperature_k".to_string(),
                "stream:stream-feed:pressure_pa".to_string()
            ]
        );
        let stream = &app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-feed")];
        assert_eq!(stream.temperature_k, 333.5);
        assert_eq!(stream.pressure_pa, 202650.0);
        assert!(app_state.workspace.drafts.fields.is_empty());
    }

    #[test]
    fn inspector_draft_driver_commits_composition_draft_through_command_id() {
        let mut flowsheet = Flowsheet::new("demo");
        flowsheet
            .insert_stream(MaterialStreamState::from_tpzf(
                "stream-feed",
                "Feed stream",
                298.15,
                101_325.0,
                1.0,
                [
                    (ComponentId::new("component-a"), 0.4),
                    (ComponentId::new("component-b"), 0.6),
                ]
                .into_iter()
                .collect(),
            ))
            .expect("expected stream insert");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc", "Demo", std::time::UNIX_EPOCH),
        ));
        app_state.focus_inspector_target(InspectorTarget::Stream(StreamId::new("stream-feed")));
        update_inspector_draft(
            &mut app_state,
            StudioInspectorDraftUpdateCommand::new(
                "stream:stream-feed:overall_mole_fraction:component-a",
                "0.25",
            ),
        )
        .expect("expected composition draft update");

        let outcome = commit_inspector_draft_at(
            &mut app_state,
            StudioInspectorDraftCommitCommand::new(
                "stream:stream-feed:overall_mole_fraction:component-a",
            ),
            std::time::UNIX_EPOCH,
        )
        .expect("expected composition draft commit");

        assert!(outcome.applied);
        assert_eq!(outcome.document_revision, 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::SetStreamSpecification {
                stream_id: StreamId::new("stream-feed"),
                field: "overall_mole_fraction:component-a".to_string(),
                value: CommandValue::Number(0.25),
            })
        );
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-feed")]
                .overall_mole_fractions[&ComponentId::new("component-a")],
            0.25
        );
    }
}
