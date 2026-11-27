//! In-session navigation between a recycle report and its qualified stream target.
use super::*;
use radishflow_studio::{FlowResultGuard, FlowResultReview, StudioRunId};
use rf_ui::result_query::{FlowResultOrigin, FlowResultView};

#[derive(Default)]
pub(super) struct RecycleReview {
    guard: Option<FlowResultGuard>,
    generation: u64,
    visit: Option<Visit>,
    return_focus: Option<(egui::Id, u64)>,
}

struct Visit {
    stream: StreamId,
    binding: String,
    screen: StudioShellScreen,
    action: egui::Id,
    focus_row: bool,
}

pub(super) fn tear_is_bound(workspace: &rf_ui::WorkspaceState, tear: &TearSpecification) -> bool {
    let flow = &workspace.document.flowsheet;
    flow.streams.contains_key(&tear.stream_id)
        && [
            (&tear.source, PortDirection::Outlet),
            (&tear.sink, PortDirection::Inlet),
        ]
        .iter()
        .all(|(end, direction)| material_port_is_bound(flow, &tear.stream_id, end, *direction))
}

pub(super) fn material_port_is_bound(
    flow: &rf_model::Flowsheet,
    stream: &StreamId,
    end: &DiagnosticPortTarget,
    direction: PortDirection,
) -> bool {
    flow.units.get(&end.unit_id).is_some_and(|unit| {
        unit.ports.iter().any(|port| {
            port.kind == PortKind::Material
                && port.direction == direction
                && port.name == end.port_name
                && port.stream_id.as_ref() == Some(stream)
        })
    })
}

impl RecycleReview {
    pub(super) fn bind(&mut self, workspace: &rf_ui::WorkspaceState) -> u64 {
        if !self.guard.as_ref().is_some_and(|g| g.is_current(workspace)) {
            self.guard = FlowResultReview::capture(workspace).guard;
            self.generation += 1;
            self.visit = None;
            self.return_focus = None;
        }
        self.generation
    }
    pub(super) fn stream_action(
        &mut self,
        stream: &StreamId,
        response: &egui::Response,
        clicked: &mut Option<(StreamId, egui::Id)>,
    ) {
        reveal_plan_focus(response);
        if response.clicked() {
            *clicked = Some((stream.clone(), response.id));
        }
        if let Some((id, after)) = self.return_focus
            && id == response.id
        {
            if response.ctx.cumulative_pass_nr() > after {
                response.scroll_to_me(Some(egui::Align::Center));
                response.request_focus();
                self.return_focus = None;
            } else {
                response.ctx.request_repaint();
            }
        }
    }
}

impl ReadyAppState {
    pub(super) fn recycle_source_text(&self, origin: &FlowResultOrigin, zh: bool) -> String {
        let mut parts = vec![format!(
            "{} {} · {}",
            tr(zh, "输入修订", "Input revision"),
            origin.revision(),
            origin.document_id()
        )];
        if let Some(e) = origin.execution() {
            if let Some(owner) = &self.background_runs
                && let Some(number) = owner.borrow().history().iter().find_map(|item| {
                    matches!(&item.activity.id,StudioRunId::Flow(id) if id == &e.run_id)
                        .then_some(item.activity.number)
                })
            {
                parts.insert(0, format!("{} #{number}", tr(zh, "运行", "Run")));
            }
            parts.push(
                e.plan
                    .as_ref()
                    .map_or(e.solver_implementation, |plan| plan.method)
                    .to_owned(),
            );
        }
        parts.join(" · ")
    }
    pub(super) fn open_recycle_stream(&mut self, stream: StreamId, action: egui::Id) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if !self
            .flow_plan
            .recycle
            .guard
            .as_ref()
            .is_some_and(|g| g.is_current(workspace))
        {
            return;
        }
        let (report, snapshot_id) = match workspace.flow_result() {
            FlowResultView::Current {
                snapshot: rf_ui::FlowResultRecord::Partitioned(p),
                ..
            } if p.result.streams.contains_key(&stream) => (
                p.result
                    .blocks
                    .iter()
                    .filter_map(|b| b.recycle.as_ref())
                    .find(|r| r.tear.stream_id == stream),
                Some(p.common.id.to_string()),
            ),
            FlowResultView::Failed { .. } => (
                workspace.partitioned_failure_evidence().and_then(|f| {
                    f.completed_blocks
                        .iter()
                        .filter_map(|b| b.recycle.as_ref())
                        .chain(&f.active_recycle)
                        .find(|r| r.tear.stream_id == stream && tear_is_bound(workspace, &r.tear))
                }),
                None,
            ),
            _ => return,
        };
        let Some(report) = report else { return };
        let t = &report.tear;
        let binding = format!(
            "{}.{} → {} → {}.{}",
            t.source.unit_id, t.source.port_name, stream, t.sink.unit_id, t.sink.port_name
        );
        if let Some(id) = snapshot_id {
            self.focus_result_table_stream(&id, stream.as_str());
            self.screen = StudioShellScreen::Results;
        } else {
            self.focus_browser_object(rf_ui::variable_browser::ObjectId::Stream(stream.clone()));
            self.left_sidebar_tab = StudioShellLeftSidebarTab::Project;
        }
        self.background_run_details_open = false;
        self.flow_plan.recycle.visit = Some(Visit {
            stream,
            binding,
            action,
            screen: self.screen,
            focus_row: true,
        });
    }
    pub(in crate::studio_gui_shell) fn focus_recycle_stream_row(
        &mut self,
        screen: StudioShellScreen,
        stream: &str,
        response: &egui::Response,
    ) {
        if let Some(v) = &mut self.flow_plan.recycle.visit
            && v.focus_row
            && self.screen == screen
            && v.screen == screen
            && stream == v.stream.as_str()
        {
            response.scroll_to_me(None);
            response.request_focus();
            v.focus_row = false;
        }
    }
    pub(in crate::studio_gui_shell) fn render_recycle_return(&mut self, ui: &mut egui::Ui) {
        let review = &mut self.flow_plan.recycle;
        let Some(visit) = &review.visit else { return };
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if self.screen != visit.screen
            || !review
                .guard
                .as_ref()
                .is_some_and(|g| g.is_current(workspace))
        {
            review.visit = None;
            return;
        }
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.small(&visit.binding);
        if let FlowResultView::Current {
            snapshot: rf_ui::FlowResultRecord::Partitioned(p),
            ..
        } = workspace.flow_result()
            && let Some(stream) = p.result.streams.get(&visit.stream)
        {
            ui.label(format!(
                "{} · {} · {} mol/s",
                tr(
                    zh,
                    "生产端返回总流量 (SI)",
                    "Producer returned total flow (SI)"
                ),
                stream.id,
                stream.total_molar_flow_mol_s
            ));
        }
        if ui
            .button(tr(zh, "返回回流残差", "Back to recycle residuals"))
            .clicked()
        {
            ui.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                i.consume_key(egui::Modifiers::NONE, egui::Key::Space);
            });
            review.return_focus = Some((visit.action, ui.ctx().cumulative_pass_nr()));
            review.visit = None;
            self.screen = StudioShellScreen::Run;
            ui.ctx().request_repaint();
        }
    }
}
