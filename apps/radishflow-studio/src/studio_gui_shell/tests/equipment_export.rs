use super::equipment_editor::{change, complete_case};
use super::*;
use equipment::CalculationTableFormat as Format;
use rf_model::{engineering::*, equipment::*};
use rf_types::{
    EquipmentTaskId, SourceObservationId,
    units::{DisplayUnitSet, MeasurementUnit as U, QuantityKind as Q},
};
use rf_ui::engineering::{EquipmentResultStatus, editor::*};
use serde_json::Value;
use std::path::Path;

fn task_id(target: &EquipmentEditorTarget) -> EquipmentTaskId {
    let EquipmentEditorTarget::Task(id) = target else {
        panic!("task target")
    };
    id.clone()
}
fn apply(app: &mut ReadyAppState, target: &EquipmentEditorTarget) {
    app.equipment_command(EquipmentEditorCommand::Apply(target.clone()));
    let confirmation = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .confirmation()
        .cloned();
    if let Some(confirmation) = confirmation {
        app.equipment_command(EquipmentEditorCommand::Confirm(confirmation.key()));
    }
    assert!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .equipment_editor
            .error()
            .is_none()
    );
}
fn calculate(app: &mut ReadyAppState, target: &EquipmentEditorTarget) {
    app.equipment_command(EquipmentEditorCommand::Calculate(task_id(target)));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .result_status(&task_id(target)),
        EquipmentResultStatus::Current
    );
}
fn path(name: &str, extension: &str) -> PathBuf {
    let path = test_preferences_path(name).with_extension(extension);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    path
}
fn export(app: &mut ReadyAppState, target: &EquipmentEditorTarget, format: Format, path: &Path) {
    let output = app
        .capture_equipment_output(&task_id(target), format)
        .unwrap();
    app.export_equipment_to_path(output, path.into());
}
fn row<'a>(table: &'a Value, section: &str, id: &str, field: &str) -> &'a Value {
    table["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["section"] == section && r["recordId"] == id && r["field"] == field)
        .unwrap_or_else(|| panic!("missing {section}/{id}/{field}"))
}
fn read(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

// Independent CSV reader used only to verify multiline, Unicode and quote escapes.
fn csv_records(text: &str) -> Vec<Vec<String>> {
    let mut records = Vec::new();
    let mut record = Vec::new();
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                chars.next();
                cell.push('"');
            }
            '"' => quoted = !quoted,
            ',' if !quoted => record.push(std::mem::take(&mut cell)),
            '\r' if !quoted => {
                assert_eq!(chars.next(), Some('\n'));
                record.push(std::mem::take(&mut cell));
                records.push(std::mem::take(&mut record));
            }
            c => cell.push(c),
        }
    }
    assert!(!quoted && cell.is_empty() && record.is_empty());
    records
}
#[test]
fn equipment_export_formats_share_complete_rows_units_precision_and_unknowns() {
    let (mut app, task, source) = complete_case();
    app.equipment_command(EquipmentEditorCommand::Select(source.clone()));
    change(&mut app, &source, |draft| {
        let EquipmentDraft::Source(s) = draft else {
            panic!()
        };
        s.title = "=SUM(1,2)\n\"来源\"".into();
        s.reference = "Public synthetic fixture".into();
        s.locator = "p. 1, lines 2–3".into();
        s.publication_version = "fixture-v1".into();
        s.accessed_on = "2026-10-20".into();
    });
    apply(&mut app, &source);
    calculate(&mut app, &task);
    let mut units = DisplayUnitSet::engineering();
    units
        .set_unit(Q::ActualVolumeFlow, U::CubicMetrePerHour)
        .unwrap();
    units.set_unit(Q::Duration, U::Minute).unwrap();
    units.set_unit(Q::Length, U::Millimetre).unwrap();
    app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(units))
        .unwrap();
    let before = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .clone();
    let json = path("equipment-data", "json");
    let csv = json.with_extension("csv");
    export(&mut app, &task, Format::Json, &json);
    export(&mut app, &task, Format::Csv, &csv);
    assert_eq!(
        app.platform_host.modeling_inputs().workspace.engineering,
        before
    );
    assert!(before.is_dirty());
    let table = read(&json);
    assert_eq!(table["format"], "radishflow.separator-calculation-table");
    assert_eq!(table["version"], 1);
    for (field, si, value, unit) in [
        ("operatingTemperature", 300.0, "26.85", "celsius"),
        ("operatingPressure", 100000.0, "1", "bar"),
        ("gasActualVolumeFlow", 0.8, "2880", "cubic_metre_per_hour"),
        ("workingDuration", 120.0, "2", "minute"),
        ("maximumWorkingLiquidHeight", 1.0, "1000", "millimetre"),
    ] {
        let r = row(&table, "input", field, "effectiveValue");
        assert_eq!(r["valueSi"], si);
        assert_eq!(r["displayValue"], value);
        assert_eq!(r["displayUnitId"], unit);
    }
    let id = task_id(&task);
    let diameter = row(&table, "result", id.as_str(), "minimumDiameter");
    assert!(
        (diameter["valueSi"].as_f64().unwrap() - (4.0 / std::f64::consts::PI).sqrt()).abs() < 1e-14
    );
    assert_eq!(diameter["displayValue"], "1128.38");
    for field in ["tangentToTangentHeight", "bufferVolume"] {
        let r = row(&table, "result", id.as_str(), field);
        assert_eq!(r["status"], "unknown");
        assert!(r["valueSi"].is_null());
        assert!(r["displayValue"].is_null());
    }
    let rows = table["rows"].as_array().unwrap();
    assert_eq!(
        rows.iter().filter(|r| r["section"] == "unchecked").count(),
        7
    );
    let records = csv_records(&fs::read_to_string(&csv).unwrap());
    assert_eq!(records.len(), rows.len() + 1);
    for (csv, json) in records[1..].iter().zip(rows) {
        assert_eq!(csv.len(), 11);
        for (i, field) in records[0].iter().enumerate() {
            let value = &json[field];
            let mut expected = if value.is_null() {
                String::new()
            } else if let Some(s) = value.as_str() {
                s.into()
            } else {
                value.as_f64().unwrap().to_string()
            };
            if field == "text" && expected.starts_with('=') {
                expected.insert(0, '\'');
            }
            assert_eq!(csv[i], expected, "{json:?} {field}");
        }
    }
    fs::remove_dir_all(json.parent().unwrap()).unwrap();
}

