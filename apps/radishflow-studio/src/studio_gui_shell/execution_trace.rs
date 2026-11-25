use super::*;
use radishflow_studio::{
    StudioRunId, WorkspaceExecutionOutcome, WorkspaceExecutionTrace, WorkspaceRunSummary,
    WorkspaceTraceDiagnostic, WorkspaceTraceRecord, modeling_specifications::SpecificationTarget,
};
use rf_ui::result_query::FlowResultView;
use rf_ui::{FlowRunDisposition, FlowRunId, FlowRunQualification, WorkspaceState};

const SPLIT_WIDTH: f32 = 720.0;

#[derive(Default)]
pub(super) struct ExecutionTraceView {
    selected_run: Option<FlowRunId>,
    displayed_run: Option<FlowRunId>,
    selected_step: Option<usize>,
    narrow_details: bool,
    focus: Option<TraceFocus>,
    return_target: Option<SpecificationTarget>,
    location: Option<TraceLocation>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TraceFocus {
    Step,
    Detail,
    Locate,
    Log,
}

struct TraceLocation {
    window: Option<StudioWindowHostId>,
    session: FlowRunQualification,
    screen: StudioShellScreen,
    drawer: StudioShellBottomDrawerTab,
    inspector: StudioShellRightSidebarTab,
    run: FlowRunId,
    step: Option<usize>,
    narrow_details: bool,
    target: SpecificationTarget,
}

impl ExecutionTraceView {
    pub(super) fn follow_latest(&mut self) {
        self.selected_run = None;
        self.displayed_run = None;
        self.selected_step = None;
        self.narrow_details = false;
        self.focus = None;
        self.return_target = None;
    }

    pub(super) fn restore_log_button(&mut self) {
        self.focus = Some(TraceFocus::Log);
    }

    fn run_index<'a>(&self, mut newest_first: impl Iterator<Item = &'a FlowRunId>) -> usize {
        newest_first
            .position(|id| self.selected_run.as_ref() == Some(id))
            .unwrap_or(0)
    }

    fn display_run(&mut self, id: &FlowRunId) {
        if self.displayed_run.as_ref() != Some(id) {
            self.displayed_run = Some(id.clone());
            self.selected_step = None;
            self.narrow_details = false;
        }
    }

    fn restore_focus(&mut self, kind: TraceFocus, response: &egui::Response) {
        if self.focus == Some(kind) {
            response.request_focus();
            response.scroll_to_me(Some(egui::Align::Center));
            self.focus = None;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StepStatus {
    Completed,
    Failed,
    Interrupted,
    NotExecuted,
    Missing,
}

impl StepStatus {
    fn label(self, locale: StudioShellLocale) -> &'static str {
        match self {
            Self::Completed => tr(locale, "已完成", "Completed"),
            Self::Failed => tr(locale, "失败", "Failed"),
            Self::Interrupted => tr(locale, "执行中断", "Interrupted"),
            Self::NotExecuted => tr(locale, "未执行", "Not executed"),
            Self::Missing => tr(locale, "记录缺失", "Record missing"),
        }
    }

    fn color(self) -> egui::Color32 {
        run_status_color(match self {
            Self::Completed => "Converged",
            Self::Failed => "Error",
            Self::Interrupted | Self::Missing => "Unconverged",
            Self::NotExecuted => "Idle",
        })
    }
}

fn tr(locale: StudioShellLocale, zh: &'static str, en: &'static str) -> &'static str {
    if locale == StudioShellLocale::ZhCn {
        zh
    } else {
        en
    }
}

fn outcome_label(locale: StudioShellLocale, outcome: WorkspaceExecutionOutcome) -> &'static str {
    match outcome {
        WorkspaceExecutionOutcome::Succeeded => tr(locale, "已完成", "Completed"),
        WorkspaceExecutionOutcome::Failed => tr(locale, "失败", "Failed"),
        WorkspaceExecutionOutcome::Cancelled => tr(locale, "已取消", "Cancelled"),
        WorkspaceExecutionOutcome::InfrastructureFailed => {
            tr(locale, "执行异常", "Execution error")
        }
    }
}

