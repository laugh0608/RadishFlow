//! Given geometry and operating inputs for separator rating, independent of
//! execution, result eligibility, editing transactions and persistence.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeparatorRatingInput {
    OperatingTemperature,
    OperatingPressure,
    GasVolumeFlow,
    LiquidVolumeFlow,
    GasDensity,
    LiquidDensity,
    EffectiveK,
    GasVelocityUtilization,
    RequiredWorkingTime,
}
impl SeparatorRatingInput {
    pub const ALL: [Self; 9] = [
        Self::OperatingTemperature,
        Self::OperatingPressure,
        Self::GasVolumeFlow,
        Self::LiquidVolumeFlow,
        Self::GasDensity,
        Self::LiquidDensity,
        Self::EffectiveK,
        Self::GasVelocityUtilization,
        Self::RequiredWorkingTime,
    ];
    pub const fn quantity(self) -> QuantityKind {
        match self {
            Self::OperatingTemperature => QuantityKind::AbsoluteTemperature,
            Self::OperatingPressure => QuantityKind::AbsolutePressure,
            Self::GasVolumeFlow | Self::LiquidVolumeFlow => QuantityKind::ActualVolumeFlow,
            Self::GasDensity | Self::LiquidDensity => QuantityKind::MassDensity,
            Self::EffectiveK => QuantityKind::Velocity,
            Self::GasVelocityUtilization => QuantityKind::DimensionlessRatio,
            Self::RequiredWorkingTime => QuantityKind::Duration,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeparatorGeometryInput {
    InternalDiameter,
    /// Given LLL -> NLL layer, not a calculated minimum or a maximum allowance.
    WorkingLiquidHeight,
}
impl SeparatorGeometryInput {
    pub const ALL: [Self; 2] = [Self::InternalDiameter, Self::WorkingLiquidHeight];
    pub const fn quantity(self) -> QuantityKind {
        QuantityKind::Length
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeparatorGeometryApplicability {
    pub orientation: Option<SeparatorOrientation>,
    pub gas_direction: Option<SeparatorGasDirection>,
    pub geometry: Option<SeparatorSectionGeometry>,
    pub internals_description: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeparatorRatingApplicability {
    pub service: Option<SeparatorService>,
    pub k_basis: Option<SeparatorKBasis>,
    pub flow_basis: Option<SeparatorFlowBasis>,
}

/// One geometry owned by a rating task. Missing bindings preserve incomplete
/// formal records; the method determines whether a calculation is possible.
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorGivenGeometry {
    pub revision: u64,
    pub applicability: SeparatorGeometryApplicability,
    pub applicability_source_ids: Vec<SourceId>,
    pub inputs: BTreeMap<SeparatorGeometryInput, InputBinding>,
    pub axial_sections: BTreeMap<SeparatorAxialSection, InputBinding>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorRatingCase {
    pub id: EngineeringCaseId,
    pub revision: u64,
    pub name: String,
    pub purpose: String,
    pub fluid_system: String,
    pub applicability: SeparatorRatingApplicability,
    pub applicability_source_ids: Vec<SourceId>,
    pub inputs: BTreeMap<SeparatorRatingInput, InputBinding>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorRatingTask {
    pub id: EquipmentTaskId,
    pub revision: u64,
    pub tag: String,
    pub name: String,
    pub method: EquipmentMethod,
    pub geometry: SeparatorGivenGeometry,
    /// Ordered, nonempty membership validated with the task identities. Method
    /// version 1 accepts exactly one case; version 2 supports finite membership.
    pub cases: Vec<SeparatorRatingCase>,
}

/// Owned input for one explicitly selected case. Other members are deliberately
/// absent; this record carries neither execution policy nor result eligibility.
#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorRatingCaseInput {
    pub task_id: EquipmentTaskId,
    pub task_revision: u64,
    pub tag: String,
    pub name: String,
    pub method: EquipmentMethod,
    pub geometry: SeparatorGivenGeometry,
    pub case: SeparatorRatingCase,
}
impl SeparatorRatingTask {
    pub fn case_input(&self, id: &EngineeringCaseId) -> Option<SeparatorRatingCaseInput> {
        let case = self.cases.iter().find(|case| case.id == *id)?;
        Some(SeparatorRatingCaseInput {
            task_id: self.id.clone(),
            task_revision: self.revision,
            tag: self.tag.clone(),
            name: self.name.clone(),
            method: self.method.clone(),
            geometry: self.geometry.clone(),
            case: case.clone(),
        })
    }
}
