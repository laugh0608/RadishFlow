pub(crate) mod execution;
#[cfg(test)]
mod execution_tests;
pub use execution::{
    CompletedWorkspaceSolve, PreparedWorkspaceSolve, WorkspaceExecutionOutcome,
    WorkspaceSolveProgress, complete_workspace_solve, prepare_workspace_solve,
};

use std::path::Path;

use rf_property_package::{
    CachedPropertyPackageProvider, LocalThermoContext, PackageFailureKind, PackageLoadError,
    PropertyPackageProvider, prepare_local_context,
};
use rf_solver::{SequentialModularSolver, SolveFailureContext, SolverServices};
use rf_store::StoredAuthCacheIndex;
use rf_types::{RfError, RfResult};
use rf_ui::{AppLogLevel, AppState, DiagnosticSeverity, DiagnosticSummary, RunStatus};
use std::time::SystemTime;

pub(crate) const WORKSPACE_RUN_DIAGNOSTIC_LOCAL_CACHE_UNAVAILABLE: &str =
    "workspace.run.local_cache_unavailable";
pub(crate) const WORKSPACE_RUN_DIAGNOSTIC_PROPERTY_PACKAGE_MISSING: &str =
    "workspace.run.property_package_missing";

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

pub fn next_solver_snapshot_sequence(app_state: &AppState) -> RfResult<u64> {
    app_state
        .workspace
        .snapshot_history
        .back()
        .map_or(Some(1), |snapshot| snapshot.sequence.checked_add(1))
        .ok_or_else(|| {
            RfError::invalid_input("solver snapshot sequence exhausted")
                .with_diagnostic_code("workspace.run.snapshot_sequence_exhausted")
        })
}

pub fn solve_workspace_with_property_package<P>(
    app_state: &mut AppState,
    package_provider: &P,
    request: &StudioSolveRequest,
) -> RfResult<()>
where
    P: PropertyPackageProvider,
{
    let mut completion = prepare_workspace_solve(app_state, request)?
        .execute_with_property_package(package_provider);
    complete_workspace_solve(app_state, &mut completion);
    completion.result().map(|_| ())
}

fn workspace_property_package_load_error(error: PackageLoadError) -> RfError {
    match error.kind {
        PackageFailureKind::Missing => error
            .source
            .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_PROPERTY_PACKAGE_MISSING),
        PackageFailureKind::Expired | PackageFailureKind::UnsupportedSource => {
            RfError::missing_entity(
                "property package",
                error.package_id.as_deref().unwrap_or_default(),
            )
            .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_PROPERTY_PACKAGE_MISSING)
        }
        _ => error
            .source
            .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_LOCAL_CACHE_UNAVAILABLE),
    }
}

pub fn solve_workspace_from_auth_cache(
    app_state: &mut AppState,
    cache_root: impl AsRef<Path>,
    auth_cache_index: &StoredAuthCacheIndex,
    request: &StudioSolveRequest,
) -> RfResult<()> {
    let mut completion = prepare_workspace_solve(app_state, request)?
        .execute_from_auth_cache(cache_root, auth_cache_index);
    complete_workspace_solve(app_state, &mut completion);
    completion.result().map(|_| ())
}

