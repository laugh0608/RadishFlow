use super::*;
use radishflow_studio::prepare_mixed_serial_recycle_save;
use rf_solver::{
    mixed::{
        MixedBlock, MixedBlockId, MixedBlockMethod, MixedOptions, MixedRecycleConfiguration,
        MixedRecycleOptions,
    },
    partitioned::{
        FixedRecycleBoundary, PRESSURE_TOLERANCE_PA, TEMPERATURE_TOLERANCE_K, TearSpecification,
    },
};
use rf_store::project_v4::MIXED_SERIAL_RECYCLE_CAPABILITY;
use rf_types::DiagnosticPortTarget;
use rf_ui::flow_plan::{FlowPlanCommand, FlowPlanConfiguration, FlowPlanEditContext};

const M3: &str = "saved M3.rfproj.json";
const DOC: &str = "example-qs4-serial-recycle-eo-reference";
const METHOD: &str = rf_solver::mixed::SERIAL_RECYCLE_METHOD;

fn save_m3(f: &Fixture, budgets: [usize; 2], first_tear: &str) {
    let source = f.root.join("reference.rfproj.json");
    let bytes = include_bytes!(
        "../../../../examples/flowsheets/fixed-tp-serial-recycle-eo-reference.rfproj.json"
    );
    let mut wire: Value = serde_json::from_slice(bytes).unwrap();
    let flow = &mut wire["document"]["flowsheet"];
    // Give the upstream tear a later sort key, separating plan order from result order.
    let streams = flow["streams"].as_object_mut().unwrap();
    let mut tear = streams.remove("s-ra").unwrap();
    tear["id"] = json!("z-upstream");
    streams.insert("z-upstream".into(), tear);
    for unit in flow["units"].as_object_mut().unwrap().values_mut() {
        for port in unit["ports"].as_array_mut().unwrap() {
            if port["stream_id"] == "s-ra" {
                port["stream_id"] = json!("z-upstream");
            }
        }
    }
    fs::write(&source, wire.to_string()).unwrap();
    let mut app = load_project_app_state(&source).unwrap();
    let mut blocks = [
        "feed-a",
        "feed-b",
        "mixer-makeup-b",
        "splitter-recycle-a",
        "splitter-recycle-b",
    ]
    .map(|id| MixedBlock {
        id: id.into(),
        members: [id.into()].into(),
        method: MixedBlockMethod::Local,
    })
    .to_vec();
    let regions = [
        ("a", first_tear, None, budgets[0]),
        ("b", "s-rb", Some([2., 3.]), budgets[1]),
    ]
    .map(|(side, stream_id, initial, max_iteration_evaluations)| {
        let mixer = format!("mixer-{side}");
        blocks.push(MixedBlock {
            id: MixedBlockId::new(format!("equations-{side}")),
            members: [
                mixer.clone().into(),
                format!("splitter-primary-{side}").into(),
            ]
            .into(),
            method: MixedBlockMethod::EquationOriented { scales: None },
        });
        MixedRecycleOptions {
            tear: TearSpecification {
                stream_id: stream_id.into(),
                source: DiagnosticPortTarget::new(format!("splitter-recycle-{side}"), "outlet_a"),
                sink: DiagnosticPortTarget::new(mixer, "inlet_b"),
                initial_component_flows_mol_s: initial.map(|[methane, ethane]| {
                    [("methane".into(), methane), ("ethane".into(), ethane)].into()
                }),
                component_scales_mol_s: None,
            },
            max_iteration_evaluations,
        }
    });
    let context = FlowPlanEditContext::capture(&app.workspace);
    app.workspace
        .apply_flow_plan(
            &context,
            FlowPlanCommand::Apply(FlowPlanConfiguration::Mixed(MixedOptions {
                boundary: FixedRecycleBoundary {
                    temperature_k: 300.0,
                    pressure_pa: 650_000.0,
                },
                blocks,
                recycle: Some(Box::new(MixedRecycleConfiguration::Serial(regions))),
            })),
        )
        .unwrap();
    prepare_mixed_serial_recycle_save(&app, &f.root.join(M3))
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    fs::write(
        f.root.join(format!("{M3}.rfstudio-layout.json")),
        b"original sidecar",
    )
    .unwrap();
}

fn query() -> Value {
    json!({
        "schema_version": 15, "project": M3,
        "queries": [
            {"id":"plan","query":{"kind":"results","scope":"flow_plan"}},
            {"id":"flow","query":{"kind":"results","scope":"flow"}},
        ],
    })
}

