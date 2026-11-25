use super::*;
use radishflow_studio::outlet_temperature::{
    MaterialOutlet, TemperatureCondition, TemperatureObservation, TemperatureReading,
    TemperatureThreshold, ThresholdError, UnavailableTemperature,
};
use rf_types::units::MeasurementUnit;
use rf_ui::FlowExecutionQualification;

pub(super) const START: &str = "run_panel.run_with_outlet_temperature";

#[derive(Default, PartialEq)]
enum Focus {
    #[default]
    None,
    Outlet,
    Threshold,
}

pub(super) struct TemperatureWatchDraft {
    qualification: Option<FlowExecutionQualification>,
    selected: Option<MaterialOutlet>,
    raw: String,
    unit: MeasurementUnit,
    // Preserve the exact canonical value across display-unit changes.
    converted: Option<TemperatureThreshold>,
    focus: Focus,
    outlet_error: bool,
}

impl Default for TemperatureWatchDraft {
    fn default() -> Self {
        Self {
            qualification: None,
            selected: None,
            raw: "38.85".into(),
            unit: MeasurementUnit::Celsius,
            converted: None,
            focus: Focus::None,
            outlet_error: false,
        }
    }
}
impl TemperatureWatchDraft {
    fn threshold(&self) -> Result<TemperatureThreshold, ThresholdError> {
        self.converted
            .clone()
            .map_or_else(|| TemperatureThreshold::parse(&self.raw, self.unit), Ok)
    }
}

fn tr(zh: bool, cn: &'static str, en: &'static str) -> &'static str {
    if zh { cn } else { en }
}

pub(super) fn observation_text(watch: &TemperatureObservation, terminal: bool, zh: bool) -> String {
    let threshold = &watch.condition.threshold;
    let outlet = &watch.condition.outlet;
    let label = tr(zh, "出口温度", "Outlet temperature");
    let limit = tr(zh, "阈值", "threshold");
    let raw = threshold.raw();
    let unit = threshold.unit().definition().symbol;
    let canonical = threshold.kelvin().value();
    let input = if raw.trim().parse::<f64>().is_ok() {
        format!("{raw} {unit}")
    } else {
        raw.to_owned()
    };
    let canonical_label = format!("{canonical} K");
    let limit_value = if input == canonical_label {
        input
    } else {
        format!("{input} ({canonical_label})")
    };
    let source = &outlet.source;
    let evidence = match watch.reading {
        TemperatureReading::Pending if terminal => {
            tr(zh, "本次未求值", "Not evaluated this run").into()
        }
        TemperatureReading::Pending => {
            tr(zh, "等待生产单元完成", "Waiting for the producing unit").into()
        }
        TemperatureReading::Unavailable { step, reason } => {
            let reason = match reason {
                UnavailableTemperature::MissingStream => tr(
                    zh,
                    "完成步缺少目标流股",
                    "Completed step omitted the target stream",
                ),
                UnavailableTemperature::InvalidTemperature => {
                    tr(zh, "实测温度无效", "Invalid measured temperature")
                }
                UnavailableTemperature::TargetNotReached => tr(
                    zh,
                    "完成前未取得目标步读数",
                    "No target reading before completion",
                ),
            };
            let position = step
                .map(|s| format!("{s}/{} · ", watch.total))
                .unwrap_or_default();
            format!("{position}{reason}")
        }
        TemperatureReading::Measured { step, actual } => {
            let actual = actual.value();
            let judgement = if actual > canonical {
                if step == watch.total {
                    tr(zh, "末步命中", "Matched at final step")
                } else {
                    tr(zh, "命中", "Matched")
                }
            } else if actual == canonical {
                tr(zh, "等于阈值，未命中", "Equal to threshold, no match")
            } else {
                tr(zh, "低于阈值，未命中", "Below threshold, no match")
            };
            format!("{step}/{} · {actual} K · {judgement}", watch.total)
        }
    };
    format!(
        "{label} · {}.{} → {} · {limit} {limit_value} · {evidence}",
        source.unit_id, source.port, outlet.stream
    )
}

