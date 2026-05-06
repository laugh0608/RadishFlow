use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    RadishFlowControlPlaneClient, RadishFlowControlPlaneClientError,
    RadishFlowControlPlaneClientErrorKind, RadishFlowControlPlaneResponse,
};
use rf_model::{Component, Flowsheet};
use rf_store::{
    StoredAntoineCoefficients, StoredAuthCacheIndex, StoredCredentialReference,
    StoredEntitlementCache, StoredProjectFile, StoredPropertyPackageManifest,
    StoredPropertyPackagePayload, StoredPropertyPackageRecord, StoredPropertyPackageSource,
    StoredThermoComponent, property_package_payload_integrity, write_auth_cache_index,
    write_property_package_manifest, write_property_package_payload,
};
use rf_types::{ComponentId, RfError, RfResult};
use rf_ui::{
    AppLogLevel, AppState, AuthenticatedUser, DocumentMetadata, EntitlementSnapshot,
    FlowsheetDocument, OfflineLeaseRefreshRequest, OfflineLeaseRefreshResponse,
    PropertyPackageLeaseGrant, PropertyPackageLeaseRequest, PropertyPackageManifest,
    PropertyPackageManifestList, PropertyPackageSource, SecureCredentialHandle, TokenLease,
};

use super::StudioBootstrapEntitlementSeed;

pub(super) const BOOTSTRAP_MVP_PROPERTY_PACKAGE_ID: &str = "binary-hydrocarbon-lite-v1";
const BOOTSTRAP_MVP_COMPONENT_A_ID: &str = "component-a";
const BOOTSTRAP_MVP_COMPONENT_B_ID: &str = "component-b";

#[derive(Debug, Clone)]
pub(super) struct BootstrapSeedState {
    pub(super) auth_cache_index: StoredAuthCacheIndex,
    snapshot: EntitlementSnapshot,
    manifest: PropertyPackageManifest,
    synced_at: SystemTime,
    seed_mode: StudioBootstrapEntitlementSeed,
}

#[derive(Debug, Clone)]
pub(super) struct BootstrapControlPlaneClient {
    synced_snapshot: EntitlementSnapshot,
    manifest_list: PropertyPackageManifestList,
    sync_received_at: SystemTime,
    refresh_response: OfflineLeaseRefreshResponse,
    refresh_received_at: SystemTime,
}

impl BootstrapControlPlaneClient {
    pub(super) fn from_seed(seed: &BootstrapSeedState) -> Self {
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

pub(super) fn app_state_from_project_file(
    project_file: &StoredProjectFile,
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

pub(super) fn initialize_blank_project_thermo_basis(
    app_state: &mut AppState,
    changed_at: SystemTime,
) -> RfResult<Option<u64>> {
    if !app_state.workspace.document.flowsheet.components.is_empty() {
        return Ok(None);
    }

    let mut flowsheet = app_state.workspace.document.flowsheet.clone();
    flowsheet.insert_component(Component::new(BOOTSTRAP_MVP_COMPONENT_A_ID, "Component A"))?;
    flowsheet.insert_component(Component::new(BOOTSTRAP_MVP_COMPONENT_B_ID, "Component B"))?;

    let revision = app_state
        .workspace
        .document
        .replace_flowsheet(flowsheet, changed_at);
    app_state.workspace.canvas_interaction.invalidate_all();
    app_state
        .workspace
        .solve_session
        .mark_document_revision_advanced(revision);
    app_state.workspace.drafts.clear();
    app_state.push_log(
        AppLogLevel::Info,
        format!(
            "initialized blank project with MVP components `{}` / `{}` and property package `{}`",
            BOOTSTRAP_MVP_COMPONENT_A_ID,
            BOOTSTRAP_MVP_COMPONENT_B_ID,
            BOOTSTRAP_MVP_PROPERTY_PACKAGE_ID
        ),
    );
    Ok(Some(revision))
}

pub(super) fn seed_sample_auth_cache(
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

pub(super) fn seed_bootstrap_runtime_state(app_state: &mut AppState, seed: &BootstrapSeedState) {
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
    component_ids: Vec<ComponentId>,
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

pub(super) fn normalized_system_time_now() -> RfResult<SystemTime> {
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
    if components.len() < 2 {
        return Err(RfError::invalid_input(format!(
            "studio bootstrap expects at least 2 flowsheet components, got {}",
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
