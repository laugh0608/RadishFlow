//! Conditional single-case checks against given separator geometry. Exceeded
//! criteria are successful calculations, not failures. No IO, session validity,
//! persistence, flowsheet coupling or geometry adoption occurs here.
mod calculation;
mod evidence;
#[cfg(test)]
mod tests;

use rf_model::{
    engineering::{BasisEntry, EngineeringDataError, SourceRecord},
    equipment::*,
};
use rf_types::{BasisEntryId, EngineeringCaseId, SourceId};
use std::collections::BTreeMap;

pub use crate::separator_evidence::UncheckedRequirement;
pub use calculation::calculate;
pub use evidence::check;

pub const METHOD_ID: &str = "radishflow.separator.vertical-sb-working-volume-rating";
pub const METHOD_VERSION: u32 = 1;
/// Absolute distance from a dimensionless load of 1. Numerical comparison only,
/// never an engineering margin or uncertainty budget.
pub const LOAD_BOUNDARY_TOLERANCE: f64 = 1e-10;

pub fn method() -> EquipmentMethod {
    EquipmentMethod {
        id: METHOD_ID.into(),
        version: METHOD_VERSION,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryApplicabilityField {
    Orientation,
    GasDirection,
    Geometry,
    Internals,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseApplicabilityField {
    Service,
    KBasis,
    FlowBasis,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RatingIssueTarget {
    Task,
    Method,
    Basis,
    Geometry,
    GeometryInput(SeparatorGeometryInput),
    GeometryApplicability(GeometryApplicabilityField),
    Case(EngineeringCaseId),
    Input {
        case_id: EngineeringCaseId,
        field: SeparatorRatingInput,
    },
    CaseApplicability {
        case_id: EngineeringCaseId,
        field: CaseApplicabilityField,
    },
    AxialSection(SeparatorAxialSection),
    Source(SourceId),
    Calculation,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalculationStep {
    DiameterSquared,
    GivenArea,
    AvailableWorkingVolume,
    DensityRatio,
    SoudersBrownVelocity,
    AllowedVelocity,
    GasFlowLimit,
    RequiredWorkingVolume,
    AvailableWorkingTime,
    LiquidFlowLimit,
    GasSuperficialVelocity,
    GasLoad,
    LiquidLoad,
    TangentHeight,
    BufferVolume,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RatingIssueKind {
    MissingInput,
    InvalidIdentity(EquipmentIdentityErrorKind),
    UnsupportedMethod,
    UnsupportedCaseCount { actual: usize },
    InvalidBinding(EngineeringDataError),
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
pub struct RatingIssue {
    pub target: RatingIssueTarget,
    pub kind: RatingIssueKind,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatingFailure {
    pub issues: Vec<RatingIssue>,
}
impl std::fmt::Display for RatingFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "separator rating rejected: {:?}", self.issues)
    }
}
impl std::error::Error for RatingFailure {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintStatus {
    WithinCriteria,
    BoundaryReview,
    Exceeded,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintAssessment {
    pub load_fraction: f64,
    /// 1 - load, retaining the sign and unrounded precision.
    pub margin_fraction: f64,
    pub status: ConstraintStatus,
}
impl ConstraintAssessment {
    fn from_load(load: f64) -> Self {
        debug_assert!(load.is_finite() && load > 0.0);
        Self {
            load_fraction: load,
            margin_fraction: 1.0 - load,
            status: if (load - 1.0).abs() <= LOAD_BOUNDARY_TOLERANCE {
                ConstraintStatus::BoundaryReview
            } else if load > 1.0 {
                ConstraintStatus::Exceeded
            } else {
                ConstraintStatus::WithinCriteria
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorPerformance {
    pub given_area_m2: f64,
    pub available_working_volume_m3: f64,
    pub required_working_volume_m3: f64,
    pub souders_brown_velocity_m_s: f64,
    pub allowed_gas_velocity_m_s: f64,
    pub actual_gas_velocity_m_s: f64,
    /// Conditional limits with the captured properties and criteria held fixed.
    pub gas_flow_limit_m3_s: f64,
    pub liquid_flow_limit_m3_s: f64,
    pub available_working_time_s: f64,
    pub gas: ConstraintAssessment,
    pub liquid: ConstraintAssessment,
    pub tangent_to_tangent_height_m: Option<f64>,
    pub buffer_volume_m3: Option<f64>,
}
impl SeparatorPerformance {
    /// Conclusion for these two checks only, not overall equipment certification.
    pub fn conclusion(&self) -> ConstraintStatus {
        use ConstraintStatus::*;
        match (self.gas.status, self.liquid.status) {
            (Exceeded, _) | (_, Exceeded) => Exceeded,
            (BoundaryReview, _) | (_, BoundaryReview) => BoundaryReview,
            (WithinCriteria, WithinCriteria) => WithinCriteria,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct RatingInputCapture {
    pub task: SeparatorRatingTask,
    pub geometry_values_m: BTreeMap<SeparatorGeometryInput, f64>,
    pub values_si: BTreeMap<SeparatorRatingInput, f64>,
    pub axial_lengths_m: BTreeMap<SeparatorAxialSection, f64>,
    pub basis_entries: BTreeMap<BasisEntryId, BasisEntry>,
    pub sources: BTreeMap<SourceId, SourceRecord>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorRatingResult {
    capture: RatingInputCapture,
    performance: SeparatorPerformance,
    assumed_sources: Vec<SourceId>,
    pending_review_sources: Vec<SourceId>,
    missing_axial_sections: Vec<SeparatorAxialSection>,
    height_issues: Vec<RatingIssue>,
}
impl SeparatorRatingResult {
    pub fn capture(&self) -> &RatingInputCapture {
        &self.capture
    }
    pub fn performance(&self) -> &SeparatorPerformance {
        &self.performance
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
    pub fn height_issues(&self) -> &[RatingIssue] {
        &self.height_issues
    }
    pub fn unchecked_requirements(&self) -> &'static [UncheckedRequirement] {
        crate::separator_evidence::UNCHECKED_REQUIREMENTS
    }
}
