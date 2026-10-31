use super::*;
use rf_ui::engineering::{GeometryAdjustments, GeometryAdoptionRequest, GeometryAdoptionTarget};

const ADOPTED: &str =
    include_str!("../../../../../crates/rf-store/tests/fixtures/project-v3-rating-adoption.json");

#[test]
fn adoption_save_consent_failure_actual_target_and_reopen_keep_fixed_evidence() {
    let files = Files::new("adoption-lifecycle");
    let path = files.path("before-adoption.rfproj.json");
    let mut previous: serde_json::Value = serde_json::from_str(ADOPTED).unwrap();
    previous["engineering"]["tasks"]
        .as_array_mut()
        .unwrap()
        .pop();
    let original = previous.to_string();
    fs::write(&path, &original).unwrap();
    let mut app = load_project_app_state(&path).unwrap();
    let session = &mut app.workspace.engineering;
    session.calculate(&"sizing-source".into()).unwrap();
    let proposal = session
        .prepare_geometry_adoption(GeometryAdoptionRequest {
            source_task: "sizing-source".into(),
            target: GeometryAdoptionTarget::New {
                task_id: "adopted-v3".into(),
                case_id: "adopted-v3-case".into(),
                tag: "V-202".into(),
                name: "Adopted".into(),
                copy_case: true,
            },
            adjustments: GeometryAdjustments::default(),
        })
        .unwrap();
    session.commit_geometry_adoption(proposal).unwrap();
    let key = EquipmentCaseKey {
        task_id: "adopted-v3".into(),
        case_id: "adopted-v3-case".into(),
    };
    assert_eq!(
        session.case_result_status(&key),
        EquipmentResultStatus::Missing
    );
    session.calculate_rating(&key).unwrap();
    let before = session.clone();
    assert_eq!(
        project_save_compatibility(&app, &path).unwrap(),
        Some(ProjectSaveCompatibility::RatingAdoptionCapability)
    );
    assert!(dispatch_document_lifecycle(&mut app, StudioDocumentLifecycleCommand::Save).is_err());
    assert_eq!(app.workspace.engineering, before);
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
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
    let saved = files.path("after-adoption.rfproj.json");
    dispatch_document_lifecycle(
        &mut app,
        StudioDocumentLifecycleCommand::SaveUpgraded {
            path: saved.clone(),
        },
    )
    .unwrap();
    assert!(!app.workspace.engineering.is_dirty());
    assert_eq!(project_save_compatibility(&app, &saved).unwrap(), None);
    assert_eq!(
        project_save_compatibility(&app, &path).unwrap(),
        Some(ProjectSaveCompatibility::RatingAdoptionCapability)
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    let reopened = load_project_app_state(&saved).unwrap();
    assert_eq!(
        reopened.workspace.engineering.case_result_status(&key),
        EquipmentResultStatus::Missing
    );
    assert_eq!(reopened.workspace.engineering.inputs(), before.inputs());
    assert!(!reopened.workspace.engineering.can_undo());
    assert!(!reopened.workspace.engineering.is_dirty());
}