fn step_status(trace: &WorkspaceExecutionTrace, number: usize) -> StepStatus {
    if let Some(stop) = trace.stop.as_ref().filter(|stop| stop.number == number) {
        if stop.failed_sequence.is_some() {
            StepStatus::Failed
        } else {
            StepStatus::Interrupted
        }
    } else if trace
        .steps
        .get(number - 1)
        .is_some_and(|step| step.completed_sequence.is_some())
    {
        StepStatus::Completed
    } else if number > trace.last_started {
        StepStatus::NotExecuted
    } else {
        StepStatus::Missing
    }
}

fn step_unit(trace: &WorkspaceExecutionTrace, number: usize) -> Option<&rf_types::UnitId> {
    trace
        .stop
        .as_ref()
        .filter(|stop| stop.number == number)
        .and_then(|stop| stop.unit.as_ref())
        .or_else(|| {
            trace
                .steps
                .get(number - 1)
                .and_then(|step| step.unit.as_ref())
        })
        .or_else(|| trace.plan_prefix.get(number - 1).and_then(Option::as_ref))
}

fn current_result_run(workspace: &WorkspaceState) -> Option<&FlowRunId> {
    let FlowResultView::Current { origin, .. } = workspace.flow_result() else {
        return None;
    };
    origin.execution().map(|execution| &execution.run_id)
}

fn diagnostic_targets(diagnostic: &WorkspaceTraceDiagnostic) -> Vec<SpecificationTarget> {
    let mut targets = Vec::new();
    if let Some(ids) = &diagnostic.compositions {
        targets.extend(ids.iter().cloned().map(SpecificationTarget::Composition));
    }
    if let Some(ports) = &diagnostic.ports {
        targets.extend(ports.iter().cloned().map(SpecificationTarget::Port));
    }
    if let Some(streams) = &diagnostic.streams {
        targets.extend(
            streams
                .iter()
                .filter(|id| {
                    !diagnostic
                        .compositions
                        .as_ref()
                        .is_some_and(|ids| ids.contains(id))
                })
                .cloned()
                .map(SpecificationTarget::Stream),
        );
    }
    if targets.is_empty()
        && let Some(units) = &diagnostic.units
    {
        targets.extend(units.iter().cloned().map(SpecificationTarget::Unit));
    }
    targets
}

fn target_is_exact(
    run: &WorkspaceRunSummary,
    target: &SpecificationTarget,
    workspace: &WorkspaceState,
) -> bool {
    let qualification = workspace.flow_run_qualification();
    run.qualification.input.same_session(&qualification)
        && (run.qualification.input.same_input(&qualification)
            || (run.object_identities_enabled && workspace.object_identities_enabled()))
        && target.exists(&workspace.document)
}

impl ReadyAppState {
    pub(super) fn open_execution_trace(&mut self) {
        self.bottom_drawer_tab = StudioShellBottomDrawerTab::ExecutionTrace;
        self.background_run_details_open = false;
    }

