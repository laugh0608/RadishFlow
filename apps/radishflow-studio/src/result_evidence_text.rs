//! Display-unit conversion shared by evidence consumers.
use crate::StudioGuiQuantityPresentationError;
use rf_model::MaterialStreamState;
use rf_solver::partitioned::LocalUnitEvaluation;
use rf_types::units::{DisplayUnitSet, QuantityKind, from_canonical};
#[derive(Default)]
pub struct EvidenceLines {
    pub lines: Vec<String>,
    pub errors: Vec<StudioGuiQuantityPresentationError>,
}
pub(crate) struct Projection<'a> {
    units: &'a DisplayUnitSet,
    pub(crate) output: EvidenceLines,
}
impl<'a> Projection<'a> {
    pub(crate) fn new(units: &'a DisplayUnitSet) -> Self {
        Self {
            units,
            output: EvidenceLines::default(),
        }
    }
    pub(crate) fn quantity(&mut self, id: &str, kind: QuantityKind, value: f64) -> String {
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
    pub(crate) fn flow(&mut self, id: &str, value: f64) -> String {
        self.quantity(id, QuantityKind::MolarFlow, value)
    }
    pub(crate) fn state(&mut self, role: &str, stream: &MaterialStreamState) {
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
    pub(crate) fn local_units(&mut self, units: &[LocalUnitEvaluation]) {
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
