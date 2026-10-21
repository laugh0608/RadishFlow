//! In-memory equipment transactions and result qualification, independent of flowsheet Run.
//! Formal data is separate from editor drafts and current-result qualification.
mod adoption;
#[cfg(test)]
mod adoption_tests;
pub use adoption::*;
mod commands;
pub mod editor;
mod impact;
mod matrix;
pub use matrix::{RatingMatrixCapture, RatingMatrixCaptureError, RatingMatrixRow};
mod rating_commands;
#[cfg(test)]
mod rating_tests;
mod results;
#[cfg(test)]
mod tests;

pub use commands::{
    EngineeringCommand, EngineeringError, EngineeringReference, PreparedEngineeringEdit,
};
pub use impact::TaskImpact;
pub use results::{
    CalculationId, CalculationRequest, CompletedCalculation, EquipmentResultStatus,
    ResultGeneration, ResultStaleness,
};

use crate::DocumentId;
use rf_model::{
    engineering::EngineeringBasis,
    equipment::{EquipmentTask, SeparatorRatingTask, SeparatorSizingTask},
};
use rf_types::{BasisEntryId, EngineeringCaseId, EquipmentTaskId, SourceId};
use std::{collections::BTreeMap, sync::Arc};

/// Opaque identity also used by departure confirmations. Clones retain identity;
/// every committed edit creates a fresh identity, including on divergent clones.
#[derive(Debug, Clone)]
pub struct EngineeringCheckpoint(Arc<()>);
impl EngineeringCheckpoint {
    fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for EngineeringCheckpoint {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for EngineeringCheckpoint {}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EngineeringInputs {
    pub basis: EngineeringBasis,
    pub tasks: Vec<EquipmentTask>,
}
impl EngineeringInputs {
    pub fn is_empty(&self) -> bool {
        self.basis.sources.is_empty()
            && self.basis.basis_entries.is_empty()
            && self.tasks.is_empty()
    }
    pub fn task(&self, id: &EquipmentTaskId) -> Option<&SeparatorSizingTask> {
        self.equipment_task(id)?.as_sizing()
    }
    pub fn equipment_task(&self, id: &EquipmentTaskId) -> Option<&EquipmentTask> {
        self.tasks.iter().find(|task| task.id() == id)
    }
    pub fn rating_task(&self, id: &EquipmentTaskId) -> Option<&SeparatorRatingTask> {
        self.equipment_task(id)?.as_rating()
    }
    pub fn has_rating(&self) -> bool {
        self.rating_method_version() != 0
    }
    pub fn rating_method_version(&self) -> u32 {
        self.tasks
            .iter()
            .filter_map(|t| t.as_rating().map(|r| r.method.version))
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EngineeringRecordId {
    Source(SourceId),
    Basis(BasisEntryId),
    Task(EquipmentTaskId),
    Case(EngineeringCaseId),
    Geometry(EquipmentTaskId),
}
/// Explicit result membership, independent of task display order and UI selection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EquipmentCaseKey {
    pub task_id: EquipmentTaskId,
    pub case_id: EngineeringCaseId,
}
#[derive(Debug, Clone, PartialEq)]
struct HistoryEntry {
    before: EngineeringInputs,
    after: EngineeringInputs,
}

/// Owned application session. All formal writes pass through prepare/commit or
/// undo/redo; generations and retired identities are never restored from history.
#[derive(Debug, Clone, PartialEq)]
pub struct EngineeringSession {
    document_id: DocumentId,
    identity: EngineeringCheckpoint,
    checkpoint: EngineeringCheckpoint,
    current: EngineeringInputs,
    baseline: EngineeringInputs,
    stored_block_present: bool,
    revisions: BTreeMap<EngineeringRecordId, u64>,
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    membership: BTreeMap<EquipmentTaskId, EngineeringCheckpoint>,
    generations: BTreeMap<EquipmentCaseKey, ResultGeneration>,
    eligibility: BTreeMap<EquipmentCaseKey, EngineeringCheckpoint>,
    pending: BTreeMap<EquipmentCaseKey, CalculationId>,
    results: BTreeMap<EquipmentCaseKey, results::CalculationRecord>,
}
impl EngineeringSession {
    pub fn new(document_id: DocumentId) -> Self {
        let mut state = Self::from_inputs(document_id, EngineeringInputs::default())
            .expect("empty engineering inputs are valid");
        state.stored_block_present = false;
        state
    }
    /// A fresh in-memory session never imports result caches or execution tokens.
    /// Loaded formal inputs establish the clean baseline; history starts empty.
    pub fn from_inputs(
        document_id: DocumentId,
        inputs: EngineeringInputs,
    ) -> Result<Self, EngineeringError> {
        commands::validate(&inputs)?;
        let mut state = Self {
            document_id,
            identity: EngineeringCheckpoint::fresh(),
            checkpoint: EngineeringCheckpoint::fresh(),
            current: inputs.clone(),
            baseline: inputs,
            stored_block_present: true,
            revisions: BTreeMap::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            membership: BTreeMap::new(),
            generations: BTreeMap::new(),
            eligibility: BTreeMap::new(),
            pending: BTreeMap::new(),
            results: BTreeMap::new(),
        };
        state.revisions = commands::record_revisions(&state.current);
        for task in &state.current.tasks {
            state
                .membership
                .insert(task.id().clone(), EngineeringCheckpoint::fresh());
            for case in task.case_ids() {
                let key = EquipmentCaseKey {
                    task_id: task.id().clone(),
                    case_id: case.clone(),
                };
                state
                    .generations
                    .insert(key.clone(), ResultGeneration::default());
                state
                    .eligibility
                    .insert(key, EngineeringCheckpoint::fresh());
            }
        }
        Ok(state)
    }
    pub fn inputs(&self) -> &EngineeringInputs {
        &self.current
    }
    pub fn checkpoint(&self) -> EngineeringCheckpoint {
        self.checkpoint.clone()
    }
    pub fn is_dirty(&self) -> bool {
        !commands::same_content(&self.current, &self.baseline)
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn rating_capability_is_new(&self) -> bool {
        self.current.rating_method_version() > self.baseline.rating_method_version()
    }
}

/// Frozen formal input snapshot for successful-write acknowledgement. A late save
/// records what was written; later edits stay dirty and history remains intact.
#[derive(Debug, Clone)]
pub struct EngineeringSaveSnapshot {
    identity: EngineeringCheckpoint,
    inputs: EngineeringInputs,
    block_present: bool,
}
impl EngineeringSaveSnapshot {
    pub fn inputs(&self) -> &EngineeringInputs {
        &self.inputs
    }
    pub fn block_present(&self) -> bool {
        self.block_present
    }
}
impl EngineeringSession {
    pub fn capture_save(&self) -> EngineeringSaveSnapshot {
        EngineeringSaveSnapshot {
            identity: self.identity.clone(),
            inputs: self.current.clone(),
            block_present: self.stored_block_present || !self.current.is_empty(),
        }
    }
    pub fn record_saved(
        &mut self,
        snapshot: EngineeringSaveSnapshot,
    ) -> Result<(), EngineeringError> {
        if self.identity != snapshot.identity {
            return Err(EngineeringError::StalePreparation);
        }
        self.baseline = snapshot.inputs;
        self.stored_block_present = snapshot.block_present;
        Ok(())
    }
}
