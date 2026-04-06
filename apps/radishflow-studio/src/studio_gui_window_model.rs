use crate::{
    EntitlementSessionHostRuntimeOutput, StudioGuiCanvasWidgetModel, StudioGuiCommandRegistry,
    StudioGuiCommandSection, StudioGuiSnapshot, StudioGuiWindowLayoutState, StudioWindowHostId,
    WorkspaceControlState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowHeaderModel {
    pub title: &'static str,
    pub status_line: String,
    pub registered_window_count: usize,
    pub foreground_window_id: Option<StudioWindowHostId>,
    pub entitlement_timer_owner_window_id: Option<StudioWindowHostId>,
    pub has_parked_entitlement_timer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioGuiWindowCommandAreaModel {
    pub title: &'static str,
    pub sections: Vec<StudioGuiCommandSection>,
    pub total_command_count: usize,
    pub enabled_command_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowCanvasAreaModel {
    pub title: &'static str,
    pub widget: StudioGuiCanvasWidgetModel,
    pub focused_suggestion_id: Option<String>,
    pub suggestion_count: usize,
    pub enabled_action_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowRuntimeAreaModel {
    pub title: &'static str,
    pub control_state: WorkspaceControlState,
    pub run_panel: rf_ui::RunPanelWidgetModel,
    pub entitlement_host: Option<EntitlementSessionHostRuntimeOutput>,
    pub log_entries: Vec<rf_ui::AppLogEntry>,
    pub latest_log_entry: Option<rf_ui::AppLogEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StudioGuiWindowModel {
    pub header: StudioGuiWindowHeaderModel,
    pub commands: StudioGuiWindowCommandAreaModel,
    pub canvas: StudioGuiWindowCanvasAreaModel,
    pub runtime: StudioGuiWindowRuntimeAreaModel,
    pub layout_state: StudioGuiWindowLayoutState,
}

impl StudioGuiWindowModel {
    pub fn from_snapshot(snapshot: &StudioGuiSnapshot) -> Self {
        Self::from_snapshot_for_window(snapshot, None)
    }

    pub fn from_snapshot_for_window(
        snapshot: &StudioGuiSnapshot,
        window_id: Option<StudioWindowHostId>,
    ) -> Self {
        Self {
            header: header_from_snapshot(snapshot),
            commands: commands_from_registry(&snapshot.command_registry),
            canvas: canvas_from_snapshot(snapshot),
            runtime: runtime_from_snapshot(snapshot),
            layout_state: StudioGuiWindowLayoutState::from_snapshot_for_window(snapshot, window_id),
        }
    }
}

impl StudioGuiSnapshot {
    pub fn window_model(&self) -> StudioGuiWindowModel {
        StudioGuiWindowModel::from_snapshot(self)
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        StudioGuiWindowModel::from_snapshot_for_window(self, window_id)
    }
}

fn header_from_snapshot(snapshot: &StudioGuiSnapshot) -> StudioGuiWindowHeaderModel {
    let state = &snapshot.app_host_state;
    let registered_window_count = state.registered_windows.len();
    let foreground_window_id = state.foreground_window_id;
    let entitlement_timer_owner_window_id = state.entitlement_timer_owner_window_id();
    let has_parked_entitlement_timer = state.parked_entitlement_timer().is_some();
    let status_line = [
        format!("registered windows: {registered_window_count}"),
        foreground_window_id
            .map(|window_id| format!("foreground: #{window_id}"))
            .unwrap_or_else(|| "foreground: none".to_string()),
        match entitlement_timer_owner_window_id {
            Some(window_id) => format!("timer owner: #{window_id}"),
            None if has_parked_entitlement_timer => "timer owner: parked".to_string(),
            None => "timer owner: none".to_string(),
        },
    ]
    .join(" | ");

    StudioGuiWindowHeaderModel {
        title: "RadishFlow Studio",
        status_line,
        registered_window_count,
        foreground_window_id,
        entitlement_timer_owner_window_id,
        has_parked_entitlement_timer,
    }
}

fn commands_from_registry(registry: &StudioGuiCommandRegistry) -> StudioGuiWindowCommandAreaModel {
    let total_command_count = registry
        .sections
        .iter()
        .map(|section| section.commands.len())
        .sum();
    let enabled_command_count = registry
        .sections
        .iter()
        .flat_map(|section| section.commands.iter())
        .filter(|command| command.enabled)
        .count();

    StudioGuiWindowCommandAreaModel {
        title: "Commands",
        sections: registry.sections.clone(),
        total_command_count,
        enabled_command_count,
    }
}

fn canvas_from_snapshot(snapshot: &StudioGuiSnapshot) -> StudioGuiWindowCanvasAreaModel {
    let widget = snapshot.canvas.clone();
    let focused_suggestion_id = widget.view().focused_suggestion_id.clone();
    let suggestion_count = widget.view().suggestion_count;
    let enabled_action_count = widget.actions.iter().filter(|action| action.enabled).count();

    StudioGuiWindowCanvasAreaModel {
        title: "Canvas",
        widget,
        focused_suggestion_id,
        suggestion_count,
        enabled_action_count,
    }
}

fn runtime_from_snapshot(snapshot: &StudioGuiSnapshot) -> StudioGuiWindowRuntimeAreaModel {
    StudioGuiWindowRuntimeAreaModel {
        title: "Runtime",
        control_state: snapshot.runtime.control_state.clone(),
        run_panel: snapshot.runtime.run_panel.clone(),
        entitlement_host: snapshot.runtime.entitlement_host.clone(),
        latest_log_entry: snapshot.runtime.log_entries.last().cloned(),
        log_entries: snapshot.runtime.log_entries.clone(),
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
        StudioGuiDriver, StudioGuiDriverOutcome, StudioGuiEvent, StudioGuiHostCommandOutcome,
        StudioGuiWindowLayoutScopeKind, StudioRuntimeConfig,
        StudioRuntimeEntitlementPreflight, StudioRuntimeEntitlementSeed,
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
            "radishflow-studio-window-model-{timestamp}.rfproj.json"
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
    fn studio_gui_window_model_groups_snapshot_into_window_regions() {
        let (config, project_path) = flash_drum_local_rules_config();
        let mut driver = StudioGuiDriver::new(&config).expect("expected driver");

        let dispatch = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window = dispatch.snapshot.window_model();

        assert_eq!(window.header.title, "RadishFlow Studio");
        assert_eq!(window.header.registered_window_count, 1);
        assert_eq!(window.header.foreground_window_id, Some(1));
        assert_eq!(window.header.entitlement_timer_owner_window_id, Some(1));
        assert!(window.header.status_line.contains("registered windows: 1"));
        assert!(window.header.status_line.contains("foreground: #1"));
        assert!(window.header.status_line.contains("timer owner: #1"));

        assert_eq!(
            window.commands.total_command_count,
            dispatch.snapshot.ui_commands.actions.len()
        );
        assert!(
            window.commands.enabled_command_count >= 1,
            "expected at least one enabled command"
        );
        assert_eq!(
            window.commands.sections.first().map(|section| section.title),
            Some("Run Panel")
        );

        assert_eq!(window.canvas.title, "Canvas");
        assert_eq!(window.canvas.suggestion_count, 3);
        assert_eq!(window.canvas.enabled_action_count, 4);
        assert_eq!(
            window.canvas.focused_suggestion_id.as_deref(),
            Some("local.flash_drum.connect_inlet.flash-1.stream-heated")
        );
        assert_eq!(window.canvas.widget.primary_action().label, "Accept suggestion");

        assert_eq!(window.runtime.title, "Runtime");
        assert_eq!(window.runtime.control_state.run_status, rf_ui::RunStatus::Idle);
        assert_eq!(window.runtime.run_panel.view().primary_action.label, "Resume");
        assert!(window.runtime.entitlement_host.is_some());
        assert_eq!(
            window.runtime.latest_log_entry,
            window.runtime.log_entries.last().cloned()
        );
        assert_eq!(window.layout_state.scope.kind, StudioGuiWindowLayoutScopeKind::Window);
        assert_eq!(window.layout_state.scope.layout_key, "studio.window.owner.1");

        let _ = fs::remove_file(project_path);
    }

    #[test]
    fn studio_gui_window_model_reports_parked_timer_after_last_window_closes() {
        let mut driver = StudioGuiDriver::new(&lease_expiring_config()).expect("expected driver");
        let opened = driver
            .dispatch_event(StudioGuiEvent::OpenWindowRequested)
            .expect("expected open dispatch");
        let window_id = match opened.outcome {
            StudioGuiDriverOutcome::HostCommand(StudioGuiHostCommandOutcome::WindowOpened(
                opened,
            )) => opened.registration.window_id,
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

        let window = driver.snapshot().window_model();

        assert_eq!(window.header.registered_window_count, 0);
        assert_eq!(window.header.foreground_window_id, None);
        assert_eq!(window.header.entitlement_timer_owner_window_id, None);
        assert!(window.header.has_parked_entitlement_timer);
        assert!(window.header.status_line.contains("timer owner: parked"));
        assert_eq!(
            window.layout_state.scope.kind,
            StudioGuiWindowLayoutScopeKind::EmptyWorkspace
        );
        assert_eq!(window.layout_state.scope.layout_key, "studio.window.empty");
    }
}
