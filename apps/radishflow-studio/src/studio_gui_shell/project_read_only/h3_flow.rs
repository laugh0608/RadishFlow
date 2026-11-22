//! Read-only presentation of the saved association and its own review task.

use super::{h3, tr};
use crate::studio_gui_shell::StudioShellLocale;
use eframe::egui;
use radishflow_studio::{
    H3FlowPreview, H3FlowReviewPhase, H3FlowReviewResult, StudioH3FlowReview, WorkspaceRunOwner,
};
use rf_model::equipment::{
    ConcentricTubeFlowAssociation, ConcentricTubeFlowStream, ConcentricTubeRatingTask,
};
use rf_types::{RfError, units::QuantityKind};
use rf_ui::engineering::EquipmentCaseKey;
use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Hot,
    Cold,
}

impl Side {
    fn label(self, l: StudioShellLocale) -> &'static str {
        match self {
            Self::Hot => tr(l, "热侧", "Hot side"),
            Self::Cold => tr(l, "冷侧", "Cold side"),
        }
    }

    fn stream(self, association: &ConcentricTubeFlowAssociation) -> &ConcentricTubeFlowStream {
        match self {
            Self::Hot => &association.hot,
            Self::Cold => &association.cold,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Run,
    Cancel,
    Activity,
    Diagnostic,
    Reference(Side),
}

pub(super) struct ReviewState {
    session: u64,
    selected: Option<EquipmentCaseKey>,
    evidence_open: bool,
    target: Option<Side>,
    focus: Option<Focus>,
    activity_focused: bool,
}

impl Default for ReviewState {
    fn default() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self {
            session: NEXT.fetch_add(1, Ordering::Relaxed),
            selected: None,
            evidence_open: false,
            target: None,
            focus: None,
            activity_focused: false,
        }
    }
}

