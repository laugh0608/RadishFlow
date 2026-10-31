//! Fixed adoption evidence has its own sections, independent of mutable source records.
use super::*;
use rf_model::{engineering::EngineeringBasis, equipment::*};

pub(super) fn append(
    table: &mut CalculationTable,
    task_id: &str,
    geometry: &SeparatorGivenGeometry,
    current_basis: &EngineeringBasis,
) -> Result<(), TableError> {
    let Some(adoption) = &geometry.adoption else {
        return Ok(());
    };
    let source = &adoption.source_task;
    for (field, value) in [
        ("sourceDocumentId", adoption.source_document_id.as_str()),
        ("sourceTaskId", source.id.as_str()),
        ("sourceCaseId", source.case.id.as_str()),
        ("sourceTag", source.tag.as_str()),
        ("sourceName", source.name.as_str()),
        ("sourceCaseName", source.case.name.as_str()),
        ("sourceCasePurpose", source.case.purpose.as_str()),
        ("sourceFluidSystem", source.case.fluid_system.as_str()),
        ("sourceMethodId", source.method.id.as_str()),
        ("geometryOrigin", "designAdoptedFromPreliminarySizing"),
        (
            "evidenceScope",
            "fixedAtAdoption; not a current sizing execution or measured geometry",
        ),
        (
            "internalsDescription",
            source.case.applicability.internals_description.as_str(),
        ),
    ] {
        table.text("adoption", task_id, field, value);
    }
    table.text("adoption", task_id, "sourceRevision", source.revision);
    table.text(
        "adoption",
        task_id,
        "sourceMethodVersion",
        source.method.version,
    );
    for (i, id) in source.case.applicability_source_ids.iter().enumerate() {
        table.text("adoption", task_id, &format!("applicabilitySource.{i}"), id);
    }
    for (name, field, dimension) in [
        (
            "internalDiameter",
            SeparatorGeometryInput::InternalDiameter,
            &adoption.diameter,
        ),
        (
            "workingLiquidHeight",
            SeparatorGeometryInput::WorkingLiquidHeight,
            &adoption.working_liquid_height,
        ),
    ] {
        let current = geometry
            .inputs
            .get(&field)
            .and_then(|b| current_basis.resolve(QuantityKind::Length, b).ok())
            .map(|v| v.value.value_si);
        table.quantity(
            "adoptionDimension",
            name,
            "originalValue",
            Some(dimension.original_m),
            QuantityKind::Length,
        )?;
        table.quantity(
            "adoptionDimension",
            name,
            "adoptedValue",
            Some(dimension.adopted_m),
            QuantityKind::Length,
        )?;
        table.text(
            "adoptionDimension",
            name,
            "adjustmentReason",
            &dimension.reason,
        );
        comparison(
            table,
            "adoptionDimension",
            name,
            current,
            Some(dimension.adopted_m),
        )?;
    }
    for field in SeparatorAxialSection::ALL {
        let name = keys::axial(field);
        let adopted = adoption.axial_lengths_m.get(&field).copied();
        let current = geometry
            .axial_sections
            .get(&field)
            .and_then(|b| current_basis.resolve(QuantityKind::Length, b).ok())
            .map(|v| v.value.value_si);
        table.quantity(
            "adoptionAxialSection",
            name,
            "adoptedValue",
            adopted,
            QuantityKind::Length,
        )?;
        comparison(table, "adoptionAxialSection", name, current, adopted)?;
        if let Some(binding) = source.axial_sections.get(&field) {
            table.binding("adoptionOriginalAxialInput", name, binding)?;
        }
    }
    for field in SeparatorInput::ALL {
        let name = keys::input(field);
        if let Some(binding) = source.case.inputs.get(&field) {
            table.binding("adoptionInput", name, binding)?;
            let value = adoption
                .source_basis
                .resolve(field.quantity(), binding)
                .ok()
                .map(|r| r.value.value_si);
            table.quantity(
                "adoptionInput",
                name,
                "effectiveValue",
                value,
                field.quantity(),
            )?;
        }
    }
    let start = table.rows.len();
    keys::applicability(table, &source.case);
    evidence::records(
        table,
        &adoption
            .source_basis
            .basis_entries
            .iter()
            .map(|b| (b.id.clone(), b.clone()))
            .collect(),
        &adoption
            .source_basis
            .sources
            .iter()
            .map(|s| (s.id.clone(), s.clone()))
            .collect(),
    )?;
    // The frozen IDs may also exist in the current project with different values.
    for row in &mut table.rows[start..] {
        row.section = match row.section {
            "basis" => "adoptionBasis",
            "source" => "adoptionSource",
            "applicability" => "adoptionApplicability",
            _ => unreachable!("only fixed basis, source and declaration rows"),
        };
    }
    Ok(())
}

fn comparison(
    table: &mut CalculationTable,
    section: &'static str,
    name: &str,
    current: Option<f64>,
    adopted: Option<f64>,
) -> Result<(), TableError> {
    table.quantity(section, name, "currentValue", current, QuantityKind::Length)?;
    table.text(
        section,
        name,
        "comparison",
        match (current, adopted) {
            (Some(c), Some(a)) if c == a => "unchanged",
            (Some(_), Some(_)) => "changed",
            (Some(_), None) => "addedSinceAdoption",
            (None, Some(_)) => "currentUnknown",
            (None, None) => "bothUnknown",
        },
    );
    Ok(())
}
