use rf_solver::simultaneous::ModularOptions;
use rf_types::{RfError, RfResult};

/// Shape validation permits a saved, editable plan whose graph is not ready yet.
pub(super) fn validate_configuration(options: &ModularOptions) -> RfResult<()> {
    let boundary = &options.boundary;
    if !boundary.temperature_k.is_finite()
        || boundary.temperature_k <= 0.0
        || !boundary.pressure_pa.is_finite()
        || boundary.pressure_pa <= 0.0
    {
        return Err(RfError::invalid_input("fixed T/P must be positive finite")
            .with_diagnostic_code("flow.plan.modular_boundary"));
    }
    for (b, s) in options.scales.iter().flat_map(|s| s.iter()) {
        if [
            b.stream.as_str(),
            b.source.unit_id.as_str(),
            &b.source.port_name,
            b.sink.unit_id.as_str(),
            &b.sink.port_name,
            b.component.as_str(),
        ]
        .iter()
        .any(|id| id.trim().is_empty())
            || !s.variable_mol_s.is_finite()
            || s.variable_mol_s <= 0.0
            || !s.residual_mol_s.is_finite()
            || s.residual_mol_s <= 0.0
        {
            return Err(RfError::invalid_input(
                "coupling identities must be nonblank and both scales positive finite",
            )
            .with_diagnostic_code("flow.plan.modular_scale")
            .with_related_stream_id(b.stream.clone())
            .with_related_component_id(b.component.clone()));
        }
    }
    Ok(())
}
