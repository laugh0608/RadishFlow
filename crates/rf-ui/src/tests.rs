use std::time::{Duration, UNIX_EPOCH};

use rf_flash::PlaceholderTpFlashSolver;
use rf_model::{Flowsheet, MaterialStreamState, UnitNode, UnitPort};
use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolverServices};
use rf_thermo::{AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem};
use rf_types::{ComponentId, PortDirection, PortKind, StreamId, UnitId};
use rf_unitops::{BuiltinUnitKind, builtin_unit_spec};

use crate::{
    AppLogLevel, AppState, AuthSessionStatus, AuthenticatedUser, CanvasEditIntent, CanvasPoint,
    CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding, CanvasSuggestion,
    CanvasSuggestionAcceptance, CanvasSuggestionId, CanvasViewMode, CommandHistory,
    CommandHistoryEntry, CommandValue, DiagnosticSeverity, DiagnosticSummary, DocumentCommand,
    DocumentMetadata, EntitlementActionId, EntitlementPanelState, EntitlementPanelWidgetEvent,
    EntitlementPanelWidgetModel, EntitlementSnapshot, FlowsheetDocument, GhostElement,
    GhostElementKind, InspectorTarget, OfflineLeaseRefreshResponse, PropertyPackageManifest,
    PropertyPackageManifestList, PropertyPackageSource, RunPanelActionId, RunPanelActionProminence,
    RunPanelPresentation, RunPanelRecoveryWidgetEvent, RunPanelState, RunPanelTextView,
    RunPanelViewModel, RunPanelWidgetEvent, RunPanelWidgetModel, RunStatus, SecureCredentialHandle,
    SimulationMode, SolvePendingReason, SolveSnapshot, StreamVisualKind, StreamVisualState,
    SuggestionSource, SuggestionStatus, TokenLease, latest_snapshot,
};

fn timestamp(seconds: u64) -> std::time::SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn sample_document() -> FlowsheetDocument {
    let flowsheet = Flowsheet::new("demo");
    let metadata = DocumentMetadata::new("doc-1", "Demo", timestamp(10));
    FlowsheetDocument::new(flowsheet, metadata)
}

fn inspector_focus_document() -> FlowsheetDocument {
    let mut flowsheet = Flowsheet::new("demo");
    flowsheet
        .insert_unit(UnitNode::new(
            "feed-1",
            "Feed",
            "feed",
            vec![UnitPort::new(
                "outlet",
                PortDirection::Outlet,
                PortKind::Material,
                Some("stream-feed".into()),
            )],
        ))
        .expect("expected feed insert");
    flowsheet
        .insert_stream(MaterialStreamState::from_tpzf(
            "stream-feed",
            "Feed Stream",
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
    let metadata = DocumentMetadata::new("doc-1", "Demo", timestamp(10));
    FlowsheetDocument::new(flowsheet, metadata)
}

fn sample_canvas_suggestion(
    id: &str,
    confidence: f32,
    source: SuggestionSource,
) -> CanvasSuggestion {
    CanvasSuggestion::new(
        CanvasSuggestionId::new(id),
        source,
        confidence,
        GhostElement {
            kind: GhostElementKind::Connection,
            target_unit_id: UnitId::new("flash-1"),
            visual_kind: StreamVisualKind::Material,
            visual_state: StreamVisualState::Suggested,
        },
        format!("reason for {id}"),
    )
}

fn sample_existing_connection_acceptance() -> CanvasSuggestionAcceptance {
    CanvasSuggestionAcceptance::MaterialConnection(CanvasSuggestedMaterialConnection {
        stream: CanvasSuggestedStreamBinding::Existing {
            stream_id: rf_types::StreamId::new("stream-feed"),
        },
        source_unit_id: UnitId::new("feed-1"),
        source_port: "outlet".to_string(),
        sink_unit_id: Some(UnitId::new("flash-1")),
        sink_port: Some("inlet".to_string()),
    })
}

fn sample_feed_flash_document() -> FlowsheetDocument {
    let mut flowsheet = Flowsheet::new("demo");
    flowsheet
        .insert_component(rf_model::Component::new("component-a", "Component A"))
        .expect("expected component-a");
    flowsheet
        .insert_component(rf_model::Component::new("component-b", "Component B"))
        .expect("expected component-b");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "feed-1",
            "Feed",
            "feed",
            vec![rf_model::UnitPort::new(
                "outlet",
                rf_types::PortDirection::Outlet,
                rf_types::PortKind::Material,
                Some("stream-feed".into()),
            )],
        ))
        .expect("expected feed insert");
    flowsheet
        .insert_unit(rf_model::UnitNode::new(
            "flash-1",
            "Flash Drum",
            "flash_drum",
            vec![
                rf_model::UnitPort::new(
                    "inlet",
                    rf_types::PortDirection::Inlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "liquid",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    None,
                ),
                rf_model::UnitPort::new(
                    "vapor",
                    rf_types::PortDirection::Outlet,
                    rf_types::PortKind::Material,
                    None,
                ),
            ],
        ))
        .expect("expected flash insert");
    flowsheet
        .insert_stream(MaterialStreamState::new("stream-feed", "Feed"))
        .expect("expected feed stream");
    FlowsheetDocument::new(
        flowsheet,
        DocumentMetadata::new("doc-feed-flash", "Feed Flash", timestamp(10)),
    )
}

