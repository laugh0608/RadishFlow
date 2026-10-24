//! Session-bound result reads. Execution evidence travels with the accepted result,
//! never with the current package selection or a second mutable result cache.
use crate::{
    DiagnosticSummary, DocumentId, FlowRunId, FlowRunQualification, FlowRunReceipt, SolveSnapshot,
    WorkspaceState, latest_snapshot, stale_snapshot,
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
pub struct FlowExecutionEvidence {
    pub run_id: FlowRunId,
    pub solver_implementation: &'static str,
    /// None when preparation failed before a validated package was obtained.
    pub package: Option<ExecutedPropertyPackage>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowResultOrigin {
    qualification: FlowRunQualification,
    execution: Option<FlowExecutionEvidence>,
}
impl FlowResultOrigin {
    pub fn document_id(&self) -> &DocumentId {
        self.qualification.document_id()
    }
    pub fn revision(&self) -> u64 {
        self.qualification.revision()
    }
    pub fn execution(&self) -> Option<&FlowExecutionEvidence> {
        self.execution.as_ref()
    }
    pub(crate) fn local(workspace: &WorkspaceState) -> Self {
        Self {
            qualification: workspace.flow_run_qualification(),
            execution: None,
        }
    }
    fn current_for(&self, workspace: &WorkspaceState) -> bool {
        self.qualification
            .same_input(&workspace.flow_run_qualification())
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
            execution: Some(FlowExecutionEvidence {
                run_id: self.run_id().clone(),
                solver_implementation,
                package,
            }),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FlowFailureRecord {
    pub origin: FlowResultOrigin,
    pub diagnostic: DiagnosticSummary,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowResultUnavailable {
    NoResult,
    InputChanged,
    DifferentSession,
    UnqualifiedSnapshot,
}
/// Borrowed values exist only in Current. Stale metadata cannot be used to read numbers.
#[derive(Debug)]
pub enum FlowResultView<'a> {
    Current {
        snapshot: &'a SolveSnapshot,
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
                        && snapshot.document_revision == self.document.revision =>
                {
                    FlowResultView::Current { snapshot, origin }
                }
                Some(origin) if origin.belongs_to(self) => FlowResultView::Stale {
                    origin: Some(origin),
                    reason: FlowResultUnavailable::InputChanged,
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
                    reason: FlowResultUnavailable::InputChanged,
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
