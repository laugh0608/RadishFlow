use super::*;
use rf_model::MaterialStreamState;
use rf_unitops::{
    build_feed_node, build_flash_drum_node, build_heater_node, build_mixer_node, build_valve_node,
};

fn model() -> Flowsheet {
    let mut model = Flowsheet::new("pressure");
    for (id, pressure) in [("a", 150_000.0), ("b", 120_000.0), ("out", 101_325.0)] {
        model
            .insert_stream(MaterialStreamState::from_tpzf(
                id,
                id,
                300.0,
                pressure,
                1.0,
                Default::default(),
            ))
            .unwrap();
    }
    model
        .insert_unit(build_feed_node("feed-a", "A", "a"))
        .unwrap();
    model
        .insert_unit(build_feed_node("feed-b", "B", "b"))
        .unwrap();
    model
        .insert_unit(build_mixer_node("mixer", "M", "a", "b", "out"))
        .unwrap();
    model
}

#[test]
fn complete_mixer_uses_both_feed_sources_and_switches_limiting_source() {
    let mut model = model();
    let id = UnitId::new("mixer");
    let context = document_pressure_context(&model, &model.units[&id], BuiltinUnitKind::Mixer);
    assert_eq!(context.inputs.len(), 2);
    assert_eq!(context.limit().unwrap().pressure_pa, 120_000.0);
    assert_eq!(context.limit().unwrap().limiting_stream_id.as_str(), "b");
    model.streams.get_mut(&"a".into()).unwrap().pressure_pa = 110_000.0;
    let context = document_pressure_context(&model, &model.units[&id], BuiltinUnitKind::Mixer);
    assert_eq!(context.limit().unwrap().pressure_pa, 110_000.0);
    assert_eq!(
        context.inputs[0].source,
        DocumentPressureSource::FeedStream {
            unit_id: "feed-a".into(),
            stream_id: "a".into()
        }
    );
}

#[test]
fn missing_mixer_source_does_not_use_the_remaining_stream_as_a_complete_limit() {
    let mut model = model();
    model.units.remove(&"feed-b".into());
    let context = document_pressure_context(
        &model,
        &model.units[&"mixer".into()],
        BuiltinUnitKind::Mixer,
    );
    assert_eq!(context.inputs.len(), 1);
    assert_eq!(context.limit(), None);
    assert_eq!(
        context.gaps[0],
        PressureContextGap {
            inlet_port: "inlet_b",
            reason: PressureContextGapReason::MissingSource("b".into())
        }
    );
}

#[test]
fn explicit_feed_pressure_overrides_a_conflicting_stream_without_fallback() {
    let mut model = model();
    model
        .units
        .get_mut(&"feed-b".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(160_000.0);
    let context = document_pressure_context(
        &model,
        &model.units[&"mixer".into()],
        BuiltinUnitKind::Mixer,
    );
    assert_eq!(context.limit().unwrap().pressure_pa, 150_000.0);
    assert_eq!(context.inputs[1].input.pressure_pa, 160_000.0);
    assert_eq!(
        context.inputs[1].source,
        DocumentPressureSource::UnitParameter {
            unit_id: "feed-b".into()
        }
    );
    model
        .units
        .get_mut(&"feed-b".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(-1.0);
    let context = document_pressure_context(
        &model,
        &model.units[&"mixer".into()],
        BuiltinUnitKind::Mixer,
    );
    assert_eq!(context.limit(), None);
    assert!(matches!(
        context.gaps[0].reason,
        PressureContextGapReason::InvalidPressure(_)
    ));
}

#[test]
fn intermediate_templates_cannot_supply_a_pressure_specification() {
    let mut model = model();
    model.units.remove(&"feed-b".into());
    model
        .insert_unit(build_heater_node("heater", "H", "a", "b"))
        .unwrap();
    let context = document_pressure_context(
        &model,
        &model.units[&"mixer".into()],
        BuiltinUnitKind::Mixer,
    );
    assert_eq!(context.limit(), None);
    assert_eq!(
        context.gaps[0].reason,
        PressureContextGapReason::MissingExplicitPressure("heater".into())
    );
    model
        .units
        .get_mut(&"heater".into())
        .unwrap()
        .parameters
        .outlet_pressure_pa = Some(90_000.0);
    let context = document_pressure_context(
        &model,
        &model.units[&"mixer".into()],
        BuiltinUnitKind::Mixer,
    );
    assert_eq!(context.limit().unwrap().pressure_pa, 90_000.0);
    assert_eq!(model.streams[&"b".into()].pressure_pa, 120_000.0);
}

#[test]
fn either_flash_outlet_gets_the_specified_pressure_without_solving() {
    let mut model = model();
    model
        .insert_stream(MaterialStreamState::new("liquid", "L"))
        .unwrap();
    model
        .insert_stream(MaterialStreamState::new("vapor", "V"))
        .unwrap();
    let mut flash = build_flash_drum_node("flash", "Flash", "a", "liquid", "vapor");
    flash.parameters.outlet_pressure_pa = Some(200_000.0);
    model.insert_unit(flash).unwrap();
    for stream in ["liquid", "vapor"] {
        let valve = build_valve_node("valve", "Valve", stream, "out");
        let context = document_pressure_context(&model, &valve, BuiltinUnitKind::Valve);
        assert_eq!(context.limit().unwrap().pressure_pa, 200_000.0);
        assert_eq!(
            context.inputs[0].source,
            DocumentPressureSource::UnitParameter {
                unit_id: "flash".into()
            }
        );
    }
}

#[test]
fn malformed_and_ambiguous_sources_produce_explicit_context_gaps() {
    let mut model = model();
    model
        .insert_unit(build_feed_node("duplicate", "duplicate", "a"))
        .unwrap();
    let context = document_pressure_context(
        &model,
        &model.units[&"mixer".into()],
        BuiltinUnitKind::Mixer,
    );
    assert_eq!(context.limit(), None);
    assert_eq!(
        context.gaps[0].reason,
        PressureContextGapReason::AmbiguousSource("a".into())
    );
    model.units.remove(&"duplicate".into());
    model.units.get_mut(&"feed-a".into()).unwrap().ports[0].name = "wrong".into();
    let context = document_pressure_context(
        &model,
        &model.units[&"mixer".into()],
        BuiltinUnitKind::Mixer,
    );
    assert_eq!(context.limit(), None);
    assert_eq!(
        context.gaps[0].reason,
        PressureContextGapReason::InvalidSourcePorts("feed-a".into())
    );
}
