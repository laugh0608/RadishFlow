//! Independent equipment input records. No calculation, transaction or storage policy.
mod adoption;
mod rating;
pub use adoption::*;
mod task;
pub use rating::*;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use crate::engineering::InputBinding;
use rf_types::{EngineeringCaseId, EquipmentTaskId, SourceId, units::QuantityKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeparatorInput {
    OperatingTemperature,
    OperatingPressure,
    GasVolumeFlow,
    LiquidVolumeFlow,
    GasDensity,
    LiquidDensity,
    EffectiveK,
    GasVelocityUtilization,
    WorkingVolumeTime,
    MaximumWorkingLiquidHeight,
}

impl SeparatorInput {
    pub const ALL: [Self; 10] = [
        Self::OperatingTemperature,
        Self::OperatingPressure,
        Self::GasVolumeFlow,
        Self::LiquidVolumeFlow,
        Self::GasDensity,
        Self::LiquidDensity,
        Self::EffectiveK,
        Self::GasVelocityUtilization,
        Self::WorkingVolumeTime,
        Self::MaximumWorkingLiquidHeight,
    ];

    pub const fn quantity(self) -> QuantityKind {
        match self {
            Self::OperatingTemperature => QuantityKind::AbsoluteTemperature,
            Self::OperatingPressure => QuantityKind::AbsolutePressure,
            Self::GasVolumeFlow | Self::LiquidVolumeFlow => QuantityKind::ActualVolumeFlow,
            Self::GasDensity | Self::LiquidDensity => QuantityKind::MassDensity,
            Self::EffectiveK => QuantityKind::Velocity,
            Self::GasVelocityUtilization => QuantityKind::DimensionlessRatio,
            Self::WorkingVolumeTime => QuantityKind::Duration,
            Self::MaximumWorkingLiquidHeight => QuantityKind::Length,
        }
    }
}

/// Nonoverlapping intervals in addition to the calculated LLL -> NLL layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeparatorAxialSection {
    LowerTangentToLowLiquidLevel,
    NormalToHighLiquidLevel,
    HighLiquidLevelToInlet,
    InletToInternals,
    InternalsThickness,
    InternalsToUpperTangent,
}

impl SeparatorAxialSection {
    pub const ALL: [Self; 6] = [
        Self::LowerTangentToLowLiquidLevel,
        Self::NormalToHighLiquidLevel,
        Self::HighLiquidLevelToInlet,
        Self::InletToInternals,
        Self::InternalsThickness,
        Self::InternalsToUpperTangent,
    ];

