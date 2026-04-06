use rf_types::RfResult;

use crate::{
    StudioAppHostState, StudioAppHostUiCommandModel, StudioGuiCanvasState,
    StudioGuiCanvasInteractionAction,
    StudioGuiCommandRegistry, StudioGuiFocusContext, StudioGuiHost,
    StudioGuiHostCanvasInteractionResult, StudioGuiHostCommand, StudioGuiHostCommandOutcome,
    StudioGuiHostLifecycleEvent, StudioGuiHostWindowLayoutUpdateResult, StudioGuiShortcut,
    StudioGuiShortcutIgnoreReason, StudioGuiShortcutRoute, StudioGuiSnapshot,
    StudioGuiWindowLayoutMutation, StudioGuiWindowModel, StudioRuntimeConfig,
    StudioRuntimeTrigger, StudioWindowHostId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioGuiEvent {
    OpenWindowRequested,
    CloseWindowRequested {
        window_id: StudioWindowHostId,
    },
    WindowTriggerRequested {
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    },
    WindowForegrounded {
        window_id: StudioWindowHostId,
    },
    UiCommandRequested {
        command_id: String,
    },
    CanvasSuggestionAcceptRequested,
    CanvasSuggestionRejectRequested,
    CanvasSuggestionFocusNextRequested,
    CanvasSuggestionFocusPreviousRequested,
    WindowLayoutMutationRequested {
        window_id: Option<StudioWindowHostId>,
        mutation: StudioGuiWindowLayoutMutation,
    },
    ShortcutPressed {
        shortcut: StudioGuiShortcut,
        focus_context: StudioGuiFocusContext,
    },
    LoginCompleted,
    NetworkRestored,
    EntitlementTimerElapsed,
    RunPanelRecoveryRequested,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiDriverDispatch {
    pub event: StudioGuiEvent,
    pub outcome: StudioGuiDriverOutcome,
    pub snapshot: StudioGuiSnapshot,
    pub window: StudioGuiWindowModel,
    pub state: StudioAppHostState,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub command_registry: StudioGuiCommandRegistry,
    pub canvas: StudioGuiCanvasState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StudioGuiDriverOutcome {
    HostCommand(StudioGuiHostCommandOutcome),
    CanvasInteraction(StudioGuiHostCanvasInteractionResult),
    WindowLayoutUpdated(StudioGuiHostWindowLayoutUpdateResult),
    IgnoredShortcut {
        shortcut: StudioGuiShortcut,
        reason: StudioGuiShortcutIgnoreReason,
    },
}

pub struct StudioGuiDriver {
    host: StudioGuiHost,
}

impl StudioGuiDriver {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        Ok(Self {
            host: StudioGuiHost::new(config)?,
        })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.host.state()
    }

    pub fn ui_commands(&self) -> StudioAppHostUiCommandModel {
        self.host.ui_commands()
    }

    pub fn command_registry(&self) -> StudioGuiCommandRegistry {
        self.host.command_registry()
    }

    pub fn canvas_state(&self) -> StudioGuiCanvasState {
        self.host.canvas_state()
    }

    pub fn host(&self) -> &StudioGuiHost {
        &self.host
    }

    pub fn snapshot(&self) -> StudioGuiSnapshot {
        self.host.snapshot()
    }

    pub fn window_model(&self) -> StudioGuiWindowModel {
        self.snapshot().window_model()
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        self.host.window_model_for_window(window_id)
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<rf_ui::CanvasSuggestion>) {
        self.host.replace_canvas_suggestions(suggestions);
    }

    pub fn dispatch_event(&mut self, event: StudioGuiEvent) -> RfResult<StudioGuiDriverDispatch> {
        let command_registry = self.host.command_registry();
        let outcome = match route_driver_event(&event, &command_registry) {
            DriverRoute::HostCommand(command) => {
                StudioGuiDriverOutcome::HostCommand(self.host.execute_command(command)?)
            }
            DriverRoute::CanvasInteraction(action) => StudioGuiDriverOutcome::CanvasInteraction(
                match action {
                    StudioGuiCanvasInteractionAction::AcceptFocusedByTab => {
                        self.host.accept_focused_canvas_suggestion_by_tab()?
                    }
                    StudioGuiCanvasInteractionAction::RejectFocused => {
                        self.host.reject_focused_canvas_suggestion()?
                    }
                    StudioGuiCanvasInteractionAction::FocusNext => {
                        self.host.focus_next_canvas_suggestion()?
                    }
                    StudioGuiCanvasInteractionAction::FocusPrevious => {
                        self.host.focus_previous_canvas_suggestion()?
                    }
                },
            ),
            DriverRoute::IgnoredShortcut { shortcut, reason } => {
                StudioGuiDriverOutcome::IgnoredShortcut { shortcut, reason }
            }
            DriverRoute::WindowLayoutMutation { window_id, mutation } => {
                StudioGuiDriverOutcome::WindowLayoutUpdated(
                    self.host.update_window_layout(window_id, mutation)?,
                )
            }
        };
        let snapshot = self.host.snapshot();
        let window = self.host.window_model_for_window(layout_scope_window_id(&outcome));
        Ok(StudioGuiDriverDispatch {
            event,
            outcome,
            snapshot,
            window,
            state: self.host.state().clone(),
            ui_commands: self.host.ui_commands(),
            command_registry: self.host.command_registry(),
            canvas: self.host.canvas_state(),
        })
    }
}

fn layout_scope_window_id(outcome: &StudioGuiDriverOutcome) -> Option<StudioWindowHostId> {
    match outcome {
        StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
            Some(opened.registration.window_id)
        }
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::WindowDispatched(dispatch),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::LifecycleDispatched(lifecycle),
        ) => lifecycle
            .dispatch
            .as_ref()
            .map(|dispatch| dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::UiCommandDispatched(
                crate::StudioGuiHostUiCommandDispatchResult::Executed(dispatch),
            ),
        ) => Some(dispatch.target_window_id),
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::UiCommandDispatched(
                crate::StudioGuiHostUiCommandDispatchResult::IgnoredDisabled {
                    target_window_id,
                    ..
                },
            ),
        ) => *target_window_id,
        StudioGuiDriverOutcome::HostCommand(
            StudioGuiHostCommandOutcome::UiCommandDispatched(
                crate::StudioGuiHostUiCommandDispatchResult::IgnoredMissing { .. },
            ),
        )
        | StudioGuiDriverOutcome::WindowLayoutUpdated(
            crate::StudioGuiHostWindowLayoutUpdateResult { target_window_id: None, .. },
        )
        | StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowClosed(_))
        | StudioGuiDriverOutcome::CanvasInteraction(_)
        | StudioGuiDriverOutcome::IgnoredShortcut { .. } => None,
        StudioGuiDriverOutcome::WindowLayoutUpdated(
            crate::StudioGuiHostWindowLayoutUpdateResult {
                target_window_id: Some(window_id),
                ..
            },
        ) => Some(*window_id),
    }
}

