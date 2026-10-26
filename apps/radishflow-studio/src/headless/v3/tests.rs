use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rf-headless-v3-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, value: &Value) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
        path
    }
    fn query(&self, request: Value) -> (i32, Value) {
        let path = self.write("query.json", &request);
        let mut out = Vec::new();
        let mut err = Vec::new();
        let code = query_cli(&path, &mut out, &mut err);
        assert!(err.is_empty());
        (code, serde_json::from_slice(&out).unwrap())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn empty_project() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../examples/flowsheets/binary-hydrocarbon-basis.rfproj.json"
    ))
    .unwrap()
}
fn equipment_project() -> Value {
    use rf_ui::engineering::editor::*;
    let root = empty_project();
    let StudioProjectOpen::Editable(mut app) =
        parse_studio_project(&root.to_string(), Path::new("fixture.json")).unwrap()
    else {
        panic!("editable")
    };
    let w = &mut app.workspace;
    for action in [
        EquipmentAction::CreateSizingTask,
        EquipmentAction::CreateRatingTask,
        EquipmentAction::CreateSource,
    ] {
        let context = EquipmentActionContext::capture(&w.engineering);
        w.equipment_editor
            .execute_action(
                &mut w.engineering,
                &rf_types::units::DisplayUnitSet::si(),
                EquipmentEditAccess::Editable,
                &context,
                action,
            )
            .unwrap();
    }
    let stored = rf_store::parse_project_file_json(&root.to_string()).unwrap();
    let inputs = app.workspace.engineering.inputs();
    let project =
        rf_store::project_v3::ProjectV3Read::Editable(rf_store::project_v3::StoredProjectV3 {
            document: stored.document,
            display_units: stored.presentation.display_units,
            engineering: Some(rf_store::project_v3::StoredEngineeringInputs {
                basis: inputs.basis.clone(),
                tasks: inputs.tasks.clone(),
            }),
        });
    serde_json::from_str(&rf_store::project_v3::to_pretty_json(&project).unwrap()).unwrap()
}

#[test]
fn query_v3_discovers_equipment_and_frozen_file_identity_without_cache() {
    let f = Fixture::new();
    let p = f.write("project.json", &equipment_project());
    let before = fs::read(&p).unwrap();
    let (code,r)=f.query(json!({"schema_version":3,"project":"project.json","queries":[{"id":"objects","query":{"kind":"objects"}}]}));
    assert_eq!(code, 0);
    assert_eq!(r["schema_version"], 3);
    assert_eq!(r["project"]["project_schema_version"], 3);
    assert_eq!(r["project"]["file_sha256"], rf_store::sha256_hex(&before));
    let objects = r["queries"][0]["value"].as_array().unwrap();
    assert!(objects.iter().any(|v| v["kind"] == "sizing"));
    assert!(objects.iter().any(|v| v["kind"] == "rating"));
    assert!(objects.iter().any(|v| v["kind"] == "geometry"));
    assert!(objects.iter().any(|v| v["kind"] == "source"));
    let member = objects.iter().find(|v| v["kind"] == "case").unwrap()["target"].clone();
    let (_, r) = f.query(json!({
        "schema_version": 3,
        "project": "project.json",
        "queries": [
            {
                "id": "fields",
                "query": {
                    "kind": "fields",
                    "target": member
                }
            },
            {
                "id": "results",
                "query": {
                    "kind": "results",
                    "scope": {
                        "object": member
                    }
                }
            }
        ]
    }));
    assert_eq!(r["queries"][0]["value"].as_array().unwrap().len(), 10);
    assert_eq!(r["queries"][0]["value"][0]["value"]["state"], "unspecified");
    assert_eq!(r["queries"][1]["value"]["status"], "missing");
    assert!(r["queries"][1]["value"].get("value").is_none());
    assert_eq!(fs::read(p).unwrap(), before);
}

#[test]
fn query_v3_unknown_equipment_retains_whole_project_read_only() {
    let f = Fixture::new();
    let mut p = equipment_project();
    p["engineering"]["tasks"][0]["method"]["version"] = json!(999);
    f.write("project.json", &p);
    let (code, r) = f.query(json!({
        "schema_version": 3,
        "project": "project.json",
        "queries": [
            {
                "id": "objects",
                "query": {
                    "kind": "objects"
                }
            },
            {
                "id": "device",
                "query": {
                    "kind": "record",
                    "target": {
                        "equipment": {
                            "id": "task-ui-1"
                        }
                    }
                }
            }
        ]
    }));
    assert_eq!(code, 2);
    assert_eq!(r["project"]["read_only"], true);
    assert_eq!(r["project"]["equipment_task_count"], 2);
    assert!(r["queries"][0]["value"].is_array());
    assert_eq!(r["queries"][1]["error"]["code"], "unsupported_capability");
    let path = f.0.join("project.json");
    let mut out = Vec::new();
    assert_eq!(
        super::super::run_cli(&["inspect".into(), path.into()], &mut out, &mut Vec::new()),
        2
    );
    let old: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(old["schema_version"], 1);
}

