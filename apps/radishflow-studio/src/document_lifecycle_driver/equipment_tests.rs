use super::*;
use rf_model::{engineering::*, equipment::*};
use rf_types::units::*;
use rf_ui::engineering::{EngineeringCommand as C, EquipmentResultStatus};
fn method() -> EquipmentMethod {
    EquipmentMethod {
        id: "radishflow.separator.vertical-sb-working-volume".into(),
        version: 1,
    }
}
use std::{collections::BTreeMap, fs};

// Public synthetic arithmetic fixture S0-B01. No proprietary engineering data.
fn fixture() -> (SeparatorSizingTask, EngineeringBasis) {
    let source = SourceRecord {
        id: "synthetic".into(),
        revision: 1,
        kind: SourceKind::Assumed,
        title: "S0-B01 synthetic arithmetic input".into(),
        reference: String::new(),
        locator: String::new(),
        publication_version: String::new(),
        accessed_on: None,
        adoption_note: "Explicit synthetic test assumption; no physical accuracy claim".into(),
        review: SourceReview::Adopted,
        applicability: SourceApplicability {
            fluid_system: "synthetic two-phase fluid".into(),
            phase: SourcePhase::Both,
            operating_conditions: Some(SourceOperatingConditions {
                temperature_k: 300.0,
                pressure_pa: 100000.0,
            }),
            scope: "Full cylinder, effective whole-vessel K, explicit working-volume criteria"
                .into(),
        },
        observations: Vec::new(),
    };
    let values = [
        300.0, 100000.0, 0.8, 0.005, 10.0, 1010.0, 0.1, 0.8, 120.0, 1.0,
    ];
    let task = SeparatorSizingTask {
        id: "task-1".into(),
        revision: 1,
        tag: "V-101".into(),
        name: "Synthetic sizing".into(),
        method: method(),
        case: SeparatorCase {
            id: "case-1".into(),
            name: "Single case".into(),
            purpose: "Arithmetic verification".into(),
            fluid_system: source.applicability.fluid_system.clone(),
            applicability_source_ids: vec![source.id.clone()],
            inputs: SeparatorInput::ALL
                .into_iter()
                .zip(values)
                .map(|(field, value)| (field, direct(field.quantity(), value)))
                .collect(),
            applicability: SeparatorApplicability {
                orientation: Some(SeparatorOrientation::Vertical),
                gas_direction: Some(SeparatorGasDirection::Upward),
                service: Some(SeparatorService::CleanNonFoamingTwoPhase),
                geometry: Some(SeparatorSectionGeometry::FullEqualDiameterCylinder),
                k_basis: Some(SeparatorKBasis::EffectiveWholeVesselForDeclaredInternals),
                flow_basis: Some(SeparatorFlowBasis::ActualAtOperatingConditions),
                internals_description:
                    "No reduction of effective full-cylinder area; test assumption".into(),
            },
        },
        axial_sections: BTreeMap::new(),
    };
    (
        task,
        EngineeringBasis {
            sources: vec![source],
            basis_entries: Vec::new(),
        },
    )
}

fn direct(quantity: QuantityKind, value_si: f64) -> InputBinding {
    InputBinding::Direct {
        value: EngineeringValue { quantity, value_si },
        source_id: "synthetic".into(),
    }
}
fn set(task: &mut SeparatorSizingTask, field: SeparatorInput, value: f64) {
    task.case
        .inputs
        .insert(field, direct(field.quantity(), value));
}

