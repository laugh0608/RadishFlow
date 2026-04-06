use crate::{
    StudioAppHostUiCommandGroup, StudioAppHostUiCommandModel, StudioWindowHostId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StudioGuiShortcutModifier {
    Ctrl,
    Shift,
    Alt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StudioGuiShortcutKey {
    F5,
    F6,
    F8,
    Tab,
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
pub struct StudioGuiCommandSection {
    pub group: StudioAppHostUiCommandGroup,
    pub title: &'static str,
    pub commands: Vec<StudioGuiCommandEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StudioGuiCommandRegistry {
    pub sections: Vec<StudioGuiCommandSection>,
}

impl StudioGuiCommandRegistry {
    pub fn from_model(model: &StudioAppHostUiCommandModel) -> Self {
        let mut run_panel = Vec::new();
        let mut recovery = Vec::new();

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
                StudioAppHostUiCommandGroup::RunPanel => run_panel.push(entry),
                StudioAppHostUiCommandGroup::Recovery => recovery.push(entry),
            }
        }

        let mut sections = Vec::new();
        if !run_panel.is_empty() {
            run_panel.sort_by_key(|entry| entry.sort_order);
            sections.push(StudioGuiCommandSection {
                group: StudioAppHostUiCommandGroup::RunPanel,
                title: "Run Panel",
                commands: run_panel,
            });
        }
        if !recovery.is_empty() {
            recovery.sort_by_key(|entry| entry.sort_order);
            sections.push(StudioGuiCommandSection {
                group: StudioAppHostUiCommandGroup::Recovery,
                title: "Recovery",
                commands: recovery,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StudioGuiCommandDefaults {
    menu_path: &'static [&'static str],
    search_terms: &'static [&'static str],
    shortcut: Option<StudioGuiShortcut>,
}

fn command_defaults(command_id: &str) -> StudioGuiCommandDefaults {
    match command_id {
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
        _ => StudioGuiCommandDefaults {
            menu_path: &["Commands"],
            search_terms: &[],
            shortcut: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        StudioAppHostUiActionModel, StudioAppHostUiCommandGroup, StudioAppHostUiCommandModel,
        StudioGuiCommandRegistry, StudioGuiShortcut, StudioGuiShortcutKey,
        StudioGuiShortcutModifier,
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
            ],
        };

        let registry = StudioGuiCommandRegistry::from_model(&model);

        assert_eq!(registry.sections.len(), 2);
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
}
