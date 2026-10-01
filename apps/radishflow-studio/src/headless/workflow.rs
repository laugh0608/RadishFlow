//! Sequential v2 modeling operations. Only successful prior steps bind identities.
use super::{apply_write, document, failure, protocol::*};
use crate::modeling_actions::{
    MaterialPortTarget, ModelingAction, ModelingActionEffect, ModelingActionError,
    ModelingActionRequest, available_material_connections, dispatch_modeling_action,
};
use crate::{StudioAppAuthCacheContext, StudioAppFacade};
use rf_ui::{AppState, BuiltinUnitKind};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::SystemTime};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityReference {
    Id(String),
    Step(String),
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectReference {
    Unit(IdentityReference),
    Stream(IdentityReference),
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceTarget {
    pub object: ObjectReference,
    pub section: Section,
    pub field: Field,
}
impl From<Target> for ReferenceTarget {
    fn from(target: Target) -> Self {
        Self {
            object: match target.object {
                Object::Unit(id) => ObjectReference::Unit(IdentityReference::Id(id)),
                Object::Stream(id) => ObjectReference::Stream(IdentityReference::Id(id)),
            },
            section: target.section,
            field: target.field,
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortReference {
    unit: IdentityReference,
    port: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitKind {
    Feed,
    Heater,
    Cooler,
    Valve,
    Mixer,
    FlashDrum,
}
impl From<UnitKind> for BuiltinUnitKind {
    fn from(kind: UnitKind) -> Self {
        match kind {
            UnitKind::Feed => Self::Feed,
            UnitKind::Heater => Self::Heater,
            UnitKind::Cooler => Self::Cooler,
            UnitKind::Valve => Self::Valve,
            UnitKind::Mixer => Self::Mixer,
            UnitKind::FlashDrum => Self::FlashDrum,
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub action: StepAction,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StepAction {
    CreateUnit {
        unit_kind: UnitKind,
    },
    Connect {
        source: PortReference,
        // Null/omitted means a controlled unbound outlet, not an inferred destination.
        sink: Option<PortReference>,
        stream: Option<IdentityReference>,
    },
    Write {
        target: ReferenceTarget,
        value: Value,
    },
}
#[derive(Debug, Serialize)]
pub struct StepReceipt {
    pub id: String,
    pub index: usize,
    pub revision: u64,
    pub effect: StepEffect,
}
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StepEffect {
    Created {
        unit_id: String,
    },
    Connected {
        stream_id: String,
        source: Port,
        sink: Option<Port>,
    },
    Written {
        target: Target,
        changed: bool,
    },
}

#[derive(Default)]
pub struct Bindings(BTreeMap<String, BoundIdentity>);
enum BoundIdentity {
    Unit(String),
    Stream(String),
    NoIdentity,
}
#[derive(Clone, Copy)]
enum IdentityKind {
    Unit,
    Stream,
}
impl Bindings {
    fn resolve(&self, reference: IdentityReference, kind: IdentityKind) -> Result<String, Failure> {
        match reference {
            IdentityReference::Id(id) => Ok(id),
            IdentityReference::Step(step) => match (self.0.get(&step), kind) {
                (Some(BoundIdentity::Unit(id)), IdentityKind::Unit)
                | (Some(BoundIdentity::Stream(id)), IdentityKind::Stream) => Ok(id.clone()),
                (None, _) => Err(failure(
                    "step",
                    "unknown_step_reference",
                    format!("step {step:?} has not completed"),
                )),
                _ => Err(failure(
                    "step",
                    "reference_kind_mismatch",
                    format!("step {step:?} does not return the required object kind"),
                )),
            },
        }
    }
    pub fn resolve_target(&self, target: ReferenceTarget) -> Result<Target, Failure> {
        Ok(Target {
            object: match target.object {
                ObjectReference::Unit(r) => Object::Unit(self.resolve(r, IdentityKind::Unit)?),
                ObjectReference::Stream(r) => {
                    Object::Stream(self.resolve(r, IdentityKind::Stream)?)
                }
            },
            section: target.section,
            field: target.field,
        })
    }
    fn port(&self, port: PortReference) -> Result<MaterialPortTarget, Failure> {
        Ok(MaterialPortTarget {
            unit_id: self.resolve(port.unit, IdentityKind::Unit)?.into(),
            port: port.port,
        })
    }
}

pub fn execute(
    app: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    steps: Vec<Step>,
    bindings: &mut Bindings,
    response: &mut Response,
) -> Result<(), Failure> {
    for (index, step) in steps.into_iter().enumerate() {
        let result = if step.id.trim().is_empty() {
            Err(failure(
                "step",
                "empty_step_id",
                "step id must not be blank",
            ))
        } else if bindings.0.contains_key(&step.id) {
            Err(failure(
                "step",
                "duplicate_step_id",
                format!("step {:?} already completed", step.id),
            ))
        } else {
            execute_action(app, context, step.action, bindings)
        };
        let effect = result.map_err(|mut error| {
            error.stage = "step";
            error.index = Some(index);
            error
        })?;
        let identity = match &effect {
            StepEffect::Created { unit_id } => BoundIdentity::Unit(unit_id.clone()),
            StepEffect::Connected { stream_id, .. } => BoundIdentity::Stream(stream_id.clone()),
            StepEffect::Written { .. } => BoundIdentity::NoIdentity,
        };
        bindings.0.insert(step.id.clone(), identity);
        response.steps.get_or_insert_default().push(StepReceipt {
            id: step.id,
            index,
            revision: app.workspace.document.revision,
            effect,
        });
        response.document = Some(document(app));
    }
    Ok(())
}

fn execute_action(
    app: &mut AppState,
    context: &StudioAppAuthCacheContext<'_>,
    action: StepAction,
    bindings: &Bindings,
) -> Result<StepEffect, Failure> {
    let action = match action {
        StepAction::Write { target, value } => {
            let target = bindings.resolve_target(target)?;
            let receipt = apply_write(app, Write { target, value })?;
            return Ok(StepEffect::Written {
                target: receipt.target,
                changed: receipt.changed,
            });
        }
        StepAction::CreateUnit { unit_kind } => ModelingAction::CreateUnit(unit_kind.into()),
        StepAction::Connect {
            source,
            sink,
            stream,
        } => {
            let source = bindings.port(source)?;
            let sink = sink.map(|p| bindings.port(p)).transpose()?;
            let stream = stream
                .map(|r| bindings.resolve(r, IdentityKind::Stream))
                .transpose()?;
            let candidates = available_material_connections(app);
            let mut matches = candidates.into_iter().filter(|c| {
                c.source == source
                    && c.sink == sink
                    && stream.as_ref().is_none_or(|id| c.stream_id.as_str() == id)
            });
            let target = matches
                .next()
                .ok_or_else(|| action_failure(ModelingActionError::ConnectionUnavailable))?;
            if matches.next().is_some() {
                return Err(action_failure(ModelingActionError::AmbiguousConnection));
            }
            ModelingAction::Connect(target)
        }
    };
    let request = ModelingActionRequest {
        document_id: app.workspace.document.metadata.document_id.clone(),
        expected_revision: app.workspace.document.revision,
        action,
    };
    let receipt = dispatch_modeling_action(
        &StudioAppFacade::new(),
        app,
        context,
        request,
        SystemTime::now(),
    )
    .map_err(action_failure)?;
    match receipt.effect {
        ModelingActionEffect::Created { unit_id, .. } => Ok(StepEffect::Created {
            unit_id: unit_id.to_string(),
        }),
        ModelingActionEffect::Connected { target, .. } => Ok(StepEffect::Connected {
            stream_id: target.stream_id.to_string(),
            source: Port {
                unit_id: target.source.unit_id.to_string(),
                port: target.source.port,
            },
            sink: target.sink.map(|p| Port {
                unit_id: p.unit_id.to_string(),
                port: p.port,
            }),
        }),
        ModelingActionEffect::Run(_) => Err(failure(
            "step",
            "unexpected_action",
            "expected create or connect outcome",
        )),
    }
}
fn action_failure(error: ModelingActionError) -> Failure {
    let code = match &error {
        ModelingActionError::DifferentDocument => "different_document",
        ModelingActionError::RevisionConflict { .. } => "revision_conflict",
        ModelingActionError::PendingDrafts => "pending_drafts",
        ModelingActionError::PendingCanvasEdit => "pending_canvas_edit",
        ModelingActionError::ConnectionUnavailable => "connection_unavailable",
        ModelingActionError::AmbiguousConnection => "ambiguous_connection",
        ModelingActionError::Rejected(error) => error.code().as_str(),
    };
    failure("step", code, error.to_string())
}
