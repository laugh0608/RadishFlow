use super::*;
use radishflow_studio::modeling_recommendations::ModelingRecommendationSession;
use radishflow_studio::modeling_specifications::*;
use rf_ui::variable_browser::{ObjectId, VariableField, VariableId};

mod presentation;
use presentation::*;

pub(super) struct ModelingProblemsState {
    pub recommendations: ModelingRecommendationSession,
    pub recommendation_ui: super::modeling_recommendations::RecommendationUiState,
    selected: Option<SpecificationProblemKey>,
    destination: Option<SpecificationTarget>,
    pending_focus: bool,
    field_page: bool,
    embedded_fields: bool,
    runtime_details: bool,
    restore_selection: bool,
}

impl ModelingProblemsState {
    pub fn new(document: &rf_ui::FlowsheetDocument) -> Self {
        Self {
            recommendations: ModelingRecommendationSession::new(document),
            recommendation_ui: Default::default(),
            selected: None,
            destination: None,
            pending_focus: false,
            field_page: false,
            embedded_fields: false,
            runtime_details: false,
            restore_selection: false,
        }
    }

    pub fn is_field_page(&self) -> bool {
        self.field_page
    }

    pub fn select_problem(&mut self, key: Option<SpecificationProblemKey>) {
        self.selected = key;
    }

    pub fn checks(&self) -> &ModelingCheckSession {
        self.recommendations.checks()
    }

    pub fn replace_document(&mut self, document: &rf_ui::FlowsheetDocument) {
        self.recommendations.replace_document(document);
        self.recommendation_ui = Default::default();
        self.selected = None;
        self.destination = None;
        self.pending_focus = false;
        self.field_page = false;
        self.embedded_fields = false;
        self.runtime_details = false;
        self.restore_selection = false;
    }
}

impl ReadyAppState {
    pub(super) fn prepare_modeling_problem_field_layout(
        &mut self,
        problem_width: f32,
        body_height: f32,
    ) {
        self.modeling_problems.embedded_fields = self.bottom_drawer_tab
            == StudioShellBottomDrawerTab::Diagnostics
            && !self.modeling_problems.runtime_details
            && self.modeling_problems.field_page
            && problem_width < 700.0 * (body_height / 18.0).max(1.0);
    }

    pub(super) fn modeling_problem_fields_embedded(&self) -> bool {
        self.modeling_problems.embedded_fields
    }