#[test]
fn equipment_export_keeps_units_and_labels_captured_before_overwrite() {
    let (mut app, task, _) = complete_case();
    calculate(&mut app, &task);
    app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(
        DisplayUnitSet::engineering(),
    ))
    .unwrap();
    let file = path("equipment-units", "json");
    fs::write(&file, "original").unwrap();
    export(&mut app, &task, Format::Json, &file);
    assert_eq!(
        app.focus_context(&egui::Context::default()),
        StudioGuiFocusContext::ModalDialog
    );
    app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(
        DisplayUnitSet::si(),
    ))
    .unwrap();
    change(&mut app, &task, |draft| {
        let EquipmentDraft::Task(t) = draft else {
            panic!()
        };
        t.name = "New label".into();
    });
    apply(&mut app, &task);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .result_status(&task_id(&task)),
        EquipmentResultStatus::Current
    );
    app.confirm_equipment_export();
    let table = read(&file);
    assert_eq!(
        row(&table, "input", "operatingTemperature", "effectiveValue")["displayValue"],
        "26.85"
    );
    assert_eq!(
        row(&table, "task", task_id(&task).as_str(), "name")["text"],
        "Synthetic separator"
    );
    export(&mut app, &task, Format::Json, &file);
    app.confirm_equipment_export();
    let table = read(&file);
    assert_eq!(
        row(&table, "task", task_id(&task).as_str(), "name")["text"],
        "New label"
    );
    assert_eq!(
        row(&table, "task", task_id(&task).as_str(), "capturedName")["text"],
        "Synthetic separator"
    );
    fs::remove_dir_all(file.parent().unwrap()).unwrap();
}

