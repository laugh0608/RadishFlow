use super::*;
use crate::{
    StudioProjectOpen, StudioReadOnlyProject, WorkspaceRunCancelReason, read_studio_project,
    read_studio_project_for_review,
};
use rf_ui::{FlowRunDiscardReason, engineering::concentric_tube::LimitCheck};
use std::{
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

fn sample(flow: u8) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/flowsheets")
        .join(format!("h3-linked-water-{flow}.rfproj.json"))
}

fn open(flow: u8) -> (Box<StudioH3FlowReview>, EquipmentCaseKey) {
    let path = sample(flow);
    let StudioProjectOpen::ReadOnly(project) = read_studio_project_for_review(&path).unwrap()
    else {
        panic!("linked fixture must open read-only");
    };
    let StudioReadOnlyProject::H3Flow(project) = *project else {
        panic!("linked fixture must open in the linked review");
    };
    assert_eq!(project.original_json(), fs::read_to_string(path).unwrap());
    let task = project.cases().next().unwrap();
    let member = EquipmentCaseKey {
        task_id: task.id.clone(),
        case_id: task.case.id.clone(),
    };
    (project, member)
}

fn poll_until_idle(review: &mut StudioH3FlowReview, owner: &mut WorkspaceRunOwner) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !owner.is_idle() || review.activity(owner).is_some() {
        review.poll(owner, SystemTime::now());
        assert!(Instant::now() < deadline, "review did not settle");
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn linked_review_accepts_the_two_saved_water_anchors_and_invalidates_on_rerun() {
    for (flow, duty, dp, ua, hot_out, cold_out, cold_limit) in [
        (
            40,
            5_521.707_636_437_1,
            23_679.130_040_479,
            874.553_423_114_86,
            297.914_262_742_51,
            294.982_508_040_12,
            LimitCheck::Passed,
        ),
        (
            50,
            5_877.489_522_370_7,
            34_962.547_685_264,
            945.093_778_801_02,
            297.576_906_873_93,
            294.710_466_075_30,
            LimitCheck::Failed,
        ),
    ] {
        let (mut project, member) = open(flow);
        let mut owner = WorkspaceRunOwner::new();
        assert!(matches!(
            project.result(&member),
            H3FlowReviewResult::Missing
        ));
        assert!(project.start(&mut owner, &member, SystemTime::now()));
        poll_until_idle(&mut project, &mut owner);
        let H3FlowReviewResult::Current { rating, preview } = project.result(&member) else {
            panic!("missing result: {:?}", project.result(&member));
        };
        assert!((rating.heat_duty_w - duty).abs() <= 1e-6);
        assert!((rating.ua_w_per_k - ua).abs() <= 1e-6);
        assert!((rating.hot.passage.pressure_drop_pa - 9_799.896_961_973_8).abs() <= 1e-4);
        assert!((rating.cold.passage.pressure_drop_pa - dp).abs() <= 1e-4);
        assert!((rating.hot.outlet.temperature_k - hot_out).abs() <= 1e-8);
        assert!((rating.cold.outlet.temperature_k - cold_out).abs() <= 1e-8);
        assert_eq!(rating.checks.heat_duty, LimitCheck::Passed);
        assert_eq!(rating.checks.hot_pressure_drop, LimitCheck::Passed);
        assert_eq!(rating.checks.cold_pressure_drop, cold_limit);
        assert_eq!(preview.cold().1.molar_flow_mol_s, Some(f64::from(flow)));
        let old_sequence = preview.sequence();
        assert!(project.start(&mut owner, &member, SystemTime::now()));
        assert!(!matches!(
            project.result(&member),
            H3FlowReviewResult::Current { .. }
        ));
        poll_until_idle(&mut project, &mut owner);
        assert!(matches!(
            project.result(&member),
            H3FlowReviewResult::Current { preview, .. } if preview.sequence() > old_sequence
        ));
    }
}

#[test]
fn cancelled_and_closed_linked_reviews_discard_successful_unaccepted_worker_results() {
    let (mut project, member) = open(40);
    let mut owner = WorkspaceRunOwner::new();
    let now = SystemTime::now();
    assert!(project.start(&mut owner, &member, now));
    owner.wait_for_unaccepted_worker_result(now);
    project.cancel(&mut owner);
    assert_eq!(
        project.activity(&owner),
        Some((&member, H3FlowReviewPhase::Stopping))
    );
    poll_until_idle(&mut project, &mut owner);
    assert!(matches!(
        project.result(&member),
        H3FlowReviewResult::Cancelled
    ));
    let cancelled = owner.history().back().unwrap();
    assert_eq!(cancelled.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert_eq!(
        cancelled.cancellation,
        Some(WorkspaceRunCancelReason::UserRequested)
    );
    assert!(matches!(
        cancelled.disposition,
        FlowRunDisposition::Discarded(_)
    ));

    assert!(project.start(&mut owner, &member, now));
    owner.wait_for_unaccepted_worker_result(now);
    project.detach(&mut owner);
    drop(project);
    let (mut reopened, reopened_member) = open(40);
    assert!(!reopened.start(&mut owner, &reopened_member, now));
    assert!(matches!(
        reopened.result(&reopened_member),
        H3FlowReviewResult::Missing
    ));
    poll_until_idle(&mut reopened, &mut owner);
    let closed = owner.history().back().unwrap();
    assert_eq!(closed.outcome, WorkspaceExecutionOutcome::Succeeded);
    assert_eq!(
        closed.cancellation,
        Some(WorkspaceRunCancelReason::Detached)
    );
    assert_eq!(
        closed.disposition,
        FlowRunDisposition::Discarded(FlowRunDiscardReason::DifferentSession)
    );
    assert!(matches!(
        reopened.result(&reopened_member),
        H3FlowReviewResult::Missing
    ));
    assert!(reopened.start(&mut owner, &reopened_member, now));
    poll_until_idle(&mut reopened, &mut owner);
    assert!(matches!(
        reopened.result(&reopened_member),
        H3FlowReviewResult::Current { .. }
    ));
}

#[test]
fn consumers_without_linked_h3_support_keep_the_opaque_original() {
    let path = sample(40);
    let StudioProjectOpen::ReadOnly(project) = read_studio_project(&path).unwrap() else {
        panic!("consumer without linked review support must stay read-only");
    };
    assert!(matches!(project.as_ref(), StudioReadOnlyProject::V4(_)));
    assert_eq!(project.original_json(), fs::read_to_string(path).unwrap());
}
