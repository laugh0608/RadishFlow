use super::CalculationTable;
use rf_model::equipment::*;

pub(super) fn input(value: SeparatorInput) -> &'static str {
    match value {
        SeparatorInput::OperatingTemperature => "operatingTemperature",
        SeparatorInput::OperatingPressure => "operatingPressure",
        SeparatorInput::GasVolumeFlow => "gasActualVolumeFlow",
        SeparatorInput::LiquidVolumeFlow => "liquidActualVolumeFlow",
        SeparatorInput::GasDensity => "gasMassDensity",
        SeparatorInput::LiquidDensity => "liquidMassDensity",
        SeparatorInput::EffectiveK => "effectiveK",
        SeparatorInput::GasVelocityUtilization => "gasUtilization",
        SeparatorInput::WorkingVolumeTime => "workingDuration",
        SeparatorInput::MaximumWorkingLiquidHeight => "maximumWorkingLiquidHeight",
    }
}
pub(super) fn axial(value: SeparatorAxialSection) -> &'static str {
    match value {
        SeparatorAxialSection::LowerTangentToLowLiquidLevel => "bottomTangentToLLL",
        SeparatorAxialSection::NormalToHighLiquidLevel => "nllToHLL",
        SeparatorAxialSection::HighLiquidLevelToInlet => "hllToInletCenter",
        SeparatorAxialSection::InletToInternals => "inletCenterToInternalsBottom",
        SeparatorAxialSection::InternalsThickness => "internalsThickness",
        SeparatorAxialSection::InternalsToUpperTangent => "internalsTopToTopTangent",
    }
}
pub(super) fn applicability(table: &mut CalculationTable, case: &SeparatorCase) {
    let a = &case.applicability;
    let id = case.id.as_str();
    // Explicit matches keep output keys independent of Debug and UI translations.
    macro_rules! declaration {
        ($field:literal, $value:expr, {$($variant:path => $key:literal),+ $(,)?}) => {
            table.text("applicability", id, $field, match $value {
                $(Some($variant) => $key,)+ None => "unknown",
            });
        };
    }
    declaration!("orientation", a.orientation, {
        SeparatorOrientation::Vertical => "vertical", SeparatorOrientation::Horizontal => "horizontal"
    });
    declaration!("gasDirection", a.gas_direction, {
        SeparatorGasDirection::Upward => "upward", SeparatorGasDirection::Other => "other"
    });
    declaration!("service", a.service, {
        SeparatorService::CleanNonFoamingTwoPhase => "cleanNonFoamingTwoPhase",
        SeparatorService::SinglePhase => "singlePhase", SeparatorService::Foaming => "foaming",
        SeparatorService::Solids => "solids", SeparatorService::Slugging => "slugging",
        SeparatorService::ThreePhase => "threePhase"
    });
    declaration!("geometry", a.geometry, {
        SeparatorSectionGeometry::FullEqualDiameterCylinder => "fullEqualDiameterCylinder",
        SeparatorSectionGeometry::ReducedEffectiveArea => "reducedEffectiveArea",
        SeparatorSectionGeometry::NonCylindrical => "nonCylindrical"
    });
    declaration!("kBasis", a.k_basis, {
        SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals => "effectiveWholeVesselForDeclaredInternals",
        SeparatorKBasis::MistEliminatorOnly => "mistEliminatorOnly",
        SeparatorKBasis::UnmatchedInternals => "unmatchedInternals"
    });
    declaration!("flowBasis", a.flow_basis, {
        SeparatorFlowBasis::ActualAtOperatingConditions => "actualAtOperatingConditions",
        SeparatorFlowBasis::StandardConditions => "standardConditions"
    });
}
