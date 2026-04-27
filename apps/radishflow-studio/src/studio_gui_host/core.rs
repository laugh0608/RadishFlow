use super::helpers::{
    dispatch_from_controller, global_event_from_controller, ui_commands_from_projection,
};
use super::*;

impl StudioGuiHost {
    pub fn new(config: &StudioRuntimeConfig) -> RfResult<Self> {
        let controller = StudioAppHostController::new(config)?;
        let layout_state_overrides = match controller.document_path() {
            Some(project_path) => load_persisted_window_layouts(project_path)?,
            None => BTreeMap::new(),
        };

        Ok(Self {
            controller,
            layout_state_overrides,
            window_drop_previews: BTreeMap::new(),
        })
    }

    pub fn state(&self) -> &StudioAppHostState {
        self.controller.state()
    }

    pub fn ui_commands(&self) -> StudioAppHostUiCommandModel {
        self.state().ui_command_model()
    }

    pub fn canvas_state(&self) -> StudioGuiCanvasState {
        let canvas = self.controller.canvas_interaction();
        StudioGuiCanvasState {
            suggestions: canvas.suggestions,
            focused_suggestion_id: canvas.focused_suggestion_id,
        }
    }

    pub fn command_registry(&self) -> StudioGuiCommandRegistry {
        StudioGuiCommandRegistry::from_surfaces(
            &self.ui_commands(),
            &self.canvas_state(),
            self.preferred_target_window_id(),
        )
    }

    pub fn snapshot(&self) -> StudioGuiSnapshot {
        let mut snapshot = StudioGuiSnapshot::new(
            self.state().clone(),
            self.ui_commands(),
            self.command_registry(),
            self.canvas_state().widget(),
            StudioGuiRuntimeSnapshot {
                workspace_document: workspace_document_snapshot_from_controller(&self.controller),
                control_state: self.controller.workspace_control_state(),
                run_panel: self.controller.run_panel_widget(),
                latest_solve_snapshot: self.controller.latest_solve_snapshot(),
                entitlement_host: self.controller.entitlement_host_output(),
                platform_notice: None,
                platform_timer_lines: Vec::new(),
                gui_activity_lines: Vec::new(),
                log_entries: self.controller.log_entries(),
            },
            self.window_drop_previews.clone(),
        );
        snapshot.layout_state = self.layout_state_for_window_from_snapshot(&snapshot, None);
        snapshot
    }

    pub fn window_model_for_window(
        &self,
        window_id: Option<StudioWindowHostId>,
    ) -> StudioGuiWindowModel {
        let snapshot = self.snapshot();
        let mut window = snapshot.window_model_for_window(window_id);
        window.layout_state = self.layout_state_for_window_from_snapshot(&snapshot, window_id);
        window
    }

    pub fn refresh_local_canvas_suggestions(&mut self) {
        self.controller.refresh_local_canvas_suggestions();
    }

    pub fn replace_canvas_suggestions(&mut self, suggestions: Vec<CanvasSuggestion>) {
        self.controller.replace_canvas_suggestions(suggestions);
    }

