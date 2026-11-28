//! Fixed-T/P material checks shared by the supported modular strategies.
use crate::partitioned::{
    ABSOLUTE_FLOW_TOLERANCE, ComponentBalanceExcess, EvaluationPhase, FixedRecycleBoundary,
    FlowDifference, MaterialBalanceReadings, PRESSURE_TOLERANCE_PA, RecycleTolerances,
    ResidualSample, SCALED_FLOW_TOLERANCE, TEMPERATURE_TOLERANCE_K,
};
use rf_model::MaterialStreamState;
use rf_types::{ComponentId, RfError, RfResult};
use rf_unitops::specifications::{CompositionCheck, CompositionPolicy, check_composition};
use std::collections::BTreeMap;
fn failure(code: &str, message: impl Into<String>) -> RfError {
    RfError::invalid_input(message).with_diagnostic_code(format!("solver.partitioned.{code}"))
}
pub(crate) fn validate_vector(
    components: &[ComponentId],
    values: &BTreeMap<ComponentId, f64>,
    positive: bool,
) -> RfResult<()> {
    if values.keys().ne(components.iter()) {
        let mut error = failure(
            "component_binding",
            "component vector must bind exactly the frozen registered components",
        );
        for id in components
            .iter()
            .filter(|id| !values.contains_key(*id))
            .chain(
                values
                    .keys()
                    .filter(|id| components.binary_search(id).is_err()),
            )
        {
            error = error.with_related_component_id(id.clone());
        }
        return Err(error);
    }
    for (component, &value) in values {
        if !value.is_finite() || value < 0.0 || (positive && value == 0.0) {
            return Err(failure(
                "invalid_component_value",
                format!(
                    "component `{component}` requires a {}finite value",
                    if positive {
                        "positive "
                    } else {
                        "nonnegative "
                    }
                ),
            )
            .with_related_component_id(component.clone()));
        }
    }
    Ok(())
}

pub(crate) fn tear_stream(
    template: &MaterialStreamState,
    fresh: &MaterialStreamState,
    boundary: FixedRecycleBoundary,
    values: &BTreeMap<ComponentId, f64>,
) -> RfResult<MaterialStreamState> {
    let total: f64 = values.values().sum();
    if !total.is_finite() {
        return Err(failure("flow_overflow", "tear component sum overflows")
            .with_related_stream_id(template.id.clone()));
    }
    let composition = if total == 0.0 {
        fresh.overall_mole_fractions.clone()
    } else {
        values
            .iter()
            .map(|(id, n)| (id.clone(), n / total))
            .collect()
    };
    Ok(MaterialStreamState::from_tpzf(
        template.id.clone(),
        template.name.clone(),
        boundary.temperature_k,
        boundary.pressure_pa,
        total,
        composition,
    ))
}

pub(crate) fn component_flows(
    components: &[ComponentId],
    stream: &MaterialStreamState,
    boundary: FixedRecycleBoundary,
) -> RfResult<BTreeMap<ComponentId, f64>> {
    let invalid = |code, message| failure(code, message).with_related_stream_id(stream.id.clone());
    if !stream.total_molar_flow_mol_s.is_finite() || stream.total_molar_flow_mol_s < 0.0 {
        return Err(invalid(
            "invalid_flow",
            "stream flow must be finite and nonnegative",
        ));
    }
    if !stream.temperature_k.is_finite()
        || stream.temperature_k <= 0.0
        || !stream.pressure_pa.is_finite()
        || stream.pressure_pa <= 0.0
        || (stream.temperature_k - boundary.temperature_k).abs() > TEMPERATURE_TOLERANCE_K
        || (stream.pressure_pa - boundary.pressure_pa).abs() > PRESSURE_TOLERANCE_PA
    {
        return Err(invalid(
            "fixed_boundary",
            "stream violates the fixed temperature/pressure boundary",
        ));
    }
    if !matches!(
        check_composition(
            &stream.overall_mole_fractions,
            CompositionPolicy::SpecifiedMoleFractions
        ),
        CompositionCheck::Valid { .. }
    ) {
        return Err(invalid(
            "invalid_composition",
            "stream requires normalized nonnegative mole fractions",
        ));
    }
    for component in stream.overall_mole_fractions.keys() {
        if components.binary_search(component).is_err() {
            return Err(invalid(
                "component_binding",
                "stream component is outside frozen component registry",
            )
            .with_related_component_id(component.clone()));
        }
    }
    for phase in &stream.phases {
        if !phase.phase_fraction.is_finite()
            || !(0.0..=1.0).contains(&phase.phase_fraction)
            || phase
                .molar_enthalpy_j_per_mol
                .is_some_and(|h| !h.is_finite())
        {
            return Err(invalid(
                "invalid_phase",
                "phase intensity must remain finite",
            ));
        }
    }
    let values: BTreeMap<_, _> = components
        .iter()
        .map(|id| {
            (
                id.clone(),
                stream.total_molar_flow_mol_s
                    * stream
                        .overall_mole_fractions
                        .get(id)
                        .copied()
                        .unwrap_or(0.0),
            )
        })
        .collect();
    validate_vector(components, &values, false)?;
    Ok(values)
}

