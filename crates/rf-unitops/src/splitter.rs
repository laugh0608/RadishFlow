//! Nonselective material splitting. Only extensive flow changes; all supplied
//! intensive state belongs to the inlet and is copied without recalculation.
use crate::specifications::{CompositionCheck, CompositionPolicy, check_composition};
use crate::*;

pub const SPLITTER_SPEC: UnitOperationSpec = UnitOperationSpec {
    kind: BuiltinUnitKind::Splitter,
    ports: &[
        UnitPortDefinition {
            name: SPLITTER_INLET_PORT,
            direction: PortDirection::Inlet,
            kind: PortKind::Material,
        },
        UnitPortDefinition {
            name: SPLITTER_OUTLET_A_PORT,
            direction: PortDirection::Outlet,
            kind: PortKind::Material,
        },
        UnitPortDefinition {
            name: SPLITTER_OUTLET_B_PORT,
            direction: PortDirection::Outlet,
            kind: PortKind::Material,
        },
    ],
};

pub fn build_splitter_node(
    id: impl Into<UnitId>,
    name: impl Into<String>,
    inlet: impl Into<StreamId>,
    outlet_a: impl Into<StreamId>,
    outlet_b: impl Into<StreamId>,
) -> UnitNode {
    UnitNode::new(
        id,
        name,
        SPLITTER_KIND,
        vec![
            material_port(SPLITTER_INLET_PORT, PortDirection::Inlet, inlet),
            material_port(SPLITTER_OUTLET_A_PORT, PortDirection::Outlet, outlet_a),
            material_port(SPLITTER_OUTLET_B_PORT, PortDirection::Outlet, outlet_b),
        ],
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct Splitter {
    outlet_a: StreamTarget,
    outlet_b: StreamTarget,
    outlet_a_split_fraction: f64,
}

impl Splitter {
    pub fn new(
        outlet_a: StreamTarget,
        outlet_b: StreamTarget,
        outlet_a_split_fraction: f64,
    ) -> Self {
        Self {
            outlet_a,
            outlet_b,
            outlet_a_split_fraction,
        }
    }
}

impl UnitOperation for Splitter {
    fn kind(&self) -> BuiltinUnitKind {
        BuiltinUnitKind::Splitter
    }

    fn run(
        &self,
        _services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        inputs.validate_against_spec(self.spec())?;
        let fraction = self.outlet_a_split_fraction;
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(RfError::invalid_input(
                "outlet_a_split_fraction must be finite and in [0, 1]",
            )
            .with_diagnostic_code("unitops.splitter.invalid_fraction"));
        }
        let inlet = inputs.require_stream(SPLITTER_INLET_PORT)?;
        validated_total_flow(inlet)?;
        validated_temperature(inlet)?;
        validated_pressure(inlet)?;
        if !matches!(
            check_composition(
                &inlet.overall_mole_fractions,
                CompositionPolicy::SpecifiedMoleFractions
            ),
            CompositionCheck::Valid { .. }
        ) {
            return Err(RfError::invalid_input(
                "splitter inlet requires normalized mole fractions",
            )
            .with_diagnostic_code("unitops.splitter.invalid_composition")
            .with_related_stream_id(inlet.id.clone()));
        }
        let mut outputs = UnitOperationOutputs::new();
        for (port, target, split) in [
            (SPLITTER_OUTLET_A_PORT, &self.outlet_a, fraction),
            (SPLITTER_OUTLET_B_PORT, &self.outlet_b, 1.0 - fraction),
        ] {
            let mut output = inlet.clone();
            output.id = target.id.clone();
            output.name = target.name.clone();
            output.total_molar_flow_mol_s *= split;
            outputs.insert_material_stream(port, output);
        }
        Ok(outputs)
    }
}

#[cfg(test)]
mod tests;
