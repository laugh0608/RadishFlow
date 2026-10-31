use super::*;
use rf_unitops::component_equations::{ComponentEquationCapability, SplitFraction};
use rf_unitops::{BuiltinUnitKind, catalog::builtin_model, validate_unit_node};
use std::collections::{BTreeSet, VecDeque};

pub fn assemble(flow: &Flowsheet, options: &EquationOptions) -> EoResult<EquationProblem> {
    assemble_controlled(flow, options, &mut Uninterrupted)
}
pub(super) fn assemble_controlled(
    flow: &Flowsheet,
    options: &EquationOptions,
    control: &mut impl EquationControl,
) -> EoResult<EquationProblem> {
    checkpoint(control, EquationProgress::at(EquationStage::Assemble))?;
    options.validate_shape()?;
    let invalid = |code, message| error(EquationFailureKind::InvalidBinding, code, message);
    if flow.units.is_empty() || flow.components.is_empty() {
        return Err(invalid(
            "empty",
            "nonempty unit and component registries required",
        ));
    }
    if flow.units.iter().any(|(id, u)| id != &u.id)
        || flow.streams.iter().any(|(id, s)| id != &s.id)
        || flow.components.iter().any(|(id, c)| id != &c.id)
    {
        return Err(invalid(
            "identity",
            "registry keys must equal object identities",
        ));
    }
    let mut capabilities = BTreeMap::new();
    let mut port_count = 0usize;
    for unit in flow.units.values() {
        let mut p = EquationProgress::at(EquationStage::Assemble);
        p.unit = Some(unit.id.clone());
        checkpoint(control, p)?;
        let capability = BuiltinUnitKind::parse(&unit.kind)
            .and_then(|k| builtin_model(k).component_equation_capability())
            .ok_or_else(|| {
                let mut e = error(
                    EquationFailureKind::UnsupportedModel,
                    "unsupported_model",
                    "model has no fixed-T/P material equation capability",
                );
                e.error = e.error.with_related_unit_id(unit.id.clone());
                e
            })?;
        validate_unit_node(unit).map_err(|e| mapped(EquationFailureKind::InvalidBinding, e))?;
        validate_parameters(unit, capability, options)?;
        port_count += capability.ports().len();
        capabilities.insert(unit.id.clone(), capability);
    }
    let variable_count = port_count
        .checked_mul(flow.components.len())
        .and_then(|n| n.checked_add(usize::from(options.released.is_some())));
    if variable_count.is_none_or(|n| n > MAX_DIMENSION) {
        return Err(error(
            EquationFailureKind::Capacity,
            "capacity",
            "EO problem exceeds 128 variables",
        ));
    }
    let dependencies = rf_flowsheet::dependencies::material_dependencies(flow)
        .map_err(|e| mapped(EquationFailureKind::InvalidBinding, e))?;
    let connections = dependencies.connections;
    let components: Vec<_> = flow.components.keys().cloned().collect();
    let mut feeds = BTreeMap::new();
    let mut feed_flows = BTreeMap::new();
    let mut fresh_total = 0.0;
    for unit in flow
        .units
        .values()
        .filter(|u| capabilities[&u.id] == ComponentEquationCapability::Feed)
    {
        let feed = rf_unitops::catalog::effective_feed_boundary(unit, &flow.streams)
            .map_err(|e| mapped(EquationFailureKind::InvalidSpecification, e))?;
        let values =
            crate::fixed_tp_material::component_flows(&components, &feed, options.boundary)
                .map_err(|e| mapped(EquationFailureKind::InvalidSpecification, e))?;
        fresh_total += feed.total_molar_flow_mol_s;
        feed_flows.insert(unit.id.clone(), values);
        feeds.insert(unit.id.clone(), feed);
    }
    if !fresh_total.is_finite() || fresh_total <= 0.0 {
        return Err(error(
            EquationFailureKind::InvalidSpecification,
            "fresh_total",
            "at least one positive Feed and finite positive total required",
        ));
    }
    let mut reached: BTreeSet<_> = feeds.keys().cloned().collect();
    let mut queue: VecDeque<_> = reached.iter().cloned().collect();
    while let Some(u) = queue.pop_front() {
        for v in &dependencies.successors[&u] {
            if reached.insert(v.clone()) {
                queue.push_back(v.clone());
            }
        }
    }
    if reached.len() != flow.units.len() {
        let mut e = invalid(
            "unanchored",
            "all units must be reachable from a Feed boundary",
        );
        e.error = e.error.with_related_unit_ids(
            flow.units
                .keys()
                .filter(|id| !reached.contains(*id))
                .cloned()
                .collect(),
        );
        return Err(e);
    }
    if let Some(r) = &options.released
        && capabilities.get(&r.binding.unit) != Some(&ComponentEquationCapability::Splitter)
    {
        return Err(invalid(
            "release_binding",
            "released parameter must bind an existing Splitter",
        ));
    }
    if let Some(t) = &options.target
        && !connections.iter().any(|c| {
            c.sink.is_none()
                && c.stream_id == t.source.stream
                && c.source.unit_id == t.source.unit
                && c.source.port_name == t.source.port.name()
        })
    {
        return Err(invalid(
            "target_binding",
            "target must bind an exact terminal product source",
        ));
    }
    let mut variables = Vec::with_capacity(variable_count.expect("bounded variable count"));
    for (id, capability) in &capabilities {
        for &port in capability.ports() {
            let binding = port_binding(flow, id, port);
            for component in &components {
                variables.push(VariableBinding::PortComponent {
                    port: binding.clone(),
                    component: component.clone(),
                });
            }
        }
    }
    if let Some(r) = &options.released {
        variables.push(VariableBinding::Parameter(r.binding.clone()));
    }
    variables.sort();
    let indices: BTreeMap<_, _> = variables
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, b)| (b, i))
        .collect();
    let fraction = options
        .released
        .as_ref()
        .map(|r| indices[&VariableBinding::Parameter(r.binding.clone())]);
    let mut assembled = BTreeMap::new();
    for (id, capability) in &capabilities {
        let unit = &flow.units[id];
        let released = options
            .released
            .as_ref()
            .is_some_and(|r| r.binding.unit == *id);
        for component in &components {
            let ports: BTreeMap<_, _> = capability
                .ports()
                .iter()
                .map(|&p| {
                    (
                        p,
                        indices[&VariableBinding::PortComponent {
                            port: port_binding(flow, id, p),
                            component: component.clone(),
                        }],
                    )
                })
                .collect();
            for &role in capability.rows() {
                let model = ComponentEquation {
                    role,
                    feed_mol_s: feed_flows.get(id).map(|v| v[component]),
                    split: if *capability == ComponentEquationCapability::Splitter {
                        Some(if released {
                            SplitFraction::Released
                        } else {
                            SplitFraction::Fixed(
                                unit.parameters
                                    .outlet_a_split_fraction
                                    .expect("validated fixed fraction"),
                            )
                        })
                    } else {
                        None
                    },
                };
                model
                    .validate()
                    .map_err(|e| mapped(EquationFailureKind::InvalidSpecification, e))?;
                assembled.insert(
                    EquationBinding::Unit {
                        unit: id.clone(),
                        role,
                        component: component.clone(),
                    },
                    AssembledEquation::Unit {
                        model,
                        ports: ports.clone(),
                        fraction: if released { fraction } else { None },
                    },
                );
            }
        }
    }
    for connection in &connections {
        if let Some(sink) = &connection.sink {
            let source = find_port(
                flow,
                &capabilities,
                &connection.source.unit_id,
                &connection.source.port_name,
            );
            let sink = find_port(flow, &capabilities, &sink.unit_id, &sink.port_name);
            for component in &components {
                let a = indices[&VariableBinding::PortComponent {
                    port: source.clone(),
                    component: component.clone(),
                }];
                let b = indices[&VariableBinding::PortComponent {
                    port: sink.clone(),
                    component: component.clone(),
                }];
                assembled.insert(
                    EquationBinding::Connection {
                        source: source.clone(),
                        sink: sink.clone(),
                        component: component.clone(),
                    },
                    AssembledEquation::Connection { source: a, sink: b },
                );
            }
        }
    }
    if let Some(t) = &options.target {
        let columns = components
            .iter()
            .map(|c| {
                indices[&VariableBinding::PortComponent {
                    port: t.source.clone(),
                    component: c.clone(),
                }]
            })
            .collect();
        assembled.insert(
            EquationBinding::ProductTotal {
                source: t.source.clone(),
            },
            AssembledEquation::ProductTotal {
                components: columns,
                target: t.value_mol_s,
            },
        );
    }
    if assembled.len() > MAX_DIMENSION {
        return Err(error(
            EquationFailureKind::Capacity,
            "capacity",
            "EO problem exceeds 128 equations",
        ));
    }
    let (equations, rows): (Vec<_>, Vec<_>) = assembled.into_iter().unzip();
    let (variable_scales, residual_scales) = if let Some(s) = &options.scales {
        if s.variables.keys().ne(variables.iter()) || s.equations.keys().ne(equations.iter()) {
            return Err(invalid(
                "scale_binding",
                "explicit scales must exactly cover all variable and equation identities",
            ));
        }
        (
            s.variables.values().copied().collect(),
            s.equations.values().copied().collect(),
        )
    } else {
        (
            variables
                .iter()
                .map(|v| {
                    if matches!(v, VariableBinding::Parameter(_)) {
                        1.0
                    } else {
                        fresh_total
                    }
                })
                .collect(),
            vec![fresh_total; equations.len()],
        )
    };
    let mut problem = EquationProblem {
        flowsheet: flow.clone(),
        connections,
        components,
        options: options.clone(),
        variables,
        equations,
        rows,
        feeds,
        variable_scales,
        residual_scales,
        fresh_total_mol_s: fresh_total,
        structure: StructuralReport {
            variable_count: 0,
            equation_count: 0,
            matching: vec![],
            unmatched_variables: vec![],
            unmatched_equations: vec![],
            underdetermined: StructuralRegion {
                variables: vec![],
                equations: vec![],
            },
            overdetermined: StructuralRegion {
                variables: vec![],
                equations: vec![],
            },
        },
    };
    problem.structure = structure::analyze(&problem, control)?;
    Ok(problem)
}
fn port_binding(flow: &Flowsheet, id: &UnitId, port: MaterialPort) -> PortBinding {
    PortBinding {
        unit: id.clone(),
        port,
        stream: flow.units[id]
            .ports
            .iter()
            .find(|p| p.name == port.name())
            .expect("canonical port")
            .stream_id
            .clone()
            .expect("validated bound port"),
    }
}
fn find_port(
    flow: &Flowsheet,
    capabilities: &BTreeMap<UnitId, ComponentEquationCapability>,
    id: &UnitId,
    name: &str,
) -> PortBinding {
    port_binding(
        flow,
        id,
        *capabilities[id]
            .ports()
            .iter()
            .find(|p| p.name() == name)
            .expect("canonical connection"),
    )
}
fn validate_parameters(
    unit: &rf_model::UnitNode,
    capability: ComponentEquationCapability,
    options: &EquationOptions,
) -> EoResult<()> {
    use ComponentEquationCapability::*;
    let b = options.boundary;
    let p = &unit.parameters;
    let valid = match capability {
        Feed => p.outlet_a_split_fraction.is_none(),
        Mixer => {
            p.outlet_temperature_k.is_none()
                && p.outlet_a_split_fraction.is_none()
                && p.outlet_pressure_pa.is_none_or(|v| v == b.pressure_pa)
        }
        Splitter => {
            p.outlet_temperature_k.is_none()
                && p.outlet_pressure_pa.is_none()
                && (options
                    .released
                    .as_ref()
                    .is_some_and(|r| r.binding.unit == unit.id)
                    || p.outlet_a_split_fraction
                        .is_some_and(|f| f.is_finite() && (0.0..=1.0).contains(&f)))
        }
    };
    if !valid {
        let mut e = error(
            EquationFailureKind::InvalidSpecification,
            "unit_parameters",
            "unit parameters are outside the fixed-T/P equation capability",
        );
        e.error = e.error.with_related_unit_id(unit.id.clone());
        return Err(e);
    }
    Ok(())
}
