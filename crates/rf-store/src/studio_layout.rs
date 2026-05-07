use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use rf_types::{RfError, RfResult};
use serde::{Deserialize, Serialize};

use crate::STORED_PROJECT_FILE_EXTENSION;

pub const STORED_STUDIO_LAYOUT_FILE_KIND: &str = "radishflow.studio-layout-file";
pub const STORED_STUDIO_LAYOUT_SCHEMA_VERSION: u32 = 1;
pub const STORED_STUDIO_LAYOUT_FILE_SUFFIX: &str = ".rfstudio-layout.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredStudioLayoutPanelState {
    pub area_id: String,
    pub dock_region: String,
    #[serde(default = "default_studio_layout_stack_group")]
    pub stack_group: u8,
    pub order: u8,
    pub visible: bool,
    pub collapsed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredStudioLayoutStackGroupState {
    pub dock_region: String,
    pub stack_group: u8,
    pub active_area_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredStudioLayoutRegionWeight {
    pub dock_region: String,
    pub weight: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredStudioCanvasUnitPosition {
    pub unit_id: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredStudioWindowLayoutEntry {
    pub layout_key: String,
    pub center_area: String,
    pub panels: Vec<StoredStudioLayoutPanelState>,
    #[serde(default)]
    pub stack_groups: Vec<StoredStudioLayoutStackGroupState>,
    pub region_weights: Vec<StoredStudioLayoutRegionWeight>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredStudioLayoutFile {
    pub kind: String,
    pub schema_version: u32,
    pub entries: Vec<StoredStudioWindowLayoutEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canvas_unit_positions: Vec<StoredStudioCanvasUnitPosition>,
}

impl StoredStudioLayoutFile {
    pub fn new(entries: Vec<StoredStudioWindowLayoutEntry>) -> Self {
        Self {
            kind: STORED_STUDIO_LAYOUT_FILE_KIND.to_string(),
            schema_version: STORED_STUDIO_LAYOUT_SCHEMA_VERSION,
            entries,
            canvas_unit_positions: Vec::new(),
        }
    }

    pub fn with_canvas_unit_positions(
        mut self,
        canvas_unit_positions: Vec<StoredStudioCanvasUnitPosition>,
    ) -> Self {
        self.canvas_unit_positions = canvas_unit_positions;
        self
    }

    pub fn validate(&self) -> RfResult<()> {
        if self.kind != STORED_STUDIO_LAYOUT_FILE_KIND {
            return Err(RfError::invalid_input(format!(
                "unsupported stored studio layout file kind `{}`",
                self.kind
            )));
        }

        if self.schema_version != STORED_STUDIO_LAYOUT_SCHEMA_VERSION {
            return Err(RfError::invalid_input(format!(
                "unsupported stored studio layout file schema version `{}`",
                self.schema_version
            )));
        }

        let mut layout_keys = BTreeSet::new();
        for entry in &self.entries {
            entry.validate()?;
            if !layout_keys.insert(entry.layout_key.clone()) {
                return Err(RfError::invalid_input(format!(
                    "stored studio layout file contains duplicate layout key `{}`",
                    entry.layout_key
                )));
            }
        }

        let mut unit_ids = BTreeSet::new();
        for position in &self.canvas_unit_positions {
            position.validate()?;
            if !unit_ids.insert(position.unit_id.clone()) {
                return Err(RfError::invalid_input(format!(
                    "stored studio layout file contains duplicate canvas unit position `{}`",
                    position.unit_id
                )));
            }
        }

        Ok(())
    }
}

impl StoredStudioWindowLayoutEntry {
    pub fn validate(&self) -> RfResult<()> {
        if self.layout_key.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored studio window layout entry must contain a non-empty layout_key",
            ));
        }
        if self.center_area.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored studio window layout entry must contain a non-empty center_area",
            ));
        }

        let mut area_ids = BTreeSet::new();
        for panel in &self.panels {
            panel.validate()?;
            if !area_ids.insert(panel.area_id.clone()) {
                return Err(RfError::invalid_input(format!(
                    "stored studio window layout entry `{}` contains duplicate panel `{}`",
                    self.layout_key, panel.area_id
                )));
            }
        }

        let mut stack_keys = BTreeSet::new();
        for stack_group in &self.stack_groups {
            stack_group.validate()?;
            if !stack_keys.insert((stack_group.dock_region.clone(), stack_group.stack_group)) {
                return Err(RfError::invalid_input(format!(
                    "stored studio window layout entry `{}` contains duplicate stack group `{}:{}`",
                    self.layout_key, stack_group.dock_region, stack_group.stack_group
                )));
            }
            if !self.panels.iter().any(|panel| {
                panel.area_id == stack_group.active_area_id
                    && panel.dock_region == stack_group.dock_region
                    && panel.stack_group == stack_group.stack_group
            }) {
                return Err(RfError::invalid_input(format!(
                    "stored studio window layout entry `{}` contains stack group `{}:{}` whose active area `{}` does not exist in the same stack",
                    self.layout_key,
                    stack_group.dock_region,
                    stack_group.stack_group,
                    stack_group.active_area_id
                )));
            }
        }

        let mut dock_regions = BTreeSet::new();
        for region in &self.region_weights {
            region.validate()?;
            if !dock_regions.insert(region.dock_region.clone()) {
                return Err(RfError::invalid_input(format!(
                    "stored studio window layout entry `{}` contains duplicate region weight `{}`",
                    self.layout_key, region.dock_region
                )));
            }
        }

        Ok(())
    }
}

