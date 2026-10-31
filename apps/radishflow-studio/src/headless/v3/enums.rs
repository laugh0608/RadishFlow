//! Explicit wire vocabulary; internal Debug strings never become protocol values.
use rf_model::engineering;
use rf_model::equipment;
use serde::{Deserialize, Serialize};

macro_rules! wire_enum {
    ($name:ident, $domain:path, [$($variant:ident),+ $(,)?]) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }
        impl From<$domain> for $name {
            fn from(value: $domain) -> Self { match value { $(<$domain>::$variant => Self::$variant),+ } }
        }
        impl From<$name> for $domain {
            fn from(value: $name) -> Self { match value { $($name::$variant => Self::$variant),+ } }
        }
    };
}
wire_enum!(
    SizingField,
    equipment::SeparatorInput,
    [
        OperatingTemperature,
        OperatingPressure,
        GasVolumeFlow,
        LiquidVolumeFlow,
        GasDensity,
        LiquidDensity,
        EffectiveK,
        GasVelocityUtilization,
        WorkingVolumeTime,
        MaximumWorkingLiquidHeight
    ]
);
wire_enum!(
    RatingField,
    equipment::SeparatorRatingInput,
    [
        OperatingTemperature,
        OperatingPressure,
        GasVolumeFlow,
        LiquidVolumeFlow,
        GasDensity,
        LiquidDensity,
        EffectiveK,
        GasVelocityUtilization,
        RequiredWorkingTime
    ]
);
wire_enum!(
    GeometryField,
    equipment::SeparatorGeometryInput,
    [InternalDiameter, WorkingLiquidHeight]
);
wire_enum!(
    AxialSection,
    equipment::SeparatorAxialSection,
    [
        LowerTangentToLowLiquidLevel,
        NormalToHighLiquidLevel,
        HighLiquidLevelToInlet,
        InletToInternals,
        InternalsThickness,
        InternalsToUpperTangent
    ]
);
wire_enum!(
    Orientation,
    equipment::SeparatorOrientation,
    [Vertical, Horizontal]
);
wire_enum!(
    GasDirection,
    equipment::SeparatorGasDirection,
    [Upward, Other]
);
wire_enum!(
    Service,
    equipment::SeparatorService,
    [
        CleanNonFoamingTwoPhase,
        SinglePhase,
        Foaming,
        Solids,
        Slugging,
        ThreePhase
    ]
);
wire_enum!(
    SectionGeometry,
    equipment::SeparatorSectionGeometry,
    [
        FullEqualDiameterCylinder,
        ReducedEffectiveArea,
        NonCylindrical
    ]
);
wire_enum!(
    KBasis,
    equipment::SeparatorKBasis,
    [
        EffectiveWholeVesselForDeclaredInternals,
        MistEliminatorOnly,
        UnmatchedInternals
    ]
);
wire_enum!(
    FlowBasis,
    equipment::SeparatorFlowBasis,
    [ActualAtOperatingConditions, StandardConditions]
);
wire_enum!(
    SourceKind,
    engineering::SourceKind,
    [Measured, Vendor, Literature, Estimated, Assumed, Unknown]
);
wire_enum!(
    SourceReview,
    engineering::SourceReview,
    [Unreviewed, Adopted, Reviewed]
);
wire_enum!(
    SourcePhase,
    engineering::SourcePhase,
    [Gas, Liquid, Both, NotApplicable]
);
// This wire vocabulary remains the six persistable application models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitKind {
    Feed,
    Heater,
    Cooler,
    Valve,
    Mixer,
    FlashDrum,
}
impl From<UnitKind> for rf_ui::BuiltinUnitKind {
    fn from(value: UnitKind) -> Self {
        match value {
            UnitKind::Feed => Self::Feed,
            UnitKind::Heater => Self::Heater,
            UnitKind::Cooler => Self::Cooler,
            UnitKind::Valve => Self::Valve,
            UnitKind::Mixer => Self::Mixer,
            UnitKind::FlashDrum => Self::FlashDrum,
        }
    }
}
wire_enum!(
    Quantity,
    rf_types::units::QuantityKind,
    [
        AbsoluteTemperature,
        TemperatureDifference,
        AbsolutePressure,
        MolarFlow,
        MoleFraction,
        MolarPhaseFraction,
        MolarEnthalpy,
        ActualVolumeFlow,
        MassDensity,
        Velocity,
        Duration,
        Length,
        Area,
        Volume,
        DimensionlessRatio
    ]
);
