use super::*;
use radishflow_studio::{
    StudioDocumentLifecycleCommand as Lifecycle, dispatch_document_lifecycle,
    prepare_object_identity_initialization,
};
use rf_types::units::{DisplayUnitSet, MeasurementUnit, QuantityKind};
use rf_ui::{
    AppState, BuiltinUnitKind, ProjectPresentationCommand,
    variable_browser::{
        ObjectId, StableVariableReference, ValueSource, ValueState, VariableField, VariableId,
        VariableSection, VariableValue, VariableWatchCommand, VariableWatchOutcome,
    },
    variable_commands::VariableWriteRequest,
};

const BASE: &str = "base.rfproj.json";
const PROJECT: &str = "工程 with identities.rfproj.json";
const EXAMPLE: &[u8] = include_bytes!(
    "../../../../examples/flowsheets/feed-mixer-flash-binary-hydrocarbon.rfproj.json"
);

fn stream_flow(app: &AppState, stream: &str, section: VariableSection) -> VariableId {
    VariableId {
        document: app.workspace.document.metadata.document_id.clone(),
        object: ObjectId::Stream(stream.into()),
        section,
        field: VariableField::MolarFlow,
    }
}

fn write_variable(app: &mut AppState, variable: VariableId, value: VariableValue) {
    app.write_variable(
        VariableWriteRequest {
            variable,
            expected_revision: app.workspace.document.revision,
            value,
        },
        UNIX_EPOCH,
    )
    .unwrap();
}

fn saved_project(f: &Fixture) -> AppState {
    let base = f.root.join(BASE);
    let path = f.root.join(PROJECT);
    fs::write(&base, EXAMPLE).unwrap();
    let mut app = load_project_app_state(&base).unwrap();
    let feed = stream_flow(&app, "stream-feed-a", VariableSection::Inputs);
    write_variable(&mut app, feed.clone(), VariableValue::Number(10.0));
    let prepared = prepare_object_identity_initialization(&app, &path).unwrap();
    dispatch_document_lifecycle(
        &mut app,
        Lifecycle::InitializeObjectIdentities(Box::new(prepared)),
    )
    .unwrap();
    assert_eq!(
        app.workspace
            .edit_variable_watch_list(&VariableWatchCommand::Add(feed))
            .unwrap(),
        VariableWatchOutcome::Added,
    );
    app.workspace
        .project_presentation
        .apply(ProjectPresentationCommand::Apply(
            DisplayUnitSet::engineering(),
        ));
    dispatch_document_lifecycle(&mut app, Lifecycle::SaveUpgraded { path: path.clone() }).unwrap();
    fs::write(
        f.root.join(format!("{PROJECT}.rfstudio-layout.json")),
        b"existing Studio layout sentinel",
    )
    .unwrap();
    load_project_app_state(&path).unwrap()
}

fn saved_reference(app: &AppState, variable: VariableId) -> StableVariableReference {
    StableVariableReference::for_saved_variable(&app.workspace, variable).unwrap()
}

fn feed_reference(app: &AppState) -> StableVariableReference {
    saved_reference(
        app,
        stream_flow(app, "stream-feed-a", VariableSection::Inputs),
    )
}

fn query(id: &str, reference: impl ToString) -> Value {
    json!({"id": id, "query": {"kind": "stable_variable", "reference": reference.to_string()}})
}

fn request(project: &str, queries: Value) -> Value {
    json!({"schema_version": 8, "project": project, "queries": queries})
}

fn assert_query_error(reply: &Value, id: &str, code: &str) {
    assert_eq!(reply["id"], id, "{reply}");
    assert_eq!(reply["error"]["query_id"], id, "{reply}");
    assert_eq!(reply["error"]["stage"], "query", "{reply}");
    assert_eq!(reply["error"]["code"], code, "{reply}");
    assert!(reply.get("value").is_none(), "{reply}");
}

