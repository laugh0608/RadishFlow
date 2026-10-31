//! Draft-aware adoption boundary shared by preview, confirmation and direct callers.
use super::*;
use rf_model::equipment::{GeometryAdoptionError, SeparatorGeometryInput};
use rf_types::units::QuantityKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionAdjustmentDraft {
    pub value: QuantityDraft,
    pub reason: String,
}
impl DimensionAdjustmentDraft {
    pub fn adjustment(&self) -> Result<DimensionAdjustment, EquipmentEditError> {
        let value_m = self.value.value(QuantityKind::Length)?;
        if !value_m.is_finite() || value_m <= 0.0 {
            return Err(
                EngineeringError::InvalidAdoption(GeometryAdoptionError::InvalidDimension).into(),
            );
        }
        if self.reason.trim().is_empty() {
            return Err(EngineeringError::InvalidAdoption(
                GeometryAdoptionError::MissingAdjustmentReason,
            )
            .into());
        }
        Ok(DimensionAdjustment {
            value_m,
            reason: self.reason.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeometryAdoptionDestination {
    New {
        tag: String,
        name: String,
        copy_case: bool,
    },
    Existing(EquipmentTaskId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryAdoptionDraft {
    pub source_task: EquipmentTaskId,
    pub target: GeometryAdoptionDestination,
    pub diameter: Option<DimensionAdjustmentDraft>,
    pub working_liquid_height: Option<DimensionAdjustmentDraft>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeometryAdoptionPreview {
    key: EngineeringCheckpoint,
    proposal: PreparedEditorGeometryAdoption,
}
impl GeometryAdoptionPreview {
    pub fn key(&self) -> EngineeringCheckpoint {
        self.key.clone()
    }
    pub fn proposal(&self) -> &PreparedGeometryAdoption {
        self.proposal.preview()
    }
}

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
    pub fn geometry_adoption_preview(&self) -> Option<&GeometryAdoptionPreview> {
        self.adoption_preview.as_ref()
    }
    pub fn validate_geometry_adoption_preview(
        &self,
        session: &EngineeringSession,
    ) -> Result<(), EquipmentEditError> {
        let preview = self
            .adoption_preview
            .as_ref()
            .ok_or(EquipmentEditError::StaleEditor)?;
        self.validate_geometry_adoption(session, &preview.proposal)
    }
    pub(super) fn preview_geometry_adoption(
        &mut self,
        session: &EngineeringSession,
        draft: GeometryAdoptionDraft,
    ) -> Result<(), EquipmentEditError> {
        let adjustment = |draft: Option<DimensionAdjustmentDraft>, field| {
            draft.map(|d| d.adjustment()).transpose().map_err(|e| {
                EquipmentEditError::RatingGeometryField(
                    RatingGeometryField::Input(field),
                    Box::new(e),
                )
            })
        };
        let adjustments = GeometryAdjustments {
            diameter: adjustment(draft.diameter, SeparatorGeometryInput::InternalDiameter)?,
            working_liquid_height: adjustment(
                draft.working_liquid_height,
                SeparatorGeometryInput::WorkingLiquidHeight,
            )?,
        };
        self.preview_typed_adoption(session, draft.source_task, draft.target, adjustments)
    }
    pub(super) fn preview_typed_adoption(
        &mut self,
        session: &EngineeringSession,
        source: EquipmentTaskId,
        destination: GeometryAdoptionDestination,
        adjustments: GeometryAdjustments,
    ) -> Result<(), EquipmentEditError> {
        let target = match destination {
            GeometryAdoptionDestination::New {
                tag,
                name,
                copy_case,
            } => GeometryAdoptionTarget::New {
                task_id: self.next_id(session, "task")?.into(),
                case_id: self.next_id(session, "case")?.into(),
                tag,
                name,
                copy_case,
            },
            GeometryAdoptionDestination::Existing(id) => GeometryAdoptionTarget::Existing(id),
        };
        let proposal = self.prepare_geometry_adoption(
            session,
            GeometryAdoptionRequest {
                source_task: source,
                target,
                adjustments,
            },
        )?;
        self.adoption_preview = Some(GeometryAdoptionPreview {
            key: EngineeringCheckpoint::fresh(),
            proposal,
        });
        Ok(())
    }
    pub(super) fn confirm_geometry_adoption(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        key: EngineeringCheckpoint,
    ) -> Result<(), EquipmentEditError> {
        let preview = self
            .adoption_preview
            .clone()
            .ok_or(EquipmentEditError::StaleEditor)?;
        if preview.key != key {
            return Err(EquipmentEditError::StaleEditor);
        }
        let target = preview.proposal.preview().target().task_id().clone();
        self.commit_geometry_adoption(session, units, preview.proposal)?;
        self.adoption_preview = None;
        self.select(
            session,
            units,
            EquipmentEditorTarget::RatingGeometry(target),
        )?;
        Ok(())
    }
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
    pub fn validate_geometry_adoption(
        &self,
        session: &EngineeringSession,
        proposal: &PreparedEditorGeometryAdoption,
    ) -> Result<(), EquipmentEditError> {
        if self.checkpoint != proposal.checkpoint {
            return Err(EquipmentEditError::StaleEditor);
        }
        if self.pending_for_task(proposal.prepared.source_task(), session)
            || self.pending_for_task(proposal.prepared.target().task_id(), session)
        {
            return Err(EquipmentEditError::PendingDrafts);
        }
        session.validate_geometry_adoption(&proposal.prepared)?;
        Ok(())
    }
    pub fn commit_geometry_adoption(
        &mut self,
        session: &mut EngineeringSession,
        units: &DisplayUnitSet,
        proposal: PreparedEditorGeometryAdoption,
    ) -> Result<(), EquipmentEditError> {
        self.validate_geometry_adoption(session, &proposal)?;
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
