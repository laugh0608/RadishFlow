use super::equipment_editor::{change, complete_case};
use super::equipment_export::{apply, csv_records, row};
use super::*;
use equipment::CalculationTableFormat as Format;
use rf_model::equipment::*;
use rf_types::{
    EquipmentTaskId,
    units::{MeasurementUnit, QuantityKind},
};
use rf_ui::engineering::{EquipmentCaseKey, EquipmentResultStatus, editor::*};
use serde_json::Value;
mod ui;

#[test]
fn adoption_fixed_review_survives_source_edits_and_readoption_is_a_new_history_step() {
    let (mut app, source) = current_case();
    let key = adopt(&mut app, &source, true);
    let evidence = task(&app, &key).geometry.adoption.clone();
    app.equipment_command(EquipmentEditorCommand::CalculateRating(key.clone()));
    let source_id = evidence.as_ref().unwrap().source_basis.sources[0]
        .id
        .clone();
    let target = EquipmentEditorTarget::Source(source_id);
    app.equipment_command(EquipmentEditorCommand::Select(target.clone()));
    change(&mut app, &target, |d| {
        let EquipmentDraft::Source(d) = d else {
            panic!()
        };
        d.title = "Revised live source; not the adoption copy".into();
        d.review = rf_model::engineering::SourceReview::Reviewed;
    });
    apply(&mut app, &target);
    assert_eq!(task(&app, &key).geometry.adoption, evidence);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&key),
        EquipmentResultStatus::Current
    );
    app.equipment_command(EquipmentEditorCommand::Calculate(source.clone()));
    app.begin_geometry_adoption(&source);
    let form = app.equipment.adoption.as_mut().unwrap();
    form.new_target = false;
    form.existing = Some(key.task_id.clone());
    app.preview_geometry_adoption_form();
    app.confirm_geometry_adoption_form();
    assert_ne!(task(&app, &key).geometry.adoption, evidence);
    app.equipment_command(EquipmentEditorCommand::History { redo: false });
    assert_eq!(task(&app, &key).geometry.adoption, evidence);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&key),
        EquipmentResultStatus::Stale
    );
}

#[test]
fn adoption_save_consent_cancel_failure_actual_target_and_reopen_missing() {
    let (mut app, source) = current_case();
    let path = test_preferences_path("adoption-save-shell").with_extension("rfproj.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    app.request_save_project_as(path.clone(), false);
    if app.pending_format_upgrade.is_some() {
        app.confirm_format_upgrade();
    }
    let original = fs::read(&path).unwrap();
    let key = adopt(&mut app, &source, true);
    app.equipment_command(EquipmentEditorCommand::CalculateRating(key.clone()));
    app.save_project();
    assert!(app.pending_format_upgrade.is_some());
    app.cancel_pending_save_as_overwrite();
    assert_eq!(fs::read(&path).unwrap(), original);
    let bad = path.with_file_name("directory-target");
    fs::create_dir(&bad).unwrap();
    app.request_save_project_as(bad.clone(), false);
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .is_dirty()
    );
    app.cancel_pending_save_as_overwrite();
    app.save_project();
    app.confirm_format_upgrade();
    let evidence = task(&app, &key).geometry.adoption.clone();
    let other = path.with_file_name("old-v3.rfproj.json");
    fs::write(&other, &original).unwrap();
    app.request_save_project_as(other.clone(), false);
    assert!(app.pending_format_upgrade.is_some());
    app.cancel_pending_save_as_overwrite();
    assert_eq!(fs::read(&other).unwrap(), original);
    app.open_project(path.clone(), "C1 reopen");
    assert_eq!(task(&app, &key).geometry.adoption, evidence);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .case_result_status(&key),
        EquipmentResultStatus::Missing
    );
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

