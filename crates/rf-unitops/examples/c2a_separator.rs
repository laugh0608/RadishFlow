//! Offline C2-A sensitivity harness. Uses existing S1/S2 public calculation APIs.
//! All criteria and geometry are synthetic; this is not equipment certification.
use rf_model::{engineering::*, equipment::*};
use rf_types::units::QuantityKind;
use rf_unitops::{separator_rating, separator_sizing};
use std::{
    collections::BTreeMap,
    error::Error,
    io::{self, BufRead},
};

fn sizing_fixture() -> (SeparatorSizingTask, EngineeringBasis) {
    let source = SourceRecord {
        id: "synthetic".into(),
        revision: 1,
        kind: SourceKind::Assumed,
        title: "S0-B01 synthetic arithmetic input".into(),
        reference: String::new(),
        locator: String::new(),
        publication_version: String::new(),
        accessed_on: None,
        adoption_note: "Explicit synthetic test assumption; no physical accuracy claim".into(),
        review: SourceReview::Adopted,
        applicability: SourceApplicability {
            fluid_system: "synthetic two-phase fluid".into(),
            phase: SourcePhase::Both,
            operating_conditions: Some(SourceOperatingConditions {
                temperature_k: 300.0,
                pressure_pa: 100000.0,
            }),
            scope: "Full cylinder, effective whole-vessel K, explicit working-volume criteria"
                .into(),
        },
        observations: Vec::new(),
    };
    let values = [
        300.0, 100000.0, 0.8, 0.005, 10.0, 1010.0, 0.1, 0.8, 120.0, 1.0,
    ];
    let task = SeparatorSizingTask {
        id: "task-1".into(),
        revision: 1,
        tag: "V-101".into(),
        name: "Synthetic sizing".into(),
        method: separator_sizing::method(),
        case: SeparatorCase {
            id: "case-1".into(),
            name: "Single case".into(),
            purpose: "Arithmetic verification".into(),
            fluid_system: source.applicability.fluid_system.clone(),
            applicability_source_ids: vec![source.id.clone()],
            inputs: SeparatorInput::ALL
                .into_iter()
                .zip(values)
                .map(|(field, value)| (field, sizing_direct(field.quantity(), value)))
                .collect(),
            applicability: SeparatorApplicability {
                orientation: Some(SeparatorOrientation::Vertical),
                gas_direction: Some(SeparatorGasDirection::Upward),
                service: Some(SeparatorService::CleanNonFoamingTwoPhase),
                geometry: Some(SeparatorSectionGeometry::FullEqualDiameterCylinder),
                k_basis: Some(SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals),
                flow_basis: Some(SeparatorFlowBasis::ActualAtOperatingConditions),
                internals_description:
                    "No reduction of effective full-cylinder area; test assumption".into(),
            },
        },
        axial_sections: BTreeMap::new(),
    };
    (
        task,
        EngineeringBasis {
            sources: vec![source],
            basis_entries: Vec::new(),
        },
    )
}

