use super::*;
use rf_model::equipment::*;

impl EquipmentEditorState {
    pub(super) fn create_rating(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
    ) -> Result<(), EquipmentEditError> {
        let id: EquipmentTaskId = self.next_id(session, "task")?.into();
        let case_id = self.next_id(session, "case")?.into();
        session.apply(vec![EngineeringCommand::CreateRatingTask(Box::new(
            SeparatorRatingTask {
                id: id.clone(),
                revision: 0,
                tag: String::new(),
                name: String::new(),
                method: rf_unitops::separator_rating::matrix_method(),
                geometry: SeparatorGivenGeometry {
                    revision: 0,
                    applicability: SeparatorGeometryApplicability::default(),
                    applicability_source_ids: vec![],
                    inputs: BTreeMap::new(),
                    axial_sections: BTreeMap::new(),
                },
                cases: vec![SeparatorRatingCase {
                    id: case_id,
                    revision: 0,
                    name: String::new(),
                    purpose: String::new(),
                    fluid_system: String::new(),
                    applicability: SeparatorRatingApplicability::default(),
                    applicability_source_ids: vec![],
                    inputs: BTreeMap::new(),
                }],
            },
        ))])?;
        self.select(session, units, EquipmentEditorTarget::RatingGeometry(id))
    }
    pub(super) fn rating_record(
        session: &EngineeringSession,
        units: &DisplayUnitSet,
        target: &EquipmentEditorTarget,
    ) -> Result<(RecordBaseline, EquipmentDraft), EquipmentEditError> {
        let id = match target {
            EquipmentEditorTarget::RatingTask(id) | EquipmentEditorTarget::RatingGeometry(id) => id,
            EquipmentEditorTarget::RatingCase(key) => &key.task_id,
            _ => return Err(EquipmentEditError::DraftTypeMismatch),
        };
        let task = session
            .inputs()
            .rating_task(id)
            .ok_or(EquipmentEditError::MissingTask)?;
        Ok(match target {
            EquipmentEditorTarget::RatingTask(_) => (
                RecordBaseline::RatingTask(task.clone()),
                EquipmentDraft::RatingTask(RatingLabelsDraft {
                    tag: task.tag.clone(),
                    name: task.name.clone(),
                }),
            ),
            EquipmentEditorTarget::RatingGeometry(_) => (
                RecordBaseline::RatingGeometry(task.geometry.clone()),
                EquipmentDraft::RatingGeometry(RatingGeometryDraft::from_record(
                    &task.geometry,
                    units,
                )),
            ),
            EquipmentEditorTarget::RatingCase(key) => {
                let case = task
                    .cases
                    .iter()
                    .find(|c| c.id == key.case_id)
                    .ok_or(EquipmentEditError::MissingEditor)?;
                (
                    RecordBaseline::RatingCase(case.clone()),
                    EquipmentDraft::RatingCase(RatingCaseDraft::from_record(case, units)),
                )
            }
            _ => return Err(EquipmentEditError::DraftTypeMismatch),
        })
    }
    pub(super) fn target_exists(
        session: &EngineeringSession,
        target: &EquipmentEditorTarget,
    ) -> bool {
        match target {
            EquipmentEditorTarget::Task(id) => session.inputs().task(id).is_some(),
            EquipmentEditorTarget::RatingTask(id) | EquipmentEditorTarget::RatingGeometry(id) => {
                session.inputs().rating_task(id).is_some()
            }
            EquipmentEditorTarget::RatingCase(key) => session
                .inputs()
                .rating_task(&key.task_id)
                .is_some_and(|t| t.cases.iter().any(|c| c.id == key.case_id)),
            EquipmentEditorTarget::Source(id) => {
                session.inputs().basis.sources.iter().any(|r| r.id == *id)
            }
            EquipmentEditorTarget::Basis(id) => session
                .inputs()
                .basis
                .basis_entries
                .iter()
                .any(|r| r.id == *id),
        }
    }
}
