use super::*;

impl Fixture {
    fn execute(&self, request: &Value) -> (i32, Value) {
        let path = self.write("execute.json", request);
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = execute_args(&[path.into_os_string()], &mut out, &mut err);
        assert!(err.is_empty());
        (code, serde_json::from_slice(&out).unwrap())
    }
    fn execution_request(&self, project: &Value, steps: Value) -> Value {
        self.write("project.json", project);
        let (code, query) =
            self.query(json!({"schema_version":3,"project":"project.json","queries":[]}));
        assert_eq!(code, 0, "{query}");
        json!({"schema_version":3,"project":"project.json","expected_project":{"document_id":query["project"]["document_id"],"file_sha256":query["project"]["file_sha256"]},"steps":steps,"queries":[]})
    }
}
fn reference(step: &str, output: &str) -> Value {
    json!({"step":step,"output":output})
}
fn record_query(id: &str, target: Value) -> Value {
    json!({"id":id,"query":{"kind":"record","target":target}})
}

#[test]
fn execute_snapshot_conflict_and_read_only_are_preoperation_failures() {
    let f = Fixture::new();
    let mut request = f.execution_request(&equipment_project(), json!([]));
    request["expected_project"]["file_sha256"] = json!("different");
    let (code, r) = f.execute(&request);
    assert_eq!(code, 2);
    assert_eq!(r["error"]["code"], "project_conflict");
    assert_eq!(r["steps"], json!([]));
    request["expected_project"]["document_id"] = json!("another-document");
    let (_, r) = f.execute(&request);
    assert_eq!(r["error"]["code"], "different_document");
    let mut project = equipment_project();
    project["engineering"]["tasks"][0]["method"]["version"] = json!(999);
    let request = f.execution_request(
        &project,
        json!([{ "id":"create","action":{"kind":"create_source"}}]),
    );
    let (code, r) = f.execute(&request);
    assert_eq!(code, 3, "{r}");
    assert_eq!(r["error"]["code"], "read_only_project");
    assert_eq!(r["steps"], json!([]));
}

#[test]
fn flow_modeling_uses_receipt_versions_and_preserves_prior_steps_on_failure() {
    let f = Fixture::new();
    let mut request=f.execution_request(&empty_project(),json!([
        {"id":"feed","action":{"kind":"create_unit","unit_kind":"feed","expected_revision":0}},
        {"id":"heater","action":{"kind":"create_unit","unit_kind":"heater","expected_revision":reference("feed","document")}},
        {"id":"rename","action":{"kind":"write","expected_revision":reference("heater","document"),"target":{"object":{"unit":reference("feed","unit")},"section":"inputs","field":"name"},"value":"Named feed"}},
        {"id":"same","action":{"kind":"write","expected_revision":reference("rename","document"),"target":{"object":{"unit":reference("feed","unit")},"section":"inputs","field":"name"},"value":"Named feed"}},
        {"id":"stale","action":{"kind":"create_unit","unit_kind":"valve","expected_revision":0}},
        {"id":"unreached","action":{"kind":"create_source"}}
    ]));
    request["queries"] = json!([record_query("feed",json!({"unit":reference("feed","unit")})),{"id":"missing","query":{"kind":"record","target":{"unit":{"id":"absent"}}}}]);
    let before = fs::read(f.0.join("project.json")).unwrap();
    let (code, r) = f.execute(&request);
    assert_eq!(code, 2, "{r}");
    assert_eq!(r["steps"].as_array().unwrap().len(), 5);
    assert_eq!(r["steps"][3]["outcome"], "unchanged");
    assert_eq!(
        r["steps"][3]["before_revision"],
        r["steps"][3]["after_revision"]
    );
    assert_eq!(r["error"]["step_id"], "stale");
    assert_eq!(r["queries"][0]["value"]["name"], "Named feed");
    assert_eq!(fs::read(f.0.join("project.json")).unwrap(), before);
}

#[test]
fn device_versions_never_borrow_flow_or_other_records_and_check_is_not_run() {
    let f = Fixture::new();
    let mut request=f.execution_request(&empty_project(),json!([
        {"id":"sizing","action":{"kind":"create_sizing_task"}},
        {"id":"rating","action":{"kind":"create_rating_task"}},
        {"id":"check","action":{"kind":"check_sizing","task":reference("sizing","task")}},
        {"id":"wrong","action":{"kind":"update_rating_labels","task":reference("rating","task"),"expected_revision":reference("sizing","task"),"data":{"tag":"R","name":"Wrong owner"}}}
    ]));
    request["queries"] = json!([record_query("rating",json!({"equipment":reference("rating","task")})),{"id":"results","query":{"kind":"results","scope":{"object":{"case":{"task":reference("sizing","task"),"case":reference("sizing","case")}}}}}]);
    let (code, r) = f.execute(&request);
    assert_eq!(code, 2, "{r}");
    assert_eq!(r["steps"][2]["outcome"], "checked");
    assert!(!r["steps"][2]["issues"].as_array().unwrap().is_empty());
    assert_eq!(r["error"]["code"], "reference_kind_mismatch");
    assert_eq!(r["queries"][1]["value"]["status"], "missing");
    assert_eq!(r["project"]["flow_revision"], 0);
}

