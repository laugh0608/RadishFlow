use super::{
    actions::*,
    errors,
    protocol::*,
    references::{Bindings, BoundOutput},
};
use crate::{
    flow_actions::*,
    headless::protocol as legacy,
    modeling_actions::{MaterialPortTarget, ModelingActionError, available_material_connections},
};
use rf_ui::{
    AppState,
    project_query::ProjectObjectId,
    variable_browser::{ObjectId, VariableValue},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub struct FlowReply {
    pub receipt: Value,
    pub outputs: BTreeMap<Output, BoundOutput>,
}

pub fn execute(
    app: &mut AppState,
    action: &Action,
    refs: &Bindings,
) -> Result<Option<FlowReply>, Failure> {
    let port = |p: &Port| -> Result<MaterialPortTarget, Failure> {
        Ok(MaterialPortTarget {
            unit_id: refs.resolve(&p.unit, Output::Unit)?.into(),
            port: p.port.clone(),
        })
    };
    let (expected, action) = match action {
        Action::Write {
            expected_revision,
            target,
            value,
        } => {
            let object = match refs.object(&target.object)? {
                ProjectObjectId::Flow(ObjectId::Unit(id)) => legacy::Object::Unit(id.to_string()),
                ProjectObjectId::Flow(ObjectId::Stream(id)) => {
                    legacy::Object::Stream(id.to_string())
                }
                _ => {
                    return Err(Failure::new(
                        Stage::Step,
                        Code::ReferenceKindMismatch,
                        "write requires a flow object",
                    ));
                }
            };
            let variable = legacy::Target {
                object,
                section: target.section.clone(),
                field: target.field.clone(),
            }
            .variable_id(&app.workspace.document.metadata.document_id);
            let value = match value {
                legacy::Value::Number(v) => VariableValue::Number(*v),
                legacy::Value::Text(v) => VariableValue::Text(v.clone()),
            };
            (expected_revision, FlowAction::Write { variable, value })
        }
        Action::CreateUnit {
            expected_revision,
            unit_kind,
        } => (
            expected_revision,
            FlowAction::CreateUnit((*unit_kind).into()),
        ),
        Action::Connect {
            expected_revision,
            source,
            sink,
            stream,
        } => {
            let source = port(source)?;
            let sink = sink.as_ref().map(port).transpose()?;
            let stream = stream
                .as_ref()
                .map(|s| refs.resolve(s, Output::Stream))
                .transpose()?;
            let mut matches = available_material_connections(app).into_iter().filter(|c| {
                c.source == source
                    && c.sink == sink
                    && stream.as_ref().is_none_or(|id| c.stream_id.as_str() == id)
            });
            let target = matches
                .next()
                .ok_or_else(|| errors::modeling(&ModelingActionError::ConnectionUnavailable))?;
            if matches.next().is_some() {
                return Err(errors::modeling(&ModelingActionError::AmbiguousConnection));
            }
            (expected_revision, FlowAction::Connect(target))
        }
        Action::Disconnect {
            expected_revision,
            port: p,
        } => (expected_revision, FlowAction::Disconnect(port(p)?)),
        Action::DisconnectStream {
            expected_revision,
            stream,
            direction,
        } => (
            expected_revision,
            FlowAction::DisconnectStream {
                stream: refs.resolve(stream, Output::Stream)?.into(),
                direction: direction.map(|d| match d {
                    Direction::Inlet => rf_types::PortDirection::Inlet,
                    Direction::Outlet => rf_types::PortDirection::Outlet,
                }),
            },
        ),
        _ => return Ok(None),
    };
    let doc = &app.workspace.document;
    refs.revision(
        expected,
        Output::Document,
        doc.metadata.document_id.as_str(),
        doc.revision,
    )?;
    let request = FlowActionRequest {
        expected: FlowEditContext::capture(&app.workspace),
        action,
    };
    let receipt = execute_flow_action(
        app,
        FlowEditAccess::Editable,
        &request,
        std::time::SystemTime::now(),
    )
    .map_err(errors::flow)?;
    let revision = receipt.after.revision();
    let mut outputs = BTreeMap::new();
    outputs.insert(
        Output::Document,
        BoundOutput {
            id: receipt.after.document_id().to_string(),
            revision,
        },
    );
    let p = |p: &MaterialPortTarget| json!({"unit":{"id":p.unit_id.as_str()},"port":p.port});
    let effect = match receipt.effect {
        FlowActionEffect::Written(id) => {
            json!({"kind":"written","target":legacy::Target::from(id)})
        }
        FlowActionEffect::Created(id) => {
            outputs.insert(
                Output::Unit,
                BoundOutput {
                    id: id.to_string(),
                    revision,
                },
            );
            json!({"kind":"created","unit":{"id":id.as_str()}})
        }
        FlowActionEffect::Connected(c) => {
            outputs.insert(
                Output::Stream,
                BoundOutput {
                    id: c.stream_id.to_string(),
                    revision,
                },
            );
            json!({"kind":"connected","stream":{"id":c.stream_id.as_str()},"source":p(&c.source),"sink":c.sink.as_ref().map(p)})
        }
        FlowActionEffect::Disconnected(port) => json!({"kind":"disconnected","port":p(&port)}),
        FlowActionEffect::StreamDisconnected {
            stream, direction, ..
        } => {
            json!({"kind":"stream_disconnected","stream":{"id":stream.as_str()},"direction":direction.map(|d|match d {rf_types::PortDirection::Inlet=>Direction::Inlet,rf_types::PortDirection::Outlet=>Direction::Outlet})})
        }
    };
    Ok(Some(FlowReply {
        receipt: json!({"outcome":match receipt.outcome {FlowEditOutcome::Committed=>"committed",FlowEditOutcome::Unchanged=>"unchanged"},"before_revision":receipt.before.revision(),"after_revision":revision,"effect":effect}),
        outputs,
    }))
}
