//! Session-bound result reads. Execution evidence travels with the accepted result,
//! never with the current package selection or a second mutable result cache.
use crate::{
    DiagnosticSummary, DocumentId, FlowRunId, FlowRunQualification, FlowRunReceipt, WorkspaceState,
    latest_snapshot, stale_snapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultStatus {
    Current,
    Stale,
    Missing,
    Failed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultDataOrigin {
    InMemory,
    Cache,
    LocalFiles,
    Bundled,
}

/// Application evidence, independent of resource IO and public transport DTOs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedPropertyPackage {
    pub package_id: String,
    pub version: String,
    pub origin: ResultDataOrigin,
    pub manifest_schema_version: u32,
    pub payload_schema_version: u32,
    pub payload_fingerprint: String,
    pub canonical_payload_size_bytes: u64,
    pub liquid_model: &'static str,
    pub vapor_model: &'static str,
    pub thermo_implementation: &'static str,
    pub flash_implementation: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedFlowPlan {
    pub method: &'static str,
    pub acceptance_contract_version: u32,
    pub configuration_sha256: String,
    pub modular: Option<ModularExecutionEvidence>,
    pub equation_oriented: Option<EquationExecutionEvidence>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquationExecutionEvidence {
    pub capability: &'static str,
    pub capability_version: u32,
    pub equation_contract_version: u32,
    pub core_backend: Option<&'static str>,
}
/// Requested contracts and actually invoked core method are separate facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModularExecutionEvidence {
    pub capability: &'static str,
    pub capability_version: u32,
    pub response_contract_version: u32,
    pub core_method: Option<&'static str>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowExecutionEvidence {
    pub plan: Option<ExecutedFlowPlan>,
    pub run_id: FlowRunId,
    pub solver_implementation: &'static str,
    /// None when preparation failed before a validated package was obtained.
    pub package: Option<ExecutedPropertyPackage>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowResultOrigin {
    qualification: FlowRunQualification,
    plan_generation: crate::flow_plan::FlowPlanGeneration,
    execution: Option<FlowExecutionEvidence>,
}
impl FlowResultOrigin {
    pub fn document_id(&self) -> &DocumentId {
        self.qualification.document_id()
    }
    pub fn revision(&self) -> u64 {
        self.qualification.revision()
    }
    pub fn with_plan_evidence(mut self, plan: ExecutedFlowPlan) -> Self {
        if let Some(e) = &mut self.execution {
            e.plan = Some(plan);
        }
        self
    }
    pub fn execution(&self) -> Option<&FlowExecutionEvidence> {
        self.execution.as_ref()
    }
    pub(crate) fn local(workspace: &WorkspaceState) -> Self {
        Self {
            qualification: workspace.flow_run_qualification(),
            plan_generation: workspace.flow_plan.generation().clone(),
            execution: None,
        }
    }
    pub(crate) fn same_plan(&self, workspace: &WorkspaceState) -> bool {
        self.plan_generation == *workspace.flow_plan.generation()
    }
    fn current_for(&self, workspace: &WorkspaceState) -> bool {
        self.qualification
            .same_input(&workspace.flow_run_qualification())
            && self.plan_generation == *workspace.flow_plan.generation()
    }
    fn belongs_to(&self, workspace: &WorkspaceState) -> bool {
        self.qualification
            .same_session(&workspace.flow_run_qualification())
    }
}
impl FlowRunReceipt {
    pub fn result_origin(
        &self,
        solver_implementation: &'static str,
        package: Option<ExecutedPropertyPackage>,
    ) -> FlowResultOrigin {
        FlowResultOrigin {
            qualification: self.qualification().clone(),
            plan_generation: self.plan_generation().clone(),
            execution: Some(FlowExecutionEvidence {
                plan: None,
                run_id: self.run_id().clone(),
                solver_implementation,
                package,
            }),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FlowFailureEvidence {
    Partitioned(std::sync::Arc<rf_solver::partitioned::PartitionedSolveFailure>),
    Simultaneous(std::sync::Arc<rf_solver::simultaneous::ModularFailure>),
    EquationOriented(std::sync::Arc<rf_solver::equation_oriented::EquationFailure>),
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FlowFailureRecord {
    pub evidence: Option<FlowFailureEvidence>,
    pub origin: FlowResultOrigin,
    pub diagnostic: DiagnosticSummary,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowResultUnavailable {
    NoResult,
    InputChanged,
    PlanChanged,
    DifferentSession,
    UnqualifiedSnapshot,
}
/// Borrowed values exist only in Current. Stale metadata cannot be used to read numbers.
#[derive(Debug)]
pub enum FlowResultView<'a> {
    Current {
        snapshot: &'a crate::FlowResultRecord,
        origin: &'a FlowResultOrigin,
    },
    Stale {
        origin: Option<&'a FlowResultOrigin>,
        reason: FlowResultUnavailable,
    },
    Missing {
        reason: FlowResultUnavailable,
    },
    Failed {
        diagnostic: &'a DiagnosticSummary,
        origin: &'a FlowResultOrigin,
    },
}
impl FlowResultView<'_> {
    pub fn status(&self) -> ResultStatus {
        match self {
            Self::Current { .. } => ResultStatus::Current,
            Self::Stale { .. } => ResultStatus::Stale,
            Self::Missing { .. } => ResultStatus::Missing,
            Self::Failed { .. } => ResultStatus::Failed,
        }
    }
}
impl WorkspaceState {
    pub fn flow_result(&self) -> FlowResultView<'_> {
        if let Some(snapshot) = latest_snapshot(self) {
            return match &snapshot.origin {
                Some(origin)
                    if origin.current_for(self)
                        && snapshot.matches_plan(self.flow_plan.current())
                        && snapshot.document_revision == self.document.revision =>
                {
                    FlowResultView::Current { snapshot, origin }
                }
                Some(origin) if origin.belongs_to(self) => FlowResultView::Stale {
                    origin: Some(origin),
                    reason: if origin.plan_generation != *self.flow_plan.generation() {
                        FlowResultUnavailable::PlanChanged
                    } else {
                        FlowResultUnavailable::InputChanged
                    },
                },
                Some(_) => FlowResultView::Missing {
                    reason: FlowResultUnavailable::DifferentSession,
                },
                None => FlowResultView::Missing {
                    reason: FlowResultUnavailable::UnqualifiedSnapshot,
                },
            };
        }
        if let Some(failure) = &self.flow_failure
            && failure.origin.current_for(self)
            && failure.diagnostic.document_revision == self.document.revision
            && self.solve_session.latest_diagnostic.as_ref() == Some(&failure.diagnostic)
        {
            return FlowResultView::Failed {
                diagnostic: &failure.diagnostic,
                origin: &failure.origin,
            };
        }
        if let Some(snapshot) = stale_snapshot(self) {
            return match &snapshot.origin {
                Some(origin) if origin.belongs_to(self) => FlowResultView::Stale {
                    origin: Some(origin),
                    reason: if origin.plan_generation != *self.flow_plan.generation() {
                        FlowResultUnavailable::PlanChanged
                    } else {
                        FlowResultUnavailable::InputChanged
                    },
                },
                Some(_) => FlowResultView::Missing {
                    reason: FlowResultUnavailable::DifferentSession,
                },
                None => FlowResultView::Stale {
                    origin: None,
                    reason: FlowResultUnavailable::UnqualifiedSnapshot,
                },
            };
        }
        FlowResultView::Missing {
            reason: FlowResultUnavailable::NoResult,
        }
    }
}
impl crate::AppState {
    /// Called by the completion publisher only after AF3 accepted its receipt.
    pub fn attach_flow_failure_origin(&mut self, origin: FlowResultOrigin) {
        if origin.current_for(&self.workspace)
            && let Some(failure) = &mut self.workspace.flow_failure
        {
            failure.origin = origin;
        }
    }
}

impl WorkspaceState {
    pub fn partitioned_failure_evidence(
        &self,
    ) -> Option<&rf_solver::partitioned::PartitionedSolveFailure> {
        if matches!(self.flow_result(), FlowResultView::Failed { .. }) {
            match self.flow_failure.as_ref()?.evidence.as_ref()? {
                FlowFailureEvidence::Partitioned(f) => Some(f.as_ref()),
                FlowFailureEvidence::Simultaneous(_) | FlowFailureEvidence::EquationOriented(_) => {
                    None
                }
            }
        } else {
            None
        }
    }
    pub fn attach_partitioned_failure(
        &mut self,
        failure: rf_solver::partitioned::PartitionedSolveFailure,
    ) {
        if let Some(record) = &mut self.flow_failure {
            record.evidence = Some(FlowFailureEvidence::Partitioned(std::sync::Arc::new(
                failure,
            )));
        }
    }
    pub fn simultaneous_failure_evidence(
        &self,
    ) -> Option<&rf_solver::simultaneous::ModularFailure> {
        if !matches!(self.flow_result(), FlowResultView::Failed { .. }) {
            return None;
        }
        match self.flow_failure.as_ref()?.evidence.as_ref()? {
            FlowFailureEvidence::Simultaneous(f) => Some(f.as_ref()),
            FlowFailureEvidence::Partitioned(_) | FlowFailureEvidence::EquationOriented(_) => None,
        }
    }
    pub fn attach_simultaneous_failure(
        &mut self,
        failure: rf_solver::simultaneous::ModularFailure,
    ) {
        if let Some(record) = &mut self.flow_failure {
            record.evidence = Some(FlowFailureEvidence::Simultaneous(std::sync::Arc::new(
                failure,
            )));
        }
    }
}

impl WorkspaceState {
    pub fn equation_failure_evidence(
        &self,
    ) -> Option<&rf_solver::equation_oriented::EquationFailure> {
        if !matches!(self.flow_result(), FlowResultView::Failed { .. }) {
            return None;
        }
        match self.flow_failure.as_ref()?.evidence.as_ref()? {
            FlowFailureEvidence::EquationOriented(f) => Some(f.as_ref()),
            _ => None,
        }
    }
    pub fn attach_equation_failure(
        &mut self,
        failure: rf_solver::equation_oriented::EquationFailure,
    ) {
        if let Some(record) = &mut self.flow_failure {
            record.evidence = Some(FlowFailureEvidence::EquationOriented(std::sync::Arc::new(
                failure,
            )));
        }
    }
}
