//! Read-only text projection of captured QS2 evidence, shared by GUI and export.
use crate::StudioGuiQuantityPresentationError;
use rf_model::MaterialStreamState;
use rf_solver::simultaneous::*;
use rf_types::units::{DisplayUnitSet, QuantityKind, from_canonical};

#[derive(Default)]
pub struct EvidenceLines {
    pub lines: Vec<String>,
    pub errors: Vec<StudioGuiQuantityPresentationError>,
}
struct Projection<'a> {
    units: &'a DisplayUnitSet,
    output: EvidenceLines,
}
impl<'a> Projection<'a> {
    fn new(units: &'a DisplayUnitSet) -> Self {
        Self {
            units,
            output: EvidenceLines::default(),
        }
    }
    fn quantity(&mut self, id: &str, kind: QuantityKind, value: f64) -> String {
        if !value.is_finite() {
            return diagnostic_number(Some(value));
        }
        let unit = self.units.unit_for(kind);
        match from_canonical(value, kind, unit) {
            Ok(value) => format!("{value:.9e} {}", unit.definition().symbol),
            Err(cause) => {
                let error = StudioGuiQuantityPresentationError {
                    stream_id: id.into(),
                    quantity: kind,
                    unit,
                    cause,
                };
                let text = format!("Conversion failed: {error}");
                self.output.errors.push(error);
                text
            }
        }
    }
    fn flow(&mut self, id: &str, value: f64) -> String {
        self.quantity(id, QuantityKind::MolarFlow, value)
    }
    fn state(&mut self, role: &str, stream: &MaterialStreamState) {
        let temperature = self.quantity(
            stream.id.as_str(),
            QuantityKind::AbsoluteTemperature,
            stream.temperature_k,
        );
        let pressure = self.quantity(
            stream.id.as_str(),
            QuantityKind::AbsolutePressure,
            stream.pressure_pa,
        );
        let flow = self.flow(stream.id.as_str(), stream.total_molar_flow_mol_s);
        self.output.lines.push(format!(
            "{role} {}: T {temperature}; P {pressure}; F {flow}",
            stream.id
        ));
        for (component, fraction) in &stream.overall_mole_fractions {
            let value = self.flow(stream.id.as_str(), fraction * stream.total_molar_flow_mol_s);
            self.output.lines.push(format!("  {component}: {value}"));
        }
    }
    fn local_units(&mut self, units: &[LocalUnitEvaluation]) {
        for unit in units {
            self.output
                .lines
                .push(format!("Local I/O: {}", unit.unit_id));
            for stream in &unit.consumed_streams {
                self.state("Consumed", stream);
            }
            for stream in &unit.produced_streams {
                self.state("Produced", stream);
            }
        }
    }
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
/// Missing is distinct from a computed non-finite diagnostic value.
pub fn diagnostic_number(value: Option<f64>) -> String {
    match value {
        None => "未计算 / not computed".into(),
        Some(v) if v.is_nan() => "NaN (非有限 / non-finite)".into(),
        Some(v) if v == f64::INFINITY => "+Infinity (正无穷 / non-finite)".into(),
        Some(v) if v == f64::NEG_INFINITY => "-Infinity (负无穷 / non-finite)".into(),
        Some(v) => format!("{v:.9e}"),
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
