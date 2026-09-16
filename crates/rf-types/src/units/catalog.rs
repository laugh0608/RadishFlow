use super::ContextRequirement;

/// Integer exponents of SI base dimensions. Semantic compatibility also requires QuantityKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dimension {
    pub length: i8,
    pub mass: i8,
    pub time: i8,
    pub current: i8,
    pub temperature: i8,
    pub amount: i8,
    pub luminous_intensity: i8,
}
const DIMENSIONLESS: Dimension = Dimension {
    length: 0,
    mass: 0,
    time: 0,
    current: 0,
    temperature: 0,
    amount: 0,
    luminous_intensity: 0,
};
const TEMPERATURE: Dimension = Dimension {
    temperature: 1,
    ..DIMENSIONLESS
};
const PRESSURE: Dimension = Dimension {
    length: -1,
    mass: 1,
    time: -2,
    ..DIMENSIONLESS
};
const MOLAR_FLOW: Dimension = Dimension {
    amount: 1,
    time: -1,
    ..DIMENSIONLESS
};
const MOLAR_ENERGY: Dimension = Dimension {
    length: 2,
    mass: 1,
    time: -2,
    amount: -1,
    ..DIMENSIONLESS
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuantityKind {
    AbsoluteTemperature,
    TemperatureDifference,
    AbsolutePressure,
    MolarFlow,
    MoleFraction,
    MolarPhaseFraction,
    MolarEnthalpy,
}
pub const ALL_QUANTITIES: &[QuantityKind] = &[
    QuantityKind::AbsoluteTemperature,
    QuantityKind::TemperatureDifference,
    QuantityKind::AbsolutePressure,
    QuantityKind::MolarFlow,
    QuantityKind::MoleFraction,
    QuantityKind::MolarPhaseFraction,
    QuantityKind::MolarEnthalpy,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeasurementUnit {
    Kelvin,
    Celsius,
    KelvinDifference,
    CelsiusDifference,
    Pascal,
    Kilopascal,
    Megapascal,
    Bar,
    MolePerSecond,
    KilomolePerSecond,
    MolePerHour,
    KilomolePerHour,
    MolePerMole,
    MolePercent,
    JoulePerMole,
    KilojoulePerMole,
    BarGauge,
    KilogramPerSecond,
    StandardCubicMetrePerHour,
}
pub const ALL_UNITS: &[MeasurementUnit] = &[
    MeasurementUnit::Kelvin,
    MeasurementUnit::Celsius,
    MeasurementUnit::KelvinDifference,
    MeasurementUnit::CelsiusDifference,
    MeasurementUnit::Pascal,
    MeasurementUnit::Kilopascal,
    MeasurementUnit::Megapascal,
    MeasurementUnit::Bar,
    MeasurementUnit::MolePerSecond,
    MeasurementUnit::KilomolePerSecond,
    MeasurementUnit::MolePerHour,
    MeasurementUnit::KilomolePerHour,
    MeasurementUnit::MolePerMole,
    MeasurementUnit::MolePercent,
    MeasurementUnit::JoulePerMole,
    MeasurementUnit::KilojoulePerMole,
    MeasurementUnit::BarGauge,
    MeasurementUnit::KilogramPerSecond,
    MeasurementUnit::StandardCubicMetrePerHour,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConversionRule {
    /// canonical_value = scale * unit_value + offset
    Affine { scale: f64, offset: f64 },
    /// Recognized only to diagnose unsupported context-dependent conversions.
    Contextual {
        target: QuantityKind,
        requirement: ContextRequirement,
    },
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitDefinition {
    pub id: &'static str,
    pub symbol: &'static str,
    pub aliases: &'static [&'static str],
    pub dimension: Dimension,
    pub conversion: ConversionRule,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuantityDefinition {
    pub id: &'static str,
    pub dimension: Dimension,
    pub canonical_unit: MeasurementUnit,
    /// Supported, context-free units only. Their order is stable, canonical unit first.
    pub units: &'static [MeasurementUnit],
}
const fn quantity(
    id: &'static str,
    dimension: Dimension,
    units: &'static [MeasurementUnit],
) -> QuantityDefinition {
    QuantityDefinition {
        id,
        dimension,
        canonical_unit: units[0],
        units,
    }
}
impl QuantityKind {
    pub const fn definition(self) -> QuantityDefinition {
        use MeasurementUnit::*;
        match self {
            Self::AbsoluteTemperature => {
                quantity("absolute_temperature", TEMPERATURE, &[Kelvin, Celsius])
            }
            Self::TemperatureDifference => quantity(
                "temperature_difference",
                TEMPERATURE,
                &[KelvinDifference, CelsiusDifference],
            ),
            Self::AbsolutePressure => quantity(
                "absolute_pressure",
                PRESSURE,
                &[Pascal, Kilopascal, Megapascal, Bar],
            ),
            Self::MolarFlow => quantity(
                "molar_flow",
                MOLAR_FLOW,
                &[
                    MolePerSecond,
                    KilomolePerSecond,
                    MolePerHour,
                    KilomolePerHour,
                ],
            ),
            Self::MoleFraction => {
                quantity("mole_fraction", DIMENSIONLESS, &[MolePerMole, MolePercent])
            }
            Self::MolarPhaseFraction => quantity(
                "molar_phase_fraction",
                DIMENSIONLESS,
                &[MolePerMole, MolePercent],
            ),
            Self::MolarEnthalpy => quantity(
                "molar_enthalpy",
                MOLAR_ENERGY,
                &[JoulePerMole, KilojoulePerMole],
            ),
        }
    }
}
const fn affine(
    id: &'static str,
    symbol: &'static str,
    aliases: &'static [&'static str],
    dimension: Dimension,
    scale: f64,
    offset: f64,
) -> UnitDefinition {
    // Catalog literals are checked during constant evaluation. No user-defined coefficients.
    assert!(scale.is_finite() && scale > 0.0 && offset.is_finite());
    UnitDefinition {
        id,
        symbol,
        aliases,
        dimension,
        conversion: ConversionRule::Affine { scale, offset },
    }
}
const fn contextual(
    id: &'static str,
    symbol: &'static str,
    aliases: &'static [&'static str],
    dimension: Dimension,
    target: QuantityKind,
    requirement: ContextRequirement,
) -> UnitDefinition {
    UnitDefinition {
        id,
        symbol,
        aliases,
        dimension,
        conversion: ConversionRule::Contextual {
            target,
            requirement,
        },
    }
}
impl MeasurementUnit {
    pub const fn definition(self) -> UnitDefinition {
        match self {
            Self::Kelvin => affine("kelvin", "K", &[], TEMPERATURE, 1.0, 0.0),
            Self::Celsius => affine("celsius", "°C", &["degC"], TEMPERATURE, 1.0, 273.15),
            Self::KelvinDifference => affine(
                "kelvin_difference",
                "K",
                &["delta_K"],
                TEMPERATURE,
                1.0,
                0.0,
            ),
            Self::CelsiusDifference => affine(
                "celsius_difference",
                "°C",
                &["delta_degC"],
                TEMPERATURE,
                1.0,
                0.0,
            ),
            Self::Pascal => affine("pascal", "Pa", &[], PRESSURE, 1.0, 0.0),
            Self::Kilopascal => affine("kilopascal", "kPa", &[], PRESSURE, 1000.0, 0.0),
            Self::Megapascal => affine("megapascal", "MPa", &[], PRESSURE, 1_000_000.0, 0.0),
            Self::Bar => affine("bar", "bar", &[], PRESSURE, 100_000.0, 0.0),
            Self::MolePerSecond => affine("mole_per_second", "mol/s", &[], MOLAR_FLOW, 1.0, 0.0),
            Self::KilomolePerSecond => affine(
                "kilomole_per_second",
                "kmol/s",
                &[],
                MOLAR_FLOW,
                1000.0,
                0.0,
            ),
            Self::MolePerHour => {
                affine("mole_per_hour", "mol/h", &[], MOLAR_FLOW, 1.0 / 3600.0, 0.0)
            }
            Self::KilomolePerHour => affine(
                "kilomole_per_hour",
                "kmol/h",
                &[],
                MOLAR_FLOW,
                1000.0 / 3600.0,
                0.0,
            ),
            Self::MolePerMole => affine("mole_per_mole", "mol/mol", &[], DIMENSIONLESS, 1.0, 0.0),
            Self::MolePercent => {
                affine("mole_percent", "mol%", &["mol %"], DIMENSIONLESS, 0.01, 0.0)
            }
            Self::JoulePerMole => affine("joule_per_mole", "J/mol", &[], MOLAR_ENERGY, 1.0, 0.0),
            Self::KilojoulePerMole => affine(
                "kilojoule_per_mole",
                "kJ/mol",
                &[],
                MOLAR_ENERGY,
                1000.0,
                0.0,
            ),
            Self::BarGauge => contextual(
                "bar_gauge",
                "bar(g)",
                &["barg"],
                PRESSURE,
                QuantityKind::AbsolutePressure,
                ContextRequirement::ReferencePressure,
            ),
            Self::KilogramPerSecond => contextual(
                "kilogram_per_second",
                "kg/s",
                &[],
                Dimension {
                    mass: 1,
                    time: -1,
                    ..DIMENSIONLESS
                },
                QuantityKind::MolarFlow,
                ContextRequirement::MixtureMolarMass,
            ),
            Self::StandardCubicMetrePerHour => contextual(
                "standard_cubic_metre_per_hour",
                "Nm³/h",
                &["Nm3/h"],
                Dimension {
                    length: 3,
                    time: -1,
                    ..DIMENSIONLESS
                },
                QuantityKind::MolarFlow,
                ContextRequirement::StandardVolumeBasisAndStateRelation,
            ),
        }
    }
}

// Force constant evaluation of all coefficient definitions, including rarely used units.
const _: () = {
    let mut index = 0;
    while index < ALL_UNITS.len() {
        let _ = ALL_UNITS[index].definition();
        index += 1;
    }
};
