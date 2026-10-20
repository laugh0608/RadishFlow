//! Evidence resolution shared by separator sizing and given-geometry rating.
//! Capturing provenance never upgrades a source's kind or review status.
use rf_model::engineering::{
    BasisEntry, BindingOrigin, EngineeringBasis, EngineeringDataError, InputBinding, SourceKind,
    SourcePhase, SourceRecord, SourceReview,
};
use rf_types::{BasisEntryId, SourceId, units::QuantityKind};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const COMPARISON_RELATIVE_TOLERANCE: f64 = 1e-10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EvidenceIssue {
    InvalidBinding(EngineeringDataError),
    MissingSource(SourceId),
    MissingSourceEvidence,
    SourceNotAdopted,
}

pub(crate) struct SourceEvidence<'a> {
    pub basis_entries: &'a mut BTreeMap<BasisEntryId, BasisEntry>,
    pub sources: &'a mut BTreeMap<SourceId, SourceRecord>,
}
impl SourceEvidence<'_> {
    /// A structurally resolved value is returned even if its documentary evidence
    /// is insufficient, so callers can still reject malformed optional dimensions.
    pub fn resolve(
        &mut self,
        binding: &InputBinding,
        quantity: QuantityKind,
        basis: &EngineeringBasis,
    ) -> (Option<f64>, Vec<EvidenceIssue>) {
        let resolved = match basis.resolve(quantity, binding) {
            Ok(value) => value,
            Err(error) => return (None, vec![EvidenceIssue::InvalidBinding(error)]),
        };
        let mut sources = vec![(resolved.source.id.clone(), true)];
        if let BindingOrigin::Basis(entry) | BindingOrigin::Override { basis: entry, .. } =
            resolved.origin
        {
            self.basis_entries.insert(entry.id.clone(), entry.clone());
            // Original overridden evidence remains traceable but is not required
            // to qualify the explicitly adopted replacement source.
            sources.push((entry.source_id.clone(), false));
        }
        (
            Some(resolved.value.value_si),
            self.capture_sources(sources, basis),
        )
    }

    pub fn capture_source(
        &mut self,
        id: &SourceId,
        basis: &EngineeringBasis,
    ) -> Vec<EvidenceIssue> {
        self.capture_sources(vec![(id.clone(), true)], basis)
    }

    fn capture_sources(
        &mut self,
        mut pending: Vec<(SourceId, bool)>,
        basis: &EngineeringBasis,
    ) -> Vec<EvidenceIssue> {
        let mut issues = Vec::new();
        let mut checked = BTreeSet::new();
        while let Some((id, active)) = pending.pop() {
            if !checked.insert((id.clone(), active)) {
                continue;
            }
            let Some(source) = basis.sources.iter().find(|s| s.id == id) else {
                issues.push(EvidenceIssue::MissingSource(id));
                continue;
            };
            if active {
                let has_reference =
                    !source.reference.trim().is_empty() && !source.locator.trim().is_empty();
                let has_assumption = !source.adoption_note.trim().is_empty();
                if source.kind == SourceKind::Unknown
                    || source.applicability.scope.trim().is_empty()
                    || (source.kind == SourceKind::Assumed && !has_assumption)
                    || (source.kind != SourceKind::Assumed && !has_reference)
                {
                    issues.push(EvidenceIssue::MissingSourceEvidence);
                }
                if source.review == SourceReview::Unreviewed {
                    issues.push(EvidenceIssue::SourceNotAdopted);
                }
            }
            self.sources.insert(id, source.clone());
            for observation in &source.observations {
                pending.extend(
                    observation
                        .depends_on
                        .iter()
                        .map(|r| (r.source_id.clone(), active)),
                );
            }
        }
        issues
    }
}

#[derive(Clone, Copy)]
pub(crate) enum SourceUse {
    General,
    Gas,
    Liquid,
    EffectiveK,
}
pub(crate) struct OperatingContext<'a> {
    pub fluid_system: &'a str,
    pub temperature_k: Option<f64>,
    pub pressure_pa: Option<f64>,
}

/// Geometry and generic criteria do not require invented fluid/T/P data.
/// Explicitly supplied restrictions still have to match the evaluated case.
pub(crate) fn conditions_match(
    source: &SourceRecord,
    usage: SourceUse,
    context: &OperatingContext<'_>,
) -> bool {
    let a = &source.applicability;
    let phase = match usage {
        SourceUse::Gas => Some(SourcePhase::Gas),
        SourceUse::Liquid => Some(SourcePhase::Liquid),
        SourceUse::General | SourceUse::EffectiveK => None,
    };
    let needs_fluid = phase.is_some() || matches!(usage, SourceUse::EffectiveK);
    let fluid_mismatch = (needs_fluid || !a.fluid_system.trim().is_empty())
        && a.fluid_system.trim() != context.fluid_system.trim();
    let phase_mismatch = phase.is_some_and(|p| a.phase != p && a.phase != SourcePhase::Both);
    let conditions_mismatch = match a.operating_conditions {
        None => phase.is_some(),
        Some(conditions) => [
            (context.temperature_k, conditions.temperature_k),
            (context.pressure_pa, conditions.pressure_pa),
        ]
        .iter()
        .any(|(value, source_value)| value.is_some_and(|v| !numerically_equal(v, *source_value))),
    };
    !fluid_mismatch && !phase_mismatch && !conditions_mismatch
}

pub(crate) fn numerically_equal(a: f64, b: f64) -> bool {
    (a - b).abs() / a.abs().max(b.abs()) <= COMPARISON_RELATIVE_TOLERANCE
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UncheckedRequirement {
    PhaseEquilibrium,
    EntrainmentAndEfficiency,
    PressureDrop,
    SlugHandling,
    MechanicalDesignAndCertification,
    StandardDiameterRounding,
    MultipleOperatingCases,
}
pub(crate) const UNCHECKED_REQUIREMENTS: &[UncheckedRequirement] = &[
    UncheckedRequirement::PhaseEquilibrium,
    UncheckedRequirement::EntrainmentAndEfficiency,
    UncheckedRequirement::PressureDrop,
    UncheckedRequirement::SlugHandling,
    UncheckedRequirement::MechanicalDesignAndCertification,
    UncheckedRequirement::StandardDiameterRounding,
    UncheckedRequirement::MultipleOperatingCases,
];
