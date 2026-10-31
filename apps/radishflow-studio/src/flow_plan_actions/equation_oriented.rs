use rf_solver::equation_oriented::*;
use rf_store::project_v4::*;
use rf_types::{RfError, RfResult, StreamId};

pub(super) fn stored(o: &EquationOptions) -> StoredEquationPlan {
    let mut specifications = Vec::new();
    if let Some(r) = &o.released {
        specifications.push(StoredEquationSpecification::ReleaseSplitFraction {
            unit_id: r.binding.unit.clone(),
            field: StoredEquationParameter::OutletASplitFraction,
            interval: StoredFractionInterval {
                lower: r.lower,
                upper: r.upper,
                max_iterations: r.max_iterations,
            },
        });
    }
    if let Some(t) = &o.target {
        specifications.push(StoredEquationSpecification::ProductTotalMolarFlow {
            stream_id: t.source.stream.clone(),
            source: stored_port(&t.source),
            value_mol_s: t.value_mol_s,
        });
    }
    StoredEquationPlan {
        fixed_boundary: StoredFixedBoundary {
            temperature_k: o.boundary.temperature_k,
            pressure_pa: o.boundary.pressure_pa,
        },
        specifications,
        scaling: match &o.scales {
            None => StoredEquationScaling::EvaluatedFreshTotal,
            Some(s) => StoredEquationScaling::ExplicitEquations {
                variables: s
                    .variables
                    .iter()
                    .map(|(b, v)| StoredEquationScale {
                        binding: stored_variable(b),
                        scale_si: *v,
                    })
                    .collect(),
                equations: s
                    .equations
                    .iter()
                    .map(|(b, v)| StoredEquationScale {
                        binding: stored_equation(b),
                        scale_si: *v,
                    })
                    .collect(),
            },
        },
    }
    .canonicalized()
}
pub(super) fn configuration(plan: StoredEquationPlan) -> RfResult<EquationOptions> {
    plan.validate()?;
    let mut o = EquationOptions::new(FixedBoundary {
        temperature_k: plan.fixed_boundary.temperature_k,
        pressure_pa: plan.fixed_boundary.pressure_pa,
    });
    for spec in plan.specifications {
        match spec {
            StoredEquationSpecification::ReleaseSplitFraction {
                unit_id, interval, ..
            } => {
                o.released = Some(ReleasedSplitFraction {
                    binding: ParameterBinding {
                        unit: unit_id,
                        field: ParameterField::OutletASplitFraction,
                    },
                    lower: interval.lower,
                    upper: interval.upper,
                    max_iterations: interval.max_iterations,
                })
            }
            StoredEquationSpecification::ProductTotalMolarFlow {
                stream_id,
                source,
                value_mol_s,
            } => {
                o.target = Some(ProductTotalTarget {
                    source: port(source, stream_id)?,
                    value_mol_s,
                })
            }
        }
    }
    if let StoredEquationScaling::ExplicitEquations {
        variables,
        equations,
    } = plan.scaling
    {
        o.scales = Some(ExplicitEquationScales {
            variables: variables
                .into_iter()
                .map(|s| (variable(s.binding), s.scale_si))
                .collect(),
            equations: equations
                .into_iter()
                .map(|s| Ok((equation(s.binding)?, s.scale_si)))
                .collect::<RfResult<_>>()?,
        });
    }
    o.validate_shape().map_err(|e| e.error)?;
    Ok(o)
}
fn stored_port(p: &PortBinding) -> StoredPortBinding {
    StoredPortBinding {
        unit_id: p.unit.clone(),
        port: p.port.name().into(),
    }
}
fn port(p: StoredPortBinding, stream: StreamId) -> RfResult<PortBinding> {
    let port = match p.port.as_str() {
        "inlet" => MaterialPort::Inlet,
        "inlet_a" => MaterialPort::InletA,
        "inlet_b" => MaterialPort::InletB,
        "outlet" => MaterialPort::Outlet,
        "outlet_a" => MaterialPort::OutletA,
        "outlet_b" => MaterialPort::OutletB,
        _ => return Err(RfError::invalid_input("unsupported EO canonical port")),
    };
    Ok(PortBinding {
        unit: p.unit_id,
        port,
        stream,
    })
}
fn stored_variable(b: &VariableBinding) -> StoredEquationVariableBinding {
    match b {
        VariableBinding::PortComponent { port, component } => {
            StoredEquationVariableBinding::PortComponent {
                unit_id: port.unit.clone(),
                port: stored_material_port(port.port),
                stream_id: port.stream.clone(),
                component_id: component.clone(),
            }
        }
        VariableBinding::Parameter(p) => StoredEquationVariableBinding::Parameter {
            unit_id: p.unit.clone(),
            field: StoredEquationParameter::OutletASplitFraction,
        },
    }
}
fn variable(b: StoredEquationVariableBinding) -> VariableBinding {
    match b {
        StoredEquationVariableBinding::PortComponent {
            unit_id,
            port,
            stream_id,
            component_id,
        } => VariableBinding::PortComponent {
            port: PortBinding {
                unit: unit_id,
                port: material_port(port),
                stream: stream_id,
            },
            component: component_id,
        },
        StoredEquationVariableBinding::Parameter { unit_id, .. } => {
            VariableBinding::Parameter(ParameterBinding {
                unit: unit_id,
                field: ParameterField::OutletASplitFraction,
            })
        }
    }
}
fn stored_equation(b: &EquationBinding) -> StoredEquationBinding {
    match b {
        EquationBinding::Unit {
            unit,
            role,
            component,
        } => StoredEquationBinding::Unit {
            unit_id: unit.clone(),
            role: stored_role(*role),
            component_id: component.clone(),
        },
        EquationBinding::Connection {
            source,
            sink,
            component,
        } => StoredEquationBinding::Connection {
            stream_id: source.stream.clone(),
            source: stored_port(source),
            sink: stored_port(sink),
            component_id: component.clone(),
        },
        EquationBinding::ProductTotal { source } => StoredEquationBinding::ProductTotal {
            stream_id: source.stream.clone(),
            source: stored_port(source),
        },
    }
}
fn equation(b: StoredEquationBinding) -> RfResult<EquationBinding> {
    Ok(match b {
        StoredEquationBinding::Unit {
            unit_id,
            role,
            component_id,
        } => EquationBinding::Unit {
            unit: unit_id,
            role: material_role(role),
            component: component_id,
        },
        StoredEquationBinding::Connection {
            stream_id,
            source,
            sink,
            component_id,
        } => EquationBinding::Connection {
            source: port(source, stream_id.clone())?,
            sink: port(sink, stream_id)?,
            component: component_id,
        },
        StoredEquationBinding::ProductTotal { stream_id, source } => {
            EquationBinding::ProductTotal {
                source: port(source, stream_id)?,
            }
        }
    })
}

