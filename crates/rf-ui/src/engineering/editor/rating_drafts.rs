use super::*;
use rf_model::equipment::*;
use rf_types::units::{DisplayUnitSet, QuantityKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatingLabelsDraft {
    pub tag: String,
    pub name: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RatingGeometryField {
    Input(SeparatorGeometryInput),
    Axial(SeparatorAxialSection),
}
impl RatingGeometryField {
    pub fn quantity(self) -> QuantityKind {
        QuantityKind::Length
    }
    pub fn all() -> impl Iterator<Item = Self> {
        SeparatorGeometryInput::ALL
            .into_iter()
            .map(Self::Input)
            .chain(SeparatorAxialSection::ALL.into_iter().map(Self::Axial))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatingGeometryDraft {
    pub applicability: SeparatorGeometryApplicability,
    pub applicability_sources: Vec<SourceId>,
    pub fields: BTreeMap<RatingGeometryField, BindingDraft>,
}
impl RatingGeometryDraft {
    pub(super) fn from_record(record: &SeparatorGivenGeometry, units: &DisplayUnitSet) -> Self {
        Self {
            applicability: record.applicability.clone(),
            applicability_sources: record.applicability_source_ids.clone(),
            fields: RatingGeometryField::all()
                .map(|field| {
                    let binding = match field {
                        RatingGeometryField::Input(i) => record.inputs.get(&i),
                        RatingGeometryField::Axial(i) => record.axial_sections.get(&i),
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
        baseline: &SeparatorGivenGeometry,
    ) -> Result<SeparatorGivenGeometry, EquipmentEditError> {
        let mut record = baseline.clone();
        record.applicability = self.applicability.clone();
        record.applicability_source_ids = self.applicability_sources.clone();
        record.inputs.clear();
        record.axial_sections.clear();
        for (field, value) in &self.fields {
            if let Some(binding) = value
                .binding(field.quantity())
                .map_err(|e| EquipmentEditError::RatingGeometryField(*field, Box::new(e)))?
            {
                match field {
                    RatingGeometryField::Input(i) => {
                        record.inputs.insert(*i, binding);
                    }
                    RatingGeometryField::Axial(i) => {
                        record.axial_sections.insert(*i, binding);
                    }
                }
            }
        }
        Ok(record)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatingCaseDraft {
    pub name: String,
    pub purpose: String,
    pub fluid_system: String,
    pub applicability: SeparatorRatingApplicability,
    pub applicability_sources: Vec<SourceId>,
    pub inputs: BTreeMap<SeparatorRatingInput, BindingDraft>,
}
impl RatingCaseDraft {
    pub(super) fn from_record(record: &SeparatorRatingCase, units: &DisplayUnitSet) -> Self {
        Self {
            name: record.name.clone(),
            purpose: record.purpose.clone(),
            fluid_system: record.fluid_system.clone(),
            applicability: record.applicability.clone(),
            applicability_sources: record.applicability_source_ids.clone(),
            inputs: SeparatorRatingInput::ALL
                .into_iter()
                .map(|field| {
                    (
                        field,
                        BindingDraft::from_binding(
                            record.inputs.get(&field),
                            field.quantity(),
                            units,
                        ),
                    )
                })
                .collect(),
        }
    }
    pub(super) fn record(
        &self,
        baseline: &SeparatorRatingCase,
    ) -> Result<SeparatorRatingCase, EquipmentEditError> {
        let mut record = baseline.clone();
        record.name = self.name.clone();
        record.purpose = self.purpose.clone();
        record.fluid_system = self.fluid_system.clone();
        record.applicability = self.applicability.clone();
        record.applicability_source_ids = self.applicability_sources.clone();
        record.inputs.clear();
        for (field, value) in &self.inputs {
            if let Some(binding) = value
                .binding(field.quantity())
                .map_err(|e| EquipmentEditError::RatingInput(*field, Box::new(e)))?
            {
                record.inputs.insert(*field, binding);
            }
        }
        Ok(record)
    }
}
