mod actions;
mod input_commands;
mod input_discard;
pub use input_discard::{InputDiscardScope, InputEditCheckpoint};
mod numeric_edit;
mod stream_inspector;
pub use numeric_edit::*;
use stream_inspector::apply_stream_specification_value;
mod project_presentation;
pub use project_presentation::{
    DisplayUnitViewId, ProjectPresentationCommand, ProjectPresentationState, ProjectSaveState,
    ViewDisplayUnits,
};
mod unit_edit;
pub use unit_edit::UnitCreateResult;
pub(crate) mod unit_inspector;
use actions::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::time::SystemTime;
pub use unit_inspector::{
    UnitInspectorDraftCommitResult, UnitInspectorDraftDiscardResult, UnitInspectorDraftField,
    UnitInspectorDraftUpdateResult, unit_inspector_draft_key, unit_inspector_draft_key_parts,
    unit_inspector_parameter_is_explicit, unit_inspector_parameter_value,
};

use rf_model::{Component, Flowsheet, MaterialStreamState, UnitNode, UnitPort};
use rf_types::{ComponentId, PortDirection, PortKind, RfError, RfResult, StreamId, UnitId};
use rf_unitops::{
    BuiltinUnitKind, UnitOperationSpec, builtin_unit_spec, builtin_unit_spec_by_name,
};

use crate::auth::{
    AuthSessionState, AuthenticatedUser, EntitlementSnapshot, EntitlementState,
    PropertyPackageManifest, TokenLease,
};
use crate::canvas_interaction::{
    CanvasEditIntent, CanvasInteractionState, CanvasSuggestedMaterialConnection,
    CanvasSuggestedStreamBinding, CanvasSuggestion, CanvasSuggestionAcceptance, CanvasViewMode,
    SuggestionSource, SuggestionStatus,
};
use crate::commands::{
    CanvasPoint, CommandHistory, CommandHistoryEntry, CommandValue, DocumentCommand,
    StreamPortBinding, StreamSpecificationValue,
};
use crate::diagnostics::DiagnosticSummary;
use crate::ids::{CanvasSuggestionId, DocumentId, SolveSnapshotId};
use crate::run::{RunStatus, SimulationMode, SolvePendingReason, SolveSessionState, SolveSnapshot};
use crate::run_panel::{RunPanelRecoveryAction, RunPanelRecoveryMutation, RunPanelState};

pub type DateTimeUtc = SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppTheme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocaleCode(String);

