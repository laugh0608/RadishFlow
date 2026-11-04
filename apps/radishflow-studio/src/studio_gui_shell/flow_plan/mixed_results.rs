use super::*;
use rf_solver::mixed::*;
pub(in crate::studio_gui_shell) fn progress_text(p: &MixedProgress, zh: bool) -> String {
    let stage = match p.stage {
        MixedStage::Assembly => tr(zh, "装配分区 DAG", "Assembling partition DAG"),
        MixedStage::BlockStarted => tr(zh, "执行分区块", "Executing block"),
        MixedStage::Equation => tr(zh, "块内 EO 求解与复核", "Block EO solve and verification"),
        MixedStage::Boundary => tr(zh, "复核真实跨块边界", "Verifying actual block boundary"),
        MixedStage::BlockCompleted => {
            tr(zh, "块完成（仅诊断）", "Block completed (diagnostic only)")
        }
        MixedStage::GlobalVerification => tr(zh, "总体物料复核", "Global material verification"),
        MixedStage::Completing => tr(
            zh,
            "准备接收整体结果",
            "Preparing complete result reception",
        ),
    };
    let mut text = stage.to_owned();
    if let Some(b) = &p.block {
        text.push_str(&format!(" · {}", b.as_str()));
    }
    if let Some(u) = &p.unit {
        text.push_str(&format!(" · {u}"));
    }
    if let Some(e) = &p.equation {
        text.push_str(&format!(
            " · {}",
            super::equation_results::progress_text(e, zh)
        ));
    }
    text
}
impl ReadyAppState {
    pub(super) fn render_mixed_current(&self, ui: &mut egui::Ui, r: &MixedResult) {
        let zh = self.locale == StudioShellLocale::ZhCn;
        ui.strong(tr(
            zh,
            "混合分区整体结果",
            "Complete mixed-partition result",
        ));
        ui.label(format!(
            "{} {} · {} {}",
            r.blocks.len(),
            tr(zh, "块", "blocks"),
            r.boundaries.len(),
            tr(zh, "真实跨块边界", "actual block boundaries")
        ));
        ui.small(tr(zh,"全部局部单元、EO、跨块边界及总体物料已复核；热量和相平衡不在 EO 方程范围。","All local units, EO blocks, boundaries and global material closure verified. EO equations cover material only."));
        let units = self
            .platform_host
            .modeling_inputs()
            .workspace
            .project_presentation
            .display_units();
        for b in &r.blocks {
            ui.collapsing(
                format!(
                    "{} · {} · {}",
                    b.id.as_str(),
                    match b.evidence {
                        MixedBlockEvidence::Local(_) => "Local",
                        MixedBlockEvidence::EquationOriented(_) => "EO",
                    },
                    b.members
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" + ")
                ),
                |ui| {
                    for line in radishflow_studio::mixed_evidence::block_lines(b, units).lines {
                        ui.label(line);
                    }
                },
            );
        }
        ui.collapsing(
            tr(
                zh,
                "跨块实际产出 / 消耗、残差及总体成本",
                "Actual boundary production / consumption, residuals and total costs",
            ),
            |ui| {
                for line in radishflow_studio::mixed_evidence::boundary_lines(r, units).lines {
                    ui.label(line);
                }
            },
        );
    }
}
