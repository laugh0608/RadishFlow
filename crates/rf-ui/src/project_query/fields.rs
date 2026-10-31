use super::*;
use crate::variable_browser::{VariableDescriptor, VariableField, VariableSection};
use rf_model::engineering::{BindingOrigin, EngineeringDataError, InputBinding, SourceRecord};
use rf_model::equipment::{
    SeparatorAxialSection, SeparatorGeometryInput, SeparatorInput, SeparatorRatingInput,
};
use rf_types::units::{MeasurementUnit, QuantityKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectField {
    Flow {
        section: VariableSection,
        field: VariableField,
    },
    SizingInput(SeparatorInput),
    RatingInput(SeparatorRatingInput),
    GeometryInput(SeparatorGeometryInput),
    AxialSection(SeparatorAxialSection),
    BasisValue,
}
#[derive(Debug)]
pub enum EngineeringQuantityState<'a> {
    Value {
        value_si: f64,
        source: &'a SourceRecord,
        origin: BindingOrigin<'a>,
    },
    Unspecified,
    Invalid(EngineeringDataError),
}
#[derive(Debug)]
pub struct EngineeringQuantityRead<'a> {
    pub quantity: QuantityKind,
    pub state: EngineeringQuantityState<'a>,
}
impl EngineeringQuantityRead<'_> {
    pub fn canonical_unit(&self) -> MeasurementUnit {
        self.quantity.definition().canonical_unit
    }
}
#[derive(Debug)]
pub enum ProjectFieldValue<'a> {
    Flow(VariableDescriptor),
    Engineering(EngineeringQuantityRead<'a>),
}
#[derive(Debug)]
pub struct ProjectFieldDescriptor<'a> {
    pub object: ProjectObjectRef,
    pub field: ProjectField,
    pub value: ProjectFieldValue<'a>,
}
impl<'a> ProjectQuery<'a> {
    pub fn fields(
        &self,
        reference: &ProjectObjectRef,
    ) -> Result<Vec<ProjectFieldDescriptor<'a>>, ProjectQueryError> {
        let fields = match self.record(reference)? {
            ProjectRecord::Flow(v) => self
                .flow_variables()
                .variables(&v.id)
                .expect("validated flow object")
                .into_iter()
                .map(|v| ProjectField::Flow {
                    section: v.id.section,
                    field: v.id.field,
                })
                .collect(),
            ProjectRecord::SizingCase(_) => SeparatorInput::ALL
                .into_iter()
                .map(ProjectField::SizingInput)
                .collect(),
            ProjectRecord::RatingCase(_) => SeparatorRatingInput::ALL
                .into_iter()
                .map(ProjectField::RatingInput)
                .collect(),
            ProjectRecord::Equipment(EquipmentTask::SeparatorSizing(_)) => {
                SeparatorAxialSection::ALL
                    .into_iter()
                    .map(ProjectField::AxialSection)
                    .collect()
            }
            ProjectRecord::Geometry(_) => SeparatorGeometryInput::ALL
                .into_iter()
                .map(ProjectField::GeometryInput)
                .chain(
                    SeparatorAxialSection::ALL
                        .into_iter()
                        .map(ProjectField::AxialSection),
                )
                .collect(),
            ProjectRecord::Basis(_) => vec![ProjectField::BasisValue],
            ProjectRecord::Source(_)
            | ProjectRecord::Equipment(EquipmentTask::SeparatorRating(_)) => Vec::new(),
        };
        fields
            .into_iter()
            .map(|field| self.read_field(reference, &field))
            .collect()
    }
    /// Numeric fields are resolved in SI, without method execution or draft edits.
    /// Non-numeric metadata is available through the typed, borrowed record().
    pub fn read_field(
        &self,
        reference: &ProjectObjectRef,
        field: &ProjectField,
    ) -> Result<ProjectFieldDescriptor<'a>, ProjectQueryError> {
        let record = self.record(reference)?;
        let missing = || ProjectQueryError::FieldMissing(field.clone());
        let value = match (record, field) {
            (ProjectRecord::Flow(v), ProjectField::Flow { section, field }) => {
                let id = variable_browser::VariableId {
                    document: reference.document.clone(),
                    object: v.id,
                    section: *section,
                    field: field.clone(),
                };
                ProjectFieldValue::Flow(self.flow_variables().read(&id).map_err(|_| missing())?)
            }
            (ProjectRecord::SizingCase(c), ProjectField::SizingInput(key)) => {
                self.binding_value(key.quantity(), c.inputs.get(key))
            }
            (ProjectRecord::RatingCase(c), ProjectField::RatingInput(key)) => {
                self.binding_value(key.quantity(), c.inputs.get(key))
            }
            (ProjectRecord::Geometry(g), ProjectField::GeometryInput(key)) => {
                self.binding_value(key.quantity(), g.inputs.get(key))
            }
            (ProjectRecord::Geometry(g), ProjectField::AxialSection(key)) => {
                self.binding_value(QuantityKind::Length, g.axial_sections.get(key))
            }
            (
                ProjectRecord::Equipment(EquipmentTask::SeparatorSizing(t)),
                ProjectField::AxialSection(key),
            ) => self.binding_value(QuantityKind::Length, t.axial_sections.get(key)),
            (ProjectRecord::Basis(entry), ProjectField::BasisValue) => {
                let basis = &self.workspace.engineering.inputs().basis;
                let source = basis
                    .sources
                    .iter()
                    .find(|v| v.id == entry.source_id)
                    .expect("validated basis reference");
                ProjectFieldValue::Engineering(EngineeringQuantityRead {
                    quantity: entry.value.quantity,
                    state: EngineeringQuantityState::Value {
                        value_si: entry.value.value_si,
                        source,
                        origin: BindingOrigin::Basis(entry),
                    },
                })
            }
            _ => return Err(missing()),
        };
        Ok(ProjectFieldDescriptor {
            object: reference.clone(),
            field: field.clone(),
            value,
        })
    }
    fn binding_value(
        &self,
        quantity: QuantityKind,
        binding: Option<&'a InputBinding>,
    ) -> ProjectFieldValue<'a> {
        let state = match binding {
            None => EngineeringQuantityState::Unspecified,
            Some(binding) => match self
                .workspace
                .engineering
                .inputs()
                .basis
                .resolve(quantity, binding)
            {
                Ok(value) => EngineeringQuantityState::Value {
                    value_si: value.value.value_si,
                    source: value.source,
                    origin: value.origin,
                },
                Err(error) => EngineeringQuantityState::Invalid(error),
            },
        };
        ProjectFieldValue::Engineering(EngineeringQuantityRead { quantity, state })
    }
}
