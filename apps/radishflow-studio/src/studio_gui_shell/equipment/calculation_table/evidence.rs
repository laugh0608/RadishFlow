use super::*;
use rf_ui::engineering::editor::separator_sizing::UncheckedRequirement;

pub(super) fn append(
    table: &mut CalculationTable,
    candidate: &SeparatorCandidate,
) -> Result<(), TableError> {
    let capture = candidate.capture();
    records(table, &capture.basis_entries, &capture.sources)?;
    for (field, ids) in [
        ("assumedSource", candidate.assumed_sources()),
        ("pendingReviewSource", candidate.pending_review_sources()),
    ] {
        for (index, id) in ids.iter().enumerate() {
            table.text("limitation", "", &format!("{field}.{index}"), id);
        }
    }
    for section in candidate.missing_axial_sections() {
        table.text(
            "limitation",
            keys::axial(*section),
            "unresolvedAxialSection",
            "Missing or insufficient evidence; excluded from calculated total height.",
        );
    }
    for (index, issue) in candidate.height_issues().iter().enumerate() {
        table.text(
            "heightIssue",
            &index.to_string(),
            "description",
            issue_label(StudioShellLocale::En, issue),
        );
    }
    unchecked(table, candidate.unchecked_requirements());
    Ok(())
}

pub(super) fn records(
    table: &mut CalculationTable,
    basis_entries: &std::collections::BTreeMap<
        rf_types::BasisEntryId,
        rf_model::engineering::BasisEntry,
    >,
    sources: &std::collections::BTreeMap<rf_types::SourceId, rf_model::engineering::SourceRecord>,
) -> Result<(), TableError> {
    for basis in basis_entries.values() {
        let id = basis.id.as_str();
        table.text("basis", id, "name", &basis.name);
        table.text("basis", id, "revision", basis.revision);
        table.text("basis", id, "sourceId", &basis.source_id);
        table.quantity(
            "basis",
            id,
            "value",
            Some(basis.value.value_si),
            basis.value.quantity,
        )?;
    }
    for source in sources.values() {
        let id = source.id.as_str();
        table.text("source", id, "revision", source.revision);
        for (field, value) in [
            ("title", &source.title),
            ("reference", &source.reference),
            ("locator", &source.locator),
            ("publicationVersion", &source.publication_version),
            ("adoptionNote", &source.adoption_note),
            ("fluidSystem", &source.applicability.fluid_system),
            ("scope", &source.applicability.scope),
        ] {
            table.text("source", id, field, value);
        }
        table.text(
            "source",
            id,
            "accessedOn",
            source
                .accessed_on
                .map(|d| d.to_string())
                .unwrap_or_default(),
        );
        table.text(
            "source",
            id,
            "kind",
            match source.kind {
                SourceKind::Measured => "measured",
                SourceKind::Vendor => "vendor",
                SourceKind::Literature => "literature",
                SourceKind::Estimated => "estimated",
                SourceKind::Assumed => "assumed",
                SourceKind::Unknown => "unknown",
            },
        );
        table.text(
            "source",
            id,
            "review",
            match source.review {
                SourceReview::Unreviewed => "unreviewed",
                SourceReview::Adopted => "adopted",
                SourceReview::Reviewed => "reviewed",
            },
        );
        table.text(
            "source",
            id,
            "phase",
            match source.applicability.phase {
                SourcePhase::Gas => "gas",
                SourcePhase::Liquid => "liquid",
                SourcePhase::Both => "both",
                SourcePhase::NotApplicable => "notApplicable",
            },
        );
        let conditions = source.applicability.operating_conditions;
        table.quantity(
            "source",
            id,
            "operatingTemperature",
            conditions.map(|c| c.temperature_k),
            QuantityKind::AbsoluteTemperature,
        )?;
        table.quantity(
            "source",
            id,
            "operatingPressure",
            conditions.map(|c| c.pressure_pa),
            QuantityKind::AbsolutePressure,
        )?;
        for (index, observation) in source.observations.iter().enumerate() {
            // Each observation uses its source as recordId and an indexed field
            // prefix. Explicit IDs/dependencies avoid ambiguous composite IDs.
            let prefix = format!("observation.{index}");
            for (field, value) in [
                ("id", observation.id.to_string()),
                ("originalValue", observation.original_value.to_string()),
                ("originalUnit", observation.original_unit.clone()),
                ("conversionNote", observation.conversion_note.clone()),
            ] {
                table.text("source", id, &format!("{prefix}.{field}"), value);
            }
            table.quantity(
                "source",
                id,
                &format!("{prefix}.canonicalValue"),
                Some(observation.canonical_value),
                observation.quantity,
            )?;
            for (i, dependency) in observation.depends_on.iter().enumerate() {
                table.text(
                    "source",
                    id,
                    &format!("{prefix}.dependency.{i}.sourceId"),
                    &dependency.source_id,
                );
                table.text(
                    "source",
                    id,
                    &format!("{prefix}.dependency.{i}.observationId"),
                    &dependency.observation_id,
                );
            }
        }
    }
    Ok(())
}

pub(super) fn unchecked(table: &mut CalculationTable, requirements: &[UncheckedRequirement]) {
    for requirement in requirements {
        let (key, description) = match requirement {
            UncheckedRequirement::PhaseEquilibrium => (
                "phaseEquilibrium",
                "Phase equilibrium and property validity",
            ),
            UncheckedRequirement::EntrainmentAndEfficiency => (
                "entrainmentAndEfficiency",
                "Entrainment and separation efficiency",
            ),
            UncheckedRequirement::PressureDrop => ("pressureDrop", "Pressure drop"),
            UncheckedRequirement::SlugHandling => ("slugHandling", "Slug handling"),
            UncheckedRequirement::MechanicalDesignAndCertification => (
                "mechanicalDesignAndCertification",
                "Mechanical design and certification",
            ),
            UncheckedRequirement::StandardDiameterRounding => {
                ("standardDiameterRounding", "Standard diameter rounding")
            }
            UncheckedRequirement::MultipleOperatingCases => {
                ("multipleOperatingCases", "Multiple operating cases")
            }
        };
        table.text("unchecked", key, "description", description);
    }
}
