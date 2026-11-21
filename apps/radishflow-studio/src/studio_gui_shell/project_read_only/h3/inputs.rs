use super::*;
use rf_model::equipment::ConcentricTubeSideInputs;
use rf_types::units::QuantityKind as Q;

pub(super) const LENGTH: &str = "geometry.activeLengthM";
pub(super) const HOT_T: &str = "case.hot.inlet.temperatureK";
pub(super) const COLD_T: &str = "case.cold.inlet.temperatureK";
pub(super) const HOT_P: &str = "case.hot.inlet.pressurePa";
pub(super) const COLD_P: &str = "case.cold.inlet.pressurePa";

#[derive(Clone, Copy)]
enum Unit {
    Project(Q),
    Si(&'static str),
}

struct Row {
    id: &'static str,
    label: &'static str,
    value: Option<f64>,
    unit: Unit,
}

impl Row {
    fn new(id: &'static str, label: &'static str, value: Option<f64>, unit: Unit) -> Self {
        Self {
            id,
            label,
            value,
            unit,
        }
    }
}

pub(super) fn render(
    review: &mut ReviewState,
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    task: &ConcentricTubeRatingTask,
    units: &DisplayUnitSet,
) {
    use Unit::{Project, Si};
    let geometry = &task.geometry;
    let limits = &task.case.limits;
    group(
        review,
        ui,
        l,
        units,
        tr(l, "给定几何", "Given geometry"),
        [
            Row::new(
                LENGTH,
                tr(l, "有效换热长度", "Active length"),
                geometry.active_length_m,
                Project(Q::Length),
            ),
            Row::new(
                "geometry.innerTubeInnerDiameterM",
                tr(l, "内管内径", "Tube inner diameter"),
                geometry.inner_tube_inner_diameter_m,
                Project(Q::Length),
            ),
            Row::new(
                "geometry.innerTubeOuterDiameterM",
                tr(l, "内管外径", "Tube outer diameter"),
                geometry.inner_tube_outer_diameter_m,
                Project(Q::Length),
            ),
            Row::new(
                "geometry.outerTubeInnerDiameterM",
                tr(l, "外管内径", "Outer pipe inner diameter"),
                geometry.outer_tube_inner_diameter_m,
                Project(Q::Length),
            ),
            Row::new(
                "geometry.wallThermalConductivityWPerMK",
                tr(l, "管壁导热系数", "Wall conductivity"),
                geometry.wall_thermal_conductivity_w_per_m_k,
                Si("W/(m·K)"),
            ),
        ],
    );
    side(review, ui, l, units, &task.case.hot, true);
    side(review, ui, l, units, &task.case.cold, false);
    group(
        review,
        ui,
        l,
        units,
        tr(l, "限值", "Limits"),
        [
            Row::new(
                "limits.minDuty",
                tr(l, "最低热负荷", "Minimum heat duty"),
                limits.min_heat_duty_w,
                Si("W"),
            ),
            Row::new(
                "limits.maxHot",
                tr(l, "内管压降上限", "Maximum tube pressure drop"),
                limits.max_hot_pressure_drop_pa,
                Si("Pa"),
            ),
            Row::new(
                "limits.maxCold",
                tr(l, "环隙压降上限", "Maximum annulus pressure drop"),
                limits.max_cold_pressure_drop_pa,
                Si("Pa"),
            ),
        ],
    );
}

fn side(
    review: &mut ReviewState,
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    units: &DisplayUnitSet,
    side: &ConcentricTubeSideInputs,
    hot: bool,
) {
    use Unit::{Project, Si};
    let inlet = side.inlet;
    let properties = &side.properties;
    let domain = properties.liquid_domain;
    let ids = if hot {
        [
            HOT_T,
            HOT_P,
            "case.hot.inlet.molarFlowMolS",
            "case.hot.properties.molarMassKgPerMol",
            "case.hot.properties.molarHeatCapacityJPerMolK",
            "case.hot.properties.densityKgPerM3",
            "case.hot.properties.dynamicViscosityPaS",
            "case.hot.properties.thermalConductivityWPerMK",
            "case.hot.properties.liquidDomain.minTemperatureK",
            "case.hot.properties.liquidDomain.maxTemperatureK",
            "case.hot.properties.liquidDomain.minPressurePa",
            "case.hot.properties.liquidDomain.maxPressurePa",
        ]
    } else {
        [
            COLD_T,
            COLD_P,
            "case.cold.inlet.molarFlowMolS",
            "case.cold.properties.molarMassKgPerMol",
            "case.cold.properties.molarHeatCapacityJPerMolK",
            "case.cold.properties.densityKgPerM3",
            "case.cold.properties.dynamicViscosityPaS",
            "case.cold.properties.thermalConductivityWPerMK",
            "case.cold.properties.liquidDomain.minTemperatureK",
            "case.cold.properties.liquidDomain.maxTemperatureK",
            "case.cold.properties.liquidDomain.minPressurePa",
            "case.cold.properties.liquidDomain.maxPressurePa",
        ]
    };
    let [
        temp,
        pressure,
        flow,
        mass,
        cp,
        density,
        viscosity,
        conductivity,
        t_min,
        t_max,
        p_min,
        p_max,
    ] = ids;
    group(
        review,
        ui,
        l,
        units,
        if hot {
            tr(l, "热侧 · 内管", "Hot side · tube")
        } else {
            tr(l, "冷侧 · 环隙", "Cold side · annulus")
        },
        [
            Row::new(
                temp,
                tr(l, "入口温度", "Inlet temperature"),
                inlet.temperature_k,
                Project(Q::AbsoluteTemperature),
            ),
            Row::new(
                pressure,
                tr(l, "入口绝压", "Inlet absolute pressure"),
                inlet.pressure_pa,
                Project(Q::AbsolutePressure),
            ),
            Row::new(
                flow,
                tr(l, "入口摩尔流量", "Inlet molar flow"),
                inlet.molar_flow_mol_s,
                Project(Q::MolarFlow),
            ),
            Row::new(
                mass,
                tr(l, "摩尔质量", "Molar mass"),
                properties.molar_mass_kg_per_mol,
                Si("kg/mol"),
            ),
            Row::new(
                cp,
                tr(l, "定压摩尔热容", "Molar heat capacity"),
                properties.molar_heat_capacity_j_per_mol_k,
                Si("J/(mol·K)"),
            ),
            Row::new(
                density,
                tr(l, "密度", "Density"),
                properties.density_kg_per_m3,
                Project(Q::MassDensity),
            ),
            Row::new(
                viscosity,
                tr(l, "动力黏度", "Dynamic viscosity"),
                properties.dynamic_viscosity_pa_s,
                Si("Pa·s"),
            ),
            Row::new(
                conductivity,
                tr(l, "导热系数", "Thermal conductivity"),
                properties.thermal_conductivity_w_per_m_k,
                Si("W/(m·K)"),
            ),
            Row::new(
                t_min,
                tr(l, "温度域下界", "Minimum domain temperature"),
                domain.min_temperature_k,
                Project(Q::AbsoluteTemperature),
            ),
            Row::new(
                t_max,
                tr(l, "温度域上界", "Maximum domain temperature"),
                domain.max_temperature_k,
                Project(Q::AbsoluteTemperature),
            ),
            Row::new(
                p_min,
                tr(l, "绝压域下界", "Minimum domain pressure"),
                domain.min_pressure_pa,
                Project(Q::AbsolutePressure),
            ),
            Row::new(
                p_max,
                tr(l, "绝压域上界", "Maximum domain pressure"),
                domain.max_pressure_pa,
                Project(Q::AbsolutePressure),
            ),
        ],
    );
}

fn group(
    review: &mut ReviewState,
    ui: &mut egui::Ui,
    l: StudioShellLocale,
    units: &DisplayUnitSet,
    heading: &str,
    rows: impl IntoIterator<Item = Row>,
) {
    ui.separator();
    ui.strong(heading);
    for row in rows {
        let value = row.value.map_or_else(
            || tr(l, "未填写", "Missing").to_owned(),
            |value| match row.unit {
                Unit::Project(kind) => quantity(value, kind, units),
                Unit::Si(symbol) => format!(
                    "{} {symbol}",
                    rf_ui::format_display_number(value).unwrap_or_default()
                ),
            },
        );
        let highlighted = review.diagnosis_field == Some(row.id);
        let text = format!("{} · {value}", row.label);
        let response = if highlighted {
            ui.push_id(row.id, |ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(text)
                            .strong()
                            .color(egui::Color32::from_rgb(160, 48, 60)),
                    )
                    .wrap()
                    .sense(egui::Sense::focusable_noninteractive()),
                )
            })
            .inner
        } else {
            ui.add(egui::Label::new(text).wrap())
        };
        review.apply_focus(&response, Focus::Field(row.id));
    }
}
