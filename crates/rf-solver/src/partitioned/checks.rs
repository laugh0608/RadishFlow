use super::*;
pub(super) use crate::fixed_tp_material::{
    component_flows, converged, residuals, sample, validate_vector,
};

pub(super) fn balance(
    plan: &PartitionedSolvePlan,
    inputs: &[MaterialStreamState],
    outputs: &[MaterialStreamState],
    scales: &BTreeMap<ComponentId, f64>,
) -> RfResult<()> {
    crate::fixed_tp_material::balance(
        &plan.components,
        plan.options.boundary,
        inputs,
        outputs,
        scales,
    )
}

pub(super) fn guess(
    plan: &PartitionedSolvePlan,
    recycle: &RecyclePlan,
    fresh: &MaterialStreamState,
    values: &BTreeMap<ComponentId, f64>,
) -> RfResult<MaterialStreamState> {
    validate_vector(&plan.components, values, false)?;
    let total = values.values().sum::<f64>();
    if !total.is_finite() {
        return Err(failure("flow_overflow", "tear component sum overflows"));
    }
    // Explicit zero-flow rule: fresh boundary composition, never an outlet template.
    let composition = if total == 0.0 {
        fresh.overall_mole_fractions.clone()
    } else {
        values
            .iter()
            .map(|(id, n)| (id.clone(), n / total))
            .collect()
    };
    let template = &plan.flowsheet.streams[&recycle.tear.stream_id];
    Ok(MaterialStreamState::from_tpzf(
        template.id.clone(),
        template.name.clone(),
        plan.options.boundary.temperature_k,
        plan.options.boundary.pressure_pa,
        total,
        composition,
    ))
}
