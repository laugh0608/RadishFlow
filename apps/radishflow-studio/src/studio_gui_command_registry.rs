use crate::{
    StudioAppHostUiCommandGroup, StudioAppHostUiCommandModel, StudioGuiCanvasActionId,
    StudioGuiCanvasState, StudioWindowHostId,
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

        if !canvas.suggestions.is_empty() || canvas.pending_edit.is_some() {
            let widget = canvas.widget();
            for action in widget.actions {
                let defaults = command_defaults(action.command_id);
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
        _ => StudioGuiCommandDefaults {
            menu_path: &["Commands"],
            search_terms: &[],
            shortcut: None,
        },
    }
}

fn canvas_sort_order(action_id: StudioGuiCanvasActionId) -> u16 {
    match action_id {
        StudioGuiCanvasActionId::AcceptFocused => 300,
        StudioGuiCanvasActionId::RejectFocused => 310,
        StudioGuiCanvasActionId::FocusNext => 320,
        StudioGuiCanvasActionId::FocusPrevious => 330,
        StudioGuiCanvasActionId::CancelPendingEdit => 340,
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
mod tests {
    use crate::{
        StudioAppHostUiActionModel, StudioAppHostUiCommandGroup, StudioAppHostUiCommandModel,
        StudioGuiCanvasActionId, StudioGuiCommandEntry, StudioGuiCommandGroup,
        StudioGuiCommandMenuCommandModel, StudioGuiCommandMenuNode, StudioGuiCommandRegistry,
        StudioGuiShortcut, StudioGuiShortcutKey, StudioGuiShortcutModifier,
        studio_gui_canvas_widget::canvas_command_id,
    };

    #[test]
    fn gui_command_registry_groups_commands_by_surface_group_and_sort_order() {
        let model = StudioAppHostUiCommandModel {
            actions: vec![
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.set_active",
                    group: StudioAppHostUiCommandGroup::RunPanel,
                    sort_order: 130,
                    label: "Activate workspace",
                    enabled: true,
                    detail: "Activate",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.run_manual",
                    group: StudioAppHostUiCommandGroup::RunPanel,
                    sort_order: 100,
                    label: "Run workspace",
                    enabled: true,
                    detail: "Run",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.recover_failure",
                    group: StudioAppHostUiCommandGroup::Recovery,
                    sort_order: 200,
                    label: "Recover run panel failure",
                    enabled: false,
                    detail: "Recover",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "entitlement.sync",
                    group: StudioAppHostUiCommandGroup::Entitlement,
                    sort_order: 300,
                    label: "Sync entitlement",
                    enabled: true,
                    detail: "Sync",
                    target_window_id: Some(2),
                },
            ],
        };

        let registry = StudioGuiCommandRegistry::from_model(&model);

        assert_eq!(registry.sections.len(), 3);
        assert_eq!(registry.sections[0].title, "Run Panel");
        assert_eq!(
            registry.sections[0]
                .commands
                .iter()
                .map(|entry| entry.command_id.as_str())
                .collect::<Vec<_>>(),
            vec!["run_panel.run_manual", "run_panel.set_active"]
        );
        assert_eq!(registry.sections[1].title, "Recovery");
        assert_eq!(
            registry.sections[1]
                .commands
                .iter()
                .map(|entry| entry.command_id.as_str())
                .collect::<Vec<_>>(),
            vec!["run_panel.recover_failure"]
        );
        assert_eq!(registry.sections[2].title, "Entitlement");
        assert_eq!(
            registry.sections[2]
                .commands
                .iter()
                .map(|entry| entry.command_id.as_str())
                .collect::<Vec<_>>(),
            vec!["entitlement.sync"]
        );
        assert_eq!(
            registry.sections[0].commands[0].menu_path,
            vec!["Run".to_string(), "Run Workspace".to_string()]
        );
        assert_eq!(
            registry.sections[0].commands[0].shortcut,
            Some(StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::F5,
            })
        );
        assert_eq!(
            registry.sections[0].commands[1].shortcut,
            Some(StudioGuiShortcut {
                modifiers: vec![StudioGuiShortcutModifier::Shift],
                key: StudioGuiShortcutKey::F6,
            })
        );
        assert_eq!(registry.sections[0].group, StudioGuiCommandGroup::RunPanel);
    }

    #[test]
    fn gui_command_registry_finds_command_by_shortcut() {
        let model = StudioAppHostUiCommandModel {
            actions: vec![StudioAppHostUiActionModel {
                action: None,
                command_id: "run_panel.resume_workspace",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 110,
                label: "Resume workspace",
                enabled: true,
                detail: "Resume",
                target_window_id: Some(2),
            }],
        };

        let registry = StudioGuiCommandRegistry::from_model(&model);
        let shortcut = StudioGuiShortcut {
            modifiers: vec![StudioGuiShortcutModifier::Shift],
            key: StudioGuiShortcutKey::F5,
        };

        let command = registry
            .find_by_shortcut(&shortcut)
            .expect("expected command from shortcut");

        assert_eq!(command.command_id, "run_panel.resume_workspace");
    }

    #[test]
    fn gui_command_registry_assigns_file_and_history_shortcuts() {
        let model = StudioAppHostUiCommandModel {
            actions: vec![
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "file.save",
                    group: StudioAppHostUiCommandGroup::File,
                    sort_order: 10,
                    label: "Save",
                    enabled: true,
                    detail: "Save",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "edit.undo",
                    group: StudioAppHostUiCommandGroup::Edit,
                    sort_order: 20,
                    label: "Undo",
                    enabled: true,
                    detail: "Undo",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "edit.redo",
                    group: StudioAppHostUiCommandGroup::Edit,
                    sort_order: 30,
                    label: "Redo",
                    enabled: true,
                    detail: "Redo",
                    target_window_id: Some(2),
                },
            ],
        };

        let registry = StudioGuiCommandRegistry::from_model(&model);

        for (shortcut, command_id) in [
            (
                StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                    key: StudioGuiShortcutKey::S,
                },
                "file.save",
            ),
            (
                StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                    key: StudioGuiShortcutKey::Z,
                },
                "edit.undo",
            ),
            (
                StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                    key: StudioGuiShortcutKey::Y,
                },
                "edit.redo",
            ),
        ] {
            assert_eq!(
                registry
                    .find_by_shortcut(&shortcut)
                    .map(|command| command.command_id.as_str()),
                Some(command_id)
            );
        }
    }

    #[test]
    fn gui_command_registry_includes_canvas_commands_when_suggestions_exist() {
        let canvas = crate::StudioGuiCanvasState {
            suggestions: vec![
                rf_ui::CanvasSuggestion::new(
                    rf_ui::CanvasSuggestionId::new("sug-a"),
                    rf_ui::SuggestionSource::LocalRules,
                    0.9,
                    rf_ui::GhostElement {
                        kind: rf_ui::GhostElementKind::Connection,
                        target_unit_id: rf_types::UnitId::new("flash-1"),
                        visual_kind: rf_ui::StreamVisualKind::Material,
                        visual_state: rf_ui::StreamVisualState::Suggested,
                    },
                    "test",
                ),
                rf_ui::CanvasSuggestion::new(
                    rf_ui::CanvasSuggestionId::new("sug-b"),
                    rf_ui::SuggestionSource::LocalRules,
                    0.7,
                    rf_ui::GhostElement {
                        kind: rf_ui::GhostElementKind::Connection,
                        target_unit_id: rf_types::UnitId::new("flash-1"),
                        visual_kind: rf_ui::StreamVisualKind::Material,
                        visual_state: rf_ui::StreamVisualState::Suggested,
                    },
                    "test",
                ),
            ],
            focused_suggestion_id: Some(rf_ui::CanvasSuggestionId::new("sug-a")),
            pending_edit: None,
        };

        let registry = StudioGuiCommandRegistry::from_surfaces(
            &StudioAppHostUiCommandModel::default(),
            &canvas,
            Some(3),
        );

        assert_eq!(registry.sections.len(), 1);
        assert_eq!(registry.sections[0].group, StudioGuiCommandGroup::Canvas);
        assert_eq!(registry.sections[0].title, "Canvas");
        assert_eq!(
            registry.sections[0]
                .commands
                .iter()
                .map(|entry| entry.command_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                canvas_command_id(StudioGuiCanvasActionId::AcceptFocused),
                canvas_command_id(StudioGuiCanvasActionId::RejectFocused),
                canvas_command_id(StudioGuiCanvasActionId::FocusNext),
                canvas_command_id(StudioGuiCanvasActionId::FocusPrevious),
                canvas_command_id(StudioGuiCanvasActionId::CancelPendingEdit),
            ]
        );
        assert_eq!(
            registry
                .command(canvas_command_id(StudioGuiCanvasActionId::AcceptFocused))
                .and_then(|entry| entry.target_window_id),
            Some(3)
        );
    }

    #[test]
    fn gui_command_registry_includes_cancel_command_for_pending_canvas_edit() {
        let canvas = crate::StudioGuiCanvasState {
            pending_edit: Some(rf_ui::CanvasEditIntent::PlaceUnit {
                unit_kind: "Flash Drum".to_string(),
            }),
            ..crate::StudioGuiCanvasState::default()
        };

        let registry = StudioGuiCommandRegistry::from_surfaces(
            &StudioAppHostUiCommandModel::default(),
            &canvas,
            Some(7),
        );

        let cancel = registry
            .command(canvas_command_id(
                StudioGuiCanvasActionId::CancelPendingEdit,
            ))
            .expect("expected cancel pending edit command");
        assert!(cancel.enabled);
        assert_eq!(cancel.target_window_id, Some(7));
    }

    #[test]
    fn gui_command_registry_builds_nested_menu_tree_from_menu_paths() {
        let model = StudioAppHostUiCommandModel {
            actions: vec![
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.run_manual",
                    group: StudioAppHostUiCommandGroup::RunPanel,
                    sort_order: 100,
                    label: "Run workspace",
                    enabled: true,
                    detail: "Run",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.recover_failure",
                    group: StudioAppHostUiCommandGroup::Recovery,
                    sort_order: 200,
                    label: "Recover run panel failure",
                    enabled: false,
                    detail: "Recover",
                    target_window_id: Some(2),
                },
            ],
        };

        let registry = StudioGuiCommandRegistry::from_model(&model);
        let menu_tree = registry.menu_tree();

        assert_eq!(menu_tree.len(), 1);
        assert_eq!(menu_tree[0].label, "Run");
        assert!(menu_tree[0].command.is_none());
        assert_eq!(
            menu_tree[0]
                .children
                .iter()
                .map(|node| node.label.as_str())
                .collect::<Vec<_>>(),
            vec!["Run Workspace", "Recovery"]
        );
        assert_eq!(
            menu_tree[0].children[0]
                .command
                .as_ref()
                .map(|entry| entry.command_id.as_str()),
            Some("run_panel.run_manual")
        );
        assert_eq!(menu_tree[0].children[1].command, None);
        assert_eq!(
            menu_tree[0].children[1]
                .children
                .iter()
                .map(command_id_from_leaf)
                .collect::<Vec<_>>(),
            vec!["run_panel.recover_failure"]
        );
    }

    #[test]
    fn gui_command_registry_filters_commands_for_palette_by_label_menu_path_and_search_terms() {
        let model = StudioAppHostUiCommandModel {
            actions: vec![
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.run_manual",
                    group: StudioAppHostUiCommandGroup::RunPanel,
                    sort_order: 100,
                    label: "Run workspace",
                    enabled: true,
                    detail: "Run",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.set_active",
                    group: StudioAppHostUiCommandGroup::RunPanel,
                    sort_order: 130,
                    label: "Activate workspace",
                    enabled: true,
                    detail: "Activate",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.recover_failure",
                    group: StudioAppHostUiCommandGroup::Recovery,
                    sort_order: 200,
                    label: "Recover run panel failure",
                    enabled: false,
                    detail: "Recover",
                    target_window_id: Some(2),
                },
            ],
        };

        let registry = StudioGuiCommandRegistry::from_model(&model);

        assert_eq!(
            filtered_command_ids(&registry, "activate"),
            vec!["run_panel.set_active"]
        );
        assert_eq!(
            filtered_command_ids(&registry, "run recovery"),
            vec!["run_panel.recover_failure"]
        );
        assert_eq!(
            filtered_command_ids(&registry, "diagnostic"),
            vec!["run_panel.recover_failure"]
        );
    }

    #[test]
    fn gui_command_registry_filtered_commands_preserve_section_order_when_query_is_empty() {
        let model = StudioAppHostUiCommandModel {
            actions: vec![
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.set_active",
                    group: StudioAppHostUiCommandGroup::RunPanel,
                    sort_order: 130,
                    label: "Activate workspace",
                    enabled: true,
                    detail: "Activate",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.run_manual",
                    group: StudioAppHostUiCommandGroup::RunPanel,
                    sort_order: 100,
                    label: "Run workspace",
                    enabled: true,
                    detail: "Run",
                    target_window_id: Some(2),
                },
                StudioAppHostUiActionModel {
                    action: None,
                    command_id: "run_panel.recover_failure",
                    group: StudioAppHostUiCommandGroup::Recovery,
                    sort_order: 200,
                    label: "Recover run panel failure",
                    enabled: false,
                    detail: "Recover",
                    target_window_id: Some(2),
                },
            ],
        };

        let registry = StudioGuiCommandRegistry::from_model(&model);

        assert_eq!(
            filtered_command_ids(&registry, ""),
            vec![
                "run_panel.run_manual",
                "run_panel.set_active",
                "run_panel.recover_failure",
            ]
        );
    }

    #[test]
    fn gui_command_entry_presentation_builds_shared_surface_labels_and_hover_text() {
        let entry = StudioGuiCommandEntry {
            command_id: "run_panel.recover_failure".to_string(),
            label: "Recover run panel failure".to_string(),
            detail: "Apply the current run panel recovery action in the target window".to_string(),
            enabled: false,
            sort_order: 200,
            target_window_id: Some(2),
            menu_path: vec![
                "Run".to_string(),
                "Recovery".to_string(),
                "Recover Run Panel Failure".to_string(),
            ],
            search_terms: vec!["recover".to_string()],
            shortcut: Some(StudioGuiShortcut {
                modifiers: Vec::new(),
                key: StudioGuiShortcutKey::F8,
            }),
        };

        let presentation = entry.presentation();

        assert_eq!(presentation.label, "Recover run panel failure");
        assert_eq!(
            presentation.label_with_shortcut,
            "Recover run panel failure (F8)"
        );
        assert_eq!(
            presentation.palette_label,
            "Recover run panel failure (F8) [disabled]"
        );
        assert_eq!(presentation.shortcut_label.as_deref(), Some("F8"));
        assert_eq!(
            presentation.menu_path_text,
            "Run > Recovery > Recover Run Panel Failure"
        );
        assert_eq!(
            presentation.hover_text,
            "Apply the current run panel recovery action in the target window\nMenu: Run > Recovery > Recover Run Panel Failure"
        );
    }

    fn command_id_from_leaf(node: &StudioGuiCommandMenuNode) -> &str {
        node.command
            .as_ref()
            .map(|entry| entry.command_id.as_str())
            .expect("expected leaf command")
    }

    #[test]
    fn gui_command_registry_menu_tree_surfaces_menu_facing_command_model() {
        let model = StudioAppHostUiCommandModel {
            actions: vec![StudioAppHostUiActionModel {
                action: None,
                command_id: "run_panel.run_manual",
                group: StudioAppHostUiCommandGroup::RunPanel,
                sort_order: 100,
                label: "Run workspace",
                enabled: true,
                detail: "Dispatch the current manual run action in the target window",
                target_window_id: Some(2),
            }],
        };

        let menu_tree = StudioGuiCommandRegistry::from_model(&model).menu_tree();
        let leaf = menu_tree[0].children[0]
            .command
            .as_ref()
            .expect("expected menu command leaf");

        assert_eq!(
            leaf,
            &StudioGuiCommandMenuCommandModel {
                command_id: "run_panel.run_manual".to_string(),
                enabled: true,
                label: "Run workspace (F5)".to_string(),
                hover_text:
                    "Dispatch the current manual run action in the target window\nMenu: Run > Run Workspace"
                        .to_string(),
                target_window_id: Some(2),
            }
        );
    }

    fn filtered_command_ids<'a>(
        registry: &'a StudioGuiCommandRegistry,
        query: &str,
    ) -> Vec<&'a str> {
        registry
            .filtered_commands(query)
            .into_iter()
            .map(|entry| entry.command_id.as_str())
            .collect()
    }
}
