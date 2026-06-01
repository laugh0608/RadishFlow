use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudioModelingRunBlocker {
    pub task: StudioModelingReadinessTask,
    pub focus_target: StudioModelingFocusTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioModelingReadinessTask {
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
        sum: String,
    },
    FixStreamCompositionValues {
        stream_id: String,
    },
    CommitUnitParameter {
        unit_id: String,
        parameter: StudioModelingUnitParameter,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioModelingUnitParameter {
    OutletTemperature,
    OutletPressure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioModelingFocusTarget {
    Package,
    Palette,
    Stream(String),
    Unit(String),
}

pub fn studio_modeling_run_blocker(
    document: &rf_ui::FlowsheetDocument,
) -> Option<StudioModelingRunBlocker> {
    let flowsheet = &document.flowsheet;

    if flowsheet.units.is_empty() {
        return Some(StudioModelingRunBlocker {
            task: StudioModelingReadinessTask::PlaceFirstUnit,
            focus_target: StudioModelingFocusTarget::Palette,
        });
    }

    if rf_flowsheet::validate_connections(flowsheet).is_err() {
        return None;
    }

    if has_unit_dependency_cycle(flowsheet) {
        return None;
    }

    if flowsheet.components.is_empty() {
        return Some(StudioModelingRunBlocker {
            task: StudioModelingReadinessTask::SelectProjectComponents,
            focus_target: StudioModelingFocusTarget::Package,
        });
    }

    if let Some((stream_id, component_id)) = first_stream_component_outside_project(flowsheet) {
        return Some(StudioModelingRunBlocker {
            task: StudioModelingReadinessTask::SelectReferencedProjectComponent {
                stream_id: stream_id.clone(),
                component_id,
            },
            focus_target: StudioModelingFocusTarget::Stream(stream_id),
        });
    }

    if let Some((stream_id, task)) = first_feed_source_stream_state_issue(flowsheet) {
        return Some(StudioModelingRunBlocker {
            task,
            focus_target: StudioModelingFocusTarget::Stream(stream_id),
        });
    }

    if let Some(stream_id) = first_feed_source_stream_missing_composition(flowsheet) {
        return Some(StudioModelingRunBlocker {
            task: StudioModelingReadinessTask::CommitFeedComposition {
                stream_id: stream_id.clone(),
            },
            focus_target: StudioModelingFocusTarget::Stream(stream_id),
        });
    }

    if let Some((stream_id, task)) = first_stream_composition_value_issue(flowsheet) {
        return Some(StudioModelingRunBlocker {
            task,
            focus_target: StudioModelingFocusTarget::Stream(stream_id),
        });
    }

    if let Some((unit_id, parameter)) = first_required_unit_parameter_missing(flowsheet) {
        return Some(StudioModelingRunBlocker {
            task: StudioModelingReadinessTask::CommitUnitParameter {
                unit_id: unit_id.clone(),
                parameter,
            },
            focus_target: StudioModelingFocusTarget::Unit(unit_id),
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
) -> Option<(String, StudioModelingReadinessTask)> {
    feed_source_streams(flowsheet).find_map(|stream| {
        let stream_id = stream.id.as_str().to_string();
        if !is_positive_finite(stream.temperature_k) {
            return Some((
                stream_id.clone(),
                StudioModelingReadinessTask::FixFeedSourceTemperature { stream_id },
            ));
        }
        if !is_positive_finite(stream.pressure_pa) {
            return Some((
                stream_id.clone(),
                StudioModelingReadinessTask::FixFeedSourcePressure { stream_id },
            ));
        }
        if !is_positive_finite(stream.total_molar_flow_mol_s) {
            return Some((
                stream_id.clone(),
                StudioModelingReadinessTask::FixFeedSourceMolarFlow { stream_id },
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
) -> Option<(String, StudioModelingReadinessTask)> {
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
                StudioModelingReadinessTask::NormalizeStreamComposition {
                    stream_id,
                    sum: format!("{sum:.6}"),
                },
            )),
            _ => Some((
                stream_id.clone(),
                StudioModelingReadinessTask::FixStreamCompositionValues { stream_id },
            )),
        }
    })
}

fn first_required_unit_parameter_missing(
    flowsheet: &rf_model::Flowsheet,
) -> Option<(String, StudioModelingUnitParameter)> {
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

fn required_unit_parameters(kind: &str) -> &'static [StudioModelingUnitParameter] {
    match kind {
        "heater" | "cooler" | "flash_drum" => &[
            StudioModelingUnitParameter::OutletTemperature,
            StudioModelingUnitParameter::OutletPressure,
        ],
        "mixer" | "valve" => &[StudioModelingUnitParameter::OutletPressure],
        _ => &[],
    }
}

fn unit_parameter_missing(
    unit: &rf_model::UnitNode,
    parameter: StudioModelingUnitParameter,
) -> bool {
    match parameter {
        StudioModelingUnitParameter::OutletTemperature => {
            unit.parameters.outlet_temperature_k.is_none()
        }
        StudioModelingUnitParameter::OutletPressure => unit.parameters.outlet_pressure_pa.is_none(),
    }
}

fn is_positive_finite(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

pub fn studio_modeling_run_blocked_title_en() -> &'static str {
    "Model inputs are not ready"
}

pub fn studio_modeling_run_blocked_title_zh() -> &'static str {
    "模型输入未完成"
}

pub fn studio_modeling_run_blocked_detail_en(task: &StudioModelingReadinessTask) -> String {
    match task {
        StudioModelingReadinessTask::PlaceFirstUnit => {
            "Place at least one unit before running the current flowsheet.".to_string()
        }
        StudioModelingReadinessTask::SelectProjectComponents => {
            "Select project components before running the current flowsheet.".to_string()
        }
        StudioModelingReadinessTask::SelectReferencedProjectComponent {
            stream_id,
            component_id,
        } => format!(
            "Stream `{stream_id}` composition references component `{component_id}` that is not selected in project components."
        ),
        StudioModelingReadinessTask::FixFeedSourceTemperature { stream_id } => format!(
            "Set feed source stream `{stream_id}` temperature to a positive finite value before running."
        ),
        StudioModelingReadinessTask::FixFeedSourcePressure { stream_id } => format!(
            "Set feed source stream `{stream_id}` pressure to a positive finite value before running."
        ),
        StudioModelingReadinessTask::FixFeedSourceMolarFlow { stream_id } => format!(
            "Set feed source stream `{stream_id}` molar flow to a positive finite value before running."
        ),
        StudioModelingReadinessTask::CommitFeedComposition { stream_id } => {
            format!("Define the feed stream `{stream_id}` overall mole fractions before running.")
        }
        StudioModelingReadinessTask::NormalizeStreamComposition { stream_id, sum } => {
            format!(
                "Stream `{stream_id}` overall mole fractions sum to {sum}, not 1.000000. Normalize composition explicitly before running; no automatic compensation is applied by the run command."
            )
        }
        StudioModelingReadinessTask::FixStreamCompositionValues { stream_id } => {
            format!("Fix stream `{stream_id}` overall mole fractions before running.")
        }
        StudioModelingReadinessTask::CommitUnitParameter { unit_id, parameter } => format!(
            "Commit unit `{unit_id}` {} before running.",
            studio_modeling_unit_parameter_label_en(*parameter)
        ),
    }
}

pub fn studio_modeling_run_blocked_detail_zh(task: &StudioModelingReadinessTask) -> String {
    match task {
        StudioModelingReadinessTask::PlaceFirstUnit => {
            "先放置至少一个单元，再运行当前流程。".to_string()
        }
        StudioModelingReadinessTask::SelectProjectComponents => {
            "先选择项目组分，再运行当前流程。".to_string()
        }
        StudioModelingReadinessTask::SelectReferencedProjectComponent {
            stream_id,
            component_id,
        } => format!(
            "流股 `{stream_id}` 的组成引用了未进入项目组分列表的 `{component_id}`，请先选择该项目组分。"
        ),
        StudioModelingReadinessTask::FixFeedSourceTemperature { stream_id } => {
            format!("先把进料源流股 `{stream_id}` 的温度设为大于 0 的有限值，再运行当前流程。")
        }
        StudioModelingReadinessTask::FixFeedSourcePressure { stream_id } => {
            format!("先把进料源流股 `{stream_id}` 的压力设为大于 0 的有限值，再运行当前流程。")
        }
        StudioModelingReadinessTask::FixFeedSourceMolarFlow { stream_id } => {
            format!("先把进料源流股 `{stream_id}` 的摩尔流量设为大于 0 的有限值，再运行当前流程。")
        }
        StudioModelingReadinessTask::CommitFeedComposition { stream_id } => {
            format!("先补齐进料流股 `{stream_id}` 的总体摩尔分率，再运行当前流程。")
        }
        StudioModelingReadinessTask::NormalizeStreamComposition { stream_id, sum } => {
            format!(
                "流股 `{stream_id}` 的总体摩尔分率合计为 {sum}，不是 1.000000；请先显式归一化组成，再运行当前流程。"
            )
        }
        StudioModelingReadinessTask::FixStreamCompositionValues { stream_id } => {
            format!("先修正流股 `{stream_id}` 的总体摩尔分率数值，再运行当前流程。")
        }
        StudioModelingReadinessTask::CommitUnitParameter { unit_id, parameter } => format!(
            "先提交单元 `{unit_id}` 的{}，再运行当前流程。",
            studio_modeling_unit_parameter_label_zh(*parameter)
        ),
    }
}

fn studio_modeling_unit_parameter_label_en(parameter: StudioModelingUnitParameter) -> &'static str {
    match parameter {
        StudioModelingUnitParameter::OutletTemperature => "outlet temperature",
        StudioModelingUnitParameter::OutletPressure => "outlet pressure",
    }
}

fn studio_modeling_unit_parameter_label_zh(parameter: StudioModelingUnitParameter) -> &'static str {
    match parameter {
        StudioModelingUnitParameter::OutletTemperature => "出口温度",
        StudioModelingUnitParameter::OutletPressure => "出口压力",
    }
}
