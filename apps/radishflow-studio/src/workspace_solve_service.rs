use std::path::Path;

use rf_store::StoredAuthCacheIndex;
use rf_thermo::PropertyPackageProvider;
use rf_types::RfResult;
use rf_ui::AppState;

use crate::{
    StudioSolveRequest, next_solver_snapshot_sequence, solve_workspace_from_auth_cache,
    solve_workspace_with_property_package,
};

#[derive(Debug, Clone, Default)]
pub struct WorkspaceSolveService;

impl WorkspaceSolveService {
    pub fn new() -> Self {
        Self
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
        let request = self.build_request(app_state, package_id)?;
        app_state.request_manual_run();
        solve_workspace_with_property_package(app_state, package_provider, &request)?;
        Ok(request)
    }

    pub fn run_from_auth_cache(
        &self,
        app_state: &mut AppState,
        cache_root: impl AsRef<Path>,
        auth_cache_index: &StoredAuthCacheIndex,
        package_id: impl Into<String>,
    ) -> RfResult<StudioSolveRequest> {
        let request = self.build_request(app_state, package_id)?;
        app_state.request_manual_run();
        solve_workspace_from_auth_cache(app_state, cache_root, auth_cache_index, &request)?;
        Ok(request)
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

#[cfg(test)]
mod tests {
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

    use crate::{WorkspaceSolveService, build_workspace_solve_request};

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
    fn build_request_uses_document_revision_and_next_sequence() {
        let mut app_state = AppState::new(sample_document());
        let provider = sample_provider();
        let service = WorkspaceSolveService::new();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
        ))
        .expect("expected project parse");
        app_state.workspace.document = FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("heater-demo", "Heater Demo", timestamp(20)),
        );

        let first = service
            .run_with_property_package(&mut app_state, &provider, "binary-hydrocarbon-lite-v1")
            .expect("expected first solve");
        let second = build_workspace_solve_request(&app_state, "binary-hydrocarbon-lite-v1")
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
            "../../../examples/flowsheets/feed-valve-flash.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-3", "Valve Demo", timestamp(30)),
        ));

        let request = service
            .run_with_property_package(&mut app_state, &provider, "binary-hydrocarbon-lite-v1")
            .expect("expected solve");

        assert_eq!(request.snapshot_id, "doc-3-rev-0-seq-1");
        assert_eq!(
            app_state.workspace.solve_session.status,
            RunStatus::Converged
        );
        assert_eq!(app_state.workspace.snapshot_history.len(), 1);
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

        write_property_package_manifest(record.manifest_path_under(&cache_root), &manifest)
            .expect("expected manifest write");
        write_property_package_payload(
            record
                .payload_path_under(&cache_root)
                .expect("expected payload path"),
            &payload,
        )
        .expect("expected payload write");
        auth_cache_index.property_packages.push(record);

        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
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
                "binary-hydrocarbon-lite-v1",
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
