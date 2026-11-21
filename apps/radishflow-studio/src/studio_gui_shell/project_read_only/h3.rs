use super::*;
use radishflow_studio::StudioH3Review;
use rf_model::equipment::ConcentricTubeRatingTask;
use rf_types::units::{DisplayUnitSet, QuantityKind};
use rf_ui::engineering::{
    EquipmentCaseKey,
    concentric_tube::{ConcentricTubeRating, RatingRevisions, SavedRatingError},
};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

mod inputs;
mod results;

type RatingOutcome = Result<ConcentricTubeRating, SavedRatingError>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Diagnostic,
    Field(&'static str),
}

pub(super) struct ReviewState {
    session: u64,
    selected: Option<EquipmentCaseKey>,
    results: BTreeMap<EquipmentCaseKey, RatingOutcome>,
    inputs_open: bool,
    evidence_open: bool,
    diagnosis_field: Option<&'static str>,
    focus: Option<Focus>,
}

impl Default for ReviewState {
    fn default() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self {
            session: NEXT.fetch_add(1, Ordering::Relaxed),
            selected: None,
            results: BTreeMap::new(),
            inputs_open: false,
            evidence_open: false,
            diagnosis_field: None,
            focus: None,
        }
    }
}

impl ReviewState {
    pub(super) fn render(
        &mut self,
        ctx: &egui::Context,
        project: &StudioH3Review,
        path: &std::path::Path,
        l: StudioShellLocale,
        escape: bool,
    ) -> bool {
        if escape && !self.return_to_diagnostic() {
            return true;
        }
        if self.selected.is_none() {
            self.selected = project.cases().next().map(member);
        }
        let mut close = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.push_id(("h3-review", self.session), |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(tr(l, "工程只读", "Read-only project"));
                    if button(ui, tr(l, "返回工作区", "Back to workspace")).clicked() {
                        close = true;
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .animated(false)
                    .show(ui, |ui| {
                        ui.label(format!(
                            "{} · {} · v{}",
                            project.metadata().title,
                            path.file_name()
                                .unwrap_or(path.as_os_str())
                                .to_string_lossy(),
                            project.schema_version(),
                        ));
                        ui.heading(tr(
                            l,
                            "同心套管逆流校核",
                            "Concentric-tube counterflow rating",
                        ));
                        ui.label(tr(
                            l,
                            "基于已保存输入复算；结果仅在本次复核中保留。",
                            "Rate the saved inputs. Results are kept for this review only.",
                        ));
                        ui.add_space(8.0);
                        let multiple = project.cases().nth(1).is_some();
                        if multiple && ui.available_width() >= 860.0 {
                            let width = ui.available_width();
                            ui.horizontal_top(|ui| {
                                ui.vertical(|ui| {
                                    ui.set_width(210.0);
                                    self.case_picker(ui, project, l);
                                });
                                ui.vertical(|ui| {
                                    ui.set_width(width - 210.0 - ui.spacing().item_spacing.x);
                                    self.selected_case(ui, project, l);
                                });
                            });
                        } else {
                            if multiple {
                                self.case_picker(ui, project, l);
                                ui.add_space(8.0);
                            }
                            self.selected_case(ui, project, l);
                        }
                    });
            });
        });
        close
    }

    fn case_picker(&mut self, ui: &mut egui::Ui, project: &StudioH3Review, l: StudioShellLocale) {
        panel(ui).show(ui, |ui| {
            ui.heading(tr(l, "工况", "Cases"));
            let tasks: Vec<_> = project.cases().collect();
            let columns = if ui.available_width() >= 320.0 { 2 } else { 1 };
            for pair in tasks.chunks(columns) {
                ui.columns(pair.len(), |uis| {
                    for (ui, task) in uis.iter_mut().zip(pair) {
                        let id = member(task);
                        let selected = self.selected.as_ref() == Some(&id);
                        let title =
                            format!("{} · {}", case_title(task), cold_flow(project, task, l));
                        let selected_button = ui
                            .push_id(&id, |ui| {
                                let choice = egui::Button::new(title).selected(selected).wrap();
                                ui.add_sized([ui.available_width(), 24.0], choice)
                            })
                            .inner;
                        reveal_focus(&selected_button);
                        if selected_button.clicked() && !selected {
                            self.selected = Some(id);
                            self.diagnosis_field = None;
                            self.focus = None;
                            self.inputs_open = false;
                            self.evidence_open = false;
                        }
                    }
                });
            }
        });
    }

    fn selected_case(&mut self, ui: &mut egui::Ui, project: &StudioH3Review, l: StudioShellLocale) {
        let Some(id) = self.selected.clone() else {
            return;
        };
        let Some(task) = project.case(&id) else {
            return;
        };
        ui.push_id(&id, |ui| {
            panel(ui).show(ui, |ui| {
                let title = |ui: &mut egui::Ui| {
                    ui.heading(case_title(task));
                    ui.label(cold_flow(project, task, l));
                };
                let rate = |ui: &mut egui::Ui| {
                    let label = egui::RichText::new(tr(l, "复算当前工况", "Rate selected case"))
                        .color(egui::Color32::WHITE);
                    let response = ui.add(
                        egui::Button::new(label)
                            .fill(egui::Color32::from_rgb(32, 83, 163))
                            .wrap(),
                    );
                    reveal_focus(&response);
                    response
                };
                let action = if ui.available_width() >= 540.0 {
                    ui.columns_const(|[heading, action]| {
                        title(heading);
                        action
                            .allocate_ui_with_layout(
                                egui::vec2(
                                    action.available_width(),
                                    action.spacing().interact_size.y,
                                ),
                                egui::Layout::right_to_left(egui::Align::Center),
                                rate,
                            )
                            .inner
                    })
                } else {
                    title(ui);
                    rate(ui)
                };
                if action.clicked()
                    && let Some(result) = project.rate(&id, RatingRevisions::from(task))
                {
                    self.results.insert(id.clone(), result);
                    self.diagnosis_field = None;
                    self.focus = None;
                }
            });
            ui.add_space(6.0);
            panel(ui).show(ui, |ui| {
                ui.heading(tr(l, "校核结果", "Rating results"));
                match self.results.get(&id) {
                    None => {
                        ui.label(tr(l, "待复算", "Not yet rated"));
                    }
                    Some(Ok(result)) => {
                        results::render(ui, l, result, &task.case.limits, project.display_units())
                    }
                    Some(Err(error)) => {
                        let target =
                            results::diagnostic(ui, l, task, error, project.display_units());
                        if let Some(field) = target {
                            let action = button(ui, tr(l, "查看输入字段", "Show input field"));
                            self.apply_focus(&action, Focus::Diagnostic);
                            if action.clicked() {
                                self.inputs_open = true;
                                self.diagnosis_field = Some(field);
                                self.focus = Some(Focus::Field(field));
                            }
                        }
                    }
                }
            });

            let saved = if self.inputs_open {
                tr(l, "− 已保存输入", "− Saved inputs")
            } else {
                tr(l, "+ 已保存输入", "+ Saved inputs")
            };
            if button(ui, saved).clicked() {
                self.inputs_open = !self.inputs_open;
                if !self.inputs_open {
                    self.diagnosis_field = None;
                }
            }
            if self.inputs_open {
                panel(ui).show(ui, |ui| {
                    if self.diagnosis_field.is_some()
                        && button(ui, tr(l, "返回诊断", "Back to diagnostic")).clicked()
                    {
                        self.return_to_diagnostic();
                    }
                    inputs::render(self, ui, l, task, project.display_units());
                });
            }

            let evidence = if self.evidence_open {
                tr(l, "− 来源与计算依据", "− Sources and calculation details")
            } else {
                tr(l, "+ 来源与计算依据", "+ Sources and calculation details")
            };
            if button(ui, evidence).clicked() {
                self.evidence_open = !self.evidence_open;
            }
            if self.evidence_open {
                results::evidence(
                    ui,
                    l,
                    project,
                    task,
                    self.results.get(&id).and_then(|r| r.as_ref().ok()),
                );
            }
        });
    }

    fn return_to_diagnostic(&mut self) -> bool {
        if self.diagnosis_field.take().is_none() {
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
        }
    }
}