struct Files {
    root: PathBuf,
}
impl Files {
    fn new() -> Self {
        let root = super::tests::temp_project_path("v3-lifecycle").with_extension("dir");
        fs::create_dir(&root).unwrap();
        Self { root }
    }
    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
    fn app(&self) -> AppState {
        super::tests::app_state_from_example(&self.path("project.rfproj.json"))
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn add_equipment(app: &mut AppState) {
    let (task, basis) = fixture();
    app.workspace
        .engineering
        .apply(vec![
            C::CreateSource(Box::new(basis.sources[0].clone())),
            C::CreateTask(Box::new(task)),
        ])
        .unwrap();
}
fn task_id() -> rf_types::EquipmentTaskId {
    "task-1".into()
}
fn update_flow(app: &mut AppState, flow: f64) {
    let mut task = app
        .workspace
        .engineering
        .inputs()
        .task(&task_id())
        .unwrap()
        .clone();
    set(&mut task, SeparatorInput::GasVolumeFlow, flow);
    app.workspace
        .engineering
        .apply(vec![C::UpdateTask(Box::new(task))])
        .unwrap();
}
fn save(app: &mut AppState, path: &Path) {
    dispatch_document_lifecycle(
        app,
        StudioDocumentLifecycleCommand::SaveUpgraded {
            path: path.to_path_buf(),
        },
    )
    .unwrap();
}

#[test]
fn equipment_upgrade_round_trip_preserves_inputs_and_reopens_without_results_or_history() {
    let files = Files::new();
    let mut app = files.app();
    add_equipment(&mut app);
    app.workspace.engineering.calculate(&task_id()).unwrap();
    let old_request = app
        .workspace
        .engineering
        .begin_calculation(&task_id())
        .unwrap();
    let doc = app.workspace.document.clone();
    let flow = app.workspace.solve_session.clone();
    let original = fs::read(files.path("project.rfproj.json")).unwrap();
    for command in [
        StudioDocumentLifecycleCommand::Save,
        StudioDocumentLifecycleCommand::SaveAs {
            path: files.path("copy.rfproj.json"),
        },
    ] {
        let before = app.clone();
        assert!(
            dispatch_document_lifecycle(&mut app, command)
                .unwrap_err()
                .message()
                .contains("confirmation")
        );
        assert_eq!(app, before);
    }
    assert_eq!(
        fs::read(files.path("project.rfproj.json")).unwrap(),
        original
    );
    save(&mut app, &files.path("copy.rfproj.json"));
    assert_eq!(app.workspace.document, doc);
    assert_eq!(app.workspace.solve_session, flow);
    assert_eq!(
        app.workspace.project_presentation.source_file_version(),
        Some(3)
    );
    assert!(!app.workspace.project_save_state().has_unsaved_changes());
    assert_eq!(
        app.workspace.engineering.result_status(&task_id()),
        EquipmentResultStatus::Current
    );
    assert!(app.workspace.engineering.can_undo());
    let text = fs::read_to_string(files.path("copy.rfproj.json")).unwrap();
    for forbidden in [
        "checkpoint",
        "generation",
        "snapshot",
        "baseline",
        "candidate",
    ] {
        assert!(!text.contains(&format!("\"{forbidden}\"")));
    }
    let mut reopened = load_project_app_state(&files.path("copy.rfproj.json")).unwrap();
    assert_eq!(
        reopened.workspace.engineering.inputs(),
        app.workspace.engineering.inputs()
    );
    assert!(!reopened.workspace.engineering.can_undo());
    assert!(
        !reopened
            .workspace
            .project_save_state()
            .has_unsaved_changes()
    );
    assert_eq!(
        reopened.workspace.engineering.result_status(&task_id()),
        EquipmentResultStatus::Missing
    );
    assert!(
        reopened
            .workspace
            .engineering
            .complete_calculation(old_request.execute())
            .is_err()
    );
    assert_eq!(
        reopened
            .workspace
            .engineering
            .calculate(&task_id())
            .unwrap(),
        EquipmentResultStatus::Current
    );
    assert_eq!(
        reopened
            .workspace
            .engineering
            .current_candidate(&task_id())
            .unwrap()
            .dimensions(),
        app.workspace
            .engineering
            .current_candidate(&task_id())
            .unwrap()
            .dimensions()
    );
    assert!(rf_store::read_project_file(files.path("copy.rfproj.json")).is_err());
}

#[test]
fn saving_preserves_history_and_undo_redo_compare_written_baseline_without_reviving_candidates() {
    let files = Files::new();
    let mut app = files.app();
    add_equipment(&mut app);
    save(&mut app, &files.path("project.rfproj.json"));
    update_flow(&mut app, 1.2);
    app.workspace.engineering.calculate(&task_id()).unwrap();
    dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).unwrap();
    assert!(!app.workspace.engineering.is_dirty());
    app.workspace.engineering.undo().unwrap();
    assert!(app.workspace.engineering.is_dirty());
    app.workspace.engineering.redo().unwrap();
    assert!(!app.workspace.engineering.is_dirty());
    assert_eq!(
        app.workspace.engineering.result_status(&task_id()),
        EquipmentResultStatus::Stale
    );
}

#[test]
fn failed_v3_save_keeps_current_path_inputs_history_baseline_and_old_file() {
    let files = Files::new();
    let mut app = files.app();
    add_equipment(&mut app);
    save(&mut app, &files.path("project.rfproj.json"));
    update_flow(&mut app, 1.4);
    let before = app.clone();
    let original = fs::read(files.path("project.rfproj.json")).unwrap();
    let directory = files.path("directory");
    fs::create_dir(&directory).unwrap();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs { path: directory }
        )
        .is_err()
    );
    assert_eq!(app, before);
    assert_eq!(
        fs::read(files.path("project.rfproj.json")).unwrap(),
        original
    );
    assert!(
        !fs::read_dir(&files.root).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("staging"))
    );
}

