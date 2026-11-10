use std::collections::BTreeSet;

use rf_flash::{PlaceholderTpFlashSolver, TpFlashInput, TpFlashResult, TpFlashSolver};
use rf_model::UnitOperationParameters;
use rf_thermo::ThermoProvider;
use rf_types::{ComponentId, PortDirection};

use super::*;
use crate::{
    UnitOperationInputs, UnitOperationServices, build_cooler_node, build_feed_node,
    build_flash_drum_node, build_heater_node, build_mixer_node, build_valve_node,
    builtin_unit_spec, validate_unit_node,
};

fn fixture(
    kind: BuiltinUnitKind,
) -> (
    UnitNode,
    BTreeMap<StreamId, MaterialStreamState>,
    UnitOperationInputs,
) {
    let mut unit = match kind {
        BuiltinUnitKind::Splitter => {
            crate::build_splitter_node("unit-1", "Splitter", "inlet", "liquid", "vapor")
        }
        BuiltinUnitKind::Feed => build_feed_node("unit-1", "Feed", "outlet"),
        BuiltinUnitKind::Mixer => build_mixer_node("unit-1", "Mixer", "inlet", "inlet-b", "outlet"),
        BuiltinUnitKind::Heater => build_heater_node("unit-1", "Heater", "inlet", "outlet"),
        BuiltinUnitKind::Cooler => build_cooler_node("unit-1", "Cooler", "inlet", "outlet"),
        BuiltinUnitKind::Valve => build_valve_node("unit-1", "Valve", "inlet", "outlet"),
        BuiltinUnitKind::FlashDrum => {
            build_flash_drum_node("unit-1", "Flash", "inlet", "liquid", "vapor")
        }
    };
    if kind == BuiltinUnitKind::Splitter {
        unit.parameters.outlet_a_split_fraction = Some(0.6);
    }
    let streams = ["inlet", "inlet-b", "outlet", "liquid", "vapor"]
        .into_iter()
        .map(|id| {
            let stream = MaterialStreamState::from_tpzf(
                id,
                id,
                300.0,
                100_000.0,
                1.0,
                [
                    (ComponentId::new("component-a"), 0.5),
                    (ComponentId::new("component-b"), 0.5),
                ]
                .into(),
            );
            (stream.id.clone(), stream)
        })
        .collect::<BTreeMap<_, _>>();
    let mut inputs = UnitOperationInputs::new();
    for port in unit
        .ports
        .iter()
        .filter(|port| port.direction == PortDirection::Inlet)
    {
        inputs.insert_material_stream(
            &port.name,
            streams[port.stream_id.as_ref().unwrap()].clone(),
        );
    }
    (unit, streams, inputs)
}

#[test]
fn catalog_covers_core_models_and_canonical_specifications() {
    let names = builtin_models()
        .iter()
        .map(|model| model.spec.kind.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        names,
        [
            "feed",
            "mixer",
            "heater",
            "cooler",
            "valve",
            "flash_drum",
            "splitter"
        ]
        .into()
    );
    assert_eq!(builtin_models().len(), names.len());
    for model in builtin_models() {
        let kind = model.spec.kind;
        assert!(std::ptr::eq(builtin_model(kind), model));
        assert_eq!(model.input_specification, input_specification(kind));
        let (unit, streams, _) = fixture(kind);
        assert_eq!(validate_unit_node(&unit).unwrap(), model.spec);
        let operation = instantiate_builtin(&unit, &streams).unwrap();
        assert_eq!(operation.kind(), kind);
        assert!(std::ptr::eq(operation.spec(), builtin_unit_spec(kind)));
    }
    assert_eq!(BuiltinUnitKind::parse("flash"), None); // Canvas alias is not a persisted kind.
}

