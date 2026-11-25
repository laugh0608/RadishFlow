use super::*;
use radishflow_studio::{
    StudioDocumentLifecycleCommand, dispatch_document_lifecycle, prepare_mixed_recycle_save,
    prepare_object_identity_initialization,
};
use rf_solver::{
    mixed::{MixedBlock, MixedBlockMethod, MixedRecycleOptions},
    partitioned::TearSpecification,
};
use rf_store::project_v4::{MIXED_RECYCLE_CAPABILITY, MIXED_RECYCLE_METHOD};
use rf_types::DiagnosticPortTarget;
use rf_ui::{
    AppState,
    flow_plan::{FlowPlanCommand, FlowPlanConfiguration, FlowPlanEditContext},
    variable_browser::{
        ObjectId, VariableField, VariableId, VariableSection, VariableWatchCommand,
    },
};

const M1: &str = "mixed-dag.rfproj.json";
const M2: &str = "已保存 mixed-recycle.rfproj.json";
const ORIGINAL: &[u8] = include_bytes!(
    "../../../../examples/flowsheets/fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json"
);

fn saved_m2(f: &Fixture, budget: usize, initial: Option<[f64; 2]>, tear_stream: &str) {
    let m1_path = f.root.join(M1);
    fs::write(&m1_path, ORIGINAL).unwrap();
    let mut app = load_project_app_state(&m1_path).unwrap();
    let path = f.root.join(M2);
    if initial.is_some() {
        add_stable_references(&mut app, &path);
    }
    let FlowPlanConfiguration::Mixed(mut options) = app.workspace.flow_plan.current().clone()
    else {
        panic!("M1 must load with its DAG plan");
    };
    options
        .blocks
        .iter_mut()
        .find(|b| b.id.as_str() == "equations")
        .unwrap()
        .members = ["mixer-1".into(), "splitter-1".into()].into();
    for id in ["mixer-2", "splitter-2"] {
        options.blocks.push(MixedBlock {
            id: id.into(),
            members: [id.into()].into(),
            method: MixedBlockMethod::Local,
        });
    }
    options.recycle = Some(Box::new(
        rf_solver::mixed::MixedRecycleConfiguration::Single(MixedRecycleOptions {
            tear: TearSpecification {
                stream_id: tear_stream.into(),
                source: DiagnosticPortTarget::new("splitter-2", "outlet_a"),
                sink: DiagnosticPortTarget::new("mixer-1", "inlet_b"),
                initial_component_flows_mol_s: initial.map(|[methane, ethane]| {
                    [("methane".into(), methane), ("ethane".into(), ethane)].into()
                }),
                component_scales_mol_s: None,
            },
            max_iteration_evaluations: budget,
        }),
    ));
    let edit = FlowPlanEditContext::capture(&app.workspace);
    app.workspace
        .apply_flow_plan(
            &edit,
            FlowPlanCommand::Apply(FlowPlanConfiguration::Mixed(options)),
        )
        .unwrap();
    prepare_mixed_recycle_save(&app, &path)
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    fs::write(
        f.root.join(format!("{M2}.rfstudio-layout.json")),
        b"keep existing layout",
    )
    .unwrap();
}

fn add_stable_references(app: &mut AppState, path: &std::path::Path) {
    let prepared = prepare_object_identity_initialization(app, path).unwrap();
    dispatch_document_lifecycle(
        app,
        StudioDocumentLifecycleCommand::InitializeObjectIdentities(Box::new(prepared)),
    )
    .unwrap();
    let input = VariableId {
        document: app.workspace.document.metadata.document_id.clone(),
        object: ObjectId::Unit("heater-1".into()),
        section: VariableSection::Inputs,
        field: VariableField::OutletTemperature,
    };
    app.workspace
        .edit_variable_watch_list(&VariableWatchCommand::Add(input))
        .unwrap();
}

