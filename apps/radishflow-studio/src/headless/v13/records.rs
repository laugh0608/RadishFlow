//! SI wire projections for the v13 H3 protocol.

use super::super::v3::{protocol::ProjectSnapshot, records};
use rf_model::{
    engineering::EngineeringBasis,
    equipment::{ConcentricTubeRatingTask, ConcentricTubeSideInputs},
};
use rf_types::SourceId;
use rf_ui::engineering::concentric_tube::{
    ConcentricTubeRating, LimitCheck, RatingRevisions, SideRating,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(in crate::headless) fn revisions(revisions: RatingRevisions) -> Value {
    json!({"task": revisions.task, "geometry": revisions.geometry, "case": revisions.case})
}

pub(in crate::headless) fn object(task: &ConcentricTubeRatingTask) -> Value {
    json!({
        "kind": "concentric_tube_rating",
        "task_id": task.id,
        "case_id": task.case.id,
        "tag": task.tag,
        "name": task.name,
        "case_name": task.case.name,
        "method": records::method(&task.method),
        "revisions": revisions(task.into()),
        "target": {"case": {"task": {"id": task.id}, "case": {"id": task.case.id}}},
    })
}

pub(in crate::headless) fn record(
    task: &ConcentricTubeRatingTask,
    basis: &EngineeringBasis,
) -> Value {
    let geometry = &task.geometry;
    let case = &task.case;
    let sources: BTreeSet<&SourceId> = geometry
        .source_id
        .iter()
        .chain([
            &case.hot.properties.source_id,
            &case.cold.properties.source_id,
        ])
        .collect();
    json!({
        "object": object(task),
        "geometry": {
            "source_id": geometry.source_id,
            "active_length_m": geometry.active_length_m,
            "inner_tube_inner_diameter_m": geometry.inner_tube_inner_diameter_m,
            "inner_tube_outer_diameter_m": geometry.inner_tube_outer_diameter_m,
            "outer_tube_inner_diameter_m": geometry.outer_tube_inner_diameter_m,
            "wall_thermal_conductivity_w_per_m_k": geometry.wall_thermal_conductivity_w_per_m_k,
        },
        "case": {
            "purpose": case.purpose,
            "hot": side_inputs(&case.hot),
            "cold": side_inputs(&case.cold),
            "limits": {
                "min_heat_duty_w": case.limits.min_heat_duty_w,
                "max_hot_pressure_drop_pa": case.limits.max_hot_pressure_drop_pa,
                "max_cold_pressure_drop_pa": case.limits.max_cold_pressure_drop_pa,
            },
        },
        "sources": basis.sources.iter().filter(|source| sources.contains(&source.id)).map(records::source).collect::<Vec<_>>(),
    })
}

fn side_inputs(side: &ConcentricTubeSideInputs) -> Value {
    let fluid = &side.properties;
    let domain = &fluid.liquid_domain;
    json!({
        "inlet": {
            "temperature_k": side.inlet.temperature_k,
            "pressure_pa": side.inlet.pressure_pa,
            "molar_flow_mol_s": side.inlet.molar_flow_mol_s,
        },
        "properties": {
            "source_id": fluid.source_id,
            "molar_mass_kg_per_mol": fluid.molar_mass_kg_per_mol,
            "molar_heat_capacity_j_per_mol_k": fluid.molar_heat_capacity_j_per_mol_k,
            "density_kg_per_m3": fluid.density_kg_per_m3,
            "dynamic_viscosity_pa_s": fluid.dynamic_viscosity_pa_s,
            "thermal_conductivity_w_per_m_k": fluid.thermal_conductivity_w_per_m_k,
            "liquid_domain": {
                "min_temperature_k": domain.min_temperature_k,
                "max_temperature_k": domain.max_temperature_k,
                "min_pressure_pa": domain.min_pressure_pa,
                "max_pressure_pa": domain.max_pressure_pa,
            },
        },
    })
}

pub(in crate::headless) fn origin(
    project: &ProjectSnapshot,
    task: &ConcentricTubeRatingTask,
    step_id: &str,
) -> Value {
    json!({
        "document_id": project.document_id,
        "file_sha256": project.file_sha256,
        "task_id": task.id,
        "case_id": task.case.id,
        "step_id": step_id,
        "revisions": revisions(task.into()),
        "method": records::method(&task.method),
        "source_ids": {
            "geometry": task.geometry.source_id,
            "hot_properties": task.case.hot.properties.source_id,
            "cold_properties": task.case.cold.properties.source_id,
        },
    })
}

pub(in crate::headless) fn current_result(result: ConcentricTubeRating) -> Value {
    json!({
        "status": "current",
        "heat_duty_w": result.heat_duty_w,
        "ua_w_per_k": result.ua_w_per_k,
        "overall_heat_transfer_coefficient_outer_w_per_m2_k": result.overall_heat_transfer_coefficient_outer_w_per_m2_k,
        "inner_heat_transfer_area_m2": result.inner_heat_transfer_area_m2,
        "outer_heat_transfer_area_m2": result.outer_heat_transfer_area_m2,
        "effectiveness": result.effectiveness,
        "hot_inlet_approach_k": result.hot_inlet_approach_k,
        "cold_inlet_approach_k": result.cold_inlet_approach_k,
        "lmtd_k": result.lmtd_k,
        "overall_sensible_energy_residual_w": result.overall_sensible_energy_residual_w,
        "lmtd_heat_duty_residual_w": result.lmtd_heat_duty_residual_w,
        "hot": side_result(result.hot),
        "cold": side_result(result.cold),
        "checks": {
            "heat_duty": limit(result.checks.heat_duty),
            "hot_pressure_drop": limit(result.checks.hot_pressure_drop),
            "cold_pressure_drop": limit(result.checks.cold_pressure_drop),
        },
    })
}

fn side_result(side: SideRating) -> Value {
    let passage = side.passage;
    json!({
        "outlet": {
            "temperature_k": side.outlet.temperature_k,
            "pressure_pa": side.outlet.pressure_pa,
            "molar_flow_mol_s": side.outlet.molar_flow_mol_s,
        },
        "passage": {
            "pressure_drop_pa": passage.pressure_drop_pa,
            "heat_capacity_rate_w_per_k": passage.heat_capacity_rate_w_per_k,
            "reynolds_number": passage.reynolds_number,
            "prandtl_number": passage.prandtl_number,
            "nusselt_number": passage.nusselt_number,
            "darcy_friction_factor": passage.darcy_friction_factor,
            "film_heat_transfer_coefficient_w_per_m2_k": passage.film_heat_transfer_coefficient_w_per_m2_k,
        },
        "sensible_energy_residual_w": side.sensible_energy_residual_w,
    })
}

fn limit(check: LimitCheck) -> &'static str {
    match check {
        LimitCheck::NotSpecified => "not_specified",
        LimitCheck::Passed => "passed",
        LimitCheck::Failed => "failed",
    }
}