#[test]
fn missing_inputs_out_of_scope_declarations_and_shared_overrides_survive_real_files() {
    let files = Files::new();
    let mut app = files.app();
    add_equipment(&mut app);
    let mut a = app
        .workspace
        .engineering
        .inputs()
        .task(&task_id())
        .unwrap()
        .clone();
    a.case.inputs.insert(
        SeparatorInput::WorkingVolumeTime,
        InputBinding::BasisRef {
            basis_entry_id: "time".into(),
        },
    );
    let mut b = a.clone();
    b.id = "task-2".into();
    b.case.id = "case-2".into();
    b.tag = "V-102".into();
    b.case.inputs.remove(&SeparatorInput::GasDensity);
    b.case.applicability.orientation = Some(SeparatorOrientation::Horizontal);
    b.case.inputs.insert(
        SeparatorInput::WorkingVolumeTime,
        InputBinding::Override {
            basis_entry_id: "time".into(),
            value: EngineeringValue {
                quantity: QuantityKind::Duration,
                value_si: 180.0,
            },
            source_id: "synthetic".into(),
            reason: "Local condition".into(),
        },
    );
    app.workspace
        .engineering
        .apply(vec![
            C::CreateBasis(BasisEntry {
                id: "time".into(),
                revision: 0,
                name: "Time".into(),
                value: EngineeringValue {
                    quantity: QuantityKind::Duration,
                    value_si: 120.0,
                },
                source_id: "synthetic".into(),
            }),
            C::UpdateTask(Box::new(a)),
            C::CreateTask(Box::new(b)),
        ])
        .unwrap();
    save(&mut app, &files.path("project.rfproj.json"));
    let mut reopened = load_project_app_state(&files.path("project.rfproj.json")).unwrap();
    assert_eq!(
        reopened.workspace.engineering.inputs(),
        app.workspace.engineering.inputs()
    );
    assert_eq!(
        reopened
            .workspace
            .engineering
            .calculate(&"task-2".into())
            .unwrap(),
        EquipmentResultStatus::Failed
    );
    let mut basis = reopened.workspace.engineering.inputs().basis.basis_entries[0].clone();
    basis.value.value_si = 150.0;
    assert_eq!(
        reopened
            .workspace
            .engineering
            .prepare(vec![C::UpdateBasis(basis)])
            .unwrap()
            .impacts()
            .len(),
        2
    );
}

#[test]
fn unknown_method_keeps_exact_file_and_is_never_an_editable_runtime_or_save_target() {
    let files = Files::new();
    let mut app = files.app();
    add_equipment(&mut app);
    save(&mut app, &files.path("known.rfproj.json"));
    let text = fs::read_to_string(files.path("known.rfproj.json"))
        .unwrap()
        .replace(
            "radishflow.separator.vertical-sb-working-volume",
            "future.method",
        );
    let path = files.path("unknown.rfproj.json");
    fs::write(&path, &text).unwrap();
    let StudioProjectOpen::ReadOnly(project) = read_studio_project(&path).unwrap() else {
        panic!("read-only required")
    };
    assert_eq!(project.original_json(), text);
    assert!(load_project_app_state(&path).is_err());
    let before = app.clone();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(app, before);
    let mut legacy = files.app();
    assert!(
        dispatch_document_lifecycle(
            &mut legacy,
            StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(fs::read_to_string(path).unwrap(), text);
}

#[test]
fn v3_presentation_only_upgrade_and_empty_project_never_downgrade() {
    let files = Files::new();
    let mut app = files.app();
    let mut units = app.workspace.project_presentation.display_units().clone();
    units
        .set_unit(QuantityKind::Duration, MeasurementUnit::Minute)
        .unwrap();
    app.workspace
        .project_presentation
        .apply(rf_ui::ProjectPresentationCommand::Apply(units.clone()));
    assert_eq!(project_save_version(&app), 3);
    assert!(project_save_requires_upgrade(&app));
    save(&mut app, &files.path("project.rfproj.json"));
    let mut reopened = load_project_app_state(&files.path("project.rfproj.json")).unwrap();
    assert_eq!(
        reopened.workspace.project_presentation.display_units(),
        &units
    );
    reopened
        .workspace
        .project_presentation
        .apply(rf_ui::ProjectPresentationCommand::Apply(
            DisplayUnitSet::si(),
        ));
    assert_eq!(project_save_version(&reopened), 3);
    assert!(!project_save_requires_upgrade(&reopened));
    dispatch_document_lifecycle(&mut reopened, StudioDocumentLifecycleCommand::Save).unwrap();
    let rf_store::project_v3::ProjectV3Read::Editable(project) =
        rf_store::project_v3::read(files.path("project.rfproj.json")).unwrap()
    else {
        unreachable!()
    };
    assert!(project.engineering.is_none());
}

#[test]
fn headless_protocol_rejects_supported_v3_before_any_operation() {
    let files = Files::new();
    let mut app = files.app();
    add_equipment(&mut app);
    let path = files.path("known.rfproj.json");
    save(&mut app, &path);
    let before = fs::read(&path).unwrap();
    let mut output = Vec::new();
    let mut error = Vec::new();
    let status = crate::headless::run_cli(
        &["inspect".into(), path.as_os_str().to_owned()],
        &mut output,
        &mut error,
    );
    assert_eq!(status, 2);
    assert!(String::from_utf8(output).unwrap().contains("schema"));
    assert!(error.is_empty());
    assert_eq!(fs::read(path).unwrap(), before);
}
