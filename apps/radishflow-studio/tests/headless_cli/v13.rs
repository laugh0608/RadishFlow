use super::*;
use rf_store::{FileOverwritePolicy, project_v3 as store_v3, project_v4 as store_v4};

const NAME: &str = "同心套管.rfproj.json";
const V3: &str =
    include_str!("../../../../examples/flowsheets/standalone-separator-basis.rfproj.json");
const V4: &str =
    include_str!("../../../../examples/flowsheets/fixed-tp-recycle-binary-hydrocarbon.rfproj.json");
const EQUIPMENT: store_v3::EquipmentSupport = store_v3::EquipmentSupport::ConcentricTubeRating;
const FLOW: store_v4::FlowPlanSupport = store_v4::FlowPlanSupport::Mixed;

fn project(schema: u32, cold_flow: f64) -> Value {
    let mut project: Value = serde_json::from_str(if schema == 3 { V3 } else { V4 }).unwrap();
    let properties = json!({
        "sourceId": "nist-water",
        "molarMassKgPerMol": 0.018_015_28,
        "molarHeatCapacityJPerMolK": 75.329_923_737_630_4,
        "densityKgPerM3": 997.047_013,
        "dynamicViscosityPaS": 0.000_889_996_774,
        "thermalConductivityWPerMK": 0.607_198_237,
        "liquidDomain": {
            "minTemperatureK": 293.15, "maxTemperatureK": 303.15,
            "minPressurePa": 50_000.0, "maxPressurePa": 120_000.0
        }
    });
    let inlet = |temperature, flow| {
        json!({"inlet":{"temperatureK":temperature,"pressurePa":100_000.0,"molarFlowMolS":flow},
               "properties":properties})
    };
    let source = |id, title, locator| {
        json!({
            "id":id,"revision":3,"kind":"literature","title":title,"reference":title,"locator":locator,
            "publicationVersion":"","accessedOn":"","adoptionNote":"Published H3 benchmark","review":"reviewed",
            "applicability":{"fluidSystem":"Water","phase":"liquid","operatingConditions":{},"scope":"Benchmark"},
            "observations":[]
        })
    };
    project["engineering"] = json!({
        "basisEntries": [],
        "sources": [
            source("nist-water", "NIST water properties", "Table 8"),
            source("van-zyl-geometry", "Van Zyl et al.", "Table 5"),
            source("unused", "Unrelated source", "")
        ],
        "tasks": [{
            "kind":"concentricTubeHeatExchangerRating",
            "id":"h3-rating","tag":"E-301","name":"同心套管","revision":5,
            "method":{"id":"radishflow.heat-exchanger.concentric-tube-rating","version":1},
            "geometry":{
                "revision":7,"sourceId":"van-zyl-geometry","activeLengthM":4.85,
                "innerTubeInnerDiameterM":0.014_46,"innerTubeOuterDiameterM":0.015_88,
                "outerTubeInnerDiameterM":0.026_76,"wallThermalConductivityWPerMK":385.0
            },
            "case":{
                "id":"h3-water","name":"Clean water","purpose":"Benchmark","revision":11,
                "hot":inlet(303.15,14.0),"cold":inlet(293.15,cold_flow),
                "limits":{"minHeatDutyW":5_500.0,"maxHotPressureDropPa":11_000.0,"maxColdPressureDropPa":25_000.0}
            }
        }]
    });
    if schema == 4 {
        project["requiredCapabilities"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "id":store_v4::CONCENTRIC_TUBE_RATING_CAPABILITY,"version":1
            }));
    }
    project
}

fn save(f: &Fixture, project: &Value) -> Vec<u8> {
    let path = f.root.join(NAME);
    let input = project.to_string();
    if project["schemaVersion"] == 3 {
        let parsed = store_v3::parse_with_equipment_support(&input, EQUIPMENT).unwrap();
        assert!(matches!(parsed, store_v3::ProjectV3Read::Editable(_)));
        store_v3::write_with_equipment_support(
            &path,
            &parsed,
            FileOverwritePolicy::Forbid,
            EQUIPMENT,
            false,
        )
        .unwrap();
    } else {
        let parsed = store_v4::parse_with_equipment_support(&input, FLOW, EQUIPMENT).unwrap();
        assert!(matches!(parsed, store_v4::ProjectV4Read::Editable(_)));
        let prepared = store_v4::prepare_write_with_equipment_support(
            &path,
            &parsed,
            FileOverwritePolicy::Forbid,
            FLOW,
            EQUIPMENT,
        )
        .unwrap();
        store_v4::write_prepared(&prepared, false).unwrap();
    }
    fs::read(path).unwrap()
}

