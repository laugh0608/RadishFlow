use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    EntitlementPanelDriverOutcome, EntitlementPanelWidgetDispatchOutcome,
    RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
    RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneResponse, RunPanelDriverOutcome,
    RunPanelWidgetDispatchOutcome, StudioAppAuthCacheContext, StudioAppCommandOutcome,
    StudioAppFacade, StudioAppMutableAuthCacheContext, WorkspaceControlState,
    dispatch_entitlement_panel_primary_action_with_control_plane,
    dispatch_entitlement_panel_widget_action_with_control_plane,
    dispatch_run_panel_intent_with_auth_cache, dispatch_run_panel_primary_action_with_auth_cache,
    dispatch_run_panel_widget_action_with_auth_cache, snapshot_entitlement_panel_driver_state,
    snapshot_run_panel_driver_state,
};
use rf_model::Flowsheet;
use rf_store::{
    StoredAntoineCoefficients, StoredAuthCacheIndex, StoredCredentialReference,
    StoredEntitlementCache, StoredPropertyPackageManifest, StoredPropertyPackagePayload,
    StoredPropertyPackageRecord, StoredPropertyPackageSource, StoredThermoComponent,
    property_package_payload_integrity, read_project_file, write_auth_cache_index,
    write_property_package_manifest, write_property_package_payload,
};
use rf_types::{RfError, RfResult};
use rf_ui::{
    AppLogEntry, AppState, AuthenticatedUser, DocumentMetadata, EntitlementActionId,
    EntitlementPanelWidgetModel, EntitlementSnapshot, FlowsheetDocument,
    OfflineLeaseRefreshRequest, OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant,
    PropertyPackageLeaseRequest, PropertyPackageManifest, PropertyPackageManifestList,
    PropertyPackageSource, RunPanelActionId, RunPanelIntent, RunPanelWidgetModel,
    SecureCredentialHandle, TokenLease,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapConfig {
    pub project_path: PathBuf,
    pub trigger: StudioBootstrapTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioBootstrapTrigger {
    Intent(RunPanelIntent),
    WidgetPrimaryAction,
    WidgetAction(RunPanelActionId),
    EntitlementWidgetPrimaryAction,
    EntitlementWidgetAction(EntitlementActionId),
}

impl Default for StudioBootstrapConfig {
    fn default() -> Self {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");

        Self {
            project_path: workspace_root
                .join("examples")
                .join("flowsheets")
                .join("feed-heater-flash.rfproj.json"),
            trigger: StudioBootstrapTrigger::WidgetPrimaryAction,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapReport {
    pub outcome: StudioAppCommandOutcome,
    pub control_state: WorkspaceControlState,
    pub run_panel: RunPanelWidgetModel,
    pub entitlement_panel: EntitlementPanelWidgetModel,
    pub log_entries: Vec<AppLogEntry>,
}

pub fn run_studio_bootstrap(config: &StudioBootstrapConfig) -> RfResult<StudioBootstrapReport> {
    let project_file = read_project_file(&config.project_path)?;
    let mut app_state = app_state_from_project_file(&project_file, &config.project_path);
    let cache_root = TemporaryCacheRoot::new("studio-bootstrap")?;
    let seeded_auth_cache = seed_sample_auth_cache(
        cache_root.path(),
        &project_file.document.flowsheet,
        "binary-hydrocarbon-lite-v1",
    )?;
    seed_bootstrap_runtime_state(&mut app_state, &seeded_auth_cache);
    let control_plane_client = BootstrapControlPlaneClient::from_seed(&seeded_auth_cache);
    let mut auth_cache_index = seeded_auth_cache.auth_cache_index;
    let facade = StudioAppFacade::new();
    let outcome = dispatch_bootstrap_trigger(
        &facade,
        &mut app_state,
        cache_root.path(),
        &mut auth_cache_index,
        &config.trigger,
        &control_plane_client,
    )?;
    let driver_state = snapshot_run_panel_driver_state(&app_state);
    let entitlement_driver_state = snapshot_entitlement_panel_driver_state(&app_state);

    Ok(StudioBootstrapReport {
        outcome,
        control_state: driver_state.control_state,
        run_panel: driver_state.widget,
        entitlement_panel: entitlement_driver_state.widget,
        log_entries: app_state.log_feed.entries.iter().cloned().collect(),
    })
}

fn dispatch_bootstrap_trigger(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    cache_root: &Path,
    auth_cache_index: &mut StoredAuthCacheIndex,
    trigger: &StudioBootstrapTrigger,
    control_plane_client: &BootstrapControlPlaneClient,
) -> RfResult<StudioAppCommandOutcome> {
    match trigger {
        StudioBootstrapTrigger::Intent(intent) => {
            let context = StudioAppAuthCacheContext::new(cache_root, &*auth_cache_index);
            command_outcome_from_workspace_control(dispatch_run_panel_intent_with_auth_cache(
                facade, app_state, &context, intent,
            )?)
        }
        StudioBootstrapTrigger::WidgetPrimaryAction => {
            let context = StudioAppAuthCacheContext::new(cache_root, &*auth_cache_index);
            match dispatch_run_panel_primary_action_with_auth_cache(facade, app_state, &context)? {
                RunPanelDriverOutcome {
                    dispatch: RunPanelWidgetDispatchOutcome::Executed(outcome),
                    ..
                } => command_outcome_from_workspace_control(outcome),
                RunPanelDriverOutcome {
                    dispatch: RunPanelWidgetDispatchOutcome::IgnoredDisabled { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap primary widget action `{:?}` is currently disabled",
                    action_id
                ))),
                RunPanelDriverOutcome {
                    dispatch: RunPanelWidgetDispatchOutcome::IgnoredMissing { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap primary widget action `{:?}` is missing from current widget model",
                    action_id
                ))),
            }
        }
        StudioBootstrapTrigger::WidgetAction(action_id) => {
            let context = StudioAppAuthCacheContext::new(cache_root, &*auth_cache_index);
            match dispatch_run_panel_widget_action_with_auth_cache(
                facade, app_state, &context, *action_id,
            )? {
                RunPanelDriverOutcome {
                    dispatch: RunPanelWidgetDispatchOutcome::Executed(outcome),
                    ..
                } => command_outcome_from_workspace_control(outcome),
                RunPanelDriverOutcome {
                    dispatch: RunPanelWidgetDispatchOutcome::IgnoredDisabled { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap widget action `{:?}` is currently disabled",
                    action_id
                ))),
                RunPanelDriverOutcome {
                    dispatch: RunPanelWidgetDispatchOutcome::IgnoredMissing { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap widget action `{:?}` is missing from current widget model",
                    action_id
                ))),
            }
        }
        StudioBootstrapTrigger::EntitlementWidgetPrimaryAction => {
            let mut context = StudioAppMutableAuthCacheContext::new(cache_root, auth_cache_index);
            match dispatch_entitlement_panel_primary_action_with_control_plane(
                facade,
                app_state,
                &mut context,
                control_plane_client,
                "bootstrap-access-token",
            )? {
                EntitlementPanelDriverOutcome {
                    dispatch: EntitlementPanelWidgetDispatchOutcome::Executed(outcome),
                    ..
                } => Ok(outcome),
                EntitlementPanelDriverOutcome {
                    dispatch: EntitlementPanelWidgetDispatchOutcome::IgnoredDisabled { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap entitlement primary widget action `{:?}` is currently disabled",
                    action_id
                ))),
                EntitlementPanelDriverOutcome {
                    dispatch: EntitlementPanelWidgetDispatchOutcome::IgnoredMissing { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap entitlement primary widget action `{:?}` is missing from current widget model",
                    action_id
                ))),
            }
        }
        StudioBootstrapTrigger::EntitlementWidgetAction(action_id) => {
            let mut context = StudioAppMutableAuthCacheContext::new(cache_root, auth_cache_index);
            match dispatch_entitlement_panel_widget_action_with_control_plane(
                facade,
                app_state,
                &mut context,
                control_plane_client,
                "bootstrap-access-token",
                *action_id,
            )? {
                EntitlementPanelDriverOutcome {
                    dispatch: EntitlementPanelWidgetDispatchOutcome::Executed(outcome),
                    ..
                } => Ok(outcome),
                EntitlementPanelDriverOutcome {
                    dispatch: EntitlementPanelWidgetDispatchOutcome::IgnoredDisabled { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap entitlement widget action `{:?}` is currently disabled",
                    action_id
                ))),
                EntitlementPanelDriverOutcome {
                    dispatch: EntitlementPanelWidgetDispatchOutcome::IgnoredMissing { action_id },
                    ..
                } => Err(RfError::invalid_input(format!(
                    "bootstrap entitlement widget action `{:?}` is missing from current widget model",
                    action_id
                ))),
            }
        }
    }
}