fn request() -> Value {
    json!({
        "schema_version": 9, "project": M2,
        "queries": [
            {"id":"plan","query":{"kind":"results","scope":"flow_plan"}},
            {"id":"result","query":{"kind":"results","scope":"flow"}}
        ]
    })
}

fn execution(f: &Fixture, name: &str) -> Value {
    let mut request = request();
    request["project"] = json!(name);
    let raw = fs::read(f.root.join(name)).unwrap();
    request["expected_project"] = json!({
        "document_id":"example-qs4-fixed-tp-mixed-material",
        "file_sha256":rf_store::sha256_hex(&raw),
    });
    request["flow_resources"] = json!({
        "cache_root":"cache", "auth_cache_index":"index.json", "package":{"kind":"preferred"},
    });
    request["steps"] = json!([{
        "id":"solve","action":{"kind":"run_flow","expected_revision":0,"expected_plan_revision":0},
    }]);
    request
}

fn assert_flow(stream: &Value, expected: [f64; 2]) {
    let components = stream["components"].as_array().unwrap();
    for (component, flow) in ["methane", "ethane"].into_iter().zip(expected) {
        let value = components
            .iter()
            .find(|item| item["component_id"] == component)
            .unwrap();
        assert!(
            (value["flow_mol_s"].as_f64().unwrap() - flow).abs() <= 1e-9,
            "{stream}"
        );
        let fraction = flow / expected.iter().sum::<f64>();
        assert!((value["mole_fraction"].as_f64().unwrap() - fraction).abs() <= 1e-10);
    }
}

