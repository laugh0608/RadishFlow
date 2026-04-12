use super::helpers::*;
use super::*;

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
        panels.sort_by_key(|panel| (panel.stack_group, panel.order, area_sort_rank(panel.area_id)));
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
        self.stack_groups
            .iter()
            .find(|group| group.dock_region == dock_region && group.stack_group == stack_group)
    }

    pub fn stack_groups_in_dock_region(
        &self,
        dock_region: StudioGuiWindowDockRegion,
    ) -> Vec<&StudioGuiWindowStackGroupState> {
        let mut groups = self
            .stack_groups
            .iter()
            .filter(|group| group.dock_region == dock_region)
            .collect::<Vec<_>>();
        groups.sort_by_key(|group| group.stack_group);
        groups
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

    pub fn drop_target_for_mutation(
        &self,
        mutation: &StudioGuiWindowLayoutMutation,
    ) -> Option<StudioGuiWindowDropTarget> {
        let (area_id, kind, dock_region, placement, creates_new_stack, merges_into_existing_stack) =
            match mutation {
                StudioGuiWindowLayoutMutation::MovePanelWithinStack { area_id, placement } => (
                    *area_id,
                    StudioGuiWindowDropTargetKind::StackTab,
                    self.panel(*area_id)
                        .map(|panel| panel.dock_region)
                        .unwrap_or_else(|| default_panel_state(*area_id).dock_region),
                    *placement,
                    false,
                    true,
                ),
                StudioGuiWindowLayoutMutation::StackPanelWith {
                    area_id,
                    anchor_area_id,
                    placement,
                } => (
                    *area_id,
                    StudioGuiWindowDropTargetKind::StackTab,
                    self.panel(*anchor_area_id)
                        .map(|panel| panel.dock_region)
                        .unwrap_or_else(|| default_panel_state(*anchor_area_id).dock_region),
                    *placement,
                    false,
                    true,
                ),
                StudioGuiWindowLayoutMutation::UnstackPanelFromGroup { area_id, placement } => {
                    let source_panel = self
                        .panel(*area_id)
                        .cloned()
                        .unwrap_or_else(|| default_panel_state(*area_id));
                    if self
                        .panels_in_stack_group(source_panel.dock_region, source_panel.stack_group)
                        .len()
                        <= 1
                    {
                        return None;
                    }
                    (
                        *area_id,
                        StudioGuiWindowDropTargetKind::DockRegionGroup,
                        source_panel.dock_region,
                        *placement,
                        true,
                        false,
                    )
                }
                StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                    area_id,
                    dock_region,
                    placement,
                } => (
                    *area_id,
                    StudioGuiWindowDropTargetKind::DockRegionGroup,
                    *dock_region,
                    *placement,
                    true,
                    false,
                ),
                _ => return None,
            };

        let source_panel = self
            .panel(area_id)
            .cloned()
            .unwrap_or_else(|| default_panel_state(area_id));
        let preview = self.applying_mutation(mutation);
        let preview_panel = preview
            .panel(area_id)
            .cloned()
            .unwrap_or_else(|| default_panel_state(area_id));
        let preview_area_ids = preview
            .panels_in_stack_group(preview_panel.dock_region, preview_panel.stack_group)
            .into_iter()
            .map(|panel| panel.area_id)
            .collect::<Vec<_>>();
        let target_group_index = preview
            .stack_groups_in_dock_region(preview_panel.dock_region)
            .iter()
            .position(|group| group.stack_group == preview_panel.stack_group)
            .unwrap_or(0);
        let target_tab_index = preview_area_ids
            .iter()
            .position(|candidate| *candidate == area_id)
            .unwrap_or(0);

        Some(StudioGuiWindowDropTarget {
            area_id,
            kind,
            dock_region,
            placement,
            anchor_area_id: anchor_area_id_from_placement(placement),
            source_dock_region: source_panel.dock_region,
            source_stack_group: source_panel.stack_group,
            target_stack_group: preview_panel.stack_group,
            target_group_index,
            target_tab_index,
            creates_new_stack,
            merges_into_existing_stack,
            preview_active_area_id: preview
                .active_panel_in_stack(preview_panel.dock_region, preview_panel.stack_group)
                .unwrap_or(area_id),
            preview_area_ids,
        })
    }

    pub fn drop_target_for_query(
        &self,
        query: &StudioGuiWindowDropTargetQuery,
    ) -> Option<StudioGuiWindowDropTarget> {
        self.drop_target_for_mutation(&query.layout_mutation())
    }

    pub fn preview_layout_state_for_query(
        &self,
        query: &StudioGuiWindowDropTargetQuery,
    ) -> Option<Self> {
        self.drop_target_for_query(query)
            .map(|_| self.applying_mutation(&query.layout_mutation()))
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
                set_active_panel_in_stack(&mut next.stack_groups, dock_region, stack_group, *area_id);
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
                set_active_panel_in_stack(&mut next.stack_groups, dock_region, stack_group, *area_id);
                reconcile_center_area_with_active_panel(&mut next, *area_id);
            }
            StudioGuiWindowLayoutMutation::ActivateNextPanelInStack { area_id } => {
                if let Some(next_area_id) =
                    adjacent_panel_in_stack(&next.panels, *area_id, StackCycleDirection::Next)
                {
                    set_active_panel_for_area(&mut next, next_area_id);
                }
            }
            StudioGuiWindowLayoutMutation::ActivatePreviousPanelInStack { area_id } => {
                if let Some(previous_area_id) =
                    adjacent_panel_in_stack(&next.panels, *area_id, StackCycleDirection::Previous)
                {
                    set_active_panel_for_area(&mut next, previous_area_id);
                }
            }
            StudioGuiWindowLayoutMutation::MovePanelWithinStack { area_id, placement } => {
                if let Some((dock_region, stack_group)) = panel_stack_location(&next.panels, *area_id)
                {
                    place_panel_in_stack_group(
                        &mut next.panels,
                        *area_id,
                        dock_region,
                        stack_group,
                        *placement,
                    );
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
                    normalize_stack_group(&mut next.panels, previous.dock_region, previous.stack_group);
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
                let stack_members =
                    next.panels_in_stack_group(previous.dock_region, previous.stack_group);
                if stack_members.len() > 1 {
                    place_panel_in_dock_region(&mut next.panels, *area_id, previous.dock_region, *placement);
                    if let Some((dock_region, stack_group)) = panel_stack_location(&next.panels, *area_id)
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
                if previous.dock_region != *dock_region
                    || previous.stack_group
                        != panel_stack_group(&next.panels, *area_id).unwrap_or(previous.stack_group)
                {
                    normalize_stack_group(&mut next.panels, previous.dock_region, previous.stack_group);
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
                        if let Some(target) = next.panels.iter_mut().find(|panel| panel.area_id == *area_id) {
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
            StudioGuiWindowLayoutMutation::SetRegionWeight { dock_region, weight } => {
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

impl StudioGuiWindowDropTargetQuery {
    pub fn layout_mutation(&self) -> StudioGuiWindowLayoutMutation {
        match self {
            StudioGuiWindowDropTargetQuery::DockRegion {
                area_id,
                dock_region,
                placement,
            } => StudioGuiWindowLayoutMutation::PlacePanelInDockRegion {
                area_id: *area_id,
                dock_region: *dock_region,
                placement: *placement,
            },
            StudioGuiWindowDropTargetQuery::Stack {
                area_id,
                anchor_area_id,
                placement,
            } => StudioGuiWindowLayoutMutation::StackPanelWith {
                area_id: *area_id,
                anchor_area_id: *anchor_area_id,
                placement: *placement,
            },
            StudioGuiWindowDropTargetQuery::CurrentStack { area_id, placement } => {
                StudioGuiWindowLayoutMutation::MovePanelWithinStack {
                    area_id: *area_id,
                    placement: *placement,
                }
            }
            StudioGuiWindowDropTargetQuery::Unstack { area_id, placement } => {
                StudioGuiWindowLayoutMutation::UnstackPanelFromGroup {
                    area_id: *area_id,
                    placement: *placement,
                }
            }
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
