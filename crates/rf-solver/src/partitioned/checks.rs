use super::*;
use rf_unitops::specifications::{CompositionCheck, CompositionPolicy, check_composition};

pub(super) fn validate_vector(
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

pub(super) fn component_flows(
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

pub(super) fn residuals(
    components: &[ComponentId],
    assumed: &BTreeMap<ComponentId, f64>,
    returned: &BTreeMap<ComponentId, f64>,
) -> BTreeMap<ComponentId, f64> {
    components
        .iter()
        .map(|id| (id.clone(), returned[id] - assumed[id]))
        .collect()
}

pub(super) fn sample(
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

pub(super) fn converged(sample: &ResidualSample) -> bool {
    sample.max_absolute_mol_s <= ABSOLUTE_FLOW_TOLERANCE
        && sample.max_scaled <= SCALED_FLOW_TOLERANCE
}

pub(super) fn balance(
    plan: &PartitionedSolvePlan,
    inputs: &[MaterialStreamState],
    outputs: &[MaterialStreamState],
    scales: &BTreeMap<ComponentId, f64>,
) -> RfResult<()> {
    let mut totals =
        vec![BTreeMap::from_iter(plan.components.iter().cloned().map(|id| (id, 0.0))); 2];
    let mut total_flows = [0.0; 2];
    for (side, streams) in [inputs, outputs].into_iter().enumerate() {
        for stream in streams {
            let values = component_flows(&plan.components, stream, plan.options.boundary)?;
            for (id, value) in values {
                *totals[side].get_mut(&id).expect("component registry") += value;
            }
            total_flows[side] += stream.total_molar_flow_mol_s;
        }
    }
    let defects = residuals(&plan.components, &totals[0], &totals[1]);
    let evidence = sample(&defects, scales, EvaluationPhase::Verification, 0)?;
    let total_defect = (total_flows[1] - total_flows[0]).abs();
    if !converged(&evidence)
        || !total_defect.is_finite()
        || total_defect > plan.components.len() as f64 * ABSOLUTE_FLOW_TOLERANCE
    {
        let mut error = failure(
            "component_balance",
            "local or overall material balance exceeds frozen absolute/scaled tolerances",
        )
        .with_related_stream_ids(inputs.iter().chain(outputs).map(|s| s.id.clone()).collect());
        for (id, defect) in &defects {
            if defect.abs() > ABSOLUTE_FLOW_TOLERANCE
                || defect.abs() / scales[id] > SCALED_FLOW_TOLERANCE
            {
                error = error.with_related_component_id(id.clone());
            }
        }
        return Err(error);
    }
    Ok(())
}

pub(super) fn guess(
    plan: &PartitionedSolvePlan,
    recycle: &RecyclePlan,
    fresh: &MaterialStreamState,
    values: &BTreeMap<ComponentId, f64>,
) -> RfResult<MaterialStreamState> {
    validate_vector(&plan.components, values, false)?;
    let total = values.values().sum::<f64>();
    if !total.is_finite() {
        return Err(failure("flow_overflow", "tear component sum overflows"));
    }
    // Explicit zero-flow rule: fresh boundary composition, never an outlet template.
    let composition = if total == 0.0 {
        fresh.overall_mole_fractions.clone()
    } else {
        values
            .iter()
            .map(|(id, n)| (id.clone(), n / total))
            .collect()
    };
    let template = &plan.flowsheet.streams[&recycle.tear.stream_id];
    Ok(MaterialStreamState::from_tpzf(
        template.id.clone(),
        template.name.clone(),
        plan.options.boundary.temperature_k,
        plan.options.boundary.pressure_pa,
        total,
        composition,
    ))
}
