use super::*;
use crate::{FileOverwritePolicy, project_file_to_pretty_json};
use rf_model::UnitNode;
use serde_json::{Value, json};
use std::fs;
mod simultaneous;

fn fixture() -> StoredProjectV4 {
    let mut file = crate::parse_project_file_json(include_str!(
        "../../tests/fixtures/project-v2-engineering.json"
    ))
    .unwrap();
    file.document
        .flowsheet
        .insert_unit(UnitNode::new("unit", "Feed", "feed", vec![]))
        .unwrap();
    file.document
        .flowsheet
        .insert_stream(rf_model::MaterialStreamState::new("stream", "Stream"))
        .unwrap();
    StoredProjectV4 {
        document: file.document,
        display_units: file.presentation.display_units,
        engineering: None,
        steady_state_plan: StoredFlowPlan::Acyclic,
    }
}
fn recycle() -> StoredFlowPlan {
    StoredFlowPlan::Partitioned(StoredRecyclePlan {
        fixed_boundary: StoredFixedBoundary {
            temperature_k: 300.0,
            pressure_pa: 650_000.0,
        },
        max_iteration_evaluations: 500,
        tears: vec![StoredTear {
            stream_id: "return".into(),
            source: StoredPortBinding {
                unit_id: "splitter".into(),
                port: "outlet_a".into(),
            },
            sink: StoredPortBinding {
                unit_id: "mixer".into(),
                port: "inlet_b".into(),
            },
            initialization: StoredTearInitialization::ZeroComponentFlows,
            residual_scaling: StoredResidualScaling::EvaluatedFreshTotal,
        }],
    })
}
fn json_project(project: StoredProjectV4) -> String {
    to_pretty_json(&ProjectV4Read::Editable(Box::new(project))).unwrap()
}
#[test]
fn known_projects_roundtrip_formal_inputs_and_plan_without_results() {
    for engineering in [None, Some(StoredEngineeringInputs::default())] {
        let mut project = fixture();
        project.engineering = engineering;
        project.steady_state_plan = recycle();
        let mut splitter = UnitNode::new("splitter", "S", "splitter", vec![]);
        splitter.parameters.outlet_a_split_fraction = Some(0.6);
        project.document.flowsheet.insert_unit(splitter).unwrap();
        let text = json_project(project.clone());
        assert_eq!(
            parse(&text).unwrap(),
            ProjectV4Read::Editable(Box::new(project))
        );
        assert!(text.contains("outlet_a_split_fraction"));
        assert!(!text.contains("result"));
        assert!(crate::parse_project_file_json(&text).is_err());
        assert!(crate::project_v3::parse(&text).is_err());
    }
}
#[test]
fn explicit_vectors_and_empty_incomplete_plan_roundtrip() {
    let mut project = fixture();
    let StoredFlowPlan::Partitioned(mut plan) = recycle() else {
        unreachable!()
    };
    plan.tears[0].initialization = StoredTearInitialization::ComponentFlows {
        values_mol_s: vec![StoredComponentValue {
            component_id: "methane".into(),
            value: 10.0,
        }],
    };
    plan.tears[0].residual_scaling = StoredResidualScaling::ComponentValues {
        values_mol_s: vec![StoredComponentValue {
            component_id: "methane".into(),
            value: 5.0,
        }],
    };
    for tears in [plan.tears.clone(), vec![]] {
        plan.tears = tears;
        project.steady_state_plan = StoredFlowPlan::Partitioned(plan.clone());
        assert_eq!(
            parse(&json_project(project.clone())).unwrap(),
            ProjectV4Read::Editable(Box::new(project.clone()))
        );
    }
}
#[test]
fn unknown_capabilities_and_methods_are_opaque_readonly_and_never_written() {
    let text = json_project(fixture());
    for mode in 0..5 {
        let mut value: Value = serde_json::from_str(&text).unwrap();
        match mode {
            0 => {
                value["requiredCapabilities"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"id":"future.model","version":2}));
                value["simulation"] = json!({"opaque":[{"future":"payload"}]});
            }
            1 => value["simulation"]["steadyStatePlan"]["method"]["version"] = json!(2),
            2 => {
                value["simulation"]["steadyStatePlan"] = serde_json::to_value(recycle()).unwrap();
                value["simulation"]["steadyStatePlan"]["acceptanceContractVersion"] = json!(2);
            }
            4 => {
                value["requiredCapabilities"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"id":"future.display","version":1}));
                value["futureSection"] = json!({"opaque":["retain everything"]});
                value["presentation"] = json!({"futureDisplayField":true});
            }
            _ => {
                value["engineering"] = json!({"basisEntries":[],"sources":[],"tasks":[{"id":"future-task","kind":"future","method":{"id":"future.method","version":1},"arbitrary":["do not decode"]}]})
            }
        }
        let raw = value.to_string();
        let read = parse(&raw).unwrap();
        let ProjectV4Read::ReadOnly(project) = &read else {
            panic!("unknown capability must be readonly")
        };
        assert_eq!(project.original_json(), raw);
        assert!(!project.reasons().is_empty());
        assert!(to_pretty_json(&read).is_err());
    }
}
#[test]
fn known_envelopes_reject_unknown_fields_duplicate_keys_and_capability_lies() {
    let text = json_project(fixture());
    for mode in 0..10 {
        let mut value: Value = serde_json::from_str(&text).unwrap();
        match mode {
            0 => value["simulation"]["steadyStatePlan"]["unknown"] = json!(1),
            1 => value["simulation"]["steadyStatePlan"]["tears"] = json!([]),
            2 => value["simulation"] = Value::Null,
            3 => value["engineering"] = Value::Null,
            4 => value["requiredCapabilities"] = json!([]),
            5 => value["requiredCapabilities"]
                .as_array_mut()
                .unwrap()
                .push(json!({"id":FLOW_PLAN_CAPABILITY,"version":1})),
            6 => value["schemaVersion"] = json!(5),
            7 => {
                value["document"]["flowsheet"]["units"]
                    .as_object_mut()
                    .unwrap()
                    .values_mut()
                    .next()
                    .unwrap()["parameters"] = json!({"future_ratio":0.6})
            }
            9 => value["futureSection"] = json!({"unknown":true}),
            _ => value["simulation"]["steadyStatePlan"]["acceptanceContractVersion"] = Value::Null,
        }
        assert!(parse(&value.to_string()).is_err(), "case {mode}");
    }
    let duplicate = text.replacen(
        "\"schemaVersion\": 4",
        "\"schemaVersion\": 4, \"schemaVersion\": 4",
        1,
    );
    assert!(parse(&duplicate).is_err());
}
#[test]
fn legacy_writers_keep_splitter_guard_and_v4_does_not_accept_ratio_on_another_kind() {
    let mut project = fixture();
    let unit = project
        .document
        .flowsheet
        .units
        .values_mut()
        .next()
        .unwrap();
    unit.parameters.outlet_a_split_fraction = Some(0.6);
    assert!(to_pretty_json(&ProjectV4Read::Editable(Box::new(project.clone()))).is_err());
    let file = crate::StoredProjectFile {
        kind: STORED_PROJECT_FILE_KIND.into(),
        schema_version: 2,
        document: project.document,
        presentation: Default::default(),
    };
    assert!(project_file_to_pretty_json(&file).is_err());
    assert!(serde_json::to_string(&file).is_err());
    let mut project = fixture();
    project
        .document
        .flowsheet
        .streams
        .values_mut()
        .next()
        .unwrap()
        .phases
        .push(rf_model::PhaseState {
            label: rf_types::PhaseLabel::Overall,
            phase_fraction: 1.0,
            mole_fractions: Default::default(),
            molar_enthalpy_j_per_mol: Some(f64::NAN),
        });
    assert!(to_pretty_json(&ProjectV4Read::Editable(Box::new(project))).is_err());
}
struct Directory(std::path::PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "rf-project-v4-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn prepared_save_binds_target_and_preserves_old_bytes_on_cancel_or_change() {
    let dir = Directory::new();
    let path = dir.0.join("project.json");
    let legacy = include_str!("../../tests/fixtures/project-v2-engineering.json");
    fs::write(&path, legacy).unwrap();
    let project = ProjectV4Read::Editable(Box::new(fixture()));
    let prepared = prepare_write(&path, &project, FileOverwritePolicy::Allow).unwrap();
    assert!(prepared.requires_upgrade());
    assert!(write_prepared(&prepared, false).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), legacy);
    fs::write(&path, format!("{legacy}\n")).unwrap();
    assert_eq!(
        write_prepared(&prepared, true)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("store.project.save_target_changed")
    );
    let fresh = prepare_write(&path, &project, FileOverwritePolicy::Allow).unwrap();
    write_prepared(&fresh, true).unwrap();
    assert_eq!(read(&path).unwrap(), project);
    assert!(prepare_write(&path, &project, FileOverwritePolicy::Forbid).is_err());
}
#[test]
fn unsupported_targets_and_new_target_races_are_protected() {
    let dir = Directory::new();
    let path = dir.0.join("project.json");
    let project = ProjectV4Read::Editable(Box::new(fixture()));
    let prepared = prepare_write(&path, &project, FileOverwritePolicy::Forbid).unwrap();
    fs::write(&path, "unrelated bytes").unwrap();
    assert!(write_prepared(&prepared, true).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "unrelated bytes");
    let mut value: Value = serde_json::from_str(&json_project(fixture())).unwrap();
    value["simulation"]["steadyStatePlan"]["method"]["version"] = json!(99);
    let raw = value.to_string();
    fs::write(&path, &raw).unwrap();
    assert!(prepare_write(&path, &project, FileOverwritePolicy::Allow).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), raw);
}

#[test]
fn v4_roundtrip_preserves_all_v3_rating_adoption_and_display_inputs() {
    for raw in [
        include_str!("../../tests/fixtures/project-v3-rating.json"),
        include_str!("../../tests/fixtures/project-v3-rating-adoption.json"),
    ] {
        let crate::project_v3::ProjectV3Read::Editable(old) =
            crate::project_v3::parse(raw).unwrap()
        else {
            panic!()
        };
        let project = ProjectV4Read::Editable(Box::new(StoredProjectV4 {
            document: old.document,
            display_units: old.display_units,
            engineering: old.engineering,
            steady_state_plan: recycle(),
        }));
        let json = to_pretty_json(&project).unwrap();
        assert_eq!(parse(&json).unwrap(), project);
    }
}

mod equation_oriented;
