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
    declarations(
        table,
        id,
        &SeparatorGeometryApplicability {
            orientation: a.orientation,
            gas_direction: a.gas_direction,
            geometry: a.geometry,
            internals_description: a.internals_description.clone(),
        },
        id,
        &SeparatorRatingApplicability {
            service: a.service,
            k_basis: a.k_basis,
            flow_basis: a.flow_basis,
        },
    );
}

pub(super) fn declarations(
    table: &mut CalculationTable,
    geometry_id: &str,
    g: &SeparatorGeometryApplicability,
    case_id: &str,
    a: &SeparatorRatingApplicability,
) {
    geometry_declarations(table, geometry_id, g);
    case_declarations(table, case_id, a);
}
// Stable output keys, independent of Debug and UI translations.
macro_rules! declaration {
    ($table:expr, $id:expr, $field:literal, $value:expr, {$($variant:path => $key:literal),+ $(,)?}) => {
        $table.text("applicability", $id, $field, match $value {
            $(Some($variant) => $key,)+ None => "unknown",
        });
    };
}
pub(super) fn geometry_declarations(
    table: &mut CalculationTable,
    id: &str,
    g: &SeparatorGeometryApplicability,
) {
    declaration!(table, id, "orientation", g.orientation, { SeparatorOrientation::Vertical => "vertical", SeparatorOrientation::Horizontal => "horizontal" });
    declaration!(table, id, "gasDirection", g.gas_direction, { SeparatorGasDirection::Upward => "upward", SeparatorGasDirection::Other => "other" });
    declaration!(table, id, "geometry", g.geometry, {
        SeparatorSectionGeometry::FullEqualDiameterCylinder => "fullEqualDiameterCylinder",
        SeparatorSectionGeometry::ReducedEffectiveArea => "reducedEffectiveArea",
        SeparatorSectionGeometry::NonCylindrical => "nonCylindrical"
    });
}
pub(super) fn case_declarations(
    table: &mut CalculationTable,
    id: &str,
    a: &SeparatorRatingApplicability,
) {
    declaration!(table, id, "service", a.service, {
        SeparatorService::CleanNonFoamingTwoPhase => "cleanNonFoamingTwoPhase",
        SeparatorService::SinglePhase => "singlePhase", SeparatorService::Foaming => "foaming",
        SeparatorService::Solids => "solids", SeparatorService::Slugging => "slugging", SeparatorService::ThreePhase => "threePhase"
    });
    declaration!(table, id, "kBasis", a.k_basis, {
        SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals => "effectiveWholeVesselForDeclaredInternals",
        SeparatorKBasis::MistEliminatorOnly => "mistEliminatorOnly", SeparatorKBasis::UnmatchedInternals => "unmatchedInternals"
    });
    declaration!(table, id, "flowBasis", a.flow_basis, {
        SeparatorFlowBasis::ActualAtOperatingConditions => "actualAtOperatingConditions", SeparatorFlowBasis::StandardConditions => "standardConditions"
    });
}