fn execute(f: &Fixture, project: &str) -> Value {
    let mut request = query();
    request["project"] = json!(project);
    request["expected_project"] = json!({
        "document_id": DOC,
        "file_sha256": rf_store::sha256_hex(&fs::read(f.root.join(project)).unwrap()),
    });
    request["flow_resources"] = json!({
        "cache_root":"cache","auth_cache_index":"index.json", "package":{"kind":"preferred"},
    });
    request["steps"] = json!([{
        "id":"solve","action":{"kind":"run_flow","expected_revision":0,"expected_plan_revision":0},
    }]);
    request
}

fn assert_missing(response: &Value) {
    assert_eq!(
        response["queries"][1]["value"],
        json!({"status":"missing","reason":"no_result"}),
        "{response}"
    );
}

fn assert_balance(balance: &Value) {
    for value in balance["component_residuals_mol_s"]
        .as_object()
        .unwrap()
        .values()
    {
        assert!(value.as_f64().unwrap().abs() <= 1e-10);
    }
    assert!(balance["total_residual_mol_s"].as_f64().unwrap().abs() <= 2e-10);
}

#[test]
fn saved_m3_is_missing_until_explicit_run_and_current_only_in_that_process() {
    let f = Fixture::new();
    save_m3(&f, [100, 240], "z-upstream");
    // The independent query must not look for execution resources.
    fs::rename(f.root.join("index.json"), f.root.join("held-index.json")).unwrap();
    let (code, before) = f.v3("query", &query());
    assert_eq!(code, 0, "{before}");
    assert_eq!(before["project"]["project_schema_version"], 4);
    assert_eq!(before["project"]["read_only"], false);
    assert_eq!(
        before["project"]["file_sha256"],
        execute(&f, M3)["expected_project"]["file_sha256"]
    );
    assert_missing(&before);
    let plan = &before["queries"][0]["value"];
    assert_eq!(plan["ready"], true, "{plan}");
    assert_eq!(plan["plan_revision"], 0);
    assert_eq!(plan["configuration"]["method"], METHOD);
    assert_eq!(
        plan["configuration"]["capability"]["id"],
        MIXED_SERIAL_RECYCLE_CAPABILITY
    );
    assert_eq!(
        plan["configuration"]["scaling"]["kind"],
        "evaluated_region_inlet_and_global_feed_total"
    );
    let regions = plan["configuration"]["recycle_regions"].as_array().unwrap();
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0]["tear"]["stream_id"], "s-rb");
    assert_eq!(regions[0]["max_iteration_evaluations"], 240);
    assert_eq!(
        regions[0]["tear"]["initialization"],
        json!({
            "kind":"component_flows",
            "values":[{"component_id":"ethane","value_mol_s":3.0},{"component_id":"methane","value_mol_s":2.0}],
        })
    );
    assert_eq!(
        regions[1],
        json!({
            "max_iteration_evaluations": 100,
            "tear": {
                "stream_id":"z-upstream",
                "source":{"unit_id":"splitter-recycle-a","port":"outlet_a"},
                "sink":{"unit_id":"mixer-a","port":"inlet_b"},
                "initialization":{"kind":"zero_component_flows"},
            }
        })
    );

    fs::rename(f.root.join("held-index.json"), f.root.join("index.json")).unwrap();
    let mut request = execute(&f, M3);
    request["expected_project"]["document_id"] = before["project"]["document_id"].clone();
    request["expected_project"]["file_sha256"] = before["project"]["file_sha256"].clone();
    request["steps"][0]["action"]["expected_revision"] = before["project"]["flow_revision"].clone();
    request["steps"][0]["action"]["expected_plan_revision"] = plan["plan_revision"].clone();
    request["flow_resources"]["package"] = json!({
        "kind":"explicit", "package_id":test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
    });
    let (code, response) = f.v3("execute", &request);
    assert_eq!(code, 0, "{response}");
    let final_flow = &response["queries"][1]["value"];
    assert_eq!(response["steps"][0]["terminal"], "succeeded");
    assert_eq!(response["steps"][0]["reception"], "applied");
    assert_eq!(&response["steps"][0]["result"], final_flow);
    assert_eq!(final_flow["status"], "current");
    assert_eq!(final_flow["execution_kind"], "mixed_serial_recycle");
    let origin = &final_flow["origin"]["execution"];
    assert_eq!(origin["step"], "solve");
    assert_eq!(origin["scope"], "request");
    assert_eq!(origin["package"]["origin"], "cache");
    assert_eq!(
        origin["package"]["id"],
        test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID
    );
    assert_eq!(
        origin["plan"],
        json!({
            "method": METHOD,
            "configuration_sha256":plan["configuration_sha256"],
            "acceptance_contract_version":1,
            "capability":{"id":MIXED_SERIAL_RECYCLE_CAPABILITY,"version":1},
        })
    );
    assert_eq!(final_flow["blocks"].as_array().unwrap().len(), 7);
    let streams = final_flow["streams"].as_array().unwrap();
    assert_eq!(streams.len(), 13);
    for (id, methane, ethane) in [
        ("s-fa", 3.0, 2.0),
        ("s-fb", 1.0, 4.0),
        ("s-ma", 3.6, 2.4),
        ("s-sa", 1.8, 1.2),
        ("z-upstream", 0.6, 0.4),
        ("s-pa", 1.8, 1.2),
        ("s-ab", 1.2, 0.8),
        ("s-bf", 2.2, 4.8),
        ("s-mb", 4.4, 9.6),
        ("s-sb", 44. / 15., 6.4),
        ("s-rb", 2.2, 4.8),
        ("s-pb", 22. / 15., 3.2),
        ("s-qb", 11. / 15., 1.6),
    ] {
        let stream = streams.iter().find(|stream| stream["id"] == id).unwrap();
        for (component, expected) in [("methane", methane), ("ethane", ethane)] {
            let value = stream["components"]
                .as_array()
                .unwrap()
                .iter()
                .find(|value| value["component_id"] == component)
                .unwrap();
            assert!(
                (value["flow_mol_s"].as_f64().unwrap() - expected).abs() <= 1e-9,
                "{id}/{component}"
            );
        }
        assert!(
            (stream["temperature_k"].as_f64().unwrap() - 300.0).abs() <= TEMPERATURE_TOLERANCE_K
        );
        assert!(
            (stream["pressure_pa"].as_f64().unwrap() - 650_000.0).abs() <= PRESSURE_TOLERANCE_PA
        );
    }

    let boundaries = final_flow["boundaries"].as_array().unwrap();
    for id in ["s-ab", "s-bf"] {
        let stream = streams.iter().find(|stream| stream["id"] == id).unwrap();
        let boundary = boundaries
            .iter()
            .find(|boundary| boundary["stream_id"] == id)
            .unwrap();
        assert_eq!(&boundary["producer"], stream);
        assert_eq!(&boundary["consumed"], stream);
    }
    let regions = final_flow["recycle_regions"].as_array().unwrap();
    assert_eq!(regions.len(), 2);
    for (region, (equation, tear, inlet, budget)) in regions.iter().zip([
        ("equations-a", "z-upstream", "s-fa", 100),
        ("equations-b", "s-rb", "s-bf", 240),
    ]) {
        assert_eq!(region["equation_block_id"], equation);
        assert_eq!(region["stream_id"], tear);
        assert_eq!(region["inlet_stream_id"], inlet);
        assert_eq!(region["max_iteration_evaluations"], budget);
        let trace = region["residual_trace"].as_array().unwrap();
        let (verification, iterations) = trace.split_last().unwrap();
        assert!(
            iterations
                .iter()
                .all(|sample| sample["phase"] == "iteration")
        );
        assert_eq!(verification["phase"], "verification");
        assert!(verification["max_absolute_mol_s"].as_f64().unwrap() <= 1e-10);
        assert!(verification["max_scaled"].as_f64().unwrap() <= 1e-10);
        assert_balance(&region["region_material"]);
        let boundary = boundaries
            .iter()
            .find(|edge| edge["stream_id"] == tear)
            .unwrap();
        assert_eq!(region["last_sweep"]["assumed"], boundary["consumed"]);
        assert_eq!(region["last_sweep"]["returned"], boundary["producer"]);
    }
    let global = &final_flow["global_material"];
    assert_balance(global);
    for (component, expected) in [("methane", 4.0), ("ethane", 6.0)] {
        assert_eq!(global["fresh_components_mol_s"][component], expected);
        assert!(
            (global["product_components_mol_s"][component]
                .as_f64()
                .unwrap()
                - expected)
                .abs()
                <= 1e-10
        );
    }
    let (code, after) = f.v3("query", &query());
    assert_eq!(code, 0, "{after}");
    assert_eq!(before, after);
    assert_missing(&after);
}