fn record_solve_failure(
    app_state: &mut AppState,
    revision: u64,
    message: String,
    context: SolveFailureContext,
) {
    let mut summary = DiagnosticSummary::new(revision, DiagnosticSeverity::Error, message.clone());
    if let Some(primary_code) = context.primary_code {
        summary = summary.with_primary_code(primary_code);
    }
    if !context.related_unit_ids.is_empty() {
        summary = summary.with_related_unit_ids(context.related_unit_ids);
    }
    if !context.related_stream_ids.is_empty() {
        summary = summary.with_related_stream_ids(context.related_stream_ids);
    }
    if !context.related_port_targets.is_empty() {
        summary = summary.with_related_port_targets(context.related_port_targets);
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
    use rf_property_package::InMemoryPropertyPackageProvider;
    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredPropertyPackageRecord,
        StoredPropertyPackageSource, parse_project_file_json,
    };
    use rf_ui::{AppState, DocumentMetadata, FlowsheetDocument, RunStatus};

    use super::{
        SolveFailureContext, StudioSolveRequest, WORKSPACE_RUN_DIAGNOSTIC_LOCAL_CACHE_UNAVAILABLE,
        WORKSPACE_RUN_DIAGNOSTIC_PROPERTY_PACKAGE_MISSING, next_solver_snapshot_sequence,
        solve_workspace_from_auth_cache, solve_workspace_with_property_package,
    };
    use crate::test_support::{
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
        build_official_binary_hydrocarbon_in_memory_provider,
        write_default_official_binary_hydrocarbon_cached_package,
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
        build_official_binary_hydrocarbon_in_memory_provider(OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID)
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
            "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected project parse");
        app_state.workspace.document = FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-2", "Heater Demo", timestamp(20)),
        );

        assert_eq!(next_solver_snapshot_sequence(&app_state).unwrap(), 1);
        solve_workspace_with_property_package(
            &mut app_state,
            &provider,
            &StudioSolveRequest::new(OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID, "snapshot-1", 1),
        )
        .expect("expected solve");

        assert_eq!(next_solver_snapshot_sequence(&app_state).unwrap(), 2);
    }

    #[test]
    fn solve_workspace_updates_app_state_with_solver_snapshot() {
        let provider = sample_provider();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-3", "Valve Demo", timestamp(30)),
        ));

        solve_workspace_with_property_package(
            &mut app_state,
            &provider,
            &StudioSolveRequest::new(OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID, "snapshot-1", 1),
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
    fn solve_workspace_records_workspace_code_when_package_is_missing() {
        let provider = InMemoryPropertyPackageProvider::default();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
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
            Some(WORKSPACE_RUN_DIAGNOSTIC_PROPERTY_PACKAGE_MISSING)
        );
        assert_eq!(app_state.log_feed.entries.len(), 1);
    }

    #[test]
    fn solve_workspace_records_solver_failure_primary_code_in_summary() {
        let provider = sample_provider();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected project parse");
        let mut flowsheet = project.document.flowsheet;
        flowsheet
            .streams
            .get_mut(&"stream-throttled".into())
            .expect("expected throttled stream")
            .pressure_pa = 730_000.0;
        flowsheet
            .units
            .get_mut(&"valve-1".into())
            .expect("expected valve unit")
            .parameters
            .outlet_pressure_pa = Some(730_000.0);
        let mut app_state = AppState::new(FlowsheetDocument::new(
            flowsheet,
            DocumentMetadata::new("doc-6", "Valve Failure Demo", timestamp(80)),
        ));

        let error = solve_workspace_with_property_package(
            &mut app_state,
            &provider,
            &StudioSolveRequest::new(
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
                "snapshot-failure-1",
                1,
            ),
        )
        .expect_err("expected solve failure");

        assert!(error.message().contains("solver.step.parameter:"));
        assert_eq!(app_state.workspace.solve_session.status, RunStatus::Error);
        assert_eq!(
            app_state
                .workspace
                .solve_session
                .latest_diagnostic
                .as_ref()
                .and_then(|summary| summary.primary_code.as_deref()),
            Some("solver.step.parameter")
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
        write_default_official_binary_hydrocarbon_cached_package(
            &cache_root,
            &mut auth_cache_index,
        );

        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
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
            &StudioSolveRequest::new(
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
                "snapshot-cache-1",
                1,
            ),
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
    fn solve_workspace_from_auth_cache_marks_local_cache_unavailable_with_diagnostic_code() {
        let cache_root = unique_temp_path("solver-cache-missing");
        let mut auth_cache_index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        let mut record = StoredPropertyPackageRecord::new(
            OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            "2026.03.1",
            StoredPropertyPackageSource::RemoteDerivedPackage,
            "sha256:missing".to_string(),
            128,
            timestamp(90),
        );
        record.expires_at = Some(SystemTime::now() + Duration::from_secs(3_600));
        auth_cache_index.property_packages.push(record);

        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-7", "Missing Cache Demo", timestamp(95)),
        ));

        let error = solve_workspace_from_auth_cache(
            &mut app_state,
            &cache_root,
            &auth_cache_index,
            &StudioSolveRequest::new(
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
                "snapshot-cache-missing",
                1,
            ),
        )
        .expect_err("expected cache preparation failure");

        assert_eq!(
            error.context().diagnostic_code(),
            Some(WORKSPACE_RUN_DIAGNOSTIC_LOCAL_CACHE_UNAVAILABLE)
        );
        assert_eq!(app_state.workspace.solve_session.status, RunStatus::Idle);

        fs::remove_dir_all(cache_root).ok();
    }

    #[test]
    fn solver_failure_context_extracts_lookup_unit_from_wrapped_message() {
        let context = SolveFailureContext::from_message(&format!(
            "flowsheet solve failed with package `{}`: solver.step.lookup: solver step 3 unit lookup failed for `flash-1`: missing unit `flash-1`",
            OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID
        ));

        assert_eq!(context.primary_code.as_deref(), Some("solver.step.lookup"));
        assert_eq!(
            context.related_unit_ids,
            vec![rf_types::UnitId::new("flash-1")]
        );
    }
}