fn command_outcome_from_workspace_control(
    outcome: crate::WorkspaceControlActionOutcome,
) -> RfResult<StudioAppCommandOutcome> {
    Ok(StudioAppCommandOutcome {
        boundary: outcome.boundary,
        dispatch: outcome.dispatch,
    })
}

#[derive(Debug, Clone)]
struct BootstrapSeedState {
    auth_cache_index: StoredAuthCacheIndex,
    snapshot: EntitlementSnapshot,
    manifest: PropertyPackageManifest,
    synced_at: SystemTime,
}

#[derive(Debug, Clone)]
struct BootstrapControlPlaneClient {
    synced_snapshot: EntitlementSnapshot,
    manifest_list: PropertyPackageManifestList,
    sync_received_at: SystemTime,
    refresh_response: OfflineLeaseRefreshResponse,
    refresh_received_at: SystemTime,
}

impl BootstrapControlPlaneClient {
    fn from_seed(seed: &BootstrapSeedState) -> Self {
        let sync_received_at = seed.synced_at + Duration::from_secs(300);
        let refresh_received_at = seed.synced_at + Duration::from_secs(600);

        let mut synced_snapshot = seed.snapshot.clone();
        synced_snapshot.expires_at = sync_received_at + Duration::from_secs(3_600);
        synced_snapshot.offline_lease_expires_at =
            Some(sync_received_at + Duration::from_secs(7_200));

        let mut refreshed_snapshot = synced_snapshot.clone();
        refreshed_snapshot.offline_lease_expires_at =
            Some(refresh_received_at + Duration::from_secs(7_200));

        Self {
            synced_snapshot: synced_snapshot.clone(),
            manifest_list: PropertyPackageManifestList::new(
                sync_received_at,
                vec![seed.manifest.clone()],
            ),
            sync_received_at,
            refresh_response: OfflineLeaseRefreshResponse {
                refreshed_at: refresh_received_at,
                snapshot: refreshed_snapshot,
                manifest_list: PropertyPackageManifestList::new(
                    refresh_received_at,
                    vec![seed.manifest.clone()],
                ),
            },
            refresh_received_at,
        }
    }
}

