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

impl Default for StudioGuiWindowLayoutState {
    fn default() -> Self {
        let panels = default_panel_states();
        Self {
            scope: StudioGuiWindowLayoutScope {
                kind: StudioGuiWindowLayoutScopeKind::EmptyWorkspace,
                window_id: None,
                window_role: None,
                layout_slot: None,
                layout_key: "studio.window.empty".to_string(),
            },
            stack_groups: default_stack_group_states(&panels),
            panels,
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
        let panels = default_panel_states();
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
            stack_groups: default_stack_group_states(&panels),
            panels,
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

    pub fn panels_in_dock_region(
        &self,
        dock_region: StudioGuiWindowDockRegion,
    ) -> Vec<&StudioGuiWindowPanelLayoutState> {
        let mut panels = self
            .panels
            .iter()
            .filter(|panel| panel.dock_region == dock_region)
            .collect::<Vec<_>>();
        panels.sort_by_key(|panel| {
            (
                panel.stack_group,
                panel.order,
                area_sort_rank(panel.area_id),
            )
        });
        panels
    }

    pub fn panels_in_stack_group(
        &self,
        dock_region: StudioGuiWindowDockRegion,
        stack_group: u8,
    ) -> Vec<&StudioGuiWindowPanelLayoutState> {
        let mut panels = self
            .panels
            .iter()
            .filter(|panel| panel.dock_region == dock_region && panel.stack_group == stack_group)
            .collect::<Vec<_>>();
        panels.sort_by_key(|panel| (panel.order, area_sort_rank(panel.area_id)));
        panels
    }

    pub fn stack_group(
        &self,
        dock_region: StudioGuiWindowDockRegion,
        stack_group: u8,
    ) -> Option<&StudioGuiWindowStackGroupState> {
        self.stack_groups.iter().find(|group| {
            group.dock_region == dock_region && group.stack_group == stack_group
        })
    }

    pub fn active_panel_in_stack(
        &self,
        dock_region: StudioGuiWindowDockRegion,
        stack_group: u8,
    ) -> Option<StudioGuiWindowAreaId> {
        self.stack_group(dock_region, stack_group)
            .map(|group| group.active_area_id)
    }

    pub fn region_weight(
        &self,
        dock_region: StudioGuiWindowDockRegion,
    ) -> Option<&StudioGuiWindowRegionWeight> {
        self.region_weights
            .iter()
            .find(|region| region.dock_region == dock_region)
    }

    pub fn merged_with_persisted(&self, persisted: &StudioGuiWindowLayoutPersistenceState) -> Self {
        let mut merged = self.clone();

        for panel in &persisted.panels {
            upsert_panel_state(&mut merged.panels, panel.clone());
        }
        for stack_group in &persisted.stack_groups {
            upsert_stack_group_state(&mut merged.stack_groups, stack_group.clone());
        }
        for region in &persisted.region_weights {
            upsert_region_weight(&mut merged.region_weights, region.clone());
        }

        merged.center_area = persisted.center_area;
        reconcile_stack_group_states(&mut merged);
        merged
    }

    pub fn applying_mutation(&self, mutation: &StudioGuiWindowLayoutMutation) -> Self {
        let mut next = self.clone();
        match mutation {
            StudioGuiWindowLayoutMutation::SetCenterArea { area_id } => {
                let mut panel = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                panel.visible = true;
                let dock_region = panel.dock_region;
                let stack_group = panel.stack_group;
                upsert_panel_state(&mut next.panels, panel);
                set_active_panel_in_stack(
                    &mut next.stack_groups,
                    dock_region,
                    stack_group,
                    *area_id,
                );
                next.center_area = *area_id;
            }
            StudioGuiWindowLayoutMutation::SetActivePanelInStack { area_id } => {
                let mut panel = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                panel.visible = true;
                let dock_region = panel.dock_region;
                let stack_group = panel.stack_group;
                upsert_panel_state(&mut next.panels, panel);
                set_active_panel_in_stack(
                    &mut next.stack_groups,
                    dock_region,
                    stack_group,
                    *area_id,
                );
                reconcile_center_area_with_active_panel(&mut next, *area_id);
            }
            StudioGuiWindowLayoutMutation::ActivateNextPanelInStack { area_id } => {
                if let Some(next_area_id) = adjacent_panel_in_stack(
                    &next.panels,
                    *area_id,
                    StackCycleDirection::Next,
                ) {
                    set_active_panel_for_area(&mut next, next_area_id);
                }
            }
            StudioGuiWindowLayoutMutation::ActivatePreviousPanelInStack { area_id } => {
                if let Some(previous_area_id) = adjacent_panel_in_stack(
                    &next.panels,
                    *area_id,
                    StackCycleDirection::Previous,
                ) {
                    set_active_panel_for_area(&mut next, previous_area_id);
                }
            }
            StudioGuiWindowLayoutMutation::StackPanelWith {
                area_id,
                anchor_area_id,
                placement,
            } => {
                let previous = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                let anchor = next
                    .panel(*anchor_area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*anchor_area_id));
                let mut panel = previous.clone();
                panel.dock_region = anchor.dock_region;
                panel.stack_group = anchor.stack_group;
                if anchor.dock_region == StudioGuiWindowDockRegion::CenterStage {
                    panel.visible = true;
                }
                upsert_panel_state(&mut next.panels, panel);
                place_panel_in_stack_group(
                    &mut next.panels,
                    *area_id,
                    anchor.dock_region,
                    anchor.stack_group,
                    *placement,
                );
                if previous.dock_region != anchor.dock_region
                    || previous.stack_group != anchor.stack_group
                {
                    normalize_stack_group(
                        &mut next.panels,
                        previous.dock_region,
                        previous.stack_group,
                    );
                }
                normalize_region_stack_groups(&mut next.panels, anchor.dock_region);
                if previous.dock_region != anchor.dock_region {
                    normalize_region_stack_groups(&mut next.panels, previous.dock_region);
                }
                if let Some((dock_region, stack_group)) = panel_stack_location(&next.panels, *area_id)
                {
                    set_active_panel_in_stack(
                        &mut next.stack_groups,
                        dock_region,
                        stack_group,
                        *area_id,
                    );
                }
                reconcile_center_stage_after_panel_move(&mut next, *area_id, anchor.dock_region);
            }
            StudioGuiWindowLayoutMutation::UnstackPanelFromGroup { area_id, placement } => {
                let previous = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                let stack_members = next.panels_in_stack_group(
                    previous.dock_region,
                    previous.stack_group,
                );
                if stack_members.len() > 1 {
                    place_panel_in_dock_region(
                        &mut next.panels,
                        *area_id,
                        previous.dock_region,
                        *placement,
                    );
                    if let Some((dock_region, stack_group)) =
                        panel_stack_location(&next.panels, *area_id)
                    {
                        set_active_panel_in_stack(
                            &mut next.stack_groups,
                            dock_region,
                            stack_group,
                            *area_id,
                        );
                    }
                    reconcile_center_area_with_active_panel(&mut next, *area_id);
                }
            }
            StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                area_id,
                dock_region,
                placement,
            } => {
                let previous = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                let mut panel = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                panel.dock_region = *dock_region;
                if *dock_region == StudioGuiWindowDockRegion::CenterStage {
                    panel.visible = true;
                }
                upsert_panel_state(&mut next.panels, panel);
                place_panel_in_dock_region(&mut next.panels, *area_id, *dock_region, *placement);
                if previous.dock_region != *dock_region || previous.stack_group != panel_stack_group(
                    &next.panels,
                    *area_id,
                )
                .unwrap_or(previous.stack_group)
                {
                    normalize_stack_group(
                        &mut next.panels,
                        previous.dock_region,
                        previous.stack_group,
                    );
                }
                normalize_region_stack_groups(&mut next.panels, *dock_region);
                if previous.dock_region != *dock_region {
                    normalize_region_stack_groups(&mut next.panels, previous.dock_region);
                }
                if let Some((dock_region, stack_group)) = panel_stack_location(&next.panels, *area_id)
                {
                    set_active_panel_in_stack(
                        &mut next.stack_groups,
                        dock_region,
                        stack_group,
                        *area_id,
                    );
                }
                reconcile_center_stage_after_panel_move(&mut next, *area_id, *dock_region);
            }
            StudioGuiWindowLayoutMutation::SetPanelDockRegion {
                area_id,
                dock_region,
                order,
            } => {
                let previous_group = next
                    .panel(*area_id)
                    .map(|panel| panel.stack_group)
                    .unwrap_or_else(|| default_panel_state(*area_id).stack_group);
                let previous_region = next
                    .panel(*area_id)
                    .map(|panel| panel.dock_region)
                    .unwrap_or_else(|| default_panel_state(*area_id).dock_region);
                let mut panel = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                panel.dock_region = *dock_region;
                if let Some(order) = order {
                    panel.order = *order;
                }
                if *dock_region == StudioGuiWindowDockRegion::CenterStage {
                    panel.visible = true;
                }
                upsert_panel_state(&mut next.panels, panel);
                if previous_region != *dock_region {
                    if order.is_some() {
                        let new_stack_group = next_available_stack_group(&next.panels, *dock_region);
                        if let Some(target) =
                            next.panels.iter_mut().find(|panel| panel.area_id == *area_id)
                        {
                            target.stack_group = new_stack_group;
                        }
                        normalize_stack_group(&mut next.panels, previous_region, previous_group);
                        normalize_region_stack_groups(&mut next.panels, previous_region);
                    } else {
                        place_panel_in_dock_region(
                            &mut next.panels,
                            *area_id,
                            *dock_region,
                            StudioGuiWindowDockPlacement::End,
                        );
                        normalize_stack_group(&mut next.panels, previous_region, previous_group);
                        normalize_region_stack_groups(&mut next.panels, *dock_region);
                        normalize_region_stack_groups(&mut next.panels, previous_region);
                    }
                }
                if let Some((dock_region, stack_group)) = panel_stack_location(&next.panels, *area_id)
                {
                    set_active_panel_in_stack(
                        &mut next.stack_groups,
                        dock_region,
                        stack_group,
                        *area_id,
                    );
                }
                reconcile_center_stage_after_panel_move(&mut next, *area_id, *dock_region);
            }
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
            StudioGuiWindowLayoutMutation::SetPanelOrder { area_id, order } => {
                let mut panel = next
                    .panel(*area_id)
                    .cloned()
                    .unwrap_or_else(|| default_panel_state(*area_id));
                panel.order = *order;
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
        reconcile_stack_group_states(&mut next);
        next
    }

    pub fn persistence_state(&self) -> StudioGuiWindowLayoutPersistenceState {
        StudioGuiWindowLayoutPersistenceState {
            layout_key: self.scope.layout_key.clone(),
            center_area: self.center_area,
            panels: self.panels.clone(),
            stack_groups: self.stack_groups.clone(),
            region_weights: self.region_weights.clone(),
        }
    }
}

impl StudioGuiWindowLayoutScope {
    pub fn legacy_layout_key(&self) -> Option<String> {
        match (self.window_id, self.window_role) {
            (Some(window_id), Some(window_role)) => {
                Some(legacy_layout_key_for_window(window_id, window_role))
            }
            _ => None,
        }
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

    pub fn panels_in_dock_region(
        &self,
        dock_region: StudioGuiWindowDockRegion,
    ) -> Vec<&StudioGuiWindowPanelLayout> {
        let mut panels = self
            .panels
            .iter()
            .filter(|panel| panel.dock_region == dock_region)
            .collect::<Vec<_>>();
        panels.sort_by_key(|panel| {
            (
                panel.stack_group,
                panel.order,
                area_sort_rank(panel.area_id),
            )
        });
        panels
    }

    pub fn panels_in_stack_group(
        &self,
        dock_region: StudioGuiWindowDockRegion,
        stack_group: u8,
    ) -> Vec<&StudioGuiWindowPanelLayout> {
        let mut panels = self
            .panels
            .iter()
            .filter(|panel| panel.dock_region == dock_region && panel.stack_group == stack_group)
            .collect::<Vec<_>>();
        panels.sort_by_key(|panel| (panel.order, area_sort_rank(panel.area_id)));
        panels
    }

    pub fn stack_group(
        &self,
        dock_region: StudioGuiWindowDockRegion,
        stack_group: u8,
    ) -> Option<&StudioGuiWindowStackGroupLayout> {
        self.stack_groups.iter().find(|group| {
            group.dock_region == dock_region && group.stack_group == stack_group
        })
    }

    pub fn stack_groups_in_dock_region(
        &self,
        dock_region: StudioGuiWindowDockRegion,
    ) -> Vec<&StudioGuiWindowStackGroupLayout> {
        let mut groups = self
            .stack_groups
            .iter()
            .filter(|group| group.dock_region == dock_region)
            .collect::<Vec<_>>();
        groups.sort_by_key(|group| group.stack_group);
        groups
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
        let stack_groups = build_stack_group_layouts(state, &panels);

        Self {
            state: state.clone(),
            titlebar,
            panels,
            stack_groups,
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
    let stack_tab_count = state
        .panels_in_stack_group(panel_state.dock_region, panel_state.stack_group)
        .len();
    let active_in_stack = state
        .active_panel_in_stack(panel_state.dock_region, panel_state.stack_group)
        == Some(area_id);
    let display_mode = if stack_tab_count <= 1 {
        StudioGuiWindowPanelDisplayMode::Standalone
    } else if active_in_stack {
        StudioGuiWindowPanelDisplayMode::ActiveTab
    } else {
        StudioGuiWindowPanelDisplayMode::InactiveTab
    };
    StudioGuiWindowPanelLayout {
        area_id,
        title,
        dock_region: panel_state.dock_region,
        stack_group: panel_state.stack_group,
        order: panel_state.order,
        display_mode,
        active_in_stack,
        visible: panel_state.visible,
        collapsed: panel_state.collapsed,
        badge,
        summary,
    }
}

fn build_stack_group_layouts(
    state: &StudioGuiWindowLayoutState,
    panels: &[StudioGuiWindowPanelLayout],
) -> Vec<StudioGuiWindowStackGroupLayout> {
    let mut groups = state
        .stack_groups
        .iter()
        .map(|group| {
            let mut tabs = panels
                .iter()
                .filter(|panel| {
                    panel.dock_region == group.dock_region && panel.stack_group == group.stack_group
                })
                .map(|panel| StudioGuiWindowStackTabLayout {
                    area_id: panel.area_id,
                    title: panel.title,
                    active: panel.area_id == group.active_area_id,
                    visible: panel.visible,
                    collapsed: panel.collapsed,
                })
                .collect::<Vec<_>>();
            tabs.sort_by_key(|tab| {
                panels
                    .iter()
                    .find(|panel| panel.area_id == tab.area_id)
                    .map(|panel| panel.order)
                    .unwrap_or(u8::MAX)
            });
            StudioGuiWindowStackGroupLayout {
                dock_region: group.dock_region,
                stack_group: group.stack_group,
                tabbed: tabs.len() > 1,
                active_area_id: group.active_area_id,
                tabs,
            }
        })
        .collect::<Vec<_>>();
    groups.sort_by_key(|group| (dock_region_rank(group.dock_region), group.stack_group));
    groups
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
            let window = state.window(window_id);
            let window_role = window.map(|window| window.role);
            let layout_slot = window.map(|window| window.layout_slot);
            StudioGuiWindowLayoutScope {
                kind: StudioGuiWindowLayoutScopeKind::Window,
                window_id: Some(window_id),
                window_role,
                layout_slot,
                layout_key: layout_key_for_window(window_role, layout_slot),
            }
        }
        None => StudioGuiWindowLayoutScope {
            kind: StudioGuiWindowLayoutScopeKind::EmptyWorkspace,
            window_id: None,
            window_role: None,
            layout_slot: None,
            layout_key: "studio.window.empty".to_string(),
        },
    }
}

fn layout_key_for_window(
    window_role: Option<StudioWindowHostRole>,
    layout_slot: Option<u16>,
) -> String {
    match (window_role, layout_slot) {
        (Some(StudioWindowHostRole::EntitlementTimerOwner), Some(layout_slot)) => {
            format!("studio.window.owner.slot-{layout_slot}")
        }
        (Some(StudioWindowHostRole::Observer), Some(layout_slot)) => {
            format!("studio.window.observer.slot-{layout_slot}")
        }
        (Some(StudioWindowHostRole::EntitlementTimerOwner), None) => {
            "studio.window.owner.slot-1".to_string()
        }
        (Some(StudioWindowHostRole::Observer), None) => "studio.window.observer.slot-1".to_string(),
        (None, Some(layout_slot)) => format!("studio.window.window.slot-{layout_slot}"),
        (None, None) => "studio.window.window.slot-1".to_string(),
    }
}

fn legacy_layout_key_for_window(
    window_id: StudioWindowHostId,
    window_role: StudioWindowHostRole,
) -> String {
    match window_role {
        StudioWindowHostRole::EntitlementTimerOwner => {
            format!("studio.window.owner.{window_id}")
        }
        StudioWindowHostRole::Observer => format!("studio.window.observer.{window_id}"),
    }
}

fn default_panel_states() -> Vec<StudioGuiWindowPanelLayoutState> {
    vec![
        default_panel_state(StudioGuiWindowAreaId::Commands),
        default_panel_state(StudioGuiWindowAreaId::Canvas),
        default_panel_state(StudioGuiWindowAreaId::Runtime),
    ]
}

fn default_stack_group_states(
    panels: &[StudioGuiWindowPanelLayoutState],
) -> Vec<StudioGuiWindowStackGroupState> {
    derive_stack_group_states(panels, &[])
}

fn default_panel_state(area_id: StudioGuiWindowAreaId) -> StudioGuiWindowPanelLayoutState {
    match area_id {
        StudioGuiWindowAreaId::Commands => StudioGuiWindowPanelLayoutState {
            area_id,
            dock_region: StudioGuiWindowDockRegion::LeftSidebar,
            stack_group: 10,
            order: 10,
            visible: true,
            collapsed: false,
        },
        StudioGuiWindowAreaId::Canvas => StudioGuiWindowPanelLayoutState {
            area_id,
            dock_region: StudioGuiWindowDockRegion::CenterStage,
            stack_group: 10,
            order: 20,
            visible: true,
            collapsed: false,
        },
        StudioGuiWindowAreaId::Runtime => StudioGuiWindowPanelLayoutState {
            area_id,
            dock_region: StudioGuiWindowDockRegion::RightSidebar,
            stack_group: 10,
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
    if let Some(index) = panels
        .iter()
        .position(|candidate| candidate.area_id == panel.area_id)
    {
        panels[index] = panel;
    } else {
        panels.push(panel);
    }
    sort_panels(panels);
}

fn upsert_stack_group_state(
    stack_groups: &mut Vec<StudioGuiWindowStackGroupState>,
    stack_group: StudioGuiWindowStackGroupState,
) {
    if let Some(index) = stack_groups.iter().position(|candidate| {
        candidate.dock_region == stack_group.dock_region
            && candidate.stack_group == stack_group.stack_group
    }) {
        stack_groups[index] = stack_group;
    } else {
        stack_groups.push(stack_group);
    }
    sort_stack_groups(stack_groups);
}

fn set_active_panel_in_stack(
    stack_groups: &mut Vec<StudioGuiWindowStackGroupState>,
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
    area_id: StudioGuiWindowAreaId,
) {
    upsert_stack_group_state(
        stack_groups,
        StudioGuiWindowStackGroupState {
            dock_region,
            stack_group,
            active_area_id: area_id,
        },
    );
}

fn set_active_panel_for_area(
    layout: &mut StudioGuiWindowLayoutState,
    area_id: StudioGuiWindowAreaId,
) {
    let mut panel = layout
        .panel(area_id)
        .cloned()
        .unwrap_or_else(|| default_panel_state(area_id));
    panel.visible = true;
    let dock_region = panel.dock_region;
    let stack_group = panel.stack_group;
    upsert_panel_state(&mut layout.panels, panel);
    set_active_panel_in_stack(&mut layout.stack_groups, dock_region, stack_group, area_id);
    reconcile_center_area_with_active_panel(layout, area_id);
}

fn reconcile_stack_group_states(layout: &mut StudioGuiWindowLayoutState) {
    layout.stack_groups = derive_stack_group_states(&layout.panels, &layout.stack_groups);
}

fn reconcile_center_area_with_active_panel(
    layout: &mut StudioGuiWindowLayoutState,
    area_id: StudioGuiWindowAreaId,
) {
    if layout
        .panel(area_id)
        .map(|panel| panel.dock_region == StudioGuiWindowDockRegion::CenterStage)
        .unwrap_or(false)
    {
        layout.center_area = area_id;
    }
}

fn derive_stack_group_states(
    panels: &[StudioGuiWindowPanelLayoutState],
    preferred: &[StudioGuiWindowStackGroupState],
) -> Vec<StudioGuiWindowStackGroupState> {
    let mut groups = grouped_area_ids_by_region(panels)
        .into_iter()
        .flat_map(|(dock_region, grouped_area_ids)| {
            grouped_area_ids
                .into_iter()
                .enumerate()
                .map(move |(group_index, area_ids)| {
                    let stack_group = dock_group_order(group_index);
                    let preferred_active = preferred
                        .iter()
                        .find(|candidate| {
                            candidate.dock_region == dock_region
                                && candidate.stack_group == stack_group
                        })
                        .map(|candidate| candidate.active_area_id);
                    let active_area_id =
                        choose_active_area_id_for_stack(panels, &area_ids, preferred_active);
                    StudioGuiWindowStackGroupState {
                        dock_region,
                        stack_group,
                        active_area_id,
                    }
                })
        })
        .collect::<Vec<_>>();
    sort_stack_groups(&mut groups);
    groups
}

fn choose_active_area_id_for_stack(
    panels: &[StudioGuiWindowPanelLayoutState],
    area_ids: &[StudioGuiWindowAreaId],
    preferred_active: Option<StudioGuiWindowAreaId>,
) -> StudioGuiWindowAreaId {
    if preferred_active
        .filter(|area_id| {
            area_ids.contains(area_id)
                && panels
                    .iter()
                    .find(|panel| panel.area_id == *area_id)
                    .map(|panel| panel.visible)
                    .unwrap_or(false)
        })
        .is_some()
    {
        return preferred_active.expect("preferred active just checked");
    }

    area_ids
        .iter()
        .copied()
        .find(|area_id| {
            panels
                .iter()
                .find(|panel| panel.area_id == *area_id)
                .map(|panel| panel.visible)
                .unwrap_or(false)
        })
        .or_else(|| area_ids.first().copied())
        .unwrap_or(StudioGuiWindowAreaId::Canvas)
}

fn place_panel_in_dock_region(
    panels: &mut Vec<StudioGuiWindowPanelLayoutState>,
    area_id: StudioGuiWindowAreaId,
    dock_region: StudioGuiWindowDockRegion,
    placement: StudioGuiWindowDockPlacement,
) {
    let mut grouped_area_ids =
        grouped_area_ids_for_region_excluding_area(panels, dock_region, Some(area_id));
    let insert_index = match placement {
        StudioGuiWindowDockPlacement::Start => 0,
        StudioGuiWindowDockPlacement::End => grouped_area_ids.len(),
        StudioGuiWindowDockPlacement::Before { anchor_area_id } => grouped_area_ids
            .iter()
            .position(|group| group.contains(&anchor_area_id))
            .unwrap_or(grouped_area_ids.len()),
        StudioGuiWindowDockPlacement::After { anchor_area_id } => grouped_area_ids
            .iter()
            .position(|group| group.contains(&anchor_area_id))
            .map(|index| index + 1)
            .unwrap_or(grouped_area_ids.len()),
    };
    grouped_area_ids.insert(insert_index.min(grouped_area_ids.len()), vec![area_id]);
    assign_region_stack_groups(panels, dock_region, &grouped_area_ids);
}

fn place_panel_in_stack_group(
    panels: &mut Vec<StudioGuiWindowPanelLayoutState>,
    area_id: StudioGuiWindowAreaId,
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
    placement: StudioGuiWindowDockPlacement,
) {
    let mut ordered_area_ids = panels
        .iter()
        .filter(|panel| {
            panel.area_id != area_id
                && panel.dock_region == dock_region
                && panel.stack_group == stack_group
        })
        .map(|panel| panel.area_id)
        .collect::<Vec<_>>();
    let insert_index = match placement {
        StudioGuiWindowDockPlacement::Start => 0,
        StudioGuiWindowDockPlacement::End => ordered_area_ids.len(),
        StudioGuiWindowDockPlacement::Before { anchor_area_id } => ordered_area_ids
            .iter()
            .position(|candidate| *candidate == anchor_area_id)
            .unwrap_or(ordered_area_ids.len()),
        StudioGuiWindowDockPlacement::After { anchor_area_id } => ordered_area_ids
            .iter()
            .position(|candidate| *candidate == anchor_area_id)
            .map(|index| index + 1)
            .unwrap_or(ordered_area_ids.len()),
    };
    ordered_area_ids.insert(insert_index.min(ordered_area_ids.len()), area_id);
    assign_stack_group_orders(panels, dock_region, stack_group, &ordered_area_ids);
}

fn normalize_stack_group(
    panels: &mut Vec<StudioGuiWindowPanelLayoutState>,
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
) {
    let ordered_area_ids = panels
        .iter()
        .filter(|panel| panel.dock_region == dock_region && panel.stack_group == stack_group)
        .map(|panel| panel.area_id)
        .collect::<Vec<_>>();
    assign_stack_group_orders(panels, dock_region, stack_group, &ordered_area_ids);
}

fn normalize_region_stack_groups(
    panels: &mut Vec<StudioGuiWindowPanelLayoutState>,
    dock_region: StudioGuiWindowDockRegion,
) {
    let grouped_area_ids = grouped_area_ids_for_region_excluding_area(panels, dock_region, None);
    assign_region_stack_groups(panels, dock_region, &grouped_area_ids);
}

fn assign_region_stack_groups(
    panels: &mut Vec<StudioGuiWindowPanelLayoutState>,
    dock_region: StudioGuiWindowDockRegion,
    grouped_area_ids: &[Vec<StudioGuiWindowAreaId>],
) {
    for (group_index, group) in grouped_area_ids.iter().enumerate() {
        let stack_group = dock_group_order(group_index);
        for area_id in group {
            if let Some(panel) = panels
                .iter_mut()
                .find(|panel| panel.area_id == *area_id && panel.dock_region == dock_region)
            {
                panel.stack_group = stack_group;
            }
        }
        assign_stack_group_orders(panels, dock_region, stack_group, group);
    }
    sort_panels(panels);
}

fn assign_stack_group_orders(
    panels: &mut Vec<StudioGuiWindowPanelLayoutState>,
    dock_region: StudioGuiWindowDockRegion,
    stack_group: u8,
    ordered_area_ids: &[StudioGuiWindowAreaId],
) {
    for (index, area_id) in ordered_area_ids.iter().enumerate() {
        if let Some(panel) = panels
            .iter_mut()
            .find(|panel| {
                panel.area_id == *area_id
                    && panel.dock_region == dock_region
                    && panel.stack_group == stack_group
            })
        {
            panel.order = dock_order(index);
        }
    }
    sort_panels(panels);
}

fn grouped_area_ids_for_region_excluding_area(
    panels: &[StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
    excluded_area_id: Option<StudioGuiWindowAreaId>,
) -> Vec<Vec<StudioGuiWindowAreaId>> {
    let mut ordered_panels = panels
        .iter()
        .filter(|panel| {
            panel.dock_region == dock_region && Some(panel.area_id) != excluded_area_id
        })
        .collect::<Vec<_>>();
    ordered_panels.sort_by_key(|panel| {
        (
            panel.stack_group,
            panel.order,
            area_sort_rank(panel.area_id),
        )
    });

    let mut groups = Vec::<Vec<StudioGuiWindowAreaId>>::new();
    let mut current_group = None;
    for panel in ordered_panels {
        if current_group != Some(panel.stack_group) {
            groups.push(Vec::new());
            current_group = Some(panel.stack_group);
        }
        if let Some(group) = groups.last_mut() {
            group.push(panel.area_id);
        }
    }
    groups
}

fn grouped_area_ids_by_region(
    panels: &[StudioGuiWindowPanelLayoutState],
) -> Vec<(StudioGuiWindowDockRegion, Vec<Vec<StudioGuiWindowAreaId>>)> {
    [
        StudioGuiWindowDockRegion::LeftSidebar,
        StudioGuiWindowDockRegion::CenterStage,
        StudioGuiWindowDockRegion::RightSidebar,
    ]
    .into_iter()
    .map(|dock_region| {
        (
            dock_region,
            grouped_area_ids_for_region_excluding_area(panels, dock_region, None),
        )
    })
    .filter(|(_, groups)| !groups.is_empty())
    .collect()
}

fn panel_stack_group(
    panels: &[StudioGuiWindowPanelLayoutState],
    area_id: StudioGuiWindowAreaId,
) -> Option<u8> {
    panels
        .iter()
        .find(|panel| panel.area_id == area_id)
        .map(|panel| panel.stack_group)
}

fn panel_stack_location(
    panels: &[StudioGuiWindowPanelLayoutState],
    area_id: StudioGuiWindowAreaId,
) -> Option<(StudioGuiWindowDockRegion, u8)> {
    panels
        .iter()
        .find(|panel| panel.area_id == area_id)
        .map(|panel| (panel.dock_region, panel.stack_group))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackCycleDirection {
    Next,
    Previous,
}

fn adjacent_panel_in_stack(
    panels: &[StudioGuiWindowPanelLayoutState],
    area_id: StudioGuiWindowAreaId,
    direction: StackCycleDirection,
) -> Option<StudioGuiWindowAreaId> {
    let (dock_region, stack_group) = panel_stack_location(panels, area_id)?;
    let ordered_area_ids = panels
        .iter()
        .filter(|panel| panel.dock_region == dock_region && panel.stack_group == stack_group)
        .map(|panel| panel.area_id)
        .collect::<Vec<_>>();
    if ordered_area_ids.len() <= 1 {
        return Some(area_id);
    }
    let current_index = ordered_area_ids.iter().position(|candidate| *candidate == area_id)?;
    let next_index = match direction {
        StackCycleDirection::Next => (current_index + 1) % ordered_area_ids.len(),
        StackCycleDirection::Previous => {
            (current_index + ordered_area_ids.len() - 1) % ordered_area_ids.len()
        }
    };
    ordered_area_ids.get(next_index).copied()
}

fn next_available_stack_group(
    panels: &[StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
) -> u8 {
    let max_group = panels
        .iter()
        .filter(|panel| panel.dock_region == dock_region)
        .map(|panel| panel.stack_group)
        .max()
        .unwrap_or(0);
    max_group.saturating_add(10).max(10)
}

fn dock_order(index: usize) -> u8 {
    ((index + 1) * 10).min(u8::MAX as usize) as u8
}

fn dock_group_order(index: usize) -> u8 {
    ((index + 1) * 10).min(u8::MAX as usize) as u8
}

fn reconcile_center_stage_after_panel_move(
    layout: &mut StudioGuiWindowLayoutState,
    moved_area_id: StudioGuiWindowAreaId,
    moved_region: StudioGuiWindowDockRegion,
) {
    if moved_region == StudioGuiWindowDockRegion::CenterStage {
        layout.center_area = moved_area_id;
        return;
    }

    if let Some(replacement_area_id) =
        first_panel_in_region(&layout.panels, StudioGuiWindowDockRegion::CenterStage)
    {
        if layout.center_area == moved_area_id {
            layout.center_area = replacement_area_id;
        }
        return;
    }

    let fallback_area_id = layout
        .panels
        .iter()
        .find(|panel| panel.visible)
        .map(|panel| panel.area_id)
        .or_else(|| layout.panels.first().map(|panel| panel.area_id))
        .unwrap_or(StudioGuiWindowAreaId::Canvas);
    let mut fallback_panel = layout
        .panel(fallback_area_id)
        .cloned()
        .unwrap_or_else(|| default_panel_state(fallback_area_id));
    fallback_panel.dock_region = StudioGuiWindowDockRegion::CenterStage;
    fallback_panel.visible = true;
    upsert_panel_state(&mut layout.panels, fallback_panel);
    layout.center_area = fallback_area_id;
}

fn first_panel_in_region(
    panels: &[StudioGuiWindowPanelLayoutState],
    dock_region: StudioGuiWindowDockRegion,
) -> Option<StudioGuiWindowAreaId> {
    panels
        .iter()
        .find(|panel| panel.dock_region == dock_region)
        .map(|panel| panel.area_id)
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

fn sort_stack_groups(stack_groups: &mut [StudioGuiWindowStackGroupState]) {
    stack_groups.sort_by_key(|candidate| {
        (
            dock_region_rank(candidate.dock_region),
            candidate.stack_group,
            area_sort_rank(candidate.active_area_id),
        )
    });
}

fn sort_panels(panels: &mut [StudioGuiWindowPanelLayoutState]) {
    panels.sort_by_key(|candidate| {
        (
            dock_region_rank(candidate.dock_region),
            candidate.stack_group,
            candidate.order,
            area_sort_rank(candidate.area_id),
        )
    });
}

fn area_sort_rank(area_id: StudioGuiWindowAreaId) -> u8 {
    match area_id {
        StudioGuiWindowAreaId::Commands => 1,
        StudioGuiWindowAreaId::Canvas => 2,
        StudioGuiWindowAreaId::Runtime => 3,
    }
}

fn dock_region_rank(dock_region: StudioGuiWindowDockRegion) -> u8 {
    match dock_region {
        StudioGuiWindowDockRegion::LeftSidebar => 1,
        StudioGuiWindowDockRegion::CenterStage => 2,
        StudioGuiWindowDockRegion::RightSidebar => 3,
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
        StudioGuiDriver, StudioGuiEvent, StudioGuiWindowAreaId, StudioGuiWindowDockPlacement,
        StudioGuiWindowDockRegion, StudioGuiWindowLayoutScopeKind, StudioGuiWindowLayoutState,
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
        assert_eq!(
            layout.state.scope.kind,
            StudioGuiWindowLayoutScopeKind::Window
        );
        assert_eq!(layout.state.scope.window_id, Some(1));
        assert_eq!(layout.state.scope.layout_slot, Some(1));
        assert_eq!(layout.state.scope.layout_key, "studio.window.owner.slot-1");
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
        assert_eq!(
            commands.display_mode,
            crate::StudioGuiWindowPanelDisplayMode::Standalone
        );
        assert!(commands.active_in_stack);
        assert!(commands.visible);
        assert!(!commands.collapsed);
        assert_eq!(commands.badge.as_deref(), Some("5"));

        let canvas = layout
            .panel(StudioGuiWindowAreaId::Canvas)
            .expect("expected canvas panel");
        assert_eq!(canvas.dock_region, StudioGuiWindowDockRegion::CenterStage);
        assert!(canvas.active_in_stack);
        assert_eq!(canvas.badge.as_deref(), Some("3"));
        assert!(canvas.summary.contains("3 suggestions"));

        let runtime = layout
            .panel(StudioGuiWindowAreaId::Runtime)
            .expect("expected runtime panel");
        assert_eq!(runtime.dock_region, StudioGuiWindowDockRegion::RightSidebar);
        assert_eq!(
            runtime.display_mode,
            crate::StudioGuiWindowPanelDisplayMode::Standalone
        );
        assert!(runtime.active_in_stack);
        assert!(runtime.summary.contains("status=Idle"));
        assert!(runtime.summary.contains("entitlement=attached"));
        assert_eq!(
            layout
                .stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
                .map(|group| group.active_area_id),
            Some(StudioGuiWindowAreaId::Runtime)
        );

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

        assert_eq!(first.window.layout_state.scope.layout_slot, Some(1));
        assert_eq!(second.window.layout_state.scope.layout_slot, Some(1));
        assert_eq!(first_layout_key, "studio.window.owner.slot-1");
        assert_eq!(second_layout_key, "studio.window.observer.slot-1");
        assert_ne!(first_layout_key, second_layout_key);
        assert_eq!(
            second.snapshot.layout_state.scope.layout_key,
            "studio.window.owner.slot-1"
        );
    }

    #[test]
    fn studio_gui_window_layout_applies_center_area_and_panel_order_mutations() {
        let state = StudioGuiWindowLayoutState::default()
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::SetPanelVisibility {
                area_id: StudioGuiWindowAreaId::Runtime,
                visible: false,
            })
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::SetCenterArea {
                area_id: StudioGuiWindowAreaId::Runtime,
            })
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::SetPanelOrder {
                area_id: StudioGuiWindowAreaId::Runtime,
                order: 5,
            });

        assert_eq!(state.center_area, StudioGuiWindowAreaId::Runtime);
        assert_eq!(
            state
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| (panel.visible, panel.order)),
            Some((true, 5))
        );
        assert_eq!(
            state
                .panels
                .iter()
                .map(|panel| (panel.area_id, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (StudioGuiWindowAreaId::Commands, 10),
                (StudioGuiWindowAreaId::Canvas, 20),
                (StudioGuiWindowAreaId::Runtime, 5),
            ]
        );
    }

    #[test]
    fn studio_gui_window_layout_moves_panels_across_dock_regions() {
        let state = StudioGuiWindowLayoutState::default()
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::SetPanelDockRegion {
                area_id: StudioGuiWindowAreaId::Runtime,
                dock_region: StudioGuiWindowDockRegion::CenterStage,
                order: Some(5),
            })
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::SetPanelDockRegion {
                area_id: StudioGuiWindowAreaId::Canvas,
                dock_region: StudioGuiWindowDockRegion::LeftSidebar,
                order: Some(25),
            });

        assert_eq!(state.center_area, StudioGuiWindowAreaId::Runtime);
        assert_eq!(
            state.panel(StudioGuiWindowAreaId::Runtime).map(|panel| (
                panel.dock_region,
                panel.order,
                panel.visible
            )),
            Some((StudioGuiWindowDockRegion::CenterStage, 10, true))
        );
        assert_eq!(
            state
                .panel(StudioGuiWindowAreaId::Canvas)
                .map(|panel| (panel.dock_region, panel.order)),
            Some((StudioGuiWindowDockRegion::LeftSidebar, 25))
        );
    }

    #[test]
    fn studio_gui_window_layout_places_panels_within_region_by_anchor() {
        let state = StudioGuiWindowLayoutState::default()
            .applying_mutation(
                &crate::StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                    area_id: StudioGuiWindowAreaId::Commands,
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            )
            .applying_mutation(
                &crate::StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    placement: StudioGuiWindowDockPlacement::After {
                        anchor_area_id: StudioGuiWindowAreaId::Commands,
                    },
                },
            );

        assert_eq!(
            state
                .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
                .into_iter()
                .map(|panel| (panel.area_id, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (StudioGuiWindowAreaId::Commands, 10),
                (StudioGuiWindowAreaId::Runtime, 10),
            ]
        );
        assert_eq!(state.center_area, StudioGuiWindowAreaId::Canvas);
    }

    #[test]
    fn studio_gui_window_layout_stacks_panel_with_anchor_group() {
        let state = StudioGuiWindowLayoutState::default()
            .applying_mutation(
                &crate::StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                    area_id: StudioGuiWindowAreaId::Commands,
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            )
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::StackPanelWith {
                area_id: StudioGuiWindowAreaId::Commands,
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            });

        assert_eq!(
            state
                .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
                .into_iter()
                .map(|panel| (panel.area_id, panel.stack_group, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (StudioGuiWindowAreaId::Commands, 10, 10),
                (StudioGuiWindowAreaId::Runtime, 10, 20),
            ]
        );
        assert_eq!(
            state
                .panels_in_stack_group(StudioGuiWindowDockRegion::RightSidebar, 10)
                .into_iter()
                .map(|panel| panel.area_id)
                .collect::<Vec<_>>(),
            vec![
                StudioGuiWindowAreaId::Commands,
                StudioGuiWindowAreaId::Runtime,
            ]
        );
        assert_eq!(
            state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
            Some(StudioGuiWindowAreaId::Commands)
        );
    }

    #[test]
    fn studio_gui_window_layout_switches_active_panel_within_stack_group() {
        let state = StudioGuiWindowLayoutState::default()
            .applying_mutation(
                &crate::StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                    area_id: StudioGuiWindowAreaId::Commands,
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            )
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::StackPanelWith {
                area_id: StudioGuiWindowAreaId::Commands,
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            })
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::SetActivePanelInStack {
                area_id: StudioGuiWindowAreaId::Runtime,
            });

        assert_eq!(
            state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
            Some(StudioGuiWindowAreaId::Runtime)
        );
    }

    #[test]
    fn studio_gui_window_layout_cycles_active_panel_within_stack_group() {
        let state = StudioGuiWindowLayoutState::default()
            .applying_mutation(
                &crate::StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                    area_id: StudioGuiWindowAreaId::Commands,
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            )
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::StackPanelWith {
                area_id: StudioGuiWindowAreaId::Commands,
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            })
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::ActivateNextPanelInStack {
                area_id: StudioGuiWindowAreaId::Commands,
            })
            .applying_mutation(
                &crate::StudioGuiWindowLayoutMutation::ActivatePreviousPanelInStack {
                    area_id: StudioGuiWindowAreaId::Runtime,
                },
            );

        assert_eq!(
            state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
            Some(StudioGuiWindowAreaId::Commands)
        );
    }

