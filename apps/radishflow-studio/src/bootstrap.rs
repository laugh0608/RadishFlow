use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    RunPanelWidgetDispatchOutcome, StudioAppAuthCacheContext, StudioAppFacade,
    WorkspaceControlActionOutcome, WorkspaceControlState,
    dispatch_run_panel_intent_with_auth_cache, dispatch_run_panel_widget_event_with_auth_cache,
    snapshot_workspace_control_state,
};
use rf_model::Flowsheet;
use rf_store::{
    StoredAntoineCoefficients, StoredAuthCacheIndex, StoredCredentialReference,
    StoredPropertyPackageManifest, StoredPropertyPackagePayload, StoredPropertyPackageRecord,
    StoredPropertyPackageSource, StoredThermoComponent, property_package_payload_integrity,
    read_project_file, write_auth_cache_index, write_property_package_manifest,
    write_property_package_payload,
};
use rf_types::{RfError, RfResult};
use rf_ui::{
    AppLogEntry, AppState, DocumentMetadata, FlowsheetDocument, RunPanelActionId, RunPanelIntent,
    RunPanelWidgetModel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapConfig {
    pub project_path: PathBuf,
    pub trigger: StudioBootstrapTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioBootstrapTrigger {
    Intent(RunPanelIntent),
    WidgetAction(RunPanelActionId),
}

impl Default for StudioBootstrapConfig {
    fn default() -> Self {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");

        Self {
            project_path: workspace_root
                .join("examples")
                .join("flowsheets")
                .join("feed-heater-flash.rfproj.json"),
            trigger: StudioBootstrapTrigger::WidgetAction(RunPanelActionId::RunManual),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapReport {
    pub outcome: WorkspaceControlActionOutcome,
    pub control_state: WorkspaceControlState,
    pub run_panel: RunPanelWidgetModel,
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
    let outcome = dispatch_bootstrap_trigger(&facade, &mut app_state, &context, &config.trigger)?;

    Ok(StudioBootstrapReport {
        outcome,
        control_state: snapshot_workspace_control_state(&app_state),
        run_panel: RunPanelWidgetModel::from_state(&app_state.workspace.run_panel),
        log_entries: app_state.log_feed.entries.iter().cloned().collect(),
    })
}

fn dispatch_bootstrap_trigger(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    trigger: &StudioBootstrapTrigger,
) -> RfResult<WorkspaceControlActionOutcome> {
    match trigger {
        StudioBootstrapTrigger::Intent(intent) => {
            dispatch_run_panel_intent_with_auth_cache(facade, app_state, context, intent)
        }
        StudioBootstrapTrigger::WidgetAction(action_id) => {
            let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);
            match dispatch_run_panel_widget_event_with_auth_cache(
                facade,
                app_state,
                context,
                &widget.activate(*action_id),
            )? {
                RunPanelWidgetDispatchOutcome::Executed(outcome) => Ok(outcome),
                RunPanelWidgetDispatchOutcome::IgnoredDisabled { action_id } => {
                    Err(RfError::invalid_input(format!(
                        "bootstrap widget action `{:?}` is currently disabled",
                        action_id
                    )))
                }
                RunPanelWidgetDispatchOutcome::IgnoredMissing { action_id } => {
                    Err(RfError::invalid_input(format!(
                        "bootstrap widget action `{:?}` is missing from current widget model",
                        action_id
                    )))
                }
            }
        }
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
) -> RfResult<StoredAuthCacheIndex> {
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
    auth_cache_index.property_packages.push(record);
    auth_cache_index.last_synced_at = Some(downloaded_at);
    write_auth_cache_index(
        auth_cache_index.index_path_under(cache_root),
        &auth_cache_index,
    )?;
    Ok(auth_cache_index)
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
        RunPanelActionId, RunPanelIntent, RunPanelPackageSelection, RunStatus, SimulationMode,
    };

    use super::{StudioBootstrapConfig, StudioBootstrapTrigger, run_studio_bootstrap};
    use crate::{
        StudioAppExecutionBoundary, StudioAppExecutionLane, StudioAppResultDispatch,
        WorkspaceSolveDispatch,
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
        assert_eq!(report.run_panel.view().mode_label, "Hold");
        assert_eq!(report.run_panel.view().status_label, "Converged");
        assert_eq!(report.run_panel.view().primary_action.label, "Run");
        assert_eq!(report.run_panel.view().secondary_actions.len(), 3);
        assert_eq!(dispatch.log_entry_count, 1);
        assert_eq!(report.log_entries.len(), 1);
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
        };
        assert_eq!(
            report.control_state.simulation_mode,
            dispatch.simulation_mode
        );
        assert_eq!(report.control_state.run_status, dispatch.run_status);
        assert_eq!(dispatch.run_status, RunStatus::Converged);
        assert!(matches!(
            dispatch.solve_dispatch,
            WorkspaceSolveDispatch::Started(_)
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
        };
        assert_eq!(dispatch.run_status, RunStatus::Converged);
        assert_eq!(report.run_panel.view().primary_action.label, "Run");
    }
}
