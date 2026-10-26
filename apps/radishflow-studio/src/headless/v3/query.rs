use super::{
    enums::*,
    issues,
    protocol::*,
    records,
    references::{Bindings, Executions, object_ref},
    results,
};
use crate::StudioProjectOpen;
use rf_model::engineering::BindingOrigin;
use rf_ui::{project_query::*, variable_browser as browser};
use serde_json::{Value, json};

pub fn queries(opened: &StudioProjectOpen, queries: &[Query], response: &mut Response) {
    let bindings = Bindings::default();
    let executions = Executions::default();
    for query in queries {
        let result = match opened {
            StudioProjectOpen::Editable(app) => editable(app, &query.query, &bindings, &executions),
            StudioProjectOpen::ReadOnly(project) => read_only(project, &query.query, &bindings),
        };
        push_reply(query, result, response);
    }
}
pub fn push_reply(query: &Query, result: Result<Value, Failure>, response: &mut Response) {
    let (value, error) = match result {
        Ok(value) => (Some(value), None),
        Err(mut e) => {
            e.stage = Stage::Query;
            e.query_id = Some(query.id.clone());
            response.fail(e.clone());
            (None, Some(e))
        }
    };
    response.queries.push(QueryReply {
        id: query.id.clone(),
        value,
        error,
    });
}
pub fn query_error(e: ProjectQueryError) -> Failure {
    match e {
        ProjectQueryError::DifferentDocument => {
            Failure::new(Stage::Query, Code::DifferentDocument, "different document")
        }
        ProjectQueryError::InputContextChanged => Failure::new(
            Stage::Query,
            Code::InputContextChanged,
            "input context changed",
        ),
        ProjectQueryError::ObjectMissing(id) => {
            Failure::new(Stage::Query, Code::ObjectMissing, "object does not exist")
                .target(&object_ref(&id))
        }
        ProjectQueryError::FieldMissing(_) => {
            Failure::new(Stage::Query, Code::FieldMissing, "field does not exist")
        }
        ProjectQueryError::NotEquipmentCase => Failure::new(
            Stage::Query,
            Code::ReferenceKindMismatch,
            "equipment results require an exact task/case target",
        ),
    }
}
pub fn editable(
    app: &rf_ui::AppState,
    kind: &QueryKind,
    bindings: &Bindings,
    executions: &Executions,
) -> Result<Value, Failure> {
    let q = ProjectQuery::new(&app.workspace);
    match kind {
        QueryKind::Objects {} => Ok(json!(
            q.objects().iter().map(descriptor).collect::<Vec<_>>()
        )),
        QueryKind::Fields { target } => {
            let reference = q.reference(bindings.object(target)?);
            Ok(json!(
                q.fields(&reference)
                    .map_err(query_error)?
                    .iter()
                    .map(field)
                    .collect::<Vec<_>>()
            ))
        }
        QueryKind::Record { target } => {
            let reference = q.reference(bindings.object(target)?);
            let value = match q.record(&reference).map_err(query_error)? {
                ProjectRecord::Flow(v) => {
                    json!({
                        "target": object_ref(&reference.object),
                        "name": v.name,
                        "model_kind": v.kind,
                        "revision": app.workspace.document.revision,
                        "fields": q.fields(&reference).map_err(query_error)?.iter().map(field).collect::<Vec<_>>()
                    })
                }
                ProjectRecord::Equipment(v) => records::task(v),
                ProjectRecord::SizingCase(v) => {
                    let ProjectObjectId::Case(member) = &reference.object else {
                        unreachable!("sizing case reference")
                    };
                    let revision = app
                        .workspace
                        .engineering
                        .inputs()
                        .task(&member.task_id)
                        .expect("queried task")
                        .revision;
                    json!({"id":v.id.as_str(),"revision":revision,"revision_owner":"task","data":records::SizingCaseData::from(v)})
                }
                ProjectRecord::RatingCase(v) => records::rating_case(v),
                ProjectRecord::Geometry(v) => records::geometry(v),
                ProjectRecord::Basis(v) => records::basis(v),
                ProjectRecord::Source(v) => records::source(v),
            };
            Ok(value)
        }
        QueryKind::Actions { scope } => {
            let reference = match scope {
                Scope::Project | Scope::Flow => None,
                Scope::Object(r) => Some(q.reference(bindings.object(r)?)),
            };
            let actions = q
                .actions(reference.as_ref(), browser::ActionExecution::Inline)
                .map_err(query_error)?;
            let mut descriptions: Vec<_> =
                actions.iter().map(|a| action_descriptor(a, app)).collect();
            if matches!(scope, Scope::Project) {
                for kind in [
                    "create_sizing_task",
                    "create_rating_task",
                    "create_source",
                    "create_basis",
                ] {
                    descriptions.push(adapter_action(kind, true));
                }
            }
            if let Some(reference) = &reference {
                match q.record(reference).map_err(query_error)? {
                    ProjectRecord::Equipment(
                        rf_model::equipment::EquipmentTask::SeparatorRating(_),
                    ) => {
                        for kind in [
                            "add_rating_case",
                            "reorder_rating_cases",
                            "delete_equipment_record",
                        ] {
                            descriptions.push(adapter_action(kind, true));
                        }
                    }
                    ProjectRecord::Equipment(_)
                    | ProjectRecord::RatingCase(_)
                    | ProjectRecord::Basis(_)
                    | ProjectRecord::Source(_) => {
                        descriptions.push(adapter_action("delete_equipment_record", true))
                    }
                    ProjectRecord::Flow(_)
                        if matches!(
                            reference.object,
                            ProjectObjectId::Flow(browser::ObjectId::Stream(_))
                        ) =>
                    {
                        descriptions.push(adapter_action("disconnect_stream", true))
                    }
                    _ => {}
                }
            }
            Ok(json!(descriptions))
        }
        QueryKind::Results { scope } => match scope {
            Scope::Project => Err(Failure::new(
                Stage::Query,
                Code::InvalidRequest,
                "results require flow or an exact equipment case",
            )),
            Scope::Flow => Ok(results::flow(app, executions)),
            Scope::Object(r) => Ok(results::equipment(
                q.equipment_result(&q.reference(bindings.object(r)?))
                    .map_err(query_error)?,
                executions,
            )),
        },
    }
}
fn descriptor(d: &ProjectObjectDescriptor) -> Value {
    let kind = match d.kind {
        ProjectObjectKind::Unit => "unit",
        ProjectObjectKind::Stream => "stream",
        ProjectObjectKind::Sizing => "sizing",
        ProjectObjectKind::Rating => "rating",
        ProjectObjectKind::Case => "case",
        ProjectObjectKind::Geometry => "geometry",
        ProjectObjectKind::Basis => "basis",
        ProjectObjectKind::Source => "source",
    };
    json!({
        "target": object_ref(&d.reference.object),
        "document_id": d.reference.document.as_str(),
        "parent": d.parent.as_ref().map(|p|object_ref(&p.object)),
        "kind": kind,
        "label": d.label
    })
}
fn field(d: &ProjectFieldDescriptor<'_>) -> Value {
    match &d.value {
        ProjectFieldValue::Flow(v) => flow_field(v.clone()),
        ProjectFieldValue::Engineering(v) => {
            let field = match &d.field {
                ProjectField::SizingInput(f) => json!({"sizing_input":SizingField::from(*f)}),
                ProjectField::RatingInput(f) => json!({"rating_input":RatingField::from(*f)}),
                ProjectField::GeometryInput(f) => json!({"geometry_input":GeometryField::from(*f)}),
                ProjectField::AxialSection(f) => json!({"axial_section":AxialSection::from(*f)}),
                ProjectField::BasisValue => json!("basis_value"),
                ProjectField::Flow { .. } => unreachable!("engineering field"),
            };
            let value = match &v.state {
                EngineeringQuantityState::Value {
                    value_si,
                    source,
                    origin,
                } => {
                    json!({
                        "state": "valid",
                        "value_si": value_si,
                        "source": records::source(source),
                        "origin": match origin {
                            BindingOrigin::Direct=>json!({
                                "kind": "direct"
                            }),
                            BindingOrigin::Basis(b)=>json!({
                                "kind": "basis",
                                "basis": records::basis(b)
                            }),
                            BindingOrigin::Override{
                                basis,
                                reason
                            }=>json!({
                                "kind": "override",
                                "basis": records::basis(basis),
                                "reason": reason
                            })
                        }
                    })
                }
                EngineeringQuantityState::Unspecified => json!({"state":"unspecified"}),
                EngineeringQuantityState::Invalid(e) => {
                    json!({"state":"invalid","error":issues::data(e)})
                }
            };
            json!({
                "target": object_ref(&d.object.object),
                "field": field,
                "quantity": v.quantity.definition().id,
                "unit": v.canonical_unit().definition().symbol,
                "value": value
            })
        }
    }
}
pub fn flow_field(v: browser::VariableDescriptor) -> Value {
    let quantity = v.quantity.map(|q| q.definition().id);
    let constraint=v.constraint.map(|c|json!({"minimum":c.minimum,"minimum_inclusive":c.minimum_inclusive,"maximum":c.maximum}));
    let note = v.note;
    let mut value: super::super::protocol::Variable = v.into();
    if !matches!(value.state, super::super::protocol::ValueState::Valid) {
        value.value = None;
    }
    json!({"variable":value,"quantity":quantity,"constraint":constraint,"note":note})
}
fn action_descriptor(d: &ProjectActionDescriptor, app: &rf_ui::AppState) -> Value {
    let (kind, modifies, undoable) = match d {
        ProjectActionDescriptor::Flow(v) => (
            match v.kind {
                browser::ActionKind::SetUnitParameter
                | browser::ActionKind::RenameUnit
                | browser::ActionKind::SetStreamSpecification => "write",
                browser::ActionKind::CreateUnit => "create_unit",
                browser::ActionKind::ConnectPorts => "connect",
                browser::ActionKind::DisconnectPorts => "disconnect",
                browser::ActionKind::Run => "run_flow",
            },
            v.modifies_document,
            v.undoable,
        ),
        ProjectActionDescriptor::Equipment(v) => (
            match &v.kind {
                EquipmentActionKind::Edit(target) => match target {
                    rf_ui::engineering::editor::EquipmentEditorTarget::Task(_) => {
                        "update_sizing_task"
                    }
                    rf_ui::engineering::editor::EquipmentEditorTarget::RatingTask(_) => {
                        "update_rating_labels"
                    }
                    rf_ui::engineering::editor::EquipmentEditorTarget::RatingGeometry(_) => {
                        "update_rating_geometry"
                    }
                    rf_ui::engineering::editor::EquipmentEditorTarget::RatingCase(_) => {
                        "update_rating_case"
                    }
                    rf_ui::engineering::editor::EquipmentEditorTarget::Source(_) => "update_source",
                    rf_ui::engineering::editor::EquipmentEditorTarget::Basis(_) => "update_basis",
                },
                EquipmentActionKind::Check(key) => {
                    if app
                        .workspace
                        .engineering
                        .inputs()
                        .task(&key.task_id)
                        .is_some()
                    {
                        "check_sizing"
                    } else {
                        "check_rating"
                    }
                }
                EquipmentActionKind::Size(_) => "size",
                EquipmentActionKind::Rate(_) => "rate",
            },
            v.modifies_inputs,
            v.undoable,
        ),
    };
    json!({"kind":kind,"execution":"inline","modifies_inputs":modifies,"undoable":undoable,"callable":!matches!(kind,"run_flow"|"size"|"rate")})
}

