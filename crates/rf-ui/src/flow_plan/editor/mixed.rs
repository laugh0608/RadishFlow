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
