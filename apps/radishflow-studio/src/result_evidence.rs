//! Shared numeric and local I/O projection for captured solver evidence.
use rf_solver::partitioned::LocalUnitEvaluation;
use rf_types::{RfError, RfResult};
use serde_json::{Value, json};
#[derive(Clone, Copy)]
pub(crate) enum Numbers {
    Current,
    Diagnostic,
}

pub(crate) fn diagnostic_number(value: f64) -> Value {
    if value.is_finite() {
        json!(value)
    } else {
        let kind = if value.is_nan() {
            "nan"
        } else if value.is_sign_positive() {
            "positive_infinity"
        } else {
            "negative_infinity"
        };
        json!({ "non_finite": kind })
    }
}

impl Numbers {
    pub(crate) fn value(self, value: f64) -> RfResult<Value> {
        if matches!(self, Self::Current) && !value.is_finite() {
            Err(
                RfError::invalid_input("nonfinite value in formal flow result")
                    .with_diagnostic_code("workspace.result.nonfinite"),
            )
        } else {
            Ok(diagnostic_number(value))
        }
    }
    pub(crate) fn optional(self, value: Option<f64>) -> RfResult<Value> {
        value.map(|v| self.value(v)).unwrap_or(Ok(Value::Null))
    }
    pub(crate) fn vector(self, values: &[f64]) -> RfResult<Vec<Value>> {
        values.iter().map(|v| self.value(*v)).collect()
    }
}

pub(crate) fn stream(s: &rf_model::MaterialStreamState, n: Numbers) -> RfResult<Value> {
    let components = s.overall_mole_fractions.iter().map(|(id,x)| Ok(json!({"component_id":id.as_str(),"mole_fraction":n.value(*x)?,"flow_mol_s":n.value(*x*s.total_molar_flow_mol_s)?}))).collect::<RfResult<Vec<_>>>()?;
    let phases = s.phases.iter().map(|p| {
        let composition = p.mole_fractions.iter().map(|(id,x)| Ok((id.to_string(),n.value(*x)?))).collect::<RfResult<serde_json::Map<_,_>>>()?;
        Ok(json!({"label":p.label,"phase_fraction":n.value(p.phase_fraction)?,"composition":composition,"molar_enthalpy_j_per_mol":n.optional(p.molar_enthalpy_j_per_mol)?}))
    }).collect::<RfResult<Vec<_>>>()?;
    Ok(
        json!({"id":s.id.as_str(),"name":s.name,"temperature_k":n.value(s.temperature_k)?,"pressure_pa":n.value(s.pressure_pa)?,"total_molar_flow_mol_s":n.value(s.total_molar_flow_mol_s)?,"components":components,"phases":phases}),
    )
}
pub(crate) fn units(units: &[LocalUnitEvaluation], n: Numbers) -> RfResult<Vec<Value>> {
    units.iter().map(|u| Ok(json!({"unit_id":u.unit_id.as_str(),
        "consumed":u.consumed_streams.iter().map(|s|stream(s,n)).collect::<RfResult<Vec<_>>>()?,
        "produced":u.produced_streams.iter().map(|s|stream(s,n)).collect::<RfResult<Vec<_>>>()?
    }))).collect()
}
