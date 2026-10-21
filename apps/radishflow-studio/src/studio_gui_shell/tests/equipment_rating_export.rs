use super::equipment_editor::change;
use super::equipment_export::{apply, csv_records, row};
use super::*;
use equipment::CalculationTableFormat as Format;
use rf_model::equipment::*;
use rf_types::units::DisplayUnitSet;
use rf_ui::engineering::{EquipmentCaseKey, EquipmentResultStatus, editor::*};
use serde_json::Value;

pub(super) fn member() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: "rating-1".into(),
        case_id: "rating-case-1".into(),
    }
}
pub(super) fn fixture(name: &str, edit: impl FnOnce(&mut Value)) -> (ReadyAppState, PathBuf) {
    let path = test_preferences_path(name).with_extension("rfproj.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut data: Value = serde_json::from_str(include_str!(
        "../../../../../crates/rf-store/tests/fixtures/project-v3-rating.json"
    ))
    .unwrap();
    edit(&mut data);
    fs::write(&path, data.to_string()).unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    app.enter_equipment();
    (app, path)
}
pub(super) fn calculate(app: &mut ReadyAppState) {
    app.equipment_command(EquipmentEditorCommand::CalculateRating(member()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&member()),
        EquipmentResultStatus::Current
    );
}
fn output(app: &mut ReadyAppState, format: Format, path: &std::path::Path) {
    let capture = app
        .capture_equipment_output(&member().task_id, format)
        .unwrap();
    app.export_equipment_to_path(capture, path.to_owned());
}

