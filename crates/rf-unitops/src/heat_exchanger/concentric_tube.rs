//! Single-pass rating of a horizontal concentric tube with a hot inner passage
//! and a cold annulus. Properties are frozen within caller-supplied liquid domains.

use super::logarithmic_mean_approach_k;
use crate::single_phase::{ENERGY_ABSOLUTE_TOLERANCE_W, ENERGY_SCALED_TOLERANCE};
use rf_types::{RfError, RfResult};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConcentricTubeGeometry {
    pub active_length_m: f64,
    pub inner_tube_inner_diameter_m: f64,
    pub inner_tube_outer_diameter_m: f64,
    pub outer_tube_inner_diameter_m: f64,
    pub wall_thermal_conductivity_w_per_m_k: f64,
}

/// Inclusive temperature and absolute pressure bounds for the supplied liquid properties.
/// The caller confirms single-phase liquid validity throughout this domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidPropertyDomain {
    pub min_temperature_k: f64,
    pub max_temperature_k: f64,
    pub min_pressure_pa: f64,
    pub max_pressure_pa: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstantLiquidProperties<'a> {
    pub molar_mass_kg_per_mol: f64,
    pub molar_heat_capacity_j_per_mol_k: f64,
    pub density_kg_per_m3: f64,
    pub dynamic_viscosity_pa_s: f64,
    pub thermal_conductivity_w_per_m_k: f64,
    pub domain: LiquidPropertyDomain,
    pub source: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidState {
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub molar_flow_mol_s: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidSide<'a> {
    pub inlet: LiquidState,
    pub properties: ConstantLiquidProperties<'a>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct RatingLimits {
    pub min_heat_duty_w: Option<f64>,
    pub max_hot_pressure_drop_pa: Option<f64>,
    pub max_cold_pressure_drop_pa: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitCheck {
    NotSpecified,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitChecks {
    pub heat_duty: LimitCheck,
    pub hot_pressure_drop: LimitCheck,
    pub cold_pressure_drop: LimitCheck,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PassagePerformance {
    pub heat_capacity_rate_w_per_k: f64,
    pub reynolds_number: f64,
    pub prandtl_number: f64,
    pub nusselt_number: f64,
    pub darcy_friction_factor: f64,
    pub film_heat_transfer_coefficient_w_per_m2_k: f64,
    pub pressure_drop_pa: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SideRating {
    pub outlet: LiquidState,
    pub passage: PassagePerformance,
    /// Actual sensible heat gain minus the target, which is -Q on the hot side and +Q on the cold side.
    pub sensible_energy_residual_w: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConcentricTubeRating {
    pub hot: SideRating,
    pub cold: SideRating,
    pub inner_heat_transfer_area_m2: f64,
    pub outer_heat_transfer_area_m2: f64,
    /// Based on the outer surface of the inner tube.
    pub overall_heat_transfer_coefficient_outer_w_per_m2_k: f64,
    pub ua_w_per_k: f64,
    pub effectiveness: f64,
    pub heat_duty_w: f64,
    pub hot_inlet_approach_k: f64,
    pub cold_inlet_approach_k: f64,
    pub lmtd_k: f64,
    pub overall_sensible_energy_residual_w: f64,
    /// Q minus UA times LMTD.
    pub lmtd_heat_duty_residual_w: f64,
    pub checks: LimitChecks,
}

/// Rate the supplied clean geometry with constant sensible heat capacity and
/// straight-run Darcy pressure loss. Unmet limits are returned with the result.
pub fn rate_countercurrent(
    geometry: ConcentricTubeGeometry,
    hot: LiquidSide<'_>,
    cold: LiquidSide<'_>,
    limits: RatingLimits,
) -> RfResult<ConcentricTubeRating> {
    let (inner_tube, annulus) = geometry.passages()?;
    limits.validate()?;
    validate_liquid_side(hot, inner_tube.kind)?;
    validate_liquid_side(cold, annulus.kind)?;
    let inlet_difference_k = hot.inlet.temperature_k - cold.inlet.temperature_k;
    if inlet_difference_k < 0.0 {
        return Err(error(
            "reversed_inlets",
            "the inner tube inlet must be at least as warm as the annulus inlet",
        ));
    }

    let length_m = geometry.active_length_m;
    let hot_passage = inner_tube.rate(length_m, hot)?;
    let cold_passage = annulus.rate(length_m, cold)?;
    let inner_area_m2 = inner_tube.heat_transfer_area_m2;
    let outer_area_m2 = annulus.heat_transfer_area_m2;
    let wall_diameter_ratio_offset = (geometry.inner_tube_outer_diameter_m
        - geometry.inner_tube_inner_diameter_m)
        / geometry.inner_tube_inner_diameter_m;
    let wall_resistance_k_per_w = wall_diameter_ratio_offset.ln_1p()
        / (2.0 * PI * length_m * geometry.wall_thermal_conductivity_w_per_m_k);
    let hot_resistance_k_per_w =
        (hot_passage.film_heat_transfer_coefficient_w_per_m2_k * inner_area_m2).recip();
    let cold_resistance_k_per_w =
        (cold_passage.film_heat_transfer_coefficient_w_per_m2_k * outer_area_m2).recip();
    let ua_w_per_k =
        (hot_resistance_k_per_w + wall_resistance_k_per_w + cold_resistance_k_per_w).recip();
    let u_outer_w_per_m2_k = ua_w_per_k / outer_area_m2;
    if !positive_finite(&[
        hot_resistance_k_per_w,
        wall_resistance_k_per_w,
        cold_resistance_k_per_w,
        ua_w_per_k,
        u_outer_w_per_m2_k,
    ]) {
        return Err(error(
            "numeric_range",
            "the thermal resistances, UA and outer-basis U must be positive and finite",
        ));
    }

    let hot_capacity = hot_passage.heat_capacity_rate_w_per_k;
    let cold_capacity = cold_passage.heat_capacity_rate_w_per_k;
    let min_capacity = hot_capacity.min(cold_capacity);
    let capacity_ratio = min_capacity / hot_capacity.max(cold_capacity);
    let ntu = ua_w_per_k / min_capacity;
    let effectiveness = countercurrent_effectiveness(ntu, capacity_ratio);
    let pinch_duty_w = min_capacity * inlet_difference_k;
    let heat_duty_w = effectiveness * pinch_duty_w;
    if !positive_finite(&[ntu, effectiveness])
        || effectiveness > 1.0
        || !heat_duty_w.is_finite()
        || (inlet_difference_k > 0.0 && heat_duty_w == 0.0)
    {
        return Err(error(
            "numeric_range",
            "the NTU, effectiveness or heat duty is outside the numeric range",
        ));
    }
    let hot_rating = rate_outlet(hot, hot_passage, -heat_duty_w, inner_tube.kind)?;
    let cold_rating = rate_outlet(cold, cold_passage, heat_duty_w, annulus.kind)?;
    let hot_inlet_approach_k = hot.inlet.temperature_k - cold_rating.outlet.temperature_k;
    let cold_inlet_approach_k = hot_rating.outlet.temperature_k - cold.inlet.temperature_k;
    if heat_duty_w > 0.0
        && (heat_duty_w >= pinch_duty_w
            || hot_inlet_approach_k <= 0.0
            || cold_inlet_approach_k <= 0.0)
    {
        return Err(error(
            "pinch_limit",
            "positive heat duty must be below the pinch with positive approaches at both ends",
        ));
    }

    let lmtd_k = logarithmic_mean_approach_k(hot_inlet_approach_k, cold_inlet_approach_k);
    let lmtd_heat_duty_residual_w = heat_duty_w - ua_w_per_k * lmtd_k;
    let overall_sensible_energy_residual_w =
        hot_rating.sensible_energy_residual_w + cold_rating.sensible_energy_residual_w;
    let tolerance_w = ENERGY_ABSOLUTE_TOLERANCE_W.max(ENERGY_SCALED_TOLERANCE * heat_duty_w);
    let residuals = [
        hot_rating.sensible_energy_residual_w,
        cold_rating.sensible_energy_residual_w,
        overall_sensible_energy_residual_w,
        lmtd_heat_duty_residual_w,
    ];
    if residuals
        .into_iter()
        .any(|residual| !residual.is_finite() || residual.abs() > tolerance_w)
    {
        return Err(RfError::thermo(format!(
            "sensible and LMTD residuals {residuals:?} W exceed the {tolerance_w} W closure tolerance"
        ))
        .with_diagnostic_code("unit.heat_exchanger.concentric_tube.energy_closure"));
    }

    Ok(ConcentricTubeRating {
        hot: hot_rating,
        cold: cold_rating,
        inner_heat_transfer_area_m2: inner_area_m2,
        outer_heat_transfer_area_m2: outer_area_m2,
        overall_heat_transfer_coefficient_outer_w_per_m2_k: u_outer_w_per_m2_k,
        ua_w_per_k,
        effectiveness,
        heat_duty_w,
        hot_inlet_approach_k,
        cold_inlet_approach_k,
        lmtd_k,
        overall_sensible_energy_residual_w,
        lmtd_heat_duty_residual_w,
        checks: limits.check(
            heat_duty_w,
            hot_passage.pressure_drop_pa,
            cold_passage.pressure_drop_pa,
        ),
    })
}

impl ConcentricTubeGeometry {
    fn passages(self) -> RfResult<(PassageGeometry, PassageGeometry)> {
        let di = self.inner_tube_inner_diameter_m;
        let dw = self.inner_tube_outer_diameter_m;
        let outer_id = self.outer_tube_inner_diameter_m;
        let length_m = self.active_length_m;
        if !positive_finite(&[
            di,
            dw,
            outer_id,
            length_m,
            self.wall_thermal_conductivity_w_per_m_k,
        ]) || di >= dw
            || dw >= outer_id
        {
            return Err(error(
                "invalid_geometry",
                "positive finite length and wall conductivity and diameters 0 < Di < Dw < Do are required",
            ));
        }
        let diameter_ratio = dw / outer_id;
        if !(0.48..=0.60).contains(&diameter_ratio) {
            return Err(error(
                "outside_correlation_domain",
                format!("annulus diameter ratio {diameter_ratio} is outside [0.48, 0.60]"),
            ));
        }
        let annulus_dh_m = outer_id - dw;
        Ok((
            PassageGeometry {
                kind: PassageKind::InnerTube,
                hydraulic_diameter_m: di,
                flow_area_m2: PI / 4.0 * di * di,
                heat_transfer_area_m2: PI * di * length_m,
            },
            PassageGeometry {
                kind: PassageKind::Annulus { diameter_ratio },
                hydraulic_diameter_m: annulus_dh_m,
                flow_area_m2: PI / 4.0 * annulus_dh_m * (outer_id + dw),
                heat_transfer_area_m2: PI * dw * length_m,
            },
        ))
    }
}

impl RatingLimits {
    fn validate(self) -> RfResult<()> {
        for (name, limit) in [
            ("minimum heat duty", self.min_heat_duty_w),
            ("maximum hot pressure drop", self.max_hot_pressure_drop_pa),
            ("maximum cold pressure drop", self.max_cold_pressure_drop_pa),
        ] {
            if limit.is_some_and(|value| !value.is_finite() || value < 0.0) {
                return Err(error(
                    "invalid_limit",
                    format!("{name} must be nonnegative and finite"),
                ));
            }
        }
        Ok(())
    }

    fn check(self, heat_duty_w: f64, hot_drop_pa: f64, cold_drop_pa: f64) -> LimitChecks {
        LimitChecks {
            heat_duty: limit_check(self.min_heat_duty_w.map(|limit| heat_duty_w >= limit)),
            hot_pressure_drop: limit_check(
                self.max_hot_pressure_drop_pa
                    .map(|limit| hot_drop_pa <= limit),
            ),
            cold_pressure_drop: limit_check(
                self.max_cold_pressure_drop_pa
                    .map(|limit| cold_drop_pa <= limit),
            ),
        }
    }
}

fn limit_check(passes: Option<bool>) -> LimitCheck {
    match passes {
        None => LimitCheck::NotSpecified,
        Some(true) => LimitCheck::Passed,
        Some(false) => LimitCheck::Failed,
    }
}

#[derive(Clone, Copy)]
enum PassageKind {
    InnerTube,
    Annulus { diameter_ratio: f64 },
}

impl PassageKind {
    fn name(self) -> &'static str {
        match self {
            Self::InnerTube => "hot inner tube",
            Self::Annulus { .. } => "cold annulus",
        }
    }

    fn correlations(self, re: f64, pr: f64, length_over_diameter: f64) -> (f64, f64) {
        let pr_term = pr.powf(2.0 / 3.0) - 1.0;
        match self {
            Self::InnerTube => {
                let f = (1.82 * re.log10() - 1.64).powi(-2);
                let nu = (f / 8.0) * (re - 1000.0) * pr / (1.0 + 12.7 * (f / 8.0).sqrt() * pr_term);
                (f, nu)
            }
            Self::Annulus { diameter_ratio: a } => {
                let ln_a = a.ln();
                let modified_re =
                    re * ((1.0 + a * a) * ln_a + (1.0 - a * a)) / ((1.0 - a).powi(2) * ln_a);
                let f = (1.8 * modified_re.log10() - 1.5).powi(-2);
                let denominator_offset = 1.07 + 900.0 / re - 0.63 / (1.0 + 10.0 * pr);
                let entry_factor = 1.0 + length_over_diameter.recip().powf(2.0 / 3.0);
                let heated_inner_wall_factor = 0.75 * a.powf(-0.17);
                let nu = (f / 8.0) * re * pr
                    / (denominator_offset + 12.7 * (f / 8.0).sqrt() * pr_term)
                    * entry_factor
                    * heated_inner_wall_factor;
                (f, nu)
            }
        }
    }
}

struct PassageGeometry {
    kind: PassageKind,
    hydraulic_diameter_m: f64,
    flow_area_m2: f64,
    heat_transfer_area_m2: f64,
}

impl PassageGeometry {
    fn rate(&self, length_m: f64, side: LiquidSide<'_>) -> RfResult<PassagePerformance> {
        let name = self.kind.name();
        let length_over_diameter = length_m / self.hydraulic_diameter_m;
        if !positive_finite(&[
            self.flow_area_m2,
            self.heat_transfer_area_m2,
            length_over_diameter,
        ]) {
            return Err(error(
                "numeric_range",
                format!("{name} areas and length ratio must be positive and finite"),
            ));
        }
        let properties = side.properties;
        let mass_flow_kg_s = side.inlet.molar_flow_mol_s * properties.molar_mass_kg_per_mol;
        let mass_flux_kg_per_m2_s = mass_flow_kg_s / self.flow_area_m2;
        let heat_capacity_rate_w_per_k =
            side.inlet.molar_flow_mol_s * properties.molar_heat_capacity_j_per_mol_k;
        let mass_heat_capacity_j_per_kg_k =
            properties.molar_heat_capacity_j_per_mol_k / properties.molar_mass_kg_per_mol;
        let re =
            mass_flux_kg_per_m2_s * self.hydraulic_diameter_m / properties.dynamic_viscosity_pa_s;
        let pr = mass_heat_capacity_j_per_kg_k * properties.dynamic_viscosity_pa_s
            / properties.thermal_conductivity_w_per_m_k;
        if !positive_finite(&[heat_capacity_rate_w_per_k, re, pr]) {
            return Err(error(
                "numeric_range",
                format!("{name} heat capacity rate, Re and Pr must be positive and finite"),
            ));
        }
        if !(10_000.0..=40_000.0).contains(&re)
            || !(3.0..=8.0).contains(&pr)
            || length_over_diameter < 100.0
        {
            return Err(error(
                "outside_correlation_domain",
                format!(
                    "{name} requires Re in [10000, 40000], Pr in [3, 8] and L/Dh >= 100, got {re}, {pr} and {length_over_diameter}"
                ),
            ));
        }
        let (darcy_friction_factor, nusselt_number) =
            self.kind.correlations(re, pr, length_over_diameter);
        let film_heat_transfer_coefficient_w_per_m2_k =
            nusselt_number * properties.thermal_conductivity_w_per_m_k / self.hydraulic_diameter_m;
        let velocity_m_s = mass_flux_kg_per_m2_s / properties.density_kg_per_m3;
        let pressure_drop_pa = darcy_friction_factor
            * length_over_diameter
            * (properties.density_kg_per_m3 * velocity_m_s.powi(2) / 2.0);
        if !positive_finite(&[
            darcy_friction_factor,
            nusselt_number,
            film_heat_transfer_coefficient_w_per_m2_k,
            pressure_drop_pa,
        ]) {
            return Err(error(
                "numeric_range",
                format!("{name} film coefficient and pressure drop must be positive and finite"),
            ));
        }
        Ok(PassagePerformance {
            heat_capacity_rate_w_per_k,
            reynolds_number: re,
            prandtl_number: pr,
            nusselt_number,
            darcy_friction_factor,
            film_heat_transfer_coefficient_w_per_m2_k,
            pressure_drop_pa,
        })
    }
}

fn validate_liquid_side(side: LiquidSide<'_>, kind: PassageKind) -> RfResult<()> {
    let name = kind.name();
    let p = side.properties;
    if p.source.trim().is_empty()
        || !positive_finite(&[
            p.molar_mass_kg_per_mol,
            p.molar_heat_capacity_j_per_mol_k,
            p.density_kg_per_m3,
            p.dynamic_viscosity_pa_s,
            p.thermal_conductivity_w_per_m_k,
        ])
    {
        return Err(error(
            "invalid_properties",
            format!("{name} requires positive finite liquid properties and their source"),
        ));
    }
    let domain = p.domain;
    if !positive_finite(&[
        domain.min_temperature_k,
        domain.max_temperature_k,
        domain.min_pressure_pa,
        domain.max_pressure_pa,
    ]) || domain.min_temperature_k > domain.max_temperature_k
        || domain.min_pressure_pa > domain.max_pressure_pa
    {
        return Err(error(
            "invalid_property_domain",
            format!(
                "{name} requires ordered positive finite temperature and absolute pressure bounds"
            ),
        ));
    }
    if !positive_finite(&[side.inlet.molar_flow_mol_s]) {
        return Err(error(
            "invalid_flow",
            format!("{name} requires positive finite inlet molar flow"),
        ));
    }
    if !domain.contains(side.inlet) {
        return Err(error(
            "inlet_outside_domain",
            format!(
                "{name} inlet at {} K and {} Pa is outside the liquid domain",
                side.inlet.temperature_k, side.inlet.pressure_pa
            ),
        ));
    }
    Ok(())
}

impl LiquidPropertyDomain {
    fn contains(self, state: LiquidState) -> bool {
        (self.min_temperature_k..=self.max_temperature_k).contains(&state.temperature_k)
            && (self.min_pressure_pa..=self.max_pressure_pa).contains(&state.pressure_pa)
    }
}

fn countercurrent_effectiveness(ntu: f64, capacity_ratio: f64) -> f64 {
    let difference = 1.0 - capacity_ratio;
    if difference == 0.0 {
        return ntu / (1.0 + ntu);
    }
    let numerator = -(-ntu * difference).exp_m1();
    numerator / (difference + capacity_ratio * numerator)
}

fn rate_outlet(
    side: LiquidSide<'_>,
    passage: PassagePerformance,
    heat_gain_w: f64,
    kind: PassageKind,
) -> RfResult<SideRating> {
    let heat_capacity = passage.heat_capacity_rate_w_per_k;
    let outlet = LiquidState {
        temperature_k: side.inlet.temperature_k + heat_gain_w / heat_capacity,
        pressure_pa: side.inlet.pressure_pa - passage.pressure_drop_pa,
        molar_flow_mol_s: side.inlet.molar_flow_mol_s,
    };
    if !side.properties.domain.contains(outlet) {
        return Err(error(
            "outlet_outside_domain",
            format!(
                "{} outlet at {} K and {} Pa is outside the liquid domain",
                kind.name(),
                outlet.temperature_k,
                outlet.pressure_pa
            ),
        ));
    }
    let sensible_energy_residual_w =
        heat_capacity * (outlet.temperature_k - side.inlet.temperature_k) - heat_gain_w;
    Ok(SideRating {
        outlet,
        passage,
        sensible_energy_residual_w,
    })
}

fn positive_finite(values: &[f64]) -> bool {
    values.iter().all(|value| value.is_finite() && *value > 0.0)
}

fn error(code: &str, message: impl Into<String>) -> RfError {
    RfError::invalid_input(message)
        .with_diagnostic_code(format!("unit.heat_exchanger.concentric_tube.{code}"))
}

#[cfg(test)]
mod tests;
