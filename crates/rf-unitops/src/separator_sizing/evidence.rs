use super::*;
use rf_model::engineering::{
    BindingOrigin, EngineeringBasis, InputBinding, SourceKind, SourcePhase, SourceReview,
};
use rf_types::units::QuantityKind;

fn issue(issues: &mut Vec<SizingIssue>, target: IssueTarget, kind: SizingIssueKind) {
    issues.push(SizingIssue { target, kind });
}

/// Blocking issues for partial process sizing; optional height evidence issues
/// are retained on a successful candidate and leave those intervals uncalculated.
pub fn check(task: &SeparatorSizingTask, basis: &EngineeringBasis) -> Vec<SizingIssue> {
    match prepare(task, basis) {
        Ok(_) => Vec::new(),
        Err(failure) => failure.issues,
    }
}

pub(super) fn prepare(
    task: &SeparatorSizingTask,
    basis: &EngineeringBasis,
) -> Result<(SizingInputCapture, Vec<SizingIssue>), SizingFailure> {
    let mut issues = Vec::new();
    if let Err(error) = validate_equipment_identities(std::slice::from_ref(task)) {
        issue(
            &mut issues,
            IssueTarget::Task,
            SizingIssueKind::InvalidIdentity(error.kind),
        );
    }
    if task.method != method() {
        issue(
            &mut issues,
            IssueTarget::Method,
            SizingIssueKind::UnsupportedMethod,
        );
    }
    if task.case.fluid_system.trim().is_empty() {
        issue(
            &mut issues,
            IssueTarget::Case,
            SizingIssueKind::MissingInput,
        );
    }
    check_applicability(&task.case.applicability, &mut issues);
    if let Err(error) = basis.validate() {
        issue(
            &mut issues,
            IssueTarget::Basis,
            SizingIssueKind::InvalidBinding(error),
        );
        return Err(SizingFailure { issues });
    }
    let mut capture = SizingInputCapture {
        task: task.clone(),
        values_si: BTreeMap::new(),
        axial_lengths_m: BTreeMap::new(),
        basis_entries: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    for field in SeparatorInput::ALL {
        let target = IssueTarget::Input(field);
        let Some(binding) = task.case.inputs.get(&field) else {
            issue(&mut issues, target, SizingIssueKind::MissingInput);
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
                    SeparatorInput::GasVolumeFlow | SeparatorInput::LiquidVolumeFlow
                )
            {
                issue(&mut issues, target, SizingIssueKind::AbsentPhase);
            } else if value <= 0.0 {
                issue(&mut issues, target, SizingIssueKind::NonPositiveInput);
            } else if field == SeparatorInput::GasVelocityUtilization && value > 1.0 {
                issue(&mut issues, target, SizingIssueKind::UtilizationAboveOne);
            }
        }
    }
    use SeparatorInput::*;
    if let (Some(gas), Some(liquid)) = (
        capture.values_si.get(&GasDensity),
        capture.values_si.get(&LiquidDensity),
    ) && liquid <= gas
    {
        issue(
            &mut issues,
            IssueTarget::Input(LiquidDensity),
            SizingIssueKind::InvalidDensityOrder,
        );
    }
    // Incomplete optional evidence leaves height unknown, while malformed values
    // remain blocking. Retain every issue and the original binding for review.
    let mut height_issues = Vec::new();
    for (&section, binding) in &task.axial_sections {
        let target = IssueTarget::AxialSection(section);
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
                    SizingIssueKind::NonPositiveInput,
                );
            }
            if let Ok(resolved) = basis.resolve(QuantityKind::Length, binding) {
                check_conditions(None, target, resolved.source, &capture, &mut section_issues);
            }
            if section_issues.is_empty() {
                capture.axial_lengths_m.insert(section, value);
            }
        }
        for problem in section_issues {
            if matches!(problem.kind, SizingIssueKind::InvalidBinding(_)) {
                issues.push(problem);
            } else {
                height_issues.push(problem);
            }
        }
    }
    for (&field, binding) in &task.case.inputs {
        // Structural errors have already been reported by resolve.
        if let Ok(resolved) = basis.resolve(field.quantity(), binding) {
            check_conditions(
                Some(field),
                IssueTarget::Input(field),
                resolved.source,
                &capture,
                &mut issues,
            );
        }
    }
    if task.case.applicability_source_ids.is_empty() {
        issue(
            &mut issues,
            IssueTarget::Case,
            SizingIssueKind::MissingSourceEvidence,
        );
    }
    for id in &task.case.applicability_source_ids {
        capture_sources(
            vec![(id.clone(), true)],
            IssueTarget::Source(id.clone()),
            basis,
            &mut capture,
            &mut issues,
        );
        if let Some(source) = basis.sources.iter().find(|s| s.id == *id) {
            check_conditions(
                Some(EffectiveK),
                IssueTarget::Source(id.clone()),
                source,
                &capture,
                &mut issues,
            );
        }
    }
    if issues.is_empty() {
        Ok((capture, height_issues))
    } else {
        Err(SizingFailure { issues })
    }
}