fn queries(target: &Value) -> Value {
    json!([
        {"id":"cases","query":{"kind":"objects"}},
        {"id":"case","query":{"kind":"record","target":target}},
        {"id":"performance","query":{"kind":"results","scope":{"object":target}}}
    ])
}

fn requests(f: &Fixture) -> (Value, Value, Value) {
    let initial = json!({"schema_version":13,"project":NAME,"queries":[{"id":"list","query":{"kind":"objects"}}]});
    let (code, discovery) = f.v3("query", &initial);
    assert_eq!(code, 0, "{discovery}");
    let snapshot = &discovery["project"];
    assert_eq!(snapshot["read_only"], false);
    let object = &discovery["queries"][0]["value"][0];
    assert_eq!(object["kind"], "concentric_tube_rating");
    assert_eq!(object["task_id"], "h3-rating");
    assert_eq!(object["case_id"], "h3-water");
    assert_eq!(
        object["revisions"],
        json!({"task":5,"geometry":7,"case":11})
    );
    let query = json!({"schema_version":13,"project":NAME,"queries":queries(&object["target"])});
    let (code, opened) = f.v3("query", &query);
    assert_eq!(code, 0, "{opened}");
    let mut run = query.clone();
    run["expected_project"] = json!({
        "document_id":snapshot["document_id"],
        "file_sha256":snapshot["file_sha256"]
    });
    run["steps"] = json!([{"id":"rate","action":{
        "kind":"rate_concentric_tube",
        "target":object["target"],
        "expected_revisions":object["revisions"]
    }}]);
    (query, opened, run)
}

fn close(value: &Value, expected: f64, tolerance: f64) {
    let value = value
        .as_f64()
        .unwrap_or_else(|| panic!("missing {expected}: {value}"));
    assert!(
        (value - expected).abs() <= tolerance,
        "{value} vs {expected}"
    );
}