#[test]
fn confirmation_replays_and_mismatch_never_publishes_current_step() {
    let f = Fixture::new();
    let mut request=f.execution_request(&empty_project(),json!([
        {"id":"source","action":{"kind":"create_source"}},
        {"id":"delete","action":{"kind":"delete_equipment_record","target":{"source":reference("source","source")},"expected_revision":reference("source","source")}},
        {"id":"after","action":{"kind":"create_rating_task"}}
    ]));
    request["queries"] = json!([{"id":"objects","query":{"kind":"objects"}}]);
    let (code, first) = f.execute(&request);
    assert_eq!(code, 6, "{first}");
    assert_eq!(first["steps"].as_array().unwrap().len(), 2);
    assert_eq!(first["queries"][0]["value"].as_array().unwrap().len(), 1);
    let digest = first["error"]["details"]["confirmation_digest"].clone();
    assert!(digest.as_str().unwrap().starts_with("af4c-confirm-v1:"));
    let (_, again) = f.execute(&request);
    assert_eq!(again["error"]["details"]["confirmation_digest"], digest);
    request["steps"][1]["confirmation"] = json!({"digest":"wrong"});
    let (code, r) = f.execute(&request);
    assert_eq!(code, 6);
    assert_eq!(r["error"]["code"], "confirmation_mismatch");
    assert_eq!(r["queries"][0]["value"].as_array().unwrap().len(), 1);
    request["steps"][1]["confirmation"] = json!({"digest":digest});
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["steps"][1]["outcome"], "committed");
    assert_eq!(r["steps"].as_array().unwrap().len(), 3);
    request["steps"][0]["id"] = json!("changed");
    request["steps"][1]["action"]["target"]["source"]["step"] = json!("changed");
    request["steps"][1]["action"]["expected_revision"]["step"] = json!("changed");
    let (code, r) = f.execute(&request);
    assert_eq!(code, 6);
    assert_eq!(r["error"]["code"], "confirmation_mismatch");
}

#[test]
fn unnecessary_confirmation_rolls_back_a_successful_domain_edit() {
    let f = Fixture::new();
    let mut request = f.execution_request(
        &empty_project(),
        json!([
            {"id":"source","action":{"kind":"create_source"},"confirmation":{"digest":"unneeded"}}
        ]),
    );
    request["queries"] = json!([{"id":"objects","query":{"kind":"objects"}}]);
    let (code, r) = f.execute(&request);
    assert_eq!(code, 2);
    assert_eq!(r["queries"][0]["value"], json!([]));
}

#[test]
fn execute_strictly_rejects_unknown_unit_variant_fields_before_any_steps() {
    let f = Fixture::new();
    let request = f.execution_request(
        &empty_project(),
        json!([
            {"id":"valid","action":{"kind":"create_source"}},
            {"id":"invalid","action":{"kind":"create_rating_task","extra":true}}
        ]),
    );
    let (code, r) = f.execute(&request);
    assert_eq!(code, 2);
    assert_eq!(r["steps"], json!([]));
    assert_eq!(r["error"]["stage"], "request");
}

fn rating_project() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../crates/rf-store/tests/fixtures/project-v3-rating.json"
    ))
    .unwrap()
}
#[test]
fn method_upgrade_preview_includes_versions_and_new_case_without_publishing() {
    let f = Fixture::new();
    let p = rating_project();
    let revision = p["engineering"]["tasks"][0]["revision"].clone();
    let mut request=f.execution_request(&p,json!([
        {"id":"add","action":{"kind":"add_rating_case","task":{"id":"rating-1"},"expected_revision":revision,"copy_from":{"id":"rating-case-1"}}}
    ]));
    request["queries"] = json!([record_query(
        "rating",
        json!({"equipment":{"id":"rating-1"}})
    )]);
    let (code, r) = f.execute(&request);
    assert_eq!(code, 6, "{r}");
    assert_eq!(r["queries"][0]["value"]["method"]["version"], 1);
    assert_eq!(
        r["error"]["details"]["preview"]["after"]["tasks"][0]["method"]["version"],
        2
    );
    assert_eq!(
        r["error"]["details"]["preview"]["after"]["tasks"][0]["cases"][1]["id"],
        reference("add", "case")
    );
    request["steps"][0]["confirmation"] =
        json!({"digest":r["error"]["details"]["confirmation_digest"]});
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["queries"][0]["value"]["method"]["version"], 2);
    request["steps"].as_array_mut().unwrap().push(json!({"id":"order","action":{"kind":"reorder_rating_cases","task":{"id":"rating-1"},"expected_revision":reference("add","task"),"cases":[reference("add","case"),{"id":"rating-case-1"}]}}));
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(
        r["queries"][0]["value"]["cases"][0]["id"],
        r["steps"][0]["outputs"]["case"]["id"]
    );
}

