use crate::{
    StudioAppHostUiCommandGroup, StudioAppHostUiCommandModel, StudioGuiCanvasActionId,
    StudioGuiCanvasObjectListItemViewModel, StudioGuiCanvasState, StudioWindowHostId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StudioGuiShortcutModifier {
    Ctrl,
    Shift,
    Alt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StudioGuiShortcutKey {
    S,
    Z,
    Y,
    F5,
    F6,
    F8,
    Tab,
    Escape,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StudioGuiShortcut {
    pub modifiers: Vec<StudioGuiShortcutModifier>,
    pub key: StudioGuiShortcutKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCommandEntry {
    pub command_id: String,
    pub label: String,
    pub detail: String,
    pub enabled: bool,
    pub sort_order: u16,
    pub target_window_id: Option<StudioWindowHostId>,
    pub menu_path: Vec<String>,
    pub search_terms: Vec<String>,
    pub shortcut: Option<StudioGuiShortcut>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCommandPresentation {
    pub label: String,
    pub label_with_shortcut: String,
    pub palette_label: String,
    pub shortcut_label: Option<String>,
    pub menu_path_text: String,
    pub hover_text: String,
}

impl StudioGuiCommandEntry {
    pub fn matches_palette_query(&self, query: &str) -> bool {
        let terms = normalize_palette_query_terms(query);
        if terms.is_empty() {
            return true;
        }

        let mut fields = Vec::with_capacity(2 + self.search_terms.len());
        fields.push(normalize_palette_query_field(&self.label));
        fields.push(normalize_palette_query_field(&self.menu_path.join(" ")));
        fields.extend(
            self.search_terms
                .iter()
                .map(|term| normalize_palette_query_field(term)),
        );

        terms
            .iter()
            .all(|term| fields.iter().any(|field| field.contains(term)))
    }

    pub fn presentation(&self) -> StudioGuiCommandPresentation {
        let shortcut_label = self.shortcut.as_ref().map(format_shortcut);
        let label_with_shortcut = match shortcut_label.as_ref() {
            Some(shortcut) => format!("{} ({shortcut})", self.label),
            None => self.label.clone(),
        };
        let palette_label = if self.enabled {
            label_with_shortcut.clone()
        } else {
            format!("{label_with_shortcut} [disabled]")
        };
        let menu_path_text = self.menu_path.join(" > ");
        let hover_text = if menu_path_text.is_empty() {
            self.detail.clone()
        } else {
            format!("{}\nMenu: {menu_path_text}", self.detail)
        };

        StudioGuiCommandPresentation {
            label: self.label.clone(),
            label_with_shortcut,
            palette_label,
            shortcut_label,
            menu_path_text,
            hover_text,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCommandSection {
    pub group: StudioGuiCommandGroup,
    pub title: &'static str,
    pub commands: Vec<StudioGuiCommandEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCommandMenuNode {
    pub label: String,
    pub command: Option<StudioGuiCommandMenuCommandModel>,
    pub children: Vec<StudioGuiCommandMenuNode>,
    sort_order: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiCommandMenuCommandModel {
    pub command_id: String,
    pub enabled: bool,
    pub label: String,
    pub hover_text: String,
    pub target_window_id: Option<StudioWindowHostId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioGuiCommandGroup {
    File,
    Edit,
    RunPanel,
    Recovery,
    Entitlement,
    Canvas,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StudioGuiCommandRegistry {
    pub sections: Vec<StudioGuiCommandSection>,
}

impl StudioGuiCommandRegistry {
    pub fn from_model(model: &StudioAppHostUiCommandModel) -> Self {
        Self::from_surfaces(model, &StudioGuiCanvasState::default(), None)
    }

    pub fn from_surfaces(
        model: &StudioAppHostUiCommandModel,
        canvas: &StudioGuiCanvasState,
        canvas_target_window_id: Option<StudioWindowHostId>,
    ) -> Self {
        let mut run_panel = Vec::new();
        let mut file = Vec::new();
        let mut edit = Vec::new();
        let mut recovery = Vec::new();
        let mut entitlement = Vec::new();
        let mut canvas_commands = Vec::new();

        for action in &model.actions {
            let defaults = command_defaults(action.command_id);
            let entry = StudioGuiCommandEntry {
                command_id: action.command_id.to_string(),
                label: action.label.to_string(),
                detail: action.detail.to_string(),
                enabled: action.enabled,
                sort_order: action.sort_order,
                target_window_id: action.target_window_id,
                menu_path: defaults
                    .menu_path
                    .iter()
                    .map(|segment| (*segment).to_string())
                    .collect(),
                search_terms: defaults
                    .search_terms
                    .iter()
                    .map(|term| (*term).to_string())
                    .collect(),
                shortcut: defaults.shortcut,
            };
            match action.group {
                StudioAppHostUiCommandGroup::File => file.push(entry),
                StudioAppHostUiCommandGroup::Edit => edit.push(entry),
                StudioAppHostUiCommandGroup::RunPanel => run_panel.push(entry),
                StudioAppHostUiCommandGroup::Recovery => recovery.push(entry),
                StudioAppHostUiCommandGroup::Entitlement => entitlement.push(entry),
            }
        }

        let widget = canvas.widget();
        for action in &widget.actions {
            let is_place_unit = matches!(action.id, StudioGuiCanvasActionId::BeginPlaceUnit(_));
            let is_layout_nudge = matches!(action.id, StudioGuiCanvasActionId::MoveSelectedUnit(_));
            let should_include = if is_place_unit {
                canvas_target_window_id.is_some()
            } else if is_layout_nudge {
                canvas_target_window_id.is_some() && action.enabled
            } else {
                !canvas.suggestions.is_empty() || canvas.pending_edit.is_some()
            };
            if should_include {
                let defaults = command_defaults(action.command_id.as_str());
                canvas_commands.push(StudioGuiCommandEntry {
                    command_id: action.command_id.to_string(),
                    label: action.label.to_string(),
                    detail: action.detail.to_string(),
                    enabled: action.enabled,
                    sort_order: canvas_sort_order(action.id),
                    target_window_id: canvas_target_window_id,
                    menu_path: defaults
                        .menu_path
                        .iter()
                        .map(|segment| (*segment).to_string())
                        .collect(),
                    search_terms: defaults
                        .search_terms
                        .iter()
                        .map(|term| (*term).to_string())
                        .collect(),
                    shortcut: defaults.shortcut,
                });
            }
        }
        if canvas_target_window_id.is_some() {
            canvas_commands.extend(widget.view().object_list.items.iter().enumerate().map(
                |(index, item)| {
                    canvas_object_navigation_command_entry(item, index, canvas_target_window_id)
                },
            ));
        }

        let mut sections = Vec::new();
        if !file.is_empty() {
            file.sort_by_key(|entry| entry.sort_order);
            sections.push(StudioGuiCommandSection {
                group: StudioGuiCommandGroup::File,
                title: "File",
                commands: file,
            });
        }
        if !edit.is_empty() {
            edit.sort_by_key(|entry| entry.sort_order);
            sections.push(StudioGuiCommandSection {
                group: StudioGuiCommandGroup::Edit,
                title: "Edit",
                commands: edit,
            });
        }
        if !run_panel.is_empty() {
            run_panel.sort_by_key(|entry| entry.sort_order);
            sections.push(StudioGuiCommandSection {
                group: StudioGuiCommandGroup::RunPanel,
                title: "Run Panel",
                commands: run_panel,
            });
        }
        if !recovery.is_empty() {
            recovery.sort_by_key(|entry| entry.sort_order);
            sections.push(StudioGuiCommandSection {
                group: StudioGuiCommandGroup::Recovery,
                title: "Recovery",
                commands: recovery,
            });
        }
        if !entitlement.is_empty() {
            entitlement.sort_by_key(|entry| entry.sort_order);
            sections.push(StudioGuiCommandSection {
                group: StudioGuiCommandGroup::Entitlement,
                title: "Entitlement",
                commands: entitlement,
            });
        }
        if !canvas_commands.is_empty() {
            canvas_commands.sort_by_key(|entry| entry.sort_order);
            sections.push(StudioGuiCommandSection {
                group: StudioGuiCommandGroup::Canvas,
                title: "Canvas",
                commands: canvas_commands,
            });
        }

        Self { sections }
    }

    pub fn find_by_shortcut(&self, shortcut: &StudioGuiShortcut) -> Option<&StudioGuiCommandEntry> {
        self.sections
            .iter()
            .flat_map(|section| section.commands.iter())
            .find(|entry| entry.shortcut.as_ref() == Some(shortcut))
    }

    pub fn command(&self, command_id: &str) -> Option<&StudioGuiCommandEntry> {
        self.sections
            .iter()
            .flat_map(|section| section.commands.iter())
            .find(|entry| entry.command_id == command_id)
    }

    pub fn filtered_commands(&self, query: &str) -> Vec<&StudioGuiCommandEntry> {
        self.sections
            .iter()
            .flat_map(|section| section.commands.iter())
            .filter(|entry| entry.matches_palette_query(query))
            .collect()
    }

    pub fn menu_tree(&self) -> Vec<StudioGuiCommandMenuNode> {
        let mut roots = Vec::new();
        for command in self
            .sections
            .iter()
            .flat_map(|section| section.commands.iter())
        {
            let path = if command.menu_path.is_empty() {
                vec![command.label.clone()]
            } else {
                command.menu_path.clone()
            };
            insert_menu_command(&mut roots, &path, command);
        }
        sort_menu_nodes(&mut roots);
        roots
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StudioGuiCommandDefaults {
    menu_path: &'static [&'static str],
    search_terms: &'static [&'static str],
    shortcut: Option<StudioGuiShortcut>,
}

fn command_defaults(command_id: &str) -> StudioGuiCommandDefaults {
    match command_id {
        "file.save" => StudioGuiCommandDefaults {
            menu_path: &["File", "Save"],
            search_terms: &["file", "save", "project"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                key: StudioGuiShortcutKey::S,
            }),
        },
        "edit.undo" => StudioGuiCommandDefaults {
            menu_path: &["Edit", "Undo"],
            search_terms: &["edit", "undo", "history"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                key: StudioGuiShortcutKey::Z,
            }),
        },
        "edit.redo" => StudioGuiCommandDefaults {
            menu_path: &["Edit", "Redo"],
            search_terms: &["edit", "redo", "history"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                key: StudioGuiShortcutKey::Y,
            }),
        },
        "run_panel.run_manual" => StudioGuiCommandDefaults {
            menu_path: &["Run", "Run Workspace"],
            search_terms: &["run", "workspace", "manual", "solve"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::F5,
            }),
        },
        "run_panel.resume_workspace" => StudioGuiCommandDefaults {
            menu_path: &["Run", "Resume Workspace"],
            search_terms: &["resume", "workspace", "continue", "solve"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Shift],
                key: StudioGuiShortcutKey::F5,
            }),
        },
        "run_panel.set_hold" => StudioGuiCommandDefaults {
            menu_path: &["Run", "Hold Workspace"],
            search_terms: &["hold", "pause", "workspace", "simulation"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::F6,
            }),
        },
        "run_panel.set_active" => StudioGuiCommandDefaults {
            menu_path: &["Run", "Activate Workspace"],
            search_terms: &["active", "activate", "workspace", "simulation"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Shift],
                key: StudioGuiShortcutKey::F6,
            }),
        },
        "run_panel.recover_failure" => StudioGuiCommandDefaults {
            menu_path: &["Run", "Recovery", "Recover Run Panel Failure"],
            search_terms: &["recover", "failure", "run panel", "diagnostic", "inspector"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::F8,
            }),
        },
        "entitlement.sync" => StudioGuiCommandDefaults {
            menu_path: &["Entitlement", "Sync Entitlement"],
            search_terms: &["entitlement", "sync", "license", "package manifests"],
            shortcut: None,
        },
        "entitlement.refresh_offline_lease" => StudioGuiCommandDefaults {
            menu_path: &["Entitlement", "Refresh Offline Lease"],
            search_terms: &["entitlement", "offline", "lease", "refresh"],
            shortcut: None,
        },
        "canvas.accept_focused" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Accept Suggestion"],
            search_terms: &["canvas", "accept", "suggestion", "apply"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::Tab,
            }),
        },
        "canvas.reject_focused" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Reject Suggestion"],
            search_terms: &["canvas", "reject", "dismiss", "suggestion"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::Escape,
            }),
        },
        "canvas.focus_next" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Next Suggestion"],
            search_terms: &["canvas", "next", "focus", "suggestion"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                key: StudioGuiShortcutKey::Tab,
            }),
        },
        "canvas.focus_previous" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Previous Suggestion"],
            search_terms: &["canvas", "previous", "focus", "suggestion"],
            shortcut: Some(StudioGuiShortcut {
                modifiers: vec![
                    StudioGuiShortcutModifier::Ctrl,
                    StudioGuiShortcutModifier::Shift,
                ],
                key: StudioGuiShortcutKey::Tab,
            }),
        },
        "canvas.cancel_pending_edit" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Cancel Pending Edit"],
            search_terms: &["canvas", "cancel", "pending", "edit"],
            shortcut: None,
        },
        "canvas.begin_place_unit.feed" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Place Unit", "Feed"],
            search_terms: &["canvas", "place", "unit", "feed"],
            shortcut: None,
        },
        "canvas.begin_place_unit.mixer" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Place Unit", "Mixer"],
            search_terms: &["canvas", "place", "unit", "mixer"],
            shortcut: None,
        },
        "canvas.begin_place_unit.heater" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Place Unit", "Heater"],
            search_terms: &["canvas", "place", "unit", "heater"],
            shortcut: None,
        },
        "canvas.begin_place_unit.cooler" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Place Unit", "Cooler"],
            search_terms: &["canvas", "place", "unit", "cooler"],
            shortcut: None,
        },
        "canvas.begin_place_unit.valve" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Place Unit", "Valve"],
            search_terms: &["canvas", "place", "unit", "valve"],
            shortcut: None,
        },
        "canvas.begin_place_unit.flash_drum" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Place Unit", "Flash Drum"],
            search_terms: &["canvas", "place", "unit", "flash drum"],
            shortcut: None,
        },
        "canvas.move_selected_unit.left" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Layout", "Move Left"],
            search_terms: &["canvas", "layout", "unit", "move", "left", "nudge"],
            shortcut: None,
        },
        "canvas.move_selected_unit.right" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Layout", "Move Right"],
            search_terms: &["canvas", "layout", "unit", "move", "right", "nudge"],
            shortcut: None,
        },
        "canvas.move_selected_unit.up" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Layout", "Move Up"],
            search_terms: &["canvas", "layout", "unit", "move", "up", "nudge"],
            shortcut: None,
        },
        "canvas.move_selected_unit.down" => StudioGuiCommandDefaults {
            menu_path: &["Canvas", "Layout", "Move Down"],
            search_terms: &["canvas", "layout", "unit", "move", "down", "nudge"],
            shortcut: None,
        },
        _ => StudioGuiCommandDefaults {
            menu_path: &["Commands"],
            search_terms: &[],
            shortcut: None,
        },
    }
}

