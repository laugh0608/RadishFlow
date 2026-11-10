use super::*;
use rf_flowsheet::{MaterialConnection, dependencies::material_dependencies};
use rf_unitops::{BuiltinUnitKind, catalog::builtin_model, validate_unit_node};

#[derive(Debug, Clone, PartialEq)]
pub struct MixedPlan {
    pub(super) flow: Flowsheet,
    pub(super) options: MixedOptions,
    pub(super) connections: Vec<MaterialConnection>,
    pub(super) ordered: Vec<MixedBlock>,
    pub(super) membership: BTreeMap<UnitId, MixedBlockId>,
}
impl MixedPlan {
    pub fn blocks(&self) -> &[MixedBlock] {
        &self.ordered
    }
    pub fn connections(&self) -> &[MaterialConnection] {
        &self.connections
    }
    pub fn options(&self) -> &MixedOptions {
        &self.options
    }
    pub fn equation_layout(&self, id: &MixedBlockId) -> RfResult<super::EquationBlockLayout> {
        let block = self
            .ordered
            .iter()
            .find(|b| &b.id == id)
            .ok_or_else(|| failure("block", "unknown block ID"))?;
        if !matches!(block.method, MixedBlockMethod::EquationOriented { .. }) {
            return Err(failure("capability", "Local block has no equation layout"));
        }
        super::layout::layout(&self.flow, block, &self.connections)
    }
}
pub fn assemble(flow: &Flowsheet, options: &MixedOptions) -> RfResult<MixedPlan> {
    options.validate_shape()?;
    if flow.units.is_empty() || flow.components.is_empty() {
        return Err(failure("empty", "nonempty flow and components required"));
    }
    if flow.units.iter().any(|(id, u)| id != &u.id)
        || flow.streams.iter().any(|(id, s)| id != &s.id)
        || flow.components.iter().any(|(id, c)| id != &c.id)
    {
        return Err(failure("identity", "registry keys must match identities"));
    }
    let dependencies = material_dependencies(flow)?;
    let mut membership = BTreeMap::new();
    for block in &options.blocks {
        for id in &block.members {
            let u = flow.units.get(id).ok_or_else(|| {
                failure("member", "block member does not exist").with_related_unit_id(id.clone())
            })?;
            if membership.insert(id.clone(), block.id.clone()).is_some() {
                return Err(failure("overlap", "unit belongs to multiple blocks")
                    .with_related_unit_id(id.clone()));
            }
            let spec = validate_unit_node(u)?;
            validate_parameters(u, options.boundary)?;
            if let Some(o) = block.equation_options(options.boundary) {
                let capability = builtin_model(spec.kind)
                    .component_equation_capability()
                    .ok_or_else(|| {
                        failure(
                            "capability",
                            "EO member has no fixed-T/P material equations",
                        )
                        .with_related_unit_id(id.clone())
                    })?;
                crate::equation_oriented::validate_equation_parameters(u, capability, &o)
                    .map_err(|e| e.error)?;
            }
            if spec.kind == BuiltinUnitKind::Splitter
                && u.parameters.outlet_a_split_fraction.is_none()
            {
                return Err(failure("specification", "fixed split fraction required")
                    .with_related_unit_id(id.clone()));
            }
        }
        if matches!(block.method, MixedBlockMethod::EquationOriented { .. }) {
            let layout = super::layout::layout(flow, block, &dependencies.connections)?;
            if let MixedBlockMethod::EquationOriented {
                scales: Some(scales),
            } = &block.method
                && (scales.variables.keys().ne(layout.variables.iter())
                    || scales.equations.keys().ne(layout.equations.iter()))
            {
                return Err(failure(
                    "scale_binding",
                    "explicit block scales must cover the exact symbolic layout",
                ));
            }
            let ports: usize = block
                .members
                .iter()
                .map(|id| flow.units[id].ports.len())
                .sum();
            if ports.saturating_mul(flow.components.len()) > crate::equation_oriented::MAX_DIMENSION
            {
                return Err(failure("capacity", "equation block exceeds 128 variables"));
            }
        }
    }
    if membership.len() != flow.units.len() {
        return Err(failure(
            "coverage",
            "every engineering unit must belong to exactly one block",
        )
        .with_related_unit_ids(
            flow.units
                .keys()
                .filter(|id| !membership.contains_key(*id))
                .cloned()
                .collect(),
        ));
    }
    let mut children: BTreeMap<_, BTreeSet<_>> = options
        .blocks
        .iter()
        .map(|b| (b.id.clone(), BTreeSet::new()))
        .collect();
    for c in &dependencies.connections {
        if let Some(sink) = &c.sink {
            let a = &membership[&c.source.unit_id];
            let b = &membership[&sink.unit_id];
            if a != b {
                children
                    .get_mut(a)
                    .expect("registered block")
                    .insert(b.clone());
            } else if matches!(
                options
                    .blocks
                    .iter()
                    .find(|x| &x.id == a)
                    .expect("block")
                    .method,
                MixedBlockMethod::Local
            ) {
                return Err(failure(
                    "local_cycle",
                    "Local block cannot contain a feedback connection",
                )
                .with_related_stream_id(c.stream_id.clone()));
            }
        }
    }
    let mut counts: BTreeMap<_, usize> = children.keys().cloned().map(|b| (b, 0)).collect();
    for next in children.values().flatten() {
        *counts.get_mut(next).expect("block") += 1;
    }
    let mut ready: BTreeSet<_> = counts
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(id, _)| id.clone())
        .collect();
    let mut ordered = Vec::new();
    while let Some(id) = ready.pop_first() {
        ordered.push(
            options
                .blocks
                .iter()
                .find(|b| b.id == id)
                .expect("block")
                .clone(),
        );
        for next in &children[&id] {
            let n = counts.get_mut(next).expect("block");
            *n -= 1;
            if *n == 0 {
                ready.insert(next.clone());
            }
        }
    }
    if ordered.len() != options.blocks.len() {
        return Err(failure(
            "cross_block_cycle",
            "cross-block cycles are unsupported; keep each SCC inside a capable block",
        )
        .with_related_stream_ids(
            dependencies
                .connections
                .iter()
                .filter(|c| {
                    c.sink
                        .as_ref()
                        .is_some_and(|s| membership[&s.unit_id] != membership[&c.source.unit_id])
                })
                .map(|c| c.stream_id.clone())
                .collect(),
        ));
    }
    Ok(MixedPlan {
        flow: flow.clone(),
        options: options.canonicalized(),
        connections: dependencies.connections,
        ordered,
        membership,
    })
}

fn validate_parameters(u: &rf_model::UnitNode, b: FixedRecycleBoundary) -> RfResult<()> {
    use rf_unitops::specifications::{UnitParameter, input_specification};
    let kind = BuiltinUnitKind::parse(&u.kind).expect("validated model");
    let contract = input_specification(kind);
    if u.parameters
        .outlet_temperature_k
        .is_some_and(|t| t != b.temperature_k)
        || u.parameters
            .outlet_pressure_pa
            .is_some_and(|p| p != b.pressure_pa)
        || [
            UnitParameter::OutletTemperature,
            UnitParameter::OutletPressure,
            UnitParameter::OutletASplitFraction,
        ]
        .iter()
        .any(|p| p.value(&u.parameters).is_some() && !contract.parameters.contains(p))
    {
        return Err(failure(
            "specification",
            "physical parameters must agree with model capability and fixed T/P",
        )
        .with_related_unit_id(u.id.clone()));
    }
    Ok(())
}
