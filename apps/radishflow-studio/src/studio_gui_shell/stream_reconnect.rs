use super::locale::tr;
use super::*;
use radishflow_studio::{
    StudioBackgroundRunAction,
    flow_actions::{FlowAction, FlowActionRequest, FlowEditAccess, FlowEditContext},
};
use rf_types::{PortDirection, StreamId};
use rf_ui::{
    InspectorTarget, StreamPortBinding, StreamReconnect, StreamReconnectOptions,
    stream_reconnect_options,
};

pub(super) struct ReconnectPicker {
    stream: StreamId,
    expected: FlowEditContext,
    options: StreamReconnectOptions,
    chosen: Option<StreamPortBinding>,
    highlighted: Option<usize>,
    pointer_highlight: bool,
    rendered_pass: Option<u64>,
    error: Option<String>,
    origin: egui::Id,
    focus_first: bool,
}

pub(super) struct ReconnectCanvasPorts<'a> {
    pub options: &'a StreamReconnectOptions,
    pub chosen: Option<&'a StreamPortBinding>,
    pub highlighted: Option<&'a StreamPortBinding>,
}

impl ReconnectPicker {
    pub(super) fn canvas_ports(
        &self,
        pass: u64,
        workspace: &rf_ui::WorkspaceState,
    ) -> Option<ReconnectCanvasPorts<'_>> {
        // Only paint a picker displayed for this input in the current window pass.
        if self.rendered_pass != Some(pass)
            || self.error.is_some()
            || self.expected.validate(workspace).is_err()
            || workspace.drafts.active_target.as_ref()
                != Some(&InspectorTarget::Stream(self.stream.clone()))
        {
            return None;
        }
        Some(ReconnectCanvasPorts {
            options: &self.options,
            chosen: self.chosen.as_ref(),
            highlighted: self
                .highlighted
                .and_then(|i| self.options.candidates.get(i)),
        })
    }
}

fn endpoint(port: &StreamPortBinding) -> String {
    format!("{}:{}", port.unit_id, port.port)
}