enum DriverRoute {
    HostCommand(StudioGuiHostCommand),
    CanvasInteraction(StudioGuiCanvasInteractionAction),
    WindowLayoutMutation {
        window_id: Option<StudioWindowHostId>,
        mutation: StudioGuiWindowLayoutMutation,
    },
    IgnoredShortcut {
        shortcut: StudioGuiShortcut,
        reason: StudioGuiShortcutIgnoreReason,
    },
}

fn route_driver_event(event: &StudioGuiEvent, registry: &StudioGuiCommandRegistry) -> DriverRoute {
    match event {
        StudioGuiEvent::OpenWindowRequested => {
            DriverRoute::HostCommand(StudioGuiHostCommand::OpenWindow)
        }
        StudioGuiEvent::CloseWindowRequested { window_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::CloseWindow {
                window_id: *window_id,
            })
        }
        StudioGuiEvent::WindowTriggerRequested { window_id, trigger } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchWindowTrigger {
                window_id: *window_id,
                trigger: trigger.clone(),
            })
        }
        StudioGuiEvent::WindowForegrounded { window_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::WindowForegrounded {
                    window_id: *window_id,
                },
            })
        }
        StudioGuiEvent::UiCommandRequested { command_id } => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchUiCommand {
                command_id: command_id.clone(),
            })
        }
        StudioGuiEvent::CanvasSuggestionAcceptRequested => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::AcceptFocusedByTab)
        }
        StudioGuiEvent::CanvasSuggestionRejectRequested => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::RejectFocused)
        }
        StudioGuiEvent::CanvasSuggestionFocusNextRequested => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::FocusNext)
        }
        StudioGuiEvent::CanvasSuggestionFocusPreviousRequested => {
            DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::FocusPrevious)
        }
        StudioGuiEvent::WindowLayoutMutationRequested { window_id, mutation } => {
            DriverRoute::WindowLayoutMutation {
                window_id: *window_id,
                mutation: mutation.clone(),
            }
        }
        StudioGuiEvent::ShortcutPressed {
            shortcut,
            focus_context,
        } => match crate::route_shortcut(registry, shortcut, *focus_context) {
            StudioGuiShortcutRoute::DispatchCommandId { command_id } => {
                DriverRoute::HostCommand(StudioGuiHostCommand::DispatchUiCommand { command_id })
            }
            StudioGuiShortcutRoute::RequestCanvasSuggestionAccept => DriverRoute::CanvasInteraction(
                StudioGuiCanvasInteractionAction::AcceptFocusedByTab,
            ),
            StudioGuiShortcutRoute::RequestCanvasSuggestionReject => DriverRoute::CanvasInteraction(
                StudioGuiCanvasInteractionAction::RejectFocused,
            ),
            StudioGuiShortcutRoute::RequestCanvasSuggestionFocusNext => {
                DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::FocusNext)
            }
            StudioGuiShortcutRoute::RequestCanvasSuggestionFocusPrevious => {
                DriverRoute::CanvasInteraction(StudioGuiCanvasInteractionAction::FocusPrevious)
            }
            StudioGuiShortcutRoute::Ignored { reason } => DriverRoute::IgnoredShortcut {
                shortcut: shortcut.clone(),
                reason,
            },
        },
        StudioGuiEvent::LoginCompleted => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::LoginCompleted,
            })
        }
        StudioGuiEvent::NetworkRestored => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::NetworkRestored,
            })
        }
        StudioGuiEvent::EntitlementTimerElapsed => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::TimerElapsed,
            })
        }
        StudioGuiEvent::RunPanelRecoveryRequested => {
            DriverRoute::HostCommand(StudioGuiHostCommand::DispatchLifecycleEvent {
                event: StudioGuiHostLifecycleEvent::RunPanelRecoveryRequested,
            })
        }
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
        StudioGuiCanvasInteractionAction, StudioGuiDriver, StudioGuiDriverOutcome,
        StudioGuiEvent, StudioGuiFocusContext, StudioGuiHostCanvasInteractionResult,
        StudioGuiHostCommandOutcome,
        StudioGuiHostUiCommandDispatchResult, StudioGuiShortcut, StudioGuiShortcutIgnoreReason,
        StudioGuiShortcutKey, StudioGuiShortcutModifier, StudioGuiWindowAreaId,
        StudioGuiWindowDockRegion, StudioGuiWindowLayoutMutation, StudioRuntimeConfig,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
    };
    use rf_ui::{
        GhostElement, GhostElementKind, StreamVisualKind, StreamVisualState, SuggestionSource,
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
            "radishflow-studio-local-rules-{timestamp}.rfproj.json"
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

    fn sample_canvas_suggestion(id: &str, confidence: f32) -> rf_ui::CanvasSuggestion {
        rf_ui::CanvasSuggestion::new(
            rf_ui::CanvasSuggestionId::new(id),
            SuggestionSource::LocalRules,
            confidence,
            GhostElement {
                kind: GhostElementKind::Connection,
                target_unit_id: rf_types::UnitId::new("flash-1"),
                visual_kind: StreamVisualKind::Material,
                visual_state: StreamVisualState::Suggested,
            },
            "accept by tab",
        )
    }

    #[test]
    fn gui_driver_opens_window_and_refreshes_command_state() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => {
                assert_eq!(opened.registration.window_id, 1);
            }
            other => panic!("expected window opened outcome, got {other:?}"),
        }
        assert_eq!(dispatch.state.windows.len(), 1);
        assert_eq!(dispatch.snapshot.app_host_state.windows.len(), 1);
        assert_eq!(dispatch.window.header.registered_window_count, 1);
        assert_eq!(
            dispatch.window.layout().default_focus_area,
            crate::StudioGuiWindowAreaId::Commands
        );
        assert_eq!(
            dispatch
                .command_registry
                .sections
                .first()
                .and_then(|section| section.commands.first())
                .and_then(|command| command.target_window_id),
            Some(1)
        );
        assert!(dispatch.canvas.suggestions.is_empty());
        assert_eq!(dispatch.snapshot.canvas.view().suggestion_count, 0);
    }

    #[test]
    fn gui_driver_routes_ui_command_request_through_single_event_entry() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let open = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match open.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "run_panel.set_active".to_string(),
            })
            .expect("expected ui command dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(executed),
                ),
            ) => {
                assert_eq!(executed.target_window_id, window_id);
            }
            other => panic!("expected executed ui command outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_surfaces_local_rules_canvas_state_from_project() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let initial_canvas = driver.canvas_state();
        assert_eq!(initial_canvas.suggestions.len(), 3);
        assert_eq!(
            initial_canvas
                .focused_suggestion_id
                .as_ref()
                .map(|id| id.as_str()),
            Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
        );

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        assert_eq!(dispatch.canvas.suggestions.len(), 3);
        assert_eq!(dispatch.snapshot.canvas.view().suggestion_count, 3);
        assert_eq!(
            dispatch
                .snapshot
                .runtime
                .run_panel
                .view()
                .primary_action
                .label,
            "Resume"
        );
        assert_eq!(
            dispatch
                .canvas
                .suggestions
                .iter()
                .map(|suggestion| suggestion.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "local.flash_drum.connect_inlet.flash-1.stream-heated",
                "local.flash_drum.create_outlet.flash-1.liquid",
                "local.flash_drum.create_outlet.flash-1.vapor",
            ]
        );

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_dispatch_snapshot_aggregates_gui_facing_state() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        assert!(
            dispatch.snapshot.command_registry.sections.len() >= 1,
            "expected at least one command section in gui snapshot"
        );
        assert_eq!(
            dispatch.snapshot.canvas.primary_action().label,
            "Accept suggestion"
        );
        assert_eq!(dispatch.window.canvas.suggestion_count, 3);
        assert_eq!(
            dispatch.window.layout().default_focus_area,
            crate::StudioGuiWindowAreaId::Canvas
        );
        assert_eq!(
            dispatch
                .snapshot
                .command_registry
                .sections
                .first()
                .map(|section| section.title),
            Some("Run Panel")
        );
        assert_eq!(
            dispatch
                .snapshot
                .runtime
                .control_state
                .run_status,
            rf_ui::RunStatus::Idle
        );
        assert_eq!(dispatch.window.runtime.run_panel.view().primary_action.label, "Resume");
        assert!(dispatch.snapshot.runtime.entitlement_host.is_some());
        assert!(!dispatch.snapshot.runtime.run_panel.view().primary_action.label.is_empty());

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_routes_network_restored_without_open_windows() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::NetworkRestored)
            .expect("expected lifecycle dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::LifecycleDispatched(lifecycle),
            ) => {
                assert!(lifecycle.dispatch.is_none());
            }
            other => panic!("expected lifecycle outcome, got {other:?}"),
        }
        assert!(dispatch.state.windows.is_empty());
    }

    #[test]
    fn gui_driver_routes_shortcut_into_ui_command_dispatch() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let _ = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Shift],
                    key: StudioGuiShortcutKey::F6,
                },
                focus_context: StudioGuiFocusContext::Global,
            })
            .expect("expected shortcut dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(
                StudioGuiHostCommandOutcome::UiCommandDispatched(
                    StudioGuiHostUiCommandDispatchResult::Executed(executed),
                ),
            ) => match &executed.effects.runtime_report.dispatch {
                crate::StudioRuntimeDispatch::AppCommand(outcome) => match &outcome.dispatch {
                    crate::StudioAppResultDispatch::WorkspaceMode(mode) => {
                        assert_eq!(mode.simulation_mode, rf_ui::SimulationMode::Active);
                    }
                    other => panic!("expected workspace mode dispatch, got {other:?}"),
                },
                other => panic!("expected app command dispatch, got {other:?}"),
            },
            other => panic!("expected executed shortcut outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_returns_canvas_tab_request_when_canvas_suggestion_is_focused() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
            })
            .expect("expected shortcut dispatch");

        assert_eq!(
            dispatch.outcome,
            StudioGuiDriverOutcome::CanvasInteraction(StudioGuiHostCanvasInteractionResult {
                action: StudioGuiCanvasInteractionAction::AcceptFocusedByTab,
                accepted: None,
                rejected: None,
                focused: None,
                applied_target: None,
                latest_log_entry: None,
                ui_commands: driver.ui_commands(),
                canvas: driver.canvas_state(),
            })
        );
    }

    #[test]
    fn gui_driver_reports_ignored_shortcut_when_text_input_owns_tab() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                focus_context: StudioGuiFocusContext::TextInput,
            })
            .expect("expected shortcut dispatch");

        assert_eq!(
            dispatch.outcome,
            StudioGuiDriverOutcome::IgnoredShortcut {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                reason: StudioGuiShortcutIgnoreReason::TextInputOwnsShortcut,
            }
        );
    }

    #[test]
    fn gui_driver_accepts_focused_canvas_suggestion_by_tab() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        driver.replace_canvas_suggestions(vec![sample_canvas_suggestion("sug-high", 0.95)]);

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Tab,
                },
                focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
            })
            .expect("expected shortcut dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::CanvasInteraction(result) => {
                assert_eq!(result.action, StudioGuiCanvasInteractionAction::AcceptFocusedByTab);
                assert_eq!(
                    result
                        .accepted
                        .as_ref()
                        .map(|suggestion| suggestion.id.as_str()),
                    Some("sug-high")
                );
                assert_eq!(
                    result.applied_target,
                    Some(rf_ui::InspectorTarget::Unit(rf_types::UnitId::new(
                        "flash-1"
                    )))
                );
                assert_eq!(
                    result
                        .latest_log_entry
                        .as_ref()
                        .map(|entry| entry.message.as_str()),
                    Some("Accepted canvas suggestion `sug-high` from local rules for unit flash-1")
                );
            }
            other => panic!("expected canvas interaction outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_focuses_next_canvas_suggestion_from_explicit_event() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::CanvasSuggestionFocusNextRequested)
            .expect("expected focus-next dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::CanvasInteraction(result) => {
                assert_eq!(result.action, StudioGuiCanvasInteractionAction::FocusNext);
                assert_eq!(
                    result.focused.as_ref().map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
                assert_eq!(
                    dispatch
                        .canvas
                        .focused_suggestion_id
                        .as_ref()
                        .map(|id| id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
            }
            other => panic!("expected canvas interaction outcome, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_rejects_focused_canvas_suggestion_from_shortcut() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: Vec::new(),
                    key: StudioGuiShortcutKey::Escape,
                },
                focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
            })
            .expect("expected reject dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::CanvasInteraction(result) => {
                assert_eq!(result.action, StudioGuiCanvasInteractionAction::RejectFocused);
                assert_eq!(
                    result
                        .rejected
                        .as_ref()
                        .map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
                );
                assert_eq!(
                    result.focused.as_ref().map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
            }
            other => panic!("expected canvas interaction outcome, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_routes_ctrl_tab_to_canvas_focus_next() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut: StudioGuiShortcut {
                    modifiers: vec![StudioGuiShortcutModifier::Ctrl],
                    key: StudioGuiShortcutKey::Tab,
                },
                focus_context: StudioGuiFocusContext::CanvasSuggestionFocused,
            })
            .expect("expected shortcut dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::CanvasInteraction(result) => {
                assert_eq!(result.action, StudioGuiCanvasInteractionAction::FocusNext);
                assert_eq!(
                    result.focused.as_ref().map(|suggestion| suggestion.id.as_str()),
                    Some("local.flash_drum.create_outlet.flash-1.liquid")
                );
            }
            other => panic!("expected canvas interaction outcome, got {other:?}"),
        }

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn gui_driver_updates_window_layout_and_preserves_per_window_overrides() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let first = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected first open dispatch");
        let first_window_id = match first.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected first window opened outcome, got {other:?}"),
        };
        let second = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected second open dispatch");
        let second_window_id = match second.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
            other => panic!("expected second window opened outcome, got {other:?}"),
        };

        let hidden_runtime = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::SetPanelVisibility {
                    area_id: StudioGuiWindowAreaId::Runtime,
                    visible: false,
                },
            })
            .expect("expected layout visibility update");
        match hidden_runtime.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result
                        .layout_state
                        .panel(StudioGuiWindowAreaId::Runtime)
                        .map(|panel| panel.visible),
                    Some(false)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let collapsed_commands = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                    area_id: StudioGuiWindowAreaId::Commands,
                    collapsed: true,
                },
            })
            .expect("expected layout collapsed update");
        match collapsed_commands.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result
                        .layout_state
                        .panel(StudioGuiWindowAreaId::Commands)
                        .map(|panel| panel.collapsed),
                    Some(true)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let weighted = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(second_window_id),
                mutation: StudioGuiWindowLayoutMutation::SetRegionWeight {
                    dock_region: StudioGuiWindowDockRegion::RightSidebar,
                    weight: 31,
                },
            })
            .expect("expected layout weight update");
        match weighted.outcome {
            StudioGuiDriverOutcome::WindowLayoutUpdated(result) => {
                assert_eq!(result.target_window_id, Some(second_window_id));
                assert_eq!(
                    result
                        .layout_state
                        .region_weight(StudioGuiWindowDockRegion::RightSidebar)
                        .map(|region| region.weight),
                    Some(31)
                );
            }
            other => panic!("expected window layout update outcome, got {other:?}"),
        }

        let first_window = driver.window_model_for_window(Some(first_window_id));
        let second_window = driver.window_model_for_window(Some(second_window_id));

        assert_eq!(
            first_window
                .layout_state
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| panel.visible),
            Some(true)
        );
        assert_eq!(
            second_window
                .layout_state
                .panel(StudioGuiWindowAreaId::Runtime)
                .map(|panel| panel.visible),
            Some(false)
        );
        assert_eq!(
            second_window
                .layout_state
                .panel(StudioGuiWindowAreaId::Commands)
                .map(|panel| panel.collapsed),
            Some(true)
        );
        assert_eq!(
            first_window
                .layout_state
                .region_weight(StudioGuiWindowDockRegion::RightSidebar)
                .map(|region| region.weight),
            Some(24)
        );
        assert_eq!(
            second_window
                .layout_state
                .region_weight(StudioGuiWindowDockRegion::RightSidebar)
                .map(|region| region.weight),
            Some(31)
        );
    }

    #[test]
    fn gui_driver_rejects_layout_update_for_unknown_window() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let error = driver
            .dispatch_event(StudioGuiEvent::WindowLayoutMutationRequested {
                window_id: Some(99),
                mutation: StudioGuiWindowLayoutMutation::SetPanelCollapsed {
                    area_id: StudioGuiWindowAreaId::Commands,
                    collapsed: true,
                },
            })
            .expect_err("expected invalid layout target");

        assert_eq!(error.code().as_str(), "invalid_input");
    }
}
