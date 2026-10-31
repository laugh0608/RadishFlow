//! QS2 cross-recycle constructed through the same formal actions as consumers.
use crate::qs1_test_support::{action, select, write};
use crate::{
    flow_actions::FlowAction,
    modeling_actions::{MaterialConnectionTarget, MaterialPortTarget},
};
use rf_solver::simultaneous::{FixedBoundary, ModularOptions};
use rf_ui::{
    AppState, BuiltinUnitKind, DocumentMetadata, FlowsheetDocument,
    flow_plan::FlowPlanConfiguration, variable_browser::*,
};
use std::time::UNIX_EPOCH;

pub(crate) fn plan() -> FlowPlanConfiguration {
    FlowPlanConfiguration::Simultaneous(ModularOptions::new(FixedBoundary {
        temperature_k: 300.0,
        pressure_pa: 650_000.0,
    }))
}

pub(crate) fn cross_case(alpha: f64, beta: f64) -> AppState {
    let mut flow = rf_model::Flowsheet::new("QS2 cross recycle");
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
        DocumentMetadata::new("qs2-cross", "QS2 fixed-T/P cross recycle", UNIX_EPOCH),
    ));
    for kind in [
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Feed,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::Mixer,
        BuiltinUnitKind::Splitter,
        BuiltinUnitKind::Splitter,
    ] {
        action(&mut app, FlowAction::CreateUnit(kind));
    }
    for (stream, source, output, sink, input) in [
        ("s-fa", "feed-1", "outlet", Some("mixer-1"), "inlet_a"),
        ("s-fb", "feed-2", "outlet", Some("mixer-2"), "inlet_a"),
        ("s-ma", "mixer-1", "outlet", Some("splitter-1"), "inlet"),
        ("s-mb", "mixer-2", "outlet", Some("splitter-2"), "inlet"),
        (
            "s-rab",
            "splitter-1",
            "outlet_a",
            Some("mixer-2"),
            "inlet_b",
        ),
        (
            "s-rba",
            "splitter-2",
            "outlet_a",
            Some("mixer-1"),
            "inlet_b",
        ),
        ("s-pa", "splitter-1", "outlet_b", None, ""),
        ("s-pb", "splitter-2", "outlet_b", None, ""),
    ] {
        action(
            &mut app,
            FlowAction::ConnectExact(MaterialConnectionTarget {
                stream_id: stream.into(),
                source: MaterialPortTarget {
                    unit_id: source.into(),
                    port: output.into(),
                },
                sink: sink.map(|id| MaterialPortTarget {
                    unit_id: id.into(),
                    port: input.into(),
                }),
            }),
        );
    }
    for id in ["feed-1", "feed-2"] {
        write(
            &mut app,
            ObjectId::Unit(id.into()),
            VariableField::OutletTemperature,
            300.0,
        );
    }
    for id in ["feed-1", "feed-2", "mixer-1", "mixer-2"] {
        write(
            &mut app,
            ObjectId::Unit(id.into()),
            VariableField::OutletPressure,
            650_000.0,
        );
    }
    for (id, f, a, b) in [("s-fa", 2.0, 0.8, 0.2), ("s-fb", 3.0, 0.2, 0.8)] {
        for (field, value) in [
            (VariableField::MolarFlow, f),
            (VariableField::MoleFraction("methane".into()), a),
            (VariableField::MoleFraction("ethane".into()), b),
        ] {
            write(&mut app, ObjectId::Stream(id.into()), field, value);
        }
    }
    for (id, value) in [("splitter-1", alpha), ("splitter-2", beta)] {
        write(
            &mut app,
            ObjectId::Unit(id.into()),
            VariableField::OutletASplitFraction,
            value,
        );
    }
    select(&mut app, plan());
    app
}

pub(crate) fn explicit_plan(app: &AppState) -> FlowPlanConfiguration {
    let FlowPlanConfiguration::Simultaneous(mut options) = plan() else {
        unreachable!()
    };
    let assembled =
        rf_solver::simultaneous::assemble(&app.workspace.document.flowsheet, &options).unwrap();
    options.scales = Some(
        assembled
            .blocks()
            .iter()
            .flat_map(|b| &b.couplings)
            .cloned()
            .map(|b| {
                (
                    b,
                    rf_solver::simultaneous::CouplingScale {
                        variable_mol_s: 2.0,
                        residual_mol_s: 5.0,
                    },
                )
            })
            .collect(),
    );
    FlowPlanConfiguration::Simultaneous(options)
}
