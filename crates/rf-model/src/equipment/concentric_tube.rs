//! Formal SI inputs for one independent concentric-tube rating.

use super::EquipmentMethod;
use rf_types::{EngineeringCaseId, EquipmentTaskId, SourceId};

pub const CONCENTRIC_TUBE_RATING_METHOD_ID: &str =
    "radishflow.heat-exchanger.concentric-tube-rating";
pub const CONCENTRIC_TUBE_RATING_METHOD_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct ConcentricTubeRatingTask {
    pub id: EquipmentTaskId,
    pub revision: u64,
    pub tag: String,
    pub name: String,
    pub method: EquipmentMethod,
    pub geometry: ConcentricTubeGivenGeometry,
    pub case: ConcentricTubeRatingCase,
}

/// A missing numeric input is an incomplete formal record, not a default.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConcentricTubeGivenGeometry {
    pub revision: u64,
    pub source_id: Option<SourceId>,
    pub active_length_m: Option<f64>,
    pub inner_tube_inner_diameter_m: Option<f64>,
    pub inner_tube_outer_diameter_m: Option<f64>,
    pub outer_tube_inner_diameter_m: Option<f64>,
    pub wall_thermal_conductivity_w_per_m_k: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConcentricTubeRatingCase {
    pub id: EngineeringCaseId,
    pub revision: u64,
    pub name: String,
    pub purpose: String,
    pub hot: ConcentricTubeSideInputs,
    pub cold: ConcentricTubeSideInputs,
    pub limits: ConcentricTubeRatingLimits,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConcentricTubeSideInputs {
    pub inlet: ConcentricTubeLiquidInlet,
    pub properties: ConcentricTubeLiquidProperties,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ConcentricTubeLiquidInlet {
    pub temperature_k: Option<f64>,
    pub pressure_pa: Option<f64>,
    pub molar_flow_mol_s: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConcentricTubeLiquidProperties {
    pub source_id: SourceId,
    pub molar_mass_kg_per_mol: Option<f64>,
    pub molar_heat_capacity_j_per_mol_k: Option<f64>,
    pub density_kg_per_m3: Option<f64>,
    pub dynamic_viscosity_pa_s: Option<f64>,
    pub thermal_conductivity_w_per_m_k: Option<f64>,
    pub liquid_domain: ConcentricTubeLiquidDomain,
}

/// Caller-declared single-liquid temperature and absolute-pressure bounds.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ConcentricTubeLiquidDomain {
    pub min_temperature_k: Option<f64>,
    pub max_temperature_k: Option<f64>,
    pub min_pressure_pa: Option<f64>,
    pub max_pressure_pa: Option<f64>,
}

/// Missing limits are intentionally unchecked.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ConcentricTubeRatingLimits {
    pub min_heat_duty_w: Option<f64>,
    pub max_hot_pressure_drop_pa: Option<f64>,
    pub max_cold_pressure_drop_pa: Option<f64>,
}
