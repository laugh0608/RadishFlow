//! Selected material path, guarded flow preview and canvas return navigation.
use super::*;
use radishflow_studio::{
    FlowResultGuard, StudioGuiCanvasStreamLineEndpointViewModel,
    StudioGuiCanvasStreamLineViewModel, StudioGuiCanvasUnitBlockViewModel,
};
use rf_types::{StreamId, units::QuantityKind};
use rf_ui::{
    WorkspaceState,
    result_query::{FlowResultView, ResultStatus},
};

mod motion_preference;
use motion_preference::MotionPreference;

#[derive(Default)]
pub(super) struct FlowPathState {
    window: Option<StudioWindowHostId>,
    stream: Option<StreamId>,
    playing: Option<FlowPathPlayback>,
    visit: Option<FlowPathVisit>,
    return_focus: Option<FlowPathReturnFocus>,
}

struct FlowPathPlayback {
    guard: FlowResultGuard,
    started: f64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum FlowPathAction {
    Results,
    Diagnostics,
}

struct FlowPathVisit {
    stream: StreamId,
    guard: FlowResultGuard,
    action: FlowPathAction,
    viewport: CanvasInitialViewportFitState,
    navigation: CanvasViewportNavigationState,
    command_result: Option<radishflow_studio::StudioGuiCanvasCommandResultViewModel>,
    drawer: StudioShellBottomDrawerTab,
    inspector: StudioShellRightSidebarTab,
    inspector_target: Option<rf_ui::InspectorTarget>,
    runtime_details: bool,
    background_details: bool,
    focus_result_row: bool,
}

enum FlowPathReturnFocus {
    Action {
        action: FlowPathAction,
        after_pass: u64,
    },
    Notice {
        reason: FlowPathReturnNotice,
        focus_pending: bool,
    },
}

#[derive(Clone, Copy)]
enum FlowPathReturnNotice {
    StreamRemoved,
    ResultChanged,
    ActionUnavailable,
}

impl FlowPathReturnFocus {
    fn notice(reason: FlowPathReturnNotice) -> Self {
        Self::Notice {
            reason,
            focus_pending: true,
        }
    }
}

#[derive(Debug, PartialEq)]
enum FlowPathReading {
    Current {
        amount: f64,
        revision: u64,
        sequence: u64,
    },
    Unavailable(ResultStatus),
    NoStreamFlow,
    InvalidFlow,
}

impl FlowPathReading {
    fn from_workspace(workspace: &WorkspaceState, stream: &StreamId) -> Self {
        let view = workspace.flow_result();
        let FlowResultView::Current { snapshot, origin } = view else {
            return Self::Unavailable(view.status());
        };
        let Some(value) = snapshot
            .streams
            .iter()
            .find(|value| value.stream_id == *stream)
        else {
            return Self::NoStreamFlow;
        };
        let amount = value.total_molar_flow_mol_s;
        if !canvas_stream_layer::is_valid_result_molar_flow(amount) {
            return Self::InvalidFlow;
        }
        Self::Current {
            amount,
            revision: origin.revision(),
            sequence: snapshot.sequence,
        }
    }

    fn unavailable_text(&self, locale: StudioShellLocale) -> &'static str {
        match self {
            Self::Unavailable(ResultStatus::Stale) => text(
                locale,
                "结果过期 · 保留连接方向",
                "Stale result · showing connection direction",
            ),
            Self::Unavailable(ResultStatus::Failed) => text(
                locale,
                "运行失败 · 保留连接方向",
                "Run failed · showing connection direction",
            ),
            Self::NoStreamFlow => text(
                locale,
                "该流股缺少当前流量 · 保留连接方向",
                "No current flow for this stream · showing connection direction",
            ),
            Self::InvalidFlow => text(
                locale,
                "该流股的流量无效 · 保留连接方向",
                "Invalid stream flow · showing connection direction",
            ),
            _ => text(
                locale,
                "尚无当前结果 · 保留连接方向",
                "No current result · showing connection direction",
            ),
        }
    }

    fn playback_unavailable(
        &self,
        locale: StudioShellLocale,
        direction_available: bool,
        preference: MotionPreference,
    ) -> Option<&'static str> {
        match (self, direction_available, preference) {
            (Self::Current { amount, .. }, _, _) if *amount == 0.0 => Some(text(
                locale,
                "零流量 · 保留连接方向",
                "Zero flow · showing connection direction",
            )),
            (Self::Current { .. }, false, _) => Some(text(
                locale,
                "流股无已绑定端点",
                "No bound endpoints for this stream",
            )),
            (Self::Current { .. }, true, MotionPreference::Reduced) => Some(text(
                locale,
                "系统已开启减少动态效果",
                "Reduced motion is enabled by your system",
            )),
            (Self::Current { .. }, true, MotionPreference::Unknown) => Some(text(
                locale,
                "系统动态效果偏好不可用 · 保留静态箭头",
                "System motion preference unavailable · showing static arrows",
            )),
            (Self::Current { .. }, true, MotionPreference::Allowed) => None,
            _ => Some(self.unavailable_text(locale)),
        }
    }
}

