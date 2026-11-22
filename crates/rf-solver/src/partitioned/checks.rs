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
    crate::fixed_tp_material::tear_stream(
        &plan.flowsheet.streams[&recycle.tear.stream_id],
        fresh,
        plan.options.boundary,
        values,
    )
}
