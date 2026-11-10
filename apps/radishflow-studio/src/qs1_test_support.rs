//! Representative QS1 flow assembled exclusively through formal shared actions.
use crate::{
    flow_actions::*,
    flow_plan_actions::execute_flow_plan_action,
    modeling_actions::{MaterialConnectionTarget, MaterialPortTarget},
};
use rf_solver::partitioned::*;
use rf_types::{DiagnosticPortTarget, UnitId};
use rf_ui::{
    AppState, BuiltinUnitKind, DocumentMetadata, FlowsheetDocument, flow_plan::*,
    variable_browser::*,
};
use std::time::UNIX_EPOCH;
pub(crate) fn action(app: &mut AppState, action: FlowAction) -> FlowActionReceipt {
    execute_flow_action(
        app,
        FlowEditAccess::Editable,
        &FlowActionRequest {
            expected: FlowEditContext::capture(&app.workspace),
            action,
        },
        UNIX_EPOCH,
    )
    .unwrap()
}
pub(crate) fn write(app: &mut AppState, object: ObjectId, field: VariableField, value: f64) {
    action(
        app,
        FlowAction::Write {
            variable: VariableId {
                document: app.workspace.document.metadata.document_id.clone(),
                object,
                section: VariableSection::Inputs,
                field,
            },
            value: VariableValue::Number(value),
        },
    );
}
pub(crate) fn select(app: &mut AppState, plan: FlowPlanConfiguration) {
    let expected = FlowPlanEditContext::capture(&app.workspace);
    execute_flow_plan_action(
        app,
        FlowEditAccess::Editable,
        &expected,
        FlowPlanCommand::Apply(plan),
    )
    .unwrap();
}
pub(crate) fn recycle_case(alpha: f64) -> AppState {
    let mut flow = rf_model::Flowsheet::new("Fixed-T/P binary recycle");
    flow.set_property_package_id(Some(
        crate::test_support::OFFICIAL_BINARY_HYDROCARBON_PACKAGE_ID.into(),
    ))
    .unwrap();
    for (id, name) in crate::test_support::OFFICIAL_BINARY_HYDROCARBON_COMPONENT_SPECS {
        flow.insert_component(rf_model::Component::new(id, name))
            .unwrap();
    }
    let mut app = AppState::new(FlowsheetDocument::new(
        flow,
        DocumentMetadata::new("qs1-recycle", "QS1 binary recycle", UNIX_EPOCH),
    ));
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::Splitter,
        BuiltinUnitKind::Heater,
    ] {
        action(&mut app, FlowAction::CreateUnit(kind));
    }
    for (stream, source, out, sink, input) in [
        ("s-fa", "feed-1", "outlet", Some("mixer-1"), "inlet_a"),
        ("s-fb", "feed-2", "outlet", Some("mixer-1"), "inlet_b"),
        ("s-fresh", "mixer-1", "outlet", Some("mixer-2"), "inlet_a"),
        (
            "s-circulation",
            "mixer-2",
            "outlet",
            Some("splitter-1"),
            "inlet",
        ),
        (
            "s-recycle",
            "splitter-1",
            "outlet_a",
            Some("mixer-2"),
            "inlet_b",
        ),
        (
            "s-bleed",
            "splitter-1",
            "outlet_b",
            Some("heater-1"),
            "inlet",
        ),
        ("s-product", "heater-1", "outlet", None, ""),
    ] {
        action(
            &mut app,
            FlowAction::ConnectExact(MaterialConnectionTarget {
                stream_id: stream.into(),
                source: MaterialPortTarget {
                    unit_id: source.into(),
                    port: out.into(),
                },
                sink: sink.map(|id| MaterialPortTarget {
                    unit_id: id.into(),
                    port: input.into(),
                }),
            }),
        );
    }
    for id in ["feed-1", "feed-2", "heater-1"] {
        write(
            &mut app,
            ObjectId::Unit(id.into()),
            VariableField::OutletTemperature,
            300.0,
        );
    }
    for id in ["feed-1", "feed-2", "mixer-1", "mixer-2", "heater-1"] {
        write(
            &mut app,
            ObjectId::Unit(id.into()),
            VariableField::OutletPressure,
            650000.0,
        );
    }
    for (stream, flow, a, b) in [("s-fa", 2.0, 0.8, 0.2), ("s-fb", 3.0, 0.2, 0.8)] {
        for (field, value) in [
            (VariableField::MolarFlow, flow),
            (VariableField::MoleFraction("methane".into()), a),
            (VariableField::MoleFraction("ethane".into()), b),
        ] {
            write(&mut app, ObjectId::Stream(stream.into()), field, value);
        }
    }
    write(
        &mut app,
        ObjectId::Unit(UnitId::new("splitter-1")),
        VariableField::OutletASplitFraction,
        alpha,
    );
    select(
        &mut app,
        FlowPlanConfiguration::Partitioned(RecycleSolveOptions::new(
            FixedRecycleBoundary {
                temperature_k: 300.0,
                pressure_pa: 650000.0,
            },
            vec![TearSpecification {
                stream_id: "s-recycle".into(),
                source: DiagnosticPortTarget::new("splitter-1", "outlet_a"),
                sink: DiagnosticPortTarget::new("mixer-2", "inlet_b"),
                initial_component_flows_mol_s: None,
                component_scales_mol_s: None,
            }],
        )),
    );
    app
}
