use super::*;
use rf_model::{
    engineering::EngineeringBasis,
    equipment::{
        CONCENTRIC_TUBE_RATING_METHOD_ID, CONCENTRIC_TUBE_RATING_METHOD_VERSION,
        ConcentricTubeRatingTask, ConcentricTubeSideInputs,
    },
};
use rf_types::SourceId;

/// Run the frozen H3 method from formal inputs and their shared source basis.
pub fn rate_task(
    task: &ConcentricTubeRatingTask,
    basis: &EngineeringBasis,
) -> RfResult<ConcentricTubeRating> {
    if task.method.id != CONCENTRIC_TUBE_RATING_METHOD_ID
        || task.method.version != CONCENTRIC_TUBE_RATING_METHOD_VERSION
    {
        return Err(error(
            "unsupported_method",
            "unsupported concentric-tube rating method",
        ));
    }
    basis
        .validate()
        .map_err(|e| error("invalid_basis", e.to_string()))?;
    let geometry = &task.geometry;
    if let Some(id) = &geometry.source_id {
        source(basis, id, "geometry")?;
    }
    let input = |value, field| required(value, "geometry", field);
    let geometry = ConcentricTubeGeometry {
        active_length_m: input(geometry.active_length_m, "activeLengthM")?,
        inner_tube_inner_diameter_m: input(
            geometry.inner_tube_inner_diameter_m,
            "innerTubeInnerDiameterM",
        )?,
        inner_tube_outer_diameter_m: input(
            geometry.inner_tube_outer_diameter_m,
            "innerTubeOuterDiameterM",
        )?,
        outer_tube_inner_diameter_m: input(
            geometry.outer_tube_inner_diameter_m,
            "outerTubeInnerDiameterM",
        )?,
        wall_thermal_conductivity_w_per_m_k: input(
            geometry.wall_thermal_conductivity_w_per_m_k,
            "wallThermalConductivityWPerMK",
        )?,
    };
    let case = &task.case;
    rate_countercurrent(
        geometry,
        side(&case.hot, basis, "case.hot")?,
        side(&case.cold, basis, "case.cold")?,
        RatingLimits {
            min_heat_duty_w: case.limits.min_heat_duty_w,
            max_hot_pressure_drop_pa: case.limits.max_hot_pressure_drop_pa,
            max_cold_pressure_drop_pa: case.limits.max_cold_pressure_drop_pa,
        },
    )
}

fn side<'a>(
    side: &ConcentricTubeSideInputs,
    basis: &'a EngineeringBasis,
    path: &str,
) -> RfResult<LiquidSide<'a>> {
    let inlet = &side.inlet;
    let props = &side.properties;
    let domain = &props.liquid_domain;
    let input = |value, field| required(value, path, field);
    Ok(LiquidSide {
        inlet: LiquidState {
            temperature_k: input(inlet.temperature_k, "inlet.temperatureK")?,
            pressure_pa: input(inlet.pressure_pa, "inlet.pressurePa")?,
            molar_flow_mol_s: input(inlet.molar_flow_mol_s, "inlet.molarFlowMolS")?,
        },
        properties: ConstantLiquidProperties {
            molar_mass_kg_per_mol: input(
                props.molar_mass_kg_per_mol,
                "properties.molarMassKgPerMol",
            )?,
            molar_heat_capacity_j_per_mol_k: input(
                props.molar_heat_capacity_j_per_mol_k,
                "properties.molarHeatCapacityJPerMolK",
            )?,
            density_kg_per_m3: input(props.density_kg_per_m3, "properties.densityKgPerM3")?,
            dynamic_viscosity_pa_s: input(
                props.dynamic_viscosity_pa_s,
                "properties.dynamicViscosityPaS",
            )?,
            thermal_conductivity_w_per_m_k: input(
                props.thermal_conductivity_w_per_m_k,
                "properties.thermalConductivityWPerMK",
            )?,
            domain: LiquidPropertyDomain {
                min_temperature_k: input(
                    domain.min_temperature_k,
                    "properties.liquidDomain.minTemperatureK",
                )?,
                max_temperature_k: input(
                    domain.max_temperature_k,
                    "properties.liquidDomain.maxTemperatureK",
                )?,
                min_pressure_pa: input(
                    domain.min_pressure_pa,
                    "properties.liquidDomain.minPressurePa",
                )?,
                max_pressure_pa: input(
                    domain.max_pressure_pa,
                    "properties.liquidDomain.maxPressurePa",
                )?,
            },
            source: source(basis, &props.source_id, &format!("{path}.properties"))?,
        },
    })
}

fn required(value: Option<f64>, path: &str, field: &str) -> RfResult<f64> {
    value.ok_or_else(|| {
        error(
            "missing_input",
            format!("{path}.{field}: required for rating"),
        )
    })
}

fn source<'a>(basis: &'a EngineeringBasis, id: &SourceId, path: &str) -> RfResult<&'a str> {
    basis
        .sources
        .iter()
        .find(|source| source.id == *id)
        .map(|source| source.id.as_str())
        .ok_or_else(|| {
            error(
                "missing_source",
                format!("{path}.sourceId: missing source {id}"),
            )
        })
}
