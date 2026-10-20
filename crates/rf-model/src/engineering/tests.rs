use super::*;
use EngineeringDataErrorKind as Error;
use rf_types::units::QuantityKind::{Duration, Length};

fn source(id: &str) -> SourceRecord {
    SourceRecord {
        id: id.into(),
        revision: 3,
        kind: SourceKind::Unknown,
        title: String::new(),
        reference: String::new(),
        locator: String::new(),
        publication_version: String::new(),
        accessed_on: None,
        adoption_note: String::new(),
        review: SourceReview::Unreviewed,
        applicability: SourceApplicability {
            fluid_system: String::new(),
            phase: SourcePhase::NotApplicable,
            operating_conditions: None,
            scope: String::new(),
        },
        observations: vec![],
    }
}
fn basis() -> EngineeringBasis {
    EngineeringBasis {
        sources: vec![source("s"), source("override")],
        basis_entries: vec![BasisEntry {
            id: "b".into(),
            revision: 7,
            name: "time".into(),
            value: EngineeringValue {
                quantity: Duration,
                value_si: 120.0,
            },
            source_id: "s".into(),
        }],
    }
}
fn observation(id: &str, depends_on: Vec<SourceObservationRef>) -> SourceObservation {
    SourceObservation {
        id: id.into(),
        quantity: Duration,
        original_value: 2.0,
        original_unit: "min".into(),
        canonical_value: 120.0,
        conversion_note: "documented conversion, never executed".into(),
        depends_on,
    }
}
fn reference(source_id: &str, observation_id: &str) -> SourceObservationRef {
    SourceObservationRef {
        source_id: source_id.into(),
        observation_id: observation_id.into(),
    }
}

#[test]
fn bindings_resolve_explicit_values_and_preserve_basis_and_source_identity() {
    let mut data = basis();
    let direct = InputBinding::Direct {
        value: EngineeringValue {
            quantity: Duration,
            value_si: 45.0,
        },
        source_id: "s".into(),
    };
    let shared = InputBinding::BasisRef {
        basis_entry_id: "b".into(),
    };
    let local = InputBinding::Override {
        basis_entry_id: "b".into(),
        value: EngineeringValue {
            quantity: Duration,
            value_si: 180.0,
        },
        source_id: "override".into(),
        reason: "explicit override".into(),
    };
    assert_eq!(
        data.resolve(Duration, &direct).unwrap().value.value_si,
        45.0
    );
    assert_eq!(
        data.resolve(Duration, &shared).unwrap().value.value_si,
        120.0
    );
    let resolved = data.resolve(Duration, &local).unwrap();
    assert_eq!(resolved.value.value_si, 180.0);
    assert_eq!(resolved.source.id.as_str(), "override");
    assert!(
        matches!(resolved.origin, BindingOrigin::Override { basis, .. } if basis.id.as_str() == "b" && basis.revision == 7)
    );
    assert_eq!(data.basis_entries[0].value.value_si, 120.0);
    data.basis_entries[0].name = "renamed".into();
    data.basis_entries[0].value.value_si = 240.0;
    data.basis_entries[0].revision += 1;
    assert_eq!(
        data.resolve(Duration, &shared).unwrap().value.value_si,
        240.0
    );
    assert_eq!(
        data.resolve(Duration, &local).unwrap().value.value_si,
        180.0
    );
    data.basis_entries.clear();
    assert!(matches!(
        data.resolve(Duration, &shared).unwrap_err().kind,
        Error::MissingBasis(_)
    ));
}

#[test]
fn invalid_bindings_fail_without_fallback_or_mutation() {
    let data = basis();
    let before = data.clone();
    for binding in [
        InputBinding::Direct {
            value: EngineeringValue {
                quantity: Length,
                value_si: 1.0,
            },
            source_id: "s".into(),
        },
        InputBinding::Direct {
            value: EngineeringValue {
                quantity: Duration,
                value_si: f64::NAN,
            },
            source_id: "s".into(),
        },
        InputBinding::Direct {
            value: EngineeringValue {
                quantity: Duration,
                value_si: 1.0,
            },
            source_id: "missing".into(),
        },
        InputBinding::Override {
            basis_entry_id: "b".into(),
            value: EngineeringValue {
                quantity: Duration,
                value_si: 1.0,
            },
            source_id: "s".into(),
            reason: " ".into(),
        },
        InputBinding::Override {
            basis_entry_id: "b".into(),
            value: EngineeringValue {
                quantity: Length,
                value_si: 1.0,
            },
            source_id: "s".into(),
            reason: "explicit".into(),
        },
    ] {
        assert!(data.resolve(Duration, &binding).is_err());
    }
    assert_eq!(data, before);
}

