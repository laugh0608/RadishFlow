use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::time::SystemTime;

use rf_flowsheet::validate_connections;
use rf_model::{Flowsheet, MaterialStreamState, UnitPort};
use rf_types::{ComponentId, RfError, RfResult, StreamId, UnitId};
use rf_unitops::{UnitOperationSpec, builtin_unit_spec_by_name};

use crate::auth::{
    AuthSessionState, AuthenticatedUser, EntitlementSnapshot, EntitlementState,
    PropertyPackageManifest, TokenLease,
};
use crate::canvas_interaction::{
    CanvasInteractionState, CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding,
    CanvasSuggestion, CanvasSuggestionAcceptance, CanvasViewMode, SuggestionSource,
    SuggestionStatus,
};
use crate::commands::{
    CommandHistory, CommandHistoryEntry, CommandValue, DocumentCommand, StreamSpecificationValue,
};
use crate::diagnostics::DiagnosticSummary;
use crate::ids::{DocumentId, SolveSnapshotId};
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

#[derive(Debug, Clone, PartialEq)]
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
}

impl InspectorDraftState {
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

#[derive(Debug, Clone, PartialEq)]
pub struct StreamInspectorDraftBatchCommitResult {
    pub keys: Vec<String>,
    pub active_target: InspectorTarget,
    pub command: DocumentCommand,
    pub revision: u64,
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
    pub document: FlowsheetDocument,
    pub document_path: Option<PathBuf>,
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
            document,
            document_path: None,
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
        let revision = self.document.replace_flowsheet(next_flowsheet, changed_at);
        self.command_history
            .record(CommandHistoryEntry::with_snapshots(
                revision, command, before, after,
            ));
        self.canvas_interaction.invalidate_all();
        self.solve_session.mark_document_revision_advanced(revision);
        self.drafts.clear();
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
        revision
    }

    fn apply_history_flowsheet(&mut self, flowsheet: Flowsheet, changed_at: DateTimeUtc) -> u64 {
        let revision = self.document.replace_flowsheet(flowsheet, changed_at);
        self.canvas_interaction.invalidate_all();
        self.solve_session.mark_document_revision_advanced(revision);
        self.prune_focus_against_document();
        self.drafts.fields.clear();
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

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<CanvasSuggestion>) {
        self.workspace
            .canvas_interaction
            .replace_suggestions(suggestions);
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

    pub fn reject_focused_canvas_suggestion(&mut self) -> Option<CanvasSuggestion> {
        self.workspace.canvas_interaction.reject_focused()
    }

    pub fn focus_next_canvas_suggestion(&mut self) -> Option<CanvasSuggestion> {
        self.workspace.canvas_interaction.focus_next()
    }

    pub fn focus_previous_canvas_suggestion(&mut self) -> Option<CanvasSuggestion> {
        self.workspace.canvas_interaction.focus_previous()
    }

    pub fn apply_run_panel_recovery_action(
        &mut self,
        action: &RunPanelRecoveryAction,
    ) -> Option<InspectorTarget> {
        if let Some(mutation) = action.mutation.as_ref() {
            if let Ok((command, next_flowsheet)) =
                apply_run_panel_recovery_mutation(&self.workspace.document.flowsheet, mutation)
            {
                self.commit_document_change(command, next_flowsheet, SystemTime::now());
            }
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

    pub fn update_stream_inspector_draft(
        &mut self,
        stream_id: &StreamId,
        field: StreamInspectorDraftField,
        raw_value: impl Into<String>,
    ) -> Option<StreamInspectorDraftUpdateResult> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return None;
        }

        let stream = self.workspace.document.flowsheet.streams.get(stream_id)?;
        if !stream_inspector_draft_fields(stream).contains(&field) {
            return None;
        }
        let raw_value = raw_value.into();
        let key = stream_inspector_draft_key(stream_id, &field);
        let (draft_value, is_dirty, validation) =
            stream_draft_value_from_raw(&field, stream, raw_value);

        if !is_dirty && validation != DraftValidationState::Invalid {
            self.workspace.drafts.fields.remove(&key);
        } else {
            self.workspace
                .drafts
                .fields
                .insert(key.clone(), draft_value);
        }

        Some(StreamInspectorDraftUpdateResult {
            key,
            active_target,
            is_dirty,
            validation,
        })
    }

    pub fn commit_stream_inspector_draft(
        &mut self,
        stream_id: &StreamId,
        field: StreamInspectorDraftField,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamInspectorDraftCommitResult>> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return Ok(None);
        }

        if !self
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(stream_id)
        {
            return Ok(None);
        }

        let key = stream_inspector_draft_key(stream_id, &field);
        let Some(draft_value) = self.workspace.drafts.fields.get(&key) else {
            return Ok(None);
        };
        let Some(command_value) = stream_command_value_from_draft(&field, draft_value)? else {
            return Ok(None);
        };

        let mut next_flowsheet = self.workspace.document.flowsheet.clone();
        apply_stream_specification_value(&mut next_flowsheet, stream_id, &field, &command_value)?;

        let command = DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: field.command_field(),
            value: command_value,
        };
        let revision = self.workspace.commit_inspector_document_change(
            command.clone(),
            next_flowsheet,
            changed_at,
        );
        self.workspace.drafts.fields.remove(&key);
        self.refresh_run_panel_state();

