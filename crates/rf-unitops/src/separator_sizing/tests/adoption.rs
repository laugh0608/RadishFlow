use super::*;
use crate::separator_adoption::*;

#[test]
fn mapping_preserves_unrounded_diameter_and_calculated_height_not_height_limit() {
    let (task, basis) = fixture();
    let candidate = calculate(&task, &basis).unwrap();
    let adopted = map_candidate(&candidate, "doc".into(), GeometryAdjustments::default()).unwrap();
    assert_eq!(
        adopted.diameter.adopted_m,
        candidate.dimensions().minimum_diameter_m
    );
    assert_eq!(adopted.working_liquid_height.adopted_m, 0.6);
    assert_ne!(
        adopted.working_liquid_height.adopted_m,
        candidate.capture().values_si[&SeparatorInput::MaximumWorkingLiquidHeight]
    );
    assert_eq!(adopted.source_task, task);
    assert_eq!(adopted.source_basis, basis);
    assert!(adopted.axial_lengths_m.is_empty());
}

#[test]
fn diameter_adjustment_is_independent_and_requires_finite_positive_value_and_reason() {
    let (task, basis) = fixture();
    let candidate = calculate(&task, &basis).unwrap();
    let adjustments = |value_m, reason: &str| GeometryAdjustments {
        diameter: Some(DimensionAdjustment {
            value_m,
            reason: reason.into(),
        }),
        working_liquid_height: None,
    };
    let adopted = map_candidate(
        &candidate,
        "doc".into(),
        adjustments(1.2, "Selected standard size"),
    )
    .unwrap();
    assert_eq!(
        adopted.diameter.original_m,
        candidate.dimensions().minimum_diameter_m
    );
    assert_eq!(adopted.diameter.adopted_m, 1.2);
    assert_eq!(adopted.working_liquid_height.adopted_m, 0.6);
    for value in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert!(map_candidate(&candidate, "doc".into(), adjustments(value, "Reason")).is_err());
    }
    assert!(map_candidate(&candidate, "doc".into(), adjustments(1.2, " ")).is_err());
    assert!(map_candidate(&candidate, " ".into(), GeometryAdjustments::default()).is_err());
}

#[test]
fn mapping_adopts_only_valid_segments_and_retains_incomplete_original_evidence() {
    let (mut task, mut basis) = fixture();
    let valid = SeparatorAxialSection::NormalToHighLiquidLevel;
    let invalid = SeparatorAxialSection::InternalsToUpperTangent;
    task.axial_sections
        .insert(valid, direct(QuantityKind::Length, 0.2));
    let mut unreviewed = basis.sources[0].clone();
    unreviewed.id = "unreviewed".into();
    unreviewed.review = SourceReview::Unreviewed;
    basis.sources.push(unreviewed);
    task.axial_sections.insert(
        invalid,
        InputBinding::Direct {
            value: EngineeringValue {
                quantity: QuantityKind::Length,
                value_si: 0.5,
            },
            source_id: "unreviewed".into(),
        },
    );
    let candidate = calculate(&task, &basis).unwrap();
    assert_eq!(candidate.dimensions().tangent_to_tangent_height_m, None);
    let adopted = map_candidate(&candidate, "doc".into(), GeometryAdjustments::default()).unwrap();
    assert_eq!(adopted.axial_lengths_m, [(valid, 0.2)].into());
    assert!(adopted.source_task.axial_sections.contains_key(&invalid));
    assert_eq!(
        adopted
            .source_basis
            .sources
            .iter()
            .find(|s| s.id.as_str() == "unreviewed")
            .unwrap()
            .review,
        SourceReview::Unreviewed
    );
}
