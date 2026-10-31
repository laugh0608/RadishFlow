use rf_solver::equation_oriented::*;
use rf_ui::{AppState, flow_plan::FlowPlanConfiguration};
pub(crate) fn options(inverse: bool) -> EquationOptions {
    let mut o = EquationOptions::new(FixedBoundary {
        temperature_k: 300.0,
        pressure_pa: 650000.0,
    });
    if inverse {
        o.released = Some(ReleasedSplitFraction {
            binding: ParameterBinding {
                unit: "splitter-1".into(),
                field: ParameterField::OutletASplitFraction,
            },
            lower: 0.0,
            upper: 0.95,
            max_iterations: 64,
        });
        o.target = Some(ProductTotalTarget {
            source: PortBinding {
                unit: "splitter-1".into(),
                port: MaterialPort::OutletB,
                stream: "s-pa".into(),
            },
            value_mol_s: 1.5,
        });
    }
    o
}
pub(crate) fn case(inverse: bool) -> AppState {
    let mut app = crate::qs2_test_support::cross_case(0.6, 0.25);
    crate::qs1_test_support::select(
        &mut app,
        FlowPlanConfiguration::EquationOriented(options(inverse)),
    );
    app
}
pub(crate) fn explicit(app: &AppState, inverse: bool) -> FlowPlanConfiguration {
    let mut o = options(inverse);
    let p = assemble(&app.workspace.document.flowsheet, &o).unwrap();
    o.scales = Some(ExplicitEquationScales {
        variables: p.variables().iter().cloned().map(|b| (b, 2.0)).collect(),
        equations: p.equations().iter().cloned().map(|b| (b, 5.0)).collect(),
    });
    FlowPlanConfiguration::EquationOriented(o)
}
