use super::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelingRunBlocker {
    task: ModelingReadinessTask,
    focus_target: ModelingFocusTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModelingReadinessTask {
    PlaceFirstUnit,
    SelectProjectComponents,
    SelectReferencedProjectComponent {
        stream_id: String,
        component_id: String,
    },
    FixFeedSourceTemperature {
        stream_id: String,
    },
    FixFeedSourcePressure {
        stream_id: String,
    },
    FixFeedSourceMolarFlow {
        stream_id: String,
    },
    CommitFeedComposition {
        stream_id: String,
    },
    NormalizeStreamComposition {
        stream_id: String,
    },
    FixStreamCompositionValues {
        stream_id: String,
    },
    CommitUnitParameter {
        unit_id: String,
        parameter: ModelingUnitParameter,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelingUnitParameter {
    OutletTemperature,
    OutletPressure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModelingFocusTarget {
    Package,
    Palette,
    Stream(String),
    Unit(String),
}

impl ReadyAppState {
    pub(super) fn intercept_modeling_readiness_shortcut_if_needed(
        &mut self,
        shortcut: &StudioGuiShortcut,
    ) -> bool {
        let Some(command_id) = modeling_readiness_run_command_from_shortcut(shortcut) else {
            return false;
        };
        self.intercept_modeling_readiness_run_if_needed(command_id)
    }

    pub(super) fn intercept_modeling_readiness_run_if_needed(&mut self, command_id: &str) -> bool {
        if !matches!(
            command_id,
            "run_panel.run_manual" | "run_panel.resume_workspace"
        ) {
            return false;
        }

        let blocker = {
            let document = self.platform_host.document();
            modeling_run_blocker(document)
        };
        let Some(blocker) = blocker else {
            return false;
        };

        self.focus_modeling_readiness_blocker(&blocker.focus_target);
        self.project_open.notice = Some(ProjectOpenNotice {
            level: ProjectOpenNoticeLevel::Warning,
            title: modeling_run_blocked_title(self.locale).to_string(),
            detail: modeling_run_blocked_detail(self.locale, &blocker.task),
        });
        self.bottom_drawer_tab = StudioShellBottomDrawerTab::Messages;
        self.platform_host.record_activity_line(format!(
            "blocked modeling run before solve: {:?}",
            blocker.task
        ));
        true
    }

    fn focus_modeling_readiness_blocker(&mut self, focus_target: &ModelingFocusTarget) {
        match focus_target {
            ModelingFocusTarget::Package => {
                self.left_sidebar_tab = StudioShellLeftSidebarTab::Project;
                self.right_sidebar_tab = StudioShellRightSidebarTab::Package;
            }
            ModelingFocusTarget::Palette => {
                self.left_sidebar_tab = StudioShellLeftSidebarTab::Palette;
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
            }
            ModelingFocusTarget::Stream(stream_id) => {
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                self.dispatch_ui_command(format!("inspector.focus_stream:{stream_id}"));
            }
            ModelingFocusTarget::Unit(unit_id) => {
                self.right_sidebar_tab = StudioShellRightSidebarTab::Inspector;
                self.dispatch_ui_command(format!("inspector.focus_unit:{unit_id}"));
            }
        }
    }
}

fn modeling_readiness_run_command_from_shortcut(
    shortcut: &StudioGuiShortcut,
) -> Option<&'static str> {
    match (shortcut.modifiers.as_slice(), shortcut.key) {
        ([], StudioGuiShortcutKey::F5) => Some("run_panel.run_manual"),
        ([StudioGuiShortcutModifier::Shift], StudioGuiShortcutKey::F5) => {
            Some("run_panel.resume_workspace")
        }
        _ => None,
    }
}

fn modeling_run_blocker(document: &rf_ui::FlowsheetDocument) -> Option<ModelingRunBlocker> {
    let flowsheet = &document.flowsheet;

    if flowsheet.units.is_empty() {
        return Some(ModelingRunBlocker {
            task: ModelingReadinessTask::PlaceFirstUnit,
            focus_target: ModelingFocusTarget::Palette,
        });
    }

    if rf_flowsheet::validate_connections(flowsheet).is_err() {
        return None;
    }

    if has_unit_dependency_cycle(flowsheet) {
        return None;
    }

    if flowsheet.components.is_empty() {
        return Some(ModelingRunBlocker {
            task: ModelingReadinessTask::SelectProjectComponents,
            focus_target: ModelingFocusTarget::Package,
        });
    }

    if let Some((stream_id, component_id)) = first_stream_component_outside_project(flowsheet) {
        return Some(ModelingRunBlocker {
            task: ModelingReadinessTask::SelectReferencedProjectComponent {
                stream_id: stream_id.clone(),
                component_id,
            },
            focus_target: ModelingFocusTarget::Stream(stream_id),
        });
    }

    if let Some((stream_id, task)) = first_feed_source_stream_state_issue(flowsheet) {
        return Some(ModelingRunBlocker {
            task,
            focus_target: ModelingFocusTarget::Stream(stream_id),
        });
    }

    if let Some(stream_id) = first_feed_source_stream_missing_composition(flowsheet) {
        return Some(ModelingRunBlocker {
            task: ModelingReadinessTask::CommitFeedComposition {
                stream_id: stream_id.clone(),
            },
            focus_target: ModelingFocusTarget::Stream(stream_id),
        });
    }

    if let Some((stream_id, task)) = first_stream_composition_value_issue(flowsheet) {
        return Some(ModelingRunBlocker {
            task,
            focus_target: ModelingFocusTarget::Stream(stream_id),
        });
    }

    if let Some((unit_id, parameter)) = first_required_unit_parameter_missing(flowsheet) {
        return Some(ModelingRunBlocker {
            task: ModelingReadinessTask::CommitUnitParameter {
                unit_id: unit_id.clone(),
                parameter,
            },
            focus_target: ModelingFocusTarget::Unit(unit_id),
        });
    }

    None
}

fn has_unit_dependency_cycle(flowsheet: &rf_model::Flowsheet) -> bool {
    let mut source_by_stream: HashMap<&str, &str> = HashMap::new();
    let mut stream_sinks = Vec::new();

    for unit in flowsheet.units.values() {
        for port in &unit.ports {
            if port.kind != rf_types::PortKind::Material {
                continue;
            }
            let Some(stream_id) = port.stream_id.as_ref() else {
                continue;
            };
            match port.direction {
                rf_types::PortDirection::Outlet => {
                    source_by_stream.insert(stream_id.as_str(), unit.id.as_str());
                }
                rf_types::PortDirection::Inlet => {
                    stream_sinks.push((stream_id.as_str(), unit.id.as_str()));
                }
            }
        }
    }

    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for (stream_id, sink_unit_id) in stream_sinks {
        if let Some(source_unit_id) = source_by_stream.get(stream_id) {
            graph.entry(*source_unit_id).or_default().push(sink_unit_id);
        }
    }

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    flowsheet
        .units
        .keys()
        .any(|unit_id| visit_unit_dependency(unit_id.as_str(), &graph, &mut visiting, &mut visited))
}

fn visit_unit_dependency<'a>(
    unit_id: &'a str,
    graph: &HashMap<&'a str, Vec<&'a str>>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
) -> bool {
    if visited.contains(unit_id) {
        return false;
    }
    if !visiting.insert(unit_id) {
        return true;
    }

    if let Some(dependencies) = graph.get(unit_id) {
        for next_unit_id in dependencies {
            if visit_unit_dependency(next_unit_id, graph, visiting, visited) {
                return true;
            }
        }
    }

    visiting.remove(unit_id);
    visited.insert(unit_id);
    false
}

fn first_stream_component_outside_project(
    flowsheet: &rf_model::Flowsheet,
) -> Option<(String, String)> {
    flowsheet.streams.values().find_map(|stream| {
        stream
            .overall_mole_fractions
            .keys()
            .find(|component_id| !flowsheet.components.contains_key(*component_id))
            .map(|component_id| {
                (
                    stream.id.as_str().to_string(),
                    component_id.as_str().to_string(),
                )
            })
    })
}

fn first_feed_source_stream_state_issue(
    flowsheet: &rf_model::Flowsheet,
) -> Option<(String, ModelingReadinessTask)> {
    feed_source_streams(flowsheet).find_map(|stream| {
        let stream_id = stream.id.as_str().to_string();
        if !is_positive_finite(stream.temperature_k) {
            return Some((
                stream_id.clone(),
                ModelingReadinessTask::FixFeedSourceTemperature { stream_id },
            ));
        }
        if !is_positive_finite(stream.pressure_pa) {
            return Some((
                stream_id.clone(),
                ModelingReadinessTask::FixFeedSourcePressure { stream_id },
            ));
        }
        if !is_positive_finite(stream.total_molar_flow_mol_s) {
            return Some((
                stream_id.clone(),
                ModelingReadinessTask::FixFeedSourceMolarFlow { stream_id },
            ));
        }
        None
    })
}

fn first_feed_source_stream_missing_composition(flowsheet: &rf_model::Flowsheet) -> Option<String> {
    feed_source_streams(flowsheet).find_map(|stream| {
        stream
            .overall_mole_fractions
            .is_empty()
            .then(|| stream.id.as_str().to_string())
    })
}

fn first_stream_composition_value_issue(
    flowsheet: &rf_model::Flowsheet,
) -> Option<(String, ModelingReadinessTask)> {
    flowsheet.streams.values().find_map(|stream| {
        if stream.overall_mole_fractions.is_empty() {
            return None;
        }

        let stream_id = stream.id.as_str().to_string();
        let sum = stream
            .overall_mole_fractions
            .values()
            .try_fold(0.0, |sum, value| {
                (value.is_finite() && (0.0..=1.0).contains(value)).then_some(sum + value)
            });

        match sum {
            Some(sum) if (sum - 1.0).abs() <= 1e-9 => None,
            Some(sum) if sum.is_finite() && sum > 0.0 => Some((
                stream_id.clone(),
                ModelingReadinessTask::NormalizeStreamComposition { stream_id },
            )),
            _ => Some((
                stream_id.clone(),
                ModelingReadinessTask::FixStreamCompositionValues { stream_id },
            )),
        }
    })
}

fn first_required_unit_parameter_missing(
    flowsheet: &rf_model::Flowsheet,
) -> Option<(String, ModelingUnitParameter)> {
    flowsheet.units.values().find_map(|unit| {
        required_unit_parameters(unit.kind.as_str())
            .iter()
            .copied()
            .find(|parameter| unit_parameter_missing(unit, *parameter))
            .map(|parameter| (unit.id.as_str().to_string(), parameter))
    })
}

fn feed_source_streams<'a>(
    flowsheet: &'a rf_model::Flowsheet,
) -> impl Iterator<Item = &'a rf_model::MaterialStreamState> + 'a {
    flowsheet
        .units
        .values()
        .filter(|unit| unit.kind == "feed")
        .flat_map(|unit| {
            unit.ports.iter().filter_map(|port| {
                if port.kind != rf_types::PortKind::Material
                    || port.direction != rf_types::PortDirection::Outlet
                {
                    return None;
                }
                let stream_id = port.stream_id.as_ref()?;
                flowsheet.streams.get(stream_id)
            })
        })
}

fn required_unit_parameters(kind: &str) -> &'static [ModelingUnitParameter] {
    match kind {
        "heater" | "cooler" | "flash_drum" => &[
            ModelingUnitParameter::OutletTemperature,
            ModelingUnitParameter::OutletPressure,
        ],
        "mixer" | "valve" => &[ModelingUnitParameter::OutletPressure],
        _ => &[],
    }
}

