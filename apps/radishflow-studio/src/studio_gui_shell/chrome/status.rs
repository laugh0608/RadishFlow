use super::*;
use rf_ui::flow_plan::FlowPlanConfiguration;

impl ReadyAppState {
    pub(super) fn render_context_toolbar_statuses(
        &self,
        ui: &mut egui::Ui,
        toolbar: &radishflow_studio::StudioGuiWindowContextToolbarModel,
    ) {
        if toolbar.status_items.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            for status in &toolbar.status_items {
                let label = format!(
                    "{}: {}",
                    self.locale.runtime_label(status.label),
                    self.locale.runtime_label(&status.value)
                );
                render_status_group(
                    ui,
                    egui::RichText::new(label),
                    self.locale.runtime_label(&status.status_label).as_ref(),
                    context_toolbar_status_color(&status.status_label),
                );
            }
        });
    }

    pub(in crate::studio_gui_shell) fn render_bottom_status_bar(
        &mut self,
        ctx: &egui::Context,
        window: &StudioGuiWindowModel,
    ) {
        egui::TopBottomPanel::bottom("studio.status_bar")
            .min_height(30.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.render_modeling_draft_access(ui);
                    ui.separator();
                    let run =
                        egui::RichText::new(self.locale.runtime_label("Run").as_ref()).strong();
                    if let Some(metric) = window
                        .status_summary
                        .metrics
                        .iter()
                        .find(|metric| metric.label == "Run")
                    {
                        render_status_group(
                            ui,
                            run,
                            self.locale.runtime_label(&metric.status_label).as_ref(),
                            run_status_color(&metric.status_label),
                        );
                    } else {
                        ui.small(run);
                    }
                    ui.separator();
                    let consistency = window.status_summary.snapshot_consistency_label;
                    render_status_group(
                        ui,
                        egui::RichText::new(self.locale.runtime_label("Snapshot").as_ref())
                            .strong(),
                        self.locale.runtime_label(consistency).as_ref(),
                        context_toolbar_status_color(consistency),
                    );
                    for label in [
                        unit_settings::project_units_label(
                            self.locale,
                            window
                                .runtime
                                .workspace_document
                                .presentation
                                .display_units(),
                        ),
                        self.solver_status_label(),
                        self.locale.text(ShellText::FlowsheetMode),
                    ] {
                        ui.separator();
                        render_status_label(ui, label);
                    }
                    if let Some(selection) = window.canvas.widget.view().current_selection.as_ref()
                    {
                        ui.separator();
                        render_status_label(
                            ui,
                            self.locale
                                .selected_target(selection.kind_label, &selection.target_id),
                        );
                    }
                });
            });
    }

    fn solver_status_label(&self) -> &'static str {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let (cn, en) = match self
            .platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .current()
        {
            FlowPlanConfiguration::Partitioned(_) => (
                "求解器: 分块回流（固定温压）",
                "Solver: Partitioned recycle (fixed T/P)",
            ),
            FlowPlanConfiguration::Simultaneous(_) => (
                "求解器: 固定温压仿射联立模块",
                "Solver: Fixed-T/P affine modular",
            ),
            FlowPlanConfiguration::EquationOriented(_) => (
                "求解器: 固定温压物料方程",
                "Solver: Fixed-T/P material equations",
            ),
            FlowPlanConfiguration::Mixed(_) => (
                "求解器: 固定温压混合分区",
                "Solver: Fixed-T/P mixed material",
            ),
            FlowPlanConfiguration::Acyclic => {
                return self.locale.text(ShellText::SolverSequentialModular);
            }
            FlowPlanConfiguration::SinglePhase(_) => ("单相显热序贯", "Single-vapor sensible heat"),
            FlowPlanConfiguration::SinglePhaseRecycle(_) => {
                ("单相显热回流", "Single-vapor sensible recycle")
            }
        };
        if zh { cn } else { en }
    }
}

fn render_status_label(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>) {
    let galley = label.into().into_galley(
        ui,
        Some(egui::TextWrapMode::Wrap),
        ui.available_width(),
        egui::TextStyle::Small,
    );
    ui.label(galley);
}

fn render_status_group(
    ui: &mut egui::Ui,
    label: egui::RichText,
    status: &str,
    color: egui::Color32,
) {
    let frame = status_chip_frame(color);
    let chip = egui::WidgetText::from(egui::RichText::new(status).color(color)).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::TextStyle::Small,
    );
    let chip_size = chip.size() + frame.total_margin().sum();
    let trailing_width = chip_size.x + ui.spacing().item_spacing.x;
    // Reserve the full group before the parent wraps the row.
    let label = egui::WidgetText::from(label).into_galley(
        ui,
        Some(egui::TextWrapMode::Wrap),
        (ui.available_width() - trailing_width).max(0.0),
        egui::TextStyle::Small,
    );
    let size = egui::vec2(
        label.size().x + trailing_width,
        label.size().y.max(chip_size.y),
    );
    ui.allocate_ui_with_layout(
        size,
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.label(label);
            frame.show(ui, |ui| ui.label(chip));
        },
    );
}