        Ok(Some(StreamInspectorDraftCommitResult {
            key,
            active_target,
            command,
            revision,
        }))
    }

    pub fn commit_stream_inspector_drafts(
        &mut self,
        stream_id: &StreamId,
        changed_at: DateTimeUtc,
    ) -> RfResult<Option<StreamInspectorDraftBatchCommitResult>> {
        let active_target = InspectorTarget::Stream(stream_id.clone());
        if self.workspace.drafts.active_target.as_ref() != Some(&active_target) {
            return Ok(None);
        }

        if !self
            .workspace
            .document
            .flowsheet
            .streams
            .contains_key(stream_id)
        {
            return Ok(None);
        }

        let stream = self
            .workspace
            .document
            .flowsheet
            .streams
            .get(stream_id)
            .expect("stream existence was checked above");
        let fields = stream_inspector_draft_fields(stream);
        let mut next_flowsheet = self.workspace.document.flowsheet.clone();
        let mut keys = Vec::new();
        let mut values = Vec::new();

        for field in fields {
            let key = stream_inspector_draft_key(stream_id, &field);
            let Some(draft_value) = self.workspace.drafts.fields.get(&key) else {
                continue;
            };
            let Some(command_value) = stream_command_value_from_draft(&field, draft_value)? else {
                continue;
            };

            apply_stream_specification_value(
                &mut next_flowsheet,
                stream_id,
                &field,
                &command_value,
            )?;
            keys.push(key);
            values.push(StreamSpecificationValue {
                field: field.command_field(),
                value: command_value,
            });
        }

        if values.is_empty() {
            return Ok(None);
        }

        let command = stream_specification_command(stream_id, values);
        let revision = self.workspace.commit_inspector_document_change(
            command.clone(),
            next_flowsheet,
            changed_at,
        );
        for key in &keys {
            self.workspace.drafts.fields.remove(key);
        }
        self.refresh_run_panel_state();

        Ok(Some(StreamInspectorDraftBatchCommitResult {
            keys,
            active_target,
            command,
            revision,
        }))
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

fn stream_draft_value_from_raw(
    field: &StreamInspectorDraftField,
    stream: &MaterialStreamState,
    raw_value: String,
) -> (DraftValue, bool, DraftValidationState) {
    match field {
        StreamInspectorDraftField::Name => {
            let original = stream.name.clone();
            let validation = if raw_value.trim().is_empty() {
                DraftValidationState::Invalid
            } else {
                DraftValidationState::Valid
            };
            let is_dirty = raw_value != original;
            let draft = FieldDraft {
                original,
                current: raw_value,
                is_dirty,
                validation,
            };
            (DraftValue::Text(draft), is_dirty, validation)
        }
        StreamInspectorDraftField::TemperatureK => {
            stream_number_draft_value(stream.temperature_k, raw_value, |value| {
                value.is_finite() && value > 0.0
            })
        }
        StreamInspectorDraftField::PressurePa => {
            stream_number_draft_value(stream.pressure_pa, raw_value, |value| {
                value.is_finite() && value > 0.0
            })
        }
        StreamInspectorDraftField::TotalMolarFlowMolS => {
            stream_number_draft_value(stream.total_molar_flow_mol_s, raw_value, |value| {
                value.is_finite() && value >= 0.0
            })
        }
        StreamInspectorDraftField::OverallMoleFraction(component_id) => {
            let original = stream
                .overall_mole_fractions
                .get(component_id)
                .copied()
                .unwrap_or(0.0);
            stream_number_draft_value(original, raw_value, |value| {
                is_valid_stream_scalar_value(field, value)
                    && composition_sum_after_fraction(stream, component_id, value)
                        .is_some_and(|sum| sum > 0.0)
            })
        }
    }
}

fn stream_number_draft_value<F>(
    original_number: f64,
    raw_value: String,
    is_valid_number: F,
) -> (DraftValue, bool, DraftValidationState)
where
    F: Fn(f64) -> bool,
{
    let original = format_edit_number(original_number);
    let parsed = raw_value.trim().parse::<f64>();
    let validation = match parsed {
        Ok(value) if is_valid_number(value) => DraftValidationState::Valid,
        _ => DraftValidationState::Invalid,
    };
    let is_dirty = match parsed {
        Ok(value) if validation == DraftValidationState::Valid => value != original_number,
        _ => raw_value != original,
    };
    let draft = FieldDraft {
        original,
        current: raw_value,
        is_dirty,
        validation,
    };
    (DraftValue::Number(draft), is_dirty, validation)
}

fn stream_command_value_from_draft(
    field: &StreamInspectorDraftField,
    draft_value: &DraftValue,
) -> RfResult<Option<CommandValue>> {
    match (field, draft_value) {
        (StreamInspectorDraftField::Name, DraftValue::Text(draft)) => {
            if !draft.is_dirty || draft.validation != DraftValidationState::Valid {
                return Ok(None);
            }
            Ok(Some(CommandValue::Text(draft.current.clone())))
        }
        (
            StreamInspectorDraftField::TemperatureK
            | StreamInspectorDraftField::PressurePa
            | StreamInspectorDraftField::TotalMolarFlowMolS
            | StreamInspectorDraftField::OverallMoleFraction(_),
            DraftValue::Number(draft),
        ) => {
            if !draft.is_dirty || draft.validation != DraftValidationState::Valid {
                return Ok(None);
            }
            let value = draft.current.trim().parse::<f64>().map_err(|_| {
                RfError::invalid_input(format!(
                    "stream inspector draft `{}` is not a valid number",
                    draft.current
                ))
            })?;
            if !is_valid_stream_scalar_value(field, value) {
                return Ok(None);
            }
            Ok(Some(CommandValue::Number(value)))
        }
        _ => Ok(None),
    }
}

fn stream_inspector_draft_fields(stream: &MaterialStreamState) -> Vec<StreamInspectorDraftField> {
    let mut fields = vec![
        StreamInspectorDraftField::Name,
        StreamInspectorDraftField::TemperatureK,
        StreamInspectorDraftField::PressurePa,
        StreamInspectorDraftField::TotalMolarFlowMolS,
    ];
    fields.extend(
        stream
            .overall_mole_fractions
            .keys()
            .cloned()
            .map(StreamInspectorDraftField::OverallMoleFraction),
    );
    fields
}

fn stream_specification_command(
    stream_id: &StreamId,
    values: Vec<StreamSpecificationValue>,
) -> DocumentCommand {
    let mut values = values;
    if values.len() == 1 {
        let value = values
            .pop()
            .expect("single stream specification value should exist");
        DocumentCommand::SetStreamSpecification {
            stream_id: stream_id.clone(),
            field: value.field,
            value: value.value,
        }
    } else {
        DocumentCommand::SetStreamSpecifications {
            stream_id: stream_id.clone(),
            values,
        }
    }
}

fn apply_stream_specification_value(
    flowsheet: &mut Flowsheet,
    stream_id: &StreamId,
    field: &StreamInspectorDraftField,
    value: &CommandValue,
) -> RfResult<()> {
    let stream = flowsheet
        .streams
        .get_mut(stream_id)
        .ok_or_else(|| RfError::missing_entity("stream", stream_id))?;

    match (field, value) {
        (StreamInspectorDraftField::Name, CommandValue::Text(value)) => {
            if value.trim().is_empty() {
                return Err(RfError::invalid_input("stream name cannot be empty"));
            }
            stream.name = value.clone();
        }
        (StreamInspectorDraftField::TemperatureK, CommandValue::Number(value))
            if is_valid_stream_scalar_value(field, *value) =>
        {
            stream.temperature_k = *value;
        }
        (StreamInspectorDraftField::PressurePa, CommandValue::Number(value))
            if is_valid_stream_scalar_value(field, *value) =>
        {
            stream.pressure_pa = *value;
        }
        (StreamInspectorDraftField::TotalMolarFlowMolS, CommandValue::Number(value))
            if is_valid_stream_scalar_value(field, *value) =>
        {
            stream.total_molar_flow_mol_s = *value;
        }
        (
            StreamInspectorDraftField::OverallMoleFraction(component_id),
            CommandValue::Number(value),
        ) if is_valid_stream_scalar_value(field, *value) => {
            stream
                .overall_mole_fractions
                .insert(component_id.clone(), *value);
            validate_stream_overall_mole_fractions(stream)?;
        }
        _ => {
            return Err(RfError::invalid_input(format!(
                "stream field `{}` cannot be set from value `{value:?}`",
                field.command_field()
            )));
        }
    }

    Ok(())
}

fn is_valid_stream_scalar_value(field: &StreamInspectorDraftField, value: f64) -> bool {
    match field {
        StreamInspectorDraftField::Name => false,
        StreamInspectorDraftField::TemperatureK | StreamInspectorDraftField::PressurePa => {
            value.is_finite() && value > 0.0
        }
        StreamInspectorDraftField::TotalMolarFlowMolS => value.is_finite() && value >= 0.0,
        StreamInspectorDraftField::OverallMoleFraction(_) => {
            value.is_finite() && (0.0..=1.0).contains(&value)
        }
    }
}

fn composition_sum_after_fraction(
    stream: &MaterialStreamState,
    component_id: &ComponentId,
    value: f64,
) -> Option<f64> {
    stream
        .overall_mole_fractions
        .iter()
        .map(|(candidate_id, fraction)| {
            if candidate_id == component_id {
                value
            } else {
                *fraction
            }
        })
        .try_fold(0.0, |sum, fraction| {
            (fraction.is_finite() && fraction >= 0.0).then_some(sum + fraction)
        })
}

fn validate_stream_overall_mole_fractions(stream: &MaterialStreamState) -> RfResult<()> {
    if stream.overall_mole_fractions.is_empty() {
        return Err(RfError::invalid_input(format!(
            "stream `{}` must define at least one overall mole fraction entry",
            stream.id
        )));
    }

    let sum = stream
        .overall_mole_fractions
        .values()
        .try_fold(0.0, |sum, value| {
            (value.is_finite() && (0.0..=1.0).contains(value)).then_some(sum + value)
        })
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "stream `{}` overall mole fractions must be finite values between zero and one",
                stream.id
            ))
        })?;
    if sum <= 0.0 {
        return Err(RfError::invalid_input(format!(
            "stream `{}` overall mole fractions must sum to a positive finite value",
            stream.id
        )));
    }

    Ok(())
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

