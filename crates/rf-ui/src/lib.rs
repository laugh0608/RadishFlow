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
    CanvasInteractionState, CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding,
    CanvasSuggestion, CanvasSuggestionAcceptance, CanvasViewMode, GhostElement, GhostElementKind,
    StreamAnimationMode, StreamVisualKind, StreamVisualState, SuggestionSource, SuggestionStatus,
};
pub use commands::{
    CanvasPoint, CommandHistory, CommandHistoryEntry, CommandValue, DocumentCommand,
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
    RunStatus, SimulationMode, SolvePendingReason, SolveSessionState, SolveSnapshot, StepSnapshot,
    StreamStateSnapshot, UnitExecutionSnapshot,
};
pub use run_panel::{
    RunPanelActionId, RunPanelActionModel, RunPanelCommandModel, RunPanelIntent, RunPanelNotice,
    RunPanelNoticeLevel, RunPanelPackageSelection, RunPanelRecoveryAction,
    RunPanelRecoveryActionKind, RunPanelState, run_panel_failure_notice,
    run_panel_failure_recovery_action_for_diagnostic_code,
    run_panel_failure_title_for_diagnostic_code,
};
pub use run_panel_presenter::RunPanelPresentation;
pub use run_panel_text::RunPanelTextView;
pub use run_panel_view::{RunPanelActionProminence, RunPanelRenderableAction, RunPanelViewModel};
pub use run_panel_widget::{RunPanelRecoveryWidgetEvent, RunPanelWidgetEvent, RunPanelWidgetModel};
pub use state::{
    AppLogEntry, AppLogFeed, AppLogLevel, AppState, AppTheme, DateTimeUtc, DocumentMetadata,
    DraftValidationState, DraftValue, FieldDraft, FlowsheetDocument, InspectorDraftState,
    InspectorTarget, LocaleCode, PanelLayoutPreferences, SelectionState, UiPanelsState,
    UserPreferences, WorkspaceState, latest_snapshot_id,
};

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use rf_flash::PlaceholderTpFlashSolver;
    use rf_model::{Flowsheet, MaterialStreamState};
    use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolverServices};
    use rf_thermo::{
        AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem,
    };
    use rf_types::UnitId;

    use crate::{
        AppLogLevel, AppState, AuthSessionStatus, AuthenticatedUser,
        CanvasPoint, CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding,
        CanvasSuggestion, CanvasSuggestionAcceptance, CanvasSuggestionId, CanvasViewMode,
        CommandHistory, CommandHistoryEntry,
        DiagnosticSeverity, DiagnosticSummary, DocumentCommand, DocumentMetadata,
        EntitlementActionId, EntitlementPanelState, EntitlementPanelWidgetEvent,
        EntitlementPanelWidgetModel, EntitlementSnapshot, FlowsheetDocument, GhostElement,
        GhostElementKind, OfflineLeaseRefreshResponse, PropertyPackageManifest,
        PropertyPackageManifestList, PropertyPackageSource, RunPanelActionId,
        RunPanelActionProminence, RunPanelPresentation, RunPanelRecoveryWidgetEvent, RunPanelState,
        RunPanelTextView, RunPanelViewModel, RunPanelWidgetEvent, RunPanelWidgetModel, RunStatus,
        SecureCredentialHandle, SimulationMode, SolvePendingReason, SolveSnapshot,
        StreamVisualKind, StreamVisualState, SuggestionSource, SuggestionStatus, TokenLease,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        let flowsheet = Flowsheet::new("demo");
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
        assert_eq!(
            app_state.workspace.document.revision,
            1
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
                message:
                    "Accepted canvas suggestion `sug-high` from local rules for unit flash-1"
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
            sample_canvas_suggestion("sug-liquid", 0.92, SuggestionSource::LocalRules).with_acceptance(
                CanvasSuggestionAcceptance::MaterialConnection(CanvasSuggestedMaterialConnection {
                    stream: CanvasSuggestedStreamBinding::Create {
                        stream: MaterialStreamState::new("stream-liquid", "Liquid Outlet"),
                    },
                    source_unit_id: UnitId::new("flash-1"),
                    source_port: "liquid".to_string(),
                    sink_unit_id: None,
                    sink_port: None,
                }),
            ),
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
        assert!(text.lines.iter().any(|line| line == "  - Run [enabled]"));
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
    fn applying_run_panel_recovery_action_selects_unit_and_opens_inspector() {
        let mut app_state = AppState::new(sample_document());
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
        assert!(widget.view().primary_action.enabled);
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
                action_id: EntitlementActionId::SyncEntitlement
            }
        );
        assert_eq!(
            widget.activate_primary(),
            EntitlementPanelWidgetEvent::Disabled {
                action_id: EntitlementActionId::SyncEntitlement
            }
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
