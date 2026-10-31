use rf_flowsheet::pressure_context::{PressureContextGapReason, document_pressure_context};
use rf_flowsheet::specifications::{
    CompositionCheck, CompositionPolicy, FeedInputSource, OutletPressureRule, ParameterViolation,
    check_composition, check_parameter_value, effective_feed_input, input_specification,
    is_positive_finite,
};
use rf_ui::BuiltinUnitKind;

use super::*;

pub(super) fn check_inputs(flowsheet: &Flowsheet) -> InputChecks {
    check_inputs_with_plan(flowsheet, &rf_ui::flow_plan::FlowPlanConfiguration::Acyclic)
}

pub(super) fn check_inputs_with_plan(
    flowsheet: &Flowsheet,
    plan: &rf_ui::flow_plan::FlowPlanConfiguration,
) -> InputChecks {
    let mut checks = InputChecks::default();
    if flowsheet.units.is_empty() {
        checks.problems.push(SpecificationProblem::EmptyFlowsheet);
    }
    let topology_valid = match match plan {
        rf_ui::flow_plan::FlowPlanConfiguration::Acyclic => {
            rf_solver::sequential_execution_order(flowsheet).map(|_| ())
        }
        rf_ui::flow_plan::FlowPlanConfiguration::Partitioned(options) => {
            rf_solver::partitioned::assemble_partitioned_plan(flowsheet, options).map(|_| ())
        }
        rf_ui::flow_plan::FlowPlanConfiguration::EquationOriented(options) => {
            rf_solver::equation_oriented::assemble(flowsheet, options)
                .and_then(|p| p.require_ready())
                .map_err(|e| e.error)
        }
        rf_ui::flow_plan::FlowPlanConfiguration::Simultaneous(options) => {
            rf_solver::simultaneous::assemble(flowsheet, options).map(|_| ())
        }
    } {
        Ok(_) => true,
        Err(error) => {
            // Empty workspaces keep the existing "place first unit" readiness.
            if !flowsheet.units.is_empty() {
                checks.problems.push(SpecificationProblem::Topology(error));
            }
            false
        }
    };
    checks.coverage(
        SpecificationCheckScope::Topology,
        SpecificationCoverage::Evaluated,
    );

    if flowsheet.components.is_empty() {
        checks
            .problems
            .push(SpecificationProblem::MissingProjectComponents);
    }
    for stream in flowsheet.streams.values() {
        for component in stream.overall_mole_fractions.keys() {
            if !flowsheet.components.contains_key(component) {
                checks
                    .problems
                    .push(SpecificationProblem::ComponentOutsideProject {
                        stream_id: stream.id.clone(),
                        component_id: component.clone(),
                    });
            }
        }
    }
    checks.coverage(
        SpecificationCheckScope::ProjectComponents,
        SpecificationCoverage::Evaluated,
    );

    if topology_valid {
        check_feed_inputs(
            flowsheet,
            &mut checks,
            matches!(
                plan,
                rf_ui::flow_plan::FlowPlanConfiguration::EquationOriented(_)
            ),
        );
    }
    checks.coverage(
        SpecificationCheckScope::FeedInputs,
        if topology_valid {
            SpecificationCoverage::Evaluated
        } else {
            SpecificationCoverage::NotCheckedTopology
        },
    );
    check_compositions(flowsheet, &mut checks);
    checks.coverage(
        SpecificationCheckScope::StreamCompositions,
        SpecificationCoverage::Evaluated,
    );

    // Preserve authoring's old ordering: every missing specification precedes
    // execution-only parameter failures, regardless of their object IDs.
    for unit in flowsheet.units.values() {
        if let Some(kind) = BuiltinUnitKind::parse(&unit.kind) {
            for &parameter in input_specification(kind).required_explicit_parameters {
                if !released_parameter(plan, &unit.id, parameter)
                    && parameter.value(&unit.parameters).is_none()
                {
                    checks
                        .problems
                        .push(SpecificationProblem::MissingParameter {
                            unit_id: unit.id.clone(),
                            parameter,
                        });
                }
            }
        }
    }
    for unit in flowsheet.units.values() {
        let Some(kind) = BuiltinUnitKind::parse(&unit.kind) else {
            checks.coverage(
                SpecificationCheckScope::UnitParameters(unit.id.clone()),
                SpecificationCoverage::UnsupportedUnitKind(unit.kind.clone()),
            );
            continue;
        };
        let spec = input_specification(kind);
        for &parameter in spec.parameters {
            if !released_parameter(plan, &unit.id, parameter)
                && let Some(value) = parameter.value(&unit.parameters)
                && check_parameter_value(kind, parameter, value, None).is_err()
            {
                checks
                    .problems
                    .push(SpecificationProblem::InvalidParameter {
                        unit_id: unit.id.clone(),
                        parameter,
                        value,
                    });
            }
        }
        checks.coverage(
            SpecificationCheckScope::UnitParameters(unit.id.clone()),
            SpecificationCoverage::Evaluated,
        );
        if spec.outlet_pressure_rule == OutletPressureRule::Unrestricted {
            continue;
        }
        let scope = SpecificationCheckScope::InletPressure(unit.id.clone());
        if !topology_valid {
            checks.coverage(scope, SpecificationCoverage::NotCheckedTopology);
            continue;
        }
        let Some(pressure_pa) = unit
            .parameters
            .outlet_pressure_pa
            .filter(|value| is_positive_finite(*value))
        else {
            checks.coverage(
                scope,
                SpecificationCoverage::NotCheckedMissingOrInvalidParameter,
            );
            continue;
        };
        if matches!(
            plan,
            rf_ui::flow_plan::FlowPlanConfiguration::Partitioned(_)
                | rf_ui::flow_plan::FlowPlanConfiguration::Simultaneous(_)
                | rf_ui::flow_plan::FlowPlanConfiguration::EquationOriented(_)
        ) {
            // Cyclic inlet pressure comes from the frozen fixed boundary and evaluated
            // streams. Document templates cannot prove the recycle inlet pressure.
            checks.coverage(
                scope,
                SpecificationCoverage::AwaitingExecution(document_pressure_context(
                    flowsheet, unit, kind,
                )),
            );
            continue;
        }
        let context = document_pressure_context(flowsheet, unit, kind);
        let Some(limit) = context.limit() else {
            let missing_inputs = context.gaps.iter().any(|gap| {
                matches!(
                    gap.reason,
                    PressureContextGapReason::MissingExplicitPressure(_)
                        | PressureContextGapReason::InvalidPressure(_)
                )
            });
            checks.coverage(
                scope,
                if missing_inputs {
                    SpecificationCoverage::NotCheckedInputPressure(context)
                } else {
                    SpecificationCoverage::AwaitingExecution(context)
                },
            );
            continue;
        };
        if check_parameter_value(
            kind,
            UnitParameter::OutletPressure,
            pressure_pa,
            Some(&limit),
        ) == Err(ParameterViolation::AboveInletPressure)
        {
            checks
                .problems
                .push(SpecificationProblem::OutletPressureAboveInlet {
                    unit_id: unit.id.clone(),
                    pressure_pa,
                    context,
                });
        }
        checks.coverage(scope, SpecificationCoverage::Evaluated);
    }
    for scope in [
        SpecificationCheckScope::StructuralDegreesOfFreedom,
        SpecificationCheckScope::NumericalSolvability,
    ] {
        checks.coverage(scope, SpecificationCoverage::NotAnalyzed);
    }
    checks
}

