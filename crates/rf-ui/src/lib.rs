mod auth;
mod canvas_interaction;
mod commands;
mod diagnostics;
mod entitlement_panel;
mod entitlement_panel_presenter;
mod entitlement_panel_text;
mod entitlement_panel_view;
mod entitlement_panel_widget;
mod ids;
mod run;
mod run_panel;
mod run_panel_presenter;
mod run_panel_text;
mod run_panel_view;
mod run_panel_widget;
mod state;

pub use auth::{
    AuditUsageAck, AuditUsageRequest, AuthSessionState, AuthSessionStatus, AuthenticatedUser,
    EntitlementNotice, EntitlementNoticeLevel, EntitlementSnapshot, EntitlementState,
    EntitlementStatus, OfflineLeaseRefreshRequest, OfflineLeaseRefreshResponse,
    PropertyPackageClassification, PropertyPackageLeaseGrant, PropertyPackageLeaseRequest,
    PropertyPackageManifest, PropertyPackageManifestList, PropertyPackageSource,
    PropertyPackageUsageEvent, PropertyPackageUsageEventKind, SecureCredentialHandle, TokenLease,
};
pub use canvas_interaction::{
    CanvasEditIntent, CanvasInteractionState, CanvasSuggestedMaterialConnection,
    CanvasSuggestedStreamBinding, CanvasSuggestion, CanvasSuggestionAcceptance, CanvasViewMode,
    GhostElement, GhostElementKind, StreamAnimationMode, StreamVisualKind, StreamVisualState,
    SuggestionSource, SuggestionStatus,
};
pub use commands::{
    CanvasPoint, CommandHistory, CommandHistoryEntry, CommandValue, DocumentCommand,
    StreamSpecificationValue,
};
pub use diagnostics::{DiagnosticSeverity, DiagnosticSnapshot, DiagnosticSummary};
pub use entitlement_panel::{
    EntitlementActionId, EntitlementActionModel, EntitlementCommandModel, EntitlementIntent,
    EntitlementPanelState,
};
pub use entitlement_panel_presenter::EntitlementPanelPresentation;
pub use entitlement_panel_text::EntitlementPanelTextView;
pub use entitlement_panel_view::{
    EntitlementActionProminence, EntitlementPanelViewModel, EntitlementRenderableAction,
};
pub use entitlement_panel_widget::{EntitlementPanelWidgetEvent, EntitlementPanelWidgetModel};
pub use ids::{CanvasSuggestionId, DocumentId, SolveSnapshotId};
pub use run::{
    PhaseStateSnapshot, RunStatus, SimulationMode, SolvePendingReason, SolveSessionState,
    SolveSnapshot, StepSnapshot, StreamStateSnapshot, UnitExecutionSnapshot,
};
pub use run_panel::{
    RunPanelActionId, RunPanelActionModel, RunPanelCommandModel, RunPanelIntent, RunPanelNotice,
    RunPanelNoticeLevel, RunPanelPackageSelection, RunPanelRecoveryAction,
    RunPanelRecoveryActionKind, RunPanelRecoveryMutation, RunPanelState, run_panel_failure_notice,
    run_panel_failure_recovery_action_for_diagnostic_code,
    run_panel_failure_title_for_diagnostic_code,
};
pub use run_panel_presenter::RunPanelPresentation;
pub use run_panel_text::RunPanelTextView;
pub use run_panel_view::{RunPanelActionProminence, RunPanelRenderableAction, RunPanelViewModel};
pub use run_panel_widget::{RunPanelRecoveryWidgetEvent, RunPanelWidgetEvent, RunPanelWidgetModel};
pub use state::{
    AppLogEntry, AppLogFeed, AppLogLevel, AppState, AppTheme, DateTimeUtc,
    DocumentHistoryApplyResult, DocumentHistoryDirection, DocumentMetadata, DraftValidationState,
    DraftValue, FieldDraft, FlowsheetDocument, InspectorDraftState, InspectorTarget, LocaleCode,
    PanelLayoutPreferences, SelectionState, StreamInspectorDraftBatchCommitResult,
    StreamInspectorDraftCommitResult, StreamInspectorDraftField, StreamInspectorDraftUpdateResult,
    UiPanelsState, UserPreferences, WorkspaceState, latest_snapshot, latest_snapshot_id,
    stream_inspector_draft_key, stream_inspector_draft_key_parts,
};

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use rf_flash::PlaceholderTpFlashSolver;
    use rf_model::{Flowsheet, MaterialStreamState, UnitNode, UnitPort};
    use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolverServices};
    use rf_thermo::{
        AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem,
    };
    use rf_types::{ComponentId, PortDirection, PortKind, StreamId, UnitId};

    use crate::{
        AppLogLevel, AppState, AuthSessionStatus, AuthenticatedUser, CanvasEditIntent, CanvasPoint,
        CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding, CanvasSuggestion,
        CanvasSuggestionAcceptance, CanvasSuggestionId, CanvasViewMode, CommandHistory,
        CommandHistoryEntry, CommandValue, DiagnosticSeverity, DiagnosticSummary, DocumentCommand,
        DocumentMetadata, EntitlementActionId, EntitlementPanelState, EntitlementPanelWidgetEvent,
        EntitlementPanelWidgetModel, EntitlementSnapshot, FlowsheetDocument, GhostElement,
        GhostElementKind, OfflineLeaseRefreshResponse, PropertyPackageManifest,
        PropertyPackageManifestList, PropertyPackageSource, RunPanelActionId,
        RunPanelActionProminence, RunPanelPresentation, RunPanelRecoveryWidgetEvent, RunPanelState,
        RunPanelTextView, RunPanelViewModel, RunPanelWidgetEvent, RunPanelWidgetModel, RunStatus,
        SecureCredentialHandle, SimulationMode, SolvePendingReason, SolveSnapshot,
        StreamVisualKind, StreamVisualState, SuggestionSource, SuggestionStatus, TokenLease,
        latest_snapshot,
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

    #[test]
    fn workspace_initializes_canvas_interaction_in_planar_mode() {
        let app_state = AppState::new(sample_document());

        assert_eq!(
            app_state.workspace.canvas_interaction.view_mode,
            CanvasViewMode::Planar
        );
        assert!(
            app_state
                .workspace
                .canvas_interaction
                .suggestions
                .is_empty()
        );
        assert_eq!(
            app_state.workspace.canvas_interaction.focused_suggestion_id,
            None
        );
        assert_eq!(app_state.workspace.canvas_interaction.pending_edit, None);
    }

    #[test]
    fn canvas_place_unit_intent_is_transient_and_not_document_history() {
        let mut app_state = AppState::new(sample_document());

        let intent = app_state.begin_canvas_place_unit("Flash Drum");

        assert_eq!(
            intent,
            CanvasEditIntent::PlaceUnit {
                unit_kind: "Flash Drum".to_string()
            }
        );
        assert_eq!(
            app_state.workspace.canvas_interaction.pending_edit,
            Some(CanvasEditIntent::PlaceUnit {
                unit_kind: "Flash Drum".to_string()
            })
        );
        assert_eq!(app_state.workspace.document.revision, 0);
        assert!(!app_state.workspace.command_history.can_undo());
    }

    #[test]
    fn cancelling_canvas_place_unit_intent_clears_pending_edit() {
        let mut app_state = AppState::new(sample_document());
        app_state.begin_canvas_place_unit("Flash Drum");

        let cancelled = app_state.cancel_canvas_pending_edit();

        assert_eq!(
            cancelled,
            Some(CanvasEditIntent::PlaceUnit {
                unit_kind: "Flash Drum".to_string()
            })
        );
        assert_eq!(app_state.workspace.canvas_interaction.pending_edit, None);
        assert_eq!(app_state.workspace.document.revision, 0);
        assert!(!app_state.workspace.command_history.can_undo());
    }

    #[test]
    fn document_change_invalidates_canvas_pending_edit() {
        let mut app_state = AppState::new(sample_document());
        app_state.begin_canvas_place_unit("Flash Drum");
        let mut next_flowsheet = app_state.workspace.document.flowsheet.clone();
        next_flowsheet
            .insert_unit(UnitNode::new(
                "flash-1",
                "Flash Drum",
                "flash_drum",
                Vec::new(),
            ))
            .expect("expected unit insert");

        app_state.commit_document_change(
            DocumentCommand::CreateUnit {
                unit_id: UnitId::new("flash-1"),
                kind: "flash_drum".to_string(),
            },
            next_flowsheet,
            timestamp(20),
        );

        assert_eq!(app_state.workspace.canvas_interaction.pending_edit, None);
    }

    #[test]
    fn replacing_canvas_suggestions_orders_by_confidence_and_focuses_first() {
        let mut app_state = AppState::new(sample_document());
        app_state.replace_canvas_suggestions(vec![
            sample_canvas_suggestion("sug-low", 0.40, SuggestionSource::LocalRules),
            sample_canvas_suggestion("sug-high", 0.95, SuggestionSource::RadishMind),
            sample_canvas_suggestion("sug-mid", 0.70, SuggestionSource::LocalRules),
        ]);

        let suggestions = &app_state.workspace.canvas_interaction.suggestions;
        assert_eq!(suggestions[0].id.as_str(), "sug-high");
        assert_eq!(suggestions[1].id.as_str(), "sug-mid");
        assert_eq!(suggestions[2].id.as_str(), "sug-low");
        assert_eq!(
            app_state
                .workspace
                .canvas_interaction
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("sug-high")
        );
        assert_eq!(suggestions[0].status, SuggestionStatus::Focused);
        assert_eq!(suggestions[1].status, SuggestionStatus::Proposed);
        assert_eq!(suggestions[2].status, SuggestionStatus::Proposed);
    }

    #[test]
    fn focus_next_canvas_suggestion_rotates_between_available_entries() {
        let mut app_state = AppState::new(sample_document());
        app_state.replace_canvas_suggestions(vec![
            sample_canvas_suggestion("sug-low", 0.40, SuggestionSource::LocalRules),
            sample_canvas_suggestion("sug-high", 0.95, SuggestionSource::RadishMind),
            sample_canvas_suggestion("sug-mid", 0.70, SuggestionSource::LocalRules),
        ]);

        let next = app_state
            .focus_next_canvas_suggestion()
            .expect("expected next focused suggestion");
        assert_eq!(next.id.as_str(), "sug-mid");
        assert_eq!(
            app_state
                .workspace
                .canvas_interaction
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("sug-mid")
        );

        let wrapped = app_state
            .focus_next_canvas_suggestion()
            .expect("expected wrapped focus");
        assert_eq!(wrapped.id.as_str(), "sug-low");
        assert_eq!(
            app_state
                .workspace
                .canvas_interaction
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("sug-low")
        );
    }

    #[test]
    fn focus_previous_canvas_suggestion_wraps_to_last_available_entry() {
        let mut app_state = AppState::new(sample_document());
        app_state.replace_canvas_suggestions(vec![
            sample_canvas_suggestion("sug-low", 0.40, SuggestionSource::LocalRules),
            sample_canvas_suggestion("sug-high", 0.95, SuggestionSource::RadishMind),
            sample_canvas_suggestion("sug-mid", 0.70, SuggestionSource::LocalRules),
        ]);

        let previous = app_state
            .focus_previous_canvas_suggestion()
            .expect("expected previous focused suggestion");
        assert_eq!(previous.id.as_str(), "sug-low");
        assert_eq!(
            app_state
                .workspace
                .canvas_interaction
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("sug-low")
        );
    }

    #[test]
    fn rejecting_focused_canvas_suggestion_advances_focus_to_next_available_entry() {
        let mut app_state = AppState::new(sample_document());
        app_state.replace_canvas_suggestions(vec![
            sample_canvas_suggestion("sug-low", 0.40, SuggestionSource::LocalRules),
            sample_canvas_suggestion("sug-high", 0.95, SuggestionSource::RadishMind),
            sample_canvas_suggestion("sug-mid", 0.70, SuggestionSource::LocalRules),
        ]);

        let rejected = app_state
            .reject_focused_canvas_suggestion()
            .expect("expected rejected suggestion");
        assert_eq!(rejected.id.as_str(), "sug-high");
        assert_eq!(rejected.status, SuggestionStatus::Rejected);
        assert_eq!(
            app_state
                .workspace
                .canvas_interaction
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("sug-mid")
        );
        assert_eq!(
            app_state.workspace.canvas_interaction.suggestions[0].status,
            SuggestionStatus::Rejected
        );
        assert_eq!(
            app_state.workspace.canvas_interaction.suggestions[1].status,
            SuggestionStatus::Focused
        );
    }

    #[test]
    fn tab_accepts_only_high_confidence_suggestions_without_recording_history() {
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
                        Some("stream-liquid".into()),
                    ),
                    rf_model::UnitPort::new(
                        "vapor",
                        rf_types::PortDirection::Outlet,
                        rf_types::PortKind::Material,
                        Some("stream-vapor".into()),
                    ),
                ],
            ))
            .expect("expected flash insert");
        for stream_id in ["stream-feed", "stream-liquid", "stream-vapor"] {
            flowsheet
                .insert_stream(MaterialStreamState::new(stream_id, stream_id))
                .expect("expected stream insert");
        }
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc-accept", "Accept", timestamp(10)),
        ));
        app_state.replace_canvas_suggestions(vec![
            sample_canvas_suggestion("sug-high", 0.90, SuggestionSource::LocalRules)
                .with_acceptance(sample_existing_connection_acceptance()),
        ]);

        let accepted = app_state
            .accept_focused_canvas_suggestion_by_tab()
            .expect("expected suggestion acceptance");

        assert_eq!(
            accepted.as_ref().map(|item| item.id.as_str()),
            Some("sug-high")
        );
        assert_eq!(app_state.workspace.command_history.len(), 1);
        assert!(matches!(
            app_state.workspace.command_history.current_entry(),
            Some(crate::CommandHistoryEntry {
                command: crate::DocumentCommand::ConnectPorts {
                    stream_id,
                    from_unit_id,
                    from_port,
                    to_unit_id: Some(to_unit_id),
                    to_port: Some(to_port),
                },
                ..
            }) if stream_id.as_str() == "stream-feed"
                && from_unit_id.as_str() == "feed-1"
                && from_port == "outlet"
                && to_unit_id.as_str() == "flash-1"
                && to_port == "inlet"
        ));
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state
                .workspace
                .document
                .flowsheet
                .units
                .get(&UnitId::new("flash-1"))
                .and_then(|unit| unit.ports.iter().find(|port| port.name == "inlet"))
                .and_then(|port| port.stream_id.as_ref())
                .map(|stream_id| stream_id.as_str()),
            Some("stream-feed")
        );
        assert_eq!(
            app_state.workspace.canvas_interaction.focused_suggestion_id,
            None
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("flash-1")))
        );
        assert!(
            app_state
                .workspace
                .selection
                .selected_units
                .contains(&UnitId::new("flash-1"))
        );
        assert!(app_state.workspace.panels.inspector_open);
        assert_eq!(
            app_state.log_feed.entries.back(),
            Some(&crate::AppLogEntry {
                level: AppLogLevel::Info,
                message: "Accepted canvas suggestion `sug-high` from local rules for unit flash-1"
                    .to_string(),
            })
        );
        assert_eq!(
            app_state.workspace.run_panel.latest_log_message.as_deref(),
            Some("Accepted canvas suggestion `sug-high` from local rules for unit flash-1")
        );
    }

    #[test]
    fn tab_accepts_suggestion_that_creates_terminal_outlet_stream() {
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
                        Some("stream-feed".into()),
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
                        Some("stream-vapor".into()),
                    ),
                ],
            ))
            .expect("expected flash insert");
        for stream_id in ["stream-feed", "stream-vapor"] {
            flowsheet
                .insert_stream(MaterialStreamState::new(stream_id, stream_id))
                .expect("expected stream insert");
        }
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc-create", "Create", timestamp(10)),
        ));
        app_state.replace_canvas_suggestions(vec![
            sample_canvas_suggestion("sug-liquid", 0.92, SuggestionSource::LocalRules)
                .with_acceptance(CanvasSuggestionAcceptance::MaterialConnection(
                    CanvasSuggestedMaterialConnection {
                        stream: CanvasSuggestedStreamBinding::Create {
                            stream: MaterialStreamState::new("stream-liquid", "Liquid Outlet"),
                        },
                        source_unit_id: UnitId::new("flash-1"),
                        source_port: "liquid".to_string(),
                        sink_unit_id: None,
                        sink_port: None,
                    },
                )),
        ]);

        let accepted = app_state
            .accept_focused_canvas_suggestion_by_tab()
            .expect("expected terminal outlet suggestion acceptance");

        assert_eq!(
            accepted.as_ref().map(|item| item.id.as_str()),
            Some("sug-liquid")
        );
        assert!(matches!(
            app_state.workspace.command_history.current_entry(),
            Some(crate::CommandHistoryEntry {
                command: crate::DocumentCommand::ConnectPorts {
                    stream_id,
                    from_unit_id,
                    from_port,
                    to_unit_id: None,
                    to_port: None,
                },
                ..
            }) if stream_id.as_str() == "stream-liquid"
                && from_unit_id.as_str() == "flash-1"
                && from_port == "liquid"
        ));
        assert_eq!(
            app_state
                .workspace
                .document
                .flowsheet
                .streams
                .get(&rf_types::StreamId::new("stream-liquid"))
                .map(|stream| stream.name.as_str()),
            Some("Liquid Outlet")
        );
        assert_eq!(
            app_state
                .workspace
                .document
                .flowsheet
                .units
                .get(&UnitId::new("flash-1"))
                .and_then(|unit| unit.ports.iter().find(|port| port.name == "liquid"))
                .and_then(|port| port.stream_id.as_ref())
                .map(|stream_id| stream_id.as_str()),
            Some("stream-liquid")
        );
    }

    #[test]
    fn tab_does_not_accept_low_confidence_suggestion() {
        let mut app_state = AppState::new(sample_document());
        app_state.replace_canvas_suggestions(vec![sample_canvas_suggestion(
            "sug-low",
            0.60,
            SuggestionSource::RadishMind,
        )]);

        let accepted = app_state
            .accept_focused_canvas_suggestion_by_tab()
            .expect("expected low-confidence acceptance check");

        assert!(accepted.is_none());
        assert_eq!(app_state.workspace.command_history.len(), 0);
        assert_eq!(
            app_state.workspace.canvas_interaction.suggestions[0].status,
            SuggestionStatus::Focused
        );
        assert_eq!(
            app_state
                .workspace
                .canvas_interaction
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("sug-low")
        );
        assert!(app_state.workspace.selection.selected_units.is_empty());
        assert_eq!(app_state.workspace.drafts.active_target, None);
        assert!(app_state.log_feed.entries.is_empty());
        assert_eq!(app_state.workspace.run_panel.latest_log_message, None);
    }

    #[test]
    fn document_change_invalidates_canvas_suggestions_but_only_records_document_command() {
        let mut app_state = AppState::new(sample_document());
        app_state.replace_canvas_suggestions(vec![sample_canvas_suggestion(
            "sug-high",
            0.95,
            SuggestionSource::LocalRules,
        )]);

        let next_flowsheet = Flowsheet::new("demo-updated");
        app_state.commit_document_change(
            DocumentCommand::MoveUnit {
                unit_id: UnitId::new("flash-1"),
                position: CanvasPoint::new(40.0, 20.0),
            },
            next_flowsheet,
            timestamp(20),
        );

        assert_eq!(app_state.workspace.command_history.len(), 1);
        assert_eq!(
            app_state.workspace.canvas_interaction.suggestions[0].status,
            SuggestionStatus::Invalidated
        );
        assert_eq!(
            app_state.workspace.canvas_interaction.focused_suggestion_id,
            None
        );
    }

    #[test]
    fn run_panel_view_model_consumes_primary_and_secondary_actions() {
        let app_state = AppState::new(sample_document());

        let view = RunPanelViewModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(view.mode_label, "Hold");
        assert_eq!(view.status_label, "Idle");
        assert_eq!(view.pending_label, Some("Snapshot missing"));
        assert_eq!(view.primary_action.id, RunPanelActionId::Resume);
        assert_eq!(view.primary_action.label, "Resume");
        assert_eq!(
            view.primary_action.detail,
            "Resume pending work while the workspace stays in Hold mode"
        );
        assert_eq!(
            view.primary_action.prominence,
            RunPanelActionProminence::Primary
        );
        assert_eq!(view.secondary_actions.len(), 3);
        assert_eq!(view.secondary_actions[0].id, RunPanelActionId::RunManual);
        assert_eq!(view.secondary_actions[1].id, RunPanelActionId::SetHold);
        assert_eq!(view.secondary_actions[2].id, RunPanelActionId::SetActive);
    }

    #[test]
    fn run_panel_text_view_renders_primary_and_secondary_actions() {
        let app_state = AppState::new(sample_document());
        let view = RunPanelViewModel::from_state(&app_state.workspace.run_panel);

        let text = RunPanelTextView::from_view_model(&view);

        assert_eq!(text.title, "Run panel");
        assert_eq!(text.lines[0], "Mode: Hold");
        assert_eq!(text.lines[1], "Status: Idle");
        assert!(
            text.lines
                .iter()
                .any(|line| line == "Pending: Snapshot missing")
        );
        assert!(
            text.lines
                .iter()
                .any(|line| line == "Primary action: Resume [enabled]")
        );
        assert!(text.lines.iter().any(|line| {
            line == "Primary detail: Resume pending work while the workspace stays in Hold mode"
        }));
        assert!(
            text.lines
                .iter()
                .any(|line| { line == "  - Run [enabled] | Run the current workspace once" })
        );
    }

    #[test]
    fn run_panel_text_view_renders_notice_when_present() {
        let mut state = AppState::new(sample_document()).workspace.run_panel;
        state.notice = Some(
            crate::RunPanelNotice::new(
                crate::RunPanelNoticeLevel::Warning,
                "Run blocked",
                "explicit package selection is required",
            )
            .with_recovery_action(crate::RunPanelRecoveryAction::new(
                crate::RunPanelRecoveryActionKind::InspectFailureDetails,
                "Choose package",
                "选择明确的 property package 后再运行。",
            )),
        );

        let view = RunPanelViewModel::from_state(&state);
        let text = RunPanelTextView::from_view_model(&view);

        assert!(
            text.lines
                .iter()
                .any(|line| line == "Notice: Run blocked [warning]")
        );
        assert!(
            text.lines
                .iter()
                .any(|line| line == "Notice detail: explicit package selection is required")
        );
        assert!(
            text.lines
                .iter()
                .any(|line| line == "Suggested action: Choose package")
        );
        assert!(
            text.lines
                .iter()
                .any(|line| line == "Suggested detail: 选择明确的 property package 后再运行。")
        );
        assert!(
            !text
                .lines
                .iter()
                .any(|line| line.starts_with("Suggested target: unit "))
        );
    }

    #[test]
    fn run_panel_view_model_returns_dispatchable_intents_for_enabled_actions() {
        let app_state = AppState::new(sample_document());
        let view = RunPanelViewModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(
            view.dispatchable_primary_intent(),
            Some(crate::RunPanelIntent::resume(
                crate::RunPanelPackageSelection::preferred()
            ))
        );
        assert_eq!(
            view.dispatchable_intent(RunPanelActionId::RunManual),
            Some(crate::RunPanelIntent::run_manual(
                crate::RunPanelPackageSelection::preferred()
            ))
        );
        assert_eq!(view.dispatchable_intent(RunPanelActionId::SetHold), None);
    }

    #[test]
    fn run_panel_presentation_combines_view_text_and_dispatchable_intents() {
        let app_state = AppState::new(sample_document());

        let presentation = RunPanelPresentation::from_state(&app_state.workspace.run_panel);

        assert_eq!(presentation.view.primary_action.label, "Resume");
        assert_eq!(presentation.text.title, "Run panel");
        assert_eq!(
            presentation.dispatchable_primary_intent(),
            Some(crate::RunPanelIntent::resume(
                crate::RunPanelPackageSelection::preferred()
            ))
        );
    }

    #[test]
    fn run_panel_widget_dispatches_primary_action_and_blocks_disabled_actions() {
        let app_state = AppState::new(sample_document());

        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(
            widget.activate_primary(),
            RunPanelWidgetEvent::Dispatched {
                action_id: RunPanelActionId::Resume,
                intent: crate::RunPanelIntent::resume(crate::RunPanelPackageSelection::preferred()),
            }
        );
        assert_eq!(
            widget.activate(RunPanelActionId::SetHold),
            RunPanelWidgetEvent::Disabled {
                action_id: RunPanelActionId::SetHold,
                detail: "Workspace is already in Hold mode",
            }
        );
    }

    #[test]
    fn run_panel_widget_exposes_recovery_action_when_solver_failure_targets_unit() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.step.inlet: solver step 1 inlet resolution failed for unit `heater-1`",
        )
        .with_primary_code("solver.step.inlet")
        .with_related_unit_ids(vec![UnitId::new("heater-1")]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(
            widget.activate_recovery_action(),
            RunPanelRecoveryWidgetEvent::Requested {
                action: crate::RunPanelRecoveryAction::new(
                    crate::RunPanelRecoveryActionKind::InspectInletPath,
                    "Inspect inlet path",
                    "检查入口连接是否完整，以及上游流股是否应先于该单元求解。",
                )
                .with_target_unit(UnitId::new("heater-1")),
            }
        );
        assert!(
            widget
                .text()
                .lines
                .iter()
                .any(|line| line == "Suggested target: unit heater-1")
        );
    }

    #[test]
    fn run_panel_widget_exposes_recovery_action_when_connection_failure_targets_disconnectable_port()
     {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.duplicate_downstream_sink: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.duplicate_downstream_sink")
        .with_related_unit_ids(vec![UnitId::new("flash-1"), UnitId::new("mixer-1")])
        .with_related_stream_ids(vec![rf_types::StreamId::new("shared-stream")])
        .with_related_port_targets(vec![
            rf_types::DiagnosticPortTarget::new("flash-1", "inlet"),
            rf_types::DiagnosticPortTarget::new("mixer-1", "inlet_a"),
        ]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(
            widget.activate_recovery_action(),
            RunPanelRecoveryWidgetEvent::Requested {
                action: crate::RunPanelRecoveryAction::new(
                    crate::RunPanelRecoveryActionKind::FixConnections,
                    "Disconnect conflicting sink",
                    "断开冲突的 inlet sink 端口，让该流股只保留一个下游去向后再继续修复。",
                )
                .with_disconnect_port(UnitId::new("mixer-1"), "inlet_a"),
            }
        );
        assert!(
            widget
                .text()
                .lines
                .iter()
                .any(|line| line == "Suggested target: unit mixer-1 port inlet_a")
        );
    }

    #[test]
    fn recording_duplicate_upstream_source_prefers_last_conflicting_port_for_disconnect() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.duplicate_upstream_source: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.duplicate_upstream_source")
        .with_related_unit_ids(vec![UnitId::new("feed-1"), UnitId::new("heater-1")])
        .with_related_stream_ids(vec![rf_types::StreamId::new("shared-stream")])
        .with_related_port_targets(vec![
            rf_types::DiagnosticPortTarget::new("feed-1", "outlet"),
            rf_types::DiagnosticPortTarget::new("heater-1", "outlet"),
        ]);

        app_state.record_failure(0, RunStatus::Error, summary);

        assert_eq!(
            app_state
                .workspace
                .run_panel
                .notice
                .as_ref()
                .and_then(|notice| notice.recovery_action.as_ref()),
            Some(
                &crate::RunPanelRecoveryAction::new(
                    crate::RunPanelRecoveryActionKind::FixConnections,
                    "Disconnect conflicting source",
                    "断开冲突的 outlet source 端口，让该流股只保留一个上游来源后再继续修复。",
                )
                .with_disconnect_port(UnitId::new("heater-1"), "outlet")
            )
        );
    }

    #[test]
    fn run_panel_widget_exposes_recovery_action_when_orphan_stream_targets_deletable_stream() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.orphan_stream: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.orphan_stream")
        .with_related_stream_ids(vec![rf_types::StreamId::new("stream-orphan")]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(
            widget.activate_recovery_action(),
            RunPanelRecoveryWidgetEvent::Requested {
                action: crate::RunPanelRecoveryAction::new(
                    crate::RunPanelRecoveryActionKind::FixConnections,
                    "Delete orphan stream",
                    "删除当前未连接到任何单元端口的孤立流股，避免它继续阻塞连接校验。",
                )
                .with_delete_stream(rf_types::StreamId::new("stream-orphan")),
            }
        );
        assert!(
            widget
                .text()
                .lines
                .iter()
                .any(|line| line == "Suggested target: stream stream-orphan")
        );
    }

    #[test]
    fn run_panel_widget_exposes_recovery_action_when_unbound_outlet_targets_stream_creation() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.unbound_outlet_port: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.unbound_outlet_port")
        .with_related_unit_ids(vec![UnitId::new("feed-1")])
        .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
            "feed-1", "outlet",
        )]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(
            widget.activate_recovery_action(),
            RunPanelRecoveryWidgetEvent::Requested {
                action: crate::RunPanelRecoveryAction::new(
                    crate::RunPanelRecoveryActionKind::FixConnections,
                    "Create outlet stream",
                    "为当前未绑定 stream 的 outlet 端口创建一条占位流股，并立即写回连接。",
                )
                .with_create_and_bind_outlet_stream(UnitId::new("feed-1"), "outlet"),
            }
        );
        assert!(
            widget
                .text()
                .lines
                .iter()
                .any(|line| line == "Suggested target: unit feed-1 port outlet")
        );
    }

    #[test]
    fn run_panel_widget_exposes_recovery_action_when_self_loop_targets_disconnectable_port() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.topological_ordering.self_loop_cycle: solver topological ordering failed",
        )
        .with_primary_code("solver.topological_ordering.self_loop_cycle")
        .with_related_unit_ids(vec![UnitId::new("flash-1")])
        .with_related_stream_ids(vec![rf_types::StreamId::new("stream-loop")])
        .with_related_port_targets(vec![
            rf_types::DiagnosticPortTarget::new("flash-1", "inlet"),
            rf_types::DiagnosticPortTarget::new("flash-1", "liquid"),
        ]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(
            widget.activate_recovery_action(),
            RunPanelRecoveryWidgetEvent::Requested {
                action: crate::RunPanelRecoveryAction::new(
                    crate::RunPanelRecoveryActionKind::BreakCycle,
                    "Disconnect self-loop inlet",
                    "断开当前单元引用自身 outlet stream 的 inlet 端口，先消除自环依赖，再继续检查剩余连接问题。",
                )
                .with_disconnect_port(UnitId::new("flash-1"), "inlet"),
            }
        );
        assert!(
            widget
                .text()
                .lines
                .iter()
                .any(|line| line == "Suggested target: unit flash-1 port inlet")
        );
    }

    #[test]
    fn run_panel_widget_exposes_recovery_action_when_two_unit_cycle_targets_disconnectable_port() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.topological_ordering.two_unit_cycle: solver topological ordering failed",
        )
        .with_primary_code("solver.topological_ordering.two_unit_cycle")
        .with_related_unit_ids(vec![UnitId::new("heater-1"), UnitId::new("valve-1")])
        .with_related_stream_ids(vec![
            rf_types::StreamId::new("stream-a"),
            rf_types::StreamId::new("stream-b"),
        ])
        .with_related_port_targets(vec![
            rf_types::DiagnosticPortTarget::new("valve-1", "inlet"),
            rf_types::DiagnosticPortTarget::new("heater-1", "inlet"),
        ]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(
            widget.activate_recovery_action(),
            RunPanelRecoveryWidgetEvent::Requested {
                action: crate::RunPanelRecoveryAction::new(
                    crate::RunPanelRecoveryActionKind::BreakCycle,
                    "Disconnect cycle inlet",
                    "断开当前双单元回路中的一个 inlet 端口，先打破互相依赖，再继续检查剩余连接问题。",
                )
                .with_disconnect_port(UnitId::new("heater-1"), "inlet"),
            }
        );
        assert!(
            widget
                .text()
                .lines
                .iter()
                .any(|line| line == "Suggested target: unit heater-1 port inlet")
        );
    }

    #[test]
    fn run_panel_widget_exposes_recovery_action_when_connection_failure_targets_port() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.missing_upstream_source: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.missing_upstream_source")
        .with_related_unit_ids(vec![UnitId::new("mixer-1")])
        .with_related_stream_ids(vec![rf_types::StreamId::new("stream-feed-a")])
        .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
            "mixer-1",
            "inlet_a",
        )]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        assert_eq!(
            widget.activate_recovery_action(),
            RunPanelRecoveryWidgetEvent::Requested {
                action: crate::RunPanelRecoveryAction::new(
                    crate::RunPanelRecoveryActionKind::FixConnections,
                    "Remove dangling inlet stream",
                    "断开当前缺少上游 source 的 inlet 绑定，并删除对应孤立流股，先消除悬空入口连接。",
                )
                .with_disconnect_port_and_delete_stream(
                    UnitId::new("mixer-1"),
                    "inlet_a",
                    rf_types::StreamId::new("stream-feed-a"),
                ),
            }
        );
        assert!(
            widget
                .text()
                .lines
                .iter()
                .any(|line| line == "Suggested target: unit mixer-1 port inlet_a")
        );
    }

    #[test]
    fn storing_snapshot_updates_run_panel_summary() {
        let mut app_state = AppState::new(sample_document());
        let snapshot = SolveSnapshot::new(
            "snapshot-ui-1",
            0,
            1,
            RunStatus::Converged,
            DiagnosticSummary::new(0, DiagnosticSeverity::Info, "snapshot ok"),
        );

        app_state.store_snapshot(snapshot);

        assert_eq!(
            app_state.workspace.run_panel.latest_snapshot_id.as_deref(),
            Some("snapshot-ui-1")
        );
        assert_eq!(
            app_state
                .workspace
                .run_panel
                .latest_snapshot_summary
                .as_deref(),
            Some("snapshot ok")
        );
        assert_eq!(
            app_state.workspace.run_panel.run_status,
            RunStatus::Converged
        );
    }

    #[test]
    fn recording_failure_updates_run_panel_status_and_log_message() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(0, DiagnosticSeverity::Error, "solve failed");

        app_state.push_log(crate::AppLogLevel::Error, "solver failed");
        app_state.record_failure(0, RunStatus::Error, summary);

        assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Error);
        assert_eq!(
            app_state.workspace.run_panel.latest_log_message.as_deref(),
            Some("solver failed")
        );
        assert_eq!(
            app_state.workspace.run_panel.commands.primary_action,
            RunPanelActionId::RunManual
        );
        assert_eq!(
            app_state
                .workspace
                .run_panel
                .notice
                .as_ref()
                .map(|notice| notice.title.as_str()),
            Some("Run failed")
        );
        assert!(
            !app_state
                .workspace
                .run_panel
                .commands
                .action(RunPanelActionId::Resume)
                .expect("expected resume action")
                .enabled
        );
    }

    #[test]
    fn recording_failure_clears_current_snapshot_for_same_revision() {
        let mut app_state = AppState::new(sample_document());
        app_state.store_snapshot(SolveSnapshot::new(
            "snapshot-success-1",
            0,
            1,
            RunStatus::Converged,
            DiagnosticSummary::new(0, DiagnosticSeverity::Info, "snapshot ok"),
        ));

        app_state.record_failure(
            0,
            RunStatus::Error,
            DiagnosticSummary::new(0, DiagnosticSeverity::Error, "solve failed again"),
        );

        assert_eq!(app_state.workspace.solve_session.latest_snapshot, None);
        assert!(latest_snapshot(&app_state.workspace).is_none());
        assert_eq!(app_state.workspace.run_panel.latest_snapshot_id, None);
        assert_eq!(app_state.workspace.run_panel.latest_snapshot_summary, None);
        assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Error);
        assert_eq!(
            app_state
                .workspace
                .run_panel
                .notice
                .as_ref()
                .map(|notice| notice.title.as_str()),
            Some("Run failed")
        );
    }

    #[test]
    fn recording_solver_failure_uses_primary_code_for_run_panel_notice_title() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.step.execution: solver step 2 unit execution failed",
        )
        .with_primary_code("solver.step.execution")
        .with_related_unit_ids(vec![UnitId::new("heater-1")]);

        app_state.record_failure(0, RunStatus::Error, summary);

        assert_eq!(
            app_state
                .workspace
                .run_panel
                .notice
                .as_ref()
                .map(|notice| notice.title.as_str()),
            Some("Unit execution failed")
        );
        assert_eq!(
            app_state
                .workspace
                .run_panel
                .notice
                .as_ref()
                .and_then(|notice| notice.recovery_action.as_ref())
                .map(|action| {
                    (
                        action.title,
                        action.detail,
                        action
                            .target_unit_id
                            .as_ref()
                            .map(|unit_id| unit_id.as_str()),
                    )
                }),
            Some((
                "Inspect unit inputs",
                "检查单元规格、物性条件和入口状态是否满足执行前提。",
                Some("heater-1"),
            ))
        );
    }

    #[test]
    fn recording_connection_validation_subcode_refines_run_panel_notice_and_recovery() {
        let mut app_state = AppState::new(sample_document());
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.invalid_port_signature: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.invalid_port_signature")
        .with_related_unit_ids(vec![UnitId::new("feed-1")]);

        app_state.record_failure(0, RunStatus::Error, summary);

        assert_eq!(
            app_state
                .workspace
                .run_panel
                .notice
                .as_ref()
                .map(|notice| notice.title.as_str()),
            Some("Invalid port signature")
        );
        assert_eq!(
            app_state
                .workspace
                .run_panel
                .notice
                .as_ref()
                .and_then(|notice| notice.recovery_action.as_ref())
                .map(|action| {
                    (
                        action.title,
                        action.detail,
                        action
                            .target_unit_id
                            .as_ref()
                            .map(|unit_id| unit_id.as_str()),
                        action.mutation.clone(),
                    )
                }),
            Some((
                "Restore canonical ports",
                "按当前内建 unit kind 的 canonical spec 重建端口签名，并尽量保留可匹配的现有 stream 绑定。",
                Some("feed-1"),
                Some(
                    crate::RunPanelRecoveryMutation::RestoreCanonicalPortSignature {
                        unit_id: UnitId::new("feed-1"),
                    }
                ),
            ))
        );
    }

    #[test]
    fn applying_run_panel_recovery_action_restores_canonical_ports_and_preserves_stream_binding() {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../examples/flowsheets/failures/invalid-port-signature.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new(
                "doc-invalid-port-signature-recovery",
                "Invalid Port Signature Recovery Demo",
                timestamp(39),
            ),
        ));
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.invalid_port_signature: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.invalid_port_signature")
        .with_related_unit_ids(vec![UnitId::new("feed-1")]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let action = app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .cloned()
            .expect("expected recovery action");

        assert_eq!(
            action.mutation,
            Some(
                crate::RunPanelRecoveryMutation::RestoreCanonicalPortSignature {
                    unit_id: UnitId::new("feed-1"),
                }
            )
        );

        let applied_target = app_state.apply_run_panel_recovery_action(&action);

        assert_eq!(
            applied_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
        );
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::RestoreCanonicalUnitPorts {
                unit_id: UnitId::new("feed-1"),
            })
        );
        let feed = app_state
            .workspace
            .document
            .flowsheet
            .units
            .get(&UnitId::new("feed-1"))
            .expect("expected feed unit");
        assert_eq!(feed.ports.len(), 1);
        assert_eq!(feed.ports[0].name, "outlet");
        assert_eq!(
            feed.ports[0]
                .stream_id
                .as_ref()
                .map(|stream_id| stream_id.as_str()),
            Some("stream-feed")
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
        );
        assert!(app_state.workspace.panels.inspector_open);
    }

    #[test]
    fn applying_run_panel_recovery_action_selects_unit_and_opens_inspector() {
        let mut document = sample_document();
        document
            .flowsheet
            .insert_unit(UnitNode::new(
                "heater-1",
                "Heater",
                "heater",
                vec![
                    UnitPort::new("inlet", PortDirection::Inlet, PortKind::Material, None),
                    UnitPort::new("outlet", PortDirection::Outlet, PortKind::Material, None),
                ],
            ))
            .expect("expected heater insert");
        let mut app_state = AppState::new(document);
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.step.spec: solver step 1 unit spec validation failed",
        )
        .with_primary_code("solver.step.spec")
        .with_related_unit_ids(vec![UnitId::new("heater-1")]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let action = app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .cloned()
            .expect("expected recovery action");

        let applied_target = app_state.apply_run_panel_recovery_action(&action);

        assert_eq!(
            applied_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
        );
        assert!(
            app_state
                .workspace
                .selection
                .selected_units
                .contains(&UnitId::new("heater-1"))
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
        );
        assert!(app_state.workspace.panels.inspector_open);
    }

    #[test]
    fn focusing_inspector_target_selects_unit_without_document_mutation() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);

        let applied_target =
            app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

        assert_eq!(
            applied_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
        );
        assert_eq!(app_state.workspace.document.revision, 0);
        assert!(app_state.workspace.command_history.is_empty());
        assert!(
            app_state
                .workspace
                .selection
                .selected_units
                .contains(&UnitId::new("feed-1"))
        );
        assert!(app_state.workspace.selection.selected_streams.is_empty());
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
        );
        assert!(app_state.workspace.panels.inspector_open);
    }

    #[test]
    fn focusing_inspector_target_selects_stream_and_clears_previous_unit() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

        let applied_target = app_state
            .focus_inspector_target(crate::InspectorTarget::Stream(StreamId::new("stream-feed")));

        assert_eq!(
            applied_target,
            Some(crate::InspectorTarget::Stream(StreamId::new("stream-feed")))
        );
        assert!(app_state.workspace.selection.selected_units.is_empty());
        assert!(
            app_state
                .workspace
                .selection
                .selected_streams
                .contains(&StreamId::new("stream-feed"))
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Stream(StreamId::new("stream-feed")))
        );
    }

    #[test]
    fn focusing_missing_inspector_target_keeps_current_focus() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

        let applied_target = app_state
            .focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("missing-unit")));

        assert_eq!(applied_target, None);
        assert!(
            app_state
                .workspace
                .selection
                .selected_units
                .contains(&UnitId::new("feed-1"))
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
        );
    }

    #[test]
    fn updating_stream_inspector_draft_keeps_document_unchanged() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        app_state
            .focus_inspector_target(crate::InspectorTarget::Stream(StreamId::new("stream-feed")));

        let outcome = app_state
            .update_stream_inspector_draft(
                &StreamId::new("stream-feed"),
                crate::StreamInspectorDraftField::TemperatureK,
                "333.5",
            )
            .expect("expected draft update");

        assert_eq!(
            outcome.key,
            crate::stream_inspector_draft_key(
                &StreamId::new("stream-feed"),
                &crate::StreamInspectorDraftField::TemperatureK,
            )
        );
        assert!(outcome.is_dirty);
        assert_eq!(outcome.validation, crate::DraftValidationState::Valid);
        assert_eq!(app_state.workspace.document.revision, 0);
        assert!(app_state.workspace.command_history.is_empty());
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&StreamId::new("stream-feed")]
                .temperature_k,
            298.15
        );
        assert_eq!(
            app_state.workspace.drafts.fields.get(&outcome.key),
            Some(&crate::DraftValue::Number(crate::FieldDraft {
                original: "298.15".to_string(),
                current: "333.5".to_string(),
                is_dirty: true,
                validation: crate::DraftValidationState::Valid,
            }))
        );
    }

    #[test]
    fn updating_stream_inspector_draft_preserves_invalid_raw_number() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        app_state
            .focus_inspector_target(crate::InspectorTarget::Stream(StreamId::new("stream-feed")));

        let outcome = app_state
            .update_stream_inspector_draft(
                &StreamId::new("stream-feed"),
                crate::StreamInspectorDraftField::PressurePa,
                "not-a-pressure",
            )
            .expect("expected draft update");

        assert_eq!(outcome.validation, crate::DraftValidationState::Invalid);
        assert_eq!(
            app_state.workspace.drafts.fields.get(&outcome.key),
            Some(&crate::DraftValue::Number(crate::FieldDraft {
                original: "101325".to_string(),
                current: "not-a-pressure".to_string(),
                is_dirty: true,
                validation: crate::DraftValidationState::Invalid,
            }))
        );
        assert_eq!(app_state.workspace.document.revision, 0);
        assert!(app_state.workspace.command_history.is_empty());
    }

    #[test]
    fn updating_stream_inspector_draft_requires_active_stream_target() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        app_state.focus_inspector_target(crate::InspectorTarget::Unit(UnitId::new("feed-1")));

        let outcome = app_state.update_stream_inspector_draft(
            &StreamId::new("stream-feed"),
            crate::StreamInspectorDraftField::Name,
            "Edited stream",
        );

        assert_eq!(outcome, None);
        assert!(app_state.workspace.drafts.fields.is_empty());
        assert_eq!(app_state.workspace.document.revision, 0);
    }

    #[test]
    fn committing_stream_inspector_draft_writes_document_command_and_preserves_focus() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        let stream_id = StreamId::new("stream-feed");
        app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
        app_state
            .update_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::TemperatureK,
                "333.5",
            )
            .expect("expected draft update");

        let outcome = app_state
            .commit_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::TemperatureK,
                timestamp(42),
            )
            .expect("expected draft commit")
            .expect("expected applied draft commit");

        assert_eq!(outcome.revision, 1);
        assert_eq!(
            outcome.command,
            DocumentCommand::SetStreamSpecification {
                stream_id: stream_id.clone(),
                field: "temperature_k".to_string(),
                value: CommandValue::Number(333.5),
            }
        );
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
            333.5
        );
        assert_eq!(app_state.workspace.command_history.len(), 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&outcome.command)
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Stream(stream_id.clone()))
        );
        assert!(!app_state.workspace.drafts.fields.contains_key(&outcome.key));
        assert_eq!(
            app_state.workspace.solve_session.pending_reason,
            Some(SolvePendingReason::DocumentRevisionAdvanced)
        );
        assert_eq!(app_state.workspace.solve_session.status, RunStatus::Dirty);
    }

    #[test]
    fn committing_stream_inspector_composition_draft_updates_overall_mole_fraction() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        let stream_id = StreamId::new("stream-feed");
        let component_id = ComponentId::new("component-a");
        let field = crate::StreamInspectorDraftField::OverallMoleFraction(component_id.clone());
        app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

        let update = app_state
            .update_stream_inspector_draft(&stream_id, field.clone(), "0.25")
            .expect("expected composition draft update");

        assert_eq!(
            update.key,
            "stream:stream-feed:overall_mole_fraction:component-a"
        );
        assert!(update.is_dirty);
        assert_eq!(update.validation, crate::DraftValidationState::Valid);
        assert_eq!(
            app_state.workspace.drafts.fields.get(&update.key),
            Some(&crate::DraftValue::Number(crate::FieldDraft {
                original: "0.4".to_string(),
                current: "0.25".to_string(),
                is_dirty: true,
                validation: crate::DraftValidationState::Valid,
            }))
        );
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].overall_mole_fractions
                [&component_id],
            0.4
        );

        let outcome = app_state
            .commit_stream_inspector_draft(&stream_id, field, timestamp(42))
            .expect("expected draft commit")
            .expect("expected applied composition draft commit");

        assert_eq!(outcome.revision, 1);
        assert_eq!(
            outcome.command,
            DocumentCommand::SetStreamSpecification {
                stream_id: stream_id.clone(),
                field: "overall_mole_fraction:component-a".to_string(),
                value: CommandValue::Number(0.25),
            }
        );
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].overall_mole_fractions
                [&component_id],
            0.25
        );
        assert!(!app_state.workspace.drafts.fields.contains_key(&update.key));
    }

    #[test]
    fn updating_stream_inspector_composition_draft_rejects_unknown_component() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        let stream_id = StreamId::new("stream-feed");
        app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));

        let outcome = app_state.update_stream_inspector_draft(
            &stream_id,
            crate::StreamInspectorDraftField::OverallMoleFraction(ComponentId::new(
                "missing-component",
            )),
            "0.25",
        );

        assert_eq!(outcome, None);
        assert!(app_state.workspace.drafts.fields.is_empty());
        assert_eq!(app_state.workspace.document.revision, 0);
    }

    #[test]
    fn committing_stream_inspector_drafts_records_one_batch_history_entry() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        let stream_id = StreamId::new("stream-feed");
        app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
        app_state
            .update_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::TemperatureK,
                "333.5",
            )
            .expect("expected temperature draft update");
        app_state
            .update_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::PressurePa,
                "202650",
            )
            .expect("expected pressure draft update");

        let outcome = app_state
            .commit_stream_inspector_drafts(&stream_id, timestamp(42))
            .expect("expected batch commit")
            .expect("expected applied batch commit");

        assert_eq!(outcome.revision, 1);
        assert_eq!(
            outcome.keys,
            vec![
                "stream:stream-feed:temperature_k".to_string(),
                "stream:stream-feed:pressure_pa".to_string()
            ]
        );
        assert_eq!(
            outcome.command,
            DocumentCommand::SetStreamSpecifications {
                stream_id: stream_id.clone(),
                values: vec![
                    crate::StreamSpecificationValue {
                        field: "temperature_k".to_string(),
                        value: CommandValue::Number(333.5),
                    },
                    crate::StreamSpecificationValue {
                        field: "pressure_pa".to_string(),
                        value: CommandValue::Number(202650.0),
                    },
                ],
            }
        );
        let stream = &app_state.workspace.document.flowsheet.streams[&stream_id];
        assert_eq!(stream.temperature_k, 333.5);
        assert_eq!(stream.pressure_pa, 202650.0);
        assert_eq!(app_state.workspace.command_history.len(), 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&outcome.command)
        );
        assert!(app_state.workspace.drafts.fields.is_empty());
        assert_eq!(
            app_state.workspace.solve_session.pending_reason,
            Some(SolvePendingReason::DocumentRevisionAdvanced)
        );
    }

    #[test]
    fn batch_commit_preserves_invalid_stream_inspector_drafts() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        let stream_id = StreamId::new("stream-feed");
        app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
        app_state
            .update_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::TemperatureK,
                "333.5",
            )
            .expect("expected temperature draft update");
        app_state
            .update_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::PressurePa,
                "not-a-pressure",
            )
            .expect("expected invalid pressure draft update");

        let outcome = app_state
            .commit_stream_inspector_drafts(&stream_id, timestamp(42))
            .expect("expected batch commit")
            .expect("expected applied batch commit");

        assert_eq!(outcome.keys, vec!["stream:stream-feed:temperature_k"]);
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
            333.5
        );
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].pressure_pa,
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
    fn undo_redo_replays_stream_inspector_document_snapshots() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        let stream_id = StreamId::new("stream-feed");
        app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
        app_state
            .update_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::TemperatureK,
                "333.5",
            )
            .expect("expected draft update");
        app_state
            .commit_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::TemperatureK,
                timestamp(42),
            )
            .expect("expected draft commit")
            .expect("expected applied draft commit");

        let undo = app_state
            .undo_document_command(timestamp(43))
            .expect("expected undo")
            .expect("expected undo result");

        assert_eq!(undo.direction, crate::DocumentHistoryDirection::Undo);
        assert_eq!(undo.revision, 2);
        assert_eq!(app_state.workspace.command_history.cursor, 0);
        assert!(app_state.workspace.command_history.can_redo());
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
            298.15
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Stream(stream_id.clone()))
        );
        assert_eq!(
            app_state.workspace.solve_session.pending_reason,
            Some(SolvePendingReason::DocumentRevisionAdvanced)
        );

        let redo = app_state
            .redo_document_command(timestamp(44))
            .expect("expected redo")
            .expect("expected redo result");

        assert_eq!(redo.direction, crate::DocumentHistoryDirection::Redo);
        assert_eq!(redo.revision, 3);
        assert_eq!(app_state.workspace.command_history.cursor, 1);
        assert!(!app_state.workspace.command_history.can_redo());
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].temperature_k,
            333.5
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Stream(stream_id))
        );
        assert!(app_state.workspace.drafts.fields.is_empty());
    }

    #[test]
    fn committing_invalid_stream_inspector_draft_is_ignored_without_document_mutation() {
        let document = inspector_focus_document();
        let mut app_state = AppState::new(document);
        let stream_id = StreamId::new("stream-feed");
        app_state.focus_inspector_target(crate::InspectorTarget::Stream(stream_id.clone()));
        let update = app_state
            .update_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::PressurePa,
                "not-a-pressure",
            )
            .expect("expected draft update");

        let outcome = app_state
            .commit_stream_inspector_draft(
                &stream_id,
                crate::StreamInspectorDraftField::PressurePa,
                timestamp(42),
            )
            .expect("expected ignored invalid commit");

        assert_eq!(outcome, None);
        assert_eq!(app_state.workspace.document.revision, 0);
        assert!(app_state.workspace.command_history.is_empty());
        assert_eq!(
            app_state.workspace.document.flowsheet.streams[&stream_id].pressure_pa,
            101_325.0
        );
        assert!(app_state.workspace.drafts.fields.contains_key(&update.key));
    }

    #[test]
    fn applying_run_panel_recovery_action_disconnects_port_and_opens_unit_inspector() {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../examples/flowsheets/failures/missing-stream-reference.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new(
                "doc-missing-stream-recovery",
                "Missing Stream Recovery Demo",
                timestamp(40),
            ),
        ));
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.missing_stream_reference: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.missing_stream_reference")
        .with_related_unit_ids(vec![UnitId::new("heater-1")])
        .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
            "heater-1",
            "outlet",
        )]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let action = app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .cloned()
            .expect("expected recovery action");

        assert_eq!(
            action.mutation,
            Some(crate::RunPanelRecoveryMutation::DisconnectPort {
                unit_id: UnitId::new("heater-1"),
                port_name: "outlet".to_string(),
            })
        );

        let applied_target = app_state.apply_run_panel_recovery_action(&action);

        assert_eq!(
            applied_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
        );
        assert!(
            app_state
                .workspace
                .selection
                .selected_units
                .contains(&UnitId::new("heater-1"))
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
        );
        assert!(app_state.workspace.panels.inspector_open);
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::DisconnectPorts {
                unit_id: UnitId::new("heater-1"),
                port: "outlet".to_string(),
            })
        );
        assert_eq!(
            app_state
                .workspace
                .document
                .flowsheet
                .units
                .get(&UnitId::new("heater-1"))
                .and_then(|unit| unit.ports.iter().find(|port| port.name == "outlet"))
                .and_then(|port| port.stream_id.as_ref())
                .map(|stream_id| stream_id.as_str()),
            None
        );
    }

    #[test]
    fn applying_run_panel_recovery_action_deletes_orphan_stream_without_selecting_missing_target() {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../examples/flowsheets/failures/orphan-stream.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new(
                "doc-orphan-stream-recovery",
                "Orphan Stream Recovery Demo",
                timestamp(41),
            ),
        ));
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.orphan_stream: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.orphan_stream")
        .with_related_stream_ids(vec![rf_types::StreamId::new("stream-orphan")]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let action = app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .cloned()
            .expect("expected recovery action");

        assert_eq!(
            action.mutation,
            Some(crate::RunPanelRecoveryMutation::DeleteStream {
                stream_id: rf_types::StreamId::new("stream-orphan"),
            })
        );

        let applied_target = app_state.apply_run_panel_recovery_action(&action);

        assert_eq!(applied_target, None);
        assert!(app_state.workspace.selection.selected_units.is_empty());
        assert!(app_state.workspace.selection.selected_streams.is_empty());
        assert_eq!(app_state.workspace.drafts.active_target, None);
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::DeleteStream {
                stream_id: rf_types::StreamId::new("stream-orphan"),
            })
        );
        assert!(
            !app_state
                .workspace
                .document
                .flowsheet
                .streams
                .contains_key(&rf_types::StreamId::new("stream-orphan"))
        );
    }

    #[test]
    fn applying_run_panel_recovery_action_creates_stream_for_unbound_outlet_and_opens_unit_inspector()
     {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../examples/flowsheets/failures/unbound-outlet-port.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new(
                "doc-unbound-outlet-recovery",
                "Unbound Outlet Recovery Demo",
                timestamp(44),
            ),
        ));
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.unbound_outlet_port: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.unbound_outlet_port")
        .with_related_unit_ids(vec![UnitId::new("feed-1")])
        .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
            "feed-1", "outlet",
        )]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let action = app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .cloned()
            .expect("expected recovery action");

        assert_eq!(
            action.mutation,
            Some(crate::RunPanelRecoveryMutation::CreateAndBindOutletStream {
                unit_id: UnitId::new("feed-1"),
                port_name: "outlet".to_string(),
            })
        );

        let applied_target = app_state.apply_run_panel_recovery_action(&action);

        assert_eq!(
            applied_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
        );
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::ConnectPorts {
                stream_id: rf_types::StreamId::new("stream-feed-1-outlet"),
                from_unit_id: UnitId::new("feed-1"),
                from_port: "outlet".to_string(),
                to_unit_id: None,
                to_port: None,
            })
        );
        assert_eq!(
            app_state
                .workspace
                .document
                .flowsheet
                .units
                .get(&UnitId::new("feed-1"))
                .and_then(|unit| unit.ports.iter().find(|port| port.name == "outlet"))
                .and_then(|port| port.stream_id.as_ref())
                .map(|stream_id| stream_id.as_str()),
            Some("stream-feed-1-outlet")
        );
        assert!(
            app_state
                .workspace
                .document
                .flowsheet
                .streams
                .contains_key(&rf_types::StreamId::new("stream-feed-1-outlet"))
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("feed-1")))
        );
        assert!(app_state.workspace.panels.inspector_open);
    }

    #[test]
    fn applying_run_panel_recovery_action_disconnects_missing_upstream_source_and_deletes_stream() {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../examples/flowsheets/failures/missing-upstream-source.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new(
                "doc-missing-upstream-recovery",
                "Missing Upstream Recovery Demo",
                timestamp(45),
            ),
        ));
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.connection_validation.missing_upstream_source: solver connection validation failed",
        )
        .with_primary_code("solver.connection_validation.missing_upstream_source")
        .with_related_unit_ids(vec![UnitId::new("mixer-1")])
        .with_related_stream_ids(vec![rf_types::StreamId::new("stream-feed-a")])
        .with_related_port_targets(vec![rf_types::DiagnosticPortTarget::new(
            "mixer-1",
            "inlet_a",
        )]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let action = app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .cloned()
            .expect("expected recovery action");

        assert_eq!(
            action.mutation,
            Some(
                crate::RunPanelRecoveryMutation::DisconnectPortAndDeleteStream {
                    unit_id: UnitId::new("mixer-1"),
                    port_name: "inlet_a".to_string(),
                    stream_id: rf_types::StreamId::new("stream-feed-a"),
                }
            )
        );

        let applied_target = app_state.apply_run_panel_recovery_action(&action);

        assert_eq!(
            applied_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("mixer-1")))
        );
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::DisconnectPortAndDeleteStream {
                unit_id: UnitId::new("mixer-1"),
                port: "inlet_a".to_string(),
                stream_id: rf_types::StreamId::new("stream-feed-a"),
            })
        );
        assert_eq!(
            app_state
                .workspace
                .document
                .flowsheet
                .units
                .get(&UnitId::new("mixer-1"))
                .and_then(|unit| unit.ports.iter().find(|port| port.name == "inlet_a"))
                .and_then(|port| port.stream_id.as_ref())
                .map(|stream_id| stream_id.as_str()),
            None
        );
        assert!(
            !app_state
                .workspace
                .document
                .flowsheet
                .streams
                .contains_key(&rf_types::StreamId::new("stream-feed-a"))
        );
    }

    #[test]
    fn applying_run_panel_recovery_action_disconnects_self_loop_inlet_and_opens_unit_inspector() {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../examples/flowsheets/failures/self-loop-cycle.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new(
                "doc-self-loop-recovery",
                "Self Loop Recovery Demo",
                timestamp(42),
            ),
        ));
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.topological_ordering.self_loop_cycle: solver topological ordering failed",
        )
        .with_primary_code("solver.topological_ordering.self_loop_cycle")
        .with_related_unit_ids(vec![UnitId::new("flash-1")])
        .with_related_stream_ids(vec![rf_types::StreamId::new("stream-loop")])
        .with_related_port_targets(vec![
            rf_types::DiagnosticPortTarget::new("flash-1", "inlet"),
            rf_types::DiagnosticPortTarget::new("flash-1", "liquid"),
        ]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let action = app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .cloned()
            .expect("expected recovery action");

        assert_eq!(
            action.mutation,
            Some(crate::RunPanelRecoveryMutation::DisconnectPort {
                unit_id: UnitId::new("flash-1"),
                port_name: "inlet".to_string(),
            })
        );

        let applied_target = app_state.apply_run_panel_recovery_action(&action);

        assert_eq!(
            applied_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("flash-1")))
        );
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::DisconnectPorts {
                unit_id: UnitId::new("flash-1"),
                port: "inlet".to_string(),
            })
        );
        assert_eq!(
            app_state
                .workspace
                .document
                .flowsheet
                .units
                .get(&UnitId::new("flash-1"))
                .and_then(|unit| unit.ports.iter().find(|port| port.name == "inlet"))
                .and_then(|port| port.stream_id.as_ref())
                .map(|stream_id| stream_id.as_str()),
            None
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("flash-1")))
        );
        assert!(app_state.workspace.panels.inspector_open);
    }

    #[test]
    fn applying_run_panel_recovery_action_disconnects_two_unit_cycle_inlet_and_opens_unit_inspector()
     {
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../examples/flowsheets/failures/multi-unit-cycle.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new(
                "doc-two-unit-cycle-recovery",
                "Two Unit Cycle Recovery Demo",
                timestamp(43),
            ),
        ));
        let summary = DiagnosticSummary::new(
            0,
            DiagnosticSeverity::Error,
            "solver.topological_ordering.two_unit_cycle: solver topological ordering failed",
        )
        .with_primary_code("solver.topological_ordering.two_unit_cycle")
        .with_related_unit_ids(vec![UnitId::new("heater-1"), UnitId::new("valve-1")])
        .with_related_stream_ids(vec![
            rf_types::StreamId::new("stream-a"),
            rf_types::StreamId::new("stream-b"),
        ])
        .with_related_port_targets(vec![
            rf_types::DiagnosticPortTarget::new("valve-1", "inlet"),
            rf_types::DiagnosticPortTarget::new("heater-1", "inlet"),
        ]);

        app_state.record_failure(0, RunStatus::Error, summary);
        let action = app_state
            .workspace
            .run_panel
            .notice
            .as_ref()
            .and_then(|notice| notice.recovery_action.as_ref())
            .cloned()
            .expect("expected recovery action");

        assert_eq!(
            action.mutation,
            Some(crate::RunPanelRecoveryMutation::DisconnectPort {
                unit_id: UnitId::new("heater-1"),
                port_name: "inlet".to_string(),
            })
        );

        let applied_target = app_state.apply_run_panel_recovery_action(&action);

        assert_eq!(
            applied_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
        );
        assert_eq!(app_state.workspace.document.revision, 1);
        assert_eq!(
            app_state
                .workspace
                .command_history
                .current_entry()
                .map(|entry| &entry.command),
            Some(&DocumentCommand::DisconnectPorts {
                unit_id: UnitId::new("heater-1"),
                port: "inlet".to_string(),
            })
        );
        assert_eq!(
            app_state
                .workspace
                .document
                .flowsheet
                .units
                .get(&UnitId::new("heater-1"))
                .and_then(|unit| unit.ports.iter().find(|port| port.name == "inlet"))
                .and_then(|port| port.stream_id.as_ref())
                .map(|stream_id| stream_id.as_str()),
            None
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(crate::InspectorTarget::Unit(UnitId::new("heater-1")))
        );
        assert!(app_state.workspace.panels.inspector_open);
    }

    #[test]
    fn completing_login_tracks_authenticated_session_without_plaintext_tokens() {
        let mut app_state = AppState::new(sample_document());
        let credential_handle =
            SecureCredentialHandle::new("radishflow-studio", "user-123-primary");
        let token_lease = TokenLease::new(timestamp(300), credential_handle.clone());
        let mut user = AuthenticatedUser::new("user-123", "luobo");
        user.tenant_id = Some("tenant-1".to_string());

        app_state.begin_browser_login("https://id.radish.local");
        app_state.complete_login("https://id.radish.local", user, token_lease, timestamp(200));

        assert_eq!(
            app_state.auth_session.status,
            AuthSessionStatus::Authenticated
        );
        assert_eq!(
            app_state
                .auth_session
                .token_lease
                .as_ref()
                .map(|lease| lease.credential_handle.account.as_str()),
            Some(credential_handle.account.as_str())
        );
    }

    #[test]
    fn entitlement_panel_disables_actions_when_session_is_signed_out() {
        let app_state = AppState::new(sample_document());

        let state =
            EntitlementPanelState::from_runtime(&app_state.auth_session, &app_state.entitlement);

        assert_eq!(
            state.commands.primary_action,
            EntitlementActionId::SyncEntitlement
        );
        assert!(
            !state
                .commands
                .action(EntitlementActionId::SyncEntitlement)
                .expect("expected sync action")
                .enabled
        );
        assert!(
            !state
                .commands
                .action(EntitlementActionId::RefreshOfflineLease)
                .expect("expected refresh action")
                .enabled
        );
    }

    #[test]
    fn entitlement_panel_prefers_offline_refresh_when_session_is_active() {
        let mut app_state = AppState::new(sample_document());
        let token_lease = TokenLease::new(
            timestamp(300),
            SecureCredentialHandle::new("radishflow-studio", "user-123-primary"),
        );
        app_state.complete_login(
            "https://id.radish.local",
            AuthenticatedUser::new("user-123", "luobo"),
            token_lease,
            timestamp(200),
        );
        app_state.update_entitlement(
            EntitlementSnapshot {
                schema_version: 1,
                subject_id: "user-123".to_string(),
                tenant_id: Some("tenant-1".to_string()),
                issued_at: timestamp(100),
                expires_at: timestamp(400),
                offline_lease_expires_at: Some(timestamp(700)),
                features: ["desktop-login".to_string()].into_iter().collect(),
                allowed_package_ids: ["binary-hydrocarbon-lite-v1".to_string()]
                    .into_iter()
                    .collect(),
            },
            vec![PropertyPackageManifest::new(
                "binary-hydrocarbon-lite-v1",
                "2026.03.1",
                PropertyPackageSource::RemoteDerivedPackage,
            )],
            timestamp(150),
        );

        let state =
            EntitlementPanelState::from_runtime(&app_state.auth_session, &app_state.entitlement);
        let widget = EntitlementPanelWidgetModel::from_state(&state);

        assert_eq!(
            state.commands.primary_action,
            EntitlementActionId::RefreshOfflineLease
        );
        assert_eq!(widget.view().primary_action.label, "Refresh offline lease");
        assert_eq!(
            widget.view().primary_action.detail,
            "Refresh the current offline lease from the control plane"
        );
        assert!(widget.view().primary_action.enabled);
        assert!(widget.text().lines.iter().any(|line| {
            line == "Primary detail: Refresh the current offline lease from the control plane"
        }));
        assert!(
            widget
                .text()
                .lines
                .iter()
                .any(|line| {
                    line == "  - Sync entitlement [enabled] | Sync entitlement and package manifests from the control plane"
                })
        );
    }

    #[test]
    fn entitlement_widget_reports_disabled_and_missing_actions() {
        let app_state = AppState::new(sample_document());
        let state =
            EntitlementPanelState::from_runtime(&app_state.auth_session, &app_state.entitlement);
        let widget = EntitlementPanelWidgetModel::from_state(&state);

        assert_eq!(
            widget.activate(EntitlementActionId::SyncEntitlement),
            EntitlementPanelWidgetEvent::Disabled {
                action_id: EntitlementActionId::SyncEntitlement,
                detail: "Sign in before syncing entitlement",
            }
        );
        assert_eq!(
            widget.activate_primary(),
            EntitlementPanelWidgetEvent::Disabled {
                action_id: EntitlementActionId::SyncEntitlement,
                detail: "Sign in before syncing entitlement",
            }
        );
        assert!(
            widget
                .text()
                .lines
                .iter()
                .any(|line| line == "Primary detail: Sign in before syncing entitlement")
        );
    }

    #[test]
    fn manifest_defaults_match_control_plane_contract_shape() {
        let bundled = PropertyPackageManifest::new(
            "bundled-pkg",
            "2026.03.1",
            PropertyPackageSource::LocalBundled,
        );
        let remote_eval = PropertyPackageManifest::new(
            "remote-eval-pkg",
            "2026.03.1",
            PropertyPackageSource::RemoteEvaluationService,
        );

        assert_eq!(bundled.schema_version, 1);
        assert!(!bundled.lease_required);
        assert_eq!(
            remote_eval.classification,
            crate::PropertyPackageClassification::RemoteOnly
        );
    }

    #[test]
    fn entitlement_sync_indexes_manifests_by_package_id() {
        let mut app_state = AppState::new(sample_document());
        let snapshot = EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            issued_at: timestamp(100),
            expires_at: timestamp(400),
            offline_lease_expires_at: Some(timestamp(700)),
            features: ["local-thermo-packages".to_string()].into_iter().collect(),
            allowed_package_ids: ["binary-hydrocarbon-lite-v1".to_string()]
                .into_iter()
                .collect(),
        };
        let mut manifest = PropertyPackageManifest::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            PropertyPackageSource::RemoteDerivedPackage,
        );
        manifest.size_bytes = 1024;

        app_state.update_entitlement(snapshot, vec![manifest], timestamp(150));

        assert!(
            app_state
                .entitlement
                .is_package_allowed("binary-hydrocarbon-lite-v1")
        );
        assert_eq!(app_state.entitlement.package_manifests.len(), 1);
    }

    #[test]
    fn clearing_auth_session_also_clears_entitlement_state() {
        let mut app_state = AppState::new(sample_document());
        let snapshot = EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: None,
            issued_at: timestamp(100),
            expires_at: timestamp(400),
            offline_lease_expires_at: None,
            features: Default::default(),
            allowed_package_ids: ["pkg-1".to_string()].into_iter().collect(),
        };

        app_state.update_entitlement(snapshot, vec![], timestamp(120));
        app_state.clear_auth_session();

        assert_eq!(app_state.auth_session.status, AuthSessionStatus::SignedOut);
        assert!(app_state.entitlement.snapshot.is_none());
        assert!(app_state.entitlement.package_manifests.is_empty());
    }

    #[test]
    fn entitlement_sync_from_manifest_list_indexes_packages() {
        let mut app_state = AppState::new(sample_document());
        let snapshot = EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            issued_at: timestamp(100),
            expires_at: timestamp(400),
            offline_lease_expires_at: Some(timestamp(700)),
            features: ["local-thermo-packages".to_string()].into_iter().collect(),
            allowed_package_ids: ["binary-hydrocarbon-lite-v1".to_string()]
                .into_iter()
                .collect(),
        };
        let manifests = PropertyPackageManifestList::new(
            timestamp(140),
            vec![PropertyPackageManifest::new(
                "binary-hydrocarbon-lite-v1",
                "2026.03.1",
                PropertyPackageSource::RemoteDerivedPackage,
            )],
        );

        app_state
            .entitlement
            .update_from_manifest_list(snapshot, manifests, timestamp(150));

        assert_eq!(app_state.entitlement.package_manifests.len(), 1);
        assert_eq!(app_state.entitlement.last_synced_at, Some(timestamp(150)));
    }

    #[test]
    fn offline_refresh_response_updates_entitlement_state() {
        let mut app_state = AppState::new(sample_document());
        let snapshot = EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            issued_at: timestamp(200),
            expires_at: timestamp(500),
            offline_lease_expires_at: Some(timestamp(900)),
            features: ["local-thermo-packages".to_string()].into_iter().collect(),
            allowed_package_ids: ["pkg-1".to_string()].into_iter().collect(),
        };
        let response = OfflineLeaseRefreshResponse {
            refreshed_at: timestamp(210),
            snapshot,
            manifest_list: PropertyPackageManifestList::new(
                timestamp(205),
                vec![PropertyPackageManifest::new(
                    "pkg-1",
                    "2026.03.1",
                    PropertyPackageSource::RemoteDerivedPackage,
                )],
            ),
        };

        app_state.entitlement.apply_offline_refresh(response);

        assert_eq!(
            app_state.entitlement.status,
            crate::EntitlementStatus::Active
        );
        assert_eq!(app_state.entitlement.last_synced_at, Some(timestamp(210)));
        assert!(app_state.entitlement.is_package_allowed("pkg-1"));
    }

    #[test]
    fn storing_solver_snapshot_maps_solver_diagnostics_into_ui_snapshot() {
        let mut app_state = AppState::new(sample_document());
        let provider = sample_solver_provider();
        let flash_solver = PlaceholderTpFlashSolver;
        let services = SolverServices {
            thermo: &provider,
            flash_solver: &flash_solver,
        };
        let project = rf_store::parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
        ))
        .expect("expected project parse");
        let solver_snapshot = SequentialModularSolver
            .solve(&services, &project.document.flowsheet)
            .expect("expected solve snapshot");

        app_state.store_solver_snapshot("snapshot-solver-1", 1, &solver_snapshot);

        let stored = app_state
            .workspace
            .snapshot_history
            .back()
            .expect("expected stored snapshot");
        assert_eq!(stored.status, RunStatus::Converged);
        assert_eq!(
            stored.summary.primary_code.as_deref(),
            Some("solver.execution_order")
        );
        assert_eq!(stored.summary.diagnostic_count, 4);
        assert_eq!(stored.diagnostics[0].code, "solver.execution_order");
        assert_eq!(stored.steps.len(), 3);
        assert_eq!(stored.steps[1].unit_id.as_str(), "heater-1");
        assert_eq!(
            stored.steps[1].streams[0].stream_id.as_str(),
            "stream-heated"
        );
        assert_eq!(
            app_state
                .workspace
                .solve_session
                .latest_snapshot
                .as_ref()
                .map(|id| id.as_str()),
            Some("snapshot-solver-1")
        );
    }
}
