use super::v6::plan as eo;
use super::*;
fn equation(e: &BlockEquation, refs: &Bindings) -> Result<StoredEquationBinding, Failure> {
    Ok(match e {
        BlockEquation::Unit {
            unit,
            role,
            component_id,
        } => StoredEquationBinding::Unit {
            unit_id: refs.resolve(unit, Output::Unit)?.into(),
            role: *role,
            component_id: component_id.clone().into(),
        },
        BlockEquation::Connection {
            stream,
            source,
            sink,
            component_id,
        } => StoredEquationBinding::Connection {
            stream_id: refs.resolve(stream, Output::Stream)?.into(),
            source: eo::stored_port(source, refs)?,
            sink: eo::stored_port(sink, refs)?,
            component_id: component_id.clone().into(),
        },
        BlockEquation::Boundary {
            stream,
            source,
            sink,
            component_id,
        } => StoredEquationBinding::Boundary {
            stream_id: refs.resolve(stream, Output::Stream)?.into(),
            source: eo::stored_port(source, refs)?,
            sink: eo::stored_port(sink, refs)?,
            component_id: component_id.clone().into(),
        },
    })
}
fn wire_equation(e: StoredEquationBinding) -> Result<BlockEquation, Failure> {
    Ok(match e {
        StoredEquationBinding::Unit {
            unit_id,
            role,
            component_id,
        } => BlockEquation::Unit {
            unit: IdentityRef::id(&unit_id),
            role,
            component_id: component_id.to_string(),
        },
        StoredEquationBinding::Connection {
            stream_id,
            source,
            sink,
            component_id,
        } => BlockEquation::Connection {
            stream: IdentityRef::id(&stream_id),
            source: eo::wire_port(&source),
            sink: eo::wire_port(&sink),
            component_id: component_id.to_string(),
        },
        StoredEquationBinding::Boundary {
            stream_id,
            source,
            sink,
            component_id,
        } => BlockEquation::Boundary {
            stream: IdentityRef::id(&stream_id),
            source: eo::wire_port(&source),
            sink: eo::wire_port(&sink),
            component_id: component_id.to_string(),
        },
        StoredEquationBinding::ProductTotal { .. } => {
            return Err(invalid(
                "Mixed blocks do not support inverse specifications",
            ));
        }
    })
}
pub(super) fn configuration(p: &Plan, refs: &Bindings) -> Result<FlowPlanConfiguration, Failure> {
    let Plan::Mixed(MixedPlan::Mixed {
        acceptance_contract_version,
        temperature_k,
        pressure_pa,
        blocks,
    }) = p
    else {
        let Plan::Existing(p) = p else { unreachable!() };
        return <v6::Plan as PlanProtocol>::configuration(p, refs);
    };
    if *acceptance_contract_version != 1 {
        return Err(invalid("unsupported mixed acceptance contract"));
    }
    let blocks = blocks
        .iter()
        .map(|b| {
            Ok(StoredMixedBlock {
                id: b.id.clone(),
                members: b
                    .members
                    .iter()
                    .map(|u| refs.resolve(u, Output::Unit).map(Into::into))
                    .collect::<Result<_, _>>()?,
                method: match &b.method {
                    BlockMethod::Local => StoredMixedBlockMethod::Local,
                    BlockMethod::EquationOriented {
                        equation_contract_version,
                        scaling,
                    } => StoredMixedBlockMethod::EquationOriented {
                        equation_contract_version: *equation_contract_version,
                        scaling: match scaling {
                            Scaling::EvaluatedFreshTotal => {
                                StoredEquationScaling::EvaluatedFreshTotal
                            }
                            Scaling::ExplicitEquations {
                                variables,
                                equations,
                            } => StoredEquationScaling::ExplicitEquations {
                                variables: variables
                                    .iter()
                                    .map(|s| {
                                        Ok(StoredEquationScale {
                                            binding: eo::stored_variable(&s.binding, refs)?,
                                            scale_si: s.scale_si,
                                        })
                                    })
                                    .collect::<Result<_, Failure>>()?,
                                equations: equations
                                    .iter()
                                    .map(|s| {
                                        Ok(StoredEquationScale {
                                            binding: equation(&s.binding, refs)?,
                                            scale_si: s.scale_si,
                                        })
                                    })
                                    .collect::<Result<_, Failure>>()?,
                            },
                        },
                    },
                },
            })
        })
        .collect::<Result<_, Failure>>()?;
    crate::flow_plan_actions::flow_plan_configuration(StoredFlowPlan::Mixed(StoredMixedPlan {
        fixed_boundary: StoredFixedBoundary {
            temperature_k: *temperature_k,
            pressure_pa: *pressure_pa,
        },
        blocks,
    }))
    .map_err(domain)
}
pub(super) fn wire(p: &FlowPlanConfiguration) -> Result<Value, Failure> {
    let StoredFlowPlan::Mixed(p) = crate::flow_plan_actions::stored_flow_plan(p) else {
        return <v6::Plan as PlanProtocol>::wire(p);
    };
    let blocks = p
        .blocks
        .into_iter()
        .map(|b| {
            Ok(Block {
                id: b.id,
                members: b.members.iter().map(IdentityRef::id).collect(),
                method: match b.method {
                    StoredMixedBlockMethod::Local => BlockMethod::Local,
                    StoredMixedBlockMethod::EquationOriented {
                        equation_contract_version,
                        scaling,
                    } => BlockMethod::EquationOriented {
                        equation_contract_version,
                        scaling: match scaling {
                            StoredEquationScaling::EvaluatedFreshTotal => {
                                Scaling::EvaluatedFreshTotal
                            }
                            StoredEquationScaling::ExplicitEquations {
                                variables,
                                equations,
                            } => Scaling::ExplicitEquations {
                                variables: variables
                                    .into_iter()
                                    .map(|s| v6::Scale {
                                        binding: eo::wire_variable(s.binding),
                                        scale_si: s.scale_si,
                                    })
                                    .collect(),
                                equations: equations
                                    .into_iter()
                                    .map(|s| {
                                        Ok(v6::Scale {
                                            binding: wire_equation(s.binding)?,
                                            scale_si: s.scale_si,
                                        })
                                    })
                                    .collect::<Result<_, Failure>>()?,
                            },
                        },
                    },
                },
            })
        })
        .collect::<Result<_, Failure>>()?;
    serde_json::to_value(MixedPlan::Mixed {
        acceptance_contract_version: 1,
        temperature_k: p.fixed_boundary.temperature_k,
        pressure_pa: p.fixed_boundary.pressure_pa,
        blocks,
    })
    .map_err(old::invalid_request)
}
pub(super) fn query(app: &AppState) -> Result<Value, Failure> {
    if let FlowPlanConfiguration::Mixed(o) = app.workspace.flow_plan.current() {
        let plan = rf_solver::mixed::assemble(&app.workspace.document.flowsheet, o);
        let blocks = if let Ok(p) = &plan {
            let mut blocks = Vec::new();
            for b in p.blocks() {
                let layout = if b.equation_options(o.boundary).is_some() {
                    let l = p.equation_layout(&b.id).map_err(domain)?;
                    Some(
                        json!({"variables":l.variables.iter().map(crate::equation_result_projection::variable).collect::<Vec<_>>(),"equations":l.equations.iter().map(crate::equation_result_projection::equation).collect::<Vec<_>>(),"numerical_solvability":"not_proven"}),
                    )
                } else {
                    None
                };
                blocks.push(json!({"id":b.id.as_str(),"members":b.members.iter().map(|u|u.as_str()).collect::<Vec<_>>(),"equation_layout":layout}));
            }
            Some(blocks)
        } else {
            None
        };
        Ok(
            json!({"plan_revision":app.workspace.flow_plan.revision(),"configuration":wire(app.workspace.flow_plan.current())?,"ready":plan.is_ok(),"not_ready":plan.err().map(|e|json!({"code":e.context().diagnostic_code(),"message":e.to_string()})),"blocks":blocks,"simulation_dirty":app.workspace.flow_plan.is_dirty(),"in_memory_required_capabilities":required_capabilities(app),"in_memory_required_project_schema_version":crate::project_save_version(app),"methods":["radishflow.acyclic-sm/1","radishflow.partitioned-sm/1","radishflow.fixed-tp-affine-modular-lu/1","radishflow.fixed-tp-material-eo/1","radishflow.fixed-tp-mixed-material/1"]}),
        )
    } else {
        let mut v = <v6::Plan as PlanProtocol>::query(app)?;
        v["methods"]
            .as_array_mut()
            .expect("methods")
            .push(json!("radishflow.fixed-tp-mixed-material/1"));
        Ok(v)
    }
}