    pub(super) fn render_execution_trace(&mut self, ui: &mut egui::Ui) {
        let locale = self.locale;
        let (runs, active, current_run) = {
            let workspace = &self.platform_host.modeling_inputs().workspace;
            let session = workspace.flow_run_qualification();
            let owner = self.background_runs.as_ref().map(|owner| owner.borrow());
            let runs = owner
                .as_ref()
                .map(|owner| {
                    owner
                        .history()
                        .iter()
                        .rev()
                        .filter_map(|item| {
                            let StudioRunId::Flow(id) = &item.activity.id else {
                                return None;
                            };
                            let summary = owner.flow_run_summary(id)?;
                            summary
                                .qualification
                                .input
                                .same_session(&session)
                                .then(|| (item.activity.number, summary.clone()))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let active = owner
                .as_ref()
                .and_then(|owner| owner.activity())
                .is_some_and(|activity| {
                    matches!(activity.id, StudioRunId::Flow(id)
                    if workspace.pending_flow_run() == Some(&id))
                });
            (runs, active, current_result_run(workspace).cloned())
        };
        let current_number = runs.iter().find_map(|(number, summary)| {
            (Some(&summary.run_id) == current_run.as_ref()).then_some(*number)
        });
        let mut active_selection = self
            .execution_trace
            .run_index(runs.iter().map(|(_, run)| &run.run_id));
        ui.horizontal_wrapped(|ui| {
            if !runs.is_empty() {
                let label = tr(locale, "查看运行", "Review run");
                let options = runs
                    .iter()
                    .map(|(number, run)| {
                        format!("#{number} · {}", outcome_label(locale, run.outcome))
                    })
                    .collect::<Vec<_>>();
                let choices = options
                    .iter()
                    .map(String::as_str)
                    .enumerate()
                    .collect::<Vec<_>>();
                ui.label(label);
                let selection = super::choice_selector::choice_selector(
                    ui,
                    "execution-trace-run",
                    label,
                    active_selection,
                    &choices,
                );
                if let Some(selected) = selection.inner {
                    active_selection = selected;
                    self.execution_trace.selected_run = Some(runs[selected].1.run_id.clone());
                }
            }
            if let Some(number) = current_number {
                ui.small(format!(
                    "{} #{number}",
                    tr(locale, "当前结果", "Current result")
                ));
            } else if current_run.is_none() {
                ui.small(tr(locale, "暂无当前结果", "No current result"));
            } else {
                ui.small(tr(
                    locale,
                    "当前结果见结果表",
                    "Current result is in the result table",
                ));
            }
            if runs
                .get(active_selection)
                .is_some_and(|(_, run)| Some(&run.run_id) == current_run.as_ref())
                && ui.button(tr(locale, "查看结果", "View results")).clicked()
            {
                self.screen = StudioShellScreen::Results;
                self.bottom_drawer_tab = StudioShellBottomDrawerTab::ResultsTable;
            }
            let log = ui.button(tr(locale, "查看运行日志", "View run log"));
            self.execution_trace.restore_focus(TraceFocus::Log, &log);
            if log.clicked() {
                self.show_background_run_details();
            }
        });
        if active {
            ui.small(tr(
                locale,
                "正在计算，结束后可审阅记录。",
                "Calculation in progress. The trace is available after it ends.",
            ));
        }
        let Some((display_number, run)) = runs.get(active_selection) else {
            if !active {
                ui.label(tr(locale, "本会话尚无运行。", "No runs in this session."));
            }
            return;
        };
        self.execution_trace.display_run(&run.run_id);
        ui.horizontal_wrapped(|ui| {
            ui.small(format!(
                "{} r{}",
                tr(locale, "输入修订", "Input revision"),
                run.qualification.input.revision()
            ));
            ui.label(outcome_label(locale, run.outcome));
            ui.small(match (run.outcome, run.disposition) {
                (_, FlowRunDisposition::Discarded(_)) => {
                    tr(locale, "结果未接收", "Result not accepted")
                }
                (WorkspaceExecutionOutcome::Succeeded, _) => {
                    tr(locale, "已接收结果", "Result accepted")
                }
                (WorkspaceExecutionOutcome::Cancelled, _) => {
                    tr(locale, "已接收取消回执", "Cancellation acknowledged")
                }
                _ => tr(locale, "已接收失败诊断", "Failure diagnostic accepted"),
            });
        });
        if let Some(reason) = run.cancellation {
            ui.small(super::background_run::cancellation_text(locale, reason));
        }
        let trace = match &run.trace {
            WorkspaceTraceRecord::Recorded(trace) => trace,
            WorkspaceTraceRecord::NotCollected => {
                ui.label(tr(
                    locale,
                    "此运行未采集计算过程。请从已用求解方法的报告复核。",
                    "No execution trace was collected. Review the report for the method used.",
                ));
                return;
            }
            WorkspaceTraceRecord::Unavailable => {
                ui.label(tr(
                    locale,
                    "计算过程记录缺失。",
                    "The execution trace was unavailable.",
                ));
                if let Some(code) = &run.diagnostic_code {
                    ui.monospace(code);
                }
                return;
            }
        };
        if let Some(sequence) = trace.coverage.first_omitted_sequence {
            ui.label(format!(
                "{} {sequence} · {} {}",
                tr(
                    locale,
                    "记录已截断，首个缺失事件",
                    "Trace truncated at event"
                ),
                trace.coverage.omitted_events,
                tr(locale, "条明细未保留", "details omitted")
            ));
        }
        if trace.coverage.identity_or_details_omitted {
            ui.small(tr(
                locale,
                "部分对象身份或诊断详情未保留。",
                "Some object identities or diagnostic details were omitted.",
            ));
        }
        if let Some(package) = &trace.package {
            ui.collapsing(
                tr(locale, "物性包来源", "Property package provenance"),
                |ui| {
                    render_wrapped_small(ui, &package.id);
                    render_wrapped_small(ui, &package.fingerprint);
                },
            );
        }
        let planned = trace.planned_steps.unwrap_or(0);
        if let Some(diagnostic) = &trace.run_diagnostic {
            self.render_trace_diagnostic(ui, run, None, diagnostic);
        }
        if planned == 0 {
            ui.label(if trace.planned_steps.is_none() {
                tr(
                    locale,
                    "未进入步骤求解。",
                    "No execution steps were entered.",
                )
            } else {
                tr(
                    locale,
                    "计划中没有执行步骤。",
                    "There are no steps in this plan.",
                )
            });
            return;
        }
        let selected = self
            .execution_trace
            .selected_step
            .get_or_insert_with(|| trace.stop.as_ref().map_or(1, |stop| stop.number));
        let selected_number = *selected;
        let narrow = ui.available_width() < SPLIT_WIDTH;
        ui.push_id(("execution-trace", display_number), |ui| {
            if narrow && self.execution_trace.narrow_details {
                self.render_trace_step(ui, run, trace, selected_number, true);
            } else if narrow {
                self.render_trace_steps(ui, run, trace, planned, true);
            } else {
                let width = ui.available_width();
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(width * 0.38);
                        self.render_trace_steps(ui, run, trace, planned, false);
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        self.render_trace_step(ui, run, trace, selected_number, false)
                    });
                });
            }
        });
    }

