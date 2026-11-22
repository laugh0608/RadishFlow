use super::*;
use crate::{
    WorkspaceSolveDispatch, WorkspaceSolveService, WorkspaceSolveSkipReason, WorkspaceSolveTrigger,
    build_workspace_solve_request,
    flow_plan_actions::runtime_configuration_sha256,
    prepare_workspace_solve,
    qs1_test_support::select,
    qs4_test_support::serial_case,
    solve_workspace_with_property_package,
    test_support::{
        OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID as PACKAGE,
        build_official_binary_hydrocarbon_in_memory_provider,
    },
};
use rf_solver::{
    mixed::{MixedOptions, MixedRecycleConfiguration, MixedRecycleOptions, SERIAL_RECYCLE_METHOD},
    partitioned::{ABSOLUTE_FLOW_TOLERANCE, EvaluationPhase, SCALED_FLOW_TOLERANCE},
};
use rf_types::units::{MeasurementUnit, QuantityKind};
use rf_ui::{
    FlowResultRecord, ProjectPresentationCommand, SimulationMode,
    result_query::{FlowExecutionEvidence, FlowResultView},
    variable_browser::{
        ObjectId, VariableField, VariableId, VariableSection, VariableWatchCommand,
    },
};
use serde_json::Value;

const SUPPORT: FlowPlanSupport = FlowPlanSupport::MixedSerialRecycle;
const REFERENCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/flowsheets/fixed-tp-serial-recycle-eo-reference.rfproj.json"
));

fn reopen(path: &Path) -> AppState {
    let StudioProjectOpen::Editable(app) = read_studio_project_with_support(path, SUPPORT).unwrap()
    else {
        panic!("explicit M3 reader must reopen as editable");
    };
    *app
}

fn regions_mut(options: &mut MixedOptions) -> &mut [MixedRecycleOptions; 2] {
    let Some(MixedRecycleConfiguration::Serial(regions)) = options.recycle.as_deref_mut() else {
        panic!("serial fixture");
    };
    regions
}

fn selected(initial: [Option<[f64; 2]>; 2]) -> FlowPlanConfiguration {
    let FlowPlanConfiguration::Mixed(mut options) =
        serial_case(100).workspace.flow_plan.current().clone()
    else {
        panic!("serial fixture");
    };
    for (index, region) in regions_mut(&mut options).iter_mut().enumerate() {
        region.max_iteration_evaluations = [100, 240][index];
        region.tear.initial_component_flows_mol_s = initial[index].map(|[methane, ethane]| {
            [("methane".into(), methane), ("ethane".into(), ethane)].into()
        });
    }
    FlowPlanConfiguration::Mixed(options)
}

fn renamed_reference(path: &Path) -> AppState {
    let mut wire: Value = serde_json::from_str(REFERENCE).unwrap();
    let flow = &mut wire["document"]["flowsheet"];
    for (old, new) in [("s-ra", "z-upstream"), ("s-rb", "a-downstream")] {
        let streams = flow["streams"].as_object_mut().unwrap();
        let mut stream = streams.remove(old).unwrap();
        stream["id"] = new.into();
        streams.insert(new.into(), stream);
        for unit in flow["units"].as_object_mut().unwrap().values_mut() {
            for port in unit["ports"].as_array_mut().unwrap() {
                if port["stream_id"] == old {
                    port["stream_id"] = new.into();
                }
            }
        }
    }
    fs::write(path, wire.to_string()).unwrap();
    load_project_app_state(path).unwrap()
}

