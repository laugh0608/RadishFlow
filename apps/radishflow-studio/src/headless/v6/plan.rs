use super::*;
fn stored_port(p: &old::actions::Port, refs: &Bindings) -> Result<StoredPortBinding, Failure> {
    Ok(StoredPortBinding {
        unit_id: refs.resolve(&p.unit, Output::Unit)?.into(),
        port: p.port.clone(),
    })
}
fn wire_port(p: &StoredPortBinding) -> old::actions::Port {
    old::actions::Port {
        unit: IdentityRef::id(&p.unit_id),
        port: p.port.clone(),
    }
}
fn stored_variable(
    b: &Variable,
    refs: &Bindings,
) -> Result<StoredEquationVariableBinding, Failure> {
    Ok(match b {
        Variable::PortComponent {
            unit,
            port,
            stream,
            component_id,
        } => StoredEquationVariableBinding::PortComponent {
            unit_id: refs.resolve(unit, Output::Unit)?.into(),
            port: *port,
            stream_id: refs.resolve(stream, Output::Stream)?.into(),
            component_id: component_id.clone().into(),
        },
        Variable::Parameter { unit, field } => StoredEquationVariableBinding::Parameter {
            unit_id: refs.resolve(unit, Output::Unit)?.into(),
            field: *field,
        },
    })
}
fn stored_equation(b: &Equation, refs: &Bindings) -> Result<StoredEquationBinding, Failure> {
    Ok(match b {
        Equation::Unit {
            unit,
            role,
            component_id,
        } => StoredEquationBinding::Unit {
            unit_id: refs.resolve(unit, Output::Unit)?.into(),
            role: *role,
            component_id: component_id.clone().into(),
        },
        Equation::Connection {
            stream,
            source,
            sink,
            component_id,
        } => StoredEquationBinding::Connection {
            stream_id: refs.resolve(stream, Output::Stream)?.into(),
            source: stored_port(source, refs)?,
            sink: stored_port(sink, refs)?,
            component_id: component_id.clone().into(),
        },
        Equation::ProductTotal { stream, source } => StoredEquationBinding::ProductTotal {
            stream_id: refs.resolve(stream, Output::Stream)?.into(),
            source: stored_port(source, refs)?,
        },
    })
}
fn wire_variable(b: StoredEquationVariableBinding) -> Variable {
    match b {
        StoredEquationVariableBinding::PortComponent {
            unit_id,
            port,
            stream_id,
            component_id,
        } => Variable::PortComponent {
            unit: IdentityRef::id(&unit_id),
            port,
            stream: IdentityRef::id(&stream_id),
            component_id: component_id.to_string(),
        },
        StoredEquationVariableBinding::Parameter { unit_id, field } => Variable::Parameter {
            unit: IdentityRef::id(&unit_id),
            field,
        },
    }
}
fn wire_equation(b: StoredEquationBinding) -> Equation {
    match b {
        StoredEquationBinding::Unit {
            unit_id,
            role,
            component_id,
        } => Equation::Unit {
            unit: IdentityRef::id(&unit_id),
            role,
            component_id: component_id.to_string(),
        },
        StoredEquationBinding::Connection {
            stream_id,
            source,
            sink,
            component_id,
        } => Equation::Connection {
            stream: IdentityRef::id(&stream_id),
            source: wire_port(&source),
            sink: wire_port(&sink),
            component_id: component_id.to_string(),
        },
        StoredEquationBinding::ProductTotal { stream_id, source } => Equation::ProductTotal {
            stream: IdentityRef::id(&stream_id),
            source: wire_port(&source),
        },
    }
}
pub(super) fn configuration(
    plan: &Plan,
    refs: &Bindings,
) -> Result<FlowPlanConfiguration, Failure> {
    let Plan::EquationOriented(EquationPlan::EquationOriented {
        equation_contract_version,
        acceptance_contract_version,
        temperature_k,
        pressure_pa,
        specifications,
        scaling,
    }) = plan
    else {
        let Plan::Existing(p) = plan else {
            unreachable!()
        };
        return <v5::Plan as PlanProtocol>::configuration(p, refs);
    };
    if *equation_contract_version != 1 || *acceptance_contract_version != 1 {
        return Err(Failure::new(
            old::protocol::Stage::Step,
            old::protocol::Code::UnsupportedCapability,
            "unsupported EO contract version",
        ));
    }
    let specifications = specifications
        .iter()
        .map(|s| {
            Ok(match s {
                Specification::ReleaseSplitFraction {
                    unit,
                    field,
                    interval,
                } => StoredEquationSpecification::ReleaseSplitFraction {
                    unit_id: refs.resolve(unit, Output::Unit)?.into(),
                    field: *field,
                    interval: StoredFractionInterval {
                        lower: interval.lower,
                        upper: interval.upper,
                        max_iterations: interval.max_iterations,
                    },
                },
                Specification::ProductTotalMolarFlow {
                    stream,
                    source,
                    value_mol_s,
                } => StoredEquationSpecification::ProductTotalMolarFlow {
                    stream_id: refs.resolve(stream, Output::Stream)?.into(),
                    source: stored_port(source, refs)?,
                    value_mol_s: *value_mol_s,
                },
            })
        })
        .collect::<Result<Vec<_>, Failure>>()?;
    let scaling = match scaling {
        Scaling::EvaluatedFreshTotal => StoredEquationScaling::EvaluatedFreshTotal,
        Scaling::ExplicitEquations {
            variables,
            equations,
        } => StoredEquationScaling::ExplicitEquations {
            variables: variables
                .iter()
                .map(|s| {
                    Ok(StoredEquationScale {
                        binding: stored_variable(&s.binding, refs)?,
                        scale_si: s.scale_si,
                    })
                })
                .collect::<Result<_, Failure>>()?,
            equations: equations
                .iter()
                .map(|s| {
                    Ok(StoredEquationScale {
                        binding: stored_equation(&s.binding, refs)?,
                        scale_si: s.scale_si,
                    })
                })
                .collect::<Result<_, Failure>>()?,
        },
    };
    crate::flow_plan_actions::flow_plan_configuration(StoredFlowPlan::EquationOriented(
        StoredEquationPlan {
            fixed_boundary: StoredFixedBoundary {
                temperature_k: *temperature_k,
                pressure_pa: *pressure_pa,
            },
            specifications,
            scaling,
        },
    ))
    .map_err(domain)
}
pub(super) fn wire(plan: &FlowPlanConfiguration) -> Result<Value, Failure> {
    let StoredFlowPlan::EquationOriented(p) = crate::flow_plan_actions::stored_flow_plan(plan)
    else {
        return <v5::Plan as PlanProtocol>::wire(plan);
    };
    let specifications = p
        .specifications
        .into_iter()
        .map(|s| match s {
            StoredEquationSpecification::ReleaseSplitFraction {
                unit_id,
                field,
                interval,
            } => Specification::ReleaseSplitFraction {
                unit: IdentityRef::id(&unit_id),
                field,
                interval: Interval {
                    lower: interval.lower,
                    upper: interval.upper,
                    max_iterations: interval.max_iterations,
                },
            },
            StoredEquationSpecification::ProductTotalMolarFlow {
                stream_id,
                source,
                value_mol_s,
            } => Specification::ProductTotalMolarFlow {
                stream: IdentityRef::id(&stream_id),
                source: wire_port(&source),
                value_mol_s,
            },
        })
        .collect();
    let scaling = match p.scaling {
        StoredEquationScaling::EvaluatedFreshTotal => Scaling::EvaluatedFreshTotal,
        StoredEquationScaling::ExplicitEquations {
            variables,
            equations,
        } => Scaling::ExplicitEquations {
            variables: variables
                .into_iter()
                .map(|s| Scale {
                    binding: wire_variable(s.binding),
                    scale_si: s.scale_si,
                })
                .collect(),
            equations: equations
                .into_iter()
                .map(|s| Scale {
                    binding: wire_equation(s.binding),
                    scale_si: s.scale_si,
                })
                .collect(),
        },
    };
    serde_json::to_value(EquationPlan::EquationOriented {
        equation_contract_version: 1,
        acceptance_contract_version: 1,
        temperature_k: p.fixed_boundary.temperature_k,
        pressure_pa: p.fixed_boundary.pressure_pa,
        specifications,
        scaling,
    })
    .map_err(old::invalid_request)
}
pub(super) fn query(app: &AppState) -> Result<Value, Failure> {
    let w = &app.workspace;
    let mut value = if matches!(
        w.flow_plan.current(),
        FlowPlanConfiguration::EquationOriented(_)
    ) {
        let ready = w.flow_plan.current().validate_for(w);
        let structure = crate::equation_actions::inspect_equation_structure(w);
        json!({"plan_revision":w.flow_plan.revision(),"configuration":wire(w.flow_plan.current())?,"ready":ready.is_ok(),"not_ready":ready.err().map(|e|json!({"code":e.context().diagnostic_code(),"message":e.to_string()})),"simulation_dirty":w.flow_plan.is_dirty(),"in_memory_required_project_schema_version":crate::project_save_version(app),"in_memory_required_capabilities":required_capabilities(app),"equation_preview":structure.as_ref().ok().map(|s|crate::equation_result_projection::definition(&s.definition)).transpose().map_err(domain)?,"preview_error":structure.err().map(|e|json!({"code":e.context().diagnostic_code(),"message":e.to_string()}))})
    } else {
        <v5::Plan as PlanProtocol>::query(app)?
    };
    value["methods"] = json!([
        "radishflow.acyclic-sm/1",
        "radishflow.partitioned-sm/1",
        "radishflow.fixed-tp-affine-modular-lu/1",
        "radishflow.fixed-tp-material-eo/1"
    ]);
    value["equation_capability"] = json!({"models":["feed","mixer","splitter"],"fixed_temperature_pressure":true,"max_variables":128,"max_equations":128,"max_released_split_fractions":1,"target":"terminal_product_total_molar_flow","forward_backend":"direct_lu","inverse_backend":"conditional_lu_bisection","energy_or_phase_equilibrium":false});
    Ok(value)
}
