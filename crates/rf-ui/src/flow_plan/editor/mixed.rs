use super::*;
use rf_solver::mixed::{MixedBlockMethod, MixedOptions};
pub(super) fn apply(
    o: &mut MixedOptions,
    field: &FlowPlanNumericField,
    value: f64,
) -> RfResult<()> {
    use FlowPlanNumericField::*;
    let unavailable = || RfError::invalid_input("mixed block numeric binding is unavailable");
    match field {
        Temperature => o.boundary.temperature_k = value,
        Pressure => o.boundary.pressure_pa = value,
        IterationBudget => {
            o.single_recycle_mut()
                .ok_or_else(unavailable)?
                .max_iteration_evaluations = iteration_budget(value)?;
        }
        RegionIterationBudget(stream) => {
            let Some(rf_solver::mixed::MixedRecycleConfiguration::Serial(regions)) =
                o.recycle.as_deref_mut()
            else {
                return Err(unavailable());
            };
            regions
                .iter_mut()
                .find(|r| &r.tear.stream_id == stream)
                .ok_or_else(unavailable)?
                .max_iteration_evaluations = iteration_budget(value)?;
        }
        InitialComponent { stream, component } => {
            let number = o
                .recycle_regions_mut()
                .iter_mut()
                .find(|recycle| &recycle.tear.stream_id == stream)
                .and_then(|recycle| recycle.tear.initial_component_flows_mol_s.as_mut())
                .and_then(|values| values.get_mut(component))
                .ok_or_else(unavailable)?;
            *number = value;
        }
        MixedVariableScale { block, binding } => {
            let b = o
                .blocks
                .iter_mut()
                .find(|b| &b.id == block)
                .ok_or_else(unavailable)?;
            let MixedBlockMethod::EquationOriented { scales: Some(s) } = &mut b.method else {
                return Err(unavailable());
            };
            *s.variables.get_mut(binding).ok_or_else(unavailable)? = value;
        }
        MixedResidualScale { block, binding } => {
            let b = o
                .blocks
                .iter_mut()
                .find(|b| &b.id == block)
                .ok_or_else(unavailable)?;
            let MixedBlockMethod::EquationOriented { scales: Some(s) } = &mut b.method else {
                return Err(unavailable());
            };
            *s.equations.get_mut(binding).ok_or_else(unavailable)? = value;
        }
        _ => return Err(unavailable()),
    }
    Ok(())
}
