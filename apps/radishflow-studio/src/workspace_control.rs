use rf_types::RfResult;
use rf_ui::{
    AppLogEntry, AppState, RunPanelCommandModel, RunPanelIntent, RunPanelPackageSelection,
    RunPanelNotice, RunPanelNoticeLevel, RunPanelState, RunPanelWidgetEvent, RunStatus,
    SimulationMode, SolvePendingReason,
};

use crate::{
    StudioAppAuthCacheContext, StudioAppCommand, StudioAppExecutionBoundary, StudioAppFacade,
    StudioAppResultDispatch, StudioWorkspaceRunOutcome, WorkspaceRunCommand,
    WorkspaceRunPackageSelection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceControlAction {
    RunManual(WorkspaceRunPackageSelection),
    Resume(WorkspaceRunPackageSelection),
    SetMode(SimulationMode),
}

impl WorkspaceControlAction {
    pub fn run_manual(package: WorkspaceRunPackageSelection) -> Self {
        Self::RunManual(package)
    }

    pub fn resume(package: WorkspaceRunPackageSelection) -> Self {
        Self::Resume(package)
    }

    pub fn set_mode(mode: SimulationMode) -> Self {
        Self::SetMode(mode)
    }

    pub fn to_app_command(&self) -> StudioAppCommand {
        match self {
            Self::RunManual(package) => StudioAppCommand::run_workspace(WorkspaceRunCommand::new(
                crate::WorkspaceSolveTrigger::Manual,
                package.clone(),
            )),
            Self::Resume(package) => StudioAppCommand::resume_workspace(package.clone()),
            Self::SetMode(mode) => StudioAppCommand::set_workspace_simulation_mode(*mode),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceControlState {
    pub simulation_mode: SimulationMode,
    pub run_status: RunStatus,
    pub pending_reason: Option<SolvePendingReason>,
    pub latest_snapshot_id: Option<String>,
    pub latest_snapshot_summary: Option<String>,
    pub latest_log_entry: Option<AppLogEntry>,
    pub notice: Option<RunPanelNotice>,
    pub can_run_manual: bool,
    pub can_resume: bool,
    pub can_set_hold: bool,
    pub can_set_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceControlActionOutcome {
    pub action: WorkspaceControlAction,
    pub boundary: StudioAppExecutionBoundary,
    pub dispatch: StudioAppResultDispatch,
    pub control_state: WorkspaceControlState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunPanelWidgetDispatchOutcome {
    Executed(WorkspaceControlActionOutcome),
    IgnoredDisabled { action_id: rf_ui::RunPanelActionId },
    IgnoredMissing { action_id: rf_ui::RunPanelActionId },
}

pub fn snapshot_workspace_control_state(app_state: &AppState) -> WorkspaceControlState {
    let run_panel = &app_state.workspace.run_panel;

    WorkspaceControlState {
        simulation_mode: run_panel.simulation_mode,
        run_status: run_panel.run_status,
        pending_reason: run_panel.pending_reason,
        latest_snapshot_id: run_panel.latest_snapshot_id.clone(),
        latest_snapshot_summary: run_panel.latest_snapshot_summary.clone(),
        latest_log_entry: app_state.log_feed.entries.back().cloned(),
        notice: run_panel.notice.clone(),
        can_run_manual: true,
        can_resume: run_panel.can_resume,
        can_set_hold: run_panel.can_set_hold,
        can_set_active: run_panel.can_set_active,
    }
}

pub fn map_workspace_control_state_to_run_panel_state(
    state: &WorkspaceControlState,
) -> RunPanelState {
    let mut run_panel = RunPanelState {
        simulation_mode: state.simulation_mode,
        run_status: state.run_status,
        pending_reason: state.pending_reason,
        latest_snapshot_id: state.latest_snapshot_id.clone(),
        latest_snapshot_summary: state.latest_snapshot_summary.clone(),
        latest_log_message: state
            .latest_log_entry
            .as_ref()
            .map(|entry| entry.message.clone()),
        notice: state.notice.clone(),
        can_run_manual: state.can_run_manual,
        can_resume: state.can_resume,
        can_set_hold: state.can_set_hold,
        can_set_active: state.can_set_active,
        commands: RunPanelCommandModel::default(),
    };
    run_panel.commands = RunPanelCommandModel::from_state(&run_panel);
    run_panel
}

pub fn map_run_panel_package_selection_to_workspace_run_package_selection(
    selection: &RunPanelPackageSelection,
) -> WorkspaceRunPackageSelection {
    match selection {
        RunPanelPackageSelection::Explicit(package_id) => {
            WorkspaceRunPackageSelection::Explicit(package_id.clone())
        }
        RunPanelPackageSelection::Preferred => WorkspaceRunPackageSelection::Preferred,
    }
}

pub fn map_run_panel_intent_to_workspace_control_action(
    intent: &RunPanelIntent,
) -> WorkspaceControlAction {
    match intent {
        RunPanelIntent::RunManual(selection) => WorkspaceControlAction::run_manual(
            map_run_panel_package_selection_to_workspace_run_package_selection(selection),
        ),
        RunPanelIntent::Resume(selection) => WorkspaceControlAction::resume(
            map_run_panel_package_selection_to_workspace_run_package_selection(selection),
        ),
        RunPanelIntent::SetMode(mode) => WorkspaceControlAction::set_mode(*mode),
    }
}

pub fn dispatch_run_panel_intent_with_auth_cache(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    intent: &RunPanelIntent,
) -> RfResult<WorkspaceControlActionOutcome> {
    dispatch_workspace_control_action_with_auth_cache(
        facade,
        app_state,
        context,
        &map_run_panel_intent_to_workspace_control_action(intent),
    )
}

pub fn dispatch_run_panel_widget_event_with_auth_cache(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    event: &RunPanelWidgetEvent,
) -> RfResult<RunPanelWidgetDispatchOutcome> {
    match event {
        RunPanelWidgetEvent::Dispatched { intent, .. } => {
            dispatch_run_panel_intent_with_auth_cache(facade, app_state, context, intent)
                .map(RunPanelWidgetDispatchOutcome::Executed)
        }
        RunPanelWidgetEvent::Disabled { action_id } => {
            Ok(RunPanelWidgetDispatchOutcome::IgnoredDisabled {
                action_id: *action_id,
            })
        }
        RunPanelWidgetEvent::Missing { action_id } => {
            Ok(RunPanelWidgetDispatchOutcome::IgnoredMissing {
                action_id: *action_id,
            })
        }
    }
}

pub fn dispatch_workspace_control_action_with_auth_cache(
    facade: &StudioAppFacade,
    app_state: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    action: &WorkspaceControlAction,
) -> RfResult<WorkspaceControlActionOutcome> {
    let command = action.to_app_command();
    let outcome = facade.execute_with_auth_cache(app_state, context, &command)?;
    let mut control_state = snapshot_workspace_control_state(app_state);
    control_state.notice = notice_from_dispatch(&outcome.dispatch);
    app_state.sync_run_panel_state(map_workspace_control_state_to_run_panel_state(
        &control_state,
    ));

    Ok(WorkspaceControlActionOutcome {
        action: action.clone(),
        boundary: outcome.boundary,
        dispatch: outcome.dispatch,
        control_state,
    })
}

fn notice_from_dispatch(dispatch: &StudioAppResultDispatch) -> Option<RunPanelNotice> {
    match dispatch {
        StudioAppResultDispatch::WorkspaceRun(dispatch) => match &dispatch.outcome {
            StudioWorkspaceRunOutcome::Started(request) => Some(RunPanelNotice::new(
                RunPanelNoticeLevel::Info,
                "Run completed",
                format!(
                    "workspace run used package `{}` and produced snapshot `{}`",
                    request.package_id, request.snapshot_id
                ),
            )),
            StudioWorkspaceRunOutcome::Skipped(reason) => Some(RunPanelNotice::new(
                RunPanelNoticeLevel::Info,
                "Run skipped",
                match reason {
                    crate::WorkspaceSolveSkipReason::HoldMode => {
                        "workspace is in Hold mode".to_string()
                    }
                    crate::WorkspaceSolveSkipReason::NoPendingRequest => {
                        "workspace has no pending solve request".to_string()
                    }
                },
            )),
            StudioWorkspaceRunOutcome::Blocked(blocked) => Some(RunPanelNotice::new(
                RunPanelNoticeLevel::Warning,
                notice_title_for_blocked_outcome(blocked.reason),
                blocked.message.clone(),
            )),
            StudioWorkspaceRunOutcome::Failed(failed) => Some(RunPanelNotice::new(
                RunPanelNoticeLevel::Error,
                notice_title_for_failed_outcome(failed.reason),
                failed.message.clone(),
            )),
        },
        StudioAppResultDispatch::WorkspaceMode(dispatch) => dispatch.latest_log_entry.as_ref().map(
            |entry| RunPanelNotice::new(RunPanelNoticeLevel::Info, "Mode updated", entry.message.clone()),
        ),
        StudioAppResultDispatch::Entitlement(_) => None,
    }
}

fn notice_title_for_blocked_outcome(
    reason: crate::StudioWorkspaceRunBlockedReason,
) -> &'static str {
    match reason {
        crate::StudioWorkspaceRunBlockedReason::CachedPackageMissing => "Run blocked",
        crate::StudioWorkspaceRunBlockedReason::ExplicitPackageSelectionRequired => {
            "Package selection required"
        }
        crate::StudioWorkspaceRunBlockedReason::EntitlementMismatch => {
            "Entitlement update required"
        }
        crate::StudioWorkspaceRunBlockedReason::InvalidSelection => "Run blocked",
    }
}

fn notice_title_for_failed_outcome(reason: crate::StudioWorkspaceRunFailedReason) -> &'static str {
    match reason {
        crate::StudioWorkspaceRunFailedReason::LocalCacheUnavailable => {
            "Local cache unavailable"
        }
        crate::StudioWorkspaceRunFailedReason::SolveFailed => "Run failed",
    }
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
        AppState, DocumentMetadata, FlowsheetDocument, RunPanelIntent, RunPanelPackageSelection,
        RunPanelWidgetEvent, RunPanelWidgetModel, RunStatus, SimulationMode, SolvePendingReason,
    };

    use super::{
        RunPanelWidgetDispatchOutcome, WorkspaceControlAction,
        dispatch_run_panel_intent_with_auth_cache, dispatch_run_panel_widget_event_with_auth_cache,
        dispatch_workspace_control_action_with_auth_cache,
        map_run_panel_intent_to_workspace_control_action,
        map_workspace_control_state_to_run_panel_state, snapshot_workspace_control_state,
    };
    use crate::{
        StudioAppAuthCacheContext, StudioAppCommand, StudioAppExecutionBoundary,
        StudioAppExecutionLane, StudioAppFacade, StudioAppResultDispatch,
        StudioWorkspaceRunOutcome, WorkspaceRunPackageSelection,
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
    fn control_state_marks_hold_workspace_as_resumable() {
        let app_state = AppState::new(sample_document());

        let state = snapshot_workspace_control_state(&app_state);

        assert_eq!(state.simulation_mode, SimulationMode::Hold);
        assert_eq!(state.run_status, RunStatus::Idle);
        assert_eq!(
            state.pending_reason,
            Some(SolvePendingReason::SnapshotMissing)
        );
        assert!(state.notice.is_none());
        assert!(state.can_run_manual);
        assert!(state.can_resume);
        assert!(!state.can_set_hold);
        assert!(state.can_set_active);
    }

    #[test]
    fn control_state_maps_into_rf_ui_run_panel_state() {
        let app_state = AppState::new(sample_document());

        let run_panel = map_workspace_control_state_to_run_panel_state(
            &snapshot_workspace_control_state(&app_state),
        );

        assert_eq!(run_panel, app_state.workspace.run_panel);
    }

    #[test]
    fn control_action_maps_manual_run_to_workspace_run_command() {
        let action = WorkspaceControlAction::run_manual(WorkspaceRunPackageSelection::Explicit(
            "pkg-1".to_string(),
        ));

        let command = action.to_app_command();

        match command {
            StudioAppCommand::RunWorkspace(command) => {
                assert_eq!(command.trigger, crate::WorkspaceSolveTrigger::Manual);
                assert_eq!(
                    command.package,
                    WorkspaceRunPackageSelection::Explicit("pkg-1".to_string())
                );
            }
            _ => panic!("expected run workspace command"),
        }
    }

    #[test]
    fn run_panel_intent_maps_to_workspace_control_action() {
        let action = map_run_panel_intent_to_workspace_control_action(&RunPanelIntent::resume(
            RunPanelPackageSelection::preferred(),
        ));

        assert_eq!(
            action,
            WorkspaceControlAction::resume(WorkspaceRunPackageSelection::Preferred)
        );
    }

    #[test]
    fn dispatching_hold_mode_action_returns_mode_dispatch_and_control_state() {
        let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        app_state.set_simulation_mode(SimulationMode::Active);
        let cache_root = PathBuf::from("D:\\cache-root");
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        let outcome = dispatch_workspace_control_action_with_auth_cache(
            &facade,
            &mut app_state,
            &context,
            &WorkspaceControlAction::set_mode(SimulationMode::Hold),
        )
        .expect("expected control action outcome");

        assert_eq!(
            outcome.boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceControl)
        );
        match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceMode(dispatch) => {
                assert_eq!(dispatch.simulation_mode, SimulationMode::Hold);
            }
            _ => panic!("expected workspace mode dispatch"),
        }
        assert_eq!(outcome.control_state.simulation_mode, SimulationMode::Hold);
        assert!(!outcome.control_state.can_set_hold);
        assert!(outcome.control_state.can_set_active);
        assert_eq!(
            app_state.workspace.run_panel.simulation_mode,
            SimulationMode::Hold
        );
    }

    #[test]
    fn dispatching_resume_action_runs_workspace_and_updates_control_state() {
        let cache_root = unique_temp_path("workspace-control-resume");
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
            DocumentMetadata::new("doc-control-resume", "Control Resume Demo", timestamp(70)),
        ));
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        let outcome = dispatch_workspace_control_action_with_auth_cache(
            &facade,
            &mut app_state,
            &context,
            &WorkspaceControlAction::resume(WorkspaceRunPackageSelection::Preferred),
        )
        .expect("expected control resume outcome");

        assert_eq!(
            outcome.boundary,
            StudioAppExecutionBoundary::Inline(StudioAppExecutionLane::WorkspaceSolve)
        );
        match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert!(matches!(
                    dispatch.outcome,
                    StudioWorkspaceRunOutcome::Started(_)
                ));
                assert_eq!(dispatch.simulation_mode, SimulationMode::Active);
            }
            _ => panic!("expected workspace run dispatch"),
        }
        assert_eq!(
            outcome.control_state.simulation_mode,
            SimulationMode::Active
        );
        assert_eq!(
            outcome
                .control_state
                .notice
                .as_ref()
                .map(|notice| (notice.level, notice.title.as_str())),
            Some((rf_ui::RunPanelNoticeLevel::Info, "Run completed"))
        );
        assert_eq!(outcome.control_state.run_status, RunStatus::Converged);
        assert!(!outcome.control_state.can_resume);
        assert!(outcome.control_state.can_set_hold);
        assert_eq!(
            app_state.workspace.run_panel.run_status,
            RunStatus::Converged
        );

        std::fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }

    #[test]
    fn dispatching_run_panel_intent_runs_through_workspace_control_action() {
        let cache_root = unique_temp_path("run-panel-intent");
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
            DocumentMetadata::new(
                "doc-run-panel-intent",
                "Run Panel Intent Demo",
                timestamp(80),
            ),
        ));
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        let outcome = dispatch_run_panel_intent_with_auth_cache(
            &facade,
            &mut app_state,
            &context,
            &RunPanelIntent::run_manual(RunPanelPackageSelection::preferred()),
        )
        .expect("expected run panel intent dispatch");

        assert_eq!(outcome.control_state.run_status, RunStatus::Converged);
        assert_eq!(
            app_state.workspace.run_panel.run_status,
            RunStatus::Converged
        );

        std::fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }

    #[test]
    fn dispatching_run_panel_widget_event_runs_through_widget_and_workspace_control() {
        let cache_root = unique_temp_path("run-panel-widget");
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
            DocumentMetadata::new(
                "doc-run-panel-widget",
                "Run Panel Widget Demo",
                timestamp(90),
            ),
        ));
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);
        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        let outcome = dispatch_run_panel_widget_event_with_auth_cache(
            &facade,
            &mut app_state,
            &context,
            &widget.activate_primary(),
        )
        .expect("expected widget event dispatch");

        match outcome {
            RunPanelWidgetDispatchOutcome::Executed(outcome) => {
                assert_eq!(outcome.control_state.run_status, RunStatus::Converged);
                assert_eq!(
                    outcome
                        .control_state
                        .notice
                        .as_ref()
                        .map(|notice| notice.title.as_str()),
                    Some("Run completed")
                );
            }
            _ => panic!("expected executed widget dispatch"),
        }
        assert_eq!(
            app_state.workspace.run_panel.run_status,
            RunStatus::Converged
        );

        std::fs::remove_dir_all(cache_root).expect("expected temp dir cleanup");
    }

    #[test]
    fn dispatching_disabled_run_panel_widget_event_is_ignored() {
        let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);
        let widget = RunPanelWidgetModel::from_state(&app_state.workspace.run_panel);

        let outcome = dispatch_run_panel_widget_event_with_auth_cache(
            &facade,
            &mut app_state,
            &context,
            &widget.activate(rf_ui::RunPanelActionId::SetHold),
        )
        .expect("expected ignored widget dispatch");

        assert_eq!(
            outcome,
            RunPanelWidgetDispatchOutcome::IgnoredDisabled {
                action_id: rf_ui::RunPanelActionId::SetHold
            }
        );
        assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Idle);
    }

    #[test]
    fn dispatching_missing_run_panel_widget_event_is_ignored() {
        let auth_cache_index = sample_auth_cache_index(&["pkg-1"]);
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        let outcome = dispatch_run_panel_widget_event_with_auth_cache(
            &facade,
            &mut app_state,
            &context,
            &RunPanelWidgetEvent::Missing {
                action_id: rf_ui::RunPanelActionId::Resume,
            },
        )
        .expect("expected missing widget dispatch");

        assert_eq!(
            outcome,
            RunPanelWidgetDispatchOutcome::IgnoredMissing {
                action_id: rf_ui::RunPanelActionId::Resume
            }
        );
        assert_eq!(app_state.workspace.run_panel.run_status, RunStatus::Idle);
    }

    #[test]
    fn blocked_workspace_run_updates_run_panel_notice() {
        let auth_cache_index = sample_auth_cache_index(&["pkg-1", "pkg-2"]);
        let facade = StudioAppFacade::new();
        let mut app_state = AppState::new(sample_document());
        let cache_root = PathBuf::from("D:\\cache-root");
        let context = StudioAppAuthCacheContext::new(&cache_root, &auth_cache_index);

        let outcome = dispatch_workspace_control_action_with_auth_cache(
            &facade,
            &mut app_state,
            &context,
            &WorkspaceControlAction::run_manual(WorkspaceRunPackageSelection::Preferred),
        )
        .expect("expected blocked control action");

        match outcome.dispatch {
            StudioAppResultDispatch::WorkspaceRun(dispatch) => {
                assert!(matches!(
                    dispatch.outcome,
                    StudioWorkspaceRunOutcome::Blocked(_)
                ));
            }
            _ => panic!("expected workspace run dispatch"),
        }
        assert_eq!(outcome.control_state.run_status, RunStatus::Idle);
        assert_eq!(
            outcome
                .control_state
                .notice
                .as_ref()
                .map(|notice| (notice.level, notice.title.as_str())),
            Some((rf_ui::RunPanelNoticeLevel::Warning, "Package selection required"))
        );
        assert_eq!(
            app_state
                .workspace
                .run_panel
                .notice
                .as_ref()
                .map(|notice| notice.message.as_str()),
            Some("multiple cached property packages are available; explicit package selection is required")
        );
    }
}
