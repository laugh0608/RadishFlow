//! In-memory qualification for flowsheet completions. These identities never
//! enter project storage, undo history, or the numeric solver protocol.
use std::sync::Arc;

use rf_model::Flowsheet;

use crate::{AppState, DocumentId, WorkspaceState};

#[derive(Debug, Clone)]
struct Identity(Arc<()>);

impl Identity {
    fn fresh() -> Self {
        Self(Arc::new(()))
    }
}

impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Identity {}

/// Opaque, non-reusable attempt identity, independent of snapshot numbering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowRunId(Identity);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowRunAcceptance {
    session: Identity,
    input: Identity,
    epoch: Identity,
    pending: Option<FlowRunId>,
}

impl FlowRunAcceptance {
    pub(crate) fn new() -> Self {
        Self {
            session: Identity::fresh(),
            input: Identity::fresh(),
            epoch: Identity::fresh(),
            pending: None,
        }
    }

    pub(crate) fn input_changed(&mut self) {
        self.input = Identity::fresh();
        self.pending = None;
    }

    pub(crate) fn revoke(&mut self) {
        self.epoch = Identity::fresh();
        self.pending = None;
    }
}

/// A receipt has a single owner. A completion may be presented repeatedly by
/// mutable reference, but its first disposition is final, including rejection.
/// Deliberately not Clone: transaction clones must not duplicate completion
/// consumption or share mutable settlement flags with the committed owner.
#[derive(Debug)]
pub struct FlowRunReceipt {
    qualification: FlowRunQualification,
    run_id: FlowRunId,
    settled: bool,
}

/// Read-only identity used for duplicate submissions and the start handshake.
/// Cloning this key does not grant completion or cancellation ownership.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowRunQualification {
    session: Identity,
    input: Identity,
    epoch: Identity,
    document_id: DocumentId,
    revision: u64,
}

impl FlowRunQualification {
    pub fn document_id(&self) -> &DocumentId {
        &self.document_id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn same_input(&self, other: &Self) -> bool {
        self.same_session(other) && self.input == other.input && self.revision == other.revision
    }

    pub fn same_session(&self, other: &Self) -> bool {
        self.session == other.session && self.document_id == other.document_id
    }
}

impl FlowRunReceipt {
    pub fn qualification(&self) -> &FlowRunQualification {
        &self.qualification
    }

    pub fn run_id(&self) -> &FlowRunId {
        &self.run_id
    }

    pub fn revision(&self) -> u64 {
        self.qualification.revision
    }
}

/// Frozen formal input. Presentation, drafts, and equipment calculations are
/// intentionally absent. Execute against this copy, never against live state.
#[derive(Debug)]
pub struct FrozenFlowRun {
    flowsheet: Flowsheet,
    receipt: FlowRunReceipt,
}

impl FrozenFlowRun {
    pub fn flowsheet(&self) -> &Flowsheet {
        &self.flowsheet
    }

    pub fn receipt(&self) -> &FlowRunReceipt {
        &self.receipt
    }

    pub fn into_parts(self) -> (Flowsheet, FlowRunReceipt) {
        (self.flowsheet, self.receipt)
    }

    pub fn into_receipt(self) -> FlowRunReceipt {
        self.receipt
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowRunDiscardReason {
    DifferentSession,
    InputChanged,
    AcceptanceRevoked,
    Superseded,
    AlreadySettled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowRunDisposition {
    Applied,
    Discarded(FlowRunDiscardReason),
}

impl WorkspaceState {
    /// Clone preserves a transaction's session. An independent flowsheet owner
    /// must call this on its clone (or construct a new WorkspaceState).
    pub fn reset_flow_run_session(&mut self) {
        self.flow_run_acceptance = FlowRunAcceptance::new();
        self.clear_results();
    }

    /// Revoke unreceived completions without erasing already accepted results.
    pub fn revoke_flow_run_acceptance(&mut self) {
        self.flow_run_acceptance.revoke();
    }

    /// Revoke exactly this attempt; an old task cannot revoke a newer pending run.
    pub fn revoke_flow_run(&mut self, id: &FlowRunId) -> bool {
        if self.flow_run_acceptance.pending.as_ref() != Some(id) {
            return false;
        }
        self.revoke_flow_run_acceptance();
        true
    }

    pub fn pending_flow_run(&self) -> Option<&FlowRunId> {
        self.flow_run_acceptance.pending.as_ref()
    }
}

impl WorkspaceState {
    pub fn flow_run_qualification(&self) -> FlowRunQualification {
        let state = &self.flow_run_acceptance;
        FlowRunQualification {
            session: state.session.clone(),
            input: state.input.clone(),
            epoch: state.epoch.clone(),
            document_id: self.document.metadata.document_id.clone(),
            revision: self.document.revision,
        }
    }
}
impl AppState {
    pub fn flow_run_qualification(&self) -> FlowRunQualification {
        self.workspace.flow_run_qualification()
    }

    /// Called only after the owning application's preflight succeeds. The
    /// synchronous adapter submits one attempt at a time; scheduling and Busy
    /// belong to the application owner, not to this result qualification layer.
    pub fn prepare_flow_run(&mut self) -> FrozenFlowRun {
        let qualification = self.flow_run_qualification();
        let run_id = FlowRunId(Identity::fresh());
        let state = &mut self.workspace.flow_run_acceptance;
        state.pending = Some(run_id.clone());
        FrozenFlowRun {
            flowsheet: self.workspace.document.flowsheet.clone(),
            receipt: FlowRunReceipt {
                qualification,
                run_id,
                settled: false,
            },
        }
    }

    /// Consume qualification before publishing either success or failure. The
    /// application keeps its exclusive AppState borrow through publication.
    /// Rejected receipts never clear another attempt's pending slot.
    pub fn accept_flow_run(&mut self, receipt: &mut FlowRunReceipt) -> FlowRunDisposition {
        use FlowRunDiscardReason as Reason;
        if receipt.settled {
            return FlowRunDisposition::Discarded(Reason::AlreadySettled);
        }
        receipt.settled = true;
        let state = &mut self.workspace.flow_run_acceptance;
        let reason = if state.session != receipt.qualification.session
            || self.workspace.document.metadata.document_id != receipt.qualification.document_id
        {
            Some(Reason::DifferentSession)
        } else if state.input != receipt.qualification.input
            || self.workspace.document.revision != receipt.qualification.revision
        {
            Some(Reason::InputChanged)
        } else if state.epoch != receipt.qualification.epoch {
            Some(Reason::AcceptanceRevoked)
        } else if state.pending.as_ref() != Some(&receipt.run_id) {
            Some(Reason::Superseded)
        } else {
            None
        };
        if let Some(reason) = reason {
            return FlowRunDisposition::Discarded(reason);
        }
        state.pending = None;
        FlowRunDisposition::Applied
    }
}