fn verify_result(app: &AppState) -> FlowExecutionEvidence {
    let FlowResultView::Current {
        snapshot: FlowResultRecord::Mixed(result),
        origin,
    } = app.workspace.flow_result()
    else {
        panic!("explicit run must publish qualified M3");
    };
    let result = &result.result;
    assert_eq!(result.streams.len(), 13);
    assert_eq!(
        result
            .serial_recycles
            .iter()
            .map(|region| region.recycle.tear.stream_id.as_str())
            .collect::<Vec<_>>(),
        ["z-upstream", "a-downstream"]
    );
    for (region, budget) in result.serial_recycles.iter().zip([100, 240]) {
        assert_eq!(region.recycle.max_iteration_evaluations, budget);
        let verified = region.recycle.residual_trace.last().unwrap();
        assert_eq!(verified.phase, EvaluationPhase::Verification);
        assert!(verified.max_absolute_mol_s <= ABSOLUTE_FLOW_TOLERANCE);
        assert!(verified.max_scaled <= SCALED_FLOW_TOLERANCE);
        assert!(
            region
                .balance
                .as_ref()
                .unwrap()
                .component_residuals_mol_s
                .values()
                .all(|v| v.abs() <= ABSOLUTE_FLOW_TOLERANCE)
        );
    }
    assert!(
        result
            .global_material
            .component_residuals_mol_s
            .values()
            .all(|v| v.abs() <= ABSOLUTE_FLOW_TOLERANCE)
    );
    for (stream, methane, ethane) in [
        ("z-upstream", 3.0 / 5.0, 2.0 / 5.0),
        ("a-downstream", 11.0 / 5.0, 24.0 / 5.0),
        ("s-ab", 6.0 / 5.0, 4.0 / 5.0),
        ("s-pb", 22.0 / 15.0, 16.0 / 5.0),
        ("s-qb", 11.0 / 15.0, 8.0 / 5.0),
    ] {
        let state = &result.streams[&stream.into()];
        for (component, expected) in [("methane", methane), ("ethane", ethane)] {
            let n = state.overall_mole_fractions[&component.into()] * state.total_molar_flow_mol_s;
            assert!((n - expected).abs() <= 1e-9, "{stream}/{component}");
        }
    }
    let execution = origin.execution().unwrap();
    assert_eq!(
        execution.plan.as_ref().unwrap().method,
        SERIAL_RECYCLE_METHOD
    );
    execution.clone()
}