fn sample_solver_provider() -> PlaceholderThermoProvider {
    let pressure_pa = 100_000.0_f64;
    let mut first = ThermoComponent::new("component-a", "Component A");
    first.antoine = Some(AntoineCoefficients::new(
        ((2.0_f64 * pressure_pa) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));

    let mut second = ThermoComponent::new("component-b", "Component B");
    second.antoine = Some(AntoineCoefficients::new(
        ((0.5_f64 * pressure_pa) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));

    PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
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
fn commit_document_change_clears_stale_current_snapshot_summary() {
    let mut app_state = AppState::new(sample_document());
    let snapshot = SolveSnapshot::new(
        "snapshot-ui-stale",
        0,
        1,
        RunStatus::Converged,
        DiagnosticSummary::new(0, DiagnosticSeverity::Info, "snapshot ok"),
    );
    app_state.store_snapshot(snapshot);

    app_state.commit_document_change(
        DocumentCommand::MoveUnit {
            unit_id: UnitId::new("heater-1"),
            position: CanvasPoint::new(120.0, 80.0),
        },
        Flowsheet::new("demo-updated"),
        timestamp(21),
    );

    assert_eq!(app_state.workspace.snapshot_history.len(), 1);
    assert_eq!(app_state.workspace.solve_session.latest_snapshot, None);
    assert_eq!(app_state.workspace.solve_session.latest_diagnostic, None);
    assert!(latest_snapshot(&app_state.workspace).is_none());
    assert_eq!(app_state.workspace.run_panel.latest_snapshot_id, None);
    assert_eq!(app_state.workspace.run_panel.latest_snapshot_summary, None);
    assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Dirty);
    assert_eq!(
        app_state.workspace.run_panel.pending_reason,
        Some(SolvePendingReason::DocumentRevisionAdvanced)
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
    assert_eq!(
        app_state.workspace.run_panel.simulation_mode,
        SimulationMode::Active
    );
    assert_eq!(
        app_state.workspace.run_panel.pending_reason,
        Some(SolvePendingReason::ModeActivated)
    );
}

#[test]
fn initial_run_panel_reflects_hold_idle_without_snapshot() {
    let app_state = AppState::new(sample_document());

    assert_eq!(
        app_state.workspace.run_panel,
        RunPanelState {
            simulation_mode: SimulationMode::Hold,
            run_status: RunStatus::Idle,
            pending_reason: Some(SolvePendingReason::SnapshotMissing),
            latest_snapshot_id: None,
            latest_snapshot_summary: None,
            latest_log_message: None,
            notice: None,
            can_run_manual: true,
            can_resume: true,
            can_set_hold: false,
            can_set_active: true,
            commands: app_state.workspace.run_panel.commands.clone(),
        }
    );
    assert_eq!(
        app_state.workspace.run_panel.commands.primary_action,
        RunPanelActionId::Resume
    );
    assert_eq!(
        app_state
            .workspace
            .run_panel
            .commands
            .action(RunPanelActionId::Resume)
            .expect("expected resume action")
            .label,
        "Resume"
    );
}

mod canvas;
mod entitlement;
mod inspector;
mod recovery;
mod run_panel;
