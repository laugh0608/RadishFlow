mod runtime;
mod seed;
mod temp_cache;

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use crate::{
    EntitlementPreflightOutcome, EntitlementSessionEventDriverOutcome,
    EntitlementSessionHostRuntime, EntitlementSessionHostRuntimeOutput, EntitlementSessionPolicy,
    EntitlementSessionState, RunPanelRecoveryOutcome, StudioAppCommandOutcome, StudioAppFacade,
    WorkspaceControlState, studio_runtime::StudioRuntime,
};
use rf_store::StoredAuthCacheIndex;
use rf_types::RfResult;
use rf_ui::{AppLogEntry, AppState, RunPanelWidgetModel};

use self::seed::BootstrapControlPlaneClient;
use self::temp_cache::TemporaryCacheRoot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapConfig {
    pub project_path: PathBuf,
    pub entitlement_preflight: StudioBootstrapEntitlementPreflight,
    pub entitlement_seed: StudioBootstrapEntitlementSeed,
    pub trigger: StudioBootstrapTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioBootstrapTrigger {
    AppCommand(crate::StudioAppCommand),
    Intent(rf_ui::RunPanelIntent),
    WidgetPrimaryAction,
    WidgetAction(rf_ui::RunPanelActionId),
    WidgetRecoveryAction,
    DocumentLifecycle(crate::StudioDocumentLifecycleCommand),
    InspectorTarget(rf_ui::InspectorTarget),
    InspectorDraftUpdate(crate::StudioInspectorDraftUpdateCommand),
    InspectorDraftCommit(crate::StudioInspectorDraftCommitCommand),
    InspectorDraftDiscard(crate::StudioInspectorDraftDiscardCommand),
    InspectorDraftBatchCommit(crate::StudioInspectorDraftBatchCommitCommand),
    InspectorDraftBatchDiscard(crate::StudioInspectorDraftBatchDiscardCommand),
    InspectorCompositionNormalize(crate::StudioInspectorCompositionNormalizeCommand),
    InspectorCompositionComponentAdd(crate::StudioInspectorCompositionComponentAddCommand),
    InspectorCompositionComponentRemove(crate::StudioInspectorCompositionComponentRemoveCommand),
    DocumentHistory(crate::StudioDocumentHistoryCommand),
    EntitlementWidgetPrimaryAction,
    EntitlementWidgetAction(rf_ui::EntitlementActionId),
    EntitlementSessionEvent(StudioBootstrapEntitlementSessionEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioBootstrapEntitlementPreflight {
    Skip,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioBootstrapEntitlementSeed {
    Synced,
    MissingSnapshot,
    LeaseExpiringSoon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioBootstrapEntitlementSessionEvent {
    LoginCompleted,
    TimerElapsed,
    NetworkRestored,
    WindowForegrounded,
}

impl Default for StudioBootstrapConfig {
    fn default() -> Self {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");

        Self {
            project_path: workspace_root
                .join("examples")
                .join("flowsheets")
                .join("feed-heater-flash-binary-hydrocarbon.rfproj.json"),
            entitlement_preflight: StudioBootstrapEntitlementPreflight::Auto,
            entitlement_seed: StudioBootstrapEntitlementSeed::Synced,
            trigger: StudioBootstrapTrigger::WidgetPrimaryAction,
        }
    }
}

pub(crate) struct BootstrapSession {
    app_state: AppState,
    cache_root: TemporaryCacheRoot,
    auth_cache_index: StoredAuthCacheIndex,
    control_plane_client: BootstrapControlPlaneClient,
    facade: StudioAppFacade,
    session_policy: EntitlementSessionPolicy,
    entitlement_session_state: EntitlementSessionState,
    host_runtime: EntitlementSessionHostRuntime,
    entitlement_preflight: Option<EntitlementPreflightOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioBootstrapReport {
    pub entitlement_preflight: Option<EntitlementPreflightOutcome>,
    pub entitlement_host: EntitlementSessionHostRuntimeOutput,
    pub dispatch: StudioBootstrapDispatch,
    pub control_state: WorkspaceControlState,
    pub run_panel: RunPanelWidgetModel,
    pub log_entries: Vec<AppLogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum StudioBootstrapDispatch {
    AppCommand(StudioAppCommandOutcome),
    RunPanelRecovery(RunPanelRecoveryOutcome),
    DocumentLifecycle(crate::DocumentLifecycleOutcome),
    InspectorTarget(crate::InspectorTargetFocusOutcome),
    InspectorDraftUpdate(crate::InspectorDraftUpdateOutcome),
    InspectorDraftCommit(crate::InspectorDraftCommitOutcome),
    InspectorDraftDiscard(crate::InspectorDraftDiscardOutcome),
    InspectorDraftBatchCommit(crate::InspectorDraftBatchCommitOutcome),
    InspectorDraftBatchDiscard(crate::InspectorDraftBatchDiscardOutcome),
    InspectorCompositionNormalize(crate::InspectorCompositionNormalizeOutcome),
    InspectorCompositionComponentAdd(crate::InspectorCompositionComponentAddOutcome),
    InspectorCompositionComponentRemove(crate::InspectorCompositionComponentRemoveOutcome),
    DocumentHistory(crate::DocumentHistoryOutcome),
    EntitlementSessionEvent(EntitlementSessionEventDriverOutcome),
}

pub fn run_studio_bootstrap(config: &StudioBootstrapConfig) -> RfResult<StudioBootstrapReport> {
    let mut runtime = StudioRuntime::new(config)?;
    runtime.dispatch_trigger(&config.trigger)
}