fn apply_canvas_suggestion_acceptance(
    app_state: &mut AppState,
    suggestion: &CanvasSuggestion,
) -> RfResult<()> {
    let acceptance = suggestion.acceptance.as_ref().ok_or_else(|| {
        RfError::invalid_input(format!(
            "canvas suggestion `{}` is missing an acceptance payload",
            suggestion.id
        ))
    })?;

    let (command, next_flowsheet) = match acceptance {
        CanvasSuggestionAcceptance::MaterialConnection(connection) => {
            apply_material_connection_acceptance(
                &app_state.workspace.document.flowsheet,
                connection,
            )?
        }
    };

    app_state.commit_document_change(command, next_flowsheet, SystemTime::now());
    Ok(())
}

fn apply_canvas_suggestion_target(app_state: &mut AppState, suggestion: &CanvasSuggestion) {
    let unit_id = suggestion.ghost.target_unit_id.clone();
    app_state.workspace.selection.selected_units.clear();
    app_state.workspace.selection.selected_streams.clear();
    app_state
        .workspace
        .selection
        .selected_units
        .insert(unit_id.clone());
    app_state.workspace.drafts.active_target = Some(InspectorTarget::Unit(unit_id));
    app_state.workspace.panels.inspector_open = true;
}

fn apply_material_connection_acceptance(
    flowsheet: &Flowsheet,
    connection: &CanvasSuggestedMaterialConnection,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    let stream_id = match &connection.stream {
        CanvasSuggestedStreamBinding::Existing { stream_id } => {
            next_flowsheet.stream(stream_id)?;
            stream_id.clone()
        }
        CanvasSuggestedStreamBinding::Create { stream } => {
            next_flowsheet.insert_stream(stream.clone())?;
            stream.id.clone()
        }
    };

    bind_material_stream_port(
        &mut next_flowsheet,
        &connection.source_unit_id,
        &connection.source_port,
        &stream_id,
    )?;
    if let (Some(sink_unit_id), Some(sink_port)) = (&connection.sink_unit_id, &connection.sink_port)
    {
        bind_material_stream_port(&mut next_flowsheet, sink_unit_id, sink_port, &stream_id)?;
    } else if connection.sink_unit_id.is_some() || connection.sink_port.is_some() {
        return Err(RfError::invalid_input(
            "canvas suggestion material connection must provide both sink unit and sink port or neither",
        ));
    }

    validate_connections(&next_flowsheet)?;

    let command = DocumentCommand::ConnectPorts {
        stream_id,
        from_unit_id: connection.source_unit_id.clone(),
        from_port: connection.source_port.clone(),
        to_unit_id: connection.sink_unit_id.clone(),
        to_port: connection.sink_port.clone(),
    };

    Ok((command, next_flowsheet))
}

