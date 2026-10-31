use super::*;
use rf_model::equipment::{SeparatorRatingApplicability, SeparatorRatingCase};

impl EquipmentEditorState {
    pub(super) fn add_rating_case(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        task: EquipmentTaskId,
        copy_from: Option<EngineeringCaseId>,
    ) -> Result<(), EquipmentEditError> {
        let new_id: EngineeringCaseId = self.next_id(session, "case")?.into();
        let record = session
            .inputs()
            .rating_task(&task)
            .ok_or(EquipmentEditError::MissingTask)?;
        let upgrades = record.method.version == 1;
        // Copy only formal data; no implicit application or discard of related drafts.
        let mut case = if let Some(id) = copy_from {
            let key = EquipmentCaseKey {
                task_id: task.clone(),
                case_id: id.clone(),
            };
            if self.pending_for_case(&key, session) {
                return Err(EquipmentEditError::PendingDrafts);
            }
            record
                .cases
                .iter()
                .find(|c| c.id == id)
                .ok_or(EquipmentEditError::MissingEditor)?
                .clone()
        } else {
            SeparatorRatingCase {
                id: new_id.clone(),
                revision: 0,
                name: String::new(),
                purpose: String::new(),
                fluid_system: String::new(),
                applicability: SeparatorRatingApplicability::default(),
                applicability_source_ids: vec![],
                inputs: BTreeMap::new(),
            }
        };
        case.id = new_id;
        case.revision = 0;
        let target = EquipmentEditorTarget::RatingCase(EquipmentCaseKey {
            task_id: task.clone(),
            case_id: case.id.clone(),
        });
        let mut commands = Vec::new();
        if upgrades {
            commands.push(EngineeringCommand::UpgradeRatingMethod(task.clone()));
        }
        commands.push(EngineeringCommand::AddRatingCase {
            task,
            case: Box::new(case),
        });
        let prepared = session.prepare(commands)?;
        if upgrades {
            self.set_pending(session, PendingOperation::AddCase { prepared, target });
        } else {
            session.commit(prepared)?;
            self.select(session, units, target)?;
            self.bump()?;
        }
        Ok(())
    }
}
impl EquipmentEditorCommand {
    pub(super) fn error_target(&self) -> Option<EquipmentEditorTarget> {
        match self {
            Self::Select(t)
            | Self::Apply(t)
            | Self::Discard(t)
            | Self::Delete(t)
            | Self::Change { target: t, .. } => Some(t.clone()),
            Self::CheckRating(k) | Self::CalculateRating(k) => {
                Some(EquipmentEditorTarget::RatingCase(k.clone()))
            }
            Self::Check(id) | Self::Calculate(id) => Some(EquipmentEditorTarget::Task(id.clone())),
            Self::AddRatingCase {
                task,
                copy_from: Some(case),
            } => Some(EquipmentEditorTarget::RatingCase(EquipmentCaseKey {
                task_id: task.clone(),
                case_id: case.clone(),
            })),
            Self::AddRatingCase { task, .. } | Self::ReorderRatingCases { task, .. } => {
                Some(EquipmentEditorTarget::RatingTask(task.clone()))
            }
            _ => None,
        }
    }
}
