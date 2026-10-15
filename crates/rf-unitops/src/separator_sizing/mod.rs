//! Conditional, single-case vertical separator sizing. No IO, Flash, flowsheet,
//! application history, persistence or current-result qualification is performed.
mod calculation;
mod evidence;

use rf_model::{
    engineering::{BasisEntry, EngineeringDataError, SourceRecord},
    equipment::*,
};
use rf_types::{BasisEntryId, SourceId};
use std::collections::BTreeMap;

pub const METHOD_ID: &str = "radishflow.separator.vertical-sb-working-volume";
pub const METHOD_VERSION: u32 = 1;
/// Relative numerical comparison only, not an engineering uncertainty budget.
pub const COMPARISON_RELATIVE_TOLERANCE: f64 = 1e-10;

pub fn method() -> EquipmentMethod {
    EquipmentMethod {
        id: METHOD_ID.into(),
        version: METHOD_VERSION,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicabilityField {
    Orientation,
    GasDirection,
    Service,
    Geometry,
    KBasis,
    FlowBasis,
    Internals,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueTarget {
    Task,
    Case,
    Method,
    Basis,
    Applicability(ApplicabilityField),
    Input(SeparatorInput),
    AxialSection(SeparatorAxialSection),
    Source(SourceId),
    Calculation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculationStep {
    DensityRatio,
    SoudersBrownVelocity,
    AllowedVelocity,
    GasArea,
    WorkingVolume,
    LiquidArea,
    DiameterSquared,
    Diameter,
    WorkingHeight,
    GasSuperficialVelocity,
    GasLoad,
    TangentHeight,
    BufferVolume,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SizingIssueKind {
    MissingInput,
    InvalidIdentity(EquipmentIdentityErrorKind),
    InvalidBinding(EngineeringDataError),
    UnsupportedMethod,
    OutsideMethodScope,
    NonPositiveInput,
    UtilizationAboveOne,
    AbsentPhase,
    InvalidDensityOrder,
    MissingSourceEvidence,
    MissingSource(SourceId),
    SourceNotAdopted,
    SourceConditionMismatch,
    NumericRange(CalculationStep),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizingIssue {
    pub target: IssueTarget,
    pub kind: SizingIssueKind,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SizingFailure {
    pub issues: Vec<SizingIssue>,
}
impl std::fmt::Display for SizingFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "separator sizing rejected: {:?}", self.issues)
    }
}
impl std::error::Error for SizingFailure {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllingConstraint {
    GasArea,
    LiquidWorkingVolume,
    Both,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorDimensions {
    pub souders_brown_velocity_m_s: f64,
    pub allowed_gas_velocity_m_s: f64,
    pub gas_area_m2: f64,
    pub liquid_area_m2: f64,
    pub required_area_m2: f64,
    pub working_volume_m3: f64,
    pub minimum_diameter_m: f64,
    pub working_liquid_height_m: f64,
    pub gas_load_fraction: f64,
    pub controlling_constraint: ControllingConstraint,
    /// Sum of explicit, nonoverlapping intervals; excludes heads and skirt.
    pub tangent_to_tangent_height_m: Option<f64>,
    /// NLL -> HLL only, separate from LLL -> NLL working volume.
    pub buffer_volume_m3: Option<f64>,
}

/// Owned evidence for this call. This is not a persisted/current application result.
#[derive(Debug, Clone, PartialEq)]
pub struct SizingInputCapture {
    pub task: SeparatorSizingTask,
    pub values_si: BTreeMap<SeparatorInput, f64>,
    pub axial_lengths_m: BTreeMap<SeparatorAxialSection, f64>,
    pub basis_entries: BTreeMap<BasisEntryId, BasisEntry>,
    /// Includes transitive observation dependencies and original basis provenance.
    pub sources: BTreeMap<SourceId, SourceRecord>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorCandidate {
    capture: SizingInputCapture,
    dimensions: SeparatorDimensions,
    assumed_sources: Vec<SourceId>,
    pending_review_sources: Vec<SourceId>,
    missing_axial_sections: Vec<SeparatorAxialSection>,
    height_issues: Vec<SizingIssue>,
}
impl SeparatorCandidate {
    pub fn capture(&self) -> &SizingInputCapture {
        &self.capture
    }
    pub fn dimensions(&self) -> &SeparatorDimensions {
        &self.dimensions
    }
    pub fn assumed_sources(&self) -> &[SourceId] {
        &self.assumed_sources
    }
    pub fn pending_review_sources(&self) -> &[SourceId] {
        &self.pending_review_sources
    }
    pub fn missing_axial_sections(&self) -> &[SeparatorAxialSection] {
        &self.missing_axial_sections
    }
    pub fn height_issues(&self) -> &[SizingIssue] {
        &self.height_issues
    }
    /// Every result is a conditional preliminary process size, even when reviewed.
    pub fn unchecked_requirements(&self) -> &'static [UncheckedRequirement] {
        use UncheckedRequirement::*;
        &[
            PhaseEquilibrium,
            EntrainmentAndEfficiency,
            PressureDrop,
            SlugHandling,
            MechanicalDesignAndCertification,
            StandardDiameterRounding,
            MultipleOperatingCases,
        ]
    }
}

pub use calculation::calculate;
pub use evidence::check;

#[cfg(test)]
mod tests;
