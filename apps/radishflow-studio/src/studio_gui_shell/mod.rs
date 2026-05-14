use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::studio_gui_preferences_store::{
    default_studio_preferences_path, load_recent_project_paths,
};
use eframe::egui;
use radishflow_studio::{
    StudioAppHostWindowState, StudioGuiCommandEntry, StudioGuiCommandMenuCommandModel,
    StudioGuiCommandMenuNode, StudioGuiDriverOutcome, StudioGuiEvent, StudioGuiFocusContext,
    StudioGuiPlatformExecutedDispatch, StudioGuiPlatformExecutedNativeTimerCallbackBatch,
    StudioGuiPlatformExecutedNativeTimerCallbackOutcome, StudioGuiPlatformHost,
    StudioGuiPlatformNativeTimerId, StudioGuiPlatformTimerCommand, StudioGuiPlatformTimerExecutor,
    StudioGuiPlatformTimerExecutorResponse, StudioGuiPlatformTimerFollowUpCommand,
    StudioGuiShortcut, StudioGuiShortcutKey, StudioGuiShortcutModifier, StudioGuiWindowAreaId,
    StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion, StudioGuiWindowDropTargetQuery,
    StudioGuiWindowLayoutModel, StudioGuiWindowLayoutMutation, StudioGuiWindowModel,
    StudioGuiWindowStackGroupLayout, StudioGuiWindowToolbarSectionModel, StudioRuntimeConfig,
    StudioRuntimeEntitlementPreflight, StudioRuntimeTrigger, StudioWindowHostId,
    StudioWindowHostRole,
};
use rf_types::RfResult;
use rf_ui::{
    RunPanelIntent, RunPanelNoticeLevel, RunPanelRecoveryWidgetEvent, RunPanelWidgetEvent,
    SimulationMode,
};

mod app;
mod chrome;
mod fonts;
mod locale;
mod panels;
mod project_picker;
mod utils;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod timer_tests;

use self::locale::{ShellText, StudioShellLocale};
use self::project_picker::{NativeProjectFilePicker, ProjectFilePicker};
use self::utils::*;

const STUDIO_INITIAL_WINDOW_SIZE: [f32; 2] = [1280.0, 860.0];
const STUDIO_MIN_WINDOW_SIZE: [f32; 2] = [1024.0, 720.0];

pub fn run() -> eframe::Result<()> {
    let native_options = studio_native_options();
    eframe::run_native(
        "RadishFlow Studio",
        native_options,
        Box::new(|cc| {
            fonts::configure_studio_fonts(&cc.egui_ctx);
            Ok(Box::new(RadishFlowStudioApp::new()))
        }),
    )
}

fn studio_native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(STUDIO_INITIAL_WINDOW_SIZE)
            .with_min_inner_size(STUDIO_MIN_WINDOW_SIZE),
        ..Default::default()
    }
}

struct RadishFlowStudioApp {
    state: AppState,
}

#[allow(clippy::large_enum_variant)]
enum AppState {
    Ready(ReadyAppState),
    Failed(String),
}

