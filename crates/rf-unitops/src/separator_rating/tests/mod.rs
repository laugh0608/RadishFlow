use super::*;
use rf_model::engineering::*;
use rf_types::{SourceObservationId, units::*};

mod evidence;
mod matrix;
mod numerical;
mod provenance;

// Public arithmetic fixture. Geometry is documented independently of the fluid;
// neither record claims empirical property or separation-performance validity.
fn fixture() -> (SeparatorRatingTask, EngineeringBasis) {
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
    drawing.kind = SourceKind::Vendor;
    drawing.title = "Synthetic geometry drawing".into();
    drawing.reference = "Public test fixture, not an actual vendor drawing".into();
    drawing.locator = "D and LLL-to-NLL dimensions".into();
    drawing.review = SourceReview::Reviewed;
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
        method: method(),
        geometry: SeparatorGivenGeometry {
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
fn set(task: &mut SeparatorRatingTask, field: SeparatorRatingInput, value: f64) {
    task.cases[0]
        .inputs
        .insert(field, direct(field.quantity(), value, "operating"));
}
fn geometry(task: &mut SeparatorRatingTask, field: SeparatorGeometryInput, value: f64) {
    task.geometry
        .inputs
        .insert(field, direct(field.quantity(), value, "drawing"));
}
fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}
fn rejects(task: &SeparatorRatingTask, basis: &EngineeringBasis, kind: RatingIssueKind) {
    let failure = calculate(task, basis).unwrap_err();
    assert!(
        failure.issues.iter().any(|issue| issue.kind == kind),
        "{failure:?}"
    );
}
