use super::execution::Execution;
use super::*;
use crate::fixed_tp_material as material;
use rf_types::PortDirection;
use rf_unitops::UnitOperationInputs;

impl Execution<'_, '_> {
    pub(super) fn verify_balance(
        &self,
        inputs: &[MaterialStreamState],
        outputs: &[MaterialStreamState],
        scales: &BTreeMap<ComponentId, f64>,
    ) -> RfResult<()> {
        material::balance(
            &self.plan.components,
            self.plan.options.boundary,
            inputs,
            outputs,
            scales,
        )?;
        let input_total = inputs.iter().map(|s| s.total_molar_flow_mol_s).sum::<f64>();
        let output_total = outputs
            .iter()
            .map(|s| s.total_molar_flow_mol_s)
            .sum::<f64>();
        let scale = scales
            .values()
            .next()
            .copied()
            .ok_or_else(|| failure("balance_scale", "missing total-flow scale"))?;
        let ratio = (output_total - input_total).abs() / scale;
        if !ratio.is_finite() || ratio > 1e-10 {
            return Err(failure(
                "total_balance",
                "scaled total material balance exceeds fixed tolerance",
            ));
        }
        Ok(())
    }

    pub(super) fn evaluate_unit(
        &mut self,
        id: &UnitId,
        assumptions: &BTreeMap<StreamId, MaterialStreamState>,
        scales: &BTreeMap<ComponentId, f64>,
    ) -> RunResult<LocalUnitEvaluation> {
        self.at(ModularStage::UnitStarted, Some(id.clone()), None)?;
        let unit = &self.plan.flowsheet.units[id];
        let mut inputs = UnitOperationInputs::new();
        let mut consumed = Vec::new();
        for p in unit
            .ports
            .iter()
            .filter(|p| p.direction == PortDirection::Inlet)
        {
            let sid = p.stream_id.as_ref().expect("validated binding");
            let stream = assumptions
                .get(sid)
                .or_else(|| self.streams.get(sid))
                .ok_or_else(|| RunError {
                    kind: ModularFailureKind::LocalEvaluation,
                    error: failure(
                        "missing_upstream",
                        "input is not a current producer or captured assumption",
                    )
                    .with_related_stream_id(sid.clone())
                    .with_related_unit_id(id.clone()),
                })?;
            classify(
                ModularFailureKind::Boundary,
                material::component_flows(
                    &self.plan.components,
                    stream,
                    self.plan.options.boundary,
                )
                .map_err(|e| {
                    e.with_related_unit_id(id.clone()).with_related_port_target(
                        DiagnosticPortTarget::new(id.clone(), p.name.clone()),
                    )
                }),
            )?;
            consumed.push(stream.clone());
            inputs.insert_material_stream(&p.name, stream.clone());
        }
        let ids: Vec<_> = consumed.iter().map(|s| s.id.clone()).collect();
        classify(
            ModularFailureKind::LocalEvaluation,
            crate::parameter_checks::validate_step_parameters(1, unit, &inputs, &ids),
        )?;
        self.costs.local_unit_calls += 1;
        *self.costs.calls_by_unit.entry(id.clone()).or_default() += 1;
        let outputs = classify(
            ModularFailureKind::LocalEvaluation,
            self.operations[id]
                .run(&self.services, &inputs)
                .map_err(|e| {
                    e.with_related_unit_id(id.clone())
                        .with_related_stream_ids(
                            unit.ports
                                .iter()
                                .filter_map(|p| p.stream_id.clone())
                                .collect(),
                        )
                        .with_related_port_targets(
                            unit.ports
                                .iter()
                                .map(|p| DiagnosticPortTarget::new(id.clone(), p.name.clone()))
                                .collect(),
                        )
                }),
        )?;
        let mut produced = Vec::new();
        for p in unit
            .ports
            .iter()
            .filter(|p| p.direction == PortDirection::Outlet)
        {
            let output = outputs.stream(&p.name).ok_or_else(|| RunError {
                kind: ModularFailureKind::LocalEvaluation,
                error: failure("missing_output", "local unit omitted canonical outlet")
                    .with_related_unit_id(id.clone()),
            })?;
            if Some(&output.id) != p.stream_id.as_ref() {
                return Err(RunError {
                    kind: ModularFailureKind::LocalEvaluation,
                    error: failure(
                        "output_identity",
                        "local output differs from source binding",
                    )
                    .with_related_unit_id(id.clone())
                    .with_related_stream_id(output.id.clone()),
                });
            }
            classify(
                ModularFailureKind::Boundary,
                material::component_flows(
                    &self.plan.components,
                    output,
                    self.plan.options.boundary,
                )
                .map_err(|e| {
                    e.with_related_unit_id(id.clone()).with_related_port_target(
                        DiagnosticPortTarget::new(id.clone(), p.name.clone()),
                    )
                }),
            )?;
            produced.push(output.clone());
        }
        let evaluated = LocalUnitEvaluation {
            unit_id: id.clone(),
            consumed_streams: consumed,
            produced_streams: produced,
        };
        self.active_units.push(evaluated.clone());
        if unit.kind != "feed" {
            classify(
                ModularFailureKind::Balance,
                self.verify_balance(
                    &evaluated.consumed_streams,
                    &evaluated.produced_streams,
                    scales,
                )
                .map_err(|e| e.with_related_unit_id(id.clone())),
            )?;
        }
        for output in &evaluated.produced_streams {
            self.streams.insert(output.id.clone(), output.clone());
        }
        self.at(ModularStage::UnitCompleted, Some(id.clone()), None)?;
        Ok(evaluated)
    }
    pub(super) fn evaluate_couplings(
        &mut self,
        block: &ModularBlock,
        values: &[f64],
        zero_rules: &BTreeMap<StreamId, StreamId>,
        scales: &BTreeMap<ComponentId, f64>,
    ) -> RunResult<(Vec<f64>, Vec<LocalUnitEvaluation>)> {
        if values.len() != block.couplings.len()
            || values.iter().any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(RunError {
                kind: ModularFailureKind::Boundary,
                error: failure(
                    "trial_domain",
                    "coupling trial must be nonnegative finite and complete",
                ),
            });
        }
        let mut grouped: BTreeMap<StreamId, BTreeMap<ComponentId, f64>> = BTreeMap::new();
        for (c, &value) in block.couplings.iter().zip(values) {
            grouped
                .entry(c.stream.clone())
                .or_default()
                .insert(c.component.clone(), value);
        }
        let mut assumptions = BTreeMap::new();
        for (sid, flows) in grouped {
            let total = flows.values().sum::<f64>();
            if !total.is_finite() {
                return Err(RunError {
                    kind: ModularFailureKind::Boundary,
                    error: failure("flow_overflow", "trial component sum overflows")
                        .with_related_stream_id(sid),
                });
            }
            let composition = if total == 0.0 {
                self.streams[&zero_rules[&sid]]
                    .overall_mole_fractions
                    .clone()
            } else {
                flows.into_iter().map(|(id, n)| (id, n / total)).collect()
            };
            let t = &self.plan.flowsheet.streams[&sid];
            let b = self.plan.options.boundary;
            assumptions.insert(
                sid.clone(),
                MaterialStreamState::from_tpzf(
                    sid,
                    t.name.clone(),
                    b.temperature_k,
                    b.pressure_pa,
                    total,
                    composition,
                ),
            );
        }
        self.active_units.clear();
        let mut units = Vec::new();
        for id in &block.members {
            units.push(self.evaluate_unit(id, &assumptions, scales)?);
        }
        let mut residuals = Vec::new();
        for (i, c) in block.couplings.iter().enumerate() {
            let produced = classify(
                ModularFailureKind::Boundary,
                material::component_flows(
                    &self.plan.components,
                    &self.streams[&c.stream],
                    self.plan.options.boundary,
                ),
            )?;
            let residual = values[i] - produced[&c.component];
            if !residual.is_finite() {
                return Err(RunError {
                    kind: ModularFailureKind::Boundary,
                    error: failure("nonfinite_residual", "coupling residual overflow")
                        .with_related_component_id(c.component.clone())
                        .with_related_stream_id(c.stream.clone()),
                });
            }
            residuals.push(residual);
        }
        Ok((residuals, units))
    }
}