    pub const fn requires_positive_clearance(self) -> bool {
        matches!(
            self,
            Self::HighLiquidLevelToInlet | Self::InletToInternals | Self::InternalsToUpperTangent
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorOrientation {
    Vertical,
    Horizontal,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorGasDirection {
    Upward,
    Other,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorService {
    CleanNonFoamingTwoPhase,
    SinglePhase,
    Foaming,
    Solids,
    Slugging,
    ThreePhase,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorSectionGeometry {
    FullEqualDiameterCylinder,
    ReducedEffectiveArea,
    NonCylindrical,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorKBasis {
    EffectiveWholeVesselForDeclaredInternals,
    MistEliminatorOnly,
    UnmatchedInternals,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorFlowBasis {
    ActualAtOperatingConditions,
    StandardConditions,
}

/// Explicit user declarations, not properties inferred from a fluid name or Flash.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SeparatorApplicability {
    pub orientation: Option<SeparatorOrientation>,
    pub gas_direction: Option<SeparatorGasDirection>,
    pub service: Option<SeparatorService>,
    pub geometry: Option<SeparatorSectionGeometry>,
    pub k_basis: Option<SeparatorKBasis>,
    pub flow_basis: Option<SeparatorFlowBasis>,
    pub internals_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentMethod {
    pub id: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorCase {
    pub id: EngineeringCaseId,
    pub name: String,
    pub purpose: String,
    pub fluid_system: String,
    pub applicability_source_ids: Vec<SourceId>,
    /// Missing entries represent incomplete formal inputs; no numeric defaults.
    pub inputs: BTreeMap<SeparatorInput, InputBinding>,
    pub applicability: SeparatorApplicability,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorSizingTask {
    pub id: EquipmentTaskId,
    /// Persistent input record revision; not an execution or session token.
    pub revision: u64,
    pub tag: String,
    pub name: String,
    pub method: EquipmentMethod,
    pub case: SeparatorCase,
    pub axial_sections: BTreeMap<SeparatorAxialSection, InputBinding>,
}

/// Formal equipment inputs. Application sessions and storage opt into new
/// variants separately; introducing a model does not enable a product capability.
#[derive(Debug, Clone, PartialEq)]
pub enum EquipmentTask {
    SeparatorSizing(SeparatorSizingTask),
    SeparatorRating(SeparatorRatingTask),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentIdentityErrorKind {
    EmptyTaskId,
    DuplicateTaskId,
    EmptyCaseId,
    EmptyCaseCollection,
    DuplicateCaseId,
    DuplicateTag,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentIdentityError {
    pub task_index: usize,
    pub kind: EquipmentIdentityErrorKind,
}

/// Names are labels; references use IDs. Blank tags are allowed; nonblank tags
/// compare after trimming, with ASCII case folding, within this task collection.
pub fn validate_equipment_identities(
    tasks: &[SeparatorSizingTask],
) -> Result<(), EquipmentIdentityError> {
    let mut identities = EquipmentIdentities::default();
    for (task_index, task) in tasks.iter().enumerate() {
        identities
            .check(&task.id, &task.tag, [&task.case.id])
            .map_err(|kind| EquipmentIdentityError { task_index, kind })?;
    }
    Ok(())
}

/// Validate project-wide identities across both supported record types, without
/// imposing a method's case-count or numerical applicability policy on the model.
pub fn validate_equipment_task_identities(
    tasks: &[EquipmentTask],
) -> Result<(), EquipmentIdentityError> {
    let mut identities = EquipmentIdentities::default();
    for (task_index, task) in tasks.iter().enumerate() {
        let result = match task {
            EquipmentTask::SeparatorSizing(task) => {
                identities.check(&task.id, &task.tag, [&task.case.id])
            }
            EquipmentTask::SeparatorRating(task) => {
                identities.check(&task.id, &task.tag, task.cases.iter().map(|c| &c.id))
            }
        };
        result.map_err(|kind| EquipmentIdentityError { task_index, kind })?;
    }
    Ok(())
}

pub fn validate_separator_rating_identities(
    tasks: &[SeparatorRatingTask],
) -> Result<(), EquipmentIdentityError> {
    let mut identities = EquipmentIdentities::default();
    for (task_index, task) in tasks.iter().enumerate() {
        identities
            .check(&task.id, &task.tag, task.cases.iter().map(|c| &c.id))
            .map_err(|kind| EquipmentIdentityError { task_index, kind })?;
    }
    Ok(())
}

#[derive(Default)]
struct EquipmentIdentities<'a> {
    tasks: BTreeSet<&'a EquipmentTaskId>,
    cases: BTreeSet<&'a EngineeringCaseId>,
    tags: BTreeSet<String>,
}
impl<'a> EquipmentIdentities<'a> {
    fn check(
        &mut self,
        id: &'a EquipmentTaskId,
        tag: &str,
        cases: impl IntoIterator<Item = &'a EngineeringCaseId>,
    ) -> Result<(), EquipmentIdentityErrorKind> {
        use EquipmentIdentityErrorKind::*;
        if id.as_str().trim().is_empty() {
            return Err(EmptyTaskId);
        }
        if !self.tasks.insert(id) {
            return Err(DuplicateTaskId);
        }
        let mut cases = cases.into_iter().peekable();
        if cases.peek().is_none() {
            return Err(EmptyCaseCollection);
        }
        for case in cases {
            if case.as_str().trim().is_empty() {
                return Err(EmptyCaseId);
            }
            if !self.cases.insert(case) {
                return Err(DuplicateCaseId);
            }
        }
        if !tag.trim().is_empty() && !self.tags.insert(tag.trim().to_ascii_lowercase()) {
            return Err(DuplicateTag);
        }
        Ok(())
    }
}