impl FlowPathState {
    pub(super) fn selected(&self) -> Option<&StreamId> {
        self.stream.as_ref()
    }

    fn select(&mut self, window: Option<StudioWindowHostId>, stream: StreamId) {
        if self.stream.as_ref() != Some(&stream) {
            self.playing = None;
            self.return_focus = None;
        }
        self.window = window;
        self.stream = Some(stream);
    }

    pub(super) fn phase(&self, time: f64) -> Option<f32> {
        self.playing
            .as_ref()
            .map(|playback| ((time - playback.started) / 1.7).rem_euclid(1.0) as f32)
    }

    fn reconcile(&mut self, window: Option<StudioWindowHostId>, workspace: &WorkspaceState) {
        if self.window != window {
            *self = Self::default();
            return;
        }
        if self
            .stream
            .as_ref()
            .is_some_and(|id| !workspace.document.flowsheet.streams.contains_key(id))
        {
            self.stream = None;
            self.playing = None;
        }
        if self
            .playing
            .as_ref()
            .is_some_and(|playback| !playback.guard.is_current(workspace))
        {
            self.playing = None;
        }
    }
}

impl ReadyAppState {
    pub(super) fn reconcile_flow_path(&mut self) {
        self.flow_path.reconcile(
            self.current_window_id(),
            &self.platform_host.modeling_inputs().workspace,
        );
    }

    pub(super) fn flow_path_has_current_amount(&self, stream: &str) -> bool {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        matches!(
            FlowPathReading::from_workspace(workspace, &StreamId::new(stream)),
            FlowPathReading::Current { .. }
        )
    }

    pub(super) fn select_canvas_stream(&mut self, command_id: &str) {
        if self.screen == StudioShellScreen::Workbench
            && let Some(rf_ui::InspectorTarget::Stream(id)) =
                radishflow_studio::inspector_target_from_command_id(command_id)
            && self
                .platform_host
                .document()
                .flowsheet
                .streams
                .contains_key(&id)
        {
            self.flow_path.select(self.current_window_id(), id);
        }
        self.dispatch_ui_command(command_id);
    }

    pub(super) fn render_flow_path(&mut self, ui: &mut egui::Ui, window: &StudioGuiWindowModel) {
        if self.screen != StudioShellScreen::Workbench {
            return;
        }
        self.reconcile_flow_path();
        if !self.at_run_diagnostic_target()
            && self
                .flow_path
                .visit
                .as_ref()
                .is_some_and(|visit| visit.action == FlowPathAction::Results)
        {
            self.return_from_flow_path_visit(ui.ctx().cumulative_pass_nr());
        }
        let view = window.canvas.widget.view();
        let streams = &view.stream_lines;
        if streams.is_empty() && self.flow_path.return_focus.is_none() {
            return;
        }
        ui.push_id("canvas-flow-path", |ui| {
            self.render_flow_path_contents(ui, streams, &view.unit_blocks);
        });
    }