fn apply_run_panel_recovery_mutation(
    flowsheet: &Flowsheet,
    mutation: &RunPanelRecoveryMutation,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    match mutation {
        RunPanelRecoveryMutation::DeleteStream { stream_id } => {
            apply_delete_stream_mutation(flowsheet, stream_id)
        }
        RunPanelRecoveryMutation::CreateAndBindOutletStream { unit_id, port_name } => {
            apply_create_and_bind_outlet_stream_mutation(flowsheet, unit_id, port_name)
        }
        RunPanelRecoveryMutation::DisconnectPortAndDeleteStream {
            unit_id,
            port_name,
            stream_id,
        } => apply_disconnect_port_and_delete_stream_mutation(
            flowsheet, unit_id, port_name, stream_id,
        ),
        RunPanelRecoveryMutation::RestoreCanonicalPortSignature { unit_id } => {
            apply_restore_canonical_port_signature_mutation(flowsheet, unit_id)
        }
        RunPanelRecoveryMutation::DisconnectPort { unit_id, port_name } => {
            apply_disconnect_port_mutation(flowsheet, unit_id, port_name)
        }
    }
}

fn apply_disconnect_port_mutation(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    disconnect_material_stream_port(&mut next_flowsheet, unit_id, port_name)?;

    Ok((
        DocumentCommand::DisconnectPorts {
            unit_id: unit_id.clone(),
            port: port_name.to_string(),
        },
        next_flowsheet,
    ))
}

