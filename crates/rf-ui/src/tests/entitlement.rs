use super::*;

#[test]
fn completing_login_tracks_authenticated_session_without_plaintext_tokens() {
    let mut app_state = AppState::new(sample_document());
    let credential_handle = SecureCredentialHandle::new("radishflow-studio", "user-123-primary");
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
        "../../../../examples/flowsheets/feed-heater-flash.rfproj.json"
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
    let liquid = stored
        .streams
        .iter()
        .find(|stream| stream.stream_id.as_str() == "stream-liquid")
        .expect("expected liquid stream snapshot");
    assert_eq!(
        liquid
            .bubble_dew_window
            .as_ref()
            .expect("expected liquid stream bubble/dew window")
            .phase_region,
        rf_types::PhaseEquilibriumRegion::TwoPhase
    );
    assert!(
        stored.steps[2].streams[0]
            .bubble_dew_window
            .as_ref()
            .is_some(),
        "expected produced flash outlet snapshot to keep bubble/dew window"
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