#[test]
fn v13_process_reviews_saved_v3_and_v4_benchmarks() {
    let f = Fixture::empty();
    let path = f.root.join(NAME);
    for schema in [3, 4] {
        for (flow, duty, ua, hot_outlet, cold_outlet, cold_dp, check) in [
            (
                40.0,
                5_521.707_636_437_1,
                874.553_423_114_86,
                297.914_262_742_51,
                294.982_508_040_12,
                23_679.130_040_479,
                "passed",
            ),
            (
                50.0,
                5_877.489_522_370_7,
                945.093_778_801_02,
                297.576_906_873_93,
                294.710_466_075_30,
                34_962.547_685_264,
                "failed",
            ),
        ] {
            let bytes = save(&f, &project(schema, flow));
            let (query, opened, run) = requests(&f);
            assert_eq!(opened["project"]["project_schema_version"], schema);
            assert_eq!(
                opened["project"]["file_sha256"],
                rf_store::sha256_hex(&bytes)
            );
            assert_eq!(opened["queries"][2]["value"], json!({"status":"missing"}));
            let case = &opened["queries"][1]["value"];
            assert_eq!(case["geometry"]["active_length_m"], 4.85);
            assert_eq!(case["case"]["cold"]["inlet"]["molar_flow_mol_s"], flow);
            assert_eq!(
                case["case"]["cold"]["properties"]["liquid_domain"]["min_pressure_pa"],
                50_000.0
            );
            assert_eq!(case["sources"].as_array().unwrap().len(), 2);
            assert_eq!(case["sources"][0]["id"], "nist-water");
            assert_eq!(case["sources"][0]["revision"], 3);
            assert_eq!(case["sources"][1]["id"], "van-zyl-geometry");
            let (code, computed) = f.v3("execute", &run);
            assert_eq!(code, 0, "{computed}");
            let result = &computed["queries"][2]["value"];
            assert_eq!(result, &computed["steps"][0]["result"]);
            assert_eq!(result["status"], "current");
            assert_eq!(result["origin"]["step_id"], "rate");
            assert_eq!(
                result["origin"]["revisions"],
                run["steps"][0]["action"]["expected_revisions"]
            );
            assert_eq!(
                result["origin"]["file_sha256"],
                opened["project"]["file_sha256"]
            );
            assert_eq!(
                result["origin"]["document_id"],
                opened["project"]["document_id"]
            );
            assert_eq!(result["origin"]["method"], case["object"]["method"]);
            assert_eq!(
                result["origin"]["source_ids"],
                json!({
                    "geometry":"van-zyl-geometry","hot_properties":"nist-water","cold_properties":"nist-water"
                })
            );
            for (value, expected, tol) in [
                (&result["heat_duty_w"], duty, 1e-6),
                (&result["ua_w_per_k"], ua, 1e-6),
                (&result["hot"]["outlet"]["temperature_k"], hot_outlet, 1e-8),
                (
                    &result["cold"]["outlet"]["temperature_k"],
                    cold_outlet,
                    1e-8,
                ),
                (
                    &result["hot"]["passage"]["pressure_drop_pa"],
                    9_799.896_961_973_8,
                    1e-4,
                ),
                (
                    &result["cold"]["passage"]["pressure_drop_pa"],
                    cold_dp,
                    1e-4,
                ),
                (
                    &result["hot"]["outlet"]["pressure_pa"],
                    100_000.0 - 9_799.896_961_973_8,
                    1e-4,
                ),
                (
                    &result["cold"]["outlet"]["pressure_pa"],
                    100_000.0 - cold_dp,
                    1e-4,
                ),
                (&result["hot"]["sensible_energy_residual_w"], 0.0, 1e-6),
                (&result["cold"]["sensible_energy_residual_w"], 0.0, 1e-6),
                (&result["overall_sensible_energy_residual_w"], 0.0, 1e-6),
                (&result["lmtd_heat_duty_residual_w"], 0.0, 1e-6),
            ] {
                close(value, expected, tol);
            }
            assert_eq!(result["hot"]["outlet"]["molar_flow_mol_s"], 14.0);
            assert_eq!(result["cold"]["outlet"]["molar_flow_mol_s"], flow);
            assert_eq!(
                result["checks"],
                json!({"heat_duty":"passed","hot_pressure_drop":"passed","cold_pressure_drop":check})
            );
            assert_eq!(f.v3("query", &query), (0, opened));
            assert_eq!(fs::read(&path).unwrap(), bytes);
            fs::remove_file(&path).unwrap();
        }
    }
    assert!(!f.root.join("cache").exists());
}

#[test]
fn v13_process_qualifies_identity_fingerprint_and_all_h3_revisions() {
    let f = Fixture::empty();
    let bytes = save(&f, &project(4, 40.0));
    let (_, _, run) = requests(&f);
    for (pointer, replacement, expected) in [
        (
            "/expected_project/document_id",
            json!("other-project"),
            "different_document",
        ),
        (
            "/steps/0/action/target/case/case/id",
            json!("other-case"),
            "object_missing",
        ),
        (
            "/steps/0/action/target/case/task/id",
            json!("other-task"),
            "object_missing",
        ),
        (
            "/steps/0/action/expected_revisions/task",
            json!(6),
            "record_conflict",
        ),
        (
            "/steps/0/action/expected_revisions/geometry",
            json!(8),
            "record_conflict",
        ),
        (
            "/steps/0/action/expected_revisions/case",
            json!(12),
            "record_conflict",
        ),
    ] {
        let mut invalid = run.clone();
        *invalid.pointer_mut(pointer).unwrap() = replacement;
        let (code, failed) = f.v3("execute", &invalid);
        assert_eq!(code, 2, "{failed}");
        assert_eq!(failed["error"]["code"], expected);
        assert!(failed.pointer("/steps/0/result").is_none());
        if expected == "record_conflict" {
            assert_eq!(
                failed["error"]["details"]["actual_revisions"],
                json!({"task":5,"geometry":7,"case":11})
            );
        }
    }

    let mut modified = bytes;
    modified.push(b'\n');
    let path = f.root.join(NAME);
    fs::write(&path, &modified).unwrap();
    let (code, conflict) = f.v3("execute", &run);
    assert_eq!(code, 2, "{conflict}");
    assert_eq!(conflict["error"]["code"], "project_conflict");
    assert_eq!(conflict["steps"], json!([]));
    assert_eq!(
        conflict["project"]["file_sha256"],
        rf_store::sha256_hex(&modified)
    );
    assert_eq!(fs::read(path).unwrap(), modified);
}

