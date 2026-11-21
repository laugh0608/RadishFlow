use super::*;
use crate::single_phase_recycle::{
    SinglePhaseRecycleOptions, SinglePhaseRecycleResult, SinglePhaseRecycleSolver,
    SinglePhaseTearBinding, SinglePhaseTearState,
};
use rf_types::DiagnosticPortTarget;
use v4::{
    StoredPortBinding, StoredSinglePhaseRecyclePlan, StoredSinglePhaseTear,
    StoredSinglePhaseTearInitialization as Initial,
};

const RECYCLE_SUPPORT: FlowPlanSupport = FlowPlanSupport::SinglePhaseRecycle;

fn recycle_project(
    kind: BuiltinUnitKind,
    temperature_k: f64,
    duty_w: Option<f64>,
) -> StoredProjectV4 {
    let (mut flowsheet, options) = super::super::recycle::case(kind, temperature_k, 0.6);
    if duty_w.is_some() {
        flowsheet
            .units
            .get_mut(&kind.as_str().into())
            .unwrap()
            .parameters
            .outlet_temperature_k = None;
    }
    let mut project = project(
        flowsheet,
        duty_w
            .map(|q| (kind.as_str().into(), q))
            .into_iter()
            .collect(),
    );
    let StoredFlowPlan::SinglePhase(single_phase) = project.steady_state_plan else {
        unreachable!()
    };
    project.steady_state_plan =
        StoredFlowPlan::SinglePhaseRecycle(Box::new(StoredSinglePhaseRecyclePlan {
            single_phase,
            tear: StoredSinglePhaseTear {
                stream_id: options.tear.stream_id,
                source: StoredPortBinding {
                    unit_id: options.tear.source.unit_id,
                    port: options.tear.source.port_name,
                },
                sink: StoredPortBinding {
                    unit_id: options.tear.sink.unit_id,
                    port: options.tear.sink.port_name,
                },
                initialization: if duty_w.is_some() {
                    Initial::ComponentFlowsAndEnthalpy {
                        component_flows_mol_s: [
                            ("component-a".into(), 3.0),
                            ("component-b".into(), 4.5),
                        ]
                        .into(),
                        enthalpy_flow_w: 10000.0,
                    }
                } else {
                    Initial::ZeroFlowsAndEnthalpy
                },
            },
            max_iteration_evaluations: 120,
        }));
    project
}

fn runtime(project: &StoredProjectV4) -> SinglePhaseRecycleResult {
    let StoredFlowPlan::SinglePhaseRecycle(plan) = &project.steady_state_plan else {
        panic!("recycle plan expected")
    };
    let thermal = &plan.single_phase;
    let properties = crate::flow_plan::SinglePhaseOptions::new(
        thermal.heat_capacities_j_per_mol_k.clone(),
        thermal.min_temperature_k..=thermal.max_temperature_k,
        thermal.pressure_pa,
        thermal.heat_duties_w.clone(),
    )
    .unwrap();
    let target = |port: &StoredPortBinding| {
        DiagnosticPortTarget::new(port.unit_id.clone(), port.port.clone())
    };
    let options = SinglePhaseRecycleOptions {
        tear: SinglePhaseTearBinding {
            stream_id: plan.tear.stream_id.clone(),
            source: target(&plan.tear.source),
            sink: target(&plan.tear.sink),
        },
        initial_state: match &plan.tear.initialization {
            Initial::ZeroFlowsAndEnthalpy => None,
            Initial::ComponentFlowsAndEnthalpy {
                component_flows_mol_s,
                enthalpy_flow_w,
            } => Some(SinglePhaseTearState {
                component_flows_mol_s: component_flows_mol_s.clone(),
                enthalpy_flow_w: *enthalpy_flow_w,
            }),
        },
        max_iteration_evaluations: plan.max_iteration_evaluations,
    };
    SinglePhaseRecycleSolver
        .solve_controlled(
            &project.document.flowsheet,
            &properties,
            &options,
            &mut UncontrolledSolve,
        )
        .unwrap()
}