#[test]
fn rating_export_preserves_signed_precision_geometry_evidence_units_and_csv_json_parity() {
    let (mut app, project) = fixture("rating-table-evidence", |v| {
        v["engineering"]["tasks"][0]["geometry"]["axialSections"] = serde_json::json!({});
        v["engineering"]["sources"][0]["title"] = serde_json::json!("=SUM(1,2)\n\"合成来源\"");
    });
    calculate(&mut app);
    let before = app.platform_host.modeling_inputs().workspace.clone();
    let json = project.with_file_name("rating.json");
    let csv = project.with_file_name("rating.csv");
    output(&mut app, Format::Json, &json);
    output(&mut app, Format::Csv, &csv);
    assert_eq!(app.platform_host.modeling_inputs().workspace, before);
    let table: Value = serde_json::from_str(&fs::read_to_string(json).unwrap()).unwrap();
    assert_eq!(table["format"], "radishflow.separator-rating-table");
    assert_eq!(table["version"], 1);
    for (section, id, field, expected) in [
        ("metadata", "", "taskId", "rating-1"),
        ("metadata", "", "caseId", "rating-case-1"),
        ("result", "rating-case-1", "conclusion", "exceeded"),
        (
            "geometryInput",
            "internalDiameter",
            "overrideReason",
            "Explicit revised geometry",
        ),
    ] {
        assert_eq!(row(&table, section, id, field)["text"], expected);
    }
    for (section, id, si, displayed, unit) in [
        (
            "geometryInput",
            "internalDiameter",
            (4.0 / std::f64::consts::PI).sqrt(),
            "1128.38",
            "millimetre",
        ),
        (
            "input",
            "gasActualVolumeFlow",
            1.0,
            "3600",
            "cubic_metre_per_hour",
        ),
        ("input", "requiredWorkingTime", 120.0, "2", "minute"),
        ("input", "operatingTemperature", 300.0, "26.85", "celsius"),
    ] {
        let r = row(&table, section, id, "effectiveValue");
        assert_eq!(r["valueSi"], si);
        assert_eq!(r["displayValue"], displayed);
        assert_eq!(r["displayUnitId"], unit);
    }
    for (field, expected) in [
        ("gasLoadFraction", 1.25),
        ("liquidLoadFraction", 1.2),
        ("gasMarginFraction", -0.25),
        ("liquidMarginFraction", -0.2),
    ] {
        let r = row(&table, "result", "rating-case-1", field);
        assert!((r["valueSi"].as_f64().unwrap() - expected).abs() < 1e-14);
        if expected < 0.0 {
            assert!(r["displayValue"].as_str().unwrap().starts_with('-'));
        }
    }
    for field in ["tangentToTangentHeight", "bufferVolume"] {
        let r = row(&table, "result", "rating-case-1", field);
        assert_eq!(r["status"], "unknown");
        assert!(r["valueSi"].is_null());
        assert!(r["displayValue"].is_null());
    }
    let rows = table["rows"].as_array().unwrap();
    for section in ["source", "basis", "unchecked", "applicability"] {
        assert!(
            rows.iter().any(|r| r["section"] == section),
            "missing {section}"
        );
    }
    assert!(
        rows.iter()
            .any(|r| r["field"] == "loadBoundaryTolerance" && r["valueSi"] == 1e-10)
    );
    assert_eq!(
        row(
            &table,
            "source",
            "source-test",
            "observation.0.originalUnit"
        )["text"],
        "m3/h"
    );
    assert_eq!(
        row(
            &table,
            "source",
            "source-test",
            "observation.0.canonicalValue"
        )["valueSi"],
        1.0
    );
    let records = csv_records(&fs::read_to_string(csv).unwrap());
    assert_eq!(records.len(), rows.len() + 1);
    for (csv, json) in records[1..].iter().zip(rows) {
        assert_eq!(csv.len(), 11);
        for (index, field) in records[0].iter().enumerate() {
            let value = &json[field];
            let mut expected = if value.is_null() {
                String::new()
            } else if let Some(s) = value.as_str() {
                s.to_owned()
            } else {
                value.as_f64().unwrap().to_string()
            };
            if field == "text" && expected.starts_with('=') {
                expected.insert(0, '\'');
            }
            if let Some(number) = value.as_f64() {
                let csv_number: f64 = csv[index].parse().unwrap();
                // serde_json's default float reader may differ by one ulp.
                assert!(
                    (csv_number - number).abs() <= f64::EPSILON * number.abs().max(1.0),
                    "{field}: {json}"
                );
            } else {
                assert_eq!(csv[index], expected, "{field}: {json}");
            }
        }
    }
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

#[test]
fn rating_boundary_and_within_criteria_are_exportable_without_rounding_the_decision() {
    for (gas, liquid, conclusion) in [
        (0.8, 0.005, "boundaryReview"),
        (0.4, 0.002, "withinCriteria"),
        (0.80000000016, 0.002, "exceeded"),
    ] {
        let (mut app, project) = fixture("rating-boundary", |v| {
            let inputs = &mut v["engineering"]["tasks"][0]["cases"][0]["inputs"];
            inputs["gasActualVolumeFlow"]["valueSi"] = gas.into();
            inputs["liquidActualVolumeFlow"]["valueSi"] = liquid.into();
        });
        calculate(&mut app);
        let path = project.with_file_name("rating.json");
        output(&mut app, Format::Json, &path);
        let table: Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(
            row(&table, "result", "rating-case-1", "conclusion")["text"],
            conclusion
        );
        if gas > 0.8 {
            assert_eq!(
                row(&table, "result", "rating-case-1", "gasLoadFraction")["displayValue"],
                "1"
            );
        }
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }
}

#[test]
fn rating_export_rechecks_identity_dependencies_drafts_and_project_units_at_overwrite() {
    for mutation in [
        "rerun", "geometry", "source", "draft", "units", "delete", "reopen",
    ] {
        let (mut app, project) = fixture("rating-export-recheck", |_| {});
        calculate(&mut app);
        let path = project.with_file_name("rating.json");
        fs::write(&path, "keep original").unwrap();
        output(&mut app, Format::Json, &path);
        assert!(app.equipment.pending_export.is_some());
        match mutation {
            "rerun" => calculate(&mut app),
            "geometry" => {
                let target = EquipmentEditorTarget::RatingGeometry(member().task_id);
                app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
                change(&mut app, &target, |d| {
                    let EquipmentDraft::RatingGeometry(d) = d else {
                        panic!()
                    };
                    d.fields
                        .get_mut(&RatingGeometryField::Input(
                            SeparatorGeometryInput::WorkingLiquidHeight,
                        ))
                        .unwrap()
                        .value
                        .set_text("700".into());
                });
                apply(&mut app, &target);
            }
            "source" => {
                let target = EquipmentEditorTarget::Source("drawing".into());
                app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
                change(&mut app, &target, |d| {
                    let EquipmentDraft::Source(d) = d else {
                        panic!()
                    };
                    d.publication_version = "revised".into();
                });
                apply(&mut app, &target);
            }
            "draft" => {
                let target = EquipmentEditorTarget::RatingCase(member());
                app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
                change(&mut app, &target, |d| {
                    let EquipmentDraft::RatingCase(d) = d else {
                        panic!()
                    };
                    d.inputs
                        .get_mut(&SeparatorRatingInput::GasVolumeFlow)
                        .unwrap()
                        .value
                        .set_text("1e-".into());
                });
            }
            "units" => {
                app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(
                    DisplayUnitSet::si(),
                ))
                .unwrap();
            }
            "delete" => {
                let target = EquipmentEditorTarget::RatingTask(member().task_id);
                app.equipment_command(EquipmentEditorCommand::Delete(target));
                let c = app
                    .platform_host
                    .modeling_inputs()
                    .workspace
                    .equipment_editor
                    .confirmation()
                    .unwrap()
                    .key();
                app.equipment_command(EquipmentEditorCommand::Confirm(c));
            }
            "reopen" => {
                app.open_project(project.clone(), "reopen rating");
                calculate(&mut app);
            }
            _ => unreachable!(),
        }
        app.confirm_equipment_export();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "keep original",
            "{mutation}"
        );
        fs::remove_dir_all(project.parent().unwrap()).unwrap();
    }
}

#[test]
fn rating_export_cancel_io_failure_and_success_leave_engineering_unchanged() {
    let (mut app, project) = fixture("rating-export-io", |_| {});
    calculate(&mut app);
    let before = app.platform_host.modeling_inputs().workspace.clone();
    let path = project.with_file_name("rating.json");
    fs::write(&path, "old").unwrap();
    output(&mut app, Format::Json, &path);
    app.cancel_equipment_export();
    assert_eq!(fs::read_to_string(&path).unwrap(), "old");
    output(&mut app, Format::Json, &path);
    app.confirm_equipment_export();
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("radishflow.separator-rating-table")
    );
    let directory = project.with_file_name("directory.json");
    fs::create_dir(&directory).unwrap();
    output(&mut app, Format::Json, &directory);
    assert_eq!(
        app.project_open.notice.as_ref().unwrap().level,
        ProjectOpenNoticeLevel::Error
    );
    assert_eq!(app.platform_host.modeling_inputs().workspace, before);
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}