    fn render_trace_steps(
        &mut self,
        ui: &mut egui::Ui,
        run: &WorkspaceRunSummary,
        trace: &WorkspaceExecutionTrace,
        planned: usize,
        narrow: bool,
    ) {
        ui.strong(format!(
            "{} · {planned}",
            tr(self.locale, "执行步骤", "Execution steps")
        ));
        let prefix = trace.plan_prefix.len().max(trace.steps.len());
        let tail = trace
            .stop
            .as_ref()
            .map(|stop| stop.number)
            .filter(|number| *number > prefix);
        egui::ScrollArea::vertical()
            .id_salt("execution-steps")
            .max_height(202.0)
            .show_rows(
                ui,
                ui.text_style_height(&egui::TextStyle::Body) + 5.0,
                prefix,
                |ui, range| {
                    for index in range {
                        self.render_trace_step_row(ui, run, trace, index + 1, narrow);
                    }
                },
            );
        if prefix < planned {
            let end = tail.map_or(planned, |number| number - 1);
            if end > prefix {
                ui.small(format!(
                    "{}–{} · {}",
                    prefix + 1,
                    end,
                    tr(self.locale, "记录缺失", "Records missing")
                ));
            }
            if let Some(number) = tail {
                self.render_trace_step_row(ui, run, trace, number, narrow);
                if number < planned {
                    ui.small(format!(
                        "{}–{} · {}",
                        number + 1,
                        planned,
                        tr(self.locale, "记录缺失", "Records missing")
                    ));
                }
            }
        }
    }

    fn render_trace_step_row(
        &mut self,
        ui: &mut egui::Ui,
        run: &WorkspaceRunSummary,
        trace: &WorkspaceExecutionTrace,
        number: usize,
        narrow: bool,
    ) {
        let unit = step_unit(trace, number);
        let name = unit.and_then(|id| {
            let workspace = &self.platform_host.modeling_inputs().workspace;
            run.qualification
                .input
                .same_input(&workspace.flow_run_qualification())
                .then(|| workspace.document.flowsheet.units.get(id))
                .flatten()
                .map(|node| &node.name)
        });
        let object = name
            .map(String::as_str)
            .or_else(|| unit.map(rf_types::UnitId::as_str))
            .unwrap_or_else(|| tr(self.locale, "对象身份缺失", "Object identity missing"));
        let status = step_status(trace, number);
        let selected = self.execution_trace.selected_step == Some(number);
        let label = format!("{number}  {}  ·  {object}", status.label(self.locale));
        let button = egui::Button::new(egui::RichText::new(&label).color(status.color()))
            .selected(selected)
            .truncate();
        let response = ui.add(button).on_hover_text(label);
        if selected {
            self.execution_trace
                .restore_focus(TraceFocus::Step, &response);
        }
        if response.clicked() {
            self.execution_trace.selected_step = Some(number);
            self.execution_trace.narrow_details = narrow;
            self.execution_trace.focus = narrow.then_some(TraceFocus::Detail);
        }
    }