impl RadishFlowControlPlaneClient for BootstrapControlPlaneClient {
    fn fetch_entitlement_snapshot(
        &self,
        _access_token: &str,
    ) -> Result<
        RadishFlowControlPlaneResponse<EntitlementSnapshot>,
        RadishFlowControlPlaneClientError,
    > {
        Ok(RadishFlowControlPlaneResponse::new(
            self.synced_snapshot.clone(),
            self.sync_received_at,
        ))
    }

    fn fetch_property_package_manifest_list(
        &self,
        _access_token: &str,
    ) -> Result<
        RadishFlowControlPlaneResponse<PropertyPackageManifestList>,
        RadishFlowControlPlaneClientError,
    > {
        Ok(RadishFlowControlPlaneResponse::new(
            self.manifest_list.clone(),
            self.sync_received_at,
        ))
    }

    fn request_property_package_lease(
        &self,
        _access_token: &str,
        _package_id: &str,
        _request: &PropertyPackageLeaseRequest,
    ) -> Result<
        RadishFlowControlPlaneResponse<PropertyPackageLeaseGrant>,
        RadishFlowControlPlaneClientError,
    > {
        Err(RadishFlowControlPlaneClientError::new(
            RadishFlowControlPlaneClientErrorKind::OtherPermanent,
            "bootstrap control plane client does not issue property package leases",
        ))
    }