    pub fn execute_command(
        &mut self,
        command: StudioGuiHostCommand,
    ) -> RfResult<StudioGuiHostCommandOutcome> {
        match command {
            StudioGuiHostCommand::OpenWindow => self
                .open_window()
                .map(StudioGuiHostCommandOutcome::WindowOpened),
            StudioGuiHostCommand::DispatchWindowTrigger { window_id, trigger } => self
                .dispatch_window_trigger(window_id, trigger)
                .map(StudioGuiHostCommandOutcome::WindowDispatched),
            StudioGuiHostCommand::DispatchCanvasInteraction { action } => self
                .dispatch_canvas_interaction(action)
                .map(StudioGuiHostCommandOutcome::CanvasInteracted),
            StudioGuiHostCommand::DispatchLifecycleEvent { event } => self
                .dispatch_lifecycle_event(event)
                .map(StudioGuiHostCommandOutcome::LifecycleDispatched),
            StudioGuiHostCommand::DispatchUiCommand { command_id } => self
                .dispatch_ui_command(&command_id)
                .map(StudioGuiHostCommandOutcome::UiCommandDispatched),
            StudioGuiHostCommand::QueryWindowDropTarget { window_id, query } => self
                .query_window_drop_target(window_id, query)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetQueried),
            StudioGuiHostCommand::SetWindowDropTargetPreview { window_id, query } => self
                .set_window_drop_target_preview(window_id, query)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetPreviewUpdated),
            StudioGuiHostCommand::ClearWindowDropTargetPreview { window_id } => self
                .clear_window_drop_target_preview(window_id)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetPreviewCleared),
            StudioGuiHostCommand::ApplyWindowDropTarget { window_id, query } => self
                .apply_window_drop_target(window_id, query)
                .map(StudioGuiHostCommandOutcome::WindowDropTargetApplied),
            StudioGuiHostCommand::CloseWindow { window_id } => self
                .close_window(window_id)
                .map(StudioGuiHostCommandOutcome::WindowClosed),
        }
    }

    pub fn open_window(&mut self) -> RfResult<StudioGuiHostWindowOpened> {
        let opened = self.controller.open_window()?;
        Ok(StudioGuiHostWindowOpened {
            ui_commands: ui_commands_from_projection(&opened.projection),
            canvas: self.canvas_state(),
            projection: opened.projection,
            registration: opened.registration,
            native_timers: StudioGuiNativeTimerEffects::from_driver(
                &opened.native_timer_transitions,
                &opened.native_timer_acks,
            ),
        })
    }

    pub fn dispatch_window_trigger(
        &mut self,
        window_id: StudioWindowHostId,
        trigger: StudioRuntimeTrigger,
    ) -> RfResult<StudioGuiHostDispatch> {
        let dispatch = self
            .controller
            .dispatch_window_trigger(window_id, trigger)?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn focus_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioGuiHostDispatch> {
        let dispatch = self.controller.focus_window(window_id)?;
        Ok(dispatch_from_controller(dispatch, self.canvas_state()))
    }

    pub fn dispatch_global_event(
        &mut self,
        event: StudioAppWindowHostGlobalEvent,
    ) -> RfResult<StudioGuiHostGlobalEventDispatch> {
        let result = self.controller.dispatch_global_event(event)?;
        Ok(global_event_from_controller(result, self.canvas_state()))
    }

    pub fn close_window(
        &mut self,
        window_id: StudioWindowHostId,
    ) -> RfResult<StudioGuiHostCloseWindowResult> {
        if self.state().window(window_id).is_some() {
            let snapshot = self.snapshot();
            let layout_state =
                self.layout_state_for_window_from_snapshot(&snapshot, Some(window_id));
            self.clear_window_drop_preview_for_scope(&layout_state.scope.layout_key);
            if let Some(legacy_layout_key) = layout_state.scope.legacy_layout_key() {
                self.clear_window_drop_preview_for_scope(&legacy_layout_key);
            }
        }
        let closed = self.controller.close_window(window_id)?;
        Ok(StudioGuiHostCloseWindowResult {
            ui_commands: ui_commands_from_projection(&closed.projection),
            canvas: self.canvas_state(),
            projection: closed.projection,
            native_timers: closed
                .close
                .as_ref()
                .map(|close| {
                    StudioGuiNativeTimerEffects::from_driver(
                        &close.native_timer_transitions,
                        &close.native_timer_acks,
                    )
                })
                .unwrap_or_default(),
            close: closed.close,
        })
    }
}

fn workspace_document_snapshot_from_controller(
    controller: &crate::StudioAppHostController,
) -> crate::StudioGuiWorkspaceDocumentSnapshot {
    let document = controller.document();
    crate::StudioGuiWorkspaceDocumentSnapshot {
        document_id: document.metadata.document_id.as_str().to_string(),
        title: document.metadata.title.clone(),
        flowsheet_name: document.flowsheet.name.clone(),
        revision: document.revision,
        project_path: controller
            .document_path()
            .map(|path| path.display().to_string()),
        unit_count: document.flowsheet.units.len(),
        stream_count: document.flowsheet.streams.len(),
        snapshot_history_count: controller.snapshot_history_count(),
    }
}