#[test]
fn m3_saved_project_keeps_both_configurations_and_requires_an_explicit_rerun() {
    let dir = Directory::new();
    let provider = build_official_binary_hydrocarbon_in_memory_provider(PACKAGE);
    let solver = WorkspaceSolveService::new();
    for (index, initialization) in [
        [None, None],
        [Some([3.0, 1.0]), Some([2.0, 3.0])],
        [None, Some([2.0, 3.0])],
    ]
    .into_iter()
    .enumerate()
    {
        let path = dir.0.join(format!("case-{index}.rfproj.json"));
        let mut app = renamed_reference(&path);
        let identity = prepare_object_identity_initialization(&app, &path).unwrap();
        dispatch_document_lifecycle(
            &mut app,
            StudioDocumentLifecycleCommand::InitializeObjectIdentities(Box::new(identity)),
        )
        .unwrap();
        let watched = VariableId {
            document: app.workspace.document.metadata.document_id.clone(),
            object: ObjectId::Unit("feed-b".into()),
            section: VariableSection::Inputs,
            field: VariableField::OutletTemperature,
        };
        app.workspace
            .edit_variable_watch_list(&VariableWatchCommand::Add(watched))
            .unwrap();
        let mut units = app.workspace.project_presentation.display_units().clone();
        units
            .set_unit(QuantityKind::AbsolutePressure, MeasurementUnit::Kilopascal)
            .unwrap();
        app.workspace
            .project_presentation
            .apply(ProjectPresentationCommand::Apply(units.clone()));

        let FlowPlanConfiguration::Mixed(mut options) = selected(initialization) else {
            unreachable!()
        };
        for (region, name) in regions_mut(&mut options)
            .iter_mut()
            .zip(["z-upstream", "a-downstream"])
        {
            region.tear.stream_id = name.into();
        }
        options.blocks.reverse();
        let selection = FlowPlanConfiguration::Mixed(options.canonicalized());
        select(&mut app, FlowPlanConfiguration::Mixed(options));
        let digest = runtime_configuration_sha256(&selection);
        let request = build_workspace_solve_request(&app, PACKAGE).unwrap();
        solve_workspace_with_property_package(&mut app, &provider, &request).unwrap();
        let first = verify_result(&app);
        assert_eq!(first.plan.as_ref().unwrap().configuration_sha256, digest);
        let draft = FlowPlanDraft::new(&app.workspace, selected([None, Some([8.0, 9.0])]));
        app.workspace.flow_plan.set_draft(draft.clone());
        let before = fs::read(&path).unwrap();
        let prepared = prepare_mixed_serial_recycle_save(&app, &path).unwrap();
        assert!(prepared.replaces_file());
        assert!(
            matches!(prepared.compatibility(), Some(ProjectSaveCompatibility::FlowCapabilities { added_from_source, .. })
            if added_from_source.iter().any(|cap| cap.id == project_v4::MIXED_SERIAL_RECYCLE_CAPABILITY))
        );
        assert!(prepared.commit(&mut app, false).is_err());
        assert_eq!(fs::read(&path).unwrap(), before);
        prepare_mixed_serial_recycle_save(&app, &path)
            .unwrap()
            .commit(&mut app, true)
            .unwrap();
        assert_eq!(verify_result(&app), first);
        assert_eq!(app.workspace.flow_plan.draft(), Some(&draft));

        let stored = fs::read_to_string(&path).unwrap();
        for key in [
            "\"snapshotHistory\"",
            "\"runId\"",
            "\"residualTrace\"",
            "\"draft\"",
            "\"planRevision\"",
        ] {
            assert!(!stored.contains(key), "{key}");
        }
        let mut reopened = reopen(&path);
        assert_eq!(reopened.workspace.document, app.workspace.document);
        assert_eq!(reopened.workspace.flow_plan.current(), &selection);
        assert_eq!(
            reopened.workspace.project_presentation.display_units(),
            &units
        );
        assert_eq!(
            reopened.workspace.current_object_identities(),
            app.workspace.current_object_identities()
        );
        assert_eq!(
            reopened.workspace.variable_watch_list.entries_for_save(),
            app.workspace.variable_watch_list.entries_for_save()
        );
        assert_eq!(
            runtime_configuration_sha256(reopened.workspace.flow_plan.current()),
            digest
        );
        assert!(
            !reopened
                .workspace
                .project_save_state()
                .has_unsaved_changes()
        );
        assert!(matches!(
            reopened.workspace.flow_result(),
            FlowResultView::Missing { .. }
        ));
        assert!(reopened.workspace.snapshot_history.is_empty());
        assert!(reopened.workspace.flow_plan.draft().is_none());
        reopened.set_simulation_mode(SimulationMode::Active);
        assert_eq!(
            solver
                .dispatch_with_property_package(
                    &mut reopened,
                    &provider,
                    PACKAGE,
                    WorkspaceSolveTrigger::Automatic
                )
                .unwrap(),
            WorkspaceSolveDispatch::Skipped(WorkspaceSolveSkipReason::ExplicitRunRequired)
        );
        assert!(matches!(
            reopened.workspace.flow_result(),
            FlowResultView::Missing { .. }
        ));
        solver
            .dispatch_with_property_package(
                &mut reopened,
                &provider,
                PACKAGE,
                WorkspaceSolveTrigger::Manual,
            )
            .unwrap();
        let second = verify_result(&reopened);
        assert_eq!(second.plan, first.plan);
        assert_ne!(second.run_id, first.run_id);
        let repeat = prepare_mixed_serial_recycle_save(&reopened, &path).unwrap();
        assert!(repeat.compatibility().is_none());
        repeat.commit(&mut reopened, false).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), stored);
    }
}

