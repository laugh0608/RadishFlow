//! Display the frozen readings of the rejected balance in their original SI units.
use super::*;
use rf_solver::partitioned::{FlowDifference, MaterialBalanceFailure, MaterialBalanceScope};

pub(super) fn location(scope: &MaterialBalanceScope, zh: bool) -> String {
    let block = tr(zh, "依赖块", "Block");
    match scope {
        MaterialBalanceScope::Unit {
            block_index,
            unit_id,
        } => format!(
            "{block} {} · {unit_id} · {}",
            block_index + 1,
            tr(
                zh,
                "单元物料衡算超限",
                "Unit material balance exceeds tolerance"
            )
        ),
        MaterialBalanceScope::RecycleRegion {
            block_index,
            tear_stream_id,
        } => format!(
            "{block} {} · {} {tear_stream_id} · {}",
            block_index + 1,
            tr(zh, "撕裂流股", "Tear stream"),
            tr(
                zh,
                "区域物料衡算超限",
                "Region material balance exceeds tolerance"
            )
        ),
        MaterialBalanceScope::Flowsheet => tr(
            zh,
            "整图物料衡算超限",
            "Flowsheet material balance exceeds tolerance",
        )
        .to_owned(),
    }
}

pub(super) fn flowsheet_boundary(failure: &MaterialBalanceFailure, zh: bool) -> String {
    let ids = |side: &[StreamId]| {
        side.iter()
            .map(StreamId::as_str)
            .collect::<Vec<_>>()
            .join(" + ")
    };
    format!(
        "{} {} → {} {}",
        tr(zh, "新鲜进料", "Fresh feeds"),
        ids(&failure.readings.inputs),
        tr(zh, "终端产品", "Terminal products"),
        ids(&failure.readings.outputs)
    )
}

fn amounts(ui: &mut egui::Ui, flow: &FlowDifference, zh: bool) {
    ui.label(format!(
        "{} {:e} mol/s",
        tr(zh, "入口合计", "Input sum"),
        flow.input_mol_s
    ));
    ui.label(format!(
        "{} {:e} mol/s",
        tr(zh, "出口合计", "Output sum"),
        flow.output_mol_s
    ));
    ui.label(format!(
        "{} {:+e} mol/s",
        tr(zh, "出口减入口", "Output minus input"),
        flow.difference_mol_s
    ));
}

fn threshold(
    ui: &mut egui::Ui,
    label: &str,
    value: f64,
    limit: f64,
    unit: &str,
    exceeded: bool,
    zh: bool,
) {
    let comparison = if value.is_finite() {
        let operator = if exceeded { ">" } else { "≤" };
        format!("{value:e} {operator} {limit:e}{unit}")
    } else {
        format!("{value} · {} {limit:e}{unit}", tr(zh, "门槛", "limit"))
    };
    if exceeded {
        ui.strong(format!(
            "{label} · {} · {comparison}",
            tr(zh, "超限", "Exceeded")
        ));
    } else {
        ui.label(format!("{label} · {comparison}"));
    }
}