#[test]
fn v8_process_reads_saved_studio_variables_in_si_and_follows_the_selected_file() {
    let f = Fixture::empty();
    let mut app = saved_project(&f);
    let feed = feed_reference(&app);
    let outlet = saved_reference(
        &app,
        stream_flow(&app, "stream-liquid", VariableSection::Results),
    );
    assert_eq!(
        app.workspace.variable_watch_list.entries(),
        &[feed.to_string()]
    );
    assert_eq!(
        app.workspace
            .project_presentation
            .display_units()
            .unit_for(QuantityKind::MolarFlow),
        MeasurementUnit::KilomolePerHour,
    );
    let current = feed.read(&app.workspace).unwrap();
    assert_eq!(current.id, *feed.variable());
    assert_eq!(current.value, Some(VariableValue::Number(10.0)));
    assert_eq!(current.source, ValueSource::DocumentInput);
    assert_eq!(current.state, ValueState::Valid);
    let missing = outlet.read(&app.workspace).unwrap();
    assert_eq!(missing.id, *outlet.variable());
    assert_eq!(missing.value, None);
    assert_eq!(missing.source, ValueSource::NoResult);
    assert_eq!(missing.state, ValueState::Missing);

    let queries = json!([query("feed", &feed), query("outlet", &outlet)]);
    let (code, response) = f.v3("query", &request(PROJECT, queries.clone()));
    assert_eq!(code, 0, "{response}");
    assert_eq!(response["schema_version"], 8);
    assert_eq!(response["status"], "ok");
    assert_eq!(response["steps"], json!([]));
    assert!(response["error"].is_null());
    let project = &response["project"];
    assert_eq!(project["document_id"], feed.variable().document.as_str());
    assert_eq!(project["flow_revision"], app.workspace.document.revision);
    assert_eq!(project["project_schema_version"], 4);
    assert_eq!(
        project["file_sha256"],
        rf_store::sha256_hex(&fs::read(f.root.join(PROJECT)).unwrap())
    );
    assert_eq!(project["read_only"], false);
    for (index, id, reference, target, state, value, source) in [
        (
            0,
            "feed",
            &feed,
            json!({"object":{"stream":"stream-feed-a"},"section":"inputs","field":"molar_flow"}),
            "valid",
            json!(10.0),
            "document_input",
        ),
        (
            1,
            "outlet",
            &outlet,
            json!({"object":{"stream":"stream-liquid"},"section":"results","field":"molar_flow"}),
            "missing",
            Value::Null,
            "no_result",
        ),
    ] {
        let reply = &response["queries"][index];
        assert_eq!(reply["id"], id);
        assert_eq!(reply["value"]["reference"], reference.to_string());
        assert_eq!(reply["value"]["quantity"], "molar_flow");
        let variable = &reply["value"]["variable"];
        assert_eq!(variable["target"], target);
        assert_eq!(variable["state"], state);
        assert_eq!(variable["value"], value);
        assert_eq!(variable["source"]["kind"], source);
        assert_eq!(variable["unit"], "mol/s");
        assert_eq!(variable["writable"], false);
    }

    // Save a branch with the same creation identities and different formal input.
    let name = VariableId {
        field: VariableField::Name,
        ..feed.variable().clone()
    };
    let name_ref = saved_reference(&app, name.clone());
    write_variable(&mut app, name, VariableValue::Text("Renamed feed".into()));
    write_variable(
        &mut app,
        feed.variable().clone(),
        VariableValue::Number(11.0),
    );
    dispatch_document_lifecycle(
        &mut app,
        Lifecycle::SaveAs {
            path: f.root.join("copy.rfproj.json"),
        },
    )
    .unwrap();
    let mut queries = queries.as_array().unwrap().clone();
    queries.push(query("name", name_ref));
    let (code, copy) = f.v3("query", &request("copy.rfproj.json", json!(queries)));
    assert_eq!(code, 0, "{copy}");
    assert_eq!(copy["queries"][0]["value"]["variable"]["value"], 11.0);
    assert_eq!(
        copy["queries"][2]["value"]["variable"]["value"],
        "Renamed feed"
    );
    let (code, original) = f.v3("query", &request(PROJECT, json!([query("feed", &feed)])));
    assert_eq!(code, 0, "{original}");
    assert_eq!(original["queries"][0], response["queries"][0]);
}

