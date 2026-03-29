use std::collections::BTreeMap;

use rf_types::{
    ComponentId, PhaseLabel, PortDirection, PortKind, RfError, RfResult, StreamId, UnitId,
};

pub type Composition = BTreeMap<ComponentId, f64>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub id: ComponentId,
    pub name: String,
    pub formula: Option<String>,
}

impl Component {
    pub fn new(id: impl Into<ComponentId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            formula: None,
        }
    }

    pub fn with_formula(mut self, formula: impl Into<String>) -> Self {
        self.formula = Some(formula.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhaseState {
    pub label: PhaseLabel,
    pub phase_fraction: f64,
    pub mole_fractions: Composition,
    pub molar_enthalpy_j_per_mol: Option<f64>,
}

impl PhaseState {
    pub fn new(label: PhaseLabel, phase_fraction: f64, mole_fractions: Composition) -> Self {
        Self {
            label,
            phase_fraction,
            mole_fractions,
            molar_enthalpy_j_per_mol: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialStreamState {
    pub id: StreamId,
    pub name: String,
    pub temperature_k: f64,
    pub pressure_pa: f64,
    pub total_molar_flow_mol_s: f64,
    pub overall_mole_fractions: Composition,
    pub phases: Vec<PhaseState>,
}

impl MaterialStreamState {
    pub fn new(id: impl Into<StreamId>, name: impl Into<String>) -> Self {
        Self::from_tpzf(id, name, 298.15, 101_325.0, 0.0, Composition::new())
    }

    pub fn from_tpzf(
        id: impl Into<StreamId>,
        name: impl Into<String>,
        temperature_k: f64,
        pressure_pa: f64,
        total_molar_flow_mol_s: f64,
        overall_mole_fractions: Composition,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            temperature_k,
            pressure_pa,
            total_molar_flow_mol_s,
            overall_mole_fractions,
            phases: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitPort {
    pub name: String,
    pub direction: PortDirection,
    pub kind: PortKind,
    pub stream_id: Option<StreamId>,
}

impl UnitPort {
    pub fn new(
        name: impl Into<String>,
        direction: PortDirection,
        kind: PortKind,
        stream_id: Option<StreamId>,
    ) -> Self {
        Self {
            name: name.into(),
            direction,
            kind,
            stream_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitNode {
    pub id: UnitId,
    pub name: String,
    pub kind: String,
    pub ports: Vec<UnitPort>,
}

impl UnitNode {
    pub fn new(
        id: impl Into<UnitId>,
        name: impl Into<String>,
        kind: impl Into<String>,
        ports: Vec<UnitPort>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind: kind.into(),
            ports,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flowsheet {
    pub name: String,
    pub components: BTreeMap<ComponentId, Component>,
    pub streams: BTreeMap<StreamId, MaterialStreamState>,
    pub units: BTreeMap<UnitId, UnitNode>,
}

impl Flowsheet {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            components: BTreeMap::new(),
            streams: BTreeMap::new(),
            units: BTreeMap::new(),
        }
    }

    pub fn insert_component(&mut self, component: Component) -> RfResult<()> {
        let component_id = component.id.clone();
        if self.components.contains_key(&component_id) {
            return Err(RfError::duplicate_id("component", component_id));
        }

        self.components.insert(component_id, component);
        Ok(())
    }

    pub fn insert_stream(&mut self, stream: MaterialStreamState) -> RfResult<()> {
        let stream_id = stream.id.clone();
        if self.streams.contains_key(&stream_id) {
            return Err(RfError::duplicate_id("stream", stream_id));
        }

        self.streams.insert(stream_id, stream);
        Ok(())
    }

    pub fn insert_unit(&mut self, unit: UnitNode) -> RfResult<()> {
        let unit_id = unit.id.clone();
        if self.units.contains_key(&unit_id) {
            return Err(RfError::duplicate_id("unit", unit_id));
        }

        self.units.insert(unit_id, unit);
        Ok(())
    }

    pub fn component(&self, id: &ComponentId) -> RfResult<&Component> {
        self.components
            .get(id)
            .ok_or_else(|| RfError::missing_entity("component", id))
    }

    pub fn stream(&self, id: &StreamId) -> RfResult<&MaterialStreamState> {
        self.streams
            .get(id)
            .ok_or_else(|| RfError::missing_entity("stream", id))
    }

    pub fn unit(&self, id: &UnitId) -> RfResult<&UnitNode> {
        self.units
            .get(id)
            .ok_or_else(|| RfError::missing_entity("unit", id))
    }
}
