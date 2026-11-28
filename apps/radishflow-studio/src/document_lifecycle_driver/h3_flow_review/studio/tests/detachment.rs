use super::*;
use crate::H3AssociationSaveTarget::{NewFile, Source};
use crate::document_lifecycle_driver::h3_flow_review::{EQUIPMENT_SUPPORT, FLOW_SUPPORT};
use rf_store::project_v4;
use rf_ui::engineering::concentric_tube::RatingRevisions;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new(flow: u8) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "rf-h3-detach-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("source.rfproj.json");
        fs::copy(sample(flow), &path).unwrap();
        Self(path)
    }

    fn linked(&self) -> (Box<StudioH3FlowReview>, EquipmentCaseKey) {
        let StudioProjectOpen::ReadOnly(view) = read_studio_project_for_review(&self.0).unwrap()
        else {
            panic!("expected H3 review")
        };
        let StudioReadOnlyProject::H3Flow(review) = *view else {
            panic!("expected linked review")
        };
        let task = review.cases().next().unwrap();
        let member = EquipmentCaseKey {
            task_id: task.id.clone(),
            case_id: task.case.id.clone(),
        };
        (review, member)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.0.parent().unwrap());
    }
}

fn run(review: &mut StudioH3FlowReview, member: &EquipmentCaseKey, owner: &mut WorkspaceRunOwner) {
    assert!(review.start(owner, member, SystemTime::now()));
    poll_until_idle(review, owner);
}