#[test]
fn saved_m3_failures_only_expose_executed_regions_as_diagnostics() {
    for failed_region in 0..2 {
        let f = Fixture::new();
        let mut budgets = [100, 240];
        budgets[failed_region] = 1;
        save_m3(&f, budgets, "z-upstream");
        let (_, before) = f.v3("query", &query());
        assert_missing(&before);
        let (code, response) = f.v3("execute", &execute(&f, M3));
        assert_eq!(code, 4, "{response}");
        assert_eq!(response["error"]["code"], "run_failed");
        let flow = &response["queries"][1]["value"];
        assert_eq!(flow, &response["steps"][0]["result"]);
        assert_eq!(response["steps"][0]["reception"], "applied");
        assert_eq!(flow["status"], "failed");
        assert!(flow.get("streams").is_none() && flow.get("fields").is_none());
        let evidence = &flow["diagnostic_evidence"];
        assert_eq!(evidence["failure_kind"], "iteration_limit");
        let regions = evidence["recycle_regions"].as_array().unwrap();
        assert_eq!(regions.len(), failed_region + 1);
        assert_eq!(
            evidence["recycle_region_id"],
            ["equations-a", "equations-b"][failed_region]
        );
        assert!(regions[failed_region].get("region_material").is_none());
        if failed_region == 0 {
            let calls = evidence["costs"]["calls_by_unit"].as_object().unwrap();
            for unit in [
                "feed-b",
                "mixer-makeup-b",
                "mixer-b",
                "splitter-primary-b",
                "splitter-recycle-b",
            ] {
                assert!(!calls.contains_key(unit), "{calls:?}");
            }
        } else {
            assert_balance(&regions[0]["region_material"]);
            assert_eq!(
                regions[0]["residual_trace"]
                    .as_array()
                    .unwrap()
                    .last()
                    .unwrap()["phase"],
                "verification"
            );
        }
        let (_, after) = f.v3("query", &query());
        assert_eq!(before, after);
    }
}

