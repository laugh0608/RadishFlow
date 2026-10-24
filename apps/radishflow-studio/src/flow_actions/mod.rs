//! Application-local formal flow edits. No IO, runtime submission or editor selection.
//! Legacy adapters retain their contracts; this entry additionally binds the input session.
use crate::modeling_actions::{
    MaterialConnectionTarget, MaterialPortTarget, ModelingActionError, connect_material_target,
};
use rf_types::{RfError, UnitId};
use rf_ui::project_query::ProjectActionDescriptor;
use rf_ui::variable_browser::{ActionKind, VariableId, VariableValue};
use rf_ui::variable_commands::{VariableWriteError, VariableWriteRequest};
use rf_ui::{
    AppState, BuiltinUnitKind, DateTimeUtc, DocumentCommand, DocumentId, FlowRunQualification,
    WorkspaceState,
};

/// Supplied by the host for each call, never stored in a request or inferred from a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowEditAccess {
    Editable,
    ReadOnly,
}

/// Input-only optimistic guard: cancellation and equipment edits do not invalidate it.
/// Transaction clones retain identity; independently owned workspaces must reset their session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowEditContext(FlowRunQualification);
impl FlowEditContext {
    pub fn capture(workspace: &WorkspaceState) -> Self {
        Self(workspace.flow_run_qualification())
    }
    pub fn document_id(&self) -> &DocumentId {
        self.0.document_id()
    }
    pub fn revision(&self) -> u64 {
        self.0.revision()
    }
    fn validate(&self, workspace: &WorkspaceState) -> Result<(), FlowEditConflict> {
        let current = workspace.flow_run_qualification();
        if self.document_id() != current.document_id() {
            return Err(FlowEditConflict::DifferentDocument);
        }
        if !self.0.same_session(&current) {
            return Err(FlowEditConflict::DifferentSession);
        }
        if self.revision() != current.revision() {
            return Err(FlowEditConflict::Revision {
                expected: self.revision(),
                actual: current.revision(),
            });
        }
        if !self.0.same_input(&current) {
            return Err(FlowEditConflict::InputChanged);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowEditConflict {
    DifferentDocument,
    DifferentSession,
    Revision { expected: u64, actual: u64 },
    InputChanged,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowActionKind {
    Write,
    CreateUnit,
    Connect,
    Disconnect,
}

/// Maps discovery to this executor's implemented subset. Run and equipment remain discoverable
/// through AF4a, but are not accepted by this edit-only entry.
pub fn callable_flow_action(descriptor: &ProjectActionDescriptor) -> Option<FlowActionKind> {
    let ProjectActionDescriptor::Flow(descriptor) = descriptor else {
        return None;
    };
    match descriptor.kind {
        ActionKind::SetUnitParameter
        | ActionKind::RenameUnit
        | ActionKind::SetStreamSpecification => Some(FlowActionKind::Write),
        ActionKind::CreateUnit => Some(FlowActionKind::CreateUnit),
        ActionKind::ConnectPorts => Some(FlowActionKind::Connect),
        ActionKind::DisconnectPorts => Some(FlowActionKind::Disconnect),
        ActionKind::Run => None,
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum FlowAction {
    Write {
        variable: VariableId,
        value: VariableValue,
    },
    CreateUnit(BuiltinUnitKind),
    Connect(MaterialConnectionTarget),
    Disconnect(MaterialPortTarget),
}
impl FlowAction {
    pub fn kind(&self) -> FlowActionKind {
        match self {
            Self::Write { .. } => FlowActionKind::Write,
            Self::CreateUnit(_) => FlowActionKind::CreateUnit,
            Self::Connect(_) => FlowActionKind::Connect,
            Self::Disconnect(_) => FlowActionKind::Disconnect,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct FlowActionRequest {
    pub expected: FlowEditContext,
    pub action: FlowAction,
}
#[derive(Debug, Clone, PartialEq)]
pub enum FlowActionError {
    ReadOnly,
    Context(FlowEditConflict),
    PendingDrafts,
    PendingCanvasEdit,
    Write(VariableWriteError),
    Connection(ModelingActionError),
    PortNotBound(MaterialPortTarget),
    Rejected(RfError),
}
impl std::fmt::Display for FlowActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => f.write_str("project is read-only"),
            Self::Context(conflict) => write!(f, "flow input conflict: {conflict:?}"),
            Self::PendingDrafts => f.write_str("resolve Inspector drafts before editing"),
            Self::PendingCanvasEdit => f.write_str("finish or cancel the pending canvas edit"),
            Self::Write(error) => write!(f, "{error}"),
            Self::Connection(error) => write!(f, "{error}"),
            Self::PortNotBound(target) => {
                write!(f, "port {}.{} is not bound", target.unit_id, target.port)
            }
            Self::Rejected(error) => write!(f, "{error}"),
        }
    }
}
impl std::error::Error for FlowActionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write(e) => Some(e),
            Self::Connection(e) => Some(e),
            Self::Rejected(e) => Some(e),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowEditOutcome {
    Committed,
    Unchanged,
}
#[derive(Debug, Clone, PartialEq)]
pub enum FlowActionEffect {
    Written(VariableId),
    Created(UnitId),
    Connected(MaterialConnectionTarget),
    Disconnected(MaterialPortTarget),
}
#[derive(Debug, Clone, PartialEq)]
pub struct FlowActionReceipt {
    pub before: FlowEditContext,
    pub after: FlowEditContext,
    pub outcome: FlowEditOutcome,
    pub effect: FlowActionEffect,
    /// The existing domain command; absent for a validated no-op. This is not a replay DTO.
    pub command: Option<DocumentCommand>,
}
impl FlowActionReceipt {
    /// Changes revoke flow result/input qualification; equipment eligibility is untouched.
    pub fn invalidates_flow_results(&self) -> bool {
        self.outcome == FlowEditOutcome::Committed
    }
}

/// Exact validation on an isolated transaction clone. Does not reserve identities or authority.
/// A successful check must always be followed by execution against the live workspace.
pub fn preflight_flow_action(
    app: &AppState,
    access: FlowEditAccess,
    request: &FlowActionRequest,
) -> Result<(), FlowActionError> {
    stage_flow_edit(app, access, request, std::time::UNIX_EPOCH).map(|_| ())
}

pub fn execute_flow_action(
    app: &mut AppState,
    access: FlowEditAccess,
    request: &FlowActionRequest,
    changed_at: DateTimeUtc,
) -> Result<FlowActionReceipt, FlowActionError> {
    let (next, receipt) = stage_flow_edit(app, access, request, changed_at)?;
    if receipt.outcome == FlowEditOutcome::Committed {
        *app = next;
    }
    Ok(receipt)
}

fn stage_flow_edit(
    app: &AppState,
    access: FlowEditAccess,
    request: &FlowActionRequest,
    changed_at: DateTimeUtc,
) -> Result<(AppState, FlowActionReceipt), FlowActionError> {
    if access != FlowEditAccess::Editable {
        return Err(FlowActionError::ReadOnly);
    }
    request
        .expected
        .validate(&app.workspace)
        .map_err(FlowActionError::Context)?;
    if app.workspace.drafts.pending_count() > 0 {
        return Err(FlowActionError::PendingDrafts);
    }
    if app.workspace.canvas_interaction.pending_edit.is_some() {
        return Err(FlowActionError::PendingCanvasEdit);
    }
    let mut next = app.clone();
    let (effect, command) = match &request.action {
        FlowAction::Write { variable, value } => {
            let receipt = next
                .write_variable(
                    VariableWriteRequest {
                        variable: variable.clone(),
                        expected_revision: request.expected.revision(),
                        value: value.clone(),
                    },
                    changed_at,
                )
                .map_err(FlowActionError::Write)?;
            (FlowActionEffect::Written(receipt.variable), receipt.command)
        }
        FlowAction::CreateUnit(kind) => {
            let receipt = next
                .create_builtin_unit(*kind, changed_at)
                .map_err(FlowActionError::Rejected)?;
            (
                FlowActionEffect::Created(receipt.unit_id),
                Some(receipt.command),
            )
        }
        FlowAction::Connect(target) => {
            let command = connect_material_target(&mut next, target, changed_at)
                .map_err(FlowActionError::Connection)?;
            (FlowActionEffect::Connected(target.clone()), Some(command))
        }
        FlowAction::Disconnect(target) => {
            let command = next
                .disconnect_material_port(&target.unit_id, &target.port, changed_at)
                .map_err(FlowActionError::Rejected)?
                .ok_or_else(|| FlowActionError::PortNotBound(target.clone()))?;
            (
                FlowActionEffect::Disconnected(target.clone()),
                Some(command),
            )
        }
    };
    let outcome = if command.is_some() {
        FlowEditOutcome::Committed
    } else {
        FlowEditOutcome::Unchanged
    };
    let receipt = FlowActionReceipt {
        before: FlowEditContext::capture(&app.workspace),
        after: FlowEditContext::capture(&next.workspace),
        outcome,
        effect,
        command,
    };
    Ok((next, receipt))
}

#[cfg(test)]
mod tests;