impl ReviewState {
    pub(super) fn render(
        &mut self,
        ctx: &egui::Context,
        project: &mut StudioH3FlowReview,
        owner: &mut WorkspaceRunOwner,
        path: &Path,
        l: StudioShellLocale,
        escape: bool,
    ) -> bool {
        if escape && !self.return_to_diagnostic() {
            return true;
        }
        if self.selected.is_none() {
            self.selected = project.cases().next().map(h3::member);
        }
        let mut close = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.push_id(("h3-flow-review", self.session), |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(tr(l, "工程只读", "Read-only project"));
                    close = h3::button(ui, tr(l, "返回工作区", "Back to workspace")).clicked();
                });
                if close {
                    return;
                }
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .animated(false)
                    .show(ui, |ui| {
                        ui.heading(tr(
                            l,
                            "同心套管流程关联复核",
                            "Linked concentric-tube review",
                        ));
                        ui.label(format!(
                            "{} · {} · v4",
                            project.metadata().title,
                            path.file_name()
                                .unwrap_or(path.as_os_str())
                                .to_string_lossy()
                        ));
                        ui.add_space(8.0);
                        let multiple = project.cases().nth(1).is_some();
                        if multiple && ui.available_width() >= 860.0 {
                            let width = ui.available_width();
                            ui.horizontal_top(|ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(210.0);
                                    self.cases(ui, project, owner, l);
                                });
                                ui.vertical(|ui| {
                                    ui.set_width(width - 210.0 - ui.spacing().item_spacing.x);
                                    self.case(ui, project, owner, l);
                                });
                            });
                        } else {
                            if multiple {
                                self.cases(ui, project, owner, l);
                                ui.add_space(6.0);
                            }
                            self.case(ui, project, owner, l);
                        }
                    });
            });
        });
        close
    }

    fn cases(
        &mut self,
        ui: &mut egui::Ui,
        project: &StudioH3FlowReview,
        owner: &WorkspaceRunOwner,
        l: StudioShellLocale,
    ) {
        h3::panel(ui).show(ui, |ui| {
            ui.heading(tr(l, "关联工况", "Linked cases"));
            let cases: Vec<_> = project.cases().collect();
            let columns = if (320.0..720.0).contains(&ui.available_width()) {
                2
            } else {
                1
            };
            for row in cases.chunks(columns) {
                ui.columns(row.len(), |columns| {
                    for (ui, task) in columns.iter_mut().zip(row) {
                        let member = h3::member(task);
                        let selected = self.selected.as_ref() == Some(&member);
                        let status = project
                            .activity(owner)
                            .filter(|(active, _)| **active == member)
                            .map_or_else(
                                || result_label(l, &project.result(&member)),
                                |(_, phase)| phase_label(l, phase),
                            );
                        let label = format!("{}\n{status}", h3::case_title(task));
                        let response = ui
                            .push_id(&member, |ui| {
                                ui.add_sized(
                                    [ui.available_width(), 24.0],
                                    egui::Button::new(label).selected(selected).wrap(),
                                )
                            })
                            .inner;
                        h3::reveal_focus(&response);
                        if response.clicked() && !selected {
                            self.selected = Some(member);
                            self.target = None;
                            self.focus = None;
                            self.evidence_open = false;
                        }
                    }
                });
            }
        });
    }

    fn case(
        &mut self,
        ui: &mut egui::Ui,
        project: &mut StudioH3FlowReview,
        owner: &mut WorkspaceRunOwner,
        l: StudioShellLocale,
    ) {
        self.activity_focused = false;
        let Some(member) = self.selected.clone() else {
            return;
        };
        let Some(task) = project.case(&member) else {
            return;
        };
        let title = h3::case_title(task);
        ui.push_id(&member, |ui| {
            h3::panel(ui).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.heading(title);
                    let available = project.activity(owner).is_none() && owner.is_idle();
                    let label = egui::RichText::new(tr(
                        l,
                        "运行并复核当前工况",
                        "Run and review this case",
                    ))
                    .color(egui::Color32::WHITE);
                    let action = ui.add_enabled(
                        available,
                        egui::Button::new(label)
                            .fill(egui::Color32::from_rgb(32, 83, 163))
                            .wrap(),
                    );
                    h3::reveal_focus(&action);
                    self.apply_focus(&action, Focus::Run);
                    if action.clicked() {
                        self.target = None;
                        self.focus = None;
                        if project.start(owner, &member, SystemTime::now()) {
                            self.focus = Some(Focus::Cancel);
                        }
                    }
                });
            });
            let task = project.case(&member).expect("selected saved association");
            let association = task.case.flow_association.as_ref().expect("linked case");
            ui.add_space(6.0);
            self.references(ui, project, association, l);
            ui.add_space(6.0);
            self.activity(ui, project, owner, l);
            let task = project.case(&member).expect("selected saved association");
            h3::panel(ui).show(ui, |ui| {
                ui.heading(tr(l, "校核结果", "Rating results"));
                let active = project
                    .activity(owner)
                    .filter(|(active, _)| **active == member);
                match (active, project.result(&member)) {
                    (Some(_), _) => {
                        ui.label(tr(l, "等待本次复核结果", "Waiting for this review"));
                    }
                    (_, H3FlowReviewResult::Current { rating, .. }) => h3::results::render(
                        ui,
                        l,
                        rating,
                        &task.case.limits,
                        project.display_units(),
                    ),
                    (_, H3FlowReviewResult::Blocked(error)) => {
                        self.diagnostic(ui, task, l, error, true)
                    }
                    (_, H3FlowReviewResult::Failed(error)) => {
                        self.diagnostic(ui, task, l, error, false)
                    }
                    (_, state) => {
                        ui.label(result_label(l, &state));
                    }
                }
            });
            self.evidence(ui, project, &member, l);
        });
    }

    fn references(
        &mut self,
        ui: &mut egui::Ui,
        project: &StudioH3FlowReview,
        association: &ConcentricTubeFlowAssociation,
        l: StudioShellLocale,
    ) {
        h3::panel(ui).show(ui, |ui| {
            ui.heading(tr(l, "已保存流股关联", "Saved stream links"));
            let mut show = |ui: &mut egui::Ui, side: Side| {
                let stream = side.stream(association);
                ui.label(egui::RichText::new(side.label(l)).strong());
                let name = project
                    .stream_name(&stream.id)
                    .unwrap_or(stream.id.as_str());
                let label = if name == stream.id.as_str() {
                    name.into()
                } else {
                    format!("{name} · {}", stream.id)
                };
                let mut widget = egui::Label::new(label).wrap();
                if self.target == Some(side) {
                    widget = widget.sense(egui::Sense::focusable_noninteractive());
                }
                let response = ui.add(widget);
                self.apply_focus(&response, Focus::Reference(side));
                if self.target == Some(side) {
                    ui.weak(tr(l, "诊断关联流股", "Diagnostic stream"));
                }
            };
            if ui.available_width() >= 520.0 {
                ui.columns_const(|[hot, cold]| {
                    show(hot, Side::Hot);
                    show(cold, Side::Cold);
                });
            } else {
                show(ui, Side::Hot);
                show(ui, Side::Cold);
            }
        });
    }

    fn activity(
        &mut self,
        ui: &mut egui::Ui,
        project: &mut StudioH3FlowReview,
        owner: &mut WorkspaceRunOwner,
        l: StudioShellLocale,
    ) {
        let Some((member, phase)) = project.activity(owner) else {
            if !owner.is_idle() {
                ui.label(tr(l, "上一复核正在停止", "Previous review is stopping"));
            }
            return;
        };
        let title = project.case(member).map(h3::case_title).unwrap_or_default();
        h3::panel(ui).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spinner();
                let label = ui.add(
                    egui::Label::new(format!("{} · {title}", phase_label(l, phase)))
                        .sense(egui::Sense::focusable_noninteractive())
                        .wrap(),
                );
                self.apply_focus(&label, Focus::Activity);
                if phase == H3FlowReviewPhase::Stopping {
                    self.activity_focused = label.has_focus();
                } else {
                    let cancel = h3::button(ui, tr(l, "取消复核", "Cancel review"));
                    self.apply_focus(&cancel, Focus::Cancel);
                    self.activity_focused = cancel.has_focus() || label.has_focus();
                    if cancel.clicked() {
                        project.cancel(owner);
                        self.focus = Some(if project.activity(owner).is_some() {
                            Focus::Activity
                        } else {
                            Focus::Run
                        });
                    }
                }
            });
        });
        ui.add_space(6.0);
    }

    fn diagnostic(
        &mut self,
        ui: &mut egui::Ui,
        task: &ConcentricTubeRatingTask,
        l: StudioShellLocale,
        error: &RfError,
        blocked: bool,
    ) {
        let title = if blocked {
            tr(l, "复核受阻", "Review blocked")
        } else {
            tr(l, "复核失败", "Review failed")
        };
        ui.colored_label(egui::Color32::from_rgb(160, 48, 60), title);
        ui.label(error.message());
        if let Some(code) = error.context().diagnostic_code() {
            ui.small(code);
        }
        let association = task.case.flow_association.as_ref().expect("linked case");
        for side in [Side::Hot, Side::Cold] {
            if !error
                .context()
                .related_stream_ids()
                .contains(&side.stream(association).id)
            {
                continue;
            }
            let label = match side {
                Side::Hot => tr(l, "查看热侧流股", "Locate hot-side stream"),
                Side::Cold => tr(l, "查看冷侧流股", "Locate cold-side stream"),
            };
            let action = h3::button(ui, label);
            if self.target == Some(side) {
                self.apply_focus(&action, Focus::Diagnostic);
            }
            if action.clicked() {
                self.target = Some(side);
                self.focus = Some(Focus::Reference(side));
                ui.ctx().request_repaint();
            }
        }
        if self.focus != Some(Focus::Diagnostic)
            && self.target.is_some()
            && h3::button(ui, tr(l, "返回诊断", "Back to diagnostic")).clicked()
        {
            self.return_to_diagnostic();
            ui.ctx().request_repaint();
        }
    }

    fn evidence(
        &mut self,
        ui: &mut egui::Ui,
        project: &StudioH3FlowReview,
        member: &EquipmentCaseKey,
        l: StudioShellLocale,
    ) {
        let title = if self.evidence_open {
            tr(
                l,
                "− 物性依据与流股身份",
                "− Property evidence and stream identity",
            )
        } else {
            tr(
                l,
                "+ 物性依据与流股身份",
                "+ Property evidence and stream identity",
            )
        };
        if h3::button(ui, title).clicked() {
            self.evidence_open = !self.evidence_open;
        }
        if !self.evidence_open {
            return;
        }
        h3::panel(ui).show(ui, |ui| {
            let Some(task) = project.case(member) else {
                return;
            };
            let association = task.case.flow_association.as_ref().expect("linked case");
            let (id, revision) = project.qualification_identity();
            ui.label(format!(
                "{} · {id} · r{revision}",
                tr(l, "复核资格", "Review qualification")
            ));
            ui.label(format!(
                "{} · NIST / Gubkov et al. · 293.15–303.15 K · 50000–120000 Pa",
                tr(l, "相态依据", "Phase basis")
            ));
            ui.hyperlink(&project.qualification().phase_reference);
            for (side, properties) in [
                (Side::Hot, &task.case.hot.properties),
                (Side::Cold, &task.case.cold.properties),
            ] {
                let stream = side.stream(association);
                ui.label(format!("{} · {}", side.label(l), stream.creation_id));
                if let Some(source) = project
                    .basis()
                    .sources
                    .iter()
                    .find(|source| source.id == properties.source_id)
                {
                    ui.label(format!(
                        "{} · {} · r{}",
                        source.title, source.locator, source.revision
                    ));
                }
            }
            if let H3FlowReviewResult::Current { preview, .. } = project.result(member) {
                current_evidence(ui, project, preview, l);
            }
        });
    }

    pub(super) fn run_changed(&mut self, project: &StudioH3FlowReview, owner: &WorkspaceRunOwner) {
        if self.activity_focused && project.activity(owner).is_none() {
            self.focus = Some(Focus::Run);
        }
    }

    fn return_to_diagnostic(&mut self) -> bool {
        if self.target.is_none() {
            return false;
        }
        self.focus = Some(Focus::Diagnostic);
        true
    }

    fn apply_focus(&mut self, response: &egui::Response, target: Focus) {
        if self.focus == Some(target) {
            response.request_focus();
            response.scroll_to_me(Some(egui::Align::Center));
            self.focus = None;
            if target == Focus::Diagnostic {
                self.target = None;
            }
        }
    }
}