    fn refresh_offline_leases(
        &self,
        _access_token: &str,
        _request: &OfflineLeaseRefreshRequest,
    ) -> Result<
        RadishFlowControlPlaneResponse<OfflineLeaseRefreshResponse>,
        RadishFlowControlPlaneClientError,
    > {
        Ok(RadishFlowControlPlaneResponse::new(
            self.refresh_response.clone(),
            self.refresh_received_at,
        ))
    }
}

fn app_state_from_project_file(
    project_file: &rf_store::StoredProjectFile,
    project_path: &Path,
) -> AppState {
    let metadata = &project_file.document.metadata;
    let mut document = FlowsheetDocument::new(
        project_file.document.flowsheet.clone(),
        DocumentMetadata::new(
            metadata.document_id.clone(),
            metadata.title.clone(),
            metadata.created_at,
        ),
    );
    document.revision = project_file.document.revision;
    document.metadata.schema_version = metadata.schema_version;
    document.metadata.updated_at = metadata.updated_at;

    let mut app_state = AppState::new(document);
    app_state.mark_saved(project_path.to_path_buf());
    app_state
}

fn seed_sample_auth_cache(
    cache_root: &Path,
    flowsheet: &Flowsheet,
    package_id: &str,
) -> RfResult<BootstrapSeedState> {
    let downloaded_at = normalized_system_time_now()?;
    let payload = build_bootstrap_payload(flowsheet, package_id)?;
    let integrity = property_package_payload_integrity(&payload)?;
    let mut manifest = StoredPropertyPackageManifest::new(
        package_id,
        "2026.04.2",
        StoredPropertyPackageSource::RemoteDerivedPackage,
        payload.component_ids(),
    );
    manifest.hash = integrity.hash.clone();
    manifest.size_bytes = integrity.size_bytes;
    manifest.expires_at = Some(downloaded_at + Duration::from_secs(3_600));

    let mut record = StoredPropertyPackageRecord::new(
        &manifest.package_id,
        &manifest.version,
        manifest.source,
        integrity.hash.clone(),
        integrity.size_bytes,
        downloaded_at,
    );
    record.expires_at = manifest.expires_at;

    write_property_package_manifest(record.manifest_path_under(cache_root), &manifest)?;
    write_property_package_payload(
        record.payload_path_under(cache_root).ok_or_else(|| {
            RfError::invalid_input(format!(
                "sample property package `{}` is missing a local payload path",
                manifest.package_id
            ))
        })?,
        &payload,
    )?;

    let mut auth_cache_index = StoredAuthCacheIndex::new(
        "https://id.radish.local",
        "bootstrap-user",
        StoredCredentialReference::new("radishflow-studio", "bootstrap-user-primary"),
    );
    let snapshot = bootstrap_snapshot(package_id, downloaded_at);
    auth_cache_index.entitlement = Some(StoredEntitlementCache {
        subject_id: snapshot.subject_id.clone(),
        tenant_id: snapshot.tenant_id.clone(),
        synced_at: downloaded_at,
        issued_at: snapshot.issued_at,
        expires_at: snapshot.expires_at,
        offline_lease_expires_at: snapshot.offline_lease_expires_at,
        feature_keys: snapshot.features.clone(),
        allowed_package_ids: snapshot.allowed_package_ids.clone(),
    });
    auth_cache_index.property_packages.push(record);
    auth_cache_index.last_synced_at = Some(downloaded_at);
    write_auth_cache_index(
        auth_cache_index.index_path_under(cache_root),
        &auth_cache_index,
    )?;
    Ok(BootstrapSeedState {
        auth_cache_index,
        snapshot,
        manifest: bootstrap_manifest(
            package_id,
            &integrity.hash,
            integrity.size_bytes,
            downloaded_at,
        ),
        synced_at: downloaded_at,
    })
}

