use super::execution::{Execution, require_residual, verify_residuals};
use super::*;
use crate::fixed_tp_material as material;
use rf_model::Composition;
use rf_types::PortDirection;
use rf_unitops::{UnitOperation, UnitOperationInputs, UnitOperationServices};
use std::collections::BTreeSet;

impl Execution<'_> {
    pub(super) fn verify(
        &mut self,
        p: &EquationProblem,
        values: &[f64],
        services: &UnitOperationServices<'_>,
        factory: &mut impl FnMut(
            &rf_model::UnitNode,
            &BTreeMap<StreamId, MaterialStreamState>,
        ) -> rf_types::RfResult<Box<dyn UnitOperation>>,
    ) -> EoResult<BTreeMap<StreamId, MaterialStreamState>> {
        self.at(EquationStage::FinalUnitVerification, None)?;
        let ports = port_flows(p, values);
        let mut assumptions = BTreeMap::new();
        for (port, components) in &ports {
            let (composition, anchor) = composition_anchor(p, &ports, port, &mut BTreeSet::new())?;
            let total = components.values().sum::<f64>();
            if total == 0.0 {
                self.evidence.zero_flow_sources.insert(port.clone(), anchor);
            }
            assumptions.insert(
                port.clone(),
                MaterialStreamState::from_tpzf(
                    port.stream.clone(),
                    p.flowsheet.streams[&port.stream].name.clone(),
                    p.options.boundary.temperature_k,
                    p.options.boundary.pressure_pa,
                    total,
                    composition,
                ),
            );
        }
        let scales: BTreeMap<_, _> = p
            .components
            .iter()
            .cloned()
            .map(|c| (c, p.fresh_total_mol_s))
            .collect();
        let mut actual = values.to_vec();
        let mut streams = BTreeMap::new();
        // All consumer inputs are the solved port assumptions, so ordering does
        // not replace or hide a connection residual with an upstream overwrite.
        for frozen in p.flowsheet.units.values() {
            let mut progress = EquationProgress::at(EquationStage::FinalUnitVerification);
            progress.unit = Some(frozen.id.clone());
            checkpoint(self, progress.clone())?;
            let mut unit = frozen.clone();
            if let Some(candidate) = &self.evidence.solved_parameter
                && candidate.binding.unit == unit.id
            {
                unit.parameters.outlet_a_split_fraction = Some(candidate.value);
            }
            let mut inputs = UnitOperationInputs::new();
            let mut consumed = Vec::new();
            for port in unit
                .ports
                .iter()
                .filter(|port| port.direction == PortDirection::Inlet)
            {
                let binding = binding_for(p, &unit.id, &port.name);
                let input = assumptions[&binding].clone();
                material::component_flows(&p.components, &input, p.options.boundary)
                    .map_err(|e| mapped(EquationFailureKind::LocalVerification, e))?;
                inputs.insert_material_stream(port.name.clone(), input.clone());
                consumed.push(input);
            }
            let ids = consumed.iter().map(|s| s.id.clone()).collect::<Vec<_>>();
            crate::parameter_checks::validate_step_parameters(1, &unit, &inputs, &ids)
                .map_err(|e| mapped(EquationFailureKind::LocalVerification, e))?;
            self.costs.operation_constructions += 1;
            let operation = factory(&unit, &p.flowsheet.streams).map_err(|e| {
                mapped(
                    EquationFailureKind::LocalVerification,
                    e.with_related_unit_id(unit.id.clone()),
                )
            })?;
            self.costs.local_unit_calls += 1;
            *self.costs.calls_by_unit.entry(unit.id.clone()).or_default() += 1;
            let outputs = operation.run(services, &inputs).map_err(|e| {
                mapped(
                    EquationFailureKind::LocalVerification,
                    e.with_related_unit_id(unit.id.clone()),
                )
            })?;
            let produced: Vec<_> = unit
                .ports
                .iter()
                .filter(|port| port.direction == PortDirection::Outlet)
                .filter_map(|port| outputs.stream(&port.name).cloned())
                .collect();
            let local = LocalUnitEvaluation {
                unit_id: unit.id.clone(),
                consumed_streams: consumed,
                produced_streams: produced,
            };
            self.evidence.final_units.push(local.clone());
            for port in &unit.ports {
                let binding = binding_for(p, &unit.id, &port.name);
                let stream = if port.direction == PortDirection::Inlet {
                    inputs.stream(&port.name)
                } else {
                    outputs.stream(&port.name)
                }
                .ok_or_else(|| {
                    let mut e = error(
                        EquationFailureKind::LocalVerification,
                        "missing_output",
                        "local unit omitted a canonical port",
                    );
                    e.error = e.error.with_related_unit_id(unit.id.clone());
                    e
                })?;
                if stream.id != binding.stream {
                    return Err(error(
                        EquationFailureKind::LocalVerification,
                        "output_identity",
                        "actual local output differs from the declared stream identity",
                    ));
                }
                let flows = material::component_flows(&p.components, stream, p.options.boundary)
                    .map_err(|e| {
                        mapped(
                            EquationFailureKind::LocalVerification,
                            e.with_related_unit_id(unit.id.clone()),
                        )
                    })?;
                for (component, value) in flows {
                    let variable = VariableBinding::PortComponent {
                        port: binding.clone(),
                        component,
                    };
                    let i = p
                        .variables
                        .binary_search(&variable)
                        .expect("declared variable");
                    actual[i] = value;
                    require_residual(value - values[i], p.variable_scales[i]).map_err(
                        |mut e| {
                            e.error = e
                                .error
                                .with_related_unit_id(unit.id.clone())
                                .with_related_stream_id(stream.id.clone());
                            e
                        },
                    )?;
                }
                if port.direction == PortDirection::Outlet {
                    streams.insert(stream.id.clone(), stream.clone());
                }
            }
            if unit.kind != "feed" {
                material::balance(
                    &p.components,
                    p.options.boundary,
                    &local.consumed_streams,
                    &local.produced_streams,
                    &scales,
                )
                .map_err(|e| {
                    mapped(
                        EquationFailureKind::LocalVerification,
                        e.with_related_unit_id(unit.id.clone()),
                    )
                })?;
                let total = local
                    .produced_streams
                    .iter()
                    .map(|s| s.total_molar_flow_mol_s)
                    .sum::<f64>()
                    - local
                        .consumed_streams
                        .iter()
                        .map(|s| s.total_molar_flow_mol_s)
                        .sum::<f64>();
                if !total.is_finite() || (total / p.fresh_total_mol_s).abs() > SCALED_TOLERANCE {
                    return Err(error(
                        EquationFailureKind::LocalVerification,
                        "local_total",
                        "local scaled total balance failed",
                    ));
                }
            }
            checkpoint(self, progress)?;
        }
        self.evidence.verified_values_si = Some(actual.clone());
        let evaluated = p.evaluate(&actual)?;
        self.evidence.verified_residuals_mol_s = Some(evaluated.residuals_mol_s.clone());
        verify_residuals(p, &evaluated.residuals_mol_s)?;
        if let Some(target) = &p.options.target {
            let total = streams[&target.source.stream].total_molar_flow_mol_s;
            let residual = total - target.value_mol_s;
            let i = p
                .equations
                .iter()
                .position(|e| matches!(e, EquationBinding::ProductTotal { .. }))
                .expect("target row");
            self.evidence.verified_target = Some(VerifiedTarget {
                target: target.clone(),
                actual_mol_s: total,
                residual_mol_s: residual,
                scaled_residual: residual / p.residual_scales[i],
            });
            require_residual(residual, p.residual_scales[i])?;
        }
        self.at(EquationStage::GlobalVerification, None)?;
        let fresh: Vec<_> = p.feeds.values().map(|f| streams[&f.id].clone()).collect();
        let products: Vec<_> = p
            .connections
            .iter()
            .filter(|c| c.sink.is_none())
            .map(|c| streams[&c.stream_id].clone())
            .collect();
        let fresh_components = totals(p, &fresh)?;
        let product_components = totals(p, &products)?;
        let total_residual = products
            .iter()
            .map(|s| s.total_molar_flow_mol_s)
            .sum::<f64>()
            - fresh.iter().map(|s| s.total_molar_flow_mol_s).sum::<f64>();
        self.evidence.global_material = Some(GlobalMaterialEvidence {
            component_residuals_mol_s: p
                .components
                .iter()
                .map(|c| (c.clone(), product_components[c] - fresh_components[c]))
                .collect(),
            fresh_components_mol_s: fresh_components,
            product_components_mol_s: product_components,
            total_residual_mol_s: total_residual,
        });
        material::balance(
            &p.components,
            p.options.boundary,
            &fresh,
            &products,
            &scales,
        )
        .map_err(|e| mapped(EquationFailureKind::GlobalVerification, e))?;
        if !total_residual.is_finite()
            || (total_residual / p.fresh_total_mol_s).abs() > SCALED_TOLERANCE
        {
            return Err(error(
                EquationFailureKind::GlobalVerification,
                "global_total",
                "global scaled total balance failed",
            ));
        }
        self.at(EquationStage::GlobalVerification, None)?;
        Ok(streams)
    }
}
fn totals(
    p: &EquationProblem,
    streams: &[MaterialStreamState],
) -> EoResult<BTreeMap<ComponentId, f64>> {
    let mut totals: BTreeMap<_, _> = p.components.iter().cloned().map(|c| (c, 0.0)).collect();
    for stream in streams {
        for (c, v) in material::component_flows(&p.components, stream, p.options.boundary)
            .map_err(|e| mapped(EquationFailureKind::GlobalVerification, e))?
        {
            *totals.get_mut(&c).expect("registered component") += v;
        }
    }
    Ok(totals)
}
fn port_flows(p: &EquationProblem, values: &[f64]) -> BTreeMap<PortBinding, Composition> {
    let mut ports: BTreeMap<PortBinding, Composition> = BTreeMap::new();
    for (binding, value) in p.variables.iter().zip(values) {
        if let VariableBinding::PortComponent { port, component } = binding {
            ports
                .entry(port.clone())
                .or_default()
                .insert(component.clone(), *value);
        }
    }
    ports
}
fn binding_for(p: &EquationProblem, unit: &UnitId, name: &str) -> PortBinding {
    p.variables
        .iter()
        .find_map(|v| match v {
            VariableBinding::PortComponent { port, .. }
                if &port.unit == unit && port.port.name() == name =>
            {
                Some(port.clone())
            }
            _ => None,
        })
        .expect("assembled port")
}
fn composition_anchor(
    p: &EquationProblem,
    ports: &BTreeMap<PortBinding, Composition>,
    port: &PortBinding,
    visited: &mut BTreeSet<PortBinding>,
) -> EoResult<(Composition, PortBinding)> {
    let values = &ports[port];
    let total = values.values().sum::<f64>();
    if !total.is_finite() || total < 0.0 {
        return Err(error(
            EquationFailureKind::PhysicalDomain,
            "port_total",
            "port total is nonfinite or negative",
        ));
    }
    if total > 0.0 {
        return Ok((
            values.iter().map(|(c, n)| (c.clone(), n / total)).collect(),
            port.clone(),
        ));
    }
    if !visited.insert(port.clone()) {
        return Err(error(
            EquationFailureKind::UndefinedComposition,
            "zero_cycle",
            "zero-flow cycle has no composition anchor",
        ));
    }
    let unit = &p.flowsheet.units[&port.unit];
    if unit.kind == "feed" {
        let feed = &p.feeds[&unit.id];
        return Ok((feed.overall_mole_fractions.clone(), port.clone()));
    }
    let next = if matches!(
        port.port,
        MaterialPort::Inlet | MaterialPort::InletA | MaterialPort::InletB
    ) {
        let connection = p
            .connections
            .iter()
            .find(|c| c.stream_id == port.stream)
            .expect("bound connection");
        binding_for(p, &connection.source.unit_id, &connection.source.port_name)
    } else if unit.kind == "splitter" {
        binding_for(p, &unit.id, "inlet")
    } else {
        return Err(error(
            EquationFailureKind::UndefinedComposition,
            "zero_mixer",
            "zero-flow Mixer has no supported local composition definition",
        ));
    };
    composition_anchor(p, ports, &next, visited)
}