#[test]
fn m3_save_as_rehearses_conflicts_preserves_source_and_allows_a_new_lower_capability_copy() {
    let dir = Directory::new();
    let source = dir.0.join("source.rfproj.json");
    let target = dir.0.join("target.rfproj.json");
    fs::write(&source, REFERENCE).unwrap();
    let mut app = load_project_app_state(&source).unwrap();
    let plan = selected([None, None]);
    select(&mut app, plan.clone());

    let stale = prepare_mixed_serial_recycle_save(&app, &target).unwrap();
    select(&mut app, selected([None, Some([2.0, 3.0])]));
    select(&mut app, plan.clone());
    let before = app.workspace.clone();
    assert_eq!(
        stale
            .commit(&mut app, true)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("workspace.project.save_source_changed")
    );
    assert!(!target.exists());
    assert_eq!(app.workspace, before);

    let previous_session = prepare_mixed_serial_recycle_save(&app, &target).unwrap();
    let mut another_session = load_project_app_state(&source).unwrap();
    select(&mut another_session, plan.clone());
    assert_eq!(
        previous_session
            .commit(&mut another_session, true)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("workspace.project.save_source_changed")
    );
    assert!(!target.exists());

    fs::write(&target, REFERENCE).unwrap();
    let replaced = prepare_mixed_serial_recycle_save(&app, &target).unwrap();
    let removed = replaced.clone();
    let external = format!("{REFERENCE}\n");
    fs::write(&target, &external).unwrap();
    assert_eq!(
        replaced
            .commit(&mut app, true)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("store.project.save_target_changed")
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), external);
    fs::remove_file(&target).unwrap();
    assert_eq!(
        removed
            .commit(&mut app, true)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("store.project.save_target_changed")
    );

    let occupied = prepare_mixed_serial_recycle_save(&app, &target).unwrap();
    fs::write(&target, &external).unwrap();
    assert_eq!(
        occupied
            .commit(&mut app, true)
            .unwrap_err()
            .context()
            .diagnostic_code(),
        Some("store.project.save_target_changed")
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), external);

    let blocked_parent = dir.0.join("unavailable");
    let unavailable =
        prepare_mixed_serial_recycle_save(&app, &blocked_parent.join("target.rfproj.json"))
            .unwrap();
    fs::write(&blocked_parent, "outside writer").unwrap();
    assert!(unavailable.commit(&mut app, true).is_err());
    assert_eq!(
        fs::read_to_string(blocked_parent).unwrap(),
        "outside writer"
    );
    assert_eq!(app.workspace, before);
    assert_eq!(fs::read_to_string(&source).unwrap(), REFERENCE);

    let outcome = prepare_mixed_serial_recycle_save(&app, &target)
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    assert_eq!(outcome.action, StudioDocumentLifecycleAction::SaveAs);
    assert_eq!(fs::read_to_string(&source).unwrap(), REFERENCE);
    let saved = fs::read(&target).unwrap();
    let StudioProjectOpen::ReadOnly(old) = read_studio_project(&target).unwrap() else {
        panic!("default reader")
    };
    assert_eq!(old.original_json().as_bytes(), saved);

    let FlowPlanConfiguration::Mixed(mut options) = plan else {
        unreachable!()
    };
    let upstream = regions_mut(&mut options)[0].clone();
    options.recycle = Some(Box::new(MixedRecycleConfiguration::Single(upstream)));
    options
        .blocks
        .retain(|block| block.id.as_str() != "splitter-recycle-b");
    options
        .blocks
        .iter_mut()
        .find(|block| block.id.as_str() == "equations-b")
        .unwrap()
        .members
        .insert("splitter-recycle-b".into());
    let m2 = FlowPlanConfiguration::Mixed(options.canonicalized());
    m2.validate_for(&app.workspace.document.flowsheet).unwrap();
    select(&mut app, m2.clone());
    assert!(prepare_mixed_recycle_save(&app, &target).is_err());
    let copy = dir.0.join("m2.rfproj.json");
    prepare_mixed_recycle_save(&app, &copy)
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    let StudioProjectOpen::Editable(lower) =
        read_studio_project_with_support(&copy, FlowPlanSupport::MixedRecycle).unwrap()
    else {
        panic!("M2 reader")
    };
    assert_eq!(lower.workspace.flow_plan.current(), &m2);
    assert_eq!(fs::read(&target).unwrap(), saved);
}

#[test]
fn m3_reopen_keeps_stale_tear_identity_and_rejects_running_it() {
    let dir = Directory::new();
    let path = dir.0.join("invalid-binding.rfproj.json");
    let mut app = serial_case(500);
    let FlowPlanConfiguration::Mixed(mut options) = selected([None, None]) else {
        unreachable!()
    };
    regions_mut(&mut options)[0].tear.stream_id = "missing-upstream".into();
    let applied = FlowPlanConfiguration::Mixed(options.canonicalized());
    select(&mut app, applied.clone());
    prepare_mixed_serial_recycle_save(&app, &path)
        .unwrap()
        .commit(&mut app, true)
        .unwrap();
    let mut reopened = reopen(&path);
    assert_eq!(reopened.workspace.flow_plan.current(), &applied);
    assert!(
        reopened
            .workspace
            .stream_identity_reserved(&"missing-upstream".into())
    );
    let request = build_workspace_solve_request(&reopened, PACKAGE).unwrap();
    let Err(error) = prepare_workspace_solve(&mut reopened, &request) else {
        panic!("missing tear must block Ready");
    };
    assert_eq!(
        error.context().diagnostic_code(),
        Some("solver.mixed.missing_tear")
    );
    assert!(
        error
            .context()
            .related_stream_ids()
            .contains(&"missing-upstream".into())
    );
    assert!(reopened.workspace.snapshot_history.is_empty());
    assert!(matches!(
        reopened.workspace.flow_result(),
        FlowResultView::Missing { .. }
    ));
}
