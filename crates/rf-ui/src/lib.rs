mod auth;
mod commands;
mod diagnostics;
mod ids;
mod run;
mod state;

pub use auth::{
    AuditUsageAck, AuditUsageRequest, AuthSessionState, AuthSessionStatus, AuthenticatedUser,
    EntitlementSnapshot, EntitlementState, EntitlementStatus, OfflineLeaseRefreshRequest,
    OfflineLeaseRefreshResponse, PropertyPackageClassification, PropertyPackageLeaseGrant,
    PropertyPackageLeaseRequest, PropertyPackageManifest, PropertyPackageManifestList,
    PropertyPackageSource, PropertyPackageUsageEvent, PropertyPackageUsageEventKind,
    SecureCredentialHandle, TokenLease,
};
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
        AppState, AuthSessionStatus, AuthenticatedUser, CanvasPoint, CommandHistory,
        CommandHistoryEntry, DiagnosticSeverity, DiagnosticSummary, DocumentCommand,
        DocumentMetadata, EntitlementSnapshot, FlowsheetDocument, OfflineLeaseRefreshResponse,
        PropertyPackageManifest, PropertyPackageManifestList, PropertyPackageSource, RunStatus,
        SecureCredentialHandle, SimulationMode, SolvePendingReason, SolveSnapshot, TokenLease,
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
}
