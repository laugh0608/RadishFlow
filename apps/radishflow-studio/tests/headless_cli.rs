use radishflow_studio::{load_project_app_state, test_support};
use rf_store::{StoredAuthCacheIndex, StoredCredentialReference, write_auth_cache_index};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "rf-headless-{}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::write(
            root.join("工程 example.rfproj.json"),
            include_bytes!(
                "../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
            ),
        )
        .unwrap();
        let mut index = StoredAuthCacheIndex::new(
            "https://example.invalid",
            "headless-test",
            StoredCredentialReference::new("test", "headless-test"),
        );
        test_support::write_official_binary_hydrocarbon_cached_package(
            &root.join("cache"),
            &mut index,
            test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID,
            SystemTime::now(),
            None,
        );
        write_auth_cache_index(root.join("index.json"), &index).unwrap();
        Self { root }
    }
    fn request(&self) -> Value {
        let mut request: Value = serde_json::from_str(include_str!(
            "../../../examples/automation/heater-330k.request.json"
        ))
        .unwrap();
        request["project"] = json!("工程 example.rfproj.json");
        request["auth_cache_index"] = json!("index.json");
        request
    }
    fn run(&self, request: &Value) -> (i32, Value) {
        let path = self.root.join("run request.json");
        fs::write(&path, serde_json::to_vec(request).unwrap()).unwrap();
        self.command("run", &path)
    }
    fn command(&self, verb: &str, path: &std::path::Path) -> (i32, Value) {
        let output = Command::new(env!("CARGO_BIN_EXE_radishflow-studio"))
            .args(["--headless", verb])
            .arg(path)
            .env(
                "RADISHFLOW_STUDIO_PREFERENCES_PATH",
                self.root.join("preferences.json"),
            )
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .output()
            .unwrap();
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!self.root.join("preferences.json").exists());
        assert!(
            !self
                .root
                .join("工程 example.rfproj.json.rfstudio-layout.json")
                .exists()
        );
        (
            output.status.code().unwrap(),
            serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
                panic!(
                    "invalid output: {e}: {}",
                    String::from_utf8_lossy(&output.stdout)
                )
            }),
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn headless_process_inspects_writes_runs_and_reads_without_mutating_files() {
    let f = Fixture::new();
    let project_path = f.root.join("工程 example.rfproj.json");
    let original = fs::read(&project_path).unwrap();
    let index_bytes = fs::read(f.root.join("index.json")).unwrap();
    let (code, inspect) = f.command("inspect", &project_path);
    assert_eq!(code, 0);
    let mut request = f.request();
    request["document_id"] = inspect["document"]["id"].clone();
    request["expected_revision"] = inspect["document"]["revision"].clone();
    // Use discovery's target verbatim instead of reconstructing its address from a label.
    request["writes"][0]["target"] = inspect["variables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| {
            v["target"]["object"]["unit"] == "heater-1"
                && v["target"]["field"] == "outlet_temperature"
        })
        .unwrap()["target"]
        .clone();
    assert!(
        inspect["variables"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["state"] == "missing" && v["value"].is_null())
    );
    let (code, result) = f.run(&request);
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["status"], "ok");
    assert_eq!(result["document"]["revision"], 1);
    assert_eq!(result["variables"][0]["value"], 330.0);
    assert_eq!(result["variables"][0]["source"]["revision"], 1);
    assert_eq!(result["variables"][1]["value"], 0.0);
    assert_eq!(result["variables"][1]["source"]["kind"], "stream_template");
    assert_eq!(result["variables"][2]["value"], 5.0);
    let mut app = load_project_app_state(&project_path).unwrap();
    app.write_variable(
        rf_ui::variable_commands::VariableWriteRequest {
            variable: rf_ui::variable_browser::VariableId {
                document: app.workspace.document.metadata.document_id.clone(),
                object: rf_ui::variable_browser::ObjectId::Unit("heater-1".into()),
                section: rf_ui::variable_browser::VariableSection::Inputs,
                field: rf_ui::variable_browser::VariableField::OutletTemperature,
            },
            expected_revision: 0,
            value: rf_ui::variable_browser::VariableValue::Number(330.0),
        },
        UNIX_EPOCH,
    )
    .unwrap();
    let index = rf_store::read_auth_cache_index(f.root.join("index.json")).unwrap();
    let root = f.root.join("cache");
    let context = radishflow_studio::StudioAppAuthCacheContext::new(&root, &index);
    radishflow_studio::dispatch_workspace_control_action_with_auth_cache(
        &radishflow_studio::StudioAppFacade::new(),
        &mut app,
        &context,
        &radishflow_studio::WorkspaceControlAction::run_manual(
            radishflow_studio::WorkspaceRunPackageSelection::Preferred,
        ),
    )
    .unwrap();
    let stream = rf_ui::latest_snapshot(&app.workspace)
        .unwrap()
        .streams
        .iter()
        .find(|s| s.stream_id.as_str() == "stream-heated")
        .unwrap();
    assert_eq!(result["variables"][0]["value"], stream.temperature_k);
    assert_eq!(
        result["variables"][2]["value"],
        stream.total_molar_flow_mol_s
    );
    assert_eq!(fs::read(&project_path).unwrap(), original);
    assert_eq!(fs::read(f.root.join("index.json")).unwrap(), index_bytes);
    // Each run starts from the input document; nothing is accumulated between invocations.
    assert_eq!(f.run(&request), (code, result));
}

