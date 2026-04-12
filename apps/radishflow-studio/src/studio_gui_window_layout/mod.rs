use serde::{Deserialize, Serialize};

use crate::{
    StudioAppHostState, StudioGuiSnapshot, StudioGuiWindowCanvasAreaModel,
    StudioGuiWindowCommandAreaModel, StudioGuiWindowHeaderModel, StudioGuiWindowModel,
    StudioGuiWindowRuntimeAreaModel, StudioWindowHostId, StudioWindowHostRole,
};

mod helpers;
mod model;
mod state;

#[cfg(test)]
mod tests;

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
    pub layout_slot: Option<u16>,
    pub layout_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudioGuiWindowPanelLayoutState {
    pub area_id: StudioGuiWindowAreaId,
    pub dock_region: StudioGuiWindowDockRegion,
    pub stack_group: u8,
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
pub struct StudioGuiWindowStackGroupState {
    pub dock_region: StudioGuiWindowDockRegion,
    pub stack_group: u8,
    pub active_area_id: StudioGuiWindowAreaId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudioGuiWindowLayoutState {
    pub scope: StudioGuiWindowLayoutScope,
    pub panels: Vec<StudioGuiWindowPanelLayoutState>,
    pub stack_groups: Vec<StudioGuiWindowStackGroupState>,
    pub region_weights: Vec<StudioGuiWindowRegionWeight>,
    pub center_area: StudioGuiWindowAreaId,
    pub default_focus_area: StudioGuiWindowAreaId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowLayoutPersistenceState {
    pub layout_key: String,
    pub center_area: StudioGuiWindowAreaId,
    pub panels: Vec<StudioGuiWindowPanelLayoutState>,
    pub stack_groups: Vec<StudioGuiWindowStackGroupState>,
    pub region_weights: Vec<StudioGuiWindowRegionWeight>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudioGuiWindowDockPlacement {
    Start,
    End,
    Before {
        anchor_area_id: StudioGuiWindowAreaId,
    },
    After {
        anchor_area_id: StudioGuiWindowAreaId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudioGuiWindowDropTargetKind {
    DockRegionGroup,
    StackTab,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudioGuiWindowDropTarget {
    pub area_id: StudioGuiWindowAreaId,
    pub kind: StudioGuiWindowDropTargetKind,
    pub dock_region: StudioGuiWindowDockRegion,
    pub placement: StudioGuiWindowDockPlacement,
    pub anchor_area_id: Option<StudioGuiWindowAreaId>,
    pub source_dock_region: StudioGuiWindowDockRegion,
    pub source_stack_group: u8,
    pub target_stack_group: u8,
    pub target_group_index: usize,
    pub target_tab_index: usize,
    pub creates_new_stack: bool,
    pub merges_into_existing_stack: bool,
    pub preview_active_area_id: StudioGuiWindowAreaId,
    pub preview_area_ids: Vec<StudioGuiWindowAreaId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudioGuiWindowDropTargetQuery {
    DockRegion {
        area_id: StudioGuiWindowAreaId,
        dock_region: StudioGuiWindowDockRegion,
        placement: StudioGuiWindowDockPlacement,
    },
    Stack {
        area_id: StudioGuiWindowAreaId,
        anchor_area_id: StudioGuiWindowAreaId,
        placement: StudioGuiWindowDockPlacement,
    },
    CurrentStack {
        area_id: StudioGuiWindowAreaId,
        placement: StudioGuiWindowDockPlacement,
    },
    Unstack {
        area_id: StudioGuiWindowAreaId,
        placement: StudioGuiWindowDockPlacement,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudioGuiWindowLayoutMutation {
    SetCenterArea {
        area_id: StudioGuiWindowAreaId,
    },
    SetActivePanelInStack {
        area_id: StudioGuiWindowAreaId,
    },
    ActivateNextPanelInStack {
        area_id: StudioGuiWindowAreaId,
    },
    ActivatePreviousPanelInStack {
        area_id: StudioGuiWindowAreaId,
    },
    MovePanelWithinStack {
        area_id: StudioGuiWindowAreaId,
        placement: StudioGuiWindowDockPlacement,
    },
    StackPanelWith {
        area_id: StudioGuiWindowAreaId,
        anchor_area_id: StudioGuiWindowAreaId,
        placement: StudioGuiWindowDockPlacement,
    },
    UnstackPanelFromGroup {
        area_id: StudioGuiWindowAreaId,
        placement: StudioGuiWindowDockPlacement,
    },
    PlacePanelInDockRegion {
        area_id: StudioGuiWindowAreaId,
        dock_region: StudioGuiWindowDockRegion,
        placement: StudioGuiWindowDockPlacement,
    },
    SetPanelDockRegion {
        area_id: StudioGuiWindowAreaId,
        dock_region: StudioGuiWindowDockRegion,
        order: Option<u8>,
    },
    SetPanelVisibility {
        area_id: StudioGuiWindowAreaId,
        visible: bool,
    },
    SetPanelCollapsed {
        area_id: StudioGuiWindowAreaId,
        collapsed: bool,
    },
    SetPanelOrder {
        area_id: StudioGuiWindowAreaId,
        order: u8,
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
    pub stack_group: u8,
    pub order: u8,
    pub display_mode: StudioGuiWindowPanelDisplayMode,
    pub active_in_stack: bool,
    pub visible: bool,
    pub collapsed: bool,
    pub badge: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiWindowPanelDisplayMode {
    Standalone,
    ActiveTab,
    InactiveTab,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowStackTabLayout {
    pub area_id: StudioGuiWindowAreaId,
    pub title: &'static str,
    pub active: bool,
    pub visible: bool,
    pub collapsed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowStackGroupLayout {
    pub dock_region: StudioGuiWindowDockRegion,
    pub stack_group: u8,
    pub tabbed: bool,
    pub active_area_id: StudioGuiWindowAreaId,
    pub tabs: Vec<StudioGuiWindowStackTabLayout>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowLayoutModel {
    pub state: StudioGuiWindowLayoutState,
    pub titlebar: StudioGuiWindowTitlebarModel,
    pub panels: Vec<StudioGuiWindowPanelLayout>,
    pub stack_groups: Vec<StudioGuiWindowStackGroupLayout>,
    pub region_weights: Vec<StudioGuiWindowRegionWeight>,
    pub center_area: StudioGuiWindowAreaId,
    pub default_focus_area: StudioGuiWindowAreaId,
}
