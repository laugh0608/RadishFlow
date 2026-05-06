mod auth;
mod canvas_interaction;
mod commands;
mod diagnostics;
mod entitlement_panel;
mod entitlement_panel_presenter;
mod entitlement_panel_text;
mod entitlement_panel_view;
mod entitlement_panel_widget;
mod ids;
mod run;
mod run_panel;
mod run_panel_presenter;
mod run_panel_text;
mod run_panel_view;
mod run_panel_widget;
mod state;

pub use auth::{
    AuditUsageAck, AuditUsageRequest, AuthSessionState, AuthSessionStatus, AuthenticatedUser,
    EntitlementNotice, EntitlementNoticeLevel, EntitlementSnapshot, EntitlementState,
    EntitlementStatus, OfflineLeaseRefreshRequest, OfflineLeaseRefreshResponse,
    PropertyPackageClassification, PropertyPackageLeaseGrant, PropertyPackageLeaseRequest,
    PropertyPackageManifest, PropertyPackageManifestList, PropertyPackageSource,
    PropertyPackageUsageEvent, PropertyPackageUsageEventKind, SecureCredentialHandle, TokenLease,
};
pub use canvas_interaction::{
    CanvasEditIntent, CanvasInteractionState, CanvasSuggestedMaterialConnection,
    CanvasSuggestedStreamBinding, CanvasSuggestion, CanvasSuggestionAcceptance, CanvasViewMode,
    GhostElement, GhostElementKind, StreamAnimationMode, StreamVisualKind, StreamVisualState,
    SuggestionSource, SuggestionStatus,
};
pub use commands::{
    CanvasPoint, CommandHistory, CommandHistoryEntry, CommandValue, DocumentCommand,
    StreamSpecificationValue,
};
pub use diagnostics::{DiagnosticSeverity, DiagnosticSnapshot, DiagnosticSummary};
pub use entitlement_panel::{
    EntitlementActionId, EntitlementActionModel, EntitlementCommandModel, EntitlementIntent,
    EntitlementPanelState,
};
pub use entitlement_panel_presenter::EntitlementPanelPresentation;
pub use entitlement_panel_text::EntitlementPanelTextView;
pub use entitlement_panel_view::{
    EntitlementActionProminence, EntitlementPanelViewModel, EntitlementRenderableAction,
};
pub use entitlement_panel_widget::{EntitlementPanelWidgetEvent, EntitlementPanelWidgetModel};
pub use ids::{CanvasSuggestionId, DocumentId, SolveSnapshotId};
pub use run::{
    PhaseStateSnapshot, RunStatus, SimulationMode, SolvePendingReason, SolveSessionState,
    SolveSnapshot, StepSnapshot, StreamStateSnapshot, UnitExecutionSnapshot,
};
pub use run_panel::{
    RunPanelActionId, RunPanelActionModel, RunPanelCommandModel, RunPanelIntent, RunPanelNotice,
    RunPanelNoticeLevel, RunPanelPackageSelection, RunPanelRecoveryAction,
    RunPanelRecoveryActionKind, RunPanelRecoveryMutation, RunPanelState, run_panel_failure_notice,
    run_panel_failure_recovery_action_for_diagnostic_code,
    run_panel_failure_title_for_diagnostic_code,
};
pub use run_panel_presenter::RunPanelPresentation;
pub use run_panel_text::RunPanelTextView;
pub use run_panel_view::{RunPanelActionProminence, RunPanelRenderableAction, RunPanelViewModel};
pub use run_panel_widget::{RunPanelRecoveryWidgetEvent, RunPanelWidgetEvent, RunPanelWidgetModel};
pub use state::{
    AppLogEntry, AppLogFeed, AppLogLevel, AppState, AppTheme, CanvasEditCommitResult, DateTimeUtc,
    DocumentHistoryApplyResult, DocumentHistoryDirection, DocumentMetadata, DraftValidationState,
    DraftValue, FieldDraft, FlowsheetDocument, InspectorDraftState, InspectorTarget, LocaleCode,
    PanelLayoutPreferences, SelectionState, StreamInspectorDraftBatchCommitResult,
    StreamInspectorDraftBatchDiscardResult, StreamInspectorDraftCommitResult,
    StreamInspectorDraftDiscardResult, StreamInspectorDraftField, StreamInspectorDraftUpdateResult,
    UiPanelsState, UserPreferences, WorkspaceState, latest_snapshot, latest_snapshot_id,
    stream_inspector_draft_key, stream_inspector_draft_key_parts,
};

#[cfg(test)]
mod tests;