fn current_evidence(
    ui: &mut egui::Ui,
    project: &StudioH3FlowReview,
    preview: &H3FlowPreview,
    l: StudioShellLocale,
) {
    ui.heading(tr(l, "本次快照入口", "Inputs used from this snapshot"));
    let units = project.display_units();
    for (side, (stream, inlet)) in [(Side::Hot, preview.hot()), (Side::Cold, preview.cold())] {
        let values = [
            (inlet.temperature_k, QuantityKind::AbsoluteTemperature),
            (inlet.pressure_pa, QuantityKind::AbsolutePressure),
            (inlet.molar_flow_mol_s, QuantityKind::MolarFlow),
        ];
        let values: Vec<_> = values
            .into_iter()
            .filter_map(|(value, kind)| value.map(|value| h3::quantity(value, kind, units)))
            .collect();
        ui.label(format!(
            "{} · {} · {}",
            side.label(l),
            stream.id,
            values.join(" · ")
        ));
    }
    ui.label(format!(
        "{} · {} · #{}",
        tr(l, "来源快照", "Source snapshot"),
        preview.snapshot_id(),
        preview.sequence()
    ));
    if let Some(package) = preview
        .origin()
        .execution()
        .and_then(|e| e.package.as_ref())
    {
        ui.label(format!("{} · v{}", package.package_id, package.version));
        ui.small(&package.payload_fingerprint);
    }
}

fn phase_label(l: StudioShellLocale, phase: H3FlowReviewPhase) -> &'static str {
    match phase {
        H3FlowReviewPhase::Flow => tr(l, "正在运行流程", "Running flowsheet"),
        H3FlowReviewPhase::Rating => tr(l, "正在复核工况", "Reviewing case"),
        H3FlowReviewPhase::Stopping => tr(l, "正在停止", "Stopping"),
    }
}

fn result_label(l: StudioShellLocale, result: &H3FlowReviewResult<'_>) -> &'static str {
    match result {
        H3FlowReviewResult::Missing => tr(l, "待运行", "Ready to run"),
        H3FlowReviewResult::Stale => tr(l, "待重新复核", "Review again"),
        H3FlowReviewResult::Cancelled => tr(l, "已取消", "Cancelled"),
        H3FlowReviewResult::Blocked(_) => tr(l, "复核受阻", "Review blocked"),
        H3FlowReviewResult::Failed(_) => tr(l, "复核失败", "Review failed"),
        H3FlowReviewResult::Current { .. } => tr(l, "当前结果", "Current result"),
    }
}
