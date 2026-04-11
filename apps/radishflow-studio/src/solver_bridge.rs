use std::path::Path;

use rf_flash::PlaceholderTpFlashSolver;
use rf_solver::{FlowsheetSolver, SequentialModularSolver, SolveFailureContext, SolverServices};
use rf_store::StoredAuthCacheIndex;
use rf_thermo::{
    CachedPropertyPackageProvider, PlaceholderThermoProvider, PropertyPackageProvider,
};
use rf_types::{RfError, RfResult};
use rf_ui::{AppLogLevel, AppState, DiagnosticSeverity, DiagnosticSummary, RunStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioSolveRequest {
    pub package_id: String,
    pub snapshot_id: String,
    pub sequence: u64,
}

impl StudioSolveRequest {
    pub fn new(
        package_id: impl Into<String>,
        snapshot_id: impl Into<String>,
        sequence: u64,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            snapshot_id: snapshot_id.into(),
            sequence,
        }
    }

    pub fn validate(&self) -> RfResult<()> {
        if self.package_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "studio solve request must contain a non-empty package_id",
            ));
        }

        if self.snapshot_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "studio solve request must contain a non-empty snapshot_id",
            ));
        }

        if self.sequence == 0 {
            return Err(RfError::invalid_input(
                "studio solve request sequence must be greater than zero",
            ));
        }

        Ok(())
    }
}

pub fn next_solver_snapshot_sequence(app_state: &AppState) -> u64 {
    app_state
        .workspace
        .snapshot_history
        .back()
        .map(|snapshot| snapshot.sequence + 1)
        .unwrap_or(1)
}

pub fn solve_workspace_with_property_package<P>(
    app_state: &mut AppState,
    package_provider: &P,
    request: &StudioSolveRequest,
) -> RfResult<()>
where
    P: PropertyPackageProvider,
{
    request.validate()?;

    let revision = app_state.workspace.document.revision;
    let package_id = request.package_id.as_str();
    app_state.workspace.solve_session.begin_checking(revision);
    app_state.workspace.solve_session.mark_runnable();
    app_state.workspace.solve_session.begin_solving();

    let thermo_system = match package_provider.load_system(package_id) {
        Ok(system) => system,
        Err(error) => {
            record_solve_failure(
                app_state,
                revision,
                format!(
                    "failed to load property package `{package_id}`: {}",
                    error.message()
                ),
            );
            return Err(error);
        }
    };

    let thermo_provider = PlaceholderThermoProvider::new(thermo_system);
    let flash_solver = PlaceholderTpFlashSolver;
    let services = SolverServices {
        thermo: &thermo_provider,
        flash_solver: &flash_solver,
    };
    let solver_snapshot =
        match SequentialModularSolver.solve(&services, &app_state.workspace.document.flowsheet) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                record_solve_failure(
                    app_state,
                    revision,
                    format!(
                        "flowsheet solve failed with package `{package_id}`: {}",
                        error.message()
                    ),
                );
                return Err(error);
            }
        };

    app_state.store_solver_snapshot(
        request.snapshot_id.as_str(),
        request.sequence,
        &solver_snapshot,
    );
    app_state.push_log(
        AppLogLevel::Info,
        format!(
            "Solved document revision {} with property package `{}` into snapshot `{}`",
            revision, package_id, request.snapshot_id
        ),
    );
    Ok(())
}

pub fn solve_workspace_from_auth_cache(
    app_state: &mut AppState,
    cache_root: impl AsRef<Path>,
    auth_cache_index: &StoredAuthCacheIndex,
    request: &StudioSolveRequest,
) -> RfResult<()> {
    let provider = CachedPropertyPackageProvider::new(cache_root, auth_cache_index)?;
    solve_workspace_with_property_package(app_state, &provider, request)
}

