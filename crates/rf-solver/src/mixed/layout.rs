//! Symbolic block bindings only; no boundary values, property calls or solvability claim.
use super::*;
use crate::equation_oriented::{EquationBinding, PortBinding, VariableBinding};
use rf_flowsheet::MaterialConnection;
use rf_unitops::{BuiltinUnitKind, catalog::builtin_model};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquationBlockLayout {
    pub variables: Vec<VariableBinding>,
    pub equations: Vec<EquationBinding>,
}
pub(super) fn layout(
    flow: &Flowsheet,
    block: &MixedBlock,
    connections: &[MaterialConnection],
) -> RfResult<EquationBlockLayout> {
    let mut ports = BTreeMap::new();
    let mut variables = Vec::new();
    let mut equations = Vec::new();
    for id in &block.members {
        let u = &flow.units[id];
        let capability = BuiltinUnitKind::parse(&u.kind)
            .and_then(|k| builtin_model(k).component_equation_capability())
            .ok_or_else(|| {
                failure("capability", "no material equation capability")
                    .with_related_unit_id(id.clone())
            })?;
        for &port in capability.ports() {
            let stream = u
                .ports
                .iter()
                .find(|p| p.name == port.name())
                .and_then(|p| p.stream_id.clone())
                .ok_or_else(|| failure("port", "unbound equation port"))?;
            let binding = PortBinding {
                unit: id.clone(),
                port,
                stream,
            };
            ports.insert((id.clone(), port.name()), binding.clone());
            for component in flow.components.keys() {
                variables.push(VariableBinding::PortComponent {
                    port: binding.clone(),
                    component: component.clone(),
                });
            }
        }
        for &role in capability.rows() {
            for component in flow.components.keys() {
                equations.push(EquationBinding::Unit {
                    unit: id.clone(),
                    role,
                    component: component.clone(),
                });
            }
        }
    }
    for c in connections {
        if let Some(sink) = &c.sink
            && block.members.contains(&sink.unit_id)
        {
            let sink = ports
                .iter()
                .find(|((u, p), _)| *u == sink.unit_id && *p == sink.port_name)
                .map(|(_, p)| p.clone())
                .ok_or_else(|| failure("port", "invalid boundary sink"))?;
            for component in flow.components.keys() {
                equations.push(if block.members.contains(&c.source.unit_id) {
                    let source = ports
                        .iter()
                        .find(|((u, p), _)| *u == c.source.unit_id && *p == c.source.port_name)
                        .map(|(_, p)| p.clone())
                        .ok_or_else(|| failure("port", "invalid source"))?;
                    EquationBinding::Connection {
                        source,
                        sink: sink.clone(),
                        component: component.clone(),
                    }
                } else {
                    EquationBinding::Boundary {
                        source_unit: c.source.unit_id.clone(),
                        source_port: c.source.port_name.clone(),
                        sink: sink.clone(),
                        component: component.clone(),
                    }
                });
            }
        }
    }
    variables.sort();
    equations.sort();
    Ok(EquationBlockLayout {
        variables,
        equations,
    })
}