impl LocaleCode {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl Default for LocaleCode {
    fn default() -> Self {
        Self::new("zh-CN")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelLayoutPreferences {
    pub inspector_open: bool,
    pub results_open: bool,
    pub log_open: bool,
}

impl Default for PanelLayoutPreferences {
    fn default() -> Self {
        Self {
            inspector_open: true,
            results_open: true,
            log_open: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserPreferences {
    pub theme: AppTheme,
    pub locale: LocaleCode,
    pub recent_project_paths: Vec<PathBuf>,
    pub panel_defaults: PanelLayoutPreferences,
    pub snapshot_history_limit: usize,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: AppTheme::System,
            locale: LocaleCode::default(),
            recent_project_paths: Vec::new(),
            panel_defaults: PanelLayoutPreferences::default(),
            snapshot_history_limit: 8,
        }
    }
}

impl UserPreferences {
    pub fn effective_snapshot_history_limit(&self) -> usize {
        self.snapshot_history_limit.max(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMetadata {
    pub document_id: DocumentId,
    pub title: String,
    pub schema_version: u32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl DocumentMetadata {
    pub fn new(
        document_id: impl Into<DocumentId>,
        title: impl Into<String>,
        created_at: DateTimeUtc,
    ) -> Self {
        Self {
            document_id: document_id.into(),
            title: title.into(),
            schema_version: 1,
            created_at,
            updated_at: created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowsheetDocument {
    pub revision: u64,
    pub flowsheet: Flowsheet,
    pub metadata: DocumentMetadata,
}

impl FlowsheetDocument {
    pub fn new(flowsheet: Flowsheet, metadata: DocumentMetadata) -> Self {
        Self {
            revision: 0,
            flowsheet,
            metadata,
        }
    }

    pub fn replace_flowsheet(&mut self, flowsheet: Flowsheet, changed_at: DateTimeUtc) -> u64 {
        self.revision += 1;
        self.flowsheet = flowsheet;
        self.metadata.updated_at = changed_at;
        self.revision
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectionState {
    pub selected_units: BTreeSet<UnitId>,
    pub selected_streams: BTreeSet<StreamId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiPanelsState {
    pub inspector_open: bool,
    pub results_open: bool,
    pub log_open: bool,
}

impl Default for UiPanelsState {
    fn default() -> Self {
        Self {
            inspector_open: true,
            results_open: true,
            log_open: true,
        }
    }
}

impl UiPanelsState {
    pub fn from_preferences(preferences: &PanelLayoutPreferences) -> Self {
        Self {
            inspector_open: preferences.inspector_open,
            results_open: preferences.results_open,
            log_open: preferences.log_open,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftValidationState {
    Unknown,
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDraft<T> {
    pub original: T,
    pub current: T,
    pub is_dirty: bool,
    pub validation: DraftValidationState,
}

impl<T: Clone + PartialEq> FieldDraft<T> {
    pub fn new(original: T) -> Self {
        Self {
            current: original.clone(),
            original,
            is_dirty: false,
            validation: DraftValidationState::Unknown,
        }
    }

    pub fn update(&mut self, current: T) {
        self.is_dirty = self.original != current;
        self.current = current;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DraftValue {
    Text(FieldDraft<String>),
    Number(FieldDraft<String>),
    Numeric(Box<NumericEditSession>),
    Choice(FieldDraft<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InspectorTarget {
    Unit(UnitId),
    Stream(StreamId),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct InspectorDraftState {
    pub active_target: Option<InspectorTarget>,
    pub fields: BTreeMap<String, DraftValue>,
    next_numeric_generation: u64,
}

impl InspectorDraftState {
    pub fn pending_count(&self) -> usize {
        self.fields
            .values()
            .filter(|draft| match draft {
                DraftValue::Numeric(session) => session.is_pending(),
                _ => true,
            })
            .count()
    }

    pub fn clear(&mut self) {
        self.active_target = None;
        self.fields.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamInspectorDraftField {
    Name,
    TemperatureK,
    PressurePa,
    TotalMolarFlowMolS,
    OverallMoleFraction(ComponentId),
}

impl StreamInspectorDraftField {
    pub fn key_segment(&self) -> String {
        match self {
            Self::Name => "name".to_string(),
            Self::TemperatureK => "temperature_k".to_string(),
            Self::PressurePa => "pressure_pa".to_string(),
            Self::TotalMolarFlowMolS => "total_molar_flow_mol_s".to_string(),
            Self::OverallMoleFraction(component_id) => {
                format!("overall_mole_fraction:{}", component_id.as_str())
            }
        }
    }

    pub fn command_field(&self) -> String {
        self.key_segment()
    }

    pub fn from_static_key_segment(value: &str) -> Option<Self> {
        match value {
            "name" => Some(Self::Name),
            "temperature_k" => Some(Self::TemperatureK),
            "pressure_pa" => Some(Self::PressurePa),
            "total_molar_flow_mol_s" => Some(Self::TotalMolarFlowMolS),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamInspectorDraftUpdateResult {
    pub key: String,
    pub active_target: InspectorTarget,
    pub is_dirty: bool,
    pub validation: DraftValidationState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamInspectorDraftCommitResult {
    pub key: String,
    pub active_target: InspectorTarget,
    pub command: DocumentCommand,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamInspectorDraftDiscardResult {
    pub key: String,
    pub active_target: InspectorTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamInspectorDraftBatchCommitResult {
    pub keys: Vec<String>,
    pub active_target: InspectorTarget,
    pub command: DocumentCommand,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamInspectorDraftBatchDiscardResult {
    pub keys: Vec<String>,
    pub active_target: InspectorTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamInspectorCompositionComponentAddResult {
    pub key: String,
    pub active_target: InspectorTarget,
    pub component_id: ComponentId,
    pub command: DocumentCommand,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamInspectorCompositionComponentRemoveResult {
    pub key: String,
    pub active_target: InspectorTarget,
    pub component_id: ComponentId,
    pub command: DocumentCommand,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamConnectionEditResult {
    pub stream_id: StreamId,
    pub disconnected_ports: Vec<StreamPortBinding>,
    pub command: DocumentCommand,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamReconnectEditResult {
    pub stream_id: StreamId,
    pub source_port: StreamPortBinding,
    pub sink_port: StreamPortBinding,
    pub command: DocumentCommand,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasEditCommitResult {
    pub intent: CanvasEditIntent,
    pub command: DocumentCommand,
    pub revision: u64,
    pub unit_id: UnitId,
    pub position: CanvasPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentHistoryDirection {
    Undo,
    Redo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DocumentHistoryApplyResult {
    pub direction: DocumentHistoryDirection,
    pub command: DocumentCommand,
    pub revision: u64,
}

pub fn stream_inspector_draft_key(
    stream_id: &StreamId,
    field: &StreamInspectorDraftField,
) -> String {
    format!("stream:{}:{}", stream_id.as_str(), field.key_segment())
}

pub fn stream_inspector_draft_key_parts(
    key: &str,
) -> Option<(StreamId, StreamInspectorDraftField)> {
    let rest = key.strip_prefix("stream:")?;
    if let Some((stream_id, component_id)) = rest.split_once(":overall_mole_fraction:") {
        if stream_id.is_empty() || component_id.is_empty() {
            return None;
        }
        return Some((
            StreamId::new(stream_id),
            StreamInspectorDraftField::OverallMoleFraction(ComponentId::new(component_id)),
        ));
    }
    let (stream_id, field) = rest.rsplit_once(':')?;
    if stream_id.is_empty() {
        return None;
    }
    Some((
        StreamId::new(stream_id),
        StreamInspectorDraftField::from_static_key_segment(field)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AppLogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppLogEntry {
    pub level: AppLogLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppLogFeed {
    pub entries: VecDeque<AppLogEntry>,
}

impl AppLogFeed {
    pub fn push(&mut self, level: AppLogLevel, message: impl Into<String>) {
        self.entries.push_back(AppLogEntry {
            level,
            message: message.into(),
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceState {
    // Retain identities across deletion and history branching for this document session.
    allocated_unit_ids: BTreeSet<UnitId>,
    pub document: FlowsheetDocument,
    pub document_path: Option<PathBuf>,
    pub project_presentation: ProjectPresentationState,
    pub last_saved_revision: Option<u64>,
    pub canvas_interaction: CanvasInteractionState,
    pub selection: SelectionState,
    pub panels: UiPanelsState,
    pub drafts: InspectorDraftState,
    pub command_history: CommandHistory,
    pub solve_session: SolveSessionState,
    pub snapshot_history: VecDeque<SolveSnapshot>,
    pub run_panel: RunPanelState,
}

impl WorkspaceState {
    pub fn new(document: FlowsheetDocument, panel_defaults: &PanelLayoutPreferences) -> Self {
        let revision = document.revision;
        let solve_session = SolveSessionState::new(revision);
        let run_panel = RunPanelState::from_runtime(&solve_session, None, None);

        Self {
            allocated_unit_ids: document.flowsheet.units.keys().cloned().collect(),
            document,
            document_path: None,
            project_presentation: ProjectPresentationState::default(),
            last_saved_revision: None,
            canvas_interaction: CanvasInteractionState::default(),
            selection: SelectionState::default(),
            panels: UiPanelsState::from_preferences(panel_defaults),
            drafts: InspectorDraftState::default(),
            command_history: CommandHistory::new(),
            solve_session,
            snapshot_history: VecDeque::new(),
            run_panel,
        }
    }

    pub fn commit_document_change(
        &mut self,
        command: DocumentCommand,
        next_flowsheet: Flowsheet,
        changed_at: DateTimeUtc,
    ) -> u64 {
        let before = self.document.flowsheet.clone();
        let after = next_flowsheet.clone();
        self.allocated_unit_ids.extend(before.units.keys().cloned());
        self.allocated_unit_ids.extend(after.units.keys().cloned());
        let revision = self.document.replace_flowsheet(next_flowsheet, changed_at);
        self.command_history
            .record(CommandHistoryEntry::with_snapshots(
                revision, command, before, after,
            ));
        self.canvas_interaction.invalidate_all();
        self.solve_session.mark_document_revision_advanced(revision);
        self.drafts
            .fields
            .retain(|_, draft| matches!(draft, DraftValue::Numeric(_)));
        self.refresh_numeric_edits();
        revision
    }

    fn commit_inspector_document_change(
        &mut self,
        command: DocumentCommand,
        next_flowsheet: Flowsheet,
        changed_at: DateTimeUtc,
    ) -> u64 {
        let before = self.document.flowsheet.clone();
        let after = next_flowsheet.clone();
        let revision = self.document.replace_flowsheet(next_flowsheet, changed_at);
        self.command_history
            .record(CommandHistoryEntry::with_snapshots(
                revision, command, before, after,
            ));
        self.canvas_interaction.invalidate_all();
        self.solve_session.mark_document_revision_advanced(revision);
        self.refresh_numeric_edits();
        revision
    }

    fn apply_history_flowsheet(&mut self, flowsheet: Flowsheet, changed_at: DateTimeUtc) -> u64 {
        let revision = self.document.replace_flowsheet(flowsheet, changed_at);
        self.canvas_interaction.invalidate_all();
        self.solve_session.mark_document_revision_advanced(revision);
        self.prune_focus_against_document();
        self.drafts
            .fields
            .retain(|_, draft| matches!(draft, DraftValue::Numeric(_)));
        self.refresh_numeric_edits();
        revision
    }

    fn prune_focus_against_document(&mut self) {
        self.selection
            .selected_units
            .retain(|unit_id| self.document.flowsheet.units.contains_key(unit_id));
        self.selection
            .selected_streams
            .retain(|stream_id| self.document.flowsheet.streams.contains_key(stream_id));

        let active_target_exists = match self.drafts.active_target.as_ref() {
            Some(InspectorTarget::Unit(unit_id)) => {
                self.document.flowsheet.units.contains_key(unit_id)
            }
            Some(InspectorTarget::Stream(stream_id)) => {
                self.document.flowsheet.streams.contains_key(stream_id)
            }
            None => true,
        };
        if !active_target_exists {
            self.drafts.active_target = None;
        }
    }

    pub fn mark_saved(&mut self, path: impl Into<PathBuf>) {
        self.document_path = Some(path.into());
        self.last_saved_revision = Some(self.document.revision);
    }

    pub fn apply_snapshot_history_limit(&mut self, limit: usize) {
        let effective_limit = limit.max(1);
        while self.snapshot_history.len() > effective_limit {
            self.snapshot_history.pop_front();
        }
    }

    pub fn store_snapshot(&mut self, snapshot: SolveSnapshot, limit: usize) {
        self.snapshot_history.push_back(snapshot.clone());
        self.apply_snapshot_history_limit(limit);
        self.solve_session.complete_with_snapshot(&snapshot);
    }

    pub fn clear_results(&mut self) {
        self.snapshot_history.clear();
        self.solve_session.latest_snapshot = None;
        self.solve_session.pending_reason = Some(SolvePendingReason::SnapshotMissing);
        self.run_panel = RunPanelState::from_runtime(&self.solve_session, None, None);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub workspace: WorkspaceState,
    pub auth_session: AuthSessionState,
    pub entitlement: EntitlementState,
    pub preferences: UserPreferences,
    pub log_feed: AppLogFeed,
}

impl AppState {
    pub fn new(document: FlowsheetDocument) -> Self {
        let preferences = UserPreferences::default();
        let workspace = WorkspaceState::new(document, &preferences.panel_defaults);
        let mut app_state = Self {
            workspace,
            auth_session: AuthSessionState::default(),
            entitlement: EntitlementState::default(),
            preferences,
            log_feed: AppLogFeed::default(),
        };
        app_state.refresh_run_panel_state();
        app_state
    }

    pub fn commit_document_change(
        &mut self,
        command: DocumentCommand,
        next_flowsheet: Flowsheet,
        changed_at: DateTimeUtc,
    ) -> u64 {
        let revision = self
            .workspace
            .commit_document_change(command, next_flowsheet, changed_at);
        self.refresh_run_panel_state();
        revision
    }

    pub fn undo_document_command(
        &mut self,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<DocumentHistoryApplyResult>> {
        if self
            .workspace
            .drafts
            .fields
            .values()
            .any(|draft| matches!(draft, DraftValue::Numeric(session) if session.is_pending()))
        {
            return Err(RfError::invalid_input(
                "finish or discard numeric edits before applying engineering history",
            ));
        }
        let Some(entry) = self.workspace.command_history.undo_entry().cloned() else {
            return Ok(None);
        };
        let Some(before) = entry.before.clone() else {
            return Err(RfError::invalid_input(format!(
                "document command history entry at revision {} cannot be undone because it has no before snapshot",
                entry.revision
            )));
        };

        let revision = self.workspace.apply_history_flowsheet(before, changed_at);
        self.workspace.command_history.step_undo();
        self.refresh_run_panel_state();

        Ok(Some(DocumentHistoryApplyResult {
            direction: DocumentHistoryDirection::Undo,
            command: entry.command,
            revision,
        }))
    }

    pub fn redo_document_command(
        &mut self,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<DocumentHistoryApplyResult>> {
        if self
            .workspace
            .drafts
            .fields
            .values()
            .any(|draft| matches!(draft, DraftValue::Numeric(session) if session.is_pending()))
        {
            return Err(RfError::invalid_input(
                "finish or discard numeric edits before applying engineering history",
            ));
        }
        let Some(entry) = self.workspace.command_history.redo_entry().cloned() else {
            return Ok(None);
        };
        let Some(after) = entry.after.clone() else {
            return Err(RfError::invalid_input(format!(
                "document command history entry at revision {} cannot be redone because it has no after snapshot",
                entry.revision
            )));
        };

        let revision = self.workspace.apply_history_flowsheet(after, changed_at);
        self.workspace.command_history.step_redo();
        self.refresh_run_panel_state();

        Ok(Some(DocumentHistoryApplyResult {
            direction: DocumentHistoryDirection::Redo,
            command: entry.command,
            revision,
        }))
    }

    pub fn store_snapshot(&mut self, snapshot: SolveSnapshot) {
        let limit = self.preferences.effective_snapshot_history_limit();
        self.workspace.store_snapshot(snapshot, limit);
        self.refresh_run_panel_state();
    }

    pub fn store_solver_snapshot(
        &mut self,
        id: impl Into<SolveSnapshotId>,
        sequence: u64,
        snapshot: &rf_solver::SolveSnapshot,
    ) {
        let ui_snapshot = SolveSnapshot::from_solver_snapshot(
            id,
            self.workspace.document.revision,
            sequence,
            snapshot,
        );
        self.store_snapshot(ui_snapshot);
    }

    pub fn mark_saved(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.workspace.mark_saved(path.clone());
        if !self
            .preferences
            .recent_project_paths
            .iter()
            .any(|item| item == &path)
        {
            self.preferences.recent_project_paths.push(path);
        }
    }

    pub fn set_simulation_mode(&mut self, mode: SimulationMode) {
        match mode {
            SimulationMode::Active => self.workspace.solve_session.activate(),
            SimulationMode::Hold => self.workspace.solve_session.mode = SimulationMode::Hold,
        }
        self.refresh_run_panel_state();
    }

    pub fn request_manual_run(&mut self) {
        self.workspace.solve_session.request_manual_run();
        self.refresh_run_panel_state();
    }

    pub fn record_failure(&mut self, revision: u64, status: RunStatus, summary: DiagnosticSummary) {
        self.workspace
            .solve_session
            .hold_with_failure(revision, status, summary);
        self.refresh_run_panel_state();
    }

    pub fn sync_run_panel_state(&mut self, state: RunPanelState) {
        self.workspace.run_panel = state;
    }

    pub fn refresh_run_panel_state(&mut self) {
        let state = RunPanelState::from_runtime(
            &self.workspace.solve_session,
            latest_snapshot(&self.workspace),
            self.log_feed.entries.back(),
        );
        self.sync_run_panel_state(state);
    }

    pub fn push_log(&mut self, level: AppLogLevel, message: impl Into<String>) {
        self.log_feed.push(level, message);
        self.refresh_run_panel_state();
    }

    pub fn set_canvas_view_mode(&mut self, view_mode: CanvasViewMode) {
        self.workspace.canvas_interaction.set_view_mode(view_mode);
    }

    pub fn set_flowsheet_property_package_id(
        &mut self,
        package_id: Option<String>,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<u64>> {
        let mut next_flowsheet = self.workspace.document.flowsheet.clone();
        next_flowsheet.set_property_package_id(package_id)?;

        if self.workspace.document.flowsheet.property_package_id()
            == next_flowsheet.property_package_id()
        {
            return Ok(None);
        }

        let package_id = next_flowsheet
            .property_package_id()
            .map(|package_id| package_id.to_string());
        let revision = self.commit_document_change(
            DocumentCommand::SetPropertyPackage { package_id },
            next_flowsheet,
            changed_at,
        );
        Ok(Some(revision))
    }

    pub fn add_flowsheet_component(
        &mut self,
        component: Component,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<u64>> {
        if self
            .workspace
            .document
            .flowsheet
            .components
            .contains_key(&component.id)
        {
            return Ok(None);
        }

        let mut next_flowsheet = self.workspace.document.flowsheet.clone();
        next_flowsheet.insert_component(component.clone())?;
        let revision = self.commit_document_change(
            DocumentCommand::AddComponent {
                component_id: component.id,
                name: component.name,
                formula: component.formula,
            },
            next_flowsheet,
            changed_at,
        );
        Ok(Some(revision))
    }

    pub fn remove_flowsheet_component(
        &mut self,
        component_id: ComponentId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<u64>> {
        if !self
            .workspace
            .document
            .flowsheet
            .components
            .contains_key(&component_id)
        {
            return Ok(None);
        }

        if let Some(stream) = self
            .workspace
            .document
            .flowsheet
            .streams
            .values()
            .find(|stream| stream.overall_mole_fractions.contains_key(&component_id))
        {
            return Err(RfError::invalid_input(format!(
                "component `{}` is still referenced by stream `{}` composition",
                component_id, stream.id
            )));
        }

        let mut next_flowsheet = self.workspace.document.flowsheet.clone();
        next_flowsheet.components.remove(&component_id);
        let revision = self.commit_document_change(
            DocumentCommand::RemoveComponent {
                component_id: component_id.clone(),
            },
            next_flowsheet,
            changed_at,
        );
        Ok(Some(revision))
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<CanvasSuggestion>) {
        self.workspace
            .canvas_interaction
            .replace_suggestions(suggestions);
    }

    pub fn begin_canvas_place_unit(&mut self, unit_kind: impl Into<String>) -> CanvasEditIntent {
        self.workspace
            .canvas_interaction
            .begin_place_unit(unit_kind)
    }

    pub fn cancel_canvas_pending_edit(&mut self) -> Option<CanvasEditIntent> {
        self.workspace.canvas_interaction.cancel_pending_edit()
    }

    pub fn commit_canvas_pending_edit_at(
        &mut self,
        position: CanvasPoint,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<CanvasEditCommitResult>> {
        let Some(intent) = self.workspace.canvas_interaction.pending_edit.clone() else {
            return Ok(None);
        };

        let CanvasEditIntent::PlaceUnit { unit_kind } = &intent;
        let kind = parse_canvas_unit_kind(unit_kind).ok_or_else(|| {
            RfError::invalid_input(format!(
                "canvas place unit intent uses unsupported unit kind `{unit_kind}`"
            ))
        })?;
        let UnitCreateResult {
            command,
            unit_id,
            revision,
        } = self.create_builtin_unit(kind, changed_at)?;
        self.workspace.selection.selected_units.clear();
        self.workspace.selection.selected_streams.clear();
        self.workspace
            .selection
            .selected_units
            .insert(unit_id.clone());
        self.workspace.drafts.active_target = Some(InspectorTarget::Unit(unit_id.clone()));
        self.workspace.panels.inspector_open = true;
        self.push_log(
            AppLogLevel::Info,
            format_canvas_edit_commit_message(&intent, &unit_id, position),
        );

        Ok(Some(CanvasEditCommitResult {
            intent,
            command,
            revision,
            unit_id,
            position,
        }))
    }

    pub fn accept_focused_canvas_suggestion_by_tab(
        &mut self,
    ) -> RfResult<Option<CanvasSuggestion>> {
        let focused = self
            .workspace
            .canvas_interaction
            .focused_suggestion()
            .cloned();
        let Some(focused) = focused else {
            return Ok(None);
        };
        if !focused.can_accept_with_tab() {
            return Ok(None);
        }

        if focused.acceptance.is_some() {
            apply_canvas_suggestion_acceptance(self, &focused)?;
            let mut accepted = focused;
            accepted.status = SuggestionStatus::Accepted;
            apply_canvas_suggestion_target(self, &accepted);
            self.push_log(
                AppLogLevel::Info,
                format_canvas_suggestion_accept_message(&accepted),
            );
            return Ok(Some(accepted));
        }

        let accepted = self
            .workspace
            .canvas_interaction
            .accept_suggestion(&focused.id)
            .expect("focused suggestion should remain addressable until it is accepted");
        apply_canvas_suggestion_target(self, &accepted);
        self.push_log(
            AppLogLevel::Info,
            format_canvas_suggestion_accept_message(&accepted),
        );
        Ok(Some(accepted))
    }

    pub fn accept_canvas_suggestion(
        &mut self,
        suggestion_id: &CanvasSuggestionId,
    ) -> RfResult<Option<CanvasSuggestion>> {
        let suggestion = self
            .workspace
            .canvas_interaction
            .suggestion(suggestion_id)
            .cloned();
        let Some(suggestion) = suggestion else {
            return Ok(None);
        };
        if !suggestion.can_accept_explicitly() {
            return Ok(None);
        }

        apply_canvas_suggestion_acceptance(self, &suggestion)?;
        let mut accepted = suggestion;
        accepted.status = SuggestionStatus::Accepted;
        apply_canvas_suggestion_target(self, &accepted);
        self.push_log(
            AppLogLevel::Info,
            format_canvas_suggestion_accept_message(&accepted),
        );
        Ok(Some(accepted))
    }

    pub fn reject_focused_canvas_suggestion(&mut self) -> Option<CanvasSuggestion> {
        self.workspace.canvas_interaction.reject_focused()
    }

    pub fn focus_next_canvas_suggestion(&mut self) -> Option<CanvasSuggestion> {
        self.workspace.canvas_interaction.focus_next()
    }

    pub fn focus_previous_canvas_suggestion(&mut self) -> Option<CanvasSuggestion> {
        self.workspace.canvas_interaction.focus_previous()
    }

    pub fn disconnect_stream_connections(
        &mut self,
        stream_id: &StreamId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamConnectionEditResult>> {
        if !self
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(stream_id)
        {
            return Ok(None);
        }

        let (command, next_flowsheet, disconnected_ports) =
            apply_disconnect_stream_mutation(&self.workspace.document.flowsheet, stream_id)?;
        if disconnected_ports.is_empty() {
            return Ok(None);
        }

        let revision = self.commit_document_change(command.clone(), next_flowsheet, changed_at);
        self.focus_inspector_target(InspectorTarget::Stream(stream_id.clone()));

        Ok(Some(StreamConnectionEditResult {
            stream_id: stream_id.clone(),
            disconnected_ports,
            command,
            revision,
        }))
    }

    pub fn disconnect_stream_endpoint(
        &mut self,
        stream_id: &StreamId,
        direction: PortDirection,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamConnectionEditResult>> {
        if !self
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(stream_id)
        {
            return Ok(None);
        }

        let Some((command, next_flowsheet, disconnected_port)) =
            apply_disconnect_stream_endpoint_mutation(
                &self.workspace.document.flowsheet,
                stream_id,
                direction,
            )?
        else {
            return Ok(None);
        };

        let revision = self.commit_document_change(command.clone(), next_flowsheet, changed_at);
        self.focus_inspector_target(InspectorTarget::Stream(stream_id.clone()));

        Ok(Some(StreamConnectionEditResult {
            stream_id: stream_id.clone(),
            disconnected_ports: vec![disconnected_port],
            command,
            revision,
        }))
    }

    pub fn reconnect_stream_to_unique_available_endpoint(
        &mut self,
        stream_id: &StreamId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamReconnectEditResult>> {
        if !self
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(stream_id)
        {
            return Ok(None);
        }

        let Some((command, next_flowsheet, source_port, sink_port)) =
            apply_reconnect_stream_to_unique_available_endpoint_mutation(
                &self.workspace.document.flowsheet,
                stream_id,
            )?
        else {
            return Ok(None);
        };

        let revision = self.commit_document_change(command.clone(), next_flowsheet, changed_at);
        self.focus_inspector_target(InspectorTarget::Stream(stream_id.clone()));

        Ok(Some(StreamReconnectEditResult {
            stream_id: stream_id.clone(),
            source_port,
            sink_port,
            command,
            revision,
        }))
    }

    pub fn delete_stream_and_connections(
        &mut self,
        stream_id: &StreamId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamConnectionEditResult>> {
        if !self
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(stream_id)
        {
            return Ok(None);
        }

        if self.workspace.drafts.fields.values().any(|draft| matches!(draft,
            DraftValue::Numeric(session) if session.is_pending() && session.variable().object == crate::variable_browser::ObjectId::Stream(stream_id.clone()))) {
            return Err(RfError::invalid_input("finish or discard numeric edits before deleting their stream"));
        }
        let (command, next_flowsheet, disconnected_ports) =
            apply_delete_stream_and_disconnect_ports_mutation(
                &self.workspace.document.flowsheet,
                stream_id,
            )?;
        let revision = self.commit_document_change(command.clone(), next_flowsheet, changed_at);
        self.workspace.selection.selected_streams.remove(stream_id);
        if self.workspace.drafts.active_target == Some(InspectorTarget::Stream(stream_id.clone())) {
            self.workspace.drafts.active_target = None;
        }

        Ok(Some(StreamConnectionEditResult {
            stream_id: stream_id.clone(),
            disconnected_ports,
            command,
            revision,
        }))
    }

    pub fn apply_run_panel_recovery_action(
        &mut self,
        action: &RunPanelRecoveryAction,
    ) -> Option<InspectorTarget> {
        if action.mutation.is_some()
            && self.workspace.drafts.fields.values().any(|draft| {
                matches!(draft,
            DraftValue::Numeric(session) if session.is_pending())
            })
        {
            return None;
        }
        if let Some(mutation) = action.mutation.as_ref()
            && let Ok((command, next_flowsheet)) =
                apply_run_panel_recovery_mutation(&self.workspace.document.flowsheet, mutation)
        {
            self.commit_document_change(command, next_flowsheet, SystemTime::now());
        }
        self.workspace.selection.selected_units.clear();
        self.workspace.selection.selected_streams.clear();
        self.workspace.drafts.active_target = None;
        if let Some(unit_id) = action.target_unit_id.as_ref() {
            if !self
                .workspace
                .document
                .flowsheet
                .units
                .contains_key(unit_id)
            {
                return None;
            }
            let unit_id = unit_id.clone();
            self.workspace
                .selection
                .selected_units
                .insert(unit_id.clone());
            self.workspace.drafts.active_target = Some(InspectorTarget::Unit(unit_id.clone()));
            self.workspace.panels.inspector_open = true;
            return Some(InspectorTarget::Unit(unit_id));
        }
        if let Some(stream_id) = action.target_stream_id.as_ref() {
            if !self
                .workspace
                .document
                .flowsheet
                .streams
                .contains_key(stream_id)
            {
                return None;
            }
            let stream_id = stream_id.clone();
            self.workspace
                .selection
                .selected_streams
                .insert(stream_id.clone());
            self.workspace.drafts.active_target = Some(InspectorTarget::Stream(stream_id.clone()));
            self.workspace.panels.inspector_open = true;
            return Some(InspectorTarget::Stream(stream_id));
        }
        None
    }

    pub fn focus_inspector_target(&mut self, target: InspectorTarget) -> Option<InspectorTarget> {
        match target {
            InspectorTarget::Unit(unit_id) => {
                if !self
                    .workspace
                    .document
                    .flowsheet
                    .units
                    .contains_key(&unit_id)
                {
                    return None;
                }
                self.workspace.selection.selected_units.clear();
                self.workspace.selection.selected_streams.clear();
                self.workspace
                    .selection
                    .selected_units
                    .insert(unit_id.clone());
                self.workspace.drafts.active_target = Some(InspectorTarget::Unit(unit_id.clone()));
                self.workspace.panels.inspector_open = true;
                Some(InspectorTarget::Unit(unit_id))
            }
            InspectorTarget::Stream(stream_id) => {
                if !self
                    .workspace
                    .document
                    .flowsheet
                    .streams
                    .contains_key(&stream_id)
                {
                    return None;
                }
                self.workspace.selection.selected_units.clear();
                self.workspace.selection.selected_streams.clear();
                self.workspace
                    .selection
                    .selected_streams
                    .insert(stream_id.clone());
                self.workspace.drafts.active_target =
                    Some(InspectorTarget::Stream(stream_id.clone()));
                self.workspace.panels.inspector_open = true;
                Some(InspectorTarget::Stream(stream_id))
            }
        }
    }

    pub fn clear_inspector_target(&mut self) -> Option<InspectorTarget> {
        let previous_target = self.workspace.drafts.active_target.take();
        self.workspace.selection.selected_units.clear();
        self.workspace.selection.selected_streams.clear();
        previous_target
    }

    pub fn begin_browser_login(&mut self, authority_url: impl Into<String>) {
        self.auth_session.begin_browser_login(authority_url);
    }

    pub fn complete_login(
        &mut self,
        authority_url: impl Into<String>,
        user: AuthenticatedUser,
        token_lease: TokenLease,
        authenticated_at: DateTimeUtc,
    ) {
        self.auth_session
            .complete_login(authority_url, user, token_lease, authenticated_at);
    }

    pub fn update_entitlement(
        &mut self,
        snapshot: EntitlementSnapshot,
        manifests: Vec<PropertyPackageManifest>,
        synced_at: DateTimeUtc,
    ) {
        self.entitlement.update(snapshot, manifests, synced_at);
    }

    pub fn clear_auth_session(&mut self) {
        self.auth_session.clear();
        self.entitlement.clear();
    }
}

fn format_edit_number(value: f64) -> String {
    value.to_string()
}

pub fn latest_snapshot_id(workspace: &WorkspaceState) -> Option<&SolveSnapshotId> {
    latest_snapshot(workspace).map(|snapshot| &snapshot.id)
}

pub fn latest_snapshot(workspace: &WorkspaceState) -> Option<&SolveSnapshot> {
    let latest_snapshot_id = workspace.solve_session.latest_snapshot.as_ref()?;
    let snapshot = workspace
        .snapshot_history
        .iter()
        .rev()
        .find(|snapshot| &snapshot.id == latest_snapshot_id)?;
    (snapshot.document_revision == workspace.solve_session.observed_revision).then_some(snapshot)
}

pub fn stale_snapshot(workspace: &WorkspaceState) -> Option<&SolveSnapshot> {
    if workspace.solve_session.latest_snapshot.is_some()
        || workspace.solve_session.pending_reason
            != Some(SolvePendingReason::DocumentRevisionAdvanced)
    {
        return None;
    }

    workspace
        .snapshot_history
        .iter()
        .rev()
        .find(|snapshot| snapshot.document_revision != workspace.solve_session.observed_revision)
}