fn apply_delete_stream_mutation(
    flowsheet: &Flowsheet,
    stream_id: &StreamId,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    next_flowsheet.remove_stream(stream_id)?;

    Ok((
        DocumentCommand::DeleteStream {
            stream_id: stream_id.clone(),
        },
        next_flowsheet,
    ))
}

fn apply_create_and_bind_outlet_stream_mutation(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    let stream_id = next_available_placeholder_stream_id(&next_flowsheet, unit_id, port_name);
    let stream_name = format!("{} {} Stream", unit_id.as_str(), port_name);
    next_flowsheet.insert_stream(MaterialStreamState::new(stream_id.clone(), stream_name))?;
    bind_material_stream_port(&mut next_flowsheet, unit_id, port_name, &stream_id)?;

    Ok((
        DocumentCommand::ConnectPorts {
            stream_id,
            from_unit_id: unit_id.clone(),
            from_port: port_name.to_string(),
            to_unit_id: None,
            to_port: None,
        },
        next_flowsheet,
    ))
}

fn apply_disconnect_port_and_delete_stream_mutation(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
    stream_id: &StreamId,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    disconnect_material_stream_port(&mut next_flowsheet, unit_id, port_name)?;
    next_flowsheet.remove_stream(stream_id)?;

    Ok((
        DocumentCommand::DisconnectPortAndDeleteStream {
            unit_id: unit_id.clone(),
            port: port_name.to_string(),
            stream_id: stream_id.clone(),
        },
        next_flowsheet,
    ))
}

