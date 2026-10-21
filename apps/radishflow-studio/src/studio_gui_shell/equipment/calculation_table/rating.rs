//! Single-case rating output. Geometry, constraints and evidence share one capture.
use super::*;
use rf_model::equipment::{SeparatorGeometryInput, SeparatorRatingInput, SeparatorRatingTask};
use rf_ui::engineering::editor::separator_rating::{ConstraintStatus, SeparatorRatingResult};

pub(in super::super) fn encode(
    result: &SeparatorRatingResult,
    current_task: &SeparatorRatingTask,
    document_id: &str,
    generation: ResultGeneration,
    units: DisplayUnitSet,
    format: CalculationTableFormat,
) -> Result<String, TableError> {
    let capture = result.capture();
    let task = &capture.task;
    let case = task.cases.first().expect("successful single-case rating");
    let current_case = current_task
        .cases
        .iter()
        .find(|c| c.id == case.id)
        .expect("qualified current rating case");
    let id = task.id.as_str();
    let case_id = case.id.as_str();
    let mut table = CalculationTable {
        format: "radishflow.separator-rating-table",
        version: 1,
        rows: Vec::new(),
        units,
    };
    table.text("metadata", "", "format", table.format);
    table.text("metadata", "", "version", table.version);
    table.text("metadata", "", "documentId", document_id);
    table.text("metadata", "", "taskId", id);
    table.text("metadata", "", "caseId", case_id);
    table.text("metadata", "", "resultStatusAtCapture", "current");
    table.text("metadata", "", "inputGeneration", generation.input);
    table.text("metadata", "", "evidenceGeneration", generation.evidence);
    table.text("metadata", "", "unitScope", "projectAtExportRequest");
    table.text("metadata", "", "displaySignificantDigits", "6");
    table.text("metadata", "", "scope", "Conditional checks of two selected criteria against given geometry; no operating envelope, separation efficiency or mechanical certification.");
    for (key, value) in [
        ("tag", &current_task.tag),
        ("name", &current_task.name),
        ("capturedTag", &task.tag),
        ("capturedName", &task.name),
        ("methodId", &task.method.id),
    ] {
        table.text("task", id, key, value);
    }
    table.text("task", id, "revision", current_task.revision);
    table.text("task", id, "capturedRevision", task.revision);
    table.text("task", id, "methodVersion", task.method.version);
    table.text("geometry", id, "revision", task.geometry.revision);
    table.text(
        "geometry",
        id,
        "internalsDescription",
        &task.geometry.applicability.internals_description,
    );
    for (index, source) in task.geometry.applicability_source_ids.iter().enumerate() {
        table.text(
            "geometry",
            id,
            &format!("applicabilitySource.{index}"),
            source,
        );
    }
    table.text("case", case_id, "revision", current_case.revision);
    table.text("case", case_id, "capturedRevision", case.revision);
    for (key, value) in [
        ("name", &current_case.name),
        ("capturedName", &case.name),
        ("purpose", &case.purpose),
        ("fluidSystem", &case.fluid_system),
    ] {
        table.text("case", case_id, key, value);
    }
    for (index, source) in case.applicability_source_ids.iter().enumerate() {
        table.text(
            "case",
            case_id,
            &format!("applicabilitySource.{index}"),
            source,
        );
    }
    keys::declarations(
        &mut table,
        id,
        &task.geometry.applicability,
        case_id,
        &case.applicability,
    );
    for field in SeparatorGeometryInput::ALL {
        let key = match field {
            SeparatorGeometryInput::InternalDiameter => "internalDiameter",
            SeparatorGeometryInput::WorkingLiquidHeight => "workingLiquidHeight",
        };
        table.quantity(
            "geometryInput",
            key,
            "effectiveValue",
            capture.geometry_values_m.get(&field).copied(),
            field.quantity(),
        )?;
        if let Some(binding) = task.geometry.inputs.get(&field) {
            table.binding("geometryInput", key, binding)?;
        }
    }
    for field in SeparatorRatingInput::ALL {
        let key = input_key(field);
        table.quantity(
            "input",
            key,
            "effectiveValue",
            capture.values_si.get(&field).copied(),
            field.quantity(),
        )?;
        if let Some(binding) = case.inputs.get(&field) {
            table.binding("input", key, binding)?;
        }
    }
    for field in SeparatorAxialSection::ALL {
        let key = keys::axial(field);
        table.quantity(
            "axialSection",
            key,
            "effectiveValue",
            capture.axial_lengths_m.get(&field).copied(),
            QuantityKind::Length,
        )?;
        if let Some(binding) = task.geometry.axial_sections.get(&field) {
            table.binding("axialSection", key, binding)?;
        }
    }
    let p = result.performance();
    use QuantityKind::*;
    for (field, value, quantity) in [
        ("givenArea", Some(p.given_area_m2), Area),
        (
            "availableWorkingVolume",
            Some(p.available_working_volume_m3),
            Volume,
        ),
        (
            "requiredWorkingVolume",
            Some(p.required_working_volume_m3),
            Volume,
        ),
        (
            "soudersBrownVelocity",
            Some(p.souders_brown_velocity_m_s),
            Velocity,
        ),
        (
            "allowedGasVelocity",
            Some(p.allowed_gas_velocity_m_s),
            Velocity,
        ),
        (
            "actualGasVelocity",
            Some(p.actual_gas_velocity_m_s),
            Velocity,
        ),
        (
            "gasFlowLimit",
            Some(p.gas_flow_limit_m3_s),
            ActualVolumeFlow,
        ),
        (
            "liquidFlowLimit",
            Some(p.liquid_flow_limit_m3_s),
            ActualVolumeFlow,
        ),
        (
            "availableWorkingTime",
            Some(p.available_working_time_s),
            Duration,
        ),
        (
            "gasLoadFraction",
            Some(p.gas.load_fraction),
            DimensionlessRatio,
        ),
        (
            "gasMarginFraction",
            Some(p.gas.margin_fraction),
            DimensionlessRatio,
        ),
        (
            "liquidLoadFraction",
            Some(p.liquid.load_fraction),
            DimensionlessRatio,
        ),
        (
            "liquidMarginFraction",
            Some(p.liquid.margin_fraction),
            DimensionlessRatio,
        ),
        (
            "tangentToTangentHeight",
            p.tangent_to_tangent_height_m,
            Length,
        ),
        ("bufferVolume", p.buffer_volume_m3, Volume),
    ] {
        table.quantity("result", case_id, field, value, quantity)?;
    }
    for (field, status) in [
        ("conclusion", p.conclusion()),
        ("gasConstraint", p.gas.status),
        ("liquidConstraint", p.liquid.status),
    ] {
        table.text("result", case_id, field, constraint_key(status));
    }
    table.quantity(
        "method",
        &task.method.id,
        "loadBoundaryTolerance",
        Some(rf_ui::engineering::editor::separator_rating::LOAD_BOUNDARY_TOLERANCE),
        DimensionlessRatio,
    )?;
    for (key, value) in [
        (
            "geometry",
            "A = pi * D_given^2 / 4; V_available = A * h_work_given",
        ),
        (
            "velocity",
            "uSB = K * sqrt((rhoL - rhoG) / rhoG); uallow = f * uSB",
        ),
        (
            "gas",
            "Qg_limit = A * uallow; load_g = Qg / Qg_limit; margin_g = 1 - load_g",
        ),
        (
            "liquid",
            "V_required = Ql * t_required; t_available = V_available / Ql; Ql_limit = V_available / t_required; load_l = V_required / V_available; margin_l = 1 - load_l",
        ),
        (
            "height",
            "H_TT = h_work_given + sum(explicit axial sections); excludes heads and skirt",
        ),
        ("workingVolumeInterval", "LLL -> NLL"),
        (
            "bufferVolumeInterval",
            "NLL -> HLL; bufferVolume = A * (NLL -> HLL height)",
        ),
        (
            "conditionalLimits",
            "Flow limits hold only with the captured properties, K, f and required time unchanged; not a maximum operating envelope.",
        ),
        (
            "boundaryComparison",
            "abs(load - 1) <= tolerance: boundaryReview; otherwise > 1: exceeded, < 1: withinCriteria. Unrounded load decides; tolerance is numerical, not engineering uncertainty.",
        ),
    ] {
        table.text("method", &task.method.id, key, value);
    }
    evidence::records(&mut table, &capture.basis_entries, &capture.sources)?;
    for (field, sources) in [
        ("assumedSource", result.assumed_sources()),
        ("pendingReviewSource", result.pending_review_sources()),
    ] {
        for (index, source) in sources.iter().enumerate() {
            table.text("limitation", "", &format!("{field}.{index}"), source);
        }
    }
    for section in result.missing_axial_sections() {
        table.text(
            "limitation",
            keys::axial(*section),
            "unresolvedAxialSection",
            "Missing or insufficient evidence; excluded from calculated total height.",
        );
    }
    for (index, issue) in result.height_issues().iter().enumerate() {
        table.text(
            "heightIssue",
            &index.to_string(),
            "description",
            super::super::rating_labels::issue_label(StudioShellLocale::En, issue),
        );
    }
    evidence::unchecked(&mut table, result.unchecked_requirements());
    table.encode(format)
}

fn constraint_key(status: ConstraintStatus) -> &'static str {
    match status {
        ConstraintStatus::WithinCriteria => "withinCriteria",
        ConstraintStatus::BoundaryReview => "boundaryReview",
        ConstraintStatus::Exceeded => "exceeded",
    }
}
fn input_key(input: SeparatorRatingInput) -> &'static str {
    use SeparatorRatingInput::*;
    match input {
        OperatingTemperature => "operatingTemperature",
        OperatingPressure => "operatingPressure",
        GasVolumeFlow => "gasActualVolumeFlow",
        LiquidVolumeFlow => "liquidActualVolumeFlow",
        GasDensity => "gasMassDensity",
        LiquidDensity => "liquidMassDensity",
        EffectiveK => "effectiveK",
        GasVelocityUtilization => "gasUtilization",
        RequiredWorkingTime => "requiredWorkingTime",
    }
}