    #[test]
    fn studio_gui_window_layout_unstacks_panel_into_separate_group() {
        let state = StudioGuiWindowLayoutState::default()
            .applying_mutation(
                &crate::StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                    area_id: StudioGuiWindowAreaId::Commands,
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    placement: StudioGuiWindowDockPlacement::Before {
                        anchor_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                },
            )
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::StackPanelWith {
                area_id: StudioGuiWindowAreaId::Commands,
                anchor_area_id: StudioGuiWindowAreaId::Runtime,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            })
            .applying_mutation(&crate::StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
                area_id: StudioGuiWindowAreaId::Commands,
                placement: StudioGuiWindowDockPlacement::Before {
                    anchor_area_id: StudioGuiWindowAreaId::Runtime,
                },
            });

        assert_eq!(
            state
                .panels_in_dock_region(StudioGuiWindowDockRegion::RightSidebar)
                .into_iter()
                .map(|panel| (panel.area_id, panel.stack_group, panel.order))
                .collect::<Vec<_>>(),
            vec![
                (StudioGuiWindowAreaId::Commands, 10, 10),
                (StudioGuiWindowAreaId::Runtime, 20, 10),
            ]
        );
        assert_eq!(
            state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 10),
            Some(StudioGuiWindowAreaId::Commands)
        );
        assert_eq!(
            state.active_panel_in_stack(StudioGuiWindowDockRegion::RightSidebar, 20),
            Some(StudioGuiWindowAreaId::Runtime)
        );
    }

    #[test]
    fn studio_gui_window_layout_promotes_visible_panel_when_center_stage_becomes_empty() {
        let state = StudioGuiWindowLayoutState::default().applying_mutation(
            &crate::StudioGuiWindowLayoutMutation::SetPanelDockRegion {
                area_id: StudioGuiWindowAreaId::Canvas,
                dock_region: StudioGuiWindowDockRegion::RightSidebar,
                order: None,
            },
        );

        assert_eq!(state.center_area, StudioGuiWindowAreaId::Commands);
        assert_eq!(
            state
                .panel(StudioGuiWindowAreaId::Canvas)
                .map(|panel| panel.dock_region),
            Some(StudioGuiWindowDockRegion::RightSidebar)
        );
        assert_eq!(
            state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| (panel.dock_region, panel.visible)),
            Some((StudioGuiWindowDockRegion::CenterStage, true))
        );
    }

    #[test]
    fn studio_gui_window_layout_keeps_observer_slots_stable_when_peer_closes() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let first = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected first open dispatch");
        let first_window_id = match first.outcome {
            crate::StudioGuiDriverOutcome::HostCommand(
                crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
            ) => opened.registration.window_id,
            other => panic!("expected first window opened outcome, got {other:?}"),
        };
        let second = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected second open dispatch");
        let second_window_id = match second.outcome {
            crate::StudioGuiDriverOutcome::HostCommand(
                crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
            ) => opened.registration.window_id,
            other => panic!("expected second window opened outcome, got {other:?}"),
        };
        let third = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected third open dispatch");
        let third_window_id = match third.outcome {
            crate::StudioGuiDriverOutcome::HostCommand(
                crate::StudioGuiHostCommandOutcome::WindowOpened(opened),
            ) => opened.registration.window_id,
            other => panic!("expected third window opened outcome, got {other:?}"),
        };

        assert_eq!(
            driver
                .window_model_for_window(Some(third_window_id))
                .layout_state
                .scope
                .layout_key,
            "studio.window.observer.slot-2"
        );

        let _ = driver
            .dispatch_event(StudioGuiEvent::CloseWindowRequested {
                window_id: second_window_id,
            })
            .expect("expected second window close");

        let remaining_observer = driver.window_model_for_window(Some(third_window_id));
        assert_eq!(remaining_observer.layout_state.scope.layout_slot, Some(2));
        assert_eq!(
            remaining_observer.layout_state.scope.layout_key,
            "studio.window.observer.slot-2"
        );

        let _ = driver
            .dispatch_event(StudioGuiEvent::CloseWindowRequested {
                window_id: first_window_id,
            })
            .expect("expected first window close");

        let promoted_owner = driver.window_model_for_window(Some(third_window_id));
        assert_eq!(promoted_owner.layout_state.scope.layout_slot, Some(1));
        assert_eq!(
            promoted_owner.layout_state.scope.layout_key,
            "studio.window.owner.slot-1"
        );
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
            layout.state.scope.kind,
            StudioGuiWindowLayoutScopeKind::EmptyWorkspace
        );
        assert_eq!(layout.state.scope.layout_key, "studio.window.empty");
        assert_eq!(
            layout
                .panel(StudioGuiWindowAreaId::Commands)
                .and_then(|panel| panel.badge.as_deref()),
            Some("5")
        );
    }
}