#[test]
fn v13_process_limits_results_to_the_executed_case() {
    let f = Fixture::empty();
    let mut input = project(4, 40.0);
    let mut second = input["engineering"]["tasks"][0].clone();
    second["id"] = json!("h3-other");
    second["tag"] = json!("E-302");
    second["case"]["id"] = json!("h3-other-water");
    input["engineering"]["tasks"]
        .as_array_mut()
        .unwrap()
        .push(second);
    let bytes = save(&f, &input);
    let (_, opened, mut run) = requests(&f);
    let objects = opened["queries"][0]["value"].as_array().unwrap();
    assert_eq!(objects.len(), 2);
    run["queries"].as_array_mut().unwrap().push(json!({
        "id": "other",
        "query": {"kind": "results", "scope": {"object": objects[1]["target"]}}
    }));
    let (code, result) = f.v3("execute", &run);
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["queries"][2]["value"]["status"], "current");
    assert_eq!(result["queries"][3]["value"], json!({"status":"missing"}));
    assert_eq!(fs::read(f.root.join(NAME)).unwrap(), bytes);
}

#[test]
fn v13_process_preserves_core_failures_and_unchecked_limits() {
    let f = Fixture::empty();
    let path = f.root.join(NAME);
    for (missing, suffix) in [(true, "missing_input"), (false, "inlet_outside_domain")] {
        let mut input = project(3, 40.0);
        if missing {
            input["engineering"]["tasks"][0]["geometry"]
                .as_object_mut()
                .unwrap()
                .remove("activeLengthM");
        } else {
            input["engineering"]["tasks"][0]["case"]["cold"]["inlet"]["pressurePa"] =
                json!(150_000.0);
        }
        let bytes = save(&f, &input);
        let (query, opened, run) = requests(&f);
        if missing {
            assert!(opened["queries"][1]["value"]["geometry"]["active_length_m"].is_null());
        }
        let (code, failed) = f.v3("execute", &run);
        assert_eq!(code, 4, "{failed}");
        assert_eq!(failed["error"]["code"], "run_failed");
        let result = &failed["queries"][2]["value"];
        assert_eq!(result["status"], "failed");
        assert_eq!(result, &failed["steps"][0]["result"]);
        assert_eq!(
            result["diagnostic"]["code"],
            format!("unit.heat_exchanger.concentric_tube.{suffix}"),
        );
        assert!(result.get("heat_duty_w").is_none());
        assert_eq!(f.v3("query", &query), (0, opened));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        fs::remove_file(&path).unwrap();
    }

    let mut input = project(3, 40.0);
    input["engineering"]["tasks"][0]["case"]["limits"] = json!({});
    let bytes = save(&f, &input);
    let (_, _, run) = requests(&f);
    let (code, completed) = f.v3("execute", &run);
    assert_eq!(code, 0, "{completed}");
    assert_eq!(
        completed["steps"][0]["result"]["checks"],
        json!({
            "heat_duty":"not_specified","hot_pressure_drop":"not_specified","cold_pressure_drop":"not_specified"
        })
    );
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[test]
fn v13_process_preserves_legacy_and_future_capability_guards() {
    let f = Fixture::empty();
    let path = f.root.join(NAME);
    for schema in [3, 4] {
        let mut input = project(schema, 40.0);
        let valid_bytes = save(&f, &input);
        let legacy_query = json!({"schema_version":12,"project":NAME,"queries":[]});
        let (code, old) = f.v3("query", &legacy_query);
        assert_eq!(code, 0, "{old}");
        assert_eq!(old["project"]["read_only"], true);
        assert_eq!(old["project"]["read_only_reason"], "unsupported_capability");
        assert_eq!(
            old["project"]["file_sha256"],
            rf_store::sha256_hex(&valid_bytes)
        );
        let legacy_run = json!({
            "schema_version":12,"project":NAME,"queries":[],
            "expected_project":{"document_id":old["project"]["document_id"],"file_sha256":old["project"]["file_sha256"]},
            "steps":[{"id":"old","action":{"kind":"run_flow","expected_revision":0,"expected_plan_revision":0}}]
        });
        let (code, blocked) = f.v3("execute", &legacy_run);
        assert_eq!(code, 3, "{blocked}");
        assert_eq!(blocked["error"]["code"], "read_only_project");
        assert_eq!(blocked["steps"], json!([]));
        assert_eq!(f.command("inspect", &path).0, 2);
        assert_eq!(fs::read(&path).unwrap(), valid_bytes);

        let (_, _, mut run) = requests(&f);
        if schema == 3 {
            input["engineering"]["tasks"][0]["method"]["version"] = json!(2);
        } else {
            input["requiredCapabilities"]
                .as_array_mut()
                .unwrap()
                .last_mut()
                .unwrap()["version"] = json!(2);
        }
        let future_bytes = serde_json::to_vec_pretty(&input).unwrap();
        fs::write(&path, &future_bytes).unwrap();
        let future_query = json!({"schema_version":13,"project":NAME,"queries":[]});
        let (code, future) = f.v3("query", &future_query);
        assert_eq!(code, 0, "{future}");
        assert_eq!(future["project"]["read_only"], true);
        assert_eq!(
            future["project"]["read_only_reason"],
            "unsupported_capability"
        );
        assert_eq!(
            future["project"]["file_sha256"],
            rf_store::sha256_hex(&future_bytes)
        );
        let (code, blocked_query) = f.v3("query", &json!({
            "schema_version":13,"project":NAME,"queries":[{"id":"list","query":{"kind":"objects"}}]
        }));
        assert_eq!(code, 3, "{blocked_query}");
        assert!(blocked_query["queries"][0].get("value").is_none());
        run["expected_project"]["file_sha256"] = future["project"]["file_sha256"].clone();
        let (code, blocked) = f.v3("execute", &run);
        assert_eq!(code, 3, "{blocked}");
        assert_eq!(blocked["error"]["code"], "read_only_project");
        assert_eq!(blocked["steps"], json!([]));
        assert_eq!(fs::read(&path).unwrap(), future_bytes);
        fs::remove_file(&path).unwrap();
    }
}

#[test]
fn v13_process_rejects_malformed_requests_before_project_io() {
    let f = Fixture::empty();
    let target = json!({"case":{"task":{"id":"h3-rating"},"case":{"id":"h3-water"}}});
    let request = json!({
        "schema_version":13,"project":NAME,"queries":[],
        "expected_project":{"document_id":"example","file_sha256":"sha"},
        "steps":[{"id":"rate","action":{"kind":"rate_concentric_tube","target":target,"expected_revisions":{
            "task":5,"geometry":7,"case":11
        }}}]
    });
    let mut cases = Vec::new();
    for (pointer, value) in [
        ("/steps", json!([])),
        ("/steps", json!([request["steps"][0], request["steps"][0]])),
        ("/steps/0/action/kind", json!("run_flow")),
        (
            "/steps/0/action/target/case/task",
            json!({"step":"x","output":"task"}),
        ),
        ("/steps/0/action/target/case/task", json!(["h3-rating"])),
        ("/steps/0/action/expected_revisions", json!([5, 7, 11])),
        ("/expected_project", json!(["example", "sha"])),
        (
            "/queries",
            json!([{"id":"unexpected","query":{"kind":"results","scope":"flow"}}]),
        ),
    ] {
        let mut invalid = request.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        cases.push(invalid.to_string());
    }
    let mut with_resources = request.clone();
    with_resources["flow_resources"] = json!({});
    cases.push(with_resources.to_string());
    let duplicate = request
        .to_string()
        .replace(r#""geometry":7"#, r#""geometry":7,"geometry":9"#);
    assert_ne!(duplicate, request.to_string());
    cases.push(duplicate);
    for invalid in cases {
        let (code, reply) = f.v3_bytes("execute", invalid.as_bytes());
        assert_eq!(code, 2, "{reply}");
        assert_eq!(reply["error"]["stage"], "request");
        assert_eq!(reply["error"]["code"], "invalid_request");
        assert!(reply["project"].is_null());
    }
    assert!(!f.root.join(NAME).exists());
}
