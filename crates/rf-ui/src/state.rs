use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::time::SystemTime;

use rf_flowsheet::validate_connections;
use rf_model::Flowsheet;
use rf_types::{RfError, RfResult, StreamId, UnitId};

use crate::auth::{
    AuthSessionState, AuthenticatedUser, EntitlementSnapshot, EntitlementState,
    PropertyPackageManifest, TokenLease,
};
use crate::canvas_interaction::{
    CanvasInteractionState, CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding,
    CanvasSuggestion, CanvasSuggestionAcceptance, CanvasViewMode, SuggestionSource,
    SuggestionStatus,
};
use crate::commands::{CommandHistory, CommandHistoryEntry, DocumentCommand};
use crate::diagnostics::DiagnosticSummary;
use crate::ids::{DocumentId, SolveSnapshotId};
use crate::run::{RunStatus, SimulationMode, SolvePendingReason, SolveSessionState, SolveSnapshot};
use crate::run_panel::{RunPanelRecoveryAction, RunPanelState};

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
    Number(FieldDraft<f64>),
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
        let revision = self.document.replace_flowsheet(next_flowsheet, changed_at);
        self.command_history
            .record(CommandHistoryEntry::new(revision, command));
        self.canvas_interaction.invalidate_all();
        self.solve_session.mark_document_revision_advanced(revision);
        self.drafts.clear();
        revision
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
            self.workspace.snapshot_history.back(),
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

    pub fn accept_focused_canvas_suggestion_by_tab(&mut self) -> RfResult<Option<CanvasSuggestion>> {
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

    pub fn apply_run_panel_recovery_action(
        &mut self,
        action: &RunPanelRecoveryAction,
    ) -> Option<InspectorTarget> {
        let unit_id = action.target_unit_id.as_ref()?.clone();
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

pub fn latest_snapshot_id(workspace: &WorkspaceState) -> Option<&SolveSnapshotId> {
    workspace.solve_session.latest_snapshot.as_ref()
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
            apply_material_connection_acceptance(&app_state.workspace.document.flowsheet, connection)?
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
    if let (Some(sink_unit_id), Some(sink_port)) =
        (&connection.sink_unit_id, &connection.sink_port)
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

fn bind_material_stream_port(
    flowsheet: &mut Flowsheet,
    unit_id: &UnitId,
    port_name: &str,
    stream_id: &StreamId,
) -> RfResult<()> {
    let unit = flowsheet.units.get_mut(unit_id).ok_or_else(|| {
        RfError::missing_entity("unit", unit_id)
    })?;
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
