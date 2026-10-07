use super::super::solve_snapshot_model_from_ui;
use rf_types::units::{ConversionError, DisplayUnitSet, MeasurementUnit as U, QuantityKind as Q};
use rf_ui::{BubbleDewWindowSnapshot, PhaseStateSnapshot, StreamStateSnapshot};

fn raw_snapshot() -> rf_ui::SolveSnapshot {
    let stream = StreamStateSnapshot {
        stream_id: "base".into(),
        label: "Base".into(),
        temperature_k: 300.0,
        pressure_pa: 100_000.0,
        total_molar_flow_mol_s: 1.0,
        overall_mole_fractions: vec![("a".into(), 0.35), ("b".into(), 0.65)],
        phases: vec![
            PhaseStateSnapshot {
                label: "overall".into(),
                phase_fraction: 1.0,
                composition: vec![("a".into(), 0.35), ("b".into(), 0.65)],
                molar_enthalpy_j_per_mol: Some(-2500.0),
            },
            PhaseStateSnapshot {
                label: "liquid".into(),
                phase_fraction: 0.25,
                composition: vec![("a".into(), 0.2), ("b".into(), 0.8)],
                molar_enthalpy_j_per_mol: None,
            },
        ],
        bubble_dew_window: Some(BubbleDewWindowSnapshot {
            phase_region: rf_types::PhaseEquilibriumRegion::TwoPhase,
            bubble_pressure_pa: 200_000.0,
            dew_pressure_pa: 50_000.0,
            bubble_temperature_k: 305.0,
            dew_temperature_k: 315.0,
        }),
    };
    let mut compared = stream.clone();
    compared.stream_id = "compared".into();
    compared.temperature_k = 310.0;
    compared.pressure_pa = 200_000.0;
    compared.total_molar_flow_mol_s = 2.0;
    compared.overall_mole_fractions[0].1 = 0.45;
    compared.overall_mole_fractions[1].1 = 0.55;
    compared.phases[0].molar_enthalpy_j_per_mol = Some(-1500.0);
    let mut snapshot = rf_ui::SolveSnapshot::new(
        "units-fixture",
        3,
        7,
        rf_ui::RunStatus::Converged,
        rf_ui::DiagnosticSummary::new(3, rf_ui::DiagnosticSeverity::Info, "Converged"),
    );
    snapshot.streams = vec![stream.clone(), compared.clone()];
    snapshot.steps.push(rf_ui::StepSnapshot {
        index: 0,
        unit_id: "heater".into(),
        summary: "Heater".into(),
        execution: rf_ui::UnitExecutionSnapshot {
            unit_id: "heater".into(),
            status: rf_ui::RunStatus::Converged,
            summary: "Converged".into(),
        },
        consumed_streams: vec![stream],
        streams: vec![compared],
    });
    snapshot
}

fn custom_units() -> DisplayUnitSet {
    let mut units = DisplayUnitSet::engineering();
    for (quantity, unit) in [
        (Q::MoleFraction, U::MolePercent),
        (Q::MolarPhaseFraction, U::MolePercent),
        (Q::MolarEnthalpy, U::KilojoulePerMole),
        (Q::TemperatureDifference, U::CelsiusDifference),
    ] {
        units.set_unit(quantity, unit).unwrap();
    }
    units
}

