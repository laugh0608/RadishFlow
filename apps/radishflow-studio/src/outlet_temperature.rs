//! Session-local evidence from one connected material outlet of a manual acyclic run.
use rf_model::Flowsheet;
use rf_types::units::{MeasurementUnit, QuantityKind, from_canonical};
use rf_types::{PortDirection, PortKind, StreamId};
use rf_ui::{
    FlowExecutionQualification, InputUnitOrigin, NumericParseOutcome, StreamPortBinding,
    WorkspaceState, parse_numeric_input,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialOutlet {
    pub source: StreamPortBinding,
    pub stream: StreamId,
}
impl MaterialOutlet {
    pub fn connected(flowsheet: &Flowsheet) -> impl Iterator<Item = Self> + '_ {
        flowsheet.units.values().flat_map(move |unit| {
            unit.ports.iter().filter_map(move |port| {
                let stream = port.stream_id.as_ref()?;
                (port.direction == PortDirection::Outlet
                    && port.kind == PortKind::Material
                    && flowsheet.streams.contains_key(stream))
                .then(|| Self {
                    source: StreamPortBinding {
                        unit_id: unit.id.clone(),
                        port: port.name.clone(),
                    },
                    stream: stream.clone(),
                })
            })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PositiveKelvin(f64);
// Construction excludes NaN, infinity, and both zero representations.
impl Eq for PositiveKelvin {}
impl PositiveKelvin {
    fn new(value: f64) -> Option<Self> {
        (value.is_finite() && value > 0.).then_some(Self(value))
    }
    pub fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemperatureThreshold {
    raw: String,
    unit: MeasurementUnit,
    kelvin: PositiveKelvin,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdError {
    Incomplete,
    Invalid,
    AbsoluteZero,
}
impl TemperatureThreshold {
    pub fn parse(raw: &str, unit: MeasurementUnit) -> Result<Self, ThresholdError> {
        match parse_numeric_input(
            raw,
            QuantityKind::AbsoluteTemperature,
            unit,
            InputUnitOrigin::Explicit,
        ) {
            NumericParseOutcome::Value { si, .. } => Ok(Self {
                raw: raw.to_owned(),
                unit,
                kelvin: PositiveKelvin::new(si).ok_or(ThresholdError::AbsoluteZero)?,
            }),
            NumericParseOutcome::Incomplete => Err(ThresholdError::Incomplete),
            NumericParseOutcome::Invalid(_) => Err(ThresholdError::Invalid),
        }
    }
    pub fn in_unit(&self, unit: MeasurementUnit) -> Result<Self, ThresholdError> {
        let value = from_canonical(self.kelvin.value(), QuantityKind::AbsoluteTemperature, unit)
            .map_err(|_| ThresholdError::Invalid)?;
        Ok(Self {
            raw: value.to_string(),
            unit,
            kelvin: self.kelvin,
        })
    }
    pub fn raw(&self) -> &str {
        &self.raw
    }
    pub fn unit(&self) -> MeasurementUnit {
        self.unit
    }
    pub fn kelvin(&self) -> PositiveKelvin {
        self.kelvin
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemperatureCondition {
    pub expected: FlowExecutionQualification,
    pub outlet: MaterialOutlet,
    pub threshold: TemperatureThreshold,
}
impl TemperatureCondition {
    pub fn is_current(&self, workspace: &WorkspaceState) -> bool {
        workspace.flow_plan.current() == &rf_ui::flow_plan::FlowPlanConfiguration::Acyclic
            && self
                .expected
                .same_input(&workspace.flow_execution_qualification())
            && MaterialOutlet::connected(&workspace.document.flowsheet)
                .any(|target| target == self.outlet)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnavailableTemperature {
    MissingStream,
    InvalidTemperature,
    TargetNotReached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureReading {
    Pending,
    Measured {
        step: usize,
        actual: PositiveKelvin,
    },
    Unavailable {
        step: Option<usize>,
        reason: UnavailableTemperature,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemperatureObservation {
    pub condition: TemperatureCondition,
    pub total: usize,
    pub reading: TemperatureReading,
}
impl TemperatureObservation {
    pub(crate) fn new(condition: TemperatureCondition) -> Self {
        Self {
            condition,
            total: 0,
            reading: TemperatureReading::Pending,
        }
    }
    pub(crate) fn observe(&mut self, event: &rf_solver::SequentialExecutionEvent<'_>) {
        use rf_solver::SequentialExecutionEvent::*;
        if self.reading != TemperatureReading::Pending {
            return;
        }
        match event {
            Planned(units) => self.total = units.len(),
            StepCompleted { number, step }
                if step.unit_id == self.condition.outlet.source.unit_id =>
            {
                let measured = step
                    .produced_streams
                    .iter()
                    .find(|s| s.id == self.condition.outlet.stream)
                    .ok_or(UnavailableTemperature::MissingStream)
                    .and_then(|s| {
                        PositiveKelvin::new(s.temperature_k)
                            .ok_or(UnavailableTemperature::InvalidTemperature)
                    });
                self.reading = match measured {
                    Ok(actual) => TemperatureReading::Measured {
                        step: *number,
                        actual,
                    },
                    Err(reason) => TemperatureReading::Unavailable {
                        step: Some(*number),
                        reason,
                    },
                };
            }
            _ => {}
        }
    }
}
