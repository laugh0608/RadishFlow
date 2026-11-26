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
    screen: StudioShellScreen,
    action: egui::Id,
    focus_row: bool,
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
        match workspace.flow_result() {
            FlowResultView::Current {
                snapshot: rf_ui::FlowResultRecord::Partitioned(p),
                ..
            } if p.result.streams.contains_key(&stream) => {
                let snapshot_id = p.common.id.to_string();
                self.focus_result_table_stream(&snapshot_id, stream.as_str());
                self.screen = StudioShellScreen::Results;
            }
            FlowResultView::Failed { .. }
                if workspace
                    .partitioned_failure_evidence()
                    .and_then(|f| f.active_recycle.as_ref())
                    .is_some_and(|r| r.tear.stream_id == stream)
                    && workspace.document.flowsheet.streams.contains_key(&stream) =>
            {
                self.focus_browser_object(rf_ui::variable_browser::ObjectId::Stream(
                    stream.clone(),
                ));
            }
            _ => return,
        }
        self.background_run_details_open = false;
        self.flow_plan.recycle.visit = Some(Visit {
            stream,
            action,
            screen: self.screen,
            focus_row: self.screen == StudioShellScreen::Results,
        });
    }
    pub(in crate::studio_gui_shell) fn focus_recycle_result_row(
        &mut self,
        stream: &str,
        response: &egui::Response,
    ) {
        if let Some(v) = &mut self.flow_plan.recycle.visit
            && v.focus_row
            && self.screen == v.screen
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
