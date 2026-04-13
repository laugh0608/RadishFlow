use std::collections::BTreeMap;
use std::path::Path;

use rf_store::{
    StoredStudioLayoutFile, StoredStudioLayoutPanelState, StoredStudioLayoutRegionWeight,
    StoredStudioLayoutStackGroupState, StoredStudioWindowLayoutEntry, read_studio_layout_file,
    studio_layout_path_for_project, write_studio_layout_file,
};
use rf_types::{RfError, RfResult};

use crate::{
    StudioGuiWindowAreaId, StudioGuiWindowDockRegion, StudioGuiWindowLayoutPersistenceState,
    StudioGuiWindowPanelLayoutState, StudioGuiWindowRegionWeight, StudioGuiWindowStackGroupState,
};

pub fn load_persisted_window_layouts(
    project_path: &Path,
) -> RfResult<BTreeMap<String, StudioGuiWindowLayoutPersistenceState>> {
    let layout_path = studio_layout_path_for_project(project_path);
    if !layout_path.exists() {
        return Ok(BTreeMap::new());
    }

    let stored = read_studio_layout_file(&layout_path)?;
    stored
        .entries
        .into_iter()
        .map(persistence_from_stored_entry)
        .map(|result| result.map(|entry| (entry.layout_key.clone(), entry)))
        .collect()
}

pub fn save_persisted_window_layouts(
    project_path: &Path,
    layouts: &BTreeMap<String, StudioGuiWindowLayoutPersistenceState>,
) -> RfResult<()> {
    let layout_path = studio_layout_path_for_project(project_path);
    let stored = StoredStudioLayoutFile::new(
        layouts
            .values()
            .cloned()
            .map(stored_entry_from_persistence)
            .collect(),
    );
    write_studio_layout_file(&layout_path, &stored)
}

fn persistence_from_stored_entry(
    entry: StoredStudioWindowLayoutEntry,
) -> RfResult<StudioGuiWindowLayoutPersistenceState> {
    Ok(StudioGuiWindowLayoutPersistenceState {
        layout_key: entry.layout_key,
        center_area: parse_area_id(&entry.center_area)?,
        panels: entry
            .panels
            .into_iter()
            .map(panel_state_from_stored)
            .collect::<RfResult<Vec<_>>>()?,
        stack_groups: entry
            .stack_groups
            .into_iter()
            .map(stack_group_state_from_stored)
            .collect::<RfResult<Vec<_>>>()?,
        region_weights: entry
            .region_weights
            .into_iter()
            .map(region_weight_from_stored)
            .collect::<RfResult<Vec<_>>>()?,
    })
}

fn stored_entry_from_persistence(
    entry: StudioGuiWindowLayoutPersistenceState,
) -> StoredStudioWindowLayoutEntry {
    StoredStudioWindowLayoutEntry {
        layout_key: entry.layout_key,
        center_area: area_id_key(entry.center_area).to_string(),
        panels: entry
            .panels
            .into_iter()
            .map(|panel| StoredStudioLayoutPanelState {
                area_id: area_id_key(panel.area_id).to_string(),
                dock_region: dock_region_key(panel.dock_region).to_string(),
                stack_group: panel.stack_group,
                order: panel.order,
                visible: panel.visible,
                collapsed: panel.collapsed,
            })
            .collect(),
        stack_groups: entry
            .stack_groups
            .into_iter()
            .map(|stack_group| StoredStudioLayoutStackGroupState {
                dock_region: dock_region_key(stack_group.dock_region).to_string(),
                stack_group: stack_group.stack_group,
                active_area_id: area_id_key(stack_group.active_area_id).to_string(),
            })
            .collect(),
        region_weights: entry
            .region_weights
            .into_iter()
            .map(|region| StoredStudioLayoutRegionWeight {
                dock_region: dock_region_key(region.dock_region).to_string(),
                weight: region.weight,
            })
            .collect(),
    }
}

fn panel_state_from_stored(
    panel: StoredStudioLayoutPanelState,
) -> RfResult<StudioGuiWindowPanelLayoutState> {
    Ok(StudioGuiWindowPanelLayoutState {
        area_id: parse_area_id(&panel.area_id)?,
        dock_region: parse_dock_region(&panel.dock_region)?,
        stack_group: panel.stack_group,
        order: panel.order,
        visible: panel.visible,
        collapsed: panel.collapsed,
    })
}

