use rf_ui::{AppState, flow_plan::FlowPlanConfiguration};
pub(crate) fn case() -> AppState {
    crate::load_project_app_state(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/flowsheets/fixed-tp-mixed-material-binary-hydrocarbon.rfproj.json"
    )))
    .unwrap()
}
pub(crate) fn explicit(app: &AppState) -> FlowPlanConfiguration {
    let FlowPlanConfiguration::Mixed(mut o) = app.workspace.flow_plan.current().clone() else {
        panic!()
    };
    let p = rf_solver::mixed::assemble(&app.workspace.document.flowsheet, &o).unwrap();
    for b in &mut o.blocks {
        if let rf_solver::mixed::MixedBlockMethod::EquationOriented { scales } = &mut b.method {
            let l = p.equation_layout(&b.id).unwrap();
            *scales = Some(rf_solver::equation_oriented::ExplicitEquationScales {
                variables: l.variables.into_iter().map(|b| (b, 2.0)).collect(),
                equations: l.equations.into_iter().map(|b| (b, 5.0)).collect(),
            });
        }
    }
    FlowPlanConfiguration::Mixed(o)
}
