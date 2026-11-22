use super::*;
use rf_model::equipment::{ConcentricTubeRatingLimits, ConcentricTubeSideInputs};
use rf_types::units::QuantityKind as Q;
use rf_ui::engineering::concentric_tube::{LimitCheck, SideRating};

const ERROR: egui::Color32 = egui::Color32::from_rgb(160, 48, 60);
const WARNING: egui::Color32 = egui::Color32::from_rgb(148, 85, 24);
const PASS: egui::Color32 = egui::Color32::from_rgb(39, 112, 75);

pub(super) fn render(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    result: &ConcentricTubeRating,
    limits: &ConcentricTubeRatingLimits,
    units: &DisplayUnitSet,
) {
    use LimitCheck::*;
    let checks = [
        (
            tr(l, "最低热负荷", "Minimum heat duty"),
            limits.min_heat_duty_w,
            "W",
            "≥",
            result.checks.heat_duty,
        ),
        (
            tr(l, "内管压降上限", "Maximum tube pressure drop"),
            limits.max_hot_pressure_drop_pa,
            "Pa",
            "≤",
            result.checks.hot_pressure_drop,
        ),
        (
            tr(l, "环隙压降上限", "Maximum annulus pressure drop"),
            limits.max_cold_pressure_drop_pa,
            "Pa",
            "≤",
            result.checks.cold_pressure_drop,
        ),
    ];
    let exceeded = checks.iter().filter(|check| check.4 == Failed).count();
    ui.horizontal_wrapped(|ui| {
        ui.colored_label(
            ui.visuals().hyperlink_color,
            tr(l, "当前结果", "Current result"),
        );
        if exceeded > 0 {
            ui.colored_label(
                WARNING,
                format!(
                    "{exceeded} {}",
                    tr(
                        l,
                        "项超限",
                        if exceeded == 1 {
                            "limit exceeded"
                        } else {
                            "limits exceeded"
                        }
                    )
                ),
            );
        }
    });
    ui.columns(2, |columns| {
        metric(
            &mut columns[0],
            tr(l, "热负荷 Q", "Heat duty Q"),
            result.heat_duty_w,
            "W",
        );
        metric(
            &mut columns[1],
            tr(l, "总传热能力 UA", "Conductance UA"),
            result.ua_w_per_k,
            "W/K",
        );
    });
    let outlets = |ui: &mut egui::Ui| {
        ui.strong(tr(l, "两侧出口", "Outlet conditions"));
        outlet(ui, l, tr(l, "热侧 · 内管", "Hot tube"), result.hot, units);
        outlet(
            ui,
            l,
            tr(l, "冷侧 · 环隙", "Cold annulus"),
            result.cold,
            units,
        );
    };
    let criteria = |ui: &mut egui::Ui| {
        ui.strong(tr(l, "校核准则", "Acceptance criteria"));
        for (label, limit, unit, comparator, status) in checks {
            ui.horizontal_wrapped(|ui| {
                ui.label(label);
                match status {
                    Passed => {
                        ui.colored_label(PASS, tr(l, "✓ 通过", "✓ Pass"));
                    }
                    Failed => {
                        ui.colored_label(WARNING, tr(l, "超限", "Limit exceeded"));
                    }
                    NotSpecified => {
                        ui.label(tr(l, "未指定", "Not specified"));
                    }
                }
            });
            if let Some(limit) = limit {
                ui.label(format!("{comparator} {limit:.2} {unit}"));
            }
        }
    };
    ui.separator();
    if ui.available_width() >= 600.0 {
        ui.columns(2, |columns| {
            outlets(&mut columns[0]);
            criteria(&mut columns[1]);
        });
    } else {
        outlets(ui);
        ui.separator();
        criteria(ui);
    }
}

fn metric(ui: &mut egui::Ui, label: &str, value: f64, unit: &str) {
    egui::Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(4)
        .inner_margin(5)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(label);
            ui.strong(format!("{value:.2} {unit}"));
        });
}

fn outlet(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    name: &str,
    side: SideRating,
    units: &DisplayUnitSet,
) {
    ui.label(egui::RichText::new(name).strong());
    ui.label(format!(
        "{} · {}",
        tr(l, "温度", "Temperature"),
        quantity(side.outlet.temperature_k, Q::AbsoluteTemperature, units)
    ));
    ui.label(format!(
        "{} · {}",
        tr(l, "绝压", "Absolute pressure"),
        quantity(side.outlet.pressure_pa, Q::AbsolutePressure, units)
    ));
    ui.label(format!(
        "{} · {:.2} Pa",
        tr(l, "沿程压降", "Pressure drop"),
        side.passage.pressure_drop_pa
    ));
}

pub(super) fn diagnostic(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    task: &ConcentricTubeRatingTask,
    error: &SavedRatingError,
    units: &DisplayUnitSet,
) -> Option<&'static str> {
    ui.colored_label(ERROR, tr(l, "复算失败", "Rating failed"));
    let SavedRatingError::Calculation(error) = error else {
        ui.label(tr(
            l,
            "已保存输入修订不符。",
            "Saved input revisions differ.",
        ));
        return None;
    };
    let code = error
        .context()
        .diagnostic_code()
        .unwrap_or(error.code().as_str());
    let field = match code {
        "unit.heat_exchanger.concentric_tube.missing_input" => {
            let path = error.message().split_once(':').map(|(path, _)| path);
            ui.strong(if path == Some(inputs::LENGTH) {
                tr(l, "缺少有效换热长度", "Active length is missing")
            } else {
                tr(
                    l,
                    "已保存工况缺少必需输入",
                    "The saved case is missing a required input",
                )
            });
            ui.label(error.message());
            missing_field(path)
        }
        "unit.heat_exchanger.concentric_tube.inlet_outside_domain" => {
            ui.strong(tr(
                l,
                "入口超出单液相适用域",
                "Inlet is outside the liquid domain",
            ));
            inlet_diagnostic(ui, l, task, units)
        }
        _ => {
            ui.label(error.message());
            None
        }
    };
    ui.label(egui::RichText::new(code).monospace().small());
    field
}