    fn render_trace_step(
        &mut self,
        ui: &mut egui::Ui,
        run: &WorkspaceRunSummary,
        trace: &WorkspaceExecutionTrace,
        number: usize,
        narrow: bool,
    ) {
        if narrow {
            let back = ui.button(tr(self.locale, "返回步骤", "Back to steps"));
            self.execution_trace
                .restore_focus(TraceFocus::Detail, &back);
            if back.clicked() || escape_on(ui, &back) {
                self.execution_trace.narrow_details = false;
                self.execution_trace.focus = Some(TraceFocus::Step);
            }
        }
        ui.strong(format!(
            "{} {number} · {}",
            tr(self.locale, "步骤", "Step"),
            step_status(trace, number).label(self.locale)
        ));
        if let Some(unit) = step_unit(trace, number) {
            render_wrapped_label(ui, unit.as_str());
        }
        let stop = trace.stop.as_ref().filter(|stop| stop.number == number);
        let step = trace.steps.get(number - 1);
        if let Some(elapsed) = stop
            .map(|stop| stop.elapsed)
            .or_else(|| step.and_then(|step| step.elapsed))
        {
            ui.small(format!(
                "{} {:.3} s",
                tr(self.locale, "用时", "Elapsed"),
                elapsed.as_secs_f64()
            ));
        }
        if let Some(streams) = step.and_then(|step| step.streams.as_ref()) {
            for (label, ids) in [
                (
                    tr(self.locale, "消耗流股", "Consumed streams"),
                    &streams.consumed,
                ),
                (
                    tr(self.locale, "产出流股", "Produced streams"),
                    &streams.produced,
                ),
            ] {
                if !ids.is_empty() {
                    let ids = ids
                        .iter()
                        .map(rf_types::StreamId::as_str)
                        .collect::<Vec<_>>()
                        .join(", ");
                    render_wrapped_small(ui, format!("{label} · {ids}"));
                }
            }
        }
        if let Some(diagnostic) = stop.and_then(|stop| stop.diagnostic.as_ref()) {
            self.render_trace_diagnostic(ui, run, Some(number), diagnostic);
        } else if self.execution_trace.focus == Some(TraceFocus::Locate) {
            self.execution_trace.focus = Some(if narrow {
                TraceFocus::Detail
            } else {
                TraceFocus::Step
            });
        }
    }

    fn render_trace_diagnostic(
        &mut self,
        ui: &mut egui::Ui,
        run: &WorkspaceRunSummary,
        step: Option<usize>,
        diagnostic: &WorkspaceTraceDiagnostic,
    ) {
        let locale = self.locale;
        let composition = diagnostic
            .compositions
            .as_ref()
            .is_some_and(|ids| !ids.is_empty());
        if composition {
            ui.strong(tr(
                locale,
                "流股组成校验失败",
                "Stream composition validation failed",
            ));
        } else if let Some(message) = &diagnostic.message {
            render_wrapped_label(ui, message);
        }
        let targets = diagnostic_targets(diagnostic);
        for target in targets {
            let label = match &target {
                SpecificationTarget::Composition(id) => format!(
                    "{} · {id}",
                    tr(locale, "定位到流股组成", "Go to stream composition")
                ),
                SpecificationTarget::Stream(id) => {
                    format!("{} · {id}", tr(locale, "定位流股", "Locate stream"))
                }
                SpecificationTarget::Port(port) => format!(
                    "{} · {}:{}",
                    tr(locale, "定位端口", "Locate port"),
                    port.unit_id,
                    port.port_name
                ),
                SpecificationTarget::Unit(id) => {
                    format!("{} · {id}", tr(locale, "定位设备", "Locate unit"))
                }
                _ => continue,
            };
            if target_is_exact(
                run,
                &target,
                &self.platform_host.modeling_inputs().workspace,
            ) {
                let response = ui.add(egui::Button::new(label).wrap());
                if self.execution_trace.return_target.as_ref() == Some(&target)
                    && self.execution_trace.focus == Some(TraceFocus::Locate)
                {
                    self.execution_trace
                        .restore_focus(TraceFocus::Locate, &response);
                    self.execution_trace.return_target = None;
                }
                if response.clicked() {
                    self.locate_trace_diagnostic(run, step, target);
                    break;
                }
                if self.execution_trace.narrow_details && escape_on(ui, &response) {
                    self.execution_trace.narrow_details = false;
                    self.execution_trace.focus = Some(TraceFocus::Step);
                }
            } else {
                render_wrapped_small(
                    ui,
                    format!(
                        "{} · {label}",
                        tr(
                            locale,
                            "原目标无法在当前输入中确认",
                            "Original target cannot be verified in the current inputs"
                        )
                    ),
                );
            }
        }
        ui.collapsing(tr(locale, "技术详情", "Technical details"), |ui| {
            if composition && let Some(message) = &diagnostic.message {
                render_wrapped_label(ui, message);
            }
            if let Some(code) = &diagnostic.code {
                render_wrapped_small(ui, code);
            }
            ui.small(diagnostic.kind.as_str());
        });
        if self.execution_trace.focus == Some(TraceFocus::Locate)
            && self.execution_trace.selected_step == step
        {
            self.execution_trace.return_target = None;
            self.execution_trace.focus = Some(if step.is_none() {
                TraceFocus::Log
            } else if self.execution_trace.narrow_details {
                TraceFocus::Detail
            } else {
                TraceFocus::Step
            });
        }
    }

