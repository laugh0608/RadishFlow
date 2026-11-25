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

pub(crate) fn recycle_case(max_iteration_evaluations: usize) -> AppState {
    use rf_solver::{
        mixed::{MixedBlock, MixedBlockMethod, MixedRecycleOptions},
        partitioned::TearSpecification,
    };
    use rf_types::DiagnosticPortTarget;

    let mut app = case();
    let FlowPlanConfiguration::Mixed(mut options) = app.workspace.flow_plan.current().clone()
    else {
        panic!("M1 fixture must supply its original DAG plan");
    };
    options
        .blocks
        .iter_mut()
        .find(|block| block.id.as_str() == "equations")
        .unwrap()
        .members = ["mixer-1".into(), "splitter-1".into()].into();
    for id in ["mixer-2", "splitter-2"] {
        options.blocks.push(MixedBlock {
            id: id.into(),
            members: [id.into()].into(),
            method: MixedBlockMethod::Local,
        });
    }
    options.recycle = Some(Box::new(
        rf_solver::mixed::MixedRecycleConfiguration::Single(MixedRecycleOptions {
            tear: TearSpecification {
                stream_id: "s-rba".into(),
                source: DiagnosticPortTarget::new("splitter-2", "outlet_a"),
                sink: DiagnosticPortTarget::new("mixer-1", "inlet_b"),
                initial_component_flows_mol_s: None,
                component_scales_mol_s: None,
            },
            max_iteration_evaluations,
        }),
    ));
    crate::qs1_test_support::select(&mut app, FlowPlanConfiguration::Mixed(options));
    app
}

pub(crate) fn serial_case(upstream_budget: usize) -> AppState {
    use rf_solver::{
        mixed::{
            MixedBlock, MixedBlockId, MixedBlockMethod, MixedOptions, MixedRecycleConfiguration,
            MixedRecycleOptions,
        },
        partitioned::{FixedRecycleBoundary, TearSpecification},
    };
    use rf_types::DiagnosticPortTarget;

    let mut app = crate::load_project_app_state(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/flowsheets/fixed-tp-serial-recycle-eo-reference.rfproj.json"
    )))
    .unwrap();
    let mut blocks = [
        "feed-a",
        "feed-b",
        "mixer-makeup-b",
        "splitter-recycle-a",
        "splitter-recycle-b",
    ]
    .map(|id| MixedBlock {
        id: id.into(),
        members: [id.into()].into(),
        method: MixedBlockMethod::Local,
    })
    .to_vec();
    let [mut upstream, downstream] = ["a", "b"].map(|side| {
        let mixer = format!("mixer-{side}");
        blocks.push(MixedBlock {
            id: MixedBlockId::new(format!("equations-{side}")),
            members: [
                mixer.clone().into(),
                format!("splitter-primary-{side}").into(),
            ]
            .into(),
            method: MixedBlockMethod::EquationOriented { scales: None },
        });
        MixedRecycleOptions::new(TearSpecification {
            stream_id: format!("s-r{side}").into(),
            source: DiagnosticPortTarget::new(format!("splitter-recycle-{side}"), "outlet_a"),
            sink: DiagnosticPortTarget::new(mixer, "inlet_b"),
            initial_component_flows_mol_s: None,
            component_scales_mol_s: None,
        })
    });
    upstream.max_iteration_evaluations = upstream_budget;
    crate::qs1_test_support::select(
        &mut app,
        FlowPlanConfiguration::Mixed(MixedOptions {
            boundary: FixedRecycleBoundary {
                temperature_k: 300.0,
                pressure_pa: 650_000.0,
            },
            blocks,
            recycle: Some(Box::new(MixedRecycleConfiguration::Serial([
                upstream, downstream,
            ]))),
        }),
    );
    app
}
