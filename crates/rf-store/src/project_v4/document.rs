//! Strict transport shape checks precede domain decoding. Original duplicate keys
//! have already been rejected; no opaque/unsupported payload reaches this path.
use super::*;
use serde_json::Value;

pub(super) fn object<'a>(
    value: &'a Value,
    fields: &[&str],
    path: &str,
) -> RfResult<&'a serde_json::Map<String, Value>> {
    let map = value
        .as_object()
        .ok_or_else(|| RfError::invalid_input(format!("{path}: expected object")))?;
    if let Some(key) = map.keys().find(|k| !fields.contains(&k.as_str())) {
        return Err(RfError::invalid_input(format!(
            "{path}.{key}: unknown field"
        )));
    }
    Ok(map)
}
fn array<'a>(value: &'a Value, path: &str) -> RfResult<&'a Vec<Value>> {
    value
        .as_array()
        .ok_or_else(|| RfError::invalid_input(format!("{path}: expected array")))
}
pub(super) fn decode(value: &Value) -> RfResult<StoredProjectDocument> {
    object(value, &["revision", "flowsheet", "metadata"], "document")?;
    object(
        &value["metadata"],
        &[
            "documentId",
            "title",
            "schemaVersion",
            "createdAt",
            "updatedAt",
        ],
        "document.metadata",
    )?;
    let flow = &value["flowsheet"];
    object(
        flow,
        &["name", "thermo", "components", "streams", "units"],
        "document.flowsheet",
    )?;
    if let Some(thermo) = flow.get("thermo") {
        object(thermo, &["property_package_id"], "flowsheet.thermo")?;
    }
    let entries = |name: &str| {
        flow[name].as_object().ok_or_else(|| {
            RfError::invalid_input(format!("flowsheet.{name}: expected identity map"))
        })
    };
    for component in entries("components")?.values() {
        object(component, &["id", "name", "formula"], "component")?;
    }
    for stream in entries("streams")?.values() {
        object(
            stream,
            &[
                "id",
                "name",
                "temperature_k",
                "pressure_pa",
                "total_molar_flow_mol_s",
                "overall_mole_fractions",
                "phases",
                "bubble_dew_window",
            ],
            "stream",
        )?;
        for phase in array(&stream["phases"], "stream.phases")? {
            object(
                phase,
                &[
                    "label",
                    "phase_fraction",
                    "mole_fractions",
                    "molar_enthalpy_j_per_mol",
                ],
                "phase",
            )?;
        }
        if let Some(window) = stream.get("bubble_dew_window").filter(|v| !v.is_null()) {
            object(
                window,
                &[
                    "phase_region",
                    "bubble_pressure_pa",
                    "dew_pressure_pa",
                    "bubble_temperature_k",
                    "dew_temperature_k",
                ],
                "bubble_dew_window",
            )?;
        }
    }
    for unit in entries("units")?.values() {
        object(unit, &["id", "name", "kind", "ports", "parameters"], "unit")?;
        for port in array(&unit["ports"], "unit.ports")? {
            object(
                port,
                &["name", "direction", "kind", "stream_id"],
                "unit.port",
            )?;
        }
        if let Some(parameters) = unit.get("parameters") {
            object(
                parameters,
                &[
                    "outlet_temperature_k",
                    "outlet_pressure_pa",
                    "outlet_a_split_fraction",
                ],
                "unit.parameters",
            )?;
        }
    }
    let document: StoredProjectDocument =
        serde_json::from_value(value.clone()).map_err(json_error)?;
    validate(&document)?;
    Ok(document)
}
pub(super) fn validate(document: &StoredProjectDocument) -> RfResult<()> {
    document.validate_common()?;
    let flow = &document.flowsheet;
    for (id, component) in &flow.components {
        if id != &component.id || id.as_str().trim().is_empty() {
            return Err(RfError::invalid_input(
                "component map identity mismatch or empty identity",
            ));
        }
    }
    for (id, stream) in &flow.streams {
        if id != &stream.id || id.as_str().trim().is_empty() {
            return Err(RfError::invalid_input(
                "stream map identity mismatch or empty identity",
            ));
        }
        let mut numbers = vec![
            stream.temperature_k,
            stream.pressure_pa,
            stream.total_molar_flow_mol_s,
        ];
        numbers.extend(stream.overall_mole_fractions.values());
        for phase in &stream.phases {
            numbers.push(phase.phase_fraction);
            numbers.extend(phase.mole_fractions.values());
            numbers.extend(phase.molar_enthalpy_j_per_mol);
        }
        if let Some(w) = &stream.bubble_dew_window {
            numbers.extend([
                w.bubble_pressure_pa,
                w.dew_pressure_pa,
                w.bubble_temperature_k,
                w.dew_temperature_k,
            ]);
        }
        if numbers.iter().any(|v| !v.is_finite()) {
            return Err(RfError::invalid_input("stream data must be finite")
                .with_related_stream_id(id.clone()));
        }
    }
    for (id, unit) in &flow.units {
        if id != &unit.id || id.as_str().trim().is_empty() {
            return Err(RfError::invalid_input(
                "unit map identity mismatch or empty identity",
            ));
        }
        if ![
            "feed",
            "mixer",
            "heater",
            "cooler",
            "valve",
            "flash_drum",
            "splitter",
        ]
        .contains(&unit.kind.as_str())
        {
            return Err(RfError::invalid_input("undeclared unit model capability")
                .with_related_unit_id(id.clone()));
        }
        let p = &unit.parameters;
        if [
            p.outlet_temperature_k,
            p.outlet_pressure_pa,
            p.outlet_a_split_fraction,
        ]
        .into_iter()
        .flatten()
        .any(|v| !v.is_finite())
        {
            return Err(RfError::invalid_input("unit parameters must be finite")
                .with_related_unit_id(id.clone()));
        }
        if p.outlet_a_split_fraction
            .is_some_and(|v| unit.kind != "splitter" || !(0.0..=1.0).contains(&v))
        {
            return Err(RfError::invalid_input(
                "split fraction requires Splitter and a value in [0,1]",
            )
            .with_related_unit_id(id.clone()));
        }
    }
    Ok(())
}
