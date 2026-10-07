//! Shared read-only quantity presentation. This never rounds stored SI or editable text.
use rf_types::units::{ConversionError, MeasurementUnit, QuantityKind, from_canonical};

/// Six significant digits with a locale-independent decimal point. Formatting the
/// scientific mantissa first also handles subnormal values and rounding across a
/// notation boundary without overflowing a power-of-ten scale factor.
pub fn format_display_number(value: f64) -> Result<String, ConversionError> {
    if !value.is_finite() {
        return Err(ConversionError::NonFiniteValue);
    }
    if value == 0.0 {
        return Ok("0".into());
    }
    let scientific = format!("{value:.5e}");
    let (mantissa, exponent) = scientific.split_once('e').expect("scientific format");
    let exponent: i32 = exponent.parse().expect("formatted decimal exponent");
    if !(-4..6).contains(&exponent) {
        return Ok(format!("{}e{exponent}", trim_decimal_zeros(mantissa)));
    }

    let (sign, magnitude) = mantissa
        .strip_prefix('-')
        .map_or(("", mantissa), |magnitude| ("-", magnitude));
    let digits = magnitude.replace('.', "");
    let point = exponent + 1;
    let decimal = if point <= 0 {
        format!("0.{}{digits}", "0".repeat((-point) as usize))
    } else {
        let point = point as usize;
        if point == digits.len() {
            digits
        } else {
            format!("{}.{}", &digits[..point], &digits[point..])
        }
    };
    Ok(format!("{sign}{}", trim_decimal_zeros(&decimal)))
}

fn trim_decimal_zeros(value: &str) -> &str {
    if value.contains('.') {
        value.trim_end_matches('0').trim_end_matches('.')
    } else {
        value
    }
}

/// The catalog owns both conversion and the unit label, including molar basis.
pub fn format_quantity(
    canonical_value: f64,
    quantity: QuantityKind,
    unit: MeasurementUnit,
) -> Result<String, ConversionError> {
    let value = from_canonical(canonical_value, quantity, unit)?;
    Ok(format!(
        "{} {}",
        format_display_number(value)?,
        unit.definition().symbol
    ))
}

/// Differences must be passed with their own quantity semantics (in particular,
/// TemperatureDifference), rather than subtracting rounded display strings.
pub fn format_quantity_delta(
    canonical_difference: f64,
    quantity: QuantityKind,
    unit: MeasurementUnit,
) -> Result<String, ConversionError> {
    let value = from_canonical(canonical_difference, quantity, unit)?;
    let sign = if value > 0.0 { "+" } else { "" };
    Ok(format!(
        "{sign}{} {}",
        format_display_number(value)?,
        unit.definition().symbol
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rf_types::units::{MeasurementUnit as U, QuantityKind as Q};

    #[test]
    fn significant_digits_and_notation_boundaries_preserve_small_nonzero_values() {
        for (value, expected) in [
            (0.0, "0"),
            (-0.0, "0"),
            (1.0, "1"),
            (100_000.0, "100000"),
            (26.850000000000023, "26.85"),
            (1.234567, "1.23457"),
            (-1.234567, "-1.23457"),
            (0.0001, "0.0001"),
            (0.00001234567, "1.23457e-5"),
            (0.00009999996, "0.0001"),
            (1_234_567.0, "1.23457e6"),
            (999_999.6, "1e6"),
            (999_999.4, "999999"),
            (f64::MIN_POSITIVE, "2.22507e-308"),
            (f64::from_bits(1), "4.94066e-324"),
            (f64::MAX, "1.79769e308"),
        ] {
            assert_eq!(format_display_number(value).unwrap(), expected, "{value}");
        }
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                format_display_number(value),
                Err(ConversionError::NonFiniteValue)
            );
        }
    }

    #[test]
    fn quantities_keep_offsets_basis_and_errors_explicit() {
        for (si, quantity, unit, expected) in [
            (300.0, Q::AbsoluteTemperature, U::Celsius, "26.85 °C"),
            (100_000.0, Q::AbsolutePressure, U::Bar, "1 bar"),
            (1.0, Q::MolarFlow, U::KilomolePerHour, "3.6 kmol/h"),
            (0.35, Q::MoleFraction, U::MolePercent, "35 mol%"),
            (0.35, Q::MolarPhaseFraction, U::MolePerMole, "0.35 mol/mol"),
            (
                -2500.0,
                Q::MolarEnthalpy,
                U::KilojoulePerMole,
                "-2.5 kJ/mol",
            ),
        ] {
            assert_eq!(format_quantity(si, quantity, unit).unwrap(), expected);
        }
        assert_eq!(
            format_quantity_delta(10.0, Q::TemperatureDifference, U::CelsiusDifference).unwrap(),
            "+10 °C"
        );
        assert_eq!(
            format_quantity_delta(-0.0, Q::TemperatureDifference, U::KelvinDifference).unwrap(),
            "0 K"
        );
        assert!(matches!(
            format_quantity(10.0, Q::TemperatureDifference, U::Celsius),
            Err(ConversionError::IncompatibleUnit { .. })
        ));
        assert!(matches!(
            format_quantity(1.0, Q::AbsolutePressure, U::BarGauge),
            Err(ConversionError::ContextRequired { .. })
        ));
        assert_eq!(
            format_quantity(f64::MAX, Q::MolarFlow, U::MolePerHour),
            Err(ConversionError::Overflow)
        );
    }
}
