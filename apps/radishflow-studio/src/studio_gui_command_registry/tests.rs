use crate::{
    StudioAppHostUiActionModel, StudioAppHostUiCommandGroup, StudioAppHostUiCommandModel,
    StudioGuiCanvasActionId, StudioGuiCanvasPlaceUnitKind, StudioGuiCommandEntry,
    StudioGuiCommandGroup, StudioGuiCommandMenuCommandModel, StudioGuiCommandMenuNode,
    StudioGuiCommandRegistry, StudioGuiShortcut, StudioGuiShortcutKey, StudioGuiShortcutModifier,
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
        ..crate::StudioGuiCanvasState::default()
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
            canvas_command_id(StudioGuiCanvasActionId::BeginPlaceUnit(
                StudioGuiCanvasPlaceUnitKind::Feed,
            )),
            canvas_command_id(StudioGuiCanvasActionId::BeginPlaceUnit(
                StudioGuiCanvasPlaceUnitKind::Mixer,
            )),
            canvas_command_id(StudioGuiCanvasActionId::BeginPlaceUnit(
                StudioGuiCanvasPlaceUnitKind::Heater,
            )),
            canvas_command_id(StudioGuiCanvasActionId::BeginPlaceUnit(
                StudioGuiCanvasPlaceUnitKind::Cooler,
            )),
            canvas_command_id(StudioGuiCanvasActionId::BeginPlaceUnit(
                StudioGuiCanvasPlaceUnitKind::Valve,
            )),
            canvas_command_id(StudioGuiCanvasActionId::BeginPlaceUnit(
                StudioGuiCanvasPlaceUnitKind::FlashDrum,
            )),
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
fn gui_command_registry_exposes_place_unit_palette_for_target_window() {
    let canvas = crate::StudioGuiCanvasState::default();
    let registry = StudioGuiCommandRegistry::from_surfaces(
        &StudioAppHostUiCommandModel::default(),
        &canvas,
        Some(5),
    );

    assert_eq!(
        registry.sections[0]
            .commands
            .iter()
            .map(|entry| (
                entry.command_id.as_str(),
                entry.label.as_str(),
                entry.menu_path.clone(),
                entry.enabled,
                entry.target_window_id
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "canvas.begin_place_unit.feed",
                "Place Feed",
                vec![
                    "Canvas".to_string(),
                    "Place Unit".to_string(),
                    "Feed".to_string()
                ],
                true,
                Some(5)
            ),
            (
                "canvas.begin_place_unit.mixer",
                "Place Mixer",
                vec![
                    "Canvas".to_string(),
                    "Place Unit".to_string(),
                    "Mixer".to_string()
                ],
                true,
                Some(5)
            ),
            (
                "canvas.begin_place_unit.heater",
                "Place Heater",
                vec![
                    "Canvas".to_string(),
                    "Place Unit".to_string(),
                    "Heater".to_string()
                ],
                true,
                Some(5)
            ),
            (
                "canvas.begin_place_unit.cooler",
                "Place Cooler",
                vec![
                    "Canvas".to_string(),
                    "Place Unit".to_string(),
                    "Cooler".to_string()
                ],
                true,
                Some(5)
            ),
            (
                "canvas.begin_place_unit.valve",
                "Place Valve",
                vec![
                    "Canvas".to_string(),
                    "Place Unit".to_string(),
                    "Valve".to_string()
                ],
                true,
                Some(5)
            ),
            (
                "canvas.begin_place_unit.flash_drum",
                "Place Flash Drum",
                vec![
                    "Canvas".to_string(),
                    "Place Unit".to_string(),
                    "Flash Drum".to_string()
                ],
                true,
                Some(5)
            ),
        ]
    );
    assert!(
        registry.command("canvas.accept_focused").is_none(),
        "empty canvas should not expose suggestion commands"
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
    let widget = canvas.widget();

    let pending_edit = widget
        .view()
        .pending_edit
        .as_ref()
        .expect("expected pending edit presentation");
    assert_eq!(pending_edit.intent_label, "place_unit");
    assert_eq!(pending_edit.summary, "place unit kind=Flash Drum");
    assert!(pending_edit.cancel_enabled);
    let begin_action = widget
        .action(StudioGuiCanvasActionId::BeginPlaceUnit(
            StudioGuiCanvasPlaceUnitKind::FlashDrum,
        ))
        .expect("expected begin place action");
    assert_eq!(
        begin_action.command_id,
        canvas_command_id(StudioGuiCanvasActionId::BeginPlaceUnit(
            StudioGuiCanvasPlaceUnitKind::FlashDrum,
        ))
    );
    assert!(!begin_action.enabled);
    let cancel_action = widget
        .action(StudioGuiCanvasActionId::CancelPendingEdit)
        .expect("expected cancel pending edit action");
    assert_eq!(
        cancel_action.command_id,
        canvas_command_id(StudioGuiCanvasActionId::CancelPendingEdit)
    );
    assert!(cancel_action.enabled);

    let registry = StudioGuiCommandRegistry::from_surfaces(
        &StudioAppHostUiCommandModel::default(),
        &canvas,
        Some(7),
    );

    let begin = registry
        .command(canvas_command_id(StudioGuiCanvasActionId::BeginPlaceUnit(
            StudioGuiCanvasPlaceUnitKind::FlashDrum,
        )))
        .expect("expected begin place command");
    assert_eq!(begin.label, "Place Flash Drum");
    assert_eq!(begin.detail, "Start placing a Flash Drum on the canvas");
    assert_eq!(begin.menu_path, vec!["Canvas", "Place Unit", "Flash Drum"]);
    assert!(!begin.enabled);
    assert_eq!(begin.target_window_id, Some(7));

    let cancel = registry
        .command(canvas_command_id(
            StudioGuiCanvasActionId::CancelPendingEdit,
        ))
        .expect("expected cancel pending edit command");
    assert_eq!(cancel.label, "Cancel pending edit");
    assert_eq!(cancel.detail, "Cancel the current canvas edit intent");
    assert_eq!(cancel.menu_path, vec!["Canvas", "Cancel Pending Edit"]);
    assert!(cancel.enabled);
    assert_eq!(cancel.target_window_id, Some(7));
}

#[test]
fn gui_command_registry_includes_canvas_object_navigation_commands() {
    let canvas = crate::StudioGuiCanvasState {
        units: vec![crate::StudioGuiCanvasUnitState {
            unit_id: rf_types::UnitId::new("flash-1"),
            name: "Flash Drum".to_string(),
            kind: "flash_drum".to_string(),
            layout_position: None,
            ports: vec![crate::StudioGuiCanvasUnitPortState {
                name: "inlet".to_string(),
                direction: rf_types::PortDirection::Inlet,
                kind: rf_types::PortKind::Material,
                stream_id: Some(rf_types::StreamId::new("stream-feed")),
            }],
            port_count: 1,
            connected_port_count: 1,
            is_active_inspector_target: false,
        }],
        streams: vec![crate::StudioGuiCanvasStreamState {
            stream_id: rf_types::StreamId::new("stream-feed"),
            name: "Feed".to_string(),
            source: None,
            sink: Some(crate::StudioGuiCanvasStreamEndpointState {
                unit_id: rf_types::UnitId::new("flash-1"),
                port_name: "inlet".to_string(),
            }),
            is_active_inspector_target: false,
        }],
        ..crate::StudioGuiCanvasState::default()
    };

    let registry = StudioGuiCommandRegistry::from_surfaces(
        &StudioAppHostUiCommandModel::default(),
        &canvas,
        Some(9),
    );
    let canvas_view = canvas.widget().view().clone();

    let commands = registry
        .sections
        .iter()
        .find(|section| section.group == StudioGuiCommandGroup::Canvas)
        .expect("expected canvas command section")
        .commands
        .iter()
        .map(|entry| {
            (
                entry.command_id.as_str(),
                entry.label.as_str(),
                entry.detail.as_str(),
                entry.menu_path.clone(),
                entry.search_terms.clone(),
                entry.target_window_id,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(commands.len(), 8);
    let object_commands = commands
        .iter()
        .filter(|command| command.0.starts_with("inspector.focus_"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(object_commands.len(), 2);
    assert_eq!(
        object_commands[0],
        (
            "inspector.focus_unit:flash-1",
            "Locate Unit Flash Drum",
            "Open the Unit Inspector for `flash-1` and request Canvas viewport focus at `unit-slot-0`. flash_drum | ports 1/1",
            vec![
                "Canvas".to_string(),
                "Objects".to_string(),
                "Unit".to_string(),
                "Flash Drum".to_string(),
            ],
            vec![
                "canvas".to_string(),
                "object".to_string(),
                "objects".to_string(),
                "locate".to_string(),
                "focus".to_string(),
                "viewport".to_string(),
                "Unit".to_string(),
                "flash-1".to_string(),
                "Flash Drum".to_string(),
                "flash_drum | ports 1/1".to_string(),
                "unit-slot-0".to_string(),
            ],
            Some(9),
        )
    );
    assert_eq!(
        object_commands[1].0, "inspector.focus_stream:stream-feed",
        "expected stream object navigation command"
    );
    assert!(
        registry
            .filtered_commands("canvas viewport stream-feed")
            .iter()
            .any(|entry| entry.command_id == "inspector.focus_stream:stream-feed")
    );
    for item in &canvas_view.object_list.items {
        let target = item.command_target();
        let command = registry
            .command(&target.command_id)
            .expect("expected object command");
        assert_eq!(command.target_window_id, Some(9));
        assert_eq!(
            command.menu_path,
            vec![
                "Canvas".to_string(),
                "Objects".to_string(),
                target.kind_label.to_string(),
                target.label.clone(),
            ]
        );
        assert!(
            command
                .detail
                .contains(target.viewport_anchor_label.as_deref().unwrap_or("none")),
            "expected command detail to include viewport anchor"
        );
        assert!(
            command
                .search_terms
                .contains(&target.viewport_anchor_label.unwrap_or_default()),
            "expected command search terms to include viewport anchor"
        );
    }
}

#[test]
fn gui_command_registry_includes_selected_unit_layout_nudge_commands() {
    let canvas = crate::StudioGuiCanvasState {
        units: vec![crate::StudioGuiCanvasUnitState {
            unit_id: rf_types::UnitId::new("feed-1"),
            name: "Feed".to_string(),
            kind: "feed".to_string(),
            layout_position: None,
            ports: Vec::new(),
            port_count: 0,
            connected_port_count: 0,
            is_active_inspector_target: true,
        }],
        ..crate::StudioGuiCanvasState::default()
    };

    let registry = StudioGuiCommandRegistry::from_surfaces(
        &StudioAppHostUiCommandModel::default(),
        &canvas,
        Some(11),
    );

    for (command_id, label, menu_tail) in [
        ("canvas.move_selected_unit.left", "Move left", "Move Left"),
        ("canvas.move_selected_unit.up", "Move up", "Move Up"),
        ("canvas.move_selected_unit.down", "Move down", "Move Down"),
        (
            "canvas.move_selected_unit.right",
            "Move right",
            "Move Right",
        ),
    ] {
        let command = registry
            .command(command_id)
            .expect("expected selected unit layout command");
        assert_eq!(command.label, label);
        assert_eq!(
            command.menu_path,
            vec![
                "Canvas".to_string(),
                "Layout".to_string(),
                menu_tail.to_string()
            ]
        );
        assert!(command.enabled);
        assert_eq!(command.target_window_id, Some(11));
        assert!(command.detail.contains("feed-1"));
        assert!(command.search_terms.contains(&"nudge".to_string()));
    }
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

fn filtered_command_ids<'a>(registry: &'a StudioGuiCommandRegistry, query: &str) -> Vec<&'a str> {
    registry
        .filtered_commands(query)
        .into_iter()
        .map(|entry| entry.command_id.as_str())
        .collect()
}