    fn render_flow_path_contents(
        &mut self,
        ui: &mut egui::Ui,
        streams: &[StudioGuiCanvasStreamLineViewModel],
        units: &[StudioGuiCanvasUnitBlockViewModel],
    ) {
        let locale = self.locale;
        let mut selected = self.flow_path.stream.clone();
        let original = selected.clone();
        let label = selected
            .as_ref()
            .and_then(|id| {
                streams
                    .iter()
                    .find(|stream| stream.stream_id == id.as_str())
            })
            .map(|stream| stream.name.as_str())
            .unwrap_or(text(locale, "选择流股", "Select stream"));
        ui.horizontal_wrapped(|ui| {
            ui.strong(text(locale, "二维流向与路径", "Flow direction and path"));
            egui::ComboBox::from_id_salt("stream")
                .selected_text(label)
                .width(ui.available_width().clamp(100.0, 220.0))
                .show_ui(ui, |ui| {
                    for stream in streams {
                        let id: StreamId = stream.stream_id.clone().into();
                        let choice = ui.selectable_value(&mut selected, Some(id), &stream.name);
                        choice.on_hover_text(&stream.stream_id);
                    }
                });
        });
        if selected != original
            && let Some(stream) = selected.as_ref()
            && let Some(model) = streams
                .iter()
                .find(|model| model.stream_id == stream.as_str())
        {
            self.select_canvas_stream(&model.command_id);
        }

        let Some((id, line)) = selected.as_ref().and_then(|id| {
            Some((
                id,
                streams
                    .iter()
                    .find(|stream| stream.stream_id == id.as_str())?,
            ))
        }) else {
            if matches!(
                self.flow_path.return_focus,
                Some(FlowPathReturnFocus::Action { .. })
            ) {
                self.flow_path.return_focus = Some(FlowPathReturnFocus::notice(
                    FlowPathReturnNotice::StreamRemoved,
                ));
            }
            self.render_flow_path_return_notice(ui);
            return;
        };
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let reading = FlowPathReading::from_workspace(workspace, id);
        let view = workspace.flow_result();
        let has_diagnostics = match &view {
            FlowResultView::Current { snapshot, .. } => !snapshot.diagnostics.is_empty(),
            FlowResultView::Failed { .. } => true,
            _ => false,
        };
        let quantity = QuantityKind::MolarFlow;
        let unit = workspace
            .project_presentation
            .display_units()
            .unit_for(quantity);
        let preference = motion_preference::system_motion_preference();
        let direction_available = line.source.is_some() || line.sink.is_some();
        let unavailable = reading.playback_unavailable(locale, direction_available, preference);
        if unavailable.is_some() {
            self.flow_path.playing = None;
        }

        let solved = matches!(reading, FlowPathReading::Current { .. });
        if let Some(FlowPathReturnFocus::Action { action, .. }) = self.flow_path.return_focus
            && !match action {
                FlowPathAction::Results => solved,
                FlowPathAction::Diagnostics => has_diagnostics,
            }
        {
            self.flow_path.return_focus = Some(FlowPathReturnFocus::notice(
                FlowPathReturnNotice::ActionUnavailable,
            ));
        }
        let upstream = if solved && line.sink.is_some() {
            text(locale, "流程入口", "Flowsheet inlet")
        } else {
            text(locale, "无上游绑定", "No upstream binding")
        };
        let downstream = if solved && line.source.is_some() {
            text(locale, "流程出口", "Flowsheet outlet")
        } else {
            text(locale, "无下游绑定", "No downstream binding")
        };
        for (label, locate, endpoint, unit_boundary) in [
            (
                text(locale, "上游", "Upstream"),
                text(locale, "定位上游", "Locate upstream"),
                line.source.as_ref(),
                upstream,
            ),
            (
                text(locale, "下游", "Downstream"),
                text(locale, "定位下游", "Locate downstream"),
                line.sink.as_ref(),
                downstream,
            ),
        ] {
            let target = endpoint.map(|endpoint| {
                let unit_name = units
                    .iter()
                    .find(|unit| unit.unit_id == endpoint.unit_id)
                    .map_or(endpoint.unit_id.as_str(), |unit| &unit.name);
                (endpoint, format!("{unit_name} · {}", endpoint.port_name))
            });
            self.render_flow_path_endpoint(ui, label, locate, target, unit_boundary);
        }

        if let FlowPathReading::Current {
            amount,
            revision,
            sequence,
        } = reading
        {
            let displayed = rf_ui::format_quantity(amount, quantity, unit)
                .unwrap_or_else(|error| error.to_string());
            render_wrapped_small(
                ui,
                format!(
                    "{} {displayed} · {} #{sequence} · r{revision}",
                    text(locale, "当前结果 · 摩尔流量", "Current result · molar flow"),
                    text(locale, "运行", "Run")
                ),
            );
        }
        let actions = |ui: &mut egui::Ui| {
            let playing = self.flow_path.playing.is_some();
            let label = if playing {
                text(locale, "暂停方向示意", "Pause direction preview")
            } else {
                text(locale, "播放方向示意", "Play direction preview")
            };
            if ui
                .add_enabled(unavailable.is_none(), egui::Button::new(label).wrap())
                .clicked()
            {
                self.flow_path.playing = if playing {
                    None
                } else {
                    self.flow_path_guard(FlowPathAction::Results, id)
                        .map(|guard| FlowPathPlayback {
                            guard,
                            started: ui.input(|input| input.time),
                        })
                };
            }
            if self.flow_path.visit.is_none() && solved {
                self.render_flow_path_action(
                    ui,
                    id,
                    FlowPathAction::Results,
                    text(locale, "在流程结果中查看", "View in flowsheet results"),
                );
            }
            if self.flow_path.visit.is_none() && has_diagnostics {
                self.render_flow_path_action(
                    ui,
                    id,
                    FlowPathAction::Diagnostics,
                    text(locale, "查看流程运行诊断", "View flowsheet run diagnostics"),
                );
            }
        };
        if ui.available_width().min(ui.clip_rect().width()) <= 410.0 {
            ui.vertical(actions);
        } else {
            ui.horizontal_wrapped(actions);
        }
        if let Some(reason) = unavailable {
            render_wrapped_small(ui, reason);
        }
        self.render_flow_path_return_notice(ui);
    }