fn unit_parameter_missing(unit: &rf_model::UnitNode, parameter: ModelingUnitParameter) -> bool {
    match parameter {
        ModelingUnitParameter::OutletTemperature => unit.parameters.outlet_temperature_k.is_none(),
        ModelingUnitParameter::OutletPressure => unit.parameters.outlet_pressure_pa.is_none(),
    }
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn modeling_run_blocked_title(locale: StudioShellLocale) -> &'static str {
    match locale {
        StudioShellLocale::En => "Model inputs are not ready",
        StudioShellLocale::ZhCn => "模型输入未完成",
    }
}

fn modeling_run_blocked_detail(locale: StudioShellLocale, task: &ModelingReadinessTask) -> String {
    match locale {
        StudioShellLocale::En => modeling_run_blocked_detail_en(task),
        StudioShellLocale::ZhCn => modeling_run_blocked_detail_zh(task),
    }
}

fn modeling_run_blocked_detail_en(task: &ModelingReadinessTask) -> String {
    match task {
        ModelingReadinessTask::PlaceFirstUnit => {
            "Place at least one unit before running the current flowsheet.".to_string()
        }
        ModelingReadinessTask::SelectProjectComponents => {
            "Select project components before running the current flowsheet.".to_string()
        }
        ModelingReadinessTask::SelectReferencedProjectComponent {
            stream_id,
            component_id,
        } => format!(
            "Stream `{stream_id}` references component `{component_id}` that is not selected in the project."
        ),
        ModelingReadinessTask::FixFeedSourceTemperature { stream_id } => format!(
            "Set feed source stream `{stream_id}` temperature to a positive finite value before running."
        ),
        ModelingReadinessTask::FixFeedSourcePressure { stream_id } => format!(
            "Set feed source stream `{stream_id}` pressure to a positive finite value before running."
        ),
        ModelingReadinessTask::FixFeedSourceMolarFlow { stream_id } => format!(
            "Set feed source stream `{stream_id}` molar flow to a positive finite value before running."
        ),
        ModelingReadinessTask::CommitFeedComposition { stream_id } => {
            format!("Define the feed stream `{stream_id}` overall mole fractions before running.")
        }
        ModelingReadinessTask::NormalizeStreamComposition { stream_id } => {
            format!("Normalize stream `{stream_id}` overall mole fractions before running.")
        }
        ModelingReadinessTask::FixStreamCompositionValues { stream_id } => {
            format!("Fix stream `{stream_id}` overall mole fractions before running.")
        }
        ModelingReadinessTask::CommitUnitParameter { unit_id, parameter } => format!(
            "Commit unit `{unit_id}` {} before running.",
            modeling_unit_parameter_label_en(*parameter)
        ),
    }
}