fn seed_bootstrap_runtime_state(app_state: &mut AppState, seed: &BootstrapSeedState) {
    app_state.complete_login(
        "https://id.radish.local",
        AuthenticatedUser::new("bootstrap-user", "bootstrap-demo"),
        TokenLease::new(
            seed.snapshot.expires_at,
            SecureCredentialHandle::new("radishflow-studio", "bootstrap-user-primary"),
        ),
        seed.synced_at,
    );
    app_state.update_entitlement(
        seed.snapshot.clone(),
        vec![seed.manifest.clone()],
        seed.synced_at,
    );
}

fn bootstrap_snapshot(package_id: &str, synced_at: SystemTime) -> EntitlementSnapshot {
    EntitlementSnapshot {
        schema_version: 1,
        subject_id: "bootstrap-user".to_string(),
        tenant_id: Some("bootstrap-tenant".to_string()),
        issued_at: synced_at - Duration::from_secs(60),
        expires_at: synced_at + Duration::from_secs(3_600),
        offline_lease_expires_at: Some(synced_at + Duration::from_secs(7_200)),
        features: std::collections::BTreeSet::from(["desktop-login".to_string()]),
        allowed_package_ids: std::collections::BTreeSet::from([package_id.to_string()]),
    }
}

fn bootstrap_manifest(
    package_id: &str,
    hash: &str,
    size_bytes: u64,
    downloaded_at: SystemTime,
) -> PropertyPackageManifest {
    let mut manifest = PropertyPackageManifest::new(
        package_id,
        "2026.04.2",
        PropertyPackageSource::RemoteDerivedPackage,
    );
    manifest.hash = hash.to_string();
    manifest.size_bytes = size_bytes;
    manifest.expires_at = Some(downloaded_at + Duration::from_secs(3_600));
    manifest
}

fn normalized_system_time_now() -> RfResult<SystemTime> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            RfError::invalid_input(format!("normalize current timestamp failed: {error}"))
        })?
        .as_secs();
    Ok(UNIX_EPOCH + Duration::from_secs(seconds))
}

