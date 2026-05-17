use std::path::Path;

use rf_store::StoredAuthCacheIndex;
use rf_thermo::PropertyPackageProvider;
use rf_types::RfResult;
use rf_ui::AppState;

use crate::{
    StudioSolveRequest, next_solver_snapshot_sequence, solve_workspace_from_auth_cache,
    solve_workspace_with_property_package,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSolveTrigger {
    Manual,
    Automatic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSolveSkipReason {
    HoldMode,
    NoPendingRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceSolveDispatch {
    Started(StudioSolveRequest),
    Skipped(WorkspaceSolveSkipReason),
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceSolveService;

impl WorkspaceSolveService {
    pub fn new() -> Self {
        Self
    }

    pub fn skip_reason(
        &self,
        app_state: &AppState,
        trigger: WorkspaceSolveTrigger,
    ) -> Option<WorkspaceSolveSkipReason> {
        should_skip_workspace_solve(app_state, trigger)
    }

    pub fn build_request(
        &self,
        app_state: &AppState,
        package_id: impl Into<String>,
    ) -> RfResult<StudioSolveRequest> {
        build_workspace_solve_request(app_state, package_id)
    }

    pub fn run_with_property_package<P>(
        &self,
        app_state: &mut AppState,
        package_provider: &P,
        package_id: impl Into<String>,
    ) -> RfResult<StudioSolveRequest>
    where
        P: PropertyPackageProvider,
    {
        match self.dispatch_with_property_package(
            app_state,
            package_provider,
            package_id,
            WorkspaceSolveTrigger::Manual,
        )? {
            WorkspaceSolveDispatch::Started(request) => Ok(request),
            WorkspaceSolveDispatch::Skipped(reason) => {
                unreachable!("manual workspace solve should never skip, got {reason:?}")
            }
        }
    }

    pub fn run_from_auth_cache(
        &self,
        app_state: &mut AppState,
        cache_root: impl AsRef<Path>,
        auth_cache_index: &StoredAuthCacheIndex,
        package_id: impl Into<String>,
    ) -> RfResult<StudioSolveRequest> {
        match self.dispatch_from_auth_cache(
            app_state,
            cache_root,
            auth_cache_index,
            package_id,
            WorkspaceSolveTrigger::Manual,
        )? {
            WorkspaceSolveDispatch::Started(request) => Ok(request),
            WorkspaceSolveDispatch::Skipped(reason) => unreachable!(
                "manual workspace solve from auth cache should never skip, got {reason:?}"
            ),
        }
    }

    pub fn dispatch_with_property_package<P>(
        &self,
        app_state: &mut AppState,
        package_provider: &P,
        package_id: impl Into<String>,
        trigger: WorkspaceSolveTrigger,
    ) -> RfResult<WorkspaceSolveDispatch>
    where
        P: PropertyPackageProvider,
    {
        if let Some(skip_reason) = self.skip_reason(app_state, trigger) {
            return Ok(WorkspaceSolveDispatch::Skipped(skip_reason));
        }

        if matches!(trigger, WorkspaceSolveTrigger::Manual) {
            app_state.request_manual_run();
        }

        let request = self.build_request(app_state, package_id)?;
        solve_workspace_with_property_package(app_state, package_provider, &request)?;
        Ok(WorkspaceSolveDispatch::Started(request))
    }

    pub fn dispatch_from_auth_cache(
        &self,
        app_state: &mut AppState,
        cache_root: impl AsRef<Path>,
        auth_cache_index: &StoredAuthCacheIndex,
        package_id: impl Into<String>,
        trigger: WorkspaceSolveTrigger,
    ) -> RfResult<WorkspaceSolveDispatch> {
        if let Some(skip_reason) = self.skip_reason(app_state, trigger) {
            return Ok(WorkspaceSolveDispatch::Skipped(skip_reason));
        }

        if matches!(trigger, WorkspaceSolveTrigger::Manual) {
            app_state.request_manual_run();
        }

        let request = self.build_request(app_state, package_id)?;
        solve_workspace_from_auth_cache(app_state, cache_root, auth_cache_index, &request)?;
        Ok(WorkspaceSolveDispatch::Started(request))
    }
}

pub fn build_workspace_solve_request(
    app_state: &AppState,
    package_id: impl Into<String>,
) -> RfResult<StudioSolveRequest> {
    let revision = app_state.workspace.document.revision;
    let sequence = next_solver_snapshot_sequence(app_state);
    let snapshot_id = format!(
        "{}-rev-{}-seq-{}",
        app_state.workspace.document.metadata.document_id.as_str(),
        revision,
        sequence
    );
    let request = StudioSolveRequest::new(package_id, snapshot_id, sequence);
    request.validate()?;
    Ok(request)
}

fn should_skip_workspace_solve(
    app_state: &AppState,
    trigger: WorkspaceSolveTrigger,
) -> Option<WorkspaceSolveSkipReason> {
    if matches!(trigger, WorkspaceSolveTrigger::Manual) {
        return None;
    }

    if !matches!(
        app_state.workspace.solve_session.mode,
        rf_ui::SimulationMode::Active
    ) {
        return Some(WorkspaceSolveSkipReason::HoldMode);
    }

    if app_state.workspace.solve_session.pending_reason.is_none() {
        return Some(WorkspaceSolveSkipReason::NoPendingRequest);
    }

    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_store::{StoredAuthCacheIndex, StoredCredentialReference, parse_project_file_json};
    use rf_thermo::InMemoryPropertyPackageProvider;
    use rf_ui::{AppState, DocumentMetadata, FlowsheetDocument, RunStatus};

    use super::{
        WorkspaceSolveDispatch, WorkspaceSolveService, WorkspaceSolveSkipReason,
        WorkspaceSolveTrigger, build_workspace_solve_request,
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
    fn build_request_uses_document_revision_and_next_sequence() {
        let mut app_state = AppState::new(sample_document());
        let provider = sample_provider();
        let service = WorkspaceSolveService::new();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected project parse");
        app_state.workspace.document = FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("heater-demo", "Heater Demo", timestamp(20)),
        );

        let first = service
            .run_with_property_package(
                &mut app_state,
                &provider,
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            )
            .expect("expected first solve");
        let second =
            build_workspace_solve_request(&app_state, OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID)
                .expect("expected next request");

        assert_eq!(first.snapshot_id, "heater-demo-rev-0-seq-1");
        assert_eq!(first.sequence, 1);
        assert_eq!(second.snapshot_id, "heater-demo-rev-0-seq-2");
        assert_eq!(second.sequence, 2);
    }

    #[test]
    fn run_with_property_package_solves_workspace_and_returns_request() {
        let provider = sample_provider();
        let service = WorkspaceSolveService::new();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-3", "Valve Demo", timestamp(30)),
        ));

        let request = service
            .run_with_property_package(
                &mut app_state,
                &provider,
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            )
            .expect("expected solve");

        assert_eq!(request.snapshot_id, "doc-3-rev-0-seq-1");
        assert_eq!(
            app_state.workspace.solve_session.status,
            RunStatus::Converged
        );
        assert_eq!(app_state.workspace.snapshot_history.len(), 1);
    }

    #[test]
    fn automatic_dispatch_skips_when_workspace_is_on_hold() {
        let provider = sample_provider();
        let service = WorkspaceSolveService::new();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-auto-hold", "Auto Hold Demo", timestamp(35)),
        ));

        let dispatch = service
            .dispatch_with_property_package(
                &mut app_state,
                &provider,
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
                WorkspaceSolveTrigger::Automatic,
            )
            .expect("expected dispatch result");

        assert_eq!(
            dispatch,
            WorkspaceSolveDispatch::Skipped(WorkspaceSolveSkipReason::HoldMode)
        );
        assert!(app_state.workspace.snapshot_history.is_empty());
    }

    #[test]
    fn automatic_dispatch_skips_when_active_workspace_has_no_pending_request() {
        let provider = sample_provider();
        let service = WorkspaceSolveService::new();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-auto-clean", "Auto Clean Demo", timestamp(36)),
        ));

        app_state.set_simulation_mode(rf_ui::SimulationMode::Active);
        let first = service
            .run_with_property_package(
                &mut app_state,
                &provider,
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            )
            .expect("expected manual solve");
        assert_eq!(first.sequence, 1);

        let dispatch = service
            .dispatch_with_property_package(
                &mut app_state,
                &provider,
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
                WorkspaceSolveTrigger::Automatic,
            )
            .expect("expected dispatch result");

        assert_eq!(
            dispatch,
            WorkspaceSolveDispatch::Skipped(WorkspaceSolveSkipReason::NoPendingRequest)
        );
        assert_eq!(app_state.workspace.snapshot_history.len(), 1);
    }

    #[test]
    fn automatic_dispatch_runs_when_active_workspace_has_pending_reason() {
        let provider = sample_provider();
        let service = WorkspaceSolveService::new();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet.clone(),
            DocumentMetadata::new("doc-auto-run", "Auto Run Demo", timestamp(37)),
        ));

        app_state.set_simulation_mode(rf_ui::SimulationMode::Active);
        app_state.commit_document_change(
            rf_ui::DocumentCommand::RenameUnit {
                unit_id: rf_types::UnitId::new("heater-1"),
                new_name: "Heater Updated".to_string(),
            },
            project.document.flowsheet,
            timestamp(38),
        );

        let dispatch = service
            .dispatch_with_property_package(
                &mut app_state,
                &provider,
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
                WorkspaceSolveTrigger::Automatic,
            )
            .expect("expected dispatch result");

        let request = match dispatch {
            WorkspaceSolveDispatch::Started(request) => request,
            WorkspaceSolveDispatch::Skipped(reason) => {
                panic!("expected automatic dispatch to run, got {reason:?}")
            }
        };

        assert_eq!(request.sequence, 1);
        assert_eq!(request.snapshot_id, "doc-auto-run-rev-1-seq-1");
        assert_eq!(
            app_state.workspace.solve_session.status,
            RunStatus::Converged
        );
        assert_eq!(app_state.workspace.solve_session.pending_reason, None);
    }

    #[test]
    fn build_request_rejects_blank_package_id_without_mutating_state() {
        let service = WorkspaceSolveService::new();
        let app_state = AppState::new(sample_document());

        let error = service
            .build_request(&app_state, "   ")
            .expect_err("expected invalid package id");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert_eq!(app_state.workspace.solve_session.status, RunStatus::Idle);
        assert_eq!(
            app_state.workspace.solve_session.pending_reason,
            Some(rf_ui::SolvePendingReason::SnapshotMissing)
        );
    }

    #[test]
    fn run_from_auth_cache_solves_workspace_and_returns_request() {
        let cache_root = unique_temp_path("workspace-solve-service");
        let service = WorkspaceSolveService::new();
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

        let request = service
            .run_from_auth_cache(
                &mut app_state,
                &cache_root,
                &auth_cache_index,
                OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            )
            .expect("expected solve from auth cache");

        assert_eq!(request.snapshot_id, "doc-5-rev-0-seq-1");
        assert_eq!(
            app_state.workspace.solve_session.status,
            RunStatus::Converged
        );
        assert_eq!(app_state.workspace.snapshot_history.len(), 1);

        std::fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }
}