fn verify_result(value: &Value, digest: &Value, iterations: usize) {
    assert_eq!(value["status"], "current", "{value}");
    assert_eq!(value["execution_kind"], "mixed_recycle");
    for (id, expected) in [
        ("s-fa", [8.0 / 5.0, 2.0 / 5.0]),
        ("s-fb", [3.0 / 5.0, 12.0 / 5.0]),
        ("s-ma", [35.0 / 17.0, 20.0 / 17.0]),
        ("s-mb", [156.0 / 85.0, 264.0 / 85.0]),
        ("s-rab", [21.0 / 17.0, 12.0 / 17.0]),
        ("s-rba", [39.0 / 85.0, 66.0 / 85.0]),
        ("s-pa", [14.0 / 17.0, 8.0 / 17.0]),
        ("s-pb", [117.0 / 85.0, 198.0 / 85.0]),
        ("s-heated", [14.0 / 17.0, 8.0 / 17.0]),
    ] {
        let streams = value["streams"].as_array().unwrap();
        assert_eq!(streams.len(), 9);
        assert_flow(
            streams.iter().find(|stream| stream["id"] == id).unwrap(),
            expected,
        );
    }
    let source = &value["origin"]["execution"];
    assert_eq!(source["step"], "solve");
    assert_eq!(source["scope"], "request");
    assert_eq!(source["plan"]["method"], rf_solver::mixed::RECYCLE_METHOD);
    assert_eq!(&source["plan"]["configuration_sha256"], digest);
    assert_eq!(source["plan"]["capability"]["id"], MIXED_RECYCLE_CAPABILITY);
    assert_eq!(source["package"]["origin"], "cache");
    assert_eq!(
        source["package"]["id"],
        test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID
    );
    assert_eq!(value["blocks"].as_array().unwrap().len(), 6);

    let recycle = &value["recycle"];
    assert_eq!(
        recycle["component_scales_mol_s"],
        json!({"methane":5.0,"ethane":5.0})
    );
    let trace = recycle["residual_trace"].as_array().unwrap();
    assert_eq!(trace.len(), iterations + 1);
    for (index, entry) in trace[..iterations].iter().enumerate() {
        assert_eq!(entry["phase"], "iteration");
        assert_eq!(entry["evaluation"], index + 1);
    }
    let final_sample = trace.last().unwrap();
    assert_eq!(final_sample["phase"], "verification");
    assert!(final_sample["max_absolute_mol_s"].as_f64().unwrap() <= 1e-10);
    assert!(final_sample["max_scaled"].as_f64().unwrap() <= 1e-10);
    let tear = value["boundaries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|edge| edge["stream_id"] == "s-rba")
        .unwrap();
    assert_eq!(recycle["last_sweep"]["assumed"], tear["consumed"]);
    assert_eq!(recycle["last_sweep"]["returned"], tear["producer"]);

    let balance = &value["global_material"];
    for (component, flow) in [("methane", 11.0 / 5.0), ("ethane", 14.0 / 5.0)] {
        assert!(
            (balance["product_components_mol_s"][component]
                .as_f64()
                .unwrap()
                - flow)
                .abs()
                <= 1e-10
        );
        assert!(
            balance["component_residuals_mol_s"][component]
                .as_f64()
                .unwrap()
                .abs()
                <= 1e-10
        );
    }
    assert!(balance["total_residual_mol_s"].as_f64().unwrap().abs() <= 2e-10);
}

#[test]
fn v9_real_process_projects_both_initializations_and_reopens_missing() {
    for (initial, iterations) in [(None, 13), (Some([10.0, 1.0]), 15)] {
        let f = Fixture::new();
        saved_m2(&f, 500, initial, "s-rba");
        // Query needs neither the property index nor a cache.
        let index = f.root.join("index.json");
        fs::rename(&index, f.root.join("parked.json")).unwrap();
        let (code, opened) = f.v3("query", &request());
        assert_eq!(code, 0, "{opened}");
        assert_eq!(opened["schema_version"], 9);
        assert_eq!(opened["project"]["read_only"], false);
        assert_eq!(opened["project"]["project_schema_version"], 4);
        assert_eq!(
            opened["project"]["file_sha256"],
            rf_store::sha256_hex(&fs::read(f.root.join(M2)).unwrap())
        );
        let plan = &opened["queries"][0]["value"];
        assert_eq!(plan["plan_revision"], 0);
        assert_eq!(plan["ready"], true);
        assert_eq!(plan["configuration"]["method"], MIXED_RECYCLE_METHOD);
        assert_eq!(plan["configuration"]["blocks"].as_array().unwrap().len(), 6);
        assert_eq!(plan["configuration"]["max_iteration_evaluations"], 500);
        let tear = &plan["configuration"]["tear"];
        assert_eq!(tear["stream_id"], "s-rba");
        assert_eq!(
            tear["source"],
            json!({"unit_id":"splitter-2","port":"outlet_a"})
        );
        assert_eq!(tear["sink"], json!({"unit_id":"mixer-1","port":"inlet_b"}));
        assert_eq!(tear["residual_scaling"]["kind"], "evaluated_fresh_total");
        let initialization = initial.map_or_else(|| json!({"kind":"zero_component_flows"}), |[m, e]| {
            json!({"kind":"component_flows","values":[{"component_id":"ethane","value_mol_s":e},{"component_id":"methane","value_mol_s":m}]})
        });
        assert_eq!(tear["initialization"], initialization);
        assert_eq!(
            opened["queries"][1]["value"],
            json!({"status":"missing","reason":"no_result"})
        );
        fs::rename(f.root.join("parked.json"), index).unwrap();

        let (code, solved) = f.v3("execute", &execution(&f, M2));
        assert_eq!(code, 0, "{}", solved["error"]);
        assert_eq!(solved["steps"][0]["terminal"], "succeeded");
        assert_eq!(solved["steps"][0]["reception"], "applied");
        assert_eq!(solved["queries"][0]["value"], *plan);
        let result = &solved["queries"][1]["value"];
        assert_eq!(solved["steps"][0]["result"], *result);
        verify_result(result, &plan["configuration_sha256"], iterations);
        if initial.is_some() {
            let mut run = execution(&f, M2);
            let mut next = run["steps"][0].clone();
            next["id"] = json!("solve-again");
            run["steps"].as_array_mut().unwrap().push(next);
            let (code, repeated) = f.v3("execute", &run);
            assert_eq!(code, 0, "{}", repeated["error"]);
            assert_eq!(
                repeated["steps"][0]["result"]["origin"]["execution"]["step"],
                "solve"
            );
            assert_eq!(
                repeated["steps"][1]["result"]["origin"]["execution"]["step"],
                "solve-again"
            );
            assert_eq!(
                repeated["queries"][1]["value"],
                repeated["steps"][1]["result"]
            );
        }
        assert_eq!(f.v3("query", &request()).1, opened);
    }
}

#[test]
fn v9_real_process_keeps_iteration_failure_in_diagnostics_and_stops_further_steps() {
    let f = Fixture::new();
    saved_m2(&f, 1, None, "s-rba");
    let mut run = execution(&f, M2);
    let mut second = run["steps"][0].clone();
    second["id"] = json!("skipped");
    run["steps"].as_array_mut().unwrap().push(second);
    let (code, response) = f.v3("execute", &run);
    assert_eq!(code, 4, "{response}");
    assert_eq!(response["status"], "failed");
    assert_eq!(response["error"]["code"], "run_failed");
    assert_eq!(response["error"]["step_id"], "solve");
    assert_eq!(response["steps"].as_array().unwrap().len(), 1);
    assert_eq!(response["steps"][0]["terminal"], "failed");
    assert_eq!(response["steps"][0]["reception"], "applied");
    let value = &response["queries"][1]["value"];
    assert_eq!(*value, response["steps"][0]["result"]);
    assert_eq!(value["status"], "failed");
    assert_eq!(value["diagnostic"]["code"], "solver.mixed.iteration_limit");
    assert!(value.get("streams").is_none() && value.get("fields").is_none());
    let diagnostic = &value["diagnostic_evidence"];
    assert_eq!(diagnostic["failure_kind"], "iteration_limit");
    assert_eq!(
        diagnostic["recycle"]["residual_trace"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_flow(
        &diagnostic["recycle"]["last_sweep"]["returned"],
        [0.39, 0.66],
    );
    assert!(
        diagnostic["completed_blocks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|block| {
                !block["members"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("heater-1"))
            })
    );
    assert_eq!(
        f.v3("query", &request()).1["queries"][1]["value"]["status"],
        "missing"
    );
}

#[test]
fn v9_real_process_rejects_requests_and_wrong_inputs_before_resource_io() {
    let f = Fixture::empty();
    saved_m2(&f, 500, None, "missing-tear");
    let (_, read) = f.v3("query", &request());
    let plan = &read["queries"][0]["value"];
    assert_eq!(plan["ready"], false);
    assert_eq!(plan["not_ready"]["code"], "solver.mixed.missing_tear");
    assert_eq!(plan["not_ready"]["stream_ids"], json!(["missing-tear"]));
    let run = execution(&f, M2);
    let (code, response) = f.v3("execute", &run);
    assert_eq!(code, 3, "{response}");
    assert_eq!(response["error"]["details"], plan["not_ready"]);
    assert_eq!(response["queries"][1]["value"]["status"], "missing");
    for (pointer, value, error) in [
        (
            "/expected_project/document_id",
            json!("wrong"),
            "different_document",
        ),
        (
            "/expected_project/file_sha256",
            json!("wrong"),
            "project_conflict",
        ),
        (
            "/steps/0/action/expected_revision",
            json!(1),
            "record_conflict",
        ),
        (
            "/steps/0/action/expected_plan_revision",
            json!(1),
            "record_conflict",
        ),
    ] {
        let mut wrong = run.clone();
        *wrong.pointer_mut(pointer).unwrap() = value;
        let (code, response) = f.v3("execute", &wrong);
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["error"]["code"], error);
    }
    let mut absent = run.clone();
    absent["project"] = json!("absent.json");
    for (pointer, value) in [
        ("/steps", json!([])),
        ("/steps/0/action/kind", json!("configure_flow_plan")),
        (
            "/steps/0/action/expected_revision",
            json!({"step":"solve","output":"document"}),
        ),
        ("/flow_resources/auth_cache_index", Value::Null),
        ("/queries/0/query/scope", json!("equipment")),
        ("/queries/0/query/kind", json!("stable_variable")),
    ] {
        let mut invalid = absent.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        let (code, response) = f.v3("execute", &invalid);
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["error"]["code"], "invalid_request");
        assert!(response["project"].is_null());
    }
    let duplicate = serde_json::to_string(&absent).unwrap().replace(
        "\"expected_plan_revision\":0",
        "\"expected_plan_revision\":0,\"expected_plan_revision\":0",
    );
    let (code, response) = f.v3_bytes("execute", duplicate.as_bytes());
    assert_eq!(code, 2);
    assert_eq!(response["error"]["code"], "invalid_request");
}

