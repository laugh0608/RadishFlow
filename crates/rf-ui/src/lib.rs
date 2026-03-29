mod commands;
mod diagnostics;
mod ids;
mod run;
mod state;

pub use commands::{
    CanvasPoint, CommandHistory, CommandHistoryEntry, CommandValue, DocumentCommand,
};
pub use diagnostics::{DiagnosticSeverity, DiagnosticSnapshot, DiagnosticSummary};
pub use ids::{DocumentId, SolveSnapshotId};
pub use run::{
    RunStatus, SimulationMode, SolvePendingReason, SolveSessionState, SolveSnapshot, StepSnapshot,
    StreamStateSnapshot, UnitExecutionSnapshot,
};
pub use state::{
    AppLogEntry, AppLogFeed, AppLogLevel, AppState, AppTheme, DateTimeUtc, DocumentMetadata,
    DraftValidationState, DraftValue, FieldDraft, FlowsheetDocument, InspectorDraftState,
    InspectorTarget, LocaleCode, PanelLayoutPreferences, SelectionState, UiPanelsState,
    UserPreferences, WorkspaceState, latest_snapshot_id,
};

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_types::UnitId;

    use crate::{
        AppState, CanvasPoint, CommandHistory, CommandHistoryEntry, DiagnosticSeverity,
        DiagnosticSummary, DocumentCommand, DocumentMetadata, FlowsheetDocument, RunStatus,
        SimulationMode, SolvePendingReason, SolveSnapshot,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new("doc-1", "Demo", timestamp(10));
        FlowsheetDocument::new(flowsheet, metadata)
    }

    #[test]
    fn command_history_truncates_redo_tail_when_recording_new_command() {
        let mut history = CommandHistory::new();
        history.record(CommandHistoryEntry::new(
            1,
            DocumentCommand::DeleteUnit {
                unit_id: UnitId::new("u-1"),
            },
        ));
        history.record(CommandHistoryEntry::new(
            2,
            DocumentCommand::DeleteUnit {
                unit_id: UnitId::new("u-2"),
            },
        ));

        let undone = history.undo().expect("expected undo entry");
        assert_eq!(undone.revision, 2);
        assert!(history.can_redo());

        history.record(CommandHistoryEntry::new(
            3,
            DocumentCommand::DeleteUnit {
                unit_id: UnitId::new("u-3"),
            },
        ));

        assert_eq!(history.len(), 2);
        assert!(!history.can_redo());
        assert_eq!(history.current_entry().map(|entry| entry.revision), Some(3));
    }

    #[test]
    fn commit_document_change_advances_revision_and_marks_solve_pending() {
        let mut app_state = AppState::new(sample_document());
        let next_flowsheet = Flowsheet::new("demo-updated");

        let revision = app_state.commit_document_change(
            DocumentCommand::MoveUnit {
                unit_id: UnitId::new("heater-1"),
                position: CanvasPoint::new(120.0, 80.0),
            },
            next_flowsheet,
            timestamp(20),
        );

        assert_eq!(revision, 1);
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(app_state.workspace.command_history.len(), 1);
        assert_eq!(app_state.workspace.solve_session.observed_revision, 1);
        assert_eq!(app_state.workspace.solve_session.status, RunStatus::Dirty);
        assert_eq!(
            app_state.workspace.solve_session.pending_reason,
            Some(SolvePendingReason::DocumentRevisionAdvanced)
        );
        assert_eq!(
            app_state.workspace.document.metadata.updated_at,
            timestamp(20)
        );
    }

    #[test]
    fn storing_snapshot_respects_history_limit_and_updates_latest_reference() {
        let mut app_state = AppState::new(sample_document());
        app_state.preferences.snapshot_history_limit = 2;

        for sequence in 1..=3 {
            let snapshot = SolveSnapshot::new(
                format!("snapshot-{sequence}"),
                1,
                sequence,
                RunStatus::Converged,
                DiagnosticSummary::new(1, DiagnosticSeverity::Info, "ok"),
            );
            app_state.store_snapshot(snapshot);
        }

        assert_eq!(app_state.workspace.snapshot_history.len(), 2);
        assert_eq!(
            app_state
                .workspace
                .snapshot_history
                .front()
                .map(|snapshot| snapshot.sequence),
            Some(2)
        );
        assert_eq!(
            app_state
                .workspace
                .solve_session
                .latest_snapshot
                .as_ref()
                .map(|id| id.as_str()),
            Some("snapshot-3")
        );
        assert_eq!(app_state.workspace.solve_session.pending_reason, None);
    }

    #[test]
    fn switching_to_active_sets_mode_activation_pending_reason() {
        let mut app_state = AppState::new(sample_document());
        app_state.set_simulation_mode(SimulationMode::Active);

        assert_eq!(
            app_state.workspace.solve_session.mode,
            SimulationMode::Active
        );
        assert_eq!(
            app_state.workspace.solve_session.pending_reason,
            Some(SolvePendingReason::ModeActivated)
        );
    }
}
