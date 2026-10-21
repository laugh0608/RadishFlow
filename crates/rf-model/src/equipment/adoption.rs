//! Fixed evidence for a design decision, independent of execution eligibility.
use super::*;
use crate::engineering::EngineeringBasis;

#[derive(Debug, Clone, PartialEq)]
pub struct AdoptedDimension {
    pub original_m: f64,
    pub adopted_m: f64,
    /// Empty only when the original value was used without adjustment.
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorGeometryAdoption {
    pub source_document_id: String,
    pub source_task: SeparatorSizingTask,
    pub source_basis: EngineeringBasis,
    pub diameter: AdoptedDimension,
    pub working_liquid_height: AdoptedDimension,
    /// Only valid captured segments. Missing segments remain unknown.
    pub axial_lengths_m: BTreeMap<SeparatorAxialSection, f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeometryAdoptionError {
    MissingDocument,
    InvalidSourceIdentity,
    InvalidSourceBasis,
    InvalidSourceBinding,
    MissingSourceEvidence,
    InvalidDimension,
    MissingAdjustmentReason,
    InvalidAxialSection,
}
impl std::fmt::Display for GeometryAdoptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid geometry adoption: {self:?}")
    }
}
impl std::error::Error for GeometryAdoptionError {}

impl AdoptedDimension {
    pub fn validate(&self) -> Result<(), GeometryAdoptionError> {
        if !self.original_m.is_finite()
            || self.original_m <= 0.0
            || !self.adopted_m.is_finite()
            || self.adopted_m <= 0.0
        {
            return Err(GeometryAdoptionError::InvalidDimension);
        }
        if self.original_m != self.adopted_m && self.reason.trim().is_empty() {
            return Err(GeometryAdoptionError::MissingAdjustmentReason);
        }
        Ok(())
    }
}
impl SeparatorGeometryAdoption {
    pub fn validate(&self) -> Result<(), GeometryAdoptionError> {
        use GeometryAdoptionError::*;
        if self.source_document_id.trim().is_empty() {
            return Err(MissingDocument);
        }
        validate_equipment_identities(std::slice::from_ref(&self.source_task))
            .map_err(|_| InvalidSourceIdentity)?;
        if self.source_task.method.id.trim().is_empty() || self.source_task.method.version == 0 {
            return Err(InvalidSourceIdentity);
        }
        self.source_basis
            .validate()
            .map_err(|_| InvalidSourceBasis)?;
        for field in SeparatorInput::ALL {
            let binding = self
                .source_task
                .case
                .inputs
                .get(&field)
                .ok_or(InvalidSourceBinding)?;
            self.source_basis
                .resolve(field.quantity(), binding)
                .map_err(|_| InvalidSourceBinding)?;
        }
        if self.source_task.case.applicability_source_ids.is_empty()
            || self
                .source_task
                .case
                .applicability_source_ids
                .iter()
                .any(|id| {
                    !self
                        .source_basis
                        .sources
                        .iter()
                        .any(|source| source.id == *id)
                })
        {
            return Err(MissingSourceEvidence);
        }
        // Optional original sections may be invalid for the sizing method, but
        // their archived bindings must still be structurally interpretable.
        for binding in self.source_task.axial_sections.values() {
            self.source_basis
                .resolve(QuantityKind::Length, binding)
                .map_err(|_| InvalidSourceBinding)?;
        }
        for (section, value) in &self.axial_lengths_m {
            if !value.is_finite()
                || *value < 0.0
                || (*value == 0.0 && section.requires_positive_clearance())
            {
                return Err(InvalidAxialSection);
            }
            let binding = self
                .source_task
                .axial_sections
                .get(section)
                .ok_or(InvalidAxialSection)?;
            if self
                .source_basis
                .resolve(QuantityKind::Length, binding)
                .map_err(|_| InvalidAxialSection)?
                .value
                .value_si
                != *value
            {
                return Err(InvalidAxialSection);
            }
        }
        self.diameter.validate()?;
        self.working_liquid_height.validate()
    }
}