    pub(super) fn modeling_problem_scroll_page(&self) -> &'static str {
        if self.bottom_drawer_tab != StudioShellBottomDrawerTab::Diagnostics {
            "standard"
        } else if self.modeling_problems.runtime_details {
            "run-diagnostics"
        } else if self.modeling_problems.field_page {
            "problem-field"
        } else {
            "problem-list"
        }
    }
    pub(super) fn refresh_modeling_problems(&mut self) {
        self.modeling_problems
            .recommendations
            .refresh(self.platform_host.modeling_inputs());
    }

    pub(super) fn open_modeling_problems(&mut self) {
        self.modeling_problems.field_page = false;
        self.modeling_problems.embedded_fields = false;
        self.modeling_problems.runtime_details = false;
        self.modeling_problems.restore_selection = true;
        self.screen = StudioShellScreen::Workbench;
        self.bottom_drawer_tab = StudioShellBottomDrawerTab::Diagnostics;
    }

    pub(super) fn render_modeling_problem_entry(&mut self, ui: &mut egui::Ui) {
        self.refresh_modeling_problems();
        let label = phase_label(self.locale, self.modeling_problems.checks());
        if ui.button(label).clicked() {
            self.open_modeling_problems();
        }
    }

    pub(super) fn render_modeling_problem_return(&mut self, ui: &mut egui::Ui) {
        if self.modeling_problems.destination.is_some()
            && ui
                .button(tr(
                    self.locale,
                    "← 返回当前问题",
                    "← Return to current problems",
                ))
                .clicked()
        {
            self.open_modeling_problems();
        }
    }

    pub(super) fn locate_specification_target(&mut self, target: SpecificationTarget) -> bool {
        if !target.exists(self.platform_host.document()) {
            return false;
        }
        let inspector = match &target {
            SpecificationTarget::Unit(id)
            | SpecificationTarget::Port(rf_types::DiagnosticPortTarget { unit_id: id, .. }) => {
                Some(rf_ui::InspectorTarget::Unit(id.clone()))
            }
            SpecificationTarget::Stream(id) | SpecificationTarget::Composition(id) => {
                Some(rf_ui::InspectorTarget::Stream(id.clone()))
            }
            SpecificationTarget::Field { object, .. } => Some(match object {
                ObjectId::Unit(id) => rf_ui::InspectorTarget::Unit(id.clone()),
                ObjectId::Stream(id) => rf_ui::InspectorTarget::Stream(id.clone()),
            }),
            _ => None,
        };
        if let Some(inspector) = inspector {
            let unit_field = matches!(
                &target,
                SpecificationTarget::Field {
                    object: ObjectId::Unit(_),
                    ..
                }
            );
            self.dispatch_ui_command(radishflow_studio::inspector_target_command_id(&inspector));
            self.screen = StudioShellScreen::Workbench;
            self.right_sidebar_tab = if unit_field {
                StudioShellRightSidebarTab::ModuleSettings
            } else {
                StudioShellRightSidebarTab::Inspector
            };
            if let Some(view) = self.current_window_id().map(rf_ui::DisplayUnitViewId) {
                self.unit_settings.closed_inspectors.remove(&view);
            }
        } else {
            match target {
                SpecificationTarget::ProjectComponents => self.screen = StudioShellScreen::Property,
                SpecificationTarget::Palette => {
                    self.screen = StudioShellScreen::Workbench;
                    self.left_sidebar_tab = StudioShellLeftSidebarTab::Palette;
                }
                _ => unreachable!(),
            }
        }
        self.modeling_problems.destination = Some(target);
        self.modeling_problems.pending_focus = true;
        self.modeling_problems.field_page = true;
        true
    }

    pub(super) fn apply_specification_field_focus(
        &mut self,
        response: &egui::Response,
        variable: &VariableId,
    ) -> bool {
        if !self.modeling_problems.pending_focus {
            return false;
        }
        let matches = self.modeling_problems.destination.as_ref().is_some_and(|target| {
            target.variable(&variable.document).as_ref() == Some(variable)
                || matches!((target, &variable.object, &variable.field), (SpecificationTarget::Composition(a), ObjectId::Stream(b), VariableField::MoleFraction(_)) if a == b)
        });
        if matches {
            response.scroll_to_me(Some(egui::Align::Center));
            response.request_focus();
            self.modeling_problems.pending_focus = false;
        }
        matches
    }

    pub(super) fn apply_specification_key_focus(&mut self, response: &egui::Response, key: &str) {
        if let Some(variable) = draft_target(key)
            .and_then(|target| target.variable(&self.platform_host.document().metadata.document_id))
        {
            self.apply_specification_field_focus(response, &variable);
        }
    }

    pub(super) fn render_specification_port(
        &mut self,
        ui: &mut egui::Ui,
        unit_id: &str,
        port: &str,
    ) {
        let selected = matches!(self.modeling_problems.destination.as_ref(), Some(SpecificationTarget::Port(target)) if target.unit_id.as_str() == unit_id && target.port_name == port);
        let response = ui.add(
            egui::Label::new(if selected {
                format!("→ {port}")
            } else {
                port.into()
            })
            .wrap()
            .sense(egui::Sense::focusable_noninteractive()),
        );
        if selected && self.modeling_problems.pending_focus {
            response.scroll_to_me(Some(egui::Align::Center));
            response.request_focus();
            self.modeling_problems.pending_focus = false;
        }
    }

    pub(super) fn focus_specification_section(
        &mut self,
        response: &egui::Response,
        target: SpecificationTarget,
    ) {
        if self.modeling_problems.pending_focus
            && self.modeling_problems.destination.as_ref() == Some(&target)
        {
            response.scroll_to_me(Some(egui::Align::Center));
            response.request_focus();
            self.modeling_problems.pending_focus = false;
        }
    }

    pub(super) fn render_modeling_problems(
        &mut self,
        ui: &mut egui::Ui,
        window: &StudioGuiWindowModel,
    ) {
        self.refresh_modeling_problems();
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(
                &mut self.modeling_problems.runtime_details,
                false,
                tr(self.locale, "当前问题", "Current problems"),
            );
            ui.selectable_value(
                &mut self.modeling_problems.runtime_details,
                true,
                tr(self.locale, "运行诊断详情", "Run diagnostic details"),
            );
        });
        if self.modeling_problems.runtime_details {
            self.render_bottom_diagnostics(ui, window);
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.strong(phase_label(self.locale, self.modeling_problems.checks()));
            if ui
                .add_enabled(
                    !matches!(
                        self.modeling_problems.checks().phase(),
                        ModelingCheckPhase::Checking
                    ),
                    egui::Button::new(tr(self.locale, "重新检查", "Recheck")),
                )
                .clicked()
            {
                self.modeling_problems
                    .recommendations
                    .recheck(self.platform_host.modeling_inputs());
            }
            self.render_modeling_draft_access(ui);
        });
        if let ModelingCheckPhase::Failed(error) = self.modeling_problems.checks().phase() {
            ui.label(format!(
                "{}: {error:?}",
                tr(
                    self.locale,
                    "检查失败；上次依据已保留",
                    "Check failed; previous evidence retained"
                )
            ));
        }
        if self.modeling_problems.checks().current_count() == Some(0) {
            ui.label(tr(
                self.locale,
                "本批已执行的局部检查未发现问题。",
                "No problems found in the local checks performed.",
            ));
        }
        let narrow = ui.available_width()
            < 700.0 * (ui.text_style_height(&egui::TextStyle::Body) / 18.0).max(1.0);
        if narrow && self.modeling_problems.destination.is_some() {
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(
                    &mut self.modeling_problems.field_page,
                    false,
                    tr(self.locale, "当前问题", "Current problems"),
                );
                ui.selectable_value(
                    &mut self.modeling_problems.field_page,
                    true,
                    tr(self.locale, "对象字段", "Object fields"),
                );
            });
        }
        if narrow && self.modeling_problems.field_page {
            // Re-project after navigation, never render the previous object's fields.
            let current = self.platform_host.snapshot().window_model();
            match &self.modeling_problems.destination {
                Some(SpecificationTarget::Field {
                    object: ObjectId::Unit(_),
                    ..
                }) => self.render_runtime_module_settings_tab(ui, &current),
                Some(SpecificationTarget::ProjectComponents | SpecificationTarget::Palette) => {
                    ui.label(tr(
                        self.locale,
                        "目标已在对应工作区打开。",
                        "The target is open in its workspace.",
                    ));
                }
                Some(_) => self.render_runtime_inspector_tab(ui, &current),
                None => (),
            }
        } else if narrow {
            self.render_specification_rows(ui);
            ui.separator();
            self.render_selected_specification(ui);
        } else {
            ui.columns(2, |columns| {
                self.render_specification_rows(&mut columns[0]);
                self.render_selected_specification(&mut columns[1]);
            });
        }
        self.render_specification_coverage(ui);
        ui.small(tr(self.locale, "检查只读已提交输入。结构自由度 / 数值可解性未分析；结果与输出仍需当前求解。", "Checks read committed inputs. Structural DOF / numerical solvability are not analyzed; results and export require a current solve."));
    }

    fn render_specification_rows(&mut self, ui: &mut egui::Ui) {
        let entries = self.modeling_problems.checks().entries().to_vec();
        let identity = self.modeling_problems.checks().identity();
        egui::ScrollArea::vertical()
            .id_salt((
                "specification-list",
                identity.document_id.as_str(),
                identity.session.0,
            ))
            .max_height(220.)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for entry in entries {
                    let selected = self.modeling_problems.selected.as_ref() == Some(&entry.key);
                    let label = format!(
                        "! {}\n{}",
                        problem_title(self.locale, &entry.problem, self.platform_host.document()),
                        entry_currency(
                            self.locale,
                            &entry,
                            self.modeling_problems.checks().phase()
                        )
                    );
                    let response = ui
                        .push_id(format!("{:?}", entry.key), |ui| {
                            ui.add(egui::Button::new(label).selected(selected).wrap())
                        })
                        .inner;
                    if selected && self.modeling_problems.restore_selection {
                        response.request_focus();
                        self.modeling_problems.restore_selection = false;
                    }
                    if response.clicked() {
                        self.modeling_problems.recommendation_ui.review = None;
                        self.modeling_problems.selected = Some(entry.key);
                    }
                }
            });
    }

    fn render_selected_specification(&mut self, ui: &mut egui::Ui) {
        self.render_recommendation_notice(ui);
        if self.render_recommendation_review(ui) {
            return;
        }
        let entry = self
            .modeling_problems
            .checks()
            .entries()
            .iter()
            .chain(self.modeling_problems.checks().changes())
            .find(|entry| Some(&entry.key) == self.modeling_problems.selected.as_ref())
            .cloned();
        let Some(entry) = entry else {
            ui.label(tr(
                self.locale,
                "选择问题查看依据；显式定位才切换对象。",
                "Select a problem to read its evidence; Locate changes the object.",
            ));
            return;
        };
        ui.strong(problem_title(
            self.locale,
            &entry.problem,
            self.platform_host.document(),
        ));
        ui.label(entry_currency(
            self.locale,
            &entry,
            self.modeling_problems.checks().phase(),
        ));
        let presentation = self
            .platform_host
            .snapshot()
            .runtime
            .workspace_document
            .presentation;
        let mut units = presentation.display_units().clone();
        for &quantity in rf_types::units::ALL_QUANTITIES {
            units
                .set_unit(
                    quantity,
                    presentation.effective_unit(
                        self.current_window_id().map(rf_ui::DisplayUnitViewId),
                        quantity,
                    ),
                )
                .expect("validated presentation units");
        }
        ui.label(problem_evidence(self.locale, &entry.problem, &units));
        for target in entry.key.targets.clone() {
            let enabled = target.exists(self.platform_host.document())
                && entry.state != SpecificationEntryState::TargetRemoved;
            let label = if enabled {
                format!(
                    "{} {}",
                    tr(self.locale, "定位", "Locate"),
                    target_label(self.locale, &target, self.platform_host.document())
                )
            } else {
                tr(
                    self.locale,
                    "定位不可用：原目标已删除",
                    "Cannot locate: original target removed",
                )
                .into()
            };
            if ui
                .add_enabled(enabled, egui::Button::new(label).wrap())
                .clicked()
            {
                ui.input_mut(|input| {
                    input.consume_key(egui::Modifiers::NONE, egui::Key::Enter);
                    input.consume_key(egui::Modifiers::NONE, egui::Key::Space);
                });
                self.locate_specification_target(target);
            }
        }
        self.render_problem_recommendations(ui, &entry.key);
        ui.collapsing(
            tr(self.locale, "技术依据 / SI", "Technical evidence / SI"),
            |ui| {
                ui.label(format!(
                    "{} v{} · r{}",
                    entry.key.rule.id(),
                    entry.key.rule_version,
                    entry.observed_revision
                ));
                ui.label(format!("{:?}", entry.problem));
            },
        );
    }

    fn render_specification_coverage(&self, ui: &mut egui::Ui) {
        let Some(report) = self.modeling_problems.checks().report() else {
            return;
        };
        for item in &report.coverage {
            if matches!(
                item.coverage,
                SpecificationCoverage::Evaluated | SpecificationCoverage::NotAnalyzed
            ) {
                continue;
            }
            ui.label(coverage_label(self.locale, item));
        }
        let resolved = self
            .modeling_problems
            .checks()
            .changes()
            .iter()
            .filter(|e| e.state == SpecificationEntryState::Resolved)
            .count();
        let removed = self.modeling_problems.checks().changes().len() - resolved;
        if resolved + removed > 0 {
            ui.small(if self.locale == StudioShellLocale::ZhCn { format!("本次变化：同范围检查已解决 {resolved} 项；目标移除 {removed} 项。") } else { format!("This check: {resolved} resolved in evaluated scopes; {removed} targets removed.") });
        }
    }

    pub(super) fn render_modeling_draft_access(&mut self, ui: &mut egui::Ui) {
        let snapshot = self.platform_host.snapshot();
        let keys = rf_ui::InputDiscardScope::All
            .pending_keys(&snapshot.runtime.workspace_document.input_edits);
        let labels = self.describe_affected_inputs(&keys);
        ui.menu_button(
            format!("{} · {}", tr(self.locale, "草稿", "Drafts"), keys.len()),
            |ui| {
                ui.label(tr(
                    self.locale,
                    "草稿未参与检查；定位保留输入。",
                    "Drafts are not checked; navigation preserves edits.",
                ));
                for (key, label) in keys.iter().zip(labels) {
                    if ui.button(label).clicked() {
                        if let Some(target) = draft_target(key) {
                            self.locate_specification_target(target);
                        }
                        ui.close_menu();
                    }
                }
            },
        );
    }
}