fn check_applicability(a: &SeparatorApplicability, issues: &mut Vec<SizingIssue>) {
    fn require<T: PartialEq>(
        actual: Option<T>,
        expected: T,
        field: ApplicabilityField,
        issues: &mut Vec<SizingIssue>,
    ) {
        let kind = match actual {
            None => Some(SizingIssueKind::MissingInput),
            Some(value) if value != expected => Some(SizingIssueKind::OutsideMethodScope),
            _ => None,
        };
        if let Some(kind) = kind {
            issue(issues, IssueTarget::Applicability(field), kind);
        }
    }
    require(
        a.orientation,
        SeparatorOrientation::Vertical,
        ApplicabilityField::Orientation,
        issues,
    );
    require(
        a.gas_direction,
        SeparatorGasDirection::Upward,
        ApplicabilityField::GasDirection,
        issues,
    );
    require(
        a.service,
        SeparatorService::CleanNonFoamingTwoPhase,
        ApplicabilityField::Service,
        issues,
    );
    require(
        a.geometry,
        SeparatorSectionGeometry::FullEqualDiameterCylinder,
        ApplicabilityField::Geometry,
        issues,
    );
    require(
        a.k_basis,
        SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals,
        ApplicabilityField::KBasis,
        issues,
    );
    require(
        a.flow_basis,
        SeparatorFlowBasis::ActualAtOperatingConditions,
        ApplicabilityField::FlowBasis,
        issues,
    );
    if a.internals_description.trim().is_empty() {
        issue(
            issues,
            IssueTarget::Applicability(ApplicabilityField::Internals),
            SizingIssueKind::MissingInput,
        );
    }
}

fn resolve(
    binding: &InputBinding,
    quantity: QuantityKind,
    target: IssueTarget,
    basis: &EngineeringBasis,
    capture: &mut SizingInputCapture,
    issues: &mut Vec<SizingIssue>,
) -> Option<f64> {
    match basis.resolve(quantity, binding) {
        Err(error) => {
            issue(issues, target, SizingIssueKind::InvalidBinding(error));
            None
        }
        Ok(resolved) => {
            let mut sources = vec![(resolved.source.id.clone(), true)];
            if let BindingOrigin::Basis(entry) | BindingOrigin::Override { basis: entry, .. } =
                resolved.origin
            {
                capture
                    .basis_entries
                    .insert(entry.id.clone(), entry.clone());
                // An override's original source is traceability, not its active evidence.
                sources.push((entry.source_id.clone(), false));
            }
            capture_sources(sources, target, basis, capture, issues);
            Some(resolved.value.value_si)
        }
    }
}

fn capture_sources(
    mut sources: Vec<(SourceId, bool)>,
    target: IssueTarget,
    basis: &EngineeringBasis,
    capture: &mut SizingInputCapture,
    issues: &mut Vec<SizingIssue>,
) {
    let mut checked = std::collections::BTreeSet::new();
    while let Some((id, active)) = sources.pop() {
        if !checked.insert((id.clone(), active)) {
            continue;
        }
        let Some(source) = basis.sources.iter().find(|s| s.id == id) else {
            issue(issues, target.clone(), SizingIssueKind::MissingSource(id));
            continue;
        };
        if active {
            check_source(source, target.clone(), issues);
        }
        capture.sources.insert(id, source.clone());
        for observation in &source.observations {
            sources.extend(
                observation
                    .depends_on
                    .iter()
                    .map(|r| (r.source_id.clone(), active)),
            );
        }
    }
}

fn check_source(source: &SourceRecord, target: IssueTarget, issues: &mut Vec<SizingIssue>) {
    let has_reference = !source.reference.trim().is_empty() && !source.locator.trim().is_empty();
    let has_assumption = !source.adoption_note.trim().is_empty();
    if source.kind == SourceKind::Unknown
        || source.applicability.scope.trim().is_empty()
        || (source.kind == SourceKind::Assumed && !has_assumption)
        || (source.kind != SourceKind::Assumed && !has_reference)
    {
        issue(
            issues,
            target.clone(),
            SizingIssueKind::MissingSourceEvidence,
        );
    }
    if source.review == SourceReview::Unreviewed {
        issue(issues, target, SizingIssueKind::SourceNotAdopted);
    }
}

fn check_conditions(
    field: Option<SeparatorInput>,
    target: IssueTarget,
    source: &SourceRecord,
    capture: &SizingInputCapture,
    issues: &mut Vec<SizingIssue>,
) {
    use SeparatorInput::*;
    let a = &source.applicability;
    let phase = match field {
        Some(GasVolumeFlow | GasDensity) => Some(SourcePhase::Gas),
        Some(LiquidVolumeFlow | LiquidDensity) => Some(SourcePhase::Liquid),
        _ => None,
    };
    let needs_fluid = phase.is_some() || field == Some(EffectiveK);
    let fluid_mismatch = (needs_fluid || !a.fluid_system.trim().is_empty())
        && a.fluid_system.trim() != capture.task.case.fluid_system.trim();
    let phase_mismatch = phase.is_some_and(|p| a.phase != p && a.phase != SourcePhase::Both);
    let conditions_mismatch = match a.operating_conditions {
        None => phase.is_some(),
        Some(conditions) => [
            (OperatingTemperature, conditions.temperature_k),
            (OperatingPressure, conditions.pressure_pa),
        ]
        .iter()
        .any(|(field, source_value)| {
            capture
                .values_si
                .get(field)
                .is_some_and(|value| !numerically_equal(*value, *source_value))
        }),
    };
    if fluid_mismatch || phase_mismatch || conditions_mismatch {
        issue(issues, target, SizingIssueKind::SourceConditionMismatch);
    }
}

pub(super) fn numerically_equal(a: f64, b: f64) -> bool {
    (a - b).abs() / a.abs().max(b.abs()) <= COMPARISON_RELATIVE_TOLERANCE
}
