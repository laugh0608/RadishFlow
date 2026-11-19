use super::*;

#[test]
fn single_phase_upgrade_requires_review_and_legacy_save_preserves_target() {
    let dir = Directory::new();
    let path = dir.0.join("project.json");
    let old_project = fixture();
    let old_text = json_project(old_project.clone());
    fs::write(&path, &old_text).unwrap();
    let mut new_project = old_project.clone();
    new_project.steady_state_plan = StoredFlowPlan::SinglePhase(StoredSinglePhasePlan {
        min_temperature_k: 280.0,
        max_temperature_k: 420.0,
        pressure_pa: 1e5,
        heat_capacities_j_per_mol_k: Default::default(),
        heat_duties_w: Default::default(),
    });
    let new_project = ProjectV4Read::Editable(Box::new(new_project));
    let prepared = prepare_write_with_support(
        &path,
        &new_project,
        FileOverwritePolicy::Allow,
        FlowPlanSupport::SinglePhase,
    )
    .unwrap();
    assert!(prepared.requires_upgrade());
    assert!(write_prepared(&prepared, false).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), old_text);
    write_prepared(&prepared, true).unwrap();
    let protected_bytes = fs::read(&path).unwrap();
    let old_project = ProjectV4Read::Editable(Box::new(old_project));
    for support in [
        FlowPlanSupport::ProductComposition,
        FlowPlanSupport::SinglePhase,
    ] {
        assert!(
            prepare_write_with_support(&path, &old_project, FileOverwritePolicy::Allow, support)
                .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), protected_bytes);
    }
    assert_eq!(
        read_with_support(path, FlowPlanSupport::SinglePhase).unwrap(),
        new_project
    );
}