fn canvas_sort_order(action_id: StudioGuiCanvasActionId) -> u16 {
    match action_id {
        StudioGuiCanvasActionId::BeginPlaceUnit(kind) => 290 + kind.sort_index(),
        StudioGuiCanvasActionId::AcceptFocused => 300,
        StudioGuiCanvasActionId::RejectFocused => 310,
        StudioGuiCanvasActionId::FocusNext => 320,
        StudioGuiCanvasActionId::FocusPrevious => 330,
        StudioGuiCanvasActionId::CancelPendingEdit => 340,
        StudioGuiCanvasActionId::MoveSelectedUnit(direction) => match direction {
            crate::StudioGuiCanvasUnitLayoutNudgeDirection::Left => 350,
            crate::StudioGuiCanvasUnitLayoutNudgeDirection::Up => 360,
            crate::StudioGuiCanvasUnitLayoutNudgeDirection::Down => 370,
            crate::StudioGuiCanvasUnitLayoutNudgeDirection::Right => 380,
        },
    }
}

fn canvas_object_navigation_command_entry(
    item: &StudioGuiCanvasObjectListItemViewModel,
    index: usize,
    target_window_id: Option<StudioWindowHostId>,
) -> StudioGuiCommandEntry {
    let target = item.command_target();
    let sort_order = 400u16.saturating_add(index.min(u16::MAX as usize - 400) as u16);
    let label = format!("Locate {} {}", target.kind_label, target.label);
    let anchor = target.viewport_anchor_label.as_deref().unwrap_or("none");
    let detail = format!(
        "Open the {} Inspector for `{}` and request Canvas viewport focus at `{}`. {}",
        target.kind_label, target.target_id, anchor, item.detail
    );

    StudioGuiCommandEntry {
        command_id: target.command_id,
        label,
        detail,
        enabled: true,
        sort_order,
        target_window_id,
        menu_path: vec![
            "Canvas".to_string(),
            "Objects".to_string(),
            target.kind_label.to_string(),
            target.label.clone(),
        ],
        search_terms: vec![
            "canvas".to_string(),
            "object".to_string(),
            "objects".to_string(),
            "locate".to_string(),
            "focus".to_string(),
            "viewport".to_string(),
            target.kind_label.to_string(),
            target.target_id,
            target.label,
            item.detail.clone(),
            anchor.to_string(),
        ],
        shortcut: None,
    }
}

