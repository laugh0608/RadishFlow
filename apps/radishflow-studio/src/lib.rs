mod app_facade;
mod auth_cache_sync;
mod bootstrap;
mod control_plane_client;
mod control_plane_sync;
mod property_package_download;
mod property_package_download_client;
mod solver_bridge;
mod workspace_control;
mod workspace_run_command;
mod workspace_solve_service;

pub use app_facade::{
    StudioAppAuthCacheContext, StudioAppCommand, StudioAppCommandOutcome,
    StudioAppExecutionBoundary, StudioAppExecutionLane, StudioAppFacade, StudioAppResultDispatch,
    StudioWorkspaceModeDispatch, StudioWorkspaceRunDispatch,
};
pub use auth_cache_sync::{
    apply_offline_refresh_to_auth_cache, build_auth_cache_index, build_offline_refresh_request,
    persist_downloaded_package_to_cache, record_downloaded_package, sync_auth_cache_index,
};
pub use bootstrap::{StudioBootstrapConfig, StudioBootstrapReport, run_studio_bootstrap};
pub use control_plane_client::{
    HttpRadishFlowControlPlaneClient, RadishFlowControlPlaneClient,
    RadishFlowControlPlaneClientError, RadishFlowControlPlaneClientErrorKind,
    RadishFlowControlPlaneEndpoints, RadishFlowControlPlaneHttpMethod,
    RadishFlowControlPlaneHttpRequest, RadishFlowControlPlaneHttpResponse,
    RadishFlowControlPlaneHttpTransport, RadishFlowControlPlaneHttpTransportError,
    RadishFlowControlPlaneHttpTransportErrorKind, RadishFlowControlPlaneResponse,
    ReqwestRadishFlowControlPlaneHttpTransport, ReqwestRadishFlowControlPlaneHttpTransportOptions,
};
pub use control_plane_sync::{EntitlementSyncResult, RadishFlowControlPlaneSyncService};
pub use property_package_download::{
    PROPERTY_PACKAGE_DOWNLOAD_KIND, PROPERTY_PACKAGE_DOWNLOAD_SCHEMA_VERSION,
    PropertyPackageDownload, PropertyPackageDownloadAntoineCoefficients,
    PropertyPackageDownloadComponent, PropertyPackageDownloadLiquidPhaseModel,
    PropertyPackageDownloadMethod, PropertyPackageDownloadVaporPhaseModel,
    parse_property_package_download_json, persist_downloaded_package_response_to_cache,
};
pub use property_package_download_client::{
    HttpPropertyPackageDownloadFetcher, PropertyPackageDownloadFetchError,
    PropertyPackageDownloadFetchErrorKind, PropertyPackageDownloadFetcher,
    PropertyPackageDownloadHttpRequest, PropertyPackageDownloadHttpResponse,
    PropertyPackageDownloadHttpTransport, PropertyPackageDownloadHttpTransportError,
    PropertyPackageDownloadHttpTransportErrorKind, PropertyPackageDownloadResponse,
    PropertyPackageDownloadRetryPolicy, ReqwestPropertyPackageDownloadHttpTransport,
    ReqwestPropertyPackageDownloadHttpTransportOptions, download_property_package_to_cache,
    download_property_package_to_cache_with_retry_policy,
};
pub use solver_bridge::{
    StudioSolveRequest, next_solver_snapshot_sequence, solve_workspace_from_auth_cache,
    solve_workspace_with_property_package,
};
pub use workspace_control::{
    WorkspaceControlAction, WorkspaceControlActionOutcome, WorkspaceControlState,
    dispatch_workspace_control_action_with_auth_cache, snapshot_workspace_control_state,
};
pub use workspace_run_command::{
    WorkspaceRunCommand, WorkspaceRunDispatchResult, WorkspaceRunPackageSelection,
    dispatch_workspace_run_from_auth_cache, resolve_workspace_run_package_id,
};
pub use workspace_solve_service::{
    WorkspaceSolveDispatch, WorkspaceSolveService, WorkspaceSolveSkipReason, WorkspaceSolveTrigger,
    build_workspace_solve_request,
};