#[test]
fn v8_process_distinguishes_reference_errors_and_keeps_following_queries() {
    let f = Fixture::empty();
    let app = saved_project(&f);
    let feed = feed_reference(&app).to_string();
    let feed_unit = saved_reference(
        &app,
        VariableId {
            document: app.workspace.document.metadata.document_id.clone(),
            object: ObjectId::Unit("feed-a".into()),
            section: VariableSection::Inputs,
            field: VariableField::Name,
        },
    )
    .to_string();
    let failures = [
        (
            "malformed",
            "rfvar:1/broken".into(),
            "invalid_variable_reference",
        ),
        (
            "version",
            feed.replacen("rfvar:1", "rfvar:2", 1),
            "unsupported_reference_version",
        ),
        (
            "unsupported",
            feed.replace("/inputs/molar_flow", "/results/name"),
            "unsupported_field",
        ),
        (
            "cross",
            feed.replacen(
                app.workspace.document.metadata.document_id.as_str(),
                "different-project",
                1,
            ),
            "different_document",
        ),
        (
            "no-field",
            feed_unit.replace("/inputs/name", "/inputs/outlet_a_split_fraction"),
            "field_missing",
        ),
    ];
    let mut queries: Vec<_> = failures
        .iter()
        .map(|(id, reference, _)| query(id, reference))
        .collect();
    queries.push(query("ok", &feed));
    let (code, response) = f.v3("query", &request(PROJECT, json!(queries)));
    assert_eq!(code, 2, "{response}");
    assert_eq!(
        response["queries"].as_array().unwrap().len(),
        failures.len() + 1
    );
    for (index, (id, _, expected)) in failures.iter().enumerate() {
        assert_query_error(&response["queries"][index], id, expected);
    }
    assert_eq!(response["error"], response["queries"][0]["error"]);
    assert_eq!(
        response["queries"][failures.len()]["value"]["variable"]["value"],
        10.0
    );

    let (code, response) = f.v3("query", &request(BASE, json!([query("old", &feed)])));
    assert_eq!(code, 2, "{response}");
    assert_query_error(
        &response["queries"][0],
        "old",
        "object_identities_unavailable",
    );

    let saved: Value = serde_json::from_slice(&fs::read(f.root.join(PROJECT)).unwrap()).unwrap();
    let mut unknown = saved.clone();
    unknown["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"future-capability","version":1}));
    fs::write(
        f.root.join("future.json"),
        serde_json::to_vec(&unknown).unwrap(),
    )
    .unwrap();
    let (code, response) = f.v3(
        "query",
        &request("future.json", json!([query("protected", "malformed")])),
    );
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["project"]["read_only"], true);
    assert_eq!(
        response["project"]["read_only_reason"],
        "unsupported_capability"
    );
    assert_query_error(
        &response["queries"][0],
        "protected",
        "unsupported_capability",
    );

    let mut corrupt = saved;
    corrupt["objectIdentities"]["streams"][0]["creationId"] = json!("invalid-uuid");
    fs::write(
        f.root.join("corrupt.json"),
        serde_json::to_vec(&corrupt).unwrap(),
    )
    .unwrap();
    let (code, response) = f.v3(
        "query",
        &request("corrupt.json", json!([query("corrupt", &feed)])),
    );
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["stage"], "project");
    assert_eq!(response["error"]["code"], "invalid_project");
    assert!(response["project"].is_null());
    assert_eq!(response["queries"], json!([]));
}

#[test]
fn v8_process_rejects_deleted_and_recreated_objects_by_saved_creation_identity() {
    let f = Fixture::empty();
    let mut app = saved_project(&f);
    let path = f.root.join(PROJECT);
    let original = app
        .create_builtin_unit(BuiltinUnitKind::Heater, UNIX_EPOCH)
        .unwrap()
        .unit_id;
    dispatch_document_lifecycle(&mut app, Lifecycle::Save).unwrap();
    let reference = saved_reference(
        &app,
        VariableId {
            document: app.workspace.document.metadata.document_id.clone(),
            object: ObjectId::Unit(original.clone()),
            section: VariableSection::Inputs,
            field: VariableField::OutletTemperature,
        },
    );
    let request = request(PROJECT, json!([query("heater", &reference)]));
    let (code, response) = f.v3("query", &request);
    assert_eq!(code, 0, "{response}");
    assert_eq!(
        response["queries"][0]["value"]["variable"]["state"],
        "unspecified"
    );
    assert!(response["queries"][0]["value"]["variable"]["value"].is_null());

    app.delete_unit(&original, UNIX_EPOCH).unwrap();
    dispatch_document_lifecycle(&mut app, Lifecycle::Save).unwrap();
    let (code, response) = f.v3("query", &request);
    assert_eq!(code, 2, "{response}");
    assert_query_error(&response["queries"][0], "heater", "object_missing");

    let mut reopened = load_project_app_state(&path).unwrap();
    let recreated = reopened
        .create_builtin_unit(BuiltinUnitKind::Heater, UNIX_EPOCH)
        .unwrap()
        .unit_id;
    assert_eq!(recreated, original);
    dispatch_document_lifecycle(&mut reopened, Lifecycle::Save).unwrap();
    let (code, response) = f.v3("query", &request);
    assert_eq!(code, 2, "{response}");
    assert_query_error(&response["queries"][0], "heater", "object_recreated");
}