fn adapter_action(kind: &str, modifies: bool) -> Value {
    json!({"kind":kind,"execution":"inline","modifies_inputs":modifies,"undoable":modifies,"callable":true})
}

fn read_only(
    project: &rf_store::project_v3::ReadOnlyProjectV3,
    kind: &QueryKind,
    bindings: &Bindings,
) -> Result<Value, Failure> {
    // Only a borrowed flow-input projection. Unknown equipment never becomes an
    // editable AppState, and this path has no command or execution dispatcher.
    let stored = project.document();
    let mut document = rf_ui::FlowsheetDocument::new(
        stored.flowsheet.clone(),
        rf_ui::DocumentMetadata::new(
            stored.metadata.document_id.clone(),
            stored.metadata.title.clone(),
            stored.metadata.created_at,
        ),
    );
    document.revision = stored.revision;
    let browser = browser::VariableBrowser::new(&document, None, None);
    match kind {
        QueryKind::Objects {} => Ok(json!(
            browser
                .objects()
                .into_iter()
                .map(|v| {
                    let kind = match v.id {
                        browser::ObjectId::Unit(_) => "unit",
                        browser::ObjectId::Stream(_) => "stream",
                    };
                    json!({
                        "target": object_ref(&ProjectObjectId::Flow(v.id)),
                        "document_id": document.metadata.document_id.as_str(),
                        "parent": null,
                        "kind": kind,
                        "label": v.name
                    })
                })
                .collect::<Vec<_>>()
        )),
        QueryKind::Fields { target } | QueryKind::Record { target } => {
            let ProjectObjectId::Flow(id) = bindings.object(target)? else {
                return Err(Failure::new(
                    Stage::Query,
                    Code::UnsupportedCapability,
                    "equipment records are unavailable in this read-only project",
                )
                .target(target));
            };
            let object = browser.object(&id).map_err(|_| {
                Failure::new(Stage::Query, Code::ObjectMissing, "object does not exist")
                    .target(target)
            })?;
            let fields = browser
                .variables(&id)
                .expect("validated object")
                .into_iter()
                .filter(|v| v.id.section == browser::VariableSection::Inputs)
                .map(|mut v| {
                    v.write_via = None;
                    flow_field(v)
                })
                .collect::<Vec<_>>();
            if matches!(kind, QueryKind::Fields { .. }) {
                Ok(json!(fields))
            } else {
                Ok(
                    json!({"target":target,"name":object.name,"model_kind":object.kind,"revision":document.revision,"fields":fields}),
                )
            }
        }
        QueryKind::Actions { scope } => {
            if let Scope::Object(target) = scope {
                let ProjectObjectId::Flow(id) = bindings.object(target)? else {
                    return Err(Failure::new(
                        Stage::Query,
                        Code::UnsupportedCapability,
                        "equipment actions are unavailable in this read-only project",
                    )
                    .target(target));
                };
                browser.object(&id).map_err(|_| {
                    Failure::new(Stage::Query, Code::ObjectMissing, "object does not exist")
                        .target(target)
                })?;
            }
            Ok(json!([]))
        }
        QueryKind::Results { .. } => Err(Failure::new(
            Stage::Query,
            Code::UnsupportedCapability,
            "results are unavailable in this read-only project",
        )),
    }
}