#[test]
fn incomplete_sources_and_review_labels_are_not_engineering_certification() {
    let mut data = basis();
    let binding = InputBinding::BasisRef {
        basis_entry_id: "b".into(),
    };
    for review in [
        SourceReview::Unreviewed,
        SourceReview::Adopted,
        SourceReview::Reviewed,
    ] {
        data.sources[0].review = review;
        let resolved = data.resolve(Duration, &binding).unwrap();
        assert_eq!(resolved.source.kind, SourceKind::Unknown);
        assert_eq!(resolved.source.review, review);
        assert!(resolved.source.reference.is_empty());
    }
}

#[test]
fn records_validate_ids_values_and_paired_operating_conditions() {
    let mut data = basis();
    data.sources.push(data.sources[0].clone());
    assert_eq!(data.validate().unwrap_err().kind, Error::DuplicateId);
    data = basis();
    data.basis_entries.push(data.basis_entries[0].clone());
    assert_eq!(data.validate().unwrap_err().kind, Error::DuplicateId);
    data = basis();
    data.sources[0].id = " ".into();
    assert_eq!(data.validate().unwrap_err().kind, Error::EmptyId);
    data = basis();
    data.basis_entries[0].source_id = "S".into();
    assert!(matches!(
        data.validate().unwrap_err().kind,
        Error::MissingSource(_)
    ));
    data = basis();
    data.basis_entries[0].value.value_si = f64::INFINITY;
    assert_eq!(data.validate().unwrap_err().kind, Error::NonFiniteValue);
    data = basis();
    data.sources[0].applicability.operating_conditions = Some(SourceOperatingConditions {
        temperature_k: 300.0,
        pressure_pa: 0.0,
    });
    assert_eq!(
        data.validate().unwrap_err().kind,
        Error::InvalidOperatingConditions
    );
}

#[test]
fn observation_links_are_scoped_by_source_and_cycles_or_missing_links_are_rejected() {
    let mut data = basis();
    data.sources[0].observations = vec![observation("a", vec![])];
    data.sources[1].observations = vec![observation("a", vec![reference("s", "a")])];
    data.validate().unwrap();
    data.sources[0].observations[0].depends_on = vec![reference("override", "a")];
    assert_eq!(data.validate().unwrap_err().kind, Error::ObservationCycle);
    data.sources[0].observations[0].depends_on = vec![reference("absent", "a")];
    let error = data.validate().unwrap_err();
    assert!(matches!(error.kind, Error::MissingObservation(_)));
    assert_eq!(error.path, "sources[0].observations[0].dependsOn[0]");
    data.sources[0].observations[0].depends_on.clear();
    data.sources[0].observations.push(observation("a", vec![]));
    assert_eq!(data.validate().unwrap_err().kind, Error::DuplicateId);
}

#[test]
fn long_observation_chains_do_not_recurse_and_self_cycles_fail() {
    let mut data = basis();
    for i in 0..2048 {
        let dependencies = if i == 0 {
            vec![]
        } else {
            vec![reference("s", &(i - 1).to_string())]
        };
        data.sources[0]
            .observations
            .push(observation(&i.to_string(), dependencies));
    }
    data.validate().unwrap();
    data.sources[0].observations[0].depends_on = vec![reference("s", "0")];
    assert_eq!(data.validate().unwrap_err().kind, Error::ObservationCycle);
}

#[test]
fn access_dates_are_exact_iso_dates_with_calendar_validation() {
    for date in ["2024-02-29", "2000-02-29", "2026-10-11", "0000-01-01"] {
        assert_eq!(date.parse::<SourceAccessDate>().unwrap().to_string(), date);
    }
    for date in [
        "1900-02-29",
        "2026-02-29",
        "2026-04-31",
        "2026-00-01",
        "2026-01-00",
        "2026-1-01",
        "２０２６-01-01",
        "2026-10-11T00:00:00Z",
    ] {
        assert!(date.parse::<SourceAccessDate>().is_err(), "{date}");
    }
}
