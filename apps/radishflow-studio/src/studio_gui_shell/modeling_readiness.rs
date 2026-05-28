use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelingRunBlocker {
    task: ModelingReadinessTask,
    focus_target: ModelingFocusTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModelingReadinessTask {
    PlaceFirstUnit,
    ConnectMaterialPort {
        unit_id: String,
        port_name: String,
    },
    RestoreMaterialStreamReference {
        unit_id: String,
        port_name: String,
        stream_id: String,
    },
    SelectProjectComponents,
    SelectReferencedProjectComponent {
        stream_id: String,
        component_id: String,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModelingFocusTarget {
    Package,
    Palette,
    Stream(String),
    Unit(String),
}

impl ReadyAppState {
    pub(super) fn intercept_modeling_readiness_run_if_needed(&mut self, command_id: &str) -> bool {
        if command_id != "run_panel.run_manual" {
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

fn modeling_run_blocker(document: &rf_ui::FlowsheetDocument) -> Option<ModelingRunBlocker> {
    let flowsheet = &document.flowsheet;

    if flowsheet.units.is_empty() {
        return Some(ModelingRunBlocker {
            task: ModelingReadinessTask::PlaceFirstUnit,
            focus_target: ModelingFocusTarget::Palette,
        });
    }

    if let Some((unit_id, port_name)) = first_unbound_material_port(flowsheet) {
        return Some(ModelingRunBlocker {
            task: ModelingReadinessTask::ConnectMaterialPort {
                unit_id: unit_id.clone(),
                port_name,
            },
            focus_target: ModelingFocusTarget::Unit(unit_id),
        });
    }

    if let Some((unit_id, port_name, stream_id)) =
        first_missing_material_stream_reference(flowsheet)
    {
        return Some(ModelingRunBlocker {
            task: ModelingReadinessTask::RestoreMaterialStreamReference {
                unit_id: unit_id.clone(),
                port_name,
                stream_id,
            },
            focus_target: ModelingFocusTarget::Unit(unit_id),
        });
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

    None
}

fn first_unbound_material_port(flowsheet: &rf_model::Flowsheet) -> Option<(String, String)> {
    flowsheet.units.values().find_map(|unit| {
        unit.ports
            .iter()
            .find(|port| port.kind == rf_types::PortKind::Material && port.stream_id.is_none())
            .map(|port| (unit.id.as_str().to_string(), port.name.clone()))
    })
}

fn first_missing_material_stream_reference(
    flowsheet: &rf_model::Flowsheet,
) -> Option<(String, String, String)> {
    flowsheet.units.values().find_map(|unit| {
        unit.ports
            .iter()
            .filter(|port| port.kind == rf_types::PortKind::Material)
            .find_map(|port| {
                let stream_id = port.stream_id.as_ref()?;
                (!flowsheet.streams.contains_key(stream_id)).then(|| {
                    (
                        unit.id.as_str().to_string(),
                        port.name.clone(),
                        stream_id.as_str().to_string(),
                    )
                })
            })
    })
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

fn first_feed_source_stream_missing_composition(flowsheet: &rf_model::Flowsheet) -> Option<String> {
    flowsheet.units.values().find_map(|unit| {
        if unit.kind != "feed" {
            return None;
        }
        unit.ports
            .iter()
            .filter(|port| {
                port.kind == rf_types::PortKind::Material
                    && port.direction == rf_types::PortDirection::Outlet
            })
            .find_map(|port| {
                let stream_id = port.stream_id.as_ref()?;
                let stream = flowsheet.streams.get(stream_id)?;
                stream
                    .overall_mole_fractions
                    .is_empty()
                    .then(|| stream_id.as_str().to_string())
            })
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
        ModelingReadinessTask::ConnectMaterialPort { unit_id, port_name } => format!(
            "Connect material port `{unit_id}.{port_name}` before running the current flowsheet."
        ),
        ModelingReadinessTask::RestoreMaterialStreamReference {
            unit_id,
            port_name,
            stream_id,
        } => format!(
            "Port `{unit_id}.{port_name}` references missing stream `{stream_id}`. Repair that connection before running."
        ),
        ModelingReadinessTask::SelectProjectComponents => {
            "Select project components before running the current flowsheet.".to_string()
        }
        ModelingReadinessTask::SelectReferencedProjectComponent {
            stream_id,
            component_id,
        } => format!(
            "Stream `{stream_id}` references component `{component_id}` that is not selected in the project."
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
    }
}

fn modeling_run_blocked_detail_zh(task: &ModelingReadinessTask) -> String {
    match task {
        ModelingReadinessTask::PlaceFirstUnit => "先放置至少一个单元，再运行当前流程。".to_string(),
        ModelingReadinessTask::ConnectMaterialPort { unit_id, port_name } => {
            format!("先连接单元 `{unit_id}` 的 material 端口 `{port_name}`，再运行当前流程。")
        }
        ModelingReadinessTask::RestoreMaterialStreamReference {
            unit_id,
            port_name,
            stream_id,
        } => format!(
            "单元 `{unit_id}` 的端口 `{port_name}` 引用了缺失流股 `{stream_id}`，请先修复连接。"
        ),
        ModelingReadinessTask::SelectProjectComponents => {
            "先选择项目组分，再运行当前流程。".to_string()
        }
        ModelingReadinessTask::SelectReferencedProjectComponent {
            stream_id,
            component_id,
        } => format!(
            "流股 `{stream_id}` 的组成引用了未进入项目组分列表的 `{component_id}`，请先选择该项目组分。"
        ),
        ModelingReadinessTask::CommitFeedComposition { stream_id } => {
            format!("先补齐进料流股 `{stream_id}` 的总体摩尔分率，再运行当前流程。")
        }
        ModelingReadinessTask::NormalizeStreamComposition { stream_id } => {
            format!("先归一化流股 `{stream_id}` 的总体摩尔分率，再运行当前流程。")
        }
        ModelingReadinessTask::FixStreamCompositionValues { stream_id } => {
            format!("先修正流股 `{stream_id}` 的总体摩尔分率数值，再运行当前流程。")
        }
    }
}