fn current_case() -> (ReadyAppState, EquipmentTaskId) {
    let (mut app, target, _) = complete_case();
    let EquipmentEditorTarget::Task(id) = target else {
        panic!()
    };
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .current_candidate(&id)
            .is_some()
    );
    (app, id)
}
fn adopt(app: &mut ReadyAppState, source: &EquipmentTaskId, copy: bool) -> EquipmentCaseKey {
    app.begin_geometry_adoption(source);
    let form = app.equipment.adoption.as_mut().unwrap();
    form.copy_case = copy;
    form.tag = "V-ADOPTED".into();
    app.preview_geometry_adoption_form();
    let w = &app.platform_host.modeling_inputs().workspace;
    let preview = w
        .equipment_editor
        .geometry_adoption_preview()
        .unwrap()
        .proposal();
    let t = preview
        .inputs()
        .rating_task(preview.target().task_id())
        .unwrap();
    let key = EquipmentCaseKey {
        task_id: t.id.clone(),
        case_id: t.cases[0].id.clone(),
    };
    app.confirm_geometry_adoption_form();
    assert!(app.equipment.adoption.is_none());
    key
}
fn task<'a>(app: &'a ReadyAppState, key: &EquipmentCaseKey) -> &'a SeparatorRatingTask {
    app.platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .inputs()
        .rating_task(&key.task_id)
        .unwrap()
}

#[test]
fn adoption_shell_cancel_blank_copy_exact_dimensions_and_atomic_history() {
    for copy in [false, true] {
        let (mut app, source) = current_case();
        let before = app
            .platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs()
            .clone();
        app.begin_geometry_adoption(&source);
        app.preview_geometry_adoption_form();
        app.cancel_geometry_adoption_form();
        assert_eq!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .engineering
                .inputs(),
            &before
        );
        let key = adopt(&mut app, &source, copy);
        let t = task(&app, &key);
        assert_eq!(t.cases[0].inputs.len(), if copy { 9 } else { 0 });
        let a = t.geometry.adoption.as_ref().unwrap();
        assert_eq!(
            a.diameter.original_m.to_bits(),
            a.diameter.adopted_m.to_bits()
        );
        assert_eq!(a.working_liquid_height.adopted_m, 0.6);
        assert_eq!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .engineering
                .case_result_status(&key),
            EquipmentResultStatus::Missing
        );
        if copy {
            app.equipment_command(EquipmentEditorCommand::CalculateRating(key.clone()));
            assert_eq!(
                app.platform_host
                    .modeling_inputs()
                    .workspace
                    .engineering
                    .current_rating(&key)
                    .unwrap()
                    .performance()
                    .conclusion(),
                separator_rating::ConstraintStatus::BoundaryReview
            );
        }
        app.equipment_command(EquipmentEditorCommand::History { redo: false });
        assert_eq!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .engineering
                .inputs(),
            &before
        );
        app.equipment_command(EquipmentEditorCommand::History { redo: true });
        assert!(task(&app, &key).geometry.adoption.is_some());
        assert_ne!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .engineering
                .case_result_status(&key),
            EquipmentResultStatus::Current
        );
    }
}