pub(super) fn details(
    ui: &mut egui::Ui,
    failure: &MaterialBalanceFailure,
    flow: &rf_model::Flowsheet,
    zh: bool,
    locate: &mut Option<StreamId>,
) {
    let r = &failure.readings;
    for (label, streams) in [
        (tr(zh, "入口流股", "Input streams"), &r.inputs),
        (tr(zh, "出口流股", "Output streams"), &r.outputs),
    ] {
        ui.horizontal_wrapped(|ui| {
            ui.label(label);
            for id in streams {
                if ui
                    .add_enabled(
                        flow.streams.contains_key(id),
                        egui::Button::new(id.as_str()),
                    )
                    .clicked()
                {
                    *locate = Some(id.clone());
                }
            }
        });
    }
    for row in &r.exceeded_components {
        ui.separator();
        let id = &row.component_id;
        let label = tr(zh, "组分", "Component");
        if let Some(c) = flow
            .components
            .get(id)
            .filter(|c| !c.name.is_empty() && c.name != id.as_str())
        {
            ui.strong(format!("{label} · {} ({id})", c.name));
        } else {
            ui.strong(format!("{label} · {id}"));
        }
        amounts(ui, &row.flow, zh);
        threshold(
            ui,
            tr(zh, "绝对差", "Absolute difference"),
            row.flow.difference_mol_s.abs(),
            r.tolerances.component_absolute_flow_mol_s,
            " mol/s",
            row.absolute_exceeded,
            zh,
        );
        ui.label(format!(
            "{} {:e} mol/s",
            tr(zh, "冻结尺度", "Frozen scale"),
            row.scale_mol_s
        ));
        threshold(
            ui,
            tr(zh, "缩放差", "Scaled difference"),
            row.scaled_absolute,
            r.tolerances.scaled_flow,
            "",
            row.scaled_exceeded,
            zh,
        );
    }
    ui.separator();
    ui.strong(tr(zh, "总摩尔流量", "Total molar flow"));
    amounts(ui, &r.total, zh);
    threshold(
        ui,
        tr(zh, "总量绝对差", "Absolute total difference"),
        r.total.difference_mol_s.abs(),
        r.tolerances.total_absolute_flow_mol_s,
        " mol/s",
        r.total_exceeded,
        zh,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use rf_solver::partitioned::{MaterialBalanceReadings, RecycleTolerances};

    #[test]
    fn total_only_failure_shows_precise_or_nonfinite_comparison_in_both_languages() {
        let limit = 1e-10_f64;
        let just_over = limit.next_up();
        let flow = rf_model::Flowsheet::new("Total-only balance");
        for (zh, component, total, exceeded, limit_label) in [
            (true, "组分 ·", "总摩尔流量", "超限", "门槛"),
            (
                false,
                "Component ·",
                "Total molar flow",
                "Exceeded",
                "limit",
            ),
        ] {
            for difference in [just_over, f64::NAN] {
                let ctx = egui::Context::default();
                ctx.enable_accesskit();
                let failure = MaterialBalanceFailure {
                    scope: MaterialBalanceScope::Flowsheet,
                    readings: Box::new(MaterialBalanceReadings {
                        inputs: vec!["feed".into()],
                        outputs: vec!["product".into()],
                        exceeded_components: vec![],
                        tolerances: RecycleTolerances {
                            component_absolute_flow_mol_s: limit,
                            scaled_flow: limit,
                            total_absolute_flow_mol_s: limit,
                            temperature_k: 1e-8,
                            pressure_pa: 1e-4,
                        },
                        total: FlowDifference {
                            input_mol_s: if difference.is_nan() {
                                f64::INFINITY
                            } else {
                                0.0
                            },
                            output_mol_s: if difference.is_nan() {
                                f64::INFINITY
                            } else {
                                difference
                            },
                            difference_mol_s: difference,
                        },
                        total_exceeded: true,
                    }),
                };
                let raw = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(320.0, 800.0),
                    )),
                    ..Default::default()
                };
                let output = ctx.run(raw, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let width = ui.available_width();
                        details(ui, &failure, &flow, zh, &mut None);
                        assert!(ui.min_rect().width() <= width + 1.0);
                    });
                });
                let tree = output.platform_output.accesskit_update.unwrap();
                let labels: Vec<_> = tree
                    .nodes
                    .iter()
                    .filter_map(|(_, node)| node.value())
                    .collect();
                assert!(labels.contains(&total));
                assert!(!labels.iter().any(|text| text.starts_with(component)));
                let comparison = if difference.is_nan() {
                    format!("{exceeded} · NaN · {limit_label} 1e-10 mol/s")
                } else {
                    format!("{exceeded} · {just_over:e} > 1e-10 mol/s")
                };
                assert!(
                    labels.iter().any(|text| text.contains(&comparison)),
                    "{labels:?}"
                );
            }
        }
    }
}