fn missing_field(path: Option<&str>) -> Option<&'static str> {
    // Core names missing fields by saved input path. Focus uses only matching
    // visible controls; an unrelated message never becomes an action target.
    [
        inputs::LENGTH,
        inputs::HOT_T,
        inputs::COLD_T,
        inputs::HOT_P,
        inputs::COLD_P,
    ]
    .into_iter()
    .find(|id| Some(*id) == path)
}

fn inlet_diagnostic(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    task: &ConcentricTubeRatingTask,
    units: &DisplayUnitSet,
) -> Option<&'static str> {
    let outside = |value: Option<f64>, lo: Option<f64>, hi: Option<f64>| {
        let (value, lo, hi) = (value?, lo?, hi?);
        (!(lo..=hi).contains(&value)).then_some((value, lo, hi))
    };
    let each = |input: &ConcentricTubeSideInputs, t, p| {
        let d = input.properties.liquid_domain;
        outside(
            input.inlet.temperature_k,
            d.min_temperature_k,
            d.max_temperature_k,
        )
        .map(|bounds| (t, Q::AbsoluteTemperature, bounds))
        .or_else(|| {
            outside(
                input.inlet.pressure_pa,
                d.min_pressure_pa,
                d.max_pressure_pa,
            )
            .map(|bounds| (p, Q::AbsolutePressure, bounds))
        })
    };
    let hot = each(&task.case.hot, inputs::HOT_T, inputs::HOT_P);
    let (side, (target, kind, (value, lo, hi))) = if let Some(hot) = hot {
        (tr(l, "热侧入口", "Hot inlet"), hot)
    } else {
        (
            tr(l, "冷侧入口", "Cold inlet"),
            each(&task.case.cold, inputs::COLD_T, inputs::COLD_P)?,
        )
    };
    let label = if kind == Q::AbsolutePressure {
        tr(l, "绝压", "absolute pressure")
    } else {
        tr(l, "温度", "temperature")
    };
    ui.label(format!("{side} {label} · {}", quantity(value, kind, units)));
    ui.label(format!(
        "{} · {} – {}",
        tr(l, "声明范围", "Declared range"),
        quantity(lo, kind, units),
        quantity(hi, kind, units)
    ));
    Some(target)
}

pub(super) fn evidence(
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    project: &StudioH3Review,
    task: &ConcentricTubeRatingTask,
    result: Option<&ConcentricTubeRating>,
) {
    panel(ui).show(ui, |ui| {
        ui.strong(tr(l, "打开时的工程快照", "Opened project snapshot"));
        ui.label(format!(
            "{} · {}",
            tr(l, "工程 ID", "Document ID"),
            project.metadata().document_id
        ));
        ui.label(format!("SHA-256 · {}", project.file_sha256()));
        ui.label(format!("{} / {}", task.id, task.case.id));
        ui.label(format!(
            "{} · {} / {} / {}",
            tr(
                l,
                "任务 / 几何 / 工况修订",
                "Task / geometry / case revisions"
            ),
            task.revision,
            task.geometry.revision,
            task.case.revision
        ));
        ui.label(format!(
            "{} · {} / {}",
            tr(l, "方法", "Method"),
            task.method.id,
            task.method.version
        ));
        for (label, id) in [
            (tr(l, "几何", "Geometry"), task.geometry.source_id.as_ref()),
            (
                tr(l, "热侧物性", "Hot properties"),
                Some(&task.case.hot.properties.source_id),
            ),
            (
                tr(l, "冷侧物性", "Cold properties"),
                Some(&task.case.cold.properties.source_id),
            ),
        ] {
            if let Some(id) = id
                && let Some(source) = project
                    .basis()
                    .sources
                    .iter()
                    .find(|source| source.id == *id)
            {
                ui.label(format!(
                    "{label} · {} · {} · r{}",
                    source.title, source.locator, source.revision
                ));
                if !source.reference.is_empty() && source.reference != source.title {
                    ui.label(&source.reference);
                }
            }
        }
        if let Some(result) = result {
            let units = project.display_units();
            ui.separator();
            ui.label(format!(
                "LMTD · {}",
                quantity(result.lmtd_k, Q::TemperatureDifference, units)
            ));
            ui.label(format!(
                "{} · {} / {}",
                tr(l, "内 / 外换热面积", "Inner / outer transfer area"),
                quantity(result.inner_heat_transfer_area_m2, Q::Area, units),
                quantity(result.outer_heat_transfer_area_m2, Q::Area, units)
            ));
            ui.label(format!(
                "{} · {:.2} / {:.2}",
                tr(l, "内管 / 环隙 Re", "Tube / annulus Re"),
                result.hot.passage.reynolds_number,
                result.cold.passage.reynolds_number
            ));
            ui.label(format!(
                "{} · {:.2} / {:.2}",
                tr(l, "内管 / 环隙 Nu", "Tube / annulus Nu"),
                result.hot.passage.nusselt_number,
                result.cold.passage.nusselt_number
            ));
            for (label, value) in [
                (
                    tr(l, "整体显热残差", "Overall sensible-heat residual"),
                    result.overall_sensible_energy_residual_w,
                ),
                ("Q − UA × LMTD", result.lmtd_heat_duty_residual_w),
            ] {
                ui.label(format!("{label} · {value:.3e} W"));
            }
        }
    });
}
