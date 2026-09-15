//! Revision-checked, application-local modeling actions. No window or transport is required.
//! Connections are selected from fresh local-rule candidates, never arbitrary stream payloads.

use crate::studio_local_rules::generate_local_canvas_suggestions;
use crate::{
    StudioAppAuthCacheContext, StudioAppFacade, WorkspaceControlAction,
    WorkspaceControlActionOutcome, WorkspaceRunPackageSelection,
    dispatch_workspace_control_action_with_auth_cache,
};
use rf_types::{RfError, StreamId, UnitId};
use rf_ui::{
    AppState, BuiltinUnitKind, CanvasSuggestedMaterialConnection, CanvasSuggestedStreamBinding,
    CanvasSuggestionAcceptance, DateTimeUtc, DocumentCommand, DocumentId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialPortTarget {
    pub unit_id: UnitId,
    pub port: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialConnectionTarget {
    pub stream_id: StreamId,
    pub source: MaterialPortTarget,
    pub sink: Option<MaterialPortTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelingAction {
    CreateUnit(BuiltinUnitKind),
    Connect(MaterialConnectionTarget),
    Run(WorkspaceRunPackageSelection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelingActionRequest {
    pub document_id: DocumentId,
    pub expected_revision: u64,
    pub action: ModelingAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelingActionEffect {
    Created {
        unit_id: UnitId,
        command: DocumentCommand,
    },
    Connected {
        target: MaterialConnectionTarget,
        command: DocumentCommand,
    },
    /// Inspect dispatch and control_state: a returned outcome may be blocked or failed.
    Run(Box<WorkspaceControlActionOutcome>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelingActionReceipt {
    pub document_id: DocumentId,
    pub revision: u64,
    pub effect: ModelingActionEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelingActionError {
    DifferentDocument,
    RevisionConflict { expected: u64, actual: u64 },
    PendingDrafts,
    PendingCanvasEdit,
    ConnectionUnavailable,
    AmbiguousConnection,
    Rejected(RfError),
}

impl std::fmt::Display for ModelingActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DifferentDocument => f.write_str("action targets a different document"),
            Self::RevisionConflict { expected, actual } => {
                write!(f, "expected document revision {expected}, found {actual}")
            }
            Self::PendingDrafts => {
                f.write_str("commit or discard Inspector drafts before invoking actions")
            }
            Self::PendingCanvasEdit => {
                f.write_str("finish or cancel the pending canvas edit before invoking actions")
            }
            Self::ConnectionUnavailable => {
                f.write_str("connection is not a current local-rule candidate")
            }
            Self::AmbiguousConnection => {
                f.write_str("connection matches more than one local-rule candidate")
            }
            Self::Rejected(error) => write!(f, "action rejected: {error}"),
        }
    }
}
impl std::error::Error for ModelingActionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Rejected(error) => Some(error),
            _ => None,
        }
    }
}

/// Read-only discovery, including controlled creation of an unbound outlet stream.
/// Pair these targets with the document ID/revision read from this same AppState.
pub fn available_material_connections(app: &AppState) -> Vec<MaterialConnectionTarget> {
    current_connections(app)
        .iter()
        .map(connection_target)
        .collect()
}

/// Edits are atomic and undoable. Run uses the normal synchronous readiness/diagnostic path;
/// it updates runtime state, never document history. The auth context is used only for Run.
pub fn dispatch_modeling_action(
    facade: &StudioAppFacade,
    app: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    request: ModelingActionRequest,
    changed_at: DateTimeUtc,
) -> Result<ModelingActionReceipt, ModelingActionError> {
    use ModelingActionError::*;
    if request.document_id != app.workspace.document.metadata.document_id {
        return Err(DifferentDocument);
    }
    if request.expected_revision != app.workspace.document.revision {
        return Err(RevisionConflict {
            expected: request.expected_revision,
            actual: app.workspace.document.revision,
        });
    }
    if !app.workspace.drafts.fields.is_empty() {
        return Err(PendingDrafts);
    }
    if app.workspace.canvas_interaction.pending_edit.is_some() {
        return Err(PendingCanvasEdit);
    }
    let effect = match request.action {
        ModelingAction::CreateUnit(kind) => {
            let created = app
                .create_builtin_unit(kind, changed_at)
                .map_err(Rejected)?;
            ModelingActionEffect::Created {
                unit_id: created.unit_id,
                command: created.command,
            }
        }
        ModelingAction::Connect(target) => {
            let connections = current_connections(app);
            let mut matching = connections
                .iter()
                .filter(|c| connection_target(c) == target);
            let connection = matching.next().ok_or(ConnectionUnavailable)?;
            if matching.next().is_some() {
                return Err(AmbiguousConnection);
            }
            let (command, _) = app
                .commit_material_connection(connection, changed_at)
                .map_err(Rejected)?;
            ModelingActionEffect::Connected { target, command }
        }
        ModelingAction::Run(package) => {
            let outcome = dispatch_workspace_control_action_with_auth_cache(
                facade,
                app,
                context,
                &WorkspaceControlAction::run_manual(package),
            )
            .map_err(Rejected)?;
            ModelingActionEffect::Run(Box::new(outcome))
        }
    };
    Ok(ModelingActionReceipt {
        document_id: request.document_id,
        revision: app.workspace.document.revision,
        effect,
    })
}

fn current_connections(app: &AppState) -> Vec<CanvasSuggestedMaterialConnection> {
    generate_local_canvas_suggestions(app)
        .into_iter()
        .filter(|s| s.can_accept_explicitly())
        .filter_map(|s| {
            s.acceptance
                .map(|CanvasSuggestionAcceptance::MaterialConnection(connection)| connection)
        })
        .collect()
}

fn connection_target(connection: &CanvasSuggestedMaterialConnection) -> MaterialConnectionTarget {
    let stream_id = match &connection.stream {
        CanvasSuggestedStreamBinding::Existing { stream_id } => stream_id.clone(),
        CanvasSuggestedStreamBinding::Create { stream } => stream.id.clone(),
    };
    MaterialConnectionTarget {
        stream_id,
        source: MaterialPortTarget {
            unit_id: connection.source_unit_id.clone(),
            port: connection.source_port.clone(),
        },
        sink: connection
            .sink_unit_id
            .as_ref()
            .zip(connection.sink_port.as_ref())
            .map(|(id, port)| MaterialPortTarget {
                unit_id: id.clone(),
                port: port.clone(),
            }),
    }
}

#[cfg(test)]
mod tests;
