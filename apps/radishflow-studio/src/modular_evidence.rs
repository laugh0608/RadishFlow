//! Read-only text projection of captured QS2 evidence, shared by GUI and export.
use rf_solver::simultaneous::*;
use rf_types::units::{DisplayUnitSet, QuantityKind};

use crate::result_evidence_text::Projection;
pub use crate::result_evidence_text::{EvidenceLines, diagnostic_number};
impl Projection<'_> {
    fn coupling(&mut self, e: &CoupledEvidence) {
        let temperature = self.quantity(
            "boundary",
            QuantityKind::AbsoluteTemperature,
            e.boundary.temperature_k,
        );
        let pressure = self.quantity(
            "boundary",
            QuantityKind::AbsolutePressure,
            e.boundary.pressure_pa,
        );
        self.output.lines.push(format!("Fixed boundary: T {temperature}; P {pressure}; response contract {}; acceptance contract {}",e.response_version,e.acceptance_version));
        for c in &e.connections {
            let b = &c.binding;
            self.output.lines.push(format!(
                "{} / {}: {}.{} → {}.{}",
                b.stream,
                b.component,
                b.source.unit_id,
                b.source.port_name,
                b.sink.unit_id,
                b.sink.port_name
            ));
            let assumed = self.flow(b.stream.as_str(), c.assumed_mol_s);
            let produced = self.flow(b.stream.as_str(), c.produced_mol_s);
            let residual = self.flow(b.stream.as_str(), c.residual_mol_s);
            let variable = self.flow(b.stream.as_str(), c.scale.variable_mol_s);
            let scale = self.flow(b.stream.as_str(), c.scale.residual_mol_s);
            self.output
                .lines
                .push(format!("Assumed: {assumed} | Produced: {produced}"));
            self.output.lines.push(format!(
                "Residual: {residual}; scaled residual: {} (dimensionless)",
                diagnostic_number(Some(c.scaled_residual))
            ));
            self.output.lines.push(format!(
                "Variable scale: {variable}; residual scale: {scale}"
            ));
        }
        if let Some(linear) = &e.linear {
            self.output.lines.push(format!(
                "LU dimension: {}; matrix infinity norm: {}; pivot threshold: {}; row swaps: {:?}",
                linear.dimension,
                diagnostic_number(Some(linear.matrix_inf_norm)),
                diagnostic_number(Some(linear.pivot_threshold)),
                linear.row_swaps
            ));
            self.output.lines.push(format!(
                "rcond_indicator: {}; backward_error: {} (dimensionless)",
                diagnostic_number(linear.rcond_indicator),
                diagnostic_number(linear.backward_error)
            ));
        } else {
            self.output
                .lines
                .push("LU / rcond_indicator / backward_error: 未计算 / not computed".into());
        }
        for (index, residual) in e.anchor_residuals_mol_s.iter().enumerate() {
            let residual = self.flow("anchor", *residual);
            self.output
                .lines
                .push(format!("Anchor residual [{index}]: {residual}"));
        }
        for (stream, fresh) in &e.zero_flow_fresh_boundaries {
            self.output
                .lines
                .push(format!("Zero-flow composition source: {stream} ← {fresh}"));
        }
        for r in &e.responses {
            self.output.lines.push(format!(
                "Response: {} / {:?}; inputs {:?}; outputs {:?}",
                r.unit_id(),
                r.capability(),
                r.inputs(),
                r.outputs()
            ));
            for output in r.outputs() {
                for input in r.inputs() {
                    let value = match r.derivative(output, input) {
                        Ok(v) => diagnostic_number(Some(v)),
                        Err(e) => format!("Unavailable: {e}"),
                    };
                    self.output.lines.push(format!(
                        "d({}.{}) / d({}.{}) = {value} (dimensionless)",
                        output.port, output.component, input.port, input.component
                    ));
                }
            }
        }
    }
}
pub fn cost_lines(c: &ModularCosts) -> Vec<String> {
    vec![
        format!(
            "Anchor: {}; responses: {}; LU factorizations: {}; triangular solves: {}; independent verification: {}",
            c.anchor_evaluations,
            c.response_evaluations,
            c.factorizations,
            c.triangular_solves,
            c.verification_evaluations
        ),
        format!(
            "Constructions: {}; local calls: {}; thermo: {}; flash: {}; calls by unit: {:?}",
            c.operation_constructions,
            c.local_unit_calls,
            c.thermo_calls,
            c.flash_calls,
            c.calls_by_unit
        ),
        format!(
            "Measured seconds: assembly {:.6}; initialization {:.6}; anchor {:.6}; response {:.6}; linear {:.6}; verification {:.6}; total {:.6}",
            c.assembly_elapsed.as_secs_f64(),
            c.initialization_elapsed.as_secs_f64(),
            c.anchor_elapsed.as_secs_f64(),
            c.response_elapsed.as_secs_f64(),
            c.linear_elapsed.as_secs_f64(),
            c.verification_elapsed.as_secs_f64(),
            c.elapsed.as_secs_f64()
        ),
    ]
}
pub fn coupling_lines(e: &CoupledEvidence, units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    p.coupling(e);
    p.output
}
pub fn local_unit_lines(e: &[LocalUnitEvaluation], units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    p.local_units(e);
    p.output
}
pub fn result_lines(result: &ModularResult, units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    p.output
        .lines
        .push(format!("Simultaneous modular result: {}", result.method));
    for (index, b) in result.blocks.iter().enumerate() {
        p.output
            .lines
            .push(format!("Block {}: {:?}", index + 1, b.members));
        if let Some(e) = &b.coupling {
            p.coupling(e);
        }
        p.local_units(&b.final_units);
    }
    p.output.lines.extend(cost_lines(&result.costs));
    p.output
}
pub fn failure_lines(failure: &ModularFailure, units: &DisplayUnitSet) -> EvidenceLines {
    let mut p = Projection::new(units);
    p.output.lines.push(format!(
        "Diagnostic evidence only: {:?}; {}",
        failure.kind, failure.error
    ));
    p.output.lines.push(format!(
        "Stage: {:?}; block {:?}; unit {:?}; column {:?}",
        failure.progress.stage,
        failure.progress.block.map(|b| b + 1),
        failure.progress.unit,
        failure.progress.column
    ));
    for (index, b) in failure.completed_blocks.iter().enumerate() {
        p.output.lines.push(format!(
            "Diagnostic completed block {}: {:?}",
            index + 1,
            b.members
        ));
        if let Some(e) = &b.coupling {
            p.coupling(e);
        }
        p.local_units(&b.final_units);
    }
    if let Some(e) = &failure.active_coupling {
        p.coupling(e);
    } else {
        p.output
            .lines
            .push("Active coupling / LU: 未计算 / not computed".into());
    }
    p.local_units(&failure.active_units);
    p.output.lines.extend(cost_lines(&failure.costs));
    p.output
}
