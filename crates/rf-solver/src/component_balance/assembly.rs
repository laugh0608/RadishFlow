use rf_flowsheet::validate_connections;
use rf_model::{Flowsheet, MaterialStreamState, UnitNode};
use rf_types::{ComponentId, DiagnosticPortTarget};
use rf_unitops::{
    BuiltinUnitKind,
    catalog::builtin_model,
    component_balance::{
        ComponentBalanceCapability, MixerBalancePort, stream_component_molar_flows,
    },
    specifications::UnitParameter,
};

use super::*;

pub(super) fn assemble(
    flowsheet: &Flowsheet,
    specification: &MixerBalanceSpecification,
) -> Result<MixerComponentBalanceProblem, ComponentBalanceError> {
    let topology_error = || ComponentBalanceError::UnsupportedTopology {
        mixer_id: specification.mixer_id.clone(),
    };
    for port in MixerBalancePort::ALL {
        let scale = specification.scales.for_port(port);
        if !scale.is_finite() || scale <= 0.0 {
            return Err(ComponentBalanceError::InvalidScale { port });
        }
    }
    let components = flowsheet.components.keys().cloned().collect::<Vec<_>>();
    if components.is_empty() {
        return Err(ComponentBalanceError::NoComponents);
    }
    let mut feeds = 0;
    let mut mixers = 0;
    for unit in flowsheet.units.values() {
        match BuiltinUnitKind::parse(&unit.kind)
            .and_then(|kind| builtin_model(kind).component_balance_capability())
        {
            Some(ComponentBalanceCapability::FixedFeedBoundary) => {
                feeds += 1;
                reject_parameter(unit, UnitParameter::OutletTemperature)?;
                reject_parameter(unit, UnitParameter::OutletPressure)?;
            }
            Some(ComponentBalanceCapability::TwoInletMixing) => {
                mixers += 1;
                reject_parameter(unit, UnitParameter::OutletTemperature)?;
            }
            None => {
                return Err(ComponentBalanceError::UnsupportedModel {
                    unit_id: unit.id.clone(),
                    kind: unit.kind.clone(),
                });
            }
        }
    }
    let mixer = flowsheet.unit(&specification.mixer_id)?;
    if feeds != 2 || mixers != 1 || mixer.kind != rf_unitops::MIXER_KIND {
        return Err(topology_error());
    }
    // Reuse canonical signatures and one-source/one-sink checks. No new graph rules.
    let connections = validate_connections(flowsheet)?;
    if connections.len() != 3 {
        return Err(topology_error());
    }
    let mut inlets = Vec::with_capacity(2);
    for port in [MixerBalancePort::InletA, MixerBalancePort::InletB] {
        let connection = connections
            .iter()
            .find(|connection| {
                connection
                    .sink
                    .as_ref()
                    .is_some_and(|sink| sink.unit_id == mixer.id && sink.port_name == port.name())
            })
            .ok_or_else(topology_error)?;
        if flowsheet.unit(&connection.source.unit_id)?.kind != rf_unitops::FEED_KIND {
            return Err(topology_error());
        }
        let stream = flowsheet.stream(&connection.stream_id)?;
        validate_components(stream, &components)?;
        let component_flows = stream_component_molar_flows(stream).map_err(|error| {
            ComponentBalanceError::Model(
                error
                    .with_related_stream_id(stream.id.clone())
                    .with_related_unit_id(mixer.id.clone())
                    .with_related_port_target(DiagnosticPortTarget::new(
                        mixer.id.clone(),
                        port.name(),
                    )),
            )
        })?;
        inlets.push((connection, stream, component_flows));
    }
    let outlet = connections
        .iter()
        .find(|connection| {
            connection.source.unit_id == mixer.id
                && connection.source.port_name == MixerBalancePort::Outlet.name()
                && connection.sink.is_none()
        })
        .ok_or_else(topology_error)?;
    let reference = inlets[0].1;
    for (_, stream, _) in &inlets {
        if !stream.temperature_k.is_finite()
            || stream.temperature_k <= 0.0
            || !stream.pressure_pa.is_finite()
            || stream.pressure_pa <= 0.0
            || stream.temperature_k != reference.temperature_k
            || stream.pressure_pa != reference.pressure_pa
        {
            return Err(ComponentBalanceError::UnsupportedThermalBoundary {
                stream_id: stream.id.clone(),
            });
        }
    }
    if mixer
        .parameters
        .outlet_pressure_pa
        .is_some_and(|p| p != reference.pressure_pa)
    {
        return Err(ComponentBalanceError::UnsupportedSpecification {
            unit_id: mixer.id.clone(),
            parameter: UnitParameter::OutletPressure,
        });
    }
    let expected_total = rf_unitops::component_balance::mixer_component_outlet(
        inlets[0].1.total_molar_flow_mol_s,
        inlets[1].1.total_molar_flow_mol_s,
    );
    if !expected_total.is_finite() || expected_total <= 0.0 {
        return Err(ComponentBalanceError::InvalidOutletTotal {
            mixer_id: mixer.id.clone(),
        });
    }
    if let Some(specified) = specification.outlet_total_molar_flow_mol_s {
        if !specified.is_finite() || specified < 0.0 {
            return Err(ComponentBalanceError::InvalidOutletSpecification);
        }
        // Specification consistency is independent of user-selected numerical scales.
        if (specified - expected_total).abs() <= COMPONENT_BALANCE_ABSOLUTE_TOLERANCE_MOL_S {
            return Err(ComponentBalanceError::RedundantOutletSpecification {
                mixer_id: mixer.id.clone(),
                expected_mol_s: expected_total,
            });
        }
        return Err(ComponentBalanceError::ConflictingOutletSpecification {
            mixer_id: mixer.id.clone(),
            expected_mol_s: expected_total,
            specified_mol_s: specified,
        });
    }

    let mut variables = Vec::with_capacity(3 * components.len());
    let mut rows = Vec::with_capacity(3 * components.len());
    let mut fixed = Vec::with_capacity(2 * components.len());
    for (port_position, port) in MixerBalancePort::ALL.into_iter().enumerate() {
        let stream_id = match port {
            MixerBalancePort::Outlet => &outlet.stream_id,
            _ => &inlets[port_position].0.stream_id,
        };
        for component in &components {
            let target = ComponentFlowBinding {
                unit_id: mixer.id.clone(),
                port: ComponentBalancePort::Mixer(port),
                stream_id: stream_id.clone(),
                component_id: component.clone(),
            };
            let equation = if port == MixerBalancePort::Outlet {
                ComponentBalanceEquation::MixerConservation {
                    unit_id: mixer.id.clone(),
                    component_id: component.clone(),
                }
            } else {
                let (connection, _, component_flows) = &inlets[port_position];
                // A registered component omitted from a sparse inlet composition is zero.
                fixed.push(component_flows.get(component).copied().unwrap_or(0.0));
                ComponentBalanceEquation::Connection {
                    source: ComponentFlowBinding {
                        unit_id: connection.source.unit_id.clone(),
                        port: ComponentBalancePort::FeedOutlet,
                        stream_id: stream_id.clone(),
                        component_id: component.clone(),
                    },
                    target: target.clone(),
                }
            };
            let scale = specification.scales.for_port(port);
            variables.push(ComponentFlowVariable {
                binding: target,
                scale_mol_s: scale,
            });
            rows.push(ComponentBalanceRow {
                equation,
                scale_mol_s: scale,
            });
        }
    }
    let n = components.len();
    let mut jacobian = vec![vec![0.0; 3 * n]; 3 * n];
    for (i, row) in jacobian.iter_mut().take(2 * n).enumerate() {
        row[i] = 1.0;
    }
    for i in 0..n {
        for (position, port) in MixerBalancePort::ALL.into_iter().enumerate() {
            jacobian[2 * n + i][position * n + i] = port.residual_derivative();
        }
    }
    let mut scaled_jacobian = jacobian.clone();
    for (i, row) in scaled_jacobian.iter_mut().enumerate() {
        for (j, coefficient) in row.iter_mut().enumerate() {
            *coefficient = *coefficient * variables[j].scale_mol_s / rows[i].scale_mol_s;
            if !coefficient.is_finite() {
                return Err(ComponentBalanceError::NonFiniteScaling {
                    equation: Box::new(rows[i].equation.clone()),
                });
            }
        }
    }
    Ok(MixerComponentBalanceProblem {
        mixer_id: mixer.id.clone(),
        components,
        variables,
        rows,
        fixed_inlet_flows_mol_s: fixed,
        jacobian_si: jacobian,
        scaled_jacobian,
    })
}

fn reject_parameter(
    unit: &UnitNode,
    parameter: UnitParameter,
) -> Result<(), ComponentBalanceError> {
    if parameter.value(&unit.parameters).is_some() {
        Err(ComponentBalanceError::UnsupportedSpecification {
            unit_id: unit.id.clone(),
            parameter,
        })
    } else {
        Ok(())
    }
}

fn validate_components(
    stream: &MaterialStreamState,
    components: &[ComponentId],
) -> Result<(), ComponentBalanceError> {
    for component in stream.overall_mole_fractions.keys() {
        if !components.contains(component) {
            return Err(ComponentBalanceError::UnknownComponent {
                stream_id: stream.id.clone(),
                component_id: component.clone(),
            });
        }
    }
    Ok(())
}