#[test]
fn adoption_shell_rejects_illegal_adjustments_late_execution_and_local_preview_edits() {
    let (mut app, source) = current_case();
    app.begin_geometry_adoption(&source);
    for value in ["0", "-1", "NaN", "inf", "1e-", "1e999"] {
        let mut number = QuantityDraft::empty(MeasurementUnit::Millimetre);
        number.set_text(value.into());
        app.equipment.adoption.as_mut().unwrap().diameter = Some(DimensionAdjustmentDraft {
            value: number,
            reason: "Explicit test".into(),
        });
        let before = app
            .platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs()
            .clone();
        app.preview_geometry_adoption_form();
        assert!(
            app.equipment.adoption.as_ref().unwrap().error.is_some(),
            "{value}"
        );
        assert!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .equipment_editor
                .geometry_adoption_preview()
                .is_none()
        );
        assert_eq!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .engineering
                .inputs(),
            &before
        );
    }
    app.equipment.adoption.as_mut().unwrap().diameter = Some(DimensionAdjustmentDraft {
        value: QuantityDraft::from_si(1.2, QuantityKind::Length, MeasurementUnit::Millimetre),
        reason: String::new(),
    });
    app.preview_geometry_adoption_form();
    assert!(app.equipment.adoption.as_ref().unwrap().error.is_some());
    app.equipment
        .adoption
        .as_mut()
        .unwrap()
        .diameter
        .as_mut()
        .unwrap()
        .reason = "Select 1200 mm".into();
    app.preview_geometry_adoption_form();
    let before = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .inputs()
        .clone();
    app.equipment_command(EquipmentEditorCommand::Calculate(source.clone())); // Same numerical result, new execution.
    app.confirm_geometry_adoption_form();
    assert!(app.equipment.adoption.as_ref().unwrap().error.is_some());
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs(),
        &before
    );
    assert_eq!(
        app.equipment
            .adoption
            .as_ref()
            .unwrap()
            .diameter
            .as_ref()
            .unwrap()
            .reason,
        "Select 1200 mm"
    );
    app.preview_geometry_adoption_form();
    app.equipment.adoption.as_mut().unwrap().copy_case = true; // Must be previewed again.
    app.confirm_geometry_adoption_form();
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .inputs(),
        &before
    );
    app.preview_geometry_adoption_form();
    app.confirm_geometry_adoption_form();
    let w = &app.platform_host.modeling_inputs().workspace;
    let t = w
        .engineering
        .inputs()
        .tasks
        .iter()
        .find_map(|t| t.as_rating())
        .unwrap();
    let a = t.geometry.adoption.as_ref().unwrap();
    assert_eq!(a.diameter.adopted_m, 1.2);
    assert_eq!(a.working_liquid_height.adopted_m, 0.6); // Diameter adjustment must not alter height.
}

#[test]
fn adoption_shell_matrix_replacement_removes_old_segments_preserves_rows_and_rejects_drafts() {
    let (mut app, source) = current_case();
    let key = adopt(&mut app, &source, true);
    app.equipment_command(EquipmentEditorCommand::AddRatingCase {
        task: key.task_id.clone(),
        copy_from: Some(key.case_id.clone()),
    });
    let geometry = EquipmentEditorTarget::RatingGeometry(key.task_id.clone());
    app.equipment_command(EquipmentEditorCommand::Select(geometry.clone()));
    let source_id = task(&app, &key).geometry.applicability_source_ids[0].clone();
    let axial = SeparatorAxialSection::NormalToHighLiquidLevel;
    change(&mut app, &geometry, |draft| {
        let EquipmentDraft::RatingGeometry(draft) = draft else {
            panic!()
        };
        let b = draft
            .fields
            .get_mut(&RatingGeometryField::Axial(axial))
            .unwrap();
        b.mode = BindingMode::Direct;
        b.value.set_text("0.2".into());
        b.source = Some(source_id);
    });
    app.begin_geometry_adoption(&source);
    {
        let f = app.equipment.adoption.as_mut().unwrap();
        f.new_target = false;
        f.existing = Some(key.task_id.clone());
    }
    app.preview_geometry_adoption_form();
    assert!(matches!(
        app.equipment.adoption.as_ref().unwrap().error,
        Some(EquipmentEditError::PendingDrafts)
    ));
    apply(&mut app, &geometry);
    let cases = task(&app, &key).cases.clone();
    for case in &cases {
        app.equipment_command(EquipmentEditorCommand::CalculateRating(EquipmentCaseKey {
            task_id: key.task_id.clone(),
            case_id: case.id.clone(),
        }));
    }
    app.preview_geometry_adoption_form();
    let preview = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .geometry_adoption_preview()
        .unwrap()
        .proposal();
    assert!(
        preview
            .previous_geometry()
            .unwrap()
            .axial_sections
            .contains_key(&axial)
    );
    assert!(
        !preview
            .inputs()
            .rating_task(&key.task_id)
            .unwrap()
            .geometry
            .axial_sections
            .contains_key(&axial)
    );
    app.confirm_geometry_adoption_form();
    assert_eq!(task(&app, &key).cases, cases);
    for case in cases {
        assert_eq!(
            app.platform_host
                .modeling_inputs()
                .workspace
                .engineering
                .case_result_status(&EquipmentCaseKey {
                    task_id: key.task_id.clone(),
                    case_id: case.id
                }),
            EquipmentResultStatus::Stale
        );
    }
}

