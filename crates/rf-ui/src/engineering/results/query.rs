use super::*;

/// Frozen record revisions and labels from submission, including failed calculations.
/// This is evidence, not a serialization of session-local publication authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentResultOrigin {
    pub document_id: DocumentId,
    pub calculation_id: CalculationId,
    pub member: EquipmentCaseKey,
    pub method: EquipmentMethod,
    pub generation: ResultGeneration,
    pub task_revision: u64,
    pub case_revision: Option<u64>,
    pub geometry_revision: Option<u64>,
    pub task_tag: String,
    pub task_name: String,
    pub case_name: String,
    pub basis_revisions: BTreeMap<BasisEntryId, u64>,
    pub source_revisions: BTreeMap<SourceId, u64>,
}
impl EquipmentResultOrigin {
    pub(super) fn capture(
        session: &EngineeringSession,
        key: &CalculationKey,
        id: &CalculationId,
    ) -> Self {
        let task = session
            .current
            .equipment_task(&key.member.task_id)
            .expect("qualified task");
        let projected =
            impact::case_projection(Some(task), &key.member.case_id).expect("qualified case");
        let basis = impact::captured_basis(&session.current, &projected);
        let (case_revision, geometry_revision, case_name) = match task {
            EquipmentTask::SeparatorSizing(t) => (None, None, t.case.name.clone()),
            EquipmentTask::SeparatorRating(t) => {
                let c = t
                    .cases
                    .iter()
                    .find(|c| c.id == key.member.case_id)
                    .expect("qualified case");
                (Some(c.revision), Some(t.geometry.revision), c.name.clone())
            }
        };
        Self {
            document_id: key.document.clone(),
            calculation_id: id.clone(),
            member: key.member.clone(),
            method: key.method.clone(),
            generation: key.generation,
            task_revision: task.revision(),
            case_revision,
            geometry_revision,
            task_tag: task.tag().into(),
            task_name: task.name().into(),
            case_name,
            basis_revisions: basis
                .basis_entries
                .into_iter()
                .map(|v| (v.id, v.revision))
                .collect(),
            source_revisions: basis
                .sources
                .into_iter()
                .map(|v| (v.id, v.revision))
                .collect(),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentResultUnavailable {
    NoResult,
    MemberMissing,
    InputChanged,
    EvidenceChanged,
    InputAndEvidenceChanged,
    EligibilityRevoked,
}
#[derive(Debug)]
pub enum EquipmentResultValue<'a> {
    Sizing(&'a SeparatorCandidate),
    Rating(&'a SeparatorRatingResult),
}
#[derive(Debug)]
pub enum EquipmentResultFailure<'a> {
    Sizing(&'a SizingFailure),
    Rating(&'a RatingFailure),
}
#[derive(Debug)]
pub enum EquipmentResultView<'a> {
    Current {
        origin: &'a EquipmentResultOrigin,
        value: EquipmentResultValue<'a>,
        pending: bool,
    },
    Stale {
        origin: &'a EquipmentResultOrigin,
        reason: EquipmentResultUnavailable,
        pending: bool,
    },
    Missing {
        reason: EquipmentResultUnavailable,
        pending: bool,
    },
    Failed {
        origin: &'a EquipmentResultOrigin,
        failure: EquipmentResultFailure<'a>,
        pending: bool,
    },
}
impl EquipmentResultView<'_> {
    pub fn origin(&self) -> Option<&EquipmentResultOrigin> {
        match self {
            Self::Current { origin, .. }
            | Self::Stale { origin, .. }
            | Self::Failed { origin, .. } => Some(origin),
            Self::Missing { .. } => None,
        }
    }

    pub fn status(&self) -> EquipmentResultStatus {
        match self {
            Self::Current { .. } => EquipmentResultStatus::Current,
            Self::Stale { .. } => EquipmentResultStatus::Stale,
            Self::Missing { .. } => EquipmentResultStatus::Missing,
            Self::Failed { .. } => EquipmentResultStatus::Failed,
        }
    }
}
impl EngineeringSession {
    pub fn query_case_result(&self, member: &EquipmentCaseKey) -> EquipmentResultView<'_> {
        let pending = self.case_calculation_pending(member);
        let Some(record) = self.results.get(member) else {
            return EquipmentResultView::Missing {
                pending,
                reason: if self.calculation_key(member).is_none() {
                    EquipmentResultUnavailable::MemberMissing
                } else {
                    EquipmentResultUnavailable::NoResult
                },
            };
        };
        let origin = &record.origin;
        if self.case_result_status(member) == EquipmentResultStatus::Stale {
            let reason = match self.calculation_key(member) {
                None => EquipmentResultUnavailable::MemberMissing,
                Some(key) => match (
                    key.generation.input != record.key.generation.input,
                    key.generation.evidence != record.key.generation.evidence,
                ) {
                    (true, true) => EquipmentResultUnavailable::InputAndEvidenceChanged,
                    (true, false) => EquipmentResultUnavailable::InputChanged,
                    (false, true) => EquipmentResultUnavailable::EvidenceChanged,
                    (false, false) => EquipmentResultUnavailable::EligibilityRevoked,
                },
            };
            return EquipmentResultView::Stale {
                origin,
                reason,
                pending,
            };
        }
        match &record.outcome {
            CalculationOutcome::Sizing(Ok(value)) => EquipmentResultView::Current {
                origin,
                value: EquipmentResultValue::Sizing(value),
                pending,
            },
            CalculationOutcome::Rating(Ok(value)) => EquipmentResultView::Current {
                origin,
                value: EquipmentResultValue::Rating(value),
                pending,
            },
            CalculationOutcome::Sizing(Err(error)) => EquipmentResultView::Failed {
                origin,
                failure: EquipmentResultFailure::Sizing(error),
                pending,
            },
            CalculationOutcome::Rating(Err(error)) => EquipmentResultView::Failed {
                origin,
                failure: EquipmentResultFailure::Rating(error),
                pending,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engineering::rating_tests::{member, session, task};
    use crate::engineering::{EngineeringCommand, EngineeringInputs};
    use rf_model::equipment::SeparatorRatingInput;

    fn origin(s: &EngineeringSession) -> EquipmentResultOrigin {
        match s.query_case_result(&member()) {
            EquipmentResultView::Current { origin, .. }
            | EquipmentResultView::Stale { origin, .. }
            | EquipmentResultView::Failed { origin, .. } => origin.clone(),
            _ => panic!("expected recorded origin"),
        }
    }
    #[test]
    fn origin_freezes_method_labels_geometry_and_dependency_versions() {
        let mut s = session();
        s.calculate_rating(&member()).unwrap();
        let original = origin(&s);
        assert_eq!(original.task_revision, 5);
        assert_eq!(original.case_revision, Some(9));
        assert_eq!(original.geometry_revision, Some(6));
        assert_eq!(original.basis_revisions.get(&"basis-time".into()), Some(&7));
        assert_eq!(
            original.basis_revisions.get(&"basis-diameter".into()),
            Some(&8)
        );
        assert_eq!(original.source_revisions.get(&"drawing".into()), Some(&4));
        s.apply(vec![EngineeringCommand::UpdateRatingLabels {
            task: member().task_id,
            revision: 5,
            tag: "new".into(),
            name: "new".into(),
        }])
        .unwrap();
        assert_eq!(
            s.query_case_result(&member()).status(),
            EquipmentResultStatus::Current
        );
        assert_eq!(origin(&s), original);
        let mut source = s.inputs().basis.sources[0].clone();
        source.adoption_note = "Changed evidence".into();
        s.apply(vec![EngineeringCommand::UpdateSource(Box::new(source))])
            .unwrap();
        assert!(matches!(
            s.query_case_result(&member()),
            EquipmentResultView::Stale {
                reason: EquipmentResultUnavailable::EvidenceChanged,
                ..
            }
        ));
        assert_eq!(origin(&s), original);
    }
    #[test]
    fn current_stale_failed_and_missing_expose_distinct_typed_payloads() {
        let mut s = session();
        assert!(matches!(
            s.query_case_result(&member()),
            EquipmentResultView::Missing {
                reason: EquipmentResultUnavailable::NoResult,
                pending: false
            }
        ));
        s.calculate_rating(&member()).unwrap();
        assert!(matches!(
            s.query_case_result(&member()),
            EquipmentResultView::Current {
                value: EquipmentResultValue::Rating(_),
                pending: false,
                ..
            }
        ));
        let mut case = task(&s).cases.remove(0);
        case.inputs.remove(&SeparatorRatingInput::GasDensity);
        s.apply(vec![EngineeringCommand::UpdateRatingCase {
            task: member().task_id,
            case: Box::new(case),
        }])
        .unwrap();
        assert!(matches!(
            s.query_case_result(&member()),
            EquipmentResultView::Stale {
                reason: EquipmentResultUnavailable::InputAndEvidenceChanged,
                ..
            }
        ));
        s.calculate_rating(&member()).unwrap();
        assert!(matches!(
            s.query_case_result(&member()),
            EquipmentResultView::Failed {
                failure: EquipmentResultFailure::Rating(_),
                pending: false,
                ..
            }
        ));
        assert_eq!(origin(&s).case_revision, Some(10));
        let missing = EquipmentCaseKey {
            case_id: "absent".into(),
            ..member()
        };
        assert!(matches!(
            s.query_case_result(&missing),
            EquipmentResultView::Missing {
                reason: EquipmentResultUnavailable::MemberMissing,
                ..
            }
        ));
    }
    #[test]
    fn rerun_keeps_sizing_current_but_revokes_rating_until_completion() {
        let (task, basis) = crate::engineering::tests::fixture();
        let sizing = EquipmentCaseKey {
            task_id: task.id.clone(),
            case_id: task.case.id.clone(),
        };
        let mut s = EngineeringSession::from_inputs(
            "sizing".into(),
            EngineeringInputs {
                basis,
                tasks: vec![task.into()],
            },
        )
        .unwrap();
        s.calculate(&sizing.task_id).unwrap();
        let request = s.begin_calculation(&sizing.task_id).unwrap();
        assert!(matches!(
            s.query_case_result(&sizing),
            EquipmentResultView::Current {
                value: EquipmentResultValue::Sizing(_),
                pending: true,
                ..
            }
        ));
        s.complete_calculation(request.execute()).unwrap();
        let mut rating = session();
        rating.calculate_rating(&member()).unwrap();
        let original = origin(&rating);
        let request = rating.begin_rating(&member()).unwrap();
        assert!(matches!(
            rating.query_case_result(&member()),
            EquipmentResultView::Stale {
                reason: EquipmentResultUnavailable::EligibilityRevoked,
                pending: true,
                ..
            }
        ));
        assert_eq!(origin(&rating), original);
        rating.complete_calculation(request.execute()).unwrap();
        assert_ne!(origin(&rating).calculation_id, original.calculation_id);
    }
    #[test]
    fn unrelated_case_dependencies_are_not_attributed_to_this_result() {
        let mut s = session();
        s.apply(vec![EngineeringCommand::UpgradeRatingMethod(
            member().task_id,
        )])
        .unwrap();
        let mut next = task(&s).cases[0].clone();
        next.id = "other".into();
        next.name = "Other case".into();
        s.apply(vec![EngineeringCommand::AddRatingCase {
            task: member().task_id,
            case: Box::new(next),
        }])
        .unwrap();
        s.calculate_rating(&member()).unwrap();
        let original = origin(&s);
        let mut other = task(&s)
            .cases
            .into_iter()
            .find(|c| c.id.as_str() == "other")
            .unwrap();
        other.inputs.remove(&SeparatorRatingInput::GasDensity);
        s.apply(vec![EngineeringCommand::UpdateRatingCase {
            task: member().task_id,
            case: Box::new(other),
        }])
        .unwrap();
        assert_eq!(
            s.query_case_result(&member()).status(),
            EquipmentResultStatus::Current
        );
        assert_eq!(origin(&s), original);
    }
}
