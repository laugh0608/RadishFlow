use super::*;
use crate::separator_evidence::{self, EvidenceIssue, OperatingContext, SourceEvidence, SourceUse};
use rf_model::engineering::{EngineeringBasis, InputBinding};
use rf_types::units::QuantityKind;

fn issue(issues: &mut Vec<RatingIssue>, target: RatingIssueTarget, kind: RatingIssueKind) {
    issues.push(RatingIssue { target, kind });
}
fn input_target(case_id: &EngineeringCaseId, field: SeparatorRatingInput) -> RatingIssueTarget {
    RatingIssueTarget::Input {
        case_id: case_id.clone(),
        field,
    }
}

/// Input/evidence readiness only. Arithmetic range checks occur in calculate.
/// Optional incomplete height evidence is reported on a successful result.
pub fn check(task: &SeparatorRatingTask, basis: &EngineeringBasis) -> Vec<RatingIssue> {
    match prepare(task, basis) {
        Ok(_) => Vec::new(),
        Err(failure) => failure.issues,
    }
}

pub fn check_case(
    task: &SeparatorRatingTask,
    case_id: &EngineeringCaseId,
    basis: &EngineeringBasis,
) -> Vec<RatingIssue> {
    match prepare_case(task, case_id, basis) {
        Ok(_) => Vec::new(),
        Err(failure) => failure.issues,
    }
}

pub(super) fn validate_task(
    task: &SeparatorRatingTask,
    allow_matrix: bool,
) -> Result<(), RatingFailure> {
    let mut issues = Vec::new();
    if let Err(error) = validate_separator_rating_identities(std::slice::from_ref(task)) {
        issue(
            &mut issues,
            RatingIssueTarget::Task,
            RatingIssueKind::InvalidIdentity(error.kind),
        );
    }
    if task.method != method()
        && !(allow_matrix && (task.method == matrix_method() || task.method == adoption_method()))
    {
        issue(
            &mut issues,
            RatingIssueTarget::Method,
            RatingIssueKind::UnsupportedMethod,
        );
    }
    if task.cases.is_empty()
        || (task.method == method() && task.cases.len() != 1)
        || (!allow_matrix && task.cases.len() != 1)
    {
        issue(
            &mut issues,
            RatingIssueTarget::Task,
            RatingIssueKind::UnsupportedCaseCount {
                actual: task.cases.len(),
            },
        );
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(RatingFailure { issues })
    }
}