fn fixture(name: &str) -> (ReadyAppState, PathBuf, EquipmentCaseKey) {
    let path = test_preferences_path(name).with_extension("rfproj.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        include_str!(
            "../../../../../crates/rf-store/tests/fixtures/project-v3-rating-adoption.json"
        ),
    )
    .unwrap();
    let mut app = ready_app_state(&StudioRuntimeConfig {
        project_path: path.clone(),
        ..synced_workspace_config()
    });
    app.enter_equipment();
    let key = EquipmentCaseKey {
        task_id: "adopted-v3".into(),
        case_id: "adopted-v3-case".into(),
    };
    app.equipment_command(EquipmentEditorCommand::Select(
        EquipmentEditorTarget::RatingGeometry(key.task_id.clone()),
    ));
    (app, path, key)
}
fn output(
    app: &mut ReadyAppState,
    key: &EquipmentCaseKey,
    matrix: bool,
    format: Format,
    path: &std::path::Path,
) {
    let capture = if matrix {
        app.capture_matrix_output(&key.task_id, format)
    } else {
        app.capture_rating_output(key, format)
    }
    .unwrap();
    app.export_equipment_to_path(capture, path.into());
}

#[test]
fn adoption_output_fixed_sources_current_difference_and_independent_csv_json_parity() {
    let (mut app, path, key) = fixture("adoption-evidence-output");
    let original = task(&app, &key).geometry.adoption.clone().unwrap();
    let source = EquipmentEditorTarget::Task("sizing-source".into());
    app.equipment_command(EquipmentEditorCommand::Delete(source));
    let confirm = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .confirmation()
        .unwrap()
        .key();
    app.equipment_command(EquipmentEditorCommand::Confirm(confirm));
    let geometry = EquipmentEditorTarget::RatingGeometry(key.task_id.clone());
    app.equipment_command(EquipmentEditorCommand::Select(geometry.clone()));
    change(&mut app, &geometry, |d| {
        let EquipmentDraft::RatingGeometry(d) = d else {
            panic!()
        };
        d.fields
            .get_mut(&RatingGeometryField::Input(
                SeparatorGeometryInput::InternalDiameter,
            ))
            .unwrap()
            .value
            .set_text("1500".into());
    });
    apply(&mut app, &geometry);
    assert_eq!(
        task(&app, &key).geometry.adoption.as_ref().unwrap(),
        &original
    );
    app.equipment_command(EquipmentEditorCommand::CalculateRating(key.clone()));
    for matrix in [false, true] {
        let jp = path.with_file_name(format!("adoption-{matrix}.json"));
        let cp = jp.with_extension("csv");
        output(&mut app, &key, matrix, Format::Json, &jp);
        output(&mut app, &key, matrix, Format::Csv, &cp);
        let data: Value = serde_json::from_str(&fs::read_to_string(&jp).unwrap()).unwrap();
        assert_eq!(data["version"], 3);
        assert_eq!(
            row(&data, "adoption", "adopted-v3", "sourceTaskId")["text"],
            "sizing-source"
        );
        assert_eq!(
            row(
                &data,
                "adoptionDimension",
                "internalDiameter",
                "currentValue"
            )["valueSi"],
            1.5
        );
        assert_eq!(
            row(&data, "adoptionDimension", "internalDiameter", "comparison")["text"],
            "changed"
        );
        assert_eq!(
            row(
                &data,
                "adoptionDimension",
                "internalDiameter",
                "adoptedValue"
            )["valueSi"],
            original.diameter.adopted_m
        );
        assert_eq!(
            row(&data, "adoptionInput", "workingDuration", "bindingKind")["text"],
            "basisRef"
        );
        assert_eq!(
            row(&data, "adoptionBasis", "basis-time", "value")["valueSi"],
            120.0
        );
        assert_eq!(
            row(&data, "adoptionSource", "source-test", "title")["text"],
            "Synthetic rating arithmetic"
        );
        let csv = csv_records(&fs::read_to_string(&cp).unwrap());
        let rows = data["rows"].as_array().unwrap();
        assert_eq!(csv.len(), rows.len() + 1);
        let fields = if matrix {
            vec![
                "caseId",
                "section",
                "recordId",
                "field",
                "status",
                "quantityId",
                "valueSi",
                "siUnitId",
                "displayValue",
                "displayUnitId",
                "displayUnitSymbol",
                "text",
            ]
        } else {
            vec![
                "section",
                "recordId",
                "field",
                "status",
                "quantityId",
                "valueSi",
                "siUnitId",
                "displayValue",
                "displayUnitId",
                "displayUnitSymbol",
                "text",
            ]
        };
        assert_eq!(csv[0], fields);
        for (r, json) in csv[1..].iter().zip(rows) {
            for (cell, field) in r.iter().zip(&fields) {
                if *field == "valueSi" && json[field].is_number() {
                    assert_eq!(cell.parse::<f64>().unwrap(), json[field].as_f64().unwrap());
                } else {
                    let text = json[field].as_str().unwrap_or("");
                    assert!(
                        cell == text || cell.strip_prefix('\'') == Some(text),
                        "{field}: {cell:?} != {text:?}"
                    );
                }
            }
        }
    }
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[test]
fn adoption_matrix_missing_stale_members_keep_evidence_and_late_output_is_rejected() {
    let (mut app, path, key) = fixture("adoption-matrix-qualification");
    app.equipment_command(EquipmentEditorCommand::AddRatingCase {
        task: key.task_id.clone(),
        copy_from: Some(key.case_id.clone()),
    });
    let out = path.with_file_name("partial.json");
    output(&mut app, &key, true, Format::Json, &out);
    let data: Value = serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(row(&data, "matrix", "", "requiredCount")["text"], "2");
    assert!(
        !data["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["section"] == "result")
    );
    assert!(
        data["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["section"] == "adoptionSource")
    );
    app.equipment_command(EquipmentEditorCommand::CalculateRating(key.clone()));
    let pending = app
        .capture_matrix_output(&key.task_id, Format::Json)
        .unwrap();
    app.equipment_command(EquipmentEditorCommand::CalculateRating(key.clone()));
    let rejected = path.with_file_name("late.json");
    app.export_equipment_to_path(pending, rejected.clone());
    assert!(!rejected.exists());
    let geometry = EquipmentEditorTarget::RatingGeometry(key.task_id.clone());
    app.equipment_command(EquipmentEditorCommand::Select(geometry.clone()));
    change(&mut app, &geometry, |d| {
        let EquipmentDraft::RatingGeometry(d) = d else {
            panic!()
        };
        d.fields
            .get_mut(&RatingGeometryField::Input(
                SeparatorGeometryInput::InternalDiameter,
            ))
            .unwrap()
            .value
            .set_text("1500".into());
    });
    assert!(
        app.capture_matrix_output(&key.task_id, Format::Json)
            .is_none()
    );
    apply(&mut app, &geometry);
    output(&mut app, &key, true, Format::Json, &out);
    app.confirm_equipment_export();
    let data: Value = serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert!(
        !data["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["section"] == "result")
    );
    assert_eq!(
        row(&data, "adoptionDimension", "internalDiameter", "comparison")["text"],
        "changed"
    );
    fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