struct ReadyAppState {
    platform_host: StudioGuiPlatformHost,
    platform_timer_executor: EguiPlatformTimerExecutor,
    command_palette: CommandPaletteState,
    project_open: ProjectOpenState,
    result_inspector: ResultInspectorState,
    canvas_object_filter: CanvasObjectListFilter,
    canvas_viewport_navigation: CanvasViewportNavigationState,
    canvas_command_result: Option<radishflow_studio::StudioGuiCanvasCommandResultViewModel>,
    project_file_picker: Box<dyn ProjectFilePicker>,
    preferences_path: PathBuf,
    locale: StudioShellLocale,
    last_area_focus: Option<StudioGuiWindowAreaId>,
    drag_session: Option<PanelDragSession>,
    active_drop_preview: Option<ActiveDropPreview>,
    drop_preview_overlay_anchor: Option<DropPreviewOverlayAnchor>,
    last_viewport_focused: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PanelDragSession {
    area_id: StudioGuiWindowAreaId,
    window_id: Option<StudioWindowHostId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActiveDropPreview {
    window_id: Option<StudioWindowHostId>,
    query: StudioGuiWindowDropTargetQuery,
}

#[derive(Debug, Clone, Copy)]
struct DropPreviewOverlayAnchor {
    rect: egui::Rect,
    priority: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CommandPaletteState {
    open: bool,
    query: String,
    selected_index: usize,
    focus_query_input: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ProjectOpenState {
    path_input: String,
    recent_projects: Vec<PathBuf>,
    notice: Option<ProjectOpenNotice>,
    pending_confirmation: Option<ProjectOpenRequest>,
    pending_save_as_overwrite: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectOpenNotice {
    level: ProjectOpenNoticeLevel,
    title: String,
    detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectOpenNoticeLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectOpenRequest {
    project_path: PathBuf,
    source_label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResultInspectorState {
    snapshot_id: Option<String>,
    selected_stream_id: Option<String>,
    comparison_stream_id: Option<String>,
    selected_unit_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CanvasViewportNavigationState {
    active_anchor: Option<CanvasViewportAnchorNavigation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanvasViewportAnchorNavigation {
    anchor_label: String,
    pending_scroll: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum CanvasObjectListFilter {
    #[default]
    All,
    Attention,
    Units,
    Streams,
}

#[derive(Debug, Default)]
struct EguiPlatformTimerExecutor {
    next_native_timer_id: StudioGuiPlatformNativeTimerId,
    active_native_timers: BTreeMap<StudioGuiPlatformNativeTimerId, EguiNativeTimerRegistration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EguiNativeTimerRegistration {
    schedule: radishflow_studio::StudioGuiNativeTimerSchedule,
}

impl RadishFlowStudioApp {
    fn new() -> Self {
        let config = studio_shell_runtime_config(None);
        let preferences_path = default_studio_preferences_path();
        let state = match ReadyAppState::from_config(&config, preferences_path) {
            Ok(ready) => AppState::Ready(ready),
            Err(error) => AppState::Failed(format!(
                "Studio 初始化失败 [{}]: {}",
                error.code().as_str(),
                error.message()
            )),
        };

        Self { state }
    }
}

fn studio_shell_runtime_config(project_path: Option<PathBuf>) -> StudioRuntimeConfig {
    let mut config = StudioRuntimeConfig {
        entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
        ..StudioRuntimeConfig::default()
    };
    if let Some(project_path) = project_path {
        config.project_path = project_path;
    }
    config
}

impl ReadyAppState {
    fn from_config(config: &StudioRuntimeConfig, preferences_path: PathBuf) -> RfResult<Self> {
        Self::from_config_with_project_file_picker(
            config,
            preferences_path,
            Box::<NativeProjectFilePicker>::default(),
        )
    }

    fn from_config_with_project_file_picker(
        config: &StudioRuntimeConfig,
        preferences_path: PathBuf,
        project_file_picker: Box<dyn ProjectFilePicker>,
    ) -> RfResult<Self> {
        let (recent_projects, preferences_notice) =
            match load_recent_project_paths(&preferences_path) {
                Ok(recent_projects) => (recent_projects, None),
                Err(error) => (
                    Vec::new(),
                    Some(ProjectOpenNotice {
                        level: ProjectOpenNoticeLevel::Warning,
                        title: "Recent projects not loaded".to_string(),
                        detail: format!(
                            "[{}] {} ({})",
                            error.code().as_str(),
                            error.message(),
                            preferences_path.display()
                        ),
                    }),
                ),
            };
        let mut ready = ReadyAppState {
            platform_host: StudioGuiPlatformHost::new(config)?,
            platform_timer_executor: EguiPlatformTimerExecutor::default(),
            command_palette: CommandPaletteState::default(),
            project_open: ProjectOpenState::from_path_and_recent(
                &config.project_path,
                recent_projects,
            ),
            result_inspector: ResultInspectorState::default(),
            canvas_object_filter: CanvasObjectListFilter::default(),
            canvas_viewport_navigation: CanvasViewportNavigationState::default(),
            canvas_command_result: None,
            project_file_picker,
            preferences_path,
            locale: StudioShellLocale::default(),
            last_area_focus: None,
            drag_session: None,
            active_drop_preview: None,
            drop_preview_overlay_anchor: None,
            last_viewport_focused: None,
        };
        if let Some(notice) = preferences_notice {
            ready.project_open.notice = Some(notice);
        }
        ready.dispatch_event(StudioGuiEvent::OpenWindowRequested);
        ready.apply_default_hidden_commands_panel_for_current_window()?;
        Ok(ready)
    }
}

impl ResultInspectorState {
    fn selected_stream_id_for_snapshot(
        &mut self,
        snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    ) -> Option<String> {
        if self.snapshot_id.as_deref() != Some(snapshot.snapshot_id.as_str()) {
            self.snapshot_id = Some(snapshot.snapshot_id.clone());
            self.selected_stream_id = None;
            self.comparison_stream_id = None;
            self.selected_unit_id = None;
        }

        if self
            .selected_stream_id
            .as_deref()
            .is_some_and(|selected_id| {
                !snapshot
                    .streams
                    .iter()
                    .any(|stream| stream.stream_id == selected_id)
            })
        {
            self.selected_stream_id = None;
        }
        if self
            .comparison_stream_id
            .as_deref()
            .is_some_and(|comparison_id| {
                !snapshot
                    .streams
                    .iter()
                    .any(|stream| stream.stream_id == comparison_id)
                    || self
                        .selected_stream_id
                        .as_deref()
                        .map(|selected_id| selected_id == comparison_id)
                        .unwrap_or(false)
            })
        {
            self.comparison_stream_id = None;
        }

        if self.selected_stream_id.is_none() {
            self.selected_stream_id = snapshot
                .streams
                .first()
                .map(|stream| stream.stream_id.clone());
        }
        if self.comparison_stream_id.as_deref() == self.selected_stream_id.as_deref() {
            self.comparison_stream_id = None;
        }

        self.selected_stream_id.clone()
    }

    fn selected_unit_id_for_snapshot(
        &mut self,
        snapshot: &radishflow_studio::StudioGuiWindowSolveSnapshotModel,
    ) -> Option<String> {
        if self.snapshot_id.as_deref() != Some(snapshot.snapshot_id.as_str()) {
            // selected_stream_id_for_snapshot is expected to run first and
            // already reset selections; this branch keeps the helper safe to
            // call independently.
            self.snapshot_id = Some(snapshot.snapshot_id.clone());
            self.selected_stream_id = None;
            self.comparison_stream_id = None;
            self.selected_unit_id = None;
        }

        if self
            .selected_unit_id
            .as_deref()
            .is_some_and(|selected_unit| {
                !snapshot
                    .steps
                    .iter()
                    .any(|step| step.unit_id == selected_unit)
            })
        {
            self.selected_unit_id = None;
        }

        if self.selected_unit_id.is_none() {
            self.selected_unit_id = snapshot.steps.first().map(|step| step.unit_id.clone());
        }

        self.selected_unit_id.clone()
    }

    fn select_stream(&mut self, snapshot_id: &str, stream_id: impl Into<String>) {
        let stream_id = stream_id.into();
        self.snapshot_id = Some(snapshot_id.to_string());
        if self.comparison_stream_id.as_deref() == Some(stream_id.as_str()) {
            self.comparison_stream_id = None;
        }
        self.selected_stream_id = Some(stream_id);
    }

    fn select_comparison_stream(&mut self, snapshot_id: &str, stream_id: impl Into<String>) {
        self.snapshot_id = Some(snapshot_id.to_string());
        self.comparison_stream_id = Some(stream_id.into());
    }

    fn select_unit(&mut self, snapshot_id: &str, unit_id: impl Into<String>) {
        self.snapshot_id = Some(snapshot_id.to_string());
        self.selected_unit_id = Some(unit_id.into());
    }

    fn reset(&mut self) {
        self.snapshot_id = None;
        self.selected_stream_id = None;
        self.comparison_stream_id = None;
        self.selected_unit_id = None;
    }
}

impl CanvasViewportNavigationState {
    fn request_anchor(&mut self, anchor_label: impl Into<String>) -> String {
        let anchor_label = anchor_label.into();
        self.active_anchor = Some(CanvasViewportAnchorNavigation {
            anchor_label: anchor_label.clone(),
            pending_scroll: true,
        });
        anchor_label
    }

    fn request_for_command(
        &mut self,
        command_id: &str,
        focus: Option<&radishflow_studio::StudioGuiCanvasViewportFocusViewModel>,
    ) -> Option<String> {
        let Some(focus) = focus else {
            self.active_anchor = None;
            return None;
        };
        if focus.command_id != command_id {
            self.active_anchor = None;
            return None;
        }

        self.active_anchor = Some(CanvasViewportAnchorNavigation {
            anchor_label: focus.anchor_label.clone(),
            pending_scroll: true,
        });
        Some(focus.anchor_label.clone())
    }

    fn reconcile(
        &mut self,
        focus: Option<&radishflow_studio::StudioGuiCanvasViewportFocusViewModel>,
    ) -> Option<String> {
        let active = self.active_anchor.as_ref()?;
        let still_current = focus
            .map(|focus| focus.anchor_label == active.anchor_label)
            .unwrap_or(false);
        if !still_current {
            let anchor_label = active.anchor_label.clone();
            self.active_anchor = None;
            return Some(anchor_label);
        }
        None
    }

    fn is_active_anchor(&self, anchor_label: &str) -> bool {
        self.active_anchor
            .as_ref()
            .map(|focus| focus.anchor_label == anchor_label)
            .unwrap_or(false)
    }

    fn take_pending_scroll_for_anchor(&mut self, anchor_label: &str) -> bool {
        let Some(active) = self.active_anchor.as_mut() else {
            return false;
        };
        if active.anchor_label == anchor_label && active.pending_scroll {
            active.pending_scroll = false;
            return true;
        }
        false
    }
}

impl CanvasObjectListFilter {
    fn from_filter_id(filter_id: &str) -> Option<Self> {
        match filter_id {
            "all" => Some(Self::All),
            "attention" => Some(Self::Attention),
            "units" => Some(Self::Units),
            "streams" => Some(Self::Streams),
            _ => None,
        }
    }

    fn filter_id(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Attention => "attention",
            Self::Units => "units",
            Self::Streams => "streams",
        }
    }

    fn matches(self, item: &radishflow_studio::StudioGuiCanvasObjectListItemViewModel) -> bool {
        match self {
            Self::All => true,
            Self::Attention => !item.status_badges.is_empty(),
            Self::Units => item.kind_label == "Unit",
            Self::Streams => item.kind_label == "Stream",
        }
    }
}

impl CommandPaletteState {
    fn toggle(&mut self) {
        if self.open {
            self.close();
        } else {
            self.open();
        }
    }

    fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.selected_index = 0;
        self.focus_query_input = true;
    }

    fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.selected_index = 0;
        self.focus_query_input = false;
    }

    fn sync_selection<T: PaletteSelectable>(&mut self, commands: &[T]) {
        self.selected_index = normalized_palette_selection(commands, self.selected_index);
    }

    fn move_selection<T: PaletteSelectable>(&mut self, delta: isize, commands: &[T]) {
        self.selected_index = moved_palette_selection(commands, self.selected_index, delta);
    }
}

impl ProjectOpenState {
    const MAX_RECENT_PROJECTS: usize = 8;

    fn from_path_and_recent(path: &std::path::Path, recent_projects: Vec<PathBuf>) -> Self {
        let mut state = Self {
            path_input: path.display().to_string(),
            recent_projects: Vec::new(),
            notice: None,
            pending_confirmation: None,
            pending_save_as_overwrite: None,
        };
        state.replace_recent_projects(recent_projects);
        state
    }

    fn replace_recent_projects(&mut self, recent_projects: Vec<PathBuf>) {
        self.recent_projects.clear();
        for project_path in recent_projects.into_iter().rev() {
            self.record_recent_project(project_path);
        }
    }

    fn current_path(&self) -> Option<PathBuf> {
        let trimmed = self.path_input.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(PathBuf::from(trimmed))
        }
    }

    fn record_recent_project(&mut self, project_path: PathBuf) {
        self.recent_projects
            .retain(|recent| !paths_match(recent, &project_path));
        self.recent_projects.insert(0, project_path);
        self.recent_projects.truncate(Self::MAX_RECENT_PROJECTS);
    }
}

fn paths_match(left: &std::path::Path, right: &std::path::Path) -> bool {
    left == right
        || left
            .canonicalize()
            .ok()
            .zip(right.canonicalize().ok())
            .map(|(left, right)| left == right)
            .unwrap_or(false)
}

impl eframe::App for RadishFlowStudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let panic_message = match &mut self.state {
            AppState::Failed(message) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("RadishFlow Studio");
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(180, 40, 40), message);
                });
                None
            }
            AppState::Ready(app) => panic::catch_unwind(AssertUnwindSafe(|| app.update(ctx)))
                .err()
                .map(panic_payload_message),
        };

        if let Some(message) = panic_message {
            eprintln!("[radishflow-studio] fatal gui panic: {message}");
            self.state = AppState::Failed(format!(
                "GUI event failed with an internal panic. See stderr for details. {message}"
            ));
        }
    }
}

fn panic_payload_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    "panic payload is not a string".to_string()
}
