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
    session: Identity,
    input: Identity,
    epoch: Identity,
    run_id: FlowRunId,
    document_id: DocumentId,
    revision: u64,
    settled: bool,
}

impl FlowRunReceipt {
    pub fn run_id(&self) -> &FlowRunId {
        &self.run_id
    }

    pub fn revision(&self) -> u64 {
        self.revision
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

    pub fn pending_flow_run(&self) -> Option<&FlowRunId> {
        self.flow_run_acceptance.pending.as_ref()
    }
}

impl AppState {
    /// Called only after the owning application's preflight succeeds. The
    /// synchronous adapter submits one attempt at a time; scheduling and Busy
    /// belong to the application owner, not to this result qualification layer.
    pub fn prepare_flow_run(&mut self) -> FrozenFlowRun {
        let run_id = FlowRunId(Identity::fresh());
        let state = &mut self.workspace.flow_run_acceptance;
        state.pending = Some(run_id.clone());
        FrozenFlowRun {
            flowsheet: self.workspace.document.flowsheet.clone(),
            receipt: FlowRunReceipt {
                session: state.session.clone(),
                input: state.input.clone(),
                epoch: state.epoch.clone(),
                run_id,
                document_id: self.workspace.document.metadata.document_id.clone(),
                revision: self.workspace.document.revision,
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
        let reason = if state.session != receipt.session
            || self.workspace.document.metadata.document_id != receipt.document_id
        {
            Some(Reason::DifferentSession)
        } else if state.input != receipt.input
            || self.workspace.document.revision != receipt.revision
        {
            Some(Reason::InputChanged)
        } else if state.epoch != receipt.epoch {
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