#[test]
fn result_consumers_use_project_units_from_si_and_preserve_the_raw_snapshot() {
    let raw = raw_snapshot();
    let original = raw.clone();
    for (units, expected) in [
        (
            DisplayUnitSet::si(),
            ["300 K", "100000 Pa", "1 mol/s", "-2500 J/mol"],
        ),
        (
            DisplayUnitSet::engineering(),
            ["26.85 °C", "1 bar", "3.6 kmol/h", "-2500 J/mol"],
        ),
        (
            custom_units(),
            ["26.85 °C", "1 bar", "3.6 kmol/h", "-2.5 kJ/mol"],
        ),
    ] {
        let model = solve_snapshot_model_from_ui(&raw, &units);
        assert!(model.presentation_errors.is_empty());
        assert_eq!(model.display_units, units);
        assert_eq!(model.snapshot_id, "units-fixture");
        assert_eq!((model.document_revision, model.sequence), (3, 7));
        let stream = &model.streams[0];
        assert_eq!(
            [
                stream.temperature_text.as_str(),
                stream.pressure_text.as_str(),
                stream.molar_flow_text.as_str(),
                stream.molar_enthalpy_text.as_deref().unwrap()
            ],
            expected
        );
        let inspector = model.result_inspector(Some("base"));
        assert_eq!(inspector.selected_stream.as_ref().unwrap(), stream);
        for (row, text) in stream.summary_rows.iter().zip(expected) {
            assert_eq!(row.value, text);
        }
        let exported = model.light_text_export().unwrap();
        for text in expected {
            assert!(exported.contains(text));
            assert!(
                model.steps[0].consumed_stream_results[0]
                    .summary
                    .contains(text)
            );
        }
        assert_eq!(raw, original);
        assert_eq!(stream.temperature_k, 300.0);
        assert_eq!(stream.pressure_pa, 100_000.0);
    }
    let model = solve_snapshot_model_from_ui(&raw, &custom_units());
    let stream = &model.streams[0];
    assert_eq!(stream.composition_text, "a=35 mol%, b=65 mol%");
    assert_eq!(stream.composition_rows[0].fraction_text, "35 mol%");
    assert_eq!(stream.phase_rows[0].phase_fraction_text, "100 mol%");
    assert_eq!(stream.phase_rows[1].phase_fraction_text, "25 mol%");
    assert_eq!(stream.phase_rows[1].molar_flow_text, "0.9 kmol/h");
    assert_eq!(
        stream.phase_rows[1].composition_text,
        "a=20 mol%, b=80 mol%"
    );
    assert!(stream.phase_rows[1].molar_enthalpy_text.is_none());
    let window = stream.bubble_dew_window.as_ref().unwrap();
    assert_eq!(window.bubble_pressure_text, "2 bar");
    assert_eq!(window.dew_pressure_text, "0.5 bar");
    assert_eq!(window.bubble_temperature_text, "31.85 °C");
    assert_eq!(window.dew_temperature_text, "41.85 °C");
}

#[test]
fn comparisons_use_difference_semantics_and_do_not_invent_missing_values() {
    let mut raw = raw_snapshot();
    raw.streams[1]
        .overall_mole_fractions
        .push(("only-compared".into(), 0.0));
    let model = solve_snapshot_model_from_ui(&raw, &custom_units());
    let comparison = model
        .result_inspector_with_comparison(Some("base"), Some("compared"))
        .comparison
        .unwrap();
    assert_eq!(comparison.summary_rows[0].base_value, "26.85 °C");
    assert_eq!(comparison.summary_rows[0].compared_value, "36.85 °C");
    assert_eq!(comparison.summary_rows[0].delta_text, "+10 °C");
    assert_eq!(comparison.summary_rows[1].delta_text, "+1 bar");
    assert_eq!(comparison.summary_rows[2].delta_text, "+3.6 kmol/h");
    assert_eq!(comparison.summary_rows[3].delta_text, "+1 kJ/mol");
    assert_eq!(comparison.composition_rows[0].delta_text, "+10 mol%");
    let missing = comparison
        .composition_rows
        .iter()
        .find(|row| row.component_id == "only-compared")
        .unwrap();
    assert_eq!(missing.base_fraction_text, "-");
    assert_eq!(missing.compared_fraction_text, "0 mol%");
    assert_eq!(missing.delta_text, "-");
    let liquid = comparison
        .phase_rows
        .iter()
        .find(|row| row.phase_label == "liquid")
        .unwrap();
    assert_eq!(liquid.molar_flow_delta_text, "+0.9 kmol/h");
    assert_eq!(liquid.molar_enthalpy_delta_text, "-");
}

#[test]
fn conversion_errors_are_explicit_and_prevent_any_successful_text_export() {
    let mut raw = raw_snapshot();
    raw.streams[0].temperature_k = f64::NAN;
    raw.streams[1].total_molar_flow_mol_s = f64::MAX;
    let model = solve_snapshot_model_from_ui(&raw, &DisplayUnitSet::engineering());
    assert_eq!(model.status_label, "Converged");
    assert!(
        model.streams[0]
            .temperature_text
            .starts_with("Conversion failed:")
    );
    assert!(
        model
            .presentation_errors
            .iter()
            .any(|error| error.quantity == Q::AbsoluteTemperature
                && error.cause == ConversionError::NonFiniteValue)
    );
    assert!(model.presentation_errors.iter().any(|error| error.quantity == Q::MolarFlow && error.cause == ConversionError::Overflow));
    assert!(model.light_text_export().is_err());
}