impl ReadyAppState {
    pub(super) fn render_temperature_watch(&mut self, ui: &mut egui::Ui) {
        let zh = self.locale == StudioShellLocale::ZhCn;
        let can_start = self.can_start_background_flow();
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let expected = workspace.flow_execution_qualification();
        let outlets: Vec<_> = MaterialOutlet::connected(&workspace.document.flowsheet).collect();
        let mut labels = vec![tr(zh, "选择物料出口", "Select material outlet").to_string()];
        labels.extend(
            outlets
                .iter()
                .map(|o| format!("{}.{} → {}", o.source.unit_id, o.source.port, o.stream)),
        );
        let choices: Vec<_> = labels.iter().map(String::as_str).enumerate().collect();
        let draft = &mut self.temperature_watch;
        if draft
            .qualification
            .as_ref()
            .is_none_or(|q| !q.input.same_session(&expected.input))
        {
            *draft = TemperatureWatchDraft {
                qualification: Some(expected.clone()),
                ..Default::default()
            };
        }
        let index = draft
            .selected
            .as_ref()
            .filter(|_| {
                draft
                    .qualification
                    .as_ref()
                    .is_some_and(|q| q.same_input(&expected))
            })
            .and_then(|outlet| outlets.iter().position(|other| other == outlet))
            .map_or(0, |i| i + 1);
        ui.label(tr(
            zh,
            "出口温度高于阈值时，在下一单元前暂停",
            "Pause before the next unit if outlet temperature exceeds the threshold",
        ));
        let target = tr(zh, "物料出口", "Material outlet");
        let picker = super::choice_selector::choice_selector(
            ui,
            "temperature-watch-outlet",
            target,
            index,
            &choices,
        );
        if draft.focus == Focus::Outlet {
            picker.response.request_focus();
            picker.response.scroll_to_me(None);
            draft.focus = Focus::None;
        }
        if let Some(selected) = picker.inner {
            draft.selected = selected.checked_sub(1).map(|i| outlets[i].clone());
            draft.qualification = Some(expected);
            draft.outlet_error = false;
        } else if index == 0 && (draft.outlet_error || draft.selected.is_some()) {
            ui.small(tr(
                zh,
                "请重新选择当前工程中已连接的物料出口。",
                "Select a connected material outlet for the current inputs.",
            ));
        }
        ui.horizontal_wrapped(|ui| {
            let label = ui.label(tr(zh, "温度阈值", "Temperature threshold"));
            let field = ui
                .add(egui::TextEdit::singleline(&mut draft.raw).desired_width(104.))
                .labelled_by(label.id);
            if field.changed() {
                draft.converted = None;
            }
            let picker = super::choice_selector::choice_selector(
                ui,
                "temperature-watch-unit",
                tr(zh, "输入单位", "Input unit"),
                draft.unit,
                &[
                    (MeasurementUnit::Celsius, "°C"),
                    (MeasurementUnit::Kelvin, "K"),
                ],
            );
            if let Some(unit) = picker.inner
                && unit != draft.unit
            {
                match draft.threshold().and_then(|t| t.in_unit(unit)) {
                    Ok(converted) => {
                        draft.raw = converted.raw().into();
                        draft.unit = unit;
                        draft.converted = Some(converted);
                    }
                    Err(_) => draft.focus = Focus::Threshold,
                }
            }
            if draft.focus == Focus::Threshold {
                field.request_focus();
                field.scroll_to_me(None);
                draft.focus = Focus::None;
            }
        });
        match draft.threshold() {
            Ok(value) => {
                ui.small(format!(
                    "{} {} K",
                    tr(zh, "规范阈值", "Canonical threshold"),
                    value.kelvin().value()
                ));
            }
            Err(error) => {
                ui.small(match error {
                    ThresholdError::Incomplete => {
                        tr(zh, "请完成阈值输入。", "Finish entering the threshold.")
                    }
                    ThresholdError::Invalid => tr(
                        zh,
                        "请输入与所选单位一致的有限绝对温度。",
                        "Enter a finite absolute temperature in the selected unit.",
                    ),
                    ThresholdError::AbsoluteZero => tr(
                        zh,
                        "绝对温度必须大于 0 K。",
                        "Absolute temperature must exceed 0 K.",
                    ),
                });
            }
        }
        if ui
            .add_enabled(
                can_start,
                egui::Button::new(tr(zh, "按此条件运行", "Run with this condition")),
            )
            .clicked()
        {
            self.dispatch_ui_command(START);
            ui.ctx().request_repaint();
        }
    }

    pub(super) fn start_temperature_watch(&mut self) {
        let workspace = &self.platform_host.modeling_inputs().workspace;
        let draft = &mut self.temperature_watch;
        let (Some(expected), Some(outlet)) = (&draft.qualification, &draft.selected) else {
            draft.outlet_error = true;
            draft.focus = Focus::Outlet;
            return;
        };
        let Ok(threshold) = draft.threshold() else {
            draft.focus = Focus::Threshold;
            return;
        };
        let condition = TemperatureCondition {
            expected: expected.clone(),
            outlet: outlet.clone(),
            threshold,
        };
        if !condition.is_current(workspace) {
            draft.outlet_error = true;
            draft.focus = Focus::Outlet;
            return;
        }
        let access = if self.read_only_project.is_some() {
            radishflow_studio::run_actions::StudioRunAccess::ReadOnly
        } else {
            radishflow_studio::run_actions::StudioRunAccess::Allowed
        };
        self.background_action(
            radishflow_studio::StudioBackgroundRunAction::StartWithTemperatureCondition {
                expected: radishflow_studio::flow_actions::FlowEditContext::capture(workspace),
                access,
                condition,
            },
        );
    }
    pub(super) fn review_temperature_condition(&mut self) {
        self.screen = StudioShellScreen::Run;
        self.temperature_watch.focus = Focus::Outlet;
    }
}
