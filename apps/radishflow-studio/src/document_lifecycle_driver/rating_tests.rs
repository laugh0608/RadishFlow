use super::*;
use rf_model::equipment::*;
use rf_ui::engineering::{EngineeringCommand as C, EquipmentCaseKey, EquipmentResultStatus};
use std::fs;

const FIXTURE: &str =
    include_str!("../../../../crates/rf-store/tests/fixtures/project-v3-rating.json");
struct Files(PathBuf);
impl Files {
    fn new(name: &str) -> Self {
        let root = super::tests::temp_project_path(name).with_extension("dir");
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    fn rating(&self) -> AppState {
        let path = self.path("rating.rfproj.json");
        fs::write(&path, FIXTURE).unwrap();
        load_project_app_state(&path).unwrap()
    }
    fn foundation(&self) -> AppState {
        let path = self.path("foundation.rfproj.json");
        let mut value: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
        value["engineering"]["tasks"] = serde_json::json!([]);
        fs::write(&path, value.to_string()).unwrap();
        load_project_app_state(&path).unwrap()
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn member() -> EquipmentCaseKey {
    EquipmentCaseKey {
        task_id: "rating-1".into(),
        case_id: "rating-case-1".into(),
    }
}
fn rating_task() -> SeparatorRatingTask {
    let rf_store::project_v3::ProjectV3Read::Editable(p) =
        rf_store::project_v3::parse(FIXTURE).unwrap()
    else {
        unreachable!()
    };
    p.engineering.unwrap().tasks[0].as_rating().unwrap().clone()
}
fn change_geometry(app: &mut AppState) {
    let mut geometry = app
        .workspace
        .engineering
        .inputs()
        .rating_task(&member().task_id)
        .unwrap()
        .geometry
        .clone();
    geometry
        .inputs
        .remove(&SeparatorGeometryInput::WorkingLiquidHeight);
    app.workspace
        .engineering
        .apply(vec![C::UpdateRatingGeometry {
            task: member().task_id,
            geometry: Box::new(geometry),
        }])
        .unwrap();
}

#[test]
fn rating_open_save_and_reopen_preserve_inputs_units_and_sources_but_reset_results_and_history() {
    let files = Files::new("rating-lifecycle");
    let mut app = files.rating();
    let path = app.workspace.document_path.clone().unwrap();
    let document = app.workspace.document.clone();
    let units = app.workspace.project_presentation.display_units().clone();
    assert_eq!(
        app.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    assert!(!project_save_requires_upgrade(&app));
    assert_eq!(
        app.workspace
            .engineering
            .calculate_rating(&member())
            .unwrap(),
        EquipmentResultStatus::Current
    );
    assert!(
        app.workspace
            .engineering
            .current_rating(&member())
            .unwrap()
            .performance()
            .gas
            .margin_fraction
            < 0.0
    );
    let task = app
        .workspace
        .engineering
        .inputs()
        .rating_task(&member().task_id)
        .unwrap()
        .clone();
    app.workspace
        .engineering
        .apply(vec![C::UpdateRatingLabels {
            task: task.id,
            revision: task.revision,
            tag: task.tag,
            name: "Saved rating".into(),
        }])
        .unwrap();
    assert!(app.workspace.engineering.can_undo());
    dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).unwrap();
    assert_eq!(app.workspace.document, document);
    assert!(!app.workspace.engineering.is_dirty());
    assert_eq!(
        app.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Current
    );
    let original = app.workspace.engineering.inputs().clone();
    let late = app.workspace.engineering.begin_rating(&member()).unwrap();
    let mut reopened = load_project_app_state(&path).unwrap();
    assert_eq!(reopened.workspace.engineering.inputs(), &original);
    assert_eq!(
        reopened.workspace.project_presentation.display_units(),
        &units
    );
    assert!(!reopened.workspace.engineering.can_undo());
    assert_eq!(
        reopened.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    assert!(
        reopened
            .workspace
            .engineering
            .complete_calculation(late.execute())
            .is_err()
    );
    let before = fs::read(&path).unwrap();
    assert!(load_legacy_project_app_state(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
}

#[test]
fn adding_rating_to_existing_v3_requires_consent_and_stays_v3_after_confirmation() {
    let files = Files::new("rating-consent");
    let mut app = files.foundation();
    let path = app.workspace.document_path.clone().unwrap();
    let before = fs::read(&path).unwrap();
    app.workspace
        .engineering
        .apply(vec![C::CreateRatingTask(Box::new(rating_task()))])
        .unwrap();
    let formal = app.workspace.engineering.clone();
    assert!(project_save_requires_upgrade(&app));
    assert_eq!(
        project_save_compatibility(&app, &path).unwrap(),
        Some(ProjectSaveCompatibility::RatingCapability)
    );
    for command in [
        StudioDocumentLifecycleCommand::Save,
        StudioDocumentLifecycleCommand::SaveAs {
            path: files.path("copy.rfproj.json"),
        },
    ] {
        assert!(dispatch_document_lifecycle(&mut app, command).is_err());
        assert_eq!(app.workspace.engineering, formal);
        assert_eq!(fs::read(&path).unwrap(), before);
    }
    assert!(!files.path("copy.rfproj.json").exists());
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    assert_eq!(project_save_version(&app), 3);
    assert!(!project_save_requires_upgrade(&app));
    assert_eq!(project_save_compatibility(&app, &path).unwrap(), None);
    assert!(app.workspace.engineering.can_undo());
    assert!(!app.workspace.engineering.is_dirty());
}

#[test]
fn save_as_to_sizing_only_target_cannot_bypass_rating_compatibility_confirmation() {
    let files = Files::new("rating-save-as");
    let mut app = files.rating();
    let foundation = files.foundation();
    let path = foundation.workspace.document_path.unwrap();
    let before = fs::read(&path).unwrap();
    assert!(!project_save_requires_upgrade(&app));
    assert_eq!(
        project_save_compatibility(&app, &path).unwrap(),
        Some(ProjectSaveCompatibility::RatingCapability)
    );
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(fs::read(&path).unwrap(), before);
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    assert!(
        load_project_app_state(&path)
            .unwrap()
            .workspace
            .engineering
            .inputs()
            .has_rating()
    );
}

#[test]
fn legacy_upgrade_merges_rating_warning_and_write_failure_preserves_dirty_state_and_history() {
    let files = Files::new("rating-failure");
    let mut app = files.rating();
    change_geometry(&mut app);
    let before = app.workspace.engineering.clone();
    let path = files.path("legacy.rfproj.json");
    fs::write(
        &path,
        include_str!("../../../../crates/rf-store/tests/fixtures/project-v2-engineering.json"),
    )
    .unwrap();
    assert_eq!(
        project_save_compatibility(&app, &path).unwrap(),
        Some(ProjectSaveCompatibility::Format {
            version: 3,
            includes_rating: true
        })
    );
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveAs { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(app.workspace.engineering, before);
    let bad = files.path("directory.rfproj.json");
    fs::create_dir(&bad).unwrap();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveUpgraded { path: bad }
        )
        .is_err()
    );
    assert_eq!(app.workspace.engineering, before);
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() },
    )
    .unwrap();
    let reopened = load_project_app_state(&path).unwrap();
    assert!(
        !reopened
            .workspace
            .engineering
            .inputs()
            .rating_task(&member().task_id)
            .unwrap()
            .geometry
            .inputs
            .contains_key(&SeparatorGeometryInput::WorkingLiquidHeight)
    );
    assert_eq!(
        reopened.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
}

#[test]
fn future_rating_is_entirely_read_only_and_cannot_be_replaced_by_known_rating() {
    let files = Files::new("rating-future");
    let mut app = files.rating();
    let path = files.path("future.rfproj.json");
    let text = FIXTURE.replace("\"version\": 1", "\"version\": 3");
    fs::write(&path, &text).unwrap();
    let StudioProjectOpen::ReadOnly(project) = read_studio_project(&path).unwrap() else {
        panic!("future method")
    };
    assert_eq!(project.original_json(), text);
    let before = app.clone();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveUpgraded { path: path.clone() }
        )
        .is_err()
    );
    assert_eq!(app, before);
    assert_eq!(fs::read_to_string(&path).unwrap(), text);
}

#[test]
fn matrix_upgrade_save_as_failure_and_reopen_preserve_versioned_capability() {
    let files = Files::new("matrix-lifecycle");
    let mut app = files.rating();
    let path = app.workspace.document_path.clone().unwrap();
    let mut copy = rating_task().cases.remove(0);
    copy.id = "second-case".into();
    app.workspace
        .engineering
        .apply(vec![
            C::UpgradeRatingMethod(member().task_id),
            C::AddRatingCase {
                task: member().task_id,
                case: Box::new(copy),
            },
        ])
        .unwrap();
    app.workspace
        .engineering
        .calculate_rating(&member())
        .unwrap();
    let before = app.workspace.engineering.clone();
    assert_eq!(
        project_save_compatibility(&app, &path).unwrap(),
        Some(ProjectSaveCompatibility::RatingMatrixCapability)
    );
    assert!(dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), FIXTURE);
    assert_eq!(app.workspace.engineering, before);
    let bad = files.path("directory");
    fs::create_dir(&bad).unwrap();
    assert!(
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::SaveUpgraded { path: bad }
        )
        .is_err()
    );
    assert_eq!(app.workspace.engineering, before);
    let new_path = files.path("matrix.rfproj.json");
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded {
            path: new_path.clone(),
        },
    )
    .unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), FIXTURE);
    assert!(!app.workspace.engineering.is_dirty());
    assert_eq!(project_save_compatibility(&app, &new_path).unwrap(), None);
    assert_eq!(
        project_save_compatibility(&app, &path).unwrap(),
        Some(ProjectSaveCompatibility::RatingMatrixCapability)
    );
    let reopened = load_project_app_state(&new_path).unwrap();
    let t = reopened
        .workspace
        .engineering
        .inputs()
        .rating_task(&member().task_id)
        .unwrap();
    assert_eq!((t.method.version, t.cases.len()), (2, 2));
    assert_eq!(
        reopened.workspace.engineering.case_result_status(&member()),
        EquipmentResultStatus::Missing
    );
    assert!(!reopened.workspace.engineering.can_undo());
    app.workspace
        .engineering
        .apply(vec![C::DeleteRatingCase(EquipmentCaseKey {
            task_id: member().task_id,
            case_id: "second-case".into(),
        })])
        .unwrap();
    dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).unwrap();
    let reopened = load_project_app_state(&new_path).unwrap();
    assert_eq!(
        reopened
            .workspace
            .engineering
            .inputs()
            .rating_method_version(),
        2
    );
    assert_eq!(
        reopened
            .workspace
            .engineering
            .inputs()
            .rating_task(&member().task_id)
            .unwrap()
            .cases
            .len(),
        1
    );
}
