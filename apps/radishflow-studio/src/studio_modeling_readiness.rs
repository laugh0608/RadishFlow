use crate::modeling_specifications::{SpecificationProblem, check_inputs};
use rf_flowsheet::specifications::FeedInputField;

pub use rf_flowsheet::specifications::UnitParameter as StudioModelingUnitParameter;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioModelingFocusTarget {
    Package,
    Palette,
    Stream(String),
    Unit(String),
}

/// Legacy first-blocker view of the same facts used by the read-only A1 report.
/// Execution-stage failures remain with the solver and retain their public codes.
pub fn studio_modeling_run_blocker(
    document: &rf_ui::FlowsheetDocument,
) -> Option<StudioModelingRunBlocker> {
    let checks = check_inputs(&document.flowsheet);
    let problem = checks.first_readiness_problem()?;
    Some(blocker_from_problem(problem))
}

/// The GUI's current report is also its legacy run-readiness projection.
pub fn studio_modeling_report_run_blocker(
    report: &crate::modeling_specifications::ModelingSpecificationReport,
) -> Option<StudioModelingRunBlocker> {
    report.first_readiness_problem().map(blocker_from_problem)
}

fn blocker_from_problem(problem: &SpecificationProblem) -> StudioModelingRunBlocker {
    let (task, focus_target) = match problem {
        SpecificationProblem::EmptyFlowsheet => (
            StudioModelingReadinessTask::PlaceFirstUnit,
            StudioModelingFocusTarget::Palette,
        ),
        SpecificationProblem::MissingProjectComponents => (
            StudioModelingReadinessTask::SelectProjectComponents,
            StudioModelingFocusTarget::Package,
        ),
        SpecificationProblem::ComponentOutsideProject {
            stream_id,
            component_id,
        } => (
            StudioModelingReadinessTask::SelectReferencedProjectComponent {
                stream_id: stream_id.to_string(),
                component_id: component_id.to_string(),
            },
            StudioModelingFocusTarget::Stream(stream_id.to_string()),
        ),
        SpecificationProblem::InvalidFeedInput {
            stream_id, field, ..
        } => {
            let stream_id = stream_id.to_string();
            let task = match field {
                FeedInputField::Temperature => {
                    StudioModelingReadinessTask::FixFeedSourceTemperature {
                        stream_id: stream_id.clone(),
                    }
                }
                FeedInputField::Pressure => StudioModelingReadinessTask::FixFeedSourcePressure {
                    stream_id: stream_id.clone(),
                },
                FeedInputField::MolarFlow => StudioModelingReadinessTask::FixFeedSourceMolarFlow {
                    stream_id: stream_id.clone(),
                },
            };
            (task, StudioModelingFocusTarget::Stream(stream_id))
        }
        SpecificationProblem::MissingFeedComposition { stream_id, .. } => (
            StudioModelingReadinessTask::CommitFeedComposition {
                stream_id: stream_id.to_string(),
            },
            StudioModelingFocusTarget::Stream(stream_id.to_string()),
        ),
        SpecificationProblem::UnnormalizedComposition { stream_id, sum } => (
            StudioModelingReadinessTask::NormalizeStreamComposition {
                stream_id: stream_id.to_string(),
                sum: format!("{sum:.6}"),
            },
            StudioModelingFocusTarget::Stream(stream_id.to_string()),
        ),
        SpecificationProblem::InvalidMoleFraction { stream_id, .. }
        | SpecificationProblem::InvalidCompositionTotal { stream_id, .. } => (
            StudioModelingReadinessTask::FixStreamCompositionValues {
                stream_id: stream_id.to_string(),
            },
            StudioModelingFocusTarget::Stream(stream_id.to_string()),
        ),
        SpecificationProblem::MissingParameter { unit_id, parameter } => (
            StudioModelingReadinessTask::CommitUnitParameter {
                unit_id: unit_id.to_string(),
                parameter: *parameter,
            },
            StudioModelingFocusTarget::Unit(unit_id.to_string()),
        ),
        SpecificationProblem::Topology(_)
        | SpecificationProblem::InvalidParameter { .. }
        | SpecificationProblem::OutletPressureAboveInlet { .. } => {
            unreachable!("execution problems do not enter authoring readiness")
        }
    };
    StudioModelingRunBlocker { task, focus_target }
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