fn check_feed_inputs(flowsheet: &Flowsheet, checks: &mut InputChecks, allow_zero_flow: bool) {
    let mut missing_compositions = Vec::new();
    for unit in flowsheet
        .units
        .values()
        .filter(|unit| unit.kind == BuiltinUnitKind::Feed.as_str())
    {
        // Validated topology establishes the canonical Feed outlet and its stream.
        let stream_id = unit
            .ports
            .iter()
            .find(|port| port.name == "outlet")
            .and_then(|port| port.stream_id.as_ref())
            .expect("validated Feed outlet");
        let stream = &flowsheet.streams[stream_id];
        for field in [
            FeedInputField::Temperature,
            FeedInputField::Pressure,
            FeedInputField::MolarFlow,
        ] {
            let input = effective_feed_input(&unit.parameters, stream, field);
            checks.feed_inputs.push(FeedSpecificationInput {
                unit_id: unit.id.clone(),
                stream_id: stream_id.clone(),
                field,
                input,
            });
            if input.source == FeedInputSource::StreamInput
                && !is_positive_finite(input.value)
                && !(allow_zero_flow && field == FeedInputField::MolarFlow && input.value == 0.0)
            {
                checks
                    .problems
                    .push(SpecificationProblem::InvalidFeedInput {
                        unit_id: unit.id.clone(),
                        stream_id: stream_id.clone(),
                        field,
                        value: input.value,
                    });
            }
        }
        if stream.overall_mole_fractions.is_empty() {
            missing_compositions.push(SpecificationProblem::MissingFeedComposition {
                unit_id: unit.id.clone(),
                stream_id: stream_id.clone(),
            });
        }
    }
    checks.problems.extend(missing_compositions);
}

fn check_compositions(flowsheet: &Flowsheet, checks: &mut InputChecks) {
    for stream in flowsheet.streams.values() {
        match check_composition(
            &stream.overall_mole_fractions,
            CompositionPolicy::SpecifiedMoleFractions,
        ) {
            CompositionCheck::Empty | CompositionCheck::Valid { .. } => {}
            CompositionCheck::InvalidFractions(components) => {
                for component_id in components {
                    checks
                        .problems
                        .push(SpecificationProblem::InvalidMoleFraction {
                            stream_id: stream.id.clone(),
                            value: stream.overall_mole_fractions[&component_id],
                            component_id,
                        });
                }
            }
            CompositionCheck::InvalidTotal { sum } => {
                checks
                    .problems
                    .push(SpecificationProblem::InvalidCompositionTotal {
                        stream_id: stream.id.clone(),
                        sum,
                    })
            }
            CompositionCheck::Unnormalized { sum } => {
                checks
                    .problems
                    .push(SpecificationProblem::UnnormalizedComposition {
                        stream_id: stream.id.clone(),
                        sum,
                    })
            }
        }
    }
}

fn released_parameter(
    plan: &rf_ui::flow_plan::FlowPlanConfiguration,
    unit: &rf_types::UnitId,
    parameter: UnitParameter,
) -> bool {
    matches!(plan, rf_ui::flow_plan::FlowPlanConfiguration::EquationOriented(o) if parameter == UnitParameter::OutletASplitFraction && o.released.as_ref().is_some_and(|r| &r.binding.unit == unit))
}
