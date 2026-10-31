//! Matrix review output keeps all members and scopes row evidence by explicit case ID.
use super::*;
use rf_model::equipment::SeparatorGeometryInput;
use rf_ui::engineering::{EquipmentResultStatus, RatingMatrixCapture};
use separator_rating::{CONTROL_TIE_TOLERANCE, MatrixCoverage};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatrixRow {
    case_id: Option<String>,
    #[serde(flatten)]
    row: TableRow,
}
#[derive(Serialize)]
struct MatrixTable {
    format: &'static str,
    version: u32,
    rows: Vec<MatrixRow>,
}
pub(in super::super) fn encode(
    capture: &RatingMatrixCapture,
    document: &str,
    units: DisplayUnitSet,
    format: CalculationTableFormat,
) -> Result<String, TableError> {
    let task = capture.task();
    let summary = capture.summary();
    let mut common = CalculationTable {
        format: "radishflow.separator-rating-matrix",
        version: 3,
        rows: vec![],
        units: units.clone(),
    };
    common.text("metadata", "", "format", common.format);
    common.text("metadata", "", "version", common.version);
    common.text("metadata", "", "documentId", document);
    common.text("metadata", "", "taskId", &task.id);
    common.text("metadata", "", "unitScope", "projectAtExportRequest");
    common.text("metadata", "", "displaySignificantDigits", 6);
    common.text("metadata", "", "scope", "Finite listed cases at given geometry and each row's own adopted conditions; no continuous operating envelope, efficiency guarantee or mechanical certification. Only Current rows contribute numerical results.");
    common.text("task", task.id.as_str(), "name", &task.name);
    common.text("task", task.id.as_str(), "tag", &task.tag);
    common.text("task", task.id.as_str(), "methodId", &task.method.id);
    common.text(
        "task",
        task.id.as_str(),
        "methodVersion",
        task.method.version,
    );
    common.text(
        "matrix",
        "",
        "coverage",
        match summary.coverage {
            MatrixCoverage::Complete => "complete",
            MatrixCoverage::Incomplete => "incomplete",
        },
    );
    common.text(
        "matrix",
        "",
        "requiredCount",
        summary.required_case_ids.len(),
    );
    common.text(
        "matrix",
        "",
        "currentCount",
        summary.evaluated_case_ids.len(),
    );
    common.text(
        "matrix",
        "",
        "allCasesWithinCriteria",
        summary.all_cases_within_criteria(),
    );
    common.text(
        "matrix",
        "",
        "subsetConclusion",
        summary
            .subset_conclusion
            .map(rating::constraint_key)
            .unwrap_or("none"),
    );
    common.text(
        "matrix",
        "",
        "controlScope",
        if summary.coverage == MatrixCoverage::Complete {
            "allListedCases"
        } else {
            "completedSubset"
        },
    );
    common.quantity(
        "method",
        &task.method.id,
        "controlTieTolerance",
        Some(CONTROL_TIE_TOLERANCE),
        QuantityKind::DimensionlessRatio,
    )?;
    common.quantity(
        "method",
        &task.method.id,
        "loadBoundaryTolerance",
        Some(separator_rating::LOAD_BOUNDARY_TOLERANCE),
        QuantityKind::DimensionlessRatio,
    )?;
    common.text("method", &task.method.id, "controlTieRule", "abs(unrounded load - maximum unrounded load) <= tolerance; no chained comparison; ties do not change each row's constraint status.");
    for (name, control) in [("gas", &summary.gas), ("liquid", &summary.liquid)] {
        common.quantity(
            "control",
            name,
            "maximumLoadFraction",
            control.as_ref().map(|c| c.maximum_load_fraction),
            QuantityKind::DimensionlessRatio,
        )?;
        if let Some(control) = control {
            for (index, id) in control.case_ids.iter().enumerate() {
                common.text("control", name, &format!("caseId.{index}"), id);
            }
        }
    }
    common.text(
        "geometry",
        task.id.as_str(),
        "revision",
        task.geometry.revision,
    );
    common.text(
        "geometry",
        task.id.as_str(),
        "internalsDescription",
        &task.geometry.applicability.internals_description,
    );
    keys::geometry_declarations(&mut common, task.id.as_str(), &task.geometry.applicability);
    for (index, source) in task.geometry.applicability_source_ids.iter().enumerate() {
        common.text(
            "geometry",
            task.id.as_str(),
            &format!("applicabilitySource.{index}"),
            source,
        );
    }
    for field in SeparatorGeometryInput::ALL {
        let name = match field {
            SeparatorGeometryInput::InternalDiameter => "internalDiameter",
            SeparatorGeometryInput::WorkingLiquidHeight => "workingLiquidHeight",
        };
        let binding = task.geometry.inputs.get(&field);
        let value = binding
            .and_then(|b| capture.basis().resolve(field.quantity(), b).ok())
            .map(|v| v.value.value_si);
        common.quantity(
            "geometryInput",
            name,
            "formalValue",
            value,
            field.quantity(),
        )?;
        if let Some(binding) = binding {
            common.binding("geometryInput", name, binding)?;
        }
    }
    for field in SeparatorAxialSection::ALL {
        let name = keys::axial(field);
        let binding = task.geometry.axial_sections.get(&field);
        let value = binding
            .and_then(|b| capture.basis().resolve(QuantityKind::Length, b).ok())
            .map(|v| v.value.value_si);
        common.quantity(
            "axialSection",
            name,
            "formalValue",
            value,
            QuantityKind::Length,
        )?;
        if let Some(binding) = binding {
            common.binding("axialSection", name, binding)?;
        }
    }
    // Formal source data is scoped to this task; calculated row evidence below
    // retains each execution's immutable capture, including label-only changes.
    evidence::records(
        &mut common,
        &capture
            .basis()
            .basis_entries
            .iter()
            .map(|b| (b.id.clone(), b.clone()))
            .collect(),
        &capture
            .basis()
            .sources
            .iter()
            .map(|s| (s.id.clone(), s.clone()))
            .collect(),
    )?;
    adoption::append(
        &mut common,
        task.id.as_str(),
        &task.geometry,
        capture.basis(),
    )?;
    let mut table = MatrixTable {
        format: common.format,
        version: common.version,
        rows: common
            .rows
            .into_iter()
            .map(|row| MatrixRow { case_id: None, row })
            .collect(),
    };
    for (index, captured) in capture.rows().iter().enumerate() {
        let case_id = captured.member.case_id.as_str();
        let case = &task.cases[index];
        let mut row = CalculationTable {
            format: table.format,
            version: table.version,
            units: units.clone(),
            rows: vec![],
        };
        if captured.result.is_none() {
            row.execution_origin(captured.origin.as_ref());
        }
        row.text("member", case_id, "order", index);
        row.text("member", case_id, "name", &case.name);
        row.text("member", case_id, "purpose", &case.purpose);
        row.text(
            "member",
            case_id,
            "resultStatus",
            match captured.status {
                EquipmentResultStatus::Current => "current",
                EquipmentResultStatus::Missing => "missing",
                EquipmentResultStatus::Failed => "failed",
                EquipmentResultStatus::Stale => "stale",
            },
        );
        row.text(
            "member",
            case_id,
            "reason",
            match captured.status {
                EquipmentResultStatus::Current => "qualifiedCurrentExecution",
                EquipmentResultStatus::Missing => "notCalculatedInThisSession",
                EquipmentResultStatus::Failed => "calculationFailed",
                EquipmentResultStatus::Stale => "executionNoLongerQualified",
            },
        );
        if let Some(stale) = captured.staleness {
            row.text("member", case_id, "inputChanged", stale.input_changed);
            row.text("member", case_id, "evidenceChanged", stale.evidence_changed);
        }
        if let Some(failure) = &captured.failure {
            for (i, issue) in failure.issues.iter().enumerate() {
                row.text(
                    "failure",
                    case_id,
                    &format!("issue.{i}"),
                    super::super::rating_labels::issue_label(StudioShellLocale::En, issue),
                );
            }
        }
        if let Some(result) = &captured.result {
            let details = rating::build(
                result,
                task,
                captured
                    .origin
                    .as_ref()
                    .expect("current result has execution origin"),
                units.clone(),
            )?;
            row.rows.extend(details.rows.into_iter().filter(|r| {
                !r.section.starts_with("adoption")
                    && (r.section != "metadata"
                        || !matches!(r.field.as_str(), "format" | "version"))
            }));
        }
        table.rows.extend(row.rows.into_iter().map(|row| MatrixRow {
            case_id: Some(case_id.into()),
            row,
        }));
    }
    match format {
        CalculationTableFormat::Json => serde_json::to_string_pretty(&table)
            .map(|t| t + "\n")
            .map_err(TableError::Serialization),
        CalculationTableFormat::Csv => {
            let mut text = String::from(
                "caseId,section,recordId,field,status,quantityId,valueSi,siUnitId,displayValue,displayUnitId,displayUnitSymbol,text\r\n",
            );
            for scoped in &table.rows {
                let r = &scoped.row;
                let si = r.value_si.map(|v| v.to_string()).unwrap_or_default();
                let cells = [
                    scoped.case_id.as_deref().unwrap_or_default(),
                    r.section,
                    &r.record_id,
                    &r.field,
                    r.status,
                    r.quantity_id.unwrap_or_default(),
                    &si,
                    r.si_unit_id.unwrap_or_default(),
                    r.display_value.as_deref().unwrap_or_default(),
                    r.display_unit_id.unwrap_or_default(),
                    r.display_unit_symbol.unwrap_or_default(),
                    r.text.as_deref().unwrap_or_default(),
                ];
                text.push_str(
                    &cells
                        .iter()
                        .enumerate()
                        .map(|(i, c)| csv_cell(c, i != 6 && i != 8))
                        .collect::<Vec<_>>()
                        .join(","),
                );
                text.push_str("\r\n");
            }
            Ok(text)
        }
    }
}