fn case_title(task: &ConcentricTubeRatingTask) -> String {
    let tag = if task.tag.trim().is_empty() {
        task.id.as_str()
    } else {
        &task.tag
    };
    let name = if task.case.name.trim().is_empty() {
        task.case.id.as_str()
    } else {
        &task.case.name
    };
    format!("{tag} · {name}")
}

fn member(task: &ConcentricTubeRatingTask) -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: task.id.clone(),
        case_id: task.case.id.clone(),
    }
}

fn cold_flow(
    project: &StudioH3Review,
    task: &ConcentricTubeRatingTask,
    l: StudioShellLocale,
) -> String {
    format!(
        "{} {}",
        tr(l, "冷侧", "Cold"),
        task.case
            .cold
            .inlet
            .molar_flow_mol_s
            .map(|v| quantity(v, QuantityKind::MolarFlow, project.display_units()))
            .unwrap_or_else(|| tr(l, "流量未填写", "flow missing").to_owned())
    )
}

fn quantity(value: f64, kind: QuantityKind, units: &DisplayUnitSet) -> String {
    rf_ui::format_quantity(value, kind, units.unit_for(kind))
        .unwrap_or_else(|error| error.to_string())
}

fn reveal_focus(response: &egui::Response) {
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
}

fn panel(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::group(ui.style())
        .inner_margin(9)
        .corner_radius(6)
        .fill(ui.visuals().extreme_bg_color)
}

fn button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let response = ui.add(egui::Button::new(label).wrap());
    reveal_focus(&response);
    response
}
