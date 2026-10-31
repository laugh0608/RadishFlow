//! Affine component-molar-flow responses at fixed T/P and physical parameters.
//! No enthalpy, phase, thermo derivative or solver index is part of this contract.
use crate::*;
use rf_types::ComponentId;

pub const RESPONSE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffineComponentCapability {
    TwoInletMixing,
    NonselectiveSplitting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentPort {
    pub port: String,
    pub component: ComponentId,
}

/// An immutable declaration of the selected complete input/output mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct AffineComponentResponse {
    unit_id: UnitId,
    capability: AffineComponentCapability,
    temperature_k: f64,
    pressure_pa: f64,
    inputs: Vec<ComponentPort>,
    outputs: Vec<ComponentPort>,
    derivatives_dimensionless: Vec<Vec<f64>>,
}

impl AffineComponentResponse {
    pub fn unit_id(&self) -> &UnitId {
        &self.unit_id
    }
    pub fn capability(&self) -> AffineComponentCapability {
        self.capability
    }
    pub fn temperature_k(&self) -> f64 {
        self.temperature_k
    }
    pub fn pressure_pa(&self) -> f64 {
        self.pressure_pa
    }
    pub fn inputs(&self) -> &[ComponentPort] {
        &self.inputs
    }
    pub fn outputs(&self) -> &[ComponentPort] {
        &self.outputs
    }
    pub fn for_unit(
        unit: &UnitNode,
        temperature_k: f64,
        pressure_pa: f64,
        components: &[ComponentId],
    ) -> RfResult<Self> {
        let invalid = || {
            RfError::invalid_input("fixed-T/P affine response is unsupported or invalid")
                .with_diagnostic_code("unitops.component_response.unsupported")
                .with_related_unit_id(unit.id.clone())
        };
        let spec = validate_unit_node(unit)?;
        let capability = catalog::builtin_model(spec.kind)
            .component_response_capability()
            .ok_or_else(invalid)?;
        if !temperature_k.is_finite()
            || temperature_k <= 0.0
            || !pressure_pa.is_finite()
            || pressure_pa <= 0.0
            || components.is_empty()
            || components.iter().any(|id| id.as_str().trim().is_empty())
            || components.windows(2).any(|p| p[0] >= p[1])
            || unit.parameters.outlet_temperature_k.is_some()
        {
            return Err(invalid());
        }
        match capability {
            AffineComponentCapability::TwoInletMixing => {
                if unit.parameters.outlet_pressure_pa != Some(pressure_pa)
                    || unit.parameters.outlet_a_split_fraction.is_some()
                {
                    return Err(invalid());
                }
            }
            AffineComponentCapability::NonselectiveSplitting => {
                if unit.parameters.outlet_pressure_pa.is_some() {
                    return Err(invalid());
                }
                crate::splitter::validate_fraction(
                    unit.parameters
                        .outlet_a_split_fraction
                        .ok_or_else(invalid)?,
                )
                .map_err(|e| e.with_related_unit_id(unit.id.clone()))?;
            }
        }
        let bindings = |direction| {
            spec.ports
                .iter()
                .filter(|p| p.direction == direction)
                .flat_map(|p| {
                    components.iter().map(move |id| ComponentPort {
                        port: p.name.into(),
                        component: id.clone(),
                    })
                })
                .collect::<Vec<_>>()
        };
        let inputs = bindings(PortDirection::Inlet);
        let outputs = bindings(PortDirection::Outlet);
        let derivatives_dimensionless = outputs
            .iter()
            .map(|out| {
                inputs
                    .iter()
                    .map(|input| {
                        if input.component != out.component {
                            return 0.0;
                        }
                        match capability {
                            AffineComponentCapability::TwoInletMixing => {
                                component_balance::mixer_component_outlet(1.0, 0.0)
                            }
                            AffineComponentCapability::NonselectiveSplitting => {
                                let a = unit
                                    .parameters
                                    .outlet_a_split_fraction
                                    .expect("validated fraction");
                                component_balance::splitter_component_outlet(
                                    1.0,
                                    if out.port == SPLITTER_OUTLET_A_PORT {
                                        a
                                    } else {
                                        1.0 - a
                                    },
                                )
                            }
                        }
                    })
                    .collect()
            })
            .collect();
        Ok(Self {
            unit_id: unit.id.clone(),
            capability,
            temperature_k,
            pressure_pa,
            inputs,
            outputs,
            derivatives_dimensionless,
        })
    }

    /// d(n_out)/(d(n_in)) at the frozen boundary and specifications, unit 1.
    pub fn derivative(&self, output: &ComponentPort, input: &ComponentPort) -> RfResult<f64> {
        let invalid = || {
            RfError::invalid_input("response port/component is outside the declared mapping")
                .with_related_unit_id(self.unit_id.clone())
        };
        let i = self
            .outputs
            .iter()
            .position(|b| b == output)
            .ok_or_else(invalid)?;
        let j = self
            .inputs
            .iter()
            .position(|b| b == input)
            .ok_or_else(invalid)?;
        Ok(self.derivatives_dimensionless[i][j])
    }
}

#[cfg(test)]
mod tests;
