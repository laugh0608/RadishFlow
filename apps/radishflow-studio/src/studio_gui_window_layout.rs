use crate::{
    StudioGuiWindowCanvasAreaModel, StudioGuiWindowCommandAreaModel, StudioGuiWindowHeaderModel,
    StudioGuiWindowModel, StudioGuiWindowRuntimeAreaModel, StudioWindowHostId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiWindowAreaId {
    Commands,
    Canvas,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiWindowDockRegion {
    LeftSidebar,
    CenterStage,
    RightSidebar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowTitlebarModel {
    pub title: String,
    pub subtitle: String,
    pub foreground_window_id: Option<StudioWindowHostId>,
    pub registered_window_count: usize,
    pub close_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowPanelLayout {
    pub area_id: StudioGuiWindowAreaId,
    pub title: &'static str,
    pub dock_region: StudioGuiWindowDockRegion,
    pub order: u8,
    pub visible: bool,
    pub collapsed: bool,
    pub badge: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowLayoutModel {
    pub titlebar: StudioGuiWindowTitlebarModel,
    pub panels: Vec<StudioGuiWindowPanelLayout>,
    pub center_area: StudioGuiWindowAreaId,
    pub default_focus_area: StudioGuiWindowAreaId,
}

impl StudioGuiWindowLayoutModel {
    pub fn from_window_model(window: &StudioGuiWindowModel) -> Self {
        Self::from_areas(
            &window.header,
            &window.commands,
            &window.canvas,
            &window.runtime,
        )
    }

    pub fn panel(&self, area_id: StudioGuiWindowAreaId) -> Option<&StudioGuiWindowPanelLayout> {
        self.panels.iter().find(|panel| panel.area_id == area_id)
    }

    fn from_areas(
        header: &StudioGuiWindowHeaderModel,
        commands: &StudioGuiWindowCommandAreaModel,
        canvas: &StudioGuiWindowCanvasAreaModel,
        runtime: &StudioGuiWindowRuntimeAreaModel,
    ) -> Self {
        let titlebar = StudioGuiWindowTitlebarModel {
            title: header.title.to_string(),
            subtitle: header.status_line.clone(),
            foreground_window_id: header.foreground_window_id,
            registered_window_count: header.registered_window_count,
            close_enabled: header.registered_window_count > 0,
        };
        let panels = vec![
            StudioGuiWindowPanelLayout {
                area_id: StudioGuiWindowAreaId::Commands,
                title: commands.title,
                dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                order: 10,
                visible: commands.total_command_count > 0,
                collapsed: false,
                badge: Some(commands.total_command_count.to_string()),
                summary: format!(
                    "{} commands, {} enabled",
                    commands.total_command_count, commands.enabled_command_count
                ),
            },
            StudioGuiWindowPanelLayout {
                area_id: StudioGuiWindowAreaId::Canvas,
                title: canvas.title,
                dock_region: StudioGuiWindowDockRegion::CenterStage,
                order: 20,
                visible: true,
                collapsed: false,
                badge: Some(canvas.suggestion_count.to_string()),
                summary: format!(
                    "{} suggestions, {} actions enabled",
                    canvas.suggestion_count, canvas.enabled_action_count
                ),
            },
            StudioGuiWindowPanelLayout {
                area_id: StudioGuiWindowAreaId::Runtime,
                title: runtime.title,
                dock_region: StudioGuiWindowDockRegion::RightSidebar,
                order: 30,
                visible: true,
                collapsed: false,
                badge: Some(runtime.log_entries.len().to_string()),
                summary: format!(
                    "status={:?}, logs={}, entitlement={}",
                    runtime.control_state.run_status,
                    runtime.log_entries.len(),
                    if runtime.entitlement_host.is_some() {
                        "attached"
                    } else {
                        "none"
                    }
                ),
            },
        ];
        let default_focus_area = if canvas.focused_suggestion_id.is_some() {
            StudioGuiWindowAreaId::Canvas
        } else if commands.enabled_command_count > 0 {
            StudioGuiWindowAreaId::Commands
        } else {
            StudioGuiWindowAreaId::Runtime
        };

        Self {
            titlebar,
            panels,
            center_area: StudioGuiWindowAreaId::Canvas,
            default_focus_area,
        }
    }
}

impl StudioGuiWindowModel {
    pub fn layout(&self) -> StudioGuiWindowLayoutModel {
        StudioGuiWindowLayoutModel::from_window_model(self)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        StudioGuiDriver, StudioGuiEvent, StudioGuiWindowAreaId, StudioGuiWindowDockRegion,
        StudioRuntimeConfig, StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
        StudioRuntimeEntitlementSessionEvent, StudioRuntimeTrigger,
    };

    fn lease_expiring_config() -> StudioRuntimeConfig {
        StudioRuntimeConfig {
            entitlement_preflight: StudioRuntimeEntitlementPreflight::Skip,
            entitlement_seed: StudioRuntimeEntitlementSeed::LeaseExpiringSoon,
            ..StudioRuntimeConfig::default()
        }
    }

    fn flash_drum_local_rules_config() -> (StudioRuntimeConfig, PathBuf) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected current timestamp")
            .as_nanos();
        let project_path = std::env::temp_dir().join(format!(
            "radishflow-studio-window-layout-{timestamp}.rfproj.json"
        ));
        let project =
            include_str!("../../../examples/flowsheets/feed-heater-flash.rfproj.json")
                .replacen(
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-heated\"",
                    "\"name\": \"inlet\",\n              \"direction\": \"inlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-liquid\"",
                    "\"name\": \"liquid\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                )
                .replacen(
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": \"stream-vapor\"",
                    "\"name\": \"vapor\",\n              \"direction\": \"outlet\",\n              \"kind\": \"material\",\n              \"stream_id\": null",
                    1,
                );
        fs::write(&project_path, project).expect("expected local rules project");

        (
            StudioRuntimeConfig {
                project_path: project_path.clone(),
                ..lease_expiring_config()
            },
            project_path,
        )
    }

    #[test]
    fn studio_gui_window_layout_maps_panels_into_dock_regions() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let layout = dispatch.window.layout();

        assert_eq!(layout.titlebar.title, "RadishFlow Studio");
        assert!(layout.titlebar.close_enabled);
        assert_eq!(layout.titlebar.registered_window_count, 1);
        assert_eq!(layout.titlebar.foreground_window_id, Some(1));
        assert_eq!(layout.center_area, StudioGuiWindowAreaId::Canvas);
        assert_eq!(layout.default_focus_area, StudioGuiWindowAreaId::Canvas);

        let commands = layout
            .panel(StudioGuiWindowAreaId::Commands)
            .expect("expected commands panel");
        assert_eq!(commands.dock_region, StudioGuiWindowDockRegion::LeftSidebar);
        assert!(commands.visible);
        assert_eq!(commands.badge.as_deref(), Some("5"));

        let canvas = layout
            .panel(StudioGuiWindowAreaId::Canvas)
            .expect("expected canvas panel");
        assert_eq!(canvas.dock_region, StudioGuiWindowDockRegion::CenterStage);
        assert_eq!(canvas.badge.as_deref(), Some("3"));
        assert!(canvas.summary.contains("3 suggestions"));

        let runtime = layout
            .panel(StudioGuiWindowAreaId::Runtime)
            .expect("expected runtime panel");
        assert_eq!(runtime.dock_region, StudioGuiWindowDockRegion::RightSidebar);
        assert!(runtime.summary.contains("status=Idle"));
        assert!(runtime.summary.contains("entitlement=attached"));

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn studio_gui_window_layout_disables_close_when_all_windows_are_closed() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            crate::StudioGuiDriverOutcome::HostCommand(
                crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
            ) => opened.registration.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };
        let _ = driver
            .dispatch_event(StudioGuiEvent::WindowTriggerRequested {
                window_id,
                trigger: StudioRuntimeTrigger::EntitlementSessionEvent(
                    StudioRuntimeEntitlementSessionEvent::TimerElapsed,
                ),
            })
            .expect("expected timer dispatch");
        let _ = driver
            .dispatch_event(StudioGuiEvent::CloseWindowRequested { window_id })
            .expect("expected close dispatch");

        let layout = driver.snapshot().window_model().layout();

        assert_eq!(layout.titlebar.registered_window_count, 0);
        assert!(!layout.titlebar.close_enabled);
        assert_eq!(layout.default_focus_area, StudioGuiWindowAreaId::Runtime);
        assert_eq!(
            layout
                .panel(StudioGuiWindowAreaId::Commands)
                .and_then(|panel| panel.badge.as_deref()),
            Some("5")
        );
    }
}
