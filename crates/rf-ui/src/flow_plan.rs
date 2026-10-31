//! Project-owned steady-state configuration. Identity, history and drafts remain
//! session local; a plan change is independent of the physical edit context.
mod editor;
mod simultaneous;
use crate::{FlowRunQualification, RunStatus, SolvePendingReason, WorkspaceState};
pub use editor::*;
use rf_solver::partitioned::{RecycleSolveOptions, assemble_partitioned_plan};
use rf_types::{RfError, RfResult, StreamId, UnitId};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum FlowPlanConfiguration {
    #[default]
    Acyclic,
    Partitioned(RecycleSolveOptions),
    Simultaneous(rf_solver::simultaneous::ModularOptions),
}
impl FlowPlanConfiguration {
    pub fn validate_configuration(&self) -> RfResult<()> {
        match self {
            Self::Acyclic => Ok(()),
            Self::Partitioned(o) => o.validate_configuration(),
            Self::Simultaneous(o) => simultaneous::validate_configuration(o),
        }
    }
    pub fn validate_for(&self, workspace: &WorkspaceState) -> RfResult<()> {
        match self {
            Self::Acyclic => {
                rf_solver::sequential_execution_order(&workspace.document.flowsheet).map(|_| ())
            }
            Self::Partitioned(o) => {
                assemble_partitioned_plan(&workspace.document.flowsheet, o).map(|_| ())
            }
            Self::Simultaneous(o) => {
                rf_solver::simultaneous::assemble(&workspace.document.flowsheet, o).map(|_| ())
            }
        }
    }
    pub fn referenced_unit_ids(&self) -> BTreeSet<UnitId> {
        match self {
            Self::Acyclic => BTreeSet::new(),
            Self::Simultaneous(o) => o
                .scales
                .iter()
                .flat_map(|s| s.keys())
                .flat_map(|b| [b.source.unit_id.clone(), b.sink.unit_id.clone()])
                .collect(),
            Self::Partitioned(o) => o
                .tears
                .iter()
                .flat_map(|t| [t.source.unit_id.clone(), t.sink.unit_id.clone()])
                .collect(),
        }
    }
    pub fn referenced_stream_ids(&self) -> BTreeSet<StreamId> {
        match self {
            Self::Acyclic => BTreeSet::new(),
            Self::Simultaneous(o) => o
                .scales
                .iter()
                .flat_map(|s| s.keys())
                .map(|b| b.stream.clone())
                .collect(),
            Self::Partitioned(o) => o.tears.iter().map(|t| t.stream_id.clone()).collect(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct FlowPlanGeneration(Arc<()>);
impl FlowPlanGeneration {
    fn fresh() -> Self {
        Self(Arc::new(()))
    }
}
impl PartialEq for FlowPlanGeneration {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for FlowPlanGeneration {}
#[derive(Debug, Clone)]
pub struct FlowPlanEditContext {
    session: FlowRunQualification,
    generation: FlowPlanGeneration,
}
impl PartialEq for FlowPlanEditContext {
    fn eq(&self, other: &Self) -> bool {
        self.session.same_session(&other.session) && self.generation == other.generation
    }
}
impl Eq for FlowPlanEditContext {}
impl FlowPlanEditContext {
    pub fn capture(workspace: &WorkspaceState) -> Self {
        Self {
            session: workspace.flow_run_qualification(),
            generation: workspace.flow_plan.generation.clone(),
        }
    }
    pub fn is_current(&self, workspace: &WorkspaceState) -> bool {
        self.session
            .same_session(&workspace.flow_run_qualification())
            && self.generation == workspace.flow_plan.generation
    }
    pub fn generation(&self) -> &FlowPlanGeneration {
        &self.generation
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct FlowPlanState {
    current: FlowPlanConfiguration,
    saved: FlowPlanConfiguration,
    saved_capabilities: BTreeMap<String, u32>,
    generation: FlowPlanGeneration,
    draft_generation: FlowPlanGeneration,
    revision: u64,
    undo: Vec<FlowPlanConfiguration>,
    redo: Vec<FlowPlanConfiguration>,
    draft: Option<FlowPlanDraft>,
}
impl Default for FlowPlanState {
    fn default() -> Self {
        Self::from_loaded(FlowPlanConfiguration::Acyclic)
    }
}
impl FlowPlanState {
    pub fn from_loaded(plan: FlowPlanConfiguration) -> Self {
        Self {
            current: plan.clone(),
            saved: plan,
            saved_capabilities: BTreeMap::new(),
            generation: FlowPlanGeneration::fresh(),
            draft_generation: FlowPlanGeneration::fresh(),
            revision: 0,
            undo: Vec::new(),
            redo: Vec::new(),
            draft: None,
        }
    }
    pub fn current(&self) -> &FlowPlanConfiguration {
        &self.current
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn generation(&self) -> &FlowPlanGeneration {
        &self.generation
    }
    pub fn is_dirty(&self) -> bool {
        self.current != self.saved
    }
    pub fn saved(&self) -> &FlowPlanConfiguration {
        &self.saved
    }
    /// Loaded or successfully saved flow capabilities, never the current draft.
    pub fn saved_capabilities(&self) -> &BTreeMap<String, u32> {
        &self.saved_capabilities
    }
    pub fn record_saved_capabilities(&mut self, capabilities: BTreeMap<String, u32>) {
        self.saved_capabilities = capabilities;
    }
    pub fn draft_generation(&self) -> &FlowPlanGeneration {
        &self.draft_generation
    }
    pub fn draft(&self) -> Option<&FlowPlanDraft> {
        self.draft.as_ref()
    }
    pub fn pending_count(&self) -> usize {
        usize::from(
            self.draft
                .as_ref()
                .is_some_and(|d| d.is_pending(&self.current)),
        )
    }
    pub fn set_draft(&mut self, draft: FlowPlanDraft) {
        if self.draft.as_ref() != Some(&draft) {
            self.draft_generation = FlowPlanGeneration::fresh();
        }
        self.draft = Some(draft);
    }
    pub fn discard_draft(&mut self) {
        if self.draft.is_some() {
            self.draft_generation = FlowPlanGeneration::fresh();
        }
        self.draft = None;
    }
    pub fn record_saved(&mut self, captured: FlowPlanConfiguration) {
        self.saved = captured;
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum FlowPlanCommand {
    Apply(FlowPlanConfiguration),
    RestoreSaved,
    Undo,
    Redo,
}
#[derive(Clone, Copy)]
enum PlanHistory {
    New,
    Undo,
    Redo,
}
impl WorkspaceState {
    pub fn apply_flow_plan(
        &mut self,
        expected: &FlowPlanEditContext,
        command: FlowPlanCommand,
    ) -> RfResult<bool> {
        if !expected.is_current(self) {
            return Err(RfError::invalid_input("flow plan edit context changed")
                .with_diagnostic_code("flow.plan.context_changed"));
        }
        let (next, history) = match command {
            FlowPlanCommand::Apply(plan) => (plan, PlanHistory::New),
            FlowPlanCommand::RestoreSaved => (self.flow_plan.saved.clone(), PlanHistory::New),
            FlowPlanCommand::Undo => match self.flow_plan.undo.last() {
                Some(p) => (p.clone(), PlanHistory::Undo),
                None => return Ok(false),
            },
            FlowPlanCommand::Redo => match self.flow_plan.redo.last() {
                Some(p) => (p.clone(), PlanHistory::Redo),
                None => return Ok(false),
            },
        };
        next.validate_configuration()?;
        if next == self.flow_plan.current {
            return Ok(false);
        }
        let revision = self
            .flow_plan
            .revision
            .checked_add(1)
            .ok_or_else(|| RfError::invalid_input("flow plan revision exhausted"))?;
        let previous = std::mem::replace(&mut self.flow_plan.current, next);
        match history {
            PlanHistory::Undo => {
                self.flow_plan.undo.pop();
                self.flow_plan.redo.push(previous);
            }
            PlanHistory::Redo => {
                self.flow_plan.redo.pop();
                self.flow_plan.undo.push(previous);
            }
            PlanHistory::New => {
                self.flow_plan.undo.push(previous);
                self.flow_plan.redo.clear();
            }
        }
        self.flow_plan.revision = revision;
        self.flow_plan.generation = FlowPlanGeneration::fresh();
        self.reserve_flow_plan_identities();
        self.revoke_flow_run_acceptance();
        self.solve_session.status = RunStatus::Dirty;
        self.solve_session.pending_reason = Some(SolvePendingReason::PlanChanged);
        self.solve_session.latest_snapshot = None;
        self.solve_session.latest_diagnostic = None;
        Ok(true)
    }
}
#[cfg(test)]
mod tests;
