//! Execution-stage projection of the shared local specification checks. Keep the
//! existing failure code, message and target ordering for CLI / FFI consumers.

use rf_unitops::specifications::{
    InletPressure, ParameterViolation, UnitParameter, check_parameter_value, inlet_pressure_limit,
    input_specification,
};

use super::*;

pub(super) fn validate_step_parameters(
    step_number: usize,
    unit: &UnitNode,
    inputs: &UnitOperationInputs,
    consumed_stream_ids: &[StreamId],
) -> RfResult<()> {
    let Some(kind) = BuiltinUnitKind::parse(&unit.kind) else {
        return Ok(()); // Unsupported kinds are rejected by topology / instantiation.
    };
    let spec = input_specification(kind);
    for &parameter in spec.parameters {
        let Some(value) = parameter.value(&unit.parameters) else {
            continue; // The execution API retains its existing default semantics.
        };
        if check_parameter_value(kind, parameter, value, None).is_err() {
            let requirement = match parameter {
                UnitParameter::OutletTemperature => format!(
                    "{} `{value}` K must be a positive finite temperature",
                    parameter.key()
                ),
                UnitParameter::OutletPressure => format!(
                    "{} `{value}` Pa must be a positive finite pressure",
                    parameter.key()
                ),
                UnitParameter::OutletASplitFraction => {
                    format!("{} `{value}` must be finite in [0, 1]", parameter.key())
                }
            };
            return Err(solver_step_invalid_input_with_context(
                step_number,
                unit,
                SolverDiagnosticCode::StepParameter,
                requirement,
                unit_parameter_related_stream_ids(unit, consumed_stream_ids),
                outlet_port_targets(unit),
            ));
        }
    }

    let Some(pressure) = unit.parameters.outlet_pressure_pa else {
        return Ok(());
    };
    let inlets = spec
        .outlet_pressure_rule
        .inlet_ports()
        .iter()
        .filter_map(|&port| {
            inputs.stream(port).map(|stream| InletPressure {
                port,
                stream_id: stream.id.clone(),
                pressure_pa: stream.pressure_pa,
            })
        })
        .collect::<Vec<_>>();
    let limit = inlet_pressure_limit(spec.outlet_pressure_rule, &inlets);
    if check_parameter_value(
        kind,
        UnitParameter::OutletPressure,
        pressure,
        limit.as_ref(),
    ) != Err(ParameterViolation::AboveInletPressure)
    {
        return Ok(());
    }
    let limit = limit.expect("a pressure conflict has a complete inlet limit");
    let mut targets = outlet_port_targets(unit);
    targets.extend(
        spec.outlet_pressure_rule
            .inlet_ports()
            .iter()
            .map(|port| DiagnosticPortTarget::new(unit.id.clone(), (*port).to_string())),
    );
    Err(solver_step_invalid_input_with_context(
        step_number,
        unit,
        SolverDiagnosticCode::StepParameter,
        format!(
            "outlet_pressure_pa `{pressure}` Pa cannot exceed {} `{}` Pa from stream `{}`",
            spec.outlet_pressure_rule.description(),
            limit.pressure_pa,
            limit.limiting_stream_id,
        ),
        unit_parameter_related_stream_ids(unit, consumed_stream_ids),
        targets,
    ))
}

fn unit_parameter_related_stream_ids(
    unit: &UnitNode,
    consumed_stream_ids: &[StreamId],
) -> Vec<StreamId> {
    dedupe_stream_ids(
        consumed_stream_ids
            .iter()
            .cloned()
            .chain(
                unit.ports
                    .iter()
                    .filter(|port| port.direction == PortDirection::Outlet)
                    .filter_map(|port| port.stream_id.clone()),
            )
            .collect(),
    )
}

fn outlet_port_targets(unit: &UnitNode) -> Vec<DiagnosticPortTarget> {
    unit.ports
        .iter()
        .filter(|port| port.direction == PortDirection::Outlet)
        .map(|port| DiagnosticPortTarget::new(unit.id.clone(), port.name.clone()))
        .collect()
}
