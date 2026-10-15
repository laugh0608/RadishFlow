//! Independent equipment input records. No calculation, transaction or storage policy.
use std::collections::{BTreeMap, BTreeSet};

use crate::engineering::InputBinding;
use rf_types::{EngineeringCaseId, EquipmentTaskId, SourceId, units::QuantityKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, PartialEq, Default)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentIdentityErrorKind {
    EmptyTaskId,
    DuplicateTaskId,
    EmptyCaseId,
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
    let mut ids = BTreeSet::new();
    let mut cases = BTreeSet::new();
    let mut tags = BTreeSet::new();
    for (task_index, task) in tasks.iter().enumerate() {
        use EquipmentIdentityErrorKind::*;
        let kind = if task.id.as_str().trim().is_empty() {
            Some(EmptyTaskId)
        } else if !ids.insert(&task.id) {
            Some(DuplicateTaskId)
        } else if task.case.id.as_str().trim().is_empty() {
            Some(EmptyCaseId)
        } else if !cases.insert(&task.case.id) {
            Some(DuplicateCaseId)
        } else if !task.tag.trim().is_empty() && !tags.insert(task.tag.trim().to_ascii_lowercase())
        {
            Some(DuplicateTag)
        } else {
            None
        };
        if let Some(kind) = kind {
            return Err(EquipmentIdentityError { task_index, kind });
        }
    }
    Ok(())
}
