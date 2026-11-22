use super::*;
pub(in crate::headless) fn stored_port(
    p: &old::actions::Port,
    refs: &Bindings,
) -> Result<StoredPortBinding, Failure> {
    Ok(StoredPortBinding {
        unit_id: refs.resolve(&p.unit, Output::Unit)?.into(),
        port: p.port.clone(),
    })
}
pub(in crate::headless) fn wire_port(p: &StoredPortBinding) -> old::actions::Port {
    old::actions::Port {
        unit: IdentityRef::id(&p.unit_id),
        port: p.port.clone(),
    }
}
pub(in crate::headless) fn stored_variable(
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
pub(in crate::headless) fn stored_equation(
    b: &Equation,
    refs: &Bindings,
) -> Result<StoredEquationBinding, Failure> {
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
pub(in crate::headless) fn wire_variable(b: StoredEquationVariableBinding) -> Variable {
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
pub(in crate::headless) fn wire_equation(b: StoredEquationBinding) -> Result<Equation, Failure> {
    Ok(match b {
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
        StoredEquationBinding::ProductMoleFraction { .. } => {
            return Err(invalid("v6 does not support product composition"));
        }
        StoredEquationBinding::Boundary { .. } => {
            return Err(invalid(
                "external boundary fields require CLI v7 Mixed configuration",
            ));
        }
        StoredEquationBinding::ProductTotal { stream_id, source } => Equation::ProductTotal {
            stream: IdentityRef::id(&stream_id),
            source: wire_port(&source),
        },
    })
}
pub(super) fn configuration(
    plan: &Plan,
    refs: &Bindings,
) -> Result<FlowPlanConfiguration, Failure> {
    match plan {
        Plan::Existing(plan) => <v5::Plan as PlanProtocol>::configuration(plan, refs),
        Plan::EquationOriented(plan) => equation_configuration(
            plan,
            refs,
            FlowPlanSupport::EquationOriented,
            stored_specification,
            stored_equation,
        ),
    }
}

pub(in crate::headless) fn stored_specification(
    s: &Specification,
    refs: &Bindings,
) -> Result<StoredEquationSpecification, Failure> {
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
}

pub(in crate::headless) fn equation_configuration<S, E>(
    plan: &EquationPlan<S, E>,
    refs: &Bindings,
    support: FlowPlanSupport,
    stored_spec: impl Fn(&S, &Bindings) -> Result<StoredEquationSpecification, Failure>,
    stored_row: impl Fn(&E, &Bindings) -> Result<StoredEquationBinding, Failure>,
) -> Result<FlowPlanConfiguration, Failure> {
    let EquationPlan::EquationOriented {
        equation_contract_version,
        acceptance_contract_version,
        temperature_k,
        pressure_pa,
        specifications,
        scaling,
    } = plan;
    if equation_contract_version != acceptance_contract_version
        || !support.supports_equation_version(*equation_contract_version)
    {
        return Err(Failure::new(
            old::protocol::Stage::Step,
            old::protocol::Code::UnsupportedCapability,
            "unsupported EO contract version",
        ));
    }
    let specifications = specifications
        .iter()
        .map(|s| stored_spec(s, refs))
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
                        binding: stored_row(&s.binding, refs)?,
                        scale_si: s.scale_si,
                    })
                })
                .collect::<Result<_, Failure>>()?,
        },
    };
    let stored = StoredEquationPlan {
        fixed_boundary: StoredFixedBoundary {
            temperature_k: *temperature_k,
            pressure_pa: *pressure_pa,
        },
        specifications,
        scaling,
    };
    if stored.contract_version() != *equation_contract_version {
        return Err(invalid("EO specification and contract version differ"));
    }
    crate::flow_plan_actions::flow_plan_configuration(StoredFlowPlan::EquationOriented(stored))
        .map_err(domain)
}

pub(in crate::headless) fn wire_specification(
    s: StoredEquationSpecification,
) -> Result<Specification, Failure> {
    Ok(match s {
        StoredEquationSpecification::ProductMoleFraction { .. } => {
            return Err(invalid("v6 does not support product composition"));
        }
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
}

pub(super) fn wire(plan: &FlowPlanConfiguration) -> Result<Value, Failure> {
    let StoredFlowPlan::EquationOriented(p) =
        crate::flow_plan_actions::stored_flow_plan(plan).map_err(domain)?
    else {
        return <v5::Plan as PlanProtocol>::wire(plan);
    };
    wire_equation_plan(p, wire_specification, wire_equation)
}

pub(in crate::headless) fn wire_equation_plan<S, E>(
    p: StoredEquationPlan,
    wire_spec: impl Fn(StoredEquationSpecification) -> Result<S, Failure>,
    wire_row: impl Fn(StoredEquationBinding) -> Result<E, Failure>,
) -> Result<Value, Failure>
where
    S: Serialize,
    E: Serialize,
{
    let contract_version = p.contract_version();
    let specifications = p
        .specifications
        .into_iter()
        .map(wire_spec)
        .collect::<Result<_, _>>()?;
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
                .map(|s| {
                    Ok(Scale {
                        binding: wire_row(s.binding)?,
                        scale_si: s.scale_si,
                    })
                })
                .collect::<Result<_, Failure>>()?,
        },
    };
    serde_json::to_value(EquationPlan::EquationOriented {
        equation_contract_version: contract_version,
        acceptance_contract_version: contract_version,
        temperature_k: p.fixed_boundary.temperature_k,
        pressure_pa: p.fixed_boundary.pressure_pa,
        specifications,
        scaling,
    })
    .map_err(old::invalid_request)
}

pub(in crate::headless) fn equation_query(
    app: &AppState,
    wire: impl Fn(&FlowPlanConfiguration) -> Result<Value, Failure>,
) -> Result<Value, Failure> {
    let w = &app.workspace;
    let ready = w.flow_plan.current().validate_for(&w.document.flowsheet);
    let structure = crate::equation_actions::inspect_equation_structure(w);
    Ok(
        json!({"plan_revision":w.flow_plan.revision(),"configuration":wire(w.flow_plan.current())?,"ready":ready.is_ok(),"not_ready":ready.err().map(|e|json!({"code":e.context().diagnostic_code(),"message":e.to_string()})),"simulation_dirty":w.flow_plan.is_dirty(),"in_memory_required_project_schema_version":crate::project_save_version(app),"in_memory_required_capabilities":required_capabilities(app)?,"equation_preview":structure.as_ref().ok().map(|s|crate::equation_result_projection::definition(&s.definition)).transpose().map_err(domain)?,"preview_error":structure.err().map(|e|json!({"code":e.context().diagnostic_code(),"message":e.to_string()}))}),
    )
}

pub(super) fn query(app: &AppState) -> Result<Value, Failure> {
    let mut value = if matches!(
        app.workspace.flow_plan.current(),
        FlowPlanConfiguration::EquationOriented(_)
    ) {
        equation_query(app, wire)?
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
