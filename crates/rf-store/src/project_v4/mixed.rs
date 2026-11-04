use super::*;
use rf_types::UnitId;
pub const MIXED_METHOD: &str = "radishflow.fixed-tp-mixed-material";
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
                    }) || equations
                        .iter()
                        .any(|s| matches!(s.binding, StoredEquationBinding::ProductTotal { .. })))
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