fn apply_restore_canonical_port_signature_mutation(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
) -> RfResult<(DocumentCommand, Flowsheet)> {
    let mut next_flowsheet = flowsheet.clone();
    let unit = next_flowsheet
        .units
        .get_mut(unit_id)
        .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
    let spec = builtin_unit_spec_by_name(&unit.kind).ok_or_else(|| {
        RfError::invalid_input(format!(
            "unit `{}` kind `{}` does not expose a canonical built-in spec",
            unit_id, unit.kind
        ))
    })?;
    let restored_ports = rebuild_ports_from_canonical_spec(&unit.ports, spec);
    unit.ports = restored_ports;

    Ok((
        DocumentCommand::RestoreCanonicalUnitPorts {
            unit_id: unit_id.clone(),
        },
        next_flowsheet,
    ))
}

fn bind_material_stream_port(
    flowsheet: &mut Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
    stream_id: &StreamId,
) -> RfResult<()> {
    let unit = flowsheet
        .units
        .get_mut(unit_id)
        .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
    let port = unit
        .ports
        .iter_mut()
        .find(|port| port.name == port_name)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "unit `{}` does not expose material port `{}`",
                unit_id, port_name
            ))
        })?;
    if port.kind != rf_types::PortKind::Material {
        return Err(RfError::invalid_input(format!(
            "unit `{}` port `{}` is not a material port",
            unit_id, port_name
        )));
    }
    port.stream_id = Some(stream_id.clone());
    Ok(())
}

fn disconnect_material_stream_port(
    flowsheet: &mut Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
) -> RfResult<()> {
    let unit = flowsheet
        .units
        .get_mut(unit_id)
        .ok_or_else(|| RfError::missing_entity("unit", unit_id))?;
    let port = unit
        .ports
        .iter_mut()
        .find(|port| port.name == port_name)
        .ok_or_else(|| {
            RfError::invalid_input(format!(
                "unit `{}` does not expose material port `{}`",
                unit_id, port_name
            ))
        })?;
    if port.kind != rf_types::PortKind::Material {
        return Err(RfError::invalid_input(format!(
            "unit `{}` port `{}` is not a material port",
            unit_id, port_name
        )));
    }
    port.stream_id = None;
    Ok(())
}

fn next_available_placeholder_stream_id(
    flowsheet: &Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
) -> StreamId {
    let base = format!("stream-{}-{}", unit_id.as_str(), port_name);
    let mut candidate = base.clone();
    let mut suffix = 1usize;
    while flowsheet
        .streams
        .contains_key(&StreamId::new(candidate.as_str()))
    {
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
    StreamId::new(candidate)
}

fn rebuild_ports_from_canonical_spec(
    existing_ports: &[UnitPort],
    spec: &UnitOperationSpec,
) -> Vec<UnitPort> {
    let mut remaining_ports = existing_ports.to_vec();
    let mut rebuilt = Vec::with_capacity(spec.ports.len());

    for expected in spec.ports {
        let stream_id =
            take_matching_stream_id(&mut remaining_ports, |port| port.name == expected.name)
                .or_else(|| {
                    take_unique_matching_stream_id(&mut remaining_ports, |port| {
                        port.direction == expected.direction && port.kind == expected.kind
                    })
                });
        rebuilt.push(UnitPort::new(
            expected.name,
            expected.direction,
            expected.kind,
            stream_id,
        ));
    }

    rebuilt
}

fn take_matching_stream_id<F>(remaining_ports: &mut Vec<UnitPort>, predicate: F) -> Option<StreamId>
where
    F: Fn(&UnitPort) -> bool,
{
    let index = remaining_ports.iter().position(predicate)?;
    Some(remaining_ports.remove(index).stream_id).flatten()
}

fn take_unique_matching_stream_id<F>(
    remaining_ports: &mut Vec<UnitPort>,
    predicate: F,
) -> Option<StreamId>
where
    F: Fn(&UnitPort) -> bool,
{
    let mut matches = remaining_ports
        .iter()
        .enumerate()
        .filter_map(|(index, port)| predicate(port).then_some(index));
    let index = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(remaining_ports.remove(index).stream_id).flatten()
}

fn format_canvas_suggestion_accept_message(suggestion: &CanvasSuggestion) -> String {
    format!(
        "Accepted canvas suggestion `{}` from {} for unit {}",
        suggestion.id.as_str(),
        canvas_suggestion_source_label(suggestion.source),
        suggestion.ghost.target_unit_id.as_str()
    )
}

fn canvas_suggestion_source_label(source: SuggestionSource) -> &'static str {
    match source {
        SuggestionSource::LocalRules => "local rules",
        SuggestionSource::RadishMind => "RadishMind",
    }
}
