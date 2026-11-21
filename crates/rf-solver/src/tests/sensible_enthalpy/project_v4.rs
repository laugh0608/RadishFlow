use super::*;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use v4::{FlowPlanSupport, ProjectV4Read, StoredFlowPlan, StoredProjectV4, StoredSinglePhasePlan};

mod recycle;

const SUPPORT: FlowPlanSupport = FlowPlanSupport::SinglePhase;

fn project(flowsheet: Flowsheet, heat_duties_w: BTreeMap<UnitId, f64>) -> StoredProjectV4 {
    StoredProjectV4 {
        document: StoredProjectDocument {
            revision: 0,
            metadata: StoredDocumentMetadata::new(
                "single-phase",
                "Two-feed sensible enthalpy",
                UNIX_EPOCH,
            ),
            flowsheet,
        },
        display_units: Default::default(),
        engineering: None,
        steady_state_plan: StoredFlowPlan::SinglePhase(StoredSinglePhasePlan {
            min_temperature_k: 280.0,
            max_temperature_k: 420.0,
            pressure_pa: PRESSURE_PA,
            heat_capacities_j_per_mol_k: [
                ("component-a".into(), 30.0),
                ("component-b".into(), 50.0),
            ]
            .into(),
            heat_duties_w,
        }),
        object_identities: None,
        variable_watch_list: None,
    }
}

fn runtime(project: &StoredProjectV4) -> SolveSnapshot {
    let StoredFlowPlan::SinglePhase(plan) = &project.steady_state_plan else {
        panic!("single phase expected")
    };
    let model = SensibleEnthalpyModel::new(
        plan.heat_capacities_j_per_mol_k.clone(),
        plan.min_temperature_k..=plan.max_temperature_k,
        plan.pressure_pa,
    )
    .unwrap();
    let ControlledSolveOutcome::Completed(snapshot) = SequentialModularSolver
        .solve_single_phase_with_heat_duties_controlled(
            &model,
            &project.document.flowsheet,
            &plan.heat_duties_w,
            &mut UncontrolledSolve,
        )
        .unwrap()
    else {
        panic!("uncontrolled solve cannot cancel")
    };
    snapshot
}

#[test]
fn project_v4_reopens_both_specifications_and_reproduces_energy_evidence() {
    let directory = std::env::temp_dir().join(format!(
        "rf-v4-heat-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    for (kind, target_k, duty_w) in [
        (BuiltinUnitKind::Heater, 380.0, 7800.0),
        (BuiltinUnitKind::Cooler, 320.0, -4800.0),
    ] {
        for inverse in [false, true] {
            let flow = heater_cooler_case(kind, (!inverse).then_some(target_k));
            let duties = inverse
                .then_some((kind.as_str().into(), duty_w))
                .into_iter()
                .collect();
            let project = project(flow, duties);
            let before = runtime(&project);
            let path = directory.join(format!("{}-{inverse}.rfproj.json", kind.as_str()));
            let expected = ProjectV4Read::Editable(Box::new(project));
            let prepared = v4::prepare_write_with_support(
                &path,
                &expected,
                FileOverwritePolicy::Forbid,
                SUPPORT,
            )
            .unwrap();
            v4::write_prepared(&prepared, false).unwrap();
            let bytes = fs::read(&path).unwrap();
            let reopened = v4::read_with_support(&path, SUPPORT).unwrap();
            assert_eq!(reopened, expected);
            let ProjectV4Read::Editable(reopened) = reopened else {
                unreachable!()
            };
            let after = runtime(&reopened);
            assert_eq!(before, after);
            assert_close(
                after.streams[&"product".into()].temperature_k,
                target_k,
                1e-8,
            );
            assert_heater_cooler_energy_balance(&after, duty_w);
            let ProjectV4Read::ReadOnly(old) =
                v4::read_with_support(&path, FlowPlanSupport::ProductComposition).unwrap()
            else {
                panic!("legacy reader must protect the complete file")
            };
            assert_eq!(old.original_json().as_bytes(), bytes);
            assert!(v4::prepare_write(&path, &expected, FileOverwritePolicy::Allow).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn project_v4_rejects_conflict_on_read_and_write_and_keeps_existing_tp_project_editable() {
    let flow = heater_cooler_case(BuiltinUnitKind::Heater, Some(380.0));
    let valid = project(flow.clone(), BTreeMap::new());
    let supported = ProjectV4Read::Editable(Box::new(valid.clone()));
    let json = v4::to_pretty_json_with_support(&supported, SUPPORT).unwrap();
    let json = json.replace(
        r#""heatDutiesW": {}"#,
        r#""heatDutiesW": {"heater": 7800.0}"#,
    );
    let conflicting = ProjectV4Read::Editable(Box::new(project(
        flow.clone(),
        [("heater".into(), 7800.0)].into(),
    )));
    for error in [
        v4::parse_with_support(&json, SUPPORT).unwrap_err(),
        v4::to_pretty_json_with_support(&conflicting, SUPPORT).unwrap_err(),
    ] {
        assert_eq!(
            error.context().diagnostic_code(),
            Some("solver.single_phase.conflicting_heat_specifications")
        );
        assert_eq!(error.context().related_unit_ids(), &["heater".into()]);
    }

    let mut old = valid;
    old.steady_state_plan = StoredFlowPlan::Acyclic;
    let old = ProjectV4Read::Editable(Box::new(old));
    let old_json = v4::to_pretty_json(&old).unwrap();
    assert_eq!(
        v4::parse_with_support(&old_json, FlowPlanSupport::SinglePhaseRecycle).unwrap(),
        old
    );
    assert!(!old_json.contains(v4::SINGLE_PHASE_CAPABILITY));
    let provider = build_provider();
    let legacy = SolverServices {
        thermo: &provider,
        flash_solver: &PlaceholderTpFlashSolver,
    };
    let ProjectV4Read::Editable(reopened) = v4::parse_with_support(&old_json, SUPPORT).unwrap()
    else {
        unreachable!()
    };
    let solved = SequentialModularSolver
        .solve(&legacy, &reopened.document.flowsheet)
        .unwrap();
    assert_eq!(
        solved,
        SequentialModularSolver.solve(&legacy, &flow).unwrap()
    );
    assert!(
        solved.streams[&"product".into()]
            .bubble_dew_window
            .is_some()
    );
}
