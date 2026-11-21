use super::*;
use radishflow_studio::PropertyPackageCacheRoot;
use rf_solver::single_phase_recycle::{SinglePhaseRecycleResult, single_phase_enthalpy_flow_w};
use rf_store::project_v4::{
    self, FlowPlanSupport, ProjectV4Read, StoredFlowPlan, StoredSinglePhaseTearInitialization,
};
use rf_types::units::MeasurementUnit;
use rf_ui::{FlowResultRecord, result_query::FlowResultView};

const PROJECT: &str =
    include_str!("../../../../../../examples/flowsheets/two-feed-single-vapor-recycle.rfproj.json");

fn solve(app: &mut ReadyAppState, ctx: &egui::Context) -> std::sync::Arc<SinglePhaseRecycleResult> {
    app.dispatch_ui_command("run_panel.run_manual");
    drain(app, ctx);
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let FlowResultView::Current {
        snapshot: FlowResultRecord::SinglePhaseRecycle(snapshot),
        origin,
    } = workspace.flow_result()
    else {
        panic!(
            "expected current recycle result, got {:?} / {:?}",
            workspace.flow_result(),
            workspace.solve_session.latest_diagnostic
        );
    };
    assert!(origin.execution().unwrap().package.is_none());
    snapshot.result.clone()
}

#[test]
fn single_phase_recycle_gui_edits_saves_reopens_and_reviews_both_thermal_modes() {
    let temp = PropertyPackageCacheRoot::temporary("single-phase-recycle-shell").unwrap();
    let ctx = egui::Context::default();
    for (kind, temperature, duty) in [("heater", 380.0, 7800.0), ("cooler", 320.0, -4800.0)] {
        let path = temp.path().join(format!("{kind}.rfproj.json"));
        let mut project: serde_json::Value = serde_json::from_str(PROJECT).unwrap();
        let units = project["document"]["flowsheet"]["units"]
            .as_object_mut()
            .unwrap();
        let mut unit = units.remove("heater").unwrap();
        unit["id"] = kind.into();
        unit["kind"] = kind.into();
        unit["parameters"]["outlet_temperature_k"] = temperature.into();
        units.insert(kind.into(), unit);
        fs::write(&path, serde_json::to_vec_pretty(&project).unwrap()).unwrap();
        let mut config = synced_workspace_config();
        config.project_path = path.clone();
        let mut app = ready_app_state(&config);
        app.locale = StudioShellLocale::En;
        app.screen = StudioShellScreen::Run;
        app.enable_background_runs();
        assert!(app.read_only_project.is_none());

        let forward = solve(&mut app, &ctx);
        let text = frame(&mut app, &ctx, 400.0).join("\n");
        assert!(text.contains("Single-vapor recycle verification"), "{text}");
        assert!(text.contains("Overall energy residual"), "{text}");
        assert!(text.contains(&format!("{duty:+.6} W")), "{text}");
        let window = app.platform_host.snapshot().window_model();
        let snapshot = window.runtime.latest_solve_snapshot.unwrap();
        let units = snapshot.final_unit_results().unwrap();
        assert_eq!(units.len(), 6);
        assert_eq!(
            units
                .iter()
                .find(|u| u.unit_id == "recycle-mixer")
                .unwrap()
                .location,
            radishflow_studio::ResultExecutionLocation::RecycleVerification,
        );
        assert_eq!(
            snapshot.review_summary.terminal_stream_results[0].stream_id,
            "product"
        );

        click_text(&mut app, &ctx, "Clear outlet temperature", 0);
        assert!(matches!(
            app.platform_host.modeling_inputs().workspace.flow_result(),
            FlowResultView::Stale { .. }
        ));
        click_text(&mut app, &ctx, "Specify tear initial state", 0);
        click_text(&mut app, &ctx, "Specify heat duty", 0);
        let mut draft = app
            .platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .draft()
            .unwrap()
            .clone();
        for (field, value, unit) in [
            (
                FlowPlanNumericField::IterationBudget,
                120.0,
                MeasurementUnit::One,
            ),
            (
                FlowPlanNumericField::InitialComponent {
                    stream: "recycle".into(),
                    component: "component-a".into(),
                },
                3.0,
                MeasurementUnit::MolePerSecond,
            ),
            (
                FlowPlanNumericField::InitialComponent {
                    stream: "recycle".into(),
                    component: "component-b".into(),
                },
                4.5,
                MeasurementUnit::MolePerSecond,
            ),
            (
                FlowPlanNumericField::InitialEnthalpyFlow("recycle".into()),
                10000.0,
                MeasurementUnit::One,
            ),
            (
                FlowPlanNumericField::HeatDuty(kind.into()),
                duty,
                MeasurementUnit::One,
            ),
        ] {
            draft.edit_number(field, 0.0, unit).unwrap().raw = value.to_string();
        }
        app.background_action(Action::FlowPlanDraft {
            access: FlowEditAccess::Editable,
            draft: Some(draft),
        });
        click_text(&mut app, &ctx, "Apply plan", 0);
        let inverse = solve(&mut app, &ctx);
        app.save_project();
        if app.pending_format_upgrade.is_some() {
            app.confirm_format_upgrade();
        }
        let bytes = fs::read(&path).unwrap();
        let ProjectV4Read::Editable(project) =
            project_v4::read_with_support(&path, FlowPlanSupport::SinglePhaseRecycle).unwrap()
        else {
            panic!("expected editable v4 recycle project")
        };
        let StoredFlowPlan::SinglePhaseRecycle(options) = &project.steady_state_plan else {
            panic!("expected saved recycle plan")
        };
        assert_eq!(options.max_iteration_evaluations, 120);
        assert_eq!(options.single_phase.heat_duties_w[&kind.into()], duty);
        assert_eq!(
            options.tear.initialization,
            StoredSinglePhaseTearInitialization::ComponentFlowsAndEnthalpy {
                component_flows_mol_s: [("component-a".into(), 3.0), ("component-b".into(), 4.5)]
                    .into(),
                enthalpy_flow_w: 10000.0,
            }
        );
        app.open_project(path.clone(), "sensible recycle");
        assert!(matches!(
            app.platform_host.modeling_inputs().workspace.flow_result(),
            FlowResultView::Missing { .. }
        ));
        let reopened = solve(&mut app, &ctx);
        assert_eq!(reopened, inverse);
        for id in ["product", "recycle"] {
            let id = id.into();
            let stream = &reopened.streams[&id];
            assert!((stream.temperature_k - temperature).abs() <= 1e-8);
            assert!(
                (single_phase_enthalpy_flow_w(stream).unwrap()
                    - single_phase_enthalpy_flow_w(&forward.streams[&id]).unwrap())
                .abs()
                    <= 1e-6
            );
        }
        let residual = &reopened.verification.residual;
        assert!(residual.material.max_absolute_mol_s <= 1e-10);
        assert!(residual.enthalpy_flow_residual_w.abs() <= 1e-6);
        assert!(reopened.overall_energy_balance.residual_w.abs() <= 1e-6);
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}
