//! Engineering basis and documentary provenance, independent of equipment methods.
//! Resolving a binding identifies a value and its source; it does not certify evidence
//! or permit an engineering calculation. No task, solver or application history lives here.
mod access_date;
mod validation;
pub use access_date::SourceAccessDate;
pub use validation::{EngineeringDataError, EngineeringDataErrorKind};

use rf_types::{BasisEntryId, SourceId, SourceObservationId, units::QuantityKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineeringValue {
    pub quantity: QuantityKind,
    pub value_si: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BasisEntry {
    pub id: BasisEntryId,
    pub revision: u64,
    pub name: String,
    pub value: EngineeringValue,
    pub source_id: SourceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Measured,
    Vendor,
    Literature,
    Estimated,
    Assumed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceReview {
    Unreviewed,
    Adopted,
    Reviewed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourcePhase {
    Gas,
    Liquid,
    Both,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceOperatingConditions {
    pub temperature_k: f64,
    pub pressure_pa: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceApplicability {
    pub fluid_system: String,
    pub phase: SourcePhase,
    pub operating_conditions: Option<SourceOperatingConditions>,
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceObservationRef {
    pub source_id: SourceId,
    pub observation_id: SourceObservationId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceObservation {
    pub id: SourceObservationId,
    pub quantity: QuantityKind,
    pub original_value: f64,
    pub original_unit: String,
    pub canonical_value: f64,
    pub conversion_note: String,
    pub depends_on: Vec<SourceObservationRef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRecord {
    pub id: SourceId,
    pub revision: u64,
    pub kind: SourceKind,
    pub title: String,
    pub reference: String,
    pub locator: String,
    pub publication_version: String,
    pub accessed_on: Option<SourceAccessDate>,
    pub adoption_note: String,
    pub review: SourceReview,
    pub applicability: SourceApplicability,
    pub observations: Vec<SourceObservation>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputBinding {
    Direct {
        value: EngineeringValue,
        source_id: SourceId,
    },
    BasisRef {
        basis_entry_id: BasisEntryId,
    },
    Override {
        basis_entry_id: BasisEntryId,
        value: EngineeringValue,
        source_id: SourceId,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EngineeringBasis {
    pub basis_entries: Vec<BasisEntry>,
    pub sources: Vec<SourceRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BindingOrigin<'a> {
    Direct,
    Basis(&'a BasisEntry),
    Override {
        basis: &'a BasisEntry,
        reason: &'a str,
    },
}

/// Borrowed structural resolution. Source kind/review stay visible and unchanged;
/// callers must separately check evidence sufficiency and method applicability.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedBasisValue<'a> {
    pub value: EngineeringValue,
    pub source: &'a SourceRecord,
    pub origin: BindingOrigin<'a>,
}

impl EngineeringBasis {
    pub fn resolve<'a>(
        &'a self,
        expected_quantity: QuantityKind,
        binding: &'a InputBinding,
    ) -> Result<ResolvedBasisValue<'a>, EngineeringDataError> {
        self.validate()?;
        use EngineeringDataErrorKind as Error;
        let basis = |id: &BasisEntryId| {
            self.basis_entries
                .iter()
                .find(|entry| entry.id == *id)
                .ok_or_else(|| {
                    EngineeringDataError::new(
                        "binding.basisEntryId",
                        Error::MissingBasis(id.clone()),
                    )
                })
        };
        let (value, source_id, origin) = match binding {
            InputBinding::Direct { value, source_id } => (*value, source_id, BindingOrigin::Direct),
            InputBinding::BasisRef { basis_entry_id } => {
                let entry = basis(basis_entry_id)?;
                (entry.value, &entry.source_id, BindingOrigin::Basis(entry))
            }
            InputBinding::Override {
                basis_entry_id,
                value,
                source_id,
                reason,
            } => {
                let entry = basis(basis_entry_id)?;
                validation::quantity_matches(
                    entry.value.quantity,
                    value.quantity,
                    "binding.quantityId",
                )?;
                if reason.trim().is_empty() {
                    return Err(EngineeringDataError::new(
                        "binding.reason",
                        Error::MissingOverrideReason,
                    ));
                }
                (
                    *value,
                    source_id,
                    BindingOrigin::Override {
                        basis: entry,
                        reason,
                    },
                )
            }
        };
        validation::finite(value.value_si, "binding.valueSi")?;
        validation::quantity_matches(expected_quantity, value.quantity, "binding.quantityId")?;
        let source = self
            .sources
            .iter()
            .find(|source| source.id == *source_id)
            .ok_or_else(|| {
                EngineeringDataError::new(
                    "binding.sourceId",
                    Error::MissingSource(source_id.clone()),
                )
            })?;
        Ok(ResolvedBasisValue {
            value,
            source,
            origin,
        })
    }
}

#[cfg(test)]
mod tests;