#[test]
fn query_v3_strict_nested_json_rejects_duplicates_unknowns_and_nulls() {
    for text in [
        r#"{"a":{"x":1,"x":2}}"#,
        r#"{"a":[{"x":1,"x":2}]}"#,
        r#"{"value":1e999}"#,
    ] {
        assert!(strict::validate(text.as_bytes()).is_err(), "{text}");
    }
    for text in [
        r#"{"schema_version":3,"project":"x","queries":null}"#,
        r#"{"schema_version":3,"project":"x","queries":[{"id":"a","query":{"kind":"objects","extra":1}}]}"#,
        r#"{"schema_version":3,"project":"x","queries":[{"id":"a","query":{"kind":"fields","target":{"unit":{"id":"u","step":"s","output":"unit"}}}}]}"#,
        r#"{"schema_version":3,"project":"x","queries":[],"steps":[]}"#,
    ] {
        assert!(
            serde_json::from_str::<QueryRequest>(text).is_err(),
            "{text}"
        );
    }
    assert!(
        serde_json::from_str::<records::SourceApplicability>(
            r#"{"fluid_system":"","phase":"gas","scope":"","operating_conditions":null}"#
        )
        .is_err()
    );
}

#[test]
fn query_v3_errors_are_independent_and_do_not_hide_later_reads() {
    let f = Fixture::new();
    f.write("project.json", &empty_project());
    let (code, r) = f.query(json!({
        "schema_version": 3,
        "project": "project.json",
        "queries": [
            {
                "id": "missing",
                "query": {
                    "kind": "fields",
                    "target": {
                        "unit": {
                            "id": "absent"
                        }
                    }
                }
            },
            {
                "id": "all",
                "query": {
                    "kind": "objects"
                }
            }
        ]
    }));
    assert_eq!(code, 2);
    assert_eq!(r["error"]["query_id"], "missing");
    assert!(r["queries"][1]["value"].is_array());
}

#[test]
fn query_v3_rejects_versions_before_project_io_and_preserves_output_failure() {
    let f = Fixture::new();
    let (code, r) = f.query(json!({"schema_version":2,"project":"absent","queries":[]}));
    assert_eq!(code, 2);
    assert_eq!(r["error"]["code"], "unsupported_version");
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut err = Vec::new();
    assert_eq!(query_cli(&f.0.join("query.json"), &mut Broken, &mut err), 5);
    assert!(!err.is_empty());
}

#[test]
fn query_v3_fingerprint_covers_same_revision_equipment_edits_and_exact_bytes() {
    let f = Fixture::new();
    let mut project = equipment_project();
    let path = f.write("project.json", &project);
    let request = json!({"schema_version":3,"project":"project.json","queries":[]});
    let (_, a) = f.query(request.clone());
    project["engineering"]["tasks"][0]["name"] = json!("different input, same revisions");
    f.write("project.json", &project);
    let (_, b) = f.query(request.clone());
    assert_eq!(a["project"]["document_id"], b["project"]["document_id"]);
    assert_eq!(a["project"]["flow_revision"], b["project"]["flow_revision"]);
    assert_ne!(a["project"]["file_sha256"], b["project"]["file_sha256"]);
    fs::write(path, project.to_string()).unwrap();
    let (_, c) = f.query(request);
    assert_ne!(b["project"]["file_sha256"], c["project"]["file_sha256"]);
}

#[test]
fn query_v3_read_only_flow_fields_never_advertise_writes() {
    let f = Fixture::new();
    let mut project = equipment_project();
    let flow: Value = serde_json::from_str(include_str!(
        "../../../../../examples/flowsheets/feed-heater-flash-binary-hydrocarbon.rfproj.json"
    ))
    .unwrap();
    project["document"] = flow["document"].clone();
    project["engineering"]["tasks"][0]["method"]["version"] = json!(999);
    f.write("project.json", &project);
    let unit = project["document"]["flowsheet"]["units"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap();
    let (code,r)=f.query(json!({"schema_version":3,"project":"project.json","queries":[{"id":"fields","query":{"kind":"fields","target":{"unit":{"id":unit}}}}]}));
    assert_eq!(code, 0, "{r}");
    let fields = r["queries"][0]["value"].as_array().unwrap();
    assert!(!fields.is_empty());
    assert!(
        fields.iter().all(|f| f["variable"]["writable"] == false
            && f["variable"]["target"]["section"] == "inputs")
    );
    let mut out = Vec::new();
    assert_eq!(
        super::super::run_cli(&["query".into()], &mut out, &mut Vec::new()),
        2
    );
    let r: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(r["schema_version"], 3);
}
