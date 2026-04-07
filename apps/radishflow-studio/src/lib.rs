mod app_facade;
mod auth_cache_sync;
mod bootstrap;
mod control_plane_client;
mod control_plane_sync;
mod entitlement_control;
mod entitlement_panel_driver;
mod entitlement_preflight;
mod entitlement_session_driver;
mod entitlement_session_host;
mod entitlement_session_host_presentation;
mod entitlement_session_host_runtime;
mod property_package_download;
mod property_package_download_client;
mod run_panel_driver;
mod solver_bridge;
mod studio_app_host;
mod studio_gui_canvas_presentation;
mod studio_gui_canvas_widget;
mod studio_gui_command_registry;
mod studio_gui_driver;
mod studio_gui_host;
mod studio_gui_layout_store;
mod studio_gui_snapshot;
mod studio_gui_window_layout;
mod studio_gui_window_model;
mod studio_gui_shortcut_router;
mod studio_gui_timer_host;
mod studio_local_rules;
mod studio_runtime;
mod studio_window_host;
mod studio_window_host_manager;
mod studio_window_session;
mod studio_window_timer_driver;
mod workspace_control;
mod workspace_run_command;
mod workspace_solve_service;

pub use app_facade::{
    StudioAppAuthCacheContext, StudioAppCommand, StudioAppCommandOutcome,
    StudioAppExecutionBoundary, StudioAppExecutionLane, StudioAppFacade,
    StudioAppMutableAuthCacheContext, StudioAppResultDispatch, StudioWorkspaceModeDispatch,
    StudioWorkspaceRunBlocked, StudioWorkspaceRunBlockedReason, StudioWorkspaceRunDispatch,
    StudioWorkspaceRunFailed, StudioWorkspaceRunFailedReason, StudioWorkspaceRunOutcome,
};
pub use auth_cache_sync::{
    apply_offline_refresh_to_auth_cache, build_auth_cache_index, build_offline_refresh_request,
    persist_downloaded_package_to_cache, persist_offline_refresh_manifests_to_cache,
    record_downloaded_package, sync_auth_cache_index,
};
pub use bootstrap::{
    StudioBootstrapConfig, StudioBootstrapDispatch, StudioBootstrapEntitlementSessionEvent,
    StudioBootstrapReport, StudioBootstrapTrigger, run_studio_bootstrap,
};
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
pub use entitlement_control::{
    StudioEntitlementAction, StudioEntitlementActionOutcome, StudioEntitlementFailure,
    StudioEntitlementFailureReason, StudioEntitlementOutcome,
    refresh_offline_lease_with_control_plane, sync_entitlement_with_control_plane,
};
pub use entitlement_panel_driver::{
    EntitlementPanelDriverOutcome, EntitlementPanelDriverState,
    EntitlementPanelWidgetDispatchOutcome, dispatch_entitlement_panel_intent_with_control_plane,
    dispatch_entitlement_panel_primary_action_with_control_plane,
    dispatch_entitlement_panel_widget_action_with_control_plane,
    dispatch_entitlement_panel_widget_event_with_control_plane,
    map_entitlement_intent_to_app_command, snapshot_entitlement_panel_driver_state,
};
pub use entitlement_preflight::{
    EntitlementPreflightAction, EntitlementPreflightDecision, EntitlementPreflightOutcome,
    EntitlementPreflightPolicy, EntitlementSessionBackoff, EntitlementSessionPolicy,
    EntitlementSessionRuntime, EntitlementSessionSchedule, EntitlementSessionState,
    EntitlementSessionTickOutcome, decide_entitlement_preflight_action,
    dispatch_entitlement_preflight_with_control_plane,
    dispatch_entitlement_session_tick_with_control_plane, record_entitlement_session_dispatch,
    record_entitlement_session_outcome, snapshot_entitlement_session_schedule,
};
pub use entitlement_session_driver::{
    EntitlementSessionDriverState, EntitlementSessionEvent, EntitlementSessionEventDriverOutcome,
    EntitlementSessionEventOutcome, EntitlementSessionPanelDriverOutcome,
    EntitlementSessionTickDriverOutcome, dispatch_entitlement_session_event_with_control_plane,
    dispatch_entitlement_session_panel_primary_action_with_control_plane,
    dispatch_entitlement_session_panel_widget_action_with_control_plane,
    dispatch_entitlement_session_panel_widget_event_with_control_plane,
    dispatch_entitlement_session_tick_driver_with_control_plane,
    snapshot_entitlement_session_driver_state,
};
pub use entitlement_session_host::{
    EntitlementSessionHostContext, EntitlementSessionHostDispatch, EntitlementSessionHostOutcome,
    EntitlementSessionHostSnapshot, EntitlementSessionHostState, EntitlementSessionHostTrigger,
    EntitlementSessionLifecycleEvent, EntitlementSessionTimerArm, EntitlementSessionTimerCommand,
    EntitlementSessionTimerReason,
    dispatch_entitlement_session_host_trigger_with_context_and_control_plane,
    dispatch_entitlement_session_host_trigger_with_control_plane,
    dispatch_entitlement_session_lifecycle_event_with_context_and_control_plane,
    dispatch_entitlement_session_lifecycle_event_with_control_plane,
    plan_entitlement_session_timer_command, snapshot_entitlement_session_host,
    snapshot_entitlement_session_host_state, snapshot_entitlement_session_host_with_context,
    snapshot_entitlement_session_panel_driver_state_with_host_notice,
};
pub use entitlement_session_host_presentation::{
    EntitlementSessionHostPresentation, EntitlementSessionHostTextView,
};
pub use entitlement_session_host_runtime::{
    EntitlementSessionHostRuntime, EntitlementSessionHostRuntimeDispatchOutcome,
    EntitlementSessionHostRuntimeOutput, EntitlementSessionHostTimerEffect,
};
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
pub use run_panel_driver::{
    RunPanelDriverOutcome, RunPanelDriverState, RunPanelRecoveryOutcome,
    apply_run_panel_recovery_action, dispatch_run_panel_primary_action_with_auth_cache,
    dispatch_run_panel_widget_action_with_auth_cache, snapshot_run_panel_driver_state,
};
pub use solver_bridge::{
    StudioSolveRequest, next_solver_snapshot_sequence, solve_workspace_from_auth_cache,
    solve_workspace_with_property_package,
};
pub use studio_app_host::{
    StudioAppHost, StudioAppHostChangeSet, StudioAppHostCloseEffects,
    StudioAppHostCloseWindowResult, StudioAppHostCommand, StudioAppHostCommandOutcome,
    StudioAppHostController, StudioAppHostDispatchEffects, StudioAppHostEntitlementTimerEffect,
    StudioAppHostEntitlementTimerState, StudioAppHostEntitlementTimerStateChange,
    StudioAppHostGlobalEventResult, StudioAppHostOpenWindowResult, StudioAppHostOutput,
    StudioAppHostProjection, StudioAppHostSnapshot, StudioAppHostState, StudioAppHostStore,
    StudioAppHostTimerSlotChange, StudioAppHostUiAction, StudioAppHostUiActionAvailability,
    StudioAppHostUiActionDisabledReason, StudioAppHostUiActionModel, StudioAppHostUiActionState,
    StudioAppHostUiCommandDispatchResult, StudioAppHostUiCommandGroup, StudioAppHostUiCommandModel,
    StudioAppHostWindowChange, StudioAppHostWindowDispatchResult,
    StudioAppHostWindowSelectionChange, StudioAppHostWindowSnapshot, StudioAppHostWindowState,
};
pub use studio_gui_canvas_presentation::{
    StudioGuiCanvasPresentation, StudioGuiCanvasSuggestionViewModel, StudioGuiCanvasTextView,
    StudioGuiCanvasViewModel,
};
pub use studio_gui_canvas_widget::{
    StudioGuiCanvasActionId, StudioGuiCanvasRenderableAction, StudioGuiCanvasWidgetEvent,
    StudioGuiCanvasWidgetModel,
};
pub use studio_gui_command_registry::{
    StudioGuiCommandEntry, StudioGuiCommandRegistry, StudioGuiCommandSection, StudioGuiShortcut,
    StudioGuiShortcutKey, StudioGuiShortcutModifier,
};
pub use studio_gui_driver::{
    StudioGuiDriver, StudioGuiDriverDispatch, StudioGuiDriverOutcome, StudioGuiEvent,
};
pub use studio_gui_host::{
    StudioGuiCanvasInteractionAction, StudioGuiCanvasState, StudioGuiHost,
    StudioGuiHostCanvasInteractionResult, StudioGuiHostCanvasSuggestionResult,
    StudioGuiHostCloseWindowResult, StudioGuiHostCommand, StudioGuiHostCommandOutcome,
    StudioGuiHostDispatch, StudioGuiHostGlobalEventDispatch, StudioGuiHostLifecycleDispatch,
    StudioGuiHostLifecycleEvent, StudioGuiHostUiCommandDispatchResult,
    StudioGuiHostWindowLayoutUpdateResult, StudioGuiHostWindowOpened,
};
pub use studio_gui_snapshot::{StudioGuiRuntimeSnapshot, StudioGuiSnapshot};
pub use studio_gui_window_layout::{
    StudioGuiWindowAreaId, StudioGuiWindowDockRegion, StudioGuiWindowLayoutModel,
    StudioGuiWindowLayoutScope, StudioGuiWindowLayoutScopeKind, StudioGuiWindowLayoutState,
    StudioGuiWindowDockPlacement, StudioGuiWindowLayoutMutation,
    StudioGuiWindowLayoutPersistenceState, StudioGuiWindowPanelLayout,
    StudioGuiWindowPanelLayoutState, StudioGuiWindowRegionWeight,
    StudioGuiWindowStackGroupLayout, StudioGuiWindowStackGroupState,
    StudioGuiWindowStackTabLayout, StudioGuiWindowTitlebarModel,
};
pub use studio_gui_window_model::{
    StudioGuiWindowCanvasAreaModel, StudioGuiWindowCommandAreaModel, StudioGuiWindowHeaderModel,
    StudioGuiWindowModel, StudioGuiWindowRuntimeAreaModel,
};
pub use studio_gui_shortcut_router::{
    StudioGuiFocusContext, StudioGuiShortcutIgnoreReason, StudioGuiShortcutRoute, route_shortcut,
};
pub use studio_gui_timer_host::{StudioGuiNativeTimerEffects, StudioGuiNativeTimerOperation};
pub use studio_runtime::{
    StudioRuntime, StudioRuntimeConfig, StudioRuntimeDispatch, StudioRuntimeEffect,
    StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
    StudioRuntimeEntitlementSessionEvent, StudioRuntimeHostAckResult, StudioRuntimeHostAckStatus,
    StudioRuntimeHostEffect, StudioRuntimeHostEffectId, StudioRuntimeHostFollowUp,
    StudioRuntimeOutput, StudioRuntimeReport, StudioRuntimeTimerHandleSlot,
    StudioRuntimeTimerHostCommand, StudioRuntimeTimerHostState, StudioRuntimeTimerHostTransition,
    StudioRuntimeTrigger,
};
pub use studio_window_host::{
    StudioRuntimeHostPort, StudioRuntimeHostPortOutput, StudioWindowHostEvent, StudioWindowHostId,
    StudioWindowHostLifecycleEvent, StudioWindowHostRegistration, StudioWindowHostRetirement,
    StudioWindowHostRole, StudioWindowHostShutdown, StudioWindowHostState,
    StudioWindowHostTimerDriverCommand,
};
pub use studio_window_host_manager::{
    StudioAppWindowHostClose, StudioAppWindowHostCommand, StudioAppWindowHostCommandOutcome,
    StudioAppWindowHostDispatch, StudioAppWindowHostGlobalEvent, StudioAppWindowHostManager,
    StudioAppWindowHostUiAction, StudioAppWindowHostUiActionAvailability,
    StudioAppWindowHostUiActionDisabledReason, StudioAppWindowHostUiActionState,
};
pub use studio_window_session::{
    StudioWindowSession, StudioWindowSessionDispatch, StudioWindowSessionShutdown,
};
pub use studio_window_timer_driver::{
    StudioWindowNativeTimerBinding, StudioWindowNativeTimerHandleId,
    StudioWindowPendingTimerBinding, StudioWindowTimerDriverAckResult,
    StudioWindowTimerDriverAckStatus, StudioWindowTimerDriverState,
    StudioWindowTimerDriverTransition,
};
pub use workspace_control::{
    RunPanelWidgetDispatchOutcome, WorkspaceControlAction, WorkspaceControlActionOutcome,
    WorkspaceControlState, dispatch_run_panel_intent_with_auth_cache,
    dispatch_run_panel_widget_event_with_auth_cache,
    dispatch_workspace_control_action_with_auth_cache,
    map_run_panel_intent_to_workspace_control_action,
    map_run_panel_package_selection_to_workspace_run_package_selection,
    map_workspace_control_state_to_run_panel_state, snapshot_workspace_control_state,
};
pub use workspace_run_command::{
    WorkspaceRunCommand, WorkspaceRunDispatchResult, WorkspaceRunPackageSelection,
    dispatch_workspace_run_from_auth_cache, resolve_workspace_run_package_id,
};
pub use workspace_solve_service::{
    WorkspaceSolveDispatch, WorkspaceSolveService, WorkspaceSolveSkipReason, WorkspaceSolveTrigger,
    build_workspace_solve_request,
};