#[test]
fn equipment_export_rejects_missing_failed_stale_and_related_drafts_without_opening_picker() {
    let (mut app, task, source) = complete_case();
    let id = task_id(&task);
    assert!(app.capture_equipment_output(&id, Format::Json).is_none());
    calculate(&mut app, &task);
    change(&mut app, &task, |d| {
        let EquipmentDraft::Task(t) = d else { panic!() };
        t.fields
            .get_mut(&EquipmentField::Input(SeparatorInput::GasVolumeFlow))
            .unwrap()
            .value
            .set_text("1e-".into());
    });
    assert!(app.capture_equipment_output(&id, Format::Json).is_none());
    app.equipment_command(EquipmentEditorCommand::Discard(task.clone()));
    app.equipment_command(EquipmentEditorCommand::Select(source.clone()));
    change(&mut app, &source, |d| {
        let EquipmentDraft::Source(s) = d else {
            panic!()
        };
        s.adoption_note.push_str(" changed");
    });
    assert!(app.capture_equipment_output(&id, Format::Csv).is_none());
    apply(&mut app, &source);
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .result_status(&id),
        EquipmentResultStatus::Stale
    );
    assert!(app.capture_equipment_output(&id, Format::Json).is_none());
    app.equipment_command(EquipmentEditorCommand::Select(task.clone()));
    change(&mut app, &task, |d| {
        let EquipmentDraft::Task(t) = d else { panic!() };
        t.fields
            .get_mut(&EquipmentField::Input(SeparatorInput::GasDensity))
            .unwrap()
            .value
            .set_text("2000".into());
    });
    apply(&mut app, &task);
    app.equipment_command(EquipmentEditorCommand::Calculate(id.clone()));
    assert_eq!(
        app.platform_host
            .modeling_inputs()
            .workspace
            .engineering
            .result_status(&id),
        EquipmentResultStatus::Failed
    );
    assert!(app.capture_equipment_output(&id, Format::Json).is_none());
}

#[test]
fn equipment_export_rechecks_input_evidence_recalculation_history_and_drafts_before_write() {
    for mutation in ["input", "evidence", "rerun", "undoRedo", "draft"] {
        let (mut app, task, source) = complete_case();
        calculate(&mut app, &task);
        let file = path(mutation, "json");
        fs::write(&file, "original").unwrap();
        export(&mut app, &task, Format::Json, &file);
        match mutation {
            "rerun" => calculate(&mut app, &task),
            "evidence" => {
                app.equipment_command(EquipmentEditorCommand::Select(source.clone()));
                change(&mut app, &source, |d| {
                    let EquipmentDraft::Source(s) = d else {
                        panic!()
                    };
                    s.adoption_note.push_str(" revised");
                });
                apply(&mut app, &source);
            }
            "input" | "undoRedo" | "draft" => {
                change(&mut app, &task, |d| {
                    let EquipmentDraft::Task(t) = d else { panic!() };
                    t.fields
                        .get_mut(&EquipmentField::Input(SeparatorInput::WorkingVolumeTime))
                        .unwrap()
                        .value
                        .set_text("180".into());
                });
                if mutation != "draft" {
                    apply(&mut app, &task);
                }
                if mutation == "undoRedo" {
                    app.equipment_command(EquipmentEditorCommand::History { redo: false });
                    app.equipment_command(EquipmentEditorCommand::History { redo: true });
                }
            }
            _ => unreachable!(),
        }
        app.confirm_equipment_export();
        assert_eq!(fs::read_to_string(&file).unwrap(), "original", "{mutation}");
        assert_eq!(
            app.project_open.notice.as_ref().unwrap().level,
            ProjectOpenNoticeLevel::Warning
        );
        fs::remove_dir_all(file.parent().unwrap()).unwrap();
    }
}

