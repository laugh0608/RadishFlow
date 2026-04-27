use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use eframe::egui;
use radishflow_studio::{
    StudioAppHostWindowState, StudioGuiCommandEntry, StudioGuiCommandMenuCommandModel,
    StudioGuiCommandMenuNode, StudioGuiEvent, StudioGuiFocusContext,
    StudioGuiPlatformExecutedNativeTimerCallbackBatch,
    StudioGuiPlatformExecutedNativeTimerCallbackOutcome, StudioGuiPlatformHost,
    StudioGuiPlatformNativeTimerId, StudioGuiPlatformTimerCommand, StudioGuiPlatformTimerExecutor,
    StudioGuiPlatformTimerExecutorResponse, StudioGuiPlatformTimerFollowUpCommand,
    StudioGuiShortcut, StudioGuiShortcutKey, StudioGuiShortcutModifier, StudioGuiWindowAreaId,
    StudioGuiWindowDockPlacement, StudioGuiWindowDockRegion, StudioGuiWindowDropTargetQuery,
    StudioGuiWindowLayoutModel, StudioGuiWindowLayoutMutation, StudioGuiWindowModel,
    StudioGuiWindowPanelDisplayMode, StudioGuiWindowStackGroupLayout,
    StudioGuiWindowToolbarSectionModel, StudioRuntimeConfig, StudioWindowHostId,
    StudioWindowHostRole,
};
use rf_types::RfResult;
use rf_ui::{
    RunPanelIntent, RunPanelNoticeLevel, RunPanelRecoveryWidgetEvent, RunPanelWidgetEvent,
    SimulationMode,
};

mod app;
mod chrome;
mod panels;
mod utils;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod timer_tests;

use self::utils::*;

pub fn run() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "RadishFlow Studio",
        native_options,
        Box::new(|_cc| Ok(Box::new(RadishFlowStudioApp::new()))),
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
        let state = match StudioGuiPlatformHost::new(&config) {
            Ok(platform_host) => {
                let mut ready = ReadyAppState {
                    platform_host,
                    platform_timer_executor: EguiPlatformTimerExecutor::default(),
                    command_palette: CommandPaletteState::default(),
                    last_area_focus: None,
                    drag_session: None,
                    active_drop_preview: None,
                    drop_preview_overlay_anchor: None,
                    last_viewport_focused: None,
                };
                ready.dispatch_event(StudioGuiEvent::OpenWindowRequested);
                AppState::Ready(ready)
            }
            Err(error) => AppState::Failed(format!(
                "Studio 初始化失败 [{}]: {}",
                error.code().as_str(),
                error.message()
            )),
        };

        Self { state }
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
