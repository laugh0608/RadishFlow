use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    EntitlementPreflightOutcome, EntitlementSessionEvent, EntitlementSessionEventDriverOutcome,
    EntitlementSessionHostDispatch, EntitlementSessionHostSnapshot, EntitlementSessionHostTrigger,
    EntitlementSessionLifecycleEvent, EntitlementSessionPanelDriverOutcome,
    EntitlementSessionPolicy, EntitlementSessionRuntime, EntitlementSessionState,
    RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
    RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneResponse, RunPanelDriverOutcome,
    RunPanelWidgetDispatchOutcome, StudioAppAuthCacheContext, StudioAppCommandOutcome,
    StudioAppFacade, StudioAppMutableAuthCacheContext, WorkspaceControlState,
    dispatch_entitlement_session_event_with_control_plane,
    dispatch_entitlement_session_host_trigger_with_control_plane,
    dispatch_run_panel_intent_with_auth_cache, dispatch_run_panel_primary_action_with_auth_cache,
    dispatch_run_panel_widget_action_with_auth_cache, snapshot_entitlement_session_driver_state,
    snapshot_entitlement_session_host, snapshot_run_panel_driver_state,
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
    EntitlementSnapshot, FlowsheetDocument, OfflineLeaseRefreshRequest,
    OfflineLeaseRefreshResponse, PropertyPackageLeaseGrant, PropertyPackageLeaseRequest,
    PropertyPackageManifest, PropertyPackageManifestList, PropertyPackageSource, RunPanelActionId,
    RunPanelIntent, RunPanelWidgetModel, SecureCredentialHandle, TokenLease,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapConfig {
    pub project_path: PathBuf,
    pub entitlement_preflight: StudioBootstrapEntitlementPreflight,
    pub entitlement_seed: StudioBootstrapEntitlementSeed,
    pub trigger: StudioBootstrapTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioBootstrapTrigger {
    Intent(RunPanelIntent),
    WidgetPrimaryAction,
    WidgetAction(RunPanelActionId),
    EntitlementWidgetPrimaryAction,
    EntitlementWidgetAction(EntitlementActionId),
    EntitlementSessionEvent(StudioBootstrapEntitlementSessionEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioBootstrapEntitlementPreflight {
    Skip,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioBootstrapEntitlementSeed {
    Synced,
    MissingSnapshot,
    LeaseExpiringSoon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioBootstrapEntitlementSessionEvent {
    LoginCompleted,
    TimerElapsed,
    NetworkRestored,
    WindowForegrounded,
}

impl Default for StudioBootstrapConfig {
    fn default() -> Self {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");

        Self {
            project_path: workspace_root
                .join("examples")
                .join("flowsheets")
                .join("feed-heater-flash.rfproj.json"),
            entitlement_preflight: StudioBootstrapEntitlementPreflight::Auto,
            entitlement_seed: StudioBootstrapEntitlementSeed::Synced,
            trigger: StudioBootstrapTrigger::WidgetPrimaryAction,
        }
    }
}

struct BootstrapSessionResources<'a> {
    app_state: &'a mut AppState,
    cache_root: &'a Path,
    auth_cache_index: &'a mut StoredAuthCacheIndex,
    control_plane_client: &'a BootstrapControlPlaneClient,
    policy: &'a EntitlementSessionPolicy,
    session_state: &'a mut EntitlementSessionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapReport {
    pub entitlement_preflight: Option<EntitlementPreflightOutcome>,
    pub entitlement_host: EntitlementSessionHostSnapshot,
    pub dispatch: StudioBootstrapDispatch,
    pub control_state: WorkspaceControlState,
    pub run_panel: RunPanelWidgetModel,
    pub log_entries: Vec<AppLogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioBootstrapDispatch {
    AppCommand(StudioAppCommandOutcome),
    EntitlementSessionEvent(EntitlementSessionEventDriverOutcome),
}

pub fn run_studio_bootstrap(config: &StudioBootstrapConfig) -> RfResult<StudioBootstrapReport> {
    let project_file = read_project_file(&config.project_path)?;
    let mut app_state = app_state_from_project_file(&project_file, &config.project_path);
    let cache_root = TemporaryCacheRoot::new("studio-bootstrap")?;
    let seeded_auth_cache = seed_sample_auth_cache(
        cache_root.path(),
        &project_file.document.flowsheet,
        "binary-hydrocarbon-lite-v1",
        config.entitlement_seed,
    )?;
    seed_bootstrap_runtime_state(&mut app_state, &seeded_auth_cache);
    let control_plane_client = BootstrapControlPlaneClient::from_seed(&seeded_auth_cache);
    let mut auth_cache_index = seeded_auth_cache.auth_cache_index;
    let facade = StudioAppFacade::new();
    let session_policy = EntitlementSessionPolicy::default();
    let mut entitlement_session_state = EntitlementSessionState::default();
    let mut session_resources = BootstrapSessionResources {
        app_state: &mut app_state,
        cache_root: cache_root.path(),
        auth_cache_index: &mut auth_cache_index,
        control_plane_client: &control_plane_client,
        policy: &session_policy,
        session_state: &mut entitlement_session_state,
    };
    let entitlement_session_tick = dispatch_bootstrap_entitlement_session_tick(
        &facade,
        &config.entitlement_preflight,
        &mut session_resources,
    )?;
    let dispatch = dispatch_bootstrap_trigger(&facade, &config.trigger, &mut session_resources)?;
    let schedule_now = normalized_system_time_now()?;
    let driver_state = snapshot_run_panel_driver_state(session_resources.app_state);
    let entitlement_host = snapshot_entitlement_session_host(
        session_resources.app_state,
        schedule_now,
        &session_policy,
        session_resources.session_state,
        None,
    );

    Ok(StudioBootstrapReport {
        entitlement_preflight: match entitlement_session_tick.outcome {
            crate::EntitlementSessionEventOutcome::Tick(tick) => tick.preflight,
            crate::EntitlementSessionEventOutcome::RecordedCommand { .. } => None,
        },
        entitlement_host,
        dispatch,
        control_state: driver_state.control_state,
        run_panel: driver_state.widget,
        log_entries: app_state.log_feed.entries.iter().cloned().collect(),
    })
}

fn dispatch_bootstrap_entitlement_session_tick(
    facade: &StudioAppFacade,
    mode: &StudioBootstrapEntitlementPreflight,
    session: &mut BootstrapSessionResources<'_>,
) -> RfResult<EntitlementSessionEventDriverOutcome> {
    if matches!(mode, StudioBootstrapEntitlementPreflight::Skip) {
        let now = normalized_system_time_now()?;
        return Ok(EntitlementSessionEventDriverOutcome {
            event: EntitlementSessionEvent::SessionStarted,
            outcome: crate::EntitlementSessionEventOutcome::Tick(Box::new(
                crate::EntitlementSessionTickOutcome {
                    preflight: None,
                    schedule: crate::snapshot_entitlement_session_schedule(
                        session.app_state,
                        now,
                        session.policy,
                        session.session_state,
                    ),
                },
            )),
            state: snapshot_entitlement_session_driver_state(
                session.app_state,
                now,
                session.policy,
                session.session_state,
            ),
        });
    }

    let mut context =
        StudioAppMutableAuthCacheContext::new(session.cache_root, session.auth_cache_index);
    let now = normalized_system_time_now()?;
    let mut runtime = EntitlementSessionRuntime {
        facade,
        app_state: session.app_state,
        context: &mut context,
        control_plane_client: session.control_plane_client,
        access_token: "bootstrap-access-token",
        now,
        policy: session.policy,
        session_state: session.session_state,
    };
    dispatch_entitlement_session_event_with_control_plane(
        EntitlementSessionEvent::SessionStarted,
        &mut runtime,
    )
}

fn dispatch_bootstrap_trigger(
    facade: &StudioAppFacade,
    trigger: &StudioBootstrapTrigger,
    session: &mut BootstrapSessionResources<'_>,
) -> RfResult<StudioBootstrapDispatch> {
    match trigger {
        StudioBootstrapTrigger::Intent(intent) => {
            let context =
                StudioAppAuthCacheContext::new(session.cache_root, &*session.auth_cache_index);
            command_outcome_from_workspace_control(dispatch_run_panel_intent_with_auth_cache(
                facade,
                session.app_state,
                &context,
                intent,
            )?)
        }
        StudioBootstrapTrigger::WidgetPrimaryAction => {
            let context =
                StudioAppAuthCacheContext::new(session.cache_root, &*session.auth_cache_index);
            match dispatch_run_panel_primary_action_with_auth_cache(
                facade,
                session.app_state,
                &context,
            )? {
                RunPanelDriverOutcome {
                    dispatch: RunPanelWidgetDispatchOutcome::Executed(outcome),
                    ..
                } => command_outcome_from_workspace_control(*outcome),
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
            let context =
                StudioAppAuthCacheContext::new(session.cache_root, &*session.auth_cache_index);
            match dispatch_run_panel_widget_action_with_auth_cache(
                facade,
                session.app_state,
                &context,
                *action_id,
            )? {
                RunPanelDriverOutcome {
                    dispatch: RunPanelWidgetDispatchOutcome::Executed(outcome),
                    ..
                } => command_outcome_from_workspace_control(*outcome),
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
            dispatch_bootstrap_entitlement_host_trigger(
                facade,
                session,
                EntitlementSessionHostTrigger::PanelPrimaryAction,
            )
        }
        StudioBootstrapTrigger::EntitlementWidgetAction(action_id) => {
            dispatch_bootstrap_entitlement_host_trigger(
                facade,
                session,
                EntitlementSessionHostTrigger::PanelAction(*action_id),
            )
        }
        StudioBootstrapTrigger::EntitlementSessionEvent(event) => {
            let trigger = match event {
                StudioBootstrapEntitlementSessionEvent::LoginCompleted => {
                    EntitlementSessionHostTrigger::LifecycleEvent(
                        EntitlementSessionLifecycleEvent::LoginCompleted,
                    )
                }
                StudioBootstrapEntitlementSessionEvent::TimerElapsed => {
                    EntitlementSessionHostTrigger::LifecycleEvent(
                        EntitlementSessionLifecycleEvent::TimerElapsed,
                    )
                }
                StudioBootstrapEntitlementSessionEvent::NetworkRestored => {
                    EntitlementSessionHostTrigger::LifecycleEvent(
                        EntitlementSessionLifecycleEvent::NetworkRestored,
                    )
                }
                StudioBootstrapEntitlementSessionEvent::WindowForegrounded => {
                    EntitlementSessionHostTrigger::LifecycleEvent(
                        EntitlementSessionLifecycleEvent::WindowForegrounded,
                    )
                }
            };
            dispatch_bootstrap_entitlement_host_trigger(facade, session, trigger)
        }
    }
}

fn dispatch_bootstrap_entitlement_host_trigger(
    facade: &StudioAppFacade,
    session: &mut BootstrapSessionResources<'_>,
    trigger: EntitlementSessionHostTrigger,
) -> RfResult<StudioBootstrapDispatch> {
    let mut context =
        StudioAppMutableAuthCacheContext::new(session.cache_root, session.auth_cache_index);
    let mut runtime = EntitlementSessionRuntime {
        facade,
        app_state: session.app_state,
        context: &mut context,
        control_plane_client: session.control_plane_client,
        access_token: "bootstrap-access-token",
        now: normalized_system_time_now()?,
        policy: session.policy,
        session_state: session.session_state,
    };
    let outcome =
        dispatch_entitlement_session_host_trigger_with_control_plane(trigger, None, &mut runtime)?;
    match outcome.dispatch {
        EntitlementSessionHostDispatch::Event(outcome) => {
            Ok(StudioBootstrapDispatch::EntitlementSessionEvent(outcome))
        }
        EntitlementSessionHostDispatch::Panel(EntitlementSessionPanelDriverOutcome {
            dispatch: crate::EntitlementPanelWidgetDispatchOutcome::Executed(outcome),
            ..
        }) => Ok(StudioBootstrapDispatch::AppCommand(outcome)),
        EntitlementSessionHostDispatch::Panel(EntitlementSessionPanelDriverOutcome {
            dispatch: crate::EntitlementPanelWidgetDispatchOutcome::IgnoredDisabled { action_id },
            ..
        }) => Err(RfError::invalid_input(format!(
            "bootstrap entitlement action `{:?}` is currently disabled",
            action_id
        ))),
        EntitlementSessionHostDispatch::Panel(EntitlementSessionPanelDriverOutcome {
            dispatch: crate::EntitlementPanelWidgetDispatchOutcome::IgnoredMissing { action_id },
            ..
        }) => Err(RfError::invalid_input(format!(
            "bootstrap entitlement action `{:?}` is missing from current widget model",
            action_id
        ))),
    }
}

fn command_outcome_from_workspace_control(
    outcome: crate::WorkspaceControlActionOutcome,
) -> RfResult<StudioBootstrapDispatch> {
    Ok(StudioBootstrapDispatch::AppCommand(
        StudioAppCommandOutcome {
            boundary: outcome.boundary,
            dispatch: outcome.dispatch,
        },
    ))
}

#[derive(Debug, Clone)]
struct BootstrapSeedState {
    auth_cache_index: StoredAuthCacheIndex,
    snapshot: EntitlementSnapshot,
    manifest: PropertyPackageManifest,
    synced_at: SystemTime,
    seed_mode: StudioBootstrapEntitlementSeed,
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
    seed_mode: StudioBootstrapEntitlementSeed,
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
    let snapshot = bootstrap_snapshot(package_id, downloaded_at, seed_mode);
    if !matches!(seed_mode, StudioBootstrapEntitlementSeed::MissingSnapshot) {
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
    }
    auth_cache_index.property_packages.push(record);
    auth_cache_index.last_synced_at =
        if matches!(seed_mode, StudioBootstrapEntitlementSeed::MissingSnapshot) {
            None
        } else {
            Some(downloaded_at)
        };
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
            payload.component_ids(),
        ),
        synced_at: downloaded_at,
        seed_mode,
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
    if !matches!(
        seed.seed_mode,
        StudioBootstrapEntitlementSeed::MissingSnapshot
    ) {
        app_state.update_entitlement(
            seed.snapshot.clone(),
            vec![seed.manifest.clone()],
            seed.synced_at,
        );
    }
}

fn bootstrap_snapshot(
    package_id: &str,
    synced_at: SystemTime,
    seed_mode: StudioBootstrapEntitlementSeed,
) -> EntitlementSnapshot {
    let offline_lease_expires_at = match seed_mode {
        StudioBootstrapEntitlementSeed::Synced => Some(synced_at + Duration::from_secs(7_200)),
        StudioBootstrapEntitlementSeed::MissingSnapshot => {
            Some(synced_at + Duration::from_secs(7_200))
        }
        StudioBootstrapEntitlementSeed::LeaseExpiringSoon => {
            Some(synced_at + Duration::from_secs(10))
        }
    };

    EntitlementSnapshot {
        schema_version: 1,
        subject_id: "bootstrap-user".to_string(),
        tenant_id: Some("bootstrap-tenant".to_string()),
        issued_at: synced_at - Duration::from_secs(60),
        expires_at: synced_at + Duration::from_secs(3_600),
        offline_lease_expires_at,
        features: std::collections::BTreeSet::from(["desktop-login".to_string()]),
        allowed_package_ids: std::collections::BTreeSet::from([package_id.to_string()]),
    }
}

fn bootstrap_manifest(
    package_id: &str,
    hash: &str,
    size_bytes: u64,
    downloaded_at: SystemTime,
    component_ids: Vec<rf_types::ComponentId>,
) -> PropertyPackageManifest {
    let mut manifest = PropertyPackageManifest::new(
        package_id,
        "2026.04.2",
        PropertyPackageSource::RemoteDerivedPackage,
    );
    manifest.hash = hash.to_string();
    manifest.size_bytes = size_bytes;
    manifest.component_ids = component_ids;
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

    use super::{
        StudioBootstrapConfig, StudioBootstrapDispatch, StudioBootstrapEntitlementSeed,
        StudioBootstrapEntitlementSessionEvent, StudioBootstrapTrigger, run_studio_bootstrap,
    };
    use crate::{
        EntitlementPreflightAction, EntitlementSessionEvent, EntitlementSessionEventOutcome,
        StudioAppExecutionBoundary, StudioAppExecutionLane, StudioAppResultDispatch,
        StudioEntitlementAction, StudioEntitlementOutcome, StudioWorkspaceRunOutcome,
    };

    #[test]
    fn bootstrap_runs_sample_workspace_from_main_entry_boundary() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig::default())
            .expect("expected bootstrap run");

        assert_eq!(
            app_command(&report).boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceSolve)
        );
        let dispatch = match &app_command(&report).dispatch {
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
        assert_eq!(report.entitlement_preflight, None);
        assert_eq!(
            report
                .entitlement_host
                .state
                .next_timer
                .as_ref()
                .map(|timer| timer.event),
            Some(crate::EntitlementSessionLifecycleEvent::TimerElapsed)
        );
        assert!(matches!(
            report.entitlement_host.timer_command,
            Some(crate::EntitlementSessionTimerCommand::Schedule { .. })
        ));
        assert_eq!(
            report
                .entitlement_host
                .state
                .host_notice
                .as_ref()
                .map(|notice| notice.title.as_str()),
            Some("Automatic check scheduled")
        );
        assert_eq!(
            report
                .entitlement_host
                .panel
                .widget
                .view()
                .notice
                .as_ref()
                .map(|notice| notice.title.as_str()),
            Some("Automatic check scheduled")
        );
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

        let dispatch = match &app_command(&report).dispatch {
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
        assert_eq!(report.entitlement_preflight, None);
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

        let dispatch = match &app_command(&report).dispatch {
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
        assert_eq!(report.entitlement_preflight, None);
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

        match &app_command(&report).dispatch {
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
        assert_eq!(report.entitlement_preflight, None);
    }

    #[test]
    fn bootstrap_can_dispatch_run_via_widget_action() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            trigger: StudioBootstrapTrigger::WidgetAction(RunPanelActionId::RunManual),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected widget action bootstrap run");

        let dispatch = match &app_command(&report).dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(dispatch.run_status, RunStatus::Converged);
        assert_eq!(report.run_panel.view().primary_action.label, "Run");
        assert_eq!(report.entitlement_preflight, None);
    }

    #[test]
    fn bootstrap_default_trigger_runs_via_primary_widget_action() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig::default())
            .expect("expected primary widget bootstrap run");

        let dispatch = match &app_command(&report).dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
            StudioAppResultDispatch::Entitlement(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(dispatch.run_status, RunStatus::Converged);
        assert_eq!(report.entitlement_preflight, None);
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
            app_command(&report).boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::EntitlementControl)
        );
        match &app_command(&report).dispatch {
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
            report
                .entitlement_host
                .panel
                .widget
                .view()
                .primary_action
                .label,
            "Refresh offline lease"
        );
        assert_eq!(report.log_entries.len(), 1);
        assert_eq!(
            report.log_entries[0].message,
            "Synced entitlement snapshot and property package manifests from control plane"
        );
        assert_eq!(report.entitlement_preflight, None);
    }

    #[test]
    fn bootstrap_can_refresh_offline_lease_via_control_plane_trigger() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            trigger: StudioBootstrapTrigger::EntitlementWidgetPrimaryAction,
            ..StudioBootstrapConfig::default()
        })
        .expect("expected offline refresh bootstrap run");

        assert_eq!(
            app_command(&report).boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::EntitlementControl)
        );
        match &app_command(&report).dispatch {
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
            report
                .entitlement_host
                .panel
                .widget
                .view()
                .primary_action
                .label,
            "Refresh offline lease"
        );
        assert_eq!(report.log_entries.len(), 1);
        assert_eq!(
            report.log_entries[0].message,
            "Refreshed offline lease state from control plane"
        );
        assert_eq!(report.entitlement_preflight, None);
    }

    #[test]
    fn bootstrap_auto_preflight_syncs_when_snapshot_is_missing() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            entitlement_seed: StudioBootstrapEntitlementSeed::MissingSnapshot,
            ..StudioBootstrapConfig::default()
        })
        .expect("expected bootstrap run with entitlement preflight sync");

        let preflight = report
            .entitlement_preflight
            .as_ref()
            .expect("expected preflight outcome");
        assert_eq!(
            preflight.decision.action,
            EntitlementPreflightAction::SyncEntitlement
        );
        match &preflight.outcome.dispatch {
            StudioAppResultDispatch::Entitlement(dispatch) => {
                assert_eq!(dispatch.action, StudioEntitlementAction::SyncEntitlement);
                assert_eq!(dispatch.outcome, StudioEntitlementOutcome::Synced);
            }
            other => panic!("expected entitlement preflight dispatch, got {other:?}"),
        }
        match &app_command(&report).dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert_eq!(dispatch.run_status, RunStatus::Converged);
            }
            other => panic!("expected workspace run after preflight, got {other:?}"),
        }
    }

    #[test]
    fn bootstrap_auto_preflight_refreshes_when_lease_is_expiring() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
            ..StudioBootstrapConfig::default()
        })
        .expect("expected bootstrap run with entitlement preflight refresh");

        let preflight = report
            .entitlement_preflight
            .as_ref()
            .expect("expected preflight outcome");
        assert_eq!(
            preflight.decision.action,
            EntitlementPreflightAction::RefreshOfflineLease
        );
        match &preflight.outcome.dispatch {
            StudioAppResultDispatch::Entitlement(dispatch) => {
                assert_eq!(
                    dispatch.action,
                    StudioEntitlementAction::RefreshOfflineLease
                );
                assert_eq!(
                    dispatch.outcome,
                    StudioEntitlementOutcome::OfflineLeaseRefreshed
                );
            }
            other => panic!("expected entitlement preflight dispatch, got {other:?}"),
        }
        match &app_command(&report).dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert_eq!(dispatch.run_status, RunStatus::Converged);
            }
            other => panic!("expected workspace run after preflight, got {other:?}"),
        }
    }

    #[test]
    fn bootstrap_can_dispatch_login_completed_session_event() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
            entitlement_seed: StudioBootstrapEntitlementSeed::MissingSnapshot,
            trigger: StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::LoginCompleted,
            ),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected login completed session event");

        let outcome = session_event(&report);
        assert_eq!(outcome.event, EntitlementSessionEvent::LoginCompleted);
        match &outcome.outcome {
            EntitlementSessionEventOutcome::Tick(tick) => {
                let preflight = tick.preflight.as_ref().expect("expected sync preflight");
                assert_eq!(
                    preflight.decision.action,
                    EntitlementPreflightAction::SyncEntitlement
                );
            }
            other => panic!("expected tick outcome, got {other:?}"),
        }
        assert_eq!(report.entitlement_preflight, None);
        assert_eq!(
            report
                .entitlement_host
                .panel
                .widget
                .view()
                .primary_action
                .label,
            "Refresh offline lease"
        );
    }

    #[test]
    fn bootstrap_can_dispatch_timer_elapsed_session_event() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
            entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
            trigger: StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::TimerElapsed,
            ),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected timer elapsed session event");

        let outcome = session_event(&report);
        assert_eq!(outcome.event, EntitlementSessionEvent::TimerElapsed);
        match &outcome.outcome {
            EntitlementSessionEventOutcome::Tick(tick) => {
                let preflight = tick
                    .preflight
                    .as_ref()
                    .expect("expected offline refresh preflight");
                assert_eq!(
                    preflight.decision.action,
                    EntitlementPreflightAction::RefreshOfflineLease
                );
            }
            other => panic!("expected tick outcome, got {other:?}"),
        }
        assert_eq!(report.entitlement_preflight, None);
        assert_eq!(report.control_state.run_status, RunStatus::Idle);
        assert_eq!(
            report
                .entitlement_host
                .state
                .next_timer
                .as_ref()
                .map(|timer| timer.event),
            Some(crate::EntitlementSessionLifecycleEvent::TimerElapsed)
        );
        assert!(matches!(
            report.entitlement_host.timer_command,
            Some(crate::EntitlementSessionTimerCommand::Schedule { .. })
        ));
        assert_eq!(
            report
                .entitlement_host
                .state
                .host_notice
                .as_ref()
                .map(|notice| notice.title.as_str()),
            Some("Automatic check scheduled")
        );
    }

    #[test]
    fn bootstrap_can_dispatch_network_restored_session_event() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
            entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
            trigger: StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::NetworkRestored,
            ),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected network restored session event");

        let outcome = session_event(&report);
        assert_eq!(outcome.event, EntitlementSessionEvent::TimerElapsed);
        match &outcome.outcome {
            EntitlementSessionEventOutcome::Tick(tick) => {
                let preflight = tick
                    .preflight
                    .as_ref()
                    .expect("expected offline refresh preflight");
                assert_eq!(
                    preflight.decision.action,
                    EntitlementPreflightAction::RefreshOfflineLease
                );
            }
            other => panic!("expected tick outcome, got {other:?}"),
        }
    }

    #[test]
    fn bootstrap_can_dispatch_window_foregrounded_session_event() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            entitlement_preflight: super::StudioBootstrapEntitlementPreflight::Skip,
            entitlement_seed: StudioBootstrapEntitlementSeed::LeaseExpiringSoon,
            trigger: StudioBootstrapTrigger::EntitlementSessionEvent(
                StudioBootstrapEntitlementSessionEvent::WindowForegrounded,
            ),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected window foregrounded session event");

        let outcome = session_event(&report);
        assert_eq!(outcome.event, EntitlementSessionEvent::TimerElapsed);
        match &outcome.outcome {
            EntitlementSessionEventOutcome::Tick(tick) => {
                let preflight = tick
                    .preflight
                    .as_ref()
                    .expect("expected offline refresh preflight");
                assert_eq!(
                    preflight.decision.action,
                    EntitlementPreflightAction::RefreshOfflineLease
                );
            }
            other => panic!("expected tick outcome, got {other:?}"),
        }
    }

    fn app_command(report: &super::StudioBootstrapReport) -> &crate::StudioAppCommandOutcome {
        match &report.dispatch {
            StudioBootstrapDispatch::AppCommand(outcome) => outcome,
            StudioBootstrapDispatch::EntitlementSessionEvent(_) => {
                panic!("expected app command dispatch")
            }
        }
    }

    fn session_event(
        report: &super::StudioBootstrapReport,
    ) -> &crate::EntitlementSessionEventDriverOutcome {
        match &report.dispatch {
            StudioBootstrapDispatch::EntitlementSessionEvent(outcome) => outcome,
            StudioBootstrapDispatch::AppCommand(_) => {
                panic!("expected entitlement session event dispatch")
            }
        }
    }
}