#[test]
fn v8_process_checks_the_query_vocabulary_and_rejects_execute_before_project_io() {
    let f = Fixture::empty();
    let app = saved_project(&f);
    let feed = feed_reference(&app);
    let missing_project = "missing.rfproj.json";
    let mut duplicate = request(
        missing_project,
        json!([query("same", &feed), query("same", &feed)]),
    );
    let mut extra = request(missing_project, json!([query("feed", &feed)]));
    extra["queries"][0]["query"]["extra"] = json!(true);
    for bad in [
        duplicate.clone(),
        extra,
        request("", json!([])),
        request(
            missing_project,
            json!([{"id": "objects", "query": {"kind": "objects"}}]),
        ),
        request(
            missing_project,
            json!([{"id": "feed", "query": {"kind": "stable_variable", "reference": null}}]),
        ),
    ] {
        let (code, response) = f.v3("query", &bad);
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["schema_version"], 8);
        assert_eq!(response["error"]["stage"], "request");
        assert_eq!(response["error"]["code"], "invalid_request");
        assert!(response["project"].is_null());
    }
    let (code, response) = f.v3_bytes(
        "query",
        br#"{"schema_version":8,"project":"missing","queries":[],"queries":[]}"#,
    );
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["stage"], "request");
    assert_eq!(response["error"]["code"], "invalid_request");
    assert!(response["project"].is_null());

    duplicate["schema_version"] = json!(u32::MAX);
    let (code, response) = f.v3("query", &duplicate);
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["stage"], "request");
    assert_eq!(response["error"]["code"], "unsupported_version");

    let (code, response) = f.v3("query", &request(PROJECT, json!([])));
    assert_eq!(code, 0, "{response}");
    assert_eq!(response["project"]["read_only"], false);
    assert_eq!(response["queries"], json!([]));
    let expected = json!({"document_id": response["project"]["document_id"], "file_sha256": response["project"]["file_sha256"]});
    for project in [PROJECT, missing_project] {
        let execute = json!({"schema_version": 8, "project": project, "expected_project": expected, "steps": [], "queries": []});
        let (code, response) = f.v3("execute", &execute);
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["schema_version"], 3);
        assert_eq!(response["error"]["stage"], "request");
        assert_eq!(response["error"]["code"], "unsupported_version");
        assert!(response["project"].is_null());
        assert_eq!(response["steps"], json!([]));
        assert_eq!(response["queries"], json!([]));
    }
}

#[test]
fn v8_process_preserves_legacy_protocol_boundaries_and_rejection_order() {
    let f = Fixture::empty();
    let app = saved_project(&f);
    let project_path = f.root.join(PROJECT);
    let saved = fs::read(&project_path).unwrap();
    let feed = feed_reference(&app);
    let document = feed.variable().document.as_str();
    let sha = rf_store::sha256_hex(&saved);
    let before = inventory(&f.root);
    let (code, response) = f.command("inspect", &project_path);
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["stage"], "project");
    assert_eq!(inventory(&f.root), before);

    let mut v1 = f.request();
    let mut v2: Value = serde_json::from_str(include_str!(
        "../../../../examples/automation/build-feed-heater-flash.request.json"
    ))
    .unwrap();
    for legacy in [&mut v1, &mut v2] {
        legacy["project"] = json!(PROJECT);
        let (code, response) = f.v3("run", legacy);
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["error"]["stage"], "project");
        assert_eq!(response["writes"], json!([]));
    }

    for version in 3..=7 {
        let unsupported = json!({"schema_version": version, "project": "missing.json", "queries": [query("feed", &feed)]});
        let (code, response) = f.v3("query", &unsupported);
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["error"]["stage"], "request");
        assert_eq!(response["error"]["code"], "invalid_request");
        let fields = json!({"schema_version": version, "project": PROJECT, "queries": [{"id": "fields", "query": {"kind": "fields", "target": {"stream": {"id": "stream-feed-a"}}}}]});
        let (code, response) = f.v3("query", &fields);
        if version == 3 {
            assert_eq!(code, 2, "{response}");
            assert_eq!(response["error"]["stage"], "project");
            assert_eq!(response["error"]["code"], "unsupported_capability");
        } else {
            assert_eq!(code, 0, "{response}");
            assert_eq!(response["project"]["file_sha256"], sha);
            assert_eq!(response["project"]["read_only"], true);
            assert_eq!(response["queries"][0]["value"]["read_only"], true);
            assert!(response["queries"][0]["value"].get("variable").is_none());
        }
        for (expected_document, expected_sha, status, error) in [
            ("different-project", "bad-sha", 2, "different_document"),
            (document, "bad-sha", 2, "project_conflict"),
            (document, sha.as_str(), 3, "read_only_project"),
        ] {
            let execute = json!({"schema_version": version, "project": PROJECT, "expected_project": {"document_id": expected_document, "file_sha256": expected_sha}, "steps": [], "queries": []});
            let (code, response) = f.v3("execute", &execute);
            let (status, error) = if version == 3 {
                (2, "unsupported_capability")
            } else {
                (status, error)
            };
            assert_eq!(code, status, "{response}");
            assert_eq!(response["error"]["stage"], "project");
            assert_eq!(response["error"]["code"], error);
            assert_eq!(response["steps"], json!([]));
            assert_eq!(response["queries"], json!([]));
        }
    }
}
