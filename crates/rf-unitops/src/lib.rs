use std::collections::{BTreeMap, BTreeSet};

use rf_flash::{TpFlashInput, TpFlashSolver, estimate_bubble_dew_window};
use rf_model::{Composition, MaterialStreamState, PhaseState, UnitNode, UnitPort};
use rf_thermo::ThermoProvider;
use rf_types::{PhaseLabel, PortDirection, PortKind, RfError, RfResult, StreamId, UnitId};

pub const FEED_KIND: &str = "feed";
pub const MIXER_KIND: &str = "mixer";
pub const HEATER_KIND: &str = "heater";
pub const COOLER_KIND: &str = "cooler";
pub const VALVE_KIND: &str = "valve";
pub const FLASH_DRUM_KIND: &str = "flash_drum";

pub const FEED_OUTLET_PORT: &str = "outlet";
pub const MIXER_INLET_A_PORT: &str = "inlet_a";
pub const MIXER_INLET_B_PORT: &str = "inlet_b";
pub const MIXER_OUTLET_PORT: &str = "outlet";
pub const HEATER_COOLER_INLET_PORT: &str = "inlet";
pub const HEATER_COOLER_OUTLET_PORT: &str = "outlet";
pub const FLASH_DRUM_INLET_PORT: &str = "inlet";
pub const FLASH_DRUM_LIQUID_PORT: &str = "liquid";
pub const FLASH_DRUM_VAPOR_PORT: &str = "vapor";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinUnitKind {
    Feed,
    Mixer,
    Heater,
    Cooler,
    Valve,
    FlashDrum,
}

impl BuiltinUnitKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Feed => FEED_KIND,
            Self::Mixer => MIXER_KIND,
            Self::Heater => HEATER_KIND,
            Self::Cooler => COOLER_KIND,
            Self::Valve => VALVE_KIND,
            Self::FlashDrum => FLASH_DRUM_KIND,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            FEED_KIND => Some(Self::Feed),
            MIXER_KIND => Some(Self::Mixer),
            HEATER_KIND => Some(Self::Heater),
            COOLER_KIND => Some(Self::Cooler),
            VALVE_KIND => Some(Self::Valve),
            FLASH_DRUM_KIND => Some(Self::FlashDrum),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitPortDefinition {
    pub name: &'static str,
    pub direction: PortDirection,
    pub kind: PortKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitOperationSpec {
    pub kind: BuiltinUnitKind,
    pub ports: &'static [UnitPortDefinition],
}

const FEED_PORTS: [UnitPortDefinition; 1] = [UnitPortDefinition {
    name: FEED_OUTLET_PORT,
    direction: PortDirection::Outlet,
    kind: PortKind::Material,
}];

const MIXER_PORTS: [UnitPortDefinition; 3] = [
    UnitPortDefinition {
        name: MIXER_INLET_A_PORT,
        direction: PortDirection::Inlet,
        kind: PortKind::Material,
    },
    UnitPortDefinition {
        name: MIXER_INLET_B_PORT,
        direction: PortDirection::Inlet,
        kind: PortKind::Material,
    },
    UnitPortDefinition {
        name: MIXER_OUTLET_PORT,
        direction: PortDirection::Outlet,
        kind: PortKind::Material,
    },
];

const HEATER_COOLER_PORTS: [UnitPortDefinition; 2] = [
    UnitPortDefinition {
        name: HEATER_COOLER_INLET_PORT,
        direction: PortDirection::Inlet,
        kind: PortKind::Material,
    },
    UnitPortDefinition {
        name: HEATER_COOLER_OUTLET_PORT,
        direction: PortDirection::Outlet,
        kind: PortKind::Material,
    },
];

const FLASH_DRUM_PORTS: [UnitPortDefinition; 3] = [
    UnitPortDefinition {
        name: FLASH_DRUM_INLET_PORT,
        direction: PortDirection::Inlet,
        kind: PortKind::Material,
    },
    UnitPortDefinition {
        name: FLASH_DRUM_LIQUID_PORT,
        direction: PortDirection::Outlet,
        kind: PortKind::Material,
    },
    UnitPortDefinition {
        name: FLASH_DRUM_VAPOR_PORT,
        direction: PortDirection::Outlet,
        kind: PortKind::Material,
    },
];

const FEED_SPEC: UnitOperationSpec = UnitOperationSpec {
    kind: BuiltinUnitKind::Feed,
    ports: &FEED_PORTS,
};

const MIXER_SPEC: UnitOperationSpec = UnitOperationSpec {
    kind: BuiltinUnitKind::Mixer,
    ports: &MIXER_PORTS,
};

const HEATER_SPEC: UnitOperationSpec = UnitOperationSpec {
    kind: BuiltinUnitKind::Heater,
    ports: &HEATER_COOLER_PORTS,
};

const COOLER_SPEC: UnitOperationSpec = UnitOperationSpec {
    kind: BuiltinUnitKind::Cooler,
    ports: &HEATER_COOLER_PORTS,
};

const VALVE_SPEC: UnitOperationSpec = UnitOperationSpec {
    kind: BuiltinUnitKind::Valve,
    ports: &HEATER_COOLER_PORTS,
};

const FLASH_DRUM_SPEC: UnitOperationSpec = UnitOperationSpec {
    kind: BuiltinUnitKind::FlashDrum,
    ports: &FLASH_DRUM_PORTS,
};

pub fn builtin_unit_spec(kind: BuiltinUnitKind) -> &'static UnitOperationSpec {
    match kind {
        BuiltinUnitKind::Feed => &FEED_SPEC,
        BuiltinUnitKind::Mixer => &MIXER_SPEC,
        BuiltinUnitKind::Heater => &HEATER_SPEC,
        BuiltinUnitKind::Cooler => &COOLER_SPEC,
        BuiltinUnitKind::Valve => &VALVE_SPEC,
        BuiltinUnitKind::FlashDrum => &FLASH_DRUM_SPEC,
    }
}

