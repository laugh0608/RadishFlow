//! Editable text and units stay outside formal equipment records.
use super::*;
use crate::{InputUnitOrigin, NumericParseOutcome, parse_numeric_input};
use rf_model::engineering::*;
use rf_model::equipment::*;
use rf_types::{
    SourceObservationId,
    units::{DisplayUnitSet, MeasurementUnit, QuantityKind, from_canonical},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantityDraft {
    text: String,
    unit: MeasurementUnit,
    // Preserve the exact committed/converted SI value until the user edits text.
    exact_si: Option<u64>,
}
impl QuantityDraft {
    pub fn empty(unit: MeasurementUnit) -> Self {
        Self {
            text: String::new(),
            unit,
            exact_si: None,
        }
    }
    pub fn from_si(value: f64, quantity: QuantityKind, unit: MeasurementUnit) -> Self {
        Self {
            text: from_canonical(value, quantity, unit)
                .expect("compatible project units")
                .to_string(),
            unit,
            exact_si: Some(value.to_bits()),
        }
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn unit(&self) -> MeasurementUnit {
        self.unit
    }
    pub fn set_text(&mut self, text: String) {
        if self.text != text {
            self.text = text;
            self.exact_si = None;
        }
    }
    pub fn value(&self, quantity: QuantityKind) -> Result<f64, EquipmentEditError> {
        if let Some(bits) = self.exact_si {
            return Ok(f64::from_bits(bits));
        }
        match parse_numeric_input(&self.text, quantity, self.unit, InputUnitOrigin::Explicit) {
            NumericParseOutcome::Value { si, .. } => Ok(si),
            NumericParseOutcome::Incomplete => Err(EquipmentEditError::IncompleteNumber),
            NumericParseOutcome::Invalid(error) => Err(EquipmentEditError::Number(error)),
        }
    }
    /// Invalid/incomplete nonempty text is preserved, never reinterpreted in another unit.
    pub fn select_unit(
        &mut self,
        quantity: QuantityKind,
        unit: MeasurementUnit,
    ) -> Result<(), EquipmentEditError> {
        if self.unit == unit {
            return Ok(());
        }
        let value = if self.text.trim().is_empty() {
            None
        } else {
            Some(self.value(quantity)?)
        };
        let text = value
            .map(|v| from_canonical(v, quantity, unit))
            .transpose()
            .map_err(EquipmentEditError::Conversion)?;
        // Check compatibility even for an empty editor.
        from_canonical(0.0, quantity, unit).map_err(EquipmentEditError::Conversion)?;
        self.text = text.map_or_else(String::new, |v| v.to_string());
        self.unit = unit;
        self.exact_si = value.map(f64::to_bits);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingMode {
    Missing,
    Direct,
    BasisRef,
    Override,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingDraft {
    pub mode: BindingMode,
    pub value: QuantityDraft,
    pub source: Option<SourceId>,
    pub basis: Option<BasisEntryId>,
    pub reason: String,
}
impl BindingDraft {
    fn from_binding(
        binding: Option<&InputBinding>,
        quantity: QuantityKind,
        units: &DisplayUnitSet,
    ) -> Self {
        let mut draft = Self {
            mode: BindingMode::Missing,
            value: QuantityDraft::empty(units.unit_for(quantity)),
            source: None,
            basis: None,
            reason: String::new(),
        };
        match binding {
            Some(InputBinding::Direct { value, source_id }) => {
                draft.mode = BindingMode::Direct;
                draft.value =
                    QuantityDraft::from_si(value.value_si, quantity, units.unit_for(quantity));
                draft.source = Some(source_id.clone());
            }
            Some(InputBinding::BasisRef { basis_entry_id }) => {
                draft.mode = BindingMode::BasisRef;
                draft.basis = Some(basis_entry_id.clone());
            }
            Some(InputBinding::Override {
                basis_entry_id,
                value,
                source_id,
                reason,
            }) => {
                draft.mode = BindingMode::Override;
                draft.value =
                    QuantityDraft::from_si(value.value_si, quantity, units.unit_for(quantity));
                draft.source = Some(source_id.clone());
                draft.basis = Some(basis_entry_id.clone());
                draft.reason = reason.clone();
            }
            None => {}
        }
        draft
    }
    pub fn binding(
        &self,
        quantity: QuantityKind,
    ) -> Result<Option<InputBinding>, EquipmentEditError> {
        Ok(Some(match self.mode {
            BindingMode::Missing => return Ok(None),
            BindingMode::BasisRef => InputBinding::BasisRef {
                basis_entry_id: self.basis.clone().ok_or(EquipmentEditError::SelectBasis)?,
            },
            BindingMode::Direct => InputBinding::Direct {
                value: EngineeringValue {
                    quantity,
                    value_si: self.value.value(quantity)?,
                },
                source_id: self
                    .source
                    .clone()
                    .ok_or(EquipmentEditError::SelectSource)?,
            },
            BindingMode::Override => {
                if self.reason.trim().is_empty() {
                    return Err(EquipmentEditError::OverrideReason);
                }
                InputBinding::Override {
                    basis_entry_id: self.basis.clone().ok_or(EquipmentEditError::SelectBasis)?,
                    value: EngineeringValue {
                        quantity,
                        value_si: self.value.value(quantity)?,
                    },
                    source_id: self
                        .source
                        .clone()
                        .ok_or(EquipmentEditError::SelectSource)?,
                    reason: self.reason.clone(),
                }
            }
        }))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EquipmentField {
    Input(SeparatorInput),
    Axial(SeparatorAxialSection),
}
impl EquipmentField {
    pub fn quantity(self) -> QuantityKind {
        match self {
            Self::Input(input) => input.quantity(),
            Self::Axial(_) => QuantityKind::Length,
        }
    }
    pub fn all() -> impl Iterator<Item = Self> {
        SeparatorInput::ALL
            .into_iter()
            .map(Self::Input)
            .chain(SeparatorAxialSection::ALL.into_iter().map(Self::Axial))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeparatorTaskDraft {
    pub tag: String,
    pub name: String,
    pub case_name: String,
    pub purpose: String,
    pub fluid_system: String,
    pub applicability: SeparatorApplicability,
    pub applicability_sources: Vec<SourceId>,
    pub fields: BTreeMap<EquipmentField, BindingDraft>,
}
impl SeparatorTaskDraft {
    pub(super) fn from_record(task: &SeparatorSizingTask, units: &DisplayUnitSet) -> Self {
        Self {
            tag: task.tag.clone(),
            name: task.name.clone(),
            case_name: task.case.name.clone(),
            purpose: task.case.purpose.clone(),
            fluid_system: task.case.fluid_system.clone(),
            applicability: task.case.applicability.clone(),
            applicability_sources: task.case.applicability_source_ids.clone(),
            fields: EquipmentField::all()
                .map(|field| {
                    let binding = match field {
                        EquipmentField::Input(i) => task.case.inputs.get(&i),
                        EquipmentField::Axial(i) => task.axial_sections.get(&i),
                    };
                    (
                        field,
                        BindingDraft::from_binding(binding, field.quantity(), units),
                    )
                })
                .collect(),
        }
    }
    pub(super) fn record(
        &self,
        baseline: &SeparatorSizingTask,
    ) -> Result<SeparatorSizingTask, EquipmentEditError> {
        let mut task = baseline.clone();
        task.tag = self.tag.clone();
        task.name = self.name.clone();
        task.case.name = self.case_name.clone();
        task.case.purpose = self.purpose.clone();
        task.case.fluid_system = self.fluid_system.clone();
        task.case.applicability = self.applicability.clone();
        task.case.applicability_source_ids = self.applicability_sources.clone();
        task.case.inputs.clear();
        task.axial_sections.clear();
        for (field, draft) in &self.fields {
            let binding = draft
                .binding(field.quantity())
                .map_err(|error| EquipmentEditError::Field(*field, Box::new(error)))?;
            if let Some(binding) = binding {
                match field {
                    EquipmentField::Input(i) => {
                        task.case.inputs.insert(*i, binding);
                    }
                    EquipmentField::Axial(i) => {
                        task.axial_sections.insert(*i, binding);
                    }
                }
            }
        }
        Ok(task)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationDraft {
    pub id: SourceObservationId,
    pub quantity: QuantityKind,
    pub original_value: String,
    pub original_unit: String,
    pub canonical: QuantityDraft,
    pub conversion_note: String,
    pub depends_on: Vec<SourceObservationRef>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDraft {
    pub kind: SourceKind,
    pub title: String,
    pub reference: String,
    pub locator: String,
    pub publication_version: String,
    pub accessed_on: String,
    pub adoption_note: String,
    pub review: SourceReview,
    pub fluid_system: String,
    pub phase: SourcePhase,
    pub temperature: QuantityDraft,
    pub pressure: QuantityDraft,
    pub scope: String,
    pub observations: Vec<ObservationDraft>,
}
impl SourceDraft {
    pub(super) fn from_record(source: &SourceRecord, units: &DisplayUnitSet) -> Self {
        Self {
            kind: source.kind,
            title: source.title.clone(),
            reference: source.reference.clone(),
            locator: source.locator.clone(),
            publication_version: source.publication_version.clone(),
            accessed_on: source
                .accessed_on
                .map_or_else(String::new, |d| d.to_string()),
            adoption_note: source.adoption_note.clone(),
            review: source.review,
            fluid_system: source.applicability.fluid_system.clone(),
            phase: source.applicability.phase,
            temperature: source.applicability.operating_conditions.map_or_else(
                || QuantityDraft::empty(units.unit_for(QuantityKind::AbsoluteTemperature)),
                |c| {
                    QuantityDraft::from_si(
                        c.temperature_k,
                        QuantityKind::AbsoluteTemperature,
                        units.unit_for(QuantityKind::AbsoluteTemperature),
                    )
                },
            ),
            pressure: source.applicability.operating_conditions.map_or_else(
                || QuantityDraft::empty(units.unit_for(QuantityKind::AbsolutePressure)),
                |c| {
                    QuantityDraft::from_si(
                        c.pressure_pa,
                        QuantityKind::AbsolutePressure,
                        units.unit_for(QuantityKind::AbsolutePressure),
                    )
                },
            ),
            scope: source.applicability.scope.clone(),
            observations: source
                .observations
                .iter()
                .map(|o| ObservationDraft {
                    id: o.id.clone(),
                    quantity: o.quantity,
                    original_value: o.original_value.to_string(),
                    original_unit: o.original_unit.clone(),
                    canonical: QuantityDraft::from_si(
                        o.canonical_value,
                        o.quantity,
                        units.unit_for(o.quantity),
                    ),
                    conversion_note: o.conversion_note.clone(),
                    depends_on: o.depends_on.clone(),
                })
                .collect(),
        }
    }
    pub(super) fn record(
        &self,
        baseline: &SourceRecord,
    ) -> Result<SourceRecord, EquipmentEditError> {
        let conditions = if self.temperature.text().trim().is_empty()
            && self.pressure.text().trim().is_empty()
        {
            None
        } else {
            Some(SourceOperatingConditions {
                temperature_k: self.temperature.value(QuantityKind::AbsoluteTemperature)?,
                pressure_pa: self.pressure.value(QuantityKind::AbsolutePressure)?,
            })
        };
        let observations = self
            .observations
            .iter()
            .map(|o| {
                Ok(SourceObservation {
                    id: o.id.clone(),
                    quantity: o.quantity,
                    original_value: o
                        .original_value
                        .trim()
                        .parse::<f64>()
                        .ok()
                        .filter(|v| v.is_finite())
                        .ok_or(EquipmentEditError::ObservationValue)?,
                    original_unit: o.original_unit.clone(),
                    canonical_value: o.canonical.value(o.quantity)?,
                    conversion_note: o.conversion_note.clone(),
                    depends_on: o.depends_on.clone(),
                })
            })
            .collect::<Result<_, EquipmentEditError>>()?;
        Ok(SourceRecord {
            id: baseline.id.clone(),
            revision: baseline.revision,
            kind: self.kind,
            title: self.title.clone(),
            reference: self.reference.clone(),
            locator: self.locator.clone(),
            publication_version: self.publication_version.clone(),
            accessed_on: if self.accessed_on.trim().is_empty() {
                None
            } else {
                Some(
                    self.accessed_on
                        .trim()
                        .parse()
                        .map_err(|_| EquipmentEditError::AccessDate)?,
                )
            },
            adoption_note: self.adoption_note.clone(),
            review: self.review,
            applicability: SourceApplicability {
                fluid_system: self.fluid_system.clone(),
                phase: self.phase,
                operating_conditions: conditions,
                scope: self.scope.clone(),
            },
            observations,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasisDraft {
    pub name: String,
    pub quantity: QuantityKind,
    pub value: QuantityDraft,
    pub source: Option<SourceId>,
}
impl BasisDraft {
    pub(super) fn from_record(basis: &BasisEntry, units: &DisplayUnitSet) -> Self {
        Self {
            name: basis.name.clone(),
            quantity: basis.value.quantity,
            value: QuantityDraft::from_si(
                basis.value.value_si,
                basis.value.quantity,
                units.unit_for(basis.value.quantity),
            ),
            source: Some(basis.source_id.clone()),
        }
    }
    pub(super) fn record(
        &self,
        id: BasisEntryId,
        revision: u64,
    ) -> Result<BasisEntry, EquipmentEditError> {
        Ok(BasisEntry {
            id,
            revision,
            name: self.name.clone(),
            value: EngineeringValue {
                quantity: self.quantity,
                value_si: self.value.value(self.quantity)?,
            },
            source_id: self
                .source
                .clone()
                .ok_or(EquipmentEditError::SelectSource)?,
        })
    }
}