fn stack_group_state_from_stored(
    stack_group: StoredStudioLayoutStackGroupState,
) -> RfResult<StudioGuiWindowStackGroupState> {
    Ok(StudioGuiWindowStackGroupState {
        dock_region: parse_dock_region(&stack_group.dock_region)?,
        stack_group: stack_group.stack_group,
        active_area_id: parse_area_id(&stack_group.active_area_id)?,
    })
}

fn region_weight_from_stored(
    region: StoredStudioLayoutRegionWeight,
) -> RfResult<StudioGuiWindowRegionWeight> {
    Ok(StudioGuiWindowRegionWeight {
        dock_region: parse_dock_region(&region.dock_region)?,
        weight: region.weight,
    })
}

fn area_id_key(area_id: StudioGuiWindowAreaId) -> &'static str {
    match area_id {
        StudioGuiWindowAreaId::Commands => "commands",
        StudioGuiWindowAreaId::Canvas => "canvas",
        StudioGuiWindowAreaId::Runtime => "runtime",
    }
}

fn parse_area_id(value: &str) -> RfResult<StudioGuiWindowAreaId> {
    match value {
        "commands" => Ok(StudioGuiWindowAreaId::Commands),
        "canvas" => Ok(StudioGuiWindowAreaId::Canvas),
        "runtime" => Ok(StudioGuiWindowAreaId::Runtime),
        other => Err(RfError::invalid_input(format!(
            "unsupported stored studio layout area `{other}`"
        ))),
    }
}

fn dock_region_key(region: StudioGuiWindowDockRegion) -> &'static str {
    match region {
        StudioGuiWindowDockRegion::LeftSidebar => "left-sidebar",
        StudioGuiWindowDockRegion::CenterStage => "center-stage",
        StudioGuiWindowDockRegion::RightSidebar => "right-sidebar",
    }
}

fn parse_dock_region(value: &str) -> RfResult<StudioGuiWindowDockRegion> {
    match value {
        "left-sidebar" => Ok(StudioGuiWindowDockRegion::LeftSidebar),
        "center-stage" => Ok(StudioGuiWindowDockRegion::CenterStage),
        "right-sidebar" => Ok(StudioGuiWindowDockRegion::RightSidebar),
        other => Err(RfError::invalid_input(format!(
            "unsupported stored studio layout dock region `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        StudioGuiWindowAreaId, StudioGuiWindowDockRegion, StudioGuiWindowLayoutPersistenceState,
        StudioGuiWindowPanelLayoutState, StudioGuiWindowRegionWeight,
        StudioGuiWindowStackGroupState,
    };

    use super::{load_persisted_window_layouts, save_persisted_window_layouts};

    #[test]
    fn layout_store_round_trips_persisted_window_layouts() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("expected time after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("radishflow-layout-store-{unique}"));
        fs::create_dir_all(&root).expect("expected temp root");
        let project_path = root.join("demo.rfproj.json");
        fs::write(&project_path, "{}").expect("expected temp project file");

        let layouts = BTreeMap::from([(
            "studio.window.owner.slot-1".to_string(),
            StudioGuiWindowLayoutPersistenceState {
                layout_key: "studio.window.owner.slot-1".to_string(),
                center_area: StudioGuiWindowAreaId::Runtime,
                panels: vec![
                    StudioGuiWindowPanelLayoutState {
                        area_id: StudioGuiWindowAreaId::Commands,
                        dock_region: StudioGuiWindowDockRegion::RightSidebar,
                        stack_group: 20,
                        order: 12,
                        visible: true,
                        collapsed: false,
                    },
                    StudioGuiWindowPanelLayoutState {
                        area_id: StudioGuiWindowAreaId::Runtime,
                        dock_region: StudioGuiWindowDockRegion::CenterStage,
                        stack_group: 10,
                        order: 5,
                        visible: true,
                        collapsed: true,
                    },
                ],
                stack_groups: vec![
                    StudioGuiWindowStackGroupState {
                        dock_region: StudioGuiWindowDockRegion::CenterStage,
                        stack_group: 10,
                        active_area_id: StudioGuiWindowAreaId::Runtime,
                    },
                    StudioGuiWindowStackGroupState {
                        dock_region: StudioGuiWindowDockRegion::RightSidebar,
                        stack_group: 20,
                        active_area_id: StudioGuiWindowAreaId::Commands,
                    },
                ],
                region_weights: vec![StudioGuiWindowRegionWeight {
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    weight: 31,
                }],
            },
        )]);

        save_persisted_window_layouts(&project_path, &layouts).expect("expected save");
        let loaded = load_persisted_window_layouts(&project_path).expect("expected load");

        assert_eq!(loaded, layouts);

        let _ = fs::remove_dir_all(&root);
    }
}
