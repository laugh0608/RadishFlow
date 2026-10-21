//! Draft-aware adoption boundary. Product entry points are intentionally separate.
use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedEditorGeometryAdoption {
    checkpoint: EngineeringCheckpoint,
    prepared: PreparedGeometryAdoption,
}
impl PreparedEditorGeometryAdoption {
    pub fn preview(&self) -> &PreparedGeometryAdoption {
        &self.prepared
    }
}
impl EquipmentEditorState {
    pub fn prepare_geometry_adoption(
        &self,
        session: &EngineeringSession,
        request: GeometryAdoptionRequest,
    ) -> Result<PreparedEditorGeometryAdoption, EquipmentEditError> {
        if self.pending_for_task(&request.source_task, session)
            || self.pending_for_task(request.target.task_id(), session)
        {
            return Err(EquipmentEditError::PendingDrafts);
        }
        Ok(PreparedEditorGeometryAdoption {
            checkpoint: self.checkpoint(),
            prepared: session.prepare_geometry_adoption(request)?,
        })
    }
    pub fn commit_geometry_adoption(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        proposal: PreparedEditorGeometryAdoption,
    ) -> Result<(), EquipmentEditError> {
        if self.checkpoint != proposal.checkpoint {
            return Err(EquipmentEditError::StaleEditor);
        }
        if self.pending_for_task(proposal.prepared.source_task(), session)
            || self.pending_for_task(proposal.prepared.target().task_id(), session)
        {
            return Err(EquipmentEditError::PendingDrafts);
        }
        // Publish formal data and refresh existing clean editors together. A
        // counter failure must not leave either half of the transaction visible.
        let task = proposal.prepared.target().task_id().clone();
        let mut staged = self.clone();
        let mut formal = session.clone();
        formal.commit_geometry_adoption(proposal.prepared)?;
        for target in [
            EquipmentEditorTarget::RatingTask(task.clone()),
            EquipmentEditorTarget::RatingGeometry(task),
        ] {
            if staged.drafts.contains_key(&target) {
                staged.reload_target(&formal, units, &target)?;
            }
            staged.target_errors.remove(&target);
        }
        staged.bump()?;
        *self = staged;
        *session = formal;
        Ok(())
    }
}
