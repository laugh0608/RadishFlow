#[path = "v3/material.rs"]
mod material;
use super::*;
use std::collections::BTreeMap;
use std::path::Path;

const DEVICE_PROJECT: &[u8] =
    include_bytes!("../../../../examples/flowsheets/standalone-separator-basis.rfproj.json");
const DEVICE_REQUEST: &str =
    include_str!("../../../../examples/automation/size-adopt-rate-v3.request.json");
const FLOW_REQUEST: &str =
    include_str!("../../../../examples/automation/build-feed-heater-flash-v3.request.json");

fn inventory(root: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn visit(root: &Path, path: &Path, out: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            let key = path.strip_prefix(root).unwrap().to_path_buf();
            if path.is_dir() {
                out.insert(key, None);
                visit(root, &path, out);
            } else {
                out.insert(key, Some(fs::read(path).unwrap()));
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}
impl Fixture {
    fn device_v3() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "rf-v3-device-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("device.json"), DEVICE_PROJECT).unwrap();
        fs::write(
            root.join("device.json.rfstudio-layout.json"),
            b"existing sidecar sentinel, must remain unchanged",
        )
        .unwrap();
        Self { root }
    }
    fn v3(&self, verb: &str, request: &Value) -> (i32, Value) {
        self.v3_bytes(verb, &serde_json::to_vec_pretty(request).unwrap())
    }
    fn v3_bytes(&self, verb: &str, bytes: &[u8]) -> (i32, Value) {
        let path = self.root.join("v3 request.json");
        fs::write(&path, bytes).unwrap();
        let before = inventory(&self.root);
        let result = self.command(verb, &path);
        assert_eq!(
            inventory(&self.root),
            before,
            "CLI changed file bytes or directory entries"
        );
        result
    }
    fn query_v3(&self, project: &str, queries: Value) -> Value {
        let (code, r) = self.v3(
            "query",
            &json!({"schema_version":3,"project":project,"queries":queries}),
        );
        assert_eq!(code, 0, "{r}");
        r
    }
    fn device_request_v3(&self) -> Value {
        let mut request: Value = serde_json::from_str(DEVICE_REQUEST).unwrap();
        request["project"] = json!("device.json");
        let q = self.query_v3("device.json", json!([]));
        assert_eq!(
            request["expected_project"]["file_sha256"],
            q["project"]["file_sha256"]
        );
        request
    }
}
fn confirmation(r: &Value) -> Value {
    json!({"digest":r["error"]["details"]["confirmation_digest"]})
}
fn find_step<'a>(request: &'a mut Value, id: &str) -> &'a mut Value {
    request["steps"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|s| s["id"] == id)
        .unwrap()
}