#[test]
fn headless_process_rejects_protocol_and_document_conflicts() {
    let f = Fixture::new();
    for (key, value, expected) in [
        ("schema_version", json!(99), "unsupported_version"),
        ("typo", json!(true), "invalid_json"),
        ("expected_revision", json!(1), "revision_conflict"),
        ("document_id", json!("foreign"), "different_document"),
        ("project", json!(""), "empty_path"),
    ] {
        let mut request = f.request();
        request[key] = value;
        let (code, result) = f.run(&request);
        assert_eq!(code, 2, "{result}");
        assert_eq!(result["error"]["code"], expected);
        assert_eq!(result["writes"], json!([]));
        assert_eq!(result["variables"], json!([]));
    }
}

#[test]
fn headless_process_stops_at_invalid_write_and_keeps_project_bytes() {
    let f = Fixture::new();
    let original = fs::read(f.root.join("工程 example.rfproj.json")).unwrap();
    let mut request = f.request();
    let mut invalid = request["writes"][0].clone();
    invalid["value"] = json!(-1);
    request["writes"].as_array_mut().unwrap().push(invalid);
    let (code, result) = f.run(&request);
    assert_eq!(code, 2);
    assert_eq!(result["error"]["stage"], "write");
    assert_eq!(result["error"]["index"], 1);
    assert_eq!(result["writes"].as_array().unwrap().len(), 1);
    assert!(result["package_id"].is_null());
    assert_eq!(result["variables"], json!([]));
    assert_eq!(
        fs::read(f.root.join("工程 example.rfproj.json")).unwrap(),
        original
    );
    request["writes"][0]["target"]["section"] = json!("results");
    assert_eq!(f.run(&request).1["error"]["code"], "read_only");
}

