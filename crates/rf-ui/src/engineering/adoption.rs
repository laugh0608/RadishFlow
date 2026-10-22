//! Atomic design adoption. Execution identity is checked again at publication.
use super::*;
use rf_model::{engineering::*, equipment::*};
pub use rf_unitops::separator_adoption::{DimensionAdjustment, GeometryAdjustments};
use rf_unitops::separator_adoption::{map_candidate, sizing_field};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeometryAdoptionTarget {
    New {
        task_id: EquipmentTaskId,
        case_id: EngineeringCaseId,
        tag: String,
        name: String,
        copy_case: bool,
    },
    Existing(EquipmentTaskId),
}
impl GeometryAdoptionTarget {
    pub fn task_id(&self) -> &EquipmentTaskId {
        match self {
            Self::New { task_id, .. } | Self::Existing(task_id) => task_id,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryAdoptionRequest {
    pub source_task: EquipmentTaskId,
    pub target: GeometryAdoptionTarget,
    pub adjustments: GeometryAdjustments,
}
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedGeometryAdoption {
    source_task: EquipmentTaskId,
    calculation: CalculationId,
    target: GeometryAdoptionTarget,
    prepared: PreparedEngineeringEdit,
    previous_geometry: Option<SeparatorGivenGeometry>,
}
impl PreparedGeometryAdoption {
    pub fn source_task(&self) -> &EquipmentTaskId {
        &self.source_task
    }
    pub fn target(&self) -> &GeometryAdoptionTarget {
        &self.target
    }
    pub fn inputs(&self) -> &EngineeringInputs {
        self.prepared.inputs()
    }
    pub fn impacts(&self) -> &[TaskImpact] {
        self.prepared.impacts()
    }
    /// Full before/after records expose removed optional segments for review.
    pub fn previous_geometry(&self) -> Option<&SeparatorGivenGeometry> {
        self.previous_geometry.as_ref()
    }
}
fn direct(
    quantity: rf_types::units::QuantityKind,
    value_si: f64,
    source_id: SourceId,
) -> InputBinding {
    InputBinding::Direct {
        value: EngineeringValue { quantity, value_si },
        source_id,
    }
}
impl EngineeringSession {
    fn adoption_calculation(
        &self,
        source: &EquipmentTaskId,
    ) -> Result<&CalculationId, EngineeringError> {
        if self.pending.keys().any(|key| key.task_id == *source)
            || self.current_candidate(source).is_none()
        {
            return Err(EngineeringError::StaleCalculation);
        }
        self.calculation_id(source)
            .ok_or(EngineeringError::StaleCalculation)
    }
    pub fn prepare_geometry_adoption(
        &self,
        request: GeometryAdoptionRequest,
    ) -> Result<PreparedGeometryAdoption, EngineeringError> {
        let calculation = self.adoption_calculation(&request.source_task)?.clone();
        let candidate = self
            .current_candidate(&request.source_task)
            .ok_or(EngineeringError::StaleCalculation)?;
        let adoption = map_candidate(candidate, self.document_id.to_string(), request.adjustments)
            .map_err(EngineeringError::InvalidAdoption)?;
        let mut commands = Vec::new();
        let mut counter = 0u64;
        let mut next_source = || -> Result<SourceId, EngineeringError> {
            loop {
                counter = counter
                    .checked_add(1)
                    .ok_or(EngineeringError::CounterExhausted)?;
                let id = SourceId::from(format!("adoption-source-{counter}"));
                if !self
                    .revisions
                    .contains_key(&EngineeringRecordId::Source(id.clone()))
                {
                    return Ok(id);
                }
            }
        };
        let copy_sources = matches!(
            request.target,
            GeometryAdoptionTarget::New {
                copy_case: true,
                ..
            }
        );
        let source_ids: BTreeMap<_, _> = adoption
            .source_basis
            .sources
            .iter()
            .filter(|_| copy_sources)
            .map(|source| Ok((source.id.clone(), next_source()?)))
            .collect::<Result<_, EngineeringError>>()?;
        for source in adoption
            .source_basis
            .sources
            .iter()
            .filter(|_| copy_sources)
        {
            let mut copy = source.clone();
            copy.id = source_ids[&source.id].clone();
            copy.revision = 0;
            for observation in &mut copy.observations {
                for dependency in &mut observation.depends_on {
                    dependency.source_id = source_ids[&dependency.source_id].clone();
                }
            }
            commands.push(EngineeringCommand::CreateSource(Box::new(copy)));
        }
        let design_source_id = next_source()?;
        let design_source = SourceRecord {
            id: design_source_id.clone(), revision: 0,
            kind: if candidate.assumed_sources().is_empty() { SourceKind::Estimated } else { SourceKind::Assumed },
            title: "Design geometry adopted from preliminary separator sizing".into(),
            reference: format!("{} / {}", self.document_id, request.source_task),
            locator: "geometry.adoption".into(), publication_version: format!("{}:{}", adoption.source_task.method.id, adoption.source_task.method.version),
            accessed_on: None,
            adoption_note: "Explicit design decision; conditional preliminary sizing. Original evidence and review states are retained in geometry.adoption.".into(),
            review: SourceReview::Adopted,
            applicability: SourceApplicability { fluid_system: String::new(), phase: SourcePhase::NotApplicable, operating_conditions: None, scope: "Adopted full-cylinder geometry; not measured or vendor-certified".into() },
            observations: Vec::new(),
        };
        commands.push(EngineeringCommand::CreateSource(Box::new(design_source)));
        let a = &adoption.source_task.case.applicability;
        let mut geometry = SeparatorGivenGeometry {
            revision: 0,
            applicability: SeparatorGeometryApplicability {
                orientation: a.orientation,
                gas_direction: a.gas_direction,
                geometry: a.geometry,
                internals_description: a.internals_description.clone(),
            },
            applicability_source_ids: vec![design_source_id.clone()],
            inputs: [
                (
                    SeparatorGeometryInput::InternalDiameter,
                    adoption.diameter.adopted_m,
                ),
                (
                    SeparatorGeometryInput::WorkingLiquidHeight,
                    adoption.working_liquid_height.adopted_m,
                ),
            ]
            .into_iter()
            .map(|(field, value)| {
                (
                    field,
                    direct(field.quantity(), value, design_source_id.clone()),
                )
            })
            .collect(),
            axial_sections: adoption
                .axial_lengths_m
                .iter()
                .map(|(section, value)| {
                    // This is a fixed design decision, including its optional segments.
                    (
                        *section,
                        direct(
                            rf_types::units::QuantityKind::Length,
                            *value,
                            design_source_id.clone(),
                        ),
                    )
                })
                .collect(),
            adoption: Some(Box::new(adoption.clone())),
        };
        let previous_geometry = match &request.target {
            GeometryAdoptionTarget::Existing(id) => {
                let previous = self
                    .current
                    .rating_task(id)
                    .ok_or_else(|| {
                        EngineeringError::MissingRecord(EngineeringRecordId::Task(id.clone()))
                    })?
                    .geometry
                    .clone();
                geometry.revision = previous.revision;
                commands.push(EngineeringCommand::AdoptRatingGeometry {
                    task: id.clone(),
                    geometry: Box::new(geometry),
                });
                Some(previous)
            }
            GeometryAdoptionTarget::New {
                task_id,
                case_id,
                tag,
                name,
                copy_case,
            } => {
                let source = &adoption.source_task.case;
                let mut case = SeparatorRatingCase {
                    id: case_id.clone(),
                    revision: 0,
                    name: String::new(),
                    purpose: String::new(),
                    fluid_system: String::new(),
                    applicability: SeparatorRatingApplicability::default(),
                    applicability_source_ids: Vec::new(),
                    inputs: BTreeMap::new(),
                };
                if *copy_case {
                    case.name = source.name.clone();
                    case.purpose = source.purpose.clone();
                    case.fluid_system = source.fluid_system.clone();
                    case.applicability = SeparatorRatingApplicability {
                        service: a.service,
                        k_basis: a.k_basis,
                        flow_basis: a.flow_basis,
                    };
                    case.applicability_source_ids = source
                        .applicability_source_ids
                        .iter()
                        .map(|id| source_ids[id].clone())
                        .collect();
                    for field in SeparatorRatingInput::ALL {
                        let binding = &source.inputs[&sizing_field(field)];
                        let resolved = adoption
                            .source_basis
                            .resolve(field.quantity(), binding)
                            .map_err(EngineeringError::InvalidBasis)?;
                        case.inputs.insert(
                            field,
                            direct(
                                field.quantity(),
                                resolved.value.value_si,
                                source_ids[&resolved.source.id].clone(),
                            ),
                        );
                    }
                }
                commands.push(EngineeringCommand::CreateRatingTask(Box::new(
                    SeparatorRatingTask {
                        id: task_id.clone(),
                        revision: 0,
                        tag: tag.clone(),
                        name: name.clone(),
                        method: rf_unitops::separator_rating::adoption_method(),
                        geometry,
                        cases: vec![case],
                    },
                )));
                None
            }
        };
        let prepared = self.prepare(commands)?;
        Ok(PreparedGeometryAdoption {
            source_task: request.source_task,
            calculation,
            target: request.target,
            prepared,
            previous_geometry,
        })
    }
    pub fn validate_geometry_adoption(
        &self,
        prepared: &PreparedGeometryAdoption,
    ) -> Result<(), EngineeringError> {
        if self.adoption_calculation(&prepared.source_task)? != &prepared.calculation {
            return Err(EngineeringError::StaleCalculation);
        }
        if !prepared.prepared.is_current(self) {
            return Err(EngineeringError::StalePreparation);
        }
        Ok(())
    }
    pub fn commit_geometry_adoption(
        &mut self,
        prepared: PreparedGeometryAdoption,
    ) -> Result<(), EngineeringError> {
        self.validate_geometry_adoption(&prepared)?;
        self.commit(prepared.prepared).map(|_| ())
    }
}
