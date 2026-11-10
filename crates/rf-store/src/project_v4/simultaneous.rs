//! Persisted affine modular settings, independent of solver and UI runtime types.
use super::{StoredFixedBoundary, StoredPortBinding};
use rf_types::{ComponentId, RfError, RfResult, StreamId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SIMULTANEOUS_METHOD: &str = "radishflow.fixed-tp-affine-modular-lu";

#[derive(Debug, Clone, PartialEq)]
pub struct StoredModularPlan {
    pub fixed_boundary: StoredFixedBoundary,
    pub scaling: StoredCouplingScaling,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredCouplingScaling {
    EvaluatedFreshTotal,
    ExplicitCouplings { values: Vec<StoredCouplingScale> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredCouplingScale {
    pub stream_id: StreamId,
    pub source: StoredPortBinding,
    pub sink: StoredPortBinding,
    pub component_id: ComponentId,
    pub variable_mol_s: f64,
    pub residual_mol_s: f64,
}

impl StoredCouplingScale {
    fn identity(&self) -> (&str, &str, &str, &str, &str, &str) {
        (
            self.stream_id.as_str(),
            self.source.unit_id.as_str(),
            &self.source.port,
            self.sink.unit_id.as_str(),
            &self.sink.port,
            self.component_id.as_str(),
        )
    }
}

impl StoredModularPlan {
    pub fn validate(&self) -> RfResult<()> {
        let b = &self.fixed_boundary;
        if !b.temperature_k.is_finite()
            || b.temperature_k <= 0.0
            || !b.pressure_pa.is_finite()
            || b.pressure_pa <= 0.0
        {
            return Err(RfError::invalid_input(
                "simulation: fixed T/P must be positive finite",
            ));
        }
        if let StoredCouplingScaling::ExplicitCouplings { values } = &self.scaling {
            let mut seen = BTreeSet::new();
            for v in values {
                let (s, u, p, t, q, c) = v.identity();
                if [s, u, p, t, q, c].iter().any(|s| s.trim().is_empty())
                    || !seen.insert(v.identity())
                    || !v.variable_mol_s.is_finite()
                    || v.variable_mol_s <= 0.0
                    || !v.residual_mol_s.is_finite()
                    || v.residual_mol_s <= 0.0
                {
                    return Err(RfError::invalid_input(
                        "simulation: coupling identities must be nonblank and unique; both scales must be positive finite",
                    )
                    .with_related_stream_id(v.stream_id.clone())
                    .with_related_component_id(v.component_id.clone()));
                }
            }
        }
        Ok(())
    }

    pub(super) fn canonical_scaling(&self) -> StoredCouplingScaling {
        let mut scaling = self.scaling.clone();
        if let StoredCouplingScaling::ExplicitCouplings { values } = &mut scaling {
            values.sort_by(|a, b| a.identity().cmp(&b.identity()));
        }
        scaling
    }
}