fn modeling_run_blocked_detail_zh(task: &ModelingReadinessTask) -> String {
    match task {
        ModelingReadinessTask::PlaceFirstUnit => "先放置至少一个单元，再运行当前流程。".to_string(),
        ModelingReadinessTask::SelectProjectComponents => {
            "先选择项目组分，再运行当前流程。".to_string()
        }
        ModelingReadinessTask::SelectReferencedProjectComponent {
            stream_id,
            component_id,
        } => format!(
            "流股 `{stream_id}` 的组成引用了未进入项目组分列表的 `{component_id}`，请先选择该项目组分。"
        ),
        ModelingReadinessTask::FixFeedSourceTemperature { stream_id } => {
            format!("先把进料源流股 `{stream_id}` 的温度设为大于 0 的有限值，再运行当前流程。")
        }
        ModelingReadinessTask::FixFeedSourcePressure { stream_id } => {
            format!("先把进料源流股 `{stream_id}` 的压力设为大于 0 的有限值，再运行当前流程。")
        }
        ModelingReadinessTask::FixFeedSourceMolarFlow { stream_id } => {
            format!("先把进料源流股 `{stream_id}` 的摩尔流量设为大于 0 的有限值，再运行当前流程。")
        }
        ModelingReadinessTask::CommitFeedComposition { stream_id } => {
            format!("先补齐进料流股 `{stream_id}` 的总体摩尔分率，再运行当前流程。")
        }
        ModelingReadinessTask::NormalizeStreamComposition { stream_id } => {
            format!("先归一化流股 `{stream_id}` 的总体摩尔分率，再运行当前流程。")
        }
        ModelingReadinessTask::FixStreamCompositionValues { stream_id } => {
            format!("先修正流股 `{stream_id}` 的总体摩尔分率数值，再运行当前流程。")
        }
        ModelingReadinessTask::CommitUnitParameter { unit_id, parameter } => format!(
            "先提交单元 `{unit_id}` 的{}，再运行当前流程。",
            modeling_unit_parameter_label_zh(*parameter)
        ),
    }
}

fn modeling_unit_parameter_label_en(parameter: ModelingUnitParameter) -> &'static str {
    match parameter {
        ModelingUnitParameter::OutletTemperature => "outlet temperature",
        ModelingUnitParameter::OutletPressure => "outlet pressure",
    }
}

fn modeling_unit_parameter_label_zh(parameter: ModelingUnitParameter) -> &'static str {
    match parameter {
        ModelingUnitParameter::OutletTemperature => "出口温度",
        ModelingUnitParameter::OutletPressure => "出口压力",
    }
}
