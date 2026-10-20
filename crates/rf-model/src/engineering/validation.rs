use super::{EngineeringBasis, SourceObservationRef};
use rf_types::{BasisEntryId, SourceId, units::QuantityKind};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineeringDataErrorKind {
    EmptyId,
    DuplicateId,
    NonFiniteValue,
    InvalidOperatingConditions,
    InvalidAccessDate,
    MissingSource(SourceId),
    MissingBasis(BasisEntryId),
    MissingObservation(SourceObservationRef),
    ObservationCycle,
    QuantityMismatch {
        expected: QuantityKind,
        actual: QuantityKind,
    },
    MissingOverrideReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineeringDataError {
    pub path: String,
    pub kind: EngineeringDataErrorKind,
}

impl EngineeringDataError {
    pub(super) fn new(path: impl Into<String>, kind: EngineeringDataErrorKind) -> Self {
        Self {
            path: path.into(),
            kind,
        }
    }
}

impl fmt::Display for EngineeringDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.path)?;
        use EngineeringDataErrorKind::*;
        match &self.kind {
            EmptyId => f.write_str("ID must be nonempty"),
            DuplicateId => f.write_str("duplicate ID"),
            NonFiniteValue => f.write_str("value must be finite"),
            InvalidOperatingConditions => {
                f.write_str("temperature and pressure must both be finite and positive")
            }
            InvalidAccessDate => f.write_str("expected a valid ISO date YYYY-MM-DD"),
            MissingSource(id) => write!(f, "missing source {id:?}"),
            MissingBasis(id) => write!(f, "missing basis entry {id:?}"),
            MissingObservation(reference) => write!(f, "missing source observation {reference:?}"),
            ObservationCycle => f.write_str("cyclic source observation dependencies"),
            QuantityMismatch { expected, actual } => write!(
                f,
                "expected {}, got {}",
                expected.definition().id,
                actual.definition().id
            ),
            MissingOverrideReason => f.write_str("override requires a reason"),
        }
    }
}
impl std::error::Error for EngineeringDataError {}

pub(super) fn finite(value: f64, path: &str) -> Result<(), EngineeringDataError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(EngineeringDataError::new(
            path,
            EngineeringDataErrorKind::NonFiniteValue,
        ))
    }
}

pub(super) fn quantity_matches(
    expected: QuantityKind,
    actual: QuantityKind,
    path: &str,
) -> Result<(), EngineeringDataError> {
    if expected == actual {
        Ok(())
    } else {
        Err(EngineeringDataError::new(
            path,
            EngineeringDataErrorKind::QuantityMismatch { expected, actual },
        ))
    }
}

fn unique_id<'a>(
    id: &'a str,
    seen: &mut BTreeSet<&'a str>,
    path: &str,
) -> Result<(), EngineeringDataError> {
    if id.trim().is_empty() {
        Err(EngineeringDataError::new(
            path,
            EngineeringDataErrorKind::EmptyId,
        ))
    } else if !seen.insert(id) {
        Err(EngineeringDataError::new(
            path,
            EngineeringDataErrorKind::DuplicateId,
        ))
    } else {
        Ok(())
    }
}

impl EngineeringBasis {
    /// Structural validation only. Empty documentary text and Unknown sources may
    /// be retained as incomplete input; success is not an evidence-quality verdict.
    pub fn validate(&self) -> Result<(), EngineeringDataError> {
        let mut source_ids = BTreeSet::new();
        for (i, source) in self.sources.iter().enumerate() {
            let path = format!("sources[{i}]");
            unique_id(source.id.as_str(), &mut source_ids, &format!("{path}.id"))?;
            if let Some(conditions) = source.applicability.operating_conditions
                && (!conditions.temperature_k.is_finite()
                    || conditions.temperature_k <= 0.0
                    || !conditions.pressure_pa.is_finite()
                    || conditions.pressure_pa <= 0.0)
            {
                return Err(EngineeringDataError::new(
                    format!("{path}.applicability.operatingConditions"),
                    EngineeringDataErrorKind::InvalidOperatingConditions,
                ));
            }
            let mut observation_ids = BTreeSet::new();
            for (j, observation) in source.observations.iter().enumerate() {
                let path = format!("{path}.observations[{j}]");
                unique_id(
                    observation.id.as_str(),
                    &mut observation_ids,
                    &format!("{path}.id"),
                )?;
                finite(observation.original_value, &format!("{path}.originalValue"))?;
                finite(
                    observation.canonical_value,
                    &format!("{path}.canonicalValue"),
                )?;
            }
        }
        let mut basis_ids = BTreeSet::new();
        for (i, entry) in self.basis_entries.iter().enumerate() {
            let path = format!("basisEntries[{i}]");
            unique_id(entry.id.as_str(), &mut basis_ids, &format!("{path}.id"))?;
            finite(entry.value.value_si, &format!("{path}.valueSi"))?;
            if !source_ids.contains(entry.source_id.as_str()) {
                return Err(EngineeringDataError::new(
                    format!("{path}.sourceId"),
                    EngineeringDataErrorKind::MissingSource(entry.source_id.clone()),
                ));
            }
        }
        self.validate_observation_dependencies()
    }

    fn validate_observation_dependencies(&self) -> Result<(), EngineeringDataError> {
        let mut nodes = Vec::new();
        let mut indices = BTreeMap::new();
        for (i, source) in self.sources.iter().enumerate() {
            for (j, observation) in source.observations.iter().enumerate() {
                indices.insert(
                    SourceObservationRef {
                        source_id: source.id.clone(),
                        observation_id: observation.id.clone(),
                    },
                    nodes.len(),
                );
                nodes.push((observation, format!("sources[{i}].observations[{j}]")));
            }
        }
        // Iterative topological traversal avoids stack growth on long valid chains.
        let mut pending = vec![0usize; nodes.len()];
        let mut dependents = vec![Vec::new(); nodes.len()];
        for (i, (observation, path)) in nodes.iter().enumerate() {
            for (j, dependency) in observation.depends_on.iter().enumerate() {
                let Some(&index) = indices.get(dependency) else {
                    return Err(EngineeringDataError::new(
                        format!("{path}.dependsOn[{j}]"),
                        EngineeringDataErrorKind::MissingObservation(dependency.clone()),
                    ));
                };
                pending[i] += 1;
                dependents[index].push(i);
            }
        }
        let mut ready: VecDeque<_> = pending
            .iter()
            .enumerate()
            .filter_map(|(i, &n)| (n == 0).then_some(i))
            .collect();
        while let Some(index) = ready.pop_front() {
            for &dependent in &dependents[index] {
                pending[dependent] -= 1;
                if pending[dependent] == 0 {
                    ready.push_back(dependent);
                }
            }
        }
        if let Some(index) = pending.iter().position(|&n| n != 0) {
            return Err(EngineeringDataError::new(
                format!("{}.dependsOn", nodes[index].1),
                EngineeringDataErrorKind::ObservationCycle,
            ));
        }
        Ok(())
    }
}