#[test]
fn v3_process_public_flow_example_builds_runs_and_preserves_all_input_assets() {
    let f = Fixture::new();
    fs::write(
        f.root.join("basis.json"),
        include_bytes!("../../../../examples/flowsheets/binary-hydrocarbon-basis.rfproj.json"),
    )
    .unwrap();
    fs::write(
        f.root.join("basis.json.rfstudio-layout.json"),
        b"existing flow sidecar",
    )
    .unwrap();
    let mut request: Value = serde_json::from_str(FLOW_REQUEST).unwrap();
    request["project"] = json!("basis.json");
    request["flow_resources"]["auth_cache_index"] = json!("index.json");
    let q = f.query_v3(
        "basis.json",
        json!([{ "id":"actions","query":{"kind":"actions","scope":"flow"}}]),
    );
    assert_eq!(
        q["project"]["file_sha256"],
        request["expected_project"]["file_sha256"]
    );
    let (code, r) = f.v3("execute", &request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(
        r["steps"].as_array().unwrap().len(),
        request["steps"].as_array().unwrap().len()
    );
    let result = &r["queries"][0]["value"];
    assert_eq!(result["status"], "current");
    assert_eq!(result["origin"]["execution"]["step"], "solve");
    assert_eq!(result["origin"]["execution"]["package"]["origin"], "cache");
    let variables = r["queries"][1]["value"].as_array().unwrap();
    for (field, expected, unit) in [
        ("temperature", 330.0, "K"),
        ("pressure", 90000.0, "Pa"),
        ("molar_flow", 5.0, "mol/s"),
    ] {
        let v = variables
            .iter()
            .find(|v| {
                v["variable"]["target"]["section"] == "results"
                    && v["variable"]["target"]["field"] == field
            })
            .unwrap();
        assert_eq!(v["variable"]["value"], expected);
        assert_eq!(v["variable"]["unit"], unit);
    }
    let (_, again) = f.v3("execute", &request);
    assert_eq!(again["queries"][1]["value"], r["queries"][1]["value"]);
    let loaded=f.query_v3("basis.json",json!([{ "id":"objects","query":{"kind":"objects"}},{"id":"results","query":{"kind":"results","scope":"flow"}}]));
    assert_eq!(loaded["queries"][0]["value"], json!([]));
    assert_eq!(loaded["queries"][1]["value"]["status"], "missing");
}

#[test]
fn v3_process_public_device_example_reviews_replays_and_discards_on_exit() {
    let f = Fixture::device_v3();
    let mut request = f.device_request_v3();
    let (code, first) = f.v3("execute", &request);
    assert_eq!(code, 6, "{first}");
    assert_eq!(first["error"]["step_id"], "adopt");
    assert_eq!(first["steps"][6]["terminal"], "succeeded");
    let (_, again) = f.v3("execute", &request);
    assert_eq!(confirmation(&first), confirmation(&again));
    find_step(&mut request, "adopt")["confirmation"] = confirmation(&first);
    let (code, r) = f.v3("execute", &request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["queries"][1]["value"]["status"], "current");
    assert_eq!(r["queries"][2]["value"]["status"], "current");
    assert_eq!(
        r["queries"][2]["value"]["origin"]["execution"]["step"],
        "rate"
    );
    assert_eq!(
        r["queries"][2]["value"]["value"]["capture"]["geometry"]["adoption"]["source_task"]["data"]
            ["tag"],
        "V-101"
    );
    assert!(!f.root.join("cache").exists());
    let q = f.query_v3(
        "device.json",
        json!([{ "id":"objects","query":{"kind":"objects"}}]),
    );
    assert_eq!(q["queries"][0]["value"], json!([]));
    let path = f.root.join("device.json");
    let before = inventory(&f.root);
    let (code, r) = f.command("inspect", &path);
    assert_eq!(code, 2);
    assert_eq!(r["schema_version"], 1);
    assert_eq!(inventory(&f.root), before);
}

#[test]
fn v3_process_changed_evidence_rejects_old_confirmation_and_sequential_reviews_revalidate() {
    let f = Fixture::device_v3();
    let mut request = f.device_request_v3();
    let (_, preview) = f.v3("execute", &request);
    find_step(&mut request, "adopt")["confirmation"] = confirmation(&preview);
    let mut changed = request.clone();
    find_step(&mut changed, "source_input")["action"]["data"]["title"] =
        json!("Changed evidence description");
    let (code, r) = f.v3("execute", &changed);
    assert_eq!(code, 6);
    assert_eq!(r["error"]["code"], "confirmation_mismatch");
    assert_eq!(r["error"]["step_id"], "adopt");
    let mut source = find_step(&mut request, "source_input")["action"]["data"].clone();
    source["reference"] = json!("Post-adoption evidence update");
    request["steps"].as_array_mut().unwrap().push(json!({"id":"evidence","action":{"kind":"update_source","source":{"step":"source","output":"source"},"expected_revision":{"step":"source_input","output":"source"},"data":source}}));
    let (code, r) = f.v3("execute", &request);
    assert_eq!(code, 6, "{r}");
    assert_eq!(r["error"]["step_id"], "evidence");
    find_step(&mut request, "evidence")["confirmation"] = confirmation(&r);
    let (code, r) = f.v3("execute", &request);
    assert_eq!(code, 0, "{r}");
    assert_eq!(r["queries"][1]["value"]["status"], "stale");
    assert!(r["queries"][1]["value"].get("value").is_none());
    assert_eq!(r["queries"][2]["value"]["status"], "current");
}

#[test]
fn v3_process_strict_failures_identity_conflicts_and_unknown_capability_keep_disk_frozen() {
    let f = Fixture::device_v3();
    let request = f.device_request_v3();
    let mut conflict = request.clone();
    conflict["expected_project"]["file_sha256"] = json!("not-the-file");
    let (code, r) = f.v3("execute", &conflict);
    assert_eq!(code, 2);
    assert_eq!(r["error"]["code"], "project_conflict");
    assert_eq!(r["steps"], json!([]));
    for text in [
        r#"{"schema_version":3,"project":"device.json","queries":[],"queries":[]}"#,
        r#"{"schema_version":3,"project":"device.json","queries":[{"id":"x","query":{"kind":"objects","extra":true}}]}"#,
        r#"{"schema_version":3,"project":"device.json","queries":null}"#,
        r#"{"schema_version":3,"project":"","queries":[]}"#,
        r#"{"schema_version":3,"project":"absent.json","queries":[]}"#,
        r#"{"schema_version":3,"project":"device.json","queries":[{"id":"bad","query":{"kind":"fields","target":{"equipment":{"id":1e999}}}}]}"#,
    ] {
        let (code, r) = f.v3_bytes("query", text.as_bytes());
        assert_eq!(code, 2, "{r}");
        assert_eq!(r["steps"], json!([]));
    }
    let mut unknown: Value = serde_json::from_str(include_str!(
        "../../../../crates/rf-store/tests/fixtures/project-v3-rating.json"
    ))
    .unwrap();
    unknown["engineering"]["tasks"][0]["method"]["version"] = json!(999);
    fs::write(
        f.root.join("unknown.json"),
        serde_json::to_vec(&unknown).unwrap(),
    )
    .unwrap();
    let q = f.query_v3(
        "unknown.json",
        json!([{ "id":"objects","query":{"kind":"objects"}}]),
    );
    assert_eq!(q["project"]["read_only"], true);
    let mut request = request;
    request["project"] = json!("unknown.json");
    request["expected_project"] = json!({"document_id":q["project"]["document_id"],"file_sha256":q["project"]["file_sha256"]});
    let (code, r) = f.v3("execute", &request);
    assert_eq!(code, 3);
    assert_eq!(r["error"]["code"], "read_only_project");
    assert_eq!(r["steps"], json!([]));
}

#[test]
fn v3_process_later_failure_retains_edits_and_calculation_failure_has_terminal_receipt() {
    let f = Fixture::device_v3();
    let mut request = f.device_request_v3();
    request["steps"] = json!([
        {"id":"new","action":{"kind":"create_rating_task"}},
        {"id":"bad","action":{"kind":"update_rating_labels","task":{"step":"future","output":"task"},"expected_revision":1,"data":{"tag":"X","name":"Invalid"}}},
        {"id":"future","action":{"kind":"create_rating_task"}}
    ]);
    request["queries"] = json!([{ "id":"objects","query":{"kind":"objects"}}]);
    let (code, r) = f.v3("execute", &request);
    assert_eq!(code, 2);
    assert_eq!(r["error"]["code"], "unknown_reference");
    assert_eq!(r["steps"].as_array().unwrap().len(), 2);
    assert_eq!(r["queries"][0]["value"].as_array().unwrap().len(), 3);
    request["steps"] = json!([{ "id":"new","action":{"kind":"create_sizing_task"}},{"id":"size","action":{"kind":"size","task":{"step":"new","output":"task"},"expected_revision":{"step":"new","output":"task"}}}]);
    let (code, r) = f.v3("execute", &request);
    assert_eq!(code, 4);
    assert_eq!(r["steps"][1]["terminal"], "failed");
    assert_eq!(r["steps"][1]["result"]["status"], "failed");
}

#[path = "v4.rs"]
mod v4;