pub(crate) fn residuals(
    components: &[ComponentId],
    assumed: &BTreeMap<ComponentId, f64>,
    returned: &BTreeMap<ComponentId, f64>,
) -> BTreeMap<ComponentId, f64> {
    components
        .iter()
        .map(|id| (id.clone(), returned[id] - assumed[id]))
        .collect()
}

pub(crate) fn sample(
    residuals: &BTreeMap<ComponentId, f64>,
    scales: &BTreeMap<ComponentId, f64>,
    phase: EvaluationPhase,
    evaluation: usize,
) -> RfResult<ResidualSample> {
    let mut raw: f64 = 0.0;
    let mut scaled: f64 = 0.0;
    for (id, &defect) in residuals {
        let value = defect.abs();
        let ratio = value / scales[id];
        if !value.is_finite() || !ratio.is_finite() {
            return Err(failure(
                "nonfinite_residual",
                "residual or scaled residual is nonfinite",
            )
            .with_related_component_id(id.clone()));
        }
        raw = raw.max(value);
        scaled = scaled.max(ratio);
    }
    Ok(ResidualSample {
        phase,
        evaluation,
        max_absolute_mol_s: raw,
        max_scaled: scaled,
    })
}

pub(crate) fn converged(sample: &ResidualSample) -> bool {
    sample.max_absolute_mol_s <= ABSOLUTE_FLOW_TOLERANCE
        && sample.max_scaled <= SCALED_FLOW_TOLERANCE
}

#[derive(Debug)]
pub(crate) enum BalanceCheckError {
    Invalid(RfError),
    Exceeded(Box<MaterialBalanceReadings>),
}

impl From<RfError> for BalanceCheckError {
    fn from(error: RfError) -> Self {
        Self::Invalid(error)
    }
}

impl MaterialBalanceReadings {
    pub(crate) fn diagnostic(&self) -> RfError {
        let mut error = failure(
            "component_balance",
            "local or overall material balance exceeds frozen absolute/scaled tolerances",
        )
        .with_related_stream_ids(self.inputs.iter().chain(&self.outputs).cloned().collect());
        for row in &self.exceeded_components {
            error = error.with_related_component_id(row.component_id.clone());
        }
        error
    }
}

pub(crate) fn balance(
    components: &[ComponentId],
    boundary: FixedRecycleBoundary,
    inputs: &[MaterialStreamState],
    outputs: &[MaterialStreamState],
    scales: &BTreeMap<ComponentId, f64>,
) -> RfResult<()> {
    check_balance(components, boundary, inputs, outputs, scales).map_err(|error| match error {
        BalanceCheckError::Invalid(error) => error,
        BalanceCheckError::Exceeded(readings) => readings.diagnostic(),
    })
}

pub(crate) fn check_balance(
    components: &[ComponentId],
    boundary: FixedRecycleBoundary,
    inputs: &[MaterialStreamState],
    outputs: &[MaterialStreamState],
    scales: &BTreeMap<ComponentId, f64>,
) -> Result<(), BalanceCheckError> {
    let mut totals = vec![BTreeMap::from_iter(components.iter().cloned().map(|id| (id, 0.0))); 2];
    let mut total_flows = [0.0; 2];
    for (side, streams) in [inputs, outputs].into_iter().enumerate() {
        for stream in streams {
            let values = component_flows(components, stream, boundary)?;
            for (id, value) in values {
                *totals[side].get_mut(&id).expect("component registry") += value;
            }
            total_flows[side] += stream.total_molar_flow_mol_s;
        }
    }
    let defects = residuals(components, &totals[0], &totals[1]);
    let evidence = sample(&defects, scales, EvaluationPhase::Verification, 0)?;
    let tolerances = RecycleTolerances::for_components(components.len());
    let total_difference = total_flows[1] - total_flows[0];
    let total_exceeded = !total_difference.is_finite()
        || total_difference.abs() > tolerances.total_absolute_flow_mol_s;
    if converged(&evidence) && !total_exceeded {
        return Ok(());
    }
    let mut exceeded_components = Vec::new();
    for (id, difference_mol_s) in defects {
        let scale_mol_s = scales[&id];
        let absolute = difference_mol_s.abs();
        let scaled_absolute = absolute / scale_mol_s;
        let absolute_exceeded = absolute > tolerances.component_absolute_flow_mol_s;
        let scaled_exceeded = scaled_absolute > tolerances.scaled_flow;
        if absolute_exceeded || scaled_exceeded {
            exceeded_components.push(ComponentBalanceExcess {
                flow: FlowDifference {
                    input_mol_s: totals[0][&id],
                    output_mol_s: totals[1][&id],
                    difference_mol_s,
                },
                component_id: id,
                scale_mol_s,
                scaled_absolute,
                absolute_exceeded,
                scaled_exceeded,
            });
        }
    }
    Err(BalanceCheckError::Exceeded(Box::new(
        MaterialBalanceReadings {
            inputs: inputs.iter().map(|s| s.id.clone()).collect(),
            outputs: outputs.iter().map(|s| s.id.clone()).collect(),
            exceeded_components,
            tolerances,
            total: FlowDifference {
                input_mol_s: total_flows[0],
                output_mol_s: total_flows[1],
                difference_mol_s: total_difference,
            },
            total_exceeded,
        },
    )))
}