#[test]
fn v4_recycle_reopens_and_recomputes_both_specifications() {
    let directory = std::env::temp_dir().join(format!(
        "rf-v4-single-phase-recycle-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&directory).unwrap();
    for (kind, temp, duty, product_w, recycle_w, mixed_k) in [
        (
            BuiltinUnitKind::Heater,
            380.0,
            7800.0,
            17188.5,
            25782.75,
            2556.0 / 7.0,
        ),
        (
            BuiltinUnitKind::Cooler,
            320.0,
            -4800.0,
            4588.5,
            6882.75,
            2304.0 / 7.0,
        ),
    ] {
        for spec in [None, Some(duty)] {
            let path = directory.join(format!("{}-{}.rfproj.json", kind.as_str(), spec.is_some()));
            let project = recycle_project(kind, temp, spec);
            let before = runtime(&project);
            let expected = ProjectV4Read::Editable(Box::new(project));
            let staged = v4::prepare_write_with_support(
                &path,
                &expected,
                FileOverwritePolicy::Forbid,
                RECYCLE_SUPPORT,
            )
            .unwrap();
            v4::write_prepared(&staged, false).unwrap();
            let bytes = fs::read(&path).unwrap();
            let reopened = v4::read_with_support(&path, RECYCLE_SUPPORT).unwrap();
            assert_eq!(reopened, expected);
            let ProjectV4Read::Editable(reopened) = reopened else {
                unreachable!()
            };
            let after = runtime(&reopened);
            assert_eq!(before, after);
            super::super::recycle::assert_benchmark(
                &after, temp, duty, product_w, recycle_w, mixed_k,
            );
            let ProjectV4Read::ReadOnly(legacy) = v4::read_with_support(&path, SUPPORT).unwrap()
            else {
                panic!("legacy reader must protect the entire project")
            };
            assert_eq!(legacy.original_json().as_bytes(), bytes);
            let v11_project = ProjectV4Read::Editable(Box::new(super::project(
                heater_cooler_case(kind, Some(temp)),
                BTreeMap::new(),
            )));
            assert!(
                v4::prepare_write_with_support(
                    &path,
                    &v11_project,
                    FileOverwritePolicy::Allow,
                    SUPPORT
                )
                .is_err()
            );
            assert_eq!(fs::read(&path).unwrap(), bytes);
            let old_json = v4::to_pretty_json_with_support(&v11_project, SUPPORT).unwrap();
            assert_eq!(
                v4::parse_with_support(&old_json, RECYCLE_SUPPORT).unwrap(),
                v11_project
            );
        }
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn v4_recycle_preserves_explicit_validation_on_read_and_write() {
    for (diagnostic, change, old, new) in [
        (
            "solver.single_phase_recycle.invalid_tear",
            0,
            "\"port\": \"inlet_b\"",
            "\"port\": \"inlet_a\"",
        ),
        (
            "solver.single_phase.conflicting_heat_specifications",
            1,
            "\"heatDutiesW\": {}",
            "\"heatDutiesW\": {\"heater\": 7800.0}",
        ),
    ] {
        let mut project = recycle_project(BuiltinUnitKind::Heater, 380.0, None);
        let json = v4::to_pretty_json_with_support(
            &ProjectV4Read::Editable(Box::new(project.clone())),
            RECYCLE_SUPPORT,
        )
        .unwrap();
        let StoredFlowPlan::SinglePhaseRecycle(recycle) = &mut project.steady_state_plan else {
            unreachable!()
        };
        if change == 0 {
            recycle.tear.sink.port = "inlet_a".into();
        } else {
            recycle
                .single_phase
                .heat_duties_w
                .insert("heater".into(), 7800.0);
        }
        let bad = ProjectV4Read::Editable(Box::new(project));
        for error in [
            v4::parse_with_support(&json.replace(old, new), RECYCLE_SUPPORT).unwrap_err(),
            v4::to_pretty_json_with_support(&bad, RECYCLE_SUPPORT).unwrap_err(),
        ] {
            assert_eq!(error.context().diagnostic_code(), Some(diagnostic));
        }
    }
}
