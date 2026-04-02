use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rf_model::Flowsheet;
use rf_store::{
    StoredAntoineCoefficients, StoredAuthCacheIndex, StoredCredentialReference,
    StoredPropertyPackageManifest, StoredPropertyPackagePayload, StoredPropertyPackageRecord,
    StoredPropertyPackageSource, StoredThermoComponent, property_package_payload_integrity,
    read_project_file, write_auth_cache_index, write_property_package_manifest,
    write_property_package_payload,
};
use rf_types::{RfError, RfResult};
use rf_ui::{AppLogEntry, AppState, DocumentMetadata, FlowsheetDocument};

use crate::WorkspaceRunCommand;
use crate::{
    StudioAppAuthCacheContext, StudioAppCommand, StudioAppCommandOutcome, StudioAppFacade,
    WorkspaceControlState, snapshot_workspace_control_state,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapConfig {
    pub project_path: PathBuf,
    pub command: WorkspaceRunCommand,
}

impl Default for StudioBootstrapConfig {
    fn default() -> Self {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");

        Self {
            project_path: workspace_root
                .join("examples")
                .join("flowsheets")
                .join("feed-heater-flash.rfproj.json"),
            command: WorkspaceRunCommand::manual("binary-hydrocarbon-lite-v1"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapReport {
    pub outcome: StudioAppCommandOutcome,
    pub control_state: WorkspaceControlState,
    pub log_entries: Vec<AppLogEntry>,
}

pub fn run_studio_bootstrap(config: &StudioBootstrapConfig) -> RfResult<StudioBootstrapReport> {
    let project_file = read_project_file(&config.project_path)?;
    let mut app_state = app_state_from_project_file(&project_file, &config.project_path);
    let cache_root = TemporaryCacheRoot::new("studio-bootstrap")?;
    let auth_cache_index = seed_sample_auth_cache(
        cache_root.path(),
        &project_file.document.flowsheet,
        "binary-hydrocarbon-lite-v1",
    )?;
    let context = StudioAppAuthCacheContext::new(cache_root.path(), &auth_cache_index);
    let facade = StudioAppFacade::new();
    let command = StudioAppCommand::run_workspace(config.command.clone());
    let outcome = facade.execute_with_auth_cache(&mut app_state, &context, &command)?;

    Ok(StudioBootstrapReport {
        outcome,
        control_state: snapshot_workspace_control_state(&app_state),
        log_entries: app_state.log_feed.entries.iter().cloned().collect(),
    })
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
) -> RfResult<StoredAuthCacheIndex> {
    let downloaded_at = SystemTime::now();
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
    auth_cache_index.property_packages.push(record);
    auth_cache_index.last_synced_at = Some(downloaded_at);
    write_auth_cache_index(
        auth_cache_index.index_path_under(cache_root),
        &auth_cache_index,
    )?;
    Ok(auth_cache_index)
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
    use rf_ui::RunStatus;

    use super::{StudioBootstrapConfig, run_studio_bootstrap};
    use crate::{
        StudioAppExecutionBoundary, StudioAppExecutionLane, StudioAppResultDispatch,
        WorkspaceRunCommand, WorkspaceSolveDispatch, WorkspaceSolveSkipReason,
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
            dispatch.solve_dispatch,
            WorkspaceSolveDispatch::Started(_)
        ));
        assert_eq!(
            dispatch.latest_snapshot_summary.as_deref(),
            Some(
                "solved flowsheet with 3 unit(s), 4 diagnostic entry(ies), and 4 resulting stream(s)"
            )
        );
        assert_eq!(dispatch.log_entry_count, 1);
        assert_eq!(report.log_entries.len(), 1);
    }

    #[test]
    fn bootstrap_reports_skip_for_automatic_trigger() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            command: WorkspaceRunCommand::automatic_preferred(),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected bootstrap skip");

        let dispatch = match report.outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
        };
        assert_eq!(
            report.control_state.simulation_mode,
            dispatch.simulation_mode
        );
        assert_eq!(report.control_state.run_status, dispatch.run_status);
        assert_eq!(
            dispatch.solve_dispatch,
            WorkspaceSolveDispatch::Skipped(WorkspaceSolveSkipReason::HoldMode)
        );
        assert_eq!(dispatch.run_status, RunStatus::Idle);
        assert!(dispatch.latest_snapshot_summary.is_none());
        assert_eq!(dispatch.log_entry_count, 1);
        assert_eq!(report.log_entries.len(), 1);
        assert_eq!(
            report.log_entries[0].message,
            "Skipped workspace run because simulation mode is Hold"
        );
    }

    #[test]
    fn bootstrap_accepts_preferred_package_selection_when_single_cached_package_exists() {
        let report = run_studio_bootstrap(&StudioBootstrapConfig {
            command: WorkspaceRunCommand::new(
                crate::WorkspaceSolveTrigger::Manual,
                crate::WorkspaceRunPackageSelection::Preferred,
            ),
            ..StudioBootstrapConfig::default()
        })
        .expect("expected preferred package bootstrap run");

        let dispatch = match report.outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => dispatch,
            StudioAppResultDispatch::WorkspaceMode(_) => panic!("expected workspace run dispatch"),
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
}