fn stored_material_port(v: MaterialPort) -> StoredMaterialPort {
    match v {
        MaterialPort::Inlet => StoredMaterialPort::Inlet,
        MaterialPort::InletA => StoredMaterialPort::InletA,
        MaterialPort::InletB => StoredMaterialPort::InletB,
        MaterialPort::Outlet => StoredMaterialPort::Outlet,
        MaterialPort::OutletA => StoredMaterialPort::OutletA,
        MaterialPort::OutletB => StoredMaterialPort::OutletB,
    }
}

fn material_port(v: StoredMaterialPort) -> MaterialPort {
    match v {
        StoredMaterialPort::Inlet => MaterialPort::Inlet,
        StoredMaterialPort::InletA => MaterialPort::InletA,
        StoredMaterialPort::InletB => MaterialPort::InletB,
        StoredMaterialPort::Outlet => MaterialPort::Outlet,
        StoredMaterialPort::OutletA => MaterialPort::OutletA,
        StoredMaterialPort::OutletB => MaterialPort::OutletB,
    }
}

fn stored_role(v: MaterialEquationRole) -> StoredMaterialEquationRole {
    match v {
        MaterialEquationRole::FeedBoundary => StoredMaterialEquationRole::FeedBoundary,
        MaterialEquationRole::MixerBalance => StoredMaterialEquationRole::MixerBalance,
        MaterialEquationRole::SplitA => StoredMaterialEquationRole::SplitA,
        MaterialEquationRole::SplitB => StoredMaterialEquationRole::SplitB,
    }
}

fn material_role(v: StoredMaterialEquationRole) -> MaterialEquationRole {
    match v {
        StoredMaterialEquationRole::FeedBoundary => MaterialEquationRole::FeedBoundary,
        StoredMaterialEquationRole::MixerBalance => MaterialEquationRole::MixerBalance,
        StoredMaterialEquationRole::SplitA => MaterialEquationRole::SplitA,
        StoredMaterialEquationRole::SplitB => MaterialEquationRole::SplitB,
    }
}