fn sizing_direct(quantity: QuantityKind, value_si: f64) -> InputBinding {
    InputBinding::Direct {
        value: EngineeringValue { quantity, value_si },
        source_id: "synthetic".into(),
    }
}
fn rating_fixture() -> (SeparatorRatingTask, EngineeringBasis) {
    let operating = SourceRecord {
        id: "operating".into(),
        revision: 2,
        kind: SourceKind::Assumed,
        title: "Synthetic operating case".into(),
        reference: String::new(),
        locator: String::new(),
        publication_version: String::new(),
        accessed_on: None,
        adoption_note: "Explicit arithmetic test assumption; no industrial accuracy claim".into(),
        review: SourceReview::Adopted,
        applicability: SourceApplicability {
            fluid_system: "Synthetic two-phase fluid".into(),
            phase: SourcePhase::Both,
            operating_conditions: Some(SourceOperatingConditions {
                temperature_k: 300.0,
                pressure_pa: 100000.0,
            }),
            scope: "Clean two-phase, effective whole-vessel K, actual flows".into(),
        },
        observations: vec![],
    };
    let mut drawing = operating.clone();
    drawing.id = "drawing".into();
    drawing.kind = SourceKind::Assumed;
    drawing.title = "Synthetic geometry drawing".into();
    drawing.reference = "Public test fixture, not an actual vendor drawing".into();
    drawing.locator = "D and LLL-to-NLL dimensions".into();
    drawing.review = SourceReview::Adopted;
    drawing.applicability = SourceApplicability {
        fluid_system: String::new(),
        phase: SourcePhase::NotApplicable,
        operating_conditions: None,
        scope: "Full equal-diameter cylinder with no effective area reduction".into(),
    };
    let task = SeparatorRatingTask {
        id: "rating-1".into(),
        revision: 3,
        tag: "V-101".into(),
        name: "Given-geometry arithmetic fixture".into(),
        method: separator_rating::method(),
        geometry: SeparatorGivenGeometry {
            adoption: None,
            revision: 4,
            applicability: SeparatorGeometryApplicability {
                orientation: Some(SeparatorOrientation::Vertical),
                gas_direction: Some(SeparatorGasDirection::Upward),
                geometry: Some(SeparatorSectionGeometry::FullEqualDiameterCylinder),
                internals_description: "Full-section effective K assumption".into(),
            },
            applicability_source_ids: vec![drawing.id.clone()],
            inputs: [
                (
                    SeparatorGeometryInput::InternalDiameter,
                    std::f64::consts::FRAC_2_SQRT_PI,
                ),
                (SeparatorGeometryInput::WorkingLiquidHeight, 0.6),
            ]
            .into_iter()
            .map(|(f, v)| (f, direct(f.quantity(), v, "drawing")))
            .collect(),
            axial_sections: BTreeMap::new(),
        },
        cases: vec![SeparatorRatingCase {
            id: "rating-case-1".into(),
            revision: 5,
            name: "Single case".into(),
            purpose: "Arithmetic verification".into(),
            fluid_system: operating.applicability.fluid_system.clone(),
            applicability: SeparatorRatingApplicability {
                service: Some(SeparatorService::CleanNonFoamingTwoPhase),
                k_basis: Some(SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals),
                flow_basis: Some(SeparatorFlowBasis::ActualAtOperatingConditions),
            },
            applicability_source_ids: vec![operating.id.clone()],
            inputs: SeparatorRatingInput::ALL
                .into_iter()
                .zip([300.0, 100000.0, 0.8, 0.005, 10.0, 1010.0, 0.1, 0.8, 120.0])
                .map(|(f, v)| (f, direct(f.quantity(), v, "operating")))
                .collect(),
        }],
    };
    (
        task,
        EngineeringBasis {
            basis_entries: vec![],
            sources: vec![operating, drawing],
        },
    )
}
fn direct(quantity: QuantityKind, value_si: f64, source: &str) -> InputBinding {
    InputBinding::Direct {
        value: EngineeringValue { quantity, value_si },
        source_id: source.into(),
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let expected = "id\tbackend\ttemperature_k\tpressure_pa\tgas_flow_m3_s\tliquid_flow_m3_s\tgas_density_kg_m3\tliquid_density_kg_m3\teffective_k_m_s\tvelocity_utilization\tworking_time_s\tmaximum_working_height_m\trating_diameter_m\trating_height_m";
    if lines.next().transpose()?.as_deref() != Some(expected) {
        return Err("Unexpected C2-A input header".into());
    }
    println!(
        "id\tbackend\tminimum_diameter_m\tgas_load\tliquid_load\tsizing_control\tgas_status\tliquid_status\tconclusion"
    );
    for (index, line) in lines.enumerate() {
        let line = line?;
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 14 || !["PR", "HEOS"].contains(&fields[1]) {
            return Err(format!("Invalid C2-A input row {}", index + 2).into());
        }
        let values = fields[2..]
            .iter()
            .map(|s| s.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()?;
        if values.iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return Err(format!("Invalid C2-A numeric input row {}", index + 2).into());
        }
        let (mut sizing, mut sizing_basis) = sizing_fixture();
        let (mut rating, mut rating_basis) = rating_fixture();
        for (field, value) in SeparatorInput::ALL.into_iter().zip(&values[..10]) {
            sizing
                .case
                .inputs
                .insert(field, sizing_direct(field.quantity(), *value));
        }
        for (field, value) in SeparatorRatingInput::ALL.into_iter().zip(&values[..9]) {
            rating.cases[0]
                .inputs
                .insert(field, direct(field.quantity(), *value, "operating"));
        }
        for (field, value) in [
            SeparatorGeometryInput::InternalDiameter,
            SeparatorGeometryInput::WorkingLiquidHeight,
        ]
        .into_iter()
        .zip(&values[10..])
        {
            rating
                .geometry
                .inputs
                .insert(field, direct(field.quantity(), *value, "drawing"));
        }
        for source in [&mut sizing_basis.sources[0], &mut rating_basis.sources[0]] {
            source.title = format!(
                "C2-A {} {} offline reference sensitivity",
                fields[0], fields[1]
            );
            source.reference = "tests/thermo-validation/c2a-pr76/reference-results.json".into();
            source.locator = fields[0].into();
            source.applicability.operating_conditions = Some(SourceOperatingConditions {
                temperature_k: values[0],
                pressure_pa: values[1],
            });
        }
        let sized = separator_sizing::calculate(&sizing, &sizing_basis)
            .map_err(|e| format!("Sizing {}: {e:?}", fields[0]))?;
        let rated = separator_rating::calculate(&rating, &rating_basis)
            .map_err(|e| format!("Rating {}: {e:?}", fields[0]))?;
        let d = sized.dimensions();
        let p = rated.performance();
        println!(
            "{}\t{}\t{:.17e}\t{:.17e}\t{:.17e}\t{:?}\t{:?}\t{:?}\t{:?}",
            fields[0],
            fields[1],
            d.minimum_diameter_m,
            p.gas.load_fraction,
            p.liquid.load_fraction,
            d.controlling_constraint,
            p.gas.status,
            p.liquid.status,
            p.conclusion()
        );
    }
    Ok(())
}