pub fn builtin_unit_spec_by_name(kind: &str) -> Option<&'static UnitOperationSpec> {
    BuiltinUnitKind::parse(kind).map(builtin_unit_spec)
}

pub fn validate_unit_node(node: &UnitNode) -> RfResult<&'static UnitOperationSpec> {
    let spec = builtin_unit_spec_by_name(&node.kind).ok_or_else(|| {
        RfError::invalid_input(format!(
            "unit `{}` uses unsupported kind `{}`",
            node.id, node.kind
        ))
    })?;

    if node.ports.len() != spec.ports.len() {
        return Err(RfError::invalid_input(format!(
            "unit `{}` of kind `{}` must expose {} ports, received {}",
            node.id,
            node.kind,
            spec.ports.len(),
            node.ports.len()
        )));
    }

    let mut port_names = BTreeSet::new();
    for port in &node.ports {
        if !port_names.insert(port.name.as_str()) {
            return Err(RfError::invalid_input(format!(
                "unit `{}` has duplicate port `{}`",
                node.id, port.name
            )));
        }
    }

    for expected in spec.ports {
        let actual = node
            .ports
            .iter()
            .find(|port| port.name == expected.name)
            .ok_or_else(|| {
                RfError::invalid_input(format!(
                    "unit `{}` is missing required port `{}`",
                    node.id, expected.name
                ))
            })?;

        if actual.direction != expected.direction {
            return Err(RfError::invalid_input(format!(
                "unit `{}` port `{}` must use direction `{}`, received `{}`",
                node.id, actual.name, expected.direction, actual.direction
            )));
        }

        if actual.kind != expected.kind {
            return Err(RfError::invalid_input(format!(
                "unit `{}` port `{}` must use kind `{}`, received `{}`",
                node.id, actual.name, expected.kind, actual.kind
            )));
        }
    }

    Ok(spec)
}

pub fn build_feed_node(
    id: impl Into<UnitId>,
    name: impl Into<String>,
    outlet_stream_id: impl Into<StreamId>,
) -> UnitNode {
    UnitNode::new(
        id,
        name,
        FEED_KIND,
        vec![material_port(
            FEED_OUTLET_PORT,
            PortDirection::Outlet,
            outlet_stream_id,
        )],
    )
}

pub fn build_mixer_node(
    id: impl Into<UnitId>,
    name: impl Into<String>,
    inlet_a_stream_id: impl Into<StreamId>,
    inlet_b_stream_id: impl Into<StreamId>,
    outlet_stream_id: impl Into<StreamId>,
) -> UnitNode {
    UnitNode::new(
        id,
        name,
        MIXER_KIND,
        vec![
            material_port(MIXER_INLET_A_PORT, PortDirection::Inlet, inlet_a_stream_id),
            material_port(MIXER_INLET_B_PORT, PortDirection::Inlet, inlet_b_stream_id),
            material_port(MIXER_OUTLET_PORT, PortDirection::Outlet, outlet_stream_id),
        ],
    )
}

pub fn build_flash_drum_node(
    id: impl Into<UnitId>,
    name: impl Into<String>,
    inlet_stream_id: impl Into<StreamId>,
    liquid_stream_id: impl Into<StreamId>,
    vapor_stream_id: impl Into<StreamId>,
) -> UnitNode {
    UnitNode::new(
        id,
        name,
        FLASH_DRUM_KIND,
        vec![
            material_port(FLASH_DRUM_INLET_PORT, PortDirection::Inlet, inlet_stream_id),
            material_port(
                FLASH_DRUM_LIQUID_PORT,
                PortDirection::Outlet,
                liquid_stream_id,
            ),
            material_port(
                FLASH_DRUM_VAPOR_PORT,
                PortDirection::Outlet,
                vapor_stream_id,
            ),
        ],
    )
}

pub fn build_heater_node(
    id: impl Into<UnitId>,
    name: impl Into<String>,
    inlet_stream_id: impl Into<StreamId>,
    outlet_stream_id: impl Into<StreamId>,
) -> UnitNode {
    build_single_inlet_single_outlet_node(id, name, HEATER_KIND, inlet_stream_id, outlet_stream_id)
}

pub fn build_cooler_node(
    id: impl Into<UnitId>,
    name: impl Into<String>,
    inlet_stream_id: impl Into<StreamId>,
    outlet_stream_id: impl Into<StreamId>,
) -> UnitNode {
    build_single_inlet_single_outlet_node(id, name, COOLER_KIND, inlet_stream_id, outlet_stream_id)
}

