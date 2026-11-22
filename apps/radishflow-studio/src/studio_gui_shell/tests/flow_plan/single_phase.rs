use super::*;
use radishflow_studio::PropertyPackageCacheRoot;
use rf_types::units::MeasurementUnit;
use rf_ui::result_query::FlowResultView;

const PROJECT: &str = include_str!(
    "../../../../../../examples/flowsheets/two-feed-single-vapor-sensible.rfproj.json"
);

fn verify(app: &ReadyAppState, unit: &str, temperature: f64, expected_duty: f64) {
    let workspace = &app.platform_host.modeling_inputs().workspace;
    let FlowResultView::Current { snapshot, origin } = workspace.flow_result() else {
        panic!(
            "expected current energy balance, got {:?} and {:?}",
            workspace.flow_result(),
            workspace.solve_session.latest_diagnostic
        );
    };
    assert!(origin.execution().unwrap().package.is_none());
    let snapshot = snapshot.acyclic().unwrap();
    let product = snapshot
        .streams
        .iter()
        .find(|s| s.stream_id.as_str() == "product")
        .unwrap();
    assert!((product.temperature_k - temperature).abs() <= 1e-8);
    let balance = snapshot.energy_balances[&unit.into()];
    assert!((balance.heat_duty_w - expected_duty).abs() <= 1e-6);
    assert!(balance.residual_w.abs() <= 1e-6);
    assert!(balance.scaled_residual.abs() <= 1e-10);
}

#[test]
fn single_vapor_gui_runs_edits_saves_reopens_and_reviews_energy_balance() {
    let temp = PropertyPackageCacheRoot::temporary("single-vapor-shell").unwrap();
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
        let view = app.platform_host.snapshot().window_model();
        assert!(
            view.runtime
                .workspace_document
                .project_component_choices
                .iter()
                .filter(|c| c.selected)
                .map(|c| c.component_id.as_str())
                .eq(["component-a", "component-b"])
        );

        app.dispatch_ui_command("run_panel.run_manual");
        drain(&mut app, &ctx);
        verify(&app, kind, temperature, duty);
        let text = frame(&mut app, &ctx, 400.0).join("\n");
        assert!(text.contains("Energy residual"), "{text}");
        assert!(text.contains(&format!("{duty:+.6} W")), "{text}");

        click_text(&mut app, &ctx, "Clear outlet temperature", 0);
        let workspace = &app.platform_host.modeling_inputs().workspace;
        assert!(
            workspace.document.flowsheet.units[&kind.into()]
                .parameters
                .outlet_temperature_k
                .is_none()
        );
        assert!(matches!(
            workspace.flow_result(),
            FlowResultView::Stale { .. }
        ));
        click_text(&mut app, &ctx, "Specify heat duty", 0);
        let mut draft = app
            .platform_host
            .modeling_inputs()
            .workspace
            .flow_plan
            .draft()
            .unwrap()
            .clone();
        draft
            .edit_number(
                FlowPlanNumericField::HeatDuty(kind.into()),
                0.0,
                MeasurementUnit::One,
            )
            .unwrap()
            .raw = duty.to_string();
        app.background_action(Action::FlowPlanDraft {
            access: FlowEditAccess::Editable,
            draft: Some(draft),
        });
        click_text(&mut app, &ctx, "Apply plan", 0);

        app.save_project();
        if app.pending_format_upgrade.is_some() {
            app.confirm_format_upgrade();
        }
        let saved = fs::read(&path).unwrap();
        let rf_store::project_v4::ProjectV4Read::Editable(stored) =
            rf_store::project_v4::read_with_support(
                &path,
                rf_store::project_v4::FlowPlanSupport::SinglePhase,
            )
            .unwrap()
        else {
            panic!("expected saved plan")
        };
        let rf_store::project_v4::StoredFlowPlan::SinglePhase(plan) = &stored.steady_state_plan
        else {
            panic!("expected single-vapor inputs")
        };
        assert_eq!(plan.heat_duties_w[&kind.into()], duty);
        app.open_project(path.clone(), "single vapor");
        let workspace = &app.platform_host.modeling_inputs().workspace;
        assert!(matches!(
            workspace.flow_result(),
            FlowResultView::Missing { .. }
        ));
        app.dispatch_ui_command("run_panel.run_manual");
        drain(&mut app, &ctx);
        verify(&app, kind, temperature, duty);
        assert_eq!(fs::read(&path).unwrap(), saved);
    }
}