impl ReadyAppState {
    pub(super) fn reconcile_reconnect_picker(&mut self) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if self.reconnect_picker.as_ref().is_some_and(|p| {
            workspace.drafts.active_target.as_ref()
                != Some(&InspectorTarget::Stream(p.stream.clone()))
                || !workspace.document.flowsheet.streams.contains_key(&p.stream)
                || !p.expected.belongs_to_session(workspace)
                || self.read_only_project.is_some()
        }) {
            self.reconnect_picker = None;
        }
    }

    pub(super) fn reconnect_picker_shortcut(&mut self, ctx: &egui::Context) -> bool {
        if self.reconnect_picker.is_none()
            || self.right_sidebar_tab != StudioShellRightSidebarTab::Inspector
            || !matches!(
                self.screen,
                StudioShellScreen::Workbench
                    | StudioShellScreen::Run
                    | StudioShellScreen::Results
                    | StudioShellScreen::Property
            )
        {
            return false;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.close_reconnect_picker(ctx);
            return true;
        }
        ctx.input(|i| {
            !i.modifiers.command
                && [egui::Key::Tab, egui::Key::Enter, egui::Key::Space]
                    .iter()
                    .any(|k| i.key_pressed(*k))
        })
    }

    fn close_reconnect_picker(&mut self, ctx: &egui::Context) {
        if let Some(p) = self.reconnect_picker.take() {
            // The Inspector changes layout when the picker disappears. Its old auto-ID
            // may be absent from the native accessibility tree on the closing frame.
            ctx.memory_mut(|m| m.stop_text_input());
            let next_pass = ctx.cumulative_pass_nr() + 1;
            ctx.data_mut(|d| d.insert_temp(p.origin, next_pass));
            ctx.request_repaint();
        }
    }

    pub(super) fn render_reconnect_entry(
        &mut self,
        ui: &mut egui::Ui,
        stream_id: &str,
        origin: &str,
    ) -> bool {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let Some(InspectorTarget::Stream(stream)) = &workspace.drafts.active_target else {
            return false;
        };
        if self.read_only_project.is_some() || stream.as_str() != stream_id {
            return false;
        }
        let StreamReconnect::Open(options) =
            stream_reconnect_options(&workspace.document.flowsheet, stream)
        else {
            return false;
        };
        if options.candidates.len() < 2 {
            return false;
        }
        let label = match options.missing {
            PortDirection::Outlet => tr(self.locale, "选择源端口…", "Choose source port…"),
            PortDirection::Inlet => tr(self.locale, "选择目标端口…", "Choose sink port…"),
        };
        let entry = egui::Id::new(("stream-reconnect-entry", stream_id, origin));
        let response = ui.small_button(label);
        let pass = ui.ctx().cumulative_pass_nr();
        if ui.data(|d| d.get_temp::<u64>(entry)) == Some(pass) {
            response.request_focus();
            response.scroll_to_me(None);
            ui.data_mut(|d| d.remove::<u64>(entry));
        }
        if response.clicked() {
            self.reconnect_picker = Some(ReconnectPicker {
                stream: stream.clone(),
                expected: FlowEditContext::capture(workspace),
                options,
                chosen: None,
                highlighted: None,
                pointer_highlight: false,
                rendered_pass: None,
                error: None,
                origin: entry,
                focus_first: true,
            });
            self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
            if let Some(view) = self.current_window_id().map(rf_ui::DisplayUnitViewId) {
                self.unit_settings.closed_inspectors.remove(&view);
            }
            ui.ctx().request_repaint();
        }
        true
    }

    pub(super) fn render_reconnect_picker(&mut self, ui: &mut egui::Ui, stream_id: &str) {
        self.reconcile_reconnect_picker();
        let Some(p) = self
            .reconnect_picker
            .as_mut()
            .filter(|p| p.stream.as_str() == stream_id)
        else {
            return;
        };
        let flow = &self
            .platform_host
            .modeling_inputs()
            .workspace
            .document
            .flowsheet;
        let locale = self.locale;
        let mut refresh = false;
        let mut confirm = false;
        let mut close = false;
        ui.push_id(("reconnect-picker", stream_id), |ui| {
            ui.group(|ui| {
                ui.set_max_width(ui.available_width());
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                let fixed = match p.options.missing {
                    PortDirection::Inlet => tr(locale, "固定源端", "Fixed source"),
                    PortDirection::Outlet => tr(locale, "固定目标端", "Fixed sink"),
                };
                ui.strong(format!("{} · {}", flow.streams[&p.stream].name, p.stream));
                ui.label(format!("{fixed}  {}", endpoint(&p.options.fixed)));
                if let Some(error) = &p.error {
                    ui.label(error);
                    if ui
                        .button(tr(locale, "重新查看候选", "Refresh available ports"))
                        .clicked()
                    {
                        refresh = true;
                    }
                } else {
                    p.rendered_pass = Some(ui.ctx().cumulative_pass_nr());
                    ui.small(tr(
                        locale,
                        "菱形标出固定端，虚线表示待确认的连接。",
                        "The diamond marks the fixed port. A dashed line previews the connection.",
                    ));
                    // A stationary pointer must not mask keyboard navigation among the rows.
                    ui.input(|i| {
                        for event in &i.events {
                            match event {
                                egui::Event::PointerMoved(_)
                                | egui::Event::PointerButton { pressed: true, .. } => {
                                    p.pointer_highlight = true
                                }
                                egui::Event::Key { pressed: true, .. }
                                | egui::Event::AccessKitActionRequest(_) => {
                                    p.pointer_highlight = false
                                }
                                _ => {}
                            }
                        }
                    });
                    let mut hovered = None;
                    let mut focused = None;
                    for (idx, port) in p.options.candidates.iter().enumerate() {
                        let label = match flow.units.get(&port.unit_id) {
                            Some(unit) => format!("{} · {}", unit.name, endpoint(port)),
                            None => endpoint(port),
                        };
                        let row = ui.radio(p.chosen.as_ref() == Some(port), label);
                        if idx == 0 && p.focus_first {
                            row.request_focus();
                            row.scroll_to_me(Some(egui::Align::Center));
                            p.focus_first = false;
                        }
                        if row.hovered() {
                            hovered = Some(idx);
                        }
                        if row.has_focus() {
                            focused = Some(idx);
                        }
                        if row.clicked() {
                            p.chosen = Some(port.clone());
                        }
                    }
                    p.highlighted = if p.pointer_highlight {
                        hovered.or(focused)
                    } else {
                        focused
                    };
                    if let Some(chosen) = &p.chosen {
                        let (source, sink) = match p.options.missing {
                            PortDirection::Inlet => (&p.options.fixed, chosen),
                            PortDirection::Outlet => (chosen, &p.options.fixed),
                        };
                        ui.label(format!("{} → {}", endpoint(source), endpoint(sink)));
                    }
                    confirm = ui
                        .add_enabled(
                            p.chosen.is_some(),
                            egui::Button::new(tr(locale, "确认重连", "Confirm reconnection")),
                        )
                        .clicked();
                }
                close |= ui.button(tr(locale, "取消", "Cancel")).clicked();
            })
        });
        if refresh {
            let workspace = &self.platform_host.modeling_inputs().workspace;
            match stream_reconnect_options(&workspace.document.flowsheet, &p.stream) {
                StreamReconnect::Open(options) if !options.candidates.is_empty() => {
                    p.options = options;
                    p.expected = FlowEditContext::capture(workspace);
                    p.error = None;
                    p.chosen = None;
                    p.highlighted = None;
                    p.rendered_pass = None;
                    p.focus_first = true;
                }
                _ => self.close_reconnect_picker(ui.ctx()),
            }
        } else if close {
            self.close_reconnect_picker(ui.ctx());
        } else if confirm {
            self.confirm_reconnect(ui.ctx());
        }
    }

    fn confirm_reconnect(&mut self, ctx: &egui::Context) {
        self.reconcile_reconnect_picker();
        let Some(p) = self.reconnect_picker.as_mut() else {
            return;
        };
        let Some(chosen) = p.chosen.take() else {
            return;
        };
        let request = FlowActionRequest {
            expected: p.expected.clone(),
            action: FlowAction::Reconnect {
                stream: p.stream.clone(),
                fixed: p.options.fixed.clone(),
                missing: p.options.missing,
                chosen: chosen.clone(),
            },
        };
        let access = if self.read_only_project.is_some() {
            FlowEditAccess::ReadOnly
        } else {
            FlowEditAccess::Editable
        };
        match self
            .platform_host
            .background_run_action(StudioBackgroundRunAction::EditFlow { access, request })
        {
            Ok(_) => {
                self.reconnect_picker = None;
                ctx.memory_mut(|m| m.stop_text_input());
                ctx.request_repaint();
            }
            Err(error) => {
                let workspace = &self.platform_host.modeling_inputs().workspace;
                let occupant = workspace
                    .document
                    .flowsheet
                    .units
                    .get(&chosen.unit_id)
                    .and_then(|u| u.ports.iter().find(|port| port.name == chosen.port))
                    .and_then(|port| port.stream_id.as_ref());
                p.error = Some(match occupant {
                    Some(id) => format!(
                        "{} {} {}",
                        endpoint(&chosen),
                        tr(self.locale, "已连接到", "is connected to"),
                        id
                    ),
                    None => format!(
                        "{} · {error}",
                        tr(self.locale, "重连未提交", "Reconnection declined")
                    ),
                });
            }
        }
    }
}