impl StoredStudioLayoutPanelState {
    pub fn validate(&self) -> RfResult<()> {
        if self.area_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored studio layout panel state must contain a non-empty area_id",
            ));
        }
        if self.dock_region.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored studio layout panel state must contain a non-empty dock_region",
            ));
        }
        if self.stack_group == 0 {
            return Err(RfError::invalid_input(
                "stored studio layout panel state must contain a stack_group greater than zero",
            ));
        }
        Ok(())
    }
}

fn default_studio_layout_stack_group() -> u8 {
    10
}

impl StoredStudioLayoutStackGroupState {
    pub fn validate(&self) -> RfResult<()> {
        if self.dock_region.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored studio layout stack group state must contain a non-empty dock_region",
            ));
        }
        if self.stack_group == 0 {
            return Err(RfError::invalid_input(
                "stored studio layout stack group state must contain a stack_group greater than zero",
            ));
        }
        if self.active_area_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored studio layout stack group state must contain a non-empty active_area_id",
            ));
        }
        Ok(())
    }
}

impl StoredStudioLayoutRegionWeight {
    pub fn validate(&self) -> RfResult<()> {
        if self.dock_region.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored studio layout region weight must contain a non-empty dock_region",
            ));
        }
        if self.weight == 0 {
            return Err(RfError::invalid_input(
                "stored studio layout region weight must be greater than zero",
            ));
        }
        Ok(())
    }
}

impl StoredStudioCanvasUnitPosition {
    pub fn validate(&self) -> RfResult<()> {
        if self.unit_id.trim().is_empty() {
            return Err(RfError::invalid_input(
                "stored studio canvas unit position must contain a non-empty unit_id",
            ));
        }
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(RfError::invalid_input(format!(
                "stored studio canvas unit position `{}` must contain finite coordinates",
                self.unit_id
            )));
        }
        Ok(())
    }
}

pub fn studio_layout_path_for_project(project_path: impl AsRef<Path>) -> PathBuf {
    let project_path = project_path.as_ref();
    let parent = project_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let file_name = project_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string());

    let layout_name = match file_name.strip_suffix(STORED_PROJECT_FILE_EXTENSION) {
        Some(stem) => format!("{stem}{STORED_STUDIO_LAYOUT_FILE_SUFFIX}"),
        None => format!("{file_name}{STORED_STUDIO_LAYOUT_FILE_SUFFIX}"),
    };

    parent.join(layout_name)
}
