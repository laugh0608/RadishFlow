use super::*;

impl ReadyAppState {
    pub(super) fn handle_quit_shortcut(&mut self, ctx: &egui::Context) -> bool {
        let quit_requested =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::Q));
        if !quit_requested {
            return false;
        }

        if self.close_current_window_for_viewport_request() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        true
    }

    pub(super) fn handle_command_palette_toggle_shortcut(&mut self, ctx: &egui::Context) -> bool {
        if self.has_modal_dialog() {
            self.command_palette.close();
            return false;
        }
        let toggle_requested =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::K));
        if toggle_requested {
            self.command_palette.toggle();
        }
        toggle_requested
    }

    pub(super) fn handle_command_palette_keyboard(
        &mut self,
        ctx: &egui::Context,
        commands: &radishflow_studio::StudioGuiWindowCommandAreaModel,
    ) -> bool {
        if self.has_modal_dialog() {
            self.command_palette.close();
            return false;
        }
        if !self.command_palette.open {
            return false;
        }

        let palette_items = commands.palette_items(&self.command_palette.query);
        self.command_palette.sync_selection(&palette_items);

        if ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.command_palette.close();
            return true;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::ArrowDown)) {
            self.command_palette.move_selection(1, &palette_items);
            return true;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::ArrowUp)) {
            self.command_palette.move_selection(-1, &palette_items);
            return true;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::Enter)) {
            let selected_command_id = selected_palette_item_command_id(
                &palette_items,
                self.command_palette.selected_index,
            );
            if let Some(command_id) = selected_command_id {
                self.dispatch_ui_command(command_id);
                self.command_palette.close();
            }
            return true;
        }

        false
    }

    pub(super) fn dispatch_shortcuts(&mut self, ctx: &egui::Context) {
        let focus_context = self.focus_context(ctx);
        if matches!(
            focus_context,
            StudioGuiFocusContext::CommandPalette | StudioGuiFocusContext::ModalDialog
        ) {
            return;
        }

        if self.drag_session.is_some() && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.cancel_drag_session(self.current_window_id());
            return;
        }

        let shortcuts = ctx.input(collect_shortcuts);
        for shortcut in shortcuts {
            if self.intercept_modeling_readiness_shortcut_if_needed(&shortcut) {
                continue;
            }
            if let radishflow_studio::StudioGuiShortcutRoute::DispatchCommandId { command_id } =
                radishflow_studio::route_shortcut(
                    &self.platform_host.snapshot().command_registry,
                    &shortcut,
                    focus_context,
                )
            {
                // Menu, palette and keyboard commands share shell confirmations and pickers.
                self.dispatch_ui_command(command_id);
                continue;
            }
            self.dispatch_event(StudioGuiEvent::ShortcutPressed {
                shortcut,
                focus_context,
            });
        }
    }

    fn has_modal_dialog(&self) -> bool {
        self.unit_settings.is_open()
            || self.pending_format_upgrade.is_some()
            || self.variable_browser.open
            || self.pending_input_action.is_some()
            || self.pending_unit_deletion.is_some()
            || self.pending_result_export.is_some()
            || self.project_open.pending_confirmation.is_some()
            || self.project_open.pending_blank_project_confirmation
            || self.project_open.pending_save_as_overwrite.is_some()
            || self
                .project_open
                .pending_close_window_confirmation
                .is_some()
    }

    pub(super) fn focus_context(&self, ctx: &egui::Context) -> StudioGuiFocusContext {
        if self.has_modal_dialog() {
            StudioGuiFocusContext::ModalDialog
        } else if self.command_palette.open {
            StudioGuiFocusContext::CommandPalette
        } else if ctx.wants_keyboard_input() || numeric_input::owns_keyboard(ctx) {
            StudioGuiFocusContext::TextInput
        } else if self
            .platform_host
            .snapshot()
            .window_model()
            .canvas
            .focused_suggestion_id
            .is_some()
        {
            StudioGuiFocusContext::CanvasSuggestionFocused
        } else if self.last_area_focus == Some(StudioGuiWindowAreaId::Canvas) {
            StudioGuiFocusContext::Canvas
        } else {
            StudioGuiFocusContext::Global
        }
    }
}
