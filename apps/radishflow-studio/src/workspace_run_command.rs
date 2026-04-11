use std::collections::BTreeSet;
use std::path::Path;

use rf_store::StoredAuthCacheIndex;
use rf_types::{RfError, RfResult};
use rf_ui::AppState;

use crate::{WorkspaceSolveDispatch, WorkspaceSolveService, WorkspaceSolveTrigger};

pub(crate) const WORKSPACE_RUN_DIAGNOSTIC_CACHED_PACKAGE_MISSING: &str =
    "workspace.run.cached_package_missing";
pub(crate) const WORKSPACE_RUN_DIAGNOSTIC_EXPLICIT_PACKAGE_SELECTION_REQUIRED: &str =
    "workspace.run.explicit_package_selection_required";
pub(crate) const WORKSPACE_RUN_DIAGNOSTIC_ENTITLEMENT_MISMATCH: &str =
    "workspace.run.entitlement_mismatch";
pub(crate) const WORKSPACE_RUN_DIAGNOSTIC_INVALID_SELECTION: &str =
    "workspace.run.invalid_selection";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceRunPackageSelection {
    Explicit(String),
    Preferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRunCommand {
    pub trigger: WorkspaceSolveTrigger,
    pub package: WorkspaceRunPackageSelection,
}

impl WorkspaceRunCommand {
    pub fn new(trigger: WorkspaceSolveTrigger, package: WorkspaceRunPackageSelection) -> Self {
        Self { trigger, package }
    }

    pub fn manual(package_id: impl Into<String>) -> Self {
        Self::new(
            WorkspaceSolveTrigger::Manual,
            WorkspaceRunPackageSelection::Explicit(package_id.into()),
        )
    }

    pub fn automatic_preferred() -> Self {
        Self::new(
            WorkspaceSolveTrigger::Automatic,
            WorkspaceRunPackageSelection::Preferred,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRunDispatchResult {
    pub package_id: Option<String>,
    pub dispatch: WorkspaceSolveDispatch,
}

pub fn dispatch_workspace_run_from_auth_cache(
    app_state: &mut AppState,
    solve_service: &WorkspaceSolveService,
    cache_root: impl AsRef<Path>,
    auth_cache_index: &StoredAuthCacheIndex,
    command: &WorkspaceRunCommand,
) -> RfResult<WorkspaceRunDispatchResult> {
    if let Some(skip_reason) = solve_service.skip_reason(app_state, command.trigger) {
        return Ok(WorkspaceRunDispatchResult {
            package_id: None,
            dispatch: WorkspaceSolveDispatch::Skipped(skip_reason),
        });
    }

    let package_id =
        resolve_workspace_run_package_id(app_state, auth_cache_index, &command.package)?;
    let dispatch = solve_service.dispatch_from_auth_cache(
        app_state,
        cache_root,
        auth_cache_index,
        package_id.clone(),
        command.trigger,
    )?;

    Ok(WorkspaceRunDispatchResult {
        package_id: Some(package_id),
        dispatch,
    })
}

pub fn resolve_workspace_run_package_id(
    app_state: &AppState,
    auth_cache_index: &StoredAuthCacheIndex,
    selection: &WorkspaceRunPackageSelection,
) -> RfResult<String> {
    match selection {
        WorkspaceRunPackageSelection::Explicit(package_id) => {
            resolve_explicit_package_id(app_state, auth_cache_index, package_id)
        }
        WorkspaceRunPackageSelection::Preferred => {
            resolve_preferred_package_id(app_state, auth_cache_index)
        }
    }
}

fn resolve_explicit_package_id(
    app_state: &AppState,
    auth_cache_index: &StoredAuthCacheIndex,
    package_id: &str,
) -> RfResult<String> {
    if package_id.trim().is_empty() {
        return Err(
            RfError::invalid_input("workspace run command must contain a non-empty package_id")
                .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_INVALID_SELECTION),
        );
    }

    if !auth_cache_index
        .property_packages
        .iter()
        .any(|record| record.package_id == package_id)
    {
        return Err(
            RfError::missing_entity("cached property package", package_id)
                .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_CACHED_PACKAGE_MISSING),
        );
    }

    if !app_state.entitlement.package_manifests.is_empty()
        && !app_state
            .entitlement
            .package_manifests
            .contains_key(package_id)
    {
        return Err(
            RfError::invalid_input(format!(
                "workspace run package `{package_id}` is not present in entitlement manifests"
            ))
            .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_ENTITLEMENT_MISMATCH),
        );
    }

    Ok(package_id.to_string())
}

fn resolve_preferred_package_id(
    app_state: &AppState,
    auth_cache_index: &StoredAuthCacheIndex,
) -> RfResult<String> {
    let cached_package_ids = auth_cache_index
        .property_packages
        .iter()
        .map(|record| record.package_id.clone())
        .collect::<BTreeSet<_>>();

    if cached_package_ids.is_empty() {
        return Err(
            RfError::missing_entity("cached property package", "preferred-package")
                .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_CACHED_PACKAGE_MISSING),
        );
    }

    let preferred_candidates = if app_state.entitlement.package_manifests.is_empty() {
        cached_package_ids.into_iter().collect::<Vec<_>>()
    } else {
        cached_package_ids
            .into_iter()
            .filter(|package_id| {
                app_state
                    .entitlement
                    .package_manifests
                    .contains_key(package_id)
            })
            .collect::<Vec<_>>()
    };

    match preferred_candidates.as_slice() {
        [] => Err(
            RfError::invalid_input("no cached property package matches current entitlement manifests")
                .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_ENTITLEMENT_MISMATCH),
        ),
        [package_id] => Ok(package_id.clone()),
        _ => Err(
            RfError::invalid_input(
                "multiple cached property packages are available; explicit package selection is required",
            )
            .with_diagnostic_code(WORKSPACE_RUN_DIAGNOSTIC_EXPLICIT_PACKAGE_SELECTION_REQUIRED),
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::time::{Duration, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_store::{
        StoredAuthCacheIndex, StoredCredentialReference, StoredPropertyPackageRecord,
        StoredPropertyPackageSource,
    };
    use rf_ui::{
        AppState, DocumentMetadata, EntitlementSnapshot, EntitlementStatus, FlowsheetDocument,
        PropertyPackageManifest, PropertyPackageSource,
    };

    use super::{
        WORKSPACE_RUN_DIAGNOSTIC_CACHED_PACKAGE_MISSING,
        WORKSPACE_RUN_DIAGNOSTIC_ENTITLEMENT_MISMATCH,
        WORKSPACE_RUN_DIAGNOSTIC_EXPLICIT_PACKAGE_SELECTION_REQUIRED, WorkspaceRunCommand,
        WorkspaceRunPackageSelection, dispatch_workspace_run_from_auth_cache,
        resolve_workspace_run_package_id,
    };
    use crate::{
        WorkspaceSolveDispatch, WorkspaceSolveService, WorkspaceSolveSkipReason,
        WorkspaceSolveTrigger,
    };

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new("doc-1", "Demo", timestamp(10));
        FlowsheetDocument::new(flowsheet, metadata)
    }

    fn sample_auth_cache_index(package_ids: &[&str]) -> StoredAuthCacheIndex {
        let mut index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        index.property_packages = package_ids
            .iter()
            .map(|package_id| {
                let mut record = StoredPropertyPackageRecord::new(
                    *package_id,
                    "2026.03.1",
                    StoredPropertyPackageSource::RemoteDerivedPackage,
                    "sha256:test",
                    128,
                    timestamp(20),
                );
                record.expires_at = Some(timestamp(9_999_999_999));
                record
            })
            .collect();
        index
    }

    fn insert_manifest(app_state: &mut AppState, package_id: &str) {
        let snapshot = EntitlementSnapshot {
            schema_version: 1,
            subject_id: "user-123".to_string(),
            tenant_id: Some("tenant-1".to_string()),
            issued_at: timestamp(30),
            expires_at: timestamp(300),
            offline_lease_expires_at: Some(timestamp(400)),
            features: BTreeSet::from(["desktop-login".to_string()]),
            allowed_package_ids: BTreeSet::from([package_id.to_string()]),
        };
        let mut manifest = PropertyPackageManifest::new(
            package_id,
            "2026.03.1",
            PropertyPackageSource::RemoteDerivedPackage,
        );
        manifest.component_ids = vec!["component-a".into(), "component-b".into()];
        app_state
            .entitlement
            .update(snapshot, vec![manifest], timestamp(31));
        app_state.entitlement.status = EntitlementStatus::Active;
    }

    #[test]
    fn preferred_package_uses_single_cached_package_without_entitlement() {
        let app_state = AppState::new(sample_document());
        let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);

        let package_id = resolve_workspace_run_package_id(
            &app_state,
            &auth_cache_index,
            &WorkspaceRunPackageSelection::Preferred,
        )
        .expect("expected preferred package");

        assert_eq!(package_id, "pkg-1");
    }

    #[test]
    fn preferred_package_requires_explicit_selection_when_multiple_cached_packages_exist() {
        let app_state = AppState::new(sample_document());
        let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);

        let error = resolve_workspace_run_package_id(
            &app_state,
            &auth_cache_index,
            &WorkspaceRunPackageSelection::Preferred,
        )
        .expect_err("expected ambiguous package error");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert_eq!(
            error.context().diagnostic_code(),
            Some(WORKSPACE_RUN_DIAGNOSTIC_EXPLICIT_PACKAGE_SELECTION_REQUIRED)
        );
        assert!(error.message().contains("explicit package selection"));
    }

    #[test]
    fn preferred_package_intersects_cached_and_entitled_manifests() {
        let mut app_state = AppState::new(sample_document());
        let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);
        insert_manifest(&mut app_state, "pkg-2");

        let package_id = resolve_workspace_run_package_id(
            &app_state,
            &auth_cache_index,
            &WorkspaceRunPackageSelection::Preferred,
        )
        .expect("expected intersected package");

        assert_eq!(package_id, "pkg-2");
    }

    #[test]
    fn explicit_package_requires_cached_record() {
        let app_state = AppState::new(sample_document());
        let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);

        let error = resolve_workspace_run_package_id(
            &app_state,
            &auth_cache_index,
            &WorkspaceRunPackageSelection::Explicit("pkg-2".to_string()),
        )
        .expect_err("expected missing cache error");

        assert_eq!(error.code().as_str(), "missing_entity");
        assert_eq!(
            error.context().diagnostic_code(),
            Some(WORKSPACE_RUN_DIAGNOSTIC_CACHED_PACKAGE_MISSING)
        );
    }

    #[test]
    fn explicit_package_must_match_entitlement_when_manifests_exist() {
        let mut app_state = AppState::new(sample_document());
        let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);
        insert_manifest(&mut app_state, "pkg-1");

        let error = resolve_workspace_run_package_id(
            &app_state,
            &auth_cache_index,
            &WorkspaceRunPackageSelection::Explicit("pkg-2".to_string()),
        )
        .expect_err("expected entitlement mismatch");

        assert_eq!(error.code().as_str(), "invalid_input");
        assert_eq!(
            error.context().diagnostic_code(),
            Some(WORKSPACE_RUN_DIAGNOSTIC_ENTITLEMENT_MISMATCH)
        );
        assert!(error.message().contains("entitlement manifests"));
    }

    #[test]
    fn automatic_command_uses_preferred_package_and_returns_skip_dispatch() {
        let mut app_state = AppState::new(sample_document());
        let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);
        let service = WorkspaceSolveService::new();
        let command = WorkspaceRunCommand::new(
            WorkspaceSolveTrigger::Automatic,
            WorkspaceRunPackageSelection::Preferred,
        );

        let result = dispatch_workspace_run_from_auth_cache(
            &mut app_state,
            &service,
            "D:\\cache-root",
            &auth_cache_index,
            &command,
        )
        .expect("expected dispatch result");

        assert_eq!(result.package_id, None);
        assert_eq!(
            result.dispatch,
            WorkspaceSolveDispatch::Skipped(WorkspaceSolveSkipReason::HoldMode)
        );
    }

    #[test]
    fn automatic_command_skips_before_preferred_package_resolution() {
        let mut app_state = AppState::new(sample_document());
        let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);
        let service = WorkspaceSolveService::new();
        let command = WorkspaceRunCommand::automatic_preferred();

        let result = dispatch_workspace_run_from_auth_cache(
            &mut app_state,
            &service,
            "D:\\cache-root",
            &auth_cache_index,
            &command,
        )
        .expect("expected dispatch result");

        assert_eq!(result.package_id, None);
        assert_eq!(
            result.dispatch,
            WorkspaceSolveDispatch::Skipped(WorkspaceSolveSkipReason::HoldMode)
        );
    }
}
