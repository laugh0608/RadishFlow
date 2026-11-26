//! Keep flowsheet publication and per-block verification on their respective receipts.
use super::*;
use recycle_review::{RecycleReview, tear_is_bound};
use rf_solver::partitioned::{
    EvaluationPhase, PartitionedFailureKind, PartitionedSolveFailure, PartitionedSolveResult,
    RecycleBlockReport, SolveCosts,
};

#[derive(Clone, Copy)]
enum BlockState {
    Published,
    CompletedDiagnostic,
    Interrupted(PartitionedFailureKind),
}

pub(super) struct Panel<'a> {
    workspace: &'a rf_ui::WorkspaceState,
    review: &'a mut RecycleReview,
    generation: u64,
    unit: MeasurementUnit,
    zh: bool,
    action: Option<(StreamId, egui::Id)>,
}

impl<'a> Panel<'a> {
    pub(super) fn new(
        workspace: &'a rf_ui::WorkspaceState,
        review: &'a mut RecycleReview,
        unit: MeasurementUnit,
        zh: bool,
    ) -> Self {
        let generation = review.bind(workspace);
        Self {
            workspace,
            review,
            generation,
            unit,
            zh,
            action: None,
        }
    }

    pub(super) fn current(
        mut self,
        ui: &mut egui::Ui,
        result: &PartitionedSolveResult,
    ) -> Option<(StreamId, egui::Id)> {
        let zh = self.zh;
        let loops: Vec<_> = result
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(index, b)| Some((index, b.recycle.as_ref()?)))
            .collect();
        if let Some((_, last)) = loops.last() {
            self.downstream(ui, last, &result.costs);
        }
        let compact = loops.len() > 1;
        if compact {
            for &(index, r) in &loops {
                transfer(ui, &result.blocks, index, r, zh);
            }
        }
        for (index, r) in loops {
            self.block(
                ui,
                Some(index),
                r,
                BlockState::Published,
                compact,
                result.streams.contains_key(&r.tear.stream_id),
            );
        }
        ui.small(tr(
            zh,
            "固定温压回流 · 整体物料衡算",
            "Fixed-T/P recycle · overall material balance",
        ));
        for (index, block) in result.blocks.iter().enumerate() {
            let members = block
                .members
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" + ");
            egui::CollapsingHeader::new(format!(
                "{} {} · {:?} · {members}",
                tr(zh, "依赖块", "Block"),
                index + 1,
                block.kind
            ))
            .id_salt(("partitioned-block-io", self.generation, index))
            .show(ui, |ui| {
                for u in &block.final_units {
                    ui.collapsing(
                        format!(
                            "{} · {}",
                            u.unit_id,
                            tr(zh, "最终实际输入 / 输出", "Final actual inputs / outputs")
                        ),
                        |ui| {
                            for stream in &u.consumed_streams {
                                results::state(ui, tr(zh, "消耗", "Consumed"), stream);
                            }
                            for stream in &u.produced_streams {
                                results::state(ui, tr(zh, "产出", "Produced"), stream);
                            }
                        },
                    );
                }
            });
        }
        let cost = &result.costs;
        ui.small(format!(
            "{}: {} · thermo {} · flash {}",
            tr(zh, "实际局部调用", "Actual local calls"),
            cost.local_unit_calls,
            cost.thermo_calls,
            cost.flash_calls
        ));
        self.action
    }

    pub(super) fn failed(
        mut self,
        ui: &mut egui::Ui,
        failure: &PartitionedSolveFailure,
    ) -> Option<(StreamId, egui::Id)> {
        let zh = self.zh;
        let completed: Vec<_> = failure
            .completed_blocks
            .iter()
            .enumerate()
            .filter_map(|(index, b)| Some((index, b.recycle.as_ref()?)))
            .collect();
        let active = failure.active_recycle.as_ref();
        if let Some(last) = active.or_else(|| completed.last().map(|(_, r)| *r)) {
            self.downstream(ui, last, &failure.costs);
        }
        let compact = completed.len() + usize::from(active.is_some()) > 1;
        if compact {
            for &(index, r) in &completed {
                transfer(ui, &failure.completed_blocks, index, r, zh);
            }
            if let (Some(index), Some(r)) = (failure.stage.block, active) {
                transfer(ui, &failure.completed_blocks, index, r, zh);
            }
        }
        for (index, r) in completed {
            self.block(
                ui,
                Some(index),
                r,
                BlockState::CompletedDiagnostic,
                compact,
                tear_is_bound(self.workspace, &r.tear),
            );
        }
        if let Some(r) = active {
            self.block(
                ui,
                failure.stage.block,
                r,
                BlockState::Interrupted(failure.kind),
                compact,
                tear_is_bound(self.workspace, &r.tear),
            );
        } else if let Some(index) = failure.stage.block {
            ui.label(format!(
                "{} {}",
                tr(zh, "终止阶段 · 依赖块", "Stopped at block"),
                index + 1
            ));
        }
        ui.collapsing(tr(zh, "诊断依据", "Diagnostic evidence"), |ui| {
            ui.label(failure.error.to_string());
            if let Some(code) = failure.error.context().diagnostic_code() {
                ui.label(code);
            }
            if let Some(id) = &failure.stage.unit {
                ui.label(format!("{} {id}", tr(zh, "单元", "Unit")));
            }
        });
        ui.small(format!(
            "{} {}",
            tr(zh, "实际局部调用", "Actual local calls"),
            failure.costs.local_unit_calls
        ));
        self.action
    }

    fn block(
        &mut self,
        ui: &mut egui::Ui,
        index: Option<usize>,
        r: &RecycleBlockReport,
        state: BlockState,
        compact: bool,
        can_locate: bool,
    ) {
        let zh = self.zh;
        let tear = &r.tear;
        let name = index.map_or_else(
            || tr(zh, "回流块", "Recycle block").to_owned(),
            |n| format!("{} {}", tr(zh, "依赖块", "Block"), n + 1),
        );
        ui.push_id(
            (
                "partitioned-recycle",
                self.generation,
                index,
                &tear.stream_id,
            ),
            |ui| {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.strong(format!("{name} · {}", tear.stream_id));
                    ui.label(format!("{} + {}", tear.sink.unit_id, tear.source.unit_id));
                });
                let verdict = match state {
                    BlockState::Published => tr(zh, "本块复核通过", "Block verification passed"),
                    BlockState::CompletedDiagnostic => tr(
                        zh,
                        "本块复核通过 · 仅诊断",
                        "Block verification passed · Diagnostic only",
                    ),
                    BlockState::Interrupted(PartitionedFailureKind::IterationLimit)
                        if r.verification_evaluations == 0 =>
                    {
                        tr(zh, "独立复核未执行", "Independent verification not run")
                    }
                    BlockState::Interrupted(_) => {
                        tr(zh, "终止块 · 仅诊断", "Interrupted block · Diagnostic only")
                    }
                };
                ui.horizontal_wrapped(|ui| {
                    ui.strong(verdict);
                    ui.label(format!(
                        "{} {} · {} {}",
                        tr(zh, "迭代求值", "Iteration evaluations"),
                        r.iteration_evaluations,
                        tr(zh, "独立复核", "Independent verification"),
                        r.verification_evaluations
                    ));
                    let label = if matches!(state, BlockState::Published) {
                        tr(zh, "查看正式流股", "View formal stream")
                    } else {
                        tr(zh, "定位工程流股", "Locate project stream")
                    };
                    let response = ui.add_enabled(
                        can_locate,
                        egui::Button::new(format!("{label} {}", tear.stream_id)),
                    );
                    self.review
                        .stream_action(&tear.stream_id, &response, &mut self.action);
                });
                if matches!(state, BlockState::Interrupted(_))
                    && let Some(last) = r.residual_trace.last()
                    && last.phase == EvaluationPhase::Iteration
                {
                    ui.strong(format!(
                        "{} {}",
                        tr(zh, "最后试算", "Last trial"),
                        last.evaluation
                    ));
                }
                partitioned_residuals::evidence(
                    ui,
                    r,
                    self.unit,
                    zh,
                    !matches!(state, BlockState::Interrupted(_)),
                    compact,
                );
            },
        );
    }

    // An uncalled independent unit is not necessarily downstream. Follow the actual product port.
    fn downstream(&self, ui: &mut egui::Ui, last: &RecycleBlockReport, costs: &SolveCosts) {
        let flow = &self.workspace.document.flowsheet;
        let tear = &last.tear;
        let Some(splitter) = flow.units.get(&tear.source.unit_id) else {
            return;
        };
        for product in splitter
            .ports
            .iter()
            .filter(|p| p.direction == PortDirection::Outlet && p.kind == PortKind::Material)
            .filter_map(|p| p.stream_id.as_ref())
            .filter(|s| *s != &tear.stream_id)
        {
            for sink in flow.units.values().filter(|u| {
                u.id != tear.sink.unit_id
                    && u.id != tear.source.unit_id
                    && u.ports.iter().any(|p| {
                        p.kind == PortKind::Material
                            && p.direction == PortDirection::Inlet
                            && p.stream_id.as_ref() == Some(product)
                    })
            }) {
                let calls = costs.calls_by_unit.get(&sink.id).copied().unwrap_or(0);
                let status = if calls == 0 {
                    tr(self.zh, "未执行", "did not run").into()
                } else {
                    format!("{} {calls}", tr(self.zh, "实际调用", "local calls"))
                };
                ui.label(format!(
                    "{} {} {status} · {product}",
                    tr(self.zh, "下游", "Downstream"),
                    sink.id
                ));
            }
        }
    }
}

// Only link blocks when the earlier sweep actually produced the later block's fresh feed.
fn transfer(
    ui: &mut egui::Ui,
    completed: &[rf_solver::partitioned::BlockReport],
    index: usize,
    r: &RecycleBlockReport,
    zh: bool,
) {
    let rf_solver::partitioned::ZeroFlowCompositionRule::EvaluatedFreshBoundary(feed) =
        &r.zero_flow_composition_rule;
    if let Some((source, _)) = completed.iter().take(index).enumerate().find(|(_, b)| {
        b.recycle.is_some()
            && b.final_units
                .iter()
                .flat_map(|u| &u.produced_streams)
                .any(|s| s.id == *feed)
    }) {
        let block = tr(zh, "依赖块", "Block");
        ui.small(format!(
            "{block} {} → {feed} → {block} {}",
            source + 1,
            index + 1
        ));
    }
}