    fn locate_trace_diagnostic(
        &mut self,
        run: &WorkspaceRunSummary,
        step: Option<usize>,
        target: SpecificationTarget,
    ) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        if !target_is_exact(run, &target, workspace) {
            return;
        }
        let location = TraceLocation {
            window: self.current_window_id(),
            session: workspace.flow_run_qualification(),
            screen: self.screen,
            drawer: self.bottom_drawer_tab,
            inspector: self.right_sidebar_tab,
            run: run.run_id.clone(),
            step,
            narrow_details: self.execution_trace.narrow_details,
            target: target.clone(),
        };
        if self.locate_specification_target(target) {
            self.execution_trace.location = Some(location);
            self.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
        }
    }

    pub(super) fn at_execution_trace_target(&self) -> bool {
        self.screen == StudioShellScreen::Workbench
            && self
                .execution_trace
                .location
                .as_ref()
                .is_some_and(|origin| {
                    origin.window == self.current_window_id()
                        && origin.session.same_session(
                            &self
                                .platform_host
                                .modeling_inputs()
                                .workspace
                                .flow_run_qualification(),
                        )
                })
    }

    pub(super) fn render_execution_trace_return(&mut self, ui: &mut egui::Ui) {
        if !self.at_execution_trace_target() {
            return;
        }
        let back = ui.button(tr(
            self.locale,
            "← 返回原计算步骤",
            "← Back to execution step",
        ));
        if back.clicked() || escape_on(ui, &back) {
            let origin = self
                .execution_trace
                .location
                .take()
                .expect("trace destination has an origin");
            self.modeling_problems.clear_destination();
            self.screen = origin.screen;
            self.right_sidebar_tab = origin.inspector;
            self.bottom_drawer_tab = origin.drawer;
            self.execution_trace.selected_run = Some(origin.run.clone());
            self.execution_trace.displayed_run = Some(origin.run);
            self.execution_trace.selected_step = origin.step;
            self.execution_trace.narrow_details = origin.narrow_details;
            self.execution_trace.focus = Some(TraceFocus::Locate);
            self.execution_trace.return_target = Some(origin.target);
        }
    }
}

fn escape_on(ui: &mut egui::Ui, response: &egui::Response) -> bool {
    (response.has_focus() || response.lost_focus())
        && ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn showing_old_trace_during_a_run_does_not_pin_it_after_the_worker_finishes() {
        let document = rf_ui::FlowsheetDocument::new(
            rf_model::Flowsheet::new("Trace"),
            rf_ui::DocumentMetadata::new("trace", "Trace", std::time::UNIX_EPOCH),
        );
        let mut app = rf_ui::AppState::new(document);
        let old = app.prepare_flow_run().receipt().run_id().clone();
        let new = app.prepare_flow_run().receipt().run_id().clone();
        let mut view = ExecutionTraceView::default();
        view.display_run(&old);
        view.selected_step = Some(3);
        assert_eq!(view.run_index([&new, &old].into_iter()), 0);
        view.display_run(&new);
        assert_eq!(view.selected_step, None);

        view.selected_run = Some(old.clone());
        assert_eq!(view.run_index([&new, &old].into_iter()), 1);
        view.follow_latest();
        assert_eq!(view.run_index([&new, &old].into_iter()), 0);
    }
}
