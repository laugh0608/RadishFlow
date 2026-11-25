use super::*;
use rf_types::{StreamId, UnitId};
pub const MIXED_METHOD: &str = "radishflow.fixed-tp-mixed-material";
pub const MIXED_RECYCLE_METHOD: &str = "radishflow.fixed-tp-mixed-recycle";
pub const MIXED_SERIAL_RECYCLE_METHOD: &str = "radishflow.fixed-tp-mixed-serial-recycle";

#[derive(Debug, Clone, PartialEq)]
pub struct StoredMixedRecyclePlan {
    pub mixed: StoredMixedPlan,
    pub max_iteration_evaluations: usize,
    pub tear: StoredTear,
}

impl StoredMixedRecyclePlan {
    pub(super) fn validate(&self) -> RfResult<()> {
        self.mixed.validate()?;
        if !(1..=500).contains(&self.max_iteration_evaluations)
            || self.tear.residual_scaling != StoredResidualScaling::EvaluatedFreshTotal
        {
            return Err(RfError::invalid_input(
                "simulation: mixed recycle requires a budget in 1..=500 and fresh-total residual scaling",
            ));
        }
        self.tear.validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoredMixedSerialRecyclePlan {
    pub mixed: StoredMixedPlan,
    pub recycle_regions: [StoredMixedRecycleRegion; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredMixedRecycleRegion {
    pub tear: StoredMixedSerialRecycleTear,
    pub max_iteration_evaluations: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredMixedSerialRecycleTear {
    pub stream_id: StreamId,
    pub source: StoredPortBinding,
    pub sink: StoredPortBinding,
    pub initialization: StoredTearInitialization,
}

impl StoredMixedSerialRecyclePlan {
    pub(super) fn validate(&self) -> RfResult<()> {
        self.mixed.validate()?;
        for region in &self.recycle_regions {
            let tear = &region.tear;
            if !(1..=500).contains(&region.max_iteration_evaluations) {
                return Err(RfError::invalid_input(
                    "simulation.recycleRegions: iteration budget must be in 1..=500",
                )
                .with_related_stream_id(tear.stream_id.clone()));
            }
            plan::validate_tear_inputs(
                &tear.stream_id,
                &tear.source,
                &tear.sink,
                &tear.initialization,
            )?;
        }
        let [first, second] = self.recycle_regions.each_ref().map(|region| &region.tear);
        if first.stream_id == second.stream_id
            || first.source == second.source
            || first.sink == second.sink
        {
            return Err(RfError::invalid_input(
                "simulation.recycleRegions: repeated tear stream or port binding",
            )
            .with_related_stream_id(first.stream_id.clone())
            .with_related_stream_id(second.stream_id.clone()));
        }
        Ok(())
    }

    pub(super) fn canonicalized(&self) -> Self {
        let mut recycle_regions = self.recycle_regions.clone();
        recycle_regions.sort_by(|a, b| a.tear.stream_id.cmp(&b.tear.stream_id));
        for region in &mut recycle_regions {
            region.tear.initialization.canonicalize();
        }
        Self {
            mixed: self.mixed.canonicalized(),
            recycle_regions,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredMixedPlan {
    pub fixed_boundary: StoredFixedBoundary,
    pub blocks: Vec<StoredMixedBlock>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredMixedBlock {
    pub id: String,
    pub members: Vec<UnitId>,
    pub method: StoredMixedBlockMethod,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StoredMixedBlockMethod {
    Local,
    EquationOriented {
        equation_contract_version: u32,
        scaling: StoredEquationScaling,
    },
}
impl StoredMixedPlan {
    pub fn validate(&self) -> RfResult<()> {
        let invalid = || {
            RfError::invalid_input(
                "simulation: invalid Mixed blocks, identity, capability or scale",
            )
        };
        let b = &self.fixed_boundary;
        if !b.temperature_k.is_finite()
            || b.temperature_k <= 0.0
            || !b.pressure_pa.is_finite()
            || b.pressure_pa <= 0.0
        {
            return Err(invalid());
        }
        let mut ids = BTreeSet::new();
        let mut members = BTreeSet::new();
        for block in &self.blocks {
            if block.id.trim().is_empty()
                || !ids.insert(&block.id)
                || block.members.is_empty()
                || matches!(block.method, StoredMixedBlockMethod::Local) && block.members.len() != 1
            {
                return Err(invalid());
            }
            for id in &block.members {
                if id.as_str().trim().is_empty() || !members.insert(id) {
                    return Err(invalid());
                }
            }
            if let StoredMixedBlockMethod::EquationOriented {
                equation_contract_version,
                scaling,
            } = &block.method
            {
                if *equation_contract_version != 1 {
                    return Err(invalid());
                }
                scaling.validate(true)?;
                if let StoredEquationScaling::ExplicitEquations {
                    variables,
                    equations,
                } = scaling
                    && (variables.iter().any(|s| {
                        matches!(s.binding, StoredEquationVariableBinding::Parameter { .. })
                    }) || equations.iter().any(|s| {
                        matches!(
                            s.binding,
                            StoredEquationBinding::ProductTotal { .. }
                                | StoredEquationBinding::ProductMoleFraction { .. }
                        )
                    }))
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
    pub fn canonicalized(&self) -> Self {
        let mut p = self.clone();
        p.blocks.sort_by(|a, b| a.id.cmp(&b.id));
        for b in &mut p.blocks {
            b.members.sort();
            if let StoredMixedBlockMethod::EquationOriented {
                scaling:
                    StoredEquationScaling::ExplicitEquations {
                        variables,
                        equations,
                    },
                ..
            } = &mut b.method
            {
                variables.sort_by(|a, b| a.binding.cmp(&b.binding));
                equations.sort_by(|a, b| a.binding.cmp(&b.binding));
            }
        }
        p
    }
}
