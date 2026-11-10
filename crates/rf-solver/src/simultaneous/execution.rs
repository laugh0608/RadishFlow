use super::*;
use crate::{
    fixed_tp_material as material,
    local_services::{CountedFlash, CountedThermo},
};
use rf_unitops::{UnitOperation, UnitOperationServices, catalog::instantiate_builtin};
use std::{cell::Cell, collections::BTreeSet, time::Instant};

pub(super) struct Execution<'a, 'c> {
    pub(super) plan: &'a ModularPlan,
    pub(super) services: UnitOperationServices<'a>,
    pub(super) operations: BTreeMap<UnitId, Box<dyn UnitOperation>>,
    pub(super) control: &'c mut dyn ModularControl,
    pub(super) streams: BTreeMap<StreamId, MaterialStreamState>,
    pub(super) blocks: Vec<ModularBlockReport>,
    pub(super) active_units: Vec<LocalUnitEvaluation>,
    pub(super) active_coupling: Option<CoupledEvidence>,
    pub(super) costs: ModularCosts,
    pub(super) progress: ModularProgress,
    pub(super) global_scales: BTreeMap<ComponentId, f64>,
}
pub(super) fn solve(
    services: &crate::SolverServices<'_>,
    flow: &Flowsheet,
    options: &ModularOptions,
    control: &mut impl ModularControl,
) -> Result<ModularResult, Box<ModularFailure>> {
    let start = Instant::now();
    let mut progress = ModularProgress {
        stage: ModularStage::AssemblyStarted,
        block: None,
        unit: None,
        column: None,
    };
    let early = |kind, error, progress: ModularProgress| {
        Box::new(ModularFailure {
            kind,
            error,
            completed_blocks: Vec::new(),
            active_units: Vec::new(),
            active_coupling: None,
            progress,
            costs: ModularCosts {
                assembly_elapsed: start.elapsed(),
                elapsed: start.elapsed(),
                ..Default::default()
            },
        })
    };
    if control.checkpoint(progress.clone()).is_break() {
        return Err(early(
            ModularFailureKind::Cancelled,
            failure("cancelled", "cancelled before assembly"),
            progress,
        ));
    }
    let plan = assemble(flow, options)
        .map_err(|e| early(ModularFailureKind::Assembly, e, progress.clone()))?;
    progress.stage = ModularStage::AssemblyCompleted;
    if control.checkpoint(progress.clone()).is_break() {
        return Err(early(
            ModularFailureKind::Cancelled,
            failure("cancelled", "cancelled after assembly"),
            progress,
        ));
    }
    let thermo = CountedThermo {
        provider: services.thermo,
        calls: Cell::new(0),
    };
    let flash = CountedFlash {
        solver: services.flash_solver,
        calls: Cell::new(0),
    };
    let mut run = Execution {
        plan: &plan,
        services: UnitOperationServices {
            thermo: Some(&thermo),
            flash_solver: Some(&flash),
        },
        operations: BTreeMap::new(),
        control,
        streams: BTreeMap::new(),
        blocks: Vec::new(),
        active_units: Vec::new(),
        active_coupling: None,
        costs: ModularCosts {
            assembly_elapsed: start.elapsed(),
            ..Default::default()
        },
        progress,
        global_scales: BTreeMap::new(),
    };
    let init = Instant::now();
    let initialized = run.initialize();
    run.costs.initialization_elapsed = init.elapsed();
    let outcome = initialized.and_then(|()| run.execute());
    run.costs.thermo_calls = thermo.calls.get();
    run.costs.flash_calls = flash.calls.get();
    run.costs.elapsed = start.elapsed();
    match outcome {
        Ok(()) => Ok(ModularResult {
            method: METHOD,
            streams: run.streams,
            blocks: run.blocks,
            costs: run.costs,
        }),
        Err(e) => Err(Box::new(ModularFailure {
            kind: e.kind,
            error: e.error,
            completed_blocks: run.blocks,
            active_units: run.active_units,
            active_coupling: run.active_coupling,
            progress: run.progress,
            costs: run.costs,
        })),
    }
}
impl Execution<'_, '_> {
    pub(super) fn at(
        &mut self,
        stage: ModularStage,
        unit: Option<UnitId>,
        column: Option<usize>,
    ) -> RunResult<()> {
        self.progress = ModularProgress {
            stage,
            unit,
            column,
            block: self.progress.block,
        };
        if self.control.checkpoint(self.progress.clone()).is_break() {
            let mut error = failure("cancelled", format!("cancelled at {stage:?}"));
            if let Some(u) = &self.progress.unit {
                error = error.with_related_unit_id(u.clone());
            }
            return Err(RunError {
                kind: ModularFailureKind::Cancelled,
                error,
            });
        }
        Ok(())
    }
    pub(super) fn initialize(&mut self) -> RunResult<()> {
        self.at(ModularStage::Initializing, None, None)?;
        let ids: Vec<_> = self
            .services
            .thermo
            .expect("thermo")
            .system()
            .components
            .iter()
            .map(|c| c.id.clone())
            .collect();
        let unique: BTreeSet<_> = ids.iter().cloned().collect();
        if unique.len() != ids.len() || unique.iter().ne(self.plan.components.iter()) {
            return Err(RunError {
                kind: ModularFailureKind::Initialization,
                error: failure(
                    "thermo_component_binding",
                    "thermo and document components must match exactly",
                ),
            });
        }
        let mut templates = self.plan.flowsheet.streams.clone();
        let mut total = 0.0;
        for unit in self.plan.flowsheet.units.values() {
            self.at(ModularStage::Initializing, Some(unit.id.clone()), None)?;
            if unit.kind == "feed" {
                let id = classify(
                    ModularFailureKind::Initialization,
                    crate::port_stream_id(unit, "outlet"),
                )?;
                let s = templates.get_mut(id).expect("validated stream");
                if let Some(t) = unit.parameters.outlet_temperature_k {
                    s.temperature_k = t;
                }
                if let Some(p) = unit.parameters.outlet_pressure_pa {
                    s.pressure_pa = p;
                }
                s.phases.clear();
                s.bubble_dew_window = None;
                classify(
                    ModularFailureKind::Boundary,
                    material::component_flows(&self.plan.components, s, self.plan.options.boundary),
                )?;
                for c in &self.plan.components {
                    s.overall_mole_fractions.entry(c.clone()).or_insert(0.0);
                }
                total += s.total_molar_flow_mol_s;
            }
            self.costs.operation_constructions += 1;
            self.operations.insert(
                unit.id.clone(),
                classify(
                    ModularFailureKind::Initialization,
                    instantiate_builtin(unit, &templates),
                )?,
            );
        }
        if !total.is_finite() || total <= 0.0 {
            return Err(RunError {
                kind: ModularFailureKind::Initialization,
                error: failure(
                    "nonpositive_fresh_flow",
                    "positive finite fresh inflow required",
                ),
            });
        }
        self.global_scales = self
            .plan
            .components
            .iter()
            .cloned()
            .map(|id| (id, total))
            .collect();
        Ok(())
    }
    pub(super) fn execute(&mut self) -> RunResult<()> {
        for (i, block) in self.plan.blocks.iter().enumerate() {
            self.progress.block = Some(i);
            self.active_units.clear();
            self.active_coupling = None;
            self.at(ModularStage::BlockStarted, None, None)?;
            let report = if block.couplings.is_empty() {
                let scales = self.global_scales.clone();
                let unit = self.evaluate_unit(&block.members[0], &BTreeMap::new(), &scales)?;
                ModularBlockReport {
                    members: block.members.clone(),
                    final_units: vec![unit],
                    coupling: None,
                }
            } else {
                self.coupled(block)?
            };
            self.blocks.push(report);
            self.active_coupling = None;
            self.at(ModularStage::BlockCompleted, None, None)?;
        }
        self.at(ModularStage::Completing, None, None)?;
        let feeds: Vec<_> = self
            .plan
            .connections
            .iter()
            .filter(|c| self.plan.flowsheet.units[&c.source.unit_id].kind == "feed")
            .map(|c| self.streams[&c.stream_id].clone())
            .collect();
        let products: Vec<_> = self
            .plan
            .connections
            .iter()
            .filter(|c| c.sink.is_none())
            .map(|c| self.streams[&c.stream_id].clone())
            .collect();
        classify(
            ModularFailureKind::Balance,
            self.verify_balance(&feeds, &products, &self.global_scales),
        )
    }
    fn coupled(&mut self, block: &ModularBlock) -> RunResult<ModularBlockReport> {
        let mut fresh_total = 0.0;
        let mut zero_rules = BTreeMap::new();
        for id in block.fresh_by_mixer.values() {
            let fresh = self.streams.get(id).ok_or_else(|| RunError {
                kind: ModularFailureKind::Initialization,
                error: failure("missing_fresh", "fresh boundary was not evaluated")
                    .with_related_stream_id(id.clone()),
            })?;
            if !fresh.total_molar_flow_mol_s.is_finite() || fresh.total_molar_flow_mol_s <= 0.0 {
                return Err(RunError {
                    kind: ModularFailureKind::Initialization,
                    error: failure(
                        "nonpositive_fresh_flow",
                        "each cyclic Mixer requires positive fresh boundary",
                    )
                    .with_related_stream_id(id.clone()),
                });
            }
            fresh_total += fresh.total_molar_flow_mol_s;
        }
        if !fresh_total.is_finite() {
            return Err(RunError {
                kind: ModularFailureKind::Boundary,
                error: failure("flow_overflow", "block fresh total overflows"),
            });
        }
        for c in &block.couplings {
            let mixer = if self.plan.flowsheet.units[&c.sink.unit_id].kind == "mixer" {
                &c.sink.unit_id
            } else {
                &c.source.unit_id
            };
            zero_rules.insert(c.stream.clone(), block.fresh_by_mixer[mixer].clone());
        }
        let scales: Vec<_> = block
            .couplings
            .iter()
            .map(|c| {
                self.plan
                    .options
                    .scales
                    .as_ref()
                    .map(|s| s[c].clone())
                    .unwrap_or(CouplingScale {
                        variable_mol_s: fresh_total,
                        residual_mol_s: fresh_total,
                    })
            })
            .collect();
        let local_scales: BTreeMap<_, _> = self
            .plan
            .components
            .iter()
            .cloned()
            .map(|c| (c, fresh_total))
            .collect();
        self.active_coupling = Some(CoupledEvidence {
            boundary: self.plan.options.boundary,
            response_version: rf_unitops::component_response::RESPONSE_VERSION,
            acceptance_version: ACCEPTANCE_VERSION,
            responses: block.responses.clone(),
            zero_flow_fresh_boundaries: zero_rules.clone(),
            anchor_residuals_mol_s: Vec::new(),
            connections: Vec::new(),
            linear: None,
        });
        if block
            .members
            .iter()
            .filter(|id| self.plan.flowsheet.units[*id].kind == "splitter")
            .all(|id| {
                self.plan.flowsheet.units[id]
                    .parameters
                    .outlet_a_split_fraction
                    == Some(1.0)
            })
        {
            return Err(RunError {
                kind: ModularFailureKind::NoFiniteSteadyState,
                error: failure(
                    "no_finite_steady_state",
                    "all product fractions zero with positive fresh inflow",
                )
                .with_related_unit_ids(block.members.clone()),
            });
        }
        self.at(ModularStage::Anchor, None, None)?;
        self.costs.anchor_evaluations += 1;
        let anchor_start = Instant::now();
        let anchor = self.evaluate_couplings(
            block,
            &vec![0.0; block.couplings.len()],
            &zero_rules,
            &local_scales,
        );
        self.costs.anchor_elapsed += anchor_start.elapsed();
        let (r0, _) = anchor?;
        self.active_coupling
            .as_mut()
            .expect("active")
            .anchor_residuals_mol_s = r0.clone();
        let response_start = Instant::now();
        let matrix = self.jacobian(block, &scales);
        self.costs.response_elapsed += response_start.elapsed();
        let matrix = matrix?;
        let rhs: Vec<_> = r0
            .iter()
            .zip(&scales)
            .map(|(r, s)| -r / s.residual_mol_s)
            .collect();
        let linear_start = Instant::now();
        let mut counts = ModularCosts::default();
        let mut diagnostic = None;
        let linear = super::dense::solve(
            &matrix,
            &rhs,
            &mut counts,
            &mut diagnostic,
            &mut |stage, column| self.at(stage, None, Some(column)),
        );
        self.costs.linear_elapsed += linear_start.elapsed();
        self.costs.factorizations += counts.factorizations;
        self.costs.triangular_solves += counts.triangular_solves;
        self.active_coupling.as_mut().expect("active").linear = diagnostic;
        let linear = linear?;
        self.active_coupling.as_mut().expect("active").linear = Some(linear.evidence);
        let candidate: Vec<_> = linear
            .values
            .iter()
            .zip(&scales)
            .map(|(x, s)| x * s.variable_mol_s)
            .collect();
        self.at(ModularStage::Verification, None, None)?;
        self.costs.verification_evaluations += 1;
        let verification_start = Instant::now();
        let result = self.evaluate_couplings(block, &candidate, &zero_rules, &local_scales);
        self.costs.verification_elapsed += verification_start.elapsed();
        let (residuals, units) = result?;
        let evidence: Vec<_> = block
            .couplings
            .iter()
            .zip(scales)
            .enumerate()
            .map(|(i, (binding, scale))| {
                let output = &self.streams[&binding.stream];
                ConnectionEvidence {
                    binding: binding.clone(),
                    assumed_mol_s: candidate[i],
                    produced_mol_s: output.total_molar_flow_mol_s
                        * output
                            .overall_mole_fractions
                            .get(&binding.component)
                            .copied()
                            .unwrap_or(0.0),
                    residual_mol_s: residuals[i],
                    scaled_residual: residuals[i] / scale.residual_mol_s,
                    scale,
                }
            })
            .collect();
        let accepted = evidence
            .iter()
            .all(|e| e.residual_mol_s.abs() <= 1e-10 && e.scaled_residual.abs() <= 1e-10);
        self.active_coupling.as_mut().expect("active").connections = evidence;
        if !accepted {
            return Err(RunError {
                kind: ModularFailureKind::Verification,
                error: failure(
                    "connection_verification",
                    "independent module evaluation failed coupling tolerances",
                )
                .with_related_stream_ids(
                    block.couplings.iter().map(|c| c.stream.clone()).collect(),
                ),
            });
        }
        let inputs: Vec<_> = block
            .fresh_by_mixer
            .values()
            .map(|s| self.streams[s].clone())
            .collect();
        let products: Vec<_> = block
            .products
            .iter()
            .map(|s| self.streams[s].clone())
            .collect();
        classify(
            ModularFailureKind::Balance,
            self.verify_balance(&inputs, &products, &local_scales),
        )?;
        Ok(ModularBlockReport {
            members: block.members.clone(),
            final_units: units,
            coupling: self.active_coupling.clone(),
        })
    }
    pub(super) fn jacobian(
        &mut self,
        block: &ModularBlock,
        scales: &[CouplingScale],
    ) -> RunResult<Vec<Vec<f64>>> {
        use rf_unitops::component_response::ComponentPort;
        let n = block.couplings.len();
        let mut a = vec![vec![0.0; n]; n];
        let mut used_responses = BTreeSet::new();
        for (j, input) in block.couplings.iter().enumerate() {
            self.at(
                ModularStage::Response,
                Some(input.sink.unit_id.clone()),
                Some(j),
            )?;
            for (i, out) in block.couplings.iter().enumerate() {
                let mut derivative = if i == j { 1.0 } else { 0.0 };
                if input.sink.unit_id == out.source.unit_id {
                    if used_responses.insert(out.source.unit_id.clone()) {
                        self.costs.response_evaluations += 1;
                    }
                    let response = block
                        .responses
                        .iter()
                        .find(|r| r.unit_id() == &out.source.unit_id)
                        .expect("assembled response");
                    derivative -= classify(
                        ModularFailureKind::Response,
                        response.derivative(
                            &ComponentPort {
                                port: out.source.port_name.clone(),
                                component: out.component.clone(),
                            },
                            &ComponentPort {
                                port: input.sink.port_name.clone(),
                                component: input.component.clone(),
                            },
                        ),
                    )?;
                }
                a[i][j] = derivative * scales[j].variable_mol_s / scales[i].residual_mol_s;
                if !a[i][j].is_finite() {
                    return Err(RunError {
                        kind: ModularFailureKind::Response,
                        error: failure(
                            "scaled_derivative_overflow",
                            "scaled component response is nonfinite",
                        )
                        .with_related_unit_id(out.source.unit_id.clone())
                        .with_related_stream_id(out.stream.clone())
                        .with_related_component_id(out.component.clone()),
                    });
                }
            }
        }
        Ok(a)
    }
}
