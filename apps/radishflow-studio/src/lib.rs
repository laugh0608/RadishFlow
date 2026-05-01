mod app_facade;
mod auth_cache_sync;
mod bootstrap;
mod control_plane_client;
mod control_plane_sync;
mod document_history_driver;
mod document_lifecycle_driver;
mod entitlement_control;
mod entitlement_panel_driver;
mod entitlement_preflight;
mod entitlement_session_driver;
mod entitlement_session_host;
mod entitlement_session_host_presentation;
mod entitlement_session_host_runtime;
mod inspector_draft_driver;
mod inspector_target_driver;
mod property_package_download;
mod property_package_download_client;
mod run_panel_driver;
mod solver_bridge;
mod studio_app_host;
mod studio_document_history_command;
mod studio_example_projects;
mod studio_gui_canvas_presentation;
mod studio_gui_canvas_widget;
mod studio_gui_command_registry;
mod studio_gui_driver;
mod studio_gui_host;
mod studio_gui_layout_store;
mod studio_gui_platform_host;
mod studio_gui_platform_timer_driver;
mod studio_gui_shortcut_router;
mod studio_gui_snapshot;
mod studio_gui_timer_host;
mod studio_gui_window_layout;
mod studio_gui_window_model;
mod studio_inspector_draft_command;
mod studio_inspector_target_command;
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
pub use document_history_driver::{
    DocumentHistoryOutcome, dispatch_document_history, dispatch_document_history_at,
};
pub use document_lifecycle_driver::{
    DocumentLifecycleOutcome, FILE_SAVE_AS_COMMAND_ID, FILE_SAVE_COMMAND_ID,
    StudioDocumentLifecycleAction, StudioDocumentLifecycleCommand, dispatch_document_lifecycle,
};
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
pub use inspector_draft_driver::{
    InspectorDraftBatchCommitOutcome, InspectorDraftCommitOutcome, InspectorDraftUpdateOutcome,
    commit_inspector_draft, commit_inspector_draft_at, commit_inspector_drafts,
    commit_inspector_drafts_at, update_inspector_draft,
};
pub use inspector_target_driver::{InspectorTargetFocusOutcome, focus_inspector_target};
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
pub use studio_document_history_command::{
    EDIT_REDO_COMMAND_ID, EDIT_UNDO_COMMAND_ID, StudioDocumentHistoryCommand,
    document_history_command_from_id,
};
pub use studio_example_projects::{StudioExampleProjectModel, studio_example_project_models};
pub use studio_gui_canvas_presentation::{
    StudioGuiCanvasFocusCalloutViewModel, StudioGuiCanvasObjectListItemViewModel,
    StudioGuiCanvasObjectListViewModel, StudioGuiCanvasPendingEditViewModel,
    StudioGuiCanvasPresentation, StudioGuiCanvasRunStatusViewModel,
    StudioGuiCanvasSelectionViewModel, StudioGuiCanvasStatusBadgeViewModel,
    StudioGuiCanvasStreamLineEndpointViewModel, StudioGuiCanvasStreamLineViewModel,
    StudioGuiCanvasSuggestionViewModel, StudioGuiCanvasTextView, StudioGuiCanvasUnitBlockViewModel,
    StudioGuiCanvasUnitPortViewModel, StudioGuiCanvasViewModel,
};
pub use studio_gui_canvas_widget::{
    StudioGuiCanvasActionId, StudioGuiCanvasRenderableAction, StudioGuiCanvasWidgetEvent,
    StudioGuiCanvasWidgetModel,
};
pub use studio_gui_command_registry::{
    StudioGuiCommandEntry, StudioGuiCommandGroup, StudioGuiCommandMenuCommandModel,
    StudioGuiCommandMenuNode, StudioGuiCommandPresentation, StudioGuiCommandRegistry,
    StudioGuiCommandSection, StudioGuiShortcut, StudioGuiShortcutKey, StudioGuiShortcutModifier,
};
pub use studio_gui_driver::{
    StudioGuiDriver, StudioGuiDriverDispatch, StudioGuiDriverOutcome, StudioGuiEvent,
};
pub use studio_gui_host::{
    StudioGuiCanvasDiagnosticState, StudioGuiCanvasInteractionAction, StudioGuiCanvasState,
    StudioGuiCanvasStreamEndpointState, StudioGuiCanvasStreamState, StudioGuiCanvasUnitPortState,
    StudioGuiCanvasUnitState, StudioGuiHost, StudioGuiHostCanvasInteractionResult,
    StudioGuiHostCanvasSuggestionResult, StudioGuiHostCloseWindowResult, StudioGuiHostCommand,
    StudioGuiHostCommandOutcome, StudioGuiHostDispatch, StudioGuiHostGlobalEventDispatch,
    StudioGuiHostLifecycleDispatch, StudioGuiHostLifecycleEvent,
    StudioGuiHostUiCommandDispatchResult, StudioGuiHostWindowDropPreviewClearResult,
    StudioGuiHostWindowDropTargetApplyResult, StudioGuiHostWindowDropTargetQueryResult,
    StudioGuiHostWindowLayoutUpdateResult, StudioGuiHostWindowOpened,
};
pub use studio_gui_platform_host::{
    StudioGuiPlatformAsyncRound, StudioGuiPlatformAsyncRoundAction,
    StudioGuiPlatformAsyncRoundInput, StudioGuiPlatformDispatch, StudioGuiPlatformDueTimerDrain,
    StudioGuiPlatformExecutedAsyncRound, StudioGuiPlatformExecutedAsyncRoundAction,
    StudioGuiPlatformExecutedDispatch, StudioGuiPlatformExecutedDueTimerDrain,
    StudioGuiPlatformExecutedNativeTimerCallbackBatch,
    StudioGuiPlatformExecutedNativeTimerCallbackOutcome, StudioGuiPlatformHost,
    StudioGuiPlatformNativeTimerCallbackBatch, StudioGuiPlatformNativeTimerCallbackOutcome,
    StudioGuiPlatformTimerExecutionOutcome, StudioGuiPlatformTimerExecutor,
    StudioGuiPlatformTimerExecutorResponse, StudioGuiPlatformTimerFollowUpCommand,
    StudioGuiPlatformTimerHostOutcome, StudioGuiPlatformTimerRequest,
    StudioGuiPlatformTimerStartFailedFeedback, StudioGuiPlatformTimerStartFailedFeedbackBatch,
    StudioGuiPlatformTimerStartFailedFeedbackEntry, StudioGuiPlatformTimerStartFailedOutcome,
    StudioGuiPlatformTimerStartedFeedback, StudioGuiPlatformTimerStartedFeedbackBatch,
    StudioGuiPlatformTimerStartedFeedbackEntry, StudioGuiPlatformTimerStartedOutcome,
};
pub use studio_gui_platform_timer_driver::{
    StudioGuiPlatformNativeTimerId, StudioGuiPlatformTimerBinding,
    StudioGuiPlatformTimerCallbackResolution, StudioGuiPlatformTimerCommand,
    StudioGuiPlatformTimerDriverState, StudioGuiPlatformTimerStartAckResult,
    StudioGuiPlatformTimerStartAckStatus, StudioGuiPlatformTimerStartFailureResult,
    StudioGuiPlatformTimerStartFailureStatus,
};
pub use studio_gui_shortcut_router::{
    StudioGuiFocusContext, StudioGuiShortcutIgnoreReason, StudioGuiShortcutRoute, route_shortcut,
};
pub use studio_gui_snapshot::{
    StudioGuiInspectorTargetDetailSnapshot, StudioGuiInspectorTargetFieldSnapshot,
    StudioGuiInspectorTargetFieldValidationSnapshot,
    StudioGuiInspectorTargetFieldValueKindSnapshot, StudioGuiInspectorTargetPortSnapshot,
    StudioGuiInspectorTargetSummaryRowSnapshot, StudioGuiRuntimeSnapshot, StudioGuiSnapshot,
    StudioGuiWorkspaceDocumentSnapshot,
};
pub use studio_gui_timer_host::{
    StudioGuiNativeTimerDueEvent, StudioGuiNativeTimerEffects, StudioGuiNativeTimerOperation,
    StudioGuiNativeTimerRuntime, StudioGuiNativeTimerSchedule,
};
pub use studio_gui_window_layout::{
    StudioGuiWindowAreaId, StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion,
    StudioGuiWindowDropTarget, StudioGuiWindowDropTargetKind, StudioGuiWindowDropTargetQuery,
    StudioGuiWindowLayoutModel, StudioGuiWindowLayoutMutation,
    StudioGuiWindowLayoutPersistenceState, StudioGuiWindowLayoutScope,
    StudioGuiWindowLayoutScopeKind, StudioGuiWindowLayoutState, StudioGuiWindowPanelDisplayMode,
    StudioGuiWindowPanelLayout, StudioGuiWindowPanelLayoutState, StudioGuiWindowRegionWeight,
    StudioGuiWindowStackGroupLayout, StudioGuiWindowStackGroupState, StudioGuiWindowStackTabLayout,
    StudioGuiWindowTitlebarModel,
};
pub use studio_gui_window_model::{
    StudioGuiWindowCanvasAreaModel, StudioGuiWindowCommandActionModel,
    StudioGuiWindowCommandAreaModel, StudioGuiWindowCommandListItemModel,
    StudioGuiWindowCommandListSectionModel, StudioGuiWindowCommandPaletteItemModel,
    StudioGuiWindowCompositionResultModel, StudioGuiWindowDiagnosticModel,
    StudioGuiWindowDropPreviewModel, StudioGuiWindowDropPreviewOverlayModel,
    StudioGuiWindowDropPreviewState, StudioGuiWindowFailureResultModel, StudioGuiWindowHeaderModel,
    StudioGuiWindowInspectorTargetDetailModel, StudioGuiWindowInspectorTargetFieldModel,
    StudioGuiWindowInspectorTargetModel, StudioGuiWindowInspectorTargetPortModel,
    StudioGuiWindowInspectorTargetSummaryRowModel, StudioGuiWindowModel,
    StudioGuiWindowPhaseResultModel, StudioGuiWindowResultInspectorComparisonModel,
    StudioGuiWindowResultInspectorComparisonRowModel,
    StudioGuiWindowResultInspectorCompositionComparisonRowModel,
    StudioGuiWindowResultInspectorModel, StudioGuiWindowResultInspectorStreamOptionModel,
    StudioGuiWindowRuntimeAreaModel, StudioGuiWindowSolveSnapshotModel,
    StudioGuiWindowSolveStepModel, StudioGuiWindowStreamResultModel,
    StudioGuiWindowStreamSummaryRowModel, StudioGuiWindowToolbarItemModel,
    StudioGuiWindowToolbarSectionModel,
};
pub use studio_inspector_draft_command::{
    StudioInspectorDraftBatchCommitCommand, StudioInspectorDraftCommitCommand,
    StudioInspectorDraftUpdateCommand, inspector_draft_batch_commit_command_from_id,
    inspector_draft_batch_commit_command_id, inspector_draft_commit_command_from_id,
    inspector_draft_commit_command_id, inspector_draft_update_command_from_id,
    inspector_draft_update_command_id,
};
pub use studio_inspector_target_command::{
    inspector_target_command_id, inspector_target_from_command_id,
};
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
    StudioAppWindowHostCanvasInteractionResult, StudioAppWindowHostClose,
    StudioAppWindowHostCommand, StudioAppWindowHostCommandOutcome, StudioAppWindowHostDispatch,
    StudioAppWindowHostGlobalEvent, StudioAppWindowHostManager, StudioAppWindowHostOpenWindow,
    StudioAppWindowHostUiAction, StudioAppWindowHostUiActionAvailability,
    StudioAppWindowHostUiActionDisabledReason, StudioAppWindowHostUiActionState,
    StudioCanvasInteractionAction,
};
pub use studio_window_session::{
    StudioWindowSession, StudioWindowSessionDispatch, StudioWindowSessionOpenWindow,
    StudioWindowSessionShutdown,
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
