//! Calculation provenance supplements the existing result panel.
use super::*;
use rf_ui::result_query::{FlowResultUnavailable, ResultDataOrigin};

fn label(l: StudioShellLocale, zh: &'static str, en: &'static str) -> &'static str {
    match l {
        StudioShellLocale::ZhCn => zh,
        StudioShellLocale::En => en,
    }
}
fn technical(ui: &mut egui::Ui, l: StudioShellLocale, rows: Vec<(String, String)>) {
    egui::CollapsingHeader::new(label(l, "技术详情", "Technical details")).show(ui, |ui| {
        for (key, value) in rows {
            ui.add(egui::Label::new(format!("{key}: {value}")).wrap());
        }
    });
}
pub(super) fn flow(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    review: &radishflow_studio::FlowResultReview,
) {
    if let Some(reason) = review.unavailable {
        ui.label(match reason {
            FlowResultUnavailable::NoResult => label(
                l,
                "没有可用计算结果或来源。",
                "No available calculation result or origin.",
            ),
            FlowResultUnavailable::InputChanged => label(
                l,
                "正式输入已变化；请重新运行。",
                "Committed inputs changed; run again.",
            ),
            FlowResultUnavailable::DifferentSession => label(
                l,
                "结果不属于当前工程会话。",
                "Result belongs to another project session.",
            ),
            FlowResultUnavailable::UnqualifiedSnapshot => label(
                l,
                "旧结果未记录执行资格；请重新运行。",
                "Legacy result has no execution qualification; run again.",
            ),
        });
    }
    egui::CollapsingHeader::new(label(l, "计算来源", "Calculation origin")).show(ui, |ui| {
        let Some(origin) = &review.origin else {
            ui.label(label(l, "执行来源未记录", "Execution origin unrecorded"));
            return;
        };
        ui.label(format!(
            "{}: {}",
            label(l, "执行时输入修订", "Executed input revision"),
            origin.revision()
        ));
        match origin.execution() {
            None => {
                ui.label(label(l, "执行来源未记录", "Execution origin unrecorded"));
            }
            Some(execution) => match &execution.package {
                None => {
                    ui.label(label(
                        l,
                        "准备阶段未加载物性包",
                        "Property package not loaded during preparation",
                    ));
                }
                Some(package) => {
                    ui.add(
                        egui::Label::new(format!(
                            "{}: {} v{}",
                            label(l, "实际物性包", "Actual property package"),
                            package.package_id,
                            package.version
                        ))
                        .wrap(),
                    );
                    ui.label(match package.origin {
                        ResultDataOrigin::InMemory => label(l, "来源：内存", "Source: memory"),
                        ResultDataOrigin::Cache => label(l, "来源：缓存", "Source: cache"),
                        ResultDataOrigin::LocalFiles => {
                            label(l, "来源：本地文件", "Source: local files")
                        }
                        ResultDataOrigin::Bundled => {
                            label(l, "来源：内置资源", "Source: bundled resources")
                        }
                    });
                }
            },
        }
        technical(ui, l, radishflow_studio::flow_origin_lines(Some(origin)));
    });
}
pub(super) fn equipment(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    view: &rf_ui::engineering::EquipmentResultView<'_>,
) {
    use rf_ui::engineering::{EquipmentResultUnavailable as R, EquipmentResultView as V};
    if let V::Stale { reason, .. } | V::Missing { reason, .. } = view {
        ui.label(match reason {
            R::NoResult => label(l, "尚未计算", "Not calculated"),
            R::MemberMissing => label(l, "任务或工况已不存在", "Task or case no longer exists"),
            R::InputChanged => label(l, "正式输入已变化", "Committed inputs changed"),
            R::EvidenceChanged => label(l, "依据已变化", "Evidence changed"),
            R::InputAndEvidenceChanged => {
                label(l, "输入与依据均已变化", "Inputs and evidence changed")
            }
            R::EligibilityRevoked => {
                label(l, "计算资格已失效", "Calculation qualification revoked")
            }
        });
    }
    egui::CollapsingHeader::new(label(l, "计算来源", "Calculation origin")).show(ui, |ui| {
        let Some(origin) = view.origin() else {
            ui.label(label(l, "执行来源未记录", "Execution origin unrecorded"));
            return;
        };
        ui.add(
            egui::Label::new(format!(
                "{} · {} · {}",
                origin.task_tag, origin.task_name, origin.case_name
            ))
            .wrap(),
        );
        ui.add(
            egui::Label::new(format!(
                "{}: {} v{}",
                label(l, "方法", "Method"),
                origin.method.id,
                origin.method.version
            ))
            .wrap(),
        );
        ui.label(format!(
            "{}: {}",
            label(l, "执行时任务修订", "Executed task revision"),
            origin.task_revision
        ));
        if let Some(rev) = origin.case_revision {
            ui.label(format!("{}: {rev}", label(l, "工况修订", "Case revision")));
        }
        if let Some(rev) = origin.geometry_revision {
            ui.label(format!(
                "{}: {rev}",
                label(l, "几何修订", "Geometry revision")
            ));
        }
        technical(ui, l, radishflow_studio::equipment_origin_lines(origin));
    });
}

impl ReadyAppState {
    pub(super) fn dispatch_result_action(
        &mut self,
        action: &radishflow_studio::StudioGuiWindowCommandActionModel,
    ) {
        if !action.result_is_current(&self.platform_host.modeling_inputs().workspace) {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: label(self.locale, "定位不可用", "Navigation unavailable").into(),
                detail: label(
                    self.locale,
                    "结果或会话已变化，请返回当前结果。未提交输入已保留。",
                    "Result or session changed. Return to current results. Draft inputs are preserved.",
                ).into(),
            });
            return;
        }
        self.dispatch_ui_command(&action.command_id);
    }
}

pub(super) struct EquipmentNavigation<T> {
    pub target: T,
    pub guard: radishflow_studio::EquipmentResultGuard,
}
impl ReadyAppState {
    pub(super) fn current_equipment_navigation<T>(
        &mut self,
        navigation: EquipmentNavigation<T>,
    ) -> Option<T> {
        if navigation
            .guard
            .is_current(&self.platform_host.modeling_inputs().workspace.engineering)
        {
            Some(navigation.target)
        } else {
            self.project_open.notice = Some(ProjectOpenNotice {
                level: ProjectOpenNoticeLevel::Warning,
                title: label(self.locale, "定位不可用", "Navigation unavailable").into(),
                detail: label(
                    self.locale,
                    "工况结果或目标已变化，请返回当前结果。未提交输入已保留。",
                    "Case result or target changed. Return to current results. Draft inputs are preserved.",
                ).into(),
            });
            None
        }
    }
}