fn insert_menu_command(
    nodes: &mut Vec<StudioGuiCommandMenuNode>,
    path: &[String],
    command: &StudioGuiCommandEntry,
) {
    if path.is_empty() {
        return;
    }

    if path.len() == 1 {
        let presentation = command.presentation();
        nodes.push(StudioGuiCommandMenuNode {
            label: path[0].clone(),
            command: Some(StudioGuiCommandMenuCommandModel {
                command_id: command.command_id.clone(),
                enabled: command.enabled,
                label: presentation.label_with_shortcut,
                hover_text: presentation.hover_text,
                target_window_id: command.target_window_id,
            }),
            children: Vec::new(),
            sort_order: command.sort_order,
        });
        return;
    }

    let label = &path[0];
    let index = nodes
        .iter()
        .position(|node| node.command.is_none() && node.label == *label)
        .unwrap_or_else(|| {
            nodes.push(StudioGuiCommandMenuNode {
                label: label.clone(),
                command: None,
                children: Vec::new(),
                sort_order: command.sort_order,
            });
            nodes.len() - 1
        });
    nodes[index].sort_order = nodes[index].sort_order.min(command.sort_order);
    insert_menu_command(&mut nodes[index].children, &path[1..], command);
}

fn sort_menu_nodes(nodes: &mut [StudioGuiCommandMenuNode]) {
    for node in nodes.iter_mut() {
        sort_menu_nodes(&mut node.children);
    }
    nodes.sort_by(|left, right| {
        left.sort_order
            .cmp(&right.sort_order)
            .then_with(|| left.label.cmp(&right.label))
    });
}

fn normalize_palette_query_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(normalize_palette_query_field)
        .filter(|term| !term.is_empty())
        .collect()
}

fn normalize_palette_query_field(value: &str) -> String {
    value.trim().to_lowercase()
}

fn format_shortcut(shortcut: &StudioGuiShortcut) -> String {
    let mut parts = Vec::new();
    for modifier in &shortcut.modifiers {
        let label = match modifier {
            StudioGuiShortcutModifier::Ctrl => "Ctrl",
            StudioGuiShortcutModifier::Shift => "Shift",
            StudioGuiShortcutModifier::Alt => "Alt",
        };
        parts.push(label);
    }
    let key = match shortcut.key {
        StudioGuiShortcutKey::S => "S",
        StudioGuiShortcutKey::Z => "Z",
        StudioGuiShortcutKey::Y => "Y",
        StudioGuiShortcutKey::F5 => "F5",
        StudioGuiShortcutKey::F6 => "F6",
        StudioGuiShortcutKey::F8 => "F8",
        StudioGuiShortcutKey::Tab => "Tab",
        StudioGuiShortcutKey::Escape => "Escape",
    };
    parts.push(key);
    parts.join("+")
}

#[cfg(test)]
mod tests;