#[test]
fn headless_process_distinguishes_missing_cache_and_blocked_model() {
    let f = Fixture::new();
    let mut request = f.request();
    request["auth_cache_index"] = json!("absent.json");
    let (code, result) = f.run(&request);
    assert_eq!(code, 2);
    assert_eq!(result["error"]["stage"], "cache");
    let path = f.root.join("工程 example.rfproj.json");
    let mut project = rf_store::read_project_file(&path).unwrap();
    project
        .document
        .flowsheet
        .units
        .get_mut(&"heater-1".into())
        .unwrap()
        .ports
        .iter_mut()
        .find(|p| p.name == "inlet")
        .unwrap()
        .stream_id = None;
    rf_store::write_project_file(&path, &project).unwrap();
    let (code, result) = f.run(&f.request());
    assert_eq!(code, 4, "{result}");
    assert_eq!(
        result["diagnostic"]["ports"][0],
        json!({"unit_id":"heater-1","port":"inlet"})
    );
    assert_eq!(result["variables"], json!([]));
    project
        .document
        .flowsheet
        .units
        .get_mut(&"heater-1".into())
        .unwrap()
        .ports
        .iter_mut()
        .find(|p| p.name == "inlet")
        .unwrap()
        .stream_id = Some("stream-feed".into());
    project
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-feed".into())
        .unwrap()
        .overall_mole_fractions
        .insert("methane".into(), 0.2);
    rf_store::write_project_file(&path, &project).unwrap();
    let (code, result) = f.run(&f.request());
    assert_eq!(code, 3, "{result}");
    assert_eq!(result["status"], "blocked");
    assert_eq!(result["error"]["code"], "unnormalized_stream_composition");
    assert_eq!(result["variables"], json!([]));
}

#[test]
fn headless_process_preserves_solver_failure_diagnostic_targets() {
    let f = Fixture::new();
    let mut project = rf_store::parse_project_file_json(include_str!(
        "../../../examples/flowsheets/feed-valve-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    project
        .document
        .flowsheet
        .units
        .get_mut(&"valve-1".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(730_000.0);
    project
        .document
        .flowsheet
        .streams
        .get_mut(&"stream-throttled".into())
        .unwrap()
        .pressure_pa = 730_000.0;
    rf_store::write_project_file(f.root.join("valve.json"), &project).unwrap();
    let mut request = f.request();
    request["project"] = json!("valve.json");
    request["document_id"] = json!(project.document.metadata.document_id);
    request["writes"] = json!([]);
    let (code, result) = f.run(&request);
    assert_eq!(code, 4, "{result}");
    assert_eq!(result["status"], "failed");
    assert_eq!(result["error"]["code"], "solve_failed");
    assert_eq!(result["diagnostic"]["code"], "solver.step.parameter");
    assert!(
        result["diagnostic"]["unit_ids"]
            .as_array()
            .unwrap()
            .contains(&json!("valve-1"))
    );
    assert_eq!(result["variables"], json!([]));
}

#[test]
fn headless_process_rejects_missing_query_without_partial_values() {
    let f = Fixture::new();
    let mut request = f.request();
    request["reads"][2]["object"] = json!({"stream":"not-found"});
    let (code, result) = f.run(&request);
    assert_eq!(code, 2, "{result}");
    assert_eq!(result["error"]["stage"], "read");
    assert_eq!(result["error"]["index"], 2);
    assert_eq!(result["variables"], json!([]));
}

#[test]
fn headless_process_reports_unavailable_assets_without_seeding_replacements() {
    let f = Fixture::new();
    let index = rf_store::read_auth_cache_index(f.root.join("index.json")).unwrap();
    let manifest = index.property_packages[0].manifest_path_under(f.root.join("cache"));
    fs::remove_file(&manifest).unwrap();
    let (code, result) = f.run(&f.request());
    assert_eq!(code, 4, "{result}");
    assert_eq!(result["error"]["code"], "local_cache_unavailable");
    assert_eq!(result["variables"], json!([]));
    assert!(!manifest.exists());
}

#[test]
fn headless_process_returns_json_for_bad_arguments_and_unreadable_files() {
    let f = Fixture::new();
    let missing = f.root.join("missing.json");
    for (verb, stage) in [
        ("inspect", "project"),
        ("run", "request"),
        ("unknown", "arguments"),
    ] {
        let (code, result) = f.command(verb, &missing);
        assert_eq!(code, 2, "{result}");
        assert_eq!(result["error"]["stage"], stage);
    }
    let invalid = f.root.join("invalid.json");
    fs::write(&invalid, "{\"schema_version\":").unwrap();
    assert_eq!(
        f.command("run", &invalid).1["error"]["code"],
        "invalid_json"
    );
}

#[path = "headless_cli/workflow.rs"]
mod workflow;
