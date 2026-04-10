use rf_types::RfResult;
use rf_ui::{
    AppState, InspectorTarget, RunPanelActionId, RunPanelRecoveryAction,
    RunPanelRecoveryWidgetEvent, RunPanelWidgetModel,
};

use crate::{
    RunPanelWidgetDispatchOutcome, StudioAppAuthCacheContext, StudioAppFacade,
    WorkspaceControlState, dispatch_run_panel_widget_event_with_auth_cache,
    snapshot_workspace_control_state,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelDriverState {
    pub widget: RunPanelWidgetModel,
    pub control_state: WorkspaceControlState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelDriverOutcome {
    pub dispatch: RunPanelWidgetDispatchOutcome,
    pub state: RunPanelDriverState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPanelRecoveryOutcome {
    pub action: RunPanelRecoveryAction,
    pub applied_target: Option<InspectorTarget>,
    pub state: RunPanelDriverState,
}

pub fn snapshot_run_panel_driver_state(app_state: &AppState) -> RunPanelDriverState {
    RunPanelDriverState {
        widget: RunPanelWidgetModel::from_state(&app_state.workspace.run_panel),
        control_state: snapshot_workspace_control_state(app_state),
    }
}

pub fn dispatch_run_panel_widget_action_with_auth_cache(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    action_id: RunPanelActionId,
) -> RfResult<RunPanelDriverOutcome> {
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);
    let dispatch = dispatch_run_panel_widget_event_with_auth_cache(
        facade,
        app_state,
        context,
        &widget.activate(action_id),
    )?;
    let state = snapshot_run_panel_driver_state(app_state);

    Ok(RunPanelDriverOutcome { dispatch, state })
}

pub fn dispatch_run_panel_primary_action_with_auth_cache(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
) -> RfResult<RunPanelDriverOutcome> {
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);
    let dispatch = dispatch_run_panel_widget_event_with_auth_cache(
        facade,
        app_state,
        context,
        &widget.activate_primary(),
    )?;
    let state = snapshot_run_panel_driver_state(app_state);

    Ok(RunPanelDriverOutcome { dispatch, state })
}

pub fn apply_run_panel_recovery_action(
    app_state: &mut AppState,
) -> Option<RunPanelRecoveryOutcome> {
    let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);
    let action = match widget.activate_recovery_action() {
        RunPanelRecoveryWidgetEvent::Requested { action } => action,
        RunPanelRecoveryWidgetEvent::Missing => return None,
    };
    let applied_target = app_state.apply_run_panel_recovery_action(&action);
    let state = snapshot_run_panel_driver_state(app_state);

    Some(RunPanelRecoveryOutcome {
        action,
        applied_target,
        state,
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rf_model::Flowsheet;
    use rf_store::{
        StoredAntoineCoefficients, StoredAuthCacheIndex, StoredCredentialReference,
        StoredPropertyPackageManifest, StoredPropertyPackagePayload, StoredPropertyPackageRecord,
        StoredPropertyPackageSource, StoredThermoComponent, parse_project_file_json,
        property_package_payload_integrity, write_property_package_manifest,
        write_property_package_payload,
    };
    use rf_types::ComponentId;
    use rf_ui::{
        AppState, DocumentMetadata, FlowsheetDocument, InspectorTarget, RunPanelActionId,
        RunPanelWidgetModel, RunStatus,
    };

    use super::{
        RunPanelDriverOutcome, RunPanelDriverState, apply_run_panel_recovery_action,
        dispatch_run_panel_primary_action_with_auth_cache,
        dispatch_run_panel_widget_action_with_auth_cache, snapshot_run_panel_driver_state,
    };
    use crate::{RunPanelWidgetDispatchOutcome, StudioAppAuthCacheContext, StudioAppFacade};

    fn timestamp(seconds: u64) -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn sample_document() -> FlowsheetDocument {
        let flowsheet = Flowsheet::new("demo");
        let metadata = DocumentMetadata::new("doc-1", "Demo", timestamp(10));
        FlowsheetDocument::new(flowsheet, metadata)
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("radishflow-{name}-{unique}"))
    }

    fn write_cached_package(
        cache_root: &Path,
        auth_cache_index: &mut StoredAuthCacheIndex,
        package_id: &str,
    ) {
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

        let payload =
            StoredPropertyPackagePayload::new(package_id, "2026.03.1", vec![first, second]);
        let integrity =
            property_package_payload_integrity(&payload).expect("expected payload integrity");
        let expires_at = Some(SystemTime::now() + Duration::from_secs(3_600));
        let mut manifest = StoredPropertyPackageManifest::new(
            package_id,
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

        write_property_package_manifest(record.manifest_path_under(cache_root), &manifest)
            .expect("expected manifest write");
        write_property_package_payload(
            record
                .payload_path_under(cache_root)
                .expect("expected payload path"),
            &payload,
        )
        .expect("expected payload write");
        auth_cache_index.property_packages.push(record);
    }

    #[test]
    fn snapshot_run_panel_driver_state_builds_widget_and_control_state() {
        let app_state = AppState::new(sample_document());

        let state = snapshot_run_panel_driver_state(&app_state);

        assert_eq!(state.widget.view().primary_action.label, "Resume");
        assert_eq!(state.control_state.run_status, RunStatus::Idle);
    }

    #[test]
    fn dispatching_primary_action_through_driver_executes_workspace_run() {
        let cache_root = unique_temp_path("run-panel-driver-primary");
        let mut auth_cache_index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        write_cached_package(
            &cache_root,
            &mut auth_cache_index,
            "binary-hydrocarbon-lite-v1",
        );
        let facade = StudioAppFacade::new();
        let project = parse_project_file_json(include_str!(
            "../../../examples/flowsheets/feed-heater-flash.rfproj.json"
        ))
        .expect("expected project parse");
        let mut app_state = AppState::new(FlowsheetDocument::new(
            project.document.flowsheet,
            DocumentMetadata::new("doc-driver-primary", "Driver Primary Demo", timestamp(70)),
        ));
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        let outcome =
            dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
                .expect("expected primary driver dispatch");

        match outcome {
            RunPanelDriverOutcome {
                dispatch: RunPanelWidgetDispatchOutcome::Executed(outcome),
                state,
            } => {
                assert_eq!(outcome.control_state.run_status, RunStatus::Converged);
                assert_eq!(state.widget.view().primary_action.label, "Run");
            }
            _ => panic!("expected executed driver outcome"),
        }

        std::fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }

    #[test]
    fn dispatching_disabled_action_through_driver_returns_ignored_state() {
        let auth_cache_index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        let outcome = dispatch_run_panel_widget_action_with_auth_cache(
            &facade,
            &mut app_state,
            &context,
            RunPanelActionId::SetHold,
        )
        .expect("expected ignored driver dispatch");

        assert_eq!(
            outcome,
            RunPanelDriverOutcome {
                dispatch: RunPanelWidgetDispatchOutcome::IgnoredDisabled {
                    action_id: RunPanelActionId::SetHold,
                    detail: "Workspace is already in Hold mode",
                },
                state: RunPanelDriverState {
                    widget: RunPanelWidgetModel::from_state(&app_state.workspace.run_panel),
                    control_state: snapshot_run_panel_driver_state(&app_state).control_state,
                },
            }
        );
    }

    #[test]
    fn applying_recovery_action_without_notice_returns_none() {
        let mut app_state = AppState::new(sample_document());

        assert_eq!(apply_run_panel_recovery_action(&mut app_state), None);
    }

    #[test]
    fn applying_recovery_action_focuses_related_unit_in_inspector() {
        let cache_root = unique_temp_path("run-panel-driver-recovery");
        let mut auth_cache_index = StoredAuthCacheIndex::new(
            "https://id.radish.local",
            "user-123",
            StoredCredentialReference::new("radishflow-studio", "user-123-primary"),
        );
        write_cached_package(
            &cache_root,
            &mut auth_cache_index,
            "binary-hydrocarbon-lite-v1",
        );
        let facade = StudioAppFacade::new();
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
            DocumentMetadata::new("doc-driver-recovery", "Driver Recovery Demo", timestamp(90)),
        ));
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        dispatch_run_panel_primary_action_with_auth_cache(&facade, &mut app_state, &context)
            .expect("expected failed primary driver dispatch");

        let outcome = apply_run_panel_recovery_action(&mut app_state)
            .expect("expected run panel recovery outcome");

        assert_eq!(outcome.action.title, "Inspect unit inputs");
        assert_eq!(
            outcome.applied_target,
            Some(InspectorTarget::Unit(rf_types::UnitId::new("valve-1")))
        );
        assert_eq!(
            outcome
                .state
                .control_state
                .notice
                .as_ref()
                .and_then(|notice| {
                    notice
                        .recovery_action
                        .as_ref()
                        .and_then(|action| action.target_unit_id.as_ref())
                        .map(|unit_id| unit_id.as_str())
                }),
            Some("valve-1")
        );
        assert!(
            app_state
                .workspace
                .selection
                .selected_units
                .contains(&rf_types::UnitId::new("valve-1"))
        );
        assert_eq!(
            app_state.workspace.drafts.active_target,
            Some(InspectorTarget::Unit(rf_types::UnitId::new("valve-1")))
        );
        assert!(app_state.workspace.panels.inspector_open);

        std::fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }
}