#[test]
fn declared_service_requirements_match_real_execution_for_all_models() {
    let provider = crate::tests::build_provider([2.0, 0.5], 100_000.0);
    let flash = PlaceholderTpFlashSolver;
    for model in builtin_models() {
        let (unit, streams, inputs) = fixture(model.spec.kind);
        let operation = instantiate_builtin(&unit, &streams).unwrap();
        for thermo in [None, Some(&provider as &dyn ThermoProvider)] {
            for flash_solver in [None, Some(&flash as &dyn TpFlashSolver)] {
                let result = operation.run(
                    &UnitOperationServices {
                        thermo,
                        flash_solver,
                    },
                    &inputs,
                );
                let missing_required = (thermo.is_none() && model.services.thermo == Required)
                    || (flash_solver.is_none() && model.services.tp_flash == Required);
                assert_eq!(
                    result.is_err(),
                    missing_required,
                    "{}: {thermo_present}/{flash_present}",
                    unit.kind,
                    thermo_present = thermo.is_some(),
                    flash_present = flash_solver.is_some()
                );
                if let Ok(outputs) = result {
                    for port in model
                        .spec
                        .ports
                        .iter()
                        .filter(|port| port.direction == PortDirection::Outlet)
                    {
                        assert!(
                            outputs.stream(port.name).is_some(),
                            "{}: {}",
                            unit.kind,
                            port.name
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn supplied_optional_flash_failures_are_not_silently_ignored() {
    struct FailingFlash;
    impl TpFlashSolver for FailingFlash {
        fn flash(&self, _: &dyn ThermoProvider, _: &TpFlashInput) -> RfResult<TpFlashResult> {
            Err(RfError::thermo("test flash failure"))
        }
    }
    let provider = crate::tests::build_provider([2.0, 0.5], 100_000.0);
    let services = UnitOperationServices {
        thermo: Some(&provider),
        flash_solver: Some(&FailingFlash),
    };
    for model in builtin_models()
        .iter()
        .filter(|m| m.spec.kind != BuiltinUnitKind::Splitter)
    {
        let (unit, streams, inputs) = fixture(model.spec.kind);
        let error = instantiate_builtin(&unit, &streams)
            .unwrap()
            .run(&services, &inputs)
            .unwrap_err();
        assert_eq!(error.message(), "test flash failure", "{}", unit.kind);
    }
}

#[test]
fn instances_own_their_parameters_and_templates_across_repeated_execution() {
    let (mut first_unit, mut streams, inputs) = fixture(BuiltinUnitKind::Feed);
    first_unit.parameters.outlet_temperature_k = Some(310.0);
    let first = instantiate_builtin(&first_unit, &streams).unwrap();
    let mut second_unit = first_unit.clone();
    second_unit.id = "unit-2".into();
    second_unit.parameters.outlet_temperature_k = Some(320.0);
    let second = instantiate_builtin(&second_unit, &streams).unwrap();
    // Editing after assembly must not change either instance.
    first_unit.parameters.outlet_temperature_k = Some(330.0);
    streams.get_mut(&"outlet".into()).unwrap().temperature_k = 340.0;
    let saved_streams = streams.clone();
    for _ in 0..2 {
        for (operation, temperature) in [(&first, 310.0), (&second, 320.0)] {
            let outputs = operation
                .run(&UnitOperationServices::default(), &inputs)
                .unwrap();
            let outlet = outputs.stream("outlet").unwrap();
            assert_eq!(outlet.temperature_k, temperature);
            assert_eq!(outlet.pressure_pa, 100_000.0);
        }
    }
    assert_eq!(streams, saved_streams);
    assert_eq!(first_unit.parameters.outlet_temperature_k, Some(330.0));
}

#[test]
fn construction_preserves_template_defaults_and_explicit_parameter_overrides() {
    let provider = crate::tests::build_provider([2.0, 0.5], 100_000.0);
    let flash = PlaceholderTpFlashSolver;
    let services = UnitOperationServices {
        thermo: Some(&provider),
        flash_solver: Some(&flash),
    };
    for model in builtin_models()
        .iter()
        .filter(|m| m.spec.kind != BuiltinUnitKind::Splitter)
    {
        let (mut unit, streams, inputs) = fixture(model.spec.kind);
        let original_streams = streams.clone();
        for explicit in [false, true] {
            if explicit {
                unit.parameters = UnitOperationParameters {
                    outlet_temperature_k: model
                        .input_specification
                        .parameters
                        .contains(&crate::specifications::UnitParameter::OutletTemperature)
                        .then_some(310.0),
                    outlet_pressure_pa: Some(90_000.0),
                    outlet_a_split_fraction: None,
                };
            }
            let outputs = instantiate_builtin(&unit, &streams)
                .unwrap()
                .run(&services, &inputs)
                .unwrap();
            for port in model
                .spec
                .ports
                .iter()
                .filter(|port| port.direction == PortDirection::Outlet)
            {
                let outlet = outputs.stream(port.name).unwrap();
                assert_eq!(
                    outlet.id,
                    *unit
                        .ports
                        .iter()
                        .find(|p| p.name == port.name)
                        .unwrap()
                        .stream_id
                        .as_ref()
                        .unwrap()
                );
                assert_eq!(
                    outlet.pressure_pa,
                    if explicit { 90_000.0 } else { 100_000.0 },
                    "{}",
                    unit.kind
                );
                let expected_temperature = if explicit
                    && !matches!(
                        model.spec.kind,
                        BuiltinUnitKind::Mixer | BuiltinUnitKind::Valve
                    ) {
                    310.0
                } else {
                    300.0
                };
                assert!(
                    (outlet.temperature_k - expected_temperature).abs() < 1e-8,
                    "{}",
                    unit.kind
                );
            }
        }
        assert_eq!(streams, original_streams);
    }
}

#[test]
fn construction_errors_keep_model_and_output_context_without_eager_parameter_validation() {
    let (mut unit, mut streams, _) = fixture(BuiltinUnitKind::Feed);
    unit.parameters.outlet_temperature_k = Some(-1.0);
    // Execution owns scalar validation; construction must still accept this parameter.
    assert!(instantiate_builtin(&unit, &streams).is_ok());
    streams.clear();
    let error = instantiate_builtin(&unit, &streams).err().unwrap();
    assert_eq!(error.code().as_str(), "missing_entity");
    assert_eq!(
        error.message(),
        "unit `unit-1` (`feed`) port `outlet` references missing stream `outlet`: missing stream `outlet`"
    );
    unit.ports[0].stream_id = None;
    let error = instantiate_builtin(&unit, &streams).err().unwrap();
    assert_eq!(
        error.message(),
        "unit `unit-1` (`feed`) port `outlet` is missing a connected stream id"
    );
    unit.ports.clear();
    let error = instantiate_builtin(&unit, &streams).err().unwrap();
    assert_eq!(
        error.message(),
        "unit `unit-1` (`feed`) does not define expected port `outlet`"
    );
    unit.kind = "pump".into();
    let error = instantiate_builtin(&unit, &streams).err().unwrap();
    assert_eq!(error.code().as_str(), "invalid_input");
    assert_eq!(error.context().related_unit_ids(), &[unit.id]);
    assert_eq!(
        error.message(),
        "unit `unit-1` (`pump`) uses unsupported model kind `pump`"
    );
}