#[test]
fn saved_m3_preflight_precedes_resource_access_and_preserves_missing() {
    let f = Fixture::new();
    save_m3(&f, [100, 240], "z-upstream");
    let mut valid = execute(&f, M3);
    valid["flow_resources"]["auth_cache_index"] = json!("absent.json");
    for (pointer, replacement, expected) in [
        (
            "/expected_project/document_id",
            json!("other"),
            "different_document",
        ),
        (
            "/expected_project/file_sha256",
            json!("outdated"),
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
        let mut request = valid.clone();
        *request.pointer_mut(pointer).unwrap() = replacement;
        let (code, response) = f.v3("execute", &request);
        assert_eq!(code, 2, "{response}");
        assert_eq!(response["error"]["code"], expected);
    }
    let (code, response) = f.v3("execute", &valid);
    assert_eq!(code, 4, "{response}");
    assert_eq!(response["error"]["code"], "run_start_failed");
    assert_missing(&response);

    save_m3(&f, [100, 240], "absent-stream");
    let (_, before) = f.v3("query", &query());
    let plan = &before["queries"][0]["value"];
    assert_eq!(plan["ready"], false, "{before}");
    assert_eq!(plan["not_ready"]["stream_ids"], json!(["absent-stream"]));
    assert_eq!(plan["not_ready"]["ports"].as_array().unwrap().len(), 2);
    assert_missing(&before);
    valid["expected_project"] = execute(&f, M3)["expected_project"].clone();
    let (code, response) = f.v3("execute", &valid);
    assert_eq!(code, 3, "{response}");
    assert_eq!(response["error"]["code"], "action_rejected");
    assert_eq!(response["error"]["details"], plan["not_ready"]);
    assert_missing(&response);
}

#[test]
fn saved_m3_uses_strict_requests_and_preserves_legacy_whole_file_protection() {
    let f = Fixture::new();
    save_m3(&f, [100, 240], "z-upstream");
    let valid = execute(&f, M3);
    for (pointer, replacement) in [
        ("/steps", json!([])),
        ("/steps", json!([valid["steps"][0], valid["steps"][0]])),
        ("/steps/0", json!(["solve", valid["steps"][0]["action"]])),
        ("/steps/0/id", json!(" ")),
        ("/steps/0/action/kind", json!("save_project")),
        ("/steps/0/action/expected_revision", json!(-1)),
        ("/steps/0/action/expected_plan_revision", json!("0")),
        ("/project", json!(" ")),
        ("/queries/1/id", json!("plan")),
        ("/queries/0/query/scope", json!("object")),
        (
            "/queries/0",
            json!(["plan", {"kind":"results","scope":"flow_plan"}]),
        ),
        (
            "/flow_resources",
            json!(["cache", "index.json", {"kind":"preferred"}]),
        ),
        (
            "/flow_resources/package",
            json!({"kind":"preferred","unknown":true}),
        ),
        ("/flow_resources/cache_root", json!(" ")),
        (
            "/expected_project",
            json!([DOC, valid["expected_project"]["file_sha256"]]),
        ),
    ] {
        let mut request = valid.clone();
        *request.pointer_mut(pointer).unwrap() = replacement;
        let (code, response) = f.v3("execute", &request);
        assert_eq!(code, 2, "{pointer}: {response}");
        assert_eq!(response["error"]["stage"], "request");
        assert_eq!(response["error"]["code"], "invalid_request");
        assert!(response["project"].is_null());
    }
    let bytes = serde_json::to_string(&valid).unwrap().replace(
        "\"expected_plan_revision\":0",
        "\"expected_plan_revision\":0,\"expected_plan_revision\":1",
    );
    let (code, response) = f.v3_bytes("execute", bytes.as_bytes());
    assert_eq!(code, 2, "{response}");
    assert_eq!(response["error"]["stage"], "request");

    let mut unknown: Value = serde_json::from_slice(&fs::read(f.root.join(M3)).unwrap()).unwrap();
    let caps = unknown["requiredCapabilities"].as_array_mut().unwrap();
    caps.iter_mut()
        .find(|cap| cap["id"] == MIXED_SERIAL_RECYCLE_CAPABILITY)
        .unwrap()["version"] = json!(2);
    fs::write(f.root.join("unknown.json"), unknown.to_string()).unwrap();

    let mut unmarked = unknown;
    unmarked["requiredCapabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|cap| cap["id"] != MIXED_SERIAL_RECYCLE_CAPABILITY);
    fs::write(f.root.join("unmarked.json"), unmarked.to_string()).unwrap();
    for project in [M3, "unmarked.json"] {
        for version in (4..=14).chain([16]) {
            let discovery = json!({"schema_version":version,"project":project,"queries":[]});
            let (code, response) = f.v3("query", &discovery);
            assert_eq!(code, 0, "v{version}: {response}");
            assert_eq!(response["project"]["read_only"], true);
            let mut run = execute(&f, project);
            run["schema_version"] = json!(version);
            run["queries"] = json!([]);
            run["flow_resources"]["auth_cache_index"] = json!("absent.json");
            if matches!(version, 13 | 14) {
                run.as_object_mut().unwrap().remove("flow_resources");
                run["steps"][0]["action"] = json!({
                    "kind": if version == 13 {"rate_concentric_tube"} else {"rate_concentric_tube_from_flow"},
                    "target":{"case":{"task":{"id":"task"},"case":{"id":"case"}}},
                    "expected_revisions":{"task":0,"geometry":0,"case":0},
                });
                if version == 14 {
                    run["steps"][0]["action"]["qualification"] =
                        json!({"id":"missing","revision":0});
                }
            }
            let (code, response) = f.v3("execute", &run);
            if version == 8 {
                assert_eq!(code, 2, "{response}");
                assert_eq!(response["error"]["code"], "unsupported_version");
            } else {
                assert_eq!(code, 3, "v{version}: {response}");
                assert_eq!(response["project"]["read_only"], true);
                assert_eq!(response["error"]["code"], "read_only_project");
                assert_eq!(response["steps"], json!([]));
            }
        }
    }

    for (project, exit, error) in [
        ("unmarked.json", 2, "invalid_project"),
        ("reference.rfproj.json", 2, "unsupported_capability"),
        ("unknown.json", 0, ""),
    ] {
        let mut discovery = query();
        discovery["project"] = json!(project);
        let (code, response) = f.v3("query", &discovery);
        assert_eq!(code, exit, "{response}");
        if exit == 0 {
            assert_eq!(response["project"]["read_only"], true);
            for row in response["queries"].as_array().unwrap() {
                assert_eq!(row["value"]["read_only"], true);
                assert!(row["value"]["reasons"].as_str().is_some());
                assert!(row["value"].get("status").is_none());
            }
            let (code, response) = f.v3("execute", &execute(&f, project));
            assert_eq!(code, 3, "{response}");
            assert_eq!(response["error"]["code"], "read_only_project");
            assert_eq!(response["steps"], json!([]));
        } else {
            assert_eq!(response["error"]["code"], error);
        }
    }
}
