use serde::{Deserialize, Serialize};

use crate::{
    StudioAppHostState, StudioGuiSnapshot, StudioGuiWindowCanvasAreaModel,
    StudioGuiWindowCommandAreaModel, StudioGuiWindowHeaderModel, StudioGuiWindowModel,
    StudioGuiWindowRuntimeAreaModel, StudioWindowHostId, StudioWindowHostRole,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudioGuiWindowAreaId {
    Commands,
    Canvas,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudioGuiWindowDockRegion {
    LeftSidebar,
    CenterStage,
    RightSidebar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudioGuiWindowLayoutScopeKind {
    EmptyWorkspace,
    Window,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudioGuiWindowLayoutScope {
    pub kind: StudioGuiWindowLayoutScopeKind,
    pub window_id: Option<StudioWindowHostId>,
    pub window_role: Option<StudioWindowHostRole>,
    pub layout_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudioGuiWindowPanelLayoutState {
    pub area_id: StudioGuiWindowAreaId,
    pub dock_region: StudioGuiWindowDockRegion,
    pub order: u8,
    pub visible: bool,
    pub collapsed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudioGuiWindowRegionWeight {
    pub dock_region: StudioGuiWindowDockRegion,
    pub weight: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudioGuiWindowLayoutState {
    pub scope: StudioGuiWindowLayoutScope,
    pub panels: Vec<StudioGuiWindowPanelLayoutState>,
    pub region_weights: Vec<StudioGuiWindowRegionWeight>,
    pub center_area: StudioGuiWindowAreaId,
    pub default_focus_area: StudioGuiWindowAreaId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudioGuiWindowLayoutMutation {
    SetPanelVisibility {
        area_id: StudioGuiWindowAreaId,
        visible: bool,
    },
    SetPanelCollapsed {
        area_id: StudioGuiWindowAreaId,
        collapsed: bool,
    },
    SetRegionWeight {
        dock_region: StudioGuiWindowDockRegion,
        weight: u16,
    },
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
    pub state: StudioGuiWindowLayoutState,
    pub titlebar: StudioGuiWindowTitlebarModel,
    pub panels: Vec<StudioGuiWindowPanelLayout>,
    pub region_weights: Vec<StudioGuiWindowRegionWeight>,
    pub center_area: StudioGuiWindowAreaId,
    pub default_focus_area: StudioGuiWindowAreaId,
}

impl Default for StudioGuiWindowLayoutState {
    fn default() -> Self {
        Self {
            scope: StudioGuiWindowLayoutScope {
                kind: StudioGuiWindowLayoutScopeKind::EmptyWorkspace,
                window_id: None,
                window_role: None,
                layout_key: "studio.window.empty".to_string(),
            },
            panels: default_panel_states(),
            region_weights: default_region_weights(),
            center_area: StudioGuiWindowAreaId::Canvas,
            default_focus_area: StudioGuiWindowAreaId::Commands,
        }
    }
}

impl StudioGuiWindowLayoutState {
    pub fn from_snapshot(snapshot: &StudioGuiSnapshot) -> Self {
        Self::from_snapshot_for_window(snapshot, None)
    }

    pub fn from_snapshot_for_window(
        snapshot: &StudioGuiSnapshot,
        window_id: Option<StudioWindowHostId>,
    ) -> Self {
        let default_focus_area = if snapshot.canvas.view().focused_suggestion_id.is_some() {
            StudioGuiWindowAreaId::Canvas
        } else if snapshot
            .command_registry
            .sections
            .iter()
            .flat_map(|section| section.commands.iter())
            .any(|command| command.enabled)
        {
            StudioGuiWindowAreaId::Commands
        } else {
            StudioGuiWindowAreaId::Runtime
        };

        Self {
            scope: layout_scope_from_state(&snapshot.app_host_state, window_id),
            panels: default_panel_states(),
            region_weights: default_region_weights(),
            center_area: StudioGuiWindowAreaId::Canvas,
            default_focus_area,
        }
    }

    pub fn panel(
        &self,
        area_id: StudioGuiWindowAreaId,
    ) -> Option<&StudioGuiWindowPanelLayoutState> {
        self.panels.iter().find(|panel| panel.area_id == area_id)
    }

    pub fn region_weight(
        &self,
        dock_region: StudioGuiWindowDockRegion,
    ) -> Option<&StudioGuiWindowRegionWeight> {
        self.region_weights
            .iter()
            .find(|region| region.dock_region == dock_region)
    }

    pub fn merged_with_persisted(&self, persisted: &Self) -> Self {
        let mut merged = self.clone();

        for panel in &persisted.panels {
            upsert_panel_state(&mut merged.panels, panel.clone());
        }
        for region in &persisted.region_weights {
            upsert_region_weight(&mut merged.region_weights, region.clone());
        }

        merged.center_area = persisted.center_area;
        merged
    }

    pub fn applying_mutation(&self, mutation: &StudioGuiWindowLayoutMutation) -> Self {
        let mut next = self.clone();
        match mutation {
            StudioGuiWindowLayoutMutation::SetPanelVisibility { area_id, visible } => {
                let mut panel = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                panel.visible = *visible;
                upsert_panel_state(&mut next.panels, panel);
            }
            StudioGuiWindowLayoutMutation::SetPanelCollapsed { area_id, collapsed } => {
                let mut panel = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                panel.collapsed = *collapsed;
                upsert_panel_state(&mut next.panels, panel);
            }
            StudioGuiWindowLayoutMutation::SetRegionWeight {
                dock_region,
                weight,
            } => {
                upsert_region_weight(
                    &mut next.region_weights,
                    StudioGuiWindowRegionWeight {
                        dock_region: *dock_region,
                        weight: (*weight).max(1),
                    },
                );
            }
        }
        next
    }
}

impl StudioGuiWindowLayoutModel {
    pub fn from_window_model(window: &StudioGuiWindowModel) -> Self {
        Self::from_areas(
            &window.layout_state,
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
        state: &StudioGuiWindowLayoutState,
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
            build_panel_layout(
                state,
                StudioGuiWindowAreaId::Commands,
                commands.title,
                Some(commands.total_command_count.to_string()),
                format!(
                    "{} commands, {} enabled",
                    commands.total_command_count, commands.enabled_command_count
                ),
            ),
            build_panel_layout(
                state,
                StudioGuiWindowAreaId::Canvas,
                canvas.title,
                Some(canvas.suggestion_count.to_string()),
                format!(
                    "{} suggestions, {} actions enabled",
                    canvas.suggestion_count, canvas.enabled_action_count
                ),
            ),
            build_panel_layout(
                state,
                StudioGuiWindowAreaId::Runtime,
                runtime.title,
                Some(runtime.log_entries.len().to_string()),
                format!(
                    "status={:?}, logs={}, entitlement={}",
                    runtime.control_state.run_status,
                    runtime.log_entries.len(),
                    if runtime.entitlement_host.is_some() {
                        "attached"
                    } else {
                        "none"
                    }
                ),
            ),
        ];

        Self {
            state: state.clone(),
            titlebar,
            panels,
            region_weights: state.region_weights.clone(),
            center_area: state.center_area,
            default_focus_area: state.default_focus_area,
        }
    }
}

impl StudioGuiWindowModel {
    pub fn layout(&self) -> StudioGuiWindowLayoutModel {
        StudioGuiWindowLayoutModel::from_window_model(self)
    }
}

fn build_panel_layout(
    state: &StudioGuiWindowLayoutState,
    area_id: StudioGuiWindowAreaId,
    title: &'static str,
    badge: Option<String>,
    summary: String,
) -> StudioGuiWindowPanelLayout {
    let panel_state = state
        .panel(area_id)
        .cloned()
        .unwrap_or_else(|| default_panel_state(area_id));
    StudioGuiWindowPanelLayout {
        area_id,
        title,
        dock_region: panel_state.dock_region,
        order: panel_state.order,
        visible: panel_state.visible,
        collapsed: panel_state.collapsed,
        badge,
        summary,
    }
}

fn layout_scope_from_state(
    state: &StudioAppHostState,
    requested_window_id: Option<StudioWindowHostId>,
) -> StudioGuiWindowLayoutScope {
    let window_id = requested_window_id
        .filter(|window_id| state.window(*window_id).is_some())
        .or(state.foreground_window_id)
        .or_else(|| state.registered_windows.first().copied());

    match window_id {
        Some(window_id) => {
            let window_role = state.window(window_id).map(|window| window.role);
            StudioGuiWindowLayoutScope {
                kind: StudioGuiWindowLayoutScopeKind::Window,
                window_id: Some(window_id),
                window_role,
                layout_key: layout_key_for_window(window_id, window_role),
            }
        }
        None => StudioGuiWindowLayoutScope {
            kind: StudioGuiWindowLayoutScopeKind::EmptyWorkspace,
            window_id: None,
            window_role: None,
            layout_key: "studio.window.empty".to_string(),
        },
    }
}

fn layout_key_for_window(
    window_id: StudioWindowHostId,
    window_role: Option<StudioWindowHostRole>,
) -> String {
    match window_role {
        Some(StudioWindowHostRole::EntitlementTimerOwner) => {
            format!("studio.window.owner.{window_id}")
        }
        Some(StudioWindowHostRole::Observer) => format!("studio.window.observer.{window_id}"),
        None => format!("studio.window.window.{window_id}"),
    }
}

fn default_panel_states() -> Vec<StudioGuiWindowPanelLayoutState> {
    vec![
        default_panel_state(StudioGuiWindowAreaId::Commands),
        default_panel_state(StudioGuiWindowAreaId::Canvas),
        default_panel_state(StudioGuiWindowAreaId::Runtime),
    ]
}

fn default_panel_state(area_id: StudioGuiWindowAreaId) -> StudioGuiWindowPanelLayoutState {
    match area_id {
        StudioGuiWindowAreaId::Commands => StudioGuiWindowPanelLayoutState {
            area_id,
            dock_region: StudioGuiWindowDockRegion::LeftSidebar,
            order: 10,
            visible: true,
            collapsed: false,
        },
        StudioGuiWindowAreaId::Canvas => StudioGuiWindowPanelLayoutState {
            area_id,
            dock_region: StudioGuiWindowDockRegion::CenterStage,
            order: 20,
            visible: true,
            collapsed: false,
        },
        StudioGuiWindowAreaId::Runtime => StudioGuiWindowPanelLayoutState {
            area_id,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            order: 30,
            visible: true,
            collapsed: false,
        },
    }
}

fn default_region_weights() -> Vec<StudioGuiWindowRegionWeight> {
    vec![
        StudioGuiWindowRegionWeight {
            dock_region: StudioGuiWindowDockRegion::LeftSidebar,
            weight: 24,
        },
        StudioGuiWindowRegionWeight {
            dock_region: StudioGuiWindowDockRegion::CenterStage,
            weight: 52,
        },
        StudioGuiWindowRegionWeight {
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            weight: 24,
        },
    ]
}

fn upsert_panel_state(
    panels: &mut Vec<StudioGuiWindowPanelLayoutState>,
    panel: StudioGuiWindowPanelLayoutState,
) {
    if let Some(index) = panels.iter().position(|candidate| candidate.area_id == panel.area_id) {
        panels[index] = panel;
    } else {
        panels.push(panel);
        panels.sort_by_key(|candidate| candidate.order);
    }
}

fn upsert_region_weight(
    regions: &mut Vec<StudioGuiWindowRegionWeight>,
    region: StudioGuiWindowRegionWeight,
) {
    if let Some(index) = regions
        .iter()
        .position(|candidate| candidate.dock_region == region.dock_region)
    {
        regions[index] = region;
    } else {
        regions.push(region);
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
        StudioGuiWindowLayoutScopeKind, StudioRuntimeConfig,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
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
        assert_eq!(layout.state.scope.kind, StudioGuiWindowLayoutScopeKind::Window);
        assert_eq!(layout.state.scope.window_id, Some(1));
        assert_eq!(layout.state.scope.layout_key, "studio.window.owner.1");
        assert_eq!(
            layout
                .state
                .region_weight(StudioGuiWindowDockRegion::CenterStage)
                .map(|region| region.weight),
            Some(52)
        );

        let commands = layout
            .panel(StudioGuiWindowAreaId::Commands)
            .expect("expected commands panel");
        assert_eq!(commands.dock_region, StudioGuiWindowDockRegion::LeftSidebar);
        assert!(commands.visible);
        assert!(!commands.collapsed);
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
    fn studio_gui_window_layout_uses_distinct_scope_keys_for_different_windows() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let first = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected first open dispatch");
        let first_layout_key = first.window.layout_state.scope.layout_key.clone();

        let second = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected second open dispatch");
        let second_layout_key = second.window.layout_state.scope.layout_key.clone();

        assert_eq!(first_layout_key, "studio.window.owner.1");
        assert_eq!(second_layout_key, "studio.window.observer.2");
        assert_ne!(first_layout_key, second_layout_key);
        assert_eq!(second.snapshot.layout_state.scope.layout_key, "studio.window.owner.1");
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
        assert_eq!(layout.state.scope.kind, StudioGuiWindowLayoutScopeKind::EmptyWorkspace);
        assert_eq!(layout.state.scope.layout_key, "studio.window.empty");
        assert_eq!(
            layout
                .panel(StudioGuiWindowAreaId::Commands)
                .and_then(|panel| panel.badge.as_deref()),
            Some("5")
        );
    }
}