/// Preserve the original single-case version 1 entry point.
pub(super) fn prepare(
    task: &SeparatorRatingTask,
    basis: &EngineeringBasis,
) -> Result<(RatingInputCapture, Vec<RatingIssue>), RatingFailure> {
    validate_task(task, false)?;
    prepare_case(task, &task.cases[0].id, basis)
}
pub(super) fn prepare_case(
    task: &SeparatorRatingTask,
    case_id: &EngineeringCaseId,
    basis: &EngineeringBasis,
) -> Result<(RatingInputCapture, Vec<RatingIssue>), RatingFailure> {
    validate_task(task, true)?;
    let input = task.case_input(case_id).ok_or_else(|| RatingFailure {
        issues: vec![RatingIssue {
            target: RatingIssueTarget::Case(case_id.clone()),
            kind: RatingIssueKind::MissingInput,
        }],
    })?;
    prepare_input(input, basis)
}
fn prepare_input(
    input: SeparatorRatingCaseInput,
    basis: &EngineeringBasis,
) -> Result<(RatingInputCapture, Vec<RatingIssue>), RatingFailure> {
    let mut issues = Vec::new();
    let case = &input.case;
    if case.fluid_system.trim().is_empty() {
        issue(
            &mut issues,
            RatingIssueTarget::Case(case.id.clone()),
            RatingIssueKind::MissingInput,
        );
    }
    check_applicability(&input.geometry, case, &mut issues);
    if let Err(error) = basis.validate() {
        issue(
            &mut issues,
            RatingIssueTarget::Basis,
            RatingIssueKind::InvalidBinding(error),
        );
        return Err(RatingFailure { issues });
    }
    let mut capture = RatingInputCapture {
        input: input.clone(),
        geometry_values_m: BTreeMap::new(),
        values_si: BTreeMap::new(),
        axial_lengths_m: BTreeMap::new(),
        basis_entries: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    for field in SeparatorRatingInput::ALL {
        let target = input_target(&case.id, field);
        let Some(binding) = case.inputs.get(&field) else {
            issue(&mut issues, target, RatingIssueKind::MissingInput);
            continue;
        };
        if let Some(value) = resolve(
            binding,
            field.quantity(),
            target.clone(),
            basis,
            &mut capture,
            &mut issues,
        ) {
            capture.values_si.insert(field, value);
            if value == 0.0
                && matches!(
                    field,
                    SeparatorRatingInput::GasVolumeFlow | SeparatorRatingInput::LiquidVolumeFlow
                )
            {
                issue(&mut issues, target, RatingIssueKind::AbsentPhase);
            } else if value <= 0.0 {
                issue(&mut issues, target, RatingIssueKind::NonPositiveInput);
            } else if field == SeparatorRatingInput::GasVelocityUtilization && value > 1.0 {
                issue(&mut issues, target, RatingIssueKind::UtilizationAboveOne);
            }
        }
    }
    use SeparatorRatingInput::*;
    if let (Some(gas), Some(liquid)) = (
        capture.values_si.get(&GasDensity),
        capture.values_si.get(&LiquidDensity),
    ) && liquid <= gas
    {
        issue(
            &mut issues,
            input_target(&case.id, LiquidDensity),
            RatingIssueKind::InvalidDensityOrder,
        );
    }
    for (&field, binding) in &case.inputs {
        let usage = match field {
            GasVolumeFlow | GasDensity => SourceUse::Gas,
            LiquidVolumeFlow | LiquidDensity => SourceUse::Liquid,
            EffectiveK => SourceUse::EffectiveK,
            _ => SourceUse::General,
        };
        binding_conditions(
            binding,
            field.quantity(),
            usage,
            input_target(&case.id, field),
            basis,
            &capture,
            &mut issues,
        );
    }
    for field in SeparatorGeometryInput::ALL {
        let target = RatingIssueTarget::GeometryInput(field);
        let Some(binding) = input.geometry.inputs.get(&field) else {
            issue(&mut issues, target, RatingIssueKind::MissingInput);
            continue;
        };
        if let Some(value) = resolve(
            binding,
            field.quantity(),
            target.clone(),
            basis,
            &mut capture,
            &mut issues,
        ) {
            capture.geometry_values_m.insert(field, value);
            if value <= 0.0 {
                issue(
                    &mut issues,
                    target.clone(),
                    RatingIssueKind::NonPositiveInput,
                );
            }
        }
        binding_conditions(
            binding,
            field.quantity(),
            SourceUse::General,
            target,
            basis,
            &capture,
            &mut issues,
        );
    }

    // Optional documentary gaps leave height unknown. Structural errors and
    // invalid lengths remain blocking even when the source is also incomplete.
    let mut height_issues = Vec::new();
    for (&section, binding) in &input.geometry.axial_sections {
        let target = RatingIssueTarget::AxialSection(section);
        let mut section_issues = Vec::new();
        let value = resolve(
            binding,
            QuantityKind::Length,
            target.clone(),
            basis,
            &mut capture,
            &mut section_issues,
        );
        if let Some(value) = value {
            if value < 0.0 || (value == 0.0 && section.requires_positive_clearance()) {
                issue(
                    &mut issues,
                    target.clone(),
                    RatingIssueKind::NonPositiveInput,
                );
            }
            binding_conditions(
                binding,
                QuantityKind::Length,
                SourceUse::General,
                target,
                basis,
                &capture,
                &mut section_issues,
            );
            if section_issues.is_empty() {
                capture.axial_lengths_m.insert(section, value);
            }
        }
        for problem in section_issues {
            if matches!(problem.kind, RatingIssueKind::InvalidBinding(_)) {
                issues.push(problem);
            } else {
                height_issues.push(problem);
            }
        }
    }
    scope_evidence(
        &input.geometry.applicability_source_ids,
        SourceUse::General,
        RatingIssueTarget::Geometry,
        basis,
        &mut capture,
        &mut issues,
    );
    scope_evidence(
        &case.applicability_source_ids,
        SourceUse::EffectiveK,
        RatingIssueTarget::Case(case.id.clone()),
        basis,
        &mut capture,
        &mut issues,
    );
    if issues.is_empty() {
        Ok((capture, height_issues))
    } else {
        Err(RatingFailure { issues })
    }
}

fn check_applicability(
    geometry: &SeparatorGivenGeometry,
    case: &SeparatorRatingCase,
    issues: &mut Vec<RatingIssue>,
) {
    fn require<T: PartialEq>(
        actual: Option<T>,
        expected: T,
        target: RatingIssueTarget,
        issues: &mut Vec<RatingIssue>,
    ) {
        let kind = match actual {
            None => Some(RatingIssueKind::MissingInput),
            Some(value) if value != expected => Some(RatingIssueKind::OutsideMethodScope),
            _ => None,
        };
        if let Some(kind) = kind {
            issue(issues, target, kind);
        }
    }
    let a = &geometry.applicability;
    let geometry = RatingIssueTarget::GeometryApplicability;
    require(
        a.orientation,
        SeparatorOrientation::Vertical,
        geometry(GeometryApplicabilityField::Orientation),
        issues,
    );
    require(
        a.gas_direction,
        SeparatorGasDirection::Upward,
        geometry(GeometryApplicabilityField::GasDirection),
        issues,
    );
    require(
        a.geometry,
        SeparatorSectionGeometry::FullEqualDiameterCylinder,
        geometry(GeometryApplicabilityField::Geometry),
        issues,
    );
    if a.internals_description.trim().is_empty() {
        issue(
            issues,
            geometry(GeometryApplicabilityField::Internals),
            RatingIssueKind::MissingInput,
        );
    }
    let a = &case.applicability;
    let case_target = |field| RatingIssueTarget::CaseApplicability {
        case_id: case.id.clone(),
        field,
    };
    require(
        a.service,
        SeparatorService::CleanNonFoamingTwoPhase,
        case_target(CaseApplicabilityField::Service),
        issues,
    );
    require(
        a.k_basis,
        SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals,
        case_target(CaseApplicabilityField::KBasis),
        issues,
    );
    require(
        a.flow_basis,
        SeparatorFlowBasis::ActualAtOperatingConditions,
        case_target(CaseApplicabilityField::FlowBasis),
        issues,
    );
}

fn resolve(
    binding: &InputBinding,
    quantity: QuantityKind,
    target: RatingIssueTarget,
    basis: &EngineeringBasis,
    capture: &mut RatingInputCapture,
    issues: &mut Vec<RatingIssue>,
) -> Option<f64> {
    let (value, problems) = SourceEvidence {
        basis_entries: &mut capture.basis_entries,
        sources: &mut capture.sources,
    }
    .resolve(binding, quantity, basis);
    append_evidence_issues(problems, target, issues);
    value
}
fn append_evidence_issues(
    problems: Vec<EvidenceIssue>,
    target: RatingIssueTarget,
    issues: &mut Vec<RatingIssue>,
) {
    for problem in problems {
        let kind = match problem {
            EvidenceIssue::InvalidBinding(error) => RatingIssueKind::InvalidBinding(error),
            EvidenceIssue::MissingSource(id) => RatingIssueKind::MissingSource(id),
            EvidenceIssue::MissingSourceEvidence => RatingIssueKind::MissingSourceEvidence,
            EvidenceIssue::SourceNotAdopted => RatingIssueKind::SourceNotAdopted,
        };
        issue(issues, target.clone(), kind);
    }
}
fn scope_evidence(
    ids: &[SourceId],
    usage: SourceUse,
    missing_target: RatingIssueTarget,
    basis: &EngineeringBasis,
    capture: &mut RatingInputCapture,
    issues: &mut Vec<RatingIssue>,
) {
    if ids.is_empty() {
        issue(
            issues,
            missing_target,
            RatingIssueKind::MissingSourceEvidence,
        );
    }
    for id in ids {
        let problems = SourceEvidence {
            basis_entries: &mut capture.basis_entries,
            sources: &mut capture.sources,
        }
        .capture_source(id, basis);
        let target = RatingIssueTarget::Source(id.clone());
        append_evidence_issues(problems, target.clone(), issues);
        if let Some(source) = basis.sources.iter().find(|s| s.id == *id) {
            check_conditions(source, usage, target, capture, issues);
        }
    }
}
fn binding_conditions(
    binding: &InputBinding,
    quantity: QuantityKind,
    usage: SourceUse,
    target: RatingIssueTarget,
    basis: &EngineeringBasis,
    capture: &RatingInputCapture,
    issues: &mut Vec<RatingIssue>,
) {
    // Structural problems have already been reported by resolve.
    if let Ok(resolved) = basis.resolve(quantity, binding) {
        check_conditions(resolved.source, usage, target, capture, issues);
    }
}
fn check_conditions(
    source: &SourceRecord,
    usage: SourceUse,
    target: RatingIssueTarget,
    capture: &RatingInputCapture,
    issues: &mut Vec<RatingIssue>,
) {
    let context = OperatingContext {
        fluid_system: &capture.input.case.fluid_system,
        temperature_k: capture
            .values_si
            .get(&SeparatorRatingInput::OperatingTemperature)
            .copied(),
        pressure_pa: capture
            .values_si
            .get(&SeparatorRatingInput::OperatingPressure)
            .copied(),
    };
    if !separator_evidence::conditions_match(source, usage, &context) {
        issue(issues, target, RatingIssueKind::SourceConditionMismatch);
    }
}
