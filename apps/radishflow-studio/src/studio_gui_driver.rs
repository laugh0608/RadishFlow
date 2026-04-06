use rf_types::RfResult;

use crate::{
    StudioAppHostState, StudioAppHostUiCommandModel, StudioGuiCommandRegistry, StudioGuiHost,
    StudioGuiFocusContext, StudioGuiHostCommand, StudioGuiHostCommandOutcome,
    StudioGuiHostLifecycleEvent, StudioGuiHostCanvasSuggestionResult, StudioGuiShortcut,
    StudioGuiShortcutIgnoreReason, StudioGuiShortcutRoute, StudioRuntimeConfig,
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
    pub state: StudioAppHostState,
    pub ui_commands: StudioAppHostUiCommandModel,
    pub command_registry: StudioGuiCommandRegistry,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StudioGuiDriverOutcome {
    HostCommand(StudioGuiHostCommandOutcome),
    CanvasSuggestionAccepted(StudioGuiHostCanvasSuggestionResult),
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

    pub fn host(&self) -> &StudioGuiHost {
        &self.host
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
            DriverRoute::CanvasSuggestionAcceptRequested => {
                StudioGuiDriverOutcome::CanvasSuggestionAccepted(
                    self.host.accept_focused_canvas_suggestion_by_tab(),
                )
            }
            DriverRoute::IgnoredShortcut { shortcut, reason } => {
                StudioGuiDriverOutcome::IgnoredShortcut { shortcut, reason }
            }
        };
        Ok(StudioGuiDriverDispatch {
            event,
            outcome,
            state: self.host.state().clone(),
            ui_commands: self.host.ui_commands(),
            command_registry: self.host.command_registry(),
        })
    }
}

enum DriverRoute {
    HostCommand(StudioGuiHostCommand),
    CanvasSuggestionAcceptRequested,
    IgnoredShortcut {
        shortcut: StudioGuiShortcut,
        reason: StudioGuiShortcutIgnoreReason,
    },
}

fn route_driver_event(
    event: &StudioGuiEvent,
    registry: &StudioGuiCommandRegistry,
) -> DriverRoute {
    match event {
        StudioGuiEvent::OpenWindowRequested => DriverRoute::HostCommand(StudioGuiHostCommand::OpenWindow),
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
        StudioGuiEvent::ShortcutPressed {
            shortcut,
            focus_context,
        } => match crate::route_shortcut(registry, shortcut, *focus_context) {
            StudioGuiShortcutRoute::DispatchCommandId { command_id } => {
                DriverRoute::HostCommand(StudioGuiHostCommand::DispatchUiCommand { command_id })
            }
            StudioGuiShortcutRoute::RequestCanvasSuggestionAcceptByTab => {
                DriverRoute::CanvasSuggestionAcceptRequested
            }
            StudioGuiShortcutRoute::Ignored { reason } => DriverRoute::IgnoredShortcut {
                shortcut: shortcut.clone(),
                reason,
            },
        }
        StudioGuiEvent::LoginCompleted => DriverRoute::HostCommand(
            StudioGuiHostCommand::DispatchLifecycleEvent {
            event: StudioGuiHostLifecycleEvent::LoginCompleted,
        }),
        StudioGuiEvent::NetworkRestored => DriverRoute::HostCommand(
            StudioGuiHostCommand::DispatchLifecycleEvent {
            event: StudioGuiHostLifecycleEvent::NetworkRestored,
        }),
        StudioGuiEvent::EntitlementTimerElapsed => DriverRoute::HostCommand(
            StudioGuiHostCommand::DispatchLifecycleEvent {
            event: StudioGuiHostLifecycleEvent::TimerElapsed,
        }),
        StudioGuiEvent::RunPanelRecoveryRequested => DriverRoute::HostCommand(
            StudioGuiHostCommand::DispatchLifecycleEvent {
            event: StudioGuiHostLifecycleEvent::RunPanelRecoveryRequested,
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        StudioGuiDriver, StudioGuiDriverOutcome, StudioGuiEvent, StudioGuiFocusContext,
        StudioGuiHostCanvasSuggestionResult, StudioGuiHostCommandOutcome,
        StudioGuiHostUiCommandDispatchResult, StudioGuiShortcut, StudioGuiShortcutIgnoreReason,
        StudioGuiShortcutKey, StudioGuiShortcutModifier, StudioRuntimeConfig,
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

    fn sample_canvas_suggestion(
        id: &str,
        confidence: f32,
    ) -> rf_ui::CanvasSuggestion {
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
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
                assert_eq!(opened.registration.window_id, 1);
            }
            other => panic!("expected window opened outcome, got {other:?}"),
        }
        assert_eq!(dispatch.state.windows.len(), 1);
        assert_eq!(
            dispatch
                .command_registry
                .sections
                .first()
                .and_then(|section| section.commands.first())
                .and_then(|command| command.target_window_id),
            Some(1)
        );
    }

    #[test]
    fn gui_driver_routes_ui_command_request_through_single_event_entry() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let open = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match open.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(opened)) => {
                opened.registration.window_id
            }
            other => panic!("expected window opened outcome, got {other:?}"),
        };

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: "run_panel.set_active".to_string(),
            })
            .expect("expected ui command dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
                StudioGuiHostUiCommandDispatchResult::Executed(executed),
            )) => {
                assert_eq!(executed.target_window_id, window_id);
            }
            other => panic!("expected executed ui command outcome, got {other:?}"),
        }
    }

    #[test]
    fn gui_driver_routes_network_restored_without_open_windows() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::NetworkRestored)
            .expect("expected lifecycle dispatch");

        match dispatch.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::LifecycleDispatched(
                lifecycle,
            )) => {
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
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::UiCommandDispatched(
                StudioGuiHostUiCommandDispatchResult::Executed(executed),
            )) => match &executed.effects.runtime_report.dispatch {
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
            StudioGuiDriverOutcome::CanvasSuggestionAccepted(
                StudioGuiHostCanvasSuggestionResult {
                    accepted: None,
                    ui_commands: driver.ui_commands(),
                }
            )
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
            StudioGuiDriverOutcome::CanvasSuggestionAccepted(result) => {
                assert_eq!(
                    result.accepted.as_ref().map(|suggestion| suggestion.id.as_str()),
                    Some("sug-high")
                );
            }
            other => panic!("expected canvas suggestion accepted outcome, got {other:?}"),
        }
    }
}