fn saved(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn detachment_freezes_both_water_anchors_and_preserves_the_independent_rating() {
    for flow in [40, 50] {
        let file = Fixture::new(flow);
        let source = fs::read(&file.0).unwrap();
        let (mut review, member) = file.linked();
        assert!(review.prepare_detachment(&member, Source).is_err());
        run(&mut review, &member, &mut WorkspaceRunOwner::new());
        let H3FlowReviewResult::Current { rating, .. } = review.result(&member) else {
            panic!("expected linked rating")
        };
        let expected = *rating;
        let copy = file.0.with_file_name("independent.rfproj.json");
        review
            .prepare_detachment(&member, if flow == 40 { Source } else { NewFile(&copy) })
            .unwrap();
        let path = review.confirm_detachment(&member).unwrap();
        assert!(review.confirm_detachment(&member).is_err());
        if flow == 50 {
            assert_eq!(path, copy);
            assert_eq!(fs::read(&file.0).unwrap(), source);
        }
        let mut output = saved(&path);
        let case = &output["engineering"]["tasks"][0]["case"];
        for (side, kelvin, molar_flow) in [("hot", 303.15, 14.0), ("cold", 293.15, f64::from(flow))]
        {
            assert_eq!(
                case[side]["inlet"],
                json!({"temperatureK":kelvin,"pressurePa":100000.0,"molarFlowMolS":molar_flow})
            );
        }
        assert!(case.get("flowAssociation").is_none());
        assert_eq!(output["document"]["revision"], 2);
        assert_eq!(output["engineering"]["tasks"][0]["revision"], 7);
        assert_eq!(case["revision"], 13);
        assert!(
            !output["requiredCapabilities"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["id"] == project_v4::CONCENTRIC_TUBE_FLOW_ASSOCIATION_CAPABILITY)
        );
        if flow == 40 {
            output["document"]["flowsheet"]["streams"]["cold-feed"]["total_molar_flow_mol_s"] =
                json!(50.0);
            fs::write(&path, serde_json::to_vec(&output).unwrap()).unwrap();
        }
        let StudioProjectOpen::ReadOnly(view) = read_studio_project_for_review(&path).unwrap()
        else {
            panic!("independent review missing")
        };
        let StudioReadOnlyProject::H3(independent) = *view else {
            panic!("expected independent review")
        };
        let task = independent.case(&member).unwrap();
        assert_eq!(
            independent
                .rate(&member, RatingRevisions::from(task))
                .unwrap()
                .unwrap(),
            expected
        );
    }
}

#[test]
fn detachment_confirmation_is_single_use_and_bound_to_the_source_case_and_snapshot() {
    let file = Fixture::new(40);
    let original = fs::read(&file.0).unwrap();
    let (mut review, member) = file.linked();
    let mut owner = WorkspaceRunOwner::new();
    assert!(review.start_detachment_preview(&mut owner, &member, SystemTime::now()));
    poll_until_idle(&mut review, &mut owner);
    assert!(matches!(
        review.result(&member),
        H3FlowReviewResult::Missing
    ));
    review.prepare_detachment(&member, Source).unwrap();
    review.cancel_detachment();
    assert!(review.confirm_detachment(&member).is_err());
    review.prepare_detachment(&member, Source).unwrap();
    let other_file = Fixture::new(40);
    let (mut other, _) = other_file.linked();
    run(&mut other, &member, &mut owner);
    assert!(other.confirm_detachment(&member).is_err());
    assert_eq!(fs::read(&other_file.0).unwrap(), original);
    let wrong = EquipmentCaseKey {
        case_id: "another-case".into(),
        ..member.clone()
    };
    assert!(review.confirm_detachment(&wrong).is_err());
    assert!(review.confirm_detachment(&member).is_err());

    review.prepare_detachment(&member, Source).unwrap();
    assert!(review.start(&mut owner, &member, SystemTime::now()));
    assert!(review.confirm_detachment(&member).is_err());
    poll_until_idle(&mut review, &mut owner);
    assert_eq!(fs::read(&file.0).unwrap(), original);

    let missing_dir = file.0.with_file_name("gone");
    fs::create_dir(&missing_dir).unwrap();
    let unavailable = missing_dir.join("copy.rfproj.json");
    review
        .prepare_detachment(&member, NewFile(&unavailable))
        .unwrap();
    fs::remove_dir(&missing_dir).unwrap();
    assert!(review.confirm_detachment(&member).is_err());
    assert!(!unavailable.exists());
    assert_eq!(fs::read(&file.0).unwrap(), original);

    let copy = file.0.with_file_name("new.rfproj.json");
    review.prepare_detachment(&member, NewFile(&copy)).unwrap();
    fs::write(&copy, b"other writer").unwrap();
    assert!(review.confirm_detachment(&member).is_err());
    assert_eq!(fs::read(&copy).unwrap(), b"other writer");
    assert!(review.prepare_detachment(&member, NewFile(&copy)).is_err());

    review.prepare_detachment(&member, Source).unwrap();
    fs::write(&file.0, b"external source").unwrap();
    assert!(review.confirm_detachment(&member).is_err());
    assert_eq!(fs::read(&file.0).unwrap(), b"external source");
    assert!(review.prepare_detachment(&member, Source).is_err());
    fs::write(&file.0, &original).unwrap();
    assert!(review.detachment_needs_reload());
    assert!(review.prepare_detachment(&member, Source).is_err());
    let (mut review, member) = file.linked();
    run(&mut review, &member, &mut owner);
    let retry = file.0.with_file_name("retry.rfproj.json");
    review.prepare_detachment(&member, NewFile(&retry)).unwrap();
    fs::remove_file(&file.0).unwrap();
    assert!(review.confirm_detachment(&member).is_err());
    assert!(!file.0.exists() && !retry.exists());
    fs::write(&file.0, &original).unwrap();

    let (mut review, member) = file.linked();
    run(&mut review, &member, &mut owner);
    review.prepare_detachment(&member, NewFile(&retry)).unwrap();
    let mut identities = review
        .review
        .workspace()
        .current_object_identities()
        .unwrap();
    *identities
        .streams
        .get_mut(&StreamId::new("cold-feed"))
        .unwrap() = rf_types::ObjectCreationId::from_random_bytes([42; 16]);
    review
        .review
        .flow
        .workspace
        .install_saved_object_identities(identities);
    assert!(review.confirm_detachment(&member).is_err());
    assert!(!retry.exists());
    assert_eq!(fs::read(&file.0).unwrap(), original);
    let (mut reloaded, member) = file.linked();
    run(&mut reloaded, &member, &mut owner);
    reloaded
        .prepare_detachment(&member, NewFile(&retry))
        .unwrap();
    assert_eq!(reloaded.confirm_detachment(&member).unwrap(), retry);
}

#[test]
fn detachment_of_one_of_two_pairs_preserves_other_inputs_and_the_ordinary_write_guard() {
    let file = Fixture::new(50);
    let mut input = saved(&file.0);
    let mut sibling = input["engineering"]["tasks"][0].clone();
    sibling["id"] = json!("h3-still-linked");
    sibling["tag"] = json!("E-302");
    sibling["case"]["id"] = json!("second-water");
    input["engineering"]["tasks"]
        .as_array_mut()
        .unwrap()
        .push(sibling.clone());
    fs::write(&file.0, serde_json::to_vec(&input).unwrap()).unwrap();
    let (mut review, member) = file.linked();
    let mut owner = WorkspaceRunOwner::new();
    run(&mut review, &member, &mut owner);
    let before = review.review.saved.case(&member).unwrap().clone();
    let copy = file.0.with_file_name("mixed.rfproj.json");
    review.prepare_detachment(&member, NewFile(&copy)).unwrap();
    review.confirm_detachment(&member).unwrap();
    let document =
        project_v4::read_with_equipment_support(&copy, FLOW_SUPPORT, EQUIPMENT_SUPPORT).unwrap();
    let denied = project_v4::prepare_write_with_equipment_support(
        &file.0,
        &document,
        rf_store::FileOverwritePolicy::Allow,
        FLOW_SUPPORT,
        EQUIPMENT_SUPPORT,
    )
    .unwrap_err();
    assert_eq!(
        denied.context().diagnostic_code(),
        Some("store.project.h3_flow_association_removal")
    );
    review.prepare_detachment(&member, Source).unwrap();
    review.confirm_detachment(&member).unwrap();
    let output = saved(&file.0);
    assert_eq!(output["engineering"]["tasks"][1], sibling);
    let (mut reopened, linked_member) = file.linked();
    assert_eq!(reopened.cases().count(), 1);
    let independent = reopened.saved_inputs();
    let task = independent.independent_cases().next().unwrap();
    assert_eq!(task.id, member.task_id);
    assert_eq!(task.geometry, before.geometry);
    assert_eq!(task.case.hot.properties, before.case.hot.properties);
    assert_eq!(task.case.cold.properties, before.case.cold.properties);
    assert_eq!(task.case.limits, before.case.limits);
    let detached = independent
        .rate(&member, RatingRevisions::from(task))
        .unwrap()
        .unwrap();
    assert!(matches!(
        reopened.result(&linked_member),
        H3FlowReviewResult::Missing
    ));
    run(&mut reopened, &linked_member, &mut owner);
    let H3FlowReviewResult::Current { rating, .. } = reopened.result(&linked_member) else {
        panic!("linked sibling unavailable")
    };
    assert_eq!(detached, *rating);
}