#[test]
fn equipment_export_reopen_same_ids_revokes_captured_output_and_pending_confirmation() {
    let (mut app, task, _) = complete_case();
    calculate(&mut app, &task);
    let project = path("equipment-reopen", "rfproj.json");
    app.request_save_project_as(project.clone(), false);
    if app.pending_format_upgrade.is_some() {
        app.confirm_format_upgrade();
    }
    let output = app
        .capture_equipment_output(&task_id(&task), Format::Json)
        .unwrap();
    let file = project.with_file_name("calculation.json");
    fs::write(&file, "original").unwrap();
    export(&mut app, &task, Format::Json, &file);
    app.open_project(project.clone(), "test reopen");
    assert!(app.equipment.pending_export.is_none());
    calculate(&mut app, &task);
    app.export_equipment_to_path(output, file.clone());
    app.confirm_equipment_export();
    assert_eq!(fs::read_to_string(&file).unwrap(), "original");
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

struct EquipmentPicker {
    path: Option<PathBuf>,
    supported: bool,
}
impl ProjectFilePicker for EquipmentPicker {
    fn supports_equipment_export_dialogs(&self) -> bool {
        self.supported
    }
    fn pick_equipment_export_file(&mut self, _: Format) -> Option<PathBuf> {
        assert!(self.supported);
        self.path.take()
    }
    fn pick_project_file(&mut self) -> Option<PathBuf> {
        unreachable!()
    }
    fn pick_save_project_file(&mut self) -> Option<PathBuf> {
        unreachable!()
    }
    fn pick_result_export_file(&mut self) -> Option<PathBuf> {
        unreachable!()
    }
}
#[test]
fn equipment_export_cancel_unsupported_io_failure_and_success_preserve_inputs_and_history() {
    let (mut app, task, _) = complete_case();
    calculate(&mut app, &task);
    let before = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .clone();
    app.project_file_picker = Box::new(EquipmentPicker {
        path: None,
        supported: true,
    });
    app.export_equipment_from_picker(&task_id(&task), Format::Csv);
    let canceled = app.project_open.notice.as_ref().unwrap().clone();
    assert_eq!(canceled.level, ProjectOpenNoticeLevel::Info);
    app.project_file_picker = Box::new(EquipmentPicker {
        path: None,
        supported: false,
    });
    app.export_equipment_from_picker(&task_id(&task), Format::Csv);
    assert_eq!(
        app.project_open.notice.as_ref().unwrap().level,
        ProjectOpenNoticeLevel::Warning
    );
    let file = path("equipment-io", "csv");
    fs::create_dir(&file).unwrap();
    fs::write(file.join("keep"), "preserve").unwrap();
    export(&mut app, &task, Format::Csv, &file);
    assert_eq!(
        app.project_open.notice.as_ref().unwrap().level,
        ProjectOpenNoticeLevel::Error
    );
    assert_eq!(fs::read_to_string(file.join("keep")).unwrap(), "preserve");
    let file = file.with_file_name("输出 空格.CSV");
    fs::write(&file, "original").unwrap();
    app.project_file_picker = Box::new(EquipmentPicker {
        path: Some(file.clone()),
        supported: true,
    });
    app.export_equipment_from_picker(&task_id(&task), Format::Csv);
    assert!(app.equipment.pending_export.is_some());
    app.cancel_equipment_export();
    assert_eq!(fs::read_to_string(&file).unwrap(), "original");
    export(&mut app, &task, Format::Csv, &file);
    app.confirm_equipment_export();
    assert!(fs::read_to_string(&file).unwrap().starts_with("section,"));
    assert_eq!(
        app.platform_host.modeling_inputs().workspace.engineering,
        before
    );
    fs::remove_dir_all(file.parent().unwrap()).unwrap();
}

#[test]
fn equipment_export_retains_basis_override_observation_dependencies_and_partial_height() {
    let (mut app, task, source) = complete_case();
    let EquipmentEditorTarget::Source(source_id) = &source else {
        panic!()
    };
    app.equipment_command(EquipmentEditorCommand::Select(source.clone()));
    change(&mut app, &source, |d| {
        let EquipmentDraft::Source(s) = d else {
            panic!()
        };
        s.observations = vec![
            ObservationDraft {
                id: SourceObservationId::new("time-original"),
                quantity: Q::Duration,
                original_value: "2".into(),
                original_unit: "min".into(),
                canonical: QuantityDraft::from_si(120.0, Q::Duration, U::Second),
                conversion_note: "2 min × 60 s/min".into(),
                depends_on: vec![],
            },
            ObservationDraft {
                id: SourceObservationId::new("time-copy"),
                quantity: Q::Duration,
                original_value: "120".into(),
                original_unit: "s".into(),
                canonical: QuantityDraft::from_si(120.0, Q::Duration, U::Second),
                conversion_note: "Copied observation".into(),
                depends_on: vec![SourceObservationRef {
                    source_id: source_id.clone(),
                    observation_id: SourceObservationId::new("time-original"),
                }],
            },
        ];
    });
    apply(&mut app, &source);
    app.equipment_command(EquipmentEditorCommand::NewBasis);
    let basis = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .active()
        .unwrap()
        .clone();
    let EquipmentEditorTarget::Basis(basis_id) = &basis else {
        panic!()
    };
    change(&mut app, &basis, |d| {
        let EquipmentDraft::Basis(b) = d else {
            panic!()
        };
        b.name = "Shared duration".into();
        b.value.set_text("120".into());
        b.source = Some(source_id.clone());
    });
    apply(&mut app, &basis);
    app.equipment_command(EquipmentEditorCommand::Select(task.clone()));
    change(&mut app, &task, |d| {
        let EquipmentDraft::Task(t) = d else { panic!() };
        let b = t
            .fields
            .get_mut(&EquipmentField::Input(SeparatorInput::WorkingVolumeTime))
            .unwrap();
        b.mode = BindingMode::Override;
        b.basis = Some(basis_id.clone());
        b.value.set_text("180".into());
        b.reason = "Explicit longer working duration".into();
        let b = t
            .fields
            .get_mut(&EquipmentField::Axial(
                SeparatorAxialSection::NormalToHighLiquidLevel,
            ))
            .unwrap();
        b.mode = BindingMode::Direct;
        b.value.set_text("0.5".into());
        b.source = Some(source_id.clone());
    });
    apply(&mut app, &task);
    calculate(&mut app, &task);
    let file = path("equipment-provenance", "json");
    export(&mut app, &task, Format::Json, &file);
    let table = read(&file);
    assert_eq!(
        row(&table, "input", "workingDuration", "bindingKind")["text"],
        "override"
    );
    assert_eq!(
        row(&table, "input", "workingDuration", "effectiveValue")["valueSi"],
        180.0
    );
    assert_eq!(
        row(&table, "input", "workingDuration", "overrideReason")["text"],
        "Explicit longer working duration"
    );
    assert_eq!(
        row(&table, "basis", basis_id.as_str(), "value")["valueSi"],
        120.0
    );
    assert_eq!(
        row(
            &table,
            "source",
            source_id.as_str(),
            "observation.0.originalUnit"
        )["text"],
        "min"
    );
    assert_eq!(
        row(
            &table,
            "source",
            source_id.as_str(),
            "observation.1.dependency.0.observationId"
        )["text"],
        "time-original"
    );
    assert_eq!(
        row(&table, "result", task_id(&task).as_str(), "bufferVolume")["valueSi"],
        0.5
    );
    assert_eq!(
        row(
            &table,
            "result",
            task_id(&task).as_str(),
            "tangentToTangentHeight"
        )["status"],
        "unknown"
    );
    change(&mut app, &task, |d| {
        let EquipmentDraft::Task(t) = d else { panic!() };
        t.fields
            .get_mut(&EquipmentField::Input(SeparatorInput::WorkingVolumeTime))
            .unwrap()
            .mode = BindingMode::BasisRef;
    });
    apply(&mut app, &task);
    calculate(&mut app, &task);
    export(&mut app, &task, Format::Json, &file);
    app.confirm_equipment_export();
    let table = read(&file);
    assert_eq!(
        row(&table, "input", "workingDuration", "bindingKind")["text"],
        "basisRef"
    );
    assert_eq!(
        row(&table, "input", "workingDuration", "effectiveValue")["valueSi"],
        120.0
    );
    fs::remove_dir_all(file.parent().unwrap()).unwrap();
}

#[test]
fn equipment_export_conversion_failure_precedes_picker_and_preserves_result() {
    let (mut app, task, source) = complete_case();
    app.equipment_command(EquipmentEditorCommand::Select(source.clone()));
    change(&mut app, &source, |d| {
        let EquipmentDraft::Source(s) = d else {
            panic!()
        };
        s.observations.push(ObservationDraft {
            id: SourceObservationId::new("large-finite-flow"),
            quantity: Q::ActualVolumeFlow,
            original_value: "1e308".into(),
            original_unit: "m3/s".into(),
            canonical: QuantityDraft::from_si(1e308, Q::ActualVolumeFlow, U::CubicMetrePerSecond),
            conversion_note: "Finite arithmetic fixture for export overflow".into(),
            depends_on: vec![],
        });
    });
    apply(&mut app, &source);
    calculate(&mut app, &task);
    let mut units = DisplayUnitSet::si();
    units
        .set_unit(Q::ActualVolumeFlow, U::CubicMetrePerHour)
        .unwrap();
    app.apply_presentation_command(rf_ui::ProjectPresentationCommand::Apply(units))
        .unwrap();
    struct NeverOpen;
    impl ProjectFilePicker for NeverOpen {
        fn supports_equipment_export_dialogs(&self) -> bool {
            true
        }
        fn pick_equipment_export_file(&mut self, _: Format) -> Option<PathBuf> {
            panic!("conversion must finish before picker")
        }
        fn pick_project_file(&mut self) -> Option<PathBuf> {
            unreachable!()
        }
        fn pick_save_project_file(&mut self) -> Option<PathBuf> {
            unreachable!()
        }
        fn pick_result_export_file(&mut self) -> Option<PathBuf> {
            unreachable!()
        }
    }
    app.project_file_picker = Box::new(NeverOpen);
    let before = app
        .platform_host
        .modeling_inputs()
        .workspace
        .engineering
        .clone();
    for format in [Format::Csv, Format::Json] {
        app.export_equipment_from_picker(&task_id(&task), format);
        assert_eq!(
            app.project_open.notice.as_ref().unwrap().level,
            ProjectOpenNoticeLevel::Error
        );
        assert!(app.equipment.pending_export.is_none());
        assert_eq!(
            app.platform_host.modeling_inputs().workspace.engineering,
            before
        );
    }
}

#[test]
fn equipment_export_unrelated_drafts_do_not_block_current_task() {
    let (mut app, task, _) = complete_case();
    calculate(&mut app, &task);
    app.equipment_command(EquipmentEditorCommand::CreateSource);
    let unrelated = app
        .platform_host
        .modeling_inputs()
        .workspace
        .equipment_editor
        .active()
        .unwrap()
        .clone();
    change(&mut app, &unrelated, |d| {
        let EquipmentDraft::Source(s) = d else {
            panic!()
        };
        s.title = "unrelated draft".into();
    });
    assert!(
        app.capture_equipment_output(&task_id(&task), Format::Csv)
            .is_some()
    );
}

#[test]
fn equipment_export_normalizes_extension_and_preserves_project_files() {
    let (mut app, task, _) = complete_case();
    calculate(&mut app, &task);
    let project = path("equipment-suffix", "rfproj.json");
    fs::write(&project, "project bytes").unwrap();
    export(&mut app, &task, Format::Json, &project);
    assert_eq!(fs::read_to_string(&project).unwrap(), "project bytes");
    assert!(PathBuf::from(format!("{}.calculation.json", project.display())).is_file());
    let no_extension = project.with_file_name("table");
    let csv = no_extension.with_extension("csv");
    fs::write(&csv, "original").unwrap();
    export(&mut app, &task, Format::Csv, &no_extension);
    assert!(app.equipment.pending_export.is_some());
    app.cancel_equipment_export();
    assert_eq!(fs::read_to_string(&csv).unwrap(), "original");
    fs::remove_dir_all(project.parent().unwrap()).unwrap();
}

#[test]
fn equipment_export_full_frame_renders_confirmation_safe_focus_and_notice() {
    for locale in [StudioShellLocale::ZhCn, StudioShellLocale::En] {
        for key in [egui::Key::Enter, egui::Key::Escape] {
            let (mut app, task, _) = complete_case();
            app.locale = locale;
            calculate(&mut app, &task);
            let file = path("equipment-modal", "json");
            fs::write(&file, "original").unwrap();
            export(&mut app, &task, Format::Json, &file);
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            fonts::configure_studio_fonts(&ctx);
            let mut labels = String::new();
            for event in [None, None, Some(key), None] {
                let out = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(400.0, 900.0),
                        )),
                        focused: true,
                        events: event
                            .map(|key| egui::Event::Key {
                                key,
                                physical_key: Some(key),
                                pressed: true,
                                repeat: false,
                                modifiers: egui::Modifiers::NONE,
                            })
                            .into_iter()
                            .collect(),
                        ..Default::default()
                    },
                    |ctx| app.update(ctx),
                );
                labels.push_str(
                    &out.platform_output
                        .accesskit_update
                        .unwrap()
                        .nodes
                        .iter()
                        .flat_map(|(_, n)| [n.label(), n.value()].into_iter().flatten())
                        .collect::<Vec<_>>()
                        .join("\n"),
                );
            }
            assert!(
                labels.contains(if locale == StudioShellLocale::En {
                    "Replace equipment calculation table?"
                } else {
                    "覆盖设备计算表？"
                }),
                "{labels}"
            );
            assert!(
                app.equipment.pending_export.is_none(),
                "safe initial focus / Escape must cancel"
            );
            assert!(
                labels.contains(if locale == StudioShellLocale::En {
                    "Calculation table export canceled"
                } else {
                    "已取消计算表导出"
                }),
                "{labels}"
            );
            assert_eq!(fs::read_to_string(&file).unwrap(), "original");
            fs::remove_dir_all(file.parent().unwrap()).unwrap();
        }
    }
}