    fn render_flow_path_endpoint(
        &mut self,
        ui: &mut egui::Ui,
        side: &str,
        locate: &str,
        target: Option<(&StudioGuiCanvasStreamLineEndpointViewModel, String)>,
        unbound: &str,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.small(side);
            if let Some((endpoint, label)) = target {
                render_wrapped_small(ui, &label);
                if ui
                    .add(egui::Button::new(locate).small().wrap())
                    .on_hover_text(&label)
                    .clicked()
                {
                    self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                    self.dispatch_ui_command(format!("inspector.focus_unit:{}", endpoint.unit_id));
                }
            } else {
                render_wrapped_small(ui, unbound);
            }
        });
    }

    fn flow_path_guard(
        &self,
        action: FlowPathAction,
        stream: &StreamId,
    ) -> Option<FlowResultGuard> {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if !workspace.document.flowsheet.streams.contains_key(stream) {
            return None;
        }
        match (action, workspace.flow_result()) {
            (FlowPathAction::Results, FlowResultView::Current { snapshot, .. })
                if snapshot
                    .streams
                    .iter()
                    .any(|value| &value.stream_id == stream) =>
            {
                FlowResultGuard::from_snapshot(snapshot)
            }
            (FlowPathAction::Diagnostics, FlowResultView::Current { snapshot, .. })
                if !snapshot.diagnostics.is_empty() =>
            {
                FlowResultGuard::from_snapshot(snapshot)
            }
            (FlowPathAction::Diagnostics, FlowResultView::Failed { .. }) => {
                radishflow_studio::FlowResultReview::capture(workspace).guard
            }
            _ => None,
        }
    }

    fn render_flow_path_action(
        &mut self,
        ui: &mut egui::Ui,
        stream: &StreamId,
        action: FlowPathAction,
        label: &str,
    ) {
        let response = ui
            .push_id(action, |ui| ui.add(egui::Button::new(label).wrap()))
            .inner;
        if let Some(FlowPathReturnFocus::Action {
            action: origin,
            after_pass,
        }) = self.flow_path.return_focus
            && origin == action
        {
            if ui.ctx().cumulative_pass_nr() > after_pass {
                response.scroll_to_me(None);
                response.request_focus();
                self.flow_path.return_focus = None;
            } else {
                ui.ctx().request_repaint();
            }
        }
        if response.clicked() {
            self.open_flow_path_visit(stream, action);
        }
    }

    fn open_flow_path_visit(&mut self, stream: &StreamId, action: FlowPathAction) {
        let Some(guard) = self.flow_path_guard(action, stream) else {
            return;
        };
        self.flow_path.return_focus = None;
        self.flow_path.visit = Some(FlowPathVisit {
            stream: stream.clone(),
            guard,
            action,
            viewport: self.canvas_initial_viewport_fit,
            navigation: self.canvas_viewport_navigation.clone(),
            command_result: self.canvas_command_result.clone(),
            drawer: self.bottom_drawer_tab,
            inspector: self.right_sidebar_tab,
            inspector_target: self
                .platform_host
                .snapshot()
                .runtime
                .active_inspector_target,
            runtime_details: self.modeling_problems.runtime_details(),
            background_details: self.background_run_details_open,
            focus_result_row: action == FlowPathAction::Results,
        });
        match action {
            FlowPathAction::Diagnostics => self.open_flow_diagnostics(),
            FlowPathAction::Results => {
                if let FlowResultView::Current { snapshot, .. } =
                    self.platform_host.modeling_inputs().workspace.flow_result()
                {
                    self.result_inspector
                        .select_stream(snapshot.id.as_str(), stream.as_str());
                }
                self.screen = StudioShellScreen::Results;
                self.bottom_drawer_tab = StudioShellBottomDrawerTab::ResultsTable;
                self.background_run_details_open = false;
            }
        }
    }

    pub(super) fn focus_flow_path_result_row(&mut self, stream: &str, response: &egui::Response) {
        if self.screen == StudioShellScreen::Results
            && let Some(visit) = self.flow_path.visit.as_mut()
            && visit.focus_result_row
            && visit.stream.as_str() == stream
            && visit
                .guard
                .is_current(&self.platform_host.modeling_inputs().workspace)
        {
            response.scroll_to_me(None);
            response.request_focus();
            visit.focus_result_row = false;
        }
    }

    pub(super) fn render_flow_path_return(&mut self, ui: &mut egui::Ui) {
        self.reconcile_flow_path();
        let Some(visit) = self.flow_path.visit.as_ref() else {
            return;
        };
        let on_visit = match visit.action {
            FlowPathAction::Results => self.screen == StudioShellScreen::Results,
            FlowPathAction::Diagnostics => {
                self.screen == StudioShellScreen::Workbench && !self.at_run_diagnostic_target()
            }
        };
        if on_visit
            && ui
                .add(
                    egui::Button::new(text(
                        self.locale,
                        "← 返回二维流向与路径",
                        "← Back to flow direction and path",
                    ))
                    .wrap(),
                )
                .clicked()
        {
            ui.input_mut(|input| {
                input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                input.consume_key(egui::Modifiers::NONE, egui::Key::Space);
            });
            self.return_from_flow_path_visit(ui.ctx().cumulative_pass_nr());
        }
    }

    fn return_from_flow_path_visit(&mut self, after_pass: u64) {
        let Some(visit) = self.flow_path.visit.take() else {
            return;
        };
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let exists = workspace
            .document
            .flowsheet
            .streams
            .contains_key(&visit.stream);
        let current = visit.guard.is_current(workspace);
        let inspector_target = visit.inspector_target.filter(|target| match target {
            rf_ui::InspectorTarget::Unit(id) => workspace.document.flowsheet.units.contains_key(id),
            rf_ui::InspectorTarget::Stream(id) => {
                workspace.document.flowsheet.streams.contains_key(id)
            }
        });
        if exists {
            self.flow_path
                .select(self.current_window_id(), visit.stream);
        }
        self.flow_path.return_focus = Some(if !exists {
            FlowPathReturnFocus::notice(FlowPathReturnNotice::StreamRemoved)
        } else if !current {
            FlowPathReturnFocus::notice(FlowPathReturnNotice::ResultChanged)
        } else {
            FlowPathReturnFocus::Action {
                action: visit.action,
                after_pass,
            }
        });
        self.screen = StudioShellScreen::Workbench;
        self.bottom_drawer_tab = visit.drawer;
        if let Some(target) = inspector_target
            && self
                .platform_host
                .snapshot()
                .runtime
                .active_inspector_target
                .as_ref()
                != Some(&target)
        {
            // Restore Inspector identity separately from the saved canvas viewport.
            self.dispatch_event(StudioGuiEvent::UiCommandRequested {
                command_id: radishflow_studio::inspector_target_command_id(&target),
            });
        }
        self.right_sidebar_tab = visit.inspector;
        self.modeling_problems
            .set_runtime_details(visit.runtime_details);
        self.background_run_details_open = visit.background_details;
        self.canvas_initial_viewport_fit = visit.viewport;
        self.canvas_viewport_navigation = visit.navigation;
        self.canvas_command_result = visit.command_result;
    }

    fn render_flow_path_return_notice(&mut self, ui: &mut egui::Ui) {
        let Some(focus) = &mut self.flow_path.return_focus else {
            return;
        };
        let FlowPathReturnFocus::Notice {
            reason,
            focus_pending,
        } = focus
        else {
            return;
        };
        let notice = match reason {
            FlowPathReturnNotice::StreamRemoved => text(
                self.locale,
                "原流股已移除。",
                "The originating stream was removed.",
            ),
            FlowPathReturnNotice::ResultChanged => text(
                self.locale,
                "流程结果已变化。已返回当前画布。",
                "The flow result changed. Returned to the current canvas.",
            ),
            FlowPathReturnNotice::ActionUnavailable => text(
                self.locale,
                "原动作已不可用。",
                "The originating action is unavailable.",
            ),
        };
        let response = ui.add(
            egui::Label::new(notice)
                .sense(egui::Sense::focusable_noninteractive())
                .wrap(),
        );
        if std::mem::take(focus_pending) {
            response.scroll_to_me(None);
            response.request_focus();
        }
    }
}

fn text(locale: StudioShellLocale, zh: &'static str, en: &'static str) -> &'static str {
    match locale {
        StudioShellLocale::ZhCn => zh,
        StudioShellLocale::En => en,
    }
}

#[cfg(test)]
mod tests;
