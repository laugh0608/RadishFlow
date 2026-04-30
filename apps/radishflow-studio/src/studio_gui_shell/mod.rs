use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::studio_gui_preferences_store::{
    default_studio_preferences_path, load_recent_project_paths,
};
use eframe::egui;
use radishflow_studio::{
    StudioAppHostWindowState, StudioGuiCommandEntry, StudioGuiCommandMenuCommandModel,
    StudioGuiCommandMenuNode, StudioGuiEvent, StudioGuiFocusContext,
    StudioGuiPlatformExecutedDispatch, StudioGuiPlatformExecutedNativeTimerCallbackBatch,
    StudioGuiPlatformExecutedNativeTimerCallbackOutcome, StudioGuiPlatformHost,
    StudioGuiPlatformNativeTimerId, StudioGuiPlatformTimerCommand, StudioGuiPlatformTimerExecutor,
    StudioGuiPlatformTimerExecutorResponse, StudioGuiPlatformTimerFollowUpCommand,
    StudioGuiShortcut, StudioGuiShortcutKey, StudioGuiShortcutModifier, StudioGuiWindowAreaId,
    StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion, StudioGuiWindowDropTargetQuery,
    StudioGuiWindowLayoutModel, StudioGuiWindowLayoutMutation, StudioGuiWindowModel,
    StudioGuiWindowPanelDisplayMode, StudioGuiWindowStackGroupLayout,
    StudioGuiWindowToolbarSectionModel, StudioRuntimeConfig, StudioRuntimeTrigger,
    StudioWindowHostId, StudioWindowHostRole,
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

pub fn run() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "RadishFlow Studio",
        native_options,
        Box::new(|cc| {
            fonts::configure_studio_fonts(&cc.egui_ctx);
            Ok(Box::new(RadishFlowStudioApp::new()))
        }),
    )
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
        let config = StudioRuntimeConfig::default();
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

        if self.selected_stream_id.is_none() {
            self.selected_stream_id = snapshot
                .streams
                .first()
                .map(|stream| stream.stream_id.clone());
        }

        self.selected_stream_id.clone()
    }

    fn select_stream(&mut self, snapshot_id: &str, stream_id: impl Into<String>) {
        self.snapshot_id = Some(snapshot_id.to_string());
        self.selected_stream_id = Some(stream_id.into());
    }

    fn reset(&mut self) {
        self.snapshot_id = None;
        self.selected_stream_id = None;
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
        match &mut self.state {
            AppState::Failed(message) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("RadishFlow Studio");
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(180, 40, 40), message);
                });
            }
            AppState::Ready(app) => app.update(ctx),
        }
    }
}