pub fn build_valve_node(
    id: impl Into<UnitId>,
    name: impl Into<String>,
    inlet_stream_id: impl Into<StreamId>,
    outlet_stream_id: impl Into<StreamId>,
) -> UnitNode {
    build_single_inlet_single_outlet_node(id, name, VALVE_KIND, inlet_stream_id, outlet_stream_id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamTarget {
    pub id: StreamId,
    pub name: String,
}

impl StreamTarget {
    pub fn new(id: impl Into<StreamId>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UnitOperationInputs {
    material_streams: BTreeMap<String, MaterialStreamState>,
}

impl UnitOperationInputs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_material_stream(
        &mut self,
        port_name: impl Into<String>,
        stream: MaterialStreamState,
    ) {
        self.material_streams.insert(port_name.into(), stream);
    }

    pub fn with_material_stream(
        mut self,
        port_name: impl Into<String>,
        stream: MaterialStreamState,
    ) -> Self {
        self.insert_material_stream(port_name, stream);
        self
    }

    pub fn stream(&self, port_name: &str) -> Option<&MaterialStreamState> {
        self.material_streams.get(port_name)
    }

    pub fn require_stream(&self, port_name: &str) -> RfResult<&MaterialStreamState> {
        self.stream(port_name).ok_or_else(|| {
            RfError::invalid_input(format!(
                "material input port `{port_name}` is missing from unit execution context"
            ))
        })
    }

    pub fn validate_against_spec(&self, spec: &UnitOperationSpec) -> RfResult<()> {
        let mut expected_ports = spec
            .ports
            .iter()
            .filter(|port| {
                port.kind == PortKind::Material && port.direction == PortDirection::Inlet
            })
            .map(|port| port.name.to_owned())
            .collect::<Vec<_>>();
        expected_ports.sort();

        let actual_ports = self.material_streams.keys().cloned().collect::<Vec<_>>();

        if actual_ports != expected_ports {
            return Err(RfError::invalid_input(format!(
                "unit `{}` expects material inlet ports [{}], received [{}]",
                spec.kind.as_str(),
                format_port_names(&expected_ports),
                format_port_names(&actual_ports)
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UnitOperationOutputs {
    material_streams: BTreeMap<String, MaterialStreamState>,
}

impl UnitOperationOutputs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_material_stream(
        &mut self,
        port_name: impl Into<String>,
        stream: MaterialStreamState,
    ) {
        self.material_streams.insert(port_name.into(), stream);
    }

    pub fn stream(&self, port_name: &str) -> Option<&MaterialStreamState> {
        self.material_streams.get(port_name)
    }
}

#[derive(Clone, Copy, Default)]
pub struct UnitOperationServices<'a> {
    pub thermo: Option<&'a dyn ThermoProvider>,
    pub flash_solver: Option<&'a dyn TpFlashSolver>,
}

impl<'a> UnitOperationServices<'a> {
    pub fn require_thermo(&self) -> RfResult<&'a dyn ThermoProvider> {
        self.thermo
            .ok_or_else(|| RfError::invalid_input("unit operation requires a thermo provider"))
    }

    pub fn require_flash_solver(&self) -> RfResult<&'a dyn TpFlashSolver> {
        self.flash_solver
            .ok_or_else(|| RfError::invalid_input("unit operation requires a TP flash solver"))
    }
}

pub trait UnitOperation {
    fn kind(&self) -> BuiltinUnitKind;

    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs>;

    fn spec(&self) -> &'static UnitOperationSpec {
        builtin_unit_spec(self.kind())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Feed {
    outlet_stream: MaterialStreamState,
}

impl Feed {
    pub fn new(outlet_stream: MaterialStreamState) -> Self {
        Self { outlet_stream }
    }
}

impl UnitOperation for Feed {
    fn kind(&self) -> BuiltinUnitKind {
        BuiltinUnitKind::Feed
    }

    fn run(
        &self,
        _services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        inputs.validate_against_spec(self.spec())?;

        let mut outputs = UnitOperationOutputs::new();
        outputs.insert_material_stream(FEED_OUTLET_PORT, self.outlet_stream.clone());
        Ok(outputs)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mixer {
    outlet: StreamTarget,
}

impl Mixer {
    pub fn new(outlet: StreamTarget) -> Self {
        Self { outlet }
    }
}

impl UnitOperation for Mixer {
    fn kind(&self) -> BuiltinUnitKind {
        BuiltinUnitKind::Mixer
    }

    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        inputs.validate_against_spec(self.spec())?;

        let inlet_a = inputs.require_stream(MIXER_INLET_A_PORT)?;
        let inlet_b = inputs.require_stream(MIXER_INLET_B_PORT)?;

        let flow_a = validated_total_flow(inlet_a)?;
        let flow_b = validated_total_flow(inlet_b)?;
        let total_flow = flow_a + flow_b;
        if total_flow <= 0.0 {
            return Err(RfError::invalid_input(
                "mixer total molar flow must be greater than zero",
            ));
        }

        let temperature_k = weighted_average_by_flow(
            [
                (flow_a, inlet_a.temperature_k),
                (flow_b, inlet_b.temperature_k),
            ],
            "mixer outlet temperature",
        )?;
        let pressure_pa = validated_pressure(inlet_a)?.min(validated_pressure(inlet_b)?);
        let overall_mole_fractions = mixed_composition([inlet_a, inlet_b])?;

        let mut outlet_stream = MaterialStreamState::from_tpzf(
            self.outlet.id.clone(),
            self.outlet.name.clone(),
            temperature_k,
            pressure_pa,
            total_flow,
            overall_mole_fractions.clone(),
        );
        outlet_stream.phases.push(PhaseState::new(
            PhaseLabel::Overall,
            1.0,
            overall_mole_fractions,
        ));
        let thermo = services.require_thermo()?;
        attach_bubble_dew_window(thermo, &mut outlet_stream)?;

        let mut outputs = UnitOperationOutputs::new();
        outputs.insert_material_stream(MIXER_OUTLET_PORT, outlet_stream);
        Ok(outputs)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeaterCooler {
    kind: BuiltinUnitKind,
    outlet_stream: MaterialStreamState,
}

impl HeaterCooler {
    pub fn new(kind: BuiltinUnitKind, outlet_stream: MaterialStreamState) -> RfResult<Self> {
        match kind {
            BuiltinUnitKind::Heater | BuiltinUnitKind::Cooler => Ok(Self {
                kind,
                outlet_stream,
            }),
            _ => Err(RfError::invalid_input(format!(
                "heater/cooler operation cannot be created with unit kind `{}`",
                kind.as_str()
            ))),
        }
    }
}

impl UnitOperation for HeaterCooler {
    fn kind(&self) -> BuiltinUnitKind {
        self.kind
    }

    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        inputs.validate_against_spec(self.spec())?;

        let inlet = inputs.require_stream(HEATER_COOLER_INLET_PORT)?;
        let temperature_k = validated_temperature(&self.outlet_stream)?;
        let pressure_pa = validated_pressure(&self.outlet_stream)?;
        let total_flow = validated_total_flow(inlet)?;
        let overall_mole_fractions = normalized_composition(inlet)?;

        let mut outlet_stream = MaterialStreamState::from_tpzf(
            self.outlet_stream.id.clone(),
            self.outlet_stream.name.clone(),
            temperature_k,
            pressure_pa,
            total_flow,
            overall_mole_fractions.clone(),
        );

        if total_flow > 0.0 {
            outlet_stream.phases.push(PhaseState::new(
                PhaseLabel::Overall,
                1.0,
                overall_mole_fractions,
            ));
        }
        let thermo = services.require_thermo()?;
        attach_bubble_dew_window(thermo, &mut outlet_stream)?;

        let mut outputs = UnitOperationOutputs::new();
        outputs.insert_material_stream(HEATER_COOLER_OUTLET_PORT, outlet_stream);
        Ok(outputs)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Valve {
    outlet_stream: MaterialStreamState,
}

impl Valve {
    pub fn new(outlet_stream: MaterialStreamState) -> Self {
        Self { outlet_stream }
    }
}

impl UnitOperation for Valve {
    fn kind(&self) -> BuiltinUnitKind {
        BuiltinUnitKind::Valve
    }

    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        inputs.validate_against_spec(self.spec())?;

        let inlet = inputs.require_stream(HEATER_COOLER_INLET_PORT)?;
        let inlet_temperature_k = validated_temperature(inlet)?;
        let inlet_pressure_pa = validated_pressure(inlet)?;
        let outlet_pressure_pa = validated_pressure(&self.outlet_stream)?;
        if outlet_pressure_pa > inlet_pressure_pa {
            return Err(RfError::invalid_input(format!(
                "valve outlet pressure `{outlet_pressure_pa}` Pa cannot exceed inlet pressure `{inlet_pressure_pa}` Pa for stream `{}`",
                inlet.id
            )));
        }

        let total_flow = validated_total_flow(inlet)?;
        let overall_mole_fractions = normalized_composition(inlet)?;
        let mut outlet_stream = MaterialStreamState::from_tpzf(
            self.outlet_stream.id.clone(),
            self.outlet_stream.name.clone(),
            inlet_temperature_k,
            outlet_pressure_pa,
            total_flow,
            overall_mole_fractions.clone(),
        );

        if total_flow > 0.0 {
            outlet_stream.phases.push(PhaseState::new(
                PhaseLabel::Overall,
                1.0,
                overall_mole_fractions,
            ));
        }
        let thermo = services.require_thermo()?;
        attach_bubble_dew_window(thermo, &mut outlet_stream)?;

        let mut outputs = UnitOperationOutputs::new();
        outputs.insert_material_stream(HEATER_COOLER_OUTLET_PORT, outlet_stream);
        Ok(outputs)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlashDrum {
    liquid_outlet: StreamTarget,
    vapor_outlet: StreamTarget,
}

impl FlashDrum {
    pub fn new(liquid_outlet: StreamTarget, vapor_outlet: StreamTarget) -> Self {
        Self {
            liquid_outlet,
            vapor_outlet,
        }
    }
}

impl UnitOperation for FlashDrum {
    fn kind(&self) -> BuiltinUnitKind {
        BuiltinUnitKind::FlashDrum
    }

    fn run(
        &self,
        services: &UnitOperationServices<'_>,
        inputs: &UnitOperationInputs,
    ) -> RfResult<UnitOperationOutputs> {
        inputs.validate_against_spec(self.spec())?;

        let thermo = services.require_thermo()?;
        let flash_solver = services.require_flash_solver()?;
        let inlet = inputs.require_stream(FLASH_DRUM_INLET_PORT)?;

        let flash_input = TpFlashInput::new(
            inlet.id.clone(),
            inlet.name.clone(),
            validated_temperature(inlet)?,
            validated_pressure(inlet)?,
            validated_total_flow(inlet)?,
            stream_composition_vector(inlet, thermo)?,
        );
        let flash_result = flash_solver.flash(thermo, &flash_input)?;
        let liquid_phase = flash_result
            .stream
            .phases
            .iter()
            .find(|phase| phase.label == PhaseLabel::Liquid);
        let vapor_phase = flash_result
            .stream
            .phases
            .iter()
            .find(|phase| phase.label == PhaseLabel::Vapor);

        let mut liquid_stream = build_phase_outlet_stream(
            &self.liquid_outlet,
            &flash_result.stream,
            liquid_phase,
            PhaseLabel::Liquid,
            &inlet.overall_mole_fractions,
        );
        let mut vapor_stream = build_phase_outlet_stream(
            &self.vapor_outlet,
            &flash_result.stream,
            vapor_phase,
            PhaseLabel::Vapor,
            &inlet.overall_mole_fractions,
        );
        attach_bubble_dew_window(thermo, &mut liquid_stream)?;
        attach_bubble_dew_window(thermo, &mut vapor_stream)?;

        let mut outputs = UnitOperationOutputs::new();
        outputs.insert_material_stream(FLASH_DRUM_LIQUID_PORT, liquid_stream);
        outputs.insert_material_stream(FLASH_DRUM_VAPOR_PORT, vapor_stream);
        Ok(outputs)
    }
}

fn material_port(
    name: impl Into<String>,
    direction: PortDirection,
    stream_id: impl Into<StreamId>,
) -> UnitPort {
    UnitPort::new(name, direction, PortKind::Material, Some(stream_id.into()))
}

fn build_single_inlet_single_outlet_node(
    id: impl Into<UnitId>,
    name: impl Into<String>,
    kind: impl Into<String>,
    inlet_stream_id: impl Into<StreamId>,
    outlet_stream_id: impl Into<StreamId>,
) -> UnitNode {
    UnitNode::new(
        id,
        name,
        kind,
        vec![
            material_port(
                HEATER_COOLER_INLET_PORT,
                PortDirection::Inlet,
                inlet_stream_id,
            ),
            material_port(
                HEATER_COOLER_OUTLET_PORT,
                PortDirection::Outlet,
                outlet_stream_id,
            ),
        ],
    )
}

fn format_port_names(port_names: &[String]) -> String {
    if port_names.is_empty() {
        return "<none>".to_owned();
    }

    port_names.join(", ")
}

fn validated_temperature(stream: &MaterialStreamState) -> RfResult<f64> {
    if !stream.temperature_k.is_finite() || stream.temperature_k <= 0.0 {
        return Err(RfError::invalid_input(format!(
            "stream `{}` temperature must be a finite value greater than zero kelvin",
            stream.id
        )));
    }

    Ok(stream.temperature_k)
}

fn validated_pressure(stream: &MaterialStreamState) -> RfResult<f64> {
    if !stream.pressure_pa.is_finite() || stream.pressure_pa <= 0.0 {
        return Err(RfError::invalid_input(format!(
            "stream `{}` pressure must be a finite value greater than zero pascal",
            stream.id
        )));
    }

    Ok(stream.pressure_pa)
}

fn validated_total_flow(stream: &MaterialStreamState) -> RfResult<f64> {
    if !stream.total_molar_flow_mol_s.is_finite() || stream.total_molar_flow_mol_s < 0.0 {
        return Err(RfError::invalid_input(format!(
            "stream `{}` total molar flow must be a finite non-negative value",
            stream.id
        )));
    }

    Ok(stream.total_molar_flow_mol_s)
}

fn normalized_composition(stream: &MaterialStreamState) -> RfResult<Composition> {
    if stream.overall_mole_fractions.is_empty() {
        return Err(RfError::invalid_input(format!(
            "stream `{}` must define at least one overall mole fraction entry",
            stream.id
        )));
    }

    if stream
        .overall_mole_fractions
        .values()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(RfError::invalid_input(format!(
            "stream `{}` overall mole fractions must be finite non-negative values",
            stream.id
        )));
    }

    let sum = stream.overall_mole_fractions.values().sum::<f64>();
    if !sum.is_finite() || sum <= 0.0 {
        return Err(RfError::invalid_input(format!(
            "stream `{}` overall mole fractions must sum to a positive finite value",
            stream.id
        )));
    }

    Ok(stream
        .overall_mole_fractions
        .iter()
        .map(|(component_id, value)| (component_id.clone(), value / sum))
        .collect())
}

fn mixed_composition(streams: [&MaterialStreamState; 2]) -> RfResult<Composition> {
    let mut component_molar_flows = Composition::new();
    let mut total_flow = 0.0;

    for stream in streams {
        let flow = validated_total_flow(stream)?;
        let normalized = normalized_composition(stream)?;
        total_flow += flow;

        for (component_id, fraction) in normalized {
            *component_molar_flows.entry(component_id).or_insert(0.0) += flow * fraction;
        }
    }

    if !total_flow.is_finite() || total_flow <= 0.0 {
        return Err(RfError::invalid_input(
            "mixed stream total molar flow must be a positive finite value",
        ));
    }

    Ok(component_molar_flows
        .into_iter()
        .map(|(component_id, molar_flow)| (component_id, molar_flow / total_flow))
        .collect())
}

fn weighted_average_by_flow(items: [(f64, f64); 2], label: &str) -> RfResult<f64> {
    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for (flow, value) in items {
        if !value.is_finite() {
            return Err(RfError::invalid_input(format!(
                "{label} cannot be derived from a non-finite stream property"
            )));
        }

        numerator += flow * value;
        denominator += flow;
    }

    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(RfError::invalid_input(format!(
            "{label} requires a positive finite total molar flow"
        )));
    }

    Ok(numerator / denominator)
}

fn stream_composition_vector(
    stream: &MaterialStreamState,
    thermo: &dyn ThermoProvider,
) -> RfResult<Vec<f64>> {
    let normalized = normalized_composition(stream)?;
    let component_ids = thermo.system().component_ids();

    if normalized.len() != component_ids.len() {
        return Err(RfError::invalid_input(format!(
            "stream `{}` composition has {} components, but thermo system expects {}",
            stream.id,
            normalized.len(),
            component_ids.len()
        )));
    }

    component_ids
        .into_iter()
        .map(|component_id| {
            normalized.get(&component_id).copied().ok_or_else(|| {
                RfError::invalid_input(format!(
                    "stream `{}` is missing thermo component `{}`",
                    stream.id, component_id
                ))
            })
        })
        .collect()
}

fn attach_bubble_dew_window(
    thermo: &dyn ThermoProvider,
    stream: &mut MaterialStreamState,
) -> RfResult<()> {
    if stream.total_molar_flow_mol_s <= 0.0 || stream.overall_mole_fractions.is_empty() {
        stream.bubble_dew_window = None;
        return Ok(());
    }

    let composition = stream_composition_vector(stream, thermo)?;
    stream.bubble_dew_window = Some(estimate_bubble_dew_window(
        thermo,
        stream.temperature_k,
        stream.pressure_pa,
        composition,
    )?);
    Ok(())
}

fn build_phase_outlet_stream(
    target: &StreamTarget,
    flashed_stream: &MaterialStreamState,
    phase: Option<&PhaseState>,
    phase_label: PhaseLabel,
    fallback_composition: &Composition,
) -> MaterialStreamState {
    let total_flow = phase
        .map(|phase| flashed_stream.total_molar_flow_mol_s * phase.phase_fraction)
        .unwrap_or(0.0);
    let overall_mole_fractions = phase
        .map(|phase| phase.mole_fractions.clone())
        .unwrap_or_else(|| fallback_composition.clone());

    let mut outlet = MaterialStreamState::from_tpzf(
        target.id.clone(),
        target.name.clone(),
        flashed_stream.temperature_k,
        flashed_stream.pressure_pa,
        total_flow,
        overall_mole_fractions.clone(),
    );

    if total_flow > 0.0 {
        let phase_enthalpy = phase.and_then(|phase| phase.molar_enthalpy_j_per_mol);
        let mut overall_phase =
            PhaseState::new(PhaseLabel::Overall, 1.0, overall_mole_fractions.clone());
        overall_phase.molar_enthalpy_j_per_mol = phase_enthalpy;
        let mut outlet_phase = PhaseState::new(phase_label, 1.0, overall_mole_fractions);
        outlet_phase.molar_enthalpy_j_per_mol = phase_enthalpy;
        outlet.phases.push(overall_phase);
        outlet.phases.push(outlet_phase);
    }

    outlet
}

#[cfg(test)]
mod tests {
    use super::{
        BuiltinUnitKind, FEED_OUTLET_PORT, FLASH_DRUM_INLET_PORT, FLASH_DRUM_LIQUID_PORT,
        FLASH_DRUM_VAPOR_PORT, Feed, FlashDrum, HEATER_COOLER_INLET_PORT,
        HEATER_COOLER_OUTLET_PORT, HeaterCooler, MIXER_INLET_A_PORT, MIXER_INLET_B_PORT,
        MIXER_OUTLET_PORT, Mixer, StreamTarget, UnitOperation, UnitOperationInputs,
        UnitOperationServices, Valve, build_cooler_node, build_feed_node, build_flash_drum_node,
        build_heater_node, build_mixer_node, build_valve_node, validate_unit_node,
    };
    use rf_flash::{PlaceholderTpFlashSolver, TpFlashSolver};
    use rf_model::{Composition, MaterialStreamState};
    use rf_thermo::{
        AntoineCoefficients, PlaceholderThermoProvider, ThermoComponent, ThermoSystem,
    };
    use rf_types::{ComponentId, PhaseLabel};

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {actual} to be within {tolerance} of {expected}, delta was {delta}"
        );
    }

    fn binary_composition(first: f64, second: f64) -> Composition {
        [
            (ComponentId::new("component-a"), first),
            (ComponentId::new("component-b"), second),
        ]
        .into_iter()
        .collect()
    }

    fn build_stream(
        id: &str,
        name: &str,
        temperature_k: f64,
        pressure_pa: f64,
        total_molar_flow_mol_s: f64,
        composition: Composition,
    ) -> MaterialStreamState {
        MaterialStreamState::from_tpzf(
            id,
            name,
            temperature_k,
            pressure_pa,
            total_molar_flow_mol_s,
            composition,
        )
    }

    fn build_test_antoine_coefficients(k_value: f64, pressure_pa: f64) -> AntoineCoefficients {
        const TEST_ANTOINE_BOUNDARY_SLOPE: f64 = 250.0;
        const TEST_REFERENCE_TEMPERATURE_K: f64 = 300.0;

        AntoineCoefficients::new(
            ((k_value * pressure_pa) / 1_000.0).ln()
                + TEST_ANTOINE_BOUNDARY_SLOPE / TEST_REFERENCE_TEMPERATURE_K,
            TEST_ANTOINE_BOUNDARY_SLOPE,
            0.0,
        )
    }

    fn build_provider(k_values: [f64; 2], pressure_pa: f64) -> PlaceholderThermoProvider {
        let mut first = ThermoComponent::new(ComponentId::new("component-a"), "Component A");
        first.antoine = Some(build_test_antoine_coefficients(k_values[0], pressure_pa));
        first.liquid_heat_capacity_j_per_mol_k = Some(35.0);
        first.vapor_heat_capacity_j_per_mol_k = Some(36.5);

        let mut second = ThermoComponent::new(ComponentId::new("component-b"), "Component B");
        second.antoine = Some(build_test_antoine_coefficients(k_values[1], pressure_pa));
        second.liquid_heat_capacity_j_per_mol_k = Some(52.0);
        second.vapor_heat_capacity_j_per_mol_k = Some(65.0);

        PlaceholderThermoProvider::new(ThermoSystem::binary([first, second]))
    }

    #[test]
    fn builtin_unit_nodes_match_canonical_specs() {
        let feed = build_feed_node("feed-1", "Feed", "stream-feed");
        let mixer = build_mixer_node("mixer-1", "Mixer", "stream-a", "stream-b", "stream-out");
        let heater = build_heater_node("heater-1", "Heater", "stream-in", "stream-out");
        let cooler = build_cooler_node("cooler-1", "Cooler", "stream-in", "stream-out");
        let valve = build_valve_node("valve-1", "Valve", "stream-in", "stream-out");
        let flash = build_flash_drum_node(
            "flash-1",
            "Flash Drum",
            "stream-feed",
            "stream-liquid",
            "stream-vapor",
        );

        validate_unit_node(&feed).expect("expected feed spec");
        validate_unit_node(&mixer).expect("expected mixer spec");
        validate_unit_node(&heater).expect("expected heater spec");
        validate_unit_node(&cooler).expect("expected cooler spec");
        validate_unit_node(&valve).expect("expected valve spec");
        validate_unit_node(&flash).expect("expected flash drum spec");
    }

    #[test]
    fn feed_emits_configured_outlet_stream() {
        let outlet_stream = build_stream(
            "feed-out",
            "Feed Outlet",
            298.15,
            101_325.0,
            10.0,
            binary_composition(0.25, 0.75),
        );
        let feed = Feed::new(outlet_stream.clone());

        let outputs = feed
            .run(
                &UnitOperationServices::default(),
                &UnitOperationInputs::new(),
            )
            .expect("expected feed output");

        assert_eq!(outputs.stream(FEED_OUTLET_PORT), Some(&outlet_stream));
    }

    #[test]
    fn mixer_combines_two_material_inlets_into_one_overall_stream() {
        let provider = build_provider([2.0, 0.5], 100_000.0);
        let inlet_a = build_stream(
            "stream-a",
            "Feed A",
            300.0,
            120_000.0,
            2.0,
            binary_composition(0.25, 0.75),
        );
        let inlet_b = build_stream(
            "stream-b",
            "Feed B",
            360.0,
            100_000.0,
            3.0,
            binary_composition(0.60, 0.40),
        );
        let mixer = Mixer::new(StreamTarget::new("stream-out", "Mixer Outlet"));

        let inputs = UnitOperationInputs::new()
            .with_material_stream(MIXER_INLET_A_PORT, inlet_a)
            .with_material_stream(MIXER_INLET_B_PORT, inlet_b);
        let services = UnitOperationServices {
            thermo: Some(&provider),
            flash_solver: None,
        };
        let outputs = mixer
            .run(&services, &inputs)
            .expect("expected mixer output");

        let outlet = outputs
            .stream(MIXER_OUTLET_PORT)
            .expect("expected mixer outlet stream");
        assert_eq!(outlet.id.as_str(), "stream-out");
        assert_close(outlet.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(outlet.temperature_k, 336.0, 1e-12);
        assert_close(outlet.pressure_pa, 100_000.0, 1e-12);
        assert_close(
            *outlet
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.46,
            1e-12,
        );
        assert_eq!(outlet.phases.len(), 1);
        assert_eq!(outlet.phases[0].label, PhaseLabel::Overall);
        let window = outlet
            .bubble_dew_window
            .as_ref()
            .expect("expected mixer outlet bubble/dew window");
        assert_eq!(window.phase_region.as_str(), "two_phase");
        assert!(window.dew_pressure_pa < outlet.pressure_pa);
        assert!(window.bubble_pressure_pa > outlet.pressure_pa);
        assert!(window.bubble_temperature_k < outlet.temperature_k);
        assert!(window.dew_temperature_k > outlet.temperature_k);
    }

    #[test]
    fn heater_cooler_updates_outlet_tp_and_preserves_flow_and_composition() {
        let provider = build_provider([2.0, 0.5], 100_000.0);
        let inlet = build_stream(
            "stream-feed",
            "Feed",
            300.0,
            120_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        );
        let outlet_template = build_stream(
            "stream-heated",
            "Heated Outlet",
            345.0,
            95_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        );
        let heater = HeaterCooler::new(BuiltinUnitKind::Heater, outlet_template)
            .expect("expected heater operation");

        let inputs =
            UnitOperationInputs::new().with_material_stream(HEATER_COOLER_INLET_PORT, inlet);
        let services = UnitOperationServices {
            thermo: Some(&provider),
            flash_solver: None,
        };
        let outputs = heater
            .run(&services, &inputs)
            .expect("expected heater output");

        let outlet = outputs
            .stream(HEATER_COOLER_OUTLET_PORT)
            .expect("expected heater outlet stream");
        assert_eq!(outlet.id.as_str(), "stream-heated");
        assert_close(outlet.temperature_k, 345.0, 1e-12);
        assert_close(outlet.pressure_pa, 95_000.0, 1e-12);
        assert_close(outlet.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(
            *outlet
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.35,
            1e-12,
        );
        assert_eq!(outlet.phases.len(), 1);
        assert_eq!(outlet.phases[0].label, PhaseLabel::Overall);
        let window = outlet
            .bubble_dew_window
            .as_ref()
            .expect("expected heater outlet bubble/dew window");
        assert_eq!(window.phase_region.as_str(), "two_phase");
        assert!(window.dew_pressure_pa < outlet.pressure_pa);
        assert!(window.bubble_pressure_pa > outlet.pressure_pa);
        assert!(window.bubble_temperature_k < outlet.temperature_k);
        assert!(window.dew_temperature_k > outlet.temperature_k);
    }

    #[test]
    fn valve_updates_outlet_pressure_and_preserves_inlet_state() {
        let provider = build_provider([2.0, 0.5], 100_000.0);
        let inlet = build_stream(
            "stream-feed",
            "Feed",
            315.0,
            120_000.0,
            5.0,
            binary_composition(0.35, 0.65),
        );
        let outlet_template = build_stream(
            "stream-valve-out",
            "Valve Outlet",
            350.0,
            90_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        );
        let valve = Valve::new(outlet_template);

        let inputs =
            UnitOperationInputs::new().with_material_stream(HEATER_COOLER_INLET_PORT, inlet);
        let services = UnitOperationServices {
            thermo: Some(&provider),
            flash_solver: None,
        };
        let outputs = valve
            .run(&services, &inputs)
            .expect("expected valve output");

        let outlet = outputs
            .stream(HEATER_COOLER_OUTLET_PORT)
            .expect("expected valve outlet stream");
        assert_eq!(outlet.id.as_str(), "stream-valve-out");
        assert_close(outlet.temperature_k, 315.0, 1e-12);
        assert_close(outlet.pressure_pa, 90_000.0, 1e-12);
        assert_close(outlet.total_molar_flow_mol_s, 5.0, 1e-12);
        assert_close(
            *outlet
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected component-a"),
            0.35,
            1e-12,
        );
        assert_eq!(outlet.phases.len(), 1);
        assert_eq!(outlet.phases[0].label, PhaseLabel::Overall);
        let window = outlet
            .bubble_dew_window
            .as_ref()
            .expect("expected valve outlet bubble/dew window");
        assert_eq!(window.phase_region.as_str(), "two_phase");
        assert!(window.dew_pressure_pa < outlet.pressure_pa);
        assert!(window.bubble_pressure_pa > outlet.pressure_pa);
        assert!(window.bubble_temperature_k < outlet.temperature_k);
        assert!(window.dew_temperature_k > outlet.temperature_k);
    }

    #[test]
    fn valve_rejects_outlet_pressure_higher_than_inlet_pressure() {
        let inlet = build_stream(
            "stream-feed",
            "Feed",
            315.0,
            120_000.0,
            5.0,
            binary_composition(0.25, 0.75),
        );
        let outlet_template = build_stream(
            "stream-valve-out",
            "Valve Outlet",
            300.0,
            125_000.0,
            0.0,
            binary_composition(0.5, 0.5),
        );
        let valve = Valve::new(outlet_template);

        let inputs =
            UnitOperationInputs::new().with_material_stream(HEATER_COOLER_INLET_PORT, inlet);
        let error = valve
            .run(&UnitOperationServices::default(), &inputs)
            .expect_err("expected valve pressure validation error");

        assert!(error.message().contains("cannot exceed inlet pressure"));
    }

    #[test]
    fn flash_drum_splits_feed_into_liquid_and_vapor_outlets() {
        let provider = build_provider([2.0, 0.5], 100_000.0);
        let flash_solver = PlaceholderTpFlashSolver;
        let flash_drum = FlashDrum::new(
            StreamTarget::new("stream-liquid", "Liquid Outlet"),
            StreamTarget::new("stream-vapor", "Vapor Outlet"),
        );
        let feed = build_stream(
            "stream-feed",
            "Flash Feed",
            300.0,
            100_000.0,
            8.0,
            binary_composition(0.5, 0.5),
        );
        let inputs = UnitOperationInputs::new().with_material_stream(FLASH_DRUM_INLET_PORT, feed);
        let services = UnitOperationServices {
            thermo: Some(&provider),
            flash_solver: Some(&flash_solver as &dyn TpFlashSolver),
        };

        let outputs = flash_drum
            .run(&services, &inputs)
            .expect("expected flash drum outputs");

        let liquid = outputs
            .stream(FLASH_DRUM_LIQUID_PORT)
            .expect("expected liquid outlet");
        let vapor = outputs
            .stream(FLASH_DRUM_VAPOR_PORT)
            .expect("expected vapor outlet");

        assert_close(liquid.total_molar_flow_mol_s, 4.0, 1e-10);
        assert_close(vapor.total_molar_flow_mol_s, 4.0, 1e-10);
        assert_eq!(liquid.phases[1].label, PhaseLabel::Liquid);
        assert_eq!(vapor.phases[1].label, PhaseLabel::Vapor);
        assert_close(
            liquid.phases[1]
                .molar_enthalpy_j_per_mol
                .expect("expected liquid outlet enthalpy"),
            85.7166666666677,
            1e-10,
        );
        assert_close(
            vapor.phases[1]
                .molar_enthalpy_j_per_mol
                .expect("expected vapor outlet enthalpy"),
            85.100000000001,
            1e-10,
        );
        assert_close(
            *liquid
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected liquid component-a"),
            1.0 / 3.0,
            1e-10,
        );
        assert_close(
            *vapor
                .overall_mole_fractions
                .get(&ComponentId::new("component-a"))
                .expect("expected vapor component-a"),
            2.0 / 3.0,
            1e-10,
        );
        let liquid_window = liquid
            .bubble_dew_window
            .as_ref()
            .expect("expected liquid outlet bubble/dew window");
        assert_eq!(liquid_window.phase_region.as_str(), "two_phase");
        assert_close(liquid_window.bubble_pressure_pa, liquid.pressure_pa, 1e-6);
        assert_close(
            liquid_window.bubble_temperature_k,
            liquid.temperature_k,
            1e-4,
        );
        let vapor_window = vapor
            .bubble_dew_window
            .as_ref()
            .expect("expected vapor outlet bubble/dew window");
        assert_eq!(vapor_window.phase_region.as_str(), "two_phase");
        assert_close(vapor_window.dew_pressure_pa, vapor.pressure_pa, 1e-6);
        assert_close(vapor_window.dew_temperature_k, vapor.temperature_k, 1e-4);
    }
}
