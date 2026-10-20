//! Read-only equipment calculation table. Both encodings consume the same rows.
use super::*;
use rf_model::engineering::{InputBinding, SourceKind, SourcePhase, SourceReview};
use rf_model::equipment::{SeparatorAxialSection, SeparatorInput, SeparatorSizingTask};
use rf_types::units::ConversionError;
use rf_ui::engineering::ResultGeneration;
use rf_ui::engineering::editor::separator_sizing::{ControllingConstraint, SeparatorCandidate};
use serde::Serialize;

mod evidence;
mod keys;

#[derive(Debug, Clone, Copy)]
pub(in crate::studio_gui_shell) enum CalculationTableFormat {
    Csv,
    Json,
}
impl CalculationTableFormat {
    pub(in crate::studio_gui_shell) fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
        }
    }
}

#[derive(Debug)]
pub(super) enum TableError {
    Conversion {
        field: String,
        cause: ConversionError,
    },
    Serialization(serde_json::Error),
}
impl std::fmt::Display for TableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conversion { field, cause } => write!(f, "{field}: {cause}"),
            Self::Serialization(cause) => write!(f, "{cause}"),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TableRow {
    section: &'static str,
    record_id: String,
    field: String,
    status: &'static str,
    quantity_id: Option<&'static str>,
    value_si: Option<f64>,
    si_unit_id: Option<&'static str>,
    display_value: Option<String>,
    display_unit_id: Option<&'static str>,
    display_unit_symbol: Option<&'static str>,
    text: Option<String>,
}
impl TableRow {
    fn new(section: &'static str, id: &str, field: &str) -> Self {
        Self {
            section,
            record_id: id.into(),
            field: field.into(),
            status: "known",
            quantity_id: None,
            value_si: None,
            si_unit_id: None,
            display_value: None,
            display_unit_id: None,
            display_unit_symbol: None,
            text: None,
        }
    }
}
#[derive(Serialize)]
struct CalculationTable {
    format: &'static str,
    version: u32,
    rows: Vec<TableRow>,
    #[serde(skip)]
    units: DisplayUnitSet,
}
impl CalculationTable {
    fn text(&mut self, section: &'static str, id: &str, field: &str, text: impl ToString) {
        let mut row = TableRow::new(section, id, field);
        row.text = Some(text.to_string());
        self.rows.push(row);
    }
    fn quantity(
        &mut self,
        section: &'static str,
        id: &str,
        field: &str,
        value: Option<f64>,
        quantity: QuantityKind,
    ) -> Result<(), TableError> {
        let mut row = TableRow::new(section, id, field);
        let canonical = quantity.definition().canonical_unit.definition();
        let unit = self.units.unit_for(quantity);
        row.quantity_id = Some(quantity.definition().id);
        row.si_unit_id = Some(canonical.id);
        row.display_unit_id = Some(unit.definition().id);
        row.display_unit_symbol = Some(unit.definition().symbol);
        row.value_si = value;
        row.status = if value.is_some() { "known" } else { "unknown" };
        row.display_value = value
            .map(|value| {
                let converted = rf_types::units::from_canonical(value, quantity, unit)?;
                rf_ui::format_display_number(converted)
            })
            .transpose()
            .map_err(|cause| TableError::Conversion {
                field: format!("{section}/{id}/{field}"),
                cause,
            })?;
        self.rows.push(row);
        Ok(())
    }
    fn binding(
        &mut self,
        section: &'static str,
        id: &str,
        binding: &InputBinding,
    ) -> Result<(), TableError> {
        match binding {
            InputBinding::Direct { value, source_id } => {
                self.text(section, id, "bindingKind", "direct");
                self.text(section, id, "sourceId", source_id);
                self.quantity(
                    section,
                    id,
                    "boundValue",
                    Some(value.value_si),
                    value.quantity,
                )?;
            }
            InputBinding::BasisRef { basis_entry_id } => {
                self.text(section, id, "bindingKind", "basisRef");
                self.text(section, id, "basisEntryId", basis_entry_id);
            }
            InputBinding::Override {
                basis_entry_id,
                value,
                source_id,
                reason,
            } => {
                self.text(section, id, "bindingKind", "override");
                self.text(section, id, "basisEntryId", basis_entry_id);
                self.text(section, id, "sourceId", source_id);
                self.text(section, id, "overrideReason", reason);
                self.quantity(
                    section,
                    id,
                    "boundValue",
                    Some(value.value_si),
                    value.quantity,
                )?;
            }
        }
        Ok(())
    }
    fn encode(&self, format: CalculationTableFormat) -> Result<String, TableError> {
        match format {
            CalculationTableFormat::Json => serde_json::to_string_pretty(self)
                .map(|text| text + "\n")
                .map_err(TableError::Serialization),
            CalculationTableFormat::Csv => {
                let mut csv = String::from(
                    "section,recordId,field,status,quantityId,valueSi,siUnitId,displayValue,displayUnitId,displayUnitSymbol,text\r\n",
                );
                for row in &self.rows {
                    let number = row.value_si.map(|v| v.to_string()).unwrap_or_default();
                    let fields = [
                        row.section,
                        &row.record_id,
                        &row.field,
                        row.status,
                        row.quantity_id.unwrap_or_default(),
                        &number,
                        row.si_unit_id.unwrap_or_default(),
                        row.display_value.as_deref().unwrap_or_default(),
                        row.display_unit_id.unwrap_or_default(),
                        row.display_unit_symbol.unwrap_or_default(),
                        row.text.as_deref().unwrap_or_default(),
                    ];
                    for (index, field) in fields.iter().enumerate() {
                        if index > 0 {
                            csv.push(',');
                        }
                        // Numerical columns are generated from finite values. Other
                        // cells are text, including user-provided titles and locators.
                        let text_cell = index != 5 && index != 7;
                        csv.push_str(&csv_cell(field, text_cell));
                    }
                    csv.push_str("\r\n");
                }
                Ok(csv)
            }
        }
    }
}
fn csv_cell(value: &str, text_cell: bool) -> String {
    // Quoting alone does not prevent spreadsheet formula interpretation. Escape
    // leading apostrophes too, making this text-only prefix reversible.
    let prefix = text_cell
        && (value.starts_with('\'')
            || value
                .trim_start_matches([' ', '\t', '\r', '\n', '\u{feff}'])
                .starts_with(['=', '+', '-', '@'])
            || value.starts_with(['\t', '\r', '\n']));
    format!(
        "\"{}{}\"",
        if prefix { "'" } else { "" },
        value.replace('"', "\"\"")
    )
}