#[test]
fn source_and_basis_updates_preserve_si_metadata_and_require_impact_review() {
    let f = Fixture::new();
    let mut request = f.execution_request(&rating_project(), json!([]));
    request["queries"] = json!([record_query("basis", json!({"basis":{"id":"basis-time"}}))]);
    let (_, initial) = f.execute(&request);
    let record = &initial["queries"][0]["value"];
    let mut data = record["data"].clone();
    data["value"]["value_si"] = json!(123.0);
    request["steps"] = json!([{ "id":"basis","action":{"kind":"update_basis","basis":{"id":"basis-time"},"expected_revision":record["revision"],"data":data}}]);
    let (code, r) = f.execute(&request);
    assert_eq!(code, 6, "{r}");
    assert_eq!(r["queries"][0]["value"], *record);
    assert!(
        !r["error"]["details"]["preview"]["impacts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    request["steps"][0]["confirmation"] =
        json!({"digest":r["error"]["details"]["confirmation_digest"]});
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["queries"][0]["value"]["data"]["value"]["value_si"], 123.0);
    assert_eq!(
        r["queries"][0]["value"]["data"]["source"],
        record["data"]["source"]
    );
}

#[test]
fn connection_and_both_disconnect_modes_use_controlled_domain_transactions() {
    let f = Fixture::new();
    let request=f.execution_request(&empty_project(),json!([
        {"id":"feed","action":{"kind":"create_unit","unit_kind":"feed","expected_revision":0}},
        {"id":"out","action":{"kind":"connect","expected_revision":reference("feed","document"),"source":{"unit":reference("feed","unit"),"port":"outlet"}}},
        {"id":"heater","action":{"kind":"create_unit","unit_kind":"heater","expected_revision":reference("out","document")}},
        {"id":"join","action":{"kind":"connect","expected_revision":reference("heater","document"),"source":{"unit":reference("feed","unit"),"port":"outlet"},"sink":{"unit":reference("heater","unit"),"port":"inlet"},"stream":reference("out","stream")}},
        {"id":"end","action":{"kind":"disconnect","expected_revision":reference("join","document"),"port":{"unit":reference("feed","unit"),"port":"outlet"}}},
        {"id":"all","action":{"kind":"disconnect_stream","expected_revision":reference("end","document"),"stream":reference("out","stream")}},
        {"id":"same","action":{"kind":"disconnect_stream","expected_revision":reference("all","document"),"stream":reference("out","stream")}}
    ]));
    let (code, r) = f.execute(&request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["steps"][6]["outcome"], "unchanged");
    assert_eq!(
        r["steps"][5]["after_revision"].as_u64().unwrap(),
        r["steps"][5]["before_revision"].as_u64().unwrap() + 1
    );
}

#[test]
fn confirmation_normalizes_typed_allocations_but_never_display_strings() {
    use super::super::{confirmation::Aliases, references::BoundOutput};
    use rf_ui::engineering::{EngineeringSession, editor::*};
    use std::collections::BTreeMap;
    let StudioProjectOpen::Editable(before) =
        parse_studio_project(&empty_project().to_string(), Path::new("fixture")).unwrap()
    else {
        panic!()
    };
    let mut a = (*before).clone();
    let receipt = super::super::equipment::apply(&mut a, EquipmentAction::CreateSource).unwrap();
    let EquipmentEditorTarget::Source(original) = receipt.target.unwrap() else {
        panic!()
    };
    let mut input = a.workspace.engineering.inputs().clone();
    input.basis.sources[0].title = original.to_string();
    a.workspace.engineering = EngineeringSession::from_inputs(
        a.workspace.document.metadata.document_id.clone(),
        input.clone(),
    )
    .unwrap();
    input.basis.sources[0].id = "different-allocation".into();
    let mut b = a.clone();
    b.workspace.engineering =
        EngineeringSession::from_inputs(b.workspace.document.metadata.document_id.clone(), input)
            .unwrap();
    let mut aliases_a = Aliases::default();
    let mut aliases_b = Aliases::default();
    aliases_a.created(
        "created",
        &before,
        &a,
        &BTreeMap::from([(
            Output::Source,
            BoundOutput {
                id: original.to_string(),
                revision: 1,
            },
        )]),
    );
    aliases_b.created(
        "created",
        &before,
        &b,
        &BTreeMap::from([(
            Output::Source,
            BoundOutput {
                id: "different-allocation".into(),
                revision: 1,
            },
        )]),
    );
    assert_eq!(aliases_a.snapshot(&a), aliases_b.snapshot(&b));
    assert_eq!(
        aliases_a.snapshot(&a)["sources"][0]["data"]["title"],
        original.as_str()
    );
}
