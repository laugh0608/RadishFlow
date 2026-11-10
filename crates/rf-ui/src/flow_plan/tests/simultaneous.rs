use super::*;
use rf_solver::simultaneous::{CouplingBinding, CouplingScale, ModularOptions};
use rf_types::{DiagnosticPortTarget, units::MeasurementUnit};

fn modular() -> FlowPlanConfiguration {
    FlowPlanConfiguration::Simultaneous(ModularOptions::new(FixedRecycleBoundary {
        temperature_k: 300.0,
        pressure_pa: 650_000.0,
    }))
}
fn binding() -> CouplingBinding {
    CouplingBinding {
        stream: "old-stream".into(),
        source: DiagnosticPortTarget::new("old-splitter", "outlet_a"),
        sink: DiagnosticPortTarget::new("old-mixer", "inlet_b"),
        component: "methane".into(),
    }
}

#[test]
fn modular_plan_keeps_invalid_graph_bindings_but_rejects_invalid_settings() {
    let mut app = app();
    let FlowPlanConfiguration::Simultaneous(mut options) = modular() else {
        unreachable!()
    };
    options.scales = Some(
        [(
            binding(),
            CouplingScale {
                variable_mol_s: 2.0,
                residual_mol_s: 5.0,
            },
        )]
        .into(),
    );
    let plan = FlowPlanConfiguration::Simultaneous(options.clone());
    assert!(plan.validate_configuration().is_ok());
    assert!(plan.validate_for(&app.workspace).is_err());
    let input = app.flow_run_qualification();
    apply(&mut app, FlowPlanCommand::Apply(plan.clone()));
    assert!(input.same_input(&app.flow_run_qualification()));
    assert!(plan.referenced_unit_ids().contains(&"old-splitter".into()));
    assert!(plan.referenced_stream_ids().contains(&"old-stream".into()));
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        options
            .scales
            .as_mut()
            .unwrap()
            .get_mut(&binding())
            .unwrap()
            .residual_mol_s = value;
        let before = app.workspace.flow_plan.clone();
        assert!(
            app.workspace
                .apply_flow_plan(
                    &FlowPlanEditContext::capture(&app.workspace),
                    FlowPlanCommand::Apply(FlowPlanConfiguration::Simultaneous(options.clone()))
                )
                .is_err()
        );
        assert_eq!(before, app.workspace.flow_plan);
    }
    let context = FlowPlanEditContext::capture(&app.workspace);
    apply(&mut app, FlowPlanCommand::Apply(modular()));
    apply(&mut app, FlowPlanCommand::Undo);
    assert_eq!(app.workspace.flow_plan.current(), &plan);
    assert!(!context.is_current(&app.workspace));
}

#[test]
fn modular_draft_preserves_incomplete_text_units_and_separate_coupling_scales() {
    let mut app = app();
    let FlowPlanConfiguration::Simultaneous(mut options) = modular() else {
        unreachable!()
    };
    options.scales = Some(
        [(
            binding(),
            CouplingScale {
                variable_mol_s: 2.0,
                residual_mol_s: 5.0,
            },
        )]
        .into(),
    );
    let plan = FlowPlanConfiguration::Simultaneous(options);
    apply(&mut app, FlowPlanCommand::Apply(plan.clone()));
    let mut draft = FlowPlanDraft::new(&app.workspace, plan.clone());
    let field = FlowPlanNumericField::CouplingVariableScale(binding());
    draft
        .edit_number(field.clone(), 2.0, MeasurementUnit::MolePerSecond)
        .unwrap()
        .raw = "1e-".into();
    assert!(draft.configuration().is_err());
    draft.replace_candidate(plan.clone());
    assert_eq!(draft.numbers()[&field].raw, "1e-");
    draft
        .edit_number(field.clone(), 2.0, MeasurementUnit::MolePerSecond)
        .unwrap()
        .raw = "3".into();
    let FlowPlanConfiguration::Simultaneous(candidate) = draft.configuration().unwrap() else {
        unreachable!()
    };
    assert_eq!(
        candidate.scales.unwrap()[&binding()],
        CouplingScale {
            variable_mol_s: 3.0,
            residual_mol_s: 5.0
        }
    );
    app.workspace.flow_plan.set_draft(draft.clone());
    app.workspace.flow_plan.record_saved(plan);
    assert_eq!(app.workspace.flow_plan.draft(), Some(&draft));
}