fn record_solve_failure(app_state: &mut AppState, revision: u64, message: String) {
    let context = SolveFailureContext::from_message(&message);
    let mut summary = DiagnosticSummary::new(revision, DiagnosticSeverity::Error, message.clone());
    if let Some(primary_code) = context.primary_code {
        summary = summary.with_primary_code(primary_code);
    }
    if !context.related_unit_ids.is_empty() {
        summary = summary.with_related_unit_ids(context.related_unit_ids);
    }
    app_state.record_failure(revision, RunStatus::Error, summary);
    app_state.push_log(AppLogLevel::Error, message);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_store::{
        StoredAntoineCoefficients, StoredAuthCacheIndex, StoredCredentialReference,
        StoredPropertyPackageManifest, StoredPropertyPackagePayload, StoredPropertyPackageRecord,
        StoredPropertyPackageSource, StoredThermoComponent, parse_project_file_json,
        property_package_payload_integrity, write_property_package_manifest,
        write_property_package_payload,
    };
    use rf_thermo::{
        AntoineCoefficients, InMemoryPropertyPackageProvider, PropertyPackageManifest,
        PropertyPackageSource, ThermoComponent, ThermoSystem,
    };
    use rf_types::ComponentId;
    use rf_ui::{AppState, DocumentMetadata, FlowsheetDocument, RunStatus};

    use super::{
        SolveFailureContext, StudioSolveRequest, next_solver_snapshot_sequence,
        solve_workspace_from_auth_cache, solve_workspace_with_property_package,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new("doc-1", "Demo", timestamp(10));
        FlowsheetDocument::new(flowsheet, metadata)
    }

    fn sample_provider() -> InMemoryPropertyPackageProvider {
        let mut first = ThermoComponent::new("component-a", "Component A");
        first.antoine = Some(AntoineCoefficients::new(
            ((2.0_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        let mut second = ThermoComponent::new("component-b", "Component B");
        second.antoine = Some(AntoineCoefficients::new(
            ((0.5_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        InMemoryPropertyPackageProvider::new(vec![(
            PropertyPackageManifest::new(
                "binary-hydrocarbon-lite-v1",
                "2026.03.1",
                PropertyPackageSource::LocalBundled,
                vec!["component-a".into(), "component-b".into()],
            ),
            ThermoSystem::binary([first, second]),
        )])
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
    }

    #[test]
    fn next_snapshot_sequence_advances_from_snapshot_history() {
        let mut app_state = AppState::new(sample_document());
        let provider = sample_provider();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
        ))
        .expect("expected project parse");
        app_state.workspace.document = FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-2", "Heater Demo", timestamp(20)),
        );

        assert_eq!(next_solver_snapshot_sequence(&app_state), 1);
        solve_workspace_with_property_package(
            &mut app_state,
            &provider,
            &StudioSolveRequest::new("binary-hydrocarbon-lite-v1", "snapshot-1", 1),
        )
        .expect("expected solve");

        assert_eq!(next_solver_snapshot_sequence(&app_state), 2);
    }

    #[test]
    fn solve_workspace_updates_app_state_with_solver_snapshot() {
        let provider = sample_provider();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-valve-flash.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-3", "Valve Demo", timestamp(30)),
        ));

        solve_workspace_with_property_package(
            &mut app_state,
            &provider,
            &StudioSolveRequest::new("binary-hydrocarbon-lite-v1", "snapshot-1", 1),
        )
        .expect("expected solve");

        assert_eq!(
            app_state.workspace.solve_session.status,
            RunStatus::Converged
        );
        assert_eq!(app_state.workspace.snapshot_history.len(), 1);
        assert_eq!(
            app_state.workspace.snapshot_history[0]
                .summary
                .primary_message,
            "solved flowsheet with 3 unit(s), 4 diagnostic entry(ies), and 4 resulting stream(s)"
        );
        assert_eq!(app_state.log_feed.entries.len(), 1);
    }

    #[test]
    fn solve_workspace_records_failure_when_package_is_missing() {
        let provider = InMemoryPropertyPackageProvider::default();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-4", "Heater Demo", timestamp(40)),
        ));

        let error = solve_workspace_with_property_package(
            &mut app_state,
            &provider,
            &StudioSolveRequest::new("missing-package", "snapshot-1", 1),
        )
        .expect_err("expected missing package error");

        assert!(error.message().contains("missing property package"));
        assert_eq!(app_state.workspace.solve_session.status, RunStatus::Error);
        assert_eq!(
            app_state
                .workspace
                .solve_session
                .latest_diagnostic
                .as_ref()
                .and_then(|summary| summary.primary_code.as_deref()),
            None
        );
        assert_eq!(app_state.log_feed.entries.len(), 1);
    }

    #[test]
    fn solve_workspace_records_solver_failure_primary_code_in_summary() {
        let provider = sample_provider();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-valve-flash.rfproj.json"
        ))
        .expect("expected project parse");
        let mut flowsheet = project.document.flowsheet;
        flowsheet
            .streams
            .get_mut(&"stream-throttled".into())
            .expect("expected throttled stream")
            .pressure_pa = 130_000.0;
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc-6", "Valve Failure Demo", timestamp(80)),
        ));

        let error = solve_workspace_with_property_package(
            &mut app_state,
            &provider,
            &StudioSolveRequest::new("binary-hydrocarbon-lite-v1", "snapshot-failure-1", 1),
        )
        .expect_err("expected solve failure");

        assert!(error.message().contains("solver.step.execution:"));
        assert_eq!(app_state.workspace.solve_session.status, RunStatus::Error);
        assert_eq!(
            app_state
                .workspace
                .solve_session
                .latest_diagnostic
                .as_ref()
                .and_then(|summary| summary.primary_code.as_deref()),
            Some("solver.step.execution")
        );
        assert_eq!(
            app_state
                .workspace
                .solve_session
                .latest_diagnostic
                .as_ref()
                .map(|summary| summary.related_unit_ids.as_slice()),
            Some([rf_types::UnitId::new("valve-1")].as_slice())
        );
    }

    #[test]
    fn solve_workspace_from_auth_cache_loads_cached_property_package() {
        let cache_root = unique_temp_path("solver-cache");
        let mut auth_cache_index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        let mut first = StoredThermoComponent::new(ComponentId::new("component-a"), "Component A");
        first.antoine = Some(StoredAntoineCoefficients::new(
            ((2.0_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        let mut second = StoredThermoComponent::new(ComponentId::new("component-b"), "Component B");
        second.antoine = Some(StoredAntoineCoefficients::new(
            ((0.5_f64 * 100_000.0_f64) / 1_000.0_f64).ln(),
            0.0,
            0.0,
        ));

        let payload = StoredPropertyPackagePayload::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            vec![first, second],
        );
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let expires_at = Some(SystemTime::now() + Duration::from_secs(3_600));
        let mut manifest = StoredPropertyPackageManifest::new(
            "binary-hydrocarbon-lite-v1",
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            vec![
                ComponentId::new("component-a"),
                ComponentId::new("component-b"),
            ],
        );
        manifest.hash = integrity.hash.clone();
        manifest.size_bytes = integrity.size_bytes;
        manifest.expires_at = expires_at;
        let mut record = StoredPropertyPackageRecord::new(
            &manifest.package_id,
            &manifest.version,
            StoredPropertyPackageSource::RemoteDerivedPackage,
            manifest.hash.clone(),
            manifest.size_bytes,
            timestamp(60),
        );
        record.expires_at = expires_at;

        let manifest_target = record.manifest_path_under(&cache_root);
        let payload_target = record
            .payload_path_under(&cache_root)
            .expect("expected payload path");
        write_property_package_manifest(&manifest_target, &manifest)
            .expect("expected manifest write");
        write_property_package_payload(&payload_target, &payload).expect("expected payload write");
        auth_cache_index.property_packages.push(record);

        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-5", "Cached Provider Demo", timestamp(70)),
        ));

        solve_workspace_from_auth_cache(
            &mut app_state,
            &cache_root,
            &auth_cache_index,
            &StudioSolveRequest::new("binary-hydrocarbon-lite-v1", "snapshot-cache-1", 1),
        )
        .expect("expected solve from auth cache");

        assert_eq!(
            app_state.workspace.solve_session.status,
            RunStatus::Converged
        );
        assert_eq!(app_state.workspace.snapshot_history.len(), 1);
        assert_eq!(
            app_state.workspace.snapshot_history[0]
                .summary
                .primary_message,
            "solved flowsheet with 3 unit(s), 4 diagnostic entry(ies), and 4 resulting stream(s)"
        );

        fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }

    #[test]
    fn solver_failure_context_extracts_lookup_unit_from_wrapped_message() {
        let context = SolveFailureContext::from_message(
            "flowsheet solve failed with package `binary-hydrocarbon-lite-v1`: solver.step.lookup: solver step 3 unit lookup failed for `flash-1`: missing unit `flash-1`",
        );

        assert_eq!(context.primary_code.as_deref(), Some("solver.step.lookup"));
        assert_eq!(context.related_unit_ids, vec![rf_types::UnitId::new("flash-1")]);
    }
}