pub(super) fn encode(
    candidate: &SeparatorCandidate,
    current_task: &SeparatorSizingTask,
    document_id: &str,
    generation: ResultGeneration,
    units: DisplayUnitSet,
    format: CalculationTableFormat,
) -> Result<String, TableError> {
    let capture = candidate.capture();
    let task = &capture.task;
    let mut table = CalculationTable {
        format: "radishflow.separator-calculation-table",
        version: 1,
        rows: Vec::new(),
        units,
    };
    table.text("metadata", "", "format", table.format);
    table.text("metadata", "", "version", table.version);
    table.text("metadata", "", "documentId", document_id);
    table.text("metadata", "", "resultStatusAtCapture", "current");
    table.text("metadata", "", "inputGeneration", generation.input);
    table.text("metadata", "", "evidenceGeneration", generation.evidence);
    table.text("metadata", "", "unitScope", "projectAtExportRequest");
    table.text("metadata", "", "displaySignificantDigits", "6");
    table.text("metadata", "", "scope", "Conditional preliminary process dimensions; no mechanical certification, standard diameter rounding or separation efficiency guarantee.");
    let id = task.id.as_str();
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
    table.text("task", id, "caseId", &task.case.id);
    for (key, value) in [
        ("name", &current_task.case.name),
        ("capturedName", &task.case.name),
        ("purpose", &task.case.purpose),
        ("fluidSystem", &task.case.fluid_system),
        (
            "internalsDescription",
            &task.case.applicability.internals_description,
        ),
    ] {
        table.text("case", task.case.id.as_str(), key, value);
    }
    keys::applicability(&mut table, &task.case);
    for (i, source) in task.case.applicability_source_ids.iter().enumerate() {
        table.text(
            "case",
            task.case.id.as_str(),
            &format!("applicabilitySource.{i}"),
            source,
        );
    }
    for field in SeparatorInput::ALL {
        let key = keys::input(field);
        table.quantity(
            "input",
            key,
            "effectiveValue",
            capture.values_si.get(&field).copied(),
            field.quantity(),
        )?;
        if let Some(binding) = task.case.inputs.get(&field) {
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
        if let Some(binding) = task.axial_sections.get(&field) {
            table.binding("axialSection", key, binding)?;
        }
    }
    let d = candidate.dimensions();
    use QuantityKind::*;
    for (field, value, quantity) in [
        ("minimumDiameter", Some(d.minimum_diameter_m), Length),
        ("workingVolume", Some(d.working_volume_m3), Volume),
        (
            "workingLiquidHeight",
            Some(d.working_liquid_height_m),
            Length,
        ),
        (
            "soudersBrownVelocity",
            Some(d.souders_brown_velocity_m_s),
            Velocity,
        ),
        (
            "allowedGasVelocity",
            Some(d.allowed_gas_velocity_m_s),
            Velocity,
        ),
        ("gasArea", Some(d.gas_area_m2), Area),
        ("liquidArea", Some(d.liquid_area_m2), Area),
        ("requiredArea", Some(d.required_area_m2), Area),
        (
            "gasLoadFraction",
            Some(d.gas_load_fraction),
            DimensionlessRatio,
        ),
        (
            "tangentToTangentHeight",
            d.tangent_to_tangent_height_m,
            Length,
        ),
        ("bufferVolume", d.buffer_volume_m3, Volume),
    ] {
        table.quantity("result", id, field, value, quantity)?;
    }
    table.text(
        "result",
        id,
        "controllingConstraint",
        match d.controlling_constraint {
            ControllingConstraint::GasArea => "gasArea",
            ControllingConstraint::LiquidWorkingVolume => "liquidWorkingVolume",
            ControllingConstraint::Both => "both",
        },
    );
    for (key, text) in [
        (
            "velocity",
            "uSB = K * sqrt((rhoL - rhoG) / rhoG); uallow = f * uSB",
        ),
        (
            "area",
            "Ag = Qg / uallow; Vw = QL * tw; AL = Vw / hw,max; A = max(Ag, AL)",
        ),
        (
            "geometry",
            "Dmin = sqrt(4 * A / pi); hw = Vw / A; gasLoad = Qg / A / uallow",
        ),
        (
            "height",
            "H_TT = hw + sum(explicit axial sections); excludes heads and skirt",
        ),
        ("workingVolumeInterval", "LLL -> NLL"),
        (
            "bufferVolumeInterval",
            "NLL -> HLL; bufferVolume = A * (NLL -> HLL height)",
        ),
    ] {
        table.text("method", &task.method.id, key, text);
    }
    evidence::append(&mut table, candidate)?;
    table.encode(format)
}