#[test]
fn v9_real_process_preserves_legacy_protocols_and_protects_capability_boundaries() {
    let f = Fixture::empty();
    saved_m2(&f, 500, None, "s-rba");
    let mut file: Value = serde_json::from_slice(&fs::read(f.root.join(M2)).unwrap()).unwrap();
    let capabilities = file["requiredCapabilities"].as_array_mut().unwrap();
    let index = capabilities
        .iter()
        .position(|entry| entry["id"] == MIXED_RECYCLE_CAPABILITY)
        .unwrap();
    capabilities[index]["version"] = json!(2);
    fs::write(f.root.join("future.json"), file.to_string()).unwrap();
    file["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .remove(index);
    fs::write(f.root.join("hidden.json"), file.to_string()).unwrap();

    for name in [M2, "hidden.json"] {
        let mut q = request();
        q["project"] = json!(name);
        q["schema_version"] = json!(7);
        let (code, response) = f.v3("query", &q);
        assert_eq!(code, 0, "{response}");
        assert_eq!(response["project"]["read_only"], true);
        assert!(
            response["queries"]
                .as_array()
                .unwrap()
                .iter()
                .all(|item| item["value"]["read_only"] == true)
        );
        let (code, response) = f.v3(
            "query",
            &json!({"schema_version":8,"project":name,"queries":[]}),
        );
        assert_eq!(code, 0, "{response}");
        assert_eq!(response["project"]["read_only"], true);
        let mut run = execution(&f, name);
        run["schema_version"] = json!(7);
        let (code, response) = f.v3("execute", &run);
        assert_eq!(code, 3, "{response}");
        assert_eq!(response["steps"], json!([]));
        assert_eq!(response["error"]["code"], "read_only_project");
        run["schema_version"] = json!(8);
        let (code, response) = f.v3("execute", &run);
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["schema_version"], 3);
        assert_eq!(response["error"]["code"], "unsupported_version");
        assert!(response["project"].is_null());
    }

    for (name, exit, error) in [
        ("future.json", 3, "read_only_project"),
        ("hidden.json", 2, "invalid_project"),
        (M1, 2, "unsupported_capability"),
    ] {
        let (code, response) = f.v3("execute", &execution(&f, name));
        assert_eq!(code, exit, "{response}");
        assert_eq!(response["error"]["code"], error);
        assert_eq!(response["steps"], json!([]));
    }
    let mut future = request();
    future["project"] = json!("future.json");
    let (code, response) = f.v3("query", &future);
    assert_eq!(code, 0, "{response}");
    assert_eq!(response["project"]["read_only"], true);
    assert_eq!(response["queries"][0]["value"]["read_only"], true);
    assert!(
        response["queries"][0]["value"]
            .get("configuration")
            .is_none()
    );
}
