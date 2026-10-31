use super::*;
use radishflow_studio::{
    StudioAppAuthCacheContext, StudioAppFacade, StudioAppResultDispatch, StudioWorkspaceRunOutcome,
    WorkspaceControlAction, WorkspaceRunPackageSelection,
    dispatch_workspace_control_action_with_auth_cache,
};

#[test]
fn a1_shared_checks_preserve_gui_and_both_cli_run_contracts() {
    let f = Fixture::new();
    let path = f.root.join("工程 example.rfproj.json");
    let original = rf_store::read_project_file(&path).unwrap();
    // Expected stages and codes are independent of the shared check implementation.
    for (case, exit, status, diagnostic) in [
        ("normal", 0, "ok", Some("solver.execution_order")),
        ("missing", 3, "blocked", None),
        ("composition", 3, "blocked", None),
        (
            "unbound",
            4,
            "failed",
            Some("solver.connection_validation.unbound_inlet_port"),
        ),
        ("pressure", 4, "failed", Some("solver.step.parameter")),
        ("effective_feed", 0, "ok", Some("solver.execution_order")),
        (
            "invalid_override",
            4,
            "failed",
            Some("solver.step.parameter"),
        ),
    ] {
        let mut project = original.clone();
        let model = &mut project.document.flowsheet;
        match case {
            "missing" => {
                model
                    .units
                    .get_mut(&"heater-1".into())
                    .unwrap()
                    .parameters
                    .outlet_temperature_k = None
            }
            "composition" => {
                model
                    .streams
                    .get_mut(&"stream-feed".into())
                    .unwrap()
                    .overall_mole_fractions
                    .insert("methane".into(), 0.2);
            }
            "unbound" => {
                let heater = model.units.get_mut(&"heater-1".into()).unwrap();
                heater.ports[0].stream_id = None;
                // Topology still wins over authoring incompleteness.
                heater.parameters.outlet_temperature_k = None;
            }
            "pressure" => {
                model
                    .units
                    .get_mut(&"heater-1".into())
                    .unwrap()
                    .parameters
                    .outlet_pressure_pa = Some(120_001.0)
            }
            "effective_feed" => {
                let stream = model.streams.get_mut(&"stream-feed".into()).unwrap();
                stream.temperature_k = -1.0;
                stream.pressure_pa = -1.0;
                let params = &mut model.units.get_mut(&"feed-1".into()).unwrap().parameters;
                params.outlet_temperature_k = Some(300.0);
                params.outlet_pressure_pa = Some(120_000.0);
            }
            "invalid_override" => {
                model
                    .units
                    .get_mut(&"feed-1".into())
                    .unwrap()
                    .parameters
                    .outlet_pressure_pa = Some(-1.0)
            }
            "normal" => {}
            _ => unreachable!(),
        }
        rf_store::write_project_file(&path, &project).unwrap();
        let before = fs::read(&path).unwrap();
        let mut v1 = f.request();
        v1["writes"] = json!([]);
        v1["reads"] = json!([]);
        let mut v2 = v1.clone();
        v2["schema_version"] = json!(2);
        v2.as_object_mut().unwrap().remove("writes");
        v2["steps"] = json!([]);
        let (code1, result1) = f.run(&v1);
        let (code2, result2) = f.run(&v2);
        assert_eq!(
            (code1, code2),
            (exit, exit),
            "{case}: {result1} / {result2}"
        );
        for result in [&result1, &result2] {
            assert_eq!(result["status"], status, "{case}: {result}");
            assert_eq!(
                result["diagnostic"]["code"].as_str(),
                diagnostic,
                "{case}: {result}"
            );
            assert_eq!(result["document"]["revision"], 0);
            assert!(result.get("specifications").is_none());
        }
        assert_eq!(result1["diagnostic"], result2["diagnostic"], "{case}");
        assert_eq!(result1["error"]["code"], result2["error"]["code"], "{case}");
        let mut gui = load_project_app_state(&path).unwrap();
        let index = rf_store::read_auth_cache_index(f.root.join("index.json")).unwrap();
        let cache = f.root.join("cache");
        let outcome = dispatch_workspace_control_action_with_auth_cache(
            &StudioAppFacade::new(),
            &mut gui,
            &StudioAppAuthCacheContext::new(&cache, &index),
            &WorkspaceControlAction::run_manual(WorkspaceRunPackageSelection::Preferred),
        )
        .unwrap();
        let StudioAppResultDispatch::WorkspaceRun(run) = outcome.dispatch else {
            panic!("expected run")
        };
        let gui_status = match run.outcome {
            StudioWorkspaceRunOutcome::Started(_) => "ok",
            StudioWorkspaceRunOutcome::Blocked(_) => "blocked",
            StudioWorkspaceRunOutcome::Failed(_) => "failed",
            StudioWorkspaceRunOutcome::Skipped(_) => panic!("unexpected skip"),
            StudioWorkspaceRunOutcome::Background(_) => {
                panic!("headless comparison requires inline execution")
            }
        };
        assert_eq!(gui_status, status, "{case}");
        if let Some(code) = diagnostic {
            let summary = gui
                .workspace
                .solve_session
                .latest_diagnostic
                .as_ref()
                .unwrap();
            assert_eq!(summary.primary_code.as_deref(), Some(code));
            let units: Vec<_> = summary
                .related_unit_ids
                .iter()
                .map(|id| json!(id.as_str()))
                .collect();
            assert_eq!(json!(units), result1["diagnostic"]["unit_ids"]);
        }
        assert_eq!(fs::read(&path).unwrap(), before);
    }
}