fn build_bootstrap_payload(
    flowsheet: &Flowsheet,
    package_id: &str,
) -> RfResult<StoredPropertyPackagePayload> {
    let components = flowsheet.components.values().collect::<Vec<_>>();
    if components.len() != 2 {
        return Err(RfError::invalid_input(format!(
            "studio bootstrap expects exactly 2 flowsheet components, got {}",
            components.len()
        )));
    }

    let mut more_volatile =
        StoredThermoComponent::new(components[0].id.clone(), components[0].name.clone());
    more_volatile.antoine = Some(StoredAntoineCoefficients::new(
        ((2.0_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));
    more_volatile.liquid_heat_capacity_j_per_mol_k = Some(35.0);
    more_volatile.vapor_heat_capacity_j_per_mol_k = Some(36.5);

    let mut less_volatile =
        StoredThermoComponent::new(components[1].id.clone(), components[1].name.clone());
    less_volatile.antoine = Some(StoredAntoineCoefficients::new(
        ((0.5_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
        0.0,
        0.0,
    ));
    less_volatile.liquid_heat_capacity_j_per_mol_k = Some(52.0);
    less_volatile.vapor_heat_capacity_j_per_mol_k = Some(65.0);

    Ok(StoredPropertyPackagePayload::new(
        package_id,
        "2026.04.2",
        vec![more_volatile, less_volatile],
    ))
}

#[derive(Debug)]
struct TemporaryCacheRoot {
    path: PathBuf,
}

impl TemporaryCacheRoot {
    fn new(prefix: &str) -> RfResult<Self> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| {
                RfError::invalid_input(format!(
                    "create temporary cache root timestamp failed: {error}"
                ))
            })?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("radishflow-{prefix}-{unique}"));
        fs::create_dir_all(&path).map_err(|error| {
            RfError::invalid_input(format!(
                "create temporary cache root `{}`: {error}",
                path.display()
            ))
        })?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryCacheRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use rf_ui::{
        EntitlementActionId, RunPanelActionId, RunPanelIntent, RunPanelPackageSelection, RunStatus,
        SimulationMode,
    };

    use super::{StudioBootstrapConfig, StudioBootstrapTrigger, run_studio_bootstrap};
    use crate::{
        StudioAppExecutionBoundary, StudioAppExecutionLane, StudioAppResultDispatch,
        StudioEntitlementAction, StudioEntitlementOutcome, StudioWorkspaceRunOutcome,
    };

    #[test]
    fn bootstrap_runs_sample_workspace_from_main_entry_boundary() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig::default())
            .expect("expected bootstrap run");

        assert_eq!(
            report.outcome.boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceSolve)
        );
        let dispatch = match report.outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(
            report.control_state.simulation_mode,
            dispatch.simulation_mode
        );
        assert_eq!(report.control_state.run_status, dispatch.run_status);
        assert_eq!(dispatch.run_status, RunStatus::Converged);
        assert_eq!(
            dispatch.package_id.as_deref(),
            Some("binary-hydrocarbon-lite-v1")
        );
        assert!(matches!(
            dispatch.outcome,
            StudioWorkspaceRunOutcome::Started(_)
        ));
        assert_eq!(
            dispatch.latest_snapshot_summary.as_deref(),
            Some(
                "solved flowsheet with 3 unit(s), 4 diagnostic entry(ies), and 4 resulting stream(s)"
            )
        );
        assert_eq!(report.run_panel.view().mode_label, "Active");
        assert_eq!(report.run_panel.view().status_label, "Converged");
        assert_eq!(report.run_panel.view().primary_action.label, "Run");
        assert_eq!(report.run_panel.view().secondary_actions.len(), 2);
        assert_eq!(dispatch.log_entry_count, 2);
        assert_eq!(report.log_entries.len(), 2);
    }

    #[test]
    fn bootstrap_resumes_workspace_from_hold_via_run_panel_intent() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            trigger: StudioBootstrapTrigger::Intent(RunPanelIntent::resume(
                RunPanelPackageSelection::preferred(),
            )),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected bootstrap resume");

        let dispatch = match report.outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(
            report.control_state.simulation_mode,
            dispatch.simulation_mode
        );
        assert_eq!(report.control_state.run_status, dispatch.run_status);
        assert_eq!(dispatch.run_status, RunStatus::Converged);
        assert!(matches!(
            dispatch.outcome,
            StudioWorkspaceRunOutcome::Started(_)
        ));
        assert_eq!(dispatch.log_entry_count, 2);
        assert_eq!(report.log_entries.len(), 2);
        assert_eq!(
            report.log_entries[0].message,
            "Activated workspace simulation mode"
        );
    }

    #[test]
    fn bootstrap_accepts_preferred_package_selection_when_single_cached_package_exists() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            trigger: StudioBootstrapTrigger::Intent(RunPanelIntent::run_manual(
                RunPanelPackageSelection::preferred(),
            )),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected preferred package bootstrap run");

        let dispatch = match report.outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(
            report.control_state.simulation_mode,
            dispatch.simulation_mode
        );
        assert_eq!(
            dispatch.package_id.as_deref(),
            Some("binary-hydrocarbon-lite-v1")
        );
        assert_eq!(dispatch.log_entry_count, 1);
    }

    #[test]
    fn bootstrap_can_switch_workspace_mode_from_run_panel_intent() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            trigger: StudioBootstrapTrigger::Intent(RunPanelIntent::set_mode(
                SimulationMode::Active,
            )),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected mode intent bootstrap run");

        match report.outcome.dispatch {
            StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
                assert_eq!(dispatch.run_status, RunStatus::Idle);
            }
            StudioAppResultDispatch::WorkspaceRun(_) => panic!("expected workspace mode dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace mode dispatch"),
        }
        assert_eq!(report.control_state.simulation_mode, SimulationMode::Active);
        assert_eq!(report.run_panel.view().mode_label, "Active");
        assert_eq!(report.run_panel.view().primary_action.label, "Run");
        assert_eq!(report.log_entries.len(), 1);
        assert_eq!(
            report.log_entries[0].message,
            "Set workspace simulation mode to Active"
        );
    }

    #[test]
    fn bootstrap_can_dispatch_run_via_widget_action() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            trigger: StudioBootstrapTrigger::WidgetAction(RunPanelActionId::RunManual),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected widget action bootstrap run");

        let dispatch = match report.outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(dispatch.run_status, RunStatus::Converged);
        assert_eq!(report.run_panel.view().primary_action.label, "Run");
    }

    #[test]
    fn bootstrap_default_trigger_runs_via_primary_widget_action() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig::default())
            .expect("expected primary widget bootstrap run");

        let dispatch = match report.outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(dispatch.run_status, RunStatus::Converged);
    }

    #[test]
    fn bootstrap_can_sync_entitlement_via_control_plane_trigger() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            trigger: StudioBootstrapTrigger::EntitlementWidgetAction(
                EntitlementActionId::SyncEntitlement,
            ),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected entitlement sync bootstrap run");

        assert_eq!(
            report.outcome.boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::EntitlementControl)
        );
        match report.outcome.dispatch {
            StudioAppResultDispatch::Entitlement(dispatch) => {
                assert_eq!(dispatch.action, StudioEntitlementAction::SyncEntitlement);
                assert_eq!(dispatch.outcome, StudioEntitlementOutcome::Synced);
                assert_eq!(
                    dispatch.notice.as_ref().map(|notice| notice.title.as_str()),
                    Some("Entitlement synced")
                );
            }
            StudioAppResultDispatch::WorkspaceRun(_) => panic!("expected entitlement dispatch"),
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected entitlement dispatch"),
        }
        assert_eq!(report.control_state.run_status, RunStatus::Idle);
        assert_eq!(report.run_panel.view().primary_action.label, "Resume");
        assert_eq!(
            report.entitlement_panel.view().primary_action.label,
            "Refresh offline lease"
        );
        assert_eq!(report.log_entries.len(), 1);
        assert_eq!(
            report.log_entries[0].message,
            "Synced entitlement snapshot and property package manifests from control plane"
        );
    }

    #[test]
    fn bootstrap_can_refresh_offline_lease_via_control_plane_trigger() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            trigger: StudioBootstrapTrigger::EntitlementWidgetPrimaryAction,
            ..StudioBootstrapConfig::default()
        })
        .expect("expected offline refresh bootstrap run");

        assert_eq!(
            report.outcome.boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::EntitlementControl)
        );
        match report.outcome.dispatch {
            StudioAppResultDispatch::Entitlement(dispatch) => {
                assert_eq!(
                    dispatch.action,
                    StudioEntitlementAction::RefreshOfflineLease
                );
                assert_eq!(
                    dispatch.outcome,
                    StudioEntitlementOutcome::OfflineLeaseRefreshed
                );
                assert_eq!(
                    dispatch.notice.as_ref().map(|notice| notice.title.as_str()),
                    Some("Offline lease refreshed")
                );
            }
            StudioAppResultDispatch::WorkspaceRun(_) => panic!("expected entitlement dispatch"),
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected entitlement dispatch"),
        }
        assert_eq!(report.control_state.run_status, RunStatus::Idle);
        assert_eq!(report.run_panel.view().primary_action.label, "Resume");
        assert_eq!(
            report.entitlement_panel.view().primary_action.label,
            "Refresh offline lease"
        );
        assert_eq!(report.log_entries.len(), 1);
        assert_eq!(
            report.log_entries[0].message,
            "Refreshed offline lease state from control plane"
        );
    }
}
